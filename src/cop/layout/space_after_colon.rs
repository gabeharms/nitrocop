use crate::cop::node_type::{ASSOC_NODE, IMPLICIT_NODE, SYMBOL_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-10)
///
/// CI baseline reported FP=0, FN=33.
///
/// Attempted fix: flag `OptionalKeywordParameterNode` so RubyMotion-style
/// parameter syntax like `name:value` matched RuboCop's `on_kwoptarg`
/// handling. That removed the sampled FN, but the corpus rerun regressed to
/// expected=510, actual=568, CI baseline=477, raw excess=58, file-drop
/// noise=73, which still left 18 excess beyond the CI baseline.
///
/// The new false positives concentrated outside the RubyMotion repo in
/// `cerebris__jsonapi-resources__e92afc6`, `openjournals__joss__c3cc59f`, and
/// `browsermedia__browsercms__0a7fb92`, so the broader kwoptarg hook was
/// reverted. A correct fix needs to distinguish the RubyMotion selector-style
/// parameter form from ordinary Prism optional-keyword parameters that
/// RuboCop does not count here.
pub struct SpaceAfterColon;

impl Cop for SpaceAfterColon {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterColon"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[ASSOC_NODE, IMPLICIT_NODE, SYMBOL_NODE]
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
        let assoc = match node.as_assoc_node() {
            Some(a) => a,
            None => return,
        };

        // Skip value-omission shorthand hash syntax (Ruby 3.1+): { url:, driver: }
        // In Prism, when value is omitted, the value node is an ImplicitNode.
        if assoc.value().as_implicit_node().is_some() {
            return;
        }

        let key = assoc.key();
        let sym = match key.as_symbol_node() {
            Some(s) => s,
            None => return,
        };

        let colon_loc = match sym.closing_loc() {
            Some(loc) if loc.as_slice() == b":" => loc,
            _ => return,
        };

        let bytes = source.as_bytes();
        let after_colon = colon_loc.end_offset();
        // RuboCop accepts any whitespace after colon (space, newline, tab)
        match bytes.get(after_colon) {
            Some(b) if b.is_ascii_whitespace() => {}
            _ => {
                let (line, column) = source.offset_to_line_col(colon_loc.start_offset());
                let mut diag = self.diagnostic(
                    source,
                    line,
                    column,
                    "Space missing after colon.".to_string(),
                );
                if let Some(ref mut corr) = corrections {
                    corr.push(crate::correction::Correction {
                        start: after_colon,
                        end: after_colon,
                        replacement: " ".to_string(),
                        cop_name: self.name(),
                        cop_index: 0,
                    });
                    diag.corrected = true;
                }
                diagnostics.push(diag);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    crate::cop_fixture_tests!(SpaceAfterColon, "cops/layout/space_after_colon");
    crate::cop_autocorrect_fixture_tests!(SpaceAfterColon, "cops/layout/space_after_colon");
}
