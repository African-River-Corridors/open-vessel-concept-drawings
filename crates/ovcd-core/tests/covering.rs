//! Seam: the covering above the moulded depth D of a gas hull (issue #410). By default it is a
//! non-structural weather covering: the hull steel is weighed to D only, and the wing plate, the side
//! plating up to it and the weather shield are lightship items, each at its own centre. A user who
//! sets D over the tanks makes the covering structural.
//!
//! Label: property.

use ovcd_core::barge_capacity::{BargeCapacityInputs, compute};
use ovcd_core::gas_stability::{GasCapacityResult, assess};
use ovcd_core::hull::HullGeometry;
use ovcd_core::models::{BargeInputs, TankResult, WeightBreakdown};
use ovcd_core::tanks::calculate_tank_arrangement_wide;
use ovcd_core::weights::{MassItem, calculate_weights, covering_items, total_mass};

/// The quickstart gas barge of `tests/weights.rs`.
fn quickstart() -> BargeInputs {
    let mut i = BargeInputs::new(90.0, 14.8, 6.5, 6.5, 65.0);
    i.bow_rake_rise = Some(6.5);
    i.stern_rake_rise = Some(6.5);
    i.bow_non_tank_length = Some(10.9);
    i.stern_non_tank_length = Some(21.892000000000003);
    i.n_tanks_wide = 2;
    i.n_tanks_long = 2;
    i.double_bottom_height = Some(0.5);
    i.post_init().unwrap();
    i
}

fn weigh(i: &BargeInputs) -> (TankResult, WeightBreakdown, Vec<MassItem>) {
    let hull = HullGeometry::new(
        i.loa,
        i.beam,
        i.depth,
        i.bow_rake_length,
        i.bow_rake_rise.unwrap(),
        i.stern_rake_length,
        i.stern_rake_rise.unwrap(),
    );
    let (tanks, _) = calculate_tank_arrangement_wide(i).unwrap();
    let w = calculate_weights(i, &hull, &tanks).unwrap();
    let c = covering_items(i, &tanks).unwrap();
    (tanks, w, c)
}

fn item<'a>(items: &'a [MassItem], label: &str) -> &'a MassItem {
    items
        .iter()
        .find(|i| i.label == label)
        .unwrap_or_else(|| panic!("no {label} in {items:?}"))
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9 * b.abs().max(1.0)
}

/// (wing plate top, shield top) above the keel.
fn levels(i: &BargeInputs, t: &TankResult) -> (f64, f64) {
    let bottom = i.double_bottom_height.unwrap_or(0.0) + i.tank_support_height;
    (
        bottom + i.wing_tank_height_ratio * t.outer_diameter,
        bottom + t.outer_diameter + i.weather_shield_clearance,
    )
}

#[test]
fn with_the_wing_plate_above_d_the_covering_items_sit_at_their_own_heights() {
    let mut i = quickstart();
    i.wing_tank_height_ratio = 0.9;
    i.depth = 3.0;
    let (t, w, c) = weigh(&i);
    let (wing_top, shield_top) = levels(&i, &t);
    assert!(wing_top > i.depth + 1.0, "{wing_top}");

    // The hull steel uses D, not the wing plate.
    assert_eq!(w.moulded_depth, 3.0);
    assert_eq!(
        w.steel_top_down,
        i.steel_rate_per_cbm * (i.loa * i.beam * 3.0)
    );

    // Each covering item at its stated VCG.
    assert_eq!(c.len(), 4);
    assert_eq!(item(&c, "wing plate").kcg_m, wing_top);
    assert_eq!(
        item(&c, "side plating to the wing plate").kcg_m,
        (i.depth + wing_top) / 2.0
    );
    assert_eq!(item(&c, "weather shield top").kcg_m, shield_top);
    assert_eq!(
        item(&c, "weather shield sides and ends").kcg_m,
        (wing_top + shield_top) / 2.0
    );
    // The shield and the side plating are the ones the bottom-up steel weighs.
    assert!(close(
        item(&c, "weather shield top").mass_t + item(&c, "weather shield sides and ends").mass_t,
        w.weather_shield
    ));
    assert!(close(
        item(&c, "wing plate").mass_t + item(&c, "side plating to the wing plate").mass_t,
        w.wing_plate
    ));
    // The side plating is at mid-length, the shield at the tank zone centre.
    let zone_centre =
        (i.stern_non_tank_length.unwrap() + i.loa - i.bow_non_tank_length.unwrap()) / 2.0;
    assert_eq!(
        item(&c, "side plating to the wing plate").lcg_m,
        i.loa / 2.0
    );
    assert!(close(item(&c, "weather shield top").lcg_m, zone_centre));

    // The lightship holds them.
    let rest = w.steel_top_down + w.tank_total + w.insulation + w.piping_machinery + w.outfitting;
    assert!(close(w.lightship - rest, total_mass(&c)));
    // Piping and outfitting stay factors on the hull steel to D and the tanks.
    assert!(close(
        w.piping_machinery,
        i.piping_factor * (w.steel_top_down + w.tank_total)
    ));
}

#[test]
fn the_gas_stability_takes_the_covering_at_its_own_heights() {
    let mut ci = BargeCapacityInputs::new(110.0, 16.0, 4.0);
    ci.cargo_type = "LNG".into();
    let raw = compute(&ci).unwrap();
    let r = GasCapacityResult::try_from(&raw).unwrap();
    let b = r.gas_basis.clone().unwrap();
    assert!(!b.covering_items.is_empty());
    let light = |r: &GasCapacityResult| {
        let a = assess(r, 0.0, 0.85, 1.0, "IS_CODE_2008").unwrap();
        let c = a
            .conditions
            .iter()
            .find(|c| c.name == "Light")
            .unwrap()
            .clone();
        (c.kg_m, c.displacement_t)
    };
    let (kg1, disp1) = light(&r);
    let mut bare = r.clone();
    bare.gas_basis.as_mut().unwrap().covering_items.clear();
    let (kg0, disp0) = light(&bare);
    // The displacement grows by the covering mass, the vertical moment by its moment.
    assert!(close(disp1 - disp0, total_mass(&b.covering_items)));
    let moment: f64 = b.covering_items.iter().map(|i| i.mass_t * i.kcg_m).sum();
    assert!((kg1 * disp1 - kg0 * disp0 - moment).abs() < 1e-6);
    // The weights' lightship and the stability's light ship agree.
    assert!(close(disp1, raw.lightship_t), "{disp1} {}", raw.lightship_t);
}

#[test]
fn with_d_at_or_above_the_wing_plate_no_covering_item_sits_below_d() {
    let base = quickstart();
    let (t, _, _) = weigh(&base);
    let (wing_top, shield_top) = levels(&base, &t);

    // D at the wing plate: the wing plate and side plating are hull; the shield stands on the deck.
    let mut i = base.clone();
    i.depth = wing_top;
    let (_, w, c) = weigh(&i);
    assert_eq!(
        w.steel_top_down,
        i.steel_rate_per_cbm * (i.loa * i.beam * i.depth)
    );
    assert!(
        c.iter().all(|x| x.label.starts_with("weather shield")),
        "{c:?}"
    );
    let sides = item(&c, "weather shield sides and ends");
    assert_eq!(sides.kcg_m, (i.depth + shield_top) / 2.0);
    assert!(c.iter().all(|x| x.kcg_m > i.depth));

    // D between the wing plate and the shield top: only what is above D is covering.
    let mut i = base.clone();
    i.depth = (wing_top + shield_top) / 2.0;
    let (_, w, c) = weigh(&i);
    assert_eq!(
        w.steel_top_down,
        i.steel_rate_per_cbm * (i.loa * i.beam * i.depth)
    );
    assert_eq!(
        item(&c, "weather shield sides and ends").kcg_m,
        (i.depth + shield_top) / 2.0
    );
    assert!(c.iter().all(|x| x.kcg_m > i.depth));

    // D at or above the shield top: the covering is all structural, so no item.
    for depth in [shield_top, shield_top + 1.0] {
        let mut i = base.clone();
        i.depth = depth;
        let (_, w, c) = weigh(&i);
        assert!(c.is_empty(), "{c:?}");
        assert_eq!(w.moulded_depth, depth);
        assert_eq!(
            w.steel_top_down,
            i.steel_rate_per_cbm * (i.loa * i.beam * depth)
        );
        let rest =
            w.steel_top_down + w.tank_total + w.insulation + w.piping_machinery + w.outfitting;
        assert_eq!(w.lightship, rest);
    }
}

#[test]
fn no_wing_tank_means_no_covering() {
    let mut i = quickstart();
    i.wing_tank_width = Some(0.0);
    i.depth = 3.0;
    assert!(weigh(&i).2.is_empty());
}
