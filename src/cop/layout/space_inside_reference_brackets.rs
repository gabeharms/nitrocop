use crate::cop::node_type::{
    CALL_NODE, INDEX_AND_WRITE_NODE, INDEX_OPERATOR_WRITE_NODE, INDEX_OR_WRITE_NODE,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-10)
///
/// Cached corpus oracle reported FP=12, FN=1.
///
/// Fixed FN=1: multiline empty brackets such as `items[\n ]` were treated as
/// non-empty because the empty-bracket check only accepted spaces/tabs and ran
/// after the multiline early return. Empty-bracket detection now treats CR/LF
/// as whitespace and runs before the multiline guard.
///
/// ## Corpus investigation (2026-03-13)
///
/// FP=9 across 3 repos: zammad (5), activemerchant (3), puppet (1). Two root
/// causes:
///
/// 1. **Multiline node skip (2 FPs):** RuboCop's `return if node.multiline?`
///    checks the entire send node span, not just the bracket span. For
///    `mail[ key ] = if ... end` and `memo[ key ] = { ... }`, the brackets are
///    on one line but the node spans multiple lines. Added a whole-node
///    multiline check.
///
/// 2. **Nested bracket selection (7 FPs):** RuboCop's token-based
///    `left_ref_bracket` method picks the first or last `tLBRACK2` token in
///    the node range. For `[]` (read) calls where arguments contain chained
///    brackets (e.g. `CONST[ resp[:x][:y] ]`) or the receiver has brackets
///    (e.g. `user['k'][ arg['id'] ]`), the outer brackets are never checked.
///    Added `should_skip_outer_brackets` to match this behavior.
pub struct SpaceInsideReferenceBrackets;

impl Cop for SpaceInsideReferenceBrackets {
    fn name(&self) -> &'static str {
        "Layout/SpaceInsideReferenceBrackets"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            CALL_NODE,
            INDEX_AND_WRITE_NODE,
            INDEX_OPERATOR_WRITE_NODE,
            INDEX_OR_WRITE_NODE,
        ]
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
        let enforced_style = config.get_str("EnforcedStyle", "no_space");
        let empty_style = config.get_str("EnforcedStyleForEmptyBrackets", "no_space");

        let bytes = source.as_bytes();

        let (open_start, close_start) = match reference_bracket_offsets(node) {
            Some(offsets) => offsets,
            None => return,
        };
        let open_end = open_start + 1;

        // Check for empty brackets
        let is_empty = close_start == open_end
            || (close_start > open_end
                && bytes[open_end..close_start]
                    .iter()
                    .all(|&b| matches!(b, b' ' | b'\t' | b'\n' | b'\r')));

        if is_empty {
            match empty_style {
                "no_space" => {
                    if close_start > open_end {
                        let (line, col) = source.offset_to_line_col(open_end);
                        let mut diag = self.diagnostic(
                            source,
                            line,
                            col,
                            "Do not use space inside empty reference brackets.".to_string(),
                        );
                        if let Some(ref mut corr) = corrections {
                            corr.push(crate::correction::Correction {
                                start: open_end,
                                end: close_start,
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
                    if close_start == open_end || (close_start - open_end) != 1 {
                        let (line, col) = source.offset_to_line_col(open_start);
                        let mut diag = self.diagnostic(
                            source,
                            line,
                            col,
                            "Use one space inside empty reference brackets.".to_string(),
                        );
                        if let Some(ref mut corr) = corrections {
                            corr.push(crate::correction::Correction {
                                start: open_end,
                                end: close_start,
                                replacement: " ".to_string(),
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
            return;
        }

        // Skip multiline non-empty brackets (bracket span).
        let (open_line, _) = source.offset_to_line_col(open_start);
        let (close_line, _) = source.offset_to_line_col(close_start);
        if open_line != close_line {
            return;
        }

        // RuboCop skips when the entire node is multiline (e.g. `obj[key] = if\n...\nend`),
        // not just when the brackets span multiple lines.
        let node_start_line = source.offset_to_line_col(node.location().start_offset()).0;
        let node_end_line = source.offset_to_line_col(node.location().end_offset()).0;
        if node_start_line != node_end_line {
            return;
        }

        // RuboCop's token-based bracket selection can skip the outer brackets of
        // a `[]` read call when the arguments contain nested reference brackets.
        // Match that behavior: for `[]` calls (not `[]=`), skip if the bytes
        // between the outer brackets contain `[` AND either (a) the last inner
        // `[` is preceded by `]` (chained access like `[:x][:y]`), or (b) the
        // byte before the outer `[` is `]` (receiver has brackets).
        if should_skip_outer_brackets(node, bytes, open_start, open_end, close_start) {
            return;
        }

        let space_after_open = bytes.get(open_end) == Some(&b' ');
        let space_before_close = close_start > 0 && bytes.get(close_start - 1) == Some(&b' ');

        match enforced_style {
            "no_space" => {
                if space_after_open {
                    let (line, col) = source.offset_to_line_col(open_end);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "Do not use space inside reference brackets.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: open_end,
                            end: open_end + 1,
                            replacement: String::new(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }
                if space_before_close {
                    let (line, col) = source.offset_to_line_col(close_start - 1);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "Do not use space inside reference brackets.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: close_start - 1,
                            end: close_start,
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
                if !space_after_open {
                    let (line, col) = source.offset_to_line_col(open_end);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "Use space inside reference brackets.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: open_end,
                            end: open_end,
                            replacement: " ".to_string(),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }
                if !space_before_close {
                    let (line, col) = source.offset_to_line_col(close_start);
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        col,
                        "Use space inside reference brackets.".to_string(),
                    );
                    if let Some(ref mut corr) = corrections {
                        corr.push(crate::correction::Correction {
                            start: close_start,
                            end: close_start,
                            replacement: " ".to_string(),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(
        SpaceInsideReferenceBrackets,
        "cops/layout/space_inside_reference_brackets"
    );
    crate::cop_autocorrect_fixture_tests!(
        SpaceInsideReferenceBrackets,
        "cops/layout/space_inside_reference_brackets"
    );
}

fn reference_bracket_offsets(node: &ruby_prism::Node<'_>) -> Option<(usize, usize)> {
    if let Some(call) = node.as_call_node() {
        return call_bracket_offsets(&call);
    }
    if let Some(index) = node.as_index_and_write_node() {
        return index_write_bracket_offsets(
            index.receiver(),
            index.opening_loc().start_offset(),
            index.closing_loc().start_offset(),
        );
    }
    if let Some(index) = node.as_index_operator_write_node() {
        return index_write_bracket_offsets(
            index.receiver(),
            index.opening_loc().start_offset(),
            index.closing_loc().start_offset(),
        );
    }
    if let Some(index) = node.as_index_or_write_node() {
        return index_write_bracket_offsets(
            index.receiver(),
            index.opening_loc().start_offset(),
            index.closing_loc().start_offset(),
        );
    }
    None
}

fn call_bracket_offsets(call: &ruby_prism::CallNode<'_>) -> Option<(usize, usize)> {
    let method_name = call.name().as_slice();
    if method_name != b"[]" && method_name != b"[]=" {
        return None;
    }

    let receiver = call.receiver()?;
    if method_name == b"[]=" {
        if let Some(offsets) = nested_reference_brackets(&receiver) {
            return Some(offsets);
        }
    }

    let opening_loc = call.opening_loc()?;
    let closing_loc = call.closing_loc()?;
    if opening_loc.as_slice() != b"[" || closing_loc.as_slice() != b"]" {
        return None;
    }

    Some((opening_loc.start_offset(), closing_loc.start_offset()))
}

fn index_write_bracket_offsets(
    receiver: Option<ruby_prism::Node<'_>>,
    open_start: usize,
    close_start: usize,
) -> Option<(usize, usize)> {
    receiver?;
    Some((open_start, close_start))
}

/// Returns true if the outer brackets of a `[]` read call should be skipped
/// because RuboCop's token-based bracket selection would not check them.
///
/// This matches RuboCop's `left_ref_bracket` method behavior: for a `[]` call,
/// it picks the last or first reference bracket token within the node range.
/// When arguments contain nested `[` brackets, the outer brackets can be
/// skipped if either:
/// (a) the last `[` in the bracket content is preceded by `]` (chained access), or
/// (b) the byte before the outer `[` is `]` (receiver has brackets).
fn should_skip_outer_brackets(
    node: &ruby_prism::Node<'_>,
    bytes: &[u8],
    open_start: usize,
    open_end: usize,
    close_start: usize,
) -> bool {
    // Only applies to `[]` read calls, not `[]=` or index write nodes.
    let call = match node.as_call_node() {
        Some(c) => c,
        None => return false,
    };
    if call.name().as_slice() != b"[]" {
        return false;
    }

    let inner = &bytes[open_end..close_start];

    // Check if arguments contain any `[` (nested reference brackets).
    let last_bracket_pos = match inner.iter().rposition(|&b| b == b'[') {
        Some(pos) => pos,
        None => return false, // no inner brackets
    };

    // (a) Is the last inner `[` preceded by `]` (ignoring whitespace)?
    // This indicates chained access like `response[:x][:y]`.
    let before_last = &inner[..last_bracket_pos];
    let last_non_ws = before_last
        .iter()
        .rev()
        .find(|&&b| !matches!(b, b' ' | b'\t'));
    if last_non_ws == Some(&b']') {
        return true;
    }

    // (b) Is the byte before the outer `[` a `]` (ignoring whitespace)?
    // This indicates the receiver has brackets like `user['key'][...]`.
    if open_start > 0 {
        let before_open = &bytes[..open_start];
        let prev_non_ws = before_open
            .iter()
            .rev()
            .find(|&&b| !matches!(b, b' ' | b'\t'));
        if prev_non_ws == Some(&b']') {
            return true;
        }
    }

    false
}

fn nested_reference_brackets(receiver: &ruby_prism::Node<'_>) -> Option<(usize, usize)> {
    if let Some(call) = receiver.as_call_node() {
        let method_name = call.name().as_slice();
        if method_name != b"[]" && method_name != b"[]=" {
            return None;
        }

        let opening_loc = call.opening_loc()?;
        let closing_loc = call.closing_loc()?;
        if opening_loc.as_slice() != b"[" || closing_loc.as_slice() != b"]" {
            return None;
        }

        return Some((opening_loc.start_offset(), closing_loc.start_offset()));
    }

    if let Some(index) = receiver.as_index_and_write_node() {
        return Some((
            index.opening_loc().start_offset(),
            index.closing_loc().start_offset(),
        ));
    }
    if let Some(index) = receiver.as_index_operator_write_node() {
        return Some((
            index.opening_loc().start_offset(),
            index.closing_loc().start_offset(),
        ));
    }
    if let Some(index) = receiver.as_index_or_write_node() {
        return Some((
            index.opening_loc().start_offset(),
            index.closing_loc().start_offset(),
        ));
    }

    None
}
