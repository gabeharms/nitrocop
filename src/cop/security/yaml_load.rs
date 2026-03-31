use crate::cop::node_type::CALL_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

pub struct YamlLoad;

impl Cop for YamlLoad {
    fn name(&self) -> &'static str {
        "Security/YAMLLoad"
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
        // Ruby 3.1+ (Psych 4) made YAML.load safe by default.
        let ruby_version = config
            .options
            .get("TargetRubyVersion")
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_u64().map(|u| u as f64))
                    .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
            })
            .unwrap_or(2.7);
        if ruby_version > 3.0 {
            return;
        }

        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        if call.name().as_slice() != b"load" {
            return;
        }

        let recv = match call.receiver() {
            Some(r) => r,
            None => return,
        };
        let is_yaml = recv
            .as_constant_read_node()
            .is_some_and(|c| c.name().as_slice() == b"YAML")
            || recv.as_constant_path_node().is_some_and(|cp| {
                cp.parent().is_none() && cp.name().is_some_and(|n| n.as_slice() == b"YAML")
            });
        if !is_yaml {
            return;
        }

        let loc = call.location();
        let (line, column) = source.offset_to_line_col(loc.start_offset());
        let mut diag = self.diagnostic(
            source,
            line,
            column,
            "Prefer using `YAML.safe_load` over `YAML.load`.".to_string(),
        );

        if let Some(corrections) = corrections.as_mut() {
            if let Some(msg_loc) = call.message_loc() {
                corrections.push(crate::correction::Correction {
                    start: msg_loc.start_offset(),
                    end: msg_loc.end_offset(),
                    replacement: "safe_load".to_string(),
                    cop_name: self.name(),
                    cop_index: 0,
                });
                diag.corrected = true;
            }
        }

        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(YamlLoad, "cops/security/yaml_load");
    crate::cop_autocorrect_fixture_tests!(YamlLoad, "cops/security/yaml_load");
}
