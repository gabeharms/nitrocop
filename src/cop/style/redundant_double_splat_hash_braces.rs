use crate::cop::node_type::{ASSOC_NODE, ASSOC_SPLAT_NODE, HASH_NODE, KEYWORD_HASH_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct RedundantDoubleSplatHashBraces;

impl Cop for RedundantDoubleSplatHashBraces {
    fn name(&self) -> &'static str {
        "Style/RedundantDoubleSplatHashBraces"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[ASSOC_NODE, ASSOC_SPLAT_NODE, HASH_NODE, KEYWORD_HASH_NODE]
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn check_node(
        &self,
        source: &SourceFile,
        node: &ruby_prism::Node<'_>,
        _parse_result: &ruby_prism::ParseResult<'_>,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        mut corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // Look for **{key: val, ...} in keyword arguments (KeywordHashNode in method calls)
        // Only check KeywordHashNode (method call keyword args), not plain HashNode
        let keyword_hash = match node.as_keyword_hash_node() {
            Some(kh) => kh,
            None => return,
        };

        self.check_hash_elements(
            source,
            keyword_hash.elements().iter(),
            diagnostics,
            corrections.as_deref_mut(),
        );
    }
}

impl RedundantDoubleSplatHashBraces {
    /// Check if any element in a hash uses hash rocket (=>) syntax.
    /// Non-symbol keys can't be keyword arguments, so we skip those.
    fn has_hash_rocket(hash: &ruby_prism::HashNode<'_>) -> bool {
        hash.elements().iter().any(|e| {
            if let Some(assoc) = e.as_assoc_node() {
                assoc.operator_loc().is_some() && assoc.operator_loc().unwrap().as_slice() == b"=>"
            } else {
                false
            }
        })
    }

    fn check_hash_elements<'a, I>(
        &self,
        source: &SourceFile,
        elements: I,
        diagnostics: &mut Vec<Diagnostic>,
        corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) where
        I: Iterator<Item = ruby_prism::Node<'a>>,
    {
        let mut corrections = corrections;

        for element in elements {
            if let Some(splat) = element.as_assoc_splat_node() {
                // Check if the splatted value is a hash literal with elements
                if let Some(value) = splat.value() {
                    if let Some(hash) = value.as_hash_node() {
                        // Don't flag empty hashes: **{}
                        if hash.elements().iter().next().is_none() {
                            continue;
                        }
                        if Self::has_hash_rocket(&hash) {
                            continue;
                        }
                        let loc = element.location();
                        let (line, column) = source.offset_to_line_col(loc.start_offset());
                        let mut diag = self.diagnostic(
                            source,
                            line,
                            column,
                            "Remove the redundant double splat and braces, use keyword arguments directly.".to_string(),
                        );

                        if let Some(ref mut corr) = corrections {
                            let first = hash.elements().iter().next();
                            let last = hash.elements().iter().last();
                            if let (Some(first), Some(last)) = (first, last) {
                                let start = first.location().start_offset();
                                let end = last.location().end_offset();
                                if let Ok(replacement) =
                                    std::str::from_utf8(&source.as_bytes()[start..end])
                                {
                                    corr.push(crate::correction::Correction {
                                        start: loc.start_offset(),
                                        end: loc.end_offset(),
                                        replacement: replacement.to_string(),
                                        cop_name: self.name(),
                                        cop_index: 0,
                                    });
                                    diag.corrected = true;
                                }
                            }
                        }

                        diagnostics.push(diag);
                    } else if let Some(call) = value.as_call_node() {
                        // Detect **{foo: bar, baz: qux}.merge(options)
                        // The double splat on a hash literal merged with another value
                        // can be rewritten as inline keyword args + splatted merge args.
                        if call.name().as_slice() != b"merge" {
                            continue;
                        }
                        let receiver = match call.receiver() {
                            Some(r) => r,
                            None => continue,
                        };
                        let hash = match receiver.as_hash_node() {
                            Some(h) => h,
                            None => continue,
                        };
                        if hash.elements().iter().next().is_none() {
                            continue;
                        }
                        if Self::has_hash_rocket(&hash) {
                            continue;
                        }
                        let merge_args = match call.arguments() {
                            Some(a) => a,
                            None => continue,
                        };
                        let arg_list: Vec<_> = merge_args.arguments().iter().collect();
                        if arg_list.is_empty() {
                            continue;
                        }

                        let loc = element.location();
                        let (line, column) = source.offset_to_line_col(loc.start_offset());
                        let mut diag = self.diagnostic(
                            source,
                            line,
                            column,
                            "Remove the redundant double splat and braces, use keyword arguments directly.".to_string(),
                        );

                        if let Some(ref mut corr) = corrections {
                            // Build replacement: inline hash pairs, then **each_merge_arg
                            let first = hash.elements().iter().next();
                            let last = hash.elements().iter().last();
                            if let (Some(first), Some(last)) = (first, last) {
                                let start = first.location().start_offset();
                                let end = last.location().end_offset();
                                if let Ok(hash_pairs) =
                                    std::str::from_utf8(&source.as_bytes()[start..end])
                                {
                                    let mut replacement = hash_pairs.to_string();
                                    for arg in &arg_list {
                                        let arg_start = arg.location().start_offset();
                                        let arg_end = arg.location().end_offset();
                                        if let Ok(arg_text) = std::str::from_utf8(
                                            &source.as_bytes()[arg_start..arg_end],
                                        ) {
                                            replacement.push_str(", **");
                                            replacement.push_str(arg_text);
                                        }
                                    }
                                    corr.push(crate::correction::Correction {
                                        start: loc.start_offset(),
                                        end: loc.end_offset(),
                                        replacement,
                                        cop_name: self.name(),
                                        cop_index: 0,
                                    });
                                    diag.corrected = true;
                                }
                            }
                        }

                        diagnostics.push(diag);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        RedundantDoubleSplatHashBraces,
        "cops/style/redundant_double_splat_hash_braces"
    );
    crate::cop_autocorrect_fixture_tests!(
        RedundantDoubleSplatHashBraces,
        "cops/style/redundant_double_splat_hash_braces"
    );
}
