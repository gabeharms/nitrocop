use ruby_prism::Visit;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-11)
///
/// Corpus oracle reported FP=1, FN=0.
///
/// FP=1: the corpus false positive is an explicit `class << self` body that
/// contains `def ClassName.method`.
///
/// Attempted fixes:
/// - skipping singleton-class scopes in the visitor regressed the corpus gate
///   to `Actual=307` against `Expected=356` (49 FN)
/// - rewriting the cop to inspect only direct class/module body children still
///   regressed to `Actual=326` (30 FN)
/// - skipping only defs directly inside `class << self` while resetting for
///   nested class/module bodies still regressed to `Actual=308` (48 FN)
///
/// Reverted. A correct fix needs to identify the explicit singleton-class false
/// positive without suppressing the ordinary `def ClassName.method` shapes that
/// the original visitor already catches across the corpus.
///
/// Attempt #4 (2026-03-14): Track `in_singleton_class_self` flag in the visitor.
/// When inside `class << self`, skip `visit_def_node` entirely — any `def X.method`
/// inside a singleton class is already on the eigenclass, so the cop should not
/// suggest `self.method`. Nested class/module nodes reset the flag so that defs
/// inside nested bodies are still checked. This is narrower than attempts 1-3
/// because it only suppresses defs that are direct descendants of `class << self`
/// (not nested class/module bodies), matching RuboCop's behavior.
pub struct ClassMethods;

impl Cop for ClassMethods {
    fn name(&self) -> &'static str {
        "Style/ClassMethods"
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut visitor = ClassMethodsVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
            class_names: Vec::new(),
            in_singleton_class_self: false,
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

struct ClassMethodsVisitor<'a, 'src> {
    cop: &'a ClassMethods,
    source: &'src SourceFile,
    diagnostics: Vec<Diagnostic>,
    class_names: Vec<Vec<u8>>,
    in_singleton_class_self: bool,
}

impl<'pr> Visit<'pr> for ClassMethodsVisitor<'_, '_> {
    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
        let name = node.constant_path().location().as_slice().to_vec();
        self.class_names.push(name);
        let prev = self.in_singleton_class_self;
        self.in_singleton_class_self = false;
        if let Some(body) = node.body() {
            self.visit(&body);
        }
        self.in_singleton_class_self = prev;
        self.class_names.pop();
    }

    fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode<'pr>) {
        let name = node.constant_path().location().as_slice().to_vec();
        self.class_names.push(name);
        let prev = self.in_singleton_class_self;
        self.in_singleton_class_self = false;
        if let Some(body) = node.body() {
            self.visit(&body);
        }
        self.in_singleton_class_self = prev;
        self.class_names.pop();
    }

    fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode<'pr>) {
        let is_self = node.expression().as_self_node().is_some();
        let prev = self.in_singleton_class_self;
        if is_self {
            self.in_singleton_class_self = true;
        }
        if let Some(body) = node.body() {
            self.visit(&body);
        }
        self.in_singleton_class_self = prev;
    }

    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        if self.in_singleton_class_self {
            return;
        }
        let receiver = match node.receiver() {
            Some(r) => r,
            None => return,
        };

        let current_class = match self.class_names.last() {
            Some(n) => n,
            None => return,
        };

        let recv_bytes = receiver.location().as_slice();
        if recv_bytes == current_class.as_slice() {
            let method_name = node.name();
            let (line, column) = self
                .source
                .offset_to_line_col(receiver.location().start_offset());
            let msg = format!(
                "Use `self.{}` instead of `{}.{}`.",
                String::from_utf8_lossy(method_name.as_slice()),
                String::from_utf8_lossy(current_class),
                String::from_utf8_lossy(method_name.as_slice()),
            );
            self.diagnostics
                .push(self.cop.diagnostic(self.source, line, column, msg));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ClassMethods, "cops/style/class_methods");
}
