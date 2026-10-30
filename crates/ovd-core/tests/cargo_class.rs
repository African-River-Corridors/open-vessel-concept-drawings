//! Seam: the Cargo Class (spec 0006 §6, issue #328). A class that has no path is an error, never the
//! gas path or the dry path.

use ovd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovd_core::cargo::{CARGO_DATABASE, CargoClass};

#[test]
fn every_key_reads_back_to_its_class() {
    for class in CargoClass::ALL {
        assert_eq!(CargoClass::from_key(class.key()), Ok(class));
    }
    assert_eq!(CargoClass::Gas.key(), "liquid_gas");
    assert_eq!(CargoClass::DryBulk.key(), "dry_bulk");
    assert_eq!(CargoClass::LiquidBulk.key(), "liquid_bulk");
}

#[test]
fn an_unknown_key_is_an_error_not_a_default_class() {
    for key in ["", "crude_oil", "liquid", "gas", "Dry_Bulk"] {
        let e = CargoClass::from_key(key).unwrap_err();
        assert!(e.contains("Unknown cargo class"), "{key}: {e}");
    }
}

#[test]
fn liquid_bulk_without_a_tank_layout_is_refused_never_sent_down_the_gas_or_dry_path() {
    let mut inp = BargeCapacityInputs::new(76.5, 11.4, 3.5);
    inp.cargo_type = "crude_oil".to_string();
    // Like a gas hull that cannot hold its tanks: infeasible, with the reason, not a silent path.
    let r = compute(&inp).unwrap();
    assert!(r.liquid.is_none());
    let feasible = r
        .checks
        .iter()
        .find(|c| c.name == "feasible_geometry")
        .unwrap();
    assert!(!feasible.ok);
    assert!(
        feasible.message.contains("cargo tank layout"),
        "{}",
        feasible.message
    );
    assert_eq!(r.cargo_capacity_t, 0.0);
}

#[test]
fn every_cargo_but_the_oils_is_gas_or_dry_bulk() {
    for (key, props) in CARGO_DATABASE.iter() {
        if matches!(*key, "crude_oil" | "diesel_en590") {
            assert_eq!(props.phase, CargoClass::LiquidBulk);
            continue;
        }
        assert!(
            matches!(props.phase, CargoClass::Gas | CargoClass::DryBulk),
            "{key}"
        );
    }
}
