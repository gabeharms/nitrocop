use crate::cop::node_type::{
    ARRAY_NODE, FLOAT_NODE, INTEGER_NODE, INTERPOLATED_STRING_NODE, SPLAT_NODE, STRING_NODE,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/RedundantSplatExpansion — detects unnecessary `*` on literals.
///
/// ## AllowPercentLiteralArrayArgument handling
///
/// RuboCop's `use_percent_literal_array_argument?` checks
/// `method_argument?(node) && percent_literal?`, where `method_argument?`
/// means `node.parent.call_type?` — i.e., the splat is a direct child of
/// the call's arguments list. When `*%w[...]` appears inside an array
/// literal `[*%w[...]]` that is itself a method argument, the splat's
/// parent is the ArrayNode, not the CallNode, so the exemption does NOT
/// apply. Previously nitrocop skipped ALL percent literal splats
/// unconditionally, causing 17 FN in the corpus (mostly jruby patterns
/// like `assert_in_out_err([*%W"--disable=gems ..."])`).
///
/// Fix: only exempt percent literal splats when their immediate parent is
/// a method call (detected by scanning backwards from the `*` to find the
/// nearest enclosing `(` or `[`).
pub struct RedundantSplatExpansion;

impl Cop for RedundantSplatExpansion {
    fn name(&self) -> &'static str {
        "Lint/RedundantSplatExpansion"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            ARRAY_NODE,
            FLOAT_NODE,
            INTEGER_NODE,
            INTERPOLATED_STRING_NODE,
            SPLAT_NODE,
            STRING_NODE,
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let allow_percent = config.get_bool("AllowPercentLiteralArrayArgument", true);

        let splat = match node.as_splat_node() {
            Some(s) => s,
            None => return,
        };

        let child = match splat.expression() {
            Some(e) => e,
            None => return,
        };

        // Check if the splat is on a literal: array, string, integer, float
        let is_literal = child.as_array_node().is_some()
            || child.as_string_node().is_some()
            || child.as_integer_node().is_some()
            || child.as_float_node().is_some()
            || child.as_interpolated_string_node().is_some();

        if !is_literal {
            return;
        }

        // Determine if this is an array splat (child is array) inside an
        // explicit array literal `[...]` — affects both the exemption and message.
        let is_array_splat = child.as_array_node().is_some();
        let in_array_literal = is_array_splat && is_inside_array_literal(source, &splat);

        // When AllowPercentLiteralArrayArgument is true (default), skip
        // percent literal arrays that are NOT inside array literals.
        // RuboCop checks: method_argument?(node) && percent_literal?
        // When *%w[...] is inside [*%w[...]], the splat's parent is the
        // ArrayNode, not the CallNode, so method_argument? is false and
        // the exemption does NOT apply.
        if allow_percent && is_array_splat && !in_array_literal {
            if let Some(array_node) = child.as_array_node() {
                if is_percent_literal(&array_node) {
                    return;
                }
            }
        }

        let loc = splat.location();
        let (line, column) = source.offset_to_line_col(loc.start_offset());

        // Use the "pass as separate arguments" message when the splat is
        // in a context where the array contents should be inlined.
        let is_method_arg = is_array_splat && is_direct_method_argument(source, &splat);
        let message = if is_method_arg || in_array_literal {
            "Pass array contents as separate arguments."
        } else {
            "Replace splat expansion with comma separated values."
        };

        diagnostics.push(self.diagnostic(source, line, column, message.to_string()));
    }
}

/// Check if an array node is a percent literal (%w, %W, %i, %I).
fn is_percent_literal(array_node: &ruby_prism::ArrayNode<'_>) -> bool {
    if let Some(open_loc) = array_node.opening_loc() {
        let open = open_loc.as_slice();
        return open.starts_with(b"%w")
            || open.starts_with(b"%W")
            || open.starts_with(b"%i")
            || open.starts_with(b"%I");
    }
    false
}

/// Check if the splat is inside an explicit array literal `[...]`.
/// Scans backwards from the `*` to find the nearest unmatched `[` or `(`.
/// If `[` is found first, the splat is inside an array literal.
fn is_inside_array_literal(source: &SourceFile, splat: &ruby_prism::SplatNode<'_>) -> bool {
    let bytes = source.as_bytes();
    let start = splat.location().start_offset();
    find_enclosing_bracket(bytes, start) == Some(b'[')
}

/// Check if the splat is a direct method argument (parent is a call node).
/// Scans backwards from the `*` to find the nearest unmatched `[` or `(`.
/// If `(` is found first, the splat is a direct method argument.
fn is_direct_method_argument(source: &SourceFile, splat: &ruby_prism::SplatNode<'_>) -> bool {
    let bytes = source.as_bytes();
    let start = splat.location().start_offset();
    find_enclosing_bracket(bytes, start) == Some(b'(')
}

/// Scan backwards from `pos` to find the nearest unmatched `[` or `(`,
/// tracking bracket nesting. Returns the bracket character found, or None.
fn find_enclosing_bracket(bytes: &[u8], pos: usize) -> Option<u8> {
    let mut depth_square: i32 = 0;
    let mut depth_paren: i32 = 0;
    let mut i = pos;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b']' => depth_square += 1,
            b'[' => {
                if depth_square == 0 {
                    return Some(b'[');
                }
                depth_square -= 1;
            }
            b')' => depth_paren += 1,
            b'(' => {
                if depth_paren == 0 {
                    return Some(b'(');
                }
                depth_paren -= 1;
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        RedundantSplatExpansion,
        "cops/lint/redundant_splat_expansion"
    );
}
