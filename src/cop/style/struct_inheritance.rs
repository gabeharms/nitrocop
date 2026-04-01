use crate::cop::node_type::{CALL_NODE, CLASS_NODE, CONSTANT_PATH_NODE, CONSTANT_READ_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct StructInheritance;

impl Cop for StructInheritance {
    fn name(&self) -> &'static str {
        "Style/StructInheritance"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            CALL_NODE,
            CLASS_NODE,
            CONSTANT_PATH_NODE,
            CONSTANT_READ_NODE,
        ]
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
        let class_node = match node.as_class_node() {
            Some(c) => c,
            None => return,
        };

        // Must have a superclass
        let superclass = match class_node.superclass() {
            Some(s) => s,
            None => return,
        };

        // Check if superclass is Struct.new(...) or ::Struct.new(...)
        if is_struct_new(&superclass) || is_struct_new_block(&superclass) {
            let loc = superclass.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            let mut diag = self.diagnostic(
                source,
                line,
                column,
                "Don't extend an instance initialized by `Struct.new`. Use a block to customize the struct.".to_string(),
            );

            if let Some(corrections) = corrections.as_mut() {
                let class_name =
                    std::str::from_utf8(class_node.name().as_slice()).unwrap_or("ClassName");
                let super_src = source.byte_slice(loc.start_offset(), loc.end_offset(), "");
                let replacement = if let Some(body) = class_node.body() {
                    let body_src = source.byte_slice(
                        body.location().start_offset(),
                        body.location().end_offset(),
                        "",
                    );
                    if body_src.trim().is_empty() {
                        format!("{class_name} = {super_src}")
                    } else {
                        let mut lines = body_src.lines();
                        let indented_body = if let Some(first_line) = lines.next() {
                            let mut out = String::new();
                            if first_line.is_empty() {
                                out.push_str("  ");
                            } else {
                                out.push_str("  ");
                                out.push_str(first_line);
                            }
                            for line in lines {
                                out.push('\n');
                                out.push_str(line);
                            }
                            out
                        } else {
                            String::new()
                        };
                        format!("{class_name} = {super_src} do\n{indented_body}\nend")
                    }
                } else {
                    format!("{class_name} = {super_src}")
                };

                corrections.push(crate::correction::Correction {
                    start: class_node.location().start_offset(),
                    end: class_node.location().end_offset(),
                    replacement,
                    cop_name: self.name(),
                    cop_index: 0,
                });
                diag.corrected = true;
            }

            diagnostics.push(diag);
        }
    }
}

fn is_struct_new(node: &ruby_prism::Node<'_>) -> bool {
    let call = match node.as_call_node() {
        Some(c) => c,
        None => return false,
    };

    if call.name().as_slice() != b"new" {
        return false;
    }

    match call.receiver() {
        Some(recv) => is_struct_const(&recv),
        None => false,
    }
}

fn is_struct_new_block(node: &ruby_prism::Node<'_>) -> bool {
    // block { Struct.new(...) do ... end }
    // Prism represents this as a CallNode with a block
    // Actually let's check if this is a block whose call is Struct.new
    if let Some(call) = node.as_call_node() {
        if let Some(block) = call.block() {
            // The call itself is Struct.new, and there's a block
            if call.name().as_slice() == b"new" {
                if let Some(recv) = call.receiver() {
                    if is_struct_const(&recv) {
                        let _ = block;
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn is_struct_const(node: &ruby_prism::Node<'_>) -> bool {
    // Check for `Struct` (ConstantReadNode) or `::Struct` (ConstantPathNode)
    if let Some(c) = node.as_constant_read_node() {
        return c.name().as_slice() == b"Struct";
    }
    if let Some(cp) = node.as_constant_path_node() {
        return cp.parent().is_none() && cp.name().is_some_and(|n| n.as_slice() == b"Struct");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(StructInheritance, "cops/style/struct_inheritance");
    crate::cop_autocorrect_fixture_tests!(StructInheritance, "cops/style/struct_inheritance");
}
