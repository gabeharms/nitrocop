use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

pub struct EagerEvaluationLogMessage;

impl Cop for EagerEvaluationLogMessage {
    fn name(&self) -> &'static str {
        "Rails/EagerEvaluationLogMessage"
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::parse::codemap::CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut visitor = EagerEvalVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
            sole_block_stmt: false,
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

struct EagerEvalVisitor<'a> {
    cop: &'a EagerEvaluationLogMessage,
    source: &'a SourceFile,
    diagnostics: Vec<Diagnostic>,
    /// True when visiting the sole statement inside a block body.
    /// Matches RuboCop's `return if node.parent&.block_type?` — in Parser AST,
    /// a block with a single statement has the statement as a direct child of the
    /// block node (no `begin` wrapper), so `parent.block_type?` is true.
    sole_block_stmt: bool,
}

impl<'pr> Visit<'pr> for EagerEvalVisitor<'_> {
    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        // If the block body has exactly 1 statement, set the flag while visiting it.
        if let Some(body) = node.body() {
            if let Some(stmts) = body.as_statements_node() {
                let count = stmts.body().iter().count();
                if count == 1 {
                    let was = self.sole_block_stmt;
                    self.sole_block_stmt = true;
                    self.visit(&body);
                    self.sole_block_stmt = was;
                    return;
                }
            }
        }
        ruby_prism::visit_block_node(self, node);
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        self.check_debug_call(node);
        ruby_prism::visit_call_node(self, node);
    }
}

impl EagerEvalVisitor<'_> {
    fn check_debug_call(&mut self, call: &ruby_prism::CallNode<'_>) {
        if call.name().as_slice() != b"debug" {
            return;
        }

        // If already using a block, skip
        if call.block().is_some() {
            return;
        }

        // RuboCop: `return if node.parent&.block_type?` — skip when the debug call
        // is the sole statement in a block body.
        if self.sole_block_stmt {
            return;
        }

        // RuboCop's pattern matches `send` (not `csend`), so safe navigation
        // `Rails.logger&.debug(...)` is excluded.
        if let Some(op) = call.call_operator_loc() {
            if self.source.as_bytes()[op.start_offset()..op.end_offset()] == *b"&." {
                return;
            }
        }

        // Receiver must be Rails.logger (a 2-method chain)
        let receiver = match call.receiver() {
            Some(r) => r,
            None => return,
        };
        let inner_call = match receiver.as_call_node() {
            Some(c) => c,
            None => return,
        };
        if inner_call.name().as_slice() != b"logger" {
            return;
        }

        // Inner receiver must be `Rails` constant
        let inner_recv = match inner_call.receiver() {
            Some(r) => r,
            None => return,
        };

        let is_rails = if let Some(cr) = inner_recv.as_constant_read_node() {
            cr.name().as_slice() == b"Rails"
        } else if let Some(cp) = inner_recv.as_constant_path_node() {
            // ::Rails
            cp.parent().is_none() && cp.name().is_some_and(|n| n.as_slice() == b"Rails")
        } else {
            false
        };

        if !is_rails {
            return;
        }

        // First argument must be an interpolated string (dstr)
        let args = match call.arguments() {
            Some(a) => a,
            None => return,
        };
        let arg_list: Vec<_> = args.arguments().iter().collect();
        if arg_list.is_empty() {
            return;
        }

        if arg_list[0].as_interpolated_string_node().is_none() {
            return;
        }

        let loc = call.location();
        let (line, column) = self.source.offset_to_line_col(loc.start_offset());
        self.diagnostics.push(self.cop.diagnostic(
            self.source,
            line,
            column,
            "Pass a block to `Rails.logger.debug`.".to_string(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        EagerEvaluationLogMessage,
        "cops/rails/eager_evaluation_log_message"
    );
}
