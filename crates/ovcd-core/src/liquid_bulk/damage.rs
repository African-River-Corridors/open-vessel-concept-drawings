//! The damage cases of a liquid bulk layout (issue #341, spec 0006 section 5, ADN 2025 9.3.2.15 and
//! 9.3.3.15), as flooded boxes for [`crate::damage_stability`].
//!
//! The cargo zone is the Spaces. A ballast space (a double bottom or a wing tank) is cut at the transverse
//! bulkheads of the cargo tanks beside it, so a damage floods one compartment of it, not the whole length of
//! the tank block: the cut is an assumption of this model (the Spaces hold each ballast tank as one prism).
//! A cofferdam and a pump room are single compartments. The end spaces beyond the cargo zone (peaks, an engine
//! room) are not modelled, so a damage there floods nothing.
//!
//! The extents are those of the ADN text: a longitudinal extent of 0.10 L but not less than 5.00 m, placed
//! anywhere along the hull; a side damage 0.79 m (type C, 9.3.2.15.1) or 0.59 m (type N, 9.3.3.15.1) inboard
//! of the side, from the base line up without limit; a bottom damage 3.00 m wide and 0.59 m (type C) or 0.49 m
//! (type N) high. A bulkhead within the damage is damaged, so every compartment the extent touches floods,
//! and for a bottom damage the adjacent compartments athwartships flood too. Permeability is 95 % in a space
//! that is empty at the maximum draught (ballast, cofferdams, pump room) and 0 % in a cargo or slop tank,
//! which is full.
//!
//! MARPOL Annex I regulation 28 has its own extents ([`MarpolExtents`], [`marpol_damage_cases`]): they reach the
//! cargo tanks. A damaged cargo or slop tank is flooded at 95 % (the oil is lost and sea water takes its
//! place, regulation 28.4.2 "other liquids 0 to 0.95"); the loading condition keeps the mass of the oil, as the
//! lost buoyancy method keeps the mass of the damaged hull.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::compartments::{Space, SpaceKind};
use crate::damage_stability::FloodedBox;

/// The extents of the assumed damage (ADN 2025 9.3.2.15.1 and 9.3.3.15.1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdnExtents {
    /// Side damage: how far inboard of the side, m.
    pub side_inboard_m: f64,
    /// Bottom damage: how wide, m.
    pub bottom_width_m: f64,
    /// Bottom damage: how high above the base line, m.
    pub bottom_height_m: f64,
}

impl AdnExtents {
    /// Type C: 9.3.2.15.1.
    pub const TYPE_C: AdnExtents = AdnExtents {
        side_inboard_m: 0.79,
        bottom_width_m: 3.00,
        bottom_height_m: 0.59,
    };
    /// Type N: 9.3.3.15.1.
    pub const TYPE_N: AdnExtents = AdnExtents {
        side_inboard_m: 0.59,
        bottom_width_m: 3.00,
        bottom_height_m: 0.49,
    };
}

/// The longitudinal extent of a damage: 0.10 L, but not less than 5.00 m.
pub fn longitudinal_extent_m(loa_m: f64) -> f64 {
    let tenth = 0.10 * loa_m;
    if tenth > 5.0 { tenth } else { 5.0 }
}

/// One damage: what it floods.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageCase {
    pub name: String,
    pub flooded: Vec<FloodedBox>,
}

/// The extents of the assumed damage of MARPOL Annex I regulation 28.2, for a hull of length L and beam B.
///
/// The bottom damage has two sets: for 0.3 L from the forward perpendicular and for any other part.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarpolExtents {
    /// Side damage: longitudinal extent, `(1/3) L^(2/3)` or 14.5 m, whichever is less (28.2.1.1).
    pub side_long_m: f64,
    /// Side damage: how far inboard of the side, `B/5` or 11.5 m, whichever is less (28.2.1.2).
    pub side_inboard_m: f64,
    /// Bottom damage, forward 0.3 L: longitudinal extent, `(1/3) L^(2/3)` or 14.5 m (28.2.2.1).
    pub bottom_fwd_long_m: f64,
    /// Bottom damage, forward 0.3 L: width, `B/6` or 10 m, whichever is less (28.2.2.2).
    pub bottom_fwd_width_m: f64,
    /// Bottom damage, any other part: longitudinal extent, `(1/3) L^(2/3)` or 5 m (28.2.2.1).
    pub bottom_aft_long_m: f64,
    /// Bottom damage, any other part: width, `B/6` or 5 m (28.2.2.2).
    pub bottom_aft_width_m: f64,
    /// Bottom damage: height from the moulded line of the bottom shell, `B/15` or 6 m (28.2.2.3).
    pub bottom_height_m: f64,
}

impl MarpolExtents {
    pub fn of(loa_m: f64, beam_m: f64) -> MarpolExtents {
        let cbrt = libm::cbrt(loa_m);
        let third_l23 = cbrt * cbrt / 3.0;
        MarpolExtents {
            side_long_m: third_l23.min(14.5),
            side_inboard_m: (beam_m / 5.0).min(11.5),
            bottom_fwd_long_m: third_l23.min(14.5),
            bottom_fwd_width_m: (beam_m / 6.0).min(10.0),
            bottom_aft_long_m: third_l23.min(5.0),
            bottom_aft_width_m: (beam_m / 6.0).min(5.0),
            bottom_height_m: (beam_m / 15.0).min(6.0),
        }
    }
}

/// A compartment of the cargo zone, with the kind of the Space it comes from.
struct Compartment {
    name: String,
    kind: SpaceKind,
    x: (f64, f64),
    y: (f64, f64),
    z: (f64, f64),
}

/// The 0.95 of ADN 9.3.2.15.1 for a space that is empty at the maximum draught, and 0 for a full one.
fn permeability(kind: SpaceKind) -> f64 {
    match kind {
        SpaceKind::CargoTank | SpaceKind::Slop => 0.0,
        _ => 0.95,
    }
}

fn bbox(s: &Space) -> ((f64, f64), (f64, f64)) {
    let (mut y0, mut y1, mut z0, mut z1) = (
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    );
    for &(y, z) in s.section() {
        y0 = y0.min(y);
        y1 = y1.max(y);
        z0 = z0.min(z);
        z1 = z1.max(z);
    }
    ((y0, y1), (z0, z1))
}

/// The compartments: every Space, with each ballast space cut at the transverse bulkheads of the cargo
/// tanks.
fn compartments(spaces: &[Space]) -> Vec<Compartment> {
    let mut bulkheads: Vec<f64> = spaces
        .iter()
        .filter(|s| matches!(s.kind, SpaceKind::CargoTank | SpaceKind::Slop))
        .flat_map(|s| [s.x_aft, s.x_fwd])
        .collect();
    bulkheads.sort_by(f64::total_cmp);
    bulkheads.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let mut out = Vec::new();
    for s in spaces {
        let (y, z) = bbox(s);
        let cuts = matches!(
            s.kind,
            SpaceKind::DoubleBottomBallast | SpaceKind::WingBallast | SpaceKind::PeakBallast
        );
        let mut edges = alloc::vec![s.x_aft];
        if cuts {
            edges.extend(
                bulkheads
                    .iter()
                    .copied()
                    .filter(|&b| b > s.x_aft + 1e-9 && b < s.x_fwd - 1e-9),
            );
        }
        edges.push(s.x_fwd);
        for (i, w) in edges.windows(2).enumerate() {
            out.push(Compartment {
                name: if edges.len() > 2 {
                    format!("{}/{}", s.name, i + 1)
                } else {
                    s.name.clone()
                },
                kind: s.kind,
                x: (w[0], w[1]),
                y,
                z,
            });
        }
    }
    out
}

fn overlaps(a: (f64, f64), b: (f64, f64)) -> bool {
    a.0.max(b.0) + 1e-9 < a.1.min(b.1)
}

/// The damage cases of a layout: the side damage to starboard (the hull is symmetric, so port is its
/// mirror) and the bottom damage, each at every position along the hull that floods a different set of
/// compartments. The damage is placed with an edge on each bulkhead and on each end of the hull.
pub fn damage_cases(
    spaces: &[Space],
    loa_m: f64,
    beam_m: f64,
    extents: &AdnExtents,
) -> Vec<DamageCase> {
    let parts = compartments(spaces);
    let len = longitudinal_extent_m(loa_m).min(loa_m);
    let mut edges: Vec<f64> = parts.iter().flat_map(|c| [c.x.0, c.x.1]).collect();
    edges.push(0.0);
    edges.push(loa_m);
    // The damage starts at every edge, ends at every edge, and, to flood the compartments on both sides of a
    // bulkhead, starts or ends just beyond it.
    const BEYOND_M: f64 = 1e-3;
    let mut starts: Vec<f64> = edges
        .iter()
        .flat_map(|&e| [e, e - len, e - BEYOND_M, e - len + BEYOND_M])
        .map(|s| s.clamp(0.0, loa_m - len))
        .collect();
    starts.sort_by(f64::total_cmp);
    starts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let half = 0.5 * beam_m;
    let mut cases: Vec<DamageCase> = Vec::new();
    for &s0 in &starts {
        let window = (s0, s0 + len);
        for side in [true, false] {
            let hit: Vec<&Compartment> = parts
                .iter()
                .filter(|c| overlaps(c.x, window))
                .filter(|c| {
                    if side {
                        // starboard: the strip inboard of the side, from the base line up without limit
                        overlaps(c.y, (half - extents.side_inboard_m, half))
                    } else {
                        // bottom: below the height of the damage; athwartships neighbours flood too
                        c.z.0 + 1e-9 < extents.bottom_height_m
                    }
                })
                .filter(|c| permeability(c.kind) > 0.0)
                .collect();
            if hit.is_empty() {
                continue;
            }
            let mut names: Vec<&str> = hit.iter().map(|c| c.name.as_str()).collect();
            names.sort_unstable();
            if cases.iter().any(|k| {
                let mut other: Vec<&str> = k.flooded.iter().map(|f| f.name.as_str()).collect();
                other.sort_unstable();
                other == names && k.name.starts_with(if side { "side" } else { "bottom" })
            }) {
                continue;
            }
            cases.push(DamageCase {
                name: format!(
                    "{} damage, {:.1} to {:.1} m",
                    if side { "side (starboard)" } else { "bottom" },
                    window.0,
                    window.1
                ),
                flooded: hit
                    .iter()
                    .map(|c| FloodedBox {
                        name: c.name.clone(),
                        x_aft_m: c.x.0,
                        x_fwd_m: c.x.1,
                        y_min_m: c.y.0,
                        y_max_m: c.y.1,
                        z_min_m: c.z.0,
                        z_max_m: c.z.1,
                        permeability: permeability(c.kind),
                    })
                    .collect(),
            });
        }
    }
    cases
}

/// The length from which regulation 28.1.1 and 28.1.2 put the damage anywhere in the ship's length (above
/// 150 m). At 150 m or less, regulation 28.1.3 puts it between adjacent transverse bulkheads.
const BETWEEN_BULKHEADS_UP_TO_M: f64 = 150.0;

/// The damage cases of MARPOL Annex I regulation 28 (issue #341): the side damage to starboard and the bottom
/// damage at each position along and across the hull that floods a different set of compartments.
///
/// Placement. Above 150 m the damage is anywhere in the length (28.1.1; the machinery space of 28.1.2 is not
/// modelled). At 150 m or less (28.1.3) it lies between adjacent transverse bulkheads, and a bulkhead is damaged
/// only where it is next to a bay shorter than the damage (28.2.5.1; 28.2.5.2, the step or recess, is not
/// modelled). A bottom damage is in the forward set when its centre is in the forward 0.3 L, which takes
/// `L` as the overall length. The bottom damage lies across the beam wherever it floods a different set, from the
/// centre line to starboard (the hull is symmetric).
pub fn marpol_damage_cases(spaces: &[Space], loa_m: f64, beam_m: f64) -> Vec<DamageCase> {
    let ext = MarpolExtents::of(loa_m, beam_m);
    let parts = compartments(spaces);
    let half = 0.5 * beam_m;
    let mut bulkheads: Vec<f64> = parts
        .iter()
        .flat_map(|c| [c.x.0, c.x.1])
        .filter(|&e| e > 1e-9 && e < loa_m - 1e-9)
        .collect();
    bulkheads.sort_by(f64::total_cmp);
    bulkheads.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    let mut bounds = alloc::vec![0.0];
    bounds.extend(bulkheads.iter().copied());
    bounds.push(loa_m);
    let between = loa_m <= BETWEEN_BULKHEADS_UP_TO_M;
    // The windows of one longitudinal extent: with an edge on each bulkhead and on each end of the hull, and
    // just beyond each bulkhead, so the compartments on both sides of it flood.
    let windows = |len: f64| -> Vec<(f64, f64)> {
        let len = len.min(loa_m);
        const BEYOND_M: f64 = 1e-3;
        let mut starts: Vec<f64> = bounds
            .iter()
            .flat_map(|&e| [e, e - len, e - BEYOND_M, e - len + BEYOND_M])
            .map(|s| s.clamp(0.0, loa_m - len))
            .collect();
        starts.sort_by(f64::total_cmp);
        starts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        starts
            .into_iter()
            .map(|s| (s, s + len))
            .filter(|&(a, b)| {
                !between
                    || bulkheads
                        .iter()
                        .filter(|&&k| k > a + 1e-9 && k < b - 1e-9)
                        .all(|&k| {
                            // 28.2.5.1: damaged only next to a bay shorter than the damage
                            let i = bounds
                                .iter()
                                .position(|&e| (e - k).abs() < 1e-9)
                                .unwrap_or(0);
                            let before = bounds[i] - bounds[i.saturating_sub(1)];
                            let after = bounds[(i + 1).min(bounds.len() - 1)] - bounds[i];
                            before < len - 1e-9 || after < len - 1e-9
                        })
            })
            .collect()
    };
    let flooded_of = |hit: &[&Compartment]| -> Vec<FloodedBox> {
        hit.iter()
            .map(|c| FloodedBox {
                name: c.name.clone(),
                x_aft_m: c.x.0,
                x_fwd_m: c.x.1,
                y_min_m: c.y.0,
                y_max_m: c.y.1,
                z_min_m: c.z.0,
                z_max_m: c.z.1,
                permeability: 0.95,
            })
            .collect()
    };
    let mut cases: Vec<DamageCase> = Vec::new();
    let mut seen: Vec<(bool, Vec<String>)> = Vec::new();
    let mut push = |side: bool, name: String, hit: Vec<&Compartment>| {
        if hit.is_empty() {
            return;
        }
        let mut names: Vec<String> = hit.iter().map(|c| c.name.clone()).collect();
        names.sort_unstable();
        if seen.iter().any(|(s, n)| *s == side && *n == names) {
            return;
        }
        seen.push((side, names));
        cases.push(DamageCase {
            name,
            flooded: flooded_of(&hit),
        });
    };
    for (a, b) in windows(ext.side_long_m) {
        let hit: Vec<&Compartment> = parts
            .iter()
            .filter(|c| overlaps(c.x, (a, b)) && overlaps(c.y, (half - ext.side_inboard_m, half)))
            .collect();
        push(
            true,
            format!("side damage (starboard), {a:.1} to {b:.1} m"),
            hit,
        );
    }
    // The forward set and the other set differ in length and width; a window belongs to the set its centre is in.
    for (forward, long_m, width_m) in [
        (true, ext.bottom_fwd_long_m, ext.bottom_fwd_width_m),
        (false, ext.bottom_aft_long_m, ext.bottom_aft_width_m),
    ] {
        let mut y_edges: Vec<f64> = parts.iter().flat_map(|c| [c.y.0, c.y.1]).collect();
        y_edges.push(-half);
        y_edges.push(half);
        const BEYOND_M: f64 = 1e-3;
        let mut y_starts: Vec<f64> = y_edges
            .iter()
            .flat_map(|&e| [e, e - width_m, e - BEYOND_M, e - width_m + BEYOND_M])
            .map(|y| y.clamp(-half, half - width_m))
            .filter(|&y| y + 0.5 * width_m >= -1e-9)
            .collect();
        y_starts.sort_by(f64::total_cmp);
        y_starts.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
        for (a, b) in windows(long_m) {
            if (0.5 * (a + b) >= 0.7 * loa_m - 1e-9) != forward {
                continue;
            }
            for &y0 in &y_starts {
                let y1 = y0 + width_m;
                let hit: Vec<&Compartment> = parts
                    .iter()
                    .filter(|c| {
                        overlaps(c.x, (a, b))
                            && overlaps(c.y, (y0, y1))
                            && c.z.0 + 1e-9 < ext.bottom_height_m
                    })
                    .collect();
                push(
                    false,
                    format!("bottom damage, {a:.1} to {b:.1} m, {y0:.1} to {y1:.1} m across"),
                    hit,
                );
            }
        }
    }
    cases
}
