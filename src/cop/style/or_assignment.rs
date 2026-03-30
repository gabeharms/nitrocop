use crate::cop::node_type::{
    CLASS_VARIABLE_READ_NODE, CLASS_VARIABLE_WRITE_NODE, GLOBAL_VARIABLE_READ_NODE,
    GLOBAL_VARIABLE_WRITE_NODE, IF_NODE, INSTANCE_VARIABLE_READ_NODE, INSTANCE_VARIABLE_WRITE_NODE,
    LOCAL_VARIABLE_READ_NODE, LOCAL_VARIABLE_WRITE_NODE, OR_NODE,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct OrAssignment;

impl OrAssignment {
    fn node_source(source: &SourceFile, node: &ruby_prism::Node<'_>) -> String {
        String::from_utf8_lossy(
            &source.as_bytes()[node.location().start_offset()..node.location().end_offset()],
        )
        .to_string()
    }

    /// Get variable name from a local/instance/class/global variable write node
    fn get_write_name(node: &ruby_prism::Node<'_>) -> Option<Vec<u8>> {
        if let Some(lv) = node.as_local_variable_write_node() {
            return Some(lv.name().as_slice().to_vec());
        }
        if let Some(iv) = node.as_instance_variable_write_node() {
            return Some(iv.name().as_slice().to_vec());
        }
        if let Some(cv) = node.as_class_variable_write_node() {
            return Some(cv.name().as_slice().to_vec());
        }
        if let Some(gv) = node.as_global_variable_write_node() {
            return Some(gv.name().as_slice().to_vec());
        }
        None
    }

    /// Get variable name from a local/instance/class/global variable read node
    fn get_read_name(node: &ruby_prism::Node<'_>) -> Option<Vec<u8>> {
        if let Some(lv) = node.as_local_variable_read_node() {
            return Some(lv.name().as_slice().to_vec());
        }
        if let Some(iv) = node.as_instance_variable_read_node() {
            return Some(iv.name().as_slice().to_vec());
        }
        if let Some(cv) = node.as_class_variable_read_node() {
            return Some(cv.name().as_slice().to_vec());
        }
        if let Some(gv) = node.as_global_variable_read_node() {
            return Some(gv.name().as_slice().to_vec());
        }
        None
    }

    /// Check for `x = x || y` or `x = x ? x : y` patterns.
    fn check_or_assign(
        cop: &OrAssignment,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        corrections: &mut Option<&mut Vec<crate::correction::Correction>>,
    ) -> Vec<Diagnostic> {
        let write_name = match Self::get_write_name(node) {
            Some(n) => n,
            None => return Vec::new(),
        };
        let write_name_str = String::from_utf8_lossy(&write_name).to_string();

        // Get the value being assigned
        let value = if let Some(lv) = node.as_local_variable_write_node() {
            lv.value()
        } else if let Some(iv) = node.as_instance_variable_write_node() {
            iv.value()
        } else if let Some(cv) = node.as_class_variable_write_node() {
            cv.value()
        } else if let Some(gv) = node.as_global_variable_write_node() {
            gv.value()
        } else {
            return Vec::new();
        };

        let loc = node.location();
        let (line, column) = source.offset_to_line_col(loc.start_offset());

        // Check if the value is `x || y` where x is the same variable
        if let Some(or_node) = value.as_or_node() {
            let left = or_node.left();
            if let Some(read_name) = Self::get_read_name(&left) {
                if read_name == write_name {
                    let mut diag = cop.diagnostic(
                        source,
                        line,
                        column,
                        "Use the double pipe equals operator `||=` instead.".to_string(),
                    );
                    if let Some(corr) = corrections.as_mut() {
                        corr.push(crate::correction::Correction {
                            start: loc.start_offset(),
                            end: loc.end_offset(),
                            replacement: format!(
                                "{} ||= {}",
                                write_name_str,
                                Self::node_source(source, &or_node.right())
                            ),
                            cop_name: cop.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    return vec![diag];
                }
            }
        }

        // Check for ternary: `x = x ? x : y`
        if let Some(if_node) = value.as_if_node() {
            let predicate = if_node.predicate();
            if let Some(pred_name) = Self::get_read_name(&predicate) {
                if pred_name == write_name {
                    // Check if true branch is the same variable
                    if let Some(true_branch) = if_node.statements() {
                        let true_nodes: Vec<_> = true_branch.body().into_iter().collect();
                        if true_nodes.len() == 1 {
                            if let Some(true_name) = Self::get_read_name(&true_nodes[0]) {
                                if true_name == write_name {
                                    let else_value_src = if_node.subsequent().and_then(|sub| {
                                        sub.as_else_node().and_then(|else_node| {
                                            else_node.statements().and_then(|stmts| {
                                                let values: Vec<_> = stmts.body().iter().collect();
                                                if values.len() == 1 {
                                                    Some(Self::node_source(source, &values[0]))
                                                } else {
                                                    None
                                                }
                                            })
                                        })
                                    });

                                    let mut diag = cop.diagnostic(
                                        source,
                                        line,
                                        column,
                                        "Use the double pipe equals operator `||=` instead."
                                            .to_string(),
                                    );

                                    if let Some(default_value_src) = else_value_src {
                                        if let Some(corr) = corrections.as_mut() {
                                            corr.push(crate::correction::Correction {
                                                start: loc.start_offset(),
                                                end: loc.end_offset(),
                                                replacement: format!(
                                                    "{} ||= {}",
                                                    write_name_str, default_value_src
                                                ),
                                                cop_name: cop.name(),
                                                cop_index: 0,
                                            });
                                            diag.corrected = true;
                                        }
                                    }

                                    return vec![diag];
                                }
                            }
                        }
                    }
                }
            }
        }

        Vec::new()
    }
}

impl Cop for OrAssignment {
    fn name(&self) -> &'static str {
        "Style/OrAssignment"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            CLASS_VARIABLE_READ_NODE,
            CLASS_VARIABLE_WRITE_NODE,
            GLOBAL_VARIABLE_READ_NODE,
            GLOBAL_VARIABLE_WRITE_NODE,
            IF_NODE,
            INSTANCE_VARIABLE_READ_NODE,
            INSTANCE_VARIABLE_WRITE_NODE,
            LOCAL_VARIABLE_READ_NODE,
            LOCAL_VARIABLE_WRITE_NODE,
            OR_NODE,
        ]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        diagnostics.extend(Self::check_or_assign(self, source, node, &mut corrections));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(OrAssignment, "cops/style/or_assignment");
    crate::cop_autocorrect_fixture_tests!(OrAssignment, "cops/style/or_assignment");
}
