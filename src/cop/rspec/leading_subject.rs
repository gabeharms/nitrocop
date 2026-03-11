use crate::cop::node_type::PROGRAM_NODE;
use crate::cop::util::{
    self, RSPEC_DEFAULT_INCLUDE, is_rspec_example, is_rspec_example_group, is_rspec_hook,
    is_rspec_let, is_rspec_subject,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// RSpec/LeadingSubject checks that `subject` is declared before `let`, hooks,
/// examples, and other declarations within an example group.
///
/// RuboCop uses `InsideExampleGroup` to determine whether a `subject` node
/// should be checked. This check walks up to the file's root-level node and
/// verifies it is a spec group (describe/context/shared_examples block). When
/// the describe block is wrapped in a `module` or `class` declaration, the
/// root-level node is the module/class — NOT a spec group — so RuboCop skips
/// the cop entirely. This is a documented side-effect of `InsideExampleGroup`.
///
/// We replicate this by only checking subjects inside spec groups that are
/// at the file's top level (direct children of the program node, or within a
/// top-level `begin`). Spec groups inside module/class wrappers are skipped.
///
/// ## Investigation (2026-03-11)
///
/// **Root cause of 118 FNs:** Two issues found:
///
/// 1. Include-family blocks (it_behaves_like, include_context, include_examples,
///    it_should_behave_like) were not recursed into. RuboCop's `on_block` fires
///    on ALL blocks, so subjects inside `it_behaves_like do...end` are checked
///    independently for ordering within that block. The nitrocop code only
///    recursed into example group blocks (describe/context/shared_examples).
///    Fixed by adding `recurse_into_block()` for include-family calls.
///
/// 2. `RSpec.describe` nested inside another example group was recursed into but
///    NOT treated as an offending node (the `continue` after recursion skipped
///    the `first_relevant_name` update). RuboCop's `spec_group?` includes
///    `RSpec.describe`, so it IS offending. Fixed by setting `first_relevant_name`
///    for `RSpec.describe` calls.
pub struct LeadingSubject;

impl Cop for LeadingSubject {
    fn name(&self) -> &'static str {
        "RSpec/LeadingSubject"
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_DEFAULT_INCLUDE
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[PROGRAM_NODE]
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
        let program = match node.as_program_node() {
            Some(p) => p,
            None => return,
        };

        // Walk top-level statements looking for spec groups.
        // Only spec groups at the file root (not inside module/class) are checked,
        // matching RuboCop's InsideExampleGroup behavior.
        for stmt in program.statements().body().iter() {
            if is_spec_group_call(&stmt) {
                self.check_example_group_recursive(source, &stmt, diagnostics);
            }
            // Skip modules, classes, requires, and anything else at the top level.
        }
    }
}

impl LeadingSubject {
    /// Recursively check an example group and all nested example groups
    /// for subject ordering violations.
    fn check_example_group_recursive(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        let block = match call.block() {
            Some(b) => match b.as_block_node() {
                Some(bn) => bn,
                None => return,
            },
            None => return,
        };

        let body = match block.body() {
            Some(b) => b,
            None => return,
        };

        let stmts = match body.as_statements_node() {
            Some(s) => s,
            None => return,
        };

        // Check subject ordering within this example group
        let mut first_relevant_name: Option<&[u8]> = None;

        for stmt in stmts.body().iter() {
            if let Some(c) = stmt.as_call_node() {
                let name = c.name().as_slice();

                // Handle calls with receiver (e.g. RSpec.describe)
                if c.receiver().is_some() {
                    let is_rspec_describe = util::constant_name(&c.receiver().unwrap())
                        .is_some_and(|n| n == b"RSpec")
                        && name == b"describe";
                    if is_rspec_describe {
                        // Recurse into RSpec.describe nested calls
                        self.check_example_group_recursive(source, &stmt, diagnostics);
                        // Also treat as offending (spec_group in RuboCop)
                        if first_relevant_name.is_none() {
                            first_relevant_name = Some(name);
                        }
                    }
                    continue;
                }

                if is_rspec_subject(name) {
                    // Subject found -- check if something relevant came before it
                    if let Some(prev_name) = first_relevant_name {
                        let prev_str = std::str::from_utf8(prev_name).unwrap_or("let");
                        let loc = stmt.location();
                        let (line, column) = source.offset_to_line_col(loc.start_offset());
                        diagnostics.push(self.diagnostic(
                            source,
                            line,
                            column,
                            format!("Declare `subject` above any other `{prev_str}` declarations."),
                        ));
                    }
                } else if is_rspec_example_group(name) {
                    // Recurse into nested context/describe/shared_examples blocks
                    self.check_example_group_recursive(source, &stmt, diagnostics);
                    if first_relevant_name.is_none() {
                        first_relevant_name = Some(name);
                    }
                } else if is_example_include(name) {
                    // Recurse into include-family blocks (it_behaves_like,
                    // include_context, etc.) — subjects inside these blocks
                    // are independently checked for ordering, matching RuboCop's
                    // on_block + parent(node) behavior.
                    self.recurse_into_block(source, &stmt, diagnostics);
                    if first_relevant_name.is_none() {
                        first_relevant_name = Some(name);
                    }
                } else if (is_rspec_let(name) || is_rspec_hook(name) || is_rspec_example(name))
                    && first_relevant_name.is_none()
                {
                    first_relevant_name = Some(name);
                }
            }
        }
    }

    /// Recurse into a block-bearing call (e.g. it_behaves_like, include_context)
    /// to check subject ordering within its block body. This matches RuboCop's
    /// on_block + parent(node) behavior where subjects inside any block within
    /// a spec group are checked independently.
    fn recurse_into_block(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        let block = match call.block() {
            Some(b) => match b.as_block_node() {
                Some(bn) => bn,
                None => return,
            },
            None => return,
        };

        let body = match block.body() {
            Some(b) => b,
            None => return,
        };

        let stmts = match body.as_statements_node() {
            Some(s) => s,
            None => return,
        };

        // Check subject ordering within this block, same as example group
        let mut first_relevant_name: Option<&[u8]> = None;

        for stmt in stmts.body().iter() {
            if let Some(c) = stmt.as_call_node() {
                let name = c.name().as_slice();
                if c.receiver().is_some() {
                    continue;
                }

                if is_rspec_subject(name) {
                    if let Some(prev_name) = first_relevant_name {
                        let prev_str = std::str::from_utf8(prev_name).unwrap_or("let");
                        let loc = stmt.location();
                        let (line, column) = source.offset_to_line_col(loc.start_offset());
                        diagnostics.push(self.diagnostic(
                            source,
                            line,
                            column,
                            format!("Declare `subject` above any other `{prev_str}` declarations."),
                        ));
                    }
                } else if (is_rspec_let(name)
                    || is_rspec_hook(name)
                    || is_rspec_example(name)
                    || is_rspec_example_group(name)
                    || is_example_include(name))
                    && first_relevant_name.is_none()
                {
                    first_relevant_name = Some(name);
                }
            }
        }
    }
}

fn is_spec_group_call(node: &ruby_prism::Node<'_>) -> bool {
    let call = match node.as_call_node() {
        Some(c) => c,
        None => return false,
    };
    let name = call.name().as_slice();
    if let Some(recv) = call.receiver() {
        util::constant_name(&recv).is_some_and(|n| n == b"RSpec") && name == b"describe"
    } else {
        is_rspec_example_group(name)
    }
}

fn is_example_include(name: &[u8]) -> bool {
    name == b"include_examples"
        || name == b"it_behaves_like"
        || name == b"it_should_behave_like"
        || name == b"include_context"
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(LeadingSubject, "cops/rspec/leading_subject");
}
