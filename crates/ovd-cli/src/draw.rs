//! `ovd draw`: drawings from the Drawing Format Standard.

use ovd_draw::Files;
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
        ["run", ..] | ["build", ..] => draw_document_command(args),
        _ => draw_design(args),
    }
}

fn draw_document_command(args: &[&str]) -> Result<(), String> {
    match args {
        ["run", document] => run_document(document),
        ["build", document, "--out", out] => build_document(document, out),
        ["build", ..] => Err("usage: ovd draw build <document.json> --out <dir>".into()),
        ["run"] | ["run", _, ..] => {
            Err("give one drawing document: ovd draw run <document.json>".into())
        }
        _ => Err("unknown draw command; see `ovd --help`".into()),
    }
}

/// The runner of clause 17.3: print the result of one document as JSON.
fn run_document(path: &str) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let folder = Path::new(path)
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let result = ovd_draw::run_text(&text, &DirFiles(folder));
    let json = serde_json::to_string(&result.to_value()).map_err(|e| e.to_string())?;
    println!("{json}");
    Ok(())
}

/// Build the outputs of one drawing document: normalised content, DXF, SVG and PDF.
fn build_document(path: &str, out: &str) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let folder = Path::new(path)
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let files_in = DirFiles(folder);
    let norm = match ovd_draw::run_text(&text, &files_in) {
        ovd_draw::RunResult::Ok(n) => n,
        ovd_draw::RunResult::Errors(set) => {
            return Err(format!(
                "the document has errors: clause {}",
                set.into_iter().collect::<Vec<_>>().join(", clause ")
            ));
        }
    };
    // The document was read once by `run_text`; the DXF needs its model coordinates too.
    let document =
        ovd_draw::json::parse(&text).map_err(|_| "the document is not JSON".to_string())?;
    let dxf = ovd_draw::render::dxf::to_dxf(&document, &norm, &files_in);
    for warning in &dxf.warnings {
        eprintln!("warning: {warning}");
    }
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
    let files: [(String, Vec<u8>); 4] = [
        (format!("{stem}.normalised.json"), json.into_bytes()),
        (format!("{stem}.dxf"), dxf.text.into_bytes()),
        (
            format!("{stem}.svg"),
            ovd_draw::render::svg::to_svg(&norm, &ovd_draw::render::svg::SvgOptions::default())
                .into_bytes(),
        ),
        (format!("{stem}.pdf"), ovd_draw::render::pdf::to_pdf(&norm)),
    ];
    for (file, bytes) in files {
        let target = Path::new(out).join(&file);
        std::fs::write(&target, bytes)
            .map_err(|e| format!("cannot write {}: {e}", target.display()))?;
        println!("wrote {}", target.display());
    }
    Ok(())
}

// ---- drawings of a Design --------------------------------------------------------------------

/// The words that make a file name from an Option name: lower case letters and digits, one dash
/// between runs of anything else.
fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() {
        "option".to_string()
    } else {
        out
    }
}

/// What `ovd draw <file.vdesign>` was asked to do.
struct DrawArgs<'a> {
    profile: Option<&'a str>,
    out: Option<&'a str>,
    option: Option<&'a str>,
    documents_only: bool,
    file: &'a str,
}

fn parse_draw_args<'a>(args: &[&'a str]) -> Result<DrawArgs<'a>, String> {
    let (mut profile, mut out, mut option, mut documents_only, mut files) =
        (None, None, None, false, Vec::new());
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        match arg {
            "--profile" => profile = Some(*it.next().ok_or("--profile needs a file")?),
            "--out" => out = Some(*it.next().ok_or("--out needs a folder")?),
            "--option" => option = Some(*it.next().ok_or("--option needs an Option name")?),
            "--document" => documents_only = true,
            a if a.starts_with('-') => return Err(format!("unknown option `{a}`")),
            a => files.push(a),
        }
    }
    let [file] = files.as_slice() else {
        return Err("usage: ovd draw <file.vdesign> --out <dir> [--option <name>]  |  ovd draw --document <file.vdesign> [--option <name>]".into());
    };
    if documents_only == out.is_some() {
        return Err("give --out <dir> to write the drawings, or --document to print the drawing documents (not both)".into());
    }
    Ok(DrawArgs {
        profile,
        out,
        option,
        documents_only,
        file,
    })
}

fn draw_design(args: &[&str]) -> Result<(), String> {
    use ovd_format::RunOutcome;
    let a = parse_draw_args(args)?;
    let profile = super::load_profile(a.profile)?;
    let text =
        std::fs::read_to_string(a.file).map_err(|e| format!("cannot read {}: {e}", a.file))?;
    let checked = ovd_format::check(&text, &profile);
    let design = match (&checked.design, checked.report.is_ok()) {
        (Some(design), true) => design,
        _ => {
            let mut message = checked
                .report
                .errors
                .iter()
                .map(|e| format!("error: {e}"))
                .collect::<Vec<_>>()
                .join("\n");
            message.push_str(
                "\nnothing drawn. Fix the file, or run `ovd fill` if inputs are left out.",
            );
            return Err(message);
        }
    };
    let names: Vec<String> = design.options.iter().map(|o| o.name.clone()).collect();
    if let Some(wanted) = a.option
        && !names.iter().any(|n| n == wanted)
    {
        return Err(format!(
            "the Design has no Option named `{wanted}`; its Options are: {}",
            names.join(", ")
        ));
    }
    let runs = ovd_format::run_design(design);
    let mut documents = serde_json::Map::new();
    let mut used: Vec<String> = Vec::new();
    let mut failed = false;
    for run in runs.iter().filter(|r| a.option.is_none_or(|w| w == r.name)) {
        let views = match &run.outcome {
            RunOutcome::Results { .. } => ovd_format::ga_views_named(&run.outcome, Some(&run.name)),
            RunOutcome::NotAvailable(m) | RunOutcome::Failed(m) => Err(m.clone()),
        };
        // Issue #360: a Formation also gets its arrangement plan; a convoy gets only that.
        let arrangement = ovd_format::arrangement_plan_named(&run.outcome, Some(&run.name)).ok();
        let mut documents_of: Vec<(&str, serde_json::Value)> = match views {
            Ok(v) => v
                .all()
                .into_iter()
                .map(|(view, d)| (view, d.clone()))
                .collect(),
            Err(_) if arrangement.is_some() => Vec::new(),
            Err(m) => {
                eprintln!("Option '{}': no drawings: {m}", run.name);
                failed = true;
                continue;
            }
        };
        documents_of.extend(arrangement.map(|d| ("arrangement-plan", d)));
        // The tug of a Formation that includes it gets its own GA.
        documents_of.extend(
            ovd_format::tug_ga_named(&run.outcome, Some(&run.name))
                .ok()
                .map(|d| ("tug-ga", d)),
        );
        let mut stem = slug(&run.name);
        let mut n = 2;
        while used.contains(&stem) {
            stem = format!("{}-{n}", slug(&run.name));
            n += 1;
        }
        used.push(stem.clone());
        let mut set = serde_json::Map::new();
        for (view, document) in documents_of {
            if let Some(dir) = a.out {
                // A drawing the standard refuses does not stop the others (issue #660).
                match ovd_draw::run_value(&document, &ovd_draw::NoFiles) {
                    ovd_draw::RunResult::Ok(norm) => {
                        write_outputs(&document, &norm, &format!("{stem}-{view}"), dir)?;
                    }
                    ovd_draw::RunResult::Errors(set) => {
                        eprintln!(
                            "the drawing {stem}-{view} breaks the standard (clause {}): this is a fault in `ovd`, please report it",
                            set.into_iter().collect::<Vec<_>>().join(", clause ")
                        );
                        failed = true;
                    }
                }
            }
            set.insert(view.to_string(), document);
        }
        documents.insert(run.name.clone(), serde_json::Value::Object(set));
    }
    if a.documents_only {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::Value::Object(documents))
                .map_err(|e| e.to_string())?
        );
    }
    if failed {
        Err("some Options have no drawings".to_string())
    } else {
        Ok(())
    }
}

/// Write the drawing document, its normalised content, DXF, SVG and PDF.
fn write_outputs(
    document: &serde_json::Value,
    norm: &ovd_draw::Normalised,
    stem: &str,
    out: &str,
) -> Result<(), String> {
    let dxf = ovd_draw::render::dxf::to_dxf(document, norm, &ovd_draw::NoFiles);
    for warning in &dxf.warnings {
        eprintln!("warning: {stem}: {warning}");
    }
    std::fs::create_dir_all(out).map_err(|e| format!("cannot create {out}: {e}"))?;
    let json = |v: &serde_json::Value| {
        serde_json::to_string_pretty(v)
            .map(|t| t + "\n")
            .map_err(|e| e.to_string())
    };
    let files: [(String, Vec<u8>); 5] = [
        (format!("{stem}.drawing.json"), json(document)?.into_bytes()),
        (format!("{stem}.dxf"), dxf.text.into_bytes()),
        (
            format!("{stem}.normalised.json"),
            json(&norm.to_value())?.into_bytes(),
        ),
        (
            format!("{stem}.svg"),
            ovd_draw::render::svg::to_svg(norm, &ovd_draw::render::svg::SvgOptions::default())
                .into_bytes(),
        ),
        (format!("{stem}.pdf"), ovd_draw::render::pdf::to_pdf(norm)),
    ];
    for (file, bytes) in files {
        let target = Path::new(out).join(&file);
        std::fs::write(&target, bytes)
            .map_err(|e| format!("cannot write {}: {e}", target.display()))?;
        println!("wrote {}", target.display());
    }
    Ok(())
}
