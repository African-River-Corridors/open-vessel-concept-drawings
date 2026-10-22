//! Seam: the panel mesh and offsets exports (issue #461, spec 0008 "Phase 2: exports"):
//! `ovcd_format::export::{option_export, stl_binary, obj_text, clip_below, option_panel_mesh}`.
//!
//! Label: property. For every fixture Option the exported hull is closed (every edge in exactly two
//! triangles, once in each direction, found from the file's own coordinates, not the mesh's indices)
//! and its divergence-theorem volume matches the sections within the ruled-quad error; clipped at
//! each loading condition's waterline it is closed again and holds the immersed volume of the
//! sections there within 1 % (the spec's Capytaine bound); the STL has the length binary STL has and
//! the OBJ the faces of the mesh; the offsets CSV of an offsets Option read back gives the same
//! results hash.

use std::collections::HashMap;
use std::path::Path;

use ovcd_core::sections::{SectionsTable, Waterplane, immersed};
use ovcd_format::export::{obj_text, option_export, option_panel_mesh, stl_binary};
use ovcd_format::{
    Profile, RunOutcome, check, option_hull_table, option_model_waterlines, results_hash,
    run_design,
};
use serde_json::{Value, json};

type Triangle = [[f32; 3]; 3];

/// The triangles of a binary STL, after the length check.
fn read_stl(b: &[u8]) -> Vec<Triangle> {
    let n = u32::from_le_bytes(b[80..84].try_into().unwrap()) as usize;
    assert_eq!(b.len(), 84 + 50 * n, "a binary STL is 84 + 50 n bytes");
    (0..n)
        .map(|k| {
            let at = 84 + 50 * k + 12;
            let f =
                |i: usize| f32::from_le_bytes(b[at + 4 * i..at + 4 * i + 4].try_into().unwrap());
            [[f(0), f(1), f(2)], [f(3), f(4), f(5)], [f(6), f(7), f(8)]]
        })
        .collect()
}

/// Every directed edge of the triangles, by coordinates, is in exactly one triangle and its
/// reverse in exactly one: the mesh is closed and consistently oriented.
fn closed(tris: &[Triangle]) -> Result<(), String> {
    let key = |p: [f32; 3]| p.map(|v| (v + 0.0).to_bits());
    let mut count: HashMap<([u32; 3], [u32; 3]), i32> = HashMap::new();
    for t in tris {
        for k in 0..3 {
            *count.entry((key(t[k]), key(t[(k + 1) % 3]))).or_default() += 1;
        }
    }
    for (&(a, b), &n) in &count {
        if n != 1 || count.get(&(b, a)) != Some(&1) {
            return Err(format!("edge {a:?} {b:?} ({n} times)"));
        }
    }
    Ok(())
}

fn volume(tris: &[Triangle]) -> f64 {
    tris.iter()
        .map(|t| {
            let [a, b, c] = t.map(|p| p.map(f64::from));
            (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                + a[2] * (b[0] * c[1] - b[1] * c[0]))
                / 6.0
        })
        .sum()
}

/// Half the tetrahedron volume of every ruled quad: the bound on the volume the mesh's planar
/// triangles can differ from the ruled surface by.
fn quad_error(t: &SectionsTable) -> f64 {
    let tet = |a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]| {
        let (u, v, w) = (
            [b[0] - a[0], b[1] - a[1], b[2] - a[2]],
            [c[0] - a[0], c[1] - a[1], c[2] - a[2]],
            [d[0] - a[0], d[1] - a[1], d[2] - a[2]],
        );
        (u[0] * (v[1] * w[2] - v[2] * w[1]) - u[1] * (v[0] * w[2] - v[2] * w[0])
            + u[2] * (v[0] * w[1] - v[1] * w[0]))
            .abs()
            / 6.0
    };
    let mut e = 0.0;
    for p in t.sections.windows(2) {
        let (s0, s1) = (&p[0], &p[1]);
        for i in 0..s0.points.len() - 1 {
            let at =
                |s: &ovcd_core::sections::Section, j: usize| [s.x, s.points[j].0, s.points[j].1];
            e += tet(at(s0, i), at(s0, i + 1), at(s1, i + 1), at(s1, i));
        }
    }
    e
}

fn fixtures() -> Vec<std::path::PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "vdesign"))
        .collect();
    files.sort();
    files
}

#[test]
fn every_fixture_hull_is_a_closed_stl_and_obj_with_the_sections_volume() {
    let mut checked = 0;
    for path in fixtures() {
        let text = std::fs::read_to_string(&path).unwrap();
        let design = check(&text, &Profile::built_in())
            .design
            .expect("a valid fixture");
        for option in &design.options {
            let Ok(mesh) = option_panel_mesh(option, None) else {
                continue;
            };
            let at = format!("{} / {}", path.display(), option.name);
            let stl = option_export("stl", option, None).unwrap();
            let tris = read_stl(&stl);
            assert_eq!(tris.len() * 3, mesh.indices.len(), "{at}");
            closed(&tris).unwrap_or_else(|e| panic!("{at}: {e}"));
            let table = option_hull_table(option).unwrap();
            // The gas search may change the depth, so a gas hull is checked by closure and sign only.
            let v = volume(&tris);
            assert!(v > 0.0, "{at}: volume {v}");
            if !path.to_string_lossy().contains("gas") {
                let exact = immersed(&table, &Waterplane::upright(1e6)).volume_m3;
                assert!(
                    (v - exact).abs() <= quad_error(&table) + 1e-6 * exact,
                    "{at}: {v} vs {exact}"
                );
            }
            // The OBJ has the same vertices and faces, one-based.
            let obj = String::from_utf8(option_export("obj", option, None).unwrap()).unwrap();
            assert_eq!(obj, obj_text(&mesh));
            let vs = obj.lines().filter(|l| l.starts_with("v ")).count();
            let fs: Vec<Vec<usize>> = obj
                .lines()
                .filter(|l| l.starts_with("f "))
                .map(|l| l[2..].split(' ').map(|i| i.parse().unwrap()).collect())
                .collect();
            assert_eq!(vs * 3, mesh.positions.len(), "{at}");
            assert_eq!(fs.len() * 3, mesh.indices.len(), "{at}");
            assert!(fs.iter().flatten().all(|&i| (1..=vs).contains(&i)), "{at}");
            assert_eq!(stl_binary(&mesh), stl, "{at}");
            checked += 1;
        }
    }
    assert!(checked >= 8, "{checked}");
}

#[test]
fn clipped_at_a_condition_the_hull_is_closed_with_the_immersed_volume_of_the_sections() {
    let mut checked = 0;
    for path in fixtures() {
        let text = std::fs::read_to_string(&path).unwrap();
        let design = check(&text, &Profile::built_in())
            .design
            .expect("a valid fixture");
        for option in &design.options {
            let Ok((model, waterlines)) = option_model_waterlines(option) else {
                continue;
            };
            let l = model.hull.sections.last().unwrap().x - model.hull.sections[0].x;
            for (name, aft, fwd) in waterlines {
                let at = format!("{} / {} / {name}", path.display(), option.name);
                let stl = option_export("stl", option, Some(&name)).unwrap();
                let tris = read_stl(&stl);
                if tris.is_empty() {
                    continue;
                }
                closed(&tris).unwrap_or_else(|e| panic!("{at}: {e}"));
                // The sections' immersed volume at the same plane: c = aft draught cos(trim).
                let trim = libm::atan((fwd - aft) / l) * 180.0 / std::f64::consts::PI;
                let c = aft * libm::cos(trim * std::f64::consts::PI / 180.0);
                let exact = immersed(
                    &model.hull,
                    &Waterplane {
                        heel_deg: 0.0,
                        trim_deg: trim,
                        c,
                    },
                )
                .volume_m3;
                let v = volume(&tris);
                assert!((v - exact).abs() <= 0.01 * exact, "{at}: {v} vs {exact}");
                checked += 1;
            }
        }
    }
    assert!(checked >= 8, "{checked}");
}

#[test]
fn a_clip_names_its_condition_or_says_which_there_are() {
    let text = std::fs::read_to_string(&fixtures()[0]).unwrap();
    let design = check(&text, &Profile::built_in()).design.unwrap();
    let e = option_panel_mesh(&design.options[0], Some("No such condition")).unwrap_err();
    assert!(e.contains("no condition named `No such condition`"), "{e}");
    let e = option_export("gltf", &design.options[0], Some("Laden")).unwrap_err();
    assert!(e.contains("--below clips a mesh"), "{e}");
    let e = option_export("step", &design.options[0], None).unwrap_err();
    assert!(e.contains("unknown format"), "{e}");
}

#[test]
fn the_offsets_csv_of_an_offsets_option_read_back_gives_the_same_results_hash() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/offsets_dry_bulk.vdesign");
    let text = std::fs::read_to_string(path).unwrap();
    let design = check(&text, &Profile::built_in()).design.unwrap();
    let option = &design.options[0];
    let csv = String::from_utf8(option_export("offsets-csv", option, None).unwrap()).unwrap();
    assert!(csv.starts_with("station_x_m,y_m,z_m\n"));
    let rows = ovcd_format::offsets::parse_csv(&csv).unwrap();
    let hash = |text: &str| -> String {
        let design = check(text, &Profile::built_in()).design.unwrap();
        match run_design(&design).remove(0).outcome {
            RunOutcome::Results { results, .. } => results_hash(&results),
            other => panic!("{other:?}"),
        }
    };
    let mut file: Value = serde_json::from_str(&text).unwrap();
    file["options"][0]["inputs"]["hull_offsets"] =
        json!({ "value": ovcd_format::offsets::rows_json(&rows), "source": "typed" });
    file["options"][0]
        .as_object_mut()
        .unwrap()
        .remove("results_hash");
    assert_eq!(hash(&file.to_string()), hash(&text));
}
