use crate::cop::node_type::{INTERPOLATED_REGULAR_EXPRESSION_NODE, REGULAR_EXPRESSION_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Check if a `%r` regexp literal appears to be a method argument (with or without parens).
/// RuboCop exempts `%r` with space/eq-starting content when `node.parent&.call_type?`.
/// We approximate by scanning backwards from the node start: if the preceding
/// non-whitespace token is an identifier character (bare arg) or `(` or `,` (paren arg),
/// it's likely a method call argument. If preceded by `=` (assignment), newline/start-of-file
/// (standalone), `[`, `|`, etc., it's not a method argument.
fn is_method_call_argument(source: &[u8], node_start: usize) -> bool {
    let mut i = node_start;
    while i > 0 {
        i -= 1;
        match source[i] {
            b' ' | b'\t' => continue,
            // Identifier-like chars: method name immediately before the %r (bare arg)
            c if c.is_ascii_alphanumeric() || c == b'_' || c == b'?' || c == b'!' => {
                return true;
            }
            // Opening paren or comma: method call with parens
            b'(' | b',' => return true,
            // Anything else (=, [, newline, etc.) means not a method arg
            _ => return false,
        }
    }
    false
}

/// FP fix (2026-03): slashes inside `#{}` interpolation segments were wrongly
/// counted as inner slashes, causing false "Use %r" suggestions on regexps like
/// `/#{Regexp.quote("</")}/ `. RuboCop's `node_body` only examines `:str` children,
/// so interpolation content is excluded. Fixed by iterating over Prism's `parts()`
/// and only collecting `StringNode` content for the slash check.
///
/// FN fix (2026-03): The space/eq content exemption was applied unconditionally,
/// but RuboCop only exempts `%r` with space/eq-starting content when the regexp
/// is a direct argument to a method call without parentheses (`node.parent&.call_type?`).
/// Fixed by scanning bytes before the node to detect bare method argument context.
pub struct RegexpLiteral;

impl Cop for RegexpLiteral {
    fn name(&self) -> &'static str {
        "Style/RegexpLiteral"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            INTERPOLATED_REGULAR_EXPRESSION_NODE,
            REGULAR_EXPRESSION_NODE,
        ]
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
        let enforced_style = config.get_str("EnforcedStyle", "slashes");
        let allow_inner_slashes = config.get_bool("AllowInnerSlashes", false);

        let (open_bytes, content_bytes, node_start, node_end): (Vec<u8>, Vec<u8>, usize, usize) =
            if let Some(re) = node.as_regular_expression_node() {
                let opening = re.opening_loc();
                let content = re.content_loc().as_slice();
                let loc = re.location();
                (
                    opening.as_slice().to_vec(),
                    content.to_vec(),
                    loc.start_offset(),
                    loc.end_offset(),
                )
            } else if let Some(re) = node.as_interpolated_regular_expression_node() {
                let opening = re.opening_loc();
                let loc = re.location();
                let open = opening.as_slice();
                // Only collect content from string literal parts, skipping interpolation.
                // RuboCop's `node_body` only examines `:str` children, so slashes
                // inside `#{}` interpolation are not counted as inner slashes.
                let mut content = Vec::new();
                for part in re.parts().iter() {
                    if let Some(s) = part.as_string_node() {
                        content.extend_from_slice(s.location().as_slice());
                    }
                }
                (open.to_vec(), content, loc.start_offset(), loc.end_offset())
            } else {
                return;
            };

        let is_slash = open_bytes == b"/";
        let is_percent_r = open_bytes.starts_with(b"%r");

        // Check if content contains forward slashes (escaped or unescaped).
        // RuboCop counts escaped slashes (`\/`) as inner slashes too, because
        // using `%r{}` would eliminate the need for escaping them.
        // In slash-delimited regexps, slashes are always escaped as `\/`.
        // In %r-delimited regexps, slashes appear as bare `/`.
        let has_slash = content_bytes.contains(&b'/');

        let is_multiline = {
            let (start_line, _) = source.offset_to_line_col(node_start);
            let (end_line, _) = source.offset_to_line_col(node_end);
            end_line > start_line
        };

        // %r with content starting with space or = may be used to avoid syntax errors
        // when the regexp is a method argument without parentheses:
        //   do_something %r{ regexp}  # valid
        //   do_something / regexp/    # syntax error
        // RuboCop only exempts this when node.parent is a call_type (method call).
        // We approximate by checking the source context before the %r node.
        let content_starts_with_space_or_eq = !content_bytes.is_empty()
            && (content_bytes[0] == b' ' || content_bytes[0] == b'=')
            && is_method_call_argument(source.as_bytes(), node_start);

        match enforced_style {
            "slashes" => {
                if is_percent_r {
                    if has_slash && !allow_inner_slashes {
                        return;
                    }
                    if content_starts_with_space_or_eq {
                        return;
                    }
                    let (line, column) = source.offset_to_line_col(node_start);
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use `//` around regular expression.".to_string(),
                    ));
                }
            }
            "percent_r" => {
                if is_slash {
                    let (line, column) = source.offset_to_line_col(node_start);
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use `%r` around regular expression.".to_string(),
                    ));
                }
            }
            "mixed" => {
                if is_multiline {
                    if is_slash {
                        let (line, column) = source.offset_to_line_col(node_start);
                        diagnostics.push(self.diagnostic(
                            source,
                            line,
                            column,
                            "Use `%r` around regular expression.".to_string(),
                        ));
                    }
                } else if is_percent_r {
                    if has_slash && !allow_inner_slashes {
                        return;
                    }
                    if content_starts_with_space_or_eq {
                        return;
                    }
                    let (line, column) = source.offset_to_line_col(node_start);
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use `//` around regular expression.".to_string(),
                    ));
                }
            }
            _ => {}
        }

        // For slashes style: check for inner slashes
        if enforced_style == "slashes" && is_slash && has_slash && !allow_inner_slashes {
            let (line, column) = source.offset_to_line_col(node_start);
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                "Use `%r` around regular expression.".to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RegexpLiteral, "cops/style/regexp_literal");
}
