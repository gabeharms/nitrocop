use crate::cop::node_type::{BEGIN_NODE, CONSTANT_PATH_TARGET_NODE, CONSTANT_TARGET_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct ConstantOverwrittenInRescue;

impl Cop for ConstantOverwrittenInRescue {
    fn name(&self) -> &'static str {
        "Lint/ConstantOverwrittenInRescue"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[BEGIN_NODE, CONSTANT_PATH_TARGET_NODE, CONSTANT_TARGET_NODE]
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
        let begin_node = match node.as_begin_node() {
            Some(n) => n,
            None => return,
        };

        let mut rescue_opt = begin_node.rescue_clause();

        while let Some(rescue_node) = rescue_opt {
            // Check if: no exception classes specified AND reference is a constant
            // AND rescue body is empty.
            // This matches `rescue => ConstantName` with no handler body.
            // RuboCop's pattern: (resbody nil? $(casgn _ _) nil?)
            // If there IS a body, the user may intentionally capture into a constant.
            if rescue_node.exceptions().is_empty() && rescue_node.statements().is_none() {
                if let Some(reference) = rescue_node.reference() {
                    let is_constant = reference.as_constant_target_node().is_some()
                        || reference.as_constant_path_target_node().is_some();

                    if is_constant {
                        if let Some(operator_loc) = rescue_node.operator_loc() {
                            let (line, column) =
                                source.offset_to_line_col(operator_loc.start_offset());
                            // Get the constant name from the reference
                            let ref_src = if let Some(ct) = reference.as_constant_target_node() {
                                std::str::from_utf8(ct.name().as_slice())
                                    .unwrap_or("constant")
                                    .to_string()
                            } else if let Some(cpt) = reference.as_constant_path_target_node() {
                                std::str::from_utf8(cpt.location().as_slice())
                                    .unwrap_or("constant")
                                    .to_string()
                            } else {
                                "constant".to_string()
                            };
                            let mut diag = self.diagnostic(
                                source,
                                line,
                                column,
                                format!("`{ref_src}` is overwritten by `rescue =>`."),
                            );

                            if let Some(corr) = corrections.as_mut() {
                                let keyword = rescue_node.keyword_loc();
                                corr.push(crate::correction::Correction {
                                    start: keyword.end_offset(),
                                    end: operator_loc.end_offset(),
                                    replacement: "".to_string(),
                                    cop_name: self.name(),
                                    cop_index: 0,
                                });
                                diag.corrected = true;
                            }

                            diagnostics.push(diag);
                        }
                    }
                }
            }
            rescue_opt = rescue_node.subsequent();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        ConstantOverwrittenInRescue,
        "cops/lint/constant_overwritten_in_rescue"
    );
    crate::cop_autocorrect_fixture_tests!(
        ConstantOverwrittenInRescue,
        "cops/lint/constant_overwritten_in_rescue"
    );
}
