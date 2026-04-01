use std::collections::HashMap;
use std::io::Write;

use crate::diagnostic::{Diagnostic, Severity};
use crate::formatter::Formatter;

pub struct ProgressFormatter;

impl Formatter for ProgressFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], file_count: usize, out: &mut dyn Write) {
        // Build map of file path -> worst severity
        let mut worst_by_file: HashMap<&str, Severity> = HashMap::new();
        for d in diagnostics {
            worst_by_file
                .entry(&d.path)
                .and_modify(|s| {
                    if d.severity > *s {
                        *s = d.severity;
                    }
                })
                .or_insert(d.severity);
        }

        // Print progress line: severity letter for offense files, dot for clean files
        let offense_file_count = worst_by_file.len();
        let clean_file_count = file_count.saturating_sub(offense_file_count);
        let mut progress = String::new();
        for (_, &severity) in &worst_by_file {
            progress.push(severity.letter());
        }
        for _ in 0..clean_file_count {
            progress.push('.');
        }
        let _ = writeln!(out, "{progress}");

        // Print offense details
        for d in diagnostics {
            let _ = writeln!(out, "{d}");
        }

        // Summary
        let offense_word = if diagnostics.len() == 1 {
            "offense"
        } else {
            "offenses"
        };
        let file_word = if file_count == 1 { "file" } else { "files" };
        let corrected_count = diagnostics.iter().filter(|d| d.corrected).count();
        if corrected_count > 0 {
            let corrected_word = if corrected_count == 1 {
                "offense"
            } else {
                "offenses"
            };
            let _ = writeln!(
                out,
                "\n{file_count} {file_word} inspected, {} {offense_word} detected, {corrected_count} {corrected_word} corrected",
                diagnostics.len(),
            );
        } else {
            let _ = writeln!(
                out,
                "\n{file_count} {file_word} inspected, {} {offense_word} detected",
                diagnostics.len(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Severity};

    fn make_diag(path: &str, sev: Severity) -> Diagnostic {
        Diagnostic {
            path: path.to_string(),
            location: Location { line: 1, column: 0 },
            severity: sev,
            cop_name: "Style/Test".to_string(),
            message: "test".to_string(),

            corrected: false,
        }
    }

    fn render(diagnostics: &[Diagnostic], file_count: usize) -> String {
        let mut buf = Vec::new();
        ProgressFormatter.format_to(diagnostics, file_count, &mut buf);
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn all_clean_files_show_dots() {
        let out = render(&[], 3);
        assert!(out.starts_with("...\n"));
        assert!(out.contains("3 files inspected, 0 offenses detected"));
    }

    #[test]
    fn offense_file_shows_severity_letter() {
        let diags = vec![make_diag("b.rb", Severity::Convention)];
        let out = render(&diags, 3);
        let first_line = out.lines().next().unwrap();
        // 1 offense letter + 2 clean dots
        assert_eq!(first_line.len(), 3);
        assert!(first_line.contains('C'));
        assert_eq!(first_line.matches('.').count(), 2);
    }

    #[test]
    fn worst_severity_wins() {
        let diags = vec![
            make_diag("a.rb", Severity::Convention),
            make_diag("a.rb", Severity::Error),
        ];
        let out = render(&diags, 1);
        assert!(out.starts_with("E\n"));
    }

    #[test]
    fn mixed_files() {
        let diags = vec![
            make_diag("a.rb", Severity::Convention),
            make_diag("c.rb", Severity::Warning),
        ];
        let out = render(&diags, 4);
        let first_line = out.lines().next().unwrap();
        // 2 offense letters + 2 clean dots
        assert_eq!(first_line.len(), 4);
        assert!(first_line.contains('C'));
        assert!(first_line.contains('W'));
        assert_eq!(first_line.matches('.').count(), 2);
        assert!(out.contains("4 files inspected, 2 offenses detected"));
    }

    #[test]
    fn offense_details_included() {
        let d = Diagnostic {
            path: "foo.rb".to_string(),
            location: Location { line: 5, column: 3 },
            severity: Severity::Warning,
            cop_name: "Lint/Bad".to_string(),
            message: "bad thing".to_string(),

            corrected: false,
        };
        let out = render(&[d], 1);
        assert!(out.contains("foo.rb:5:3: W: Lint/Bad: bad thing"));
    }

    #[test]
    fn empty_files() {
        let out = render(&[], 0);
        assert!(out.starts_with("\n")); // empty progress line
        assert!(out.contains("0 files inspected, 0 offenses detected"));
    }

    #[test]
    fn summary_includes_corrected_count() {
        let mut d1 = make_diag("a.rb", Severity::Convention);
        d1.corrected = true;
        let d2 = make_diag("a.rb", Severity::Convention);
        let out = render(&[d1, d2], 1);
        assert!(
            out.contains("2 offenses detected, 1 offense corrected"),
            "Expected corrected count in summary, got: {out}"
        );
    }
}
