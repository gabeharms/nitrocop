use ruby_prism::Visit;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Checks if `include` or `prepend` is called in a `refine` block.
/// These methods are deprecated and should be replaced with `import_methods`.
///
/// ## Corpus investigation (2026-03-08)
///
/// Corpus oracle reported FP=1, FN=0.
///
/// FP=1: the remaining mismatch is an excluded `ruby-next` fixture under
/// `spec/core/**/*`, so this is config/path-filtering noise rather than a cop-logic
/// mismatch. Earlier fixes already aligned the cop's pending default-disabled state
/// and restricted matches to direct `include`/`prepend` children of a `refine` block.
/// FN=0: no missing detections were reported for this cop in the corpus run.
pub struct RefinementImportMethods;

impl Cop for RefinementImportMethods {
    fn name(&self) -> &'static str {
        "Lint/RefinementImportMethods"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn default_enabled(&self) -> bool {
        false
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
        let mut visitor = RefineVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

struct RefineVisitor<'a, 'src> {
    cop: &'a RefinementImportMethods,
    source: &'src SourceFile,
    diagnostics: Vec<Diagnostic>,
}

impl<'pr> Visit<'pr> for RefineVisitor<'_, '_> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        let method_name = node.name().as_slice();

        // Check if this is a `refine` call with a block
        if method_name == b"refine" && node.receiver().is_none() {
            if let Some(block) = node.block() {
                // Check direct children of the block body for include/prepend
                if let Some(block_node) = block.as_block_node() {
                    if let Some(body) = block_node.body() {
                        self.check_refine_body(&body);
                    }
                }
            }
        }

        // Continue visiting children for nested refine blocks
        ruby_prism::visit_call_node(self, node);
    }
}

impl RefineVisitor<'_, '_> {
    fn check_refine_body(&mut self, body: &ruby_prism::Node<'_>) {
        // Body is typically a StatementsNode containing the block's statements
        if let Some(stmts) = body.as_statements_node() {
            for stmt in stmts.body().iter() {
                if let Some(call) = stmt.as_call_node() {
                    let name = call.name().as_slice();
                    if (name == b"include" || name == b"prepend") && call.receiver().is_none() {
                        let msg_loc = call.message_loc().unwrap_or(call.location());
                        let (line, column) = self.source.offset_to_line_col(msg_loc.start_offset());
                        let method_str = if name == b"include" {
                            "include"
                        } else {
                            "prepend"
                        };
                        self.diagnostics.push(self.cop.diagnostic(
                            self.source,
                            line,
                            column,
                            format!(
                                "Use `import_methods` instead of `{}` because it is deprecated in Ruby 3.1.",
                                method_str
                            ),
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        RefinementImportMethods,
        "cops/lint/refinement_import_methods"
    );
}
