/// Lint/DeprecatedClassMethods
///
/// Detects deprecated class method usage and suggests replacements:
/// - `File.exists?` / `Dir.exists?` → `exist?`
/// - `ENV.clone` / `ENV.dup` → `ENV.to_h`
/// - `ENV.freeze` → `ENV`
/// - `iterator?` → `block_given?`
/// - `attr :name, true` → `attr_accessor :name`
/// - `attr :name, false` → `attr_reader :name`
/// - `Socket.gethostbyaddr` → `Addrinfo#getnameinfo`
/// - `Socket.gethostbyname` → `Addrinfo.getaddrinfo`
///
/// Investigation notes (2026-03):
/// Original implementation only handled File.exists? and Dir.exists?.
/// Added all remaining patterns from RuboCop's RESTRICT_ON_SEND list.
/// FN=60 was caused by missing ENV, iterator?, attr, and Socket patterns.
// Handles both as_constant_read_node and as_constant_path_node (qualified constants like ::File)
use crate::cop::node_type::CALL_NODE;
use crate::cop::util::constant_name;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct DeprecatedClassMethods;

impl Cop for DeprecatedClassMethods {
    fn name(&self) -> &'static str {
        "Lint/DeprecatedClassMethods"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE]
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

        // Handle receiver-less calls: `iterator?` and `attr :name, true/false`
        if call.receiver().is_none() {
            if method_name == b"iterator?" {
                // `iterator?` → `block_given?`
                let loc = call.location();
                let (line, column) = source.offset_to_line_col(loc.start_offset());
                let message = "`iterator?` is deprecated in favor of `block_given?`.".to_string();
                diagnostics.push(self.diagnostic(source, line, column, message));
                return;
            }

            if method_name == b"attr" {
                // `attr :name, true` → `attr_accessor :name`
                // `attr :name, false` → `attr_reader :name`
                // Only flag when the second argument is a boolean literal
                if let Some(args) = call.arguments() {
                    let arg_list: Vec<_> = args.arguments().iter().collect();
                    if arg_list.len() == 2 {
                        let second = &arg_list[1];
                        let is_true = second.as_true_node().is_some();
                        let is_false = second.as_false_node().is_some();
                        if is_true || is_false {
                            let loc = call.location();
                            let call_source =
                                source.byte_slice(loc.start_offset(), loc.end_offset(), "attr");

                            let first_arg_source = {
                                let first = &arg_list[0];
                                let fl = first.location();
                                source.byte_slice(fl.start_offset(), fl.end_offset(), ":name")
                            };

                            let preferred = if is_true {
                                format!("attr_accessor {}", first_arg_source)
                            } else {
                                format!("attr_reader {}", first_arg_source)
                            };

                            let message = format!(
                                "`{}` is deprecated in favor of `{}`.",
                                call_source, preferred
                            );
                            let (line, column) = source.offset_to_line_col(loc.start_offset());
                            diagnostics.push(self.diagnostic(source, line, column, message));
                        }
                    }
                }
                return;
            }

            return;
        }

        let receiver = call.receiver().unwrap();
        let class_name = match constant_name(&receiver) {
            Some(n) => n,
            None => return,
        };

        // Get the receiver source text (e.g., "File", "::File", "ENV", "Socket")
        let receiver_loc = receiver.location();
        let receiver_source =
            source.byte_slice(receiver_loc.start_offset(), receiver_loc.end_offset(), "");

        match (class_name, method_name) {
            // File.exists? / Dir.exists?
            (b"File" | b"Dir", b"exists?") => {
                let current = format!("{}.exists?", receiver_source);
                let prefer = format!("{}.exist?", receiver_source);
                let message = format!("`{}` is deprecated in favor of `{}`.", current, prefer);

                // Offense range: from receiver start to end of method selector
                let (line, column) = source.offset_to_line_col(receiver_loc.start_offset());
                diagnostics.push(self.diagnostic(source, line, column, message));
            }

            // ENV.clone / ENV.dup
            (b"ENV", b"clone" | b"dup") => {
                let method_str = if method_name == b"clone" {
                    "clone"
                } else {
                    "dup"
                };
                let current = format!("{}.{}", receiver_source, method_str);
                let prefer = format!("{}.to_h", receiver_source);
                let message = format!("`{}` is deprecated in favor of `{}`.", current, prefer);

                let (line, column) = source.offset_to_line_col(receiver_loc.start_offset());
                diagnostics.push(self.diagnostic(source, line, column, message));
            }

            // ENV.freeze
            (b"ENV", b"freeze") => {
                let current = format!("{}.freeze", receiver_source);
                let prefer = "ENV";
                let message = format!("`{}` is deprecated in favor of `{}`.", current, prefer);

                let (line, column) = source.offset_to_line_col(receiver_loc.start_offset());
                diagnostics.push(self.diagnostic(source, line, column, message));
            }

            // Socket.gethostbyaddr / Socket.gethostbyname
            (b"Socket", b"gethostbyaddr") => {
                let current = format!("{}.gethostbyaddr", receiver_source);
                let message = format!(
                    "`{}` is deprecated in favor of `Addrinfo#getnameinfo`.",
                    current
                );

                let (line, column) = source.offset_to_line_col(receiver_loc.start_offset());
                diagnostics.push(self.diagnostic(source, line, column, message));
            }

            (b"Socket", b"gethostbyname") => {
                let current = format!("{}.gethostbyname", receiver_source);
                let message = format!(
                    "`{}` is deprecated in favor of `Addrinfo.getaddrinfo`.",
                    current
                );

                let (line, column) = source.offset_to_line_col(receiver_loc.start_offset());
                diagnostics.push(self.diagnostic(source, line, column, message));
            }

            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(DeprecatedClassMethods, "cops/lint/deprecated_class_methods");
}
