use crate::cop::node_type::{INTERPOLATED_X_STRING_NODE, X_STRING_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CommandLiteral;

impl Cop for CommandLiteral {
    fn name(&self) -> &'static str {
        "Style/CommandLiteral"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[INTERPOLATED_X_STRING_NODE, X_STRING_NODE]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let enforced_style = config.get_str("EnforcedStyle", "backticks");
        let allow_inner_backticks = config.get_bool("AllowInnerBackticks", false);

        // Check both XStringNode and InterpolatedXStringNode
        let (opening, closing, node_loc, node_source) = if let Some(x) = node.as_x_string_node() {
            (
                x.opening_loc(),
                x.closing_loc(),
                x.location(),
                x.location().as_slice().to_vec(),
            )
        } else if let Some(x) = node.as_interpolated_x_string_node() {
            (
                x.opening_loc(),
                x.closing_loc(),
                x.location(),
                x.location().as_slice().to_vec(),
            )
        } else {
            return;
        };

        let opening_bytes = opening.as_slice();
        let is_backtick = opening_bytes == b"`";
        let is_multiline = node_source.iter().filter(|&&b| b == b'\n').count() > 1;

        // Check if inner content contains backticks
        let content_has_backticks = if is_backtick {
            // In backtick form, inner backticks are escaped: \`
            node_source.windows(2).any(|w| w == b"\\`")
        } else {
            // In %x form, literal backticks appear as-is
            let open_len = opening_bytes.len();
            let close_len = closing.as_slice().len();
            let inner = if node_source.len() > open_len + close_len {
                &node_source[open_len..node_source.len() - close_len]
            } else {
                &[]
            };
            inner.contains(&b'`')
        };

        let disallowed_backtick = !allow_inner_backticks && content_has_backticks;

        match enforced_style {
            "backticks" => {
                // Flag %x usage unless it contains backticks (and AllowInnerBackticks is false)
                if !is_backtick && !disallowed_backtick {
                    let (line, column) = source.offset_to_line_col(node_loc.start_offset());
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        column,
                        "Use backticks around command string.".to_string(),
                    );
                    if !content_has_backticks {
                        if let Some(ref mut corr) = corrections {
                            corr.push(crate::correction::Correction {
                                start: opening.start_offset(),
                                end: opening.end_offset(),
                                replacement: "`".to_string(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                            corr.push(crate::correction::Correction {
                                start: closing.start_offset(),
                                end: closing.end_offset(),
                                replacement: "`".to_string(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                            diag.corrected = true;
                        }
                    }
                    diagnostics.push(diag);
                }
            }
            "percent_x" => {
                // Flag backtick usage
                if is_backtick {
                    let (line, column) = source.offset_to_line_col(node_loc.start_offset());
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use `%x` around command string.".to_string(),
                    ));
                }
            }
            "mixed" => {
                if is_backtick && (is_multiline || disallowed_backtick) {
                    let (line, column) = source.offset_to_line_col(node_loc.start_offset());
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use `%x` around command string.".to_string(),
                    ));
                } else if !is_backtick && !is_multiline && !disallowed_backtick {
                    let (line, column) = source.offset_to_line_col(node_loc.start_offset());
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        column,
                        "Use backticks around command string.".to_string(),
                    );
                    if !content_has_backticks {
                        if let Some(ref mut corr) = corrections {
                            corr.push(crate::correction::Correction {
                                start: opening.start_offset(),
                                end: opening.end_offset(),
                                replacement: "`".to_string(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                            corr.push(crate::correction::Correction {
                                start: closing.start_offset(),
                                end: closing.end_offset(),
                                replacement: "`".to_string(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                            diag.corrected = true;
                        }
                    }
                    diagnostics.push(diag);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(CommandLiteral, "cops/style/command_literal");
    crate::cop_autocorrect_fixture_tests!(CommandLiteral, "cops/style/command_literal");
}
