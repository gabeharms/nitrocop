use crate::cop::node_type::BLOCK_NODE;
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
pub struct SpaceAroundBlockParameters;

impl Cop for SpaceAroundBlockParameters {
    fn name(&self) -> &'static str {
        "Layout/SpaceAroundBlockParameters"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[BLOCK_NODE]
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

        let block = match node.as_block_node() {
            Some(b) => b,
            None => return,
        };

        let params = match block.parameters() {
            Some(p) => p,
            None => return,
        };
        let block_params = match params.as_block_parameters_node() {
            Some(bp) => bp,
            None => return,
        };
        let opening_loc = match block_params.opening_loc() {
            Some(loc) if loc.as_slice() == b"|" => loc,
            _ => return,
        };
        let closing_loc = match block_params.closing_loc() {
            Some(loc) if loc.as_slice() == b"|" => loc,
            _ => return,
        };

        let bytes = source.as_bytes();
        let inner_start = opening_loc.end_offset();
        let inner_end = closing_loc.start_offset();
        if inner_start > inner_end || inner_end > bytes.len() {
            return;
        }
        let Some(first_non_ws) = first_non_whitespace(bytes, inner_start, inner_end) else {
            return;
        };
        let Some(last_non_ws) = last_non_whitespace(bytes, inner_start, inner_end) else {
            return;
        };
        let trailing_start = last_non_ws + 1;

        match style {
            "no_space" => {
                if first_non_ws > inner_start
                    && !contains_line_break(bytes, inner_start, first_non_ws)
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
                            end: first_non_ws,
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
            }
            "space" => {
                let opening_has_newline = contains_line_break(bytes, inner_start, first_non_ws);
                if !opening_has_newline && first_non_ws == inner_start {
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

                if !opening_has_newline && first_non_ws > inner_start + 1 {
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
                            end: first_non_ws,
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
            }
            _ => {}
        }

        let Some(body) = block.body() else {
            return;
        };
        let after_closing_start = closing_loc.end_offset();
        let body_start = body.location().start_offset();
        if after_closing_start > body_start
            || contains_line_break(bytes, after_closing_start, body_start)
        {
            return;
        }
        if after_closing_start == body_start {
            let (line, col) = source.offset_to_line_col(closing_loc.start_offset());
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

fn first_non_whitespace(bytes: &[u8], start: usize, end: usize) -> Option<usize> {
    (start..end).find(|&idx| !matches!(bytes[idx], b' ' | b'\t' | b'\n' | b'\r'))
}

fn last_non_whitespace(bytes: &[u8], start: usize, end: usize) -> Option<usize> {
    (start..end)
        .rev()
        .find(|&idx| !matches!(bytes[idx], b' ' | b'\t' | b'\n' | b'\r'))
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
