use living_docs::{
    AnnotationMap, SourceMap, convert_to_docs, generate_markdown, parse_annotations,
    parse_cargo_output, parse_test_locations,
};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

fn collect_annotations(crates_dir: &Path) -> AnnotationMap {
    let mut all_annotations: AnnotationMap = HashMap::new();

    let Ok(entries) = std::fs::read_dir(crates_dir) else {
        eprintln!("warning: could not read crates directory");
        return all_annotations;
    };

    for entry in entries.flatten() {
        let src_dir = entry.path().join("src");
        if !src_dir.is_dir() {
            continue;
        }
        scan_dir(&src_dir, &mut all_annotations, &mut HashMap::new());
    }

    all_annotations
}

fn collect_sources(crates_dir: &Path) -> SourceMap {
    let mut all_sources: SourceMap = HashMap::new();

    let Ok(entries) = std::fs::read_dir(crates_dir) else {
        return all_sources;
    };

    for entry in entries.flatten() {
        let src_dir = entry.path().join("src");
        if !src_dir.is_dir() {
            continue;
        }
        scan_dir(&src_dir, &mut HashMap::new(), &mut all_sources);
    }

    all_sources
}

fn scan_dir(dir: &Path, annotations: &mut AnnotationMap, sources: &mut SourceMap) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, annotations, sources);
        } else if path.extension().is_some_and(|ext| ext == "rs")
            && let Ok(source) = std::fs::read_to_string(&path)
        {
            let file_path = path.to_string_lossy().to_string();
            annotations.extend(parse_annotations(&source));
            sources.extend(parse_test_locations(&source, &file_path));
        }
    }
}

fn main() {
    // "Running" headers go to stderr, test names to stdout — merge via shell redirect
    let output = Command::new("bash")
        .args(["-c", "cargo.exe test -- --list 2>&1"])
        .output()
        .expect("failed to run cargo.exe test -- --list");

    let combined = String::from_utf8_lossy(&output.stdout);

    let parsed = parse_cargo_output(&combined);

    let mut annotations = collect_annotations(Path::new("crates"));
    annotations.extend(collect_annotations(Path::new("tools")));

    let mut sources = collect_sources(Path::new("crates"));
    sources.extend(collect_sources(Path::new("tools")));

    let docs = convert_to_docs(&parsed, &annotations, &sources);
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
