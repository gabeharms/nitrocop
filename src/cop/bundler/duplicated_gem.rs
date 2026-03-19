use std::collections::HashMap;
use std::collections::hash_map::Entry;

use ruby_prism::Visit;

use crate::cop::{Cop, CopConfig, util};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct DuplicatedGem;

/// ## Corpus investigation (2026-03-19)
///
/// ### Round 5 — Extended corpus FP=3, FN=7
///
/// **FP=3**: case/when/else with nested if/else inside else. RuboCop's
/// `within_conditional?` checks `branch.child_nodes.include?(node)` using
/// structural equality. In Parser gem, `if` node's child_nodes include
/// if_body and else_body directly (single-statement). So gems inside a
/// nested if that is the else branch of a case are found via structural
/// equality against gems in other case branches. Fixed by changing Path 1
/// from "all identical source" to per-gem check: each gem's source must
/// match some "branch member" (gem with blocks_above==0 and same root).
///
/// **FN=7**: Two root causes:
/// 1. Gems inside `path`/`git`/`source` blocks within conditionals were
///    exempted by Path 1 (all identical source) even though their first
///    non-begin ancestor is `:block`, not `:if`/`:when`. Fixed by requiring
///    `first.blocks_above_conditional == 0` in the conditional check.
/// 2. Modifier `if`/`unless` was treated as transparent (BeginLike), making
///    gems inside them appear as direct children of the enclosing real
///    conditional. In Parser gem, modifier if IS an `if` node and stops the
///    ancestor search. Fixed by treating modifier ifs as conditional roots,
///    and using `take()` on `pending_elsif_root` to prevent leaking into
///    nested ifs.
///
/// ### Round 4 — FP=0, FN=0 (standard corpus)
///
/// Fixed the Autolab FN: structural equality Path 1 used `any_conditional`
/// (any declaration in a conditional) but RuboCop's `conditional_declaration?`
/// checks `nodes[0]`'s ancestor first. When the first declaration is NOT in a
/// conditional (e.g., in a `group` block), structural equality never applies.
/// Fix: changed Path 1 to require `first.conditional_root.is_some()`.
impl Cop for DuplicatedGem {
    fn name(&self) -> &'static str {
        "Bundler/DuplicatedGem"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn default_include(&self) -> &'static [&'static str] {
        &["**/*.gemfile", "**/Gemfile", "**/gems.rb"]
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
        let mut visitor = GemDeclarationVisitor {
            source,
            declarations: Vec::new(),
            ancestors: Vec::new(),
            next_conditional_root_id: 1,
            pending_elsif_root: None,
        };
        visitor.visit(&parse_result.node());

        let mut grouped: HashMap<Vec<u8>, Vec<GemDeclaration>> = HashMap::new();
        for declaration in visitor.declarations {
            match grouped.entry(declaration.gem_name.clone()) {
                Entry::Occupied(mut occupied) => occupied.get_mut().push(declaration),
                Entry::Vacant(vacant) => {
                    vacant.insert(vec![declaration]);
                }
            }
        }

        for declarations in grouped.into_values() {
            if declarations.len() < 2 {
                continue;
            }

            let first = &declarations[0];

            // RuboCop's `conditional_declaration?` requires that nodes[0]'s first
            // non-begin ancestor is an `:if` or `:when` node. In Prism terms, this
            // means the first gem must have blocks_above_conditional == 0 and be
            // inside a conditional root. Gems inside blocks (path/git/source/group)
            // have blocks_above > 0 and are NOT considered conditional.
            let first_root = match first.conditional_root {
                Some(root) if first.blocks_above_conditional == 0 => root,
                _ => {
                    // First gem is not directly in a conditional — flag all duplicates.
                    let gem_name = String::from_utf8_lossy(&first.gem_name);
                    for duplicate in declarations.iter().skip(1) {
                        diagnostics.push(self.diagnostic(
                            source,
                            duplicate.line,
                            duplicate.column,
                            format!(
                                "Gem `{}` requirements already given on line {} of the Gemfile.",
                                gem_name, first.line
                            ),
                        ));
                    }
                    continue;
                }
            };

            // Collect "accessible" gem sources: sources of gems that are reachable
            // via `branch.child_nodes.include?(node)` from the root conditional.
            // This includes:
            // 1. Gems directly in the root conditional (same root, blocks_above=0)
            // 2. Gems in nested conditionals within the root (their ancestor chain
            //    contains the root, blocks_above=0 for their nearest conditional)
            //
            // This replicates RuboCop's structural equality where child_nodes of
            // a branch (e.g., an `if` node that is the else body) expose single-
            // statement bodies as direct children.
            let accessible_sources: Vec<&[u8]> = declarations
                .iter()
                .filter(|d| {
                    d.blocks_above_conditional == 0
                        && d.ancestor_conditional_roots.contains(&first_root)
                })
                .map(|d| d.call_source.as_slice())
                .collect();

            // RuboCop's `within_conditional?` checks each gem individually:
            // `branch == node || branch.child_nodes.include?(node)` using
            // structural equality. A gem is "within" the conditional if its
            // source matches any accessible source.
            let all_within_conditional = declarations
                .iter()
                .all(|decl| accessible_sources.contains(&decl.call_source.as_slice()));

            if all_within_conditional {
                continue;
            }

            let gem_name = String::from_utf8_lossy(&first.gem_name);
            for duplicate in declarations.iter().skip(1) {
                diagnostics.push(self.diagnostic(
                    source,
                    duplicate.line,
                    duplicate.column,
                    format!(
                        "Gem `{}` requirements already given on line {} of the Gemfile.",
                        gem_name, first.line
                    ),
                ));
            }
        }
    }
}

#[derive(Clone, Copy)]
enum AncestorKind {
    /// Opaque block — breaks direct-child relationship for conditional exemption.
    /// Used for CallNode, BlockNode with multi-statement body, and similar.
    Block,
    /// Transparent wrapper — does not break the conditional ancestor chain.
    /// Used for StatementsNode, BeginNode, ElseNode, ProgramNode, single-stmt BlockNode.
    BeginLike,
    If {
        root_id: usize,
    },
    Case {
        root_id: usize,
    },
    When {
        root_id: usize,
    },
}

struct AncestorFrame {
    kind: AncestorKind,
}

struct GemDeclaration {
    gem_name: Vec<u8>,
    line: usize,
    column: usize,
    conditional_root: Option<usize>,
    /// Number of opaque Block frames between this gem and its nearest conditional root.
    /// Must be 0 for conditional exemption (matches RuboCop's direct-child check).
    blocks_above_conditional: usize,
    /// Full source bytes of the CallNode (e.g., `gem "redcarpet"`). Used to replicate
    /// RuboCop's AST structural equality in `within_conditional?` where `branch == node`
    /// compares by structure, not identity.
    call_source: Vec<u8>,
    /// All conditional root IDs in the gem's ancestry, from innermost to outermost.
    /// Used for structural equality: when checking `within_conditional?`, the gem
    /// might be nested inside a conditional that is itself a branch of the root.
    ancestor_conditional_roots: Vec<usize>,
}

struct GemDeclarationVisitor<'a> {
    source: &'a SourceFile,
    declarations: Vec<GemDeclaration>,
    ancestors: Vec<AncestorFrame>,
    next_conditional_root_id: usize,
    pending_elsif_root: Option<usize>,
}

impl GemDeclarationVisitor<'_> {
    /// Find the nearest conditional root, count opaque Block frames, and collect
    /// all conditional root IDs in the ancestry chain.
    fn conditional_info(&self) -> (Option<usize>, usize, Vec<usize>) {
        let ancestors = self
            .ancestors
            .get(..self.ancestors.len().saturating_sub(1))
            .unwrap_or(&[]);
        let mut blocks_above = 0;
        let mut nearest: Option<usize> = None;
        let mut nearest_blocks = 0;
        let mut all_roots = Vec::new();
        for frame in ancestors.iter().rev() {
            match frame.kind {
                AncestorKind::BeginLike => continue,
                AncestorKind::Block => {
                    blocks_above += 1;
                    continue;
                }
                AncestorKind::If { root_id }
                | AncestorKind::When { root_id }
                | AncestorKind::Case { root_id } => {
                    if nearest.is_none() {
                        nearest = Some(root_id);
                        nearest_blocks = blocks_above;
                    }
                    all_roots.push(root_id);
                }
            }
        }
        (nearest, nearest_blocks, all_roots)
    }

    fn allocate_conditional_root_id(&mut self) -> usize {
        let id = self.next_conditional_root_id;
        self.next_conditional_root_id += 1;
        id
    }
}

fn gem_name_from_call(call: &ruby_prism::CallNode<'_>) -> Option<Vec<u8>> {
    if call.receiver().is_some() || call.name().as_slice() != b"gem" {
        return None;
    }
    let first_arg = util::first_positional_arg(call)?;
    util::string_value(&first_arg)
}

/// Check if a node is a "transparent" wrapper that should not create an
/// opaque block frame.
///
/// **Why CallNode is transparent:** In Parser gem's AST, a method call with a
/// block (e.g., `group :dev do gem "x" end`) is represented as a single
/// `(block (send ...) (args) body)` node. The `send` node is a child of the
/// `block` node, not a parent. In Prism, the structure is inverted: CallNode
/// contains a BlockNode child. Making CallNode transparent ensures that the
/// opaque/transparent decision is made at the BlockNode level (matching
/// Parser gem's structure).
///
/// **BlockNode is opaque:** In Parser gem, `block` type is NOT `begin_type?`,
/// so it stops the ancestor walk in `each_ancestor.find { |a| !a.begin_type? }`.
/// This means gems inside ANY block (even single-statement) have `:block` as
/// their first non-begin ancestor, NOT `:if`/`:when`. RuboCop's structural
/// equality (`child_nodes.include?`) still finds gems through blocks, but
/// that's handled separately via the `call_source` matching in Path 1.
fn is_transparent_node(node: &ruby_prism::Node<'_>) -> bool {
    node.as_statements_node().is_some()
        || node.as_begin_node().is_some()
        || node.as_else_node().is_some()
        || node.as_program_node().is_some()
        || node.as_call_node().is_some()
        || node.as_arguments_node().is_some()
}

impl<'pr> Visit<'pr> for GemDeclarationVisitor<'_> {
    fn visit_branch_node_enter(&mut self, node: ruby_prism::Node<'pr>) {
        // Transparent wrappers (StatementsNode, BeginNode, ElseNode, ProgramNode,
        // single-statement BlockNode) get BeginLike. Everything else gets Block
        // (opaque). Conditional nodes override their frame in specific visit methods.
        let kind = if is_transparent_node(&node) {
            AncestorKind::BeginLike
        } else {
            AncestorKind::Block
        };
        self.ancestors.push(AncestorFrame { kind });
    }

    fn visit_branch_node_leave(&mut self) {
        self.ancestors.pop();
    }

    fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
        // Both modifier and block `if` are conditional roots. In Parser gem,
        // modifier `if` produces the same `(if ...)` AST node, and the gem's
        // `each_ancestor.find { |a| !a.begin_type? }` stops at the `if` in
        // both cases. Using `take()` to consume pending_elsif_root prevents
        // it from leaking into nested ifs inside elsif/else bodies.
        let root_id = self
            .pending_elsif_root
            .take()
            .unwrap_or_else(|| self.allocate_conditional_root_id());
        if let Some(frame) = self.ancestors.last_mut() {
            frame.kind = AncestorKind::If { root_id };
        }

        self.visit(&node.predicate());
        if let Some(statements) = node.statements() {
            for statement in statements.body().iter() {
                self.visit(&statement);
            }
        }
        if let Some(subsequent) = node.subsequent() {
            let previous = self.pending_elsif_root;
            if subsequent.as_if_node().is_some() {
                self.pending_elsif_root = Some(root_id);
            } else {
                // Clear pending_elsif_root when entering an else clause to prevent
                // it from leaking into nested if statements inside the else body.
                self.pending_elsif_root = None;
            }
            self.visit(&subsequent);
            self.pending_elsif_root = previous;
        }
    }

    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
        // Both modifier and block `unless` are conditional roots, matching
        // Parser gem behavior where the gem's ancestor walk stops at `if`
        // (unless is represented as if with inverted condition in Parser).
        let root_id = self.allocate_conditional_root_id();
        if let Some(frame) = self.ancestors.last_mut() {
            frame.kind = AncestorKind::If { root_id };
        }

        self.visit(&node.predicate());
        if let Some(statements) = node.statements() {
            for statement in statements.body().iter() {
                self.visit(&statement);
            }
        }
        if let Some(else_clause) = node.else_clause() {
            if let Some(statements) = else_clause.statements() {
                for statement in statements.body().iter() {
                    self.visit(&statement);
                }
            }
        }
    }

    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode<'pr>) {
        let root_id = self.allocate_conditional_root_id();
        if let Some(frame) = self.ancestors.last_mut() {
            frame.kind = AncestorKind::Case { root_id };
        }
        ruby_prism::visit_case_node(self, node);
    }

    fn visit_when_node(&mut self, node: &ruby_prism::WhenNode<'pr>) {
        let case_root_id = self
            .ancestors
            .iter()
            .rev()
            .find_map(|frame| match frame.kind {
                AncestorKind::Case { root_id } => Some(root_id),
                _ => None,
            });
        if let Some(frame) = self.ancestors.last_mut() {
            frame.kind = case_root_id
                .map(|root_id| AncestorKind::When { root_id })
                .unwrap_or(AncestorKind::Block);
        }
        ruby_prism::visit_when_node(self, node);
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if let Some(gem_name) = gem_name_from_call(node) {
            let loc = node.message_loc().unwrap_or(node.location());
            let (line, column) = self.source.offset_to_line_col(loc.start_offset());
            let (conditional_root, blocks_above_conditional, ancestor_conditional_roots) =
                self.conditional_info();
            let call_loc = node.location();
            let call_source =
                self.source.as_bytes()[call_loc.start_offset()..call_loc.end_offset()].to_vec();
            self.declarations.push(GemDeclaration {
                gem_name,
                line,
                column,
                conditional_root,
                blocks_above_conditional,
                call_source,
                ancestor_conditional_roots,
            });
        }
        ruby_prism::visit_call_node(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(DuplicatedGem, "cops/bundler/duplicated_gem");
}
