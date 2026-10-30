//! Seam: the public `ovd_core::waves` API. Expected values are what the original Python printed.
//! Label: snapshot

use ovd_core::waves::{WaveResistanceInputs, compute};

fn base(hs: f64) -> WaveResistanceInputs {
    WaveResistanceInputs {
        speed_kn: 8.5,
        ..WaveResistanceInputs::new(hs, 14.8, 87.3)
    }
}

#[test]
fn one_metre_head_seas_on_the_example_hull() {
    let r = compute(&base(1.0)).unwrap();
    assert!((r.added_resistance_kn - 8.560418792202103).abs() < 1e-12);
    assert_eq!(r.heading_factor, 1.0);
    assert!((r.bow_waterline_length_m - 17.46).abs() < 1e-12);
    assert!((r.added_power_kw - 37.43280906301265).abs() < 1e-12);
    // Re-pinned (issue #526); was 37.43277672365276.
    assert!(r.ok());
    assert_eq!(
        r.checks[2].message,
        "L_BWL not given, assumed 17.5 m = 0.2 x Lpp (a blunt barge bow). A finer bow reduces \
         added resistance, so this is the conservative assumption"
    );
    assert_eq!(
        r.warnings,
        vec![
            "no wave period given; T_p estimated at 4.0 s from H_s, which sets only the \
             validity check, not the resistance"
        ]
    );
}

#[test]
fn abeam_the_heading_factor_is_exactly_zero_not_cos_residue() {
    let inp = WaveResistanceInputs {
        heading_deg_off_bow: 90.0,
        ..base(1.0)
    };
    let r = compute(&inp).unwrap();
    assert_eq!(r.heading_factor, 0.0);
    assert_eq!(r.added_resistance_kn, 0.0);
    assert_eq!(r.warnings.len(), 2);
    assert!(r.warnings[1].starts_with("beam-ish seas: this module returns a FLOOR"));
}

#[test]
fn flat_water_has_zero_wavelength_and_a_failed_validity_check() {
    let r = compute(&base(0.0)).unwrap();
    assert_eq!(r.wavelength_m, 0.0);
    assert!(!r.checks[0].ok);
    assert!(
        r.checks[0]
            .message
            .starts_with("lambda/Lpp = 0.00 (lambda 0 m on Lpp 87 m).")
    );
    assert!(r.warnings.is_empty());
}

#[test]
fn bad_inputs_fail_with_the_python_messages() {
    let err = |inp: WaveResistanceInputs| compute(&inp).unwrap_err();
    assert_eq!(
        err(WaveResistanceInputs::new(-1.0, 14.8, 87.3)),
        "significant_wave_height_m must be >= 0"
    );
    assert_eq!(
        err(WaveResistanceInputs::new(1.0, 0.0, 87.3)),
        "beam_waterline_m must be > 0"
    );
    assert_eq!(
        err(WaveResistanceInputs::new(1.0, 14.8, 0.0)),
        "lpp_m must be > 0"
    );
    let h = WaveResistanceInputs {
        heading_deg_off_bow: 200.0,
        ..WaveResistanceInputs::new(1.0, 14.8, 87.3)
    };
    assert_eq!(err(h), "heading_deg_off_bow must be 0-180");
}
