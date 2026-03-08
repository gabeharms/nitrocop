use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Checks for ambiguous endless method definitions that use low-precedence operators.
///
/// ## Corpus conformance fix (2026-03-08)
/// **Root cause:** RuboCop requires `minimum_target_ruby_version 3.0` — endless methods
/// were introduced in Ruby 3.0, so this cop only fires for repos targeting Ruby >= 3.0.
/// The nitrocop implementation was missing this version check, causing 13 false positives
/// across 10 repos (rbs, newrelic, yard, nats.rb, blueprinter, puppet, rack, rails,
/// ferrum, stripe-mock) that target Ruby < 3.0.
///
/// **Fix:** Added TargetRubyVersion >= 3.0 check at the start of `check_lines`,
/// matching the pattern used by `Style/ItBlockParameter` for its 3.4 threshold.
pub struct AmbiguousEndlessMethodDefinition;

impl Cop for AmbiguousEndlessMethodDefinition {
    fn name(&self) -> &'static str {
        "Style/AmbiguousEndlessMethodDefinition"
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // RuboCop: minimum_target_ruby_version 3.0
        // Endless methods were introduced in Ruby 3.0
        let ruby_version = config
            .options
            .get("TargetRubyVersion")
            .and_then(|v| v.as_f64().or_else(|| v.as_u64().map(|u| u as f64)))
            .unwrap_or(2.7);
        if ruby_version < 3.0 {
            return;
        }

        let low_precedence_ops = [" and ", " or ", " if ", " unless ", " while ", " until "];

        for (i, line) in source.lines().enumerate() {
            let line_str = match std::str::from_utf8(line) {
                Ok(s) => s.trim_end(),
                Err(_) => continue,
            };

            // Check for endless method definition: `def foo = ...`
            let trimmed = line_str.trim_start();
            if !trimmed.starts_with("def ") {
                continue;
            }

            // Find the `=` that makes it endless
            // Look for `= ` after method name (not `==`)
            let after_def = &trimmed[4..];
            let eq_pos = after_def.find(" = ");
            if eq_pos.is_none() {
                continue;
            }

            let eq_pos = eq_pos.unwrap();
            let after_eq = &after_def[eq_pos + 3..];

            // Check if there's a low-precedence operator in the body
            // that isn't wrapped in parentheses
            for op in &low_precedence_ops {
                if after_eq.contains(op) {
                    // Check it's not inside parentheses
                    let op_pos = after_eq.find(op).unwrap();
                    let before_op = &after_eq[..op_pos];
                    let paren_depth: i32 = before_op
                        .chars()
                        .map(|c| match c {
                            '(' => 1,
                            ')' => -1,
                            _ => 0,
                        })
                        .sum();

                    if paren_depth == 0 {
                        let col = 0;
                        let op_name = op.trim();
                        diagnostics.push(self.diagnostic(
                            source,
                            i + 1,
                            col,
                            format!("Avoid using `{}` statements with endless methods.", op_name),
                        ));
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cop::CopConfig;

    fn ruby30_config() -> CopConfig {
        let mut config = CopConfig::default();
        config.options.insert(
            "TargetRubyVersion".to_string(),
            serde_yml::Value::Number(serde_yml::Number::from(3.0)),
        );
        config
    }

    #[test]
    fn offense_with_ruby30() {
        crate::testutil::assert_cop_offenses_full_with_config(
            &AmbiguousEndlessMethodDefinition,
            include_bytes!(
                "../../../tests/fixtures/cops/style/ambiguous_endless_method_definition/offense.rb"
            ),
            ruby30_config(),
        );
    }

    #[test]
    fn no_offense_with_ruby30() {
        crate::testutil::assert_cop_no_offenses_full_with_config(
            &AmbiguousEndlessMethodDefinition,
            include_bytes!(
                "../../../tests/fixtures/cops/style/ambiguous_endless_method_definition/no_offense.rb"
            ),
            ruby30_config(),
        );
    }

    #[test]
    fn no_offense_below_ruby30() {
        // Default Ruby version (2.7) — cop should be completely silent
        crate::testutil::assert_cop_no_offenses_full(
            &AmbiguousEndlessMethodDefinition,
            include_bytes!(
                "../../../tests/fixtures/cops/style/ambiguous_endless_method_definition/offense.rb"
            ),
        );
    }
}
