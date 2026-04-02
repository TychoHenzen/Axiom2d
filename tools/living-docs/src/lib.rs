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
#[allow(clippy::too_many_lines)]
pub fn parse_test_entry(line: &str) -> Option<ParsedTest> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    let rest = trimmed
        .strip_suffix(": test")
        .or_else(|| trimmed.strip_suffix(" ... ok"))
        .or_else(|| trimmed.strip_suffix(" ... FAILED"))
        .or_else(|| trimmed.strip_suffix(" ... ignored"))?;

    if let Some(dash_pos) = rest.find(" - ") {
        let name = rest[dash_pos + 3..].to_string();
        return Some(ParsedTest {
            module: "doc_tests".to_string(),
            name,
        });
    }

    if let Some(tests_pos) = rest.find("::tests::") {
        let module = rest[..tests_pos].to_string();
        let name = rest[tests_pos + 9..].to_string();
        let top_module = module.split("::").next().unwrap_or(&module).to_string();
        return Some(ParsedTest {
            module: top_module,
            name,
        });
    }

    if let Some(name) = rest.strip_prefix("tests::") {
        return Some(ParsedTest {
            module: "tests".to_string(),
            name: name.to_string(),
        });
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
    pub body: Option<String>,
    pub passed: Option<bool>,
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

/// Body map: maps test function name → extracted function body.
pub type BodyMap = HashMap<String, String>;

/// Result map: maps test function name → passed (true/false).
pub type ResultMap = HashMap<String, bool>;

/// Parse source file content for `/// @doc:` annotations above `#[test]` functions.
///
/// Returns a map of test function name → annotation text.
pub fn parse_annotations(source: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let lines: Vec<&str> = source.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("/// @doc:") {
            let mut parts = vec![rest.trim()];
            for next_line in &lines[(i + 1)..] {
                let next = next_line.trim();
                if next.starts_with("fn ") {
                    if let Some(name) = next.strip_prefix("fn ").and_then(|s| s.split('(').next()) {
                        result.insert(name.trim().to_string(), parts.join(" "));
                    }
                    break;
                }
                if let Some(cont) = next.strip_prefix("///") {
                    let cont = cont.trim();
                    if !cont.starts_with("@doc:") && !cont.is_empty() {
                        parts.push(cont);
                    }
                    continue;
                }
                if next.is_empty() || next.starts_with("#[") {
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

/// Scan source content for `#[test]` functions and extract their body lines.
///
/// Returns a map of test function name → the lines between the opening and
/// closing braces (indentation preserved, trailing newline included).
pub fn parse_test_bodies(source: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].trim() != "#[test]" {
            i += 1;
            continue;
        }

        let Some(fn_idx) = find_fn_line_after_test(&lines, i) else {
            i += 1;
            continue;
        };

        let fn_line = lines[fn_idx].trim();
        let name = extract_fn_name(fn_line);
        let past_fn = fn_idx + 1;
        let body_start_idx = find_body_start(&lines, fn_line, past_fn);
        let body = extract_brace_delimited_body(&lines, body_start_idx);
        result.insert(name, body);

        i = past_fn;
    }

    result
}

fn find_fn_line_after_test(lines: &[&str], test_attr_idx: usize) -> Option<usize> {
    for (j, line) in lines.iter().enumerate().skip(test_attr_idx + 1) {
        let next = line.trim();
        if next.starts_with("fn ") {
            return Some(j);
        }
        if next.is_empty() || next.starts_with("#[") && next != "#[test]" || next.starts_with("///")
        {
            continue;
        }
        break;
    }
    None
}

fn extract_fn_name(fn_line: &str) -> String {
    fn_line
        .strip_prefix("fn ")
        .expect("fn_line found via starts_with check")
        .split('(')
        .next()
        .expect("split always yields at least one element")
        .trim()
        .to_string()
}

fn find_body_start(lines: &[&str], fn_line: &str, past_fn: usize) -> usize {
    if fn_line.ends_with('{') {
        past_fn
    } else {
        lines[past_fn..]
            .iter()
            .enumerate()
            .find_map(|(offset, line)| (line.trim() == "{").then_some(past_fn + offset + 1))
            .unwrap_or(past_fn)
    }
}

fn extract_brace_delimited_body(lines: &[&str], start: usize) -> String {
    let mut depth: usize = 1;
    let mut body_lines: Vec<&str> = Vec::new();

    for line in lines.iter().skip(start) {
        for ch in line.chars() {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
        if depth == 0 {
            break;
        }
        body_lines.push(line);
    }

    body_lines.join("\n") + "\n"
}

/// Parse cargo test output for pass/fail results.
///
/// Recognises lines like `module::tests::test_name ... ok` or `... FAILED`.
/// Returns a map of bare test function name → passed (true/false).
pub fn parse_test_results(output: &str) -> HashMap<String, bool> {
    let mut result = HashMap::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_suffix(" ... ok")
            && let Some(name) = rest.rsplit("::").next()
        {
            result.insert(name.to_string(), true);
        } else if let Some(rest) = trimmed.strip_suffix(" ... FAILED")
            && let Some(name) = rest.rsplit("::").next()
        {
            result.insert(name.to_string(), false);
        }
    }

    result
}

/// Convert parsed cargo output into `CrateDoc` structs.
#[allow(clippy::too_many_lines)]
pub fn convert_to_docs(
    parsed: &BTreeMap<String, Vec<ParsedTest>>,
    annotations: &AnnotationMap,
    sources: &SourceMap,
    bodies: &BodyMap,
    results: &ResultMap,
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
                let body = bodies.get(&test.name).cloned();
                let did_pass = results.get(&test.name).copied();
                modules
                    .entry(test.module.clone())
                    .or_default()
                    .push(TestDoc {
                        description,
                        annotation,
                        source,
                        body,
                        passed: did_pass,
                    });
            }
            CrateDoc {
                name: crate_name.clone(),
                modules,
            }
        })
        .collect()
}

/// Generate compact plain-text documentation optimized for LLM consumption.
///
/// No HTML, no code blocks. Source locations and annotations are inlined.
/// Uses `##` / `###` headings and simple bullet lists for minimal token usage.
pub fn generate_llm_markdown(docs: &[CrateDoc], total: usize, date: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# Axiom2d Living Documentation ({total} tests, {date})\n"
    );

    let mut sorted: Vec<&CrateDoc> = docs.iter().collect();
    sorted.sort_by_key(|d| &d.name);

    for doc in sorted {
        let test_count: usize = doc.modules.values().map(Vec::len).sum();
        let _ = writeln!(out, "## {} ({test_count} tests)", doc.name);

        for (module, tests) in &doc.modules {
            let _ = writeln!(out, "### {module} ({} tests)", tests.len());
            for test in tests {
                let status = match test.passed {
                    Some(true) => "pass",
                    Some(false) => "FAIL",
                    None => "?",
                };
                let loc = test
                    .source
                    .as_ref()
                    .map_or(String::new(), |s| format!(" ({}:{})", s.file, s.line));
                if let Some(ref ann) = test.annotation {
                    let _ = writeln!(out, "- [{status}] {}{loc} — {ann}", test.description);
                } else {
                    let _ = writeln!(out, "- [{status}] {}{loc}", test.description);
                }
            }
        }
        out.push('\n');
    }

    out
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
        write_crate_section(&mut out, doc);
    }

    out
}

fn write_crate_section(out: &mut String, doc: &CrateDoc) {
    let test_count: usize = doc.modules.values().map(Vec::len).sum();
    let _ = writeln!(out, "<details>");
    let _ = writeln!(
        out,
        "<summary><strong>{}</strong> ({test_count} tests)</summary>\n",
        doc.name
    );

    for (module, tests) in &doc.modules {
        write_module_section(out, module, tests);
    }

    let _ = writeln!(out, "</details>\n");
}

fn write_module_section(out: &mut String, module: &str, tests: &[TestDoc]) {
    let _ = writeln!(out, "<blockquote>");
    let _ = writeln!(out, "<details>");
    let _ = writeln!(
        out,
        "<summary><strong>{module}</strong> ({} tests)</summary>\n",
        tests.len()
    );
    for test in tests {
        write_test_entry(out, test);
    }
    let _ = writeln!(out, "\n</details>");
    let _ = writeln!(out, "</blockquote>\n");
}

fn write_test_entry(out: &mut String, test: &TestDoc) {
    let status = match test.passed {
        Some(true) => "\u{2705} ",
        Some(false) => "\u{274c} ",
        None => "",
    };
    let has_details = test.annotation.is_some() || test.source.is_some() || test.body.is_some();
    if has_details {
        let _ = writeln!(out, "<blockquote>");
        let _ = writeln!(out, "<details>");
        let _ = writeln!(out, "<summary>{status}{}</summary>\n", test.description);
        if let Some(ref ann) = test.annotation {
            let _ = writeln!(out, "*{ann}*\n");
        }
        if let Some(ref loc) = test.source {
            let _ = writeln!(out, "<code>{}:{}</code>\n", loc.file, loc.line);
        }
        if let Some(ref body) = test.body {
            let _ = writeln!(out, "```rust\n{body}```\n");
        }
        let _ = writeln!(out, "</details>");
        let _ = writeln!(out, "</blockquote>");
    } else {
        let _ = writeln!(out, "- {status}{}", test.description);
    }
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
