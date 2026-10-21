//! Label: property
//!
//! The kernel is domain-free: it depends on no crate of the tool that uses it, and its source
//! names none. A path or git dependency on a domain crate would have to be in `Cargo.toml`.

use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// The dependency names of every `[…dependencies]` table of the manifest.
fn dependencies(manifest: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_deps = false;
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_deps = line.trim_matches(['[', ']']).ends_with("dependencies");
            continue;
        }
        if in_deps
            && let Some((name, _)) = line.split_once('=')
            && !line.starts_with('#')
        {
            out.push(name.trim().to_string());
        }
    }
    out
}

#[test]
fn the_kernel_depends_on_no_domain_crate() {
    let manifest = std::fs::read_to_string(Path::new(ROOT).join("Cargo.toml")).unwrap();
    let deps = dependencies(&manifest);
    assert_eq!(deps, ["serde_json", "sha2"], "{deps:?}");
    assert!(
        !manifest.contains("ovcd"),
        "the manifest names a domain crate"
    );
}

#[test]
fn the_kernel_source_names_no_domain_crate() {
    for dir in ["src", "tests"] {
        for entry in std::fs::read_dir(Path::new(ROOT).join(dir)).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().is_some_and(|n| n == "independence.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            for word in ["ovcd_core", "ovcd_format", "ovcd-core", "ovcd-format"] {
                assert!(!text.contains(word), "{} names `{word}`", path.display());
            }
        }
    }
}
