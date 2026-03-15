use crate::cop::node_type::{IF_NODE, UNLESS_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Checks for uses of if/unless modifiers with multiple-line bodies.
///
/// ## Investigation findings (2026-03-15)
///
/// **Root cause of FNs (12):** The previous implementation used a
/// `lines_joined_by_backslash` function to exempt backslash-continued lines.
/// This was too broad — it exempted cases where the body itself spans multiple
/// physical lines joined by `\` (e.g., `raise "msg" \ "more" if cond`).
/// RuboCop flags these because `node.body.multiline?` checks if the body AST
/// node's first_line != last_line, regardless of `\` continuation.
///
/// **Root cause of FPs (44):** Primarily config-related (project-level
/// `.rubocop_todo.yml` excludes or file-level disables), not cop logic bugs.
/// RuboCop does flag patterns like `begin...end if cond`, `def...end if cond`,
/// and `block do...end if cond`.
///
/// **Fix:** Replaced the `body_start_line < if_kw_line` + backslash exemption
/// approach with a direct check: `body_start_line != body_end_line` (whether
/// the body AST node itself spans multiple lines). This matches RuboCop's
/// `node.body.multiline?` semantics and eliminates the need for backslash
/// continuation handling entirely.
pub struct MultilineIfModifier;

impl Cop for MultilineIfModifier {
    fn name(&self) -> &'static str {
        "Style/MultilineIfModifier"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[IF_NODE, UNLESS_NODE]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Check `if` modifier form
        if let Some(if_node) = node.as_if_node() {
            let if_kw_loc = match if_node.if_keyword_loc() {
                Some(loc) => loc,
                None => return,
            };

            if if_kw_loc.as_slice() != b"if" {
                return;
            }

            // Must be modifier form (no end keyword)
            if if_node.end_keyword_loc().is_some() {
                return;
            }

            // Check if the body spans multiple lines
            if let Some(stmts) = if_node.statements() {
                let body_nodes: Vec<_> = stmts.body().into_iter().collect();
                if body_nodes.is_empty() {
                    return;
                }

                let first = &body_nodes[0];
                let last = &body_nodes[body_nodes.len() - 1];
                let body_start_line = source.offset_to_line_col(first.location().start_offset()).0;
                let body_end_line = source
                    .offset_to_line_col(last.location().end_offset().saturating_sub(1))
                    .0;

                // Body is multiline if it spans more than one line
                if body_start_line < body_end_line {
                    let body_start = first.location().start_offset();
                    let (line, column) = source.offset_to_line_col(body_start);
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Favor a normal if-statement over a modifier clause in a multiline statement.".to_string(),
                    ));
                }
            }

            return;
        }

        // Check `unless` modifier form
        if let Some(unless_node) = node.as_unless_node() {
            let kw_loc = unless_node.keyword_loc();

            if kw_loc.as_slice() != b"unless" {
                return;
            }

            // Must be modifier form (no end keyword)
            if unless_node.end_keyword_loc().is_some() {
                return;
            }

            // Check if the body spans multiple lines
            if let Some(stmts) = unless_node.statements() {
                let body_nodes: Vec<_> = stmts.body().into_iter().collect();
                if body_nodes.is_empty() {
                    return;
                }

                let first = &body_nodes[0];
                let last = &body_nodes[body_nodes.len() - 1];
                let body_start_line = source.offset_to_line_col(first.location().start_offset()).0;
                let body_end_line = source
                    .offset_to_line_col(last.location().end_offset().saturating_sub(1))
                    .0;

                // Body is multiline if it spans more than one line
                if body_start_line < body_end_line {
                    let body_start = first.location().start_offset();
                    let (line, column) = source.offset_to_line_col(body_start);
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Favor a normal unless-statement over a modifier clause in a multiline statement.".to_string(),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(MultilineIfModifier, "cops/style/multiline_if_modifier");
}
