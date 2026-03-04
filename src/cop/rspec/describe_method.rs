use crate::cop::util::RSPEC_DEFAULT_INCLUDE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// RuboCop's DescribeMethod uses `TopLevelGroup` mixin, which only inspects the
/// outermost describe/context block. Nested describes (e.g., `describe Klass, "behavior"`)
/// inside an outer `describe Klass do` are NOT checked. Our original implementation used
/// `check_node` which visited ALL describe calls regardless of nesting, causing 148 FPs.
///
/// Fix: switched to `check_source` to walk only top-level statements (unwrapping
/// module/class/begin wrappers) and check describe calls at that level only.
pub struct DescribeMethod;

impl Cop for DescribeMethod {
    fn name(&self) -> &'static str {
        "RSpec/DescribeMethod"
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_DEFAULT_INCLUDE
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::parse::codemap::CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let program = match parse_result.node().as_program_node() {
            Some(p) => p,
            None => return,
        };

        let stmts: Vec<ruby_prism::Node<'_>> = program.statements().body().iter().collect();

        // Mirror RuboCop's TopLevelGroup logic: if single top-level statement,
        // unwrap module/class/begin wrappers. If multiple, check each directly.
        if stmts.len() == 1 {
            self.collect_from_wrapper(&stmts[0], source, diagnostics);
        } else {
            for stmt in &stmts {
                self.check_describe_call(stmt, source, diagnostics);
            }
        }
    }
}

impl DescribeMethod {
    /// Unwrap module/class/begin nodes to find top-level describe calls.
    fn collect_from_wrapper(
        &self,
        node: &ruby_prism::Node<'_>,
        source: &SourceFile,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Direct describe call
        if self.check_describe_call(node, source, diagnostics) {
            return;
        }

        // Unwrap module
        if let Some(module_node) = node.as_module_node() {
            if let Some(body) = module_node.body() {
                for child in body
                    .as_statements_node()
                    .iter()
                    .flat_map(|s| s.body().iter())
                {
                    self.collect_from_wrapper(&child, source, diagnostics);
                }
            }
            return;
        }

        // Unwrap class
        if let Some(class_node) = node.as_class_node() {
            if let Some(body) = class_node.body() {
                for child in body
                    .as_statements_node()
                    .iter()
                    .flat_map(|s| s.body().iter())
                {
                    self.collect_from_wrapper(&child, source, diagnostics);
                }
            }
            return;
        }

        // Unwrap begin
        if let Some(begin_node) = node.as_begin_node() {
            if let Some(stmts) = begin_node.statements() {
                for child in stmts.body().iter() {
                    self.collect_from_wrapper(&child, source, diagnostics);
                }
            }
        }
    }

    /// Check if a node is a describe call with a non-method string second argument.
    /// Returns true if the node was a describe call (regardless of offense).
    fn check_describe_call(
        &self,
        node: &ruby_prism::Node<'_>,
        source: &SourceFile,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> bool {
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return false,
        };

        let name = call.name().as_slice();

        // Match both `describe` and `RSpec.describe`
        let is_describe =
            name == b"describe" && (call.receiver().is_none() || is_rspec_receiver(&call));

        if !is_describe {
            return false;
        }

        // Must have a block to be a top-level group
        if call.block().is_none() {
            return false;
        }

        let args = match call.arguments() {
            Some(a) => a,
            None => return true,
        };

        let arg_list: Vec<ruby_prism::Node<'_>> = args.arguments().iter().collect();

        // Need at least 2 args: a class and a string description
        if arg_list.len() < 2 {
            return true;
        }

        // First argument should be a class/constant
        if arg_list[0].as_constant_read_node().is_none()
            && arg_list[0].as_constant_path_node().is_none()
        {
            return true;
        }

        // Second argument should be a string
        let string_arg = if let Some(s) = arg_list[1].as_string_node() {
            s
        } else {
            return true;
        };

        let content = string_arg.unescaped();
        let content_str = match std::str::from_utf8(content) {
            Ok(s) => s,
            Err(_) => return true,
        };

        // Method descriptions must start with '#' or '.'
        if content_str.starts_with('#') || content_str.starts_with('.') {
            return true;
        }

        let loc = arg_list[1].location();
        let (line, column) = source.offset_to_line_col(loc.start_offset());
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            "The second argument to describe should be the method being tested. '#instance' or '.class'.".to_string(),
        ));
        true
    }
}

/// Check if a call's receiver is `RSpec` (for `RSpec.describe`).
fn is_rspec_receiver(call: &ruby_prism::CallNode<'_>) -> bool {
    call.receiver()
        .and_then(|r| r.as_constant_read_node())
        .is_some_and(|c| c.name().as_slice() == b"RSpec")
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(DescribeMethod, "cops/rspec/describe_method");
}
