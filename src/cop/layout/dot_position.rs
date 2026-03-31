use crate::cop::node_type::CALL_NODE;
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-09)
///
/// Corpus oracle reported FP=2, FN=0.
///
/// FP=2: Fixed by skipping `::` scope resolution operators — only `.` and `&.` should be checked.
/// The 2 FPs were from rufo's spec file with `foo::\n bar` patterns.
pub struct DotPosition;

impl Cop for DotPosition {
    fn name(&self) -> &'static str {
        "Layout/DotPosition"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "leading");

        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        // Must have a dot (regular `.` or safe navigation `&.`)
        let dot_loc = match call.call_operator_loc() {
            Some(loc) => loc,
            None => return,
        };

        // Skip `::` scope resolution operators — only `.` and `&.` are relevant
        if dot_loc.as_slice() == b"::" {
            return;
        }

        // Must have a receiver
        let receiver = match call.receiver() {
            Some(r) => r,
            None => return,
        };

        // Must have a method name (message)
        let msg_loc = match call.message_loc() {
            Some(loc) => loc,
            None => return,
        };

        let (dot_line, dot_col) = source.offset_to_line_col(dot_loc.start_offset());
        let (recv_line, _) =
            source.offset_to_line_col(receiver.location().end_offset().saturating_sub(1));
        let (msg_line, _) = source.offset_to_line_col(msg_loc.start_offset());

        // Single line call — no issue
        if recv_line == msg_line {
            return;
        }

        // If there's a blank line between dot and selector, skip (could be reformatted oddly)
        if (msg_line as i64 - dot_line as i64).abs() > 1
            || (dot_line as i64 - recv_line as i64).abs() > 1
        {
            return;
        }

        let dot_text = String::from_utf8_lossy(dot_loc.as_slice()).into_owned();

        match style {
            "trailing" => {
                // Dot should be on the same line as the receiver (trailing)
                if dot_line != recv_line {
                    let mut diagnostic = self.diagnostic(
                        source,
                        dot_line,
                        dot_col,
                        format!(
                            "Place the `{}` on the previous line, together with the method call receiver.",
                            dot_text
                        ),
                    );
                    if let Some(corrections) = corrections.as_mut() {
                        corrections.push(Correction {
                            start: receiver.location().end_offset(),
                            end: receiver.location().end_offset(),
                            replacement: dot_text.clone(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        corrections.push(Correction {
                            start: dot_loc.start_offset(),
                            end: dot_loc.end_offset(),
                            replacement: String::new(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diagnostic.corrected = true;
                    }
                    diagnostics.push(diagnostic);
                }
            }
            _ => {
                // "leading" (default): dot should be on the same line as the method name
                if dot_line != msg_line {
                    let mut diagnostic = self.diagnostic(
                        source,
                        dot_line,
                        dot_col,
                        format!(
                            "Place the `{}` on the next line, together with the method name.",
                            dot_text
                        ),
                    );
                    if let Some(corrections) = corrections.as_mut() {
                        corrections.push(Correction {
                            start: msg_loc.start_offset(),
                            end: msg_loc.start_offset(),
                            replacement: dot_text.clone(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        corrections.push(Correction {
                            start: dot_loc.start_offset(),
                            end: dot_loc.end_offset(),
                            replacement: String::new(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diagnostic.corrected = true;
                    }
                    diagnostics.push(diagnostic);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(DotPosition, "cops/layout/dot_position");
    crate::cop_autocorrect_fixture_tests!(DotPosition, "cops/layout/dot_position");

    #[test]
    fn autocorrect_trailing_style_moves_dot_up() {
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([(
                "EnforcedStyle".into(),
                serde_yml::Value::String("trailing".into()),
            )]),
            ..CopConfig::default()
        };

        let source = b"foo\n  .bar\n";
        let (_diagnostics, corrections) =
            crate::testutil::run_cop_autocorrect_with_config(&DotPosition, source, config);
        let corrected = crate::correction::CorrectionSet::from_vec(corrections).apply(source);
        assert_eq!(corrected, b"foo.\n  bar\n");
    }
}
