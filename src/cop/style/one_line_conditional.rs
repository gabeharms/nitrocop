use crate::cop::node_type::{IF_NODE, UNLESS_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Flags single-line `if/then/else/end` and `unless/then/else/end` constructs.
///
/// Root cause of 12 FNs: the cop previously required `then_keyword_loc` to be present,
/// but Ruby also allows semicolons instead of `then` (e.g., `if cond; run else dont end`).
/// Prism sets `then_keyword_loc` to None for the semicolon form. Fix: removed the
/// `then_keyword_loc` requirement, matching RuboCop's behavior which triggers on any
/// single-line if/unless with an else branch.
///
/// Also added: skip when then-body has multiple statements (matches RuboCop's
/// `return if node.if_branch&.begin_type?`), and skip when else body is empty.
pub struct OneLineConditional;

impl Cop for OneLineConditional {
    fn name(&self) -> &'static str {
        "Style/OneLineConditional"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[IF_NODE, UNLESS_NODE]
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
        // AlwaysCorrectToMultiline only affects auto-correction (ternary vs multiline),
        // not detection. Read it to satisfy config completeness.
        let _always_multiline = config.get_bool("AlwaysCorrectToMultiline", false);
        // Check `if ... then ... else ... end` on one line
        if let Some(if_node) = node.as_if_node() {
            let kw_loc = match if_node.if_keyword_loc() {
                Some(loc) => loc,
                None => return, // ternary
            };

            let kw_bytes = kw_loc.as_slice();
            if kw_bytes != b"if" {
                return;
            }

            // Must not be modifier form
            if if_node.end_keyword_loc().is_none() {
                return;
            }

            // Must have an else branch
            if if_node.subsequent().is_none() {
                return;
            }

            // Skip if then-body has multiple statements (begin_type equivalent)
            if let Some(stmts) = if_node.statements() {
                if stmts.body().len() > 1 {
                    return;
                }
            }

            // Skip if else branch body is empty
            if let Some(else_node) = if_node.subsequent().and_then(|s| s.as_else_node()) {
                if else_node.statements().is_none()
                    || else_node.statements().is_some_and(|s| s.body().is_empty())
                {
                    return;
                }
            }

            // Must be single-line
            let loc = if_node.location();
            let (start_line, _) = source.offset_to_line_col(loc.start_offset());
            let (end_line, _) = source.offset_to_line_col(loc.end_offset().saturating_sub(1));
            if start_line != end_line {
                return;
            }

            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                "Favor the ternary operator (`?:`) over single-line `if/then/else/end` constructs.".to_string(),
            ));
        }

        // Check `unless ... then ... else ... end` on one line
        if let Some(unless_node) = node.as_unless_node() {
            let kw_loc = unless_node.keyword_loc();
            if kw_loc.as_slice() != b"unless" {
                return;
            }

            // Must not be modifier form
            if unless_node.end_keyword_loc().is_none() {
                return;
            }

            // Must have an else branch
            if unless_node.else_clause().is_none() {
                return;
            }

            // Skip if then-body has multiple statements
            if let Some(stmts) = unless_node.statements() {
                if stmts.body().len() > 1 {
                    return;
                }
            }

            // Skip if else branch body is empty
            if let Some(else_node) = unless_node.else_clause() {
                if else_node.statements().is_none()
                    || else_node.statements().is_some_and(|s| s.body().is_empty())
                {
                    return;
                }
            }

            // Must be single-line
            let loc = unless_node.location();
            let (start_line, _) = source.offset_to_line_col(loc.start_offset());
            let (end_line, _) = source.offset_to_line_col(loc.end_offset().saturating_sub(1));
            if start_line != end_line {
                return;
            }

            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                "Favor the ternary operator (`?:`) over single-line `unless/then/else/end` constructs.".to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(OneLineConditional, "cops/style/one_line_conditional");
}
