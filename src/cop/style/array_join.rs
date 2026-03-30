use crate::cop::node_type::{ARRAY_NODE, CALL_NODE, INTERPOLATED_STRING_NODE, STRING_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ArrayJoin;

impl Cop for ArrayJoin {
    fn name(&self) -> &'static str {
        "Style/ArrayJoin"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[ARRAY_NODE, CALL_NODE, INTERPOLATED_STRING_NODE, STRING_NODE]
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
        let call_node = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        // Must be the `*` method
        if call_node.name().as_slice() != b"*" {
            return;
        }

        // The receiver must be an array literal
        let receiver = match call_node.receiver() {
            Some(r) => r,
            None => return,
        };

        if receiver.as_array_node().is_none() {
            return;
        }

        // The argument must be a string literal
        let args = match call_node.arguments() {
            Some(a) => a,
            None => return,
        };

        let arg_list: Vec<_> = args.arguments().iter().collect();
        if arg_list.len() != 1 {
            return;
        }

        if arg_list[0].as_string_node().is_none()
            && arg_list[0].as_interpolated_string_node().is_none()
        {
            return;
        }

        let msg_loc = call_node
            .message_loc()
            .unwrap_or_else(|| call_node.location());
        let (line, column) = source.offset_to_line_col(msg_loc.start_offset());
        let mut diag = self.diagnostic(
            source,
            line,
            column,
            "Favor `Array#join` over `Array#*`.".to_string(),
        );

        if let Some(ref mut corr) = corrections {
            let array_src = std::str::from_utf8(receiver.location().as_slice()).unwrap_or("");
            let arg_src = std::str::from_utf8(arg_list[0].location().as_slice()).unwrap_or("");
            if !array_src.is_empty() && !arg_src.is_empty() {
                corr.push(crate::correction::Correction {
                    start: call_node.location().start_offset(),
                    end: call_node.location().end_offset(),
                    replacement: format!("{array_src}.join({arg_src})"),
                    cop_name: self.name(),
                    cop_index: 0,
                });
                diag.corrected = true;
            }
        }

        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ArrayJoin, "cops/style/array_join");
    crate::cop_autocorrect_fixture_tests!(ArrayJoin, "cops/style/array_join");
}
