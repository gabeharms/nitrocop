use crate::cop::node_type::{UNTIL_NODE, WHILE_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct WhileUntilDo;

impl Cop for WhileUntilDo {
    fn name(&self) -> &'static str {
        "Style/WhileUntilDo"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[UNTIL_NODE, WHILE_NODE]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Check while ... do
        if let Some(while_node) = node.as_while_node() {
            if let Some(diag) = check_loop_with_do_loc(
                self,
                source,
                &while_node.keyword_loc(),
                while_node.closing_loc(),
                while_node.do_keyword_loc(),
                "while",
            ) {
                diagnostics.push(diag);
            }
            return;
        }

        // Check until ... do
        if let Some(until_node) = node.as_until_node() {
            if let Some(diag) = check_loop_with_do_loc(
                self,
                source,
                &until_node.keyword_loc(),
                until_node.closing_loc(),
                until_node.do_keyword_loc(),
                "until",
            ) {
                diagnostics.push(diag);
            }
        }
    }
}

fn check_loop_with_do_loc(
    cop: &WhileUntilDo,
    source: &SourceFile,
    keyword_loc: &ruby_prism::Location<'_>,
    closing_loc: Option<ruby_prism::Location<'_>>,
    do_keyword_loc: Option<ruby_prism::Location<'_>>,
    keyword: &str,
) -> Option<Diagnostic> {
    // Must have a closing `end` (not a modifier form)
    let closing = closing_loc?;

    // Must have a `do` keyword
    let do_loc = do_keyword_loc?;

    let (start_line, _) = source.offset_to_line_col(keyword_loc.start_offset());
    let (end_line, _) = source.offset_to_line_col(closing.start_offset());

    // Single-line: no offense
    if start_line == end_line {
        return None;
    }

    let (line, column) = source.offset_to_line_col(do_loc.start_offset());

    Some(cop.diagnostic(
        source,
        line,
        column,
        format!("Do not use `do` with multi-line `{}`.", keyword),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(WhileUntilDo, "cops/style/while_until_do");
}
