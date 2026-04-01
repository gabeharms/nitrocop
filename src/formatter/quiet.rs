use std::io::Write;

use crate::diagnostic::Diagnostic;
use crate::formatter::Formatter;

pub struct QuietFormatter;

impl Formatter for QuietFormatter {
    fn format_to(&self, diagnostics: &[Diagnostic], file_count: usize, out: &mut dyn Write) {
        if diagnostics.is_empty() {
            return;
        }
        for d in diagnostics {
            let _ = writeln!(out, "{d}");
        }
        let offense_word = if diagnostics.len() == 1 {
            "offense"
        } else {
            "offenses"
        };
        let file_word = if file_count == 1 { "file" } else { "files" };
        let _ = writeln!(
            out,
            "\n{file_count} {file_word} inspected, {} {offense_word} detected",
            diagnostics.len(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::{Location, Severity};

    fn render(diagnostics: &[Diagnostic], file_count: usize) -> String {
        let mut buf = Vec::new();
        QuietFormatter.format_to(diagnostics, file_count, &mut buf);
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn empty_produces_no_output() {
        let out = render(&[], 2);
        assert_eq!(out, "");
    }

    #[test]
    fn with_offenses_shows_details_and_summary() {
        let d = Diagnostic {
            path: "foo.rb".to_string(),
            location: Location { line: 3, column: 5 },
            severity: Severity::Convention,
            cop_name: "Style/Foo".to_string(),
            message: "bad style".to_string(),

            corrected: false,
        };
        let out = render(&[d], 1);
        assert!(out.contains("foo.rb:3:5: C: Style/Foo: bad style"));
        assert!(out.contains("1 file inspected, 1 offense detected"));
    }

    #[test]
    fn zero_offenses_zero_files_still_silent() {
        let out = render(&[], 0);
        assert_eq!(out, "");
    }
}
