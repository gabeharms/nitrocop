use crate::cop::node_type::{CASE_NODE, ELSE_NODE, IF_NODE, NIL_NODE, UNLESS_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct EmptyElse;

impl Cop for EmptyElse {
    fn name(&self) -> &'static str {
        "Style/EmptyElse"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CASE_NODE, ELSE_NODE, IF_NODE, NIL_NODE, UNLESS_NODE]
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
        let enforced_style = config.get_str("EnforcedStyle", "both");
        let allow_comments = config.get_bool("AllowComments", false);

        let check_empty = enforced_style == "empty" || enforced_style == "both";
        let check_nil = enforced_style == "nil" || enforced_style == "both";

        // Check if/unless nodes
        if let Some(if_node) = node.as_if_node() {
            // Only process if this is the top-level if (has `if` keyword, not elsif)
            let kw_loc = match if_node.if_keyword_loc() {
                Some(loc) => loc,
                None => return,
            };
            if kw_loc.as_slice() != b"if" {
                return;
            }

            // Walk the chain to find the else
            diagnostics.extend(self.check_if_chain_for_else(
                source,
                &if_node,
                check_empty,
                check_nil,
                allow_comments,
            ));
        }

        if let Some(unless_node) = node.as_unless_node() {
            if let Some(else_clause) = unless_node.else_clause() {
                diagnostics.extend(self.check_else_node(
                    source,
                    &else_clause,
                    check_empty,
                    check_nil,
                    allow_comments,
                ));
            }
            return;
        }

        if let Some(case_node) = node.as_case_node() {
            if let Some(else_clause) = case_node.else_clause() {
                diagnostics.extend(self.check_else_node(
                    source,
                    &else_clause,
                    check_empty,
                    check_nil,
                    allow_comments,
                ));
            }
        }
    }
}

impl EmptyElse {
    fn check_if_chain_for_else(
        &self,
        source: &SourceFile,
        if_node: &ruby_prism::IfNode<'_>,
        check_empty: bool,
        check_nil: bool,
        allow_comments: bool,
    ) -> Vec<Diagnostic> {
        // Walk subsequent chain
        let mut current_subsequent = if_node.subsequent();
        while let Some(sub) = current_subsequent {
            // If the subsequent is an ElseNode, check it
            if let Some(else_node) = sub.as_else_node() {
                return self.check_else_node(
                    source,
                    &else_node,
                    check_empty,
                    check_nil,
                    allow_comments,
                );
            }
            // If it's another IfNode (elsif), continue the chain
            if let Some(next_if) = sub.as_if_node() {
                current_subsequent = next_if.subsequent();
                continue;
            }
            break;
        }
        Vec::new()
    }

    fn check_else_node(
        &self,
        source: &SourceFile,
        else_node: &ruby_prism::ElseNode<'_>,
        check_empty: bool,
        check_nil: bool,
        allow_comments: bool,
    ) -> Vec<Diagnostic> {
        let kw_loc = else_node.else_keyword_loc();

        match else_node.statements() {
            None => {
                // Empty else clause
                if check_empty {
                    // When AllowComments is true, skip if there are comments in the else body
                    if allow_comments {
                        if let Some(end_kw) = else_node.end_keyword_loc() {
                            let else_end = kw_loc.end_offset();
                            let body_end = end_kw.start_offset();
                            if else_end < body_end {
                                let body_bytes = &source.as_bytes()[else_end..body_end];
                                if body_bytes.contains(&b'#') {
                                    return Vec::new();
                                }
                            }
                        }
                    }
                    let (line, column) = source.offset_to_line_col(kw_loc.start_offset());
                    return vec![self.diagnostic(
                        source,
                        line,
                        column,
                        "Redundant `else`-clause.".to_string(),
                    )];
                }
            }
            Some(stmts) => {
                // Check if the only statement is `nil`
                let body: Vec<_> = stmts.body().iter().collect();
                if body.len() == 1 && body[0].as_nil_node().is_some() && check_nil {
                    let (line, column) = source.offset_to_line_col(kw_loc.start_offset());
                    return vec![self.diagnostic(
                        source,
                        line,
                        column,
                        "Redundant `else`-clause.".to_string(),
                    )];
                }
            }
        }

        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;
    use crate::testutil::run_cop_full_with_config;

    crate::cop_fixture_tests!(EmptyElse, "cops/style/empty_else");

    fn allow_comments_config() -> CopConfig {
        use std::collections::HashMap;
        CopConfig {
            options: HashMap::from([("AllowComments".into(), serde_yml::Value::Bool(true))]),
            ..CopConfig::default()
        }
    }

    #[test]
    fn allow_comments_empty_else_with_comment_no_offense() {
        let source = b"if condition\n  statement\nelse\n  # valid reason\nend\n";
        let diags = run_cop_full_with_config(&EmptyElse, source, allow_comments_config());
        assert!(
            diags.is_empty(),
            "AllowComments: true should skip empty else with comment"
        );
    }

    #[test]
    fn allow_comments_case_else_with_comment_no_offense() {
        let source = b"case x\nwhen 1\n  'one'\nelse\n  # intentionally empty\nend\n";
        let diags = run_cop_full_with_config(&EmptyElse, source, allow_comments_config());
        assert!(
            diags.is_empty(),
            "AllowComments: true should skip empty case/else with comment"
        );
    }

    #[test]
    fn allow_comments_empty_else_without_comment_still_offense() {
        let source = b"if condition\n  statement\nelse\nend\n";
        let diags = run_cop_full_with_config(&EmptyElse, source, allow_comments_config());
        assert_eq!(
            diags.len(),
            1,
            "AllowComments: true should still flag empty else without comment"
        );
    }
}
