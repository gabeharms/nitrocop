use ruby_prism::Visit;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Style/DoubleNegation: Avoid the use of double negation (`!!`).
///
/// Corpus investigation (round 3): 70 FPs, 40 FNs.
///
/// Root cause of FPs: nitrocop used byte-range matching for return positions and
/// unconditionally excluded `!!` inside hash/array/keyword_hash nodes. RuboCop uses
/// a much looser line-based check: if `!!` is on or after the first line of the def
/// body's last statement, it's allowed — regardless of whether it's inside a hash
/// value, method argument, XOR expression, etc. RuboCop only excludes hash/array
/// when the last_child of the def body itself is a pair/hash node or the parent is
/// an array type (i.e., the method returns a literal hash or array).
///
/// Root cause of FNs: nitrocop recursively marked all branch endpoints in nested
/// conditionals as return positions. RuboCop uses a stricter check for nested
/// conditionals: it finds the innermost conditional ancestor and checks if that
/// conditional's last line >= the def body's last child's last line. Additionally,
/// when the `!!` node's parent is a statement sequence (begin_type?), RuboCop checks
/// that `!!` is on the last line of that sequence — otherwise it's not a return value
/// even if it's inside a return-position conditional.
///
/// Fix (round 3): Replaced byte-range approach with line-based checks matching
/// RuboCop's `end_of_method_definition?` / `double_negative_condition_return_value?`
/// logic. Tracks def body info (last child first/last line, hash/array type) and
/// conditional ancestor last lines on stacks.
///
/// Corpus investigation (round 4): 28 FPs, 25 FNs.
///
/// FP root cause: The `stmts_last_line` check (for `begin_type?` parent) was applied
/// unconditionally. In RuboCop, `find_parent_not_enumerable` walks up from the `!!`
/// node skipping pair/hash/array; if the non-enumerable parent is NOT `begin_type?`
/// (e.g., it's a send/if/assignment), the line check is skipped. Additionally, Prism
/// always wraps branch bodies in StatementsNode even for single-statement branches,
/// while Parser AST only creates `begin` wrappers for multi-statement bodies. This
/// caused `!!` inside hash values, method call args, and assignments within
/// conditional branches to be incorrectly flagged.
///
/// FN root cause: For single-statement method bodies, RuboCop calls
/// `node.child_nodes.last` on the expression itself (not just the statements
/// wrapper), which digs into the expression's last child. For a method call, this
/// reaches the keyword hash args. nitrocop wasn't doing this dig-in, so `!!` inside
/// hash args of a single-statement method call was treated as return position.
///
/// Fix (round 4): (1) Track `parent_is_statements` flag — only true when the
/// StatementsNode has >1 statement (matching Parser's `begin` wrapper behavior).
/// Reset to false when entering CallNode children. Only apply the `stmts_last_line`
/// check when true. (2) Added `parser_last_child()` to dig into single-statement
/// method bodies (CallNode → last arg), matching RuboCop's `child_nodes.last`.
pub struct DoubleNegation;

impl Cop for DoubleNegation {
    fn name(&self) -> &'static str {
        "Style/DoubleNegation"
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
        let enforced_style = config.get_str("EnforcedStyle", "allowed_in_returns");
        let mut visitor = DoubleNegationVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
            enforced_style,
            def_info_stack: Vec::new(),
            conditional_last_line_stack: Vec::new(),
            statements_last_line_stack: Vec::new(),
            parent_is_statements: false,
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

/// Info about the enclosing method definition's body.
#[derive(Clone)]
struct DefBodyInfo {
    /// First line of the last child of the def body (1-indexed).
    last_child_first_line: usize,
    /// Last line of the last child of the def body (1-indexed).
    last_child_last_line: usize,
    /// Whether the last child is a hash/pair node (literal hash return).
    last_child_is_hash_or_pair: bool,
    /// Whether the last child is an array or its parent is an array.
    last_child_parent_is_array: bool,
}

struct DoubleNegationVisitor<'a> {
    cop: &'a DoubleNegation,
    source: &'a SourceFile,
    diagnostics: Vec<Diagnostic>,
    enforced_style: &'a str,
    /// Stack of def body info (innermost at top).
    def_info_stack: Vec<DefBodyInfo>,
    /// Stack of conditional ancestor last lines (innermost at top).
    conditional_last_line_stack: Vec<usize>,
    /// Stack of enclosing statements-node last lines. Used for the
    /// `begin_type?` parent check in `double_negative_condition_return_value?`.
    statements_last_line_stack: Vec<usize>,
    /// Whether the current node's non-enumerable parent (skipping pair/hash/
    /// array/keyword_hash) is a StatementsNode. Only when true should the
    /// stmts_last_line check apply — matching RuboCop's
    /// `find_parent_not_enumerable` + `begin_type?` check.
    parent_is_statements: bool,
}

impl DoubleNegationVisitor<'_> {
    fn line_of_offset(&self, offset: usize) -> usize {
        let (line, _) = self.source.offset_to_line_col(offset);
        line
    }

    fn last_line_of_node(&self, start: usize, end: usize) -> usize {
        let adjusted = if end > start { end - 1 } else { start };
        self.line_of_offset(adjusted)
    }

    /// Check if the !! call is preceded by the `return` keyword.
    fn is_after_return_keyword(&self, node: &ruby_prism::CallNode<'_>) -> bool {
        let start = node.location().start_offset();
        let src = self.source.as_bytes();
        if start >= 7 {
            let prefix = &src[..start];
            let trimmed = prefix.trim_ascii_end();
            if trimmed.ends_with(b"return") {
                let before_return = trimmed.len() - 6;
                if before_return == 0 {
                    return true;
                }
                let c = trimmed[before_return - 1];
                if !c.is_ascii_alphanumeric() && c != b'_' {
                    return true;
                }
            }
        }
        false
    }

    fn check_double_negation(&mut self, node: &ruby_prism::CallNode<'_>) {
        // Must be the `!` method
        if node.name().as_slice() != b"!" {
            return;
        }

        // Check the message_loc to ensure it's `!` not `not`
        if let Some(msg_loc) = node.message_loc() {
            if msg_loc.as_slice() == b"not" {
                return;
            }
        }

        // Receiver must also be a `!` call
        let receiver = match node.receiver() {
            Some(r) => r,
            None => return,
        };

        let inner_call = match receiver.as_call_node() {
            Some(c) => c,
            None => return,
        };

        if inner_call.name().as_slice() != b"!" {
            return;
        }

        // Verify inner is also `!` not `not`
        if let Some(msg_loc) = inner_call.message_loc() {
            if msg_loc.as_slice() == b"not" {
                return;
            }
        }

        // For "allowed_in_returns" style, skip if in return position
        if self.enforced_style == "allowed_in_returns" {
            // Check explicit `return` keyword
            if self.is_after_return_keyword(node) {
                return;
            }

            // Check if in return position using line-based logic matching RuboCop
            if self.is_end_of_method_definition(node) {
                return;
            }
        }

        let loc = node.location();
        let (line, column) = self.source.offset_to_line_col(loc.start_offset());
        self.diagnostics.push(self.cop.diagnostic(
            self.source,
            line,
            column,
            "Avoid the use of double negation (`!!`).".to_string(),
        ));
    }

    /// RuboCop-compatible `end_of_method_definition?` check.
    fn is_end_of_method_definition(&self, node: &ruby_prism::CallNode<'_>) -> bool {
        let def_info = match self.def_info_stack.last() {
            Some(info) => info,
            None => return false,
        };

        let node_line = self.line_of_offset(node.location().start_offset());

        // If inside a conditional ancestor, use RuboCop's
        // double_negative_condition_return_value? logic
        if let Some(&cond_last_line) = self.conditional_last_line_stack.last() {
            // RuboCop: find_parent_not_enumerable → if parent.begin_type?
            // Only apply the statements line check when the !! node's
            // non-enumerable parent IS a StatementsNode (begin_type? in
            // Parser AST). When !! is inside another expression (method call,
            // assignment, hash value, etc.), skip this check.
            if self.parent_is_statements {
                if let Some(&stmts_last_line) = self.statements_last_line_stack.last() {
                    // The "parent" of the !! node in RuboCop terms:
                    // If the parent is a begin node (statement sequence), check if !! is
                    // on the last line of that sequence. This prevents treating `!!foo`
                    // followed by `bar` as a return value even if inside a return-position
                    // conditional.
                    if stmts_last_line != node_line {
                        // !! is not on the last line of its enclosing statements → not a return
                        return false;
                    }
                }
            }
            // Check if the conditional covers the def body's last child
            return def_info.last_child_last_line <= cond_last_line;
        }

        // Not inside a conditional — use the simple line-based check
        // RuboCop: if last_child is pair/hash or parent is array → always offense
        if def_info.last_child_is_hash_or_pair || def_info.last_child_parent_is_array {
            return false;
        }

        // RuboCop: last_child.first_line <= node.first_line
        def_info.last_child_first_line <= node_line
    }

    /// Find the "last child" of a body node, recursing through rescue/ensure.
    fn find_last_child_info(&self, node: &ruby_prism::Node<'_>) -> Option<DefBodyInfo> {
        // Handle StatementsNode: get last child
        if let Some(stmts) = node.as_statements_node() {
            return self.find_last_child_of_stmts(&stmts);
        }

        // Handle BeginNode: may have rescue/ensure
        // RuboCop recurses: rescue → body, ensure → first child
        // In Prism, BeginNode wraps the whole structure; main body is in statements()
        if let Some(begin) = node.as_begin_node() {
            if let Some(stmts) = begin.statements() {
                return self.find_last_child_of_stmts(&stmts);
            }
            return None;
        }

        // Default: this node itself is the "last child"
        Some(self.node_to_def_body_info(node))
    }

    fn find_last_child_of_stmts(
        &self,
        stmts: &ruby_prism::StatementsNode<'_>,
    ) -> Option<DefBodyInfo> {
        let body: Vec<_> = stmts.body().iter().collect();
        let last = body.last()?;

        // In RuboCop's Parser AST, a single-expression def body doesn't get a
        // `begin` wrapper, so `find_last_child` calls `child_nodes.last` directly
        // on the expression (hash → last pair, array → last element, send → last
        // arg). With multiple statements there IS a `begin` wrapper and
        // `child_nodes.last` returns the last statement without digging in.
        //
        // Prism always wraps in StatementsNode. To match RuboCop, when there's
        // exactly one statement, dig into its last child.
        if body.len() == 1 {
            if let Some(hash) = last.as_hash_node() {
                let elements: Vec<_> = hash.elements().iter().collect();
                if let Some(last_pair) = elements.last() {
                    return Some(self.node_to_def_body_info(last_pair));
                }
                // Empty hash — treat the hash itself as last child
                return Some(self.node_to_def_body_info(last));
            }
            if let Some(array) = last.as_array_node() {
                let elements: Vec<_> = array.elements().iter().collect();
                if let Some(last_elem) = elements.last() {
                    // Set parent_is_array = true since we dug into the array
                    let mut info = self.node_to_def_body_info(last_elem);
                    info.last_child_parent_is_array = true;
                    return Some(info);
                }
                return Some(self.node_to_def_body_info(last));
            }
            // For other single-statement bodies (method calls, assignments, etc.),
            // dig into the "last child" to match Parser AST's child_nodes.last.
            // For a CallNode, the last child is the last argument (or block body).
            // If that last child is a hash/keyword_hash, it causes the offense.
            if let Some(last_child) = self.parser_last_child(last) {
                return Some(self.node_to_def_body_info(&last_child));
            }
        }

        Some(self.node_to_def_body_info(last))
    }

    /// Approximate Parser AST's `node.child_nodes.last` for a given Prism node.
    /// Returns the "last child" in Parser AST terms, which for call nodes is
    /// the last argument (or block body), for assignments is the value, etc.
    fn parser_last_child<'n>(&self, node: &ruby_prism::Node<'n>) -> Option<ruby_prism::Node<'n>> {
        // CallNode: last argument (keyword hash or positional)
        if let Some(call) = node.as_call_node() {
            // In Parser AST, blocks wrap the send: (block (send ...) (args) body).
            // child_nodes.last of a block is the body. But for
            // find_last_child purposes, we want the send's last arg because
            // RuboCop calls find_last_child(def_node.body) where def_node.body
            // is the send (for non-block) or the block (for block calls).
            // For block calls in Parser: child_nodes.last = body of block.
            // For regular calls: child_nodes.last = last argument.
            if call.block().is_some() {
                // Block call: in Parser AST, the block node wraps the send.
                // child_nodes.last of the block = block body.
                // For our purposes, treat block body as the last child.
                // But RuboCop only gets here if the block IS the body expression,
                // and child_nodes.last of a block = its body.
                // We can just return None to fall through to the default behavior.
                return None;
            }
            if let Some(args) = call.arguments() {
                let arg_list: Vec<_> = args.arguments().iter().collect();
                return arg_list.into_iter().last();
            }
            // No arguments: last child is receiver (if any)
            return call.receiver();
        }

        // LocalVariableWriteNode: value is the last child
        if let Some(lvar) = node.as_local_variable_write_node() {
            return Some(lvar.value());
        }

        // InstanceVariableWriteNode
        if let Some(ivar) = node.as_instance_variable_write_node() {
            return Some(ivar.value());
        }

        None
    }

    fn node_to_def_body_info(&self, node: &ruby_prism::Node<'_>) -> DefBodyInfo {
        let first_line = self.line_of_offset(node.location().start_offset());
        let last_line =
            self.last_line_of_node(node.location().start_offset(), node.location().end_offset());

        let is_hash_or_pair = node.as_hash_node().is_some()
            || node.as_keyword_hash_node().is_some()
            || node.as_assoc_node().is_some()
            || node.as_assoc_splat_node().is_some();

        // parent_is_array is set by the caller when digging into an array;
        // by default it's false
        DefBodyInfo {
            last_child_first_line: first_line,
            last_child_last_line: last_line,
            last_child_is_hash_or_pair: is_hash_or_pair,
            last_child_parent_is_array: false,
        }
    }

    /// Enter a method body: compute last-child info, push to stack, visit body, pop.
    fn with_def_body<F>(&mut self, body: Option<ruby_prism::Node<'_>>, visit_fn: F)
    where
        F: FnOnce(&mut Self),
    {
        let prev_def_len = self.def_info_stack.len();

        if let Some(ref body_node) = body {
            if let Some(info) = self.find_last_child_info(body_node) {
                self.def_info_stack.push(info);
            }
        }

        // Save and clear conditional/statements stacks — these don't cross def boundaries
        let saved_cond = std::mem::take(&mut self.conditional_last_line_stack);
        let saved_stmts = std::mem::take(&mut self.statements_last_line_stack);
        let saved_parent_is_statements = self.parent_is_statements;
        self.parent_is_statements = false;

        visit_fn(self);

        self.def_info_stack.truncate(prev_def_len);
        self.conditional_last_line_stack = saved_cond;
        self.statements_last_line_stack = saved_stmts;
        self.parent_is_statements = saved_parent_is_statements;
    }
}

impl<'pr> Visit<'pr> for DoubleNegationVisitor<'_> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        self.check_double_negation(node);

        // After checking this node, clear parent_is_statements for children.
        // Children of a call node are not direct children of the StatementsNode.
        let saved_parent = self.parent_is_statements;
        self.parent_is_statements = false;

        // Check if this is a define_method or define_singleton_method call with a block
        if let Some(block) = node.block() {
            if let Some(block_node) = block.as_block_node() {
                let method_name = node.name().as_slice();
                if (method_name == b"define_method" || method_name == b"define_singleton_method")
                    && node.receiver().is_none()
                {
                    let body = block_node.body();
                    self.with_def_body(body, |this| {
                        ruby_prism::visit_call_node(this, node);
                    });
                    self.parent_is_statements = saved_parent;
                    return;
                }
            }
        }

        ruby_prism::visit_call_node(self, node);
        self.parent_is_statements = saved_parent;
    }

    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        let body = node.body();
        self.with_def_body(body, |this| {
            ruby_prism::visit_def_node(this, node);
        });
    }

    fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
        if !self.def_info_stack.is_empty() {
            let last_line = self
                .last_line_of_node(node.location().start_offset(), node.location().end_offset());
            self.conditional_last_line_stack.push(last_line);
            // Clear statements stack: the condition is not inside a StatementsNode
            // within this conditional, so the begin_type? check should not apply.
            // StatementsNodes inside branches will re-push as they're visited.
            let saved_stmts = std::mem::take(&mut self.statements_last_line_stack);
            ruby_prism::visit_if_node(self, node);
            self.statements_last_line_stack = saved_stmts;
            self.conditional_last_line_stack.pop();
        } else {
            ruby_prism::visit_if_node(self, node);
        }
    }

    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
        if !self.def_info_stack.is_empty() {
            let last_line = self
                .last_line_of_node(node.location().start_offset(), node.location().end_offset());
            self.conditional_last_line_stack.push(last_line);
            let saved_stmts = std::mem::take(&mut self.statements_last_line_stack);
            ruby_prism::visit_unless_node(self, node);
            self.statements_last_line_stack = saved_stmts;
            self.conditional_last_line_stack.pop();
        } else {
            ruby_prism::visit_unless_node(self, node);
        }
    }

    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode<'pr>) {
        if !self.def_info_stack.is_empty() {
            let last_line = self
                .last_line_of_node(node.location().start_offset(), node.location().end_offset());
            self.conditional_last_line_stack.push(last_line);
            let saved_stmts = std::mem::take(&mut self.statements_last_line_stack);
            ruby_prism::visit_case_node(self, node);
            self.statements_last_line_stack = saved_stmts;
            self.conditional_last_line_stack.pop();
        } else {
            ruby_prism::visit_case_node(self, node);
        }
    }

    fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode<'pr>) {
        if !self.def_info_stack.is_empty() {
            let last_line = self
                .last_line_of_node(node.location().start_offset(), node.location().end_offset());
            self.conditional_last_line_stack.push(last_line);
            let saved_stmts = std::mem::take(&mut self.statements_last_line_stack);
            ruby_prism::visit_case_match_node(self, node);
            self.statements_last_line_stack = saved_stmts;
            self.conditional_last_line_stack.pop();
        } else {
            ruby_prism::visit_case_match_node(self, node);
        }
    }

    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode<'pr>) {
        if !self.def_info_stack.is_empty() {
            let last_line = self
                .last_line_of_node(node.location().start_offset(), node.location().end_offset());
            self.statements_last_line_stack.push(last_line);

            // In Parser AST, only multi-statement bodies get a `begin` wrapper.
            // Single-statement bodies are unwrapped. Prism always wraps in
            // StatementsNode. To match RuboCop's `begin_type?` check, only set
            // parent_is_statements when there are multiple statements.
            let stmt_count = node.body().iter().count();
            let saved = self.parent_is_statements;
            self.parent_is_statements = stmt_count > 1;
            ruby_prism::visit_statements_node(self, node);
            self.parent_is_statements = saved;

            self.statements_last_line_stack.pop();
        } else {
            ruby_prism::visit_statements_node(self, node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(DoubleNegation, "cops/style/double_negation");
}
