//! `ovcd`: the Open Vessel Concept Drawings command-line tool. A thin layer over `ovcd-format`.
//!
//! `ovcd new` writes a starter Design, `ovcd fill` writes the left-out inputs of a hand-written
//! Design in from the Defaults Profile, `ovcd validate` checks one without running it, and
//! `ovcd schema` prints the input schema, `ovcd profile` prints the built-in Defaults Profile for a
//! user to copy, and `ovcd run` computes a saved Design and prints a summary (or the full results
//! as JSON).

mod draw;

use ovcd_format::Profile;
use ovcd_format::rule_set::{Library, describe};
use std::process::ExitCode;

const HELP: &str = "\
ovcd - Open Vessel Concept Drawings

Usage:
  ovcd new [--profile <profile.json>] [--class <class id>] [<file.vdesign>]
      Write a starter Design with one Option. Its five required inputs (Cargo, Hull Form,
      Formation, length, beam) are examples to edit, and so is its moulded depth, which you may
      delete: a left-out depth is the beam times the depth-to-beam ratio of the Cargo, from the
      Defaults Profile. Every other input comes from the Defaults Profile and is marked \"default\".
      Prints to stdout when no file is given.
      --class defines the Option by a standard Vessel Class (see `ovcd schema`, input
      `vessel_class`): length, beam and Formation come from the class and are marked \"class\".
  ovcd hull import [--profile <profile.json>] <file.csv> --option <name> <in.vdesign> [<out.vdesign>]
      Import an offsets table (spec 0008): the CSV header station_x_m,y_m,z_m, one row per point
      from the keel round to the deck on the centre line, stations in ascending x, blank lines
      allowed. The Option becomes an offsets hull with that table, and its length, beam and depth
      take the table's values. Prints to stdout when no output file is given; never overwrites.
  ovcd fill [--profile <profile.json>] [--rule-set <file>]... [--add-rule-set <id>@<version>]...
            [--remove-rule-set <id>]... [--as-new] <in.vdesign> [<out.vdesign>]
      Write every input a hand-written Design leaves out in, as used and marked \"default\",
      from the Defaults Profile. Prints to stdout when no output file is given; never
      overwrites a file. Writes nothing and exits 1 if there is any hard error.
      An input that was added after the first files (see ADR 0007) takes the fixed value the
      engine used before it existed, so an old file keeps its results. --as-new takes the
      Defaults Profile value instead, as a new Design does: use it for a file you write now.
      --add-rule-set adds a Rule Set to every Option, marked \"typed\" (a loaded --rule-set file
      may supply it); --remove-rule-set removes one. The `ovcd` set stays, and so does a set that a
      selected set includes.
  ovcd validate [--profile <profile.json>] [--rule-set <file>]... [--inputs] <file.vdesign>
      List hard errors and soft warnings without running. --inputs also lists every input
      as used, typed or default. Exits 1 if there is any hard error. A file must store
      every input as used, so a left-out input is a hard error: run `ovcd fill` first.
  ovcd run [--profile <profile.json>] [--rule-set <file>]... [--json] <file.vdesign>
      Compute every Option of a Design and print a short summary. --json prints the full
      results instead. Each Option's results hash is compared with the one saved in the file;
      a difference is a warning that the results differ from the hash saved in the file (an
      engine change, or an input edited by hand, can cause it). A pushed convoy prints its totals
      (cargo, lightship, deadweight, overall size) and then the numbers of one barge; the pusher
      is not in the totals yet. In --json each Option has a \"status\": \"results\", \"not_yet_supported\"
      or \"failed\".
      Warnings go to stderr. Exits 0 for a valid file, even if some Options are not yet
      supported. Exits 1 if the file has a hard error or an Option fails to compute.
  ovcd schema [--profile <profile.json>]
      Print the input schema as JSON: every input with its kind, unit, choices, group, default,
      hard and soft ranges and basis, from the Defaults Profile. Use it to build valid Designs.
  ovcd profile
      Print the built-in Defaults Profile. Copy it, edit it, and pass it with --profile.
  ovcd rules [--rule-set <file>]...
      List the Rule Sets: the built-in sets, then each file loaded with --rule-set, with
      version, status, number of Rules and content hash.
  ovcd rules --measures
      Print the Measure registry: each Measure id, its object, unit, arguments and method
      status. Input Measures are `input.<name>` for an input of the schema.
  ovcd rules --provenance
      The CI gate of the built-in Rule Sets: every version in RELEASED.json is built in with
      its hash; each later version goes up by at least its class (patch, minor, major); a rule
      id keeps its Measure and direction and is never reused once dropped; every regulation
      or class Rule cites a paragraph. Prints the unverified blocking Rules of each set.
  ovcd rules --released-entries | --release-check
      For a release: print the RELEASED.json entries of every built-in set version, or fail
      if a built-in version is not listed yet.
  ovcd rules show <id>[@<version>] [--rule-set <file>]...
      Print one Rule Set with each Rule, its test, its citation and its verification status.
      With no version, the highest version.
  ovcd draw run <document.json>
      The conformance runner of the Drawing Format Standard (clause 17.3): read one drawing
      document and print the result as JSON, either the normalised content or the list of
      error clauses. It reads only the document and the files the document names.
  ovcd draw build <document.json> --out <dir>
      Build the outputs of one drawing document: the normalised content, a DXF, an SVG and a PDF.
  ovcd draw [--profile <profile.json>] [--option <name>] --out <dir> <file.vdesign>
      Draw the side elevation, plan and midship section of each Option of a Design (or of the
      one Option named). Writes, for each view, the drawing document, its normalised content,
      a DXF, an SVG and a PDF. A pushed convoy, or an Option that cannot be computed, gets no drawings.
  ovcd draw [--profile <profile.json>] [--option <name>] --document <file.vdesign>
      Print the drawing documents only, as JSON, so that they can be built with another
      implementation of the standard.
  ovcd export [--profile <profile.json>] --format gltf [--option <name>] (-o <file> | --hash) <file.vdesign>
      Write the Vessel Model of an Option (spec 0008, issue #460): `--format gltf` is a glTF 2.0
      binary (.glb) with one node for each layer and each Space. `-o` names the file; `--option`
      picks the Option (the only one, when there is one). `--hash` prints the SHA-256 of each
      Option's bytes instead, for the web parity check.
  ovcd mesh [--profile <profile.json>] --hash <file.vdesign>
      Print the mesh hash of each Option (spec 0008, issue #456): the SHA-256 of the meshes of its
      Vessel Model (hull, decks, bulkheads, Spaces, waterlines). The web parity check compares it
      with the WebAssembly build. It is not the results hash. A convoy, or an Option that cannot
      be computed, prints `-` and the reason.
  ovcd --help | --version

--rule-set <file> (repeatable) loads a Rule Set file at run time, after the built-in sets
and in the order given. A file is refused with a message when it is malformed, or when it has
the id and version of a loaded set but other content: a changed Rule Set needs a new version.
`ovcd run` checks each Option against the Rule Sets it stores (`rule_sets`, at the stored
versions) and each loaded set (a loaded set with the id of a stored set replaces it for the
run); a stored version the engine lacks refuses the Option, naming the set; --json gives the Rule Trace as
`rule_trace` in the results. fill and validate only check the files they are given.

All files and output are SI, except speeds, which are in knots.
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
        ["mesh", rest @ ..] => report(mesh(rest), "mesh"),
        ["export", rest @ ..] => report(export(rest), "export"),
        ["schema", rest @ ..] => report(schema(rest), "schema"),
        ["profile"] => {
            print!("{}", Profile::built_in_text());
            ExitCode::SUCCESS
        }
        ["new", rest @ ..] => report(new(rest), "new"),
        ["rules", rest @ ..] => report(rules(rest), "rules"),
        ["fill", rest @ ..] => report(fill(rest), "fill"),
        ["hull", "import", rest @ ..] => report(hull_import(rest), "hull import"),
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
    rule_sets: Vec<&'a str>,
    add_rule_sets: Vec<&'a str>,
    remove_rule_sets: Vec<&'a str>,
    inputs: bool,
    json: bool,
    as_new: bool,
    files: Vec<&'a str>,
}

fn parse_flags<'a>(
    args: &[&'a str],
    allow_inputs: bool,
    allow_json: bool,
    allow_as_new: bool,
) -> Result<Flags<'a>, String> {
    let mut flags = Flags {
        profile: None,
        rule_sets: Vec::new(),
        add_rule_sets: Vec::new(),
        remove_rule_sets: Vec::new(),
        inputs: false,
        json: false,
        as_new: false,
        files: Vec::new(),
    };
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        match arg {
            "--profile" => {
                flags.profile = Some(it.next().ok_or("--profile needs a file")?);
            }
            // The design commands that check a Design take Rule Sets: fill, validate and run.
            "--rule-set" if allow_inputs || allow_json || allow_as_new => {
                flags
                    .rule_sets
                    .push(it.next().ok_or("--rule-set needs a file")?);
            }
            // Issue #275: only `fill` writes a Design, so only `fill` takes them.
            "--add-rule-set" if allow_as_new => {
                flags
                    .add_rule_sets
                    .push(it.next().ok_or("--add-rule-set needs <id>@<version>")?);
            }
            "--remove-rule-set" if allow_as_new => {
                flags
                    .remove_rule_sets
                    .push(it.next().ok_or("--remove-rule-set needs a set id")?);
            }
            "--inputs" if allow_inputs => flags.inputs = true,
            "--json" if allow_json => flags.json = true,
            "--as-new" if allow_as_new => flags.as_new = true,
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

/// The built-in Rule Sets, then each file in order. Every `includes` target must be loaded once all
/// files are in. Loader warnings go to stderr.
/// Returns the library and the `(id, version)` of each loaded file, in order.
fn load_rule_sets(paths: &[&str]) -> Result<(Library, Vec<(String, String)>), String> {
    let mut lib = Library::built_in();
    let mut loaded = Vec::new();
    for p in paths {
        let text =
            std::fs::read_to_string(p).map_err(|e| format!("cannot read Rule Set {p}: {e}"))?;
        let key = lib
            .add_text(&text)
            .map_err(|e| format!("Rule Set {p}: {e}"))?;
        let (id, version) = key.split_once('@').expect("a set key is id@version");
        loaded.push((id.to_string(), version.to_string()));
    }
    lib.check()?;
    for w in &lib.warnings {
        eprintln!("warning: {w}");
    }
    Ok((lib, loaded))
}

fn rules(args: &[&str]) -> Result<(), String> {
    let mut files: Vec<&str> = Vec::new();
    let mut rest: Vec<&str> = Vec::new();
    let mut measures = false;
    let mut special: Option<&str> = None;
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        match arg {
            "--rule-set" => files.push(it.next().ok_or("--rule-set needs a file")?),
            "--measures" => measures = true,
            "--provenance" | "--released-entries" | "--release-check" => special = Some(arg),
            a if a.starts_with('-') => return Err(format!("unknown option `{a}`")),
            a => rest.push(a),
        }
    }
    if measures {
        if !(files.is_empty() && rest.is_empty()) {
            return Err("--measures takes nothing else".into());
        }
        print!("{}", ovcd_format::trace::measures_text());
        return Ok(());
    }
    if let Some(flag) = special {
        if !(files.is_empty() && rest.is_empty()) {
            return Err(format!(
                "{flag} takes nothing else: it checks the built-in sets"
            ));
        }
        return rule_set_gate(flag);
    }
    let (lib, _) = load_rule_sets(&files)?;
    match rest.as_slice() {
        [] => {
            if lib.sets().is_empty() {
                println!("no Rule Sets");
            }
            for s in lib.sets() {
                println!(
                    "{}@{}  {}  {} Rules  {}  {}",
                    s.id,
                    s.version,
                    s.status.key(),
                    s.rules.len(),
                    &s.hash[..12],
                    s.title
                );
            }
            Ok(())
        }
        ["show", spec] => {
            let set = lib
                .lookup(spec)
                .ok_or_else(|| format!("no Rule Set `{spec}`; `ovcd rules` lists them"))?;
            print!("{}", describe(set));
            Ok(())
        }
        _ => Err("use `ovcd rules` or `ovcd rules show <id>[@<version>]`".into()),
    }
}

/// The built-in Rule Sets' CI gate (`--provenance`), and the release steps (#273).
fn rule_set_gate(flag: &str) -> Result<(), String> {
    use ovcd_format::rule_set::{RELEASED, gate, released};
    let lib = Library::built_in();
    match flag {
        "--provenance" => {
            let (errors, report) = gate(&lib);
            print!("{report}");
            if errors.is_empty() {
                println!("rule set gate: ok");
                Ok(())
            } else {
                for e in &errors {
                    eprintln!("error: {e}");
                }
                Err(format!(
                    "{} problem(s) in the built-in Rule Sets",
                    errors.len()
                ))
            }
        }
        "--released-entries" => {
            // The entries a version-bump PR adds to RELEASED.json: every built-in set version.
            let list: Vec<serde_json::Value> = lib
                .sets()
                .iter()
                .map(|s| serde_json::json!({"id": s.id, "version": s.version, "hash": s.hash}))
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&list).map_err(|e| e.to_string())?
            );
            Ok(())
        }
        _ => {
            // At a release: every built-in set version must be listed as released.
            let listed = released(RELEASED)?;
            let missing: Vec<String> = lib
                .sets()
                .iter()
                .filter(|s| {
                    !listed
                        .iter()
                        .any(|r| r.id == s.id && r.version == s.version)
                })
                .map(|s| s.key())
                .collect();
            if missing.is_empty() {
                println!("every built-in Rule Set version is in RELEASED.json");
                Ok(())
            } else {
                Err(format!(
                    "not in RELEASED.json yet: {}; add them with `ovcd rules --released-entries`",
                    missing.join(", ")
                ))
            }
        }
    }
}

fn new(args: &[&str]) -> Result<(), String> {
    // `--class <id>` defines the starter Option by a standard Vessel Class (issue #94).
    let mut class: Option<&str> = None;
    let mut rest: Vec<&str> = Vec::new();
    let mut it = args.iter();
    while let Some(&arg) = it.next() {
        if arg == "--class" {
            class = Some(it.next().ok_or("--class needs a Vessel Class id")?);
        } else {
            rest.push(arg);
        }
    }
    let flags = parse_flags(&rest, false, false, false)?;
    let profile = load_profile(flags.profile)?;
    let text = match class {
        None => ovcd_format::new_design(&profile)?.to_text(),
        Some(id) => {
            let mut session = ovcd_format::DesignSession::new_design(profile)?;
            // The starter's example depth is for the starter hull; the class's beam gets the rule's.
            session.reset_input("Option 1", "moulded_depth_m")?;
            session.set_input("Option 1", "vessel_class", serde_json::json!(id))?;
            session.save()?
        }
    };
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
    let flags = parse_flags(args, false, false, false)?;
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
    let flags = parse_flags(args, false, false, true)?;
    let (input, output) = match flags.files.as_slice() {
        [input] => (*input, None),
        [input, output] => (*input, Some(*output)),
        _ => return Err("give one .vdesign file, and optionally an output file".into()),
    };
    let profile = load_profile(flags.profile)?;
    // `fill` computes nothing; the sets are read for `--add-rule-set` and `--remove-rule-set`.
    let (library, _) = load_rule_sets(&flags.rule_sets)?;
    let text = std::fs::read_to_string(input).map_err(|e| format!("cannot read {input}: {e}"))?;
    let filled = if flags.as_new {
        ovcd_format::fill_as_new(&text, &profile)
    } else {
        ovcd_format::fill(&text, &profile)
    };
    let r = &filled.report;
    for e in &r.errors {
        eprintln!("error: {e}");
    }
    for w in &r.warnings {
        eprintln!("warning: {w}");
    }
    let mut design = match (&filled.design, r.is_ok()) {
        (Some(design), true) => design.clone(),
        _ => {
            return Err(format!(
                "{} error(s); nothing written",
                r.errors.len().max(1)
            ));
        }
    };
    // Issue #275: add or remove Rule Sets in every Option; a refusal writes nothing.
    for option in &mut design.options {
        for spec in &flags.add_rule_sets {
            ovcd_format::trace::add_rule_set(option, &library, spec)
                .map_err(|e| format!("Option '{}': {e}", option.name))?;
        }
        for id in &flags.remove_rule_sets {
            ovcd_format::trace::remove_rule_set(option, &library, id)
                .map_err(|e| format!("Option '{}': {e}", option.name))?;
        }
    }
    let text = design.to_text();
    match output {
        None => {
            print!("{text}");
            Ok(())
        }
        Some(path) => write_new(path, &text),
    }
}

/// Issue #451: `ovcd hull import <file.csv> --option <name> <in.vdesign> [<out.vdesign>]`.
fn hull_import(args: &[&str]) -> Result<(), String> {
    let (mut option, mut rest) = (None, Vec::new());
    let mut it = args.iter();
    while let Some(&a) = it.next() {
        match a {
            "--option" => option = Some(*it.next().ok_or("--option needs an Option name")?),
            _ => rest.push(a),
        }
    }
    let option = option.ok_or("give the Option with --option <name>")?;
    let flags = parse_flags(&rest, false, false, false)?;
    let (csv, input, output) = match flags.files.as_slice() {
        [csv, input] => (*csv, *input, None),
        [csv, input, output] => (*csv, *input, Some(*output)),
        _ => {
            return Err(
                "give the offsets CSV and one .vdesign file, and optionally an output file".into(),
            );
        }
    };
    let profile = load_profile(flags.profile)?;
    let csv_text = std::fs::read_to_string(csv).map_err(|e| format!("cannot read {csv}: {e}"))?;
    let text = std::fs::read_to_string(input).map_err(|e| format!("cannot read {input}: {e}"))?;
    let (mut session, _) = ovcd_format::DesignSession::open(&text, profile)?;
    for w in session.import_offsets(option, &csv_text)? {
        eprintln!("warning: {w}");
    }
    let text = session.save()?;
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
    let flags = parse_flags(args, true, false, false)?;
    let [path] = flags.files.as_slice() else {
        return Err("give exactly one .vdesign file".into());
    };
    let profile = load_profile(flags.profile)?;
    // Checked now; `validate` computes nothing, so nothing reads the sets.
    load_rule_sets(&flags.rule_sets)?;
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

/// Run a saved Design. Returns whether the file had no hard error and no Option failed. An Option
/// that is not yet supported (a convoy) is not a failure.
/// `ovcd export`: the Vessel Model of an Option as a file, or its hash (issue #460).
fn export(args: &[&str]) -> Result<(), String> {
    let (mut format, mut option, mut out, mut hash) = (None, None, None, false);
    let mut rest = Vec::new();
    let mut it = args.iter();
    while let Some(&a) = it.next() {
        match a {
            "--format" => format = Some(*it.next().ok_or("--format needs a name")?),
            "--option" => option = Some(*it.next().ok_or("--option needs a name")?),
            "-o" => out = Some(*it.next().ok_or("-o needs a file")?),
            "--hash" => hash = true,
            a => rest.push(a),
        }
    }
    let format = format.ok_or("give --format gltf")?;
    if format != "gltf" {
        return Err(format!("unknown format `{format}` (use gltf)"));
    }
    let flags = parse_flags(&rest, false, false, false)?;
    let [path] = flags.files.as_slice() else {
        return Err("give exactly one .vdesign file".into());
    };
    if out.is_none() && !hash {
        return Err("give -o <file>, or --hash".into());
    }
    let profile = load_profile(flags.profile)?;
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let checked = ovcd_format::check(&text, &profile);
    let design = checked
        .design
        .filter(|_| checked.report.is_ok())
        .ok_or_else(|| format!("{path} has errors; run `ovcd validate`"))?;
    let chosen: Vec<_> = design
        .options
        .iter()
        .filter(|o| option.is_none_or(|n| o.name == n))
        .collect();
    if chosen.is_empty() {
        return Err(format!("no Option named `{}`", option.unwrap_or("")));
    }
    if hash {
        for o in chosen {
            match ovcd_format::export::option_glb(o) {
                Ok(b) => println!("{}	{}", o.name, ovcd_format::export::bytes_hash(&b)),
                Err(e) => println!("{}	-	{e}", o.name),
            }
        }
        return Ok(());
    }
    let [one] = chosen.as_slice() else {
        return Err("the file has more than one Option: give --option <name>".into());
    };
    let bytes = ovcd_format::export::option_glb(one)?;
    let file = out.ok_or("give -o <file>")?;
    std::fs::write(file, &bytes).map_err(|e| format!("cannot write {file}: {e}"))?;
    println!("{file}: {} bytes", bytes.len());
    Ok(())
}

/// `ovcd mesh --hash`: one line per Option, its name and its mesh hash.
fn mesh(args: &[&str]) -> Result<(), String> {
    let mut hash = false;
    let mut rest = Vec::new();
    for &a in args {
        if a == "--hash" {
            hash = true;
        } else {
            rest.push(a);
        }
    }
    if !hash {
        return Err("give --hash: the mesh hash is the only mesh output so far".into());
    }
    let flags = parse_flags(&rest, false, false, false)?;
    let [path] = flags.files.as_slice() else {
        return Err("give exactly one .vdesign file".into());
    };
    let profile = load_profile(flags.profile)?;
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let checked = ovcd_format::check(&text, &profile);
    let design = checked
        .design
        .filter(|_| checked.report.is_ok())
        .ok_or_else(|| format!("{path} has errors; run `ovcd validate`"))?;
    for option in &design.options {
        match ovcd_format::option_meshes(option) {
            Ok(m) => println!("{}\t{}", option.name, ovcd_format::mesh_hash(&m)),
            Err(e) => println!("{}\t-\t{e}", option.name),
        }
    }
    Ok(())
}

fn run(args: &[&str]) -> Result<bool, String> {
    use ovcd_format::{HashStatus, RunOutcome};
    let flags = parse_flags(args, false, true, false)?;
    let [path] = flags.files.as_slice() else {
        return Err("give exactly one .vdesign file".into());
    };
    let profile = load_profile(flags.profile)?;
    let (rules, extra) = load_rule_sets(&flags.rule_sets)?;
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
    let runs = ovcd_format::run_design_with(design, &rules, &extra);
    for r in &runs {
        if let HashStatus::Differs { stored, computed } = &r.hash_status {
            eprintln!(
                "warning: {}",
                ovcd_format::hash_warning(&r.name, &design.engine_version, stored, computed)
            );
        }
    }
    // A convoy is valid but not computable yet: a warning, not a failure.
    for r in &runs {
        if let RunOutcome::NotAvailable(m) = &r.outcome {
            eprintln!("warning: Option '{}': {m}", r.name);
            if r.hash_status == HashStatus::NotChecked {
                eprintln!(
                    "warning: Option '{}': the stored results hash is not checked ({m})",
                    r.name
                );
            }
        }
    }
    let no_failure = !runs.iter().any(|r| r.outcome.is_failure());
    if flags.json {
        let options: Vec<serde_json::Value> = runs
            .iter()
            .map(|r| match &r.outcome {
                RunOutcome::Results { results, hash } => serde_json::json!({
                    "name": r.name,
                    "status": r.outcome.status(),
                    "results_hash": hash,
                    "hash_status": r.hash_status.as_str(),
                    "results": results,
                }),
                RunOutcome::NotAvailable(m) | RunOutcome::Failed(m) => serde_json::json!({
                    "name": r.name,
                    "status": r.outcome.status(),
                    "hash_status": r.hash_status.as_str(),
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
                        HashStatus::NotStored | HashStatus::NotChecked => "none saved in the file",
                    };
                    println!("  Results hash {} ({note})", &hash[..12]);
                    // Issue #274: a stored Rule Set whose rules differ from the file, or that has
                    // a newer built-in version.
                    for w in results["rule_trace"]["warnings"]
                        .as_array()
                        .into_iter()
                        .flatten()
                    {
                        if let Some(w) = w.as_str() {
                            println!("  Rule Sets: {w}");
                        }
                    }
                }
                RunOutcome::NotAvailable(m) => println!("Option '{}': {m}", r.name),
                RunOutcome::Failed(m) => println!("Option '{}': cannot be computed: {m}", r.name),
            }
        }
    }
    Ok(no_failure)
}
