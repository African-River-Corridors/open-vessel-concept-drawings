//! Seam: the tank, dome and weather shield meshes of every gas design, and the source of every mesh
//! group (spec 0009, "no geometry without a source", issue #686).
//!
//! Label: property. Each part's bounding box is rebuilt from the results JSON (`drawing.tank`,
//! `geometry`, `arrangement`, `drawing.aft_arrangement`) and compared with its mesh's box to 1 mm;
//! the tank count is `n_tanks_total`; each tank mesh holds the closed-form volume at the outer
//! diameter to 0.5 %; every new mesh is closed; and no tank mesh crosses another tank, a ballast
//! Space (double bottom, wing, peak) or the hull, except through the deck surface, at the deck
//! height and inside the tank's own footprint (its deck opening): the tanks of two fixtures stand up
//! through the deck. That reading of #686 was agreed by the coordinator overnight on 2026-10-30,
//! pending the maintainer's review. Each condition has a test that moves a tank to break it alone.

use core::f64::consts::PI;
use std::collections::HashMap;
use std::path::Path;

use ovd_core::mesh::{Mesh, PartSource};
use ovd_core::vessel_model::WheelhousePosition;
use ovd_format::{Profile, RunOutcome, check, option_vessel_run, run_option};
use serde_json::Value;

struct Case {
    at: String,
    results: Value,
    mesh: Mesh,
}

/// Every computed Option of every fixture and website example.
fn cases() -> Vec<Case> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    for dir in [
        root.join("tests/fixtures"),
        root.join("../../website/assets/examples"),
    ] {
        for e in std::fs::read_dir(&dir).unwrap() {
            let p = e.unwrap().path();
            if p.extension().is_some_and(|x| x == "vdesign") {
                files.push(p);
            }
        }
    }
    files.sort();
    let mut out = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let design = check(&text, &Profile::built_in())
            .design
            .expect("a valid design");
        for option in &design.options {
            let RunOutcome::Results { results, .. } = run_option(option) else {
                continue;
            };
            let Ok(run) = option_vessel_run(option) else {
                continue;
            };
            out.push(Case {
                at: format!(
                    "{} / {}",
                    path.file_name().unwrap().to_string_lossy(),
                    option.name
                ),
                results,
                mesh: run.meshes(),
            });
        }
    }
    assert!(out.len() >= 12, "{} designs", out.len());
    out
}

fn group_mesh(m: &Mesh, start: u32, count: u32) -> Mesh {
    Mesh {
        positions: m.positions.clone(),
        indices: m.indices[start as usize..(start + count) as usize].to_vec(),
        groups: Vec::new(),
    }
}

fn part(m: &Mesh, source: &PartSource) -> Mesh {
    let g = m
        .groups
        .iter()
        .find(|g| &g.source == source)
        .unwrap_or_else(|| panic!("no group for {source:?}"));
    group_mesh(m, g.start, g.count)
}

fn bbox(m: &Mesh) -> [f64; 6] {
    let mut b = [
        f64::INFINITY,
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for &i in &m.indices {
        let v = m.vertex(i);
        for k in 0..3 {
            b[k] = b[k].min(v[k]);
            b[k + 3] = b[k + 3].max(v[k]);
        }
    }
    b
}

fn closed(m: &Mesh) -> Result<(), String> {
    let mut count: HashMap<(u32, u32), i32> = HashMap::new();
    for t in m.indices.as_chunks::<3>().0 {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            *count.entry((a, b)).or_default() += 1;
        }
    }
    for (&(a, b), &n) in &count {
        if n != 1 || count.get(&(b, a)) != Some(&1) {
            return Err(format!("edge {:?} {:?}", m.vertex(a), m.vertex(b)));
        }
    }
    Ok(())
}

fn num(v: &Value, path: &[&str]) -> f64 {
    let mut cur = v;
    for k in path {
        cur = &cur[*k];
    }
    cur.as_f64()
        .unwrap_or_else(|| panic!("no number at {}", path.join(".")))
}

/// The box of each part that has one in the results, by its source: the tanks and domes of a gas
/// design, its weather shield, and the Deck Items of a stern block.
fn expected_boxes(results: &Value) -> Vec<(PartSource, [f64; 6])> {
    let mut out = Vec::new();
    if results["phase"] == "liquid_gas" {
        let t = &results["drawing"]["tank"];
        let od = num(t, &["tank_outer_diameter_m"]);
        let r = od / 2.0;
        let length = num(t, &["overall_tank_length_m"]);
        let (gap_long, gap_across) = (
            num(t, &["tank_gap_longitudinal_m"]),
            num(t, &["tank_gap_transverse_m"]),
        );
        let stern = num(results, &["geometry", "stern_void_length_m"]);
        let side = num(results, &["geometry", "side_clearance_m"]);
        let beam = num(results, &["dims", "beam_m"]);
        let depth = num(results, &["dims", "depth_m"]);
        let n_long = num(results, &["arrangement", "n_long"]) as usize;
        let n_wide = num(results, &["arrangement", "n_wide"]) as usize;
        let bottom = num(results, &["drawing", "double_bottom_m"])
            + num(results, &["drawing", "tank_support_height_m"]);
        let z = bottom + r;
        let dome = (num(t, &["dome_diameter_m"]), num(t, &["dome_height_m"]));
        let mut index = 0;
        for i in 0..n_long {
            let x0 = stern + i as f64 * (length + gap_long);
            for j in 0..n_wide {
                let y = -(beam - 2.0 * side) / 2.0 + r + j as f64 * (od + gap_across);
                let name = match (n_wide, j) {
                    (1, _) => format!("CT{}C", i + 1),
                    (2, 0) => format!("CT{}P", i + 1),
                    (2, _) => format!("CT{}S", i + 1),
                    _ => format!("CT{}.{}", i + 1, j + 1),
                };
                out.push((
                    PartSource::Tank {
                        index,
                        name: name.clone(),
                    },
                    [x0, y - r, z - r, x0 + length, y + r, z + r],
                ));
                if dome.0 > 0.0 && dome.1 > 0.0 {
                    let rd = dome.0 / 2.0;
                    let xm = x0 + length / 2.0;
                    out.push((
                        PartSource::Dome { index, name },
                        [
                            xm - rd,
                            y - rd,
                            z + (r * r - rd * rd).sqrt(),
                            xm + rd,
                            y + rd,
                            bottom + od + dome.1,
                        ],
                    ));
                }
                index += 1;
            }
        }
        let d = &results["drawing"];
        let thickness = num(d, &["weather_shield_thickness_mm"]) / 1000.0;
        let ratio = num(d, &["wing_tank_height_ratio"]);
        let top = bottom + od + num(d, &["weather_shield_clearance_m"]);
        if thickness > 0.0 && num(d, &["wing_tank_width_m"]) > 0.0 && ratio < 1.0 && top > depth {
            let bow = num(results, &["geometry", "bow_void_length_m"]);
            let loa = num(results, &["dims", "loa_m"]);
            out.push((
                PartSource::WeatherShield,
                [
                    stern,
                    -beam / 2.0,
                    (bottom + ratio * od).max(depth),
                    loa - bow,
                    beam / 2.0,
                    top,
                ],
            ));
        }
    }
    if let Some(a) = results["drawing"]["aft_arrangement"].as_object() {
        let n = |k: &str| a[k].as_f64().unwrap();
        let half = n("container_unit_width_m") / 2.0;
        for (i, u) in a["container_units"].as_array().unwrap().iter().enumerate() {
            let y = -u[3].as_f64().unwrap();
            // The base is the deck under the unit: the deck items test checks it against the hull.
            out.push((
                PartSource::Unit(i),
                [
                    u[1].as_f64().unwrap(),
                    y - half,
                    f64::NAN,
                    u[2].as_f64().unwrap(),
                    y + half,
                    n("container_top_above_keel_m"),
                ],
            ));
        }
        let hw = n("wheelhouse_width_m") / 2.0;
        for (p, lo, hi) in [
            (
                WheelhousePosition::Lowered,
                "wheelhouse_floor_lowered_m",
                "wheelhouse_top_lowered_m",
            ),
            (
                WheelhousePosition::Raised,
                "wheelhouse_floor_raised_m",
                "wheelhouse_top_raised_m",
            ),
        ] {
            out.push((
                PartSource::Wheelhouse(p),
                [
                    n("wheelhouse_x0_m"),
                    -hw,
                    n(lo),
                    n("wheelhouse_x1_m"),
                    hw,
                    n(hi),
                ],
            ));
        }
    }
    out
}

#[test]
fn every_part_with_a_source_in_the_results_has_its_box() {
    let mut tanks = 0;
    let mut domes = 0;
    let mut shields = 0;
    for c in cases() {
        let expected = expected_boxes(&c.results);
        for (source, want) in &expected {
            let got = bbox(&part(&c.mesh, source));
            for k in 0..6 {
                if want[k].is_nan() {
                    continue;
                }
                assert!(
                    (got[k] - want[k]).abs() <= 1e-3,
                    "{}: {source:?} [{k}] mesh {} vs results {}",
                    c.at,
                    got[k],
                    want[k]
                );
            }
            match source {
                PartSource::Tank { .. } => tanks += 1,
                PartSource::Dome { .. } => domes += 1,
                PartSource::WeatherShield => shields += 1,
                _ => {}
            }
        }
        // Every group of a new part is one the results account for, and every group has a source
        // whose layer and id it carries. A lift column is checked by the deck items test.
        for g in &c.mesh.groups {
            assert_eq!(g.layer, g.source.layer(), "{}", c.at);
            assert_eq!(g.id, g.source.id(), "{}", c.at);
            let new = matches!(
                g.source,
                PartSource::Tank { .. }
                    | PartSource::Dome { .. }
                    | PartSource::WeatherShield
                    | PartSource::Unit(_)
                    | PartSource::Wheelhouse(_)
            );
            if new {
                assert!(
                    expected.iter().any(|(s, _)| *s == g.source),
                    "{}: {} has no source in the results",
                    c.at,
                    g.id
                );
            }
        }
        // As many tank meshes as the results count tanks.
        let n = c.results["geometry"]["n_tanks_total"].as_u64().unwrap_or(0);
        let count = c
            .mesh
            .groups
            .iter()
            .filter(|g| matches!(g.source, PartSource::Tank { .. }))
            .count() as u64;
        assert_eq!(count, n, "{}", c.at);
    }
    assert!(
        tanks >= 12 && domes >= 12 && shields >= 2,
        "{tanks} {domes} {shields}"
    );
}

#[test]
fn each_tank_mesh_is_closed_and_holds_the_closed_form_volume_at_the_outer_diameter() {
    let mut checked = 0;
    for c in cases() {
        let t = &c.results["drawing"]["tank"];
        for g in &c.mesh.groups {
            let PartSource::Tank { .. } = g.source else {
                if matches!(
                    g.source,
                    PartSource::Dome { .. } | PartSource::WeatherShield
                ) {
                    closed(&group_mesh(&c.mesh, g.start, g.count))
                        .unwrap_or_else(|e| panic!("{} {}: {e}", c.at, g.id));
                }
                continue;
            };
            let m = group_mesh(&c.mesh, g.start, g.count);
            closed(&m).unwrap_or_else(|e| panic!("{} {}: {e}", c.at, g.id));
            let r = num(t, &["tank_outer_diameter_m"]) / 2.0;
            let length = num(t, &["overall_tank_length_m"]);
            // The head is half an ellipsoid: a hemisphere (r long) or a 2:1 head.
            let h = match t["head_type"].as_str() {
                Some("hemisphere") => r,
                _ => {
                    let ins = num(&c.results["drawing"]["tank"], &["insulation_thickness_mm"]);
                    (r - ins / 1000.0) / 2.0 + ins / 1000.0
                }
            };
            let want = PI * r * r * (length - 2.0 * h) + 2.0 * (2.0 / 3.0) * PI * r * r * h;
            assert!(
                (m.volume() - want).abs() <= 0.005 * want,
                "{} {}: {} vs {want}",
                c.at,
                g.id,
                m.volume()
            );
            checked += 1;
        }
    }
    assert!(checked >= 12, "{checked}");
}

type Tri = [[f64; 3]; 3];

fn triangles(m: &Mesh) -> Vec<Tri> {
    m.indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|t| [m.vertex(t[0]), m.vertex(t[1]), m.vertex(t[2])])
        .collect()
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Where the segment `p q` passes through the inside of triangle `t` (Moller and Trumbore), not at
/// its ends or edges: a touch is not a crossing.
fn segment_crossing(p: [f64; 3], q: [f64; 3], t: &Tri) -> Option<[f64; 3]> {
    let eps = 1e-7;
    let d = sub(q, p);
    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
    let h = cross(d, e2);
    let a = dot(e1, h);
    if a.abs() < 1e-12 {
        return None;
    }
    let s = sub(p, t[0]);
    let u = dot(s, h) / a;
    let qv = cross(s, e1);
    let v = dot(d, qv) / a;
    let w = dot(e2, qv) / a;
    (u > eps && v > eps && u + v < 1.0 - eps && w > eps && w < 1.0 - eps)
        .then(|| [p[0] + w * d[0], p[1] + w * d[1], p[2] + w * d[2]])
}

fn bounds(t: &Tri) -> [f64; 6] {
    let mut b = [
        f64::INFINITY,
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for v in t {
        for k in 0..3 {
            b[k] = b[k].min(v[k]);
            b[k + 3] = b[k + 3].max(v[k]);
        }
    }
    b
}

fn overlap(a: &[f64; 6], b: &[f64; 6]) -> bool {
    (0..3).all(|k| a[k] <= b[k + 3] && b[k] <= a[k + 3])
}

/// The points where the surfaces of `a` and `b` cross: each edge of either through the other.
fn crossing_points(a: &[Tri], b: &[Tri]) -> Vec<[f64; 3]> {
    let ab: Vec<_> = a.iter().map(bounds).collect();
    let mut out = Vec::new();
    for tb in b {
        let bb = bounds(tb);
        for (ta, ba) in a.iter().zip(&ab) {
            if !overlap(ba, &bb) {
                continue;
            }
            for i in 0..3 {
                out.extend(segment_crossing(ta[i], ta[(i + 1) % 3], tb));
                out.extend(segment_crossing(tb[i], tb[(i + 1) % 3], ta));
            }
        }
    }
    out
}

/// A tank as the results place it: its axis from `x0` to `x1` at `y`, and its outer radius.
struct Footprint {
    x0: f64,
    x1: f64,
    y: f64,
    r: f64,
}

/// What a tank mesh `tank` must not do (issue #686, as the coordinator agreed it overnight, pending
/// the maintainer's review): cross another tank, a ballast Space (double bottom, wing or peak), or
/// the hull anywhere but the deck. The one hull crossing allowed is through the deck surface, at the
/// deck height and inside the tank's own footprint (its deck opening). Each fault, named.
fn tank_faults(
    tank: &[Tri],
    foot: &Footprint,
    depth: f64,
    hull: &[Tri],
    others: &[(String, Vec<Tri>)],
) -> Vec<String> {
    let mut faults = Vec::new();
    for (name, u) in others {
        if let Some(p) = crossing_points(tank, u).first() {
            faults.push(format!("crosses {name} at {p:?}"));
        }
    }
    for p in crossing_points(tank, hull) {
        let at_deck = (p[2] - depth).abs() < 1e-4;
        let inside = p[0] >= foot.x0 - 1e-6
            && p[0] <= foot.x1 + 1e-6
            && (p[1] - foot.y).abs() <= foot.r + 1e-6;
        if !(at_deck && inside) {
            faults.push(format!("crosses the hull at {p:?}"));
            break;
        }
    }
    faults
}

/// The tanks, the ballast Spaces and the hull of a design, with each tank's footprint from the
/// results.
struct Arrangement {
    tanks: Vec<(String, Vec<Tri>, Footprint)>,
    ballast: Vec<(String, Vec<Tri>)>,
    hull: Vec<Tri>,
    depth: f64,
}

fn arrangement(c: &Case) -> Option<Arrangement> {
    let boxes = expected_boxes(&c.results);
    let tanks: Vec<_> = c
        .mesh
        .groups
        .iter()
        .filter(|g| matches!(g.source, PartSource::Tank { .. }))
        .map(|g| {
            let b = boxes
                .iter()
                .find(|(s, _)| *s == g.source)
                .map(|(_, b)| *b)
                .expect("a tank box from the results");
            let foot = Footprint {
                x0: b[0],
                x1: b[3],
                y: (b[1] + b[4]) / 2.0,
                r: (b[4] - b[1]) / 2.0,
            };
            (
                g.id.clone(),
                triangles(&group_mesh(&c.mesh, g.start, g.count)),
                foot,
            )
        })
        .collect();
    if tanks.is_empty() {
        return None;
    }
    let ballast = c
        .mesh
        .groups
        .iter()
        .filter(|g| {
            g.layer == "space" && ["DB", "WB", "FP", "AP"].iter().any(|p| g.id.starts_with(p))
        })
        .map(|g| {
            (
                g.id.clone(),
                triangles(&group_mesh(&c.mesh, g.start, g.count)),
            )
        })
        .collect();
    Some(Arrangement {
        tanks,
        ballast,
        hull: triangles(&part(&c.mesh, &PartSource::Hull)),
        depth: num(&c.results, &["dims", "depth_m"]),
    })
}

/// The faults of tank `i` of an arrangement, its mesh moved `dx` forward, `dy` to starboard and `dz`
/// up (its footprint stays where the results put it).
fn faults_of(a: &Arrangement, i: usize, (dx, dy, dz): (f64, f64, f64)) -> Vec<String> {
    let (_, tank, foot) = &a.tanks[i];
    let moved: Vec<Tri> = tank
        .iter()
        .map(|t| t.map(|v| [v[0] + dx, v[1] + dy, v[2] + dz]))
        .collect();
    let mut others = a.ballast.clone();
    others.extend(
        a.tanks
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, (n, t, _))| (n.clone(), t.clone())),
    );
    tank_faults(&moved, foot, a.depth, &a.hull, &others)
}

#[test]
fn no_tank_crosses_another_tank_a_ballast_space_or_the_hull_but_its_deck_opening() {
    let (mut tanks, mut through_deck) = (0, 0);
    for c in cases() {
        let Some(a) = arrangement(&c) else { continue };
        for (i, (name, tank, foot)) in a.tanks.iter().enumerate() {
            let faults = faults_of(&a, i, (0.0, 0.0, 0.0));
            assert!(faults.is_empty(), "{}: {name} {faults:?}", c.at);
            tanks += 1;
            let hull = crossing_points(tank, &a.hull);
            through_deck += usize::from(!hull.is_empty());
            // Where a tank stands through the deck, its crossing is its deck opening.
            for p in hull {
                assert!((p[1] - foot.y).abs() <= foot.r + 1e-6, "{}: {name}", c.at);
            }
        }
        assert!(
            !a.ballast.is_empty(),
            "{}: ballast Spaces to check against",
            c.at
        );
    }
    assert!(tanks >= 12, "{tanks} tanks");
    assert!(through_deck > 0, "a fixture's tanks stand through the deck");
}

#[test]
fn a_tank_moved_outboard_into_the_side_shell_fails_the_check() {
    // The check can fail: each tank of each gas design, moved outboard until its side is 0.5 m into
    // the side shell, crosses the hull below the deck (issue #686, the coordinator's negative test).
    let mut moved = 0;
    for c in cases() {
        let Some(a) = arrangement(&c) else { continue };
        let half_b = num(&c.results, &["dims", "beam_m"]) / 2.0;
        for (i, (name, _, foot)) in a.tanks.iter().enumerate() {
            let side = if foot.y < 0.0 { -1.0 } else { 1.0 };
            let dy = side * (half_b - (foot.y.abs() + foot.r) + 0.5);
            let faults = faults_of(&a, i, (0.0, dy, 0.0));
            assert!(
                faults.iter().any(|f| f.starts_with("crosses the hull")),
                "{}: {name} 0.5 m into the side shell passed: {faults:?}",
                c.at
            );
            moved += 1;
        }
    }
    assert!(moved >= 12, "{moved}");
}

#[test]
fn a_tank_that_crosses_the_deck_outside_its_own_opening_fails_the_check() {
    // The footprint condition can fail on its own: a tank that stands through the deck, moved 0.5 m
    // forward (less than the gap to the next tank and the bow void), crosses the deck outside the
    // opening the results give it, and nothing below the deck.
    let mut moved = 0;
    for c in cases() {
        let Some(a) = arrangement(&c) else { continue };
        for (i, (name, tank, _)) in a.tanks.iter().enumerate() {
            if crossing_points(tank, &a.hull).is_empty() {
                continue;
            }
            let faults = faults_of(&a, i, (0.5, 0.0, 0.0));
            assert!(
                faults.iter().any(|f| f.starts_with("crosses the hull")),
                "{}: {name} moved 0.5 m forward passed: {faults:?}",
                c.at
            );
            moved += 1;
        }
    }
    assert!(moved >= 2, "{moved} tanks through the deck");
}

#[test]
fn a_tank_moved_into_the_bottom_shell_fails_the_check() {
    // The deck-height condition can fail on its own: moved down until it is 0.5 m into the bottom
    // shell, a tank crosses the hull inside its own footprint, but not at the deck.
    let mut moved = 0;
    for c in cases() {
        let Some(a) = arrangement(&c) else { continue };
        let bottom = num(&c.results, &["drawing", "double_bottom_m"])
            + num(&c.results, &["drawing", "tank_support_height_m"]);
        for (i, (name, _, _)) in a.tanks.iter().enumerate() {
            let faults = faults_of(&a, i, (0.0, 0.0, -(bottom + 0.5)));
            assert!(
                faults.iter().any(|f| f.starts_with("crosses the hull")),
                "{}: {name} 0.5 m into the bottom passed: {faults:?}",
                c.at
            );
            moved += 1;
        }
    }
    assert!(moved >= 12, "{moved}");
}

#[test]
fn a_tank_moved_into_a_wing_tank_or_the_next_tank_fails_the_check() {
    // The other conditions can fail on their own: a tank moved 0.2 m past the gap to the wing tank
    // crosses the wing tank (and not the hull), and one moved 0.2 m past the gap to the next tank
    // along crosses that tank.
    let (mut wings, mut alongs) = (0, 0);
    for c in cases() {
        let Some(a) = arrangement(&c) else { continue };
        let d = &c.results["drawing"];
        let (wing, gap) = (num(d, &["wing_tank_width_m"]), num(d, &["wing_tank_gap_m"]));
        let gap_long = num(d, &["tank", "tank_gap_longitudinal_m"]);
        let last = a
            .tanks
            .iter()
            .map(|t| t.2.x1)
            .fold(f64::NEG_INFINITY, f64::max);
        for (i, (name, _, foot)) in a.tanks.iter().enumerate() {
            let side = if foot.y < 0.0 { -1.0 } else { 1.0 };
            let outer = a.tanks.iter().all(|t| side * t.2.y <= side * foot.y);
            if wing > 0.0 && outer {
                let faults = faults_of(&a, i, (0.0, side * (gap + 0.2), 0.0));
                assert!(
                    faults.iter().any(|f| f.starts_with("crosses WB")),
                    "{}: {name} into the wing tank passed: {faults:?}",
                    c.at
                );
                wings += 1;
            }
            if foot.x1 < last - 1e-6 {
                let faults = faults_of(&a, i, (gap_long + 0.2, 0.0, 0.0));
                assert!(
                    faults.iter().any(|f| f.starts_with("crosses CT")),
                    "{}: {name} into the next tank passed: {faults:?}",
                    c.at
                );
                alongs += 1;
            }
        }
    }
    assert!(wings >= 2 && alongs >= 2, "{wings} {alongs}");
}
