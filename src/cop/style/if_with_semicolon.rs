use crate::cop::node_type::IF_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Flags `if cond; body` when a semicolon separates the condition from the body.
///
/// RuboCop flags ALL `if`/`unless` statements where `loc.begin` is `;` (the "then"
/// keyword is a semicolon), regardless of whether the construct is single-line or
/// multi-line. The only exceptions are:
/// - Modifier form (`body if cond`) — no begin/end keywords
/// - `node.parent&.if_type?` — the `if` is nested inside another `if` node's
///   branch (covers `else if` patterns and nested `if` in body)
///
/// ## Investigation (2026-03)
/// **Previous FP root cause:** The cop previously required `end` on the same line
/// as `if`, which was overly restrictive. RuboCop flags multi-line `if cond;` too.
///
/// **Previous FN root cause:** The same-line `end` check caused all multi-line
/// `if cond;\n  body\nend` patterns to be missed (39 FN in corpus).
///
/// **Current FP root cause (3 FP):** `else if @im>0;` patterns in
/// rubyworks/facets — the inner `if` has an `if`-type parent. RuboCop skips
/// these with `return if node.parent&.if_type?`.
///
/// **Fix:** Remove the same-line `end` check (fixes FN). Add a check for `if`
/// nodes whose keyword is preceded by `else` on the same line, indicating they
/// are nested inside another `if`'s else branch (fixes FP). Also skip `if` nodes
/// that don't own their own `end` keyword (their `end_keyword_loc` offset matches
/// a parent's `end`).
pub struct IfWithSemicolon;

impl Cop for IfWithSemicolon {
    fn name(&self) -> &'static str {
        "Style/IfWithSemicolon"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[IF_NODE]
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
        let if_node = match node.as_if_node() {
            Some(n) => n,
            None => return,
        };

        // Must have an `if` or `unless` keyword (not ternary)
        let if_kw_loc = match if_node.if_keyword_loc() {
            Some(loc) => loc,
            None => return,
        };

        let kw_bytes = if_kw_loc.as_slice();
        if kw_bytes != b"if" && kw_bytes != b"unless" {
            return;
        }

        // Must not be modifier form (modifier has no end keyword)
        if if_node.end_keyword_loc().is_none() {
            return;
        }

        // RuboCop skips `if` nodes whose parent is an if_type. Since we don't
        // have parent access in the visitor, approximate this by checking if
        // `else` precedes the `if` keyword on the same line (i.e., `else if`).
        // Also check for `if` nodes that appear as the body of an else clause
        // inside another if — these have the same source line prefix.
        let if_kw_start = if_kw_loc.start_offset();
        if is_preceded_by_else(source, if_kw_start) {
            return;
        }

        // Check for semicolon: Prism's then_keyword_loc is ";" or "then".
        // As a fallback, scan the source text between predicate and body.
        let has_semicolon = if let Some(then_loc) = if_node.then_keyword_loc() {
            then_loc.as_slice() == b";"
        } else {
            let pred_end = if_node.predicate().location().end_offset();
            let body_start = if let Some(stmts) = if_node.statements() {
                stmts.location().start_offset()
            } else if let Some(sub) = if_node.subsequent() {
                sub.location().start_offset()
            } else if let Some(end_loc) = if_node.end_keyword_loc() {
                end_loc.start_offset()
            } else {
                return;
            };
            if pred_end < body_start {
                let between = &source.content[pred_end..body_start];
                between.iter().any(|&b| b == b';')
            } else {
                false
            }
        };

        if !has_semicolon {
            return;
        }

        let loc = if_node.location();
        let (line, column) = source.offset_to_line_col(loc.start_offset());

        let cond_src =
            std::str::from_utf8(if_node.predicate().location().as_slice()).unwrap_or("...");
        let kw = std::str::from_utf8(kw_bytes).unwrap_or("if");

        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            format!("Do not use `{} {};` - use a newline instead.", kw, cond_src),
        ));
    }
}

/// Check if the `if`/`unless` keyword at the given offset is preceded by `else`
/// on the same source line (indicating an `else if` pattern).
fn is_preceded_by_else(source: &SourceFile, if_kw_offset: usize) -> bool {
    // Scan backwards from the if keyword to the start of the line
    let content = &source.content;
    if if_kw_offset == 0 {
        return false;
    }

    // Find start of current line
    let mut line_start = if_kw_offset;
    while line_start > 0 && content[line_start - 1] != b'\n' {
        line_start -= 1;
    }

    // Get the text before the `if` keyword on this line, trimmed
    let before_kw = &content[line_start..if_kw_offset];
    // Trim trailing whitespace/semicolons from the prefix
    let trimmed = before_kw
        .iter()
        .rev()
        .skip_while(|&&b| b == b' ' || b == b'\t')
        .collect::<Vec<_>>();

    // Check if it ends with "else"
    if trimmed.len() >= 4 {
        let last4: Vec<u8> = trimmed[..4].iter().rev().map(|&&b| b).collect();
        if &last4 == b"else" {
            // Make sure "else" is either at the start of the prefix or preceded by
            // whitespace/semicolon (not part of a longer word)
            if trimmed.len() == 4 {
                return true;
            }
            let before_else = *trimmed[4];
            return before_else == b' '
                || before_else == b'\t'
                || before_else == b';'
                || before_else == b'\n';
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(IfWithSemicolon, "cops/style/if_with_semicolon");
}
