use crate::cop::node_type::{
    CALL_AND_WRITE_NODE, CALL_NODE, CALL_OPERATOR_WRITE_NODE, CALL_OR_WRITE_NODE,
    INDEX_AND_WRITE_NODE, INDEX_OPERATOR_WRITE_NODE, INDEX_OR_WRITE_NODE,
};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-10)
///
/// Corpus oracle reported FP=0, FN=12.
///
/// FP=0: no corpus false positives are currently known.
///
/// FN=12: all remaining misses came from indexed operator writes like
/// `value [0] += 1` in `jruby`. Prism represents spaced bracket writes as
/// `Call*WriteNode` variants with `read_name == "[]"`, while unspaced indexed
/// writes use `Index*WriteNode` variants. The original cop only handled `[]` /
/// `[]=` call nodes, so it never visited the write-node forms. This cop now
/// applies the same whitespace-gap check to both families.
pub struct SpaceBeforeBrackets;

const MESSAGE: &str = "Remove the space before the opening brackets.";

impl Cop for SpaceBeforeBrackets {
    fn name(&self) -> &'static str {
        "Layout/SpaceBeforeBrackets"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            CALL_AND_WRITE_NODE,
            CALL_NODE,
            CALL_OPERATOR_WRITE_NODE,
            CALL_OR_WRITE_NODE,
            INDEX_AND_WRITE_NODE,
            INDEX_OPERATOR_WRITE_NODE,
            INDEX_OR_WRITE_NODE,
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        if let Some(call) = node.as_call_node() {
            let method_name = call.name().as_slice();
            if method_name != b"[]" && method_name != b"[]=" {
                return;
            }

            // Skip desugared calls like `collection.[](key)` — these have a dot
            if call.call_operator_loc().is_some() {
                return;
            }

            let receiver = match call.receiver() {
                Some(r) => r,
                None => return,
            };

            check_receiver_gap_before_brackets(
                self,
                source,
                receiver.location().end_offset(),
                call.opening_loc().map(|loc| loc.start_offset()),
                diagnostics,
                &mut corrections,
            );
            return;
        }

        if let Some(write) = node.as_index_operator_write_node() {
            let receiver = match write.receiver() {
                Some(receiver) => receiver,
                None => return,
            };
            check_receiver_gap_before_brackets(
                self,
                source,
                receiver.location().end_offset(),
                Some(write.opening_loc().start_offset()),
                diagnostics,
                &mut corrections,
            );
            return;
        }

        if let Some(write) = node.as_index_and_write_node() {
            let receiver = match write.receiver() {
                Some(receiver) => receiver,
                None => return,
            };
            check_receiver_gap_before_brackets(
                self,
                source,
                receiver.location().end_offset(),
                Some(write.opening_loc().start_offset()),
                diagnostics,
                &mut corrections,
            );
            return;
        }

        if let Some(write) = node.as_index_or_write_node() {
            let receiver = match write.receiver() {
                Some(receiver) => receiver,
                None => return,
            };
            check_receiver_gap_before_brackets(
                self,
                source,
                receiver.location().end_offset(),
                Some(write.opening_loc().start_offset()),
                diagnostics,
                &mut corrections,
            );
            return;
        }

        if let Some(write) = node.as_call_operator_write_node() {
            if write.read_name().as_slice() != b"[]" || write.call_operator_loc().is_some() {
                return;
            }
            let receiver = match write.receiver() {
                Some(receiver) => receiver,
                None => return,
            };
            check_receiver_gap_before_scanned_brackets(
                self,
                source,
                receiver.location().end_offset(),
                write.location().end_offset(),
                diagnostics,
                &mut corrections,
            );
            return;
        }

        if let Some(write) = node.as_call_and_write_node() {
            if write.read_name().as_slice() != b"[]" || write.call_operator_loc().is_some() {
                return;
            }
            let receiver = match write.receiver() {
                Some(receiver) => receiver,
                None => return,
            };
            check_receiver_gap_before_scanned_brackets(
                self,
                source,
                receiver.location().end_offset(),
                write.location().end_offset(),
                diagnostics,
                &mut corrections,
            );
            return;
        }

        if let Some(write) = node.as_call_or_write_node() {
            if write.read_name().as_slice() != b"[]" || write.call_operator_loc().is_some() {
                return;
            }
            let receiver = match write.receiver() {
                Some(receiver) => receiver,
                None => return,
            };
            check_receiver_gap_before_scanned_brackets(
                self,
                source,
                receiver.location().end_offset(),
                write.location().end_offset(),
                diagnostics,
                &mut corrections,
            );
        }
    }
}

fn check_receiver_gap_before_brackets(
    cop: &dyn Cop,
    source: &SourceFile,
    receiver_end: usize,
    selector_start: Option<usize>,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let Some(selector_start) = selector_start else {
        return;
    };

    if receiver_end >= selector_start {
        return;
    }

    let bytes = source.as_bytes();
    let gap = &bytes[receiver_end..selector_start];
    if !gap.iter().all(|&b| b == b' ' || b == b'\t') {
        return;
    }

    let (line, col) = source.offset_to_line_col(receiver_end);
    let mut diagnostic = cop.diagnostic(source, line, col, MESSAGE.to_string());
    if let Some(corrections) = corrections.as_mut() {
        corrections.push(Correction {
            start: receiver_end,
            end: selector_start,
            replacement: String::new(),
            cop_name: cop.name(),
            cop_index: 0,
        });
        diagnostic.corrected = true;
    }
    diagnostics.push(diagnostic);
}

fn check_receiver_gap_before_scanned_brackets(
    cop: &dyn Cop,
    source: &SourceFile,
    receiver_end: usize,
    node_end: usize,
    diagnostics: &mut Vec<Diagnostic>,
    corrections: &mut Option<&mut Vec<Correction>>,
) {
    let bytes = source.as_bytes();
    let selector_start = bytes[receiver_end..node_end]
        .iter()
        .position(|&byte| byte == b'[')
        .map(|offset| receiver_end + offset);
    check_receiver_gap_before_brackets(
        cop,
        source,
        receiver_end,
        selector_start,
        diagnostics,
        corrections,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(SpaceBeforeBrackets, "cops/layout/space_before_brackets");
    crate::cop_autocorrect_fixture_tests!(SpaceBeforeBrackets, "cops/layout/space_before_brackets");

    #[test]
    fn index_operator_write_offense() {
        let source = b"value = nil\nvalue [0] += 1\n";
        let diagnostics = crate::testutil::run_cop_full(&SpaceBeforeBrackets, source);
        assert_eq!(
            diagnostics.len(),
            1,
            "Expected one offense: {diagnostics:?}"
        );
    }

    #[test]
    fn autocorrect_index_operator_write_offense() {
        let source = b"value = nil\nvalue [0] += 1\n";
        let (_diagnostics, corrections) =
            crate::testutil::run_cop_autocorrect(&SpaceBeforeBrackets, source);
        let corrected = crate::correction::CorrectionSet::from_vec(corrections).apply(source);
        assert_eq!(corrected, b"value = nil\nvalue[0] += 1\n");
    }
}
