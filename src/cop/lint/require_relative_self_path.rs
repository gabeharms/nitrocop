use crate::cop::node_type::{CALL_NODE, STRING_NODE};
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::{Diagnostic, Severity};
use crate::parse::source::SourceFile;
use std::path::Path;

/// Detects `require_relative` calls that require the file itself.
///
/// ## FP fix (2026-03-24): non-.rb file extensions
/// Ruby's `require_relative` only appends `.rb` when no extension is given.
/// So `require_relative './bin'` in `bin.rake` loads `bin.rb`, not `bin.rake`.
/// The fix skips flagging when the current file is not `.rb` and the required
/// path has no explicit extension — those cannot be self-requires.
/// This resolved 11 FPs (6 from wxRuby3 .rake files, 1 from jubilee .ru, etc.).
pub struct RequireRelativeSelfPath;

impl Cop for RequireRelativeSelfPath {
    fn name(&self) -> &'static str {
        "Lint/RequireRelativeSelfPath"
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn interested_node_types(&self) -> &'static [u8] {
        &[CALL_NODE, STRING_NODE]
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
        // Look for `require_relative 'self_filename'`
        let call = match node.as_call_node() {
            Some(c) => c,
            None => return,
        };

        if call.name().as_slice() != b"require_relative" {
            return;
        }

        // Must have no receiver
        if call.receiver().is_some() {
            return;
        }

        let arguments = match call.arguments() {
            Some(a) => a,
            None => return,
        };

        let args = arguments.arguments();
        if args.len() != 1 {
            return;
        }

        let first_arg = args.iter().next().unwrap();
        let string_node = match first_arg.as_string_node() {
            Some(s) => s,
            None => return,
        };

        let required_path = string_node.unescaped();
        let required_str = match std::str::from_utf8(required_path) {
            Ok(s) => s,
            Err(_) => return,
        };

        // Get the current file's basename without extension
        let file_path = Path::new(source.path_str());
        let file_stem = match file_path.file_stem() {
            Some(s) => s.to_str().unwrap_or(""),
            None => return,
        };

        // The required path's filename (last component)
        let required_path_obj = Path::new(required_str);
        let required_stem = match required_path_obj.file_stem() {
            Some(s) => s.to_str().unwrap_or(""),
            None => return,
        };

        // Check if the extension (if any) is `.rb` or absent
        let required_ext = required_path_obj
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if !required_ext.is_empty() && required_ext != "rb" {
            return;
        }

        // Ruby's require_relative only appends `.rb` when the path has no extension.
        // If the current file is not `.rb` (e.g., `.rake`, `.ru`), then
        // `require_relative 'same_name'` resolves to `same_name.rb`, not the
        // current file — so it is NOT a self-require.
        // Only flag non-.rb files if the required path has an explicit `.rb` extension
        // that happens to match the full filename (e.g., `foo.rake` requiring `foo.rake`
        // — though this is extremely unlikely in practice).
        let file_ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if file_ext != "rb" && required_ext.is_empty() {
            return;
        }

        // Check if it's requiring itself (same directory, same name)
        // Only flag if the required path has no directory component or its directory
        // resolves to the same file
        let required_parent = required_path_obj.parent();
        let is_same_dir = match required_parent {
            None => true,
            Some(p) => p.as_os_str().is_empty() || p.as_os_str() == ".",
        };

        if is_same_dir && required_stem == file_stem {
            let loc = call.location();
            let (line, column) = source.offset_to_line_col(loc.start_offset());
            diagnostics.push(self.diagnostic(
                source,
                line,
                column,
                "Remove the `require_relative` that requires itself.".to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(
        RequireRelativeSelfPath,
        "cops/lint/require_relative_self_path"
    );

    #[test]
    fn no_offense_rake_file_same_basename() {
        // .rake file requiring same basename — resolves to .rb, not self
        let source = b"require_relative './bin'\nrequire_relative 'bin'\n";
        let diags = crate::testutil::run_cop_full_internal(
            &RequireRelativeSelfPath,
            source,
            crate::cop::CopConfig::default(),
            "rakelib/bin.rake",
        );
        assert!(
            diags.is_empty(),
            "Expected no offenses for .rake file but got: {:?}",
            diags
        );
    }

    #[test]
    fn no_offense_ru_file_same_basename() {
        // .ru file requiring same basename — resolves to .rb, not self
        let source = b"require_relative './persistent'\n";
        let diags = crate::testutil::run_cop_full_internal(
            &RequireRelativeSelfPath,
            source,
            crate::cop::CopConfig::default(),
            "test/apps/persistent.ru",
        );
        assert!(
            diags.is_empty(),
            "Expected no offenses for .ru file but got: {:?}",
            diags
        );
    }

    #[test]
    fn offense_rb_file_still_detected() {
        // .rb file requiring same basename — IS a self-require
        let source = b"require_relative './foo'\n";
        let diags = crate::testutil::run_cop_full_internal(
            &RequireRelativeSelfPath,
            source,
            crate::cop::CopConfig::default(),
            "lib/foo.rb",
        );
        assert_eq!(diags.len(), 1, "Expected 1 offense for .rb self-require");
    }
}
