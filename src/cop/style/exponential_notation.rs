use crate::cop::node_type::FLOAT_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ExponentialNotation;

impl Cop for ExponentialNotation {
    fn name(&self) -> &'static str {
        "Style/ExponentialNotation"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[FLOAT_NODE]
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
        let float_node = match node.as_float_node() {
            Some(f) => f,
            None => return,
        };

        let loc = float_node.location();
        let src_bytes = loc.as_slice();
        let src_str = match std::str::from_utf8(src_bytes) {
            Ok(s) => s,
            Err(_) => return,
        };

        // Only care about exponential notation (contains 'e' or 'E')
        let lower = src_str.to_lowercase();
        if !lower.contains('e') {
            return;
        }

        // The parser gem folds a leading unary `+` into the float literal's
        // source (e.g. `+2.5e20` → float source "+2.5e20").  Prism does the
        // same: the FloatNode location includes the `+`.  RuboCop's regex
        // `/^-?[1-9]…/` does not allow a leading `+`, so any `+`-prefixed
        // float in exponential notation is always an offense.
        let has_leading_plus = lower.starts_with('+');

        // Strip leading sign for mantissa analysis
        let working = lower
            .strip_prefix('-')
            .or_else(|| lower.strip_prefix('+'))
            .unwrap_or(&lower);

        let parts: Vec<&str> = working.splitn(2, 'e').collect();
        if parts.len() != 2 {
            return;
        }

        let mantissa_str = parts[0].replace('_', "");
        let exponent_str = parts[1].replace('_', "");

        let mantissa: f64 = match mantissa_str.parse() {
            Ok(v) => v,
            Err(_) => return,
        };

        let exponent: i64 = match exponent_str.parse() {
            Ok(v) => v,
            Err(_) => return,
        };

        let style = config.get_str("EnforcedStyle", "scientific");

        let (line, column) = source.offset_to_line_col(loc.start_offset());

        match style {
            "scientific" => {
                // Mantissa must be >= 1 and < 10.
                // A leading `+` is never valid (RuboCop's regex `/^-?…/`
                // rejects it), so always flag `+`-prefixed floats.
                let abs_mantissa = mantissa.abs();
                if has_leading_plus || !(1.0..10.0).contains(&abs_mantissa) {
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use a mantissa >= 1 and < 10.".to_string(),
                    ));
                }
            }
            "engineering" => {
                // Exponent must be divisible by 3, mantissa >= 0.1 and < 1000.
                // A leading `+` is never valid per RuboCop's regex.
                let abs_mantissa = mantissa.abs();
                if has_leading_plus || exponent % 3 != 0 || !(0.1..1000.0).contains(&abs_mantissa) {
                    diagnostics.push(
                        self.diagnostic(
                            source,
                            line,
                            column,
                            "Use an exponent divisible by 3 and a mantissa >= 0.1 and < 1000."
                                .to_string(),
                        ),
                    );
                }
            }
            "integral" => {
                // Mantissa must be an integer without trailing zeros.
                // A leading `+` is never valid per RuboCop's regex.
                if has_leading_plus {
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use an integer as mantissa, without trailing zero.".to_string(),
                    ));
                    return;
                }
                let has_decimal = mantissa_str.contains('.');
                let mantissa_int: i64 = if has_decimal {
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use an integer as mantissa, without trailing zero.".to_string(),
                    ));
                    return;
                } else {
                    match mantissa_str.parse() {
                        Ok(v) => v,
                        Err(_) => return,
                    }
                };
                if mantissa_int != 0 && mantissa_int % 10 == 0 {
                    diagnostics.push(self.diagnostic(
                        source,
                        line,
                        column,
                        "Use an integer as mantissa, without trailing zero.".to_string(),
                    ));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ExponentialNotation, "cops/style/exponential_notation");
}
