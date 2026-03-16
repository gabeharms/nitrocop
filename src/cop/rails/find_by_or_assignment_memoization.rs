use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

/// Investigation notes (2026-03-16):
///
/// Root cause of 8 FNs in `basecamp__fizzy__a02042b`: the visitor's
/// `visit_instance_variable_or_write_node` was returning early (without recursing into children)
/// when the value was not a direct `find_by` call. This caused nested `||=` inside
/// `@outer ||= begin ... Class.new do ... def method ... @inner ||= foo.find_by(...) end end end`
/// patterns to be silently skipped — the visitor never descended into the begin block or the
/// anonymous class body.
///
/// Fix: always call `ruby_prism::visit_instance_variable_or_write_node` at the end to recurse,
/// even when the current node doesn't match (value is not a direct find_by call, or we're
/// inside an if/unless). The if/unless skip logic (`in_if_depth`) correctly matches RuboCop's
/// `assignment_node.each_ancestor(:if).any?` check — a `||=` inside an if body is skipped
/// regardless of depth in other constructs.
///
/// After fix: FN=0, FP=0 on corpus.
pub struct FindByOrAssignmentMemoization;

/// Check if a node is a `find_by` call (not `find_by!`) without safe navigation.
fn is_find_by_call_without_safe_nav(node: &ruby_prism::Node<'_>) -> bool {
    if let Some(call) = node.as_call_node() {
        if call.name().as_slice() != b"find_by" {
            return false;
        }
        // RuboCop uses (send ...) not (csend ...), so &.find_by is excluded
        if call
            .call_operator_loc()
            .is_some_and(|op| op.as_slice() == b"&.")
        {
            return false;
        }
        return true;
    }
    false
}

impl Cop for FindByOrAssignmentMemoization {
    fn name(&self) -> &'static str {
        "Rails/FindByOrAssignmentMemoization"
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
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
        let mut visitor = FindByVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
            in_if_depth: 0,
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

struct FindByVisitor<'a> {
    cop: &'a FindByOrAssignmentMemoization,
    source: &'a SourceFile,
    diagnostics: Vec<Diagnostic>,
    in_if_depth: usize,
}

impl<'pr> Visit<'pr> for FindByVisitor<'_> {
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
        self.in_if_depth += 1;
        ruby_prism::visit_if_node(self, node);
        self.in_if_depth -= 1;
    }

    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
        self.in_if_depth += 1;
        ruby_prism::visit_unless_node(self, node);
        self.in_if_depth -= 1;
    }

    fn visit_instance_variable_or_write_node(
        &mut self,
        node: &ruby_prism::InstanceVariableOrWriteNode<'pr>,
    ) {
        // When inside an if/unless, the ||= itself has an if ancestor — RuboCop skips these.
        // We still recurse into children because an inner def (e.g., inside a block passed
        // to Class.new) starts a fresh method scope and its own ||= nodes are independent.
        if self.in_if_depth == 0 {
            let value = node.value();

            // The value should be a direct find_by call (not part of || or ternary),
            // and not using safe navigation (&.find_by)
            if is_find_by_call_without_safe_nav(&value) {
                let loc = node.location();
                let (line, column) = self.source.offset_to_line_col(loc.start_offset());
                self.diagnostics.push(self.cop.diagnostic(
                    self.source,
                    line,
                    column,
                    "Avoid memoizing `find_by` results with `||=`.".to_string(),
                ));
            }
        }

        // Always recurse into children so we catch ||= nodes nested inside begin blocks,
        // blocks passed to Class.new, inner method definitions, etc.
        ruby_prism::visit_instance_variable_or_write_node(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        FindByOrAssignmentMemoization,
        "cops/rails/find_by_or_assignment_memoization"
    );
}
