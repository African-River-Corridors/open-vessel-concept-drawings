//! Compartments: the typed Spaces of a hull (issue #426, ADR 0016).
//!
//! One compartment model for every liquid a hull holds: water ballast now, the crude oil cargo tanks
//! of the `liquid_bulk` module later (#330). A [`Space`] is a prism: a cross-section polygon in the
//! (y, z) plane, swept from `x_aft` to `x_fwd`. x is from the aft end of the hull, y from the centre
//! line (positive to starboard), z from the keel.
//!
//! A fill gives the liquid's volume, level, centroid and the transverse inertia of its free surface,
//! by clipping the section at the level. The volumes are moulded: no deduction for structure.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::barge_capacity::{BargeCapacityInputs, BargeCapacityResult};
use crate::box_stability::{area_centroid, clip_below};
use crate::cargo::CargoClass;

/// What a Space holds. The crude oil module adds its own kinds (#330).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceKind {
    /// A double-bottom water ballast tank.
    DoubleBottomBallast,
    /// A wing (side) water ballast tank.
    WingBallast,
    /// A peak (end) water ballast tank across the full beam: the fore peak, or the aft peak of a
    /// pushed barge.
    PeakBallast,
}

impl SpaceKind {
    pub fn key(self) -> &'static str {
        match self {
            SpaceKind::DoubleBottomBallast => "double_bottom_ballast",
            SpaceKind::WingBallast => "wing_ballast",
            SpaceKind::PeakBallast => "peak_ballast",
        }
    }

    pub fn is_ballast(self) -> bool {
        matches!(
            self,
            SpaceKind::DoubleBottomBallast | SpaceKind::WingBallast | SpaceKind::PeakBallast
        )
    }
}

/// The side of the centre line a Space lies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Port,
    Starboard,
    /// Across the centre line.
    Centre,
}

impl Side {
    pub fn key(self) -> &'static str {
        match self {
            Side::Port => "port",
            Side::Starboard => "starboard",
            Side::Centre => "centre",
        }
    }

    fn sign(self) -> f64 {
        match self {
            Side::Port => -1.0,
            Side::Starboard | Side::Centre => 1.0,
        }
    }

    fn letter(self) -> &'static str {
        match self {
            Side::Port => "P",
            Side::Starboard => "S",
            Side::Centre => "C",
        }
    }
}

/// One compartment: a prism of `section` from `x_aft` to `x_fwd`.
#[derive(Debug, Clone, PartialEq)]
pub struct Space {
    /// A short name, the same in every language: "DB1P", "WB1S".
    pub name: String,
    pub kind: SpaceKind,
    pub side: Side,
    pub x_aft: f64,
    pub x_fwd: f64,
    /// The cross-section, (y, z) corners in order. A simple polygon.
    pub section: Vec<(f64, f64)>,
}

/// The liquid in a Space at one fill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    /// The fraction of the Space's volume that is full, 0 to 1.
    pub fraction: f64,
    pub volume_m3: f64,
    /// The height of the liquid surface above the keel (m).
    pub level_m: f64,
    pub lcg_m: f64,
    pub tcg_m: f64,
    pub vcg_m: f64,
    /// The transverse moment of inertia of the free surface about its own centre line (m⁴). Zero
    /// for an empty or a pressed-full Space: there is no free surface.
    pub free_surface_inertia_m4: f64,
}

impl Space {
    pub fn length(&self) -> f64 {
        self.x_fwd - self.x_aft
    }

    pub fn volume_m3(&self) -> f64 {
        area_centroid(&self.section).0 * self.length()
    }

    /// The mass of the Space pressed full of a liquid of `density` (t/m³).
    pub fn capacity_t(&self, density: f64) -> f64 {
        self.volume_m3() * density
    }

    /// The lowest and highest z of the section.
    pub fn z_range(&self) -> (f64, f64) {
        self.section
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &(_, z)| {
                (lo.min(z), hi.max(z))
            })
    }

    /// The liquid at `fraction` of the volume (clamped to 0..1).
    pub fn fill(&self, fraction: f64) -> Fill {
        let fraction = fraction.clamp(0.0, 1.0);
        let (z_lo, z_hi) = self.z_range();
        let full = area_centroid(&self.section).0;
        let lcg = 0.5 * (self.x_aft + self.x_fwd);
        if fraction <= 0.0 || full <= 0.0 {
            return Fill {
                fraction: 0.0,
                volume_m3: 0.0,
                level_m: z_lo,
                lcg_m: lcg,
                tcg_m: 0.0,
                vcg_m: z_lo,
                free_surface_inertia_m4: 0.0,
            };
        }
        let level = if fraction >= 1.0 {
            z_hi
        } else {
            // The wetted area rises with the level: bisect.
            let target = fraction * full;
            let (mut lo, mut hi) = (z_lo, z_hi);
            for _ in 0..100 {
                let mid = 0.5 * (lo + hi);
                if area_centroid(&clip_below(&self.section, mid)).0 < target {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            0.5 * (lo + hi)
        };
        let wet = clip_below(&self.section, level);
        let (area, y, z) = area_centroid(&wet);
        let inertia = if fraction >= 1.0 {
            0.0
        } else {
            self.length() * chord_inertia(&self.section, level)
        };
        Fill {
            fraction,
            volume_m3: area * self.length(),
            level_m: level,
            lcg_m: lcg,
            tcg_m: y,
            vcg_m: z,
            free_surface_inertia_m4: inertia,
        }
    }
}

/// The second moment, per metre of length, of the chords of `poly` at `level`, about their joint
/// centre line: ∫ (y − ȳ)² dy over the free surface.
fn chord_inertia(poly: &[(f64, f64)], level: f64) -> f64 {
    let mut ys: Vec<f64> = Vec::new();
    for i in 0..poly.len() {
        let p = poly[i];
        let q = poly[(i + 1) % poly.len()];
        if (p.1 <= level) != (q.1 <= level) {
            let t = (level - p.1) / (q.1 - p.1);
            ys.push(p.0 + t * (q.0 - p.0));
        }
    }
    ys.sort_by(f64::total_cmp);
    let chords: Vec<(f64, f64)> = ys.as_chunks::<2>().0.iter().map(|&[a, b]| (a, b)).collect();
    let breadth: f64 = chords.iter().map(|(a, b)| b - a).sum();
    if breadth <= 0.0 {
        return 0.0;
    }
    let ybar = chords
        .iter()
        .map(|(a, b)| (b - a) * 0.5 * (a + b))
        .sum::<f64>()
        / breadth;
    chords
        .iter()
        .map(|(a, b)| {
            let (u, v) = (a - ybar, b - ybar);
            (v * v * v - u * u * u) / 3.0
        })
        .sum()
}

/// A rectangle section from `y0` to `y1` (|y|, outboard positive) and `z0` to `z1`, on `side`.
fn rect(side: Side, y0: f64, y1: f64, z0: f64, z1: f64) -> Vec<(f64, f64)> {
    poly(side, &[(y0, z0), (y1, z0), (y1, z1), (y0, z1)])
}

/// Mirror |y| corners to `side`, keeping the corners counter-clockwise in the (y, z) plane.
fn poly(side: Side, corners: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut out: Vec<(f64, f64)> = corners.iter().map(|&(y, z)| (side.sign() * y, z)).collect();
    if side == Side::Port {
        out.reverse();
    }
    out
}

/// A pair of Spaces, port and starboard, named `<prefix>1P` and `<prefix>1S`.
fn pair(
    prefix: &str,
    kind: SpaceKind,
    (x_aft, x_fwd): (f64, f64),
    section: impl Fn(Side) -> Vec<(f64, f64)>,
) -> Vec<Space> {
    [Side::Port, Side::Starboard]
        .into_iter()
        .map(|side| Space {
            name: alloc::format!("{prefix}1{}", side.letter()),
            kind,
            side,
            x_aft,
            x_fwd,
            section: section(side),
        })
        .filter(|s| s.volume_m3() > 0.0)
        .collect()
}

/// The water ballast tanks of a barge-capacity result: a double-bottom tank and a wing tank on each
/// side, the length of the cargo zone, and the peaks. One tank per side and kind: the engine has no
/// ballast bulkheads along the hull yet. A tank of no volume is left out.
///
/// - Type C tanks: the double bottom (its height), and the wing tank (its width) from the double
///   bottom up to the wing plate, as the weights model has them.
/// - Holds: the double bottom, and the wing space outboard of the hold side up to the deck. For a
///   sloped hopper hold the wing space takes in the hopper.
/// - The fore peak: the bow void, full beam and depth. The raked part counts half its length, as a
///   rake that rises the full depth has half the volume of a box.
/// - The aft peak, for a pushed barge only: the stern void. A self-propelled hull has its machinery
///   there.
pub fn ballast_tanks(inp: &BargeCapacityInputs, r: &BargeCapacityResult) -> Vec<Space> {
    let half_b = inp.beam / 2.0;
    let stern = r.stern_void_length_m.unwrap_or(0.0);
    let bow = r.bow_void_length_m.unwrap_or(0.0);
    let zone = (stern, (inp.loa - bow).max(stern));
    let mut out = Vec::new();
    match r.phase {
        CargoClass::Gas => {
            let Some(g) = r.gas_basis.as_ref() else {
                return out;
            };
            let db = g.inputs.double_bottom_height.unwrap_or(0.0);
            let w = g.inputs.wing_tank_width.unwrap_or(0.0).clamp(0.0, half_b);
            let top = db
                + g.inputs.tank_support_height
                + g.inputs.wing_tank_height_ratio * g.tanks.outer_diameter;
            out.extend(pair("DB", SpaceKind::DoubleBottomBallast, zone, |s| {
                rect(s, 0.0, half_b, 0.0, db)
            }));
            out.extend(pair("WB", SpaceKind::WingBallast, zone, |s| {
                rect(s, half_b - w, half_b, db, top.max(db))
            }));
        }
        CargoClass::DryBulk => {
            let db = inp.double_bottom_height_bulk.clamp(0.0, inp.depth);
            let d = inp.depth;
            let w_top = r
                .hold_top_width_m
                .unwrap_or(inp.beam - 2.0 * inp.wing_void_width);
            let w_bot = r.hold_bottom_width_m.unwrap_or(w_top);
            let h = r.hold_height_m.unwrap_or(d - db);
            // The hold side runs from half the bottom width at the inner bottom to half the top
            // width at the hold top; at the deck it is part way.
            let at_deck = if h > 0.0 {
                w_bot / 2.0 + (w_top - w_bot) / 2.0 * ((d - db) / h)
            } else {
                w_top / 2.0
            };
            let (y_bot, y_deck) = ((w_bot / 2.0).min(half_b), at_deck.min(half_b));
            out.extend(pair("DB", SpaceKind::DoubleBottomBallast, zone, |s| {
                rect(s, 0.0, half_b, 0.0, db)
            }));
            out.extend(pair("WB", SpaceKind::WingBallast, zone, |s| {
                poly(s, &[(y_bot, db), (half_b, db), (half_b, d), (y_deck, d)])
            }));
        }
        // The liquid bulk module lays out its own Spaces, ballast included (#330).
        CargoClass::LiquidBulk => return out,
    }
    let rake = r.bow_rake_length_m.unwrap_or(0.0).clamp(0.0, bow);
    let fp_len = (bow - rake) + rake / 2.0;
    out.extend(peak(
        "FP1C",
        (inp.loa - bow, inp.loa - bow + fp_len),
        half_b,
        inp.depth,
    ));
    if !inp.self_propelled {
        out.extend(peak("AP1C", (0.0, stern), half_b, inp.depth));
    }
    out
}

/// A peak tank, full beam and depth, from `x_aft` to `x_fwd`. None when it has no length.
fn peak(name: &str, (x_aft, x_fwd): (f64, f64), half_b: f64, depth: f64) -> Option<Space> {
    (x_fwd > x_aft && depth > 0.0).then(|| Space {
        name: name.into(),
        kind: SpaceKind::PeakBallast,
        side: Side::Centre,
        x_aft,
        x_fwd,
        section: vec![
            (-half_b, 0.0),
            (half_b, 0.0),
            (half_b, depth),
            (-half_b, depth),
        ],
    })
}

/// The water ballast tanks' total volume (m³).
pub fn total_volume_m3(spaces: &[Space]) -> f64 {
    spaces.iter().map(Space::volume_m3).sum()
}

/// A fill per Space, in order: every Space empty.
pub fn empty_fills(spaces: &[Space]) -> Vec<f64> {
    vec![0.0; spaces.len()]
}
