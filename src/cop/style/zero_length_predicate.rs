use crate::cop::node_type::{CALL_NODE, INTEGER_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Style/ZeroLengthPredicate: Checks for `size == 0`, `length.zero?`, etc.
///
/// ## Investigation findings (2026-03-14)
/// FP=2, FN=0. Two false positive patterns found:
/// 1. Safe navigation chains (e.g., `values&.length&.> 0`) — `empty?` is not equivalent
///    because nil handling differs. Fixed by checking `call_operator()` for `&.` on any
///    call in the chain.
/// 2. Non-collection `.size`/`.length` (e.g., `File.stat(path).size.zero?`) — the receiver
///    returns an integer (file size), not a collection. Fixed by checking if the receiver
///    of `.size`/`.length` is a call on a constant (e.g., `File.stat`), which indicates
///    a non-collection context.
pub struct ZeroLengthPredicate;

impl ZeroLengthPredicate {
    /// Check if a CallNode uses safe navigation (`&.`)
    fn uses_safe_navigation(call: &ruby_prism::CallNode<'_>) -> bool {
        call.call_operator_loc()
            .is_some_and(|op: ruby_prism::Location<'_>| op.as_slice() == b"&.")
    }

    /// Check if the receiver of `.size`/`.length` is a call on a constant,
    /// indicating a non-collection return type (e.g., `File.stat(path).size`).
    fn is_non_collection_receiver(call: &ruby_prism::CallNode<'_>) -> bool {
        if let Some(receiver) = call.receiver() {
            if let Some(recv_call) = receiver.as_call_node() {
                if let Some(recv_recv) = recv_call.receiver() {
                    return recv_recv.as_constant_read_node().is_some()
                        || recv_recv.as_constant_path_node().is_some();
                }
            }
        }
        false
    }

    /// Check if a call is `.length` or `.size` on a collection receiver
    /// (excludes safe navigation and non-collection receivers)
    fn is_length_or_size(node: &ruby_prism::Node<'_>) -> bool {
        if let Some(call) = node.as_call_node() {
            let name = call.name();
            let name_bytes = name.as_slice();
            if (name_bytes == b"length" || name_bytes == b"size")
                && call.arguments().is_none()
                && call.receiver().is_some()
                && !Self::uses_safe_navigation(&call)
                && !Self::is_non_collection_receiver(&call)
            {
                return true;
            }
        }
        false
    }

    /// Get the integer value from a node
    fn int_value(node: &ruby_prism::Node<'_>) -> Option<i64> {
        if let Some(int_node) = node.as_integer_node() {
            // We need to extract the integer value from the source
            let src = int_node.location().as_slice();
            if let Ok(s) = std::str::from_utf8(src) {
                return s.parse::<i64>().ok();
            }
        }
        None
    }

    fn receiver_empty_predicate(
        source: &SourceFile,
        len_call: &ruby_prism::CallNode<'_>,
    ) -> Option<String> {
        let recv = len_call.receiver()?;
        let recv_src = String::from_utf8_lossy(
            &source.as_bytes()[recv.location().start_offset()..recv.location().end_offset()],
        )
        .to_string();
        Some(format!("{}.empty?", recv_src))
    }

    fn comparison_replacement(
        source: &SourceFile,
        call: &ruby_prism::CallNode<'_>,
    ) -> Option<String> {
        let method = call.name().as_slice();
        let args = call.arguments()?;
        let arg_list: Vec<_> = args.arguments().iter().collect();
        if arg_list.len() != 1 {
            return None;
        }

        let left = call.receiver()?;
        let right = &arg_list[0];

        if let Some(len_call) = left.as_call_node() {
            if Self::is_length_or_size(&left) {
                let rhs_int = Self::int_value(right)?;
                let empty = match method {
                    b"==" => rhs_int == 0,
                    b"<" => rhs_int == 1,
                    b"!=" => {
                        if rhs_int == 0 {
                            return Some(format!(
                                "!{}",
                                Self::receiver_empty_predicate(source, &len_call)?
                            ));
                        }
                        false
                    }
                    b">" => {
                        if rhs_int == 0 {
                            return Some(format!(
                                "!{}",
                                Self::receiver_empty_predicate(source, &len_call)?
                            ));
                        }
                        false
                    }
                    _ => false,
                };
                if empty {
                    return Self::receiver_empty_predicate(source, &len_call);
                }
                return None;
            }
        }

        if let Some(lhs_int) = Self::int_value(&left) {
            if let Some(len_call) = right.as_call_node() {
                if Self::is_length_or_size(right) {
                    let empty = match method {
                        b"==" => lhs_int == 0,
                        b">" => lhs_int == 1,
                        b"!=" => {
                            if lhs_int == 0 {
                                return Some(format!(
                                    "!{}",
                                    Self::receiver_empty_predicate(source, &len_call)?
                                ));
                            }
                            false
                        }
                        b"<" => {
                            if lhs_int == 0 {
                                return Some(format!(
                                    "!{}",
                                    Self::receiver_empty_predicate(source, &len_call)?
                                ));
                            }
                            false
                        }
                        _ => false,
                    };
                    if empty {
                        return Self::receiver_empty_predicate(source, &len_call);
                    }
                }
            }
        }

        None
    }
}

impl Cop for ZeroLengthPredicate {
    fn name(&self) -> &'static str {
        "Style/ZeroLengthPredicate"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE, INTEGER_NODE]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        let method_name = call.name();
        let method_bytes = method_name.as_slice();

        // Pattern: x.length.zero? or x.size.zero?
        if method_bytes == b"zero?"
            && call.arguments().is_none()
            && !Self::uses_safe_navigation(&call)
        {
            if let Some(receiver) = call.receiver() {
                if let Some(len_call) = receiver.as_call_node() {
                    if Self::is_length_or_size(&receiver) {
                        let loc = node.location();
                        let (line, column) = source.offset_to_line_col(loc.start_offset());
                        let src = std::str::from_utf8(loc.as_slice()).unwrap_or("");
                        let mut diag = self.diagnostic(
                            source,
                            line,
                            column,
                            format!("Use `empty?` instead of `{}`.", src),
                        );
                        if let Some(replacement) = Self::receiver_empty_predicate(source, &len_call)
                        {
                            if let Some(corr) = corrections.as_mut() {
                                corr.push(crate::correction::Correction {
                                    start: loc.start_offset(),
                                    end: loc.end_offset(),
                                    replacement,
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
        }

        // Pattern: x.length == 0, x.size == 0, 0 == x.length, x.length < 1, etc.
        if matches!(method_bytes, b"==" | b"!=" | b">" | b"<") && !Self::uses_safe_navigation(&call)
        {
            if let Some(replacement) = Self::comparison_replacement(source, &call) {
                let loc = node.location();
                let (line, column) = source.offset_to_line_col(loc.start_offset());
                let src = std::str::from_utf8(loc.as_slice()).unwrap_or("");
                let mut diag = self.diagnostic(
                    source,
                    line,
                    column,
                    if replacement.starts_with('!') {
                        format!("Use `!empty?` instead of `{}`.", src)
                    } else {
                        format!("Use `empty?` instead of `{}`.", src)
                    },
                );
                if let Some(corr) = corrections.as_mut() {
                    corr.push(crate::correction::Correction {
                        start: loc.start_offset(),
                        end: loc.end_offset(),
                        replacement,
                        cop_name: self.name(),
                        cop_index: 0,
                    });
                    diag.corrected = true;
                }
                diagnostics.push(diag);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ZeroLengthPredicate, "cops/style/zero_length_predicate");
    crate::cop_autocorrect_fixture_tests!(ZeroLengthPredicate, "cops/style/zero_length_predicate");
}
