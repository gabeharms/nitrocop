use crate::cop::node_type::CALL_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FloatDivision;

impl FloatDivision {
    fn is_to_f_call(node: &ruby_prism::Node<'_>) -> bool {
        if let Some(call) = node.as_call_node() {
            if call.name().as_slice() == b"to_f" && call.receiver().is_some() {
                // Make sure it has no arguments (not an implicit receiver call)
                if call.arguments().is_none() {
                    return true;
                }
            }
        }
        false
    }
}

impl Cop for FloatDivision {
    fn name(&self) -> &'static str {
        "Style/FloatDivision"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        if call.name().as_slice() != b"/" {
            return;
        }

        let receiver = match call.receiver() {
            Some(r) => r,
            None => return,
        };

        let args = match call.arguments() {
            Some(a) => a,
            None => return,
        };

        let arg_list: Vec<_> = args.arguments().iter().collect();
        if arg_list.len() != 1 {
            return;
        }

        let left_is_to_f = Self::is_to_f_call(&receiver);
        let right_is_to_f = Self::is_to_f_call(&arg_list[0]);

        if !left_is_to_f && !right_is_to_f {
            return;
        }

        let style = config.get_str("EnforcedStyle", "single_coerce");

        let loc = call.location();
        let (line, column) = source.offset_to_line_col(loc.start_offset());

        match style {
            "single_coerce" => {
                if left_is_to_f && right_is_to_f {
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        column,
                        "Prefer using `.to_f` on one side only.".to_string(),
                    );
                    if let Some(corr) = corrections.as_mut() {
                        if let Some(right_call) = arg_list[0].as_call_node() {
                            remove_to_f_method(self.name(), &right_call, corr);
                            diag.corrected = true;
                        }
                    }
                    diagnostics.push(diag);
                }
            }
            "left_coerce" => {
                if right_is_to_f {
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        column,
                        "Prefer using `.to_f` on the left side.".to_string(),
                    );
                    if let Some(corr) = corrections.as_mut() {
                        if let Some(right_call) = arg_list[0].as_call_node() {
                            remove_to_f_method(self.name(), &right_call, corr);
                        }
                        if !left_is_to_f {
                            add_to_f_method(self.name(), &receiver, corr);
                        }
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }
            }
            "right_coerce" => {
                if left_is_to_f {
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        column,
                        "Prefer using `.to_f` on the right side.".to_string(),
                    );
                    if let Some(corr) = corrections.as_mut() {
                        if let Some(left_call) = receiver.as_call_node() {
                            remove_to_f_method(self.name(), &left_call, corr);
                        }
                        if !right_is_to_f {
                            add_to_f_method(self.name(), &arg_list[0], corr);
                        }
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }
            }
            "fdiv" => {
                if left_is_to_f || right_is_to_f {
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        column,
                        "Prefer using `fdiv` for float divisions.".to_string(),
                    );
                    if let Some(corr) = corrections.as_mut() {
                        let lhs = operand_without_to_f(source, &receiver);
                        let rhs = operand_without_to_f(source, &arg_list[0]);
                        corr.push(crate::correction::Correction {
                            start: loc.start_offset(),
                            end: loc.end_offset(),
                            replacement: format!("{lhs}.fdiv({rhs})"),
                            cop_name: self.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }
                    diagnostics.push(diag);
                }
            }
            _ => {}
        }
    }
}

fn remove_to_f_method(
    cop_name: &'static str,
    call: &ruby_prism::CallNode<'_>,
    corrections: &mut Vec<crate::correction::Correction>,
) {
    if let (Some(dot), Some(message)) = (call.call_operator_loc(), call.message_loc()) {
        corrections.push(crate::correction::Correction {
            start: dot.start_offset(),
            end: message.end_offset(),
            replacement: String::new(),
            cop_name,
            cop_index: 0,
        });
    }
}

fn add_to_f_method(
    cop_name: &'static str,
    node: &ruby_prism::Node<'_>,
    corrections: &mut Vec<crate::correction::Correction>,
) {
    let end = node.location().end_offset();
    corrections.push(crate::correction::Correction {
        start: end,
        end,
        replacement: ".to_f".to_string(),
        cop_name,
        cop_index: 0,
    });
}

fn operand_without_to_f(source: &SourceFile, node: &ruby_prism::Node<'_>) -> String {
    if let Some(call) = node.as_call_node() {
        if call.name().as_slice() == b"to_f" {
            if let Some(receiver) = call.receiver() {
                let loc = receiver.location();
                return source
                    .byte_slice(loc.start_offset(), loc.end_offset(), "")
                    .to_string();
            }
        }
    }

    let loc = node.location();
    source
        .byte_slice(loc.start_offset(), loc.end_offset(), "")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(FloatDivision, "cops/style/float_division");
    crate::cop_autocorrect_fixture_tests!(FloatDivision, "cops/style/float_division");
}
