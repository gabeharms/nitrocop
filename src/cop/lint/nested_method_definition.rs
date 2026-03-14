use ruby_prism::Visit;

use crate::cop::node_type::{CALL_NODE, CLASS_NODE, DEF_NODE, MODULE_NODE, SINGLETON_CLASS_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Checks for nested method definitions.
///
/// ## Investigation findings (2026-03-14)
///
/// Root causes of FP/FN:
/// 1. **FN (def self.y):** nitrocop skipped ALL defs with receivers, but RuboCop only
///    skips defs whose receiver is a variable (local/ivar/cvar/gvar), constant, or
///    method call — NOT `self`. `def self.y` inside another def IS an offense.
/// 2. **FP (AllowedMethods/AllowedPatterns):** nitrocop checked these against the
///    *outer def's* name, but RuboCop checks them against *enclosing block call* names.
///    e.g., `AllowedMethods: ['has_many']` exempts `def` inside `has_many do...end` blocks.
/// 3. **FN (Data.define):** Missing `Data.define` as a scope-creating call (added in
///    Ruby 3.2, recognized by rubocop-ast's `class_constructor?`).
pub struct NestedMethodDefinition;

struct NestedDefFinder<'a> {
    found: Vec<usize>,
    skip_depth: usize,
    // Stack of booleans: true if the branch node was a scope-creating node
    scope_stack: Vec<bool>,
    // AllowedMethods/AllowedPatterns from config, checked against enclosing block method names
    allowed_methods: Option<&'a [String]>,
    allowed_patterns: Option<&'a [String]>,
}

/// Check if a `defs` node (def with receiver) has an allowed receiver type.
/// RuboCop allows `def obj.method` when the receiver is a variable (local,
/// instance, class, global), a constant, or a method call. The `self` keyword
/// is NOT allowed — `def self.method` nested inside another def IS an offense.
fn has_allowed_receiver(def_node: &ruby_prism::DefNode<'_>) -> bool {
    let receiver = match def_node.receiver() {
        Some(r) => r,
        None => return false, // No receiver = regular def, not a defs
    };
    // Variables: local, instance, class, global
    if receiver.as_local_variable_read_node().is_some()
        || receiver.as_instance_variable_read_node().is_some()
        || receiver.as_class_variable_read_node().is_some()
        || receiver.as_global_variable_read_node().is_some()
    {
        return true;
    }
    // Constants
    if receiver.as_constant_read_node().is_some() || receiver.as_constant_path_node().is_some() {
        return true;
    }
    // Method calls (e.g., def do_something.y)
    if receiver.as_call_node().is_some() {
        return true;
    }
    // Parenthesized expressions (e.g., def (do_something&.y).z)
    if receiver.as_parentheses_node().is_some() {
        return true;
    }
    // self is NOT allowed — def self.y inside def is still an offense
    false
}

impl<'pr> Visit<'pr> for NestedDefFinder<'_> {
    fn visit_branch_node_enter(&mut self, node: ruby_prism::Node<'pr>) {
        let is_scope = node.as_class_node().is_some()
            || node.as_module_node().is_some()
            || node.as_singleton_class_node().is_some()
            || is_scope_creating_call(&node)
            || is_allowed_method_call(&node, self.allowed_methods, self.allowed_patterns);
        self.scope_stack.push(is_scope);
        if is_scope {
            self.skip_depth += 1;
        }
        if self.skip_depth == 0 {
            if let Some(def_node) = node.as_def_node() {
                // Skip defs with allowed receiver types (variable, constant, call).
                // But NOT self — def self.method is still an offense.
                if !has_allowed_receiver(&def_node) {
                    self.found.push(node.location().start_offset());
                }
            }
        }
    }

    fn visit_branch_node_leave(&mut self) {
        if let Some(true) = self.scope_stack.pop() {
            self.skip_depth -= 1;
        }
    }
}

/// Check if a node is a scope-creating call like Module.new, Class.new,
/// define_method, class_eval, etc. that creates a new method scope.
fn is_scope_creating_call(node: &ruby_prism::Node<'_>) -> bool {
    let call = match node.as_call_node() {
        Some(c) => c,
        None => return false,
    };
    // Must have a block for defs inside to be in a new scope
    if call.block().is_none() {
        return false;
    }
    let method_name = call.name().as_slice();
    // Metaprogramming methods that create new scopes
    if matches!(
        method_name,
        b"define_method"
            | b"class_eval"
            | b"module_eval"
            | b"instance_eval"
            | b"class_exec"
            | b"module_exec"
            | b"instance_exec"
    ) {
        return true;
    }
    // Module.new, Class.new, Struct.new (also handles qualified like ::Module.new via constant_path_node)
    if method_name == b"new" {
        if let Some(receiver) = call.receiver() {
            if let Some(name) = crate::cop::util::constant_name(&receiver) {
                if name == b"Module" || name == b"Class" || name == b"Struct" {
                    return true;
                }
            }
        }
    }
    // Data.define (Ruby 3.2+, recognized by rubocop-ast class_constructor?)
    if method_name == b"define" {
        if let Some(receiver) = call.receiver() {
            if let Some(name) = crate::cop::util::constant_name(&receiver) {
                if name == b"Data" {
                    return true;
                }
            }
        }
    }
    false
}

/// Check if a call node with a block matches AllowedMethods or AllowedPatterns.
/// This is used to treat such blocks as scope-creating (suppressing the offense).
fn is_allowed_method_call(
    node: &ruby_prism::Node<'_>,
    allowed_methods: Option<&[String]>,
    allowed_patterns: Option<&[String]>,
) -> bool {
    let call = match node.as_call_node() {
        Some(c) => c,
        None => return false,
    };
    // Must have a block
    if call.block().is_none() {
        return false;
    }
    let method_name = std::str::from_utf8(call.name().as_slice()).unwrap_or("");
    if let Some(allowed) = allowed_methods {
        if allowed.iter().any(|m| m == method_name) {
            return true;
        }
    }
    if let Some(patterns) = allowed_patterns {
        if patterns.iter().any(|p| method_name.contains(p.as_str())) {
            return true;
        }
    }
    false
}

impl Cop for NestedMethodDefinition {
    fn name(&self) -> &'static str {
        "Lint/NestedMethodDefinition"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            CALL_NODE,
            CLASS_NODE,
            DEF_NODE,
            MODULE_NODE,
            SINGLETON_CLASS_NODE,
        ]
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
        let def_node = match node.as_def_node() {
            Some(n) => n,
            None => return,
        };

        let body = match def_node.body() {
            Some(b) => b,
            None => return,
        };

        // AllowedMethods/AllowedPatterns are checked against enclosing block call
        // names inside the finder, not against the outer def's name.
        let allowed_methods = config.get_string_array("AllowedMethods");
        let allowed_patterns = config.get_string_array("AllowedPatterns");

        let mut finder = NestedDefFinder {
            found: vec![],
            skip_depth: 0,
            scope_stack: vec![],
            allowed_methods: allowed_methods.as_deref(),
            allowed_patterns: allowed_patterns.as_deref(),
        };
        finder.visit(&body);

        diagnostics.extend(finder.found.iter().map(|&offset| {
            let (line, column) = source.offset_to_line_col(offset);
            self.diagnostic(
                source,
                line,
                column,
                "Method definitions must not be nested. Use `lambda` instead.".to_string(),
            )
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(NestedMethodDefinition, "cops/lint/nested_method_definition");
}
