//! Calibration of the calm-water resistance methods against barge towing-tank data (issue #735).
//! Label: sourced - calibration/resistance_cases.csv (Zentari 2023 thesis, Figs 5.9 and 5.10;
//! Tabaczek and Zawislak 2018, Fig. 4), each row with its citation in the file header.
//!
//! Every case is compared at model scale: the published CT against each method's CT with the test
//! water's viscosity and no correlation allowance. `cargo test -p ovd-core --test
//! resistance_calibration -- --nocapture` prints the error table.

use ovd_core::holtrop_mennen::{self, Flow, HullInputs, Variant};
use ovd_core::resistance::{
    BARGE_FORM_FACTOR_ENVELOPE, EnvironmentInputs, ResistanceCalculator, ResistanceInputs,
    barge_form_factor_ratio, residuary_ratio,
};

const CASES: &str = include_str!("../../../calibration/resistance_cases.csv");
const G: f64 = 9.81;

#[derive(Debug, Clone)]
struct Case {
    id: String,
    lwl: f64,
    beam: f64,
    draught: f64,
    volume: f64,
    wetted: f64,
    speed: f64,
    nu: f64,
    depth: f64,
    width: f64,
    ct: f64,
}

fn cases() -> Vec<Case> {
    CASES
        .lines()
        .filter(|l| !l.starts_with('#') && !l.starts_with("id,") && !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(',').collect();
            let n = |i: usize| f[i].trim().parse::<f64>().unwrap();
            Case {
                id: f[0].to_string(),
                lwl: n(2),
                beam: n(3),
                draught: n(4),
                volume: n(5),
                wetted: n(6),
                speed: n(7),
                nu: n(8),
                depth: n(9),
                width: n(10),
                ct: n(11) / 1000.0,
            }
        })
        .collect()
}

fn cf_ittc(c: &Case) -> f64 {
    let rn = c.speed * c.lwl / c.nu;
    0.075 / libm::pow(libm::log10(rn) - 2.0, 2.0)
}

fn froude(c: &Case) -> f64 {
    c.speed / libm::sqrt(G * c.lwl)
}

fn cb(c: &Case) -> f64 {
    c.volume / (c.lwl * c.beam * c.draught)
}

/// OVD's shallow and confined water factor for this case (1 in deep water).
fn shallow_factor(c: &Case) -> f64 {
    let env = EnvironmentInputs {
        name: "tank".to_string(),
        water_depth: if c.depth > 0.0 { c.depth } else { 100.0 },
        channel_width_bottom: c.width,
        channel_width_surface: c.width,
        water_density: 1.0,
        is_deep_water: c.depth <= 0.0,
        ..EnvironmentInputs::default()
    };
    let inputs = ResistanceInputs {
        lwl: c.lwl,
        loa: c.lwl,
        beam: c.beam,
        draft: c.draught,
        block_coefficient: cb(c),
        ..ResistanceInputs::default()
    };
    ResistanceCalculator::new(&inputs)
        .shallow_water_factor(c.speed, &env)
        .unwrap()
}

fn ct_barge_fit(c: &Case) -> f64 {
    cf_ittc(c) * (1.0 + residuary_ratio(froude(c))) * shallow_factor(c)
}

fn holtrop(c: &Case) -> holtrop_mennen::Components {
    let cbv = cb(c);
    let mut h = HullInputs::plain(
        c.lwl,
        c.beam,
        c.draught,
        c.volume,
        1.0,
        holtrop_mennen::cwp_estimate(cbv),
    );
    h.wetted_surface_m2 = c.wetted;
    let flow = Flow {
        speed_ms: c.speed,
        rho: 1000.0,
        nu: c.nu,
        g: G,
    };
    holtrop_mennen::components(&h, flow, Variant::Hm1984)
}

fn ct_holtrop(c: &Case) -> f64 {
    let k = holtrop(c);
    let q = 0.5 * 1000.0 * c.speed * c.speed * c.wetted / 1000.0;
    (k.total_kn - k.r_a_kn) / q * shallow_factor(c)
}

/// `barge_form_factor` (issue #735), through the same function the core uses.
fn ct_barge_form_factor(c: &Case) -> f64 {
    let k1 = holtrop(c).form_factor;
    cf_ittc(c) * barge_form_factor_ratio(k1, froude(c), c.draught, c.depth) * shallow_factor(c)
}

fn group(c: &Case) -> &'static str {
    match (c.id.starts_with("z23-1x1"), c.depth > 0.0) {
        (true, false) => "deep, pusher + 1 barge",
        (false, false) => "deep, pusher + 2 or 4 barges",
        (true, true) => "shallow, pusher + 1 barge",
        (false, true) => "shallow, other",
    }
}

/// Signed percentage errors of (barge fit, Holtrop-Mennen, barge form factor) for one case.
fn errors(c: &Case) -> [f64; 3] {
    [ct_barge_fit(c), ct_holtrop(c), ct_barge_form_factor(c)].map(|p| (p / c.ct - 1.0) * 100.0)
}

fn mean_abs(cs: &[Case], m: usize) -> f64 {
    cs.iter().map(|c| errors(c)[m].abs()).sum::<f64>() / cs.len() as f64
}

#[test]
fn print_error_table() {
    println!(
        "| case | Fn | Cb | L/B | B/T | h/T | CT x1000 | barge_fit | holtrop_mennen | barge_form_factor | 1+k1 |"
    );
    println!("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|");
    for c in cases() {
        let e = errors(&c);
        println!(
            "| {} | {:.3} | {:.3} | {:.2} | {:.2} | {} | {:.3} | {:+.1} % | {:+.1} % | {:+.1} % | {:.3} |",
            c.id,
            froude(&c),
            cb(&c),
            c.lwl / c.beam,
            c.beam / c.draught,
            if c.depth > 0.0 {
                format!("{:.2}", c.depth / c.draught)
            } else {
                "deep".into()
            },
            c.ct * 1e3,
            e[0],
            e[1],
            e[2],
            holtrop(&c).form_factor,
        );
    }
    println!("\n| group | cases | barge_fit | holtrop_mennen | barge_form_factor |");
    println!("|---|---:|---|---|---|");
    let all = cases();
    for g in [
        "deep, pusher + 1 barge",
        "deep, pusher + 2 or 4 barges",
        "shallow, pusher + 1 barge",
        "shallow, other",
    ] {
        let cs: Vec<Case> = all.iter().filter(|c| group(c) == g).cloned().collect();
        let cell = |m: usize| {
            let e: Vec<f64> = cs.iter().map(|c| errors(c)[m]).collect();
            let lo = e.iter().cloned().fold(f64::INFINITY, f64::min);
            let hi = e.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            format!(
                "{:+.0} to {:+.0} % (mean abs {:.0} %)",
                lo,
                hi,
                mean_abs(&cs, m)
            )
        };
        println!(
            "| {} | {} | {} | {} | {} |",
            g,
            cs.len(),
            cell(0),
            cell(1),
            cell(2)
        );
    }
    println!(
        "| all | {} | mean abs {:.1} % | mean abs {:.1} % | mean abs {:.1} % |",
        all.len(),
        mean_abs(&all, 0),
        mean_abs(&all, 1),
        mean_abs(&all, 2)
    );
}

/// Label: property. Every case lies inside the declared range of `barge_form_factor`: the range is
/// the range of the data.
#[test]
fn every_case_is_inside_the_barge_form_factor_envelope() {
    for c in cases() {
        let out = BARGE_FORM_FACTOR_ENVELOPE.outside(
            froude(&c),
            cb(&c),
            c.lwl / c.beam,
            c.beam / c.draught,
        );
        assert!(out.is_empty(), "{} outside: {:?}", c.id, out);
        let h_over_t = (c.depth > 0.0).then(|| c.depth / c.draught + 1e-9);
        assert_eq!(
            BARGE_FORM_FACTOR_ENVELOPE.outside_depth(h_over_t),
            None,
            "{}",
            c.id
        );
    }
}

/// Label: property. On the published barge data the barge form factor has the smallest mean
/// absolute error of the three methods, over all cases and in shallow water; the bluff-barge fit
/// under-predicts every case (it is optimistic).
#[test]
fn the_barge_form_factor_is_closest_to_the_barge_tests() {
    let all = cases();
    let shallow: Vec<Case> = all.iter().filter(|c| c.depth > 0.0).cloned().collect();
    for set in [&all, &shallow] {
        let (fit, hm, bff) = (mean_abs(set, 0), mean_abs(set, 1), mean_abs(set, 2));
        assert!(
            bff < fit && bff < hm,
            "fit {fit:.1} HM {hm:.1} form factor {bff:.1}"
        );
    }
    for c in &all {
        assert!(
            errors(c)[0] < 0.0,
            "{}: the barge fit is above the test",
            c.id
        );
    }
}
