use crate::cop::util::as_method_chain;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Performance/SelectMap — flags `select.map` / `filter.map` chains, suggests `filter_map`.
///
/// ## Investigation (2026-03-04)
/// Single FN in corpus: `select.map { |e| ... }` where `select` is called without a block
/// (returns an Enumerator). RuboCop's guard only skips when `select` has non-block-pass
/// arguments (e.g., `select(key: value)`); bare `select` with no args passes through.
/// nitrocop was too strict — it required `inner_call.block()` to be present. Fixed to match
/// RuboCop: allow no-argument calls and block-pass args, only skip non-block-pass arguments.
pub struct SelectMap;

impl Cop for SelectMap {
    fn name(&self) -> &'static str {
        "Performance/SelectMap"
    }

    fn default_enabled(&self) -> bool {
        false
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
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
        let chain = match as_method_chain(node) {
            Some(c) => c,
            None => return,
        };

        if chain.outer_method != b"map" {
            return;
        }

        let inner = chain.inner_method;
        let inner_name = if inner == b"select" {
            "select"
        } else if inner == b"filter" {
            "filter"
        } else {
            return;
        };

        // RuboCop's guard: `return if (first_argument = node.first_argument) && !first_argument.block_pass_type?`
        // Skip if select/filter has non-block-pass arguments (e.g., `select(key: value)`).
        // Allow: no arguments at all (bare `select.map`) or block-pass (`select(&:foo).map`).
        if let Some(args) = chain.inner_call.arguments() {
            let arg_list = args.arguments();
            if !arg_list.is_empty() {
                let first = arg_list.iter().next().unwrap();
                if first.as_block_argument_node().is_none() {
                    return;
                }
            }
        }

        // If there's a block on the inner call, check for numblock/it patterns.
        // RuboCop's Parser gem has separate `block` and `numblock` node types.
        // `numblock` (used for _1/_2 numbered params and Ruby 3.4 `it`) returns
        // false for `block_type?`, causing RuboCop to skip these chains.
        if let Some(inner_block) = chain.inner_call.block() {
            if let Some(block_node) = inner_block.as_block_node() {
                if let Some(params) = block_node.parameters() {
                    if params.as_numbered_parameters_node().is_some()
                        || params.as_it_parameters_node().is_some()
                    {
                        return;
                    }
                }
            }
        }

        // Report at the inner method name (.select/.filter) to match RuboCop's offense_range
        let loc = chain
            .inner_call
            .message_loc()
            .unwrap_or_else(|| chain.inner_call.location());
        let (line, column) = source.offset_to_line_col(loc.start_offset());
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            format!("Use `filter_map` instead of `{inner_name}.map`."),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(SelectMap, "cops/performance/select_map");
}
