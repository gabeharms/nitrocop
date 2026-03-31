/// Style/SingleLineDoEndBlock
///
/// Checks for single-line `do`...`end` blocks and suggests converting them
/// to multiline form.
///
/// ## Investigation (2026-03-15)
///
/// Root cause of ~382 FP and ~384 FN: nitrocop was reporting the offense at the
/// `do` keyword location (column of `do`), but RuboCop reports at the start of
/// the entire expression (the CallNode, column 0 for `foo do end`). Since corpus
/// comparison matches on line:column, same-line offenses at different columns
/// appeared as both FP (nitrocop-only at `do` column) and FN (RuboCop-only at
/// call column). Also, the message was wrong ("Prefer braces" vs "Prefer multiline").
///
/// Fix: dispatch on CALL_NODE (for `foo do...end`, `lambda do...end`) and
/// LAMBDA_NODE (for `-> do...end`) to get the full expression location.
/// Report at the CallNode/LambdaNode start, matching RuboCop's `add_offense(node)`.
use crate::cop::node_type::{CALL_NODE, LAMBDA_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct SingleLineDoEndBlock;

impl SingleLineDoEndBlock {
    fn check_do_end_block(
        &self,
        source: &SourceFile,
        expr_start: usize,
        expr_end: usize,
        opening_loc: ruby_prism::Location<'_>,
        closing_loc: ruby_prism::Location<'_>,
        newline_insert_after: usize,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Check if it uses do...end
        if opening_loc.as_slice() != b"do" {
            return;
        }

        // Check if expression is on single line
        let (start_line, _) = source.offset_to_line_col(expr_start);
        let (end_line, _) = source.offset_to_line_col(expr_end.saturating_sub(1));
        if start_line != end_line {
            return;
        }

        // Report offense at the start of the entire expression (matches RuboCop)
        let (line, column) = source.offset_to_line_col(expr_start);
        let mut diag = self.diagnostic(
            source,
            line,
            column,
            "Prefer multiline `do`...`end` block.".to_string(),
        );

        if let Some(corr) = corrections {
            corr.push(crate::correction::Correction {
                start: newline_insert_after,
                end: newline_insert_after,
                replacement: "\n".to_string(),
                cop_name: self.name(),
                cop_index: 0,
            });
            corr.push(crate::correction::Correction {
                start: closing_loc.start_offset(),
                end: closing_loc.start_offset(),
                replacement: "\n".to_string(),
                cop_name: self.name(),
                cop_index: 0,
            });
            diag.corrected = true;
        }

        diagnostics.push(diag);
    }
}

impl Cop for SingleLineDoEndBlock {
    fn name(&self) -> &'static str {
        "Style/SingleLineDoEndBlock"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE, LAMBDA_NODE]
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
        // Handle CallNode with a do...end block (e.g., `foo do bar end`, `lambda do |x| x end`)
        if let Some(call) = node.as_call_node() {
            let block = match call.block() {
                Some(b) => match b.as_block_node() {
                    Some(bn) => bn,
                    None => return,
                },
                None => return,
            };

            let call_loc = call.location();
            let insert_after = if call.receiver().is_none() && call.name().as_slice() == b"lambda" {
                block.opening_loc().end_offset()
            } else if let Some(params) = block.parameters() {
                if params
                    .as_block_parameters_node()
                    .and_then(|bp| bp.opening_loc())
                    .is_some()
                {
                    params.location().end_offset()
                } else {
                    block.opening_loc().end_offset()
                }
            } else {
                block.opening_loc().end_offset()
            };

            self.check_do_end_block(
                source,
                call_loc.start_offset(),
                call_loc.end_offset(),
                block.opening_loc(),
                block.closing_loc(),
                insert_after,
                diagnostics,
                corrections.as_deref_mut(),
            );
            return;
        }

        // Handle LambdaNode with do...end (e.g., `->(arg) do foo end`)
        if let Some(lambda) = node.as_lambda_node() {
            let loc = lambda.location();
            self.check_do_end_block(
                source,
                loc.start_offset(),
                loc.end_offset(),
                lambda.opening_loc(),
                lambda.closing_loc(),
                lambda.opening_loc().end_offset(),
                diagnostics,
                corrections.as_deref_mut(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(SingleLineDoEndBlock, "cops/style/single_line_do_end_block");
    crate::cop_autocorrect_fixture_tests!(
        SingleLineDoEndBlock,
        "cops/style/single_line_do_end_block"
    );
}
