use crate::cop::node_type::{INTERPOLATED_X_STRING_NODE, X_STRING_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Style/CommandLiteral — enforces backtick vs %x for command literals.
///
/// Corpus investigation (FP=0, FN=7, all in opal): The 7 FNs were backtick
/// literals containing escaped inner backticks (e.g. `` `echo \`ls\`` ``).
/// In `backticks` mode, RuboCop flags these with "Use `%x` around command
/// string." because inner backticks make the backtick form hard to read.
/// nitrocop was missing this case — it only flagged `%x` usage in `backticks`
/// mode but didn't flag backtick literals with inner backticks.
pub struct CommandLiteral;

impl Cop for CommandLiteral {
    fn name(&self) -> &'static str {
        "Style/CommandLiteral"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[INTERPOLATED_X_STRING_NODE, X_STRING_NODE]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let enforced_style = config.get_str("EnforcedStyle", "backticks");
        let allow_inner_backticks = config.get_bool("AllowInnerBackticks", false);

        // Check both XStringNode and InterpolatedXStringNode
        let (opening_loc, node_loc, node_source) = if let Some(x) = node.as_x_string_node() {
            (
                Some(x.opening_loc()),
                x.location(),
                x.location().as_slice().to_vec(),
            )
        } else if let Some(x) = node.as_interpolated_x_string_node() {
            (
                Some(x.opening_loc()),
                x.location(),
                x.location().as_slice().to_vec(),
            )
        } else {
            return;
        };

        let opening = match opening_loc {
            Some(loc) => loc,
            None => return,
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
            let inner = if node_source.len() > open_len + 1 {
                &node_source[open_len..node_source.len() - 1]
            } else {
                &[]
            };
            inner.contains(&b'`')
        };

        let disallowed_backtick = !allow_inner_backticks && content_has_backticks;

        match enforced_style {
            "backticks" => {
                if is_backtick && disallowed_backtick {
                    // Backtick literal contains inner backticks — suggest %x instead
                    let (line, column) = source.offset_to_line_col(node_loc.start_offset());
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use `%x` around command string.".to_string(),
                    ));
                } else if !is_backtick && !disallowed_backtick {
                    // %x literal without inner backticks — suggest backticks
                    let (line, column) = source.offset_to_line_col(node_loc.start_offset());
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use backticks around command string.".to_string(),
                    ));
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
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use backticks around command string.".to_string(),
                    ));
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

    fn config_with_style(style: &str) -> crate::cop::CopConfig {
        let mut config = crate::cop::CopConfig::default();
        config.options.insert(
            "EnforcedStyle".to_string(),
            serde_yml::Value::String(style.to_string()),
        );
        config
    }

    #[test]
    fn percent_x_style_flags_backticks() {
        let source = b"x = `simple command`\ny = `cmd with #{expr}`\n";
        let config = config_with_style("percent_x");
        let diags = crate::testutil::run_cop_full_with_config(&CommandLiteral, source, config);
        assert_eq!(diags.len(), 2, "Expected 2 offenses but got {:?}", diags);
    }

    #[test]
    fn percent_x_style_flags_nested_backticks() {
        let source = b"z = `outer #{`inner`}`\n";
        let config = config_with_style("percent_x");
        let diags = crate::testutil::run_cop_full_with_config(&CommandLiteral, source, config);
        // Both outer and inner backtick xstrings should be flagged
        assert_eq!(diags.len(), 2, "Expected 2 offenses but got {:?}", diags);
    }

    #[test]
    fn percent_x_style_flags_multiline_backticks() {
        let source = b"w = `\n  multiline\n  command\n`\n";
        let config = config_with_style("percent_x");
        let diags = crate::testutil::run_cop_full_with_config(&CommandLiteral, source, config);
        assert_eq!(diags.len(), 1, "Expected 1 offense but got {:?}", diags);
    }

    #[test]
    fn backticks_style_flags_inner_backticks() {
        // In default backticks mode, backtick literals with escaped inner backticks
        // should be flagged with "Use %x" since backticks can't cleanly represent them
        let source = b"x = `echo \\`ls\\``\n";
        let config = config_with_style("backticks");
        let diags = crate::testutil::run_cop_full_with_config(&CommandLiteral, source, config);
        assert_eq!(diags.len(), 1, "Expected 1 offense but got {:?}", diags);
        assert!(
            diags[0].message.contains("%x"),
            "Expected '%x' in message but got: {}",
            diags[0].message
        );
    }

    #[test]
    fn backticks_style_allows_percent_x_with_inner_backticks() {
        // In backticks mode, %x literals with inner backticks are allowed
        let source = b"x = %x(echo `ls`)\n";
        let config = config_with_style("backticks");
        let diags = crate::testutil::run_cop_full_with_config(&CommandLiteral, source, config);
        assert_eq!(diags.len(), 0, "Expected 0 offenses but got {:?}", diags);
    }
}
