//! Seam: the Cargo Class (spec 0006 §6, issue #328). A class that has no path is an error, never the
//! gas path or the dry path.

use ovcd_core::barge_capacity::takes_gas_path;
use ovcd_core::cargo::{CARGO_DATABASE, CargoClass};

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
fn liquid_bulk_is_refused_never_sent_down_the_gas_path() {
    assert_eq!(takes_gas_path(CargoClass::Gas), Ok(true));
    assert_eq!(takes_gas_path(CargoClass::DryBulk), Ok(false));
    let e = takes_gas_path(CargoClass::LiquidBulk).unwrap_err();
    assert_eq!(e.kind, "ValueError");
    assert!(e.message.contains("liquid_bulk"), "{}", e.message);
}

#[test]
fn every_cargo_but_crude_oil_is_gas_or_dry_bulk() {
    for (key, props) in CARGO_DATABASE.iter() {
        if *key == "crude_oil" {
            assert_eq!(props.phase, CargoClass::LiquidBulk);
            continue;
        }
        assert!(
            matches!(props.phase, CargoClass::Gas | CargoClass::DryBulk),
            "{key}"
        );
    }
}
