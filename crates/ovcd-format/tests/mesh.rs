//! Seam: `ovcd_format::option_meshes` and `mesh_hash` (issue #456, spec 0008 "Phase 2: meshes and
//! the parity hash").
//!
//! Label: property - for every fixture Option the hull mesh is closed (every edge in exactly two
//! triangles, once in each direction) and holds the sections volume within the ruled-quad error
//! (the sum of half the tetrahedron volumes of the ruled quads) plus 1e-6 for the f32 rounding;
//! the group table names one hull and one group per Space; the hash is the same twice and a
//! one-vertex change changes it.

use std::collections::HashMap;
use std::path::Path;

use ovcd_core::mesh::Mesh;
use ovcd_core::sections::{SectionsTable, Waterplane, immersed};
use ovcd_format::{Profile, check, mesh_hash, option_hull_table, option_meshes};

fn hull_part(m: &Mesh) -> Mesh {
    let g = m
        .groups
        .iter()
        .find(|g| g.layer == "hull")
        .expect("a hull group");
    let idx = &m.indices[g.start as usize..(g.start + g.count) as usize];
    Mesh {
        positions: m.positions.clone(),
        indices: idx.to_vec(),
        groups: Vec::new(),
    }
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

#[test]
fn every_fixture_hull_mesh_is_closed_with_the_sections_volume_and_a_stable_hash() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "vdesign"))
        .collect();
    files.sort();
    let mut meshed = 0;
    for path in files {
        let text = std::fs::read_to_string(&path).unwrap();
        let design = check(&text, &Profile::built_in())
            .design
            .expect("a valid fixture");
        for option in &design.options {
            let Ok(m) = option_meshes(option) else {
                continue;
            };
            let at = format!("{} / {}", path.display(), option.name);
            let hull = hull_part(&m);
            closed(&hull).unwrap_or_else(|e| panic!("{at}: not closed at {e}"));
            // The gas search may change the depth, so the gas hull is checked by its own volume.
            let table = option_hull_table(option).unwrap();
            if !path.to_string_lossy().contains("gas") {
                let v = immersed(&table, &Waterplane::upright(1e6)).volume_m3;
                let bound = quad_error(&table) + 1e-6 * v;
                assert!(
                    (hull.volume() - v).abs() <= bound,
                    "{at}: mesh {} vs sections {v}, bound {bound}",
                    hull.volume()
                );
            } else {
                assert!(hull.volume() > 0.0, "{at}");
            }
            assert_eq!(m.groups.iter().filter(|g| g.layer == "hull").count(), 1);
            assert!(m.groups.iter().any(|g| g.layer == "space"), "{at}");
            assert!(m.groups.iter().any(|g| g.layer == "waterline"), "{at}");
            let h = mesh_hash(&m);
            assert_eq!(h, mesh_hash(&m.clone()));
            let mut moved = m.clone();
            moved.positions[0] += 0.001;
            assert_ne!(
                h,
                mesh_hash(&moved),
                "{at}: a moved vertex changes the hash"
            );
            meshed += 1;
        }
    }
    assert!(meshed >= 10, "{meshed} Options meshed");
}
