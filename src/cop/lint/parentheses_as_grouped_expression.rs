use crate::cop::node_type::CALL_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Checks for space between the name of a called method and a left parenthesis.
///
/// ## Root cause analysis (corpus: 39 FP, 668 FN at 46.2% match)
///
/// **FN root cause:** The `call_end > paren_end` check was meant to exclude
/// chained calls like `func (x).bar`, but also incorrectly excluded calls with
/// blocks like `func (x) { block }`. In Prism, chaining already causes the
/// first argument to NOT be a ParenthesesNode (Prism folds the chain into the
/// argument), so the `call_end > paren_end` check was both redundant for chains
/// and harmful for blocks. Additionally, the source-text based
/// `has_trailing_operator_or_chain` check was redundant — Prism already handles
/// operators/chains by incorporating them into the argument structure (making
/// `as_parentheses_node()` return None).
///
/// **FP root cause:** The source-text based trailing checks (for operators,
/// chains, hash rockets, ternaries) were incomplete. After removing them, we
/// rely purely on Prism's AST which correctly represents these structures by
/// NOT wrapping the argument in ParenthesesNode when post-paren operators are
/// present.
///
/// **Fix:** Simplified to a pure AST-based approach: check for CallNode with
/// no `opening_loc`, exactly one ParenthesesNode argument, and whitespace
/// between method name and paren. Removed redundant source-text trailing checks.
/// Removed `call_end > paren_end` to fix block FN. Kept compound range
/// exclusion (checks inside ParenthesesNode body).
pub struct ParenthesesAsGroupedExpression;

impl Cop for ParenthesesAsGroupedExpression {
    fn name(&self) -> &'static str {
        "Lint/ParenthesesAsGroupedExpression"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
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

        // Must NOT have opening_loc — when there's a space before the paren,
        // Prism treats the parens as grouping (no call-level parens), so
        // opening_loc is None.
        if call.opening_loc().is_some() {
            return;
        }

        // Must have a method name
        let msg_loc = match call.message_loc() {
            Some(loc) => loc,
            None => return,
        };

        let method_name = call.name().as_slice();

        // Skip operator methods (%, +, -, ==, etc.)
        if is_operator(method_name) {
            return;
        }

        // Skip setter methods (foo=)
        if method_name.ends_with(b"=") && method_name != b"==" && method_name != b"!=" {
            return;
        }

        let arguments = match call.arguments() {
            Some(a) => a,
            None => return,
        };

        let args = arguments.arguments();

        // Must have exactly one argument (the parenthesized expression)
        if args.len() != 1 {
            return;
        }

        let first_arg = args.iter().next().unwrap();

        // The argument must be a ParenthesesNode.
        // When Prism sees `func (x)` with space, it wraps `x` in ParenthesesNode.
        // For `func (x).bar` or `func (x) + 1`, Prism folds the chain/operator
        // into the argument, so first_arg is NOT a ParenthesesNode — those cases
        // are correctly excluded by this check.
        let paren_node = match first_arg.as_parentheses_node() {
            Some(p) => p,
            None => return,
        };

        // There must be whitespace between method name end and the `(` of the ParenthesesNode
        let msg_end = msg_loc.end_offset();
        let paren_start = paren_node.location().start_offset();

        if paren_start <= msg_end {
            return;
        }

        let between = &source.as_bytes()[msg_end..paren_start];
        if between.is_empty() || !between.iter().all(|&b| b == b' ' || b == b'\t') {
            return;
        }

        // Check for compound range inside the parens: `rand (a - b)..(c - d)`
        // Simple ranges like `rand (1..10)` ARE offenses, but compound ranges
        // where sub-expressions are calls or parenthesized are NOT.
        if let Some(body) = paren_node.body() {
            if let Some(stmts) = body.as_statements_node() {
                let inner = stmts.body();
                if inner.len() == 1 {
                    let expr = inner.iter().next().unwrap();
                    if let Some(range) = expr.as_range_node() {
                        let is_compound = |n: &ruby_prism::Node<'_>| -> bool {
                            n.as_call_node().is_some() || n.as_parentheses_node().is_some()
                        };
                        let left_compound = range.left().map(|l| is_compound(&l)).unwrap_or(false);
                        let right_compound =
                            range.right().map(|r| is_compound(&r)).unwrap_or(false);
                        if left_compound || right_compound {
                            return;
                        }
                    }
                }
            }
        }

        // Build the argument text for the message
        let paren_end = paren_node.location().end_offset();
        let arg_text = source.byte_slice(paren_start, paren_end, "(...)");

        let (line, column) = source.offset_to_line_col(paren_start);
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            format!("`{}` interpreted as grouped expression.", arg_text),
        ));
    }
}

fn is_operator(name: &[u8]) -> bool {
    matches!(
        name,
        b"=="
            | b"!="
            | b"<"
            | b">"
            | b"<="
            | b">="
            | b"<=>"
            | b"+"
            | b"-"
            | b"*"
            | b"/"
            | b"%"
            | b"**"
            | b"&"
            | b"|"
            | b"^"
            | b"~"
            | b"<<"
            | b">>"
            | b"[]"
            | b"[]="
            | b"=~"
            | b"!~"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        ParenthesesAsGroupedExpression,
        "cops/lint/parentheses_as_grouped_expression"
    );

    #[test]
    fn corpus_fn_patterns() {
        // Test patterns from corpus FN analysis - symbol arguments, blocks
        let cop = ParenthesesAsGroupedExpression;
        let test_cases: &[(&[u8], usize)] = &[
            // Common FN patterns: method (:symbol)
            (b"method (:symbol)\n", 1),
            (b"method ( :all )\n", 1),
            // Inside blocks (common in RSpec)
            (b"describe do\n  subject (:all)\nend\n", 1),
            // Assignment in parens
            (b"method (var = expr)\n", 1),
            // Method call with block (was FN due to call_end > paren_end)
            (b"func (x) { |y| y }\n", 1),
            (b"func (x) do |y| y end\n", 1),
            // Should NOT detect
            (b"method(:symbol)\n", 0),
            (b"method (x).bar\n", 0),
            (b"method (x) || y\n", 0),
            (b"method (x) + 1\n", 0),
            (b"puts (2 + 3) * 4\n", 0),
        ];
        for (src, expected_count) in test_cases {
            let diagnostics = crate::testutil::run_cop_full(&cop, src);
            if diagnostics.len() != *expected_count {
                panic!(
                    "For {:?}: expected {} offenses, got {} ({:?})",
                    std::str::from_utf8(src).unwrap().trim(),
                    expected_count,
                    diagnostics.len(),
                    diagnostics,
                );
            }
        }
    }
}
