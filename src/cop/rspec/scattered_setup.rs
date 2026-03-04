use crate::cop::node_type::{BLOCK_NODE, CALL_NODE, STATEMENTS_NODE};
use crate::cop::util::{self, RSPEC_DEFAULT_INCLUDE, is_rspec_example_group};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// RSpec/ScatteredSetup flags multiple `before`/`after` hooks of the same type,
/// scope, and metadata in the same example group.
///
/// ## Investigation notes
/// - **FP root cause (48 FPs):** Hooks were grouped only by type + scope, ignoring
///   metadata. `before(:each, :unix_only)` and `before(:each)` were incorrectly
///   treated as the same hook. RuboCop groups by type + normalized scope + metadata.
/// - **Scope normalization:** `:each` and `:example` are equivalent; `:all` and
///   `:context` are equivalent. No scope arg defaults to `:each`.
/// - **Metadata:** Additional symbol args (`:special_case`) and keyword hash args
///   (`special_case: true`) form the metadata. Symbol `:foo` is equivalent to
///   `foo: true` in RuboCop's normalization.
pub struct ScatteredSetup;

/// Normalize scope: :each/:example/no-arg → "each", :all/:context → "all".
fn normalize_scope(scope: &[u8]) -> &'static [u8] {
    match scope {
        b"each" | b"example" => b"each",
        b"all" | b"context" => b"all",
        b"suite" => b"suite",
        _ => b"each",
    }
}

/// Build a grouping key for a hook call: normalized scope + sorted metadata.
/// Metadata consists of additional symbol args (beyond the scope) and keyword args.
/// Symbol `:foo` is normalized to `foo:true`, matching RuboCop's behavior.
fn build_hook_key(call: &ruby_prism::CallNode<'_>, source: &SourceFile) -> Vec<u8> {
    let mut scope: &[u8] = b"each";
    let mut metadata_parts: Vec<Vec<u8>> = Vec::new();

    if let Some(args) = call.arguments() {
        let mut found_scope = false;
        for arg in args.arguments().iter() {
            if let Some(sym) = arg.as_symbol_node() {
                let name = sym.unescaped();
                // First symbol that looks like a scope keyword is the scope
                if !found_scope
                    && (name == b"each"
                        || name == b"example"
                        || name == b"all"
                        || name == b"context"
                        || name == b"suite")
                {
                    scope = normalize_scope(name);
                    found_scope = true;
                } else {
                    // Additional symbol args are metadata, equivalent to `name: true`
                    let mut part = name.to_vec();
                    part.extend_from_slice(b":true");
                    metadata_parts.push(part);
                }
            } else if let Some(kw_hash) = arg.as_keyword_hash_node() {
                // Keyword args like `special_case: true`
                for element in kw_hash.elements().iter() {
                    if let Some(assoc) = element.as_assoc_node() {
                        let key_src = source.byte_slice(
                            assoc.key().location().start_offset(),
                            assoc.key().location().end_offset(),
                            "",
                        );
                        let val_src = source.byte_slice(
                            assoc.value().location().start_offset(),
                            assoc.value().location().end_offset(),
                            "",
                        );
                        let mut part = key_src.as_bytes().to_vec();
                        part.push(b':');
                        part.extend_from_slice(val_src.as_bytes());
                        metadata_parts.push(part);
                    }
                }
            }
        }
    }

    metadata_parts.sort();

    let mut key = Vec::new();
    key.extend_from_slice(scope);
    for part in &metadata_parts {
        key.push(b'|');
        key.extend_from_slice(part);
    }
    key
}

impl Cop for ScatteredSetup {
    fn name(&self) -> &'static str {
        "RSpec/ScatteredSetup"
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
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        let method_name = call.name().as_slice();

        let is_example_group = if let Some(recv) = call.receiver() {
            util::constant_name(&recv).is_some_and(|n| n == b"RSpec") && method_name == b"describe"
        } else {
            is_rspec_example_group(method_name)
        };

        if !is_example_group {
            return;
        }

        let block = match call.block() {
            Some(b) => match b.as_block_node() {
                Some(bn) => bn,
                None => return,
            },
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

        // Collect all direct `before` hooks grouped by (hook_type, scope) and flag duplicates.
        // before :all and before :each (or before with no arg) are different scopes.
        let mut before_hooks: std::collections::HashMap<Vec<u8>, Vec<(usize, usize)>> =
            std::collections::HashMap::new();
        let mut after_hooks: std::collections::HashMap<Vec<u8>, Vec<(usize, usize)>> =
            std::collections::HashMap::new();

        for stmt in stmts.body().iter() {
            let c = match stmt.as_call_node() {
                Some(c) => c,
                None => continue,
            };

            let name = c.name().as_slice();
            if c.receiver().is_some() {
                continue;
            }

            let loc = stmt.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());

            let key = build_hook_key(&c, source);

            if name == b"before" || name == b"prepend_before" || name == b"append_before" {
                before_hooks.entry(key).or_default().push((line, column));
            } else if name == b"after" || name == b"prepend_after" || name == b"append_after" {
                after_hooks.entry(key).or_default().push((line, column));
            }
        }

        // Flag duplicate before hooks (same scope only)
        for hooks in before_hooks.values() {
            if hooks.len() > 1 {
                for &(line, column) in hooks {
                    let other_lines: Vec<String> = hooks
                        .iter()
                        .filter(|&&(l, _)| l != line)
                        .map(|&(l, _)| l.to_string())
                        .collect();
                    let also = if other_lines.len() == 1 {
                        format!("line {}", other_lines[0])
                    } else {
                        format!("lines {}", other_lines.join(", "))
                    };
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        format!(
                            "Do not define multiple `before` hooks in the same example group (also defined on {also})."
                        ),
                    ));
                }
            }
        }

        // Flag duplicate after hooks (same scope only)
        for hooks in after_hooks.values() {
            if hooks.len() > 1 {
                for &(line, column) in hooks {
                    let other_lines: Vec<String> = hooks
                        .iter()
                        .filter(|&&(l, _)| l != line)
                        .map(|&(l, _)| l.to_string())
                        .collect();
                    let also = if other_lines.len() == 1 {
                        format!("line {}", other_lines[0])
                    } else {
                        format!("lines {}", other_lines.join(", "))
                    };
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        format!(
                            "Do not define multiple `after` hooks in the same example group (also defined on {also})."
                        ),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ScatteredSetup, "cops/rspec/scattered_setup");
}
