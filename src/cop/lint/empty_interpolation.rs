use ruby_prism::Visit;

use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::codemap::CodeMap;
use crate::parse::source::SourceFile;

/// Checks for interpolations that only contain `nil` or an empty string literal.
///
/// ## Corpus investigation (2026-03-10)
///
/// Corpus oracle reported FP=0, FN=14.
///
/// FN:
/// - The original implementation only treated `#{}` / `#{ }` as empty. RuboCop also flags
///   `#{''}`, `#{""}`, and `#{nil}`.
/// - `%W` / `%I` arrays are exempt for this cop, so the Prism port needs source-level traversal
///   to track percent-array context instead of relying on generic parent links.
/// - Prism also exposes heredoc interpolation like `#{<<~TEXT}` as an embedded node whose
///   statements body looks empty, so the source form has to be checked before flagging.
pub struct EmptyInterpolation;

impl Cop for EmptyInterpolation {
    fn name(&self) -> &'static str {
        "Lint/EmptyInterpolation"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        _parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let mut visitor = EmptyInterpolationVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
            corrections,
            in_percent_literal_array: false,
        };
        visitor.visit(&_parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

fn embedded_body_is_empty(embedded: &ruby_prism::EmbeddedStatementsNode<'_>) -> bool {
    let Some(statements) = embedded.statements() else {
        return true;
    };

    statements
        .body()
        .iter()
        .all(|node| interpolation_child_is_empty(&node))
}

fn interpolation_child_is_empty(node: &ruby_prism::Node<'_>) -> bool {
    node.as_nil_node().is_some()
        || node
            .as_string_node()
            .is_some_and(|string| string.unescaped().is_empty())
}

struct EmptyInterpolationVisitor<'a, 'src, 'corr> {
    cop: &'a EmptyInterpolation,
    source: &'src SourceFile,
    diagnostics: Vec<Diagnostic>,
    corrections: Option<&'corr mut Vec<crate::correction::Correction>>,
    in_percent_literal_array: bool,
}

impl EmptyInterpolationVisitor<'_, '_, '_> {
    fn check_embedded(&mut self, embedded: &ruby_prism::EmbeddedStatementsNode<'_>) {
        if self.in_percent_literal_array
            || heredoc_interpolation(self.source, embedded)
            || !embedded_body_is_empty(embedded)
        {
            return;
        }

        let loc = embedded.location();
        let (line, column) = self.source.offset_to_line_col(loc.start_offset());
        let mut diag = self.cop.diagnostic(
            self.source,
            line,
            column,
            "Empty interpolation detected.".to_string(),
        );

        if let Some(corrections) = self.corrections.as_mut() {
            corrections.push(crate::correction::Correction {
                start: loc.start_offset(),
                end: loc.end_offset(),
                replacement: String::new(),
                cop_name: self.cop.name(),
                cop_index: 0,
            });
            diag.corrected = true;
        }

        self.diagnostics.push(diag);
    }
}

fn heredoc_interpolation(
    source: &SourceFile,
    embedded: &ruby_prism::EmbeddedStatementsNode<'_>,
) -> bool {
    let loc = embedded.location();
    let bytes = &source.as_bytes()[loc.start_offset()..loc.end_offset()];
    if !bytes.starts_with(b"#{") {
        return false;
    }

    let mut i = 2;
    while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b'\n' | b'\r') {
        i += 1;
    }

    bytes[i..].starts_with(b"<<")
}

impl<'pr> Visit<'pr> for EmptyInterpolationVisitor<'_, '_, '_> {
    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode<'pr>) {
        let was_percent_literal_array = self.in_percent_literal_array;

        if let Some(opening) = node.opening_loc() {
            let opening = opening.as_slice();
            if opening.starts_with(b"%W") || opening.starts_with(b"%I") {
                self.in_percent_literal_array = true;
            }
        }

        ruby_prism::visit_array_node(self, node);

        self.in_percent_literal_array = was_percent_literal_array;
    }

    fn visit_embedded_statements_node(&mut self, node: &ruby_prism::EmbeddedStatementsNode<'pr>) {
        self.check_embedded(node);
        ruby_prism::visit_embedded_statements_node(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(EmptyInterpolation, "cops/lint/empty_interpolation");
    crate::cop_autocorrect_fixture_tests!(EmptyInterpolation, "cops/lint/empty_interpolation");
}
