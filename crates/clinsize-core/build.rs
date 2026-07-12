//! Embeds every `validation/**/cases.json` into the crate so validation
//! reports work in packaged builds, where the repository's `validation/`
//! directory does not exist on disk.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let validation_root = manifest_dir.join("../../validation");
    let validation_root = validation_root
        .canonicalize()
        .unwrap_or(validation_root.clone());
    println!("cargo:rerun-if-changed={}", validation_root.display());

    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    collect_case_files(&validation_root, &validation_root, &mut entries);
    entries.sort();

    let mut code = String::from(
        "/// `(method_id, cases.json content)` pairs embedded at compile time.\n\
         pub static EMBEDDED_CASES: &[(&str, &str)] = &[\n",
    );
    for (method_id, path) in &entries {
        code.push_str(&format!(
            "    ({method_id:?}, include_str!({:?})),\n",
            path.display()
        ));
    }
    code.push_str("];\n");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("embedded_cases.rs");
    fs::write(&out_path, code).expect("write embedded_cases.rs");
}

fn collect_case_files(root: &Path, dir: &Path, entries: &mut Vec<(String, PathBuf)>) {
    let Ok(children) = fs::read_dir(dir) else {
        return;
    };
    for child in children.flatten() {
        let path = child.path();
        if path.is_dir() {
            // Directory mtime changes when files are added or removed;
            // per-file lines below track content edits.
            println!("cargo:rerun-if-changed={}", path.display());
            collect_case_files(root, &path, entries);
        } else if path.file_name().is_some_and(|name| name == "cases.json") {
            println!("cargo:rerun-if-changed={}", path.display());
            let parent = path.parent().expect("cases.json has a parent");
            let relative = parent.strip_prefix(root).expect("parent under root");
            let method_id = relative
                .iter()
                .map(|part| part.to_string_lossy().replace('-', "_"))
                .collect::<Vec<_>>()
                .join(".");
            entries.push((method_id, path));
        }
    }
}
