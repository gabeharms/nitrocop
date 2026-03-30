use ruby_prism::Visit;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct RescueType;

impl Cop for RescueType {
    fn name(&self) -> &'static str {
        "Lint/RescueType"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::parse::codemap::CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut visitor = RescueTypeVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
            corrections: Vec::new(),
            emit_corrections: corrections.is_some(),
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
        if let Some(ref mut corr) = corrections {
            corr.extend(visitor.corrections);
        }
    }
}

struct RescueTypeVisitor<'a, 'src> {
    cop: &'a RescueType,
    source: &'src SourceFile,
    diagnostics: Vec<Diagnostic>,
    corrections: Vec<crate::correction::Correction>,
    emit_corrections: bool,
}

impl<'pr> Visit<'pr> for RescueTypeVisitor<'_, '_> {
    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode<'pr>) {
        let exceptions = node.exceptions();
        let mut invalid = Vec::new();
        let mut valid = Vec::new();

        for exc in exceptions.iter() {
            let loc = exc.location();
            let src = &self.source.as_bytes()[loc.start_offset()..loc.end_offset()];
            let src_str = std::str::from_utf8(src).unwrap_or("?").to_string();

            if is_invalid_rescue_type(&exc) {
                invalid.push(src_str);
            } else {
                valid.push(src_str);
            }
        }

        if !invalid.is_empty() {
            let loc = node.keyword_loc();
            let (line, column) = self.source.offset_to_line_col(loc.start_offset());
            let mut diag = self.cop.diagnostic(
                self.source,
                line,
                column,
                format!(
                    "Rescuing from `{}` will raise a `TypeError` instead of catching the actual exception.",
                    invalid.join(", ")
                ),
            );

            if self.emit_corrections {
                if let Some(last_exc) = exceptions.iter().last() {
                    let start = loc.end_offset();
                    let end = last_exc.location().end_offset();
                    let replacement = if valid.is_empty() {
                        "".to_string()
                    } else {
                        format!(" {}", valid.join(", "))
                    };
                    self.corrections.push(crate::correction::Correction {
                        start,
                        end,
                        replacement,
                        cop_name: self.cop.name(),
                        cop_index: 0,
                    });
                    diag.corrected = true;
                }
            }

            self.diagnostics.push(diag);
        }

        // Continue visiting child rescue nodes (subsequent)
        ruby_prism::visit_rescue_node(self, node);
    }
}

fn is_invalid_rescue_type(node: &ruby_prism::Node<'_>) -> bool {
    node.as_nil_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_array_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_keyword_hash_node().is_some()
        || node.as_interpolated_string_node().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(RescueType, "cops/lint/rescue_type");
    crate::cop_autocorrect_fixture_tests!(RescueType, "cops/lint/rescue_type");
}
