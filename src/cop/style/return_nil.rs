use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

/// Corpus investigation (2026-03-17):
/// - FP=5: All in fastlane, `return nil` inside `proc do |result| ... end` blocks.
///   Root cause: proc creates non-local exit context (return exits the enclosing method),
///   so RuboCop suppresses the offense (defers to Lint/NonLocalExitFromIterator).
///   Fix: detect `proc` and `Proc.new` calls and treat their blocks as iterator blocks.
/// - FN=2: `return nil` inside `lambda do...end` (method-style lambda, not stabby `-> {}`).
///   Root cause: Prism parses `lambda do...end` as CallNode (not LambdaNode). The
///   visit_call_node pushed a block context but didn't reset the block stack like
///   visit_lambda_node does for stabby lambdas. When nested inside an outer iterator
///   block, the outer block remained on the stack and suppressed the offense.
///   Fix: detect `lambda` calls and save/restore block stack (same as visit_lambda_node).
pub struct ReturnNil;

impl Cop for ReturnNil {
    fn name(&self) -> &'static str {
        "Style/ReturnNil"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn default_enabled(&self) -> bool {
        false
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::parse::codemap::CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let enforced_style = config.get_str("EnforcedStyle", "return");
        let mut visitor = ReturnNilVisitor {
            cop: self,
            source,
            enforced_style,
            diagnostics: Vec::new(),
            corrections: Vec::new(),
            autocorrect_enabled: corrections.is_some(),
            block_stack: Vec::new(),
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
        if let Some(corr) = corrections.as_mut() {
            corr.extend(visitor.corrections);
        }
    }
}

/// Tracks block context to determine whether a `return` is inside an iterator block.
#[derive(Clone)]
struct BlockContext {
    has_args: bool,
    is_chained_send: bool,
    is_define_method: bool,
}

struct ReturnNilVisitor<'a, 'src> {
    cop: &'a ReturnNil,
    source: &'src SourceFile,
    enforced_style: &'a str,
    diagnostics: Vec<Diagnostic>,
    corrections: Vec<Correction>,
    autocorrect_enabled: bool,
    block_stack: Vec<BlockContext>,
}

impl ReturnNilVisitor<'_, '_> {
    /// Check if `return` is inside an iterator block (chained send with args).
    /// Mirrors RuboCop's ancestor walk in `on_return`:
    /// - If we hit a define_method block → stop (it creates its own scope)
    /// - If block has no args → skip, keep looking outward
    /// - If block has args and is a chained send → suppress (iterator, non-local exit)
    fn inside_iterator_block(&self) -> bool {
        for ctx in self.block_stack.iter().rev() {
            if ctx.is_define_method {
                return false;
            }
            if !ctx.has_args {
                continue;
            }
            if ctx.is_chained_send {
                return true;
            }
        }
        false
    }
}

impl<'pr> Visit<'pr> for ReturnNilVisitor<'_, '_> {
    fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode<'pr>) {
        // RuboCop suppresses the offense when `return` is inside an iterator block
        // to avoid double-reporting with Lint/NonLocalExitFromIterator.
        if self.inside_iterator_block() {
            return;
        }

        match self.enforced_style {
            "return" => {
                // Flag `return nil` — prefer `return`
                if let Some(args) = node.arguments() {
                    let arg_list: Vec<_> = args.arguments().iter().collect();
                    if arg_list.len() == 1 && arg_list[0].as_nil_node().is_some() {
                        let loc = node.location();
                        let (line, column) = self.source.offset_to_line_col(loc.start_offset());
                        let mut diag = self.cop.diagnostic(
                            self.source,
                            line,
                            column,
                            "Use `return` instead of `return nil`.".to_string(),
                        );

                        if self.autocorrect_enabled {
                            self.corrections.push(Correction {
                                start: loc.start_offset(),
                                end: loc.end_offset(),
                                replacement: "return".to_string(),
                                cop_name: self.cop.name(),
                                cop_index: 0,
                            });
                            diag.corrected = true;
                        }

                        self.diagnostics.push(diag);
                    }
                }
            }
            "return_nil" => {
                // Flag bare `return` — prefer `return nil`
                if node.arguments().is_none() {
                    let loc = node.location();
                    let (line, column) = self.source.offset_to_line_col(loc.start_offset());
                    let mut diag = self.cop.diagnostic(
                        self.source,
                        line,
                        column,
                        "Use `return nil` instead of `return`.".to_string(),
                    );

                    if self.autocorrect_enabled {
                        self.corrections.push(Correction {
                            start: loc.start_offset(),
                            end: loc.end_offset(),
                            replacement: "return nil".to_string(),
                            cop_name: self.cop.name(),
                            cop_index: 0,
                        });
                        diag.corrected = true;
                    }

                    self.diagnostics.push(diag);
                }
            }
            _ => {}
        }
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        // Visit receiver first
        if let Some(recv) = node.receiver() {
            self.visit(&recv);
        }
        // Visit arguments
        if let Some(args) = node.arguments() {
            self.visit(&args.as_node());
        }
        // If call has a block, push block context and visit block body
        if let Some(block) = node.block() {
            if let Some(block_node) = block.as_block_node() {
                let method_name = node.name().as_slice();

                // `lambda do...end` creates its own scope (like stabby `-> {}`).
                // In Prism, method-style `lambda` is a CallNode, not LambdaNode.
                // Save and restore the block stack to isolate the lambda scope.
                if method_name == b"lambda" && node.receiver().is_none() {
                    let saved = std::mem::take(&mut self.block_stack);
                    if let Some(body) = block_node.body() {
                        self.visit(&body);
                    }
                    self.block_stack = saved;
                    return;
                }

                // `proc do...end` and `Proc.new do...end` create non-local exit
                // contexts — `return` inside a proc returns from the enclosing
                // method. Treat as an iterator block to suppress the offense,
                // matching RuboCop's behavior which defers to
                // Lint/NonLocalExitFromIterator.
                let is_proc = (method_name == b"proc" && node.receiver().is_none())
                    || (method_name == b"new"
                        && node.receiver().is_some_and(|r| {
                            r.as_constant_read_node()
                                .is_some_and(|c| c.name().as_slice() == b"Proc")
                                || r.as_constant_path_node().is_some_and(|cp| {
                                    cp.parent().is_none()
                                        && cp.name().is_some_and(|n| n.as_slice() == b"Proc")
                                })
                        }));
                if is_proc {
                    self.block_stack.push(BlockContext {
                        has_args: true,
                        is_chained_send: true,
                        is_define_method: false,
                    });
                    if let Some(body) = block_node.body() {
                        self.visit(&body);
                    }
                    self.block_stack.pop();
                    return;
                }

                let has_args = block_node.parameters().is_some();
                let is_chained_send = node.receiver().is_some();
                let is_define_method =
                    method_name == b"define_method" || method_name == b"define_singleton_method";

                self.block_stack.push(BlockContext {
                    has_args,
                    is_chained_send,
                    is_define_method,
                });
                if let Some(body) = block_node.body() {
                    self.visit(&body);
                }
                self.block_stack.pop();
            } else {
                // BlockArgumentNode (&block) — visit it normally
                self.visit(&block);
            }
        }
    }

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        // Standalone block (not attached to a call — handled via visit_call_node above)
        let has_args = node.parameters().is_some();
        self.block_stack.push(BlockContext {
            has_args,
            is_chained_send: false,
            is_define_method: false,
        });
        if let Some(body) = node.body() {
            self.visit(&body);
        }
        self.block_stack.pop();
    }

    // Don't recurse into nested def/class/module/lambda (they create their own scope)
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        // Reset block stack inside method definitions — they create a new scope
        let saved = std::mem::take(&mut self.block_stack);
        ruby_prism::visit_def_node(self, node);
        self.block_stack = saved;
    }

    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
        let saved = std::mem::take(&mut self.block_stack);
        ruby_prism::visit_lambda_node(self, node);
        self.block_stack = saved;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ReturnNil, "cops/style/return_nil");
    crate::cop_autocorrect_fixture_tests!(ReturnNil, "cops/style/return_nil");
}
