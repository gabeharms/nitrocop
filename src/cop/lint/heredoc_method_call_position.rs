use ruby_prism::Visit;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Checks that method calls on HEREDOC receivers are on the same line as the opening.
pub struct HeredocMethodCallPosition;

impl Cop for HeredocMethodCallPosition {
    fn name(&self) -> &'static str {
        "Lint/HeredocMethodCallPosition"
    }

    fn default_enabled(&self) -> bool {
        false
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::parse::codemap::CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut visitor = HeredocVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
            corrections,
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

struct HeredocVisitor<'a, 'src> {
    cop: &'a HeredocMethodCallPosition,
    source: &'src SourceFile,
    diagnostics: Vec<Diagnostic>,
    corrections: Option<&'a mut Vec<crate::correction::Correction>>,
}

impl<'pr> Visit<'pr> for HeredocVisitor<'_, '_> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if let Some(recv) = node.receiver() {
            if is_heredoc(&recv) {
                let heredoc_opening_line = self
                    .source
                    .offset_to_line_col(recv.location().start_offset())
                    .0;

                if let Some(msg_loc) = node.message_loc() {
                    let method_line = self.source.offset_to_line_col(msg_loc.start_offset()).0;

                    if method_line != heredoc_opening_line {
                        let (line, column) = self.source.offset_to_line_col(msg_loc.start_offset());
                        let mut diag = self.cop.diagnostic(
                            self.source,
                            line,
                            column,
                            "Put a method call with a HEREDOC receiver on the same line as the HEREDOC opening.".to_string(),
                        );

                        if let Some(corrections) = self.corrections.as_deref_mut() {
                            if let Some((insert_at, suffix, remove_start, remove_end)) =
                                autocorrect_parts(self.source, &recv, node, method_line)
                            {
                                corrections.push(crate::correction::Correction {
                                    start: insert_at,
                                    end: insert_at,
                                    replacement: suffix,
                                    cop_name: self.cop.name(),
                                    cop_index: 0,
                                });
                                corrections.push(crate::correction::Correction {
                                    start: remove_start,
                                    end: remove_end,
                                    replacement: String::new(),
                                    cop_name: self.cop.name(),
                                    cop_index: 0,
                                });
                                diag.corrected = true;
                            }
                        }

                        self.diagnostics.push(diag);
                    }
                }
            }
        }

        ruby_prism::visit_call_node(self, node);
    }
}

fn is_heredoc(node: &ruby_prism::Node<'_>) -> bool {
    heredoc_opening_end(node).is_some()
}

fn heredoc_opening_end(node: &ruby_prism::Node<'_>) -> Option<usize> {
    if let Some(str_node) = node.as_interpolated_string_node() {
        if let Some(open) = str_node.opening_loc() {
            let open_bytes = open.as_slice();
            if open_bytes.starts_with(b"<<") {
                return Some(open.end_offset());
            }
        }
    }
    if let Some(str_node) = node.as_string_node() {
        if let Some(open) = str_node.opening_loc() {
            let open_bytes = open.as_slice();
            if open_bytes.starts_with(b"<<") {
                return Some(open.end_offset());
            }
        }
    }
    None
}

fn autocorrect_parts(
    source: &SourceFile,
    recv: &ruby_prism::Node<'_>,
    call: &ruby_prism::CallNode<'_>,
    method_line: usize,
) -> Option<(usize, String, usize, usize)> {
    if call.arguments().is_some() || call.block().is_some() {
        return None;
    }

    let op_loc = call.call_operator_loc()?;
    let insert_at = heredoc_opening_end(recv)?;
    let suffix_start = op_loc.start_offset();
    let suffix_end = call.location().end_offset();
    let suffix = std::str::from_utf8(source.try_byte_slice(suffix_start, suffix_end)?.as_bytes())
        .ok()?
        .to_string();

    let line_start = source.line_col_to_offset(method_line, 0)?;
    let line_end = source
        .line_col_to_offset(method_line + 1, 0)
        .unwrap_or(source.as_bytes().len());
    let line_text = source.try_byte_slice(line_start, line_end)?;
    if line_text.trim() != suffix {
        return None;
    }

    Some((insert_at, suffix, line_start, line_end))
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        HeredocMethodCallPosition,
        "cops/lint/heredoc_method_call_position"
    );
    crate::cop_autocorrect_fixture_tests!(
        HeredocMethodCallPosition,
        "cops/lint/heredoc_method_call_position"
    );
}
