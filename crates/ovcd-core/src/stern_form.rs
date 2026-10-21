//! The Stern Form of a self-propelled hull (issue #350, spec 0007 "Stern geometry"): transom, pram
//! or tunnel, in closed form. One entry function, [`geometry`].
//!
//! x is measured forward from the transom (the aft end of the hull), z up from the base. The stern
//! run is the length `run` aft of which the hull bottom leaves the base line.
//!
//! - **Transom:** a straight rake from the base at x = run up to the transom height at x = 0, as
//!   the raked box has its stern rake (`hull.rs`). With run 0 it is today's square stern, and the
//!   raked box gives exactly today's values.
//! - **Pram:** the bottom rises along a parabola z_b(x) = h_t (1 - x / run)^2, tangent to the base at
//!   x = run, to the transom height h_t at x = 0.
//! - **Tunnel:** the bottom stays on the base to the transom; over each propulsor a tunnel of radius
//!   D (Heuser, Rotteveel 2019 sec. 2.3.1) with its crown at `tunnel_crown_height_m` is cut into it.
//!   Its section at the propeller plane is the circular segment above the base; it tapers linearly to
//!   nothing at x = run, so the volume is the section below the waterline x run / 2 per tunnel.
//!
//! The sections table of ADR 0012 will later replace these closed forms with sections the generator
//! writes from the same inputs ([`SternFormInputs`]); the closed forms then stay as its test, as the
//! raked box does.

use alloc::vec::Vec;

use libm::{acos, pow, sqrt};

/// The three Stern Forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SternForm {
    Transom,
    Pram,
    Tunnel,
}

impl SternForm {
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "transom" => Some(Self::Transom),
            "pram" => Some(Self::Pram),
            "tunnel" => Some(Self::Tunnel),
            _ => None,
        }
    }
}

/// Spec 0007: the largest propeller over the design draught, under a tunnel (estimate: inland
/// D/T runs 0.57 to 0.93, the research note sec. 2.4) and at an open transom or pram stern
/// (sourced: MAN, seagoing, "d/D below approximately 0.65"). Under a tunnel D_max is also capped
/// so the tunnels fit ([`tunnel_fit_diameter`]).
pub const K_D_TUNNEL: f64 = 0.80;
pub const K_D_OPEN: f64 = 0.65;
/// The tip clearance under the tunnel crown, as a share of D (estimate).
pub const TUNNEL_TIP_CLEARANCE: f64 = 0.10;
/// The default run lengths of a transom and a pram stern, over the moulded depth (estimates; the
/// pram ratio is today's bow rake ratio).
pub const TRANSOM_RUN_OVER_DEPTH: f64 = 1.0;
pub const PRAM_RUN_OVER_DEPTH: f64 = 1.6;

/// The inputs of the Stern Form, as the `.vdesign` gives them; `None` is the rule of spec 0007.
#[derive(Debug, Clone, PartialEq)]
pub struct SternFormInputs {
    pub form: SternForm,
    pub run_length_m: Option<f64>,
    pub transom_height_m: Option<f64>,
    pub tunnel_crown_height_m: Option<f64>,
    /// The number of propulsors (tunnels under a tunnel stern).
    pub propulsor_count: u32,
    pub propeller_diameter_m: Option<f64>,
    /// The largest unit of the Propulsor's maker range (m), which caps the default D (the
    /// maintainer's decision on issue #352); `None`: no cap.
    pub max_unit_diameter_m: Option<f64>,
    /// Shaft propulsors run parallel shafts (Heuser 0.45 B spacing), azimuth units converge (0.28 B).
    pub parallel_shafts: bool,
}

/// The hull the stern belongs to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HullDims {
    pub loa: f64,
    pub beam: f64,
    pub depth: f64,
    pub design_draught: f64,
}

/// The stern of one hull, worked out.
#[derive(Debug, Clone, PartialEq)]
pub struct SternGeometry {
    pub form: SternForm,
    pub beam: f64,
    /// The run length (m) and the transom height above base (m).
    pub run_m: f64,
    pub transom_height_m: f64,
    /// The largest propeller the stern allows (m), and the diameter in force (typed, or D_max).
    pub d_max_m: f64,
    pub propeller_diameter_m: f64,
    /// Under a tunnel stern: the crown height (m) and the athwartships centre of each tunnel (m).
    pub tunnel_crown_height_m: f64,
    pub tunnel_offsets_m: Vec<f64>,
}

/// The Heuser stern run length of a tunnel stern (Rotteveel 2019 sec. 2.3.1): single screw
/// 1.9 D + 1.85 B; twin, converging 2.5 D + 1.65 B; twin, parallel 3.2 D + 1.45 B.
pub fn heuser_run_length(d: f64, beam: f64, count: u32, parallel: bool) -> f64 {
    match (count, parallel) {
        (0 | 1, _) => 1.9 * d + 1.85 * beam,
        (_, false) => 2.5 * d + 1.65 * beam,
        (_, true) => 3.2 * d + 1.45 * beam,
    }
}

/// Work out the stern from its inputs and the hull.
pub fn geometry(inp: &SternFormInputs, hull: &HullDims) -> SternGeometry {
    let t = hull.design_draught;
    let k_d = match inp.form {
        SternForm::Tunnel => K_D_TUNNEL,
        _ => K_D_OPEN,
    };
    let n = inp.propulsor_count.max(1);
    let spacing = if inp.parallel_shafts { 0.45 } else { 0.28 } * hull.beam;
    let d_max = match inp.form {
        SternForm::Tunnel => (k_d * t).min(tunnel_fit_diameter(hull.beam, spacing, n)),
        _ => k_d * t,
    };
    let d = inp
        .propeller_diameter_m
        .unwrap_or(match inp.max_unit_diameter_m {
            Some(largest) => d_max.min(largest),
            None => d_max,
        });
    let run = inp.run_length_m.unwrap_or(match inp.form {
        SternForm::Transom => TRANSOM_RUN_OVER_DEPTH * hull.depth,
        SternForm::Pram => PRAM_RUN_OVER_DEPTH * hull.depth,
        SternForm::Tunnel => heuser_run_length(d, hull.beam, n, inp.parallel_shafts),
    });
    let transom = inp.transom_height_m.unwrap_or(match inp.form {
        SternForm::Tunnel => 0.0,
        _ => t,
    });
    // The crown: the propeller tip on the base, the axis at D/2, the crown D/2 + 0.10 D above it.
    let crown = inp
        .tunnel_crown_height_m
        .unwrap_or(d + TUNNEL_TIP_CLEARANCE * d);
    // One tunnel on the centreline; two or more spread evenly over the Heuser shaft spacing.
    let tunnel_offsets_m = match (inp.form, n) {
        (SternForm::Tunnel, 1) => alloc::vec![0.0],
        (SternForm::Tunnel, n) => (0..n)
            .map(|i| (f64::from(i) / f64::from(n - 1) - 0.5) * spacing)
            .collect(),
        _ => Vec::new(),
    };
    SternGeometry {
        form: inp.form,
        beam: hull.beam,
        run_m: run,
        transom_height_m: transom,
        d_max_m: d_max,
        propeller_diameter_m: d,
        tunnel_crown_height_m: crown,
        tunnel_offsets_m,
    }
}

/// The largest D whose tunnels fit the stern (the maintainer's decision on issue #350): with the
/// crown at D + the tip clearance, a tunnel of radius D is 2 D sqrt(1 - 0.1^2) wide at the base, so
/// side by side the tunnels may meet but not overlap, and the outer ones stay inside the beam.
pub fn tunnel_fit_diameter(beam: f64, spacing: f64, count: u32) -> f64 {
    let half_width = sqrt(1.0 - TUNNEL_TIP_CLEARANCE * TUNNEL_TIP_CLEARANCE);
    if count <= 1 {
        return beam / 2.0 / half_width;
    }
    let pitch = spacing / f64::from(count - 1);
    (pitch / 2.0).min((beam - spacing) / 2.0) / half_width
}

/// The area of the circular segment of radius `r` whose top (crown) is at `crown` above the base,
/// cut by the base (z = 0) and taken below the level `z` (z clipped to [0, crown]): the section of
/// one tunnel at the propeller plane below a waterline.
pub fn tunnel_section_area(r: f64, crown: f64, z: f64) -> f64 {
    // The circle's centre is r below the crown; the part above the base is the segment.
    let zc = crown - r;
    let area_between = |z0: f64, z1: f64| -> f64 {
        // The area of the circle between heights z0 and z1 (z0 <= z1), both clipped to the circle.
        let f = |z: f64| {
            let u = ((z - zc) / r).clamp(-1.0, 1.0);
            r * r * (u * sqrt(1.0 - u * u) + libm::asin(u))
        };
        f(z1) - f(z0)
    };
    let lo = 0.0_f64.max(zc - r);
    let hi = z.clamp(0.0, crown);
    if hi <= lo { 0.0 } else { area_between(lo, hi) }
}

/// The width of one tunnel at height `z` (its chord), 0 outside it.
pub fn tunnel_width(r: f64, crown: f64, z: f64) -> f64 {
    let zc = crown - r;
    if z < 0.0 || z > crown {
        return 0.0;
    }
    let dz = z - zc;
    2.0 * sqrt((r * r - dz * dz).max(0.0))
}

impl SternGeometry {
    /// The volume the stern removes from the box below the waterline `draft` (m^3), and its first
    /// moment about the transom (m^4). A transom stern is the raked box's stern rake and returns its
    /// removed volume too.
    pub fn removed(&self, draft: f64) -> (f64, f64) {
        if draft <= 0.0 || self.run_m <= 0.0 {
            return (0.0, 0.0);
        }
        let (l, b) = (self.run_m, self.beam);
        match self.form {
            SternForm::Transom => {
                // The rake's bottom: z_b(x) = h (1 - x / L). Removed = B ∫ min(z_b, T) dx.
                let h = self.transom_height_m;
                if h <= 0.0 {
                    return (0.0, 0.0);
                }
                let x_t = if draft < h {
                    l * (1.0 - draft / h)
                } else {
                    0.0
                };
                let t = draft.min(h);
                // Below x_t the whole draught is removed; from x_t to L the triangle under z_b.
                let v = b * (t * x_t + (l - x_t) * t / 2.0);
                let m = b
                    * (t * x_t * x_t / 2.0
                        + t * ((l - x_t) * x_t / 2.0 + (l - x_t) * (l - x_t) / 6.0));
                (v, m)
            }
            SternForm::Pram => {
                let h = self.transom_height_m;
                if h <= 0.0 {
                    return (0.0, 0.0);
                }
                let u_t = if draft < h { sqrt(draft / h) } else { 1.0 };
                let x_t = l * (1.0 - u_t);
                let t = draft.min(h);
                // Bottom part u from 0 to u_t: ∫ h u^2 dx, x = L (1 - u).
                let v = b * (t * x_t + h * l * pow(u_t, 3.0) / 3.0);
                let m = b
                    * (t * x_t * x_t / 2.0
                        + h * l * l * (pow(u_t, 3.0) / 3.0 - pow(u_t, 4.0) / 4.0));
                (v, m)
            }
            SternForm::Tunnel => {
                let a = tunnel_section_area(
                    self.propeller_diameter_m,
                    self.tunnel_crown_height_m,
                    draft,
                );
                let n = self.tunnel_offsets_m.len() as f64;
                // Tapering linearly from the transom to x = L: volume A L / 2, centroid L / 3.
                (n * a * l / 2.0, n * a * l * l / 6.0)
            }
        }
    }

    /// The waterplane area the stern removes at `draft` (m^2).
    pub fn waterplane_cut(&self, draft: f64) -> f64 {
        if draft <= 0.0 || self.run_m <= 0.0 {
            return 0.0;
        }
        let (l, b, h) = (self.run_m, self.beam, self.transom_height_m);
        match self.form {
            SternForm::Transom if h > 0.0 && draft < h => b * l * (1.0 - draft / h),
            SternForm::Pram if h > 0.0 && draft < h => b * l * (1.0 - sqrt(draft / h)),
            SternForm::Tunnel => {
                let w = tunnel_width(self.propeller_diameter_m, self.tunnel_crown_height_m, draft);
                self.tunnel_offsets_m.len() as f64 * w * l / 2.0
            }
            _ => 0.0,
        }
    }

    /// The wetted surface the stern adds over the flat bottom it replaces (m^2): the tunnel's
    /// curved roof less its chord at the base, over the tapering run; the bottom of a transom or
    /// pram run is longer than its plan by its slope.
    pub fn extra_wetted_surface(&self, draft: f64) -> f64 {
        if self.run_m <= 0.0 {
            return 0.0;
        }
        let (l, b) = (self.run_m, self.beam);
        match self.form {
            SternForm::Tunnel => {
                let r = self.propeller_diameter_m;
                let zc = self.tunnel_crown_height_m - r;
                // The arc above the base: 2 r acos(-zc / r) when the base cuts the circle.
                let u = (-zc / r).clamp(-1.0, 1.0);
                let arc = 2.0 * r * acos(u);
                let chord = tunnel_width(r, self.tunnel_crown_height_m, 0.0);
                let _ = draft;
                self.tunnel_offsets_m.len() as f64 * (arc - chord).max(0.0) * l / 2.0
            }
            SternForm::Transom => {
                let h = self.transom_height_m.min(draft.max(0.0));
                b * (sqrt(l * l + h * h) - l)
            }
            SternForm::Pram => {
                // The parabola's length, by 50 strips.
                let h = self.transom_height_m;
                let n = 50;
                let mut s = 0.0;
                for i in 0..n {
                    let (x0, x1) = (l * i as f64 / n as f64, l * (i + 1) as f64 / n as f64);
                    let z = |x: f64| h * pow(1.0 - x / l, 2.0);
                    s += sqrt((x1 - x0) * (x1 - x0) + (z(x1) - z(x0)) * (z(x1) - z(x0)));
                }
                b * (s - l)
            }
        }
    }

    /// Whether the stern differs from today's square stern (run 0).
    pub fn is_legacy(&self) -> bool {
        self.run_m <= 0.0
    }

    /// The profile outline of the stern bottom, from the transom to the end of the run:
    /// (x, z) points for the drawings.
    pub fn profile(&self) -> Vec<[f64; 2]> {
        let (l, h) = (self.run_m, self.transom_height_m);
        match self.form {
            SternForm::Transom => alloc::vec![[0.0, h], [l, 0.0]],
            SternForm::Pram => (0..=16)
                .map(|i| {
                    let x = l * i as f64 / 16.0;
                    [x, h * pow(1.0 - x / l, 2.0)]
                })
                .collect(),
            SternForm::Tunnel => (0..=16)
                .map(|i| {
                    // The tunnel crown along the run, from the crown at the transom to the base.
                    let x = l * i as f64 / 16.0;
                    [x, self.tunnel_crown_height_m * (1.0 - x / l)]
                })
                .collect(),
        }
    }

    /// The stern section at the transom: the tunnel outlines (y, z), one list per tunnel.
    pub fn tunnel_sections(&self) -> Vec<Vec<[f64; 2]>> {
        if self.form != SternForm::Tunnel {
            return Vec::new();
        }
        let r = self.propeller_diameter_m;
        let zc = self.tunnel_crown_height_m - r;
        // The half-angle about the top of the circle to where it meets the base: more than a
        // quarter turn when the centre is above the base (issue #486).
        let a0 = if r > 0.0 {
            acos((-zc / r).clamp(-1.0, 1.0))
        } else {
            0.0
        };
        self.tunnel_offsets_m
            .iter()
            .map(|y0| {
                (0..=16)
                    .map(|i| {
                        let a = -a0 + 2.0 * a0 * i as f64 / 16.0;
                        [y0 + r * libm::sin(a), zc + r * libm::cos(a)]
                    })
                    .collect()
            })
            .collect()
    }
}

/// The removed volume and moment by numerical integration of the same shape (for the tests):
/// `strips` strips along the run.
pub fn removed_by_strips(g: &SternGeometry, draft: f64, strips: u32) -> (f64, f64) {
    let l = g.run_m;
    let mut v = 0.0;
    let mut m = 0.0;
    for i in 0..strips {
        let x = (f64::from(i) + 0.5) * l / f64::from(strips);
        let dx = l / f64::from(strips);
        let area = match g.form {
            SternForm::Transom => g.beam * (g.transom_height_m * (1.0 - x / l)).min(draft),
            SternForm::Pram => g.beam * (g.transom_height_m * pow(1.0 - x / l, 2.0)).min(draft),
            SternForm::Tunnel => {
                g.tunnel_offsets_m.len() as f64
                    * tunnel_section_area(g.propeller_diameter_m, g.tunnel_crown_height_m, draft)
                    * (1.0 - x / l)
            }
        };
        v += area * dx;
        m += area * x * dx;
    }
    (v, m)
}
