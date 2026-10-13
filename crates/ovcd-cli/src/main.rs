//! `ovcd`: the Open Vessel Concept Drawings command-line tool. A thin layer over `ovcd-format`.
//!
//! `ovcd new` writes a starter Design, `ovcd validate` checks one without running it, and
//! `ovcd profile` prints the built-in Defaults Profile for a user to copy. `run` arrives in a
//! later ticket. The hidden `ovcd eval <entry-point>` reads one parity case input as JSON on stdin
//! and writes the result as JSON on stdout. It exists for parity only and is removed with the
//! Reference Results.

use ovcd_format::Profile;
use std::io::Read;
use std::process::ExitCode;

const HELP: &str = "\
ovcd - Open Vessel Concept Drawings

Usage:
  ovcd new [--profile <profile.json>] [<file.vdesign>]
      Write a starter Design with one Option. Its six required inputs (Cargo, Hull Form,
      Formation, length, beam, moulded depth) are examples to edit; every other input comes
      from the Defaults Profile and is marked \"default\". Prints to stdout when no file is given.
  ovcd validate [--profile <profile.json>] [--inputs] <file.vdesign>
      List hard errors and soft warnings without running. --inputs also lists every input
      as used, typed or default. Exits 1 if there is any hard error.
  ovcd profile
      Print the built-in Defaults Profile. Copy it, edit it, and pass it with --profile.
  ovcd --help | --version

All files and output are SI. `ovcd run` is not available yet.
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
        ["eval", entry_point] => match eval(entry_point) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(msg) => {
                eprintln!("ovcd eval: {msg}");
                ExitCode::FAILURE
            }
        },
        ["profile"] => {
            print!("{}", Profile::built_in_text());
            ExitCode::SUCCESS
        }
        ["new", rest @ ..] => report(new(rest), "new"),
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
    files: Vec<&'a str>,
}

fn parse_flags<'a>(args: &[&'a str], allow_inputs: bool) -> Result<Flags<'a>, String> {
    let mut flags = Flags {
        profile: None,
        inputs: false,
        files: Vec::new(),
    };
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        match arg {
            "--profile" => {
                flags.profile = Some(it.next().ok_or("--profile needs a file")?);
            }
            "--inputs" if allow_inputs => flags.inputs = true,
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
    let flags = parse_flags(args, false)?;
    let profile = load_profile(flags.profile)?;
    let text = ovcd_format::new_design(&profile).to_text();
    match flags.files.as_slice() {
        [] => {
            print!("{text}");
            Ok(())
        }
        [path] => {
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
        _ => Err("give at most one file".into()),
    }
}

/// Returns whether the file has no hard errors.
fn validate(args: &[&str]) -> Result<bool, String> {
    let flags = parse_flags(args, true)?;
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

fn eval(entry_point: &str) -> Result<String, String> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|e| format!("cannot read stdin: {e}"))?;
    let input: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("stdin is not JSON: {e}"))?;
    let output = ovcd_format::eval(entry_point, &input).map_err(|e| e.to_string())?;
    Ok(output.to_string())
}
