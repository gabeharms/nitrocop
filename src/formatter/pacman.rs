use std::collections::HashSet;
use std::io::Write;

use crate::diagnostic::Diagnostic;
use crate::formatter::Formatter;

pub struct PacmanFormatter;

/// Pac-Man character
const PACMAN: char = '\u{15E7}'; // ᗧ
/// Ghost character (file with offenses)
const GHOST: char = '\u{15E3}'; // ᗣ
/// Pacdot (clean file)
const PACDOT: char = '\u{2022}'; // •

impl Formatter for PacmanFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], file_count: usize, out: &mut dyn Write) {
        // Collect unique files with offenses
        let offense_files: HashSet<&str> = diagnostics.iter().map(|d| d.path.as_str()).collect();
        let offense_file_count = offense_files.len();
        let clean_file_count = file_count.saturating_sub(offense_file_count);

        // Build the pacman line: ghosts for offense files, pacdots for clean files
        let mut line = String::new();
        line.push(PACMAN);
        for _ in 0..offense_file_count {
            line.push(GHOST);
        }
        for _ in 0..clean_file_count {
            line.push(PACDOT);
        }
        let _ = writeln!(out, "{line}");

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

    fn make_diag(path: &str) -> Diagnostic {
        Diagnostic {
            path: path.to_string(),
            location: Location { line: 1, column: 0 },
            severity: Severity::Convention,
            cop_name: "Style/Test".to_string(),
            message: "test".to_string(),

            corrected: false,
        }
    }

    fn render(diagnostics: &[Diagnostic], file_count: usize) -> String {
        let mut buf = Vec::new();
        PacmanFormatter.format_to(diagnostics, file_count, &mut buf);
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn all_clean_shows_pacdots() {
        let out = render(&[], 2);
        let first_line = out.lines().next().unwrap();
        // Pacman + 2 pacdots
        assert_eq!(first_line, format!("{PACMAN}{PACDOT}{PACDOT}"));
    }

    #[test]
    fn offense_file_shows_ghost() {
        let diags = vec![make_diag("b.rb")];
        let out = render(&diags, 3);
        let first_line = out.lines().next().unwrap();
        // 1 ghost (offense file) + 2 pacdots (clean files)
        assert_eq!(first_line, format!("{PACMAN}{GHOST}{PACDOT}{PACDOT}"));
    }

    #[test]
    fn summary_line() {
        let diags = vec![make_diag("a.rb")];
        let out = render(&diags, 2);
        assert!(out.contains("2 files inspected, 1 offense detected"));
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
        let first_line = out.lines().next().unwrap();
        // Just pacman, no dots
        assert_eq!(first_line, format!("{PACMAN}"));
    }
}
