use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

/// Checks for operators, variables, literals, lambda, proc and nonmutating
/// methods used in void context.
///
/// ## Investigation findings (2026-03-10)
///
/// Root causes of FPs (670) and FNs (683):
///
/// **FP: each block operator exemption missing** — RuboCop exempts void operators
/// (like `==`, `>=`, `<=>`) inside `each` blocks because the receiver may be an
/// Enumerator used as a filter. Our implementation flagged these operators.
///
/// **FN: void context for last expression** — RuboCop checks the last expression
/// in void contexts: `initialize`, setter methods (`def foo=`), `each`/`tap` blocks,
/// `for` loops, and `ensure` bodies. Our implementation always skipped the last
/// expression.
///
/// **FN: lambda/proc not detected** — `-> { }`, `lambda { }`, `proc { }` in void
/// context were not detected as void expressions.
///
/// **FN: `.freeze` on literal** — `'foo'.freeze` was not treated as entirely literal.
///
/// **FN: single-expression void blocks** — Single-expression `each`/`tap`/`for`
/// bodies and `ensure` bodies were not checked because we required `len > 1`.
///
/// **FP: binary operator with dot and no args** — `a.+` (no args, with dot) should
/// not be flagged; only `a.+(b)` should be.
///
/// Fixes applied: void context tracking via parent node inspection, each block
/// operator exemption, lambda/proc detection, `.freeze` on literal detection,
/// single-expression void body checking, dot-operator-no-args exemption.
pub struct Void;

impl Cop for Void {
    fn name(&self) -> &'static str {
        "Lint/Void"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
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
        let _check_methods = config.get_bool("CheckForMethodsWithNoSideEffects", false);

        let mut visitor = VoidVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
            in_each_block: false,
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

struct VoidVisitor<'a, 'src> {
    cop: &'a Void,
    source: &'src SourceFile,
    diagnostics: Vec<Diagnostic>,
    /// Whether we are currently inside an `each` block body.
    /// Used to exempt void operators (enumerator filter pattern).
    in_each_block: bool,
}

impl VoidVisitor<'_, '_> {
    fn check_void_expression(&mut self, stmt: &ruby_prism::Node<'_>) {
        if is_void_expression(stmt, self.in_each_block) {
            let loc = stmt.location();
            let (line, column) = self.source.offset_to_line_col(loc.start_offset());
            self.diagnostics.push(self.cop.diagnostic(
                self.source,
                line,
                column,
                "Void value expression detected.".to_string(),
            ));
        }
    }

    /// Check statements in a body, optionally including the last expression
    /// (when in void context).
    fn check_statements(&mut self, body: &[ruby_prism::Node<'_>], void_context: bool) {
        if body.is_empty() {
            return;
        }

        let check_up_to = if void_context {
            body.len()
        } else {
            body.len().saturating_sub(1)
        };

        for stmt in &body[..check_up_to] {
            self.check_void_expression(stmt);
        }
    }
}

/// Check if a node is an `each` or `tap` method call (for void context detection).
fn is_void_context_method(call: &ruby_prism::CallNode<'_>) -> bool {
    let name = call.name().as_slice();
    matches!(name, b"each" | b"tap")
}

/// Check if a node is specifically an `each` method call.
fn is_each_method(call: &ruby_prism::CallNode<'_>) -> bool {
    call.name().as_slice() == b"each"
}

/// Check if a def node is a void context (initialize or setter method).
fn is_void_def(node: &ruby_prism::DefNode<'_>) -> bool {
    let name = node.name().as_slice();
    name == b"initialize" || name.ends_with(b"=")
}

fn is_void_expression(node: &ruby_prism::Node<'_>, in_each_block: bool) -> bool {
    // Simple literals
    node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_self_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        // Variable reads
        || node.as_local_variable_read_node().is_some()
        || node.as_instance_variable_read_node().is_some()
        || node.as_class_variable_read_node().is_some()
        || node.as_global_variable_read_node().is_some()
        // Constants
        || node.as_constant_read_node().is_some()
        || node.as_constant_path_node().is_some()
        // Containers — only when ALL elements are literals (matches RuboCop's entirely_literal?)
        // Note: ranges are excluded (RuboCop's check_literal skips range_type?)
        || is_entirely_literal_container(node)
        || node.as_regular_expression_node().is_some()
        // Note: interpolated strings/symbols/regexps are NOT void (interpolation may have side effects)
        // Keywords
        || node.as_source_file_node().is_some()
        || node.as_source_line_node().is_some()
        || node.as_source_encoding_node().is_some()
        // defined?
        || node.as_defined_node().is_some()
        // Lambda/proc in void context
        || is_void_lambda_or_proc(node)
        // Literal.freeze
        || is_literal_freeze(node)
        // Operators (binary/unary) via CallNode — exempted in each blocks
        || is_void_operator(node, in_each_block)
}

/// Check if a node is a lambda literal `-> { }` that is NOT called.
/// A lambda literal is a `LambdaNode` in Prism. If it's called (e.g., `-> { }.call`),
/// it won't appear as a standalone LambdaNode — it will be wrapped in a CallNode.
fn is_void_lambda_or_proc(node: &ruby_prism::Node<'_>) -> bool {
    // -> { bar } — lambda literal
    if node.as_lambda_node().is_some() {
        return true;
    }

    // lambda { bar } or proc { bar } — these are CallNode with a block
    if let Some(call) = node.as_call_node() {
        let name = call.name().as_slice();
        if (name == b"lambda" || name == b"proc")
            && call.receiver().is_none()
            && call.block().is_some()
        {
            return true;
        }
        // Proc.new { bar }
        if name == b"new" {
            if let Some(recv) = call.receiver() {
                if let Some(c) = recv.as_constant_read_node() {
                    if c.name().as_slice() == b"Proc" && call.block().is_some() {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Check if a node is `literal.freeze` or `literal&.freeze` (entirely literal when frozen).
fn is_literal_freeze(node: &ruby_prism::Node<'_>) -> bool {
    if let Some(call) = node.as_call_node() {
        if call.name().as_slice() == b"freeze" {
            if let Some(recv) = call.receiver() {
                return is_entirely_literal(&recv);
            }
        }
    }
    false
}

/// Check if a node is an entirely-literal container (array or hash where all
/// elements are literals). Matches RuboCop's `entirely_literal?` method.
fn is_entirely_literal_container(node: &ruby_prism::Node<'_>) -> bool {
    if let Some(arr) = node.as_array_node() {
        arr.elements().iter().all(|e| is_entirely_literal(&e))
    } else if let Some(hash) = node.as_hash_node() {
        hash.elements().iter().all(|e| {
            if let Some(assoc) = e.as_assoc_node() {
                is_entirely_literal(&assoc.key()) && is_entirely_literal(&assoc.value())
            } else {
                false
            }
        })
    } else if let Some(hash) = node.as_keyword_hash_node() {
        hash.elements().iter().all(|e| {
            if let Some(assoc) = e.as_assoc_node() {
                is_entirely_literal(&assoc.key()) && is_entirely_literal(&assoc.value())
            } else {
                false
            }
        })
    } else {
        false
    }
}

/// Recursively check if a node is entirely literal (no variables, method calls, etc.)
fn is_entirely_literal(node: &ruby_prism::Node<'_>) -> bool {
    node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_regular_expression_node().is_some()
        || is_entirely_literal_container(node)
        || is_literal_freeze(node)
}

fn is_void_operator(node: &ruby_prism::Node<'_>, in_each_block: bool) -> bool {
    // Unwrap parentheses nodes to find the inner operator
    if let Some(parens) = node.as_parentheses_node() {
        if let Some(body) = parens.body() {
            if let Some(stmts) = body.as_statements_node() {
                let stmts_vec: Vec<_> = stmts.body().iter().collect();
                if stmts_vec.len() == 1 {
                    return is_void_operator(&stmts_vec[0], in_each_block);
                }
            }
        }
        return false;
    }

    if let Some(call) = node.as_call_node() {
        let name = call.name().as_slice();
        let is_operator = matches!(
            name,
            b"+" | b"-"
                | b"*"
                | b"/"
                | b"%"
                | b"**"
                | b"=="
                | b"==="
                | b"!="
                | b"<"
                | b">"
                | b"<="
                | b">="
                | b"<=>"
                | b"!"
                | b"~"
                | b"-@"
                | b"+@"
        );

        if !is_operator {
            return false;
        }

        // Exempt operators inside `each` blocks (enumerator filter pattern)
        if in_each_block {
            return false;
        }

        // Binary operators called with dot notation and no arguments are NOT void
        // e.g., `a.+` is not flagged, but `a.+(b)` is
        let is_unary = matches!(name, b"!" | b"~" | b"-@" | b"+@");
        if !is_unary && call.call_operator_loc().is_some() && call.arguments().is_none() {
            return false;
        }

        true
    } else {
        false
    }
}

impl<'pr> Visit<'pr> for VoidVisitor<'_, '_> {
    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode<'pr>) {
        let body: Vec<_> = node.body().iter().collect();
        // For regular statements nodes (not in special void contexts),
        // check all non-last expressions. Void context handling for
        // for/each/tap/ensure/initialize/setter is done in their respective
        // visit methods.
        self.check_statements(&body, false);
        ruby_prism::visit_statements_node(self, node);
    }

    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        if is_void_def(node) {
            // In void context methods (initialize, setters), ALL expressions
            // including the last are void.
            if let Some(body) = node.body() {
                if let Some(stmts) = body.as_statements_node() {
                    let body_stmts: Vec<_> = stmts.body().iter().collect();
                    // Check all including last (void context)
                    self.check_statements(&body_stmts, true);
                    // Visit children but don't re-check via visit_statements_node
                    // We need to visit into child nodes for nested blocks, etc.
                    for stmt in &body_stmts {
                        self.visit(stmt);
                    }
                    return;
                }
                // Single expression body (no StatementsNode wrapper) — check it
                self.check_void_expression(&body);
                self.visit(&body);
                return;
            }
        }
        // Non-void def: let the default visitor handle it
        ruby_prism::visit_def_node(self, node);
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        // Check for each/tap blocks with void context
        if is_void_context_method(node) {
            if let Some(block) = node.block() {
                if let Some(block_node) = block.as_block_node() {
                    let is_each = is_each_method(node);
                    let old_in_each = self.in_each_block;
                    if is_each {
                        self.in_each_block = true;
                    }

                    if let Some(body) = block_node.body() {
                        if let Some(stmts) = body.as_statements_node() {
                            let body_stmts: Vec<_> = stmts.body().iter().collect();
                            // Void context: check all including last
                            self.check_statements(&body_stmts, true);
                            for stmt in &body_stmts {
                                self.visit(stmt);
                            }
                        } else {
                            // Single expression block body — check it (void context)
                            self.check_void_expression(&body);
                            self.visit(&body);
                        }
                    }

                    self.in_each_block = old_in_each;

                    // Visit receiver and arguments but NOT the block body again
                    if let Some(recv) = node.receiver() {
                        self.visit(&recv);
                    }
                    if let Some(args) = node.arguments() {
                        for arg in args.arguments().iter() {
                            self.visit(&arg);
                        }
                    }
                    return;
                }
            }
        }
        ruby_prism::visit_call_node(self, node);
    }

    fn visit_for_node(&mut self, node: &ruby_prism::ForNode<'pr>) {
        // For loops are void context — check all expressions including last
        if let Some(stmts) = node.statements() {
            let body: Vec<_> = stmts.body().iter().collect();
            self.check_statements(&body, true);
            for stmt in &body {
                self.visit(stmt);
            }
        }
        // Visit collection
        self.visit(&node.collection());
    }

    fn visit_ensure_node(&mut self, node: &ruby_prism::EnsureNode<'pr>) {
        // Ensure bodies are void context — check all expressions including last
        if let Some(stmts) = node.statements() {
            let body: Vec<_> = stmts.body().iter().collect();
            self.check_statements(&body, true);
            for stmt in &body {
                self.visit(stmt);
            }
        }
        // Don't use default visitor since we handled statements manually
        // But we still need to visit rescue/else clauses if any
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(Void, "cops/lint/void");
}
