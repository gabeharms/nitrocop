use std::collections::HashMap;

use ruby_prism::Visit;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// Checks for ambiguous regexp literals in the first argument of a method
/// invocation without parentheses.
///
/// ## Implementation
///
/// RuboCop implements this by reading parser diagnostics (warnings) from the
/// Ruby parser. We do the same: Prism emits `PM_WARN_AMBIGUOUS_SLASH` warnings
/// when it encounters a `/` that could be either a regexp delimiter or division
/// operator. We iterate over `parse_result.warnings()` and report offenses for
/// those whose message matches the ambiguous slash pattern.
///
/// The previous AST-based approach (walking CallNodes to find regexp first
/// arguments without parentheses) missed several patterns:
/// - Regexp with method chain: `p /pattern/.do_something`
/// - MatchWriteNode arguments: `assert /pattern/ =~ string`
/// - Complex nesting patterns
///
/// Using Prism warnings directly is simpler, more correct, and mirrors
/// RuboCop's approach exactly.
pub struct AmbiguousRegexpLiteral;

impl Cop for AmbiguousRegexpLiteral {
    fn name(&self) -> &'static str {
        "Lint/AmbiguousRegexpLiteral"
    }

    fn supports_autocorrect(&self) -> bool {
        true
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
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut collector = RegexLocationCollector {
            by_start: HashMap::new(),
        };
        collector.visit(&parse_result.node());

        for warning in parse_result.warnings() {
            let msg = warning.message();
            // Prism emits PM_WARN_AMBIGUOUS_SLASH with message:
            // "ambiguous first argument; put parentheses or a space even after `/` operator"
            if !msg.contains("ambiguous") || !msg.contains('/') {
                continue;
            }

            let loc = warning.location();
            let start_offset = loc.start_offset();

            let (line, column) = source.offset_to_line_col(start_offset);
            let mut diag = self.diagnostic(
                source,
                line,
                column,
                "Ambiguous regexp literal. Parenthesize the method arguments if it's surely a regexp literal, or add a whitespace to the right of the `/` if it should be a division.".to_string(),
            );

            if let Some(corr) = corrections.as_mut() {
                if let Some((regex_start, regex_end)) =
                    collector.by_start.get(&start_offset).copied()
                {
                    corr.push(crate::correction::Correction {
                        start: regex_start,
                        end: regex_start,
                        replacement: "(".to_string(),
                        cop_name: self.name(),
                        cop_index: 0,
                    });
                    corr.push(crate::correction::Correction {
                        start: regex_end,
                        end: regex_end,
                        replacement: ")".to_string(),
                        cop_name: self.name(),
                        cop_index: 0,
                    });
                    diag.corrected = true;
                }
            }

            diagnostics.push(diag);
        }
    }
}

struct RegexLocationCollector {
    by_start: HashMap<usize, (usize, usize)>,
}

impl<'pr> Visit<'pr> for RegexLocationCollector {
    fn visit_regular_expression_node(&mut self, node: &ruby_prism::RegularExpressionNode<'pr>) {
        let loc = node.location();
        self.by_start
            .insert(loc.start_offset(), (loc.start_offset(), loc.end_offset()));
        ruby_prism::visit_regular_expression_node(self, node);
    }

    fn visit_interpolated_regular_expression_node(
        &mut self,
        node: &ruby_prism::InterpolatedRegularExpressionNode<'pr>,
    ) {
        let loc = node.location();
        self.by_start
            .insert(loc.start_offset(), (loc.start_offset(), loc.end_offset()));
        ruby_prism::visit_interpolated_regular_expression_node(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(AmbiguousRegexpLiteral, "cops/lint/ambiguous_regexp_literal");
    crate::cop_autocorrect_fixture_tests!(
        AmbiguousRegexpLiteral,
        "cops/lint/ambiguous_regexp_literal"
    );
}
