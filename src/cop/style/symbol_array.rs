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
///
/// ## Corpus investigation (2026-03-23)
///
/// Corpus oracle reported FP=0, FN=2. Match rate 99.9%.
///
/// FN=1 (rails): `@recorder.inverse_of :drop_table, [:musics, :artists], &block`
/// — block-pass `&block` was wrongly treated as an ambiguous block context.
/// `is_ambiguous_block_call` checked `call.block().is_some()` which matches
/// both `BlockNode` (literal `do/end` or `{}`) and `BlockArgumentNode` (`&block`).
/// RuboCop's `block_literal?` only matches literal blocks. Fixed by checking
/// `block.as_block_node().is_some()` to exclude `BlockArgumentNode`.
///
/// FN=1 (rufo): `%I( one  two #{ 1 } )` with `EnforcedStyle: brackets` —
/// the `brackets` style enforcement (flagging `%i/%I` arrays) was not implemented.
/// Added `check_percent_array` to detect percent symbol arrays and build the
/// bracketed representation message matching RuboCop's format.
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

/// RuboCop special global variables that don't need quoting as symbols.
const SPECIAL_GVARS: &[&str] = &[
    "$!", "$\"", "$$", "$&", "$'", "$*", "$+", "$,", "$/", "$;", "$:", "$.", "$<", "$=", "$>",
    "$?", "$@", "$\\", "$_", "$`", "$~", "$0", "$-0", "$-F", "$-I", "$-K", "$-W", "$-a", "$-d",
    "$-i", "$-l", "$-p", "$-v", "$-w",
];

/// RuboCop redefinable operators that don't need quoting as symbols.
const REDEFINABLE_OPERATORS: &[&str] = &[
    "|", "^", "&", "<=>", "==", "===", "=~", ">", ">=", "<", "<=", "<<", ">>", "+", "-", "*", "/",
    "%", "**", "~", "+@", "-@", "[]", "[]=", "`", "!", "!=", "!~",
];

/// Check if a symbol string can be represented without quotes (e.g., `:foo` not `:"foo"`).
fn symbol_without_quote(s: &str) -> bool {
    use regex::Regex;
    // method name
    static RE_METHOD: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"^[a-zA-Z_]\w*[!?]?$").unwrap());
    // instance / class variable
    static RE_IVAR: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"^@@?[a-zA-Z_]\w*$").unwrap());
    // global variable
    static RE_GVAR_NUM: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"^\$[1-9]\d*$").unwrap());
    static RE_GVAR_NAMED: std::sync::LazyLock<Regex> =
        std::sync::LazyLock::new(|| Regex::new(r"^\$[a-zA-Z_]\w*$").unwrap());

    RE_METHOD.is_match(s)
        || RE_IVAR.is_match(s)
        || RE_GVAR_NUM.is_match(s)
        || RE_GVAR_NAMED.is_match(s)
        || SPECIAL_GVARS.contains(&s)
        || REDEFINABLE_OPERATORS.contains(&s)
}

/// Convert a string to a symbol literal representation.
/// Returns `:foo` for simple symbols, `:"foo bar"` for complex ones.
fn to_symbol_literal(s: &str) -> String {
    if symbol_without_quote(s) {
        format!(":{s}")
    } else {
        format!(":\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Build the bracketed array representation from a percent symbol array.
/// Preserves whitespace between elements to match RuboCop's behavior.
fn build_bracketed_array(node: &ruby_prism::ArrayNode<'_>, source: &SourceFile) -> String {
    let elements = node.elements();
    if elements.is_empty() {
        return "[]".to_string();
    }

    let mut syms = Vec::new();
    for elem in elements.iter() {
        if let Some(sym) = elem.as_symbol_node() {
            let value = sym.unescaped();
            let content = std::str::from_utf8(value).unwrap_or("");
            syms.push(to_symbol_literal(content));
        } else if elem.as_interpolated_symbol_node().is_some() {
            // For interpolated symbols (from %I), reconstruct as :"..."
            let elem_source = source.byte_slice(
                elem.location().start_offset(),
                elem.location().end_offset(),
                "",
            );
            syms.push(format!(":\"{}\"", elem_source));
        } else {
            // Fallback: use source directly
            let elem_source = source.byte_slice(
                elem.location().start_offset(),
                elem.location().end_offset(),
                "",
            );
            syms.push(format!(":{elem_source}"));
        }
    }

    // Build with preserved whitespace between elements
    let mut result = String::from("[");

    // Leading whitespace: from opening delimiter end to first element start
    let opening_end = node.opening_loc().map(|o| o.end_offset()).unwrap_or(0);
    let first_start = elements
        .iter()
        .next()
        .map(|e| e.location().start_offset())
        .unwrap_or(opening_end);
    let leading = source.byte_slice(opening_end, first_start, "");
    result.push_str(leading);

    for (i, sym) in syms.iter().enumerate() {
        if i > 0 {
            // Whitespace between previous element end and current element start
            let prev_end = elements
                .iter()
                .nth(i - 1)
                .map(|e| e.location().end_offset())
                .unwrap_or(0);
            let curr_start = elements
                .iter()
                .nth(i)
                .map(|e| e.location().start_offset())
                .unwrap_or(0);
            let between = source.byte_slice(prev_end, curr_start, " ");
            result.push(',');
            result.push_str(between);
        }
        result.push_str(sym);
    }

    // Trailing whitespace: from last element end to closing delimiter start
    let last_end = elements
        .iter()
        .last()
        .map(|e| e.location().end_offset())
        .unwrap_or(0);
    let closing_start = node
        .closing_loc()
        .map(|c| c.start_offset())
        .unwrap_or(last_end);
    let trailing = source.byte_slice(last_end, closing_start, "");
    result.push_str(trailing);

    result.push(']');
    result
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

        let mut visitor = SymbolArrayVisitor {
            cop: self,
            source,
            parse_result,
            min_size,
            diagnostics: Vec::new(),
            in_ambiguous_block_context: false,
            enforced_style: enforced_style.to_string(),
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
    enforced_style: String,
}

impl<'pr> SymbolArrayVisitor<'_, '_, 'pr> {
    fn check_array(&mut self, node: &ruby_prism::ArrayNode<'pr>) {
        let opening = match node.opening_loc() {
            Some(loc) => loc,
            None => return,
        };

        let opening_slice = opening.as_slice();

        if opening_slice == b"[" {
            // Bracket array — check for percent style enforcement
            if self.enforced_style != "percent" {
                return;
            }
            self.check_bracket_array(node, &opening);
        } else if opening_slice.starts_with(b"%i") || opening_slice.starts_with(b"%I") {
            // Percent literal array — check for brackets style enforcement
            if self.enforced_style != "brackets" {
                return;
            }
            self.check_percent_array(node, &opening);
        }
    }

    /// Check a bracket symbol array when EnforcedStyle is "percent".
    fn check_bracket_array(
        &mut self,
        node: &ruby_prism::ArrayNode<'pr>,
        opening: &ruby_prism::Location<'pr>,
    ) {
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

    /// Check a percent literal symbol array (%i or %I) when EnforcedStyle is "brackets".
    fn check_percent_array(
        &mut self,
        node: &ruby_prism::ArrayNode<'pr>,
        opening: &ruby_prism::Location<'pr>,
    ) {
        let elements = node.elements();

        if elements.len() < self.min_size {
            return;
        }

        // All elements must be symbol or interpolated symbol nodes
        for elem in elements.iter() {
            if elem.as_symbol_node().is_none() && elem.as_interpolated_symbol_node().is_none() {
                return;
            }
        }

        // Build the bracket representation for the message
        let bracketed = build_bracketed_array(node, self.source);
        let prefer = if bracketed.contains('\n') {
            "an array literal `[...]`".to_string()
        } else {
            format!("`{bracketed}`")
        };
        let message = format!("Use {prefer} for an array of symbols.");

        let (line, column) = self.source.offset_to_line_col(opening.start_offset());
        self.diagnostics
            .push(self.cop.diagnostic(self.source, line, column, message));
    }

    /// Check if a call node represents an ambiguous block context:
    /// non-parenthesized method call with a literal block (do...end or {}).
    /// Block-pass (`&block`) does NOT create ambiguity — only literal blocks do.
    /// Matches RuboCop's `block_literal?` check.
    fn is_ambiguous_block_call(&self, call: &ruby_prism::CallNode<'pr>) -> bool {
        // Must have a literal block (BlockNode), not a block-pass (BlockArgumentNode)
        let has_literal_block = call.block().is_some_and(|b| b.as_block_node().is_some());
        if !has_literal_block {
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

    #[test]
    fn brackets_style_flags_percent_i_arrays() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([(
                "EnforcedStyle".into(),
                serde_yml::Value::String("brackets".into()),
            )]),
            ..CopConfig::default()
        };
        // %i array should be flagged with brackets style
        let source = b"%i[foo bar baz]\n";
        let diags = run_cop_full_with_config(&SymbolArray, source, config.clone());
        assert!(
            !diags.is_empty(),
            "Should flag %i array with brackets style"
        );
        assert!(
            diags[0].message.contains("[:foo, :bar, :baz]"),
            "Message should suggest bracket equivalent, got: {}",
            diags[0].message
        );
    }

    #[test]
    fn brackets_style_flags_percent_i_uppercase_with_interpolation() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([(
                "EnforcedStyle".into(),
                serde_yml::Value::String("brackets".into()),
            )]),
            ..CopConfig::default()
        };
        // %I array with interpolation should be flagged with brackets style
        let source = "%I( one  two  #{1} )\n".as_bytes();
        let diags = run_cop_full_with_config(&SymbolArray, source, config);
        assert!(
            !diags.is_empty(),
            "Should flag %I array with brackets style"
        );
        // Message should contain the bracket representation
        assert!(
            diags[0].message.contains("for an array of symbols"),
            "Message should mention array of symbols, got: {}",
            diags[0].message
        );
    }

    #[test]
    fn brackets_style_respects_min_size() {
        use crate::testutil::run_cop_full_with_config;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([
                (
                    "EnforcedStyle".into(),
                    serde_yml::Value::String("brackets".into()),
                ),
                ("MinSize".into(), serde_yml::Value::Number(5.into())),
            ]),
            ..CopConfig::default()
        };
        // 3-element %i array should not trigger with MinSize:5
        let source = b"%i[foo bar baz]\n";
        let diags = run_cop_full_with_config(&SymbolArray, source, config);
        assert!(diags.is_empty(), "Should not flag %i array below MinSize");
    }

    #[test]
    fn block_pass_not_ambiguous() {
        use crate::testutil::run_cop_full;
        // &block is a BlockArgumentNode, not a BlockNode — should not suppress the offense
        let source = b"foo :a, [:b, :c], &blk\n";
        let diags = run_cop_full(&SymbolArray, source);
        assert!(
            !diags.is_empty(),
            "Block-pass should not suppress symbol array offense"
        );
    }
}
