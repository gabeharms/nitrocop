use std::collections::{HashMap, HashSet};

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

/// ## Corpus investigation (2026-03-08, updated 2026-03-15)
///
/// Corpus oracle reported FP=2, FN=1.
///
/// FP=2: repeated requires whose return values are consumed by different
/// wrappers (`assert require(...)`, `result = require ...`) are not duplicates
/// in RuboCop because it keys by `node.parent` with `compare_by_identity`.
/// Two requires with different parent nodes (e.g. one wrapped in `assert`,
/// another in an assignment) are independent even if they share the same
/// argument string.
///
/// FN=1: `Kernel.require` calls were not detected as duplicates of plain
/// `require`. RuboCop's node matcher accepts `{nil? (const _ :Kernel)}` as
/// valid receivers.
///
/// Fix (2026-03-11): Accept `Kernel` as equivalent receiver for require calls.
/// Key duplicates by immediate parent node (tracked via `current_parent_offset`
/// during AST walk), matching RuboCop's `@required[node.parent]` behavior.
/// Each parent node gets its own `HashSet`, so wrapped requires with different
/// parents don't conflict.
///
/// Fix (2026-03-15): accept non-string first arguments such as `require x`.
/// RuboCop keys on `node.first_argument`, not just string literals, so repeated
/// local-variable reads inside rufo `.rb.spec` fixture files are duplicates.
/// Static strings still normalize to their unescaped value so `'foo'` and
/// `"foo"` collide like RuboCop; all other argument node types key on their
/// exact source slice with a distinct discriminator so `require 'foo'` and
/// `require foo` remain different.
pub struct DuplicateRequire;

impl Cop for DuplicateRequire {
    fn name(&self) -> &'static str {
        "Lint/DuplicateRequire"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::parse::codemap::CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut visitor = RequireVisitor {
            cop: self,
            source,
            // Per RuboCop: keyed by parent node identity.
            // We use the parent node's start offset as a proxy for identity.
            required: HashMap::new(),
            current_parent_offset: 0,
            diagnostics: Vec::new(),
            corrections: Vec::new(),
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
        if let Some(corr) = corrections {
            corr.extend(visitor.corrections);
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum RequireArgKey {
    String(Vec<u8>),
    Source(Vec<u8>),
}

/// Key: (method_name, normalized first-argument key). Value: set of seen keys per parent node.
type RequireKey = (Vec<u8>, RequireArgKey);

struct RequireVisitor<'a, 'src> {
    cop: &'a DuplicateRequire,
    source: &'src SourceFile,
    /// Seen requires keyed by parent node start offset (proxy for identity).
    required: HashMap<usize, HashSet<RequireKey>>,
    /// Start offset of the current parent node being visited.
    current_parent_offset: usize,
    diagnostics: Vec<Diagnostic>,
    corrections: Vec<crate::correction::Correction>,
}

impl RequireVisitor<'_, '_> {
    fn require_argument_key(&self, node: ruby_prism::Node<'_>) -> Option<RequireArgKey> {
        if let Some(string) = node.as_string_node() {
            return Some(RequireArgKey::String(string.unescaped().to_vec()));
        }

        let loc = node.location();
        Some(RequireArgKey::Source(
            self.source
                .as_bytes()
                .get(loc.start_offset()..loc.end_offset())?
                .to_vec(),
        ))
    }

    fn duplicate_line_removal_range(
        &self,
        node: &ruby_prism::CallNode<'_>,
    ) -> Option<(usize, usize)> {
        let loc = node.location();
        let bytes = self.source.as_bytes();
        let (line, _) = self.source.offset_to_line_col(loc.start_offset());
        let line_start = self.source.line_start_offset(line);
        let line_end = self
            .source
            .line_col_to_offset(line + 1, 0)
            .unwrap_or(bytes.len());

        let before = std::str::from_utf8(&bytes[line_start..loc.start_offset()]).ok()?;
        if !before.trim().is_empty() {
            return None;
        }

        let after = std::str::from_utf8(&bytes[loc.end_offset()..line_end]).ok()?;
        let after_trimmed = after.trim_start();
        if !(after_trimmed.is_empty() || after_trimmed.starts_with('#')) {
            return None;
        }

        Some((line_start, line_end))
    }

    fn check_require_call(&mut self, node: &ruby_prism::CallNode<'_>) {
        let method_name = node.name().as_slice();

        if method_name != b"require" && method_name != b"require_relative" {
            return;
        }

        // Accept receiverless calls and Kernel.require / Kernel.require_relative
        // Handles both ConstantReadNode (`Kernel`) and ConstantPathNode (`::Kernel`)
        if let Some(receiver) = node.receiver() {
            let is_kernel = if let Some(const_node) = receiver.as_constant_read_node() {
                const_node.name().as_slice() == b"Kernel"
            } else if let Some(const_path) = receiver.as_constant_path_node() {
                const_path
                    .name()
                    .map(|n| n.as_slice() == b"Kernel")
                    .unwrap_or(false)
            } else {
                false
            };
            if !is_kernel {
                return;
            }
        }

        if let Some(args) = node.arguments() {
            let arg_list = args.arguments();
            if arg_list.len() == 1 {
                if let Some(first) = arg_list.iter().next() {
                    if let Some(arg_key) = self.require_argument_key(first) {
                        let key = (method_name.to_vec(), arg_key);
                        let loc = node.location();
                        let parent_set =
                            self.required.entry(self.current_parent_offset).or_default();
                        if parent_set.contains(&key) {
                            let (line, column) = self.source.offset_to_line_col(loc.start_offset());
                            let mut diag = self.cop.diagnostic(
                                self.source,
                                line,
                                column,
                                "Duplicate `require` detected.".to_string(),
                            );

                            if let Some((start, end)) = self.duplicate_line_removal_range(node) {
                                self.corrections.push(crate::correction::Correction {
                                    start,
                                    end,
                                    replacement: String::new(),
                                    cop_name: self.cop.name(),
                                    cop_index: 0,
                                });
                                diag.corrected = true;
                            }
                            self.diagnostics.push(diag);
                        } else {
                            parent_set.insert(key);
                        }
                    }
                }
            }
        }
    }
}

impl<'pr> Visit<'pr> for RequireVisitor<'_, '_> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        // Check require with current parent offset (the node that contains this call).
        self.check_require_call(node);

        // When descending into child nodes (e.g. arguments of this call),
        // this call becomes the parent. This matches RuboCop's `node.parent`.
        let prev_parent = self.current_parent_offset;
        self.current_parent_offset = node.location().start_offset();
        ruby_prism::visit_call_node(self, node);
        self.current_parent_offset = prev_parent;
    }

    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode<'pr>) {
        let prev_parent = self.current_parent_offset;
        self.current_parent_offset = node.location().start_offset();
        ruby_prism::visit_statements_node(self, node);
        self.current_parent_offset = prev_parent;
    }

    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        let prev_parent = self.current_parent_offset;
        self.current_parent_offset = node.location().start_offset();
        ruby_prism::visit_local_variable_write_node(self, node);
        self.current_parent_offset = prev_parent;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(DuplicateRequire, "cops/lint/duplicate_require");
    crate::cop_autocorrect_fixture_tests!(DuplicateRequire, "cops/lint/duplicate_require");
}
