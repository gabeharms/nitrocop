use crate::cop::node_type::POST_EXECUTION_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EndBlock;

impl Cop for EndBlock {
    fn name(&self) -> &'static str {
        "Style/EndBlock"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[POST_EXECUTION_NODE]
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
        let post_exe = match node.as_post_execution_node() {
            Some(n) => n,
            None => return,
        };

        let kw_loc = post_exe.keyword_loc();
        let (line, column) = source.offset_to_line_col(kw_loc.start_offset());
        let mut diag = self.diagnostic(
            source,
            line,
            column,
            "Avoid the use of `END` blocks. Use `Kernel#at_exit` instead.".to_string(),
        );

        if let Some(ref mut corr) = corrections {
            corr.push(crate::correction::Correction {
                start: kw_loc.start_offset(),
                end: kw_loc.end_offset(),
                replacement: "at_exit".to_string(),
                cop_name: self.name(),
                cop_index: 0,
            });
            diag.corrected = true;
        }

        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(EndBlock, "cops/style/end_block");
    crate::cop_autocorrect_fixture_tests!(EndBlock, "cops/style/end_block");
}
