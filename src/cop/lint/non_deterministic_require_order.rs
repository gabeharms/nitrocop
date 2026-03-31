use crate::cop::node_type::CALL_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct NonDeterministicRequireOrder;

impl Cop for NonDeterministicRequireOrder {
    fn name(&self) -> &'static str {
        "Lint/NonDeterministicRequireOrder"
    }

    fn supports_autocorrect(&self) -> bool {
        true
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
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // RuboCop limits this cop to Ruby <= 2.7.
        let ruby_version = config
            .options
            .get("TargetRubyVersion")
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_u64().map(|u| u as f64))
                    .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
            })
            .unwrap_or(2.7);
        if ruby_version > 2.7 {
            return;
        }

        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        if call.name().as_slice() != b"each" {
            return;
        }

        let recv = match call.receiver() {
            Some(r) => r,
            None => return,
        };
        if !is_unsorted_dir_list(&recv) {
            return;
        }

        let block = match call.block() {
            Some(b) => b,
            None => return,
        };
        let block_node = match block.as_block_node() {
            Some(b) => b,
            None => return,
        };

        if !block_requires_iteration_var(&block_node) {
            return;
        }

        let loc = call.location();
        let (line, column) = source.offset_to_line_col(loc.start_offset());
        let mut diag = self.diagnostic(
            source,
            line,
            column,
            "Sort files before requiring them.".to_string(),
        );

        if let Some(corr) = corrections.as_mut() {
            if let Some(selector) = call.message_loc() {
                corr.push(crate::correction::Correction {
                    start: selector.start_offset(),
                    end: selector.end_offset(),
                    replacement: "sort.each".to_string(),
                    cop_name: self.name(),
                    cop_index: 0,
                });
                diag.corrected = true;
            }
        }

        diagnostics.push(diag);
    }
}

fn is_unsorted_dir_list(node: &ruby_prism::Node<'_>) -> bool {
    let call = match node.as_call_node() {
        Some(c) => c,
        None => return false,
    };

    match call.name().as_slice() {
        b"[]" | b"glob" => {
            if let Some(recv) = call.receiver() {
                recv.as_constant_read_node()
                    .is_some_and(|c| c.name().as_slice() == b"Dir")
                    || recv.as_constant_path_node().is_some_and(|cp| {
                        cp.parent().is_none() && cp.name().is_some_and(|n| n.as_slice() == b"Dir")
                    })
            } else {
                false
            }
        }
        b"sort" => false,
        _ => false,
    }
}

fn block_requires_iteration_var(block: &ruby_prism::BlockNode<'_>) -> bool {
    let params = match block
        .parameters()
        .and_then(|p| p.as_block_parameters_node())
    {
        Some(p) => p,
        None => return false,
    };
    let parameter_list = match params.parameters() {
        Some(p) => p,
        None => return false,
    };
    let mut req = parameter_list.requireds().iter();
    let var = match req.next().and_then(|n| n.as_required_parameter_node()) {
        Some(v) => v.name().as_slice().to_vec(),
        None => return false,
    };
    if req.next().is_some() {
        return false;
    }

    let body = match block.body() {
        Some(b) => b,
        None => return false,
    };

    contains_require_of_var(&body, &var)
}

fn contains_require_of_var(node: &ruby_prism::Node<'_>, var: &[u8]) -> bool {
    if let Some(call) = node.as_call_node() {
        let name = call.name().as_slice();
        if (name == b"require" || name == b"require_relative") && call.receiver().is_none() {
            if let Some(args) = call.arguments() {
                let mut it = args.arguments().iter();
                if let Some(arg) = it.next() {
                    if it.next().is_none()
                        && arg
                            .as_local_variable_read_node()
                            .is_some_and(|l| l.name().as_slice() == var)
                    {
                        return true;
                    }
                }
            }
        }

        if let Some(recv) = call.receiver() {
            if contains_require_of_var(&recv, var) {
                return true;
            }
        }
        if let Some(args) = call.arguments() {
            for arg in args.arguments().iter() {
                if contains_require_of_var(&arg, var) {
                    return true;
                }
            }
        }
        if let Some(block) = call.block() {
            if let Some(block_node) = block.as_block_node() {
                if let Some(body) = block_node.body() {
                    if contains_require_of_var(&body, var) {
                        return true;
                    }
                }
            }
        }
    }

    if let Some(stmts) = node.as_statements_node() {
        for child in stmts.body().iter() {
            if contains_require_of_var(&child, var) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::run_cop_full_with_config;
    use std::collections::HashMap;

    crate::cop_fixture_tests!(
        NonDeterministicRequireOrder,
        "cops/lint/non_deterministic_require_order"
    );
    crate::cop_autocorrect_fixture_tests!(
        NonDeterministicRequireOrder,
        "cops/lint/non_deterministic_require_order"
    );

    #[test]
    fn ruby3_plus_is_noop() {
        let config = CopConfig {
            options: HashMap::from([(
                "TargetRubyVersion".into(),
                serde_yml::Value::Number(serde_yml::Number::from(3.2)),
            )]),
            ..CopConfig::default()
        };
        let source = b"Dir['./lib/**/*.rb'].each do |file|\n  require file\nend\n";
        let diags = run_cop_full_with_config(&NonDeterministicRequireOrder, source, config);
        assert!(diags.is_empty());
    }
}
