use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// Checks for ambiguous operators in the first argument of a method invocation
/// without parentheses. For example, `do_something *some_array` where `*` could
/// be interpreted as either a splat or multiplication.
///
/// ## Implementation
///
/// Uses Prism parser warnings to detect ambiguous operators, matching RuboCop's
/// approach of relying on parser diagnostics (`:ambiguous_prefix` reason).
///
/// Prism emits these verbose-level warnings:
/// - `PM_WARN_AMBIGUOUS_PREFIX_STAR`: `*` splat vs multiplication
/// - `PM_WARN_AMBIGUOUS_PREFIX_STAR_STAR`: `**` keyword splat vs exponent
/// - `PM_WARN_AMBIGUOUS_PREFIX_AMPERSAND`: `&` block vs binary AND
/// - `PM_WARN_AMBIGUOUS_FIRST_ARGUMENT_PLUS`: `+` positive number vs addition
/// - `PM_WARN_AMBIGUOUS_FIRST_ARGUMENT_MINUS`: `-` negative number vs subtraction
///
/// ## Root cause of historical FNs (473 FNs, 50.7% match rate)
///
/// The original implementation only handled `*` (splat) via AST node inspection
/// of `CallNode`/`SplatNode`, missing `+`, `-`, `&`, and `**`. Switching to
/// Prism parser warnings covers all 5 operators in a single pass.
pub struct AmbiguousOperator;

/// Describes an ambiguous operator type.
struct AmbiguityInfo {
    actual: &'static str,
    operator: &'static str,
    possible: &'static str,
}

/// Try to classify a Prism warning message as an ambiguous operator.
fn classify_warning(message: &str) -> Option<AmbiguityInfo> {
    if message.contains("ambiguous `*`") && !message.contains("`**`") {
        Some(AmbiguityInfo {
            actual: "splat",
            operator: "*",
            possible: "a multiplication",
        })
    } else if message.contains("ambiguous `**`") {
        Some(AmbiguityInfo {
            actual: "keyword splat",
            operator: "**",
            possible: "an exponent",
        })
    } else if message.contains("ambiguous `&`") {
        Some(AmbiguityInfo {
            actual: "block",
            operator: "&",
            possible: "a binary AND",
        })
    } else if message.contains("after `+` operator") {
        Some(AmbiguityInfo {
            actual: "positive number",
            operator: "+",
            possible: "an addition",
        })
    } else if message.contains("after `-` operator") {
        Some(AmbiguityInfo {
            actual: "negative number",
            operator: "-",
            possible: "a subtraction",
        })
    } else {
        None
    }
}

impl Cop for AmbiguousOperator {
    fn name(&self) -> &'static str {
        "Lint/AmbiguousOperator"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        for warning in parse_result.warnings() {
            let message = warning.message();
            let info = match classify_warning(message) {
                Some(i) => i,
                None => continue,
            };

            let loc = warning.location();
            let start = loc.start_offset();
            let (line, column) = source.offset_to_line_col(start);

            let msg = format!(
                "Ambiguous {} operator. Parenthesize the method arguments \
                 if it's surely a {} operator, or add a whitespace to the \
                 right of the `{}` if it should be {}.",
                info.actual, info.actual, info.operator, info.possible
            );

            diagnostics.push(self.diagnostic(source, line, column, msg));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(AmbiguousOperator, "cops/lint/ambiguous_operator");
}
