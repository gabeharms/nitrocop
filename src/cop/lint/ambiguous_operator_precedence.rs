use crate::cop::node_type::{AND_NODE, CALL_NODE, OR_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/AmbiguousOperatorPrecedence: detects expressions with mixed operator
/// precedence that lack parentheses.
///
/// Investigation notes:
/// - Original implementation only handled `||`/`&&` mixing and arithmetic-only
///   mixing. Missed the cross-category case: `||`/`&&` mixed with arithmetic
///   operators (e.g., `a || b + c`, `a << b || c`, `a && b | c`).
/// - RuboCop treats `&&` and `||` as the two lowest levels in a unified
///   precedence table (indices 6 and 7). Its `on_send` checks if a send node's
///   parent is also an operator with lower precedence.
/// - In Prism, `||`/`&&` produce `OrNode`/`AndNode` (not `CallNode`), so we
///   check from the parent side: when visiting `OrNode`/`AndNode`, flag any
///   `CallNode` children that are arithmetic/bitwise operators.
pub struct AmbiguousOperatorPrecedence;

// Precedence levels (lower index = higher precedence).
// Indices 0-5 are arithmetic/bitwise (represented as CallNode in Prism).
// Indices 6-7 are logical (represented as AndNode/OrNode in Prism).
const PRECEDENCE: &[&[&[u8]]] = &[
    &[b"**"],
    &[b"*", b"/", b"%"],
    &[b"+", b"-"],
    &[b"<<", b">>"],
    &[b"&"],
    &[b"|", b"^"],
    // && is index 6 (AndNode in Prism)
    // || is index 7 (OrNode in Prism)
];

const AND_PREC: usize = 6;
const OR_PREC: usize = 7;

fn precedence_level(op: &[u8]) -> Option<usize> {
    for (i, group) in PRECEDENCE.iter().enumerate() {
        if group.contains(&op) {
            return Some(i);
        }
    }
    None
}

const MSG: &str = "Wrap expressions with varying precedence with parentheses to avoid ambiguity.";

impl Cop for AmbiguousOperatorPrecedence {
    fn name(&self) -> &'static str {
        "Lint/AmbiguousOperatorPrecedence"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[AND_NODE, CALL_NODE, OR_NODE]
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
        if let Some(or_node) = node.as_or_node() {
            // Skip keyword `or` — only symbolic `||` triggers this cop
            if or_node.operator_loc().as_slice() == b"||" {
                self.check_logical_children(
                    source,
                    or_node.left(),
                    or_node.right(),
                    OR_PREC,
                    diagnostics,
                );
            }
            return;
        }

        if let Some(and_node) = node.as_and_node() {
            // Skip keyword `and` — only symbolic `&&` triggers this cop
            if and_node.operator_loc().as_slice() == b"&&" {
                self.check_logical_children(
                    source,
                    and_node.left(),
                    and_node.right(),
                    AND_PREC,
                    diagnostics,
                );
            }
            return;
        }

        // Handle arithmetic/bitwise CallNode with CallNode children of different precedence
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        let method = call.name().as_slice();
        let outer_prec = match precedence_level(method) {
            Some(p) => p,
            None => return,
        };

        // Check arguments for higher-precedence operators
        // e.g., `a + b * c`: outer is `+` (prec 2), arg `b * c` is `*` (prec 1)
        if let Some(args) = call.arguments() {
            for arg in args.arguments().iter() {
                if let Some(arg_call) = arg.as_call_node() {
                    let arg_method = arg_call.name().as_slice();
                    if let Some(arg_prec) = precedence_level(arg_method) {
                        if arg_prec < outer_prec {
                            let loc = arg_call.location();
                            let (line, column) = source.offset_to_line_col(loc.start_offset());
                            diagnostics.push(self.diagnostic(
                                source,
                                line,
                                column,
                                MSG.to_string(),
                            ));
                        }
                    }
                }
            }
        }

        // Check if receiver is a higher-precedence operator
        // e.g., `a ** b + c`: outer is `+` (prec 2), recv `a ** b` is `**` (prec 0)
        if let Some(recv) = call.receiver() {
            if let Some(recv_call) = recv.as_call_node() {
                let recv_method = recv_call.name().as_slice();
                if let Some(recv_prec) = precedence_level(recv_method) {
                    if recv_prec < outer_prec {
                        let loc = recv_call.location();
                        let (line, column) = source.offset_to_line_col(loc.start_offset());
                        diagnostics.push(self.diagnostic(source, line, column, MSG.to_string()));
                    }
                }
            }
        }
    }
}

impl AmbiguousOperatorPrecedence {
    /// Check children of an OrNode or AndNode for higher-precedence operators.
    /// `parent_prec` is OR_PREC (7) for OrNode, AND_PREC (6) for AndNode.
    fn check_logical_children(
        &self,
        source: &SourceFile,
        left: ruby_prism::Node<'_>,
        right: ruby_prism::Node<'_>,
        parent_prec: usize,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for child in [left, right] {
            let child_prec = if child.as_and_node().is_some() {
                Some(AND_PREC)
            } else if let Some(call) = child.as_call_node() {
                precedence_level(call.name().as_slice())
            } else {
                None
            };

            if let Some(cp) = child_prec {
                if cp < parent_prec {
                    let loc = child.location();
                    let (line, column) = source.offset_to_line_col(loc.start_offset());
                    diagnostics.push(self.diagnostic(source, line, column, MSG.to_string()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        AmbiguousOperatorPrecedence,
        "cops/lint/ambiguous_operator_precedence"
    );
}
