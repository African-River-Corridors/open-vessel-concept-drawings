//! Intact stability of a box section: the GZ curve, the areas under it and the IS Code 2008 Part A
//! criteria. Shared by every Cargo Class: moved out of `gas_stability` with no change (spec 0006
//! §6), so the liquid bulk path can call it. The Python semantics notes of `gas_stability` apply.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Heel angles of the GZ curve, in degrees: `range(0, 61)`.
pub const ANGLES_DEG: core::ops::RangeInclusive<i32> = 0..=60;

/// The two heel angles at which the IS Code criteria cut the GZ curve: the areas run to 30 and 40
/// degrees, and the GZ criterion looks at 30 degrees and above. A view marks them from here.
pub const CRITERIA_ANGLES_DEG: [i32; 2] = [30, 40];

/// What Python would raise.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PyError {
    /// `ValueError(message)`.
    Value(String),
    /// `ZeroDivisionError("float division by zero")`.
    ZeroDivision,
    /// `OverflowError(34, 'Numerical result out of range')`, from a float `**` that leaves the finite range.
    Overflow,
    /// `OverflowError("int too large to convert to float")`, from an int that Python must turn
    /// into a float and cannot.
    IntToFloat,
}

impl PyError {
    /// The Python exception class name.
    pub fn type_name(&self) -> &'static str {
        match self {
            PyError::Value(_) => "ValueError",
            PyError::ZeroDivision => "ZeroDivisionError",
            PyError::Overflow | PyError::IntToFloat => "OverflowError",
        }
    }

    /// The Python exception message.
    pub fn message(&self) -> &str {
        match self {
            PyError::Value(m) => m,
            PyError::ZeroDivision => "float division by zero",
            PyError::Overflow => "(34, 'Numerical result out of range')",
            PyError::IntToFloat => "int too large to convert to float",
        }
    }
}

// --- Python semantics ------------------------------------------------------------------------

/// Python 3.12 `sum()` of floats: Neumaier compensated summation, starting from 0.
pub(crate) fn py_sum(items: impl IntoIterator<Item = f64>) -> f64 {
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
pub(crate) fn py_sum_mixed(items: impl IntoIterator<Item = (f64, bool)>) -> f64 {
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
pub(crate) fn py_max2(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

/// Python `min(a, b)`.
pub(crate) fn py_min2(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Python `max(iterable)` of floats, first of equal items. The iterable must not be empty.
pub(crate) fn py_max(items: impl IntoIterator<Item = f64>) -> f64 {
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
pub(crate) fn py_min(items: impl IntoIterator<Item = f64>) -> f64 {
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
pub(crate) fn div(a: f64, b: f64) -> Result<f64, PyError> {
    if b == 0.0 {
        Err(PyError::ZeroDivision)
    } else {
        Ok(a / b)
    }
}

/// `math.radians`.
pub(crate) fn radians(x: f64) -> f64 {
    x * (core::f64::consts::PI / 180.0)
}

/// `math.degrees`.
pub(crate) fn degrees(x: f64) -> f64 {
    x * (180.0 / core::f64::consts::PI)
}

/// `math.sin` / `math.cos`: an infinity raises `ValueError`.
pub(crate) fn sin(x: f64) -> Result<f64, PyError> {
    domain_checked(x).map(libm::sin)
}

pub(crate) fn cos(x: f64) -> Result<f64, PyError> {
    domain_checked(x).map(libm::cos)
}

pub(crate) fn domain_checked(x: f64) -> Result<f64, PyError> {
    if x.is_infinite() {
        Err(PyError::Value("math domain error".to_string()))
    } else {
        Ok(x)
    }
}

/// Float `x ** y`: a finite base with a finite exponent that gives an infinity is Python's
/// `OverflowError`.
pub(crate) fn py_pow(x: f64, y: f64) -> Result<f64, PyError> {
    let r = libm::pow(x, y);
    if r.is_infinite() && x.is_finite() && y.is_finite() {
        Err(PyError::Overflow)
    } else {
        Ok(r)
    }
}

/// `x ** 2`.
pub(crate) fn sq(x: f64) -> Result<f64, PyError> {
    py_pow(x, 2.0)
}

// --- geometry helpers ------------------------------------------------------------------------

pub(crate) type Pt = (f64, f64);

/// Sutherland-Hodgman: keep the part of polygon `poly` (list of (Y, Z)) with Z <= h.
pub(crate) fn clip_below(poly: &[Pt], h: f64) -> Vec<Pt> {
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
pub(crate) fn area_centroid(poly: &[Pt]) -> (f64, f64, f64) {
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

/// Trapezoid area under the GZ curve between `a0` and `a1` degrees.
pub(crate) fn area(gz: &[(i32, f64)], a0: i32, a1: i32) -> f64 {
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

pub(crate) fn criterion(name: &str, value: f64, limit: f64, ok: bool, source: &str) -> Criterion {
    Criterion {
        name: name.to_string(),
        value: PyNum::Float(value),
        limit,
        ok,
        source: source.to_string(),
    }
}

pub(crate) fn criteria(cset: &str, c: &Condition) -> Result<Vec<Criterion>, PyError> {
    if cset == "IS_CODE_2008" {
        let src = "IMO IS Code 2008, Part A 2.2";
        let gz_30_plus = py_max(
            c.gz.iter()
                .filter(|p| p.0 >= CRITERIA_ANGLES_DEG[0])
                .map(|p| p.1),
        );
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
    Err(PyError::Value(format!(
        "unknown criteria set {}",
        crate::pystr::repr_str(cset)
    )))
}

/// `max(gz, key=lambda p: p[1])`: the first of equal GZ values. `gz` must not be empty.
pub(crate) fn max_by_gz(gz: &[(i32, f64)]) -> (i32, f64) {
    let (mut best_deg, mut best) = gz[0];
    for &(deg, g) in &gz[1..] {
        if g > best {
            best_deg = deg;
            best = g;
        }
    }
    (best_deg, best)
}

/// Initial GM of a box section with no free surface: `KB + BM - KG`, with `KB = T / 2` and
/// `BM = B^2 / (12 T)`.
pub(crate) fn box_gm_solid(beam: f64, draught: f64, kg: f64) -> Result<f64, PyError> {
    let (kb, bm) = (draught / 2.0, div(beam * beam, 12.0 * draught)?);
    Ok(kb + bm - kg)
}

/// The stability of one box section at one condition: the GZ curve on the 1 degree grid, the
/// areas, the angle of maximum GZ, and the criteria of `criteria_set`. `gm_solid` is the GM with no
/// free surface; `free_surface_m` is the free surface correction (m of GM), applied to the GM and,
/// as `FSC x sin(phi)`, to every GZ. The identity fields (`name`, cargo, fill, displacement) are
/// left empty or zero for the caller to set.
pub(crate) fn box_condition_at(
    beam: f64,
    depth: f64,
    draught: f64,
    kg: f64,
    gm_solid: f64,
    free_surface_m: f64,
    criteria_set: &str,
) -> Result<Condition, PyError> {
    let (b, d, t, fsc) = (beam, depth, draught, free_surface_m);
    let mut gz: Vec<(i32, f64)> = Vec::with_capacity(61);
    for a in ANGLES_DEG {
        let heel = box_gz(b, d, t, kg, f64::from(a))?;
        gz.push((a, heel - fsc * sin(radians(f64::from(a)))?));
    }
    let (max_gz_deg, max_gz) = max_by_gz(&gz);
    let deck_edge = degrees(libm::atan(div(2.0 * (d - t), b)?));
    let mut c = Condition {
        name: String::new(),
        cargo_t: 0.0,
        cargo_is_int: false,
        fill_fraction: 0.0,
        displacement_t: 0.0,
        draught_m: t,
        kg_m: kg,
        gm_solid_m: gm_solid,
        free_surface_m: fsc,
        gm_fluid_m: gm_solid - fsc,
        max_gz_m: max_gz,
        max_gz_deg,
        area_0_27: area(&gz, 0, 27),
        area_0_30: area(&gz, 0, CRITERIA_ANGLES_DEG[0]),
        area_0_40: area(&gz, 0, CRITERIA_ANGLES_DEG[1]),
        area_30_40: area(&gz, CRITERIA_ANGLES_DEG[0], CRITERIA_ANGLES_DEG[1]),
        deck_edge_deg: Some(deck_edge),
        gz,
        criteria: Vec::new(),
    };
    c.criteria = criteria(criteria_set, &c)?;
    Ok(c)
}

/// The intact stability criteria of a box section, for a known KG (issue #184). The same work
/// `assess` does for each loading condition of a gas barge, without the gas arrangement: give the
/// section (beam, depth, draught), KG above the keel, the free surface correction in metres of GM,
/// and the criteria set (`"IS_CODE_2008"`). The draught must be below the depth (a hull that does
/// not float with freeboard has no curve). `displacement_t`, cargo and fill are 0 here: the
/// section does not know its length.
pub fn box_condition(
    beam: f64,
    depth: f64,
    draught: f64,
    kg: f64,
    free_surface_m: f64,
    criteria_set: &str,
) -> Result<Condition, PyError> {
    if draught >= depth {
        return Err(PyError::Value(
            "box_condition needs a draught below the depth".to_string(),
        ));
    }
    let gm_solid = box_gm_solid(beam, draught, kg)?;
    let mut c = box_condition_at(
        beam,
        depth,
        draught,
        kg,
        gm_solid,
        free_surface_m,
        criteria_set,
    )?;
    c.name = "Box".to_string();
    Ok(c)
}
