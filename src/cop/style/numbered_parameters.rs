use crate::cop::node_type::{BLOCK_NODE, CALL_NODE, LAMBDA_NODE, NUMBERED_PARAMETERS_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-23)
///
/// FP=0, FN=3. All FN were lambda literals (`-> do ... end`) using numbered
/// parameters in multi-line blocks. The cop only handled `CallNode` with a
/// `BlockNode` child but missed `LambdaNode` which is the Prism representation
/// of `->` lambda literals. Fixed by adding `LAMBDA_NODE` to interested node
/// types and handling it alongside block nodes.
pub struct NumberedParameters;

impl Cop for NumberedParameters {
    fn name(&self) -> &'static str {
        "Style/NumberedParameters"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[BLOCK_NODE, CALL_NODE, LAMBDA_NODE, NUMBERED_PARAMETERS_NODE]
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
        let style = config.get_str("EnforcedStyle", "allow_single_line");

        // Handle lambda literals (-> { ... } / -> do ... end)
        if let Some(lambda_node) = node.as_lambda_node() {
            let params = match lambda_node.parameters() {
                Some(p) => p,
                None => return,
            };
            if params.as_numbered_parameters_node().is_none() {
                return;
            }
            self.report_if_applicable(source, &lambda_node.location(), style, diagnostics);
            return;
        }

        // Handle method call blocks (foo.bar { ... } / foo.bar do ... end)
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        let block = match call.block() {
            Some(b) => b,
            None => return,
        };

        let block_node = match block.as_block_node() {
            Some(b) => b,
            None => return,
        };

        // In Prism, blocks with numbered params have parameters() set to a
        // NumberedParametersNode. Blocks with explicit params have BlockParametersNode.
        // Only proceed if parameters is a NumberedParametersNode — this is the
        // authoritative way to detect numbered parameter usage via the AST,
        // avoiding false positives from string matching _1.._9 in comments,
        // strings, or variable names like _1_foo.
        let params = match block_node.parameters() {
            Some(p) => p,
            None => return,
        };

        if params.as_numbered_parameters_node().is_none() {
            return;
        }

        self.report_if_applicable(source, &node.location(), style, diagnostics);
    }
}

impl NumberedParameters {
    fn report_if_applicable(
        &self,
        source: &SourceFile,
        loc: &ruby_prism::Location<'_>,
        style: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if style == "disallow" {
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                "Avoid using numbered parameters.".to_string(),
            ));
        }

        if style == "allow_single_line" {
            let (start_line, _) = source.offset_to_line_col(loc.start_offset());
            let (end_line, _) = source.offset_to_line_col(loc.end_offset().saturating_sub(1));
            if start_line != end_line {
                let (line, column) = source.offset_to_line_col(loc.start_offset());
                diagnostics.push(self.diagnostic(
                    source,
                    line,
                    column,
                    "Avoid using numbered parameters for multi-line blocks.".to_string(),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(NumberedParameters, "cops/style/numbered_parameters");
}
