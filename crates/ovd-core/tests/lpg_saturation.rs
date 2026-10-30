//! Seam: saturated LPG density and vapour pressure, and the IGC 15.5.1 loading limit (issue #315).
//!
//! The tables are the NIST Chemistry WebBook saturation properties of propane and n-butane, kept in
//! `catalogue/lpg_saturation/`. Worked case, by hand from those rows: fully pressurised propane,
//! MARVS 17.7 barg = 18.71325 bar absolute. The vapour pressure is 18.672 bar at 54 C and 19.072 bar
//! at 55 C, so the reference temperature (IGC 15.1.3.1) is 54 + 0.04125 / 0.4 = 54.103 C, where the
//! density is 440.83 - 0.103 x 2.07 = 440.62 kg/m3. Loaded at 15 C (507.50 kg/m3), LL = 0.98 x
//! 440.62 / 507.50 = 0.8508 (IGC 15.5.1). The cargo mass per m3 of tank is LL x rho_L = 0.98 x
//! rho_R = 431.80 kg.
//!
//! Label: sourced - NIST Chemistry WebBook, saturation properties of propane and n-butane (catalogue/lpg_saturation/); IGC Code 15.1.3.1 and 15.5.1 (MSC.370(93)); the worked case is a hand calculation (above).

use ovd_core::lpg_saturation::{
    ATMOSPHERE_BAR, N_BUTANE, PROPANE, TABLE_LOW_C, liquid_density_kg_m3, loading_limit,
    mix_propane_mole_fraction, saturation_temperature_c, vapour_pressure_bar,
};

fn catalogue(name: &str) -> Vec<(f64, f64, f64)> {
    let path = format!(
        "{}/../../catalogue/lpg_saturation/{name}.tsv",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{path}: {e}"))
        .lines()
        .skip(1)
        .map(|l| {
            let v: Vec<f64> = l.split('\t').map(|x| x.parse().unwrap()).collect();
            (v[0], v[1], v[2])
        })
        .collect()
}

#[test]
fn the_tables_are_the_nist_rows_of_the_catalogue() {
    for (name, table) in [
        ("nist_propane", &PROPANE[..]),
        ("nist_n_butane", &N_BUTANE[..]),
    ] {
        let rows = catalogue(name);
        assert_eq!(rows.len(), table.len(), "{name}");
        for (i, (t, p, rho)) in rows.iter().enumerate() {
            assert_eq!(*t, TABLE_LOW_C + i as f64, "{name}: one row every 1 C");
            assert_eq!((*p, *rho), table[i], "{name} at {t} C");
        }
    }
}

#[test]
fn the_density_and_the_pressure_are_the_nist_values_at_a_row() {
    // NIST rows, as printed.
    assert_eq!(liquid_density_kg_m3("LPG_propane", -42.0), Some(580.75));
    assert_eq!(liquid_density_kg_m3("LPG_propane", 15.0), Some(507.50));
    assert_eq!(liquid_density_kg_m3("LPG_propane", 45.0), Some(458.40));
    assert_eq!(liquid_density_kg_m3("LPG_butane", -1.0), Some(601.81));
    assert_eq!(liquid_density_kg_m3("LPG_butane", 45.0), Some(548.70));
    assert_eq!(vapour_pressure_bar("LPG_propane", 50.0), Some(17.133));
    // Between rows: the straight line.
    let mid = liquid_density_kg_m3("LPG_propane", 14.5).unwrap();
    let (a, b) = (
        liquid_density_kg_m3("LPG_propane", 14.0).unwrap(),
        liquid_density_kg_m3("LPG_propane", 15.0).unwrap(),
    );
    assert!((mid - (a + b) / 2.0).abs() < 1e-9);
    // Outside the table, and another Cargo: none.
    assert_eq!(liquid_density_kg_m3("LPG_propane", -60.0), None);
    assert_eq!(liquid_density_kg_m3("LPG_propane", 81.0), None);
    assert_eq!(liquid_density_kg_m3("LNG", 0.0), None);
    assert_eq!(liquid_density_kg_m3("LPG_propane", f64::NAN), None);
}

#[test]
fn the_igc_worked_case_for_fully_pressurised_propane() {
    let p = 17.7 + ATMOSPHERE_BAR;
    let t_r = saturation_temperature_c("LPG_propane", p).unwrap();
    assert!((t_r - 54.103125).abs() < 1e-9, "{t_r}");
    let rho_r = liquid_density_kg_m3("LPG_propane", t_r).unwrap();
    assert!((rho_r - 440.61653125).abs() < 1e-9, "{rho_r}");
    let rho_l = liquid_density_kg_m3("LPG_propane", 15.0).unwrap();
    let ll = loading_limit(0.98, rho_r, rho_l);
    assert!((ll - 0.8508457).abs() < 1e-6, "{ll}");
    assert!((ll * rho_l - 0.98 * rho_r).abs() < 1e-9);
    // The saturation temperature is the inverse of the vapour pressure.
    let back = vapour_pressure_bar("LPG_propane", t_r).unwrap();
    assert!((back - p).abs() < 1e-9);
}

#[test]
fn the_mix_is_an_ideal_solution_of_its_two_parts() {
    // 60/40 by mass is 66.41 % propane by moles.
    let x = mix_propane_mole_fraction();
    assert!((x - 0.66406).abs() < 5e-5, "{x}");
    let t = -10.0;
    let (rp, rb) = (
        liquid_density_kg_m3("LPG_propane", t).unwrap(),
        liquid_density_kg_m3("LPG_butane", t).unwrap(),
    );
    let rho = liquid_density_kg_m3("LPG_mix", t).unwrap();
    assert!((1.0 / rho - (0.6 / rp + 0.4 / rb)).abs() < 1e-15);
    assert!(rp < rho && rho < rb);
    let p = vapour_pressure_bar("LPG_mix", t).unwrap();
    let (pp, pb) = (
        vapour_pressure_bar("LPG_propane", t).unwrap(),
        vapour_pressure_bar("LPG_butane", t).unwrap(),
    );
    assert!((p - (x * pp + (1.0 - x) * pb)).abs() < 1e-12);
    let back = saturation_temperature_c("LPG_mix", p).unwrap();
    assert!((back - t).abs() < 1e-9);
}

#[test]
fn a_pressure_outside_the_table_has_no_saturation_temperature() {
    assert_eq!(saturation_temperature_c("LPG_propane", 0.01), None);
    assert_eq!(saturation_temperature_c("LPG_propane", 100.0), None);
    assert_eq!(saturation_temperature_c("LNG", 5.0), None);
}
