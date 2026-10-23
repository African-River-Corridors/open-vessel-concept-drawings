//! Seam: the Vessel Model calls of the 3D view on `AppState` (issue #457), the ones the desktop
//! commands and the WebAssembly session forward.
//!
//! Label: property - the meshes and their hash are the ones `ovcd mesh --hash` gives; `section_at`
//! at a Station is the table's Section and between two it is the ruled interpolation; a Space's
//! values are the Vessel Model's; each waterline names its mesh group.

use std::path::Path;

use ovcd_format::{mesh_hash, option_hull_table, option_meshes, option_vessel_model};
use ovcd_shell::AppState;

fn open(name: &str) -> AppState {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../ovcd-format/tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(&path).unwrap();
    AppState::open(&path, &text).unwrap()
}

#[test]
fn the_3d_calls_give_the_cores_meshes_sections_spaces_and_waterlines() {
    let mut s = open("dry_bulk_box_hold.vdesign");
    let option = s.session().design().options[0].clone();
    let name = option.name.clone();

    let mesh = option_meshes(&option).unwrap();
    let (positions, indices) = s.mesh_buffers(&name).unwrap();
    assert_eq!(
        (positions, indices),
        (mesh.positions.clone(), mesh.indices.clone())
    );
    let groups = s.mesh_groups(&name).unwrap();
    assert_eq!(groups["hash"], mesh_hash(&mesh));
    assert_eq!(
        groups["groups"].as_array().unwrap().len(),
        mesh.groups.len()
    );

    let table = option_hull_table(&option).unwrap();
    // At a Station: that Section.
    let st = &table.sections[1];
    let at = s.section_at(&name, st.x).unwrap();
    let ring = at["rings"][0].as_array().unwrap();
    for (i, p) in st.points.iter().enumerate() {
        assert_eq!(ring[i][0].as_f64().unwrap(), p.0);
        assert_eq!(ring[i][1].as_f64().unwrap(), p.1);
    }
    // Between two: the ruled interpolation.
    let (a, b) = (&table.sections[1], &table.sections[2]);
    let x = 0.3 * a.x + 0.7 * b.x;
    let (want, depth) = table.section_at(x);
    let at = s.section_at(&name, x).unwrap();
    assert_eq!(at["depth_at_side_m"].as_f64().unwrap(), depth);
    let ring = at["rings"][0].as_array().unwrap();
    for (i, p) in want.points.iter().enumerate() {
        assert_eq!(ring[i][0].as_f64().unwrap(), p.0);
        assert_eq!(ring[i][1].as_f64().unwrap(), p.1);
    }
    // The mirror closes the full section.
    assert_eq!(ring.len(), 2 * want.points.len() - 2);

    let model = option_vessel_model(&option).unwrap();
    let sp = &model.spaces[0];
    let info = s.space_info(&name, &sp.space.name).unwrap();
    assert_eq!(info["volume_m3"].as_f64().unwrap(), sp.volume_m3);
    assert_eq!(info["lcg_m"].as_f64().unwrap(), sp.centroid_m.0);
    assert_eq!(info["kind"], sp.space.kind.key());
    assert!(s.space_info(&name, "no such space").is_err());

    let lines = s.waterlines(&name).unwrap();
    let lines = lines.as_array().unwrap();
    assert!(!lines.is_empty());
    for l in lines {
        let g = l["group"].as_u64().unwrap() as usize;
        assert_eq!(mesh.groups[g].layer, "waterline");
        assert_eq!(mesh.groups[g].id, l["name"].as_str().unwrap());
    }
    assert!(s.mesh_groups("no such option").is_err());
}

#[test]
fn a_changed_option_is_run_again() {
    let mut s = open("dry_bulk_box_hold.vdesign");
    let name = s.session().design().options[0].name.clone();
    let before = s.mesh_groups(&name).unwrap()["hash"].clone();
    s.set_input(&name, "length_overall_m", serde_json::json!(80.0))
        .unwrap();
    let after = s.mesh_groups(&name).unwrap()["hash"].clone();
    assert_ne!(before, after);
    let option = s.session().design().options[0].clone();
    assert_eq!(after, mesh_hash(&option_meshes(&option).unwrap()));
}
