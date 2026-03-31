use crate::cop::node_type::FOR_NODE;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

pub struct ForCop;

impl Cop for ForCop {
    fn name(&self) -> &'static str {
        "Style/For"
    }

    fn supports_autocorrect(&self) -> bool {
        true
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[FOR_NODE]
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
        let enforced_style = config.get_str("EnforcedStyle", "each");

        if enforced_style != "each" {
            return;
        }

        let for_node = match node.as_for_node() {
            Some(n) => n,
            None => return,
        };

        let loc = for_node.location();
        let (line, column) = source.offset_to_line_col(loc.start_offset());
        let mut diag = self.diagnostic(
            source,
            line,
            column,
            "Prefer `each` over `for`.".to_string(),
        );

        if let Some(corr) = corrections.as_mut() {
            let index = for_node.index();
            let collection = for_node.collection();
            let index_src = source
                .byte_slice(
                    index.location().start_offset(),
                    index.location().end_offset(),
                    "",
                )
                .to_string();
            let raw_collection = source
                .byte_slice(
                    collection.location().start_offset(),
                    collection.location().end_offset(),
                    "",
                )
                .to_string();
            let collection_src = if collection.as_range_node().is_some() {
                format!("({raw_collection})")
            } else {
                raw_collection
            };

            let body_src = for_node
                .statements()
                .map(|s| {
                    let raw = source
                        .byte_slice(s.location().start_offset(), s.location().end_offset(), "")
                        .to_string();
                    raw.lines()
                        .map(|line| {
                            if line.is_empty() {
                                String::new()
                            } else {
                                format!("  {line}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                })
                .unwrap_or_default();

            let replacement = if body_src.is_empty() {
                format!("{collection_src}.each do |{index_src}|\nend")
            } else {
                format!("{collection_src}.each do |{index_src}|\n{body_src}\nend")
            };

            corr.push(crate::correction::Correction {
                start: loc.start_offset(),
                end: loc.end_offset(),
                replacement,
                cop_name: self.name(),
                cop_index: 0,
            });
            diag.corrected = true;
        }

        diagnostics.push(diag);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(ForCop, "cops/style/for_cop");
    crate::cop_autocorrect_fixture_tests!(ForCop, "cops/style/for_cop");
}
