use crate::cop::node_type::{CALL_NODE, REGULAR_EXPRESSION_NODE, STRING_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// Performance/StringReplacement
///
/// Identifies places where `gsub`/`gsub!` can be replaced by `tr`/`delete`.
///
/// Investigation notes:
/// - Original implementation only handled `gsub` (not `gsub!`) and only single-byte chars.
/// - Root cause of 1054 FNs: byte length check (`len() != 1`) rejected multi-byte UTF-8
///   single characters (e.g., "Á" is 2 bytes but 1 char). Also missed empty replacement
///   pattern (→ `delete`) and `gsub!` (→ `tr!`/`delete!`).
/// - RuboCop only flags `gsub`/`gsub!`, NOT `sub`/`sub!`.
/// - Message format: "Use `tr` instead of `gsub`." / "Use `delete` instead of `gsub`."
///   with bang variants for `gsub!`.
/// - 338 FNs from regex first-arg patterns: `gsub(/\n/, '')`, `gsub(/ /, '-')`, etc.
///   RuboCop accepts single-character deterministic regex literals (matching LITERAL_REGEX).
///   After `interpret_string_escapes`, the regex content must be exactly 1 char.
pub struct StringReplacement;

/// Check if a regex pattern (raw bytes between `/`...`/`) represents a single
/// deterministic literal character, matching RuboCop's `DETERMINISTIC_REGEX`
/// filtered to length == 1 after `interpret_string_escapes`.
///
/// Returns `true` for: `/a/`, `/ /`, `/\n/`, `/\t/`, `/\\/`, `/\./`, `/\y/`, etc.
/// Returns `false` for: `/\d/`, `/[abc]/`, `/a*/`, `/a|b/`, `//`, `/ab/`, etc.
fn is_single_char_deterministic_regex(content: &[u8]) -> bool {
    if content.is_empty() {
        return false;
    }
    if content[0] == b'\\' {
        // Escaped sequence: must be exactly 2 bytes and not a regex metachar class
        if content.len() != 2 {
            return false;
        }
        let next = content[1];
        // Regex metachar classes that are NOT literal: \A, \b, \B, \d, \D, \g, \G,
        // \h, \H, \k, \p, \P, \R, \w, \W, \X, \s, \S, \z, \Z, \0-\9
        !is_regex_escape_metachar(next)
    } else {
        // Unescaped single char: must be 1 byte and a literal char
        content.len() == 1 && is_literal_char(content[0])
    }
}

/// Characters in RuboCop's literal allowlist: `[\w\s\-,"'!#%&<>=;:`~/]`
fn is_literal_char(b: u8) -> bool {
    matches!(
        b,
        b'a'..=b'z'
            | b'A'..=b'Z'
            | b'0'..=b'9'
            | b'_'
            | b' '
            | b'\t'
            | b'\n'
            | b'\r'
            | 0x0C
            | b'-'
            | b','
            | b'"'
            | b'\''
            | b'!'
            | b'#'
            | b'%'
            | b'&'
            | b'<'
            | b'>'
            | b'='
            | b';'
            | b':'
            | b'`'
            | b'~'
            | b'/'
    )
}

/// Regex metachar classes: `\d`, `\s`, `\w`, `\A`, `\b`, `\B`, etc.
/// and digit escapes `\0`-`\9`. These are NOT literal when escaped.
fn is_regex_escape_metachar(b: u8) -> bool {
    matches!(
        b,
        b'A' | b'b'
            | b'B'
            | b'd'
            | b'D'
            | b'g'
            | b'G'
            | b'h'
            | b'H'
            | b'k'
            | b'p'
            | b'P'
            | b'R'
            | b'w'
            | b'W'
            | b'X'
            | b's'
            | b'S'
            | b'z'
            | b'Z'
            | b'0'..=b'9'
    )
}

impl Cop for StringReplacement {
    fn name(&self) -> &'static str {
        "Performance/StringReplacement"
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE, REGULAR_EXPRESSION_NODE, STRING_NODE]
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
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        let method_name = call.name().as_slice();
        let is_bang = match method_name {
            b"gsub" => false,
            b"gsub!" => true,
            _ => return,
        };

        // Must have a receiver (str.gsub)
        if call.receiver().is_none() {
            return;
        }

        let arguments = match call.arguments() {
            Some(a) => a,
            None => return,
        };

        let args = arguments.arguments();
        if args.len() != 2 {
            return;
        }

        let mut args_iter = args.iter();
        let first_node = match args_iter.next() {
            Some(a) => a,
            None => return,
        };
        let second_node = match args_iter.next() {
            Some(a) => a,
            None => return,
        };

        // First arg: either a StringNode or a RegularExpressionNode with a single literal char
        let first_is_single_char = if let Some(first) = first_node.as_string_node() {
            let first_str = first.unescaped();
            let first_text = String::from_utf8_lossy(first_str);
            first_text.chars().count() == 1
        } else if let Some(regex) = first_node.as_regular_expression_node() {
            // Reject if regex has flags (e.g., /a/i)
            let closing = regex.closing_loc().as_slice();
            if closing.len() > 1 {
                return;
            }
            let content = regex.content_loc().as_slice();
            is_single_char_deterministic_regex(content)
        } else {
            return;
        };

        if !first_is_single_char {
            return;
        }

        // Second arg must be a StringNode with 0 or 1 characters
        let second = match second_node.as_string_node() {
            Some(s) => s,
            None => return,
        };

        let second_str = second.unescaped();
        let second_text = String::from_utf8_lossy(second_str);
        let second_char_count = second_text.chars().count();
        if second_char_count > 1 {
            return;
        }

        let (prefer, current) = if second_char_count == 0 {
            if is_bang {
                ("delete!", "gsub!")
            } else {
                ("delete", "gsub")
            }
        } else if is_bang {
            ("tr!", "gsub!")
        } else {
            ("tr", "gsub")
        };

        // RuboCop points at the method name through end of args (node.loc.selector → end)
        let loc = call.message_loc().unwrap_or_else(|| call.location());
        let (line, column) = source.offset_to_line_col(loc.start_offset());
        diagnostics.push(self.diagnostic(
            source,
            line,
            column,
            format!("Use `{prefer}` instead of `{current}`."),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(StringReplacement, "cops/performance/string_replacement");
}
