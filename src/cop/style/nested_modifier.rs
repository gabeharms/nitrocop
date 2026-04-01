use crate::cop::node_type::{IF_NODE, UNLESS_NODE, UNTIL_NODE, WHILE_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Checks for nested modifier conditionals/loops.
///
/// ## Investigation findings
/// FN root cause: only handled `IfNode` and `UnlessNode` as outer/inner modifiers.
/// `WhileNode` and `UntilNode` can also be modifier forms (no `end` keyword) and
/// participate in nested modifier combinations like `something if a while b`.
/// Fix: added WHILE_NODE/UNTIL_NODE to interested_node_types and inner body checks.
pub struct NestedModifier;

impl Cop for NestedModifier {
    fn name(&self) -> &'static str {
        "Style/NestedModifier"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[IF_NODE, UNLESS_NODE, WHILE_NODE, UNTIL_NODE]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Get the body of a modifier conditional/loop (if/unless/while/until)
        let body_node = if let Some(if_node) = node.as_if_node() {
            // Must be modifier form (no end keyword, has if keyword, not ternary)
            if if_node.end_keyword_loc().is_some() {
                return;
            }
            let kw_loc = match if_node.if_keyword_loc() {
                Some(loc) => loc,
                None => return, // ternary
            };
            let kw_bytes = kw_loc.as_slice();
            if kw_bytes != b"if" && kw_bytes != b"unless" {
                return;
            }
            if_node.statements()
        } else if let Some(unless_node) = node.as_unless_node() {
            if unless_node.end_keyword_loc().is_some() {
                return;
            }
            unless_node.statements()
        } else if let Some(while_node) = node.as_while_node() {
            // Must be modifier form (no closing/end keyword)
            if while_node.closing_loc().is_some() {
                return;
            }
            while_node.statements()
        } else if let Some(until_node) = node.as_until_node() {
            if until_node.closing_loc().is_some() {
                return;
            }
            until_node.statements()
        } else {
            return;
        };

        let stmts = match body_node {
            Some(s) => s,
            None => return,
        };

        let body: Vec<_> = stmts.body().iter().collect();
        if body.len() != 1 {
            return;
        }

        let inner_keyword_offset = if let Some(inner_if) = body[0].as_if_node() {
            if inner_if.end_keyword_loc().is_some() {
                return;
            }
            inner_if.if_keyword_loc().and_then(|inner_kw| {
                let inner_bytes = inner_kw.as_slice();
                if inner_bytes == b"if" || inner_bytes == b"unless" {
                    Some(inner_kw.start_offset())
                } else {
                    None
                }
            })
        } else if let Some(inner_unless) = body[0].as_unless_node() {
            if inner_unless.end_keyword_loc().is_some() {
                return;
            }
            let inner_kw = inner_unless.keyword_loc();
            if inner_kw.as_slice() == b"unless" {
                Some(inner_kw.start_offset())
            } else {
                None
            }
        } else if let Some(inner_while) = body[0].as_while_node() {
            if inner_while.closing_loc().is_some() {
                return;
            }
            Some(inner_while.keyword_loc().start_offset())
        } else if let Some(inner_until) = body[0].as_until_node() {
            if inner_until.closing_loc().is_some() {
                return;
            }
            Some(inner_until.keyword_loc().start_offset())
        } else {
            None
        };

        let Some(inner_keyword_offset) = inner_keyword_offset else {
            return;
        };

        let (line, column) = source.offset_to_line_col(inner_keyword_offset);
        let mut diagnostic = self.diagnostic(
            source,
            line,
            column,
            "Avoid using nested modifiers.".to_string(),
        );

        if let Some(corrections) = corrections {
            if let Some(replacement) = nested_modifier_replacement(source, node) {
                corrections.push(crate::correction::Correction {
                    start: node.location().start_offset(),
                    end: node.location().end_offset(),
                    replacement,
                    cop_name: self.name(),
                    cop_index: 0,
                });
                diagnostic.corrected = true;
            }
        }

        diagnostics.push(diagnostic);
    }
}

fn nested_modifier_replacement(source: &SourceFile, node: &ruby_prism::Node<'_>) -> Option<String> {
    let (keyword, predicate, statements) = if let Some(if_node) = node.as_if_node() {
        let kw_loc = if_node.if_keyword_loc()?;
        let kw = match kw_loc.as_slice() {
            b"if" => "if",
            b"unless" => "unless",
            _ => return None,
        };
        (kw, if_node.predicate(), if_node.statements()?)
    } else if let Some(unless_node) = node.as_unless_node() {
        ("unless", unless_node.predicate(), unless_node.statements()?)
    } else if let Some(while_node) = node.as_while_node() {
        ("while", while_node.predicate(), while_node.statements()?)
    } else if let Some(until_node) = node.as_until_node() {
        ("until", until_node.predicate(), until_node.statements()?)
    } else {
        return None;
    };

    let body_node = statements.body().iter().next()?;
    let body_source = String::from_utf8_lossy(
        &source.as_bytes()[body_node.location().start_offset()..body_node.location().end_offset()],
    )
    .into_owned();
    let predicate_source = String::from_utf8_lossy(
        &source.as_bytes()[predicate.location().start_offset()..predicate.location().end_offset()],
    )
    .into_owned();

    Some(format!(
        "{} {}\n  {}\nend",
        keyword,
        predicate_source.trim(),
        body_source.trim()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(NestedModifier, "cops/style/nested_modifier");
    crate::cop_autocorrect_fixture_tests!(NestedModifier, "cops/style/nested_modifier");
}
