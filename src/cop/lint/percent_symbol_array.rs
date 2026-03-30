use crate::cop::node_type::ARRAY_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct PercentSymbolArray;

impl Cop for PercentSymbolArray {
    fn name(&self) -> &'static str {
        "Lint/PercentSymbolArray"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[ARRAY_NODE]
    }

    fn supports_autocorrect(&self) -> bool {
        true
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
        if !open_src.starts_with(b"%i") && !open_src.starts_with(b"%I") {
            return;
        }

        // Check if any element has colons or commas
        for element in array_node.elements().iter() {
            let elem_loc = element.location();
            let elem_src = &source.as_bytes()[elem_loc.start_offset()..elem_loc.end_offset()];

            // Skip non-alphanumeric only elements
            let has_alnum = elem_src.iter().any(|b| b.is_ascii_alphanumeric());
            if !has_alnum {
                continue;
            }

            if elem_src.starts_with(b":") || elem_src.ends_with(b",") {
                let loc = array_node.location();
                let (line, column) = source.offset_to_line_col(loc.start_offset());
                let mut diag = self.diagnostic(
                    source,
                    line,
                    column,
                    "Within `%i`/`%I`, ':' and ',' are unnecessary and may be unwanted in the resulting symbols.".to_string(),
                );

                if let Some(ref mut corr) = corrections {
                    for element in array_node.elements().iter() {
                        let elem_loc = element.location();
                        let elem_src =
                            &source.as_bytes()[elem_loc.start_offset()..elem_loc.end_offset()];
                        let has_alnum = elem_src.iter().any(|b| b.is_ascii_alphanumeric());
                        if !has_alnum {
                            continue;
                        }

                        if elem_src.starts_with(b":") {
                            corr.push(crate::correction::Correction {
                                start: elem_loc.start_offset(),
                                end: elem_loc.start_offset() + 1,
                                replacement: "".to_string(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                        }
                        if elem_src.ends_with(b",")
                            && elem_loc.end_offset() > elem_loc.start_offset()
                        {
                            corr.push(crate::correction::Correction {
                                start: elem_loc.end_offset() - 1,
                                end: elem_loc.end_offset(),
                                replacement: "".to_string(),
                                cop_name: self.name(),
                                cop_index: 0,
                            });
                        }
                    }
                    diag.corrected = true;
                }

                diagnostics.push(diag);
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(PercentSymbolArray, "cops/lint/percent_symbol_array");
    crate::cop_autocorrect_fixture_tests!(PercentSymbolArray, "cops/lint/percent_symbol_array");
}
