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
