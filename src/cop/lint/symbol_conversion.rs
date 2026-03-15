use crate::cop::node_type::{CALL_NODE, SYMBOL_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Lint/SymbolConversion checks for uses of literal strings converted to a symbol
/// where a literal symbol could be used instead, and for unnecessarily quoted symbols.
///
/// ## Corpus investigation (2026-03-10)
///
/// Corpus oracle reported FP=331, FN=279 on the March 10, 2026 run.
///
/// The previous regex-based correction logic only understood identifier-like
/// symbols. That caused:
/// - FPs on quoted hash labels that still require quotes, such as `"7_days":`
/// - FNs on quoted operator and variable symbols such as `:"+", :"@ivar"`
///
/// Fix: generate corrections using Ruby-like symbol literal rules instead of
/// assuming every convertible symbol is an identifier. Hash labels now only
/// autocorrect when the key can be written as a bare Ruby label.
///
/// Remaining gap: `EnforcedStyle: consistent` is still not implemented in this
/// port; the corpus regressions fixed here are all strict-style behavior.
///
/// ## Corpus investigation (2026-03-11)
///
/// Corpus oracle reported FP=37, FN=100. Root causes:
///
/// FPs: Rocket-style hash keys with non-identifier-start values (e.g.,
/// `{ :'@ivar' => val }`) were flagged as standalone symbols. RuboCop's
/// `correct_hash_key` skips keys whose value doesn't match `/\A[a-z0-9_]/i`.
/// Fix: detect rocket-style hash keys by looking for `=>` after the symbol
/// location and skip non-identifier-start values.
///
/// FNs: (1) Missing `!=` from `BARE_OPERATOR_SYMBOLS` — symbols like `:"!="`
/// were not flagged. (2) Special global variables (`$1`, `$?`, `$!`, `$~`,
/// `$0`, etc.) were not recognized as valid bare symbols because
/// `is_global_variable_symbol` only handled named globals starting with
/// identifier chars. Fix: expanded to handle numeric globals, single-char
/// special globals, and `$-x` flags.
///
/// Also fixed `normalize_single_quoted_source` which over-escaped backslashes
/// when converting `:'...'` to `:"..."` form (used `escape_double_quoted_symbol`
/// on raw source chars instead of just escaping `"`).
///
/// ## Corpus investigation (2026-03-14)
///
/// Corpus oracle reported FP=14, FN=94 (98.6% match rate).
///
/// Extensive analysis performed comparing RuboCop's `on_sym`/`on_send` logic
/// against nitrocop's `check_symbol_node`/`check_call_node`:
///
/// - Verified: `.to_sym`/`.intern` on single/double-quoted strings, operator
///   symbols, instance/class/global variables, method-name symbols with `?`/`!`/`=`
///   suffixes, colon-style and rocket-style hash keys, `%i`/`%I` arrays, `alias`
///   statements, setter exemptions, escape sequence handling, and
///   `source_matches_correction` normalization all match RuboCop behavior.
///
/// - Added comprehensive test coverage for all verified patterns.
///
/// - Two potential FP sources identified but unconfirmed (no corpus data available):
///   (1) `alias :"foo" :"bar"` — RuboCop has `in_alias?` check to skip alias
///   arguments, nitrocop relies on Prism's `opening_loc` filtering which works
///   for bare `alias foo bar` but may not for explicitly quoted alias arguments.
///   (2) Multi-line rocket-style hash keys where `=>` is on next line — nitrocop's
///   byte scanning only skips spaces/tabs, not newlines, so may miss the `=>`
///   and treat the key as standalone.
///
/// - `EnforcedStyle: consistent` is still not implemented. The corpus baseline
///   uses `strict` (default) so this should not affect corpus FP/FN. However,
///   individual project configs that set `consistent` would have FNs.
///
/// Root cause of remaining 94 FNs is not definitively identified; may require
/// corpus-level debugging with `investigate-cop.py --context` once corpus data
/// with example locations is available.
///
/// ## FP fix (2026-03-15)
///
/// Corpus oracle reported FP=6, FN=94. 6 FPs traced to escape sequences in
/// quoted symbols where `escape_double_quoted_symbol` didn't reproduce the
/// necessary escaping:
/// - Non-printable characters (e.g., `\x00`, `\a`, `\b`, `\e`) were passed
///   through as raw bytes, producing a correction that didn't match the source.
/// - `#` before `{`, `$`, `@` was not escaped, so `:"#{c}"` (with escaped
///   interpolation) would produce `:"#{c}"` (with live interpolation) as
///   the correction, changing semantics.
///
/// Fix: enhanced `escape_double_quoted_symbol` to handle all control characters
/// (`\a`, `\b`, `\e`, `\xNN` for others), high bytes (UTF-8 pass-through),
/// and `#` before interpolation triggers (`\#`). This ensures
/// `source_matches_correction` correctly identifies these symbols as already
/// using the canonical quoting form.
///
/// ## FP fix (2026-03-14)
///
/// Corpus oracle reported FP=14, FN=94. 14 FPs in jruby, natalie, BetterErrors,
/// hexapdf traced to `alias :'method' other` patterns. RuboCop has an `in_alias?`
/// check (`node.parent&.alias_type?`) that skips all symbols inside alias
/// statements. Since nitrocop's cop framework doesn't provide parent node info,
/// implemented `is_in_alias()` which scans source bytes backwards from the
/// symbol position, skipping whitespace and possibly a preceding symbol argument,
/// to detect the `alias` keyword. This matches RuboCop's behavior: alias
/// arguments are not flagged because a symbol requiring quoting is not a valid
/// method identifier.
pub struct SymbolConversion;

const BARE_OPERATOR_SYMBOLS: &[&[u8]] = &[
    b"+", b"-", b"*", b"/", b"%", b"&", b"|", b"^", b"<<", b">>", b"<", b">", b"<=", b">=", b"==",
    b"!=", b"===", b"<=>", b"=~", b"!~", b"!", b"~", b"+@", b"-@", b"**", b"[]", b"[]=", b"`",
];

fn is_identifier_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_identifier_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_method_name_symbol(value: &[u8]) -> bool {
    if value.is_empty() || !is_identifier_start(value[0]) {
        return false;
    }

    let main = match value.last() {
        Some(b'!' | b'?' | b'=') => &value[..value.len() - 1],
        _ => value,
    };

    !main.is_empty() && main.iter().copied().all(is_identifier_continue)
}

fn is_hash_label_symbol(value: &[u8]) -> bool {
    if value.is_empty() || !is_identifier_start(value[0]) {
        return false;
    }

    let main = if let Some(&last) = value.last() {
        if last == b'!' || last == b'?' {
            &value[..value.len() - 1]
        } else {
            value
        }
    } else {
        return false;
    };

    !main.is_empty() && main.iter().copied().all(is_identifier_continue)
}

fn is_instance_variable_symbol(value: &[u8]) -> bool {
    value.len() > 1
        && value[0] == b'@'
        && is_identifier_start(value[1])
        && value[2..].iter().copied().all(is_identifier_continue)
}

fn is_class_variable_symbol(value: &[u8]) -> bool {
    value.len() > 2
        && value.starts_with(b"@@")
        && is_identifier_start(value[2])
        && value[3..].iter().copied().all(is_identifier_continue)
}

/// Characters recognized by Ruby as single-char special global variables
/// (e.g., `$?`, `$!`, `$~`, `$@`, `$;`, `$,`, `$/`, `$\`, `$=`, `$<`, `$>`,
/// `$.`, `$*`, `$:`, `$+`, `$&`, `` $` ``, `$'`, `$"`, `$0`).
const SPECIAL_GLOBAL_CHARS: &[u8] = b"?!~@;,/\\=<>.*:+&`'\"0";

fn is_global_variable_symbol(value: &[u8]) -> bool {
    if value.len() < 2 || value[0] != b'$' {
        return false;
    }

    // Named globals: $foo, $LOAD_PATH, $_
    if is_identifier_start(value[1]) {
        return value[2..].iter().copied().all(is_identifier_continue);
    }

    // Numeric globals: $1, $2, ..., $9 (and possibly multi-digit like $10)
    if value[1].is_ascii_digit() {
        return value[2..].iter().copied().all(|b| b.is_ascii_digit());
    }

    // Single-char special globals: $?, $!, $~, etc.
    if value.len() == 2 && SPECIAL_GLOBAL_CHARS.contains(&value[1]) {
        return true;
    }

    // $-x flags (e.g., $-w, $-v, $-a)
    if value.len() == 3 && value[1] == b'-' && value[2].is_ascii_alphabetic() {
        return true;
    }

    false
}

fn is_operator_symbol(value: &[u8]) -> bool {
    BARE_OPERATOR_SYMBOLS.contains(&value)
}

fn can_be_unquoted_symbol(value: &[u8]) -> bool {
    is_method_name_symbol(value)
        || is_instance_variable_symbol(value)
        || is_class_variable_symbol(value)
        || is_global_variable_symbol(value)
        || is_operator_symbol(value)
}

fn escape_double_quoted_symbol(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'\\' => escaped.push_str("\\\\"),
            b'"' => escaped.push_str("\\\""),
            b'\n' => escaped.push_str("\\n"),
            b'\r' => escaped.push_str("\\r"),
            b'\t' => escaped.push_str("\\t"),
            0x0C => escaped.push_str("\\f"),
            0x07 => escaped.push_str("\\a"),
            0x08 => escaped.push_str("\\b"),
            0x1B => escaped.push_str("\\e"),
            // Escape # before {, $, @ to prevent interpolation
            b'#' if i + 1 < bytes.len()
                && (bytes[i + 1] == b'{' || bytes[i + 1] == b'$' || bytes[i + 1] == b'@') =>
            {
                escaped.push_str("\\#");
            }
            // Non-printable characters: control chars (0x00-0x1F, 0x7F) and high bytes
            _ if b < 0x20 || b == 0x7F => {
                escaped.push_str(&format!("\\x{b:02X}"));
            }
            _ if b > 0x7F => {
                // For multi-byte UTF-8, push the character as-is
                if let Some(ch) = value[i..].chars().next() {
                    escaped.push(ch);
                    i += ch.len_utf8();
                    continue;
                }
                // Fallback: hex escape the byte
                escaped.push_str(&format!("\\x{b:02X}"));
            }
            _ => escaped.push(b as char),
        }
        i += 1;
    }
    escaped
}

/// Compute the correction string for a symbol value.
/// Returns Ruby-like symbol literal syntax, e.g. `:+`, `:@ivar`, or `:"foo-bar"`.
fn symbol_correction(value: &[u8]) -> Option<String> {
    let value_str = std::str::from_utf8(value).ok()?;
    if can_be_unquoted_symbol(value) {
        Some(format!(":{value_str}"))
    } else {
        Some(format!(":\"{}\"", escape_double_quoted_symbol(value_str)))
    }
}

fn hash_key_correction(value: &[u8]) -> Option<String> {
    if !is_hash_label_symbol(value) {
        return None;
    }

    Some(std::str::from_utf8(value).ok()?.to_string())
}

/// Escape only double-quote characters in raw source content when converting
/// from single-quoted to double-quoted form. Unlike `escape_double_quoted_symbol`
/// (which escapes unescaped values), this works on raw source text where
/// backslashes are already in their source-level escaped form.
/// Matches RuboCop's `source.gsub('"', '\"').tr("'", '"')` behavior.
fn escape_quotes_for_normalization(inner: &str) -> String {
    let mut result = String::with_capacity(inner.len());
    for ch in inner.chars() {
        if ch == '"' {
            result.push('\\');
            result.push('"');
        } else {
            result.push(ch);
        }
    }
    result
}

fn normalize_single_quoted_source(source: &str) -> Option<String> {
    if source.starts_with(":'") && source.ends_with('\'') {
        let inner = source.strip_prefix(":'")?.strip_suffix('\'')?;
        return Some(format!(":\"{}\"", escape_quotes_for_normalization(inner)));
    }

    if source.starts_with('\'') && source.ends_with('\'') {
        let inner = source.strip_prefix('\'')?.strip_suffix('\'')?;
        return Some(format!("\"{}\"", escape_quotes_for_normalization(inner)));
    }

    None
}

/// Check if the value starts with an alphanumeric or underscore character.
/// Matches RuboCop's `/\A[a-z0-9_]/i` check in `correct_hash_key`.
fn value_starts_with_identifier(value: &[u8]) -> bool {
    value
        .first()
        .is_some_and(|&b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Check if a symbol node is an argument to the `alias` keyword.
/// RuboCop skips alias arguments because a symbol requiring quoting is not a
/// valid method identifier, so flagging it would be unhelpful.
///
/// Scans backwards from the symbol's start offset, skipping whitespace and
/// possibly one preceding quoted/bare symbol (the first alias argument),
/// to see if the `alias` keyword precedes it.
fn is_in_alias(source: &SourceFile, sym: &ruby_prism::SymbolNode<'_>) -> bool {
    let start = sym.location().start_offset();
    let src = source.as_bytes();
    if start == 0 {
        return false;
    }

    // Skip backwards over whitespace
    let mut pos = start;
    while pos > 0 && (src[pos - 1] == b' ' || src[pos - 1] == b'\t') {
        pos -= 1;
    }

    // Check if preceded directly by `alias` (this is the first alias argument)
    if pos >= 5 && &src[pos - 5..pos] == b"alias" {
        // Make sure `alias` is at start of token (not part of a longer identifier)
        return pos == 5 || !is_identifier_continue(src[pos - 6]);
    }

    // Maybe this is the second alias argument — skip over the first argument
    // First argument can be: bare symbol (identifier chars), or quoted symbol (:'...' or :"...")
    if pos > 0 {
        let end_char = src[pos - 1];
        if end_char == b'\'' || end_char == b'"' {
            // Quoted symbol: scan back to find matching :'  or :"
            let quote = end_char;
            if pos < 3 {
                return false;
            }
            // Find the opening :' or :"
            let mut p = pos - 2; // skip closing quote
            while p > 0 && src[p] != quote {
                p -= 1;
            }
            // p should now be at the opening quote, preceded by ':'
            if p > 0 && src[p] == quote && src[p - 1] == b':' {
                pos = p - 1; // move before ':'
            } else {
                return false;
            }
        } else if is_identifier_continue(end_char) {
            // Bare symbol (identifier): scan back over identifier chars
            let mut p = pos - 1;
            while p > 0 && is_identifier_continue(src[p - 1]) {
                p -= 1;
            }
            pos = p;
        } else {
            return false;
        }

        // Skip whitespace again
        while pos > 0 && (src[pos - 1] == b' ' || src[pos - 1] == b'\t') {
            pos -= 1;
        }

        // Now check for `alias`
        if pos >= 5 && &src[pos - 5..pos] == b"alias" {
            return pos == 5 || !is_identifier_continue(src[pos - 6]);
        }
    }

    false
}

/// Check if a symbol node is used as a rocket-style hash key (e.g., `:'foo' => val`).
/// Looks at the source bytes after the symbol to see if `=>` follows (after whitespace).
fn is_rocket_hash_key(source: &SourceFile, sym: &ruby_prism::SymbolNode<'_>) -> bool {
    let end_offset = sym.location().end_offset();
    let src = source.as_bytes();
    if end_offset >= src.len() {
        return false;
    }
    let rest = &src[end_offset..];
    // Skip whitespace, then check for =>
    let trimmed = rest.iter().position(|&b| b != b' ' && b != b'\t');
    match trimmed {
        Some(pos) => rest[pos..].starts_with(b"=>"),
        None => false,
    }
}

fn source_matches_correction(source: &[u8], correction: &str) -> bool {
    let Ok(source_str) = std::str::from_utf8(source) else {
        return false;
    };

    source_str == correction
        || normalize_single_quoted_source(source_str)
            .as_deref()
            .is_some_and(|normalized| normalized == correction)
}

impl Cop for SymbolConversion {
    fn name(&self) -> &'static str {
        "Lint/SymbolConversion"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE, SYMBOL_NODE]
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
        let _style = config.get_str("EnforcedStyle", "strict");

        // Check SymbolNode: quoted symbols (standalone or hash keys)
        if let Some(sym) = node.as_symbol_node() {
            self.check_symbol_node(source, &sym, diagnostics);
            return;
        }

        // Check CallNode: .to_sym / .intern patterns
        if let Some(call) = node.as_call_node() {
            self.check_call_node(source, &call, diagnostics);
        }
    }
}

impl SymbolConversion {
    /// Check a SymbolNode for unnecessary quoting.
    /// Handles:
    /// - Standalone quoted symbols: `:"foo"`, `:'bar'`
    /// - Hash keys (colon-style): `'foo': val`, `"foo": val`
    /// - Hash keys/values (rocket-style): `:'foo' => val`, `{ foo: :'bar' }`
    fn check_symbol_node(
        &self,
        source: &SourceFile,
        sym: &ruby_prism::SymbolNode<'_>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let src = sym.location().as_slice();
        let value = sym.unescaped();

        // Determine what kind of symbol this is based on source representation
        let opening = sym.opening_loc().map(|l| l.as_slice());

        // Check if this is a hash key in colon-style: 'foo': or "foo":
        // In Prism, these have opening_loc of "'" or "\"" and source ends with ':
        let is_colon_hash_key = match opening {
            Some(b"'" | b"\"") => src.ends_with(b"':") || src.ends_with(b"\":"),
            _ => false,
        };

        if is_colon_hash_key {
            // Hash key with colon style: 'foo': val or "foo": val
            // Only flag if the value can be a bare hash key
            if let Some(value_str) = hash_key_correction(value) {
                let loc = sym.location();
                let (line, column) = source.offset_to_line_col(loc.start_offset());
                diagnostics.push(self.diagnostic(
                    source,
                    line,
                    column,
                    format!("Unnecessary symbol conversion; use `{value_str}:` instead."),
                ));
            }
            return;
        }

        // For standalone symbols or rocket-style hash keys/values:
        // Check if the symbol is unnecessarily quoted
        // Opening must be :" or :' (quoted symbol syntax)
        match opening {
            Some(b":\"" | b":'") => {}
            _ => return,
        }

        // Skip symbols that are arguments to `alias` — a symbol requiring
        // quoting is not a valid method identifier, so flagging it is unhelpful.
        // Matches RuboCop's `in_alias?` check.
        if is_in_alias(source, sym) {
            return;
        }

        // Check if this is a rocket-style hash key (:'foo' => val).
        // RuboCop's correct_hash_key skips keys whose value doesn't start with
        // /\A[a-z0-9_]/i, so we must do the same to avoid false positives.
        if is_rocket_hash_key(source, sym) && !value_starts_with_identifier(value) {
            return;
        }

        let correction = match symbol_correction(value) {
            Some(c) => c,
            None => return,
        };

        // RuboCop leaves quoted setter-like symbols alone in strict mode.
        if correction.ends_with('=') {
            return;
        }

        if source_matches_correction(src, &correction) {
            return;
        }

        let loc = sym.location();
        let (line, column) = source.offset_to_line_col(loc.start_offset());
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            format!("Unnecessary symbol conversion; use `{correction}` instead."),
        ));
    }

    /// Check a CallNode for .to_sym / .intern on string/symbol/dstr receivers.
    fn check_call_node(
        &self,
        source: &SourceFile,
        call: &ruby_prism::CallNode<'_>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let method_name = call.name().as_slice();

        // Must be .to_sym or .intern
        if method_name != b"to_sym" && method_name != b"intern" {
            return;
        }

        // Must have no arguments
        if call.arguments().is_some() {
            return;
        }

        let recv = match call.receiver() {
            Some(r) => r,
            None => return,
        };

        // String receiver: "foo".to_sym
        if let Some(str_node) = recv.as_string_node() {
            let value = str_node.unescaped();
            let correction = match symbol_correction(value) {
                Some(c) => c,
                None => return,
            };
            let loc = call.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Unnecessary symbol conversion; use `{correction}` instead."),
            ));
            return;
        }

        // Symbol receiver: :foo.to_sym
        if let Some(sym_node) = recv.as_symbol_node() {
            let value = sym_node.unescaped();
            let correction = match symbol_correction(value) {
                Some(c) => c,
                None => return,
            };
            let loc = call.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Unnecessary symbol conversion; use `{correction}` instead."),
            ));
            return;
        }

        // Interpolated string receiver: "foo-#{bar}".to_sym
        if let Some(dstr) = recv.as_interpolated_string_node() {
            // Reconstruct the interpolated content from source
            let dstr_loc = dstr.location();
            let dstr_src = dstr_loc.as_slice();
            // Strip the surrounding quotes from the dstr source
            let inner = if dstr_src.starts_with(b"\"") && dstr_src.ends_with(b"\"") {
                &dstr_src[1..dstr_src.len() - 1]
            } else {
                return; // heredoc or other form, skip
            };
            let inner_str = match std::str::from_utf8(inner) {
                Ok(s) => s,
                Err(_) => return,
            };
            let correction = format!(":\"{}\"", inner_str);
            let loc = call.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Unnecessary symbol conversion; use `{correction}` instead."),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(SymbolConversion, "cops/lint/symbol_conversion");

    #[test]
    fn rocket_style_hash_keys_with_non_identifier_start() {
        let cop = SymbolConversion;
        // RuboCop skips rocket-style hash keys where value doesn't start
        // with /[a-z0-9_]/i — these should NOT be flagged.
        let no_offense_cases = [
            r#"{ :'@ivar' => 1 }"#,
            r#"{ :"@ivar" => 1 }"#,
            r#"{ :'$global' => 1 }"#,
            r#"{ :'+' => 1 }"#,
            r#"{ :'==' => 1 }"#,
            r#"{ :'@@cvar' => 1 }"#,
        ];
        for source in &no_offense_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert!(
                diags.is_empty(),
                "Expected no offense for {:?} but got: {:?}",
                source,
                diags.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }

        // But standalone versions of these symbols SHOULD be flagged
        let offense_cases = [r#":'@ivar'"#, r#":"@ivar""#, r#":'$global'"#, r#":'+'"#];
        for source in &offense_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert!(
                !diags.is_empty(),
                "Expected offense for {:?} but got none",
                source
            );
        }

        // Rocket-style with identifier-start values SHOULD still be flagged
        let flagged_rocket_cases = [r#"{ :'foo' => 1 }"#, r#"{ :"foo" => 1 }"#];
        for source in &flagged_rocket_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert!(
                !diags.is_empty(),
                "Expected offense for {:?} but got none",
                source
            );
        }
    }

    #[test]
    fn single_quoted_string_to_sym() {
        let cop = SymbolConversion;
        // Single-quoted strings with .to_sym should be flagged
        let offense_cases = [
            ("'foo'.to_sym", ":foo"),
            ("'foo_bar'.to_sym", ":foo_bar"),
            ("'foo-bar'.to_sym", ":\"foo-bar\""),
            ("'foo'.intern", ":foo"),
        ];
        for (source, expected) in &offense_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert!(
                !diags.is_empty(),
                "Expected offense for {:?} but got none",
                source
            );
            assert!(
                diags[0].message.contains(&format!("`{expected}`")),
                "Expected correction {} in message for {:?}, got: {}",
                expected,
                source,
                diags[0].message
            );
        }
    }

    #[test]
    fn to_sym_with_empty_parens() {
        let cop = SymbolConversion;
        // .to_sym() with empty parens should still be flagged
        let offense_cases = [
            (r#""foo".to_sym()"#, ":foo"),
            (r#"'bar'.intern()"#, ":bar"),
            (":baz.to_sym()", ":baz"),
        ];
        for (source, expected) in &offense_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert!(
                !diags.is_empty(),
                "Expected offense for {:?} but got none",
                source
            );
            assert!(
                diags[0].message.contains(&format!("`{expected}`")),
                "Expected correction {} in message for {:?}, got: {}",
                expected,
                source,
                diags[0].message
            );
        }
    }

    #[test]
    fn comprehensive_offense_check() {
        let cop = SymbolConversion;
        // Comprehensive list of patterns that should be offenses
        let offense_cases: Vec<(&str, &str)> = vec![
            // to_sym on string literals (single and double quoted)
            (r#""foo".to_sym"#, ":foo"),
            ("'foo'.to_sym", ":foo"),
            // intern
            (r#""foo".intern"#, ":foo"),
            ("'foo'.intern", ":foo"),
            // to_sym on symbol literal (redundant)
            (":foo.to_sym", ":foo"),
            (":foo.intern", ":foo"),
            // Quoted standalone symbols
            (":\"foo\"", ":foo"),
            (":'foo'", ":foo"),
            (":\"foo_bar\"", ":foo_bar"),
            (":'foo_bar'", ":foo_bar"),
            // Operator symbols that can be unquoted (excluding those ending with =)
            (":\"<<\"", ":<<"),
            (":'<<'", ":<<"),
            (":\"[]\"", ":[]"),
            (":'[]'", ":[]"),
            (":\"+\"", ":+"),
            (":'+'", ":+"),
            (":\"-\"", ":-"),
            (":'-'", ":-"),
            (":\"**\"", ":**"),
            (":'**'", ":**"),
            // Instance/class variable symbols
            (":\"@foo\"", ":@foo"),
            (":'@foo'", ":@foo"),
            (":\"@@foo\"", ":@@foo"),
            (":'@@foo'", ":@@foo"),
            // Global variable symbols
            (":\"$foo\"", ":$foo"),
            (":'$foo'", ":$foo"),
            // Colon-style hash keys
            ("{ 'foo': 1 }", "foo:"),
            ("{ \"foo\": 1 }", "foo:"),
            ("{ 'foo_bar': 1 }", "foo_bar:"),
            // Rocket-style hash keys with identifier start
            ("{ :'foo' => 1 }", ":foo"),
            ("{ :\"foo\" => 1 }", ":foo"),
            // Method-like symbols
            (":\"foo?\"", ":foo?"),
            (":'foo?'", ":foo?"),
            (":\"foo!\"", ":foo!"),
            (":'foo!'", ":foo!"),
            // Hash keys ending with ? or !
            ("{ 'foo?': 1 }", "foo?:"),
            ("{ 'foo!': 1 }", "foo!:"),
        ];
        for (source, expected_contains) in &offense_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert!(
                !diags.is_empty(),
                "Expected offense for {:?} but got none",
                source
            );
            assert!(
                diags[0].message.contains(&format!("`{expected_contains}`")),
                "Expected `{}` in message for {:?}, got: {}",
                expected_contains,
                source,
                diags[0].message
            );
        }
    }

    #[test]
    fn symbol_in_various_contexts() {
        let cop = SymbolConversion;
        // Symbols in various contexts should all be flagged
        let offense_cases = [
            // In array
            (r#"[:"foo", :"bar"]"#, 2),
            // In method arguments
            (r#"method(:"foo")"#, 1),
            // In assignment
            (r#"x = :"foo""#, 1),
            // In conditional
            (r#"if x == :"foo" then 1 end"#, 1),
            // In case/when
            (r#"case x; when :"foo" then 1; end"#, 1),
            // Nested in hash value
            (r#"{ key: :"foo" }"#, 1),
            // Multiple in same line
            (r#"[:"foo", :'bar', "baz".to_sym]"#, 3),
            // In string interpolation context
            (r#"send(:"foo")"#, 1),
            // Method receiver
            (r#":"foo".to_s"#, 1),
        ];
        for (source, expected_count) in &offense_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert_eq!(
                diags.len(),
                *expected_count,
                "Expected {} offense(s) for {:?} but got {}: {:?}",
                expected_count,
                source,
                diags.len(),
                diags.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn to_sym_with_special_strings() {
        let cop = SymbolConversion;
        // Various string forms with .to_sym
        let offense_cases = [
            // Method-like strings
            ("'foo?'.to_sym", ":foo?"),
            ("'foo!'.to_sym", ":foo!"),
            ("'foo='.to_sym", ":foo="),
            // Operator strings
            ("'+'.to_sym", ":+"),
            ("'<<'.to_sym", ":<<"),
            ("'[]'.to_sym", ":[]"),
            ("'[]='.to_sym", ":[]="),
            ("'<=>'.to_sym", ":<=>"),
            // Instance variable strings
            ("'@foo'.to_sym", ":@foo"),
            ("'@@foo'.to_sym", ":@@foo"),
            // Global variable strings
            ("'$foo'.to_sym", ":$foo"),
            // Strings that need quoting as symbols
            ("'foo-bar'.to_sym", ":\"foo-bar\""),
            ("'foo bar'.to_sym", ":\"foo bar\""),
        ];
        for (source, expected) in &offense_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert!(
                !diags.is_empty(),
                "Expected offense for {:?} but got none",
                source
            );
            assert!(
                diags[0].message.contains(&format!("`{expected}`")),
                "Expected `{}` in message for {:?}, got: {}",
                expected,
                source,
                diags[0].message
            );
        }
    }

    #[test]
    fn double_quoted_hash_key_with_suffix() {
        let cop = SymbolConversion;
        // Double-quoted hash keys with ? and ! suffix
        let offense_cases = [
            ("{ \"foo?\": 1 }", "foo?:"),
            ("{ \"foo!\": 1 }", "foo!:"),
            ("{ \"Foo\": 1 }", "Foo:"),
            ("{ \"_foo\": 1 }", "_foo:"),
        ];
        for (source, expected) in &offense_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert!(
                !diags.is_empty(),
                "Expected offense for {:?} but got none",
                source
            );
            assert!(
                diags[0].message.contains(&format!("`{expected}`")),
                "Expected `{}` in message for {:?}, got: {}",
                expected,
                source,
                diags[0].message
            );
        }
    }

    #[test]
    fn comprehensive_no_offense_check() {
        let cop = SymbolConversion;
        let no_offense_cases: Vec<&str> = vec![
            // Bare symbols
            ":foo",
            ":foo_bar",
            ":Foo",
            // Symbols that need quoting
            ":\"foo-bar\"",
            ":'foo-bar'",
            ":\"foo bar\"",
            ":'foo bar'",
            ":\"foo:bar\"",
            // Setter symbols (properly quoted)
            ":\"foo=\"",
            ":'foo='",
            // Bare hash keys
            "{ foo: 1 }",
            // Hash keys that need quoting
            "{ 'foo-bar': 1 }",
            "{ 'foo bar': 1 }",
            "{ '==': 1 }",
            "{ 'foo=': 1 }",
            // Empty symbol
            ":\"\"",
            // Percent arrays
            "%i(foo bar)",
            "%I(foo bar)",
            // Alias
            "alias foo bar",
            // to_sym on non-literal
            "name.to_sym",
            "x.to_sym",
            // to_sym with args
            "\"foo\".to_sym(1)",
            // Escape-needed symbols
            ":\"\\n\"",
            ":\"\\t\"",
            // Rocket-style with non-identifier start
            "{ :'@ivar' => 1 }",
            "{ :'+' => 1 }",
            // Numeric-start hash key (needs quotes)
            "{ '7_days': 1 }",
            "{ \"7_days\": 1 }",
            // Alias arguments — RuboCop skips these (in_alias? check)
            "alias :'foo' bar",
            "alias :\"foo\" bar",
            "alias bar :'foo'",
            "alias bar :\"foo\"",
            "alias :'foo' :'bar'",
            "alias :\"foo\" :\"bar\"",
        ];
        for source in &no_offense_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert!(
                diags.is_empty(),
                "Expected no offense for {:?} but got: {:?}",
                source,
                diags.iter().map(|d| &d.message).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn special_global_variable_symbols() {
        let cop = SymbolConversion;
        // These should be flagged — they can be unquoted
        let offense_cases = [
            (r#":"$1""#, ":$1"),
            (r#":"$?""#, ":$?"),
            (r#":"$!""#, ":$!"),
            (r#":"$0""#, ":$0"),
            (r#":"$~""#, ":$~"),
        ];
        for (source, expected_correction) in &offense_cases {
            let diags = crate::testutil::run_cop_full(&cop, source.as_bytes());
            assert!(
                !diags.is_empty(),
                "Expected offense for {:?} but got none",
                source
            );
            assert!(
                diags[0]
                    .message
                    .contains(&format!("`{expected_correction}`")),
                "Expected correction {} in message for {:?}, got: {}",
                expected_correction,
                source,
                diags[0].message
            );
        }
    }
}
