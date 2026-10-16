//! The leaders of the GA views (#213): every label text stays inside the frame, clear of the centring
//! marks; no two leaders of a view cross; no two label boxes overlap; and every leader line keeps clear
//! of the letters of every OTHER label (a leader's own text stands the standard's `u / 2` from its
//! line, clause 12.6). Each view is checked on the paper, in the normalised content of the standard
//! itself, with the text box of clause 11.7.
//!
//! Label: property - no number is fixed; each check is a geometric property of the drawing.

use ovcd_draw::rules::text_width_exact;
use ovcd_draw::{NoFiles, RunResult, run_value};
use ovcd_format::{Profile, RunOutcome, check, ga_views_of, run_design};
use serde_json::Value;
use std::path::Path;

/// The gap a leader line keeps from the letters of another label, in millimetres.
const LETTER_GAP_MM: f64 = 1.5;
/// The room a label text keeps from the frame, in millimetres: the 5 mm that a centring mark reaches
/// into the drawing space (clause 6.4) and 1 mm more, so a label is clear of a mark wherever it
/// stands along the side.
const FRAME_GAP_MM: f64 = 6.0;
/// The room a label text keeps from a centring mark itself, in millimetres.
const MARK_GAP_MM: f64 = 1.0;
/// The least clear space between two label boxes, in millimetres.
const BOX_GAP_MM: f64 = 1.0;

/// A fixture Design with some inputs changed; `formation` is `pushed_barge` or `self_propelled`.
fn design(
    fixture: &str,
    formation: &str,
    dims: Option<(f64, f64, f64)>,
    cargo: Option<&str>,
) -> String {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(fixture),
    )
    .expect("fixture");
    let text = if formation == "self_propelled" {
        text.replace("pushed_barge", "self_propelled")
    } else {
        text.replace("self_propelled", "pushed_barge")
    };
    let mut d: Value = serde_json::from_str(&text).expect("JSON");
    for option in d["options"].as_array_mut().expect("options") {
        let inputs = &mut option["inputs"];
        if let Some((l, b, t)) = dims {
            inputs["length_overall_m"]["value"] = l.into();
            inputs["beam_m"]["value"] = b.into();
            inputs["moulded_depth_m"]["value"] = t.into();
        }
        if let Some(c) = cargo {
            inputs["cargo"]["value"] = c.into();
        }
    }
    d.to_string()
}

/// The Designs checked: every fixture in both formations, the two cases of the issue (LNG 90 x 18 x 7 m
/// pushed, LNG 76.5 x 11.4 x 3.5 m in both formations) and bauxite in a box hold in both formations.
fn designs() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for fixture in [
        "dry_bulk_box_hold.vdesign",
        "dry_bulk_sloped_hopper.vdesign",
        "gas_igc_2g_self_propelled.vdesign",
        "gas_igc_2g_exact.vdesign",
        "two_options.vdesign",
    ] {
        for formation in ["pushed_barge", "self_propelled"] {
            out.push((
                format!("{fixture} {formation}"),
                design(fixture, formation, None, None),
            ));
        }
    }
    out.push((
        "LNG 90 x 18 x 7 pushed_barge".to_string(),
        design(
            "gas_igc_2g_self_propelled.vdesign",
            "pushed_barge",
            Some((90.0, 18.0, 7.0)),
            Some("LNG"),
        ),
    ));
    let crowded = Some((76.5, 11.4, 3.5));
    for formation in ["pushed_barge", "self_propelled"] {
        out.push((
            format!("LNG 76.5 x 11.4 x 3.5 {formation}"),
            design(
                "gas_igc_2g_self_propelled.vdesign",
                formation,
                crowded,
                Some("LNG"),
            ),
        ));
        out.push((
            format!("dry bulk 76.5 x 11.4 x 3.5 {formation}"),
            design("dry_bulk_box_hold.vdesign", formation, crowded, None),
        ));
    }
    out
}

/// One label on the paper: its line from the label point to the target, its text and the box of
/// its letters (the text less the spaces at its ends), in paper millimetres.
struct Label {
    text: String,
    line: ([f64; 2], [f64; 2]),
    letters: [f64; 4],
    text_box: [f64; 4],
}

/// The normalised content of every view of every computed Option of every Design checked.
fn views() -> Vec<(String, Value)> {
    let mut out = Vec::new();
    for (name, text) in designs() {
        let design = check(&text, &Profile::built_in())
            .design
            .unwrap_or_else(|| panic!("{name}: not a valid Design"));
        for run in run_design(&design) {
            if !matches!(run.outcome, RunOutcome::Results { .. }) {
                continue;
            }
            let v = ga_views_of(&run.outcome).expect("views");
            for (view, document) in v.all() {
                let norm = match run_value(document, &NoFiles) {
                    RunResult::Ok(n) => n.to_value(),
                    RunResult::Errors(e) => panic!("{name} {view}: refused (clause {e:?})"),
                };
                out.push((format!("{name} / {} / {view}", run.name), norm));
            }
        }
    }
    assert!(out.len() >= 3 * 16, "{} views", out.len());
    out
}

fn xy(v: &Value) -> [f64; 2] {
    [v[0].as_f64().unwrap(), v[1].as_f64().unwrap()]
}

/// The labels of a view. A leader's line runs from its label point to its target, and its text
/// stands `u / 2` from the label point (clause 12.6), level with it: that pairs a text with its line.
fn labels(norm: &Value) -> Vec<Label> {
    let parts: Vec<&Value> = norm["entities"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["origin"].as_str().unwrap().ends_with(".leader"))
        .collect();
    let lines: Vec<([f64; 2], [f64; 2])> = parts
        .iter()
        .filter(|e| e["type"] == "line")
        .map(|e| (xy(&e["from_mm"]), xy(&e["to_mm"])))
        .collect();
    let mut out = Vec::new();
    for e in parts.iter().filter(|e| e["type"] == "text") {
        let text = e["text"].as_str().unwrap();
        let h = e["height_mm"].as_f64().unwrap();
        let a = xy(&e["anchor_mm"]);
        let right = e["align_h"] == "right";
        let line = *lines
            .iter()
            .find(|(l, _)| {
                (l[1] - a[1]).abs() < 2e-3
                    && (a[0] - l[0] - if right { -0.9 } else { 0.9 }).abs() < 2e-3
            })
            .unwrap_or_else(|| panic!("no leader line for `{text}`"));
        let width = |t: &str| text_width_exact(t, h).expect("a known character");
        let full = width(text);
        let lead = width(&" ".repeat(text.len() - text.trim_start().len()));
        let trail = width(&" ".repeat(text.len() - text.trim_end().len()));
        let x0 = if right { a[0] - full } else { a[0] };
        assert_eq!(e["align_v"], "middle");
        out.push(Label {
            text: text.trim().to_string(),
            line,
            letters: [x0 + lead, a[1] - h / 2.0, x0 + full - trail, a[1] + h / 2.0],
            text_box: [x0, a[1] - h / 2.0, x0 + full, a[1] + h / 2.0],
        });
    }
    assert_eq!(out.len(), lines.len(), "one text for each leader line");
    out
}

fn segments_cross(a: ([f64; 2], [f64; 2]), b: ([f64; 2], [f64; 2])) -> bool {
    let side = |p: [f64; 2], q: [f64; 2], r: [f64; 2]| {
        (q[0] - p[0]) * (r[1] - p[1]) - (q[1] - p[1]) * (r[0] - p[0])
    };
    let (d1, d2) = (side(b.0, b.1, a.0), side(b.0, b.1, a.1));
    let (d3, d4) = (side(a.0, a.1, b.0), side(a.0, a.1, b.1));
    d1 * d2 < 0.0 && d3 * d4 < 0.0
}

fn point_to_segment(p: [f64; 2], (a, b): ([f64; 2], [f64; 2])) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len2 = dx * dx + dy * dy;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2).clamp(0.0, 1.0)
    };
    let (ex, ey) = (p[0] - a[0] - t * dx, p[1] - a[1] - t * dy);
    (ex * ex + ey * ey).sqrt()
}

fn point_to_box(p: [f64; 2], r: [f64; 4]) -> f64 {
    let dx = (r[0] - p[0]).max(p[0] - r[2]).max(0.0);
    let dy = (r[1] - p[1]).max(p[1] - r[3]).max(0.0);
    (dx * dx + dy * dy).sqrt()
}

/// The shortest distance from a segment to a box (0 if they meet).
fn segment_to_box(s: ([f64; 2], [f64; 2]), r: [f64; 4]) -> f64 {
    let corners = [[r[0], r[1]], [r[2], r[1]], [r[2], r[3]], [r[0], r[3]]];
    let inside = |p: [f64; 2]| p[0] >= r[0] && p[0] <= r[2] && p[1] >= r[1] && p[1] <= r[3];
    if inside(s.0) || inside(s.1) {
        return 0.0;
    }
    let edges_cross = (0..4).any(|i| segments_cross(s, (corners[i], corners[(i + 1) % 4])));
    if edges_cross {
        return 0.0;
    }
    corners
        .iter()
        .map(|c| point_to_segment(*c, s))
        .chain([point_to_box(s.0, r), point_to_box(s.1, r)])
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn every_label_text_lies_inside_the_frame_clear_of_the_centring_marks() {
    for (name, norm) in views() {
        let f = &norm["viewports"][0]["frame"];
        let g = |k: &str| f[k].as_f64().unwrap();
        let frame = [
            g("x_mm"),
            g("y_mm"),
            g("x_mm") + g("width_mm"),
            g("y_mm") + g("height_mm"),
        ];
        let marks: Vec<([f64; 2], [f64; 2])> = norm["entities"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["origin"] == "sheet.centring_mark")
            .map(|e| (xy(&e["from_mm"]), xy(&e["to_mm"])))
            .collect();
        assert_eq!(marks.len(), 4, "{name}: four centring marks");
        for l in labels(&norm) {
            let b = l.text_box;
            assert!(
                b[0] >= frame[0] + FRAME_GAP_MM
                    && b[1] >= frame[1] + FRAME_GAP_MM
                    && b[2] <= frame[2] - FRAME_GAP_MM
                    && b[3] <= frame[3] - FRAME_GAP_MM,
                "{name}: `{}` at {b:?} is not inside the frame {frame:?}",
                l.text
            );
            for m in &marks {
                let d = segment_to_box(*m, b);
                assert!(
                    d >= MARK_GAP_MM,
                    "{name}: `{}` is {d:.2} mm from a centring mark",
                    l.text
                );
            }
        }
    }
}

#[test]
fn no_two_leaders_of_a_view_cross() {
    for (name, norm) in views() {
        let ls = labels(&norm);
        for (i, a) in ls.iter().enumerate() {
            for b in &ls[i + 1..] {
                // two leaders may share a target point
                let shared = (a.line.1[0] - b.line.1[0]).abs() < 1e-6
                    && (a.line.1[1] - b.line.1[1]).abs() < 1e-6;
                assert!(
                    shared || !segments_cross(a.line, b.line),
                    "{name}: the leaders of `{}` and `{}` cross",
                    a.text,
                    b.text
                );
            }
        }
    }
}

#[test]
fn every_leader_line_keeps_clear_of_the_letters_of_every_other_label() {
    for (name, norm) in views() {
        let ls = labels(&norm);
        for (i, line) in ls.iter().enumerate() {
            for (j, text) in ls.iter().enumerate() {
                if i == j {
                    continue;
                }
                let d = segment_to_box(line.line, text.letters);
                assert!(
                    d >= LETTER_GAP_MM,
                    "{name}: the leader of `{}` is {d:.2} mm from the letters of `{}`",
                    line.text,
                    text.text
                );
            }
        }
    }
}

#[test]
fn no_two_label_boxes_overlap() {
    for (name, norm) in views() {
        let ls = labels(&norm);
        for (i, a) in ls.iter().enumerate() {
            for b in &ls[i + 1..] {
                let (s, t, g) = (a.text_box, b.text_box, BOX_GAP_MM);
                assert!(
                    !(s[0] < t[2] + g && t[0] < s[2] + g && s[1] < t[3] + g && t[1] < s[3] + g),
                    "{name}: the labels `{}` and `{}` overlap",
                    a.text,
                    b.text
                );
            }
        }
    }
}
