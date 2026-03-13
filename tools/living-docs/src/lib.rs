use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Write;

/// Parse the crate name from a cargo test runner header line.
///
/// Recognises lines of the form:
/// `Running unittests src\lib.rs (target\debug\deps\<crate>-<hash>.exe)`
/// `Running unittests src\main.rs (target\debug\deps\<crate>-<hash>.exe)`
///
/// Returns `None` for any line that does not match the pattern.
pub fn parse_crate_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("Running unittests ")?;
    let paren_start = rest.find('(')?;
    let path = &rest[paren_start + 1..];
    let filename = path.rsplit(['\\', '/']).next()?;
    let stem = filename.strip_suffix(".exe").unwrap_or(filename);
    let crate_name = stem.split('-').next()?;
    if crate_name.is_empty() {
        return None;
    }
    Some(crate_name.to_string())
}

/// A parsed test entry extracted from cargo output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTest {
    pub module: String,
    pub name: String,
}

/// Parse a test line from cargo output into module and test name.
///
/// Recognises lines like `module::tests::test_name: test`.
/// Doc-test lines are grouped under a `"doc_tests"` module.
/// Returns `None` for non-test lines (summaries, warnings, blanks).
pub fn parse_test_entry(line: &str) -> Option<ParsedTest> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Doc-test format: "path\to\file.rs - qualified::Name (line N): test"
    if let Some(rest) = trimmed.strip_suffix(": test") {
        if let Some(dash_pos) = rest.find(" - ") {
            let name = rest[dash_pos + 3..].to_string();
            return Some(ParsedTest {
                module: "doc_tests".to_string(),
                name,
            });
        }

        // Standard format: "module::tests::test_name: test"
        if let Some(tests_pos) = rest.find("::tests::") {
            let module = rest[..tests_pos].to_string();
            let name = rest[tests_pos + 9..].to_string();
            let top_module = module.split("::").next().unwrap_or(&module).to_string();
            return Some(ParsedTest {
                module: top_module,
                name,
            });
        }

        // Root-level tests: "tests::test_name: test" (no module prefix)
        if let Some(name) = rest.strip_prefix("tests::") {
            return Some(ParsedTest {
                module: "tests".to_string(),
                name: name.to_string(),
            });
        }
    }

    None
}

/// Convert a `snake_case` test name into a readable description.
///
/// `when_foo_then_bar` becomes `"When foo, then bar"`.
/// Names without `when_` get first-letter capitalization.
pub fn to_readable_description(name: &str) -> String {
    let spaced = name.replace('_', " ");

    if spaced.starts_with("when ") {
        let capitalized = format!("W{}", &spaced[1..]);
        if let Some(pos) = capitalized.find(" then ") {
            format!("{}, then {}", &capitalized[..pos], &capitalized[pos + 6..])
        } else {
            capitalized
        }
    } else {
        let mut chars = spaced.chars();
        match chars.next() {
            Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
            None => String::new(),
        }
    }
}

/// Documentation for a single crate: module → list of readable test descriptions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateDoc {
    pub name: String,
    pub modules: BTreeMap<String, Vec<TestDoc>>,
}

/// A single test's documentation entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestDoc {
    pub description: String,
    pub annotation: Option<String>,
    pub source: Option<SourceLocation>,
}

/// Source file path and line number for a test function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub line: usize,
}

/// Source location map: maps test function name → file + line.
pub type SourceMap = HashMap<String, SourceLocation>;

/// Annotation map: maps `"crate_name::module::test_name"` → extended description.
pub type AnnotationMap = HashMap<String, String>;

/// Parse source file content for `/// @doc:` annotations above `#[test]` functions.
///
/// Returns a map of test function name → annotation text.
pub fn parse_annotations(source: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("/// @doc:") {
            let annotation = rest.trim().to_string();
            // Look ahead for #[test] then fn name
            for next_line in &lines[(i + 1)..] {
                let next = next_line.trim();
                if next.starts_with("fn ") {
                    if let Some(name) = next.strip_prefix("fn ").and_then(|s| s.split('(').next()) {
                        result.insert(name.trim().to_string(), annotation);
                    }
                    break;
                }
                if next.is_empty() || next.starts_with("///") || next.starts_with("#[") {
                    continue;
                }
                break;
            }
        }
    }

    result
}

/// Scan source content for `#[test]` functions and record their line numbers.
///
/// Returns a map of test function name → source location.
pub fn parse_test_locations(source: &str, file_path: &str) -> SourceMap {
    let mut result = HashMap::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "#[test]" {
            for next_line in &lines[(i + 1)..] {
                let next = next_line.trim();
                if next.starts_with("fn ") {
                    if let Some(name) = next.strip_prefix("fn ").and_then(|s| s.split('(').next()) {
                        result.insert(
                            name.trim().to_string(),
                            SourceLocation {
                                file: file_path.to_string(),
                                line: i + 2, // 1-based, pointing at the fn line
                            },
                        );
                    }
                    break;
                }
                if next.is_empty() || next.starts_with("#[") || next.starts_with("///") {
                    continue;
                }
                break;
            }
        }
    }

    result
}

/// Convert parsed cargo output into `CrateDoc` structs.
pub fn convert_to_docs(
    parsed: &BTreeMap<String, Vec<ParsedTest>>,
    annotations: &AnnotationMap,
    sources: &SourceMap,
) -> Vec<CrateDoc> {
    parsed
        .iter()
        .map(|(crate_name, tests)| {
            let mut modules: BTreeMap<String, Vec<TestDoc>> = BTreeMap::new();
            for test in tests {
                let description = to_readable_description(&test.name);
                let key = format!(
                    "{crate_name}::{module}::{name}",
                    module = test.module,
                    name = test.name
                );
                let annotation = annotations
                    .get(&key)
                    .or_else(|| annotations.get(&test.name))
                    .cloned();
                let source = sources.get(&test.name).cloned();
                modules
                    .entry(test.module.clone())
                    .or_default()
                    .push(TestDoc {
                        description,
                        annotation,
                        source,
                    });
            }
            CrateDoc {
                name: crate_name.clone(),
                modules,
            }
        })
        .collect()
}

/// Generate markdown documentation from `CrateDoc` entries.
pub fn generate_markdown(docs: &[CrateDoc], total: usize, date: &str) -> String {
    let mut out = String::new();
    out.push_str("# Axiom2d Living Documentation\n\n");
    let _ = writeln!(
        out,
        "> Auto-generated from {total} test cases. Last updated: {date}.\n"
    );

    let mut sorted: Vec<&CrateDoc> = docs.iter().collect();
    sorted.sort_by_key(|d| &d.name);

    for doc in sorted {
        let test_count: usize = doc.modules.values().map(Vec::len).sum();
        let _ = writeln!(out, "<details>");
        let _ = writeln!(
            out,
            "<summary><strong>{}</strong> ({test_count} tests)</summary>\n",
            doc.name
        );

        for (module, tests) in &doc.modules {
            let _ = writeln!(out, "<blockquote>");
            let _ = writeln!(out, "<details>");
            let _ = writeln!(
                out,
                "<summary><strong>{module}</strong> ({} tests)</summary>\n",
                tests.len()
            );
            for test in tests {
                let has_details = test.annotation.is_some() || test.source.is_some();
                if has_details {
                    let _ = writeln!(out, "<blockquote>");
                    let _ = writeln!(out, "<details>");
                    let _ = writeln!(out, "<summary>{}</summary>\n", test.description);
                    if let Some(ref ann) = test.annotation {
                        let _ = writeln!(out, "*{ann}*\n");
                    }
                    if let Some(ref loc) = test.source {
                        let _ = writeln!(out, "<code>{}:{}</code>\n", loc.file, loc.line);
                    }
                    let _ = writeln!(out, "</details>");
                    let _ = writeln!(out, "</blockquote>");
                } else {
                    let _ = writeln!(out, "- {}", test.description);
                }
            }
            let _ = writeln!(out, "\n</details>");
            let _ = writeln!(out, "</blockquote>\n");
        }

        let _ = writeln!(out, "</details>\n");
    }

    out
}

/// Parse full cargo test output into a map of crate name → test entries.
///
/// Crates with no tests appear with an empty vec.
pub fn parse_cargo_output(output: &str) -> BTreeMap<String, Vec<ParsedTest>> {
    let mut result: BTreeMap<String, Vec<ParsedTest>> = BTreeMap::new();
    let mut current_crate: Option<String> = None;

    for line in output.lines() {
        if let Some(crate_name) = parse_crate_name(line) {
            current_crate = Some(crate_name.clone());
            result.entry(crate_name).or_default();
            continue;
        }

        if let Some(ref crate_name) = current_crate
            && let Some(test) = parse_test_entry(line)
        {
            result.entry(crate_name.clone()).or_default().push(test);
        }
    }

    result
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{
        AnnotationMap, CrateDoc, ParsedTest, SourceLocation, SourceMap, TestDoc, convert_to_docs,
        generate_markdown, parse_annotations, parse_cargo_output, parse_crate_name,
        parse_test_entry, parse_test_locations, to_readable_description,
    };
    use std::collections::BTreeMap;

    // TC001
    #[test]
    fn when_lib_unittest_line_parsed_then_returns_crate_name() {
        // Arrange
        let line = r"Running unittests src\lib.rs (target\debug\deps\engine_core-abc123.exe)";

        // Act
        let result = parse_crate_name(line);

        // Assert
        assert_eq!(result, Some("engine_core".to_string()));
    }

    // TC002
    #[test]
    fn when_binary_unittest_line_parsed_then_returns_crate_name() {
        // Arrange
        let line = r"Running unittests src\main.rs (target\debug\deps\demo-deadbeef.exe)";

        // Act
        let result = parse_crate_name(line);

        // Assert
        assert_eq!(result, Some("demo".to_string()));
    }

    // TC003
    #[test]
    fn when_non_running_line_parsed_then_returns_none() {
        // Arrange
        let line = "test engine_core::color::tests::when_color_serialized ... ok";

        // Act
        let result = parse_crate_name(line);

        // Assert
        assert!(result.is_none());
    }

    // TC004
    #[test]
    fn when_standard_test_line_parsed_then_extracts_module_and_name() {
        // Arrange
        let line = "app::tests::when_add_plugin_called_then_plugin_count_increments: test";

        // Act
        let result = parse_test_entry(line);

        // Assert
        assert_eq!(
            result,
            Some(ParsedTest {
                module: "app".to_string(),
                name: "when_add_plugin_called_then_plugin_count_increments".to_string(),
            })
        );
    }

    // TC005
    #[test]
    fn when_nested_module_test_line_parsed_then_uses_top_level_module() {
        // Arrange
        let line = "default_plugins::tests::when_audio_on_then_audio_res_present: test";

        // Act
        let result = parse_test_entry(line);

        // Assert
        assert_eq!(
            result,
            Some(ParsedTest {
                module: "default_plugins".to_string(),
                name: "when_audio_on_then_audio_res_present".to_string(),
            })
        );
    }

    // TC006
    #[test]
    fn when_doc_test_line_parsed_then_returns_doc_tests_module() {
        // Arrange
        let line = r"crates\engine_assets\src\handle.rs - handle::Handle (line 6): test";

        // Act
        let result = parse_test_entry(line);

        // Assert
        assert_eq!(
            result,
            Some(ParsedTest {
                module: "doc_tests".to_string(),
                name: "handle::Handle (line 6)".to_string(),
            })
        );
    }

    // TC007
    #[test]
    fn when_summary_line_parsed_then_returns_none() {
        // Arrange
        let line = "39 tests, 0 benchmarks";

        // Act
        let result = parse_test_entry(line);

        // Assert
        assert!(result.is_none());
    }

    // TC008
    #[test]
    fn when_empty_line_parsed_then_returns_none() {
        // Arrange
        let line = "";

        // Act
        let result = parse_test_entry(line);

        // Assert
        assert!(result.is_none());
    }

    // TC009
    #[test]
    fn when_when_then_name_converted_then_capitalizes_when_and_inserts_comma() {
        // Arrange
        let name = "when_fake_clock_advanced_then_delta_returns_pending";

        // Act
        let result = to_readable_description(name);

        // Assert
        assert_eq!(
            result,
            "When fake clock advanced, then delta returns pending"
        );
    }

    // TC010
    #[test]
    fn when_name_without_when_prefix_converted_then_capitalizes_first_letter() {
        // Arrange
        let name = "add_plugin_chained_twice_does_not_panic";

        // Act
        let result = to_readable_description(name);

        // Assert
        assert_eq!(result, "Add plugin chained twice does not panic");
    }

    // TC011
    #[test]
    fn when_when_then_with_long_middle_then_comma_before_then() {
        // Arrange
        let name = "when_atlas_inserted_and_frame_runs_then_upload_called";

        // Act
        let result = to_readable_description(name);

        // Assert
        assert_eq!(
            result,
            "When atlas inserted and frame runs, then upload called"
        );
    }

    // TC013
    #[test]
    fn when_multi_crate_output_parsed_then_groups_tests_by_crate() {
        // Arrange
        let output = "\
     Running unittests src\\lib.rs (target\\debug\\deps\\engine_core-abc123.exe)
color::tests::when_from_u8_called_then_converts: test
time::tests::when_delta_read_then_returns_seconds: test
2 tests, 0 benchmarks
     Running unittests src\\lib.rs (target\\debug\\deps\\engine_ecs-def456.exe)
schedule::tests::when_phase_index_called_then_returns_ordinal: test
1 tests, 0 benchmarks";

        // Act
        let result = parse_cargo_output(output);

        // Assert
        assert_eq!(result.len(), 2);
        assert_eq!(result["engine_core"].len(), 2);
        assert_eq!(result["engine_ecs"].len(), 1);
        assert_eq!(result["engine_core"][0].module, "color");
        assert_eq!(result["engine_ecs"][0].module, "schedule");
    }

    // TC014
    #[test]
    fn when_crate_has_no_tests_then_crate_still_appears() {
        // Arrange
        let output = "\
     Running unittests src\\lib.rs (target\\debug\\deps\\engine_empty-aaa111.exe)
0 tests, 0 benchmarks";

        // Act
        let result = parse_cargo_output(output);

        // Assert
        assert!(result.contains_key("engine_empty"));
        assert!(result["engine_empty"].is_empty());
    }

    // TC015
    #[test]
    fn when_warnings_interspersed_then_they_are_ignored() {
        // Arrange
        let output = "\
   Compiling engine_core v0.1.0
warning: unused variable
     Running unittests src\\lib.rs (target\\debug\\deps\\engine_core-abc123.exe)
color::tests::when_from_u8_then_converts: test
warning: some other warning
1 tests, 0 benchmarks";

        // Act
        let result = parse_cargo_output(output);

        // Assert
        assert_eq!(result["engine_core"].len(), 1);
    }

    // TC016
    #[test]
    fn when_crate_doc_generated_then_produces_heading_and_subheadings() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert(
            "time".to_string(),
            vec![TestDoc {
                description: "When delta read, then returns seconds".to_string(),
                annotation: None,
                source: None,
            }],
        );
        modules.insert(
            "color".to_string(),
            vec![TestDoc {
                description: "When from u8, then converts".to_string(),
                annotation: None,
                source: None,
            }],
        );
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 2, "2026-03-13");

        // Assert
        assert!(md.contains("<strong>engine_core</strong> (2 tests)"));
        assert!(md.contains("<strong>color</strong>"));
        assert!(md.contains("<strong>time</strong>"));
    }

    // TC017
    #[test]
    fn when_markdown_generated_then_tests_appear_as_list_items() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert(
            "time".to_string(),
            vec![TestDoc {
                description: "When delta read, then returns seconds".to_string(),
                annotation: None,
                source: None,
            }],
        );
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 1, "2026-03-13");

        // Assert
        assert!(md.contains("- When delta read, then returns seconds"));
    }

    // TC018
    #[test]
    fn when_markdown_generated_then_header_includes_count_and_date() {
        // Arrange
        let docs = vec![];

        // Act
        let md = generate_markdown(&docs, 658, "2026-03-13");

        // Assert
        assert!(md.contains("# Axiom2d Living Documentation"));
        assert!(md.contains("658 test cases"));
        assert!(md.contains("2026-03-13"));
    }

    // TC019
    #[test]
    fn when_multiple_crates_then_alphabetical_order() {
        // Arrange
        let docs = vec![
            CrateDoc {
                name: "engine_render".to_string(),
                modules: BTreeMap::new(),
            },
            CrateDoc {
                name: "axiom2d".to_string(),
                modules: BTreeMap::new(),
            },
            CrateDoc {
                name: "engine_core".to_string(),
                modules: BTreeMap::new(),
            },
        ];

        // Act
        let md = generate_markdown(&docs, 0, "2026-03-13");

        // Assert
        let pos_a = md
            .find("<strong>axiom2d</strong>")
            .expect("axiom2d heading");
        let pos_c = md
            .find("<strong>engine_core</strong>")
            .expect("engine_core heading");
        let pos_r = md
            .find("<strong>engine_render</strong>")
            .expect("engine_render heading");
        assert!(pos_a < pos_c);
        assert!(pos_c < pos_r);
    }

    // TC020
    #[test]
    fn when_modules_in_crate_then_alphabetical_order() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert("time".to_string(), vec![]);
        modules.insert("color".to_string(), vec![]);
        modules.insert("spatial".to_string(), vec![]);
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 0, "2026-03-13");

        // Assert
        let pos_c = md.find("<strong>color</strong>").expect("color heading");
        let pos_s = md
            .find("<strong>spatial</strong>")
            .expect("spatial heading");
        let pos_t = md.find("<strong>time</strong>").expect("time heading");
        assert!(pos_c < pos_s);
        assert!(pos_s < pos_t);
    }

    // TC021
    #[test]
    fn when_full_pipeline_then_produces_valid_markdown() {
        // Arrange
        let output = "\
     Running unittests src\\lib.rs (target\\debug\\deps\\engine_core-abc123.exe)
color::tests::when_from_u8_called_then_converts: test
time::tests::when_delta_read_then_returns_seconds: test
2 tests, 0 benchmarks
     Running unittests src\\lib.rs (target\\debug\\deps\\engine_ecs-def456.exe)
schedule::tests::when_phase_index_called_then_returns_ordinal: test
1 tests, 0 benchmarks";

        let parsed = parse_cargo_output(output);
        let annotations = AnnotationMap::new();
        let sources = SourceMap::new();
        let docs = convert_to_docs(&parsed, &annotations, &sources);

        // Act
        let total: usize = docs
            .iter()
            .map(|d| d.modules.values().map(Vec::len).sum::<usize>())
            .sum();
        let md = generate_markdown(&docs, total, "2026-03-13");

        // Assert
        assert!(md.starts_with("# Axiom2d Living Documentation"));
        assert!(md.contains("<strong>engine_core</strong>"));
        assert!(md.contains("<strong>engine_ecs</strong>"));
        assert!(md.contains("When from u8 called, then converts"));
    }

    // TC022
    #[test]
    fn when_source_has_doc_annotation_then_parse_annotations_extracts_it() {
        // Arrange
        let source = r#"
    /// @doc: Verifies that byte-to-float conversion is correct
    #[test]
    fn when_from_u8_called_then_converts() {
        // test body
    }
"#;

        // Act
        let result = parse_annotations(source);

        // Assert
        assert_eq!(
            result.get("when_from_u8_called_then_converts"),
            Some(&"Verifies that byte-to-float conversion is correct".to_string())
        );
    }

    // TC023
    #[test]
    fn when_source_has_no_annotations_then_map_is_empty() {
        // Arrange
        let source = r#"
    #[test]
    fn when_foo_then_bar() {
        // test body
    }
"#;

        // Act
        let result = parse_annotations(source);

        // Assert
        assert!(result.is_empty());
    }

    // TC025
    #[test]
    fn when_markdown_generated_then_crate_sections_are_collapsible() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert(
            "color".to_string(),
            vec![TestDoc {
                description: "When from u8, then converts".to_string(),
                annotation: None,
                source: None,
            }],
        );
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 1, "2026-03-13");

        // Assert
        assert!(md.contains("<details>"));
        assert!(md.contains("<summary>"));
        assert!(md.contains("<strong>engine_core</strong> (1 tests)"));
        assert!(md.contains("</summary>"));
        assert!(md.contains("</details>"));
    }

    // TC026
    #[test]
    fn when_markdown_generated_then_module_sections_are_collapsible() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert(
            "color".to_string(),
            vec![TestDoc {
                description: "When from u8, then converts".to_string(),
                annotation: None,
                source: None,
            }],
        );
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 1, "2026-03-13");

        // Assert
        assert!(md.contains("<blockquote>\n<details>\n<summary><strong>color</strong>"));
        assert!(md.contains("</details>\n</blockquote>"));
    }

    // TC024
    #[test]
    fn when_annotation_present_then_markdown_includes_it_as_subtext() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert(
            "color".to_string(),
            vec![TestDoc {
                description: "When from u8 called, then converts".to_string(),
                annotation: Some("Verifies byte-to-float conversion".to_string()),
                source: None,
            }],
        );
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 1, "2026-03-13");

        // Assert
        assert!(md.contains("<summary>When from u8 called, then converts</summary>"));
        assert!(md.contains("*Verifies byte-to-float conversion*"));
    }

    // TC027
    #[test]
    fn when_source_has_test_fn_then_parse_test_locations_returns_location() {
        // Arrange
        let source = "    #[test]\n    fn when_foo_then_bar() {\n    }\n";

        // Act
        let result = parse_test_locations(source, "crates/engine_core/src/color.rs");

        // Assert
        assert_eq!(
            result.get("when_foo_then_bar"),
            Some(&SourceLocation {
                file: "crates/engine_core/src/color.rs".to_string(),
                line: 2,
            })
        );
    }

    // TC028
    #[test]
    fn when_test_has_source_then_markdown_shows_location_in_foldout() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert(
            "color".to_string(),
            vec![TestDoc {
                description: "When from u8, then converts".to_string(),
                annotation: None,
                source: Some(SourceLocation {
                    file: "crates/engine_core/src/color.rs".to_string(),
                    line: 42,
                }),
            }],
        );
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 1, "2026-03-13");

        // Assert
        assert!(md.contains("<summary>When from u8, then converts</summary>"));
        assert!(md.contains("<code>crates/engine_core/src/color.rs:42</code>"));
        assert!(md.contains("<blockquote>\n<details>"));
        assert!(md.contains("</details>\n</blockquote>"));
    }

    // TC029
    #[test]
    fn when_test_has_no_annotation_or_source_then_rendered_as_plain_list_item() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert(
            "color".to_string(),
            vec![TestDoc {
                description: "When from u8, then converts".to_string(),
                annotation: None,
                source: None,
            }],
        );
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 1, "2026-03-13");

        // Assert
        assert!(md.contains("- When from u8, then converts"));
        assert!(!md.contains("<summary>When from u8"));
    }

    // TC030
    #[test]
    fn when_test_has_annotation_and_source_then_both_in_foldout() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert(
            "time".to_string(),
            vec![TestDoc {
                description: "When tick large delta, then returns multiple steps".to_string(),
                annotation: Some("Fix Your Timestep pattern".to_string()),
                source: Some(SourceLocation {
                    file: "crates/engine_core/src/time.rs".to_string(),
                    line: 99,
                }),
            }],
        );
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 1, "2026-03-13");

        // Assert
        assert!(
            md.contains("<summary>When tick large delta, then returns multiple steps</summary>")
        );
        assert!(md.contains("*Fix Your Timestep pattern*"));
        assert!(md.contains("<code>crates/engine_core/src/time.rs:99</code>"));
    }

    // TC031
    #[test]
    fn when_module_section_then_wrapped_in_blockquote() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert(
            "color".to_string(),
            vec![TestDoc {
                description: "When from u8, then converts".to_string(),
                annotation: None,
                source: None,
            }],
        );
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 1, "2026-03-13");

        // Assert
        assert!(md.contains("<blockquote>\n<details>\n<summary><strong>color</strong>"));
    }

    // TC032
    #[test]
    fn when_test_foldout_then_wrapped_in_blockquote() {
        // Arrange
        let mut modules = BTreeMap::new();
        modules.insert(
            "color".to_string(),
            vec![TestDoc {
                description: "When from u8, then converts".to_string(),
                annotation: None,
                source: Some(SourceLocation {
                    file: "src/color.rs".to_string(),
                    line: 10,
                }),
            }],
        );
        let docs = vec![CrateDoc {
            name: "engine_core".to_string(),
            modules,
        }];

        // Act
        let md = generate_markdown(&docs, 1, "2026-03-13");

        // Assert
        assert!(
            md.contains("<blockquote>\n<details>\n<summary>When from u8, then converts</summary>")
        );
    }
}
