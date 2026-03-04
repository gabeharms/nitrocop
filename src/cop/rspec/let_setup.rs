use crate::cop::util::RSPEC_DEFAULT_INCLUDE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;
use ruby_prism::Visit;
use std::collections::HashSet;

/// RSpec/LetSetup: Flag `let!` that is not referenced in tests (only used for side effects).
///
/// Investigation findings:
/// - The dominant false-positive pattern was inner `let!` overriding an outer `let!` with the
///   same name (e.g., `let!(:record) { nil }` inside a nested context that overrides a parent
///   `let!(:record) { create(...) }`). RuboCop skips these via `overrides_outer_let_bang?`.
/// - Implemented a recursive visitor that maintains a stack of ancestor `let!` names, so inner
///   overrides are correctly suppressed without needing parent node references.
pub struct LetSetup;

impl Cop for LetSetup {
    fn name(&self) -> &'static str {
        "RSpec/LetSetup"
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_DEFAULT_INCLUDE
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
        let mut visitor = LetSetupVisitor {
            cop: self,
            source,
            diagnostics,
            ancestor_let_bang_names: Vec::new(),
        };
        visitor.visit(&parse_result.node());
    }
}

struct LetSetupVisitor<'a> {
    cop: &'a LetSetup,
    source: &'a SourceFile,
    diagnostics: &'a mut Vec<Diagnostic>,
    /// Stack of sets: each set contains the `let!` names defined at that ancestor scope level.
    ancestor_let_bang_names: Vec<HashSet<Vec<u8>>>,
}

impl<'pr> LetSetupVisitor<'_> {
    fn process_example_group(&mut self, block_node: &ruby_prism::BlockNode<'pr>) {
        let body = match block_node.body() {
            Some(b) => b,
            None => return,
        };
        let stmts = match body.as_statements_node() {
            Some(s) => s,
            None => return,
        };

        // Collect let! names and all identifiers used in the same scope
        let mut let_bang_decls: Vec<(Vec<u8>, usize, usize)> = Vec::new();
        let mut used_names: HashSet<Vec<u8>> = HashSet::new();
        let mut this_scope_let_bang_names: HashSet<Vec<u8>> = HashSet::new();

        for stmt in stmts.body().iter() {
            if let Some(c) = stmt.as_call_node() {
                let m = c.name().as_slice();
                if m == b"let!" && c.receiver().is_none() {
                    if let Some(let_name) = extract_let_name(&c) {
                        let loc = c.location();
                        let (line, col) = self.source.offset_to_line_col(loc.start_offset());
                        this_scope_let_bang_names.insert(let_name.clone());
                        let_bang_decls.push((let_name, line, col));
                    }
                }
            }
            // Walk ALL siblings (including let! bodies) for identifier
            // collection. This matches RuboCop behavior where method_called?
            // searches the entire example group block, so a let! name used
            // inside a sibling let! body is not flagged.
            let mut collector = IdentifierCollector {
                names: &mut used_names,
            };
            collector.visit(&stmt);
        }

        for (let_name, line, col) in &let_bang_decls {
            // Skip if this let! overrides an outer let! with the same name
            if self.overrides_outer_let_bang(let_name) {
                continue;
            }
            if !used_names.contains(let_name) {
                self.diagnostics.push(self.cop.diagnostic(
                    self.source,
                    *line,
                    *col,
                    "Do not use `let!` to setup objects not referenced in tests.".to_string(),
                ));
            }
        }

        // Push this scope's let! names onto the ancestor stack, then recurse into children
        self.ancestor_let_bang_names.push(this_scope_let_bang_names);
        for stmt in stmts.body().iter() {
            self.visit(&stmt);
        }
        self.ancestor_let_bang_names.pop();
    }

    fn overrides_outer_let_bang(&self, name: &[u8]) -> bool {
        self.ancestor_let_bang_names
            .iter()
            .any(|scope| scope.contains(name))
    }
}

impl<'pr> Visit<'pr> for LetSetupVisitor<'_> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        let name = node.name().as_slice();
        if !is_example_group(name) {
            // Not an example group — continue default traversal
            ruby_prism::visit_call_node(self, node);
            return;
        }

        let block = match node.block() {
            Some(b) => b,
            None => return,
        };
        let block_node = match block.as_block_node() {
            Some(b) => b,
            None => return,
        };

        // Process this example group (handles let! detection + nested recursion)
        self.process_example_group(&block_node);
        // Don't call visit_call_node default — we already recursed into children
    }
}

fn extract_let_name(call: &ruby_prism::CallNode<'_>) -> Option<Vec<u8>> {
    let args = call.arguments()?;
    let first = args.arguments().iter().next()?;
    if let Some(sym) = first.as_symbol_node() {
        return Some(sym.unescaped().to_vec());
    }
    if let Some(s) = first.as_string_node() {
        return Some(s.unescaped().to_vec());
    }
    None
}

/// Walks the entire AST subtree, collecting all receiverless call names and
/// local variable reads. This ensures `let!` references are found in any
/// nested structure (conditionals, blocks, string interpolations, etc.).
struct IdentifierCollector<'a> {
    names: &'a mut HashSet<Vec<u8>>,
}

impl<'pr> Visit<'pr> for IdentifierCollector<'_> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if node.receiver().is_none() {
            self.names.insert(node.name().as_slice().to_vec());
        }
        ruby_prism::visit_call_node(self, node);
    }

    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        self.names.insert(node.name().as_slice().to_vec());
        ruby_prism::visit_local_variable_read_node(self, node);
    }
}

fn is_example_group(name: &[u8]) -> bool {
    matches!(
        name,
        b"describe"
            | b"context"
            | b"feature"
            | b"example_group"
            | b"xdescribe"
            | b"xcontext"
            | b"xfeature"
            | b"fdescribe"
            | b"fcontext"
            | b"ffeature"
            | b"shared_context"
            | b"shared_examples"
            | b"shared_examples_for"
            | b"it_behaves_like"
            | b"it_should_behave_like"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(LetSetup, "cops/rspec/let_setup");
}
