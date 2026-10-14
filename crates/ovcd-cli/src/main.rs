//! `ovcd`: the Open Vessel Concept Drawings command-line tool. A thin layer over `ovcd-format`.
//!
//! `ovcd new` writes a starter Design, `ovcd fill` writes the left-out inputs of a hand-written
//! Design in from the Defaults Profile, `ovcd validate` checks one without running it, and
//! `ovcd schema` prints the input schema, `ovcd profile` prints the built-in Defaults Profile for a
//! user to copy, and `ovcd run` computes a saved Design and prints a summary (or the full results
//! as JSON).

mod draw;

use ovcd_format::Profile;
use std::process::ExitCode;

const HELP: &str = "\
ovcd - Open Vessel Concept Drawings

Usage:
  ovcd new [--profile <profile.json>] [<file.vdesign>]
      Write a starter Design with one Option. Its six required inputs (Cargo, Hull Form,
      Formation, length, beam, moulded depth) are examples to edit; every other input comes
      from the Defaults Profile and is marked \"default\". Prints to stdout when no file is given.
  ovcd fill [--profile <profile.json>] <in.vdesign> [<out.vdesign>]
      Write every input a hand-written Design leaves out in, as used and marked \"default\",
      from the Defaults Profile. Prints to stdout when no output file is given; never
      overwrites a file. Writes nothing and exits 1 if there is any hard error.
  ovcd validate [--profile <profile.json>] [--inputs] <file.vdesign>
      List hard errors and soft warnings without running. --inputs also lists every input
      as used, typed or default. Exits 1 if there is any hard error. A file must store
      every input as used, so a left-out input is a hard error: run `ovcd fill` first.
  ovcd run [--profile <profile.json>] [--json] <file.vdesign>
      Compute every Option of a Design and print a short summary. --json prints the full
      results instead. Each Option's results hash is compared with the one saved in the file;
      a difference is a warning that results changed since the file was saved with an engine
      version. A pushed convoy prints \"convoy results not yet available\" and no numbers.
      Warnings go to stderr. Exits 1 if the file has a hard error or an Option cannot be computed.
  ovcd schema [--profile <profile.json>]
      Print the input schema as JSON: every input with its kind, unit, choices, group, default,
      hard and soft ranges and basis, from the Defaults Profile. Use it to build valid Designs.
  ovcd profile
      Print the built-in Defaults Profile. Copy it, edit it, and pass it with --profile.
  ovcd draw run <document.json>
      The conformance runner of the Drawing Format Standard (clause 17.3): read one drawing
      document and print the result as JSON, either the normalised content or the list of
      error clauses. It reads only the document and the files the document names.
  ovcd draw build <document.json> --out <dir>
      Build the outputs of one drawing document: the normalised content, an SVG and a PDF.
  ovcd --help | --version

All files and output are SI.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        [] | ["--help"] | ["-h"] => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        ["--version"] | ["-V"] => {
            println!("ovcd {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        ["run", rest @ ..] => match run(rest) {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => ExitCode::FAILURE,
            Err(msg) => usage_error("run", &msg),
        },
        ["draw", rest @ ..] => report(draw::draw(rest), "draw"),
        ["schema", rest @ ..] => report(schema(rest), "schema"),
        ["profile"] => {
            print!("{}", Profile::built_in_text());
            ExitCode::SUCCESS
        }
        ["new", rest @ ..] => report(new(rest), "new"),
        ["fill", rest @ ..] => report(fill(rest), "fill"),
        ["validate", rest @ ..] => match validate(rest) {
            Ok(ok) => {
                if ok {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                }
            }
            Err(msg) => usage_error("validate", &msg),
        },
        _ => {
            eprint!("ovcd: unknown command\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

fn report(result: Result<(), String>, command: &str) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("ovcd {command}: {msg}");
            ExitCode::FAILURE
        }
    }
}

fn usage_error(command: &str, msg: &str) -> ExitCode {
    eprint!("ovcd {command}: {msg}\n\n{HELP}");
    ExitCode::from(2)
}

/// Flags shared by the design commands.
struct Flags<'a> {
    profile: Option<&'a str>,
    inputs: bool,
    json: bool,
    files: Vec<&'a str>,
}

fn parse_flags<'a>(
    args: &[&'a str],
    allow_inputs: bool,
    allow_json: bool,
) -> Result<Flags<'a>, String> {
    let mut flags = Flags {
        profile: None,
        inputs: false,
        json: false,
        files: Vec::new(),
    };
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        match arg {
            "--profile" => {
                flags.profile = Some(it.next().ok_or("--profile needs a file")?);
            }
            "--inputs" if allow_inputs => flags.inputs = true,
            "--json" if allow_json => flags.json = true,
            a if a.starts_with('-') => return Err(format!("unknown option `{a}`")),
            a => flags.files.push(a),
        }
    }
    Ok(flags)
}

fn load_profile(path: Option<&str>) -> Result<Profile, String> {
    match path {
        None => Ok(Profile::built_in()),
        Some(p) => {
            let text =
                std::fs::read_to_string(p).map_err(|e| format!("cannot read profile {p}: {e}"))?;
            Profile::from_json(&text).map_err(|e| format!("{p}: {e}"))
        }
    }
}

fn new(args: &[&str]) -> Result<(), String> {
    let flags = parse_flags(args, false, false)?;
    let profile = load_profile(flags.profile)?;
    let text = ovcd_format::new_design(&profile)?.to_text();
    match flags.files.as_slice() {
        [] => {
            print!("{text}");
            Ok(())
        }
        [path] => write_new(path, &text),
        _ => Err("give at most one file".into()),
    }
}

fn schema(args: &[&str]) -> Result<(), String> {
    let flags = parse_flags(args, false, false)?;
    if !flags.files.is_empty() {
        return Err("takes no file; use --profile <profile.json>".into());
    }
    let profile = load_profile(flags.profile)?;
    let text = serde_json::to_string_pretty(&ovcd_format::input_schema(&profile))
        .map_err(|e| e.to_string())?;
    println!("{text}");
    Ok(())
}

/// Write `text` to a file that must not exist yet.
fn write_new(path: &str, text: &str) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::AlreadyExists => {
                format!("{path} already exists; not overwritten")
            }
            _ => format!("cannot write {path}: {e}"),
        })?;
    std::io::Write::write_all(&mut file, text.as_bytes())
        .map_err(|e| format!("cannot write {path}: {e}"))?;
    eprintln!("wrote {path}");
    Ok(())
}

fn fill(args: &[&str]) -> Result<(), String> {
    let flags = parse_flags(args, false, false)?;
    let (input, output) = match flags.files.as_slice() {
        [input] => (*input, None),
        [input, output] => (*input, Some(*output)),
        _ => return Err("give one .vdesign file, and optionally an output file".into()),
    };
    let profile = load_profile(flags.profile)?;
    let text = std::fs::read_to_string(input).map_err(|e| format!("cannot read {input}: {e}"))?;
    let filled = ovcd_format::fill(&text, &profile);
    let r = &filled.report;
    for e in &r.errors {
        eprintln!("error: {e}");
    }
    for w in &r.warnings {
        eprintln!("warning: {w}");
    }
    let design = match (&filled.design, r.is_ok()) {
        (Some(design), true) => design,
        _ => {
            return Err(format!(
                "{} error(s); nothing written",
                r.errors.len().max(1)
            ));
        }
    };
    let text = design.to_text();
    match output {
        None => {
            print!("{text}");
            Ok(())
        }
        Some(path) => write_new(path, &text),
    }
}

/// Returns whether the file has no hard errors.
fn validate(args: &[&str]) -> Result<bool, String> {
    let flags = parse_flags(args, true, false)?;
    let [path] = flags.files.as_slice() else {
        return Err("give exactly one .vdesign file".into());
    };
    let profile = load_profile(flags.profile)?;
    let checked = match std::fs::read_to_string(path) {
        Ok(text) => ovcd_format::check(&text, &profile),
        Err(e) => {
            println!("error: cannot read {path}: {e}");
            println!("1 error(s), 0 warning(s)");
            return Ok(false);
        }
    };
    if flags.inputs
        && let Some(design) = &checked.design
    {
        for option in &design.options {
            println!("Option '{}':", option.name);
            for (name, input) in &option.inputs {
                println!(
                    "  {name} = {} ({})",
                    ovcd_format::vdesign::show(&input.value),
                    input.source.as_str()
                );
            }
        }
    }
    let r = &checked.report;
    for e in &r.errors {
        println!("error: {e}");
    }
    for w in &r.warnings {
        println!("warning: {w}");
    }
    println!(
        "{} error(s), {} warning(s)",
        r.errors.len(),
        r.warnings.len()
    );
    Ok(r.is_ok())
}

/// Run a saved Design. Returns whether every Option was computed and the file had no hard error.
fn run(args: &[&str]) -> Result<bool, String> {
    use ovcd_format::{HashStatus, RunOutcome};
    let flags = parse_flags(args, false, true)?;
    let [path] = flags.files.as_slice() else {
        return Err("give exactly one .vdesign file".into());
    };
    let profile = load_profile(flags.profile)?;
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: cannot read {path}: {e}");
            return Ok(false);
        }
    };
    let checked = ovcd_format::check(&text, &profile);
    for w in &checked.report.warnings {
        eprintln!("warning: {w}");
    }
    let design = match (&checked.design, checked.report.is_ok()) {
        (Some(design), true) => design,
        _ => {
            for e in &checked.report.errors {
                eprintln!("error: {e}");
            }
            eprintln!(
                "{} error(s); nothing run. Fix the file, or run `ovcd fill` if inputs are left out.",
                checked.report.errors.len().max(1)
            );
            return Ok(false);
        }
    };
    let runs = ovcd_format::run_design(design);
    for r in &runs {
        if let HashStatus::Differs { stored, computed } = &r.hash_status {
            eprintln!(
                "warning: {}",
                ovcd_format::hash_warning(&r.name, &design.engine_version, stored, computed)
            );
        }
    }
    let all_computed = runs
        .iter()
        .all(|r| matches!(r.outcome, RunOutcome::Results { .. }));
    if flags.json {
        let options: Vec<serde_json::Value> = runs
            .iter()
            .map(|r| match &r.outcome {
                RunOutcome::Results { results, hash } => serde_json::json!({
                    "name": r.name,
                    "results_hash": hash,
                    "hash_status": r.hash_status.as_str(),
                    "results": results,
                }),
                RunOutcome::NotAvailable(m) | RunOutcome::Failed(m) => serde_json::json!({
                    "name": r.name,
                    "results": serde_json::Value::Null,
                    "message": m,
                }),
            })
            .collect();
        let out = serde_json::json!({
            "engine_version": env!("CARGO_PKG_VERSION"),
            "file_engine_version": design.engine_version,
            "options": options,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?
        );
    } else {
        for r in &runs {
            match &r.outcome {
                RunOutcome::Results { results, hash } => {
                    for line in ovcd_format::summary_lines(&r.name, results) {
                        println!("{line}");
                    }
                    let note = match r.hash_status {
                        HashStatus::Matches => "matches the file",
                        HashStatus::Differs { .. } => "DIFFERS from the file",
                        HashStatus::NotStored => "none saved in the file",
                    };
                    println!("  Results hash {} ({note})", &hash[..12]);
                }
                RunOutcome::NotAvailable(m) => println!("Option '{}': {m}", r.name),
                RunOutcome::Failed(m) => println!("Option '{}': cannot be computed: {m}", r.name),
            }
        }
    }
    Ok(all_computed)
}
