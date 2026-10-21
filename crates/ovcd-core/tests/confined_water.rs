//! Seam: `ovcd_core::confined_water`, one midship section and one blockage ratio for squat and
//! resistance, and the Schijf limiting speed (issues #396 and #397).
//! Label: sourced - Delefortrie, Verwilligen, Vantorre, Lataire 2024, "Exact and Approximated
//! Solutions to the Critical Ship Speeds in Canals", 35th PIANC World Congress, Paper 8, section 4:
//! Fr_h,crit1 = [2 sin(arcsin(1 - m) / 3)]^(3/2); and hand calculation (Am = B x T).

use ovcd_core::confined_water::{blockage_ratio, critical_froude_depth, midship_section_area};
use ovcd_core::resistance::{ResistanceInputs, compute};
use ovcd_core::squat::{self, SquatInputs};

#[test]
fn the_critical_froude_depth_number_is_one_in_open_water() {
    assert!((critical_froude_depth(0.0) - 1.0).abs() < 1e-12);
}

#[test]
fn the_critical_froude_depth_number_at_a_blockage_of_0_2_is_the_source_formula() {
    // [2 sin(arcsin(0.8) / 3)]^1.5: arcsin 0.8 = 0.927295218 rad; / 3 = 0.309098406;
    // sin = 0.304213; x 2 = 0.608426; ^1.5 = 0.474552.
    assert!((critical_froude_depth(0.2) - 0.474_551_752_757_747).abs() < 1e-12);
    // A vessel that fills the channel has no speed.
    assert_eq!(critical_froude_depth(1.0), 0.0);
}

#[test]
fn the_critical_froude_depth_number_falls_as_the_blockage_grows() {
    let mut last = critical_froude_depth(0.0);
    for i in 1..100 {
        let f = critical_froude_depth(f64::from(i) / 100.0);
        assert!(f < last, "{i}");
        last = f;
    }
}

#[test]
fn the_midship_section_is_beam_times_draught() {
    assert_eq!(midship_section_area(11.4, 3.0), 34.2);
    assert_eq!(blockage_ratio(34.2, 171.0), 0.2);
    assert_eq!(blockage_ratio(34.2, 0.0), 0.0);
}

#[test]
fn squat_and_resistance_give_one_hull_the_same_blockage_ratio() {
    let inp = ResistanceInputs::default();
    let r = compute(&inp).unwrap();
    for (env_in, env) in inp.environments.iter().zip(&r.environments) {
        if env_in.is_deep_water {
            continue;
        }
        let area =
            (env_in.channel_width_bottom + env_in.channel_width_surface) / 2.0 * env_in.water_depth;
        let mut s = SquatInputs::new(5.0, inp.draft, inp.beam, inp.lwl);
        s.block_coefficient = inp.block_coefficient;
        s.water_depth_m = env_in.water_depth;
        s.channel_area_m2 = area;
        let sq = squat::compute(&s).unwrap();
        assert_eq!(sq.blockage_ratio, env.blockage_ratio, "{}", env.name);
        assert_eq!(
            env.blockage_ratio,
            inp.beam * inp.draft / area,
            "{}",
            env.name
        );
    }
}
