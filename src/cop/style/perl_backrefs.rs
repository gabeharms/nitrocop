use crate::cop::node_type::{
    BACK_REFERENCE_READ_NODE, GLOBAL_VARIABLE_READ_NODE, NUMBERED_REFERENCE_READ_NODE,
};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// Style/PerlBackrefs detects Perl-style regexp backreferences ($1, $&, etc.)
/// and their English-name equivalents ($MATCH, $PREMATCH, $POSTMATCH,
/// $LAST_PAREN_MATCH) from Ruby's `English` library. Both forms should be
/// replaced with `Regexp.last_match(...)` calls.
///
/// The English-name globals are parsed as `GlobalVariableReadNode` by Prism,
/// not `BackReferenceReadNode`, so we must also handle that node type.
pub struct PerlBackrefs;

impl Cop for PerlBackrefs {
    fn name(&self) -> &'static str {
        "Style/PerlBackrefs"
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[
            BACK_REFERENCE_READ_NODE,
            NUMBERED_REFERENCE_READ_NODE,
            GLOBAL_VARIABLE_READ_NODE,
        ]
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
        // Check for numbered backreferences: $1, $2, ..., $9
        if let Some(back_ref) = node.as_numbered_reference_read_node() {
            let num = back_ref.number();
            let loc = node.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Prefer `Regexp.last_match({num})` over `${num}`."),
            ));
        }

        // Check for special backreferences: $&, $`, $', $+
        if let Some(back_ref) = node.as_back_reference_read_node() {
            let name_slice = back_ref.name().as_slice();
            let loc = node.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());

            let (replacement, var_display) = match name_slice {
                b"$&" => ("Regexp.last_match(0)", "$&"),
                b"$`" => ("Regexp.last_match.pre_match", "$`"),
                b"$'" => ("Regexp.last_match.post_match", "$'"),
                b"$+" => ("Regexp.last_match(-1)", "$+"),
                _ => return,
            };

            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Prefer `{replacement}` over `{var_display}`."),
            ));
        }

        // Check for English-name global variable equivalents: $MATCH, $PREMATCH,
        // $POSTMATCH, $LAST_PAREN_MATCH (from `require 'English'`)
        if let Some(gvar) = node.as_global_variable_read_node() {
            let name_slice = gvar.name().as_slice();
            let (replacement, var_display) = match name_slice {
                b"$MATCH" => ("Regexp.last_match(0)", "$MATCH"),
                b"$PREMATCH" => ("Regexp.last_match.pre_match", "$PREMATCH"),
                b"$POSTMATCH" => ("Regexp.last_match.post_match", "$POSTMATCH"),
                b"$LAST_PAREN_MATCH" => ("Regexp.last_match(-1)", "$LAST_PAREN_MATCH"),
                _ => return,
            };

            let loc = node.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                format!("Prefer `{replacement}` over `{var_display}`."),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(PerlBackrefs, "cops/style/perl_backrefs");
}
