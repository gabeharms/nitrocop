use crate::cop::node_type::{
    CALL_NODE, INTERPOLATED_SYMBOL_NODE, KEYWORD_HASH_NODE, STRING_NODE, SYMBOL_NODE,
};
use crate::cop::util::RSPEC_DEFAULT_INCLUDE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;

/// RSpec/VariableDefinition - checks that memoized helper names use symbols or strings.
///
/// ## Investigation findings (2026-03-11)
/// Root cause of 95 FN (9.5% match rate): all from mikel/mail repo.
///
/// 1. **Block requirement was too strict**: The cop previously required `call.block().is_some()`
///    to distinguish RSpec `subject` from Mail's `subject 'text'` DSL. However, RuboCop does NOT
///    check for block presence — it only checks `inside_example_group?`. When `subject 'text'`
///    appears inside a `Mail.new do...end` block within an RSpec example group, RuboCop flags it.
///    Removing the block guard matches RuboCop's behavior and fixes all 95 FN.
///
/// 2. **Missing InterpolatedSymbolNode (dsym) handling**: RuboCop's `any_sym_type?` matches both
///    `:sym` and `:"dsym_#{x}"`. Added `as_interpolated_symbol_node()` check for `strings` style.
///    Note: RuboCop's `str_type?` does NOT match `dstr` (interpolated strings), so we correctly
///    skip `InterpolatedStringNode` for `symbols` style.
pub struct VariableDefinition;

impl Cop for VariableDefinition {
    fn name(&self) -> &'static str {
        "RSpec/VariableDefinition"
    }

    fn default_severity(&self) -> Severity {
        Severity::Convention
    }

    fn default_include(&self) -> &'static [&'static str] {
        RSPEC_DEFAULT_INCLUDE
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            CALL_NODE,
            INTERPOLATED_SYMBOL_NODE,
            KEYWORD_HASH_NODE,
            STRING_NODE,
            SYMBOL_NODE,
        ]
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Config: EnforcedStyle — "symbols" (default) or "strings"
        let enforced_style = config.get_str("EnforcedStyle", "symbols");
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        if call.receiver().is_some() {
            return;
        }

        let method_name = call.name().as_slice();
        if method_name != b"let"
            && method_name != b"let!"
            && method_name != b"subject"
            && method_name != b"subject!"
        {
            return;
        }

        let args = match call.arguments() {
            Some(a) => a,
            None => return,
        };

        for arg in args.arguments().iter() {
            if arg.as_keyword_hash_node().is_some() {
                continue;
            }
            let is_offense = if enforced_style == "strings" {
                // "strings" style: flag any symbol (sym or dsym), prefer strings
                arg.as_symbol_node().is_some() || arg.as_interpolated_symbol_node().is_some()
            } else {
                // Default "symbols" style: flag string names, prefer symbols
                // Note: RuboCop's str_type? matches only plain str, not dstr
                arg.as_string_node().is_some()
            };
            if is_offense {
                let loc = arg.location();
                let (line, column) = source.offset_to_line_col(loc.start_offset());
                let msg = if enforced_style == "strings" {
                    "Use strings for variable names."
                } else {
                    "Use symbols for variable names."
                };
                diagnostics.push(self.diagnostic(source, line, column, msg.to_string()));
            }
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(VariableDefinition, "cops/rspec/variable_definition");

    #[test]
    fn strings_style_flags_symbol_names() {
        use crate::cop::CopConfig;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([(
                "EnforcedStyle".into(),
                serde_yml::Value::String("strings".into()),
            )]),
            ..CopConfig::default()
        };
        let source = b"let(:foo) { 'bar' }\n";
        let diags = crate::testutil::run_cop_full_with_config(&VariableDefinition, source, config);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("strings"));
    }

    #[test]
    fn strings_style_does_not_flag_string_names() {
        use crate::cop::CopConfig;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([(
                "EnforcedStyle".into(),
                serde_yml::Value::String("strings".into()),
            )]),
            ..CopConfig::default()
        };
        let source = b"let('foo') { 'bar' }\n";
        let diags = crate::testutil::run_cop_full_with_config(&VariableDefinition, source, config);
        assert!(diags.is_empty());
    }

    #[test]
    fn strings_style_flags_interpolated_symbol() {
        use crate::cop::CopConfig;
        use std::collections::HashMap;

        let config = CopConfig {
            options: HashMap::from([(
                "EnforcedStyle".into(),
                serde_yml::Value::String("strings".into()),
            )]),
            ..CopConfig::default()
        };
        let source = br#"let(:"foo_#{x}") { 'bar' }
"#;
        let diags = crate::testutil::run_cop_full_with_config(&VariableDefinition, source, config);
        assert_eq!(diags.len(), 1, "should flag dsym when style is strings");
        assert!(diags[0].message.contains("strings"));
    }

    #[test]
    fn subject_without_block_is_flagged() {
        // RuboCop flags bare `subject 'text'` calls even without a block,
        // as long as the method name matches a known RSpec helper.
        let source = b"subject 'testing'\n";
        let diags = crate::testutil::run_cop_full(&VariableDefinition, source);
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("symbols"));
    }
}
