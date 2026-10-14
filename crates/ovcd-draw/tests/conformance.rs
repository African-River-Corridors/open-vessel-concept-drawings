//! The conformance suite of the Drawing Format Standard (clause 17), run through this crate.
//!
//! Every case of `standard/conformance/suite.json` runs: each document through the runner of clause
//! 17.3, each unit file through the unit functions of clause 17.5. A case that this crate cannot pass
//! yet is named in `EXPECTED_TO_FAIL`, in this one place. The test fails when a case that is not on
//! the list fails, and also when a case on the list passes: the list can only shrink, and it is never
//! a way to skip a case without saying so.

use ovcd_draw::conformance::{compare, run_unit, same};
use ovcd_draw::{Files, run_text};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Cases not yet passing. Ticket #97 empties this list.
const EXPECTED_TO_FAIL: &[&str] = &[
    "ent-hatch",
    "hatch-lines-0",
    "hatch-lines-45",
    "hatch-soil",
    "hatch-water",
    "hatch-concrete",
    "hatch-rockfill",
    "hatch-concrete-30",
    "hatch-lines-rotated-viewport",
    "hatch-solid",
    "hatch-a0-unit",
    "ent-rotated-90",
    "ent-rotated-40-25",
    "dim-linear-horizontal",
    "dim-linear-vertical",
    "dim-linear-mm",
    "dim-linear-decimals-3",
    "dim-linear-points",
    "dim-aligned",
    "dim-radius",
    "dim-diameter-90",
    "dim-radius-arc",
    "dim-angle",
    "dim-angle-refs",
    "dim-leader",
    "dim-leader-left",
    "dim-rotated-viewport",
    "dim-chain-ok",
    "err-dim-chain-sum",
    "err-dim-chain-no-overall",
    "err-dim-ref-unknown",
    "err-dim-ref-place",
    "err-dim-radius-of-line",
    "err-dim-zero-value",
    "err-dim-too-short",
    "err-dim-extension-gap",
    "err-dim-outside-frame",
    "err-dim-arc-direction",
    "err-dim-radius-small",
    "err-dim-angle-zero-ray",
    "err-leader-same-point",
    "sym-paper-valve",
    "sym-paper-scale-200",
    "sym-paper-twice",
    "sym-model-tank",
    "sym-model-tank-200",
    "sym-rotate-90",
    "sym-rotate-30",
    "sym-rotated-viewport",
    "sym-line-types-and-roles",
    "sym-file-library",
    "sym-ports",
    "err-sym-unknown-symbol",
    "err-sym-unknown-library",
    "err-sym-lib-missing-file",
    "err-sym-lib-invalid-file",
    "err-sym-lib-bad-geometry",
    "err-sym-lib-duplicate-id",
    "err-sym-label-no-slot",
    "err-sym-outside-frame",
    "err-sym-port-unknown",
    "tb-minimal",
    "tb-fields-all",
    "tb-register-two",
    "tb-register-full",
    "tb-register-overflow",
    "tb-register-overflow-9",
    "tb-register-numeric",
    "tb-register-unsealed",
    "tb-register-alpha-aa",
    "tb-status-draft",
    "tb-status-concept",
    "tb-status-issued",
    "tb-scale-as-shown",
    "tb-a4-portrait",
    "tb-a0",
    "tb-private-template",
    "strips-notes-references",
    "strips-no-title-block",
    "err-tb-unknown-template",
    "err-tb-template-missing-file",
    "err-tb-template-invalid-file",
    "err-tb-template-geometry",
    "err-tb-missing-required",
    "err-tb-field-no-cell",
    "err-tb-text-too-long",
    "err-reg-text-too-long",
    "err-reg-token-duplicate",
    "err-reg-token-order",
    "err-reg-date-order",
    "err-reg-date-invalid",
    "err-reg-seal-edited",
    "err-reg-seal-inserted",
    "err-status-issued-no-approver",
    "err-status-issued-no-seal",
    "err-status-issued-no-register",
    "err-status-issued-no-title-block",
    "err-strip-no-room",
    "err-strip-duplicate-content",
    "err-notes-without-strip",
    "err-note-too-long",
    "err-too-many-notes",
    "err-region-outside",
    "err-viewport-over-title-block",
    "err-hatch-zero-area",
    "unit-symbol_point",
    "unit-revision_seals",
    "unit-register_layout",
    "unit-dimension_text",
    "unit-reading_rotation",
    "unit-chain_ok",
];

struct DirFiles(PathBuf);

impl Files for DirFiles {
    fn read(&self, relative_path: &str) -> Option<String> {
        fs::read_to_string(self.0.join(relative_path)).ok()
    }
}

fn suite_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("standard")
        .join("conformance")
}

fn read_json(path: &Path) -> Value {
    ovcd_draw::json::parse(&fs::read_to_string(path).expect("read")).expect("valid JSON")
}

fn run_case(suite: &Path, case: &Value) -> Vec<String> {
    match case["kind"].as_str() {
        Some("document") => {
            let dir = suite.join(case["dir"].as_str().expect("dir"));
            let text = fs::read_to_string(dir.join("input.json")).expect("input");
            let expected = read_json(&dir.join("expected.json"));
            let result = run_text(&text, &DirFiles(dir)).to_value();
            compare(&result, &expected)
        }
        Some("unit") => {
            let unit = read_json(&suite.join(case["file"].as_str().expect("file")));
            let function = unit["function"].as_str().expect("function");
            let mut problems = Vec::new();
            for check in unit["checks"].as_array().expect("checks") {
                match run_unit(function, &check["input"]) {
                    Some(got) if same(&got, &check["expected"]) => {}
                    Some(got) => problems.push(format!(
                        "{}: got {got}, expected {}",
                        check["id"], check["expected"]
                    )),
                    None => problems.push(format!("{function}: not implemented")),
                }
            }
            problems
        }
        other => vec![format!("unknown kind {other:?}")],
    }
}

#[test]
fn whole_suite() {
    let suite = suite_dir();
    let list = read_json(&suite.join("suite.json"));
    let cases = list["cases"].as_array().expect("cases");
    assert_eq!(cases.len(), 194, "the pinned suite has 194 cases");
    let expected: BTreeSet<&str> = EXPECTED_TO_FAIL.iter().copied().collect();
    let mut failing = BTreeSet::new();
    let mut report = Vec::new();
    for case in cases {
        let id = case["id"].as_str().expect("id");
        let problems = run_case(&suite, case);
        if !problems.is_empty() {
            failing.insert(id.to_string());
            if !expected.contains(id) {
                report.push(format!("FAIL {id}: {}", problems[0]));
            }
        }
    }
    let mut passing_but_listed: Vec<&str> = expected
        .iter()
        .copied()
        .filter(|id| !failing.contains(*id))
        .collect();
    passing_but_listed.sort_unstable();
    assert!(
        report.is_empty(),
        "cases failed that are not on the expected-to-fail list:\n{}",
        report.join("\n")
    );
    assert!(
        passing_but_listed.is_empty(),
        "cases on the expected-to-fail list now pass; remove them: {passing_but_listed:?}"
    );
    let ids: BTreeSet<&str> = cases.iter().filter_map(|c| c["id"].as_str()).collect();
    for id in &expected {
        assert!(
            ids.contains(id),
            "{id} is on the expected-to-fail list but is not in the suite"
        );
    }
}
