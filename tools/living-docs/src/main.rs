use living_docs::{
    AnnotationMap, BodyMap, SourceMap, convert_to_docs, generate_markdown, parse_annotations,
    parse_cargo_output, parse_test_bodies, parse_test_locations, parse_test_results,
};
use std::path::Path;
use std::process::Command;

fn collect_from_dir(
    crates_dir: &Path,
    annotations: &mut AnnotationMap,
    sources: &mut SourceMap,
    bodies: &mut BodyMap,
) {
    let Ok(entries) = std::fs::read_dir(crates_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let src_dir = entry.path().join("src");
        if src_dir.is_dir() {
            scan_dir(&src_dir, annotations, sources, bodies);
        }
    }
}

fn scan_dir(
    dir: &Path,
    annotations: &mut AnnotationMap,
    sources: &mut SourceMap,
    bodies: &mut BodyMap,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, annotations, sources, bodies);
        } else if path.extension().is_some_and(|ext| ext == "rs")
            && let Ok(source) = std::fs::read_to_string(&path)
        {
            let file_path = path.to_string_lossy().to_string();
            annotations.extend(parse_annotations(&source));
            sources.extend(parse_test_locations(&source, &file_path));
            bodies.extend(parse_test_bodies(&source));
        }
    }
}

fn main() {
    // Run actual tests — "Running" headers go to stderr, results to stdout — merge via shell redirect
    let output = Command::new("bash")
        .args(["-c", "cargo.exe test 2>&1"])
        .output()
        .expect("failed to run cargo.exe test");

    let combined = String::from_utf8_lossy(&output.stdout);

    let parsed = parse_cargo_output(&combined);
    let results = parse_test_results(&combined);

    let mut annotations = AnnotationMap::new();
    let mut sources = SourceMap::new();
    let mut bodies = BodyMap::new();
    for dir in [Path::new("crates"), Path::new("tools")] {
        collect_from_dir(dir, &mut annotations, &mut sources, &mut bodies);
    }

    let docs = convert_to_docs(&parsed, &annotations, &sources, &bodies, &results);
    let total: usize = docs
        .iter()
        .map(|d| d.modules.values().map(Vec::len).sum::<usize>())
        .sum();

    let date = chrono_free_date();
    let markdown = generate_markdown(&docs, total, &date);

    let out_path = Path::new("Doc").join("Living_Documentation.md");
    std::fs::write(&out_path, &markdown).expect("failed to write Living_Documentation.md");

    println!("Generated {} with {total} test cases.", out_path.display());
}

fn chrono_free_date() -> String {
    // Windows-compatible: use PowerShell to get ISO date
    Command::new("powershell.exe")
        .args(["-Command", "Get-Date -Format yyyy-MM-dd"])
        .output()
        .map_or_else(
            |_| "unknown".to_string(),
            |o| String::from_utf8_lossy(&o.stdout).trim().to_string(),
        )
}
