use crate::cop::node_type::{BLOCK_NODE, CALL_NODE, STATEMENTS_NODE};
use crate::cop::util::RSPEC_DEFAULT_INCLUDE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-07)
///
/// Corpus oracle reported FP=102, FN=317.
///
/// FP root causes:
/// 1) Non-symbol `receive` args (e.g., `receive('action_name')`) were matched.
/// 2) The matcher accepted chains RuboCop excludes, including:
///    - heredoc returns (`and_return(<<~SQL)`)
///    - splat returns (`and_return(*values)`)
///    - multi-arg returns (`and_return(1, 2)`)
///    - calls with additional chained methods after `and_return` (e.g., `.ordered`)
///
/// Fix: mirror RuboCop's node pattern shape exactly:
/// `allow(...).to receive(:symbol).and_return(single_non_heredoc_non_splat_arg)`.
///
/// Acceptance gate after fix (`check-cop --verbose --rerun`):
/// - Expected: 4,670
/// - Actual: 4,373
/// - Excess: 0 (FP resolved)
/// - Missing: 297
///
/// Remaining gap (FN): this cop currently scans block bodies only, while RuboCop also
/// catches repeated stubs in other `begin` contexts (for example method bodies in
/// helper/spec support code). FN work is deferred to a follow-up pass.
pub struct ReceiveMessages;

struct StubInfo {
    receiver_text: String,
    receive_msg: String,
    offset: usize,
}

impl Cop for ReceiveMessages {
    fn name(&self) -> &'static str {
        "RSpec/ReceiveMessages"
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_DEFAULT_INCLUDE
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[BLOCK_NODE, CALL_NODE, STATEMENTS_NODE]
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
        let block = match node.as_block_node() {
            Some(b) => b,
            None => return,
        };

        let body = match block.body() {
            Some(b) => b,
            None => return,
        };

        let stmts = match body.as_statements_node() {
            Some(s) => s,
            None => return,
        };

        let mut stubs: Vec<StubInfo> = Vec::new();

        for stmt in stmts.body().iter() {
            if let Some(info) = extract_allow_receive_info(source, &stmt) {
                stubs.push(info);
            }
        }

        let mut processed = vec![false; stubs.len()];

        for i in 0..stubs.len() {
            if processed[i] {
                continue;
            }

            let mut group = vec![i];
            for j in (i + 1)..stubs.len() {
                if processed[j] {
                    continue;
                }
                if stubs[i].receiver_text == stubs[j].receiver_text {
                    group.push(j);
                }
            }

            if group.len() < 2 {
                continue;
            }

            // Check for duplicate receive messages within this group
            let mut receive_msgs: Vec<&str> = Vec::new();
            let mut has_dups = false;
            for &idx in &group {
                if receive_msgs.contains(&&*stubs[idx].receive_msg) {
                    has_dups = true;
                    break;
                }
                receive_msgs.push(&stubs[idx].receive_msg);
            }

            if has_dups {
                continue;
            }

            for &idx in &group {
                processed[idx] = true;
                let (line, column) = source.offset_to_line_col(stubs[idx].offset);
                diagnostics.push(self.diagnostic(
                    source,
                    line,
                    column,
                    "Use `receive_messages` instead of multiple stubs.".to_string(),
                ));
            }
        }
    }
}

fn extract_allow_receive_info(
    source: &SourceFile,
    node: &ruby_prism::Node<'_>,
) -> Option<StubInfo> {
    // RuboCop node pattern:
    // (send (send nil? :allow ...) :to
    //   (send (send nil? :receive (sym _)) :and_return !#heredoc_or_splat?))
    let to_call = node.as_call_node()?;

    if to_call.name().as_slice() != b"to" || to_call.block().is_some() {
        return None;
    }

    // Receiver must be bare allow(...)
    let allow_call = to_call.receiver()?.as_call_node()?;
    if allow_call.name().as_slice() != b"allow" || allow_call.receiver().is_some() {
        return None;
    }

    // Get receiver text
    let allow_args = allow_call.arguments()?;
    let allow_arg_list: Vec<_> = allow_args.arguments().iter().collect();
    if allow_arg_list.is_empty() {
        return None;
    }
    let recv_loc = allow_arg_list[0].location();
    let receiver_text = source
        .byte_slice(recv_loc.start_offset(), recv_loc.end_offset(), "")
        .to_string();

    // Get the argument chain: receive(:y).and_return(z)
    let to_args = to_call.arguments()?;
    let to_arg_list: Vec<_> = to_args.arguments().iter().collect();
    if to_arg_list.len() != 1 {
        return None;
    }

    // Must be direct .and_return(...) call as the only `to` argument.
    let and_return_call = to_arg_list[0].as_call_node()?;
    if and_return_call.name().as_slice() != b"and_return" || and_return_call.block().is_some() {
        return None;
    }

    // and_return receiver must be direct bare receive(:symbol)
    let receive_call = and_return_call.receiver()?.as_call_node()?;
    if receive_call.name().as_slice() != b"receive"
        || receive_call.receiver().is_some()
        || receive_call.block().is_some()
    {
        return None;
    }

    let receive_args = receive_call.arguments()?;
    let receive_arg_list: Vec<_> = receive_args.arguments().iter().collect();
    if receive_arg_list.len() != 1 {
        return None;
    }
    let receive_symbol = receive_arg_list[0].as_symbol_node()?;

    // and_return must have exactly one non-heredoc/non-splat arg.
    let and_return_args = and_return_call.arguments()?;
    let and_return_arg_list: Vec<_> = and_return_args.arguments().iter().collect();
    if and_return_arg_list.len() != 1 || heredoc_or_splat(&and_return_arg_list[0]) {
        return None;
    }

    let stmt_loc = node.location();
    let msg_loc = receive_symbol.location();
    let receive_msg = source
        .byte_slice(msg_loc.start_offset(), msg_loc.end_offset(), "")
        .to_string();

    Some(StubInfo {
        receiver_text,
        receive_msg,
        offset: stmt_loc.start_offset(),
    })
}

fn heredoc_or_splat(node: &ruby_prism::Node<'_>) -> bool {
    if node.as_splat_node().is_some() {
        return true;
    }

    if let Some(string) = node.as_string_node() {
        return string
            .opening_loc()
            .is_some_and(|opening| opening.as_slice().starts_with(b"<<"));
    }

    if let Some(string) = node.as_interpolated_string_node() {
        return string
            .opening_loc()
            .is_some_and(|opening| opening.as_slice().starts_with(b"<<"));
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ReceiveMessages, "cops/rspec/receive_messages");
}
