use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

/// Checks that access modifiers are declared in the correct style (group or inline).
///
/// ## Investigation (2026-03-13)
///
/// Root cause of 543 FPs: nitrocop was flagging `private def method_name` inside
/// block bodies (e.g., `class_methods do`, `included do`, `concern do`). RuboCop's
/// `allowed?` method checks `node.parent&.type?(:pair, :any_block)` — access modifiers
/// whose parent is a block node are always skipped. This is because DSL blocks like
/// `class_methods do...end` are not class/module bodies, so the group/inline style
/// enforcement doesn't apply there.
///
/// Fix: Switched from `check_node` to `check_source` with a visitor that tracks whether
/// the current scope is a class/module body vs a block body. Access modifiers are only
/// checked when directly inside a class/module/sclass body, not inside block bodies.
///
/// ## Investigation (2026-03-15)
///
/// Root cause of remaining 471 FPs + 56 FNs: Two missing RuboCop behaviors in group mode:
/// 1. `right_siblings_same_inline_method?` — RuboCop skips flagging an inline access
///    modifier if any right sibling in the same body also uses the same inline modifier.
///    This means in a class with multiple `private def foo` / `private def bar`, only
///    the LAST one is flagged (no right sibling to skip it). nitrocop was flagging ALL.
/// 2. `!node.parent&.if_type?` — RuboCop skips inline modifiers whose parent is an
///    if/unless conditional node.
///
/// Fix: Changed from per-call-node checking to batch processing of body statements.
/// For class/module/sclass bodies, we now scan siblings to implement the right-siblings
/// logic, and check if the call's parent statement is an if/unless node.
pub struct AccessModifierDeclarations;

const ACCESS_MODIFIERS: &[&str] = &["private", "protected", "public", "module_function"];

struct AccessModifierVisitor<'a> {
    source: &'a SourceFile,
    cop: &'a AccessModifierDeclarations,
    enforced_style: &'a str,
    allow_modifiers_on_symbols: bool,
    allow_modifiers_on_attrs: bool,
    allow_modifiers_on_alias_method: bool,
    diagnostics: Vec<Diagnostic>,
    /// true when the current scope is a class/module/sclass body (not a block)
    in_class_body: bool,
}

/// Classify an access modifier call. Returns (method_name, is_inline) or None
/// if the call should be skipped entirely (not an access modifier, has receiver,
/// or is allowed by config).
fn classify_access_modifier<'a>(
    call: &ruby_prism::CallNode<'a>,
    allow_modifiers_on_symbols: bool,
    allow_modifiers_on_attrs: bool,
    allow_modifiers_on_alias_method: bool,
) -> Option<(&'a str, bool)> {
    let name_bytes = call.name();
    let method_name = std::str::from_utf8(name_bytes.as_slice()).unwrap_or("");
    if !ACCESS_MODIFIERS.contains(&method_name) {
        return None;
    }

    if call.receiver().is_some() {
        return None;
    }

    let args = match call.arguments() {
        Some(a) => a,
        None => return Some((method_name, false)), // Group-style modifier with no args
    };

    let arg_list: Vec<_> = args.arguments().iter().collect();
    if arg_list.is_empty() {
        return Some((method_name, false));
    }

    let first_arg = &arg_list[0];
    let is_symbol_arg = first_arg.as_symbol_node().is_some();

    if is_symbol_arg && allow_modifiers_on_symbols {
        return None; // Allowed
    }

    if allow_modifiers_on_attrs {
        if let Some(inner_call) = first_arg.as_call_node() {
            let inner_name = std::str::from_utf8(inner_call.name().as_slice()).unwrap_or("");
            if matches!(
                inner_name,
                "attr_reader" | "attr_writer" | "attr_accessor" | "attr"
            ) {
                return None;
            }
        }
    }

    if allow_modifiers_on_alias_method {
        if let Some(inner_call) = first_arg.as_call_node() {
            let inner_name = std::str::from_utf8(inner_call.name().as_slice()).unwrap_or("");
            if inner_name == "alias_method" {
                return None;
            }
        }
    }

    let is_inline = first_arg.as_def_node().is_some() || first_arg.as_symbol_node().is_some();
    Some((method_name, is_inline))
}

/// Info about an access modifier at a given position in a body's statement list.
struct ModifierInfo<'a> {
    method_name: &'a str,
    is_inline: bool,
    start_offset: usize,
}

impl AccessModifierVisitor<'_> {
    /// Process body statements from a class/module/sclass, implementing the
    /// right-siblings logic for group mode.
    fn check_body_statements<'pr>(&mut self, stmts: &[ruby_prism::Node<'pr>]) {
        if self.enforced_style != "group" {
            // For inline style, just use normal traversal
            for node in stmts {
                self.dispatch_visit(node);
            }
            return;
        }

        // Classify each direct child statement
        let infos: Vec<Option<ModifierInfo>> = stmts
            .iter()
            .map(|node| {
                // Only classify direct call nodes (not inside if/unless)
                let call = node.as_call_node()?;
                let offset = call.location().start_offset();
                classify_access_modifier(
                    &call,
                    self.allow_modifiers_on_symbols,
                    self.allow_modifiers_on_attrs,
                    self.allow_modifiers_on_alias_method,
                )
                .map(|(method_name, is_inline)| ModifierInfo {
                    method_name,
                    is_inline,
                    start_offset: offset,
                })
            })
            .collect();

        // Flag offenses, checking right siblings
        for (i, (_node, info)) in stmts.iter().zip(infos.iter()).enumerate() {
            if let Some(info) = info {
                if info.is_inline {
                    // Check if any right sibling has the same inline modifier
                    let has_right_sibling_same = infos[i + 1..].iter().any(|other| {
                        matches!(other, Some(o) if o.is_inline && o.method_name == info.method_name)
                    });

                    if !has_right_sibling_same {
                        let (line, column) = self.source.offset_to_line_col(info.start_offset);
                        self.diagnostics.push(self.cop.diagnostic(
                            self.source,
                            line,
                            column,
                            format!(
                                "`{}` should not be inlined in method definitions.",
                                info.method_name
                            ),
                        ));
                    }
                }
            }
        }

        // Recurse into child nodes for nested classes/modules/blocks
        for node in stmts {
            self.dispatch_visit(node);
        }
    }

    /// Dispatch to the appropriate visit method for a node.
    fn dispatch_visit<'pr>(&mut self, node: &ruby_prism::Node<'pr>) {
        if let Some(ref n) = node.as_class_node() {
            self.visit_class_node(n);
        } else if let Some(ref n) = node.as_module_node() {
            self.visit_module_node(n);
        } else if let Some(ref n) = node.as_singleton_class_node() {
            self.visit_singleton_class_node(n);
        } else if let Some(ref n) = node.as_block_node() {
            self.visit_block_node(n);
        } else if let Some(ref n) = node.as_lambda_node() {
            self.visit_lambda_node(n);
        } else if let Some(ref n) = node.as_call_node() {
            // In group mode, we already handled direct modifiers in check_body_statements.
            // But we still need to recurse into call node children (e.g., block arguments).
            self.visit_call_node(n);
        } else if let Some(ref n) = node.as_if_node() {
            self.visit_if_node(n);
        } else if let Some(ref n) = node.as_def_node() {
            self.visit_def_node(n);
        } else if let Some(ref n) = node.as_begin_node() {
            self.visit_begin_node(n);
        }
        // Other node types don't contain access modifiers we care about
    }
}

impl<'pr> Visit<'pr> for AccessModifierVisitor<'_> {
    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
        let saved = self.in_class_body;
        self.in_class_body = true;

        if let Some(body) = node.body() {
            if let Some(stmts) = body.as_statements_node() {
                let nodes: Vec<_> = stmts.body().iter().collect();
                self.check_body_statements(&nodes);
                self.in_class_body = saved;
                return;
            }
        }

        ruby_prism::visit_class_node(self, node);
        self.in_class_body = saved;
    }

    fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode<'pr>) {
        let saved = self.in_class_body;
        self.in_class_body = true;

        if let Some(body) = node.body() {
            if let Some(stmts) = body.as_statements_node() {
                let nodes: Vec<_> = stmts.body().iter().collect();
                self.check_body_statements(&nodes);
                self.in_class_body = saved;
                return;
            }
        }

        ruby_prism::visit_module_node(self, node);
        self.in_class_body = saved;
    }

    fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode<'pr>) {
        let saved = self.in_class_body;
        self.in_class_body = true;

        if let Some(body) = node.body() {
            if let Some(stmts) = body.as_statements_node() {
                let nodes: Vec<_> = stmts.body().iter().collect();
                self.check_body_statements(&nodes);
                self.in_class_body = saved;
                return;
            }
        }

        ruby_prism::visit_singleton_class_node(self, node);
        self.in_class_body = saved;
    }

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        let saved = self.in_class_body;
        self.in_class_body = false;
        ruby_prism::visit_block_node(self, node);
        self.in_class_body = saved;
    }

    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
        let saved = self.in_class_body;
        self.in_class_body = false;
        ruby_prism::visit_lambda_node(self, node);
        self.in_class_body = saved;
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        // In group mode, direct modifiers in class bodies are handled by check_body_statements.
        // Here we handle inline style and general traversal.
        if self.enforced_style == "inline" && self.in_class_body {
            let method_name = std::str::from_utf8(node.name().as_slice()).unwrap_or("");
            if ACCESS_MODIFIERS.contains(&method_name)
                && node.receiver().is_none()
                && node.arguments().is_none()
            {
                let loc = node.location();
                let (line, column) = self.source.offset_to_line_col(loc.start_offset());
                self.diagnostics.push(self.cop.diagnostic(
                    self.source,
                    line,
                    column,
                    format!("`{}` should not be used in a group style.", method_name),
                ));
            }
        }
        ruby_prism::visit_call_node(self, node);
    }
}

impl Cop for AccessModifierDeclarations {
    fn name(&self) -> &'static str {
        "Style/AccessModifierDeclarations"
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
        let enforced_style = config.get_str("EnforcedStyle", "group");
        let allow_modifiers_on_symbols = config.get_bool("AllowModifiersOnSymbols", true);
        let allow_modifiers_on_attrs = config.get_bool("AllowModifiersOnAttrs", true);
        let allow_modifiers_on_alias_method = config.get_bool("AllowModifiersOnAliasMethod", true);

        let mut visitor = AccessModifierVisitor {
            source,
            cop: self,
            enforced_style,
            allow_modifiers_on_symbols,
            allow_modifiers_on_attrs,
            allow_modifiers_on_alias_method,
            diagnostics: Vec::new(),
            in_class_body: true,
        };

        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        AccessModifierDeclarations,
        "cops/style/access_modifier_declarations"
    );
}
