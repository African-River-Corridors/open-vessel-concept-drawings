//! The free surface of a slack tank. A rectangular tank of length l and breadth b has the transverse
//! inertia i = l b³ / 12 about its own centre line; the free surface moment is the cargo density times i.
//! The value comes from the shared Space model (the chords of the section at the level), which gives
//! the same figure for a rectangle: the tests compare it with the closed form.

use alloc::string::String;
use alloc::vec::Vec;

use crate::compartments::{Space, SpaceKind};

/// The free surface of one cargo or slop tank.
#[derive(Debug, Clone, PartialEq)]
pub struct TankFreeSurface {
    pub name: String,
    pub length_m: f64,
    pub breadth_m: f64,
    /// The transverse inertia of the free surface at a slack level (m⁴). The same at every level
    /// between empty and full for a box tank.
    pub inertia_m4: f64,
}

/// The breadth of a tank: its widest chord (m).
pub(super) fn breadth_m(s: &Space) -> f64 {
    let (lo, hi) = s
        .section
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &(y, _)| {
            (lo.min(y), hi.max(y))
        });
    hi - lo
}

/// The free surface of every cargo and slop tank, in the order of the Spaces.
pub(super) fn of_tanks(spaces: &[Space]) -> Vec<TankFreeSurface> {
    spaces
        .iter()
        .filter(|s| matches!(s.kind, SpaceKind::CargoTank | SpaceKind::Slop))
        .map(|s| TankFreeSurface {
            name: s.name.clone(),
            length_m: s.length(),
            breadth_m: breadth_m(s),
            inertia_m4: s.fill(0.5).free_surface_inertia_m4,
        })
        .collect()
}
