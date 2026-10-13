//! `ovcd`: the Open Vessel Concept Drawings command-line tool. A thin layer over `ovcd-format`.
//!
//! The `new`, `validate` and `run` commands arrive in later tickets. The hidden
//! `ovcd eval <entry-point>` reads one parity case input as JSON on stdin and writes the result as
//! JSON on stdout. It exists for parity only and is removed with the Reference Results.

use std::io::Read;
use std::process::ExitCode;

const HELP: &str = "\
ovcd - Open Vessel Concept Drawings

Usage: ovcd [--help | --version]

Design commands (new, validate, run) are not available yet.
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
        _ => {
            eprint!("ovcd: unknown command\n\n{HELP}");
            ExitCode::from(2)
        }
    }
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
