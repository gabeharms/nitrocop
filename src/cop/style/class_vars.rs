use crate::cop::node_type::{
    CALL_NODE, CLASS_VARIABLE_AND_WRITE_NODE, CLASS_VARIABLE_OPERATOR_WRITE_NODE,
    CLASS_VARIABLE_OR_WRITE_NODE, CLASS_VARIABLE_TARGET_NODE, CLASS_VARIABLE_WRITE_NODE,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Checks for uses of class variables (`@@var`).
///
/// ### FN root cause (16 total)
/// `ClassVariableTargetNode` (from `MultiWriteNode` / parallel assignment like
/// `@@a, @@b = 1, 2`) was not handled. Added to `interested_node_types` and
/// `check_node`. Top FN repos: xiki (9), natalie (3), devise (1),
/// eventmachine (1), treat (1), rails (1).
pub struct ClassVars;

impl Cop for ClassVars {
    fn name(&self) -> &'static str {
        "Style/ClassVars"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            CALL_NODE,
            CLASS_VARIABLE_AND_WRITE_NODE,
            CLASS_VARIABLE_OPERATOR_WRITE_NODE,
            CLASS_VARIABLE_OR_WRITE_NODE,
            CLASS_VARIABLE_TARGET_NODE,
            CLASS_VARIABLE_WRITE_NODE,
        ]
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
        // Check class variable write: @@foo = 1
        if let Some(cvasgn) = node.as_class_variable_write_node() {
            let name = cvasgn.name();
            let name_str = String::from_utf8_lossy(name.as_slice());
            let loc = cvasgn.name_loc();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Replace class var {} with a class instance var.", name_str),
            ));
        }

        // Check class variable and-write: @@foo &&= 1
        if let Some(cvasgn) = node.as_class_variable_and_write_node() {
            let name = cvasgn.name();
            let name_str = String::from_utf8_lossy(name.as_slice());
            let loc = cvasgn.name_loc();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Replace class var {} with a class instance var.", name_str),
            ));
        }

        // Check class variable or-write: @@foo ||= 1
        if let Some(cvasgn) = node.as_class_variable_or_write_node() {
            let name = cvasgn.name();
            let name_str = String::from_utf8_lossy(name.as_slice());
            let loc = cvasgn.name_loc();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Replace class var {} with a class instance var.", name_str),
            ));
        }

        // Check class variable operator-write: @@foo += 1
        if let Some(cvasgn) = node.as_class_variable_operator_write_node() {
            let name = cvasgn.name();
            let name_str = String::from_utf8_lossy(name.as_slice());
            let loc = cvasgn.name_loc();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Replace class var {} with a class instance var.", name_str),
            ));
        }

        // Check class variable target in multi-assignment: @@foo, @@bar = 1, 2
        if let Some(cvt) = node.as_class_variable_target_node() {
            let name = cvt.name();
            let name_str = String::from_utf8_lossy(name.as_slice());
            let loc = cvt.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Replace class var {} with a class instance var.", name_str),
            ));
        }

        // Check class_variable_set(:@@foo, value) call
        if let Some(call_node) = node.as_call_node() {
            if call_node.name().as_slice() == b"class_variable_set" {
                if let Some(args) = call_node.arguments() {
                    let arg_list: Vec<_> = args.arguments().iter().collect();
                    if !arg_list.is_empty() {
                        let first_arg = &arg_list[0];
                        let arg_src = first_arg.location().as_slice();
                        let (line, column) =
                            source.offset_to_line_col(first_arg.location().start_offset());
                        diagnostics.push(self.diagnostic(
                            source,
                            line,
                            column,
                            format!(
                                "Replace class var {} with a class instance var.",
                                String::from_utf8_lossy(arg_src),
                            ),
                        ));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ClassVars, "cops/style/class_vars");
}
