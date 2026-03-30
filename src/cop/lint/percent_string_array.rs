use crate::cop::node_type::ARRAY_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/PercentStringArray: checks for quotes and commas in %w/%W arrays.
///
/// ## Investigation (2026-03-08)
/// FP in rtomayko/ronn at lib/ronn/roff.rb:149 — `%W["#{node.position + 1}." 4]`.
/// Root cause: the cop checked raw source bytes of every element, including
/// InterpolatedStringNode elements in %W arrays. The element `"#{...}."` starts
/// and ends with `"` in source, but the quotes are intentional string content
/// wrapping an interpolation. RuboCop avoids this FP because its
/// `contains_quotes_or_commas?` checks `value.children.first` (the first string
/// fragment before interpolation), which is just `"` — a pure-punctuation fragment
/// that gets skipped by the alphanumeric filter.
/// Fix: skip InterpolatedStringNode elements entirely, matching RuboCop's effective
/// behavior where interpolated elements are not flagged.
pub struct PercentStringArray;

impl Cop for PercentStringArray {
    fn name(&self) -> &'static str {
        "Lint/PercentStringArray"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[ARRAY_NODE]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let array_node = match node.as_array_node() {
            Some(a) => a,
            None => return,
        };

        let open_loc = match array_node.opening_loc() {
            Some(loc) => loc,
            None => return,
        };

        let open_src = open_loc.as_slice();
        if !open_src.starts_with(b"%w") && !open_src.starts_with(b"%W") {
            return;
        }

        // Check if any element has quotes or commas
        let mut has_offense = false;
        for element in array_node.elements().iter() {
            // Skip interpolated string elements (e.g., %W["#{expr}." other]).
            // These contain #{} interpolation; their raw source may incidentally
            // start/end with quote characters that are intentional string content.
            // RuboCop effectively skips these because it checks children.first
            // which is just a punctuation fragment that fails the alphanumeric filter.
            if element.as_interpolated_string_node().is_some() {
                continue;
            }

            let elem_loc = element.location();
            let elem_src = &source.as_bytes()[elem_loc.start_offset()..elem_loc.end_offset()];

            // Skip single-character elements (e.g., just ' or ")
            let alphanumeric_count = elem_src
                .iter()
                .filter(|b| b.is_ascii_alphanumeric())
                .count();
            if alphanumeric_count == 0 {
                continue;
            }

            let has_quotes_or_commas = elem_src.ends_with(b",")
                || (elem_src.starts_with(b"'") && elem_src.ends_with(b"'"))
                || (elem_src.starts_with(b"\"") && elem_src.ends_with(b"\""));

            if has_quotes_or_commas {
                has_offense = true;
                break;
            }
        }

        if !has_offense {
            return;
        }

        let loc = array_node.location();
        let (line, column) = source.offset_to_line_col(loc.start_offset());
        let mut diag = self.diagnostic(
            source,
            line,
            column,
            "Within `%w`/`%W`, quotes and ',' are unnecessary and may be unwanted in the resulting strings."
                .to_string(),
        );

        if let Some(corr) = corrections.as_mut() {
            for element in array_node.elements().iter() {
                if element.as_interpolated_string_node().is_some() {
                    continue;
                }

                let elem_loc = element.location();
                let elem_src = &source.as_bytes()[elem_loc.start_offset()..elem_loc.end_offset()];
                let Ok(elem_text) = std::str::from_utf8(elem_src) else {
                    continue;
                };

                if elem_text
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .count()
                    == 0
                {
                    continue;
                }

                if elem_text.starts_with('"') || elem_text.starts_with('\'') {
                    corr.push(crate::correction::Correction {
                        start: elem_loc.start_offset(),
                        end: elem_loc.start_offset() + 1,
                        replacement: String::new(),
                        cop_name: self.name(),
                        cop_index: 0,
                    });
                }

                let mut trailing_trim = 0usize;
                let bytes = elem_text.as_bytes();
                if !bytes.is_empty() && bytes[bytes.len() - 1] == b',' {
                    trailing_trim += 1;
                }
                let quote_idx = bytes.len().saturating_sub(1 + trailing_trim);
                if quote_idx < bytes.len()
                    && (bytes[quote_idx] == b'"' || bytes[quote_idx] == b'\'')
                {
                    trailing_trim += 1;
                }

                if trailing_trim > 0 {
                    corr.push(crate::correction::Correction {
                        start: elem_loc.end_offset() - trailing_trim,
                        end: elem_loc.end_offset(),
                        replacement: String::new(),
                        cop_name: self.name(),
                        cop_index: 0,
                    });
                }
            }
            diag.corrected = true;
        }

        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(PercentStringArray, "cops/lint/percent_string_array");
    crate::cop_autocorrect_fixture_tests!(PercentStringArray, "cops/lint/percent_string_array");
}
