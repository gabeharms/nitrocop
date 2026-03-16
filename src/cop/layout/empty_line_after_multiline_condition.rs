use crate::cop::node_type::{CASE_NODE, IF_NODE, UNLESS_NODE, UNTIL_NODE, WHILE_NODE};
use crate::cop::util::is_blank_or_whitespace_line;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

/// Enforces empty line after multiline condition.
///
/// ## Corpus investigation (227 FP, 801 FN)
///
/// **FP root causes (round 1):**
/// - Modifier if/unless/while/until at last position (no right sibling) were
///   being flagged. RuboCop only flags modifier forms when there's a subsequent
///   statement (`right_sibling`). Without AST parent pointers, we approximate
///   by scanning for the next non-blank line after the condition and checking
///   if it looks like a continuation statement.
/// - Multiline check compared keyword line vs predicate end line, but RuboCop's
///   `condition.multiline?` compares the predicate's own first_line vs last_line.
///   This caused FPs when `if`/`unless` is at end of line with a single-line
///   predicate on the next line (e.g., `raise ... if\n  cond`). Fixed by comparing
///   predicate start line vs end line instead.
///
/// **FP root causes (round 2, 39 FPs):**
/// - Used `is_blank_line` which only treats empty lines as blank; RuboCop's
///   `blank?` also treats whitespace-only lines as blank. Fixed by switching to
///   `is_blank_or_whitespace_line`.
/// - `elsif case ...` patterns: when the predicate of if/elsif is a CaseNode,
///   the multiline nature comes from the case structure, not a simple boolean
///   condition. RuboCop may not flag these. Fixed by skipping when predicate
///   is a CaseNode.
/// - `has_right_sibling` heuristic was too aggressive: treated comment lines
///   as right siblings, and didn't recognize `when` as a structural keyword.
///   Fixed by skipping comment lines and adding `when` to the structural
///   keyword list.
///
/// **FP root causes (round 3, 21 FP → 0 FP):**
/// - Offense location placed at keyword (`if`/`unless`/`elsif`) instead of
///   condition node. When keyword is at end of line and condition starts on
///   next line, this creates FP on keyword line + FN on condition line.
///   Fixed by reporting offense at predicate start, matching RuboCop's
///   `add_offense(condition)`.
/// - `BlockNode.multiline?` override in rubocop-ast: when condition is a
///   block call (e.g., `.all? { ... }`), RuboCop checks whether the block
///   delimiters (`{`/`}` or `do`/`end`) span multiple lines, not the full
///   expression. A multiline method chain with single-line `{ }` block is
///   NOT considered multiline. Fixed by checking block delimiter lines when
///   predicate is a CallNode with a block argument.
///
/// **FN root causes (round 1):**
/// - Missing `case/when` support: multiline when conditions need an empty line
///   after the last condition before the body.
/// - Missing `rescue` support: multiline rescue exception lists need an empty
///   line after the last exception before the handler body.
/// - Message format mismatch: RuboCop uses "Use empty line after multiline condition."
///   (no "an"), the old message had "an".
///
/// **FN root causes (round 2, 21 FN → ~10 FN):**
/// - `expr while cond` was treated as modifier (check right_sibling), but
///   Parser gem treats it as regular `while` (always check). Only
///   `begin...end while cond` is `while_post` (check right_sibling). Fixed
///   by using Prism's `is_begin_modifier()` flag instead of `closing_loc().is_none()`.
///
/// **FN root causes (round 3, 12 FN → 0 FN):**
/// - `has_right_sibling` treated `else`/`elsif`/`rescue`/`ensure` as scope
///   terminators (returning false), but in RuboCop's Parser AST, when a modifier
///   if/unless IS the direct body of an outer `if` node (single-statement body),
///   `right_sibling` returns the else/elsif body as the next child. Similarly,
///   `rescue`/`ensure` in a `begin` block are sibling positions. Fixed by removing
///   `else`/`elsif`/`rescue`/`ensure` from the terminator list, keeping only `end`,
///   `}`, and `when` as true scope-closers.
///
/// **Remaining FP (2 in camping, unfixable):**
/// - Both FPs are in `camping__camping__f2479aa` — heavily minified Ruby with
///   semicolons and code crammed on single lines. These are edge cases where
///   text-based heuristics cannot accurately determine scope boundaries.
pub struct EmptyLineAfterMultilineCondition;

impl Cop for EmptyLineAfterMultilineCondition {
    fn name(&self) -> &'static str {
        "Layout/EmptyLineAfterMultilineCondition"
    }

    fn default_enabled(&self) -> bool {
        false
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[IF_NODE, UNLESS_NODE, UNTIL_NODE, WHILE_NODE, CASE_NODE]
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::parse::codemap::CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // RescueNode is not dispatched via visit_branch_node_enter in Prism's
        // visitor, so check_node never sees it. Use a dedicated visitor here.
        let mut visitor = RescueVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
        };
        visitor.visit(&parse_result.node());
        diagnostics.append(&mut visitor.diagnostics);
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
        // Check if/unless nodes
        if let Some(if_node) = node.as_if_node() {
            let kw_loc = match if_node.if_keyword_loc() {
                Some(loc) => loc,
                None => return,
            };
            let kw_slice = kw_loc.as_slice();
            if kw_slice != b"if" && kw_slice != b"unless" && kw_slice != b"elsif" {
                return;
            }

            // Skip ternary (no end keyword, but has `?` then keyword)
            let is_ternary = if_node.end_keyword_loc().is_none()
                && if_node
                    .then_keyword_loc()
                    .is_some_and(|t| t.as_slice() == b"?");
            if is_ternary {
                return;
            }

            // Modifier form: no end keyword (ternary already excluded above)
            let is_modifier = if_node.end_keyword_loc().is_none();

            if is_modifier {
                // For modifier forms, only flag if there's a right sibling.
                let predicate = if_node.predicate();
                let pred_end = predicate.location().end_offset().saturating_sub(1);
                let (pred_end_line, _) = source.offset_to_line_col(pred_end);
                if has_right_sibling(source, pred_end_line) {
                    diagnostics.extend(self.check_multiline_condition(source, &predicate));
                }
            } else {
                let predicate = if_node.predicate();
                diagnostics.extend(self.check_multiline_condition(source, &predicate));
            }
            return;
        }

        // Check unless nodes (Prism has a separate UnlessNode)
        if let Some(unless_node) = node.as_unless_node() {
            let kw_loc = unless_node.keyword_loc();
            if kw_loc.as_slice() != b"unless" {
                return;
            }
            let is_modifier = unless_node.end_keyword_loc().is_none();
            let predicate = unless_node.predicate();
            if is_modifier {
                let pred_end = predicate.location().end_offset().saturating_sub(1);
                let (pred_end_line, _) = source.offset_to_line_col(pred_end);
                if has_right_sibling(source, pred_end_line) {
                    diagnostics.extend(self.check_multiline_condition(source, &predicate));
                }
            } else {
                diagnostics.extend(self.check_multiline_condition(source, &predicate));
            }
            return;
        }

        // Check while nodes
        if let Some(while_node) = node.as_while_node() {
            let kw_loc = while_node.keyword_loc();
            if kw_loc.as_slice() != b"while" {
                return;
            }
            let predicate = while_node.predicate();
            // In RuboCop: `on_while` always checks (block and `expr while cond`),
            // only `on_while_post` (`begin...end while cond`) checks right_sibling.
            // Prism's `is_begin_modifier()` distinguishes the post form.
            let is_begin_modifier =
                while_node.closing_loc().is_none() && while_node.is_begin_modifier();
            if is_begin_modifier {
                let pred_end = predicate.location().end_offset().saturating_sub(1);
                let (pred_end_line, _) = source.offset_to_line_col(pred_end);
                if has_right_sibling(source, pred_end_line) {
                    diagnostics.extend(self.check_multiline_condition(source, &predicate));
                }
            } else {
                diagnostics.extend(self.check_multiline_condition(source, &predicate));
            }
            return;
        }

        // Check until nodes
        if let Some(until_node) = node.as_until_node() {
            let kw_loc = until_node.keyword_loc();
            if kw_loc.as_slice() != b"until" {
                return;
            }
            let predicate = until_node.predicate();
            // Same as while: only begin...end until form checks right_sibling
            let is_begin_modifier =
                until_node.closing_loc().is_none() && until_node.is_begin_modifier();
            if is_begin_modifier {
                let pred_end = predicate.location().end_offset().saturating_sub(1);
                let (pred_end_line, _) = source.offset_to_line_col(pred_end);
                if has_right_sibling(source, pred_end_line) {
                    diagnostics.extend(self.check_multiline_condition(source, &predicate));
                }
            } else {
                diagnostics.extend(self.check_multiline_condition(source, &predicate));
            }
            return;
        }

        // Check case/when nodes
        if let Some(case_node) = node.as_case_node() {
            for condition in case_node.conditions().iter() {
                if let Some(when_node) = condition.as_when_node() {
                    let conditions = when_node.conditions();
                    if conditions.is_empty() {
                        continue;
                    }
                    let first = conditions.iter().next().unwrap();
                    let last = conditions.iter().last().unwrap();
                    let (first_line, _) =
                        source.offset_to_line_col(first.location().start_offset());
                    let last_end = last.location().end_offset().saturating_sub(1);
                    let (last_line, _) = source.offset_to_line_col(last_end);

                    // Only check multiline when conditions
                    if first_line == last_line {
                        continue;
                    }

                    let lines: Vec<&[u8]> = source.lines().collect();
                    let next_line_num = last_line + 1;
                    if next_line_num > lines.len() {
                        continue;
                    }
                    let next_line = lines[next_line_num - 1];
                    if !is_blank_or_whitespace_line(next_line) {
                        let when_kw_loc = when_node.keyword_loc();
                        let (line, col) = source.offset_to_line_col(when_kw_loc.start_offset());
                        diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
                    }
                }
            }
        }
    }
}

const MSG: &str = "Use empty line after multiline condition.";

/// Check if there's a non-blank statement-like line after the given line.
/// This approximates RuboCop's `right_sibling` check for modifier forms.
///
/// In RuboCop's AST (Parser gem), `right_sibling` returns the next child of
/// the parent node. For a modifier if/unless that is the direct body of an
/// outer `if` node (single-statement body), the parent is the outer `if` and
/// `right_sibling` returns the else/elsif body. Similarly, in a `begin` block,
/// `rescue`/`ensure` are sibling positions. So `else`, `elsif`, `rescue`, and
/// `ensure` keywords indicate a right sibling exists.
///
/// Only `end`, `}`, and `when` are true terminators (scope-closing or
/// case-branch boundaries where the modifier's parent is a `when` body).
fn has_right_sibling(source: &SourceFile, condition_end_line: usize) -> bool {
    let lines: Vec<&[u8]> = source.lines().collect();
    // Look at lines after the condition end
    for line in lines.iter().skip(condition_end_line) {
        if is_blank_or_whitespace_line(line) {
            continue;
        }
        let trimmed = line.iter().position(|&b| b != b' ' && b != b'\t');
        if let Some(pos) = trimmed {
            let rest = &line[pos..];
            // Skip comment lines — comments are not AST siblings
            if rest.starts_with(b"#") {
                continue;
            }
            // `end` and `}` close the parent scope — no right sibling
            if rest == b"end"
                || rest.starts_with(b"end ")
                || rest.starts_with(b"end\t")
                || rest.starts_with(b"end.")
                || rest.starts_with(b"end)")
                || rest == b"}"
            {
                return false;
            }
            // `when` is a case-branch boundary — the modifier's parent is
            // the when body, and the next when is NOT a right sibling of
            // the modifier node
            if rest.starts_with(b"when ") || rest.starts_with(b"when\n") || rest == b"when" {
                return false;
            }
            // `else`, `elsif`, `rescue`, `ensure` indicate that the modifier
            // is the body of a branching construct (if/begin), and the next
            // branch is a right sibling in the AST → fire
            // All other lines are also right siblings → fire
            return true;
        }
    }
    false
}

/// Check if a predicate node represents a block call where the block delimiters
/// are on the same line. In RuboCop, `BlockNode.multiline?` checks `loc.begin.line
/// == loc.end.line` (the `{`/`}` or `do`/`end`), NOT the full expression range.
/// This means a multiline method chain with single-line `{ }` block (e.g.,
/// `items\n  .all? { |x| x.valid? }`) is NOT considered multiline.
fn is_single_line_block_condition(source: &SourceFile, predicate: &ruby_prism::Node<'_>) -> bool {
    // Check if the predicate is a CallNode with a block
    if let Some(call_node) = predicate.as_call_node() {
        if let Some(block) = call_node.block() {
            if let Some(block_node) = block.as_block_node() {
                let open_loc = block_node.opening_loc();
                let close_loc = block_node.closing_loc();
                let (open_line, _) = source.offset_to_line_col(open_loc.start_offset());
                let (close_line, _) = source.offset_to_line_col(close_loc.start_offset());
                return open_line == close_line;
            }
        }
    }
    false
}

/// Visitor that handles RescueNode (which Prism dispatches via visit_rescue_node,
/// not visit_branch_node_enter, so the CopWalker never sees it).
struct RescueVisitor<'a> {
    cop: &'a EmptyLineAfterMultilineCondition,
    source: &'a SourceFile,
    diagnostics: Vec<Diagnostic>,
}

impl<'pr> Visit<'pr> for RescueVisitor<'_> {
    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode<'pr>) {
        self.cop
            .check_rescue_node(self.source, node, &mut self.diagnostics);
        // Continue visiting for chained rescue clauses
        ruby_prism::visit_rescue_node(self, node);
    }
}

impl EmptyLineAfterMultilineCondition {
    fn check_multiline_condition(
        &self,
        source: &SourceFile,
        predicate: &ruby_prism::Node<'_>,
    ) -> Vec<Diagnostic> {
        // Skip when the predicate is a CaseNode — case expressions are inherently
        // multiline (they contain when branches) and shouldn't be treated as
        // multiline boolean conditions. This matches RuboCop's behavior for
        // patterns like `elsif case states.last when :initial ...`.
        if predicate.as_case_node().is_some() || predicate.as_case_match_node().is_some() {
            return Vec::new();
        }

        let (pred_start_line, _) = source.offset_to_line_col(predicate.location().start_offset());
        let pred_end = predicate.location().end_offset().saturating_sub(1);
        let (pred_end_line, _) = source.offset_to_line_col(pred_end);

        // Only check multiline conditions — compare predicate's own start vs end line,
        // matching RuboCop's `condition.multiline?` which checks first_line vs last_line.
        if pred_start_line == pred_end_line {
            return Vec::new();
        }

        // If the condition is a block call with single-line delimiters, it's not
        // multiline per RuboCop's BlockNode.multiline? override.
        if is_single_line_block_condition(source, predicate) {
            return Vec::new();
        }

        let lines: Vec<&[u8]> = source.lines().collect();
        // The line after the condition ends
        let next_line_num = pred_end_line + 1;
        if next_line_num > lines.len() {
            return Vec::new();
        }

        let next_line = lines[next_line_num - 1];
        // Use is_blank_or_whitespace_line to match RuboCop's `blank?` which treats
        // whitespace-only lines as blank.
        if !is_blank_or_whitespace_line(next_line) {
            // Report offense at the condition (predicate) start, matching RuboCop's
            // `add_offense(condition)` which places the offense on the condition node.
            let (line, col) = source.offset_to_line_col(predicate.location().start_offset());
            return vec![self.diagnostic(source, line, col, MSG.to_string())];
        }

        Vec::new()
    }

    fn check_rescue_node(
        &self,
        source: &SourceFile,
        rescue_node: &ruby_prism::RescueNode<'_>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let exceptions = rescue_node.exceptions();
        if exceptions.len() <= 1 {
            return;
        }

        let first = exceptions.iter().next().unwrap();
        let last = exceptions.iter().last().unwrap();
        let (first_line, _) = source.offset_to_line_col(first.location().start_offset());
        let last_end = last.location().end_offset().saturating_sub(1);
        let (last_line, _) = source.offset_to_line_col(last_end);

        if first_line == last_line {
            return;
        }

        let lines: Vec<&[u8]> = source.lines().collect();
        let next_line_num = last_line + 1;
        if next_line_num > lines.len() {
            return;
        }

        let next_line = lines[next_line_num - 1];
        if !is_blank_or_whitespace_line(next_line) {
            let kw_loc = rescue_node.keyword_loc();
            let (line, col) = source.offset_to_line_col(kw_loc.start_offset());
            diagnostics.push(self.diagnostic(source, line, col, MSG.to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(
        EmptyLineAfterMultilineCondition,
        "cops/layout/empty_line_after_multiline_condition"
    );

    #[test]
    fn unless_multiline_condition() {
        let source = b"unless foo &&\n       bar\n  do_something\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert_eq!(diags.len(), 1, "Expected one offense for unless");
    }

    #[test]
    fn elsif_multiline_condition() {
        let source =
            b"if condition\n  do_something\nelsif multiline &&\n   condition\n  do_something_else\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert_eq!(diags.len(), 1, "Expected one offense for elsif");
    }

    #[test]
    fn rescue_multiline_exceptions() {
        let source = b"begin\n  do_something\nrescue FooError,\n  BarError\n  handle_error\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert_eq!(diags.len(), 1, "Expected one offense for rescue");
    }

    #[test]
    fn case_when_multiline_condition() {
        let source = b"case x\nwhen foo,\n    bar\n  do_something\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert_eq!(diags.len(), 1, "Expected one offense for case/when");
    }

    #[test]
    fn modifier_if_no_right_sibling() {
        let source = b"def m\n  do_something if multiline &&\n                condition\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert!(
            diags.is_empty(),
            "No offense when modifier if has no right sibling"
        );
    }

    #[test]
    fn fp_modifier_if_only_comment_after() {
        // Modifier if with multiline condition, only a comment follows (no real right sibling)
        let source = b"def m\n  true if depth >= 3 &&\n          caller.first.label == name\n          # TODO: incomplete\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert!(
            diags.is_empty(),
            "Should not fire when only comment follows modifier if: {:?}",
            diags
        );
    }

    #[test]
    fn fp_next_if_multiline_at_end_of_block() {
        // next if with multiline condition at end of block
        let source =
            b"items.each do |l|\n  next if\n    # comment\n    l == :foo ||\n    l == :bar\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert!(
            diags.is_empty(),
            "Should not fire on next if at end of block: {:?}",
            diags
        );
    }

    #[test]
    fn fp_elsif_case_as_predicate() {
        // elsif with case expression as predicate - the case is multiline by nature
        // but RuboCop doesn't flag this
        let source = b"if x\n  foo\nelsif case states.last\n      when :initial, :media\n        scan(/foo/)\n      end\n  bar\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert!(
            diags.is_empty(),
            "Should not fire on elsif with case as predicate: {:?}",
            diags
        );
    }

    #[test]
    fn fp_whitespace_only_blank_line() {
        // Block if with whitespace-only line after condition (treated as blank by RuboCop)
        let source = b"if foo &&\n   bar\n    \n  do_something\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert!(
            diags.is_empty(),
            "Should not fire when whitespace-only line follows condition: {:?}",
            diags
        );
    }

    #[test]
    fn fp_modifier_unless_before_when() {
        // Modifier unless inside when block — next when is not a right sibling
        let source = b"case parent\nwhen Step\n  return render_403 unless can_read?(proto) ||\n                           can_write?(proto)\nwhen Result\n  return render_403 unless can_read_result?(parent)\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert!(
            diags.is_empty(),
            "Should not fire on modifier unless before when: {:?}",
            diags
        );
    }

    #[test]
    fn fp_unless_with_single_line_block_condition() {
        // unless with method chain on next line — block { } is single-line,
        // so condition is NOT multiline per RuboCop's BlockNode.multiline?
        let source = b"def m\n  unless %w[foo bar baz]\n      .all? { |name| File.exist? File.join(path, name) }\n    run(\"command\")\n  end\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert!(
            diags.is_empty(),
            "Should not fire on unless with single-line block condition: {:?}",
            diags
        );
    }

    #[test]
    fn fn_modifier_while_non_begin_form() {
        // `nil while code.gsub!(...)` — non-begin modifier while with multiline condition.
        // RuboCop treats this as regular `while` (always check), not `while_post`.
        let source = b"nil while\n    code.gsub!(/pat/) {\n      result\n    }\ndo_something\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert_eq!(
            diags.len(),
            1,
            "Should fire on non-begin modifier while with multiline condition: {:?}",
            diags
        );
    }

    #[test]
    fn no_offense_modifier_while_non_begin_at_end() {
        // nil while with multiline condition but no right sibling — RuboCop's on_while
        // always checks, so this IS an offense if the condition is multiline. But here
        // `code.gsub!() { }` has single-line block braces, so condition is NOT multiline.
        let source =
            b"def optimize(code)\n  code = code.dup\n  nil while\n    code.gsub!(/pattern/) { |f| f.upcase }\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert!(
            diags.is_empty(),
            "Should not fire on modifier while with single-line block condition at end: {:?}",
            diags
        );
    }

    #[test]
    fn offense_if_with_multiline_do_end_block() {
        // if with do..end block condition — block delimiters on different lines → multiline
        let source = b"if items.find do |item|\n     item.ready?\n   end\n  process\nend\n";
        let diags = crate::testutil::run_cop_full(&EmptyLineAfterMultilineCondition, source);
        assert_eq!(
            diags.len(),
            1,
            "Should fire on if with multiline do..end block condition: {:?}",
            diags
        );
    }
}
