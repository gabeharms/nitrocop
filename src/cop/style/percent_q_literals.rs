use crate::cop::node_type::{INTERPOLATED_STRING_NODE, STRING_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct PercentQLiterals;

impl Cop for PercentQLiterals {
    fn name(&self) -> &'static str {
        "Style/PercentQLiterals"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[INTERPOLATED_STRING_NODE, STRING_NODE]
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
        let style = config.get_str("EnforcedStyle", "lower_case_q");

        // Check for %Q or %q string nodes using the opening_loc, which
        // reliably identifies percent literals vs regular string content.
        let opening_loc = if let Some(s) = node.as_string_node() {
            s.opening_loc()
        } else if let Some(s) = node.as_interpolated_string_node() {
            s.opening_loc()
        } else {
            None
        };

        let opening = match opening_loc {
            Some(loc) => loc,
            None => return,
        };

        let opening_bytes = opening.as_slice();

        if style == "lower_case_q" {
            // Flag %Q when %q would suffice (no interpolation, no escape sequences)
            if opening_bytes.starts_with(b"%Q") {
                if let Some(s) = node.as_string_node() {
                    // StringNode means no interpolation.
                    let raw_content = s.content_loc().as_slice();
                    // Skip if content contains backslashes — converting %Q to %q
                    // would change escape sequence interpretation (e.g. \t, \n, \\).
                    if raw_content.contains(&b'\\') {
                        return;
                    }
                    // Skip multiline strings. The Parser gem (used by RuboCop) treats
                    // multiline percent literals as `dstr` nodes, not `str`, so RuboCop's
                    // `on_str` handler never sees them. Match that behavior.
                    if raw_content.contains(&b'\n') {
                        return;
                    }
                    let loc = node.location();
                    let (line, column) = source.offset_to_line_col(loc.start_offset());
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        column,
                        "Use `%q` instead of `%Q`.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: opening.start_offset(),
                            end: opening.start_offset() + 2,
                            replacement: "%q".to_string(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }
            }
        } else if style == "upper_case_q" {
            // Flag %q when %Q is preferred
            if opening_bytes.starts_with(b"%q") {
                if let Some(s) = node.as_string_node() {
                    let raw_content = s.content_loc().as_slice();
                    // Skip if content contains backslashes — converting %q to %Q
                    // would change escape sequence interpretation or cause parse errors.
                    if raw_content.contains(&b'\\') {
                        return;
                    }
                    // Skip multiline strings (Parser gem treats these as dstr, not str).
                    if raw_content.contains(&b'\n') {
                        return;
                    }
                }
                let loc = node.location();
                let (line, column) = source.offset_to_line_col(loc.start_offset());
                let mut diag = self.diagnostic(
                    source,
                    line,
                    column,
                    "Use `%Q` instead of `%q`.".to_string(),
                );
                if let Some(ref mut corr) = corrections {
                    corr.push(crate::correction::Correction {
                        start: opening.start_offset(),
                        end: opening.start_offset() + 2,
                        replacement: "%Q".to_string(),
                        cop_name: self.name(),
                        cop_index: 0,
                    });
                    diag.corrected = true;
                }
                diagnostics.push(diag);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(PercentQLiterals, "cops/style/percent_q_literals");
    crate::cop_autocorrect_fixture_tests!(PercentQLiterals, "cops/style/percent_q_literals");
}
