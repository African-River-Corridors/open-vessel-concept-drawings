//! `ovcd draw`: drawings from the Drawing Format Standard.

use ovcd_draw::Files;
use std::path::{Path, PathBuf};

/// Reads the files that a drawing document names, relative to the folder of the document.
pub struct DirFiles(pub PathBuf);

impl Files for DirFiles {
    fn read(&self, relative_path: &str) -> Option<String> {
        std::fs::read_to_string(self.0.join(relative_path)).ok()
    }
}

pub fn draw(args: &[&str]) -> Result<(), String> {
    match args {
        ["run", document] => run_document(document),
        ["build", document, "--out", out] => build_document(document, out),
        ["build", ..] => Err("usage: ovcd draw build <document.json> --out <dir>".into()),
        ["run"] | ["run", _, ..] => {
            Err("give one drawing document: ovcd draw run <document.json>".into())
        }
        _ => Err("unknown draw command; see `ovcd --help`".into()),
    }
}

/// The runner of clause 17.3: print the result of one document as JSON.
fn run_document(path: &str) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let folder = Path::new(path)
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let result = ovcd_draw::run_text(&text, &DirFiles(folder));
    let json = serde_json::to_string(&result.to_value()).map_err(|e| e.to_string())?;
    println!("{json}");
    Ok(())
}

/// Build the outputs of one drawing document: normalised content, SVG and PDF.
fn build_document(path: &str, out: &str) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let folder = Path::new(path)
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let norm = match ovcd_draw::run_text(&text, &DirFiles(folder)) {
        ovcd_draw::RunResult::Ok(n) => n,
        ovcd_draw::RunResult::Errors(set) => {
            return Err(format!(
                "the document has errors: clause {}",
                set.into_iter().collect::<Vec<_>>().join(", clause ")
            ));
        }
    };
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("drawing");
    let stem = name
        .strip_suffix(".drawing.json")
        .or_else(|| name.strip_suffix(".json"))
        .unwrap_or(name);
    std::fs::create_dir_all(out).map_err(|e| format!("cannot create {out}: {e}"))?;
    let json = serde_json::to_string_pretty(&norm.to_value()).map_err(|e| e.to_string())? + "\n";
    let files: [(String, Vec<u8>); 3] = [
        (format!("{stem}.normalised.json"), json.into_bytes()),
        (
            format!("{stem}.svg"),
            ovcd_draw::render::svg::to_svg(&norm, &ovcd_draw::render::svg::SvgOptions::default())
                .into_bytes(),
        ),
        (format!("{stem}.pdf"), ovcd_draw::render::pdf::to_pdf(&norm)),
    ];
    for (file, bytes) in files {
        let target = Path::new(out).join(&file);
        std::fs::write(&target, bytes)
            .map_err(|e| format!("cannot write {}: {e}", target.display()))?;
        println!("wrote {}", target.display());
    }
    Ok(())
}
