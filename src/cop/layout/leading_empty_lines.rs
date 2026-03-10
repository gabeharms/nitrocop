use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-10)
///
/// Corpus artifact run `22919311621` reported FP=1, FN=0 on
/// `simplecov-ruby__simplecov__522dc7d/spec/fixtures/iso-8859.rb`, a file that
/// starts with a blank line and then contains non-UTF-8 bytes without any magic
/// encoding comment.
///
/// A local aggregate rerun (`expected=662`, `actual=660`) did not isolate that
/// example and therefore was not strong enough to declare the FP gone: net
/// counts can hide offsetting FP/FN. Treat the non-UTF-8 no-encoding case as a
/// real compatibility gap until the cop explicitly skips files RuboCop would
/// reject for encoding reasons.
pub struct LeadingEmptyLines;

impl Cop for LeadingEmptyLines {
    fn name(&self) -> &'static str {
        "Layout/LeadingEmptyLines"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        if skip_for_invalid_utf8_without_magic_encoding(source) {
            return;
        }

        let bytes = source.as_bytes();
        if bytes.is_empty() {
            return;
        }

        // Find the first non-whitespace byte (approximates RuboCop's first token).
        // RuboCop checks `processed_source.tokens[0]` and reports the offense
        // at the token's position. If no tokens exist (whitespace-only file),
        // no offense is reported.
        let first_content = match bytes.iter().position(|&b| !b.is_ascii_whitespace()) {
            Some(pos) => pos,
            None => return, // whitespace-only file — no tokens, no offense
        };

        let (line, col) = source.offset_to_line_col(first_content);
        if line <= 1 {
            return; // first content is on line 1 — no leading blank lines
        }

        let mut diag = self.diagnostic(
            source,
            line,
            col,
            "Unnecessary blank line at the beginning of the source.".to_string(),
        );
        if let Some(ref mut corr) = corrections {
            corr.push(crate::correction::Correction {
                start: 0,
                end: first_content,
                replacement: String::new(),
                cop_name: self.name(),
                cop_index: 0,
            });
            diag.corrected = true;
        }
        diagnostics.push(diag);
    }
}

fn skip_for_invalid_utf8_without_magic_encoding(source: &SourceFile) -> bool {
    if std::str::from_utf8(source.as_bytes()).is_ok() {
        return false;
    }
    !has_magic_encoding_comment(source.as_bytes())
}

fn has_magic_encoding_comment(source: &[u8]) -> bool {
    for line in source.split(|&b| b == b'\n').take(2) {
        let lower = String::from_utf8_lossy(line).to_ascii_lowercase();
        if lower.contains("coding:") || lower.contains("coding=") {
            return true;
        }
        if lower.contains("encoding:") || lower.contains("encoding=") {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_scenario_fixture_tests!(
        LeadingEmptyLines,
        "cops/layout/leading_empty_lines",
        single_blank = "single_blank.rb",
        two_blanks = "two_blanks.rb",
        three_blanks = "three_blanks.rb",
    );

    #[test]
    fn empty_file() {
        let source = SourceFile::from_bytes("test.rb", b"".to_vec());
        let mut diags = Vec::new();
        LeadingEmptyLines.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(diags.is_empty());
    }

    #[test]
    fn only_newline() {
        // A file containing only blank lines (no tokens) should not be flagged.
        // Matches RuboCop: `expect_no_offenses("\n")`
        let source = SourceFile::from_bytes("test.rb", b"\n".to_vec());
        let mut diags = Vec::new();
        LeadingEmptyLines.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(
            diags.is_empty(),
            "File with only newlines should not be flagged"
        );
    }

    #[test]
    fn only_newlines() {
        // Multiple blank lines with no content should not be flagged.
        let source = SourceFile::from_bytes("test.rb", b"\n\n\n".to_vec());
        let mut diags = Vec::new();
        LeadingEmptyLines.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(
            diags.is_empty(),
            "File with only newlines should not be flagged"
        );
    }

    #[test]
    fn offense_at_first_token_line() {
        // Offense should be reported at the first non-whitespace line, not line 1.
        // Matches RuboCop which reports at `processed_source.tokens[0].pos`.
        let source = SourceFile::from_bytes("test.rb", b"\nx = 1\n".to_vec());
        let mut diags = Vec::new();
        LeadingEmptyLines.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert_eq!(diags.len(), 1);
        assert_eq!(
            diags[0].location.line, 2,
            "offense should be at line 2 (first code line)"
        );
        assert_eq!(diags[0].location.column, 0);
    }

    #[test]
    fn offense_at_first_comment_line() {
        // Comments count as "content" -- offense at comment line, not line 1.
        let source = SourceFile::from_bytes("test.rb", b"\n# comment\n".to_vec());
        let mut diags = Vec::new();
        LeadingEmptyLines.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert_eq!(diags.len(), 1);
        assert_eq!(
            diags[0].location.line, 2,
            "offense should be at line 2 (first comment line)"
        );
    }

    #[test]
    fn autocorrect_single_blank() {
        let input = b"\nx = 1\n";
        let (diags, corrections) = crate::testutil::run_cop_autocorrect(&LeadingEmptyLines, input);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].corrected);
        let cs = crate::correction::CorrectionSet::from_vec(corrections);
        let corrected = cs.apply(input);
        assert_eq!(corrected, b"x = 1\n");
    }

    #[test]
    fn autocorrect_multiple_blanks() {
        let input = b"\n\n\nx = 1\n";
        let (diags, corrections) = crate::testutil::run_cop_autocorrect(&LeadingEmptyLines, input);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].corrected);
        let cs = crate::correction::CorrectionSet::from_vec(corrections);
        let corrected = cs.apply(input);
        assert_eq!(corrected, b"x = 1\n");
    }

    #[test]
    fn ignores_non_utf8_source_without_magic_encoding() {
        let source =
            SourceFile::from_bytes("test.rb", b"\n# localized to Espa\xf1ol thus:\n".to_vec());
        let mut diags = Vec::new();
        LeadingEmptyLines.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(
            diags.is_empty(),
            "non-UTF8 source without magic encoding should be ignored"
        );
    }

    #[test]
    fn non_utf8_source_with_magic_encoding_still_registers_offense() {
        let source = SourceFile::from_bytes(
            "test.rb",
            b"\n# encoding: iso-8859-1\n# localized to Espa\xf1ol thus:\n".to_vec(),
        );
        let mut diags = Vec::new();
        LeadingEmptyLines.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].location.line, 2);
    }
}
