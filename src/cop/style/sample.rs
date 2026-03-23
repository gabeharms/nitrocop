use crate::cop::node_type::CALL_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Detects `shuffle.first`, `shuffle.last`, `shuffle[0]`, `shuffle[-1]`,
/// `shuffle.at(0)`, `shuffle.at(-1)`, `shuffle.slice(0)`, `shuffle.slice(-1)`,
/// `shuffle[0, N]` (two-arg bracket with 0 start), and `shuffle[0..N]` /
/// `shuffle[0...N]` (range bracket starting at 0).
///
/// ## Investigation (2026-03-14)
/// Original implementation only handled `.first` and `.last` on shuffle calls.
/// Corpus FNs (4) were caused by missing detection of bracket access (`[]`),
/// `.at()`, and `.slice()` patterns with integer arguments 0 or -1.
/// Added handling for these patterns to match RuboCop's Style/Sample cop.
///
/// ## Investigation (2026-03-23)
/// Remaining 4 FNs were caused by missing detection of:
///
/// - `shuffle[0, N]` (two-arg bracket access where first arg is 0)
/// - `shuffle[0..N]` / `shuffle[0...N]` (range index starting at 0)
///
/// Added handling for both patterns to match RuboCop behavior.
pub struct Sample;

/// Compute the sample size from a range argument to `shuffle[]`.
///
/// Returns `Some(size)` for ranges starting at 0, `None` otherwise.
/// Inclusive range `0..N` yields `N + 1`, exclusive `0...N` yields `N`.
fn compute_range_sample_size(range_node: &ruby_prism::RangeNode<'_>) -> Option<usize> {
    // Left must be integer 0 (or absent, which means 0)
    let left_is_zero = match range_node.left() {
        Some(left) => left
            .as_integer_node()
            .is_some_and(|n| std::str::from_utf8(n.location().as_slice()).unwrap_or("") == "0"),
        None => true, // no left means 0
    };
    if !left_is_zero {
        return None;
    }

    // Right must be a non-negative integer
    let right = range_node.right()?;
    let right_int = right.as_integer_node()?;
    let right_str = std::str::from_utf8(right_int.location().as_slice()).unwrap_or("");
    let right_val: usize = right_str.parse().ok()?;

    // Determine inclusive vs exclusive by checking the operator source
    let op_loc = range_node.operator_loc();
    let op_src = std::str::from_utf8(
        &range_node.location().as_slice()[op_loc.start_offset()
            - range_node.location().start_offset()
            ..op_loc.end_offset() - range_node.location().start_offset()],
    )
    .unwrap_or("");

    if op_src == "..." {
        // Exclusive: 0...N → sample(N)
        Some(right_val)
    } else {
        // Inclusive: 0..N → sample(N + 1)
        Some(right_val + 1)
    }
}

impl Cop for Sample {
    fn name(&self) -> &'static str {
        "Style/Sample"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE]
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
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        let method_name = call.name();
        let method_bytes = method_name.as_slice();

        // Determine which pattern we're looking at
        enum ShufflePattern {
            FirstLast,   // .first / .last (no args or with count arg)
            IndexAccess, // [0] / [-1] — method name is `[]`
            AtOrSlice,   // .at(0) / .at(-1) / .slice(0) / .slice(-1)
        }

        // Result of validating index/at/slice arguments.
        enum SampleSize {
            Simple,           // sample (no arg) — for [0], [-1], .at(0), etc.
            WithSize(String), // sample(N) — for [0,N], [0..N], [0...N]
        }

        let pattern = match method_bytes {
            b"first" | b"last" => ShufflePattern::FirstLast,
            b"[]" => ShufflePattern::IndexAccess,
            b"at" | b"slice" => ShufflePattern::AtOrSlice,
            _ => return,
        };

        // For [] / at / slice, validate the arguments
        let sample_size = if matches!(
            pattern,
            ShufflePattern::IndexAccess | ShufflePattern::AtOrSlice
        ) {
            let args = match call.arguments() {
                Some(a) => a,
                None => return,
            };
            let arg_list: Vec<_> = args.arguments().iter().collect();

            match arg_list.len() {
                1 => {
                    let arg = &arg_list[0];
                    if let Some(int_node) = arg.as_integer_node() {
                        let val_str =
                            std::str::from_utf8(int_node.location().as_slice()).unwrap_or("");
                        if matches!(val_str, "0" | "-1") {
                            SampleSize::Simple
                        } else {
                            return;
                        }
                    } else if matches!(pattern, ShufflePattern::IndexAccess) {
                        // Check for range arg: [0..N] or [0...N]
                        if let Some(range_node) = arg.as_range_node() {
                            match compute_range_sample_size(&range_node) {
                                Some(size) => SampleSize::WithSize(size.to_string()),
                                None => return,
                            }
                        } else {
                            return;
                        }
                    } else {
                        return;
                    }
                }
                2 if matches!(pattern, ShufflePattern::IndexAccess) => {
                    // Two-arg bracket: [0, N] where first is int 0 and second is int
                    let first = &arg_list[0];
                    let second = &arg_list[1];
                    let first_is_zero = first.as_integer_node().is_some_and(|n| {
                        std::str::from_utf8(n.location().as_slice()).unwrap_or("") == "0"
                    });
                    if !first_is_zero {
                        return;
                    }
                    if let Some(int_node) = second.as_integer_node() {
                        let val = std::str::from_utf8(int_node.location().as_slice()).unwrap_or("");
                        SampleSize::WithSize(val.to_string())
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        } else {
            SampleSize::Simple
        };

        // Receiver must be a call to .shuffle
        let receiver = match call.receiver() {
            Some(r) => r,
            None => return,
        };

        if let Some(shuffle_call) = receiver.as_call_node() {
            if shuffle_call.name().as_slice() == b"shuffle" {
                // shuffle must have a receiver (the collection)
                if shuffle_call.receiver().is_none() {
                    return;
                }

                let loc = node.location();
                let incorrect = std::str::from_utf8(loc.as_slice()).unwrap_or("");
                let (line, column) = source.offset_to_line_col(loc.start_offset());

                // Determine the correct replacement
                let shuffle_arg_str = shuffle_call.arguments().map(|a| {
                    std::str::from_utf8(a.location().as_slice())
                        .unwrap_or("")
                        .to_string()
                });

                let correct = match (&sample_size, &pattern) {
                    (_, ShufflePattern::FirstLast) if call.arguments().is_some() => {
                        let arg_src = call
                            .arguments()
                            .map(|a| {
                                let args: Vec<_> = a.arguments().iter().collect();
                                if !args.is_empty() {
                                    std::str::from_utf8(args[0].location().as_slice())
                                        .unwrap_or("")
                                        .to_string()
                                } else {
                                    String::new()
                                }
                            })
                            .unwrap_or_default();

                        match &shuffle_arg_str {
                            Some(sa) => format!("sample({}, {})", arg_src, sa),
                            None => format!("sample({})", arg_src),
                        }
                    }
                    (SampleSize::WithSize(size), _) => match &shuffle_arg_str {
                        Some(sa) => format!("sample({}, {})", size, sa),
                        None => format!("sample({})", size),
                    },
                    _ => match &shuffle_arg_str {
                        Some(sa) => format!("sample({})", sa),
                        None => "sample".to_string(),
                    },
                };

                diagnostics.push(self.diagnostic(
                    source,
                    line,
                    column,
                    format!("Use `{}` instead of `{}`.", correct, incorrect),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(Sample, "cops/style/sample");
}
