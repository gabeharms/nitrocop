use crate::cop::node_type::{CALL_NODE, FOR_NODE, PROGRAM_NODE, STATEMENTS_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct CombinableLoops;

impl Cop for CombinableLoops {
    fn name(&self) -> &'static str {
        "Style/CombinableLoops"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE, FOR_NODE, PROGRAM_NODE, STATEMENTS_NODE]
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
        let stmt_list: Vec<ruby_prism::Node<'_>> =
            if let Some(stmts_node) = node.as_statements_node() {
                stmts_node.body().iter().collect()
            } else if let Some(prog_node) = node.as_program_node() {
                prog_node.statements().body().iter().collect()
            } else {
                return;
            };

        for i in 1..stmt_list.len() {
            let prev = &stmt_list[i - 1];
            let curr = &stmt_list[i];

            if let (Some(prev_info), Some(curr_info)) =
                (get_loop_info(source, prev), get_loop_info(source, curr))
            {
                if prev_info.receiver == curr_info.receiver
                    && prev_info.method == curr_info.method
                    && prev_info.arguments == curr_info.arguments
                {
                    let loc = curr.location();
                    let (line, column) = source.offset_to_line_col(loc.start_offset());
                    let mut diag = self.diagnostic(
                        source,
                        line,
                        column,
                        "Combine this loop with the previous loop.".to_string(),
                    );

                    if let Some(corr) = corrections.as_mut() {
                        let prev_prev_same = i >= 2
                            && get_loop_info(source, &stmt_list[i - 2]).is_some_and(|info| {
                                info.receiver == prev_info.receiver
                                    && info.method == prev_info.method
                                    && info.arguments == prev_info.arguments
                            });
                        let next_same = i + 1 < stmt_list.len()
                            && get_loop_info(source, &stmt_list[i + 1]).is_some_and(|info| {
                                info.receiver == curr_info.receiver
                                    && info.method == curr_info.method
                                    && info.arguments == curr_info.arguments
                            });

                        if !prev_prev_same && !next_same {
                            if let Some(repl) = combine_brace_block_pair(source, prev, curr) {
                                corr.push(crate::correction::Correction {
                                    start: prev.location().start_offset(),
                                    end: curr.location().end_offset(),
                                    replacement: repl,
                                    cop_name: self.name(),
                                    cop_index: 0,
                                });
                                diag.corrected = true;
                            }
                        }
                    }

                    diagnostics.push(diag);
                }
            }
        }
    }
}

struct LoopInfo {
    receiver: String,
    method: String,
    arguments: String,
}

fn is_collection_looping_method(method_name: &str) -> bool {
    method_name.starts_with("each") || method_name.ends_with("_each")
}

fn get_loop_info(source: &SourceFile, node: &ruby_prism::Node<'_>) -> Option<LoopInfo> {
    if let Some(for_node) = node.as_for_node() {
        let collection = for_node.collection();
        let receiver_text = source
            .try_byte_slice(
                collection.location().start_offset(),
                collection.location().end_offset(),
            )?
            .to_string();
        return Some(LoopInfo {
            receiver: receiver_text,
            method: "for".to_string(),
            arguments: String::new(),
        });
    }

    let call = node.as_call_node()?;
    let method_name = std::str::from_utf8(call.name().as_slice()).ok()?;

    if !is_collection_looping_method(method_name) {
        return None;
    }

    let block = call.block()?;
    if let Some(block_node) = block.as_block_node() {
        block_node.body()?;
    }

    let receiver = call.receiver()?;
    let receiver_text = source
        .try_byte_slice(
            receiver.location().start_offset(),
            receiver.location().end_offset(),
        )?
        .to_string();

    let arguments_text = if let Some(args) = call.arguments() {
        source
            .try_byte_slice(args.location().start_offset(), args.location().end_offset())
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };

    Some(LoopInfo {
        receiver: receiver_text,
        method: method_name.to_string(),
        arguments: arguments_text,
    })
}

fn combine_brace_block_pair(
    source: &SourceFile,
    prev: &ruby_prism::Node<'_>,
    curr: &ruby_prism::Node<'_>,
) -> Option<String> {
    let prev_call = prev.as_call_node()?;
    let curr_call = curr.as_call_node()?;

    let prev_block = prev_call.block()?.as_block_node()?;
    let curr_block = curr_call.block()?.as_block_node()?;

    if prev_block.opening_loc().as_slice() != b"{" || curr_block.opening_loc().as_slice() != b"{" {
        return None;
    }

    let prev_params = prev_block.parameters().map(|p| {
        source
            .byte_slice(p.location().start_offset(), p.location().end_offset(), "")
            .to_string()
    });
    let curr_params = curr_block.parameters().map(|p| {
        source
            .byte_slice(p.location().start_offset(), p.location().end_offset(), "")
            .to_string()
    });
    if prev_params != curr_params {
        return None;
    }

    let prev_body = prev_block.body()?.as_statements_node()?;
    let curr_body = curr_block.body()?.as_statements_node()?;
    if prev_body.body().len() != 1 || curr_body.body().len() != 1 {
        return None;
    }

    let prev_stmt = prev_body.body().iter().next()?;
    let curr_stmt = curr_body.body().iter().next()?;

    let prev_stmt_src = source
        .byte_slice(
            prev_stmt.location().start_offset(),
            prev_stmt.location().end_offset(),
            "",
        )
        .trim()
        .to_string();
    let curr_stmt_src = source
        .byte_slice(
            curr_stmt.location().start_offset(),
            curr_stmt.location().end_offset(),
            "",
        )
        .trim()
        .to_string();

    let header = source
        .byte_slice(
            prev_call.location().start_offset(),
            prev_block.opening_loc().start_offset(),
            "",
        )
        .trim_end()
        .to_string();

    let params_src = prev_params.unwrap_or_default();
    let params_part = if params_src.is_empty() {
        String::new()
    } else {
        format!("{params_src} ")
    };

    Some(format!(
        "{header}{{ {params_part}{prev_stmt_src}; {curr_stmt_src} }}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(CombinableLoops, "cops/style/combinable_loops");
    crate::cop_autocorrect_fixture_tests!(CombinableLoops, "cops/style/combinable_loops");
}
