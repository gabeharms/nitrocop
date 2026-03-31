use crate::cop::node_type::{
    BLOCK_NODE, BLOCK_PARAMETERS_NODE, CALL_NODE, INTEGER_NODE, PARENTHESES_NODE, RANGE_NODE,
    STATEMENTS_NODE,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EachForSimpleLoop;

impl Cop for EachForSimpleLoop {
    fn name(&self) -> &'static str {
        "Style/EachForSimpleLoop"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            BLOCK_NODE,
            BLOCK_PARAMETERS_NODE,
            CALL_NODE,
            INTEGER_NODE,
            PARENTHESES_NODE,
            RANGE_NODE,
            STATEMENTS_NODE,
        ]
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
        let call_node = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        if call_node.name().as_slice() != b"each" {
            return;
        }

        let block = match call_node.block() {
            Some(b) => b,
            None => return,
        };

        let block_node = match block.as_block_node() {
            Some(b) => b,
            None => return,
        };

        if let Some(params) = block_node.parameters() {
            if let Some(bp) = params.as_block_parameters_node() {
                if let Some(inner_params) = bp.parameters() {
                    let has_params = !inner_params.requireds().is_empty()
                        || !inner_params.optionals().is_empty()
                        || inner_params.rest().is_some()
                        || !inner_params.posts().is_empty()
                        || !inner_params.keywords().is_empty()
                        || inner_params.keyword_rest().is_some()
                        || inner_params.block().is_some();
                    if has_params {
                        return;
                    }
                }
            } else {
                return;
            }
        }

        let receiver = match call_node.receiver() {
            Some(r) => r,
            None => return,
        };

        let parens = match receiver.as_parentheses_node() {
            Some(p) => p,
            None => return,
        };

        let parens_body = match parens.body() {
            Some(body) => body,
            None => return,
        };

        let range_node = if let Some(r) = parens_body.as_range_node() {
            r
        } else if let Some(stmts) = parens_body.as_statements_node() {
            let body: Vec<_> = stmts.body().iter().collect();
            if body.len() != 1 {
                return;
            }
            match body[0].as_range_node() {
                Some(r) => r,
                None => return,
            }
        } else {
            return;
        };

        let left = match range_node.left() {
            Some(l) => l,
            None => return,
        };
        let right = match range_node.right() {
            Some(r) => r,
            None => return,
        };

        if left.as_integer_node().is_none() || right.as_integer_node().is_none() {
            return;
        }

        let (line, column) = source.offset_to_line_col(receiver.location().start_offset());
        let mut diag = self.diagnostic(
            source,
            line,
            column,
            "Use `Integer#times` for a simple loop which iterates a fixed number of times."
                .to_string(),
        );

        if let (Some(left_value), Some(right_value)) = (
            parse_integer_literal(source, &left),
            parse_integer_literal(source, &right),
        ) {
            let iterations = if range_node.is_exclude_end() {
                right_value - left_value
            } else {
                right_value - left_value + 1
            }
            .max(0);

            if let Some(corr) = corrections.as_mut() {
                corr.push(crate::correction::Correction {
                    start: receiver.location().start_offset(),
                    end: receiver.location().end_offset(),
                    replacement: iterations.to_string(),
                    cop_name: self.name(),
                    cop_index: 0,
                });

                if let Some(msg_loc) = call_node.message_loc() {
                    corr.push(crate::correction::Correction {
                        start: msg_loc.start_offset(),
                        end: msg_loc.end_offset(),
                        replacement: "times".to_string(),
                        cop_name: self.name(),
                        cop_index: 0,
                    });
                }
                diag.corrected = true;
            }
        }

        diagnostics.push(diag);
    }
}

fn parse_integer_literal(source: &SourceFile, node: &ruby_prism::Node<'_>) -> Option<i64> {
    let int_node = node.as_integer_node()?;
    let raw = source
        .byte_slice(
            int_node.location().start_offset(),
            int_node.location().end_offset(),
            "",
        )
        .replace('_', "");
    raw.parse::<i64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(EachForSimpleLoop, "cops/style/each_for_simple_loop");
    crate::cop_autocorrect_fixture_tests!(EachForSimpleLoop, "cops/style/each_for_simple_loop");
}
