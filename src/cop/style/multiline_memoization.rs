use crate::cop::node_type::{
    BEGIN_NODE, CALL_OR_WRITE_NODE, CLASS_VARIABLE_OR_WRITE_NODE, CONSTANT_OR_WRITE_NODE,
    GLOBAL_VARIABLE_OR_WRITE_NODE, INDEX_OR_WRITE_NODE, INSTANCE_VARIABLE_OR_WRITE_NODE,
    LOCAL_VARIABLE_OR_WRITE_NODE, PARENTHESES_NODE,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Checks multiline memoization wrapping style (`||=`).
///
/// ## Investigation (2026-03-23)
///
/// **FP root cause:** The multiline check compared the assignment start line
/// against the value end line. For `@x ||=\n  (single_expr if cond)`, the
/// assignment starts on line 1 and the value ends on line 2, so it was
/// incorrectly treated as multiline. RuboCop checks whether the *RHS node
/// itself* spans multiple lines (`rhs.multiline?`), not whether the overall
/// assignment does. Fixed by checking the value node's own start/end lines.
///
/// **FN root cause:** The cop only handled simple variable `||=` nodes
/// (`LocalVariableOrWriteNode`, `InstanceVariableOrWriteNode`, etc.) but
/// missed `CallOrWriteNode` (`foo.bar ||=`) and `IndexOrWriteNode`
/// (`foo["key"] ||=`). These are common in real-world code (e.g.,
/// `@info["exif"] ||= (...)`). Fixed by adding both node types.
pub struct MultilineMemoization;

impl Cop for MultilineMemoization {
    fn name(&self) -> &'static str {
        "Style/MultilineMemoization"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            BEGIN_NODE,
            CALL_OR_WRITE_NODE,
            CLASS_VARIABLE_OR_WRITE_NODE,
            CONSTANT_OR_WRITE_NODE,
            GLOBAL_VARIABLE_OR_WRITE_NODE,
            INDEX_OR_WRITE_NODE,
            INSTANCE_VARIABLE_OR_WRITE_NODE,
            LOCAL_VARIABLE_OR_WRITE_NODE,
            PARENTHESES_NODE,
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let enforced_style = config.get_str("EnforcedStyle", "keyword");

        // Extract (assignment location, value) from any kind of ||= node
        let (assign_loc, value) = if let Some(n) = node.as_local_variable_or_write_node() {
            (n.location(), n.value())
        } else if let Some(n) = node.as_instance_variable_or_write_node() {
            (n.location(), n.value())
        } else if let Some(n) = node.as_class_variable_or_write_node() {
            (n.location(), n.value())
        } else if let Some(n) = node.as_global_variable_or_write_node() {
            (n.location(), n.value())
        } else if let Some(n) = node.as_constant_or_write_node() {
            (n.location(), n.value())
        } else if let Some(n) = node.as_call_or_write_node() {
            (n.location(), n.value())
        } else if let Some(n) = node.as_index_or_write_node() {
            (n.location(), n.value())
        } else {
            return;
        };

        // Check if the VALUE NODE ITSELF spans multiple lines.
        // RuboCop uses `rhs.multiline?` which checks the RHS node's own span.
        // This avoids false positives where the assignment operator is on a
        // different line than the value but the value itself is single-line.
        let value_loc = value.location();
        let value_start_line = source.offset_to_line_col(value_loc.start_offset()).0;
        let value_end_offset = value_loc.start_offset() + value_loc.as_slice().len();
        let value_end_line = source
            .offset_to_line_col(value_end_offset.saturating_sub(1))
            .0;

        if value_start_line == value_end_line {
            // Value is single-line — not a multiline memoization
            return;
        }

        // It's multiline. Check the wrapping style.
        if enforced_style == "keyword" {
            // keyword style: should use begin..end, not parentheses
            if let Some(paren) = value.as_parentheses_node() {
                let (line, column) = source.offset_to_line_col(assign_loc.start_offset());
                let mut diagnostic = self.diagnostic(
                    source,
                    line,
                    column,
                    "Wrap multiline memoization blocks in `begin` and `end`.".to_string(),
                );

                if let Some(corrections) = corrections {
                    let open_line = source
                        .offset_to_line_col(paren.opening_loc().start_offset())
                        .0;
                    let close_line = source
                        .offset_to_line_col(paren.closing_loc().start_offset())
                        .0;
                    let body_line_range = paren.body().map(|body| {
                        (
                            source.offset_to_line_col(body.location().start_offset()).0,
                            source
                                .offset_to_line_col(body.location().end_offset().saturating_sub(1))
                                .0,
                        )
                    });

                    if let Some((body_start_line, body_end_line)) = body_line_range {
                        if open_line < body_start_line && close_line > body_end_line {
                            corrections.push(crate::correction::Correction {
                                start: paren.opening_loc().start_offset(),
                                end: paren.opening_loc().end_offset(),
                                replacement: "begin".to_string(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                            corrections.push(crate::correction::Correction {
                                start: paren.closing_loc().start_offset(),
                                end: paren.closing_loc().end_offset(),
                                replacement: "end".to_string(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                            diagnostic.corrected = true;
                        }
                    }
                }

                diagnostics.push(diagnostic);
            }
        } else if enforced_style == "braces" {
            // braces style: should use parentheses, not begin..end
            if let Some(begin) = value.as_begin_node() {
                let (line, column) = source.offset_to_line_col(assign_loc.start_offset());
                let mut diagnostic = self.diagnostic(
                    source,
                    line,
                    column,
                    "Wrap multiline memoization blocks in `(` and `)`.".to_string(),
                );

                if let Some(corrections) = corrections {
                    if let (Some(begin_kw), Some(end_kw)) =
                        (begin.begin_keyword_loc(), begin.end_keyword_loc())
                    {
                        corrections.push(crate::correction::Correction {
                            start: begin_kw.start_offset(),
                            end: begin_kw.end_offset(),
                            replacement: "(".to_string(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        corrections.push(crate::correction::Correction {
                            start: end_kw.start_offset(),
                            end: end_kw.end_offset(),
                            replacement: ")".to_string(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diagnostic.corrected = true;
                    }
                }

                diagnostics.push(diagnostic);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(MultilineMemoization, "cops/style/multiline_memoization");
    crate::cop_autocorrect_fixture_tests!(MultilineMemoization, "cops/style/multiline_memoization");
}
