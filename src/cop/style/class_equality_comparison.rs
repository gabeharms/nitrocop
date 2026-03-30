use crate::cop::{Cop, CopConfig};
use crate::correction::Correction;
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

/// FP investigation (2026-03-10):
/// - Root cause 1: Missing dstr_type check. RuboCop has `return if class_node.dstr_type?`
///   to skip when the RHS of the comparison is an interpolated string (e.g.,
///   `x.class.name == "#{some_class}"`). In Prism this corresponds to
///   `InterpolatedStringNode`. Fixed by checking `arguments[0].as_interpolated_string_node()`.
/// - Root cause 2: AllowedPatterns was checked against the source line text instead of
///   the enclosing def method name. RuboCop checks `matches_allowed_pattern?(def_node.method_name)`.
///   Fixed by matching AllowedPatterns against `enclosing_def_name` like AllowedMethods.
///
/// ## Corpus investigation (2026-03-12)
///
/// Corpus oracle reported FP=3, FN=0. All 3 FPs used `&.class` (safe navigation
/// on the `.class` call itself). RuboCop skips these because `instance_of?` doesn't
/// preserve nil-safety semantics.
///
/// First attempt (reverted): checked all call operators for `&.`, which was too
/// broad and dropped 3 true positives alongside the 3 FPs.
///
/// Fix: only skip when the `.class` call itself uses `&.` — not when intermediate
/// calls like `.name` or `.to_s` use `&.`. This preserves true positives like
/// `foo.class&.name == "Bar"` while correctly skipping `foo&.class == Bar`.
pub struct ClassEqualityComparison;

impl Cop for ClassEqualityComparison {
    fn name(&self) -> &'static str {
        "Style/ClassEqualityComparison"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::cop::CodeMap,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let allowed_methods: Vec<String> = config
            .get_string_array("AllowedMethods")
            .unwrap_or_else(|| vec!["==".to_string(), "equal?".to_string(), "eql?".to_string()]);
        let allowed_patterns: Vec<regex::Regex> = config
            .get_string_array("AllowedPatterns")
            .unwrap_or_default()
            .iter()
            .filter_map(|p| regex::Regex::new(p).ok())
            .collect();

        let mut visitor = ClassEqVisitor {
            cop: self,
            source,
            diagnostics: Vec::new(),
            corrections: Vec::new(),
            autocorrect_enabled: corrections.is_some(),
            allowed_methods,
            allowed_patterns,
            enclosing_def_name: None,
        };
        visitor.visit(&parse_result.node());
        diagnostics.extend(visitor.diagnostics);
        if let Some(corr) = corrections.as_mut() {
            corr.extend(visitor.corrections);
        }
    }
}

struct ClassEqVisitor<'a> {
    cop: &'a ClassEqualityComparison,
    source: &'a SourceFile,
    diagnostics: Vec<Diagnostic>,
    corrections: Vec<Correction>,
    autocorrect_enabled: bool,
    allowed_methods: Vec<String>,
    allowed_patterns: Vec<regex::Regex>,
    enclosing_def_name: Option<Vec<u8>>,
}

fn node_source(source: &SourceFile, node: &ruby_prism::Node<'_>) -> String {
    String::from_utf8_lossy(
        &source.as_bytes()[node.location().start_offset()..node.location().end_offset()],
    )
    .to_string()
}

fn is_constant_path(name: &str) -> bool {
    let candidate = name.strip_prefix("::").unwrap_or(name);
    if candidate.is_empty() {
        return false;
    }
    candidate.split("::").all(|seg| {
        let mut chars = seg.chars();
        let Some(first) = chars.next() else {
            return false;
        };
        first.is_ascii_uppercase() && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    })
}

fn build_instance_of_replacement(
    source: &SourceFile,
    compare_call: &ruby_prism::CallNode<'_>,
    recv_call: &ruby_prism::CallNode<'_>,
    is_class_name_call: bool,
) -> Option<String> {
    let receiver_node = if is_class_name_call {
        recv_call.receiver()?.as_call_node()?.receiver()?
    } else {
        recv_call.receiver()?
    };

    let rhs = compare_call.arguments()?.arguments().iter().next()?;
    let class_arg = if is_class_name_call {
        if let Some(str_node) = rhs.as_string_node() {
            let name = String::from_utf8_lossy(str_node.unescaped()).to_string();
            if !is_constant_path(&name) {
                return None;
            }
            name
        } else if rhs.as_constant_read_node().is_some() || rhs.as_constant_path_node().is_some() {
            node_source(source, &rhs)
        } else {
            return None;
        }
    } else {
        node_source(source, &rhs)
    };

    Some(format!(
        "{}.instance_of?({})",
        node_source(source, &receiver_node),
        class_arg
    ))
}

impl<'a, 'pr> Visit<'pr> for ClassEqVisitor<'a> {
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        let prev = self.enclosing_def_name.take();
        self.enclosing_def_name = Some(node.name().as_slice().to_vec());
        ruby_prism::visit_def_node(self, node);
        self.enclosing_def_name = prev;
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        let method_bytes = node.name().as_slice();

        // Must be ==, equal?, or eql?
        if method_bytes == b"==" || method_bytes == b"equal?" || method_bytes == b"eql?" {
            // Check if we're inside an allowed method
            if let Some(ref def_name) = self.enclosing_def_name {
                let def_str = std::str::from_utf8(def_name).unwrap_or("");
                if self.allowed_methods.iter().any(|m| m == def_str) {
                    // Inside an allowed method, skip
                    ruby_prism::visit_call_node(self, node);
                    return;
                }
            }

            // Receiver must be a `.class` call or `.class.name` call
            if let Some(receiver) = node.receiver() {
                if let Some(recv_call) = receiver.as_call_node() {
                    // Helper: check if a call node uses safe navigation (&.)
                    let is_safe_nav = |c: &ruby_prism::CallNode<'_>| {
                        c.call_operator_loc()
                            .is_some_and(|op| op.as_slice() == b"&.")
                    };

                    let is_class_call = recv_call.name().as_slice() == b"class";
                    let is_class_name_call = if !is_class_call {
                        let name = recv_call.name().as_slice();
                        if name == b"name" || name == b"to_s" || name == b"inspect" {
                            recv_call
                                .receiver()
                                .and_then(|ir| ir.as_call_node())
                                .is_some_and(|ic| ic.name().as_slice() == b"class")
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    // Skip if .class uses safe navigation (&.class) — RuboCop
                    // doesn't flag these since instance_of? doesn't preserve
                    // nil-safety semantics.
                    if is_class_call && is_safe_nav(&recv_call) {
                        ruby_prism::visit_call_node(self, node);
                        return;
                    }
                    if is_class_name_call {
                        if let Some(class_node) =
                            recv_call.receiver().and_then(|ir| ir.as_call_node())
                        {
                            if is_safe_nav(&class_node) {
                                ruby_prism::visit_call_node(self, node);
                                return;
                            }
                        }
                    }

                    if is_class_call || is_class_name_call {
                        // Check AllowedPatterns against the enclosing def name (like RuboCop)
                        if !self.allowed_patterns.is_empty() {
                            if let Some(ref def_name) = self.enclosing_def_name {
                                let def_str = std::str::from_utf8(def_name).unwrap_or("");
                                if self.allowed_patterns.iter().any(|p| p.is_match(def_str)) {
                                    ruby_prism::visit_call_node(self, node);
                                    return;
                                }
                            }
                        }

                        // Get the RHS argument (the class_node in RuboCop terms)
                        if let Some(args) = node.arguments() {
                            if let Some(rhs) = args.arguments().iter().next() {
                                // Skip if the RHS is an interpolated string (dstr_type)
                                if rhs.as_interpolated_string_node().is_some() {
                                    ruby_prism::visit_call_node(self, node);
                                    return;
                                }
                            }
                        }

                        let loc = recv_call
                            .message_loc()
                            .unwrap_or_else(|| recv_call.location());
                        let (line, column) = self.source.offset_to_line_col(loc.start_offset());
                        let mut diag = self.cop.diagnostic(
                            self.source,
                            line,
                            column,
                            "Use `instance_of?` instead of comparing classes.".to_string(),
                        );

                        if self.autocorrect_enabled {
                            if let Some(replacement) = build_instance_of_replacement(
                                self.source,
                                node,
                                &recv_call,
                                is_class_name_call,
                            ) {
                                self.corrections.push(Correction {
                                    start: node.location().start_offset(),
                                    end: node.location().end_offset(),
                                    replacement,
                                    cop_name: self.cop.name(),
                                    cop_index: 0,
                                });
                                diag.corrected = true;
                            }
                        }

                        self.diagnostics.push(diag);
                    }
                }
            }
        }

        ruby_prism::visit_call_node(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        ClassEqualityComparison,
        "cops/style/class_equality_comparison"
    );
    crate::cop_autocorrect_fixture_tests!(
        ClassEqualityComparison,
        "cops/style/class_equality_comparison"
    );
}
