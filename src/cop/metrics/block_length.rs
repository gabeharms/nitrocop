use crate::cop::node_type::{CALL_NODE, FORWARDING_SUPER_NODE, LAMBDA_NODE, SUPER_NODE};
use crate::cop::util::{collect_foldable_ranges, collect_heredoc_ranges, count_body_lines_ex};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-04)
///
/// Corpus oracle reported FP=105, FN=5.
///
/// A high-volume FP pattern was blocks whose body is only a heredoc expression:
/// `render do; <<~RUBY ... RUBY; end`.
///
/// In RuboCop (Parser AST), that body is a `str`/`dstr` node whose source range
/// is just the heredoc opening line (`<<~RUBY`), so it counts as one body line.
/// Our Prism implementation counted the full physical heredoc content range,
/// producing false positives on large documentation/example blocks.
///
/// Fix: detect "single heredoc expression body" and count it as one line.
///
/// Additional investigation (same run):
/// - FN: `super do ... end` blocks were not analyzed because only `CallNode`-backed blocks were handled.
/// - FP: `Data.define(...) do ... end` constructor blocks were counted, while RuboCop exempts them like `Struct.new`.
/// - FN: brace-style blocks where the last body token shares the closing `}` line
///   (e.g. `lambda { Hash.new(...) }`) were undercounted by one line.
///
/// Fixes:
/// - Analyze `SuperNode` and `ForwardingSuperNode` blocks as method name `super`.
/// - Extend constructor exemption to include `Data.define`.
/// - When block body and closing token share a line, count that trailing body line.
///
/// ## Corpus investigation (2026-03-07)
///
/// Corpus oracle reported FP=39, FN=0.
///
/// Root cause: RuboCop's `source_from_node_with_heredoc` uses `each_descendant`
/// (which excludes the body node itself) to find max last_line. When a block body
/// contains heredocs, lines belonging to the root body node (e.g. a trailing `)`
/// on its own line in `Hash.new(...)`) are excluded from the count. nitrocop was
/// using `body.location().end_offset()` which includes the full body span,
/// overcounting by 1 line in these cases.
///
/// Fix: when the body contains heredoc descendants, compute the end line as the
/// max last_line across all descendants only (via `heredoc_descendant_max_line`),
/// matching RuboCop's `source_from_node_with_heredoc` algorithm.
///
/// Result: FP=0, FN=0 (within file-drop noise).
///
/// ## Corpus investigation (2026-03-08)
///
/// Corpus oracle reported FP=28, FN=193.
///
/// FN root cause identified: `heredoc_descendant_max_line` only tracked end
/// lines for StringNode and InterpolatedStringNode descendants. When a block
/// body contained heredocs AND code after the heredoc, the descendant max line
/// was set to the heredoc terminator line, cutting off subsequent code lines.
///
/// An attempt was made to rewrite `heredoc_descendant_max_line` to track ALL
/// descendant node end lines (commit 15aeae64, reverted). The approach used
/// visit_branch_node_enter/visit_leaf_node_enter callbacks with depth tracking
/// to exclude the root body node (matching Parser's each_descendant semantics).
///
/// The fix was reverted because corpus validation produced catastrophic 130K
/// offenses (vs 19K expected). Root cause of the regression was missing bundle
/// symlinks in the worktree (bench/corpus/vendor/bundle), causing nitrocop to
/// fall back to hardcoded defaults for ALL cops, not just BlockLength. The fix
/// itself was never validated in a proper environment.
///
/// A correct fix should: (1) rewrite heredoc_descendant_max_line to track all
/// descendant types, (2) validate in an environment with proper corpus bundle
/// symlinks, (3) handle the Parser each_descendant root-exclusion semantics.
///
/// ## Corpus investigation (2026-03-09)
///
/// Re-applied the descendant-tracking fix in a worktree with the required
/// corpus wiring. The prior revert was environmental, not a behavioral
/// regression in the BlockLength logic.
///
/// Fix: track end lines for all descendants, not just string descendants.
/// For heredocs, continue using `closing_loc` so the terminator line matches
/// RuboCop's `loc.heredoc_end.line`. For single-statement bodies wrapped by
/// Prism `StatementsNode`, skip the wrapper and the single child expression so
/// the traversal matches Parser's `each_descendant` root exclusion.
///
/// Acceptance gate with `mise exec ruby@3.4 -- python3 scripts/check-cop.py
/// Metrics/BlockLength --verbose --rerun`:
/// - Expected: 19,376
/// - Actual:   19,248
/// - Excess:   0 over CI baseline after file-drop adjustment
/// - Missing:  128
///
/// Result: accepted. The fix recovered 65 missing offenses vs the prior CI
/// baseline (193 -> 128) without introducing false-positive regressions.
///
/// ## Corpus investigation (2026-03-09, second pass)
///
/// Investigated the remaining 128 FN without corpus repo access. Compared
/// RuboCop's `CodeLengthCalculator.code_length` (which uses `body.source.lines`)
/// with nitrocop's `count_block_lines` (offset-based line counting).
///
/// Found one verified FN cause: when a block's body starts on the same line
/// as the opening `do`/`{` AND that line is line 1 of the file,
/// `count_body_lines_ex` excludes it (counts from start_line+1 which is 2).
/// RuboCop's `body.source.lines` includes the opening line's body content.
/// Fix: add 1 when body_start_line == 1 && opening_line == 1.
///
/// This edge case is rare in practice (block starting on line 1 of a file),
/// so it likely explains very few of the 128 FN. The remaining FN require
/// corpus repo access to investigate — need per-repo FN examples from
/// `check-cop.py --rerun` to identify the systematic pattern.
///
/// Other investigated areas that do NOT explain the FN:
/// - Blocks with rescue/ensure: BeginNode bodies had +1 overcount (fixed 2026-03-10)
/// - heredoc_descendant_max_line: matches RuboCop's source_from_node_with_heredoc
///   for all tested patterns (single heredoc, multiple heredocs, heredoc+code)
/// - method_receiver_excluded?: not implemented but would cause FP, not FN
/// - is_single_heredoc_expression: correctly handles single-heredoc-body blocks
/// - Brace blocks with body on closing line: already handled by closing-line adjustment
///
/// ## Corpus investigation (2026-03-10)
///
/// Previous investigation claimed FP=0, FN=0 on CI, but corpus oracle at
/// 7de25cc2 reported FP=36, FN=0.
///
/// Root cause: blocks with `rescue`/`ensure` were overcounted by 1 line.
/// Prism's `BeginNode` (used for blocks with rescue/ensure) has its
/// `location().start_offset()` set to the opening keyword (`do`/`def`),
/// not the first body statement. This caused `body_start_line` to equal
/// the opening line, making `count_body_lines_ex` include the `do` line
/// as a body line.
///
/// Additionally, BeginNode's `location().end_offset()` includes the `end`
/// keyword, which triggered the brace-block closing-line adjustment
/// (`body_end_line == closing_line`), adding another +1.
///
/// Fixes:
/// - Use `begin_node.statements().start_offset()` for body_start_line
///   instead of the BeginNode's own start_offset.
/// - Skip the brace-block closing-line adjustment for BeginNode bodies.
///
/// This fixes the systematic +1 overcount (19 of 36 FPs had [26/25]).
/// Remaining FPs are config resolution issues on CI (Enabled: false,
/// DisabledByDefault, higher Max) — not cop logic bugs.
///
/// Local `check-cop.py --rerun --quick`: excess=0, missing=146.
pub struct BlockLength;

impl Cop for BlockLength {
    fn name(&self) -> &'static str {
        "Metrics/BlockLength"
    }

    fn default_exclude(&self) -> &'static [&'static str] {
        &["**/*.gemspec"]
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE, SUPER_NODE, FORWARDING_SUPER_NODE, LAMBDA_NODE]
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
        // Handle lambda nodes: ->(x) do...end / ->(x) {...}
        if let Some(lambda_node) = node.as_lambda_node() {
            self.check_lambda(source, &lambda_node, config, diagnostics);
            return;
        }

        if let Some(call_node) = node.as_call_node() {
            let block_node = match call_node.block().and_then(|b| b.as_block_node()) {
                Some(b) => b,
                None => return,
            };
            // RuboCop skips class constructor blocks (Struct.new, Class.new, etc.)
            if is_class_constructor(&call_node) {
                return;
            }
            self.check_invocation_block(
                source,
                std::str::from_utf8(call_node.name().as_slice()).unwrap_or(""),
                call_node.location().start_offset(),
                &block_node,
                config,
                diagnostics,
            );
            return;
        }

        if let Some(super_node) = node.as_super_node() {
            let block_node = match super_node.block().and_then(|b| b.as_block_node()) {
                Some(b) => b,
                None => return,
            };
            self.check_invocation_block(
                source,
                "super",
                super_node.location().start_offset(),
                &block_node,
                config,
                diagnostics,
            );
            return;
        }

        if let Some(forwarding_super_node) = node.as_forwarding_super_node() {
            let block_node = match forwarding_super_node.block() {
                Some(b) => b,
                None => return,
            };
            self.check_invocation_block(
                source,
                "super",
                forwarding_super_node.location().start_offset(),
                &block_node,
                config,
                diagnostics,
            );
        }
    }
}

impl BlockLength {
    fn check_invocation_block(
        &self,
        source: &SourceFile,
        method_name: &str,
        offense_offset: usize,
        block_node: &ruby_prism::BlockNode<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let max = config.get_usize("Max", 25);
        let count_comments = config.get_bool("CountComments", false);
        let count_as_one = config.get_string_array("CountAsOne");
        let allowed_methods = config.get_string_array("AllowedMethods");
        let allowed_patterns = config.get_string_array("AllowedPatterns");

        if let Some(allowed) = &allowed_methods {
            if allowed.iter().any(|m| m == method_name) {
                return;
            }
        }
        if let Some(patterns) = &allowed_patterns {
            for pat in patterns {
                if let Ok(re) = regex::Regex::new(pat) {
                    if re.is_match(method_name) {
                        return;
                    }
                }
            }
        }

        let end_offset = block_node.closing_loc().start_offset();
        let count = count_block_lines(
            source,
            block_node.opening_loc().start_offset(),
            end_offset,
            block_node.closing_loc().end_offset(),
            block_node.body(),
            count_comments,
            &count_as_one,
        );

        if count > max {
            let (line, column) = source.offset_to_line_col(offense_offset);
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Block has too many lines. [{count}/{max}]"),
            ));
        }
    }

    fn check_lambda(
        &self,
        source: &SourceFile,
        lambda_node: &ruby_prism::LambdaNode<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let max = config.get_usize("Max", 25);
        let count_comments = config.get_bool("CountComments", false);
        let count_as_one = config.get_string_array("CountAsOne");

        let end_offset = lambda_node.closing_loc().start_offset();
        let count = count_block_lines(
            source,
            lambda_node.opening_loc().start_offset(),
            end_offset,
            lambda_node.closing_loc().end_offset(),
            lambda_node.body(),
            count_comments,
            &count_as_one,
        );

        if count > max {
            let (line, column) = source.offset_to_line_col(lambda_node.location().start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Block has too many lines. [{count}/{max}]"),
            ));
        }
    }
}

/// Count body lines for a block, folding heredocs and CountAsOne constructs.
/// Uses the body node's start offset (not opening_loc) to avoid counting
/// heredoc content lines that physically appear before the body starts.
fn count_block_lines(
    source: &SourceFile,
    opening_offset: usize,
    end_offset: usize,
    closing_end_offset: usize,
    body: Option<ruby_prism::Node<'_>>,
    count_comments: bool,
    count_as_one: &Option<Vec<String>>,
) -> usize {
    let body = match body {
        Some(b) => b,
        None => return 0,
    };

    // Parser/RuboCop behavior: when a block body is a single heredoc expression,
    // code length is based on the heredoc opener node source, not heredoc content.
    // This makes the body count as one line.
    if is_single_heredoc_expression(source, &body) {
        return 1;
    }

    // Use body start offset to skip heredoc content that appears before body.
    // Same approach as method_length.rs.
    //
    // When the body is a BeginNode (block with rescue/ensure), Prism sets
    // its location.start_offset to the opening keyword (do/def), not the
    // first body statement. Use statements().start_offset() instead to
    // match Parser's kwbegin.body behavior.
    let body_start_offset = body
        .as_begin_node()
        .and_then(|b| b.statements())
        .map(|s| s.location().start_offset())
        .unwrap_or_else(|| body.location().start_offset());
    let (body_start_line, _) = source.offset_to_line_col(body_start_offset);
    let effective_start_offset = if body_start_line > 1 {
        source
            .line_col_to_offset(body_start_line - 1, 0)
            .unwrap_or(opening_offset)
    } else {
        opening_offset
    };

    // RuboCop's `source_from_node_with_heredoc`: when the body contains any
    // heredoc descendants, RuboCop computes the end line as the max last_line
    // across all *descendants* (not the body node itself). In Parser AST,
    // `each_descendant` excludes the root node, so structural delimiters like
    // a trailing `)` on its own line (part of the root send/call node) are not
    // counted. Without this adjustment, nitrocop overcounts by including lines
    // that RuboCop's descendant-based algorithm skips.
    let mut effective_end_offset = end_offset;
    let (closing_line, _) = source.offset_to_line_col(end_offset);
    if let Some(desc_end_line) = heredoc_descendant_max_line(source, &body) {
        // Use descendant max line as the end boundary (matches RuboCop behavior).
        // But only if the heredoc content is INSIDE the block (before the closing
        // delimiter). For brace blocks like `{ |f| f << <<EOF }`, the heredoc
        // content is physically after `}` — the Parser gem's block node does NOT
        // include it, so RuboCop's early bail-out `node.line_count <= max_length`
        // skips counting entirely. We must NOT extend past `}` in that case.
        if desc_end_line <= closing_line {
            // Heredoc terminates before or on the closing line — extend normally
            if let Some(next_line_start) = source.line_col_to_offset(desc_end_line + 1, 0) {
                effective_end_offset = next_line_start;
            }
        }
        // else: heredoc is after closing `}` — don't extend, use original end_offset
    } else {
        // No heredocs: for brace blocks like `lambda { Hash.new(...) }`, the final
        // body token can share the same line as the closing `}`. RuboCop counts that
        // final line. `count_body_lines_ex` excludes the end line, so move end to
        // next line start.
        //
        // Skip this adjustment for BeginNode bodies (blocks with rescue/ensure):
        // Prism's BeginNode location extends to include the closing `end` keyword,
        // so body_end_line == closing_line is a false match. The `end` keyword is
        // already correctly excluded by count_body_lines_ex's exclusive upper bound.
        if body.as_begin_node().is_none() {
            let (body_end_line, _) =
                source.offset_to_line_col(body.location().end_offset().saturating_sub(1));
            if body_end_line == closing_line {
                if let Some(next_line_start) = source.line_col_to_offset(closing_line + 1, 0) {
                    effective_end_offset = next_line_start;
                } else {
                    effective_end_offset = closing_end_offset;
                }
            }
        }
    }

    // Collect foldable ranges from CountAsOne config. Heredocs are only
    // folded when "heredoc" is explicitly in CountAsOne (default: []).
    // For non-bare-heredoc bodies, RuboCop's CodeLengthCalculator includes
    // heredoc content lines by default. We replicate that here.
    let mut all_foldable: Vec<(usize, usize)> = Vec::new();
    if let Some(cao) = count_as_one {
        if !cao.is_empty() {
            all_foldable.extend(collect_foldable_ranges(source, &body, cao));
            if cao.iter().any(|s| s == "heredoc") {
                all_foldable.extend(collect_heredoc_ranges(source, &body));
            }
        }
    }
    all_foldable.sort();
    all_foldable.dedup();

    let mut count = count_body_lines_ex(
        source,
        effective_start_offset,
        effective_end_offset,
        count_comments,
        &all_foldable,
    );

    // When body_start_line > 1, effective_start is set to the previous line,
    // so count_body_lines_ex counts from body_start_line onwards (correct).
    // When body_start_line == 1 AND the body is on the opening line, there is
    // no "previous line" to set as start, so count_body_lines_ex starts from
    // line 2, missing the body content on line 1. Add 1 to compensate.
    let (opening_line, _) = source.offset_to_line_col(opening_offset);
    if body_start_line == 1 && opening_line == 1 {
        count += 1;
    }

    count
}

/// When the body contains heredoc descendants, compute the max last_line across
/// all descendants (matching RuboCop's `source_from_node_with_heredoc`).
/// Returns `Some(max_line)` if heredocs are found, `None` otherwise.
///
/// RuboCop calls `each_descendant` on `extract_body(node)` which excludes the
/// root body node itself. In Parser AST, the body of a single-statement block
/// is the expression directly; in Prism it's always wrapped in StatementsNode.
/// For single-statement bodies, we skip both StatementsNode (depth 0) and the
/// single child expression (depth 1) to match Parser's exclusion of the root.
/// For multi-statement bodies, depth-1 children are proper descendants.
fn heredoc_descendant_max_line(source: &SourceFile, body: &ruby_prism::Node<'_>) -> Option<usize> {
    use ruby_prism::Visit;

    let single_child_body = body
        .as_statements_node()
        .map(|s| s.body().iter().count() == 1)
        .unwrap_or(false);
    let exclude_depth: usize = if single_child_body { 2 } else { 1 };

    struct DescendantMaxLineFinder<'s> {
        source: &'s SourceFile,
        max_line: usize,
        has_heredoc: bool,
        depth: usize,
        exclude_depth: usize,
    }

    impl DescendantMaxLineFinder<'_> {
        fn track_node_end(&mut self, node: ruby_prism::Node<'_>) {
            if self.depth < self.exclude_depth {
                return;
            }
            let (line, _) = self
                .source
                .offset_to_line_col(node.location().end_offset().saturating_sub(1));
            self.max_line = self.max_line.max(line);
        }
    }

    impl<'pr> Visit<'pr> for DescendantMaxLineFinder<'_> {
        fn visit_branch_node_enter(&mut self, node: ruby_prism::Node<'pr>) {
            self.track_node_end(node);
            self.depth += 1;
        }

        fn visit_branch_node_leave(&mut self) {
            self.depth -= 1;
        }

        fn visit_leaf_node_enter(&mut self, node: ruby_prism::Node<'pr>) {
            self.track_node_end(node);
        }

        fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
            if let Some(opening) = node.opening_loc() {
                let bytes = &self.source.as_bytes()[opening.start_offset()..opening.end_offset()];
                if bytes.starts_with(b"<<") {
                    self.has_heredoc = true;
                    // Use closing_loc (the EOS terminator) as the end line
                    if let Some(closing) = node.closing_loc() {
                        let (line, _) = self
                            .source
                            .offset_to_line_col(closing.end_offset().saturating_sub(1));
                        self.max_line = self.max_line.max(line);
                    }
                }
            }
        }

        fn visit_interpolated_string_node(
            &mut self,
            node: &ruby_prism::InterpolatedStringNode<'pr>,
        ) {
            if let Some(opening) = node.opening_loc() {
                let bytes = &self.source.as_bytes()[opening.start_offset()..opening.end_offset()];
                if bytes.starts_with(b"<<") {
                    self.has_heredoc = true;
                    if let Some(closing) = node.closing_loc() {
                        let (line, _) = self
                            .source
                            .offset_to_line_col(closing.end_offset().saturating_sub(1));
                        self.max_line = self.max_line.max(line);
                    }
                    return;
                }
            }
            ruby_prism::visit_interpolated_string_node(self, node);
        }
    }

    let mut finder = DescendantMaxLineFinder {
        source,
        max_line: 0,
        has_heredoc: false,
        depth: 0,
        exclude_depth,
    };
    finder.visit(body);

    if finder.has_heredoc {
        Some(finder.max_line)
    } else {
        None
    }
}

fn is_single_heredoc_expression(source: &SourceFile, body: &ruby_prism::Node<'_>) -> bool {
    if is_heredoc_node(source, body) {
        return true;
    }

    if let Some(stmts) = body.as_statements_node() {
        let mut iter = stmts.body().iter();
        if let Some(first) = iter.next() {
            return iter.next().is_none() && is_heredoc_node(source, &first);
        }
    }

    false
}

fn is_heredoc_node(source: &SourceFile, node: &ruby_prism::Node<'_>) -> bool {
    if let Some(s) = node.as_string_node() {
        return s
            .opening_loc()
            .map(|o| source.as_bytes()[o.start_offset()..o.end_offset()].starts_with(b"<<"))
            .unwrap_or(false);
    }

    if let Some(s) = node.as_interpolated_string_node() {
        return s
            .opening_loc()
            .map(|o| source.as_bytes()[o.start_offset()..o.end_offset()].starts_with(b"<<"))
            .unwrap_or(false);
    }

    false
}

/// Check if a call is a class constructor like `Struct.new`, `Class.new`, `Module.new`, etc.
/// RuboCop's Metrics/BlockLength does not count these blocks.
fn is_class_constructor(call: &ruby_prism::CallNode<'_>) -> bool {
    let recv = match call.receiver() {
        Some(r) => r,
        None => return false,
    };
    let recv_name = crate::cop::util::constant_name(&recv).unwrap_or_default();

    match call.name().as_slice() {
        b"new" => matches!(recv_name, b"Struct" | b"Class" | b"Module"),
        // Data.define is also a class constructor in RuboCop.
        b"define" => recv_name == b"Data",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(BlockLength, "cops/metrics/block_length");

    #[test]
    fn config_custom_max() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(3.into()))]),
            ..CopConfig::default()
        };
        // 4 body lines exceeds Max:3
        let source = b"items.each do |x|\n  a = 1\n  b = 2\n  c = 3\n  d = 4\nend\n";
        let diags = run_cop_full_with_config(&BlockLength, source, config);
        assert!(!diags.is_empty(), "Should fire with Max:3 on 4-line block");
        assert!(diags[0].message.contains("[4/3]"));
    }

    #[test]
    fn config_count_as_one_array() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([
                ("Max".into(), serde_yml::Value::Number(3.into())),
                (
                    "CountAsOne".into(),
                    serde_yml::Value::Sequence(vec![serde_yml::Value::String("array".into())]),
                ),
            ]),
            ..CopConfig::default()
        };
        // Body: a, b, [\n1,\n2\n] = 2 + 1 folded = 3 lines
        let source = b"items.each do |x|\n  a = 1\n  b = 2\n  arr = [\n    1,\n    2\n  ]\nend\n";
        let diags = run_cop_full_with_config(&BlockLength, source, config);
        assert!(
            diags.is_empty(),
            "Should not fire when array is folded (3/3)"
        );
    }

    #[test]
    fn heredoc_with_code_after() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(3.into()))]),
            ..CopConfig::default()
        };
        let source = b"records.transaction do\n  sql = <<~SQL\n    SELECT 1\n  SQL\n  a = 1\n  b = 2\n  c = 3\nend\n";
        let diags = run_cop_full_with_config(&BlockLength, source, config);
        assert!(
            !diags.is_empty(),
            "Should fire on block with heredoc + code after (6 body lines > Max:3)"
        );
        assert!(
            diags[0].message.contains("[6/3]"),
            "Expected [6/3] but got: {}",
            diags[0].message
        );
    }

    #[test]
    fn multi_statement_heredoc_body() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(3.into()))]),
            ..CopConfig::default()
        };
        let source =
            b"items.each do |i|\n  x = 1\n  sql = <<~SQL\n    SELECT 1\n  SQL\n  y = 2\nend\n";
        let diags = run_cop_full_with_config(&BlockLength, source, config);
        assert!(
            !diags.is_empty(),
            "Should fire on multi-statement block with heredoc (5 body lines > Max:3)"
        );
        assert!(
            diags[0].message.contains("[5/3]"),
            "Expected [5/3] but got: {}",
            diags[0].message
        );
    }

    #[test]
    fn block_body_on_same_line_as_opening() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(3.into()))]),
            ..CopConfig::default()
        };
        // Body starts on same line as do: 4 body lines (a, b, c, d) exceeds Max:3
        let source = b"items.each do |x| a = 1\n  b = 2\n  c = 3\n  d = 4\nend\n";
        let diags = run_cop_full_with_config(&BlockLength, source, config);
        assert!(
            !diags.is_empty(),
            "Should fire when body starts on same line as do"
        );
        assert!(
            diags[0].message.contains("[4/3]"),
            "Expected [4/3] but got: {}",
            diags[0].message
        );
    }

    #[test]
    fn single_line_block_body_on_opening() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(0.into()))]),
            ..CopConfig::default()
        };
        // Single line block: foo do a = 1; end → 1 body line exceeds Max:0
        let source = b"foo do a = 1\nend\n";
        let diags = run_cop_full_with_config(&BlockLength, source, config);
        assert!(!diags.is_empty(), "Should fire on single-line body");
        assert!(
            diags[0].message.contains("[1/0]"),
            "Expected [1/0] but got: {}",
            diags[0].message
        );
    }

    #[test]
    fn block_body_on_same_line_not_first_line() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(3.into()))]),
            ..CopConfig::default()
        };
        // Body starts on same line as do, but NOT on line 1 of the file
        // RuboCop counts 4 body lines (a, b, c, d)
        let source = b"x = 1\nitems.each do |x| a = 1\n  b = 2\n  c = 3\n  d = 4\nend\n";
        let diags = run_cop_full_with_config(&BlockLength, source, config);
        assert!(
            !diags.is_empty(),
            "Should fire when body starts on same line as do (not first file line)"
        );
        assert!(
            diags[0].message.contains("[4/3]"),
            "Expected [4/3] but got: {}",
            diags[0].message
        );
    }

    #[test]
    fn block_with_blank_lines_at_threshold() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(25.into()))]),
            ..CopConfig::default()
        };
        // 25 non-blank body lines + 3 blank lines = should NOT fire (25 <= 25)
        let mut source = String::from("items.each do |x|\n");
        for i in 1..=10 {
            source.push_str(&format!("  x{} = {}\n", i, i));
        }
        source.push('\n'); // blank line
        for i in 11..=20 {
            source.push_str(&format!("  x{} = {}\n", i, i));
        }
        source.push('\n'); // blank line
        for i in 21..=25 {
            source.push_str(&format!("  x{} = {}\n", i, i));
        }
        source.push('\n'); // blank line
        source.push_str("end\n");
        let diags = run_cop_full_with_config(&BlockLength, source.as_bytes(), config);
        assert!(
            diags.is_empty(),
            "Should NOT fire on 25 non-blank body lines (blank lines don't count). Got: {:?}",
            diags.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn block_with_rescue_at_threshold() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(25.into()))]),
            ..CopConfig::default()
        };
        // Block with rescue: 22 body lines + rescue + 2 handler lines = 25 non-blank
        // Prefix with a line so block doesn't start on line 1
        let mut source = String::from("x = 1\nitems.each do |x|\n");
        for i in 1..=22 {
            source.push_str(&format!("  x{} = {}\n", i, i));
        }
        source.push_str("rescue StandardError => e\n");
        source.push_str("  log(e)\n");
        source.push_str("  raise\n");
        source.push_str("end\n");

        let diags = run_cop_full_with_config(&BlockLength, source.as_bytes(), config);
        assert!(
            diags.is_empty(),
            "Should NOT fire on block with rescue totaling 25 body lines. Got: {:?}",
            diags.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn block_with_ensure_at_threshold() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(25.into()))]),
            ..CopConfig::default()
        };
        // Block with ensure: 23 body lines + ensure + 1 cleanup = 25 non-blank
        let mut source = String::from("x = 1\nitems.each do |x|\n");
        for i in 1..=23 {
            source.push_str(&format!("  x{} = {}\n", i, i));
        }
        source.push_str("ensure\n");
        source.push_str("  cleanup\n");
        source.push_str("end\n");
        let diags = run_cop_full_with_config(&BlockLength, source.as_bytes(), config);
        assert!(
            diags.is_empty(),
            "Should NOT fire on block with ensure totaling 25 body lines. Got: {:?}",
            diags.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn block_with_rescue_over_threshold() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(25.into()))]),
            ..CopConfig::default()
        };
        // Block with rescue: 23 body lines + rescue + 2 handler lines = 26 non-blank -> fires
        let mut source = String::from("x = 1\nitems.each do |x|\n");
        for i in 1..=23 {
            source.push_str(&format!("  x{} = {}\n", i, i));
        }
        source.push_str("rescue StandardError => e\n");
        source.push_str("  log(e)\n");
        source.push_str("  raise\n");
        source.push_str("end\n");
        let diags = run_cop_full_with_config(&BlockLength, source.as_bytes(), config);
        assert!(
            !diags.is_empty(),
            "Should fire on block with rescue totaling 26 body lines."
        );
        assert!(
            diags[0].message.contains("[26/25]"),
            "Expected [26/25] but got: {}",
            diags[0].message
        );
    }

    #[test]
    fn brace_block_heredoc_argument_not_body() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("Max".into(), serde_yml::Value::Number(25.into()))]),
            ..CopConfig::default()
        };
        // Pattern: File.open('foo', 'w') { |f| f << <<EOF }
        // The block body is `f << <<EOF` — a CallNode (<<), NOT a heredoc itself.
        // The heredoc content physically follows the `}` but is NOT part of the block body.
        // RuboCop sees this as a 1-line block body and does NOT fire.
        let source = b"x = 1\nFile.open('foo.rb', 'w') { |f| f << <<EOF }\nline1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11\nline12\nline13\nline14\nline15\nline16\nline17\nline18\nline19\nline20\nline21\nline22\nline23\nline24\nline25\nline26\nEOF\nz = 3\n";
        let diags = run_cop_full_with_config(&BlockLength, source, config);
        assert!(
            diags.is_empty(),
            "Brace block {{ |f| f << <<EOF }} should be 1-line body, not count heredoc content. Got: {:?}",
            diags.iter().map(|d| &d.message).collect::<Vec<_>>()
        );
    }

    #[test]
    fn allowed_methods_refine() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([
                ("Max".into(), serde_yml::Value::Number(3.into())),
                (
                    "AllowedMethods".into(),
                    serde_yml::Value::Sequence(vec![serde_yml::Value::String("refine".into())]),
                ),
            ]),
            ..CopConfig::default()
        };
        // refine block with 4 lines should NOT fire because refine is allowed
        let source =
            b"refine String do\n  def a; end\n  def b; end\n  def c; end\n  def d; end\nend\n";
        let diags = run_cop_full_with_config(&BlockLength, source, config);
        assert!(
            diags.is_empty(),
            "Should not fire on allowed method 'refine'"
        );
    }
}
