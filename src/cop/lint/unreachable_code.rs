use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

/// ## Investigation (2026-03-03)
///
/// Found 1 FP: FactoryBot `fail { false }` treated as `Kernel#fail`
/// (flow-breaking). `Kernel#fail` never accepts blocks, so any `fail`/`raise`
/// with a block is a DSL method call. Fixed by adding `call.block().is_none()`
/// check to `is_raise_call` (dc856393).
pub struct UnreachableCode;

impl Cop for UnreachableCode {
    fn name(&self) -> &'static str {
        "Lint/UnreachableCode"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
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
        let mut visitor = UnreachableVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

struct UnreachableVisitor<'a, 'src> {
    cop: &'a UnreachableCode,
    source: &'src SourceFile,
    diagnostics: Vec<Diagnostic>,
}

fn is_flow_breaking(node: &ruby_prism::Node<'_>) -> bool {
    node.as_return_node().is_some()
        || node.as_break_node().is_some()
        || node.as_next_node().is_some()
        || node.as_retry_node().is_some()
        || is_raise_call(node)
}

fn is_raise_call(node: &ruby_prism::Node<'_>) -> bool {
    if let Some(call) = node.as_call_node() {
        let name = call.name().as_slice();
        // Kernel#raise/fail never accept blocks — a block means this is a DSL
        // method call (e.g. FactoryBot `fail { false }`), not flow-breaking.
        if (name == b"raise" || name == b"fail")
            && call.receiver().is_none()
            && call.block().is_none()
        {
            return true;
        }
    }
    false
}

impl<'pr> Visit<'pr> for UnreachableVisitor<'_, '_> {
    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode<'pr>) {
        let body: Vec<_> = node.body().iter().collect();
        let mut flow_broken = false;

        for stmt in &body {
            if flow_broken {
                let loc = stmt.location();
                let (line, column) = self.source.offset_to_line_col(loc.start_offset());
                self.diagnostics.push(self.cop.diagnostic(
                    self.source,
                    line,
                    column,
                    "Unreachable code detected.".to_string(),
                ));
                break; // Only flag the first unreachable statement
            }
            if is_flow_breaking(stmt) {
                flow_broken = true;
            }
        }

        ruby_prism::visit_statements_node(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(UnreachableCode, "cops/lint/unreachable_code");
}
