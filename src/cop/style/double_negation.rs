use ruby_prism::Visit;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Style/DoubleNegation: Avoid the use of double negation (`!!`).
///
/// Corpus investigation: 158 FPs, 84 FNs. Root causes:
///
/// FPs: The original text-based heuristic for `allowed_in_returns` only checked if
/// the next non-blank line started with `end`. This missed many valid return positions:
/// - `!!` inside if/elsif/else/unless/case/when branches at end of method
/// - `!!` inside rescue/ensure bodies at end of method
/// - `!!` in define_method/define_singleton_method blocks
/// - `!!` with explicit `return` keyword
/// - `!!` as part of a larger expression (e.g., `!!foo || bar`) at end of method
///
/// FNs: The text heuristic wrongly suppressed offenses when `end` followed on the
/// next line but the `!!` was actually inside a hash/array value (always an offense),
/// or when `!!` was not the last expression in the method body.
///
/// Fix: Replaced text-based heuristic with AST-based analysis using a custom visitor.
/// The visitor tracks whether each `!!` node is in a "return position" by walking
/// the AST with a set of byte offsets marking last-expression positions within the
/// current method body. Handles rescue/ensure, conditionals (if/unless/case), and
/// define_method/define_singleton_method blocks. Hash/array values containing `!!`
/// are always flagged as offenses regardless of return position.
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
            return_position_ranges: Vec::new(),
            hash_array_depth: 0,
            in_def: false,
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

struct DoubleNegationVisitor<'a> {
    cop: &'a DoubleNegation,
    source: &'a SourceFile,
    diagnostics: Vec<Diagnostic>,
    enforced_style: &'a str,
    /// Stack of byte-offset ranges that represent "return positions".
    /// Each range is (start_offset, end_offset) of an expression that is in return position.
    return_position_ranges: Vec<(usize, usize)>,
    /// Nesting depth inside hash/array nodes.
    hash_array_depth: u32,
    /// Whether we're inside a def/define_method body (at any depth).
    in_def: bool,
}

impl DoubleNegationVisitor<'_> {
    /// Check if a given offset is in a return position.
    fn is_in_return_position(&self, offset: usize) -> bool {
        for &(start, end) in &self.return_position_ranges {
            if offset >= start && offset < end {
                return true;
            }
        }
        false
    }

    /// Check if the !! call is preceded by the `return` keyword.
    fn is_after_return_keyword(&self, node: &ruby_prism::CallNode<'_>) -> bool {
        let start = node.location().start_offset();
        let src = self.source.as_bytes();
        if start >= 7 {
            let prefix = &src[..start];
            let trimmed = prefix.trim_ascii_end();
            if trimmed.ends_with(b"return") {
                // Make sure 'return' is a keyword, not part of another identifier
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

            // Check if in return position AND not inside a hash/array
            if self.in_def
                && self.hash_array_depth == 0
                && self.is_in_return_position(node.location().start_offset())
            {
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

    /// Collect return-position ranges from a body node (the body of a def, block, etc).
    fn collect_return_ranges(&self, body: &ruby_prism::Node<'_>) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();
        self.collect_return_ranges_from_node(body, &mut ranges);
        ranges
    }

    fn collect_return_ranges_from_node(
        &self,
        node: &ruby_prism::Node<'_>,
        ranges: &mut Vec<(usize, usize)>,
    ) {
        // Handle StatementsNode: last statement is the return value
        if let Some(stmts) = node.as_statements_node() {
            self.collect_from_statements(&stmts, ranges);
            return;
        }

        // Handle BeginNode: may have rescue/ensure
        if let Some(begin) = node.as_begin_node() {
            if begin.rescue_clause().is_some() || begin.ensure_clause().is_some() {
                if let Some(stmts) = begin.statements() {
                    self.collect_from_statements(&stmts, ranges);
                }
            } else if let Some(stmts) = begin.statements() {
                self.collect_from_statements(&stmts, ranges);
            }
            return;
        }

        // Handle RescueNode (inside def body with rescue)
        if let Some(rescue) = node.as_rescue_node() {
            if let Some(stmts) = rescue.statements() {
                self.collect_from_statements(&stmts, ranges);
            }
            return;
        }

        // Single expression body - the expression itself is a return value
        self.add_return_range_for_expr(node, ranges);
    }

    /// Extract the last statement from a StatementsNode and add its return ranges.
    fn collect_from_statements(
        &self,
        stmts: &ruby_prism::StatementsNode<'_>,
        ranges: &mut Vec<(usize, usize)>,
    ) {
        let body: Vec<_> = stmts.body().iter().collect();
        if let Some(last) = body.last() {
            self.add_return_range_for_expr(last, ranges);
        }
    }

    /// Add return position ranges for a single expression.
    /// If the expression is a conditional, recursively find the last expr in each branch.
    fn add_return_range_for_expr(
        &self,
        node: &ruby_prism::Node<'_>,
        ranges: &mut Vec<(usize, usize)>,
    ) {
        // If it's a conditional, each branch's last expression is a return position
        if let Some(if_node) = node.as_if_node() {
            self.add_return_ranges_from_if(&if_node, ranges);
            return;
        }
        if let Some(unless_node) = node.as_unless_node() {
            if let Some(stmts) = unless_node.statements() {
                self.collect_from_statements(&stmts, ranges);
            }
            if let Some(else_clause) = unless_node.else_clause() {
                if let Some(stmts) = else_clause.statements() {
                    self.collect_from_statements(&stmts, ranges);
                }
            }
            return;
        }
        if let Some(case_node) = node.as_case_node() {
            self.add_return_ranges_from_case(&case_node, ranges);
            return;
        }
        if let Some(case_match) = node.as_case_match_node() {
            self.add_return_ranges_from_case_match(&case_match, ranges);
            return;
        }

        // For a plain expression, the entire expression range is a return position
        let start = node.location().start_offset();
        let end = node.location().end_offset();
        ranges.push((start, end));
    }

    fn add_return_ranges_from_if(
        &self,
        if_node: &ruby_prism::IfNode<'_>,
        ranges: &mut Vec<(usize, usize)>,
    ) {
        // "then" branch
        if let Some(stmts) = if_node.statements() {
            self.collect_from_statements(&stmts, ranges);
        }
        // "else" branch (may be another if for elsif)
        if let Some(subsequent) = if_node.subsequent() {
            if let Some(nested_if) = subsequent.as_if_node() {
                self.add_return_ranges_from_if(&nested_if, ranges);
            } else if let Some(else_node) = subsequent.as_else_node() {
                if let Some(stmts) = else_node.statements() {
                    self.collect_from_statements(&stmts, ranges);
                }
            }
        }
    }

    fn add_return_ranges_from_case(
        &self,
        case_node: &ruby_prism::CaseNode<'_>,
        ranges: &mut Vec<(usize, usize)>,
    ) {
        for condition in case_node.conditions().iter() {
            if let Some(when_node) = condition.as_when_node() {
                if let Some(stmts) = when_node.statements() {
                    self.collect_from_statements(&stmts, ranges);
                }
            }
        }
        if let Some(else_clause) = case_node.else_clause() {
            if let Some(stmts) = else_clause.statements() {
                self.collect_from_statements(&stmts, ranges);
            }
        }
    }

    fn add_return_ranges_from_case_match(
        &self,
        case_match: &ruby_prism::CaseMatchNode<'_>,
        ranges: &mut Vec<(usize, usize)>,
    ) {
        for condition in case_match.conditions().iter() {
            if let Some(in_node) = condition.as_in_node() {
                if let Some(stmts) = in_node.statements() {
                    self.collect_from_statements(&stmts, ranges);
                }
            }
        }
        if let Some(else_clause) = case_match.else_clause() {
            if let Some(stmts) = else_clause.statements() {
                self.collect_from_statements(&stmts, ranges);
            }
        }
    }

    /// Enter a method body: compute return ranges, push them, visit body, pop them.
    fn with_def_body<F>(&mut self, body: Option<ruby_prism::Node<'_>>, visit_fn: F)
    where
        F: FnOnce(&mut Self),
    {
        let prev_ranges_len = self.return_position_ranges.len();
        let prev_in_def = self.in_def;

        if let Some(ref body_node) = body {
            let new_ranges = self.collect_return_ranges(body_node);
            self.return_position_ranges.extend(new_ranges);
        }
        self.in_def = true;

        visit_fn(self);

        self.return_position_ranges.truncate(prev_ranges_len);
        self.in_def = prev_in_def;
    }
}

impl<'pr> Visit<'pr> for DoubleNegationVisitor<'_> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        self.check_double_negation(node);

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
                    return;
                }
            }
        }

        ruby_prism::visit_call_node(self, node);
    }

    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        let body = node.body();
        self.with_def_body(body, |this| {
            ruby_prism::visit_def_node(this, node);
        });
    }

    fn visit_hash_node(&mut self, node: &ruby_prism::HashNode<'pr>) {
        self.hash_array_depth += 1;
        ruby_prism::visit_hash_node(self, node);
        self.hash_array_depth -= 1;
    }

    fn visit_keyword_hash_node(&mut self, node: &ruby_prism::KeywordHashNode<'pr>) {
        self.hash_array_depth += 1;
        ruby_prism::visit_keyword_hash_node(self, node);
        self.hash_array_depth -= 1;
    }

    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode<'pr>) {
        self.hash_array_depth += 1;
        ruby_prism::visit_array_node(self, node);
        self.hash_array_depth -= 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(DoubleNegation, "cops/style/double_negation");
}
