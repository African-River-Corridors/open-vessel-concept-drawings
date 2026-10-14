//! Intact stability of a gas barge: GM, the GZ curve and the code criteria. Port of
//! `core/gas_stability.py`, copied exactly (no fixes).
//!
//! Python behaviour kept on purpose:
//! - `sum()` is the compensated sum of Python 3.12 (`py_sum`).
//! - `max` and `min` keep the first of equal items (`py_max`, `py_min`, strict comparisons). They
//!   are not `f64::max`/`f64::min`, which treat NaN differently.
//! - The two bisections run all 80 steps, as in Python, so the result does not depend on a stop.
//! - `math.radians` and `math.degrees` multiply by a constant; `**` is `libm::pow`.
//! - A float division by zero is Python's `ZeroDivisionError`, and `sin`/`cos` of an infinity is
//!   its `ValueError("math domain error")` (`PyError`).
//! - `Condition` fields that Python fills with the int `-1` or `0` when the barge does not float
//!   are `f64` here; `gz.is_empty()` marks that case, for the caller that writes them as ints.
//!
//! The Python `assess` takes a whole `BargeCapacityResult`. The Rust one takes only the numbers it
//! reads (`GasCapacityResult`), because `barge_capacity` is ported in its own ticket.
//!
//! - A float `**` that overflows is Python's `OverflowError(34, 'Numerical result out of range')` (`py_pow`).

use crate::pyfmt;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Heel angles of the GZ curve, in degrees: `range(0, 61)`.
pub const ANGLES_DEG: core::ops::RangeInclusive<i32> = 0..=60;

/// What Python would raise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PyError {
    /// `ValueError(message)`.
    Value(String),
    /// `ZeroDivisionError("float division by zero")`.
    ZeroDivision,
    /// `OverflowError(34, 'Numerical result out of range')`, from a float `**` that leaves the finite range.
    Overflow,
}

impl PyError {
    /// The Python exception class name.
    pub fn type_name(&self) -> &'static str {
        match self {
            PyError::Value(_) => "ValueError",
            PyError::ZeroDivision => "ZeroDivisionError",
            PyError::Overflow => "OverflowError",
        }
    }

    /// The Python exception message.
    pub fn message(&self) -> &str {
        match self {
            PyError::Value(m) => m,
            PyError::ZeroDivision => "float division by zero",
            PyError::Overflow => "(34, 'Numerical result out of range')",
        }
    }
}

// --- Python semantics ------------------------------------------------------------------------

/// Python 3.12 `sum()` of floats: Neumaier compensated summation, starting from 0.
fn py_sum(items: impl IntoIterator<Item = f64>) -> f64 {
    let mut sum = 0.0_f64;
    let mut c = 0.0_f64;
    for x in items {
        let t = sum + x;
        if sum.abs() >= x.abs() {
            c += (sum - t) + x;
        } else {
            c += (x - t) + sum;
        }
        sum = t;
    }
    if c != 0.0 && c.is_finite() {
        sum += c;
    }
    sum
}

/// Python 3.12 `sum()` of a list of floats that ends with items that may be ints. An int item
/// in the float loop is added to the running sum with no compensation (`f_result += (double)value`).
/// Every int here comes after a float, so the loop is already in float mode.
fn py_sum_mixed(items: impl IntoIterator<Item = (f64, bool)>) -> f64 {
    let mut sum = 0.0_f64;
    let mut c = 0.0_f64;
    for (x, is_int) in items {
        if is_int {
            sum += x;
            continue;
        }
        let t = sum + x;
        if sum.abs() >= x.abs() {
            c += (sum - t) + x;
        } else {
            c += (x - t) + sum;
        }
        sum = t;
    }
    if c != 0.0 && c.is_finite() {
        sum += c;
    }
    sum
}

/// Python `max(a, b)`: the first of equal items; NaN never replaces the first item.
fn py_max2(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Python `min(a, b)`.
fn py_min2(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Python `max(iterable)` of floats, first of equal items. The iterable must not be empty.
fn py_max(items: impl IntoIterator<Item = f64>) -> f64 {
    let mut it = items.into_iter();
    let mut best = it.next().expect("max() of an empty sequence");
    for x in it {
        if x > best {
            best = x;
        }
    }
    best
}

/// Python `min(iterable)` of floats, first of equal items. The iterable must not be empty.
fn py_min(items: impl IntoIterator<Item = f64>) -> f64 {
    let mut it = items.into_iter();
    let mut best = it.next().expect("min() of an empty sequence");
    for x in it {
        if x < best {
            best = x;
        }
    }
    best
}

/// Python float division: a zero divisor raises `ZeroDivisionError`.
fn div(a: f64, b: f64) -> Result<f64, PyError> {
    if b == 0.0 {
        Err(PyError::ZeroDivision)
    } else {
        Ok(a / b)
    }
}

/// `math.radians`.
fn radians(x: f64) -> f64 {
    x * (core::f64::consts::PI / 180.0)
}

/// `math.degrees`.
fn degrees(x: f64) -> f64 {
    x * (180.0 / core::f64::consts::PI)
}

/// `math.sin` / `math.cos`: an infinity raises `ValueError`.
fn sin(x: f64) -> Result<f64, PyError> {
    domain_checked(x).map(libm::sin)
}

fn cos(x: f64) -> Result<f64, PyError> {
    domain_checked(x).map(libm::cos)
}

fn domain_checked(x: f64) -> Result<f64, PyError> {
    if x.is_infinite() {
        Err(PyError::Value("math domain error".to_string()))
    } else {
        Ok(x)
    }
}

/// Float `x ** y`: a finite base with a finite exponent that gives an infinity is Python's
/// `OverflowError`.
fn py_pow(x: f64, y: f64) -> Result<f64, PyError> {
    let r = libm::pow(x, y);
    if r.is_infinite() && x.is_finite() && y.is_finite() {
        Err(PyError::Overflow)
    } else {
        Ok(r)
    }
}

/// `x ** 2`.
fn sq(x: f64) -> Result<f64, PyError> {
    py_pow(x, 2.0)
}

// --- geometry helpers ------------------------------------------------------------------------

type Pt = (f64, f64);

/// Sutherland-Hodgman: keep the part of polygon `poly` (list of (Y, Z)) with Z <= h.
fn clip_below(poly: &[Pt], h: f64) -> Vec<Pt> {
    let mut out = Vec::new();
    let n = poly.len();
    for i in 0..n {
        let (p, q) = (poly[i], poly[(i + 1) % n]);
        let (pin, qin) = (p.1 <= h, q.1 <= h);
        if pin {
            out.push(p);
        }
        if pin != qin {
            let t = (h - p.1) / (q.1 - p.1);
            out.push((p.0 + t * (q.0 - p.0), h));
        }
    }
    out
}

/// (area, centroid Y, centroid Z) of a polygon.
fn area_centroid(poly: &[Pt]) -> (f64, f64, f64) {
    let (mut a, mut cy, mut cz) = (0.0_f64, 0.0_f64, 0.0_f64);
    let n = poly.len();
    for i in 0..n {
        let ((y0, z0), (y1, z1)) = (poly[i], poly[(i + 1) % n]);
        let c = y0 * z1 - y1 * z0;
        a += c;
        cy += (y0 + y1) * c;
        cz += (z0 + z1) * c;
    }
    a *= 0.5;
    if a.abs() < 1e-12 {
        return (0.0, 0.0, 0.0);
    }
    (a.abs(), cy / (6.0 * a), cz / (6.0 * a))
}

/// Righting lever (m) of a box section heeled `phi_deg`, at constant displaced area B x T.
pub fn box_gz(beam: f64, depth: f64, draught: f64, kg: f64, phi_deg: f64) -> Result<f64, PyError> {
    if phi_deg == 0.0 {
        return Ok(0.0);
    }
    let phi = radians(phi_deg);
    let (c, s) = (cos(phi)?, sin(phi)?);
    let hull = [
        (-beam / 2.0, 0.0),
        (beam / 2.0, 0.0),
        (beam / 2.0, depth),
        (-beam / 2.0, depth),
    ];
    // Earth frame; +phi lifts starboard.
    let rot: Vec<Pt> = hull
        .iter()
        .map(|&(y, z)| (y * c - z * s, y * s + z * c))
        .collect();
    let target = beam * draught;
    let zs: Vec<f64> = rot.iter().map(|p| p.1).collect();
    let (mut lo, mut hi) = (py_min(zs.iter().copied()), py_max(zs.iter().copied()));
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        let (a, _, _) = area_centroid(&clip_below(&rot, mid));
        if a < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let (_, yb, _) = area_centroid(&clip_below(&rot, 0.5 * (lo + hi)));
    let yg = -kg * s; // G in the earth frame
    Ok(yg - yb) // positive = righting
}

/// Liquid height h (from tank bottom) and centroid height for a horizontal cylinder filled to
/// `volume_fraction` of its cross-section area.
pub fn segment_fill(radius: f64, volume_fraction: f64) -> Result<(f64, f64), PyError> {
    let f = py_min2(py_max2(volume_fraction, 0.0), 1.0);
    if f <= 0.0 {
        return Ok((0.0, 0.0));
    }
    if f >= 1.0 {
        return Ok((2.0 * radius, radius));
    }
    let pi = core::f64::consts::PI;
    let (mut lo, mut hi) = (0.0_f64, pi);
    // theta = central angle of the segment: A = r^2 (t - sin t) / 2
    for _ in 0..80 {
        let t = 0.5 * (lo + hi);
        if (t - libm::sin(t)) / (2.0 * pi) < f {
            lo = t;
        } else {
            hi = t;
        }
    }
    let t = 0.5 * (lo + hi);
    let h = radius * (1.0 - libm::cos(t / 2.0));
    let area = sq(radius)? * (t - libm::sin(t)) / 2.0;
    let ybar_from_centre = div(
        (4.0 * radius) * libm::pow(libm::sin(t / 2.0), 3.0),
        3.0 * (t - libm::sin(t)),
    )?;
    Ok((
        h,
        if area > 0.0 {
            radius - ybar_from_centre
        } else {
            0.0
        },
    ))
}

// --- results ---------------------------------------------------------------------------------

/// A number that Python keeps as an int or a float. `Criterion.value` holds the heel angle of
/// maximum GZ, an int, for one criterion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PyNum {
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Criterion {
    pub name: String,
    pub value: PyNum,
    pub limit: f64,
    pub ok: bool,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub name: String,
    pub cargo_t: f64,
    /// Python holds `cargo_t` as an int: the Laden condition of an int `payload_t`.
    pub cargo_is_int: bool,
    pub fill_fraction: f64,
    pub displacement_t: f64,
    pub draught_m: f64,
    pub kg_m: f64,
    pub gm_solid_m: f64,
    pub free_surface_m: f64,
    pub gm_fluid_m: f64,
    /// `(deg, GZ_fluid m)`. Empty when the barge does not float.
    pub gz: Vec<(i32, f64)>,
    pub max_gz_m: f64,
    pub max_gz_deg: i32,
    pub area_0_27: f64,
    pub area_0_30: f64,
    pub area_0_40: f64,
    pub area_30_40: f64,
    pub deck_edge_deg: Option<f64>,
    pub criteria: Vec<Criterion>,
}

impl Condition {
    pub fn ok(&self) -> bool {
        self.criteria.iter().all(|c| c.ok)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StabilityAssessment {
    pub criteria_set: String,
    pub conditions: Vec<Condition>,
    pub limitations: Vec<String>,
}

impl StabilityAssessment {
    pub fn ok(&self) -> bool {
        self.conditions.iter().all(Condition::ok)
    }

    /// The condition with the lowest fluid GM; the first of equal.
    pub fn governing(&self) -> &Condition {
        let mut best = &self.conditions[0];
        for c in &self.conditions[1..] {
            if c.gm_fluid_m < best.gm_fluid_m {
                best = c;
            }
        }
        best
    }
}

/// Trapezoid area under the GZ curve between `a0` and `a1` degrees.
fn area(gz: &[(i32, f64)], a0: i32, a1: i32) -> f64 {
    let pts: Vec<(i32, f64)> = gz
        .iter()
        .copied()
        .filter(|&(d, _)| a0 <= d && d <= a1)
        .collect();
    py_sum(pts.windows(2).map(|w| {
        let ((d0, g0), (d1, g1)) = (w[0], w[1]);
        0.5 * (g0 + g1) * radians(f64::from(d1 - d0))
    }))
}

fn criterion(name: &str, value: f64, limit: f64, ok: bool, source: &str) -> Criterion {
    Criterion {
        name: name.to_string(),
        value: PyNum::Float(value),
        limit,
        ok,
        source: source.to_string(),
    }
}

fn criteria(cset: &str, c: &Condition, wide_tank: bool) -> Result<Vec<Criterion>, PyError> {
    if cset == "IS_CODE_2008" {
        let src = "IMO IS Code 2008, Part A 2.2";
        let gz_30_plus = py_max(c.gz.iter().filter(|p| p.0 >= 30).map(|p| p.1));
        let s = |suffix: &str| format!("{src}{suffix}");
        return Ok(vec![
            criterion(
                "Area 0\u{2013}30\u{b0}",
                c.area_0_30,
                0.055,
                c.area_0_30 >= 0.055,
                &s(".1"),
            ),
            criterion(
                "Area 0\u{2013}40\u{b0}",
                c.area_0_40,
                0.090,
                c.area_0_40 >= 0.090,
                &s(".1"),
            ),
            criterion(
                "Area 30\u{2013}40\u{b0}",
                c.area_30_40,
                0.030,
                c.area_30_40 >= 0.030,
                &s(".1"),
            ),
            criterion(
                "GZ at \u{2265} 30\u{b0}",
                gz_30_plus,
                0.20,
                gz_30_plus >= 0.20,
                &s(".2"),
            ),
            Criterion {
                name: "Angle of max GZ".to_string(),
                value: PyNum::Int(i64::from(c.max_gz_deg)),
                limit: 25.0,
                ok: f64::from(c.max_gz_deg) >= 25.0,
                source: s(".3"),
            },
            criterion(
                "Initial GM (fluid)",
                c.gm_fluid_m,
                0.15,
                c.gm_fluid_m >= 0.15,
                &s(".4"),
            ),
        ]);
    }
    if cset == "ADN_9_3_1_14_2" {
        let src = "ADN 2025 9.3.1.14.2 (tank > 0.70 B)";
        if !wide_tank {
            return Ok(vec![criterion(
                "Initial GM (fluid) > 0",
                c.gm_fluid_m,
                0.0,
                c.gm_fluid_m > 0.0,
                "ADN 9.3.1.14.1 \u{2014} no tank wider than 0.70 B, so 9.3.1.14.2 does not apply",
            )]);
        }
        let gz_max_27 = py_max(c.gz.iter().filter(|p| p.0 <= 27).map(|p| p.1));
        return Ok(vec![
            criterion(
                "Initial GM (fluid)",
                c.gm_fluid_m,
                0.10,
                c.gm_fluid_m >= 0.10,
                src,
            ),
            criterion(
                "GZ within 27\u{b0}",
                gz_max_27,
                0.10,
                gz_max_27 >= 0.10,
                src,
            ),
            criterion(
                "Area 0\u{2013}27\u{b0}",
                c.area_0_27,
                0.024,
                c.area_0_27 >= 0.024,
                src,
            ),
        ]);
    }
    Err(PyError::Value(format!(
        "unknown criteria set {}",
        crate::pystr::repr_str(cset)
    )))
}

/// `max(gz, key=lambda p: p[1])`: the first of equal GZ values. `gz` must not be empty.
fn max_by_gz(gz: &[(i32, f64)]) -> (i32, f64) {
    let (mut best_deg, mut best) = gz[0];
    for &(deg, g) in &gz[1..] {
        if g > best {
            best_deg = deg;
            best = g;
        }
    }
    (best_deg, best)
}

/// The criteria set a ruleset uses.
pub fn criteria_set_for(ruleset: &str) -> &'static str {
    if ruleset == "ADN_G" {
        "ADN_9_3_1_14_2"
    } else {
        "IS_CODE_2008"
    }
}

/// What `assess` reads from a converged gas arrangement (`BargeCapacityResult.gas_basis`).
#[derive(Debug, Clone, PartialEq)]
pub struct GasBasis {
    pub loa: f64,
    pub beam: f64,
    pub depth: f64,
    pub double_bottom_height: f64,
    pub tank_support_height: f64,
    pub outer_diameter: f64,
    pub inner_diameter: f64,
    pub total_tank_volume: f64,
    pub n_tanks_total: u32,
    pub cylinder_length: f64,
    pub steel_top_down: f64,
    pub tank_total: f64,
    pub insulation: f64,
    pub piping_machinery: f64,
    pub outfitting: f64,
}

/// What `assess` reads from a `BargeCapacityResult`.
#[derive(Debug, Clone, PartialEq)]
pub struct GasCapacityResult {
    /// `None` for a bulk result: `assess` then refuses.
    pub gas_basis: Option<GasBasis>,
    pub cargo_density_t_per_m3: f64,
    pub lightship_t: f64,
    /// `aft_arrangement is not None`.
    pub has_aft_arrangement: bool,
}

/// Python's defaults: block coefficient 0.85, water density 1.0, criteria set `IS_CODE_2008`.
pub const DEFAULT_BLOCK_COEFFICIENT: f64 = 0.85;
pub const DEFAULT_WATER_DENSITY: f64 = 1.0;
pub const DEFAULT_CRITERIA_SET: &str = "IS_CODE_2008";

/// Assess a converged gas arrangement.
pub fn assess(
    result: &GasCapacityResult,
    payload_t: f64,
    block_coefficient: f64,
    water_density: f64,
    criteria_set: &str,
) -> Result<StabilityAssessment, PyError> {
    assess_num(
        result,
        PyNum::Float(payload_t),
        block_coefficient,
        water_density,
        criteria_set,
    )
}

/// `assess` with a `payload_t` that Python may hold as an int. An int payload stays an int in the
/// Laden condition and is summed without compensation.
pub fn assess_num(
    result: &GasCapacityResult,
    payload: PyNum,
    block_coefficient: f64,
    water_density: f64,
    criteria_set: &str,
) -> Result<StabilityAssessment, PyError> {
    let (payload_t, payload_is_int) = match payload {
        PyNum::Int(i) => (i as f64, true),
        PyNum::Float(x) => (x, false),
    };
    let basis = result.gas_basis.as_ref().ok_or_else(|| {
        PyError::Value("assess() needs a gas BargeCapacityResult (gas_basis is None)".to_string())
    })?;
    let (l, b, d) = (basis.loa, basis.beam, basis.depth);
    let rho_c = result.cargo_density_t_per_m3;
    let od = basis.outer_diameter;
    let r_in = basis.inner_diameter / 2.0;
    let tank_bottom = basis.double_bottom_height + basis.tank_support_height; // outer face, above keel
    let kg_tank = tank_bottom + od / 2.0;
    let inner_bottom = tank_bottom + (od / 2.0 - r_in);
    let v_inner_total = basis.total_tank_volume;
    let wide_tank = od > 0.70 * b;

    // (t, KG m)
    let light_items: [(f64, f64); 4] = [
        (basis.steel_top_down, 0.5 * d),
        (basis.tank_total + basis.insulation, kg_tank),
        (basis.piping_machinery, d),
        (basis.outfitting, 0.75 * d),
    ];
    let lightship = py_sum(light_items.iter().map(|&(m, _)| m));

    let mut conds = Vec::new();
    for (name, cargo, cargo_is_int) in [
        ("Laden", payload_t, payload_is_int),
        ("Half laden", 0.5 * payload_t, false),
        ("Light", 0.0, false),
    ] {
        let frac = if v_inner_total > 0.0 {
            div(cargo, rho_c * v_inner_total)?
        } else {
            0.0
        };
        let (h, ybar) = segment_fill(r_in, frac)?;
        let kg_cargo = inner_bottom + ybar;
        let mut items: Vec<(f64, f64, bool)> =
            light_items.iter().map(|&(m, z)| (m, z, false)).collect();
        if cargo > 0.0 {
            items.push((cargo, kg_cargo, cargo_is_int));
        }
        let disp = py_sum_mixed(items.iter().map(|&(m, _, i)| (m, i)));
        // An int mass times a float height is a float, so the products are all floats.
        let kg = div(py_sum(items.iter().map(|&(m, z, _)| m * z)), disp)?;
        let t = div(disp, block_coefficient * l * b * water_density)?;
        if t >= d {
            conds.push(Condition {
                name: name.to_string(),
                cargo_t: cargo,
                cargo_is_int,
                fill_fraction: frac,
                displacement_t: disp,
                draught_m: t,
                kg_m: kg,
                gm_solid_m: -1.0,
                free_surface_m: 0.0,
                gm_fluid_m: -1.0,
                gz: Vec::new(),
                max_gz_m: 0.0,
                max_gz_deg: 0,
                area_0_27: 0.0,
                area_0_30: 0.0,
                area_0_40: 0.0,
                area_30_40: 0.0,
                deck_edge_deg: None,
                criteria: vec![criterion(
                    "Floats with freeboard",
                    t,
                    d,
                    false,
                    "draught \u{2265} depth",
                )],
            });
            continue;
        }
        let (kb, bm) = (t / 2.0, div(b * b, 12.0 * t)?);
        let gm_solid = kb + bm - kg;
        // Free surface: chord at the liquid level, along the cylindrical length, every tank.
        let i_fs = if 0.0 < frac && frac < 1.0 {
            let chord = 2.0 * libm::sqrt(py_max2(sq(r_in)? - sq(r_in - h)?, 0.0));
            f64::from(basis.n_tanks_total) * basis.cylinder_length * py_pow(chord, 3.0)? / 12.0
        } else {
            0.0
        };
        let fsc = div(rho_c * i_fs, disp)?;
        let mut gz: Vec<(i32, f64)> = Vec::with_capacity(61);
        for a in ANGLES_DEG {
            let heel = box_gz(b, d, t, kg, f64::from(a))?;
            gz.push((a, heel - fsc * sin(radians(f64::from(a)))?));
        }
        let (max_gz_deg, max_gz) = max_by_gz(&gz);
        let deck_edge = degrees(libm::atan(div(2.0 * (d - t), b)?));
        let mut c = Condition {
            name: name.to_string(),
            cargo_t: cargo,
            cargo_is_int,
            fill_fraction: frac,
            displacement_t: disp,
            draught_m: t,
            kg_m: kg,
            gm_solid_m: gm_solid,
            free_surface_m: fsc,
            gm_fluid_m: gm_solid - fsc,
            max_gz_m: max_gz,
            max_gz_deg,
            area_0_27: area(&gz, 0, 27),
            area_0_30: area(&gz, 0, 30),
            area_0_40: area(&gz, 0, 40),
            area_30_40: area(&gz, 30, 40),
            deck_edge_deg: Some(deck_edge),
            gz,
            criteria: Vec::new(),
        };
        c.criteria = criteria(criteria_set, &c, wide_tank)?;
        conds.push(c);
    }

    let mut limitations = vec![
        "Concept-stage: prismatic box section, first-order weight heights, no downflooding openings."
            .to_string(),
        "No wind heel and no damage stability (ADN 9.3.1.15 / IGC 2.7 need both before approval)."
            .to_string(),
    ];
    if result.has_aft_arrangement {
        limitations.push(
            "The self-propelled stern block's mass is not in the gas lightship, so KG and \
             displacement leave it out; the true GM could be lower."
                .to_string(),
        );
    }
    if (lightship - result.lightship_t).abs() > 1.0 {
        limitations.push(format!(
            "Weight split sums to {} t vs lightship {} t.",
            pyfmt::fixed(lightship, 0),
            pyfmt::fixed(result.lightship_t, 0)
        ));
    }
    Ok(StabilityAssessment {
        criteria_set: criteria_set.to_string(),
        conditions: conds,
        limitations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // These sums need Neumaier compensation. A plain left-to-right sum gives 0.9999999999999999
    // and 0.0. Values are what Python 3.12 `sum()` prints.
    #[test]
    fn py_sum_is_compensated() {
        assert_eq!(py_sum([0.1_f64; 10]), 1.0);
        assert_eq!(py_sum([1e16, 1.0, -1e16]), 1.0);
        assert_eq!(py_sum([1.0, 1e16, -1e16]), 1.0);
        assert_eq!(py_sum([1e100, 1.0, -1e100, 1.0]), 2.0);
        assert_eq!(py_sum(core::iter::empty()), 0.0);
        // An infinite compensation is dropped, as in Python.
        assert_eq!(py_sum([f64::INFINITY, 1.0]), f64::INFINITY);
        assert_eq!(py_sum([1.5, 2.5]), 4.0);
    }

    // Python 3.12 `sum()` (the version the frozen Python runs on) adds an int item to the running
    // total with no compensation, so a trailing int gives a different last bit. Values are what
    // Python 3.12 prints. (3.14 compensates ints too; the port follows 3.12.)
    #[test]
    fn py_sum_mixed_adds_an_int_without_compensation() {
        let f = |x: f64| (x, false);
        let cases: [([f64; 4], f64, f64); 3] = [
            (
                [895.575, 973.252, 500.8, 967.21],
                4160.0,
                7496.8369999999995,
            ),
            ([86.468, 663.758, 107.931, 163.698], 3222.0, 4243.855),
            (
                [343.826, 847.461, 353.274, 909.755],
                4490.0,
                6944.316000000001,
            ),
        ];
        for (floats, int, want) in cases {
            let mut items: Vec<(f64, bool)> = floats.iter().map(|&x| f(x)).collect();
            items.push((int, true));
            assert_eq!(py_sum_mixed(items), want);
            // The same numbers all compensated give a different last bit.
            let all: Vec<f64> = floats.iter().copied().chain([int]).collect();
            assert_ne!(py_sum(all), want);
        }
        // An all-float list is the compensated sum.
        assert_eq!(py_sum_mixed([(0.1, false); 10]), 1.0);
    }

    // Pins the compensation inside `py_sum_mixed`: the branch where the new float is larger than
    // the running sum, and the rule that an infinite or NaN compensation is dropped. Values are
    // what Python 3.12 `sum()` prints for the same lists (floats first, then an optional int).
    #[test]
    fn py_sum_mixed_compensates_floats_like_py_sum() {
        let fl = |v: &[f64]| -> Vec<(f64, bool)> { v.iter().map(|&x| (x, false)).collect() };
        // Larger new float than running sum: needs `c += (x - t) + sum`.
        assert_eq!(py_sum_mixed(fl(&[1.0, 1e16, -1e16])), 1.0);
        assert_eq!(py_sum_mixed(fl(&[1e16, 1.0, -1e16])), 1.0);
        assert_eq!(py_sum_mixed(fl(&[1e100, 1.0, -1e100, 1.0])), 2.0);
        // The same lists with a trailing int: the int adds no compensation, the floats still do.
        let mut v = fl(&[1.0, 1e16, -1e16]);
        v.push((2.0, true));
        assert_eq!(py_sum_mixed(v), 3.0);
        // A non-finite compensation is dropped, so the sum stays infinite, as in Python.
        assert_eq!(py_sum_mixed(fl(&[f64::INFINITY, 1.0])), f64::INFINITY);
        assert!(py_sum_mixed(fl(&[f64::NAN, 1.0])).is_nan());
        assert_eq!(py_sum_mixed(core::iter::empty()), 0.0);
    }

    // Python `max(a, b)` and `min(a, b)` keep the first of equal items, so the sign of a zero
    // shows which one was kept: `max(-0.0, 0.0)` is `-0.0`, `min(0.0, -0.0)` is `0.0`.
    #[test]
    fn py_max2_and_py_min2_keep_the_first_of_equal_items() {
        assert_eq!(py_max2(1.0, 2.0), 2.0);
        assert_eq!(py_max2(2.0, 1.0), 2.0);
        assert!(py_max2(-0.0, 0.0).is_sign_negative());
        assert!(py_max2(0.0, -0.0).is_sign_positive());
        assert_eq!(py_min2(1.0, 2.0), 1.0);
        assert_eq!(py_min2(2.0, 1.0), 1.0);
        assert!(py_min2(0.0, -0.0).is_sign_positive());
        assert!(py_min2(-0.0, 0.0).is_sign_negative());
        // NaN never replaces the first item.
        assert_eq!(py_max2(1.0, f64::NAN), 1.0);
        assert_eq!(py_min2(1.0, f64::NAN), 1.0);
    }

    #[test]
    fn py_sum_fixed_point_rounding() {
        // sum of 0.1 .. 0.9 step 0.1: plain sum is 4.5 - ulp; Python gives 4.5.
        let v: Vec<f64> = (1..=9).map(|i| f64::from(i) / 10.0).collect();
        assert_eq!(py_sum(v), 4.5);
    }

    #[test]
    fn max_by_gz_keeps_the_first_of_equal_values() {
        assert_eq!(
            max_by_gz(&[(0, 0.0), (1, 0.5), (2, 0.5), (3, 0.25)]),
            (1, 0.5)
        );
        assert_eq!(max_by_gz(&[(0, 0.0), (1, 0.0)]), (0, 0.0));
        assert_eq!(max_by_gz(&[(0, 0.0), (1, -0.0)]), (0, 0.0));
        assert_eq!(max_by_gz(&[(0, 0.0), (1, 0.5), (2, 0.6)]), (2, 0.6));
        // NaN never replaces the first item.
        assert_eq!(max_by_gz(&[(0, 0.0), (1, f64::NAN)]), (0, 0.0));
    }

    #[test]
    fn py_max_and_py_min_of_a_list_keep_the_first_of_equal_items() {
        assert_eq!(py_max([3.0, 1.0, 2.0]), 3.0);
        assert_eq!(py_max([1.0, 3.0, 2.0]), 3.0);
        assert_eq!(py_min([3.0, 1.0, 2.0]), 1.0);
        assert_eq!(py_min([1.0, 3.0, 0.5]), 0.5);
        assert_eq!(py_min([-2.0, 4.0]), -2.0);
        assert!(py_max([0.0, -0.0]).is_sign_positive());
        assert!(py_max([-0.0, 0.0]).is_sign_negative());
        assert!(py_min([0.0, -0.0]).is_sign_positive());
        assert!(py_min([-0.0, 0.0]).is_sign_negative());
    }

    #[test]
    fn py_pow_overflow() {
        assert_eq!(sq(1e155), Err(PyError::Overflow));
        assert_eq!(sq(f64::INFINITY), Ok(f64::INFINITY));
        assert_eq!(py_pow(3.0, 3.0), Ok(27.0));
    }
}
