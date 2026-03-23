use ruby_prism::Visit;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::directives::DisabledRanges;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-04)
///
/// Corpus oracle reported FP=1, FN=0.
///
/// FP=1 was from ternary + inline rescue in
/// `activerecord-hackery__ransack__271cb42/lib/ransack/nodes/value.rb:62`.
/// Prism nests ternary `IfNode` under `RescueModifierNode`, while Parser AST
/// (RuboCop) traverses ternary `if` and `resbody` as siblings under `rescue`.
/// This cop now emulates Parser semantics for the ternary+rescue shape so the
/// two nesting increments do not stack.
///
/// Added fixture coverage in `tests/fixtures/cops/metrics/block_nesting/no_offense.rb`.
/// Local corpus rerun delta vs unchanged baseline binary was repo-local and
/// isolated to the target file (`3 -> 2` offenses in ransack), with no other
/// repo-level count changes.
///
/// ## Corpus investigation (2026-03-23)
///
/// Extended corpus reported FP=16, FN=1.
///
/// FN=1 was from `GoogleCloudPlatform/inspec-gcp-cis-benchmark` —
/// `controls/1.01-iam.rb:79`.  An inline `# rubocop:disable Metrics/BlockNesting`
/// on a parent `if` node caused nitrocop to skip the entire subtree, missing a
/// deeper offense.  In RuboCop, `ignore_node` is only called when the offense
/// is actually emitted (not suppressed by directive), so descendants of a
/// directive-suppressed node are still checked.
///
/// Fixed by collecting ALL offenses (never skipping subtrees), then applying
/// an ignore-subtree dedup pass that respects inline disable directives: only
/// unsuppressed parent offenses shadow their descendants.
pub struct BlockNesting;

impl Cop for BlockNesting {
    fn name(&self) -> &'static str {
        "Metrics/BlockNesting"
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::parse::codemap::CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let max = config.get_usize("Max", 3);
        let count_blocks = config.get_bool("CountBlocks", false);
        let count_modifier_forms = config.get_bool("CountModifierForms", false);

        let mut visitor = NestingVisitor {
            source,
            max,
            count_blocks,
            count_modifier_forms,
            depth: 0,
            offenses: Vec::new(),
        };
        visitor.visit(&parse_result.node());

        // Replicate RuboCop's ignore_node / part_of_ignored_node? semantics:
        //
        // In RuboCop, `ignore_node(node)` is called inside the `add_offense` block,
        // which only executes when the offense is NOT suppressed by an inline
        // directive.  If a parent offense is suppressed (e.g. by an inline
        // `# rubocop:disable Metrics/BlockNesting`), `ignore_node` is never called
        // for it, so child offenses at deeper nesting levels are still reported.
        //
        // To replicate this, we collect ALL offenses (the visitor never skips
        // subtrees), then filter: keep an offense unless a "parent" offense
        // (one whose byte range contains it) exists AND is NOT suppressed by
        // a disable directive.
        let disabled = DisabledRanges::from_comments(source, parse_result);
        let cop_name = self.name();

        // Build the set of "active ignored" byte ranges: offenses that are NOT
        // directive-suppressed.  These shadow their descendants.
        let ignored_ranges: Vec<(usize, usize)> = visitor
            .offenses
            .iter()
            .filter(|o| !disabled.is_disabled(cop_name, o.diag.location.line))
            .map(|o| (o.node_start, o.node_end))
            .collect();

        for offense in visitor.offenses {
            // Skip this offense if it is inside an "active ignored" parent range,
            // unless it IS that parent itself.
            let dominated = ignored_ranges.iter().any(|&(ps, pe)| {
                // Parent range strictly contains this offense's node range.
                ps < offense.node_start && offense.node_end <= pe
            });
            if dominated {
                continue;
            }
            diagnostics.push(offense.diag);
        }
    }

    fn diagnostic(
        &self,
        source: &SourceFile,
        line: usize,
        column: usize,
        message: String,
    ) -> Diagnostic {
        Diagnostic {
            path: source.path_str().to_string(),
            location: crate::diagnostic::Location { line, column },
            severity: self.default_severity(),
            cop_name: self.name().to_string(),
            message,
            corrected: false,
        }
    }
}

/// An offense with its AST node byte range for ignore-subtree dedup.
struct NestingOffense {
    diag: Diagnostic,
    /// Start byte offset of the AST node that triggered this offense.
    node_start: usize,
    /// End byte offset of the AST node that triggered this offense.
    node_end: usize,
}

struct NestingVisitor<'a> {
    source: &'a SourceFile,
    max: usize,
    count_blocks: bool,
    count_modifier_forms: bool,
    depth: usize,
    offenses: Vec<NestingOffense>,
}

impl NestingVisitor<'_> {
    /// Check nesting depth and record offense if exceeded.
    /// Always returns without skipping the subtree — the caller must continue
    /// recursing so that deeper offenses are discovered (they are deduped later
    /// in `check_source`).
    fn check_nesting(&mut self, loc: &ruby_prism::Location<'_>) {
        if self.depth > self.max {
            let (line, column) = self.source.offset_to_line_col(loc.start_offset());
            self.offenses.push(NestingOffense {
                diag: Diagnostic {
                    path: self.source.path_str().to_string(),
                    location: crate::diagnostic::Location { line, column },
                    severity: crate::diagnostic::Severity::Convention,
                    cop_name: "Metrics/BlockNesting".to_string(),
                    message: format!("Avoid more than {} levels of block nesting.", self.max),
                    corrected: false,
                },
                node_start: loc.start_offset(),
                node_end: loc.end_offset(),
            });
        }
    }
}

impl<'pr> Visit<'pr> for NestingVisitor<'_> {
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        // RuboCop does NOT reset nesting at method boundaries — it walks the
        // AST recursively, passing current_level through each_child_node without
        // any special handling for def nodes. A def inside nested conditionals
        // inherits the outer nesting depth.
        ruby_prism::visit_def_node(self, node);
    }

    fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
        // In Prism, `elsif` branches are represented as nested IfNodes.
        // RuboCop does not count elsif as additional nesting depth.
        let is_elsif = node
            .if_keyword_loc()
            .is_some_and(|kw| kw.as_slice() == b"elsif");

        // Ternary: `a ? b : c` has no if_keyword_loc (it's None).
        // Modifier if: `foo if bar` has if_keyword_loc but no end_keyword_loc.
        // Only skip modifier forms (not ternaries) when CountModifierForms is false.
        let is_ternary = node.if_keyword_loc().is_none();
        let is_modifier = !is_ternary && node.end_keyword_loc().is_none();
        let should_count = !is_elsif && (self.count_modifier_forms || !is_modifier);

        if should_count {
            self.depth += 1;
            self.check_nesting(&node.location());
        }
        ruby_prism::visit_if_node(self, node);
        if should_count {
            self.depth -= 1;
        }
    }

    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
        // Modifier unless (e.g. `foo unless bar`) has no `end` keyword.
        let is_modifier = node.end_keyword_loc().is_none();
        if !self.count_modifier_forms && is_modifier {
            ruby_prism::visit_unless_node(self, node);
            return;
        }
        self.depth += 1;
        self.check_nesting(&node.location());
        ruby_prism::visit_unless_node(self, node);
        self.depth -= 1;
    }

    fn visit_while_node(&mut self, node: &ruby_prism::WhileNode<'pr>) {
        // RuboCop always counts while/until as nesting, including modifier forms.
        // CountModifierForms only affects if/unless, not while/until.
        self.depth += 1;
        self.check_nesting(&node.location());
        ruby_prism::visit_while_node(self, node);
        self.depth -= 1;
    }

    fn visit_until_node(&mut self, node: &ruby_prism::UntilNode<'pr>) {
        // RuboCop always counts while/until as nesting, including modifier forms.
        // CountModifierForms only affects if/unless, not while/until.
        self.depth += 1;
        self.check_nesting(&node.location());
        ruby_prism::visit_until_node(self, node);
        self.depth -= 1;
    }

    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode<'pr>) {
        self.depth += 1;
        self.check_nesting(&node.location());
        ruby_prism::visit_case_node(self, node);
        self.depth -= 1;
    }

    fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode<'pr>) {
        self.depth += 1;
        self.check_nesting(&node.location());
        ruby_prism::visit_case_match_node(self, node);
        self.depth -= 1;
    }

    fn visit_for_node(&mut self, node: &ruby_prism::ForNode<'pr>) {
        self.depth += 1;
        self.check_nesting(&node.location());
        ruby_prism::visit_for_node(self, node);
        self.depth -= 1;
    }

    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode<'pr>) {
        // In Prism, rescue clauses are chained via `subsequent` (each RescueNode
        // contains a pointer to the next one). In the Parser gem AST, `resbody` nodes
        // are siblings under a `rescue` parent. We must NOT increment depth for
        // subsequent rescue clauses — they're at the same nesting level.
        //
        // Manually walk the node: visit statements at incremented depth,
        // then visit subsequent at the ORIGINAL depth.
        self.depth += 1;
        self.check_nesting(&node.location());
        // Visit the rescue body (statements) at incremented depth
        if let Some(stmts) = node.statements() {
            self.visit_statements_node(&stmts);
        }
        self.depth -= 1;

        // Visit subsequent rescue clause at the SAME depth (sibling, not nested)
        if let Some(subsequent) = node.subsequent() {
            self.visit_rescue_node(&subsequent);
        }
    }

    fn visit_rescue_modifier_node(&mut self, node: &ruby_prism::RescueModifierNode<'pr>) {
        let expression = node.expression();
        let rescue_expression = node.rescue_expression();

        // In Parser AST (used by RuboCop), modifier rescue wraps a ternary as
        // sibling nodes under :rescue (if + resbody), so their nesting does not
        // stack. Prism nests the ternary under RescueModifierNode, so emulate
        // Parser semantics only for this shape.
        let is_ternary_expression = expression
            .as_if_node()
            .is_some_and(|if_node| if_node.if_keyword_loc().is_none());

        if is_ternary_expression {
            self.visit(&expression);
            self.depth += 1;
            self.check_nesting(&node.keyword_loc());
            self.visit(&rescue_expression);
            self.depth -= 1;
            return;
        }

        // Default behavior: inline rescue contributes one nesting level.
        self.depth += 1;
        self.check_nesting(&node.keyword_loc());
        self.visit(&expression);
        self.visit(&rescue_expression);
        self.depth -= 1;
    }

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        if self.count_blocks {
            self.depth += 1;
            self.check_nesting(&node.location());
            ruby_prism::visit_block_node(self, node);
            self.depth -= 1;
        } else {
            ruby_prism::visit_block_node(self, node);
        }
    }

    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
        if self.count_blocks {
            self.depth += 1;
            self.check_nesting(&node.location());
            ruby_prism::visit_lambda_node(self, node);
            self.depth -= 1;
        } else {
            ruby_prism::visit_lambda_node(self, node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_scenario_fixture_tests!(
        BlockNesting,
        "cops/metrics/block_nesting",
        nested_ifs = "nested_ifs.rb",
        nested_unless = "nested_unless.rb",
        nested_while = "nested_while.rb",
        nested_rescue = "nested_rescue.rb",
        nested_for = "nested_for.rb",
        nested_case_match = "nested_case_match.rb",
        toplevel_nesting = "toplevel_nesting.rb",
        begin_end_while = "begin_end_while.rb",
        ignore_subtree = "ignore_subtree.rb",
        sibling_violations = "sibling_violations.rb",
        modifier_while = "modifier_while.rb",
        modifier_until = "modifier_until.rb",
        inline_rescue = "inline_rescue.rb",
        method_inside_nesting = "method_inside_nesting.rb",
        inline_disable_nested = "inline_disable_nested.rb",
    );
}
