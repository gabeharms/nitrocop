use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-03)
///
/// Corpus oracle reported FP=12,897, FN=23. Root cause: whitespace-only lines
/// (spaces/tabs) were treated as blank lines, but RuboCop's `EmptyLines` only
/// counts truly empty lines (zero bytes after newline removal). 91% of FPs came
/// from twilio-ruby's auto-generated code with indentation on blank lines.
/// Fix: changed `line.iter().all(|&b| b == b' ' || ...)` to `line.is_empty()`.
/// Acceptance gate after fix: expected=12,238, actual=13,320, excess=0, missing=0.
/// The 23 FNs are pre-existing (likely CodeMap edge cases) and unrelated.
///
/// ## Corpus investigation (2026-03-11)
///
/// FP=1,106 remained. Root cause: RuboCop uses token-based gap detection — it
/// collects line numbers that have tokens, then only checks gaps between
/// consecutive token-bearing lines. Comment-only files (no tokens) get early
/// return with no offenses. Blank lines after the last token line are never
/// checked. nitrocop was checking ALL blank lines (except inside non-code
/// ranges), which produced false positives on blank lines after the last code
/// line (common pattern: trailing comment sections after code).
/// Fix: use the Program node's end offset to find the last code line, and only
/// check blank lines within the code range. Comment-only files (empty Program
/// node where start == end) get early return.
pub struct EmptyLines;

impl Cop for EmptyLines {
    fn name(&self) -> &'static str {
        "Layout/EmptyLines"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // RuboCop uses token-based gap detection: it only checks gaps between
        // consecutive token-bearing lines. Comment-only files (no tokens) get
        // early return, and blank lines after the last token are never checked.
        // We replicate this by using the Program node's location to find the
        // last line with code (AST nodes). Blank lines after that are skipped.
        let program_node = parse_result.node();
        let program_loc = program_node.location();

        // Comment-only files: the Program node has start == end (no code).
        // RuboCop returns early when tokens are empty.
        if program_loc.start_offset() == program_loc.end_offset() {
            return;
        }

        // Find the last line that has code (1-indexed).
        let (last_code_line, _) =
            source.offset_to_line_col(program_loc.end_offset().saturating_sub(1));

        let max = config.get_usize("Max", 1);

        let mut consecutive_blanks = 0;
        let mut byte_offset: usize = 0;
        let lines: Vec<&[u8]> = source.lines().collect();
        let total_lines = lines.len();

        for (i, line) in lines.iter().enumerate() {
            let line_len = line.len() + 1; // +1 for newline
            let current_line = i + 1; // 1-indexed

            if line.is_empty() {
                // Skip the trailing empty element from split() — RuboCop's
                // EmptyLines cop doesn't flag trailing blank lines at EOF
                // (that's Layout/TrailingEmptyLines).
                if i + 1 >= total_lines {
                    break;
                }
                // Skip blank lines after the last code line. RuboCop only
                // checks between consecutive token-bearing lines and never
                // checks past the last token.
                if current_line > last_code_line {
                    byte_offset += line_len;
                    consecutive_blanks = 0;
                    continue;
                }
                // Skip blank lines inside non-code regions (heredocs, strings)
                if !code_map.is_code(byte_offset) {
                    byte_offset += line_len;
                    consecutive_blanks = 0;
                    continue;
                }
                consecutive_blanks += 1;
                if consecutive_blanks > max {
                    let mut diag = self.diagnostic(
                        source,
                        current_line,
                        0,
                        "Extra blank line detected.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: byte_offset,
                            end: byte_offset + line_len,
                            replacement: String::new(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }
            } else {
                consecutive_blanks = 0;
            }
            byte_offset += line_len;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{run_cop_full, run_cop_full_with_config};

    crate::cop_fixture_tests!(EmptyLines, "cops/layout/empty_lines");
    crate::cop_autocorrect_fixture_tests!(EmptyLines, "cops/layout/empty_lines");

    #[test]
    fn config_max_2() {
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(2.into()))]),
            ..CopConfig::default()
        };
        // 3 consecutive blank lines should trigger with Max:2
        let source = b"x = 1\n\n\n\ny = 2\n";
        let diags = run_cop_full_with_config(&EmptyLines, source, config.clone());
        assert!(
            !diags.is_empty(),
            "Should fire with Max:2 on 3 consecutive blank lines"
        );

        // 2 consecutive blank lines should NOT trigger with Max:2
        let source2 = b"x = 1\n\n\ny = 2\n";
        let diags2 = run_cop_full_with_config(&EmptyLines, source2, config);
        assert!(
            diags2.is_empty(),
            "Should not fire on 2 consecutive blank lines with Max:2"
        );

        // 2 consecutive blank lines SHOULD trigger with default Max:1
        let diags3 = run_cop_full(&EmptyLines, source2);
        assert!(
            !diags3.is_empty(),
            "Should fire with default Max:1 on 2 consecutive blank lines"
        );
    }

    #[test]
    fn autocorrect_remove_extra_blank() {
        let input = b"x = 1\n\n\ny = 2\n";
        let (_diags, corrections) = crate::testutil::run_cop_autocorrect(&EmptyLines, input);
        assert!(!corrections.is_empty());
        let cs = crate::correction::CorrectionSet::from_vec(corrections);
        let corrected = cs.apply(input);
        assert_eq!(corrected, b"x = 1\n\ny = 2\n");
    }

    #[test]
    fn autocorrect_remove_multiple_extra() {
        let input = b"x = 1\n\n\n\n\ny = 2\n";
        let (_diags, corrections) = crate::testutil::run_cop_autocorrect(&EmptyLines, input);
        assert_eq!(corrections.len(), 3); // 4 blanks, max 1, so 3 extra
        let cs = crate::correction::CorrectionSet::from_vec(corrections);
        let corrected = cs.apply(input);
        assert_eq!(corrected, b"x = 1\n\ny = 2\n");
    }

    #[test]
    fn whitespace_only_lines_are_not_blank() {
        // RuboCop only counts truly empty lines (zero bytes after stripping newline).
        // Lines with only spaces/tabs are NOT blank and should not be counted.
        let source = b"x = 1\n  \n  \ny = 2\n";
        let diags = run_cop_full(&EmptyLines, source);
        assert!(
            diags.is_empty(),
            "Whitespace-only lines should not be treated as blank: {:?}",
            diags
        );
    }

    #[test]
    fn skip_blanks_in_comment_only_file() {
        // RuboCop returns early when processed_source.tokens is empty.
        // A file with only comments has no tokens.
        let source = b"# frozen_string_literal: true\n\n\n# Another comment\n";
        let diags = run_cop_full(&EmptyLines, source);
        assert!(
            diags.is_empty(),
            "Should not fire on comment-only file: {:?}",
            diags
        );
    }

    #[test]
    fn skip_blanks_between_comment_groups() {
        // Consecutive blank lines between comments ARE checked by RuboCop
        // when there are tokens (code) in the file.
        let source = b"x = 1\n# comment\n\n\n# comment\ny = 2\n";
        let diags = run_cop_full(&EmptyLines, source);
        assert!(
            !diags.is_empty(),
            "Should fire on consecutive blank lines between comments when code exists"
        );
    }

    #[test]
    fn skip_blanks_after_last_code() {
        // RuboCop only checks between consecutive token-bearing lines.
        // After the last token, gaps are never checked.
        let source = b"x = 1\n\n\n# trailing comment\n";
        let diags = run_cop_full(&EmptyLines, source);
        assert!(
            diags.is_empty(),
            "RuboCop doesn't check after last token: {:?}",
            diags
        );
    }

    #[test]
    fn skip_blanks_after_last_code_no_trailing_comment() {
        // Consecutive blank lines after the last code with no trailing content
        let source = b"x = 1\n\n\n";
        let diags = run_cop_full(&EmptyLines, source);
        assert!(
            diags.is_empty(),
            "Should not fire after last code line: {:?}",
            diags
        );
    }

    #[test]
    fn fire_on_blanks_before_first_code() {
        // Consecutive blank lines before the first code token
        let source = b"\n\n\nx = 1\n";
        let diags = run_cop_full(&EmptyLines, source);
        assert!(
            !diags.is_empty(),
            "Should fire on consecutive blank lines at start of file"
        );
    }

    #[test]
    fn skip_blanks_in_heredoc() {
        let source = b"x = <<~RUBY\n  foo\n\n\n  bar\nRUBY\n";
        let diags = run_cop_full(&EmptyLines, source);
        assert!(
            diags.is_empty(),
            "Should not fire on blank lines inside heredoc"
        );
    }
}
