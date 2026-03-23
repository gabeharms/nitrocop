use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

/// Style/SymbolArray: flags bracket arrays of symbols that could use %i.
///
/// Investigation (FP=152): The main source of false positives was missing the
/// `invalid_percent_array_context?` check from RuboCop's PercentArray mixin.
/// When a bracket symbol array is an argument to a non-parenthesized method
/// call that also has a block (e.g. `can [:admin, :read], Model do ... end`),
/// `%i[...]` would be ambiguous — Ruby cannot distinguish the `{` as a block
/// vs hash literal. RuboCop exempts these arrays and so must we.
///
/// Also added `complex_content?` check: symbols containing spaces or
/// unmatched delimiters (`[]`, `()`) cannot be represented in `%i` syntax.
///
/// ## Corpus investigation (2026-03-15)
///
/// Corpus oracle reported FP=0, FN=8,702. Match rate 62.1%.
///
/// FN=8,702: Fixed. The `in_ambiguous_block_context` flag was set for the
/// entire CallNode subtree (including the block body), but RuboCop's
/// `invalid_percent_array_context?` only checks direct arguments of the
/// non-parenthesized call. This caused every symbol array inside the block
/// body of `describe "x" do`, `it "y" do`, `context "z" do`, etc. to be
/// incorrectly suppressed — a massive miss in spec-heavy repos. Fixed by
/// scoping the flag to only the arguments subtree, not the block body.
pub struct SymbolArray;

/// Delimiter characters that cannot appear unmatched in %i arrays.
const DELIMITERS: &[char] = &['[', ']', '(', ')'];

/// Check if a symbol has "complex content" that %i can't represent.
/// Matches RuboCop's `complex_content?` method: symbols with spaces or
/// unmatched delimiters (after removing balanced non-space-containing pairs).
fn symbol_has_complex_content(sym_node: &ruby_prism::SymbolNode<'_>) -> bool {
    let value = sym_node.unescaped();
    let content = match std::str::from_utf8(value) {
        Ok(s) => s,
        Err(_) => return true,
    };

    if content.contains(' ') {
        return true;
    }

    // Strip matched delimiter pairs that don't contain spaces or nested delimiters,
    // then check for remaining unmatched delimiters.
    let stripped = strip_balanced_pairs(content);
    DELIMITERS.iter().any(|d| stripped.contains(*d))
}

/// Remove balanced `[...]` and `(...)` pairs whose contents have no spaces
/// or nested delimiters. Matches RuboCop's gsub with
/// `/(\[[^\s\[\]]*\])|(\([^\s()]*\))/`.
fn strip_balanced_pairs(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '[' || chars[i] == '(' {
            let close = if chars[i] == '[' { ']' } else { ')' };
            // Look for matching close without spaces or nested delimiters
            if let Some(end) = find_simple_close(&chars, i + 1, close) {
                i = end + 1; // skip the matched pair
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// Find a closing delimiter that contains no spaces, brackets, or parens.
fn find_simple_close(chars: &[char], start: usize, close: char) -> Option<usize> {
    for (offset, &ch) in chars[start..].iter().enumerate() {
        if ch == close {
            return Some(start + offset);
        }
        if ch.is_whitespace() || ch == '[' || ch == ']' || ch == '(' || ch == ')' {
            return None;
        }
    }
    None
}

fn array_has_complex_content(array_node: &ruby_prism::ArrayNode<'_>) -> bool {
    for elem in array_node.elements().iter() {
        if let Some(sym) = elem.as_symbol_node() {
            if symbol_has_complex_content(&sym) {
                return true;
            }
        }
    }
    false
}

impl Cop for SymbolArray {
    fn name(&self) -> &'static str {
        "Style/SymbolArray"
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::parse::codemap::CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let min_size = config.get_usize("MinSize", 2);
        let enforced_style = config.get_str("EnforcedStyle", "percent");

        if enforced_style == "brackets" {
            return;
        }

        let mut visitor = SymbolArrayVisitor {
            cop: self,
            source,
            parse_result,
            min_size,
            diagnostics: Vec::new(),
            in_ambiguous_block_context: false,
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

struct SymbolArrayVisitor<'a, 'src, 'pr> {
    cop: &'a SymbolArray,
    source: &'src SourceFile,
    parse_result: &'a ruby_prism::ParseResult<'pr>,
    min_size: usize,
    diagnostics: Vec<Diagnostic>,
    /// True when we're inside arguments of a non-parenthesized call with a block.
    in_ambiguous_block_context: bool,
}

impl<'pr> SymbolArrayVisitor<'_, '_, 'pr> {
    fn check_array(&mut self, node: &ruby_prism::ArrayNode<'pr>) {
        // Must have `[` opening (not %i or %I)
        let opening = match node.opening_loc() {
            Some(loc) => loc,
            None => return,
        };

        if opening.as_slice() != b"[" {
            return;
        }

        let elements = node.elements();

        if elements.len() < self.min_size {
            return;
        }

        // Skip if in ambiguous block context (invalid_percent_array_context?)
        if self.in_ambiguous_block_context {
            return;
        }

        // Skip arrays containing comments — %i[] can't contain comments
        let array_start = opening.start_offset();
        let array_end = node
            .closing_loc()
            .map(|c| c.end_offset())
            .unwrap_or(array_start);
        if has_comment_in_range(self.parse_result, array_start, array_end) {
            return;
        }

        // All elements must be symbol nodes
        for elem in elements.iter() {
            if elem.as_symbol_node().is_none() {
                return;
            }
        }

        // Skip arrays with complex content (spaces, unmatched delimiters)
        if array_has_complex_content(node) {
            return;
        }

        let (line, column) = self.source.offset_to_line_col(opening.start_offset());
        self.diagnostics.push(self.cop.diagnostic(
            self.source,
            line,
            column,
            "Use `%i` or `%I` for an array of symbols.".to_string(),
        ));
    }

    /// Check if a call node represents an ambiguous block context:
    /// non-parenthesized method call with a block.
    fn is_ambiguous_block_call(&self, call: &ruby_prism::CallNode<'pr>) -> bool {
        // Must have a block
        if call.block().is_none() {
            return false;
        }
        // Must have arguments
        if call.arguments().is_none() {
            return false;
        }
        // Must NOT be parenthesized
        call.opening_loc().is_none()
    }
}

impl<'pr> Visit<'pr> for SymbolArrayVisitor<'_, '_, 'pr> {
    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode<'pr>) {
        self.check_array(node);
        ruby_prism::visit_array_node(self, node);
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if self.is_ambiguous_block_call(node) {
            // Visit receiver normally
            if let Some(receiver) = node.receiver() {
                self.visit(&receiver);
            }
            // Visit arguments — only suppress top-level ArrayNode arguments,
            // matching RuboCop's `parent.arguments.include?(node)` check.
            // Arrays nested inside keyword hashes are NOT ambiguous.
            if let Some(args) = node.arguments() {
                let prev = self.in_ambiguous_block_context;
                for arg in args.arguments().iter() {
                    if arg.as_array_node().is_some() {
                        self.in_ambiguous_block_context = true;
                        self.visit(&arg);
                        self.in_ambiguous_block_context = prev;
                    } else {
                        self.visit(&arg);
                    }
                }
            }
            // Visit block normally — arrays inside block body are NOT ambiguous
            if let Some(block) = node.block() {
                self.visit(&block);
            }
        } else {
            ruby_prism::visit_call_node(self, node);
        }
    }
}

/// Check if there are any comments within a byte offset range.
fn has_comment_in_range(
    parse_result: &ruby_prism::ParseResult<'_>,
    start: usize,
    end: usize,
) -> bool {
    for comment in parse_result.comments() {
        let comment_start = comment.location().start_offset();
        if comment_start >= start && comment_start < end {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(SymbolArray, "cops/style/symbol_array");

    #[test]
    fn config_min_size_5() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([("MinSize".into(), serde_yml::Value::Number(5.into()))]),
            ..CopConfig::default()
        };
        // 5 symbols should trigger with MinSize:5
        let source = b"x = [:a, :b, :c, :d, :e]\n";
        let diags = run_cop_full_with_config(&SymbolArray, source, config.clone());
        assert!(
            !diags.is_empty(),
            "Should fire with MinSize:5 on 5-element symbol array"
        );

        // 4 symbols should NOT trigger
        let source2 = b"x = [:a, :b, :c, :d]\n";
        let diags2 = run_cop_full_with_config(&SymbolArray, source2, config);
        assert!(
            diags2.is_empty(),
            "Should not fire on 4-element symbol array with MinSize:5"
        );
    }

    #[test]
    fn brackets_style_allows_bracket_arrays() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([(
                "EnforcedStyle".into(),
                serde_yml::Value::String("brackets".into()),
            )]),
            ..CopConfig::default()
        };
        let source = b"x = [:a, :b, :c]\n";
        let diags = run_cop_full_with_config(&SymbolArray, source, config);
        assert!(
            diags.is_empty(),
            "Should not flag brackets with brackets style"
        );
    }
}
