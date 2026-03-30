use crate::cop::node_type::{CALL_NODE, CONSTANT_PATH_NODE, CONSTANT_READ_NODE, STRING_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct FileWrite;

impl FileWrite {
    fn is_file_class(node: &ruby_prism::Node<'_>) -> bool {
        if let Some(c) = node.as_constant_read_node() {
            return c.name().as_slice() == b"File";
        }
        if let Some(cp) = node.as_constant_path_node() {
            if cp.parent().is_none() {
                return cp.name().is_some_and(|n| n.as_slice() == b"File");
            }
        }
        false
    }

    fn is_write_mode(mode: &[u8]) -> bool {
        // Must match RuboCop's TRUNCATING_WRITE_MODES exactly: %w[w wt wb w+ w+t w+b]
        matches!(mode, b"w" | b"wt" | b"wb" | b"w+" | b"w+t" | b"w+b")
    }

    fn write_method(mode: &[u8]) -> &'static str {
        if mode.contains(&b'b') {
            "File.binwrite"
        } else {
            "File.write"
        }
    }

    /// Check if a File.open call has a write-mode string argument.
    /// Returns the mode bytes if found.
    fn check_file_open_mode<'a>(open_call: &ruby_prism::CallNode<'a>) -> Option<Vec<u8>> {
        if open_call.name().as_slice() != b"open" {
            return None;
        }

        let file_recv = open_call.receiver()?;
        if !Self::is_file_class(&file_recv) {
            return None;
        }

        let open_args = open_call.arguments()?;
        let open_arg_list: Vec<_> = open_args.arguments().iter().collect();
        // Must have exactly 2 positional args: filename and mode string.
        // Additional keyword args (encoding:, etc.) mean File.write can't
        // be a drop-in replacement, matching RuboCop's pattern.
        if open_arg_list.len() != 2 {
            return None;
        }
        // Neither argument should be a keyword hash (splat, hash with labels, etc.)
        if open_arg_list[1].as_keyword_hash_node().is_some() {
            return None;
        }

        let str_node = open_arg_list[1].as_string_node()?;
        let content: Vec<u8> = str_node.unescaped().to_vec();
        if !Self::is_write_mode(&content) {
            return None;
        }

        Some(content)
    }

    fn open_filename_source(open_call: &ruby_prism::CallNode<'_>) -> Option<String> {
        let open_args = open_call.arguments()?;
        let open_arg_list: Vec<_> = open_args.arguments().iter().collect();
        if open_arg_list.is_empty() {
            return None;
        }
        Some(
            std::str::from_utf8(open_arg_list[0].location().as_slice())
                .ok()?
                .to_string(),
        )
    }

    /// Check if the block body is a single `block_param.write(content)` call
    /// where the write arg is not a splat, and return that content source.
    fn block_write_content(block: &ruby_prism::BlockNode<'_>) -> Option<String> {
        // Must have exactly one block parameter
        let params = block.parameters()?;
        let block_params = params.as_block_parameters_node()?;
        let params_node = block_params.parameters()?;
        let requireds: Vec<_> = params_node.requireds().iter().collect();
        if requireds.len() != 1 || params_node.optionals().iter().count() > 0 {
            return None;
        }
        let param = requireds[0].as_required_parameter_node()?;
        let param_name = param.name().as_slice();

        // Body must be a single statement
        let body = block.body()?;
        let stmts = body.as_statements_node()?;
        let body_nodes: Vec<_> = stmts.body().iter().collect();
        if body_nodes.len() != 1 {
            return None;
        }

        // The statement must be a call to `.write` on the block param
        let write_call = body_nodes[0].as_call_node()?;
        if write_call.name().as_slice() != b"write" {
            return None;
        }

        // Receiver must be the block parameter (local variable read)
        let recv = write_call.receiver()?;
        let lvar = recv.as_local_variable_read_node()?;
        if lvar.name().as_slice() != param_name {
            return None;
        }

        // Must have exactly one argument to write, and it must not be a splat
        let args = write_call.arguments()?;
        let arg_list: Vec<_> = args.arguments().iter().collect();
        if arg_list.len() != 1 {
            return None;
        }
        if arg_list[0].as_splat_node().is_some() {
            return None;
        }

        Some(
            std::str::from_utf8(arg_list[0].location().as_slice())
                .ok()?
                .to_string(),
        )
    }
}

impl Cop for FileWrite {
    fn name(&self) -> &'static str {
        "Style/FileWrite"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            CALL_NODE,
            CONSTANT_PATH_NODE,
            CONSTANT_READ_NODE,
            STRING_NODE,
        ]
    }

    fn supports_autocorrect(&self) -> bool {
        true
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

        // Pattern 1: File.open(filename, 'w').write(content) — chained call
        if call.name().as_slice() == b"write" {
            if let Some(receiver) = call.receiver() {
                if let Some(open_call) = receiver.as_call_node() {
                    if let Some(mode) = Self::check_file_open_mode(&open_call) {
                        if let Some(file_recv) = open_call.receiver() {
                            if let Some(filename) = Self::open_filename_source(&open_call) {
                                if let Some(write_args) = call.arguments() {
                                    let write_arg_list: Vec<_> =
                                        write_args.arguments().iter().collect();
                                    if write_arg_list.len() == 1
                                        && write_arg_list[0].as_splat_node().is_none()
                                    {
                                        if let Ok(content) = std::str::from_utf8(
                                            write_arg_list[0].location().as_slice(),
                                        ) {
                                            let write_method = Self::write_method(&mode);
                                            let replacement =
                                                format!("{write_method}({filename}, {content})");

                                            let loc = call.location();
                                            let (line, column) =
                                                source.offset_to_line_col(loc.start_offset());
                                            let mut diag = self.diagnostic(
                                                source,
                                                line,
                                                column,
                                                format!("Use `{write_method}`."),
                                            );

                                            if let Some(ref mut corr) = corrections {
                                                corr.push(crate::correction::Correction {
                                                    start: file_recv.location().start_offset(),
                                                    end: call.location().end_offset(),
                                                    replacement,
                                                    cop_name: self.name(),
                                                    cop_index: 0,
                                                });
                                                diag.corrected = true;
                                            }

                                            diagnostics.push(diag);
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Pattern 2: File.open(filename, 'w') { |f| f.write(content) } — block form
        if call.name().as_slice() == b"open" {
            if let Some(mode) = Self::check_file_open_mode(&call) {
                if let Some(file_recv) = call.receiver() {
                    if let Some(filename) = Self::open_filename_source(&call) {
                        if let Some(block) = call.block() {
                            if let Some(block_node) = block.as_block_node() {
                                if let Some(content) = Self::block_write_content(&block_node) {
                                    let write_method = Self::write_method(&mode);
                                    let replacement =
                                        format!("{write_method}({filename}, {content})");

                                    let loc = call.location();
                                    let (line, column) =
                                        source.offset_to_line_col(loc.start_offset());
                                    let mut diag = self.diagnostic(
                                        source,
                                        line,
                                        column,
                                        format!("Use `{write_method}`."),
                                    );

                                    if let Some(ref mut corr) = corrections {
                                        corr.push(crate::correction::Correction {
                                            start: file_recv.location().start_offset(),
                                            end: block.location().end_offset(),
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
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(FileWrite, "cops/style/file_write");
    crate::cop_autocorrect_fixture_tests!(FileWrite, "cops/style/file_write");
}
