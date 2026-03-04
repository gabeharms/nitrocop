use crate::cop::node_type::CALL_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-03)
///
/// Corpus oracle reported FP=3,270, FN=32,439. Four root causes identified and fixed:
/// (A) Trailing braceless KeywordHashNode not expanded into individual elements —
///     `method(key: v)` seen as 1 arg, skipped by `len < 2`. Fixed by expanding
///     last arg's `elements()` when it's a KeywordHashNode (matching RuboCop line 98).
/// (B) `AllowMultilineFinalElement` config read but stored in `_allow_multiline_final`
///     (unused). Wired into `all_on_same_line?` early return.
/// (C) Missing `all_on_same_line?` check — RuboCop returns early when all args fit on
///     one line in a multiline call. Added, matching `multiline_hash_key_line_breaks.rs`.
/// (D) Bracket assignment `[]=` not skipped (RuboCop's `return if node.method?(:[]=)`).
/// (E) Pairwise `==` replaced with `last_seen_line >= first_line` tracking.
///
/// Acceptance gate after fix: expected=60,554, actual=36,292, excess_FP=0, missing_FN=24,262.
/// +4,900 new correct detections vs CI baseline (all verified as true positives).
///
/// ## Remaining FN=24,262 (2026-03-03)
///
/// The remaining false negatives likely come from patterns not yet handled:
/// - Method calls without explicit parentheses (no opening_loc/closing_loc)
/// - `super` / `yield` calls (not CallNode in Prism)
/// - Complex nested call chains where the outer call lacks parens
/// - Possibly `send` vs `csend` differences in safe navigation edge cases
pub struct MultilineMethodArgumentLineBreaks;

impl Cop for MultilineMethodArgumentLineBreaks {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodArgumentLineBreaks"
    }

    fn default_enabled(&self) -> bool {
        false
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE]
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
        let allow_multiline_final = config.get_bool("AllowMultilineFinalElement", false);

        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        // Issue D: Skip bracket assignment ([]=)
        if call.name().as_slice() == b"[]=" {
            return;
        }

        let open_loc = match call.opening_loc() {
            Some(loc) => loc,
            None => return,
        };
        let close_loc = match call.closing_loc() {
            Some(loc) => loc,
            None => return,
        };

        if open_loc.as_slice() != b"(" || close_loc.as_slice() != b")" {
            return;
        }

        let args = match call.arguments() {
            Some(a) => a,
            None => return,
        };

        let (open_line, _) = source.offset_to_line_col(open_loc.start_offset());
        let (close_line, _) = source.offset_to_line_col(close_loc.start_offset());

        // Only check multiline calls
        if open_line == close_line {
            return;
        }

        // Issue A: Expand trailing keyword hash into individual key-value pairs.
        // RuboCop treats braceless keyword hash elements as separate arguments.
        // Collect (start_offset, end_offset) pairs for each effective argument.
        let raw_args: Vec<ruby_prism::Node<'_>> = args.arguments().iter().collect();
        let mut offsets: Vec<(usize, usize)> = Vec::new();
        for (i, arg) in raw_args.iter().enumerate() {
            if i == raw_args.len() - 1 {
                if let Some(kw_hash) = arg.as_keyword_hash_node() {
                    // Expand braceless keyword hash into individual elements
                    for elem in kw_hash.elements().iter() {
                        offsets
                            .push((elem.location().start_offset(), elem.location().end_offset()));
                    }
                    continue;
                }
            }
            offsets.push((arg.location().start_offset(), arg.location().end_offset()));
        }

        if offsets.len() < 2 {
            return;
        }

        // Issue C: all_on_same_line? early return (mirrors RuboCop's MultilineElementLineBreaks mixin)
        let first_start_line = source.offset_to_line_col(offsets[0].0).0;
        let last_offsets = offsets.last().unwrap();

        if allow_multiline_final {
            // Issue B: AllowMultilineFinalElement — check first.first_line == last.first_line
            let last_start_line = source.offset_to_line_col(last_offsets.0).0;
            if first_start_line == last_start_line {
                return;
            }
        } else {
            // Default: check first.first_line == last.last_line
            let last_end_line = source
                .offset_to_line_col(last_offsets.1.saturating_sub(1))
                .0;
            if first_start_line == last_end_line {
                return;
            }
        }

        // Issue E: Replace pairwise loop with last_seen_line tracking
        // Matches RuboCop's check_line_breaks: last_seen_line >= child.first_line → offense
        let mut last_seen_line: isize = -1;
        for &(start, end) in &offsets {
            let (arg_start_line, arg_start_col) = source.offset_to_line_col(start);
            let arg_end_line = source.offset_to_line_col(end.saturating_sub(1)).0;

            if last_seen_line >= arg_start_line as isize {
                diagnostics.push(
                    self.diagnostic(
                        source,
                        arg_start_line,
                        arg_start_col,
                        "Each argument in a multi-line method call must start on a separate line."
                            .to_string(),
                    ),
                );
            } else {
                last_seen_line = arg_end_line as isize;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(
        MultilineMethodArgumentLineBreaks,
        "cops/layout/multiline_method_argument_line_breaks"
    );
}
