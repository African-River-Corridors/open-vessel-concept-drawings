//! Damage stability by the lost buoyancy method (issue #341, spec 0006, ADN 2025 9.3.2.15 and 9.3.3.15).
//!
//! The hull is the Sections Table. A damage floods boxes (compartments): the sea fills them, so the
//! water in a flooded compartment is not displaced by the hull any more. With the lost buoyancy method
//! the mass and the centre of gravity do not change; the displaced volume of the damaged hull is the
//! immersed volume of the intact hull less the permeable volume of each flooded box that lies below the
//! waterplane. The floating position (height of the waterplane, trim and heel) is where that displaced
//! volume equals the mass over the density and its centre of buoyancy is on the vertical through G.
//!
//! Heel is positive to starboard and trim positive by the bow, as in [`crate::sections`]. The solves are
//! damped Newton solves with a finite difference Jacobian, so a damage costs a few tens of cuts of the
//! hull where a free-trim float of the sections costs a few hundred.
//!
//! What is not modelled: the intermediate stages of flooding, a free surface in the flooded spaces
//! (they are open to the sea), the flooding of undamaged compartments through openings, and the
//! non-watertight openings (the model holds none, so the limit is the 27 degrees of the criteria).

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use libm::{cos, fabs, sqrt};

use crate::sections::{Gravity, SectionsTable, Waterplane, immersed};

const RAD_PER_DEG: f64 = core::f64::consts::PI / 180.0;

/// A flooded compartment: a box in hull axes, with the fraction of its volume the water fills.
#[derive(Debug, Clone, PartialEq)]
pub struct FloodedBox {
    pub name: String,
    pub x_aft_m: f64,
    pub x_fwd_m: f64,
    pub y_min_m: f64,
    pub y_max_m: f64,
    pub z_min_m: f64,
    pub z_max_m: f64,
    /// 0 to 1: ADN 9.3.2.15.1: 95 % where the space is empty at the maximum draught, 0 % where it is full.
    pub permeability: f64,
}

/// The part of a body below a waterplane: its volume and the moments of that volume about the origin.
#[derive(Debug, Clone, Copy, Default)]
struct Part {
    v: f64,
    mx: f64,
    my: f64,
    mz: f64,
}

/// Keep the part of the polygon with `a y + b z <= d` (Sutherland-Hodgman).
fn clip(poly: &[(f64, f64)], a: f64, b: f64, d: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    let n = poly.len();
    for i in 0..n {
        let (p, q) = (poly[i], poly[(i + 1) % n]);
        let (fp, fq) = (a * p.0 + b * p.1 - d, a * q.0 + b * q.1 - d);
        if fp <= 0.0 {
            out.push(p);
        }
        if (fp <= 0.0) != (fq <= 0.0) {
            let t = fp / (fp - fq);
            out.push((p.0 + t * (q.0 - p.0), p.1 + t * (q.1 - p.1)));
        }
    }
    out
}

/// Area and first moments (about y = 0 and z = 0) of a polygon, by the shoelace formula.
fn area_moments(poly: &[(f64, f64)]) -> (f64, f64, f64) {
    let (mut a, mut my, mut mz) = (0.0, 0.0, 0.0);
    let n = poly.len();
    for i in 0..n {
        let ((y0, z0), (y1, z1)) = (poly[i], poly[(i + 1) % n]);
        let c = y0 * z1 - y1 * z0;
        a += c;
        my += (y0 + y1) * c;
        mz += (z0 + z1) * c;
    }
    let a = 0.5 * a;
    (fabs(a), my / 6.0 * a.signum(), mz / 6.0 * a.signum())
}

/// The part of the box below the waterplane. The area of the cut section is quadratic in x between the
/// x at which the waterline crosses a corner of the box, and the first moments are cubic, so Simpson's
/// rule on each of those intervals is exact.
fn box_below(wp: &Waterplane, b: &FloodedBox) -> Part {
    let (kx, ky, kz) = wp.normal();
    let corners = [
        (b.y_min_m, b.z_min_m),
        (b.y_max_m, b.z_min_m),
        (b.y_max_m, b.z_max_m),
        (b.y_min_m, b.z_max_m),
    ];
    let section = |x: f64| -> (f64, f64, f64) {
        let d = wp.c - kx * x;
        let poly = clip(&corners, ky, kz, d);
        if poly.len() < 3 {
            return (0.0, 0.0, 0.0);
        }
        area_moments(&poly)
    };
    let mut cuts = vec![b.x_aft_m, b.x_fwd_m];
    if fabs(kx) > 1e-12 {
        for &(y, z) in &corners {
            let x = (wp.c - ky * y - kz * z) / kx;
            if x > b.x_aft_m && x < b.x_fwd_m {
                cuts.push(x);
            }
        }
    }
    cuts.sort_by(f64::total_cmp);
    let mut p = Part::default();
    for pair in cuts.windows(2) {
        let (x0, x1) = (pair[0], pair[1]);
        if x1 - x0 < 1e-12 {
            continue;
        }
        let xm = 0.5 * (x0 + x1);
        let (a0, y0, z0) = section(x0);
        let (am, ym, zm) = section(xm);
        let (a1, y1, z1) = section(x1);
        let w = (x1 - x0) / 6.0;
        p.v += w * (a0 + 4.0 * am + a1);
        p.mx += w * (x0 * a0 + 4.0 * xm * am + x1 * a1);
        p.my += w * (y0 + 4.0 * ym + y1);
        p.mz += w * (z0 + 4.0 * zm + z1);
    }
    p
}

/// The displaced volume and the centre of buoyancy of the damaged hull at a waterplane.
#[derive(Debug, Clone, Copy)]
struct Buoyancy {
    v: f64,
    lcb: f64,
    tcb: f64,
    kb: f64,
}

fn buoyancy(table: &SectionsTable, flooded: &[FloodedBox], wp: &Waterplane) -> Buoyancy {
    let im = immersed(table, wp);
    let mut v = im.volume_m3;
    let (mut mx, mut my, mut mz) = (v * im.lcb_m, v * im.tcb_m, v * im.kb_m);
    for b in flooded {
        let p = box_below(wp, b);
        v -= b.permeability * p.v;
        mx -= b.permeability * p.mx;
        my -= b.permeability * p.my;
        mz -= b.permeability * p.mz;
    }
    if v > 0.0 {
        Buoyancy {
            v,
            lcb: mx / v,
            tcb: my / v,
            kb: mz / v,
        }
    } else {
        Buoyancy {
            v: 0.0,
            lcb: 0.0,
            tcb: 0.0,
            kb: 0.0,
        }
    }
}

type V3 = (f64, f64, f64);

fn dot(a: V3, b: V3) -> f64 {
    a.0 * b.0 + a.1 * b.1 + a.2 * b.2
}

/// The earth axes in hull axes: `u` horizontal across (towards the low side at a positive heel), `w`
/// horizontal forward (as [`crate::sections`] writes them).
fn earth_axes(wp: &Waterplane) -> (V3, V3) {
    let (kx, ky, kz) = wp.normal();
    let ct = cos(wp.trim_deg * RAD_PER_DEG);
    let u = (0.0, kz / ct, -ky / ct);
    let w = (u.1 * kz - u.2 * ky, u.2 * kx, -u.1 * kx);
    (u, w)
}

/// What the solves read: the hull, the flooded boxes, the mass and its centre of gravity.
struct Problem<'a> {
    table: &'a SectionsTable,
    flooded: &'a [FloodedBox],
    /// The volume of water displaced, m³: the mass over the density.
    target_m3: f64,
    g: Gravity,
}

/// The residuals at a waterplane: the volume (relative), then B - G along the earth's forward
/// horizontal and across (the righting lever, positive when righting).
fn residuals(p: &Problem, wp: &Waterplane) -> [f64; 3] {
    let b = buoyancy(p.table, p.flooded, wp);
    let (u, w) = earth_axes(wp);
    let bg = (b.lcb - p.g.lcg_m, b.tcb - p.g.tcg_m, b.kb - p.g.kg_m);
    [(b.v - p.target_m3) / p.target_m3, dot(bg, w), dot(bg, u)]
}

fn norm(r: &[f64]) -> f64 {
    sqrt(r.iter().fold(0.0, |a, x| a + x * x))
}

/// Solve `a x = b` for 3 or 2 unknowns by Gaussian elimination with partial pivoting.
fn solve_linear(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for col in 0..n {
        let mut pivot = col;
        for row in col + 1..n {
            if fabs(a[row][col]) > fabs(a[pivot][col]) {
                pivot = row;
            }
        }
        if fabs(a[pivot][col]) < 1e-14 {
            return None;
        }
        a.swap(col, pivot);
        b.swap(col, pivot);
        for row in col + 1..n {
            let f = a[row][col] / a[col][col];
            let pivot_row = a[col].clone();
            for (cell, p) in a[row].iter_mut().zip(&pivot_row).skip(col) {
                *cell -= f * p;
            }
            b[row] -= f * b[col];
        }
    }
    let mut x = vec![0.0; n];
    for row in (0..n).rev() {
        let mut s = b[row];
        for k in row + 1..n {
            s -= a[row][k] * x[k];
        }
        x[row] = s / a[row][row];
    }
    Some(x)
}

/// The step of the righting lever curve beyond the equilibrium, degrees.
const STEP_DEG: f64 = 2.0;
/// The most Newton steps of a solve.
const NEWTON_STEPS: u32 = 40;
/// A solve is converged when the residuals are under this (relative volume; metres of lever).
const TOLERANCE: f64 = 1e-7;

/// The unknowns of a solve: the height of the waterplane `c`, the trim and, when the heel is free, the
/// heel (degrees).
#[derive(Debug, Clone, Copy)]
struct State {
    c: f64,
    trim_deg: f64,
    heel_deg: f64,
}

impl State {
    fn wp(&self) -> Waterplane {
        Waterplane {
            heel_deg: self.heel_deg,
            trim_deg: self.trim_deg,
            c: self.c,
        }
    }
}

/// Damped Newton on the first `n` residuals (2: heel held; 3: heel free). `None` when the solve does not
/// converge or the state leaves its bounds.
fn newton(p: &Problem, start: State, n: usize) -> Option<State> {
    let mut s = start;
    let steps = [1e-4 * (1.0 + fabs(start.c)), 0.005, 0.01];
    let pick = |r: [f64; 3]| -> Vec<f64> { r[..n].to_vec() };
    let mut r = pick(residuals(p, &s.wp()));
    for _ in 0..NEWTON_STEPS {
        if norm(&r) < TOLERANCE {
            return Some(s);
        }
        // finite difference Jacobian, column by column
        let mut jac = vec![vec![0.0; n]; n];
        for col in 0..n {
            let mut t = s;
            match col {
                0 => t.c += steps[0],
                1 => t.trim_deg += steps[1],
                _ => t.heel_deg += steps[2],
            }
            let rt = pick(residuals(p, &t.wp()));
            for row in 0..n {
                jac[row][col] = (rt[row] - r[row]) / steps[col];
            }
        }
        let delta = solve_linear(jac, r.iter().map(|x| -x).collect())?;
        // halve the step until the residual falls
        let mut scale = 1.0;
        let mut improved = false;
        for _ in 0..12 {
            let mut t = s;
            t.c += scale * delta[0];
            t.trim_deg += scale * delta[1];
            if n == 3 {
                t.heel_deg += scale * delta[2];
            }
            if fabs(t.trim_deg) <= 20.0 && fabs(t.heel_deg) <= 80.0 {
                let rt = pick(residuals(p, &t.wp()));
                if norm(&rt) < norm(&r) {
                    s = t;
                    r = rt;
                    improved = true;
                    break;
                }
            }
            scale *= 0.5;
        }
        if !improved {
            return None;
        }
    }
    (norm(&r) < TOLERANCE * 100.0).then_some(s)
}

/// The criteria at the stage of equilibrium and beyond it: ADN 9.3.2.15.2 and 9.3.3.15.2, or MARPOL Annex I
/// regulation 28.3 (issue #341).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageCriteria {
    /// The angle of heel at the stage of equilibrium shall not exceed this (degrees): 12.
    pub max_heel_deg: f64,
    /// Beyond the equilibrium the righting lever reaches at least this (m): 0.05.
    pub min_gz_m: f64,
    /// ... in association with an area under the curve of at least this (m.rad): 0.0065.
    pub min_area_m_rad: f64,
    /// The minimum values are met up to the immersion of the first non-watertight opening and in any
    /// event up to this angle of heel (degrees): 27.
    pub limit_deg: f64,
    /// A range beyond the equilibrium over which the lever stays positive (degrees): MARPOL 28.3.3 asks 20.
    /// With a range, `limit_deg` is not used: the curve is read to the equilibrium heel plus the range.
    pub range_beyond_deg: Option<f64>,
}

impl DamageCriteria {
    /// ADN 2025 9.3.2.15.2 and 9.3.3.15.2.
    pub const ADN_2025: DamageCriteria = DamageCriteria {
        max_heel_deg: 12.0,
        min_gz_m: 0.05,
        min_area_m_rad: 0.0065,
        limit_deg: 27.0,
        range_beyond_deg: None,
    };

    /// MARPOL Annex I regulation 28.3.2 and 28.3.3: a heel at equilibrium of at most 25 degrees (the 30 degrees
    /// of 28.3.2 needs no deck edge immersion, which is not checked), a range of 20 degrees beyond it with a
    /// righting lever of at least 0.1 m and an area of at least 0.0175 m.rad within that range.
    pub const MARPOL_I_28: DamageCriteria = DamageCriteria {
        max_heel_deg: 25.0,
        min_gz_m: 0.1,
        min_area_m_rad: 0.0175,
        limit_deg: 0.0,
        range_beyond_deg: Some(20.0),
    };
}

/// How the damaged hull floats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Floats {
    /// An equilibrium was found.
    Yes,
    /// The mass is more than the hull and its unflooded volume can float: it sinks.
    Sinks,
    /// No equilibrium was found by the solves (a capsize, or a position the solves could not reach).
    NoEquilibrium,
}

/// The outcome of one damage: the position at equilibrium, the residual stability, and the verdict.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageOutcome {
    pub floats: Floats,
    /// Heel at the stage of equilibrium (degrees, positive to starboard) and trim (degrees, positive by the bow).
    pub heel_deg: f64,
    pub trim_deg: f64,
    /// The draught at mid-length at the stage of equilibrium, m.
    pub draught_mid_m: f64,
    /// The height of the waterplane along the earth vertical at the stage of equilibrium, m (see
    /// [`Waterplane`]).
    pub waterplane_c_m: f64,
    /// The angle beyond the equilibrium the residual stability is read to (degrees from the vertical):
    /// the least of the limit angle and the angle at which the righting lever is lost again.
    pub end_deg: f64,
    /// The greatest righting lever between the equilibrium and `end_deg`, m.
    pub max_gz_m: f64,
    /// The area under the righting lever curve between the equilibrium and `end_deg`, m.rad.
    pub area_m_rad: f64,
    /// Every criterion of [`DamageCriteria`] is met.
    pub ok: bool,
}

impl DamageOutcome {
    fn failed(floats: Floats) -> DamageOutcome {
        DamageOutcome {
            floats,
            heel_deg: 0.0,
            trim_deg: 0.0,
            draught_mid_m: 0.0,
            waterplane_c_m: 0.0,
            end_deg: 0.0,
            max_gz_m: 0.0,
            area_m_rad: 0.0,
            ok: false,
        }
    }
}

/// The floating position and the residual stability of a hull with flooded boxes.
///
/// `start` is the intact floating position (height of the waterplane, trim in degrees); the solves
/// begin from it. `mass_t` and `g` are those of the loading condition, unchanged by the damage.
pub fn assess(
    table: &SectionsTable,
    flooded: &[FloodedBox],
    mass_t: f64,
    g: Gravity,
    density: f64,
    start: (f64, f64),
    criteria: &DamageCriteria,
) -> DamageOutcome {
    let target = mass_t / density;
    if target.is_nan() || target <= 0.0 {
        return DamageOutcome::failed(Floats::NoEquilibrium);
    }
    // The whole hull below the waterplane, less every flooded box: the most the hull can displace.
    let top = Waterplane {
        heel_deg: 0.0,
        trim_deg: 0.0,
        c: f64::MAX / 4.0,
    };
    let total = buoyancy(table, flooded, &top).v;
    if target >= total {
        return DamageOutcome::failed(Floats::Sinks);
    }
    let p = Problem {
        table,
        flooded,
        target_m3: target,
        g,
    };
    // The stage of equilibrium: c, trim and heel together.
    let mut guess = State {
        c: start.0 + 0.1,
        trim_deg: start.1,
        heel_deg: 0.0,
    };
    let mut eq = None;
    // A free heel can start at either side; try the upright start, then each side (a symmetric damage
    // has its root at no heel, and a one-sided one on its own side).
    for heel0 in [0.0, 5.0, -5.0] {
        guess.heel_deg = heel0;
        if let Some(s) = newton(&p, guess, 3) {
            eq = Some(s);
            break;
        }
    }
    let Some(eq) = eq else {
        return DamageOutcome::failed(Floats::NoEquilibrium);
    };
    let side = if eq.heel_deg >= 0.0 { 1.0 } else { -1.0 };
    let heel_eq = eq.heel_deg;
    let draught_mid = {
        let wp = eq.wp();
        let (kx, _, kz) = wp.normal();
        let x = 0.5 * table.sections.last().map_or(0.0, |s| s.x);
        (wp.c - kx * x) / kz
    };
    // The righting lever curve beyond the equilibrium, in the direction of the heel, `STEP_DEG` at a time,
    // to the limit angle or the loss of the righting lever.
    let mut state = eq;
    let (mut max_gz, mut area, mut prev_gz, mut prev_deg) =
        (0.0_f64, 0.0_f64, 0.0_f64, fabs(heel_eq));
    let mut end_deg = fabs(heel_eq);
    let mut deg = fabs(heel_eq);
    let limit_deg = match criteria.range_beyond_deg {
        Some(r) => fabs(heel_eq) + r,
        None => criteria.limit_deg,
    };
    while deg < limit_deg {
        let next = (deg + STEP_DEG).min(limit_deg);
        let target_heel = side * next;
        let mut t = state;
        t.heel_deg = target_heel;
        let Some(s) = newton(&p, t, 2) else {
            break;
        };
        state = s;
        let gz = side * residuals(&p, &s.wp())[2];
        // The first angle at which the lever is no longer positive ends the range, at its crossing.
        if gz <= 0.0 {
            let frac = if prev_gz > 0.0 {
                prev_gz / (prev_gz - gz)
            } else {
                0.0
            };
            area += 0.5 * prev_gz * frac * (frac * (next - prev_deg)) * RAD_PER_DEG;
            end_deg = prev_deg + frac * (next - prev_deg);
            break;
        }
        // trapezoid from the previous angle
        area += 0.5 * (prev_gz + gz) * (next - prev_deg) * RAD_PER_DEG;
        max_gz = max_gz.max(gz);
        prev_gz = gz;
        prev_deg = next;
        end_deg = next;
        deg = next;
    }
    // A range asks the lever to stay positive over all of it: the curve must run to the end of the range.
    let range_met = criteria.range_beyond_deg.is_none() || end_deg >= limit_deg - 1e-9;
    let ok = fabs(heel_eq) <= criteria.max_heel_deg
        && range_met
        && max_gz >= criteria.min_gz_m
        && area >= criteria.min_area_m_rad;
    DamageOutcome {
        floats: Floats::Yes,
        heel_deg: heel_eq,
        trim_deg: eq.trim_deg,
        draught_mid_m: draught_mid,
        waterplane_c_m: eq.c,
        end_deg,
        max_gz_m: max_gz,
        area_m_rad: area,
        ok,
    }
}

/// The righting lever (m) of the damaged hull at one heel (degrees), with the floating position solved at
/// that heel (height of the waterplane and free trim): the curve of [`assess`] one point at a time.
/// `start` is the intact (height, trim in degrees). `None` when the solve does not converge.
pub fn righting_lever_at(
    table: &SectionsTable,
    flooded: &[FloodedBox],
    mass_t: f64,
    g: Gravity,
    density: f64,
    start: (f64, f64),
    heel_deg: f64,
) -> Option<f64> {
    let p = Problem {
        table,
        flooded,
        target_m3: mass_t / density,
        g,
    };
    let s = newton(
        &p,
        State {
            c: start.0 + 0.1,
            trim_deg: start.1,
            heel_deg,
        },
        2,
    )?;
    Some(residuals(&p, &s.wp())[2])
}
