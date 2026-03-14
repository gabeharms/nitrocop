use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

/// Layout/LineEndStringConcatenationIndentation
///
/// ## Investigation findings (2026-03-14)
///
/// **Root cause of 28 FNs:** The visitor only explicitly set `ParentType::Other`
/// for a handful of node types (CallNode, LocalVariableWriteNode, etc.). Any
/// node type NOT overridden (e.g., `IndexOperatorWriteNode`,
/// `LocalVariableOperatorWriteNode`, `CallOperatorWriteNode`,
/// `LocalVariableOrWriteNode`, `ParenthesesNode`, etc.) inherited the parent
/// type from its enclosing scope. Inside a `def` body, this meant operator
/// assignment nodes like `x += "a" \ "b"` inherited `ParentType::Def`, causing
/// `always_indented?` to be true and suppressing the "Align parts" check.
///
/// **Fix:** Replaced the per-node-type visitor overrides with a stack-based
/// approach using `visit_branch_node_enter`/`visit_branch_node_leave`. Every
/// branch node defaults to `ParentType::Other`; only `DefNode`, `BlockNode`,
/// `LambdaNode`, `BeginNode`, `IfNode`, and `UnlessNode` override to their
/// specific types. `StatementsNode` and `ElseNode` pass through the parent type.
/// In `check_dstr`, the parent type is read from the stack (the value saved
/// before entering the InterpolatedStringNode), ensuring it reflects the true
/// immediate parent.
///
/// **LambdaNode:** In RuboCop's Parser gem, lambdas produce `:block` nodes, so
/// `always_indented?` is true for lambda bodies. Added `LambdaNode` → `Block`.
pub struct LineEndStringConcatenationIndentation;

impl Cop for LineEndStringConcatenationIndentation {
    fn name(&self) -> &'static str {
        "Layout/LineEndStringConcatenationIndentation"
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        code_map: &CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let style = config.get_str("EnforcedStyle", "aligned");
        let indent_width = config.get_usize("IndentationWidth", 2);

        let mut visitor = ConcatVisitor {
            cop: self,
            source,
            code_map,
            diagnostics: Vec::new(),
            style,
            indent_width,
            direct_parent_type: ParentType::TopLevel,
            parent_type_stack: Vec::new(),
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

struct ConcatVisitor<'a> {
    cop: &'a LineEndStringConcatenationIndentation,
    source: &'a SourceFile,
    code_map: &'a CodeMap,
    diagnostics: Vec<Diagnostic>,
    style: &'a str,
    indent_width: usize,
    /// The type of the direct parent of the current node.
    /// RuboCop's `always_indented?` checks if the dstr's immediate parent is
    /// one of [nil, :block, :begin, :def, :defs, :if]. We track this via a
    /// stack: `visit_branch_node_enter` pushes the current type and defaults
    /// to Other; specific visitors override to the correct type.
    direct_parent_type: ParentType,
    /// Stack of saved parent types, pushed/popped by enter/leave hooks.
    parent_type_stack: Vec<ParentType>,
}

#[derive(Clone, Copy, PartialEq)]
enum ParentType {
    TopLevel,
    Block,
    Begin,
    Def,
    If,
    Other,
}

impl ConcatVisitor<'_> {
    fn check_dstr(&mut self, node: &ruby_prism::InterpolatedStringNode<'_>) {
        let parts: Vec<_> = node.parts().iter().collect();
        if parts.len() < 2 {
            return;
        }

        // Check that this is a backslash-concatenated string (multiline dstr
        // where each child is a single-line string/dstr part)
        let bytes = self.source.as_bytes();
        let (first_line, _) = self
            .source
            .offset_to_line_col(parts[0].location().start_offset());
        let (last_line, _) = self
            .source
            .offset_to_line_col(parts.last().unwrap().location().start_offset());
        if first_line == last_line {
            return; // Not multiline
        }

        // Check that each part is single-line and separated by backslash
        for part in &parts {
            let loc = part.location();
            let (sl, _) = self.source.offset_to_line_col(loc.start_offset());
            let (el, _) = self
                .source
                .offset_to_line_col(loc.end_offset().saturating_sub(1).max(loc.start_offset()));
            if sl != el {
                return; // Multi-line part
            }
        }

        // Check backslash between parts
        for pair in parts.windows(2) {
            let end_offset = pair[0].location().end_offset();
            let start_offset = pair[1].location().start_offset();
            let between = &bytes[end_offset..start_offset];
            if !between.contains(&b'\\') {
                return; // Not backslash continuation
            }
        }

        // Skip if inside a heredoc body
        if self.code_map.is_heredoc(parts[0].location().start_offset()) {
            return;
        }

        // RuboCop's `always_indented?` checks the DIRECT parent type.
        // Only these parent types force indented mode:
        //   nil (top-level), :block, :begin, :def, :defs, :if
        // Read the parent type from the stack — the value saved by
        // visit_branch_node_enter when entering this InterpolatedStringNode,
        // which is the type of this dstr's actual parent node.
        let parent_type = self
            .parent_type_stack
            .last()
            .copied()
            .unwrap_or(ParentType::TopLevel);
        let always_indented = matches!(
            parent_type,
            ParentType::TopLevel
                | ParentType::Block
                | ParentType::Begin
                | ParentType::Def
                | ParentType::If
        );
        let use_indented = self.style == "indented" || always_indented;

        // Get column positions of each part
        let columns: Vec<usize> = parts
            .iter()
            .map(|p| {
                let (_, col) = self.source.offset_to_line_col(p.location().start_offset());
                col
            })
            .collect();

        if use_indented && columns.len() >= 2 {
            // First, check indentation of the second part
            // base_column = indentation of the first part's source line
            let (first_part_line, _) = self
                .source
                .offset_to_line_col(parts[0].location().start_offset());
            let first_line_indent = if first_part_line > 0 {
                let lines: Vec<&[u8]> = self.source.lines().collect();
                lines[first_part_line - 1]
                    .iter()
                    .take_while(|&&b| b == b' ')
                    .count()
            } else {
                0
            };

            // Check if the first part's grandparent is a pair (hash key-value)
            // In that case, base_column is the pair's column
            // For simplicity, use the line indentation as base
            let expected_indent = first_line_indent + self.indent_width;

            if columns[1] != expected_indent {
                let (line_num, _) = self
                    .source
                    .offset_to_line_col(parts[1].location().start_offset());
                self.diagnostics.push(self.cop.diagnostic(
                    self.source,
                    line_num,
                    columns[1],
                    "Indent the first part of a string concatenated with backslash.".to_string(),
                ));
            }

            // Check alignment of third+ parts with the second part
            // RuboCop updates base_column after each check (rolling base)
            if columns.len() >= 3 {
                let mut base = columns[1];
                for (idx, &col) in columns[2..].iter().enumerate() {
                    if col != base {
                        let part_idx = idx + 2;
                        let (line_num, _) = self
                            .source
                            .offset_to_line_col(parts[part_idx].location().start_offset());
                        self.diagnostics.push(self.cop.diagnostic(
                            self.source,
                            line_num,
                            col,
                            "Align parts of a string concatenated with backslash.".to_string(),
                        ));
                    }
                    base = col; // Update rolling base like RuboCop
                }
            }
        } else if self.style == "aligned" {
            // check_aligned from index 1: parts should be aligned (rolling base)
            let mut base = columns[0];
            for (idx, &col) in columns[1..].iter().enumerate() {
                if col != base {
                    let part_idx = idx + 1;
                    let (line_num, _) = self
                        .source
                        .offset_to_line_col(parts[part_idx].location().start_offset());
                    self.diagnostics.push(self.cop.diagnostic(
                        self.source,
                        line_num,
                        col,
                        "Align parts of a string concatenated with backslash.".to_string(),
                    ));
                }
                base = col; // Update rolling base like RuboCop
            }
        }
    }
}

impl<'pr> Visit<'pr> for ConcatVisitor<'_> {
    // --- Stack-based parent type tracking ---
    // visit_branch_node_enter/leave are called by the Visit trait dispatch
    // for EVERY branch node. We use them to default all nodes to Other,
    // then specific visitor overrides set the correct type.

    fn visit_branch_node_enter(&mut self, _node: ruby_prism::Node<'pr>) {
        self.parent_type_stack.push(self.direct_parent_type);
        self.direct_parent_type = ParentType::Other;
    }

    fn visit_branch_node_leave(&mut self) {
        self.direct_parent_type = self.parent_type_stack.pop().unwrap_or(ParentType::TopLevel);
    }

    fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode<'pr>) {
        self.check_dstr(node);
        // Don't recurse into children — we handle the whole dstr at once
    }

    // --- "Always indented" parent types ---
    // These correspond to RuboCop's PARENT_TYPES_FOR_INDENTED:
    //   [nil, :block, :begin, :def, :defs, :if]
    // visit_branch_node_enter already pushed and set Other;
    // we override to the correct type before recursing.

    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        self.direct_parent_type = ParentType::Def;
        ruby_prism::visit_def_node(self, node);
    }

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        self.direct_parent_type = ParentType::Block;
        ruby_prism::visit_block_node(self, node);
    }

    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
        // In RuboCop's Parser gem, lambdas produce :block nodes.
        self.direct_parent_type = ParentType::Block;
        ruby_prism::visit_lambda_node(self, node);
    }

    fn visit_begin_node(&mut self, node: &ruby_prism::BeginNode<'pr>) {
        self.direct_parent_type = ParentType::Begin;
        ruby_prism::visit_begin_node(self, node);
    }

    fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
        self.direct_parent_type = ParentType::If;
        ruby_prism::visit_if_node(self, node);
    }

    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
        self.direct_parent_type = ParentType::If;
        ruby_prism::visit_unless_node(self, node);
    }

    // --- Pass-through nodes ---
    // These Prism wrappers are transparent in RuboCop's Parser gem.
    // Restore the parent type from before visit_branch_node_enter changed it.

    fn visit_else_node(&mut self, node: &ruby_prism::ElseNode<'pr>) {
        // In RuboCop's Parser gem, `else` is part of the `:if` node,
        // so `dstr.parent.type` inside an else branch is `:if`.
        self.direct_parent_type = self
            .parent_type_stack
            .last()
            .copied()
            .unwrap_or(ParentType::TopLevel);
        ruby_prism::visit_else_node(self, node);
    }

    fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode<'pr>) {
        // StatementsNode is Prism's wrapper; pass through the parent type
        // from the enclosing node.
        self.direct_parent_type = self
            .parent_type_stack
            .last()
            .copied()
            .unwrap_or(ParentType::TopLevel);
        ruby_prism::visit_statements_node(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(
        LineEndStringConcatenationIndentation,
        "cops/layout/line_end_string_concatenation_indentation"
    );
}
