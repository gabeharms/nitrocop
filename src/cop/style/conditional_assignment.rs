use crate::cop::node_type::{
    ELSE_NODE, IF_NODE, INSTANCE_VARIABLE_WRITE_NODE, LOCAL_VARIABLE_WRITE_NODE,
};
use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ConditionalAssignment;

impl Cop for ConditionalAssignment {
    fn name(&self) -> &'static str {
        "Style/ConditionalAssignment"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            ELSE_NODE,
            IF_NODE,
            INSTANCE_VARIABLE_WRITE_NODE,
            LOCAL_VARIABLE_WRITE_NODE,
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<Correction>>,
    ) {
        let enforced_style = config.get_str("EnforcedStyle", "assign_to_condition");
        let _single_line_only = config.get_bool("SingleLineConditionsOnly", true);
        let _include_ternary = config.get_bool("IncludeTernaryExpressions", true);

        if enforced_style != "assign_to_condition" {
            return;
        }

        // Check for if/else where each branch assigns to the same variable
        let if_node = match node.as_if_node() {
            Some(n) => n,
            None => return,
        };

        // Must be a top-level `if`, not an `elsif` branch
        if let Some(kw_loc) = if_node.if_keyword_loc() {
            if kw_loc.as_slice() == b"elsif" {
                return;
            }
        }

        // Must have an else clause
        let else_clause = match if_node.subsequent() {
            Some(s) => s,
            None => return,
        };

        // Must be a simple if/else (not if/elsif/else)
        if else_clause.as_if_node().is_some() {
            return;
        }

        // Check if both branches assign to the same variable
        let if_body = match if_node.statements() {
            Some(s) => s,
            None => return,
        };

        let if_stmts: Vec<_> = if_body.body().iter().collect();
        if if_stmts.len() != 1 {
            return;
        }

        let if_assign_name = get_assignment_target(&if_stmts[0]);

        if let Some(else_node) = else_clause.as_else_node() {
            if let Some(else_stmts) = else_node.statements() {
                let else_list: Vec<_> = else_stmts.body().iter().collect();
                if else_list.len() != 1 {
                    return;
                }

                let else_assign_name = get_assignment_target(&else_list[0]);

                if let (Some(if_name), Some(else_name)) = (if_assign_name, else_assign_name) {
                    if if_name == else_name {
                        let loc = if_node.location();
                        let (line, column) = source.offset_to_line_col(loc.start_offset());
                        let mut diagnostic = self.diagnostic(
                            source,
                            line,
                            column,
                            "Use the return value of `if` expression for variable assignment and comparison.".to_string(),
                        );

                        if let Some(corrs) = corrections.as_deref_mut() {
                            if let Some(correction) = autocorrect_assignment_if(source, &if_node, &if_stmts[0], &else_list[0]) {
                                corrs.push(correction);
                                diagnostic.corrected = true;
                            }
                        }

                        diagnostics.push(diagnostic);
                    }
                }
            }
        }
    }
}

fn assignment_parts(source: &SourceFile, node: &ruby_prism::Node<'_>) -> Option<(String, String)> {
    if let Some(write) = node.as_local_variable_write_node() {
        let name = std::str::from_utf8(write.name().as_slice())
            .unwrap_or("")
            .to_string();
        let value = write.value();
        let value_loc = value.location();
        let value_src = source
            .byte_slice(value_loc.start_offset(), value_loc.end_offset(), "")
            .trim()
            .to_string();
        if name.is_empty() || value_src.is_empty() {
            return None;
        }
        return Some((name, value_src));
    }

    if let Some(write) = node.as_instance_variable_write_node() {
        let name = std::str::from_utf8(write.name().as_slice())
            .unwrap_or("")
            .to_string();
        let value = write.value();
        let value_loc = value.location();
        let value_src = source
            .byte_slice(value_loc.start_offset(), value_loc.end_offset(), "")
            .trim()
            .to_string();
        if name.is_empty() || value_src.is_empty() {
            return None;
        }
        return Some((name, value_src));
    }

    None
}

fn get_assignment_target(node: &ruby_prism::Node<'_>) -> Option<String> {
    if let Some(write) = node.as_local_variable_write_node() {
        return Some(
            std::str::from_utf8(write.name().as_slice())
                .unwrap_or("")
                .to_string(),
        );
    }
    if let Some(write) = node.as_instance_variable_write_node() {
        return Some(
            std::str::from_utf8(write.name().as_slice())
                .unwrap_or("")
                .to_string(),
        );
    }
    None
}

fn autocorrect_assignment_if(
    source: &SourceFile,
    if_node: &ruby_prism::IfNode<'_>,
    if_assign: &ruby_prism::Node<'_>,
    else_assign: &ruby_prism::Node<'_>,
) -> Option<Correction> {
    let if_kw = if_node.if_keyword_loc()?;
    if if_kw.as_slice() != b"if" {
        return None;
    }

    let predicate = if_node.predicate();
    let pred_loc = predicate.location();
    let pred_src = source
        .byte_slice(pred_loc.start_offset(), pred_loc.end_offset(), "")
        .trim()
        .to_string();
    if pred_src.is_empty() {
        return None;
    }

    let (target, if_value) = assignment_parts(source, if_assign)?;
    let (else_target, else_value) = assignment_parts(source, else_assign)?;
    if target != else_target {
        return None;
    }

    let if_assign_loc = if_assign.location();
    let else_assign_loc = else_assign.location();

    let (if_line, _) = source.offset_to_line_col(if_kw.start_offset());
    let if_line_start = source.line_start_offset(if_line);
    let if_indent = source.byte_slice(if_line_start, if_kw.start_offset(), "");

    let (if_assign_line, _) = source.offset_to_line_col(if_assign_loc.start_offset());
    let if_assign_line_start = source.line_start_offset(if_assign_line);
    let if_branch_indent = source.byte_slice(if_assign_line_start, if_assign_loc.start_offset(), "");

    let (else_assign_line, _) = source.offset_to_line_col(else_assign_loc.start_offset());
    let else_assign_line_start = source.line_start_offset(else_assign_line);
    let else_branch_indent = source.byte_slice(
        else_assign_line_start,
        else_assign_loc.start_offset(),
        "",
    );

    let replacement = format!(
        "{if_indent}{target} = if {pred_src}\n{if_branch_indent}{if_value}\n{if_indent}else\n{else_branch_indent}{else_value}\n{if_indent}end"
    );

    Some(Correction {
        start: if_node.location().start_offset(),
        end: if_node.location().end_offset(),
        replacement,
        cop_name: "Style/ConditionalAssignment",
        cop_index: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ConditionalAssignment, "cops/style/conditional_assignment");
    crate::cop_autocorrect_fixture_tests!(ConditionalAssignment, "cops/style/conditional_assignment");
}
