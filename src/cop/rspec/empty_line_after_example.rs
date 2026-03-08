use crate::cop::node_type::CALL_NODE;
use crate::cop::util::{
    RSPEC_DEFAULT_INCLUDE, RSPEC_EXAMPLES, is_blank_or_whitespace_line, is_rspec_example, line_at,
    node_on_single_line,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-08)
///
/// Corpus oracle reported FP=1,547, FN=2.
///
/// FP root cause (this pass): separator lines containing only spaces/tabs were
/// treated as non-blank by `is_blank_line`, so examples followed by whitespace-only
/// lines were flagged. RuboCop uses `blank?` semantics here.
///
/// Historical FP root cause (already fixed): heredoc content extending past the
/// example call location. We account for this using heredoc closing offsets.
///
/// FN=2: no code changes here were aimed at FN behavior in this pass.
///
/// Fix: use whitespace-aware blank-line checks for separator detection in this cop.
pub struct EmptyLineAfterExample;

impl Cop for EmptyLineAfterExample {
    fn name(&self) -> &'static str {
        "RSpec/EmptyLineAfterExample"
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_DEFAULT_INCLUDE
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
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        let method_name = call.name().as_slice();
        if call.receiver().is_some() || !is_rspec_example(method_name) {
            return;
        }

        // RuboCop's EmptyLineAfterExample uses `on_block` — it only fires on example
        // calls that have a block (do..end or { }).  Bare calls like `skip('reason')`
        // inside a `before` block, or `scenario` used as a variable-like method from
        // `let(:scenario)`, are not example declarations and must be ignored.
        if call.block().is_none() {
            return;
        }

        let allow_consecutive = config.get_bool("AllowConsecutiveOneLiners", true);

        // Determine the end line of this example, accounting for heredocs
        // whose content extends past the node's own location.
        let loc = node.location();
        let mut max_end_offset = loc.end_offset();
        let heredoc_max = find_max_heredoc_end_offset(source, node);
        if heredoc_max > max_end_offset {
            max_end_offset = heredoc_max;
        }
        let end_offset = max_end_offset.saturating_sub(1).max(loc.start_offset());
        let (end_line, _) = source.offset_to_line_col(end_offset);

        let is_one_liner = node_on_single_line(source, &loc);

        // Check if the next non-blank line is another node
        let next_line = end_line + 1;
        let next_content = line_at(source, next_line);
        match next_content {
            Some(line) => {
                if is_blank_or_whitespace_line(line) {
                    return; // already has blank line
                }

                // Determine the effective "check line" — skip past comments to find
                // the first non-comment, non-blank line.  If a blank line or EOF is
                // encountered while scanning comments, the example is properly
                // separated and we return early.
                let check_line = if is_comment_line(line) {
                    let mut scan = next_line + 1;
                    loop {
                        match line_at(source, scan) {
                            Some(l) if is_blank_or_whitespace_line(l) => return,
                            Some(l) if is_comment_line(l) => {}
                            Some(l) => break l,
                            None => return, // end of file
                        }
                        scan += 1;
                    }
                } else {
                    line
                };

                // If consecutive one-liners are allowed, check if the next
                // meaningful line is also a one-liner example.
                // Both the current AND next example must be one-liners.
                if allow_consecutive && is_one_liner {
                    let trimmed = check_line.iter().position(|&b| b != b' ' && b != b'\t');
                    if let Some(start) = trimmed {
                        let rest = &check_line[start..];
                        if starts_with_example_keyword(rest) && is_single_line_block(rest) {
                            return;
                        }
                    }
                }

                // Check for terminator keywords (last example before closing
                // construct).  RuboCop uses `last_child?` on the AST; we
                // approximate by recognising `end`, `else`, `elsif`, `when`,
                // `rescue`, `ensure`, and `in` (pattern matching).
                if is_terminator_line(check_line) {
                    return;
                }
            }
            None => return, // end of file
        }

        // Report on the end line of the example
        let method_str = std::str::from_utf8(method_name).unwrap_or("it");
        let report_col_actual = if is_one_liner {
            let (_, start_col) = source.offset_to_line_col(loc.start_offset());
            start_col
        } else {
            // For multi-line, report at the `end` keyword column
            if let Some(line_bytes) = line_at(source, end_line) {
                line_bytes.iter().take_while(|&&b| b == b' ').count()
            } else {
                0
            }
        };

        diagnostics.push(self.diagnostic(
            source,
            end_line,
            report_col_actual,
            format!("Add an empty line after `{method_str}`."),
        ));
    }
}

/// Walk descendants of `node` to find the maximum `closing_loc().end_offset()`
/// among heredoc StringNode/InterpolatedStringNode children. Heredocs in Prism
/// have their `location()` covering only the opening delimiter (`<<-OUT`), but
/// `closing_loc()` covers the terminator line. Returns 0 if no heredocs found.
fn find_max_heredoc_end_offset(source: &SourceFile, node: &ruby_prism::Node<'_>) -> usize {
    use ruby_prism::Visit;

    struct MaxHeredocVisitor<'a> {
        source: &'a SourceFile,
        max_offset: usize,
    }

    impl<'pr> Visit<'pr> for MaxHeredocVisitor<'_> {
        fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
            if let Some(opening) = node.opening_loc() {
                let bytes = &self.source.as_bytes()[opening.start_offset()..opening.end_offset()];
                if bytes.starts_with(b"<<") {
                    if let Some(closing) = node.closing_loc() {
                        self.max_offset = self.max_offset.max(closing.end_offset());
                    }
                    return;
                }
            }
            ruby_prism::visit_string_node(self, node);
        }

        fn visit_interpolated_string_node(
            &mut self,
            node: &ruby_prism::InterpolatedStringNode<'pr>,
        ) {
            if let Some(opening) = node.opening_loc() {
                let bytes = &self.source.as_bytes()[opening.start_offset()..opening.end_offset()];
                if bytes.starts_with(b"<<") {
                    if let Some(closing) = node.closing_loc() {
                        self.max_offset = self.max_offset.max(closing.end_offset());
                    }
                    return;
                }
            }
            ruby_prism::visit_interpolated_string_node(self, node);
        }
    }

    let mut visitor = MaxHeredocVisitor {
        source,
        max_offset: 0,
    };
    visitor.visit(node);
    visitor.max_offset
}

/// Returns true if the trimmed line starts with `#`.
fn is_comment_line(line: &[u8]) -> bool {
    let trimmed_pos = line.iter().position(|&b| b != b' ' && b != b'\t');
    matches!(trimmed_pos, Some(start) if line[start] == b'#')
}

/// Check if a line is a block/construct terminator — i.e. the example is
/// the last child before the closing keyword.
fn is_terminator_line(line: &[u8]) -> bool {
    let trimmed = line.iter().position(|&b| b != b' ' && b != b'\t');
    if let Some(start) = trimmed {
        let rest = &line[start..];
        if rest.starts_with(b"}") {
            return true;
        }
        for keyword in &[
            b"end" as &[u8],
            b"else",
            b"elsif",
            b"when",
            b"rescue",
            b"ensure",
            b"in ",
        ] {
            if rest.starts_with(keyword) {
                // Ensure keyword isn't part of a longer identifier
                if rest.len() == keyword.len()
                    || !rest[keyword.len()].is_ascii_alphanumeric() && rest[keyword.len()] != b'_'
                {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if a line represents a single-line block (contains closing `end` or `}` on same line).
fn is_single_line_block(line: &[u8]) -> bool {
    // Single-line brace block: `it { something }`
    if line.contains(&b'{') && line.contains(&b'}') {
        return true;
    }

    // Single-line do..end: `it "foo" do something end`.
    // Require `end` as the trailing keyword to avoid matching description text.
    let trimmed = trim_ascii_whitespace(line);
    if trimmed.ends_with(b"end") && contains_keyword(trimmed, b"do") {
        return true;
    }
    false
}

fn trim_ascii_whitespace(mut line: &[u8]) -> &[u8] {
    while let Some((first, rest)) = line.split_first() {
        if *first == b' ' || *first == b'\t' {
            line = rest;
        } else {
            break;
        }
    }
    while let Some((last, rest)) = line.split_last() {
        if *last == b' ' || *last == b'\t' {
            line = rest;
        } else {
            break;
        }
    }
    line
}

fn contains_keyword(line: &[u8], keyword: &[u8]) -> bool {
    if keyword.is_empty() || line.len() < keyword.len() {
        return false;
    }
    line.windows(keyword.len()).enumerate().any(|(i, window)| {
        if window != keyword {
            return false;
        }
        let left_ok = i == 0 || !line[i - 1].is_ascii_alphanumeric() && line[i - 1] != b'_';
        let right_idx = i + keyword.len();
        let right_ok = right_idx == line.len()
            || !line[right_idx].is_ascii_alphanumeric() && line[right_idx] != b'_';
        left_ok && right_ok
    })
}

/// Check if a line starts with any RSpec example keyword followed by a
/// delimiter (space, `(`, `{`, or ` {`).  Uses the canonical
/// `RSPEC_EXAMPLES` list so that all example variants (`its`, `xit`, `fit`,
/// `pending`, etc.) are recognised for the consecutive-one-liner check.
fn starts_with_example_keyword(line: &[u8]) -> bool {
    for keyword in RSPEC_EXAMPLES {
        let kw = keyword.as_bytes();
        if line.starts_with(kw) {
            // keyword must be followed by a delimiter or be the entire line
            if line.len() == kw.len() {
                return true;
            }
            let next = line[kw.len()];
            if next == b' ' || next == b'(' || next == b'{' {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(EmptyLineAfterExample, "cops/rspec/empty_line_after_example");
}
