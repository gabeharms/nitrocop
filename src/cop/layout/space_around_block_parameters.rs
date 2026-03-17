use crate::cop::node_type::{BLOCK_NODE, LAMBDA_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-10)
///
/// CI baseline reported FP=3, FN=126.
///
/// The sampled FP fell into two shapes:
/// - empty block parameters written as `| |`, which RuboCop ignores;
/// - multiline parameter pipes where the closing `|` is on its own line and
///   the indentation before that pipe was being mistaken for "space after last
///   block parameter".
///
/// The dominant FN family was the missing `space after closing |` check on
/// single-line blocks such as `proc {|s|cmd.call s}` and `map{|x|...}`.
///
/// This pass switches the pipe checks to span-based whitespace handling:
/// newline-containing gaps are left to `Layout/MultilineBlockLayout`, empty
/// `| |` is skipped, and same-line `|body` now reports the missing space after
/// the closing pipe.
///
/// ## Corpus investigation (2026-03-14)
///
/// Remaining FN=20, all "Space before first block parameter detected."
/// Root cause: the cop did not handle `LambdaNode` (stabby lambdas with
/// `()` delimiters). RuboCop's `on_block` handles both block and lambda
/// nodes and checks `()` delimiters for lambdas. Added `LAMBDA_NODE` to
/// interested node types and handle `(` `)` delimiters.
///
/// Also added "Extra space before block parameter detected." check for
/// individual arguments (RuboCop's `check_each_arg`), which was missing
/// entirely — this detects extra whitespace before non-first args like
/// `|x,   y|`.
///
/// ## Corpus investigation (2026-03-15)
///
/// Remaining FN=18 from missing recursive descent into destructured (mlhs)
/// parameter groups. RuboCop's `check_arg` recurses into `mlhs_type?` nodes
/// to check extra space inside patterns like `(x,  y)`. nitrocop's
/// `collect_param_locations` only collected top-level params, so inner params
/// of `MultiTargetNode` groups were never checked. Fix: recurse into
/// `MultiTargetNode` children via `collect_multi_target_locations`.
///
/// ## Corpus investigation (2026-03-16)
///
/// Remaining FN=18, all "Space before first block parameter detected." in
/// blocks with block-local variables using semicolon syntax (`|; foo|`,
/// `|;glark|`, `|;a|`). Root cause: `collect_param_locations` only gathered
/// regular params from `block_params.parameters()`, ignoring block-local
/// variables from `block_params.locals()`. The byte-scanning approach
/// (`first_non_ws`) treated the `;` as the first content, missing the gap
/// between `|` and the local variable name. RuboCop's `arguments.children`
/// includes shadow vars, so its first/last arg checks naturally span them.
/// Fix: (1) include `BlockLocalVariableNode` locations in `collect_param_locations`,
/// (2) replace byte-scanning with AST-based param positions for first/last
/// arg boundary checks, (3) add `find_trailing_end` for trailing comma handling.
pub struct SpaceAroundBlockParameters;

/// Extracted info about a block or lambda's parameters and body.
struct BlockInfo {
    /// Byte offset right after the opening delimiter (| or ().
    inner_start: usize,
    /// Byte offset of the closing delimiter.
    inner_end: usize,
    /// The closing delimiter location (for "space after closing" check).
    closing_end_offset: usize,
    /// Start offset of the closing delimiter (for diagnostic location).
    closing_start_offset: usize,
    /// Body start offset (None if no body).
    body_start: Option<usize>,
    /// Whether the closing delimiter is `|` (blocks) vs `)` (lambdas).
    /// Only blocks get the "space after closing `|`" check.
    is_pipe_delimited: bool,
    /// Parameter nodes for per-arg extra-space checking.
    param_locations: Vec<(usize, usize)>,
}

impl Cop for SpaceAroundBlockParameters {
    fn name(&self) -> &'static str {
        "Layout/SpaceAroundBlockParameters"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[BLOCK_NODE, LAMBDA_NODE]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyleInsidePipes", "no_space");

        let info = if let Some(block) = node.as_block_node() {
            extract_block_info(&block)
        } else if let Some(lambda) = node.as_lambda_node() {
            extract_lambda_info(&lambda)
        } else {
            return;
        };

        let Some(info) = info else {
            return;
        };

        let bytes = source.as_bytes();
        let inner_start = info.inner_start;
        let inner_end = info.inner_end;

        if inner_start > inner_end || inner_end > bytes.len() {
            return;
        }

        // Use AST-based param locations to determine first/last arg boundaries.
        // This is more accurate than byte scanning because it correctly handles
        // block-local variables (|; x| or |a; x|) where the semicolon should
        // not be treated as the first/last parameter content.
        let first_arg_start = info.param_locations.first().map(|&(s, _)| s);
        let last_arg_end = info.param_locations.last().map(|&(_, e)| e);

        let Some(first_arg_start) = first_arg_start else {
            return;
        };
        let Some(last_arg_end) = last_arg_end else {
            return;
        };

        // Account for trailing comma after last arg (RuboCop's last_end_pos_inside_pipes)
        let trailing_start = find_trailing_end(bytes, last_arg_end, inner_end);

        match style {
            "no_space" => {
                if first_arg_start > inner_start
                    && !contains_line_break(bytes, inner_start, first_arg_start)
                {
                    let (line, col) = source.offset_to_line_col(inner_start);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "Space before first block parameter detected.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: inner_start,
                            end: first_arg_start,
                            replacement: String::new(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }

                if trailing_start < inner_end
                    && !contains_line_break(bytes, trailing_start, inner_end)
                {
                    let (line, col) = source.offset_to_line_col(trailing_start);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "Space after last block parameter detected.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: trailing_start,
                            end: inner_end,
                            replacement: String::new(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }

                // Check each individual arg for extra space before it
                // (RuboCop's check_each_arg / check_arg).
                self.check_each_arg_extra_space(
                    source,
                    bytes,
                    &info.param_locations,
                    diagnostics,
                    &mut corrections,
                );
            }
            "space" => {
                let opening_has_newline = contains_line_break(bytes, inner_start, first_arg_start);
                if !opening_has_newline && first_arg_start == inner_start {
                    let (line, col) = source.offset_to_line_col(inner_start);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "No space before first block parameter detected.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: inner_start,
                            end: inner_start,
                            replacement: " ".to_string(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }

                if !opening_has_newline && first_arg_start > inner_start + 1 {
                    let extra_start = inner_start + 1;
                    let (line, col) = source.offset_to_line_col(extra_start);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "Extra space before first block parameter detected.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: extra_start,
                            end: first_arg_start,
                            replacement: String::new(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }

                let closing_has_newline = contains_line_break(bytes, trailing_start, inner_end);
                if !closing_has_newline && trailing_start == inner_end {
                    let (line, col) = source.offset_to_line_col(inner_end);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "No space after last block parameter detected.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: inner_end,
                            end: inner_end,
                            replacement: " ".to_string(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }

                if !closing_has_newline && inner_end > trailing_start + 1 {
                    let extra_start = trailing_start + 1;
                    let (line, col) = source.offset_to_line_col(extra_start);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "Extra space after last block parameter detected.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: extra_start,
                            end: inner_end,
                            replacement: String::new(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }

                // Check each individual arg for extra space before it
                self.check_each_arg_extra_space(
                    source,
                    bytes,
                    &info.param_locations,
                    diagnostics,
                    &mut corrections,
                );
            }
            _ => {}
        }

        // "Space after closing `|` missing." — only for pipe-delimited blocks
        if info.is_pipe_delimited {
            let Some(body_start) = info.body_start else {
                return;
            };
            let after_closing_start = info.closing_end_offset;
            if after_closing_start > body_start
                || contains_line_break(bytes, after_closing_start, body_start)
            {
                return;
            }
            if after_closing_start == body_start {
                let (line, col) = source.offset_to_line_col(info.closing_start_offset);
                let mut diag = self.diagnostic(
                    source,
                    line,
                    col,
                    "Space after closing `|` missing.".to_string(),
                );
                if let Some(ref mut corr) = corrections {
                    corr.push(crate::correction::Correction {
                        start: body_start,
                        end: body_start,
                        replacement: " ".to_string(),
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

impl SpaceAroundBlockParameters {
    /// Check each argument for extra whitespace before it (more than one space
    /// after a comma). This corresponds to RuboCop's `check_each_arg` which
    /// reports "Extra space before block parameter detected."
    fn check_each_arg_extra_space(
        &self,
        source: &SourceFile,
        bytes: &[u8],
        param_locations: &[(usize, usize)],
        diagnostics: &mut Vec<Diagnostic>,
        corrections: &mut Option<&mut Vec<crate::correction::Correction>>,
    ) {
        for &(param_start, _param_end) in param_locations {
            // Look backwards from param_start for whitespace.
            // RuboCop checks `range_with_surrounding_space(side: :left)` and
            // reports if there's more than one space before the arg's start.
            // We scan backwards from param_start to find the extent of
            // whitespace, then check if the character before the whitespace
            // is a comma (or opening delimiter). Extra space = >1 space after comma.
            if param_start == 0 {
                continue;
            }
            let mut ws_start = param_start;
            while ws_start > 0 && matches!(bytes[ws_start - 1], b' ' | b'\t') {
                ws_start -= 1;
            }
            // The char before the whitespace should be a comma for this check
            if ws_start == 0 || bytes[ws_start - 1] != b',' {
                continue;
            }
            let space_len = param_start - ws_start;
            if space_len > 1 {
                // Extra space: report the range from (ws_start + 1) to param_start
                // (keeping one space, removing the rest)
                let extra_start = ws_start + 1;
                let (line, col) = source.offset_to_line_col(extra_start);
                let mut diag = self.diagnostic(
                    source,
                    line,
                    col,
                    "Extra space before block parameter detected.".to_string(),
                );
                if let Some(corr) = corrections {
                    corr.push(crate::correction::Correction {
                        start: extra_start,
                        end: param_start,
                        replacement: String::new(),
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

/// Extract block info from a BlockNode.
fn extract_block_info(block: &ruby_prism::BlockNode<'_>) -> Option<BlockInfo> {
    let params = block.parameters()?;
    let block_params = params.as_block_parameters_node()?;
    let opening_loc = block_params.opening_loc()?;
    if opening_loc.as_slice() != b"|" {
        return None;
    }
    let closing_loc = block_params.closing_loc()?;
    if closing_loc.as_slice() != b"|" {
        return None;
    }

    let param_locations = collect_param_locations(&block_params);

    Some(BlockInfo {
        inner_start: opening_loc.end_offset(),
        inner_end: closing_loc.start_offset(),
        closing_end_offset: closing_loc.end_offset(),
        closing_start_offset: closing_loc.start_offset(),
        body_start: block.body().map(|b| b.location().start_offset()),
        is_pipe_delimited: true,
        param_locations,
    })
}

/// Extract block info from a LambdaNode.
fn extract_lambda_info(lambda: &ruby_prism::LambdaNode<'_>) -> Option<BlockInfo> {
    let params = lambda.parameters()?;
    let block_params = params.as_block_parameters_node()?;
    let opening_loc = block_params.opening_loc()?;
    if opening_loc.as_slice() != b"(" {
        return None;
    }
    let closing_loc = block_params.closing_loc()?;
    if closing_loc.as_slice() != b")" {
        return None;
    }

    let param_locations = collect_param_locations(&block_params);

    Some(BlockInfo {
        inner_start: opening_loc.end_offset(),
        inner_end: closing_loc.start_offset(),
        closing_end_offset: closing_loc.end_offset(),
        closing_start_offset: closing_loc.start_offset(),
        body_start: lambda.body().map(|b| b.location().start_offset()),
        is_pipe_delimited: false,
        param_locations,
    })
}

/// Collect (start_offset, end_offset) for each parameter in the block_params.
/// Recursively descends into destructured (MultiTargetNode) parameters to check
/// inner args too, matching RuboCop's `check_arg` which recurses into `mlhs_type?`.
/// Also includes block-local variables (`|x; local|`) since RuboCop treats them
/// as children of the arguments node for spacing purposes.
fn collect_param_locations(
    block_params: &ruby_prism::BlockParametersNode<'_>,
) -> Vec<(usize, usize)> {
    let mut locations = Vec::new();

    if let Some(params_node) = block_params.parameters() {
        // Collect all required, optional, rest, keyword, etc. parameters
        for p in params_node.requireds().iter() {
            locations.push((p.location().start_offset(), p.location().end_offset()));
            // Recurse into destructured params like (x, y)
            if let Some(mt) = p.as_multi_target_node() {
                collect_multi_target_locations(&mt, &mut locations);
            }
        }
        for p in params_node.optionals().iter() {
            locations.push((p.location().start_offset(), p.location().end_offset()));
        }
        if let Some(rest) = params_node.rest() {
            locations.push((rest.location().start_offset(), rest.location().end_offset()));
        }
        for p in params_node.posts().iter() {
            locations.push((p.location().start_offset(), p.location().end_offset()));
            if let Some(mt) = p.as_multi_target_node() {
                collect_multi_target_locations(&mt, &mut locations);
            }
        }
        for p in params_node.keywords().iter() {
            locations.push((p.location().start_offset(), p.location().end_offset()));
        }
        if let Some(kw_rest) = params_node.keyword_rest() {
            locations.push((
                kw_rest.location().start_offset(),
                kw_rest.location().end_offset(),
            ));
        }
        if let Some(block) = params_node.block() {
            locations.push((
                block.location().start_offset(),
                block.location().end_offset(),
            ));
        }
    }

    // Collect block-local variables (|x; local_var| or |; local_var|).
    // RuboCop's `arguments.children` includes these as shadow variables,
    // and they participate in the first/last arg spacing checks.
    for local in block_params.locals().iter() {
        if let Some(blv) = local.as_block_local_variable_node() {
            locations.push((blv.location().start_offset(), blv.location().end_offset()));
        }
    }

    // Sort by start offset so we process them in order
    locations.sort_by_key(|&(start, _)| start);
    locations
}

/// Recursively collect inner param locations from a destructured (MultiTargetNode) group.
/// E.g., for `(x, y)` this adds locations of `x` and `y` so extra-space checks apply.
fn collect_multi_target_locations(
    mt: &ruby_prism::MultiTargetNode<'_>,
    locations: &mut Vec<(usize, usize)>,
) {
    for target in mt.lefts().iter() {
        locations.push((
            target.location().start_offset(),
            target.location().end_offset(),
        ));
        if let Some(inner_mt) = target.as_multi_target_node() {
            collect_multi_target_locations(&inner_mt, locations);
        }
    }
    if let Some(rest) = mt.rest() {
        locations.push((rest.location().start_offset(), rest.location().end_offset()));
    }
    for target in mt.rights().iter() {
        locations.push((
            target.location().start_offset(),
            target.location().end_offset(),
        ));
        if let Some(inner_mt) = target.as_multi_target_node() {
            collect_multi_target_locations(&inner_mt, locations);
        }
    }
}

/// Find the end position after the last arg, including any trailing comma.
/// Matches RuboCop's `last_end_pos_inside_pipes` which scans forward from the
/// last arg's end for a trailing comma and includes it.
fn find_trailing_end(bytes: &[u8], last_arg_end: usize, inner_end: usize) -> usize {
    let mut pos = last_arg_end;
    // Skip whitespace after last arg
    while pos < inner_end && matches!(bytes[pos], b' ' | b'\t') {
        pos += 1;
    }
    // Include trailing comma if present
    if pos < inner_end && bytes[pos] == b',' {
        pos + 1
    } else {
        last_arg_end
    }
}

fn contains_line_break(bytes: &[u8], start: usize, end: usize) -> bool {
    bytes[start..end]
        .iter()
        .any(|&b| matches!(b, b'\n' | b'\r'))
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(
        SpaceAroundBlockParameters,
        "cops/layout/space_around_block_parameters"
    );
    crate::cop_autocorrect_fixture_tests!(
        SpaceAroundBlockParameters,
        "cops/layout/space_around_block_parameters"
    );
}
