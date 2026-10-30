//! Seam: `ovd_format::eval` result tagging. A non-finite float result is a tagged string (JSON
//! has no `inf` or `nan`). Expected values are what the original Python printed under CPython 3.12.

use ovd_format::eval;
use serde_json::json;

#[test]
fn overflowing_thickness_is_tagged_not_null() {
    let input = json!({"fn": "cylinder_wall_thickness", "args": {
        "design_pressure_mpa": 1e308, "inner_radius_mm": 10.0, "allowable_stress_mpa": 113.0}});
    assert_eq!(
        eval("pressure_vessel", &input).unwrap(),
        json!({"ok": "float:-inf"})
    );
}

#[test]
fn overflowing_clearance_field_is_tagged() {
    let input = json!({"fn": "check_protective_location", "args": {
        "vessel_type_str": "IGF", "beam": 10.0, "total_tank_volume": 100.0,
        "side_clearance": 2.0, "double_bottom_height": 1.7e308, "tank_support_height": 1.7e308}});
    let out = eval("protective_location", &input).unwrap();
    assert_eq!(out["ok"]["cb_actual"], json!("float:inf"));
}

#[test]
fn nan_result_is_tagged() {
    let input = json!({"fn": "cylinder_wall_thickness", "args": {
        "design_pressure_mpa": 1e308, "inner_radius_mm": 1e308,
        "allowable_stress_mpa": 1e308, "weld_efficiency": 1e308}});
    assert_eq!(
        eval("pressure_vessel", &input).unwrap(),
        json!({"ok": "float:nan"})
    );
}

#[test]
fn unprintable_unicode_in_vessel_type_error_is_escaped() {
    let input = json!({"fn": "compute_required_clearances", "args": {
        "vessel_type_str": "\u{a0}\u{2028}\u{200b}\u{1F600}\u{e0001}\u{e9}", "beam": 10.0, "d_vc": 1.0}});
    assert_eq!(
        eval("protective_location", &input).unwrap(),
        json!({"error": {"type": "ValueError", "message":
            "'\\xa0\\u2028\\u200b\u{1F600}\\U000e0001\u{e9}' is not a valid VesselType"}})
    );
}
