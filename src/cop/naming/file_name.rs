use std::path::Path;

use crate::cop::util::is_snake_case;
use crate::cop::{Cop, CopConfig};
use crate::diagnostic::Diagnostic;
use crate::parse::source::SourceFile;

/// ## Corpus investigation (2026-03-09)
///
/// Corpus oracle reported FP=3, FN=17.
///
/// ### FP=3 (all david942j/one_gadget)
/// Files like `libc-2.23-89cc3bb9361ad139a1967462175759416c9dc82b.rb` under
/// `lib/one_gadget/builds/`. The repo's `.rubocop.yml` has
/// `AllCops: Exclude: - lib/one_gadget/builds/*.rb`, so RuboCop never sees them.
/// This is a config-exclude issue, not a cop logic bug — nitrocop's config
/// loader needs to honor the project's AllCops/Exclude for these paths.
///
/// ### FN=17 — two cop logic bugs fixed:
///
/// **Bug 1: AllowedAcronyms incorrectly applied to filename check (10 FNs).**
/// nitrocop was replacing AllowedAcronyms (e.g., URI, HTML, HTTP) in the
/// filename before the snake_case check, causing filenames like `URI_test.rb`
/// and `escapeHTML_spec.rb` to pass. RuboCop's `filename_good?` only uses the
/// `SNAKE_CASE` regex — AllowedAcronyms is for `ExpectMatchingDefinition` only.
/// Fix: removed acronym substitution from the snake_case check.
///
/// **Bug 2: ALLOWED_NAMES matched on file_stem instead of full filename (2 FNs).**
/// `Rakefile.rb` had stem `Rakefile` matching the allow list, but RuboCop's
/// `allowed_camel_case_file?` checks AllCops/Include patterns like `**/Rakefile`
/// which only match the exact filename. `Rakefile.rb` and `Vagrantfile.spec`
/// are different files and should be flagged.
/// Fix: check ALLOWED_NAMES against the full filename (with extension), not stem.
///
/// ### Remaining 5 FNs (not fixable in cop logic):
/// - `iso-8859-9-encoding.rb`, `euc-jp.rb`: contain hyphens which `is_snake_case`
///   already rejects. These are likely config/file-discovery layer issues.
/// - `💪.test.rb`: emoji filename — `is_snake_case` allows all non-ASCII bytes,
///   but RuboCop's SNAKE_CASE only allows `[[:lower:]]`. Fixing this would
///   require changing the shared `is_snake_case` utility, risking regressions
///   in other cops.
pub struct FileName;

/// Well-known Ruby files that don't follow snake_case convention.
/// These correspond to CamelCase entries in AllCops/Include from RuboCop's
/// default.yml — `allowed_camel_case_file?` skips any file matching an Include
/// pattern that contains an uppercase letter.
const ALLOWED_NAMES: &[&str] = &[
    "Appraisals",
    "Berksfile",
    "Brewfile",
    "Buildfile",
    "Capfile",
    "Cheffile",
    "Dangerfile",
    "Deliverfile",
    "Fastfile",
    "Gemfile",
    "Guardfile",
    "Jarfile",
    "Mavenfile",
    "Podfile",
    "Puppetfile",
    "Rakefile",
    "Schemafile",
    "Snapfile",
    "Steepfile",
    "Thorfile",
    "Vagabondfile",
    "Vagrantfile",
];

/// Default roots for definition path hierarchy matching.
const DEFAULT_PATH_ROOTS: &[&str] = &["lib", "spec", "test", "app"];

/// Convert a snake_case filename stem to a CamelCase module name.
fn to_module_name(stem: &str) -> String {
    stem.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Build expected namespace from a file path.
fn build_expected_namespace(path: &Path, roots: &Option<Vec<String>>) -> Vec<String> {
    let root_list: Vec<&str> = match roots {
        Some(list) => list.iter().map(|s| s.as_str()).collect(),
        None => DEFAULT_PATH_ROOTS.to_vec(),
    };

    let components: Vec<&str> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .collect();

    // Find the last occurrence of a root directory in the path
    let mut start_index = None;
    for (i, comp) in components.iter().enumerate().rev() {
        if root_list.contains(comp) {
            start_index = Some(i + 1);
            break;
        }
    }

    match start_index {
        Some(idx) => components[idx..]
            .iter()
            .map(|c| {
                let stem = Path::new(c)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(c);
                to_module_name(stem)
            })
            .collect(),
        None => {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            vec![to_module_name(stem)]
        }
    }
}

/// Simple text-based check for whether the source defines a class/module matching
/// the expected namespace.
fn has_matching_definition(source: &str, expected_namespace: &[String]) -> bool {
    if expected_namespace.is_empty() {
        return true;
    }

    let fqn = expected_namespace.join("::");
    let last_name = expected_namespace.last().unwrap();

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed
            .strip_prefix("class ")
            .or_else(|| trimmed.strip_prefix("module "))
        {
            let def_name = rest
                .split(|c: char| c == '<' || c == '(' || c.is_whitespace())
                .next()
                .unwrap_or("")
                .trim();
            if def_name == fqn || def_name == last_name {
                return true;
            }
        }
    }
    false
}

impl Cop for FileName {
    fn name(&self) -> &'static str {
        "Naming/FileName"
    }

    fn check_lines(
        &self,
        source: &SourceFile,
        config: &CopConfig,
        diagnostics: &mut Vec<Diagnostic>,
        _corrections: Option<&mut Vec<crate::correction::Correction>>,
    ) {
        let expect_matching_definition = config.get_bool("ExpectMatchingDefinition", false);
        let check_def_path_hierarchy = config.get_bool("CheckDefinitionPathHierarchy", true);
        let check_def_path_roots = config.get_string_array("CheckDefinitionPathHierarchyRoots");
        let regex_pattern = config.get_str("Regex", "");
        let ignore_executable_scripts = config.get_bool("IgnoreExecutableScripts", true);
        let _allowed_acronyms = config.get_string_array("AllowedAcronyms");

        let path = Path::new(source.path_str());
        let file_stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => return,
        };

        // Gemspecs are allowed to have dashes (bundler convention for namespaced gems)
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == "gemspec" {
            return;
        }

        // IgnoreExecutableScripts: skip files with shebang (#!) on first line
        if ignore_executable_scripts {
            let bytes = source.as_bytes();
            if bytes.starts_with(b"#!") {
                return;
            }
        }

        // Allow well-known Ruby files — only when the full filename (with extension)
        // exactly matches an allowed name. RuboCop's `allowed_camel_case_file?` checks
        // AllCops/Include patterns like `**/Rakefile` which match the exact filename,
        // NOT `Rakefile.rb` or `Vagrantfile.spec`.
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if ALLOWED_NAMES.contains(&file_name) {
            return;
        }

        // Allow files whose full name (with extension) ends with a known CamelCase name
        // (e.g., ImportFastfile, SwitcherFastfile). This matches RuboCop's
        // `allowed_camel_case_file?` which checks AllCops/Include patterns containing
        // uppercase letters like `**/*Fastfile`.
        if ALLOWED_NAMES
            .iter()
            .any(|name| file_name.len() > name.len() && file_name.ends_with(name))
        {
            return;
        }

        // Regex: if a custom regex is provided, use it instead of snake_case check
        if !regex_pattern.is_empty() {
            if let Ok(re) = regex::Regex::new(regex_pattern) {
                if re.is_match(file_stem) {
                    return;
                }
                diagnostics.push(self.diagnostic(
                    source,
                    1,
                    0,
                    format!(
                        "The name of this source file (`{file_stem}`) should match the configured Regex."
                    ),
                ));
            }
        }

        // RuboCop strips leading dot from dotfiles before checking (e.g., .pryrc -> pryrc)
        // RuboCop replaces + with _ before the snake_case check, to support
        // Action Pack Variants filenames like `some_file.xlsx+mobile.axlsx`.
        // Note: AllowedAcronyms is NOT applied to the filename check — RuboCop's
        // filename_good? only uses the SNAKE_CASE regex without acronym substitution.
        // AllowedAcronyms is only used for ExpectMatchingDefinition matching.
        let mut check_name = file_stem.strip_prefix('.').unwrap_or(file_stem).to_string();
        check_name = check_name.replacen('+', "_", 1);

        // RuboCop allows dots in filenames (e.g., show.html.haml_spec).
        // Check snake_case on each dot-separated segment individually.
        let all_segments_snake = check_name
            .split('.')
            .all(|seg| is_snake_case(seg.as_bytes()));
        if !all_segments_snake {
            diagnostics.push(self.diagnostic(
                source,
                1,
                0,
                format!("The name of this source file (`{file_stem}`) should use snake_case."),
            ));
        }

        // ExpectMatchingDefinition: require that the file defines a class/module matching the filename
        if expect_matching_definition {
            let source_text = std::str::from_utf8(&source.content).unwrap_or("");

            let expected_namespace = if check_def_path_hierarchy {
                build_expected_namespace(path, &check_def_path_roots)
            } else {
                vec![to_module_name(file_stem)]
            };

            if !has_matching_definition(source_text, &expected_namespace) {
                let namespace_str = expected_namespace.join("::");
                diagnostics.push(self.diagnostic(
                    source,
                    1,
                    0,
                    format!(
                        "`{file_stem}` should define a class or module called `{namespace_str}`."
                    ),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::source::SourceFile;

    crate::cop_scenario_fixture_tests!(
        FileName,
        "cops/naming/file_name",
        camel_case = "camel_case.rb",
        bad_name = "bad_name.rb",
        with_dash = "with_dash.rb",
        acronym_in_name = "acronym_in_name.rb",
        camelcase_acronym = "camelcase_acronym.rb",
        allowed_name_with_ext = "allowed_name_with_ext.rb",
        allowed_name_diff_ext = "allowed_name_diff_ext.rb",
    );

    #[test]
    fn offense_bad_filename() {
        let source = SourceFile::from_bytes("BadFile.rb", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].cop_name, "Naming/FileName");
        assert!(diags[0].message.contains("BadFile"));
    }

    #[test]
    fn offense_camel_case_filename() {
        let source = SourceFile::from_bytes("MyClass.rb", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert_eq!(diags.len(), 1);
    }

    #[test]
    fn no_offense_good_filename() {
        let source = SourceFile::from_bytes("good_file.rb", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(diags.is_empty());
    }

    #[test]
    fn no_offense_gemfile() {
        let source = SourceFile::from_bytes("Gemfile", b"source 'https://rubygems.org'\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(diags.is_empty());
    }

    #[test]
    fn no_offense_rakefile() {
        let source = SourceFile::from_bytes("Rakefile", b"task :default\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(diags.is_empty());
    }

    #[test]
    fn no_offense_vagrantfile() {
        let source = SourceFile::from_bytes(
            "Vagrantfile",
            b"Vagrant.configure('2') do |config|\nend\n".to_vec(),
        );
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(
            diags.is_empty(),
            "Vagrantfile should be in allowed CamelCase filenames"
        );
    }

    #[test]
    fn no_offense_test_rb() {
        let source = SourceFile::from_bytes("test.rb", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(diags.is_empty());
    }

    #[test]
    fn no_offense_gemspec() {
        let source = SourceFile::from_bytes(
            "my-gem.gemspec",
            b"Gem::Specification.new do |s|\n  s.name = 'my-gem'\nend\n".to_vec(),
        );
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(
            diags.is_empty(),
            "Should not flag .gemspec files (dashes are conventional)"
        );
    }

    #[test]
    fn no_offense_gemspec_with_namespace() {
        let source = SourceFile::from_bytes(
            "rack-protection.gemspec",
            b"Gem::Specification.new do |s|\n  s.name = 'rack-protection'\nend\n".to_vec(),
        );
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(
            diags.is_empty(),
            "Should not flag namespaced .gemspec files"
        );
    }

    #[test]
    fn config_ignore_executable_scripts() {
        let source =
            SourceFile::from_bytes("MyScript", b"#!/usr/bin/env ruby\nputs 'hi'\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(
            diags.is_empty(),
            "Should skip executable scripts with shebang"
        );
    }

    #[test]
    fn config_ignore_executable_scripts_false() {
        use std::collections::HashMap;
        let config = CopConfig {
            options: HashMap::from([(
                "IgnoreExecutableScripts".into(),
                serde_yml::Value::Bool(false),
            )]),
            ..CopConfig::default()
        };
        let source =
            SourceFile::from_bytes("MyScript.rb", b"#!/usr/bin/env ruby\nputs 'hi'\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &config, &mut diags, None);
        assert!(
            !diags.is_empty(),
            "Should flag non-snake_case even with shebang when IgnoreExecutableScripts:false"
        );
    }

    #[test]
    fn config_regex() {
        use std::collections::HashMap;
        let config = CopConfig {
            options: HashMap::from([(
                "Regex".into(),
                serde_yml::Value::String(r"\A[a-z_]+\z".into()),
            )]),
            ..CopConfig::default()
        };
        let source = SourceFile::from_bytes("good_file.rb", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &config, &mut diags, None);
        assert!(diags.is_empty(), "Should pass custom regex check");
    }

    #[test]
    fn config_allowed_acronyms_does_not_affect_filename_check() {
        // AllowedAcronyms only affects ExpectMatchingDefinition, NOT the snake_case check.
        // RuboCop's filename_good? uses SNAKE_CASE regex without acronym substitution.
        use std::collections::HashMap;
        let config = CopConfig {
            options: HashMap::from([(
                "AllowedAcronyms".into(),
                serde_yml::Value::Sequence(vec![serde_yml::Value::String("HTML".into())]),
            )]),
            ..CopConfig::default()
        };
        let source = SourceFile::from_bytes("my_HTML_parser.rb", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &config, &mut diags, None);
        assert_eq!(
            diags.len(),
            1,
            "AllowedAcronyms should not skip the snake_case filename check"
        );
    }

    #[test]
    fn expect_matching_definition_flags_missing_class() {
        use std::collections::HashMap;
        let config = CopConfig {
            options: HashMap::from([(
                "ExpectMatchingDefinition".into(),
                serde_yml::Value::Bool(true),
            )]),
            ..CopConfig::default()
        };
        let source = SourceFile::from_bytes("my_class.rb", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &config, &mut diags, None);
        assert!(
            !diags.is_empty(),
            "ExpectMatchingDefinition should flag file without matching class"
        );
        assert!(diags[0].message.contains("MyClass"));
    }

    #[test]
    fn expect_matching_definition_allows_matching_class() {
        use std::collections::HashMap;
        let config = CopConfig {
            options: HashMap::from([(
                "ExpectMatchingDefinition".into(),
                serde_yml::Value::Bool(true),
            )]),
            ..CopConfig::default()
        };
        let source = SourceFile::from_bytes("my_class.rb", b"class MyClass\nend\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &config, &mut diags, None);
        assert!(
            diags.is_empty(),
            "ExpectMatchingDefinition should accept matching class"
        );
    }

    #[test]
    fn expect_matching_definition_with_path_hierarchy() {
        use std::collections::HashMap;
        let config = CopConfig {
            options: HashMap::from([
                (
                    "ExpectMatchingDefinition".into(),
                    serde_yml::Value::Bool(true),
                ),
                (
                    "CheckDefinitionPathHierarchy".into(),
                    serde_yml::Value::Bool(true),
                ),
            ]),
            ..CopConfig::default()
        };
        let source = SourceFile::from_bytes(
            "lib/my_gem/my_class.rb",
            b"class MyGem::MyClass\nend\n".to_vec(),
        );
        let mut diags = Vec::new();
        FileName.check_lines(&source, &config, &mut diags, None);
        assert!(diags.is_empty(), "Should accept matching namespaced class");
    }

    #[test]
    fn no_offense_plus_in_filename() {
        // RuboCop replaces + with _ before snake_case check (Action Pack Variants convention)
        let source = SourceFile::from_bytes("some_file.xlsx+mobile.axlsx", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(
            diags.is_empty(),
            "Should allow + in filenames (Action Pack Variants convention)"
        );
    }

    #[test]
    fn no_offense_fastfile_suffix() {
        // Files matching *Fastfile are allowed (AllCops/Include pattern)
        let source = SourceFile::from_bytes("ImportFastfile", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(
            diags.is_empty(),
            "Should allow files matching *Fastfile pattern"
        );
    }

    #[test]
    fn no_offense_switcher_fastfile() {
        let source = SourceFile::from_bytes("SwitcherFastfile", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert!(
            diags.is_empty(),
            "Should allow files matching *Fastfile pattern"
        );
    }

    #[test]
    fn offense_allowed_name_with_extension() {
        // Rakefile.rb is NOT the same as Rakefile — it should be flagged.
        // RuboCop's allowed_camel_case_file? checks AllCops/Include patterns,
        // which match exact filenames like "Rakefile", not "Rakefile.rb".
        let source = SourceFile::from_bytes("Rakefile.rb", b"task :default\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert_eq!(
            diags.len(),
            1,
            "Rakefile.rb should be flagged (only exact Rakefile is allowed)"
        );
    }

    #[test]
    fn offense_vagrantfile_with_spec_extension() {
        // Vagrantfile.spec is NOT the same as Vagrantfile — it should be flagged.
        let source = SourceFile::from_bytes("Vagrantfile.spec", b"# spec file\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert_eq!(
            diags.len(),
            1,
            "Vagrantfile.spec should be flagged (only exact Vagrantfile is allowed)"
        );
    }

    #[test]
    fn offense_acronym_filename() {
        // Filenames with uppercase acronyms should be flagged even when acronyms are
        // in the AllowedAcronyms list. AllowedAcronyms only affects definition matching.
        let source = SourceFile::from_bytes("URI_test.rb", b"x = 1\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &CopConfig::default(), &mut diags, None);
        assert_eq!(
            diags.len(),
            1,
            "URI_test.rb should be flagged for snake_case"
        );
    }

    #[test]
    fn expect_matching_definition_no_hierarchy() {
        use std::collections::HashMap;
        let config = CopConfig {
            options: HashMap::from([
                (
                    "ExpectMatchingDefinition".into(),
                    serde_yml::Value::Bool(true),
                ),
                (
                    "CheckDefinitionPathHierarchy".into(),
                    serde_yml::Value::Bool(false),
                ),
            ]),
            ..CopConfig::default()
        };
        let source =
            SourceFile::from_bytes("lib/my_gem/my_class.rb", b"class MyClass\nend\n".to_vec());
        let mut diags = Vec::new();
        FileName.check_lines(&source, &config, &mut diags, None);
        assert!(
            diags.is_empty(),
            "Without hierarchy check, just the class name should match"
        );
    }
}
