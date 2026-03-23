use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;
use ruby_prism::Visit;

pub struct FileNull;

impl Cop for FileNull {
    fn name(&self) -> &'static str {
        "Style/FileNull"
    }

    fn check_source(
        &self,
        source: &SourceFile,
        parse_result: &ruby_prism::ParseResult<'_>,
        _code_map: &crate::cop::CodeMap,
        _config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        // First pass: check if the file contains any "/dev/null" string
        // (needed for bare "NUL" detection)
        let root = parse_result.node();
        let mut dev_null_finder = DevNullFinder { found: false };
        dev_null_finder.visit(&root);
        let contain_dev_null = dev_null_finder.found;

        // Second pass: find offenses
        let mut visitor = FileNullVisitor {
            source,
            cop: self,
            diagnostics: Vec::new(),
            contain_dev_null,
            skip_offsets: std::collections::HashSet::new(),
        };
        visitor.visit(&root);
        diagnostics.extend(visitor.diagnostics);
    }
}

struct DevNullFinder {
    found: bool,
}

impl<'pr> Visit<'pr> for DevNullFinder {
    fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
        let content = node.unescaped();
        if let Ok(s) = std::str::from_utf8(content) {
            if s.eq_ignore_ascii_case("/dev/null") {
                self.found = true;
            }
        }
    }
}

struct FileNullVisitor<'a> {
    source: &'a SourceFile,
    cop: &'a FileNull,
    diagnostics: Vec<Diagnostic>,
    contain_dev_null: bool,
    /// Start offsets of string nodes that are direct children of array elements
    /// or hash pair values — these should be skipped (RuboCop skips strings
    /// whose immediate parent is :array or :pair).
    skip_offsets: std::collections::HashSet<usize>,
}

impl<'a, 'pr> Visit<'pr> for FileNullVisitor<'a> {
    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode<'pr>) {
        // Mark direct string children for skipping, but still recurse into
        // non-string elements (and into string children's siblings).
        for element in node.elements().iter() {
            if element.as_string_node().is_some() {
                self.skip_offsets.insert(element.location().start_offset());
            }
        }
        ruby_prism::visit_array_node(self, node);
    }

    fn visit_assoc_node(&mut self, node: &ruby_prism::AssocNode<'pr>) {
        // Mark the value if it's a direct string node.
        // The key is also checked but typically is a symbol.
        if node.value().as_string_node().is_some() {
            self.skip_offsets
                .insert(node.value().location().start_offset());
        }
        ruby_prism::visit_assoc_node(self, node);
    }

    fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
        // Skip strings that are direct children of arrays or hash pairs
        if self.skip_offsets.contains(&node.location().start_offset()) {
            return;
        }

        let content_bytes = node.unescaped();
        let content_str = match std::str::from_utf8(content_bytes) {
            Ok(s) => s,
            Err(_) => return,
        };

        if content_str.is_empty() {
            return;
        }

        let lower = content_str.to_lowercase();

        let matched =
            if lower == "/dev/null" || lower == "nul:" || (lower == "nul" && self.contain_dev_null)
            {
                Some(content_str)
            } else {
                None
            };

        if let Some(matched_str) = matched {
            let loc = node.location();
            let (line, column) = self.source.offset_to_line_col(loc.start_offset());
            self.diagnostics.push(self.cop.diagnostic(
                self.source,
                line,
                column,
                format!("Use `File::NULL` instead of `{}`.", matched_str),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::cop_fixture_tests!(FileNull, "cops/style/file_null");
}
