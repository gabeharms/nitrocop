use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Flags method definitions at the top level of a file.
///
/// Handles `def`, `def self.`, `define_method` (bare call or with receiver),
/// both with block form (`define_method(:foo) { ... }`, `define_method(:foo) do ... end`)
/// and proc argument form (`define_method(:foo, instance_method(:bar))`).
///
/// FN root cause (7 FN, 0 FP): only `DefNode` was checked. Corpus FNs were all
/// `define_method` calls at top level — with receivers like `Foo.define_method(:bar)`,
/// `Foo::Bar.singleton_class.define_method(:baz)`, or bare `define_method :name do`.
pub struct TopLevelMethodDefinition;

impl Cop for TopLevelMethodDefinition {
    fn name(&self) -> &'static str {
        "Style/TopLevelMethodDefinition"
    }

    fn default_enabled(&self) -> bool {
        false
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
        let root = parse_result.node();
        if let Some(program) = root.as_program_node() {
            let stmts = program.statements();
            for stmt in stmts.body().iter() {
                let should_flag = if stmt.as_def_node().is_some() {
                    // def foo / def self.foo
                    true
                } else if is_define_method_call(&stmt) {
                    // define_method(:foo, proc) — no block
                    true
                } else if Self::is_define_method_block_call(&stmt) {
                    // define_method(:foo) { ... } or define_method(:foo) do ... end
                    true
                } else {
                    false
                };

                if should_flag {
                    let loc = stmt.location();
                    let (line, column) = source.offset_to_line_col(loc.start_offset());
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Do not define methods at the top level.".to_string(),
                    ));
                }
            }
        }
    }
}

impl TopLevelMethodDefinition {
    /// Check if a top-level node is a `define_method` call with a block.
    fn is_define_method_block_call(node: &ruby_prism::Node<'_>) -> bool {
        if let Some(call) = node.as_call_node() {
            if call.block().is_some() {
                return is_define_method_name(&call);
            }
        }
        false
    }
}

/// Check if a node is a `define_method` call without a block (proc argument form).
fn is_define_method_call(node: &ruby_prism::Node<'_>) -> bool {
    if let Some(call) = node.as_call_node() {
        if call.block().is_none() {
            return is_define_method_name(&call);
        }
    }
    false
}

/// Check if a CallNode's method name is `define_method`.
fn is_define_method_name(call: &ruby_prism::CallNode<'_>) -> bool {
    call.name().as_slice() == b"define_method"
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        TopLevelMethodDefinition,
        "cops/style/top_level_method_definition"
    );
}
