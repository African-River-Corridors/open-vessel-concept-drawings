//! Vessel Classes (GLOSSARY: a standard envelope from a published scheme or a recognised size band),
//! and the ladder a vessel climbs as its length and beam grow (issue #94).
//!
//! The classes are `catalogue/vessel_classes.yaml`, the one source: `vessel_classes_generated.rs` is
//! written from it by `tools/gen_vessel_classes.py`, and a test fails when the two differ.
//!
//! **The rules** (maintainer's answers on issue #94):
//! - A *scheme* is a class's `family` (CEMT-1992, RWS-2010, CN-GB50139, ...). The scheme never changes
//!   by itself; only the user picks it.
//! - A class a vessel can be defined by (*pickable*) has a length and a beam and a Formation kind the
//!   `.vdesign` format knows: `motor_vessel` is `self_propelled`, `pushed_barge` is `pushed_barge`,
//!   `pushed_convoy` and `coupled_unit` are `pushed_convoy`. Towed barges, ATBs, pushers, tugs and
//!   towboats have no such kind, and a class with no length or beam cannot set them: neither is
//!   pickable, and neither is on a ladder.
//! - The *ladder* is the pickable classes of one scheme and one Formation kind, smallest first (by
//!   length maximum, then beam maximum, then id). Motor vessel and convoy classes are separate ladders.
//! - A vessel sits in the smallest class whose length and beam maxima both hold it. Past the largest
//!   class of the scheme it stays in that class and [`Placement::beyond_largest`] is true.
//! - A class's length and beam are *maxima*. For a range the maximum is the upper end.

use alloc::vec::Vec;

/// One row of `catalogue/vessel_classes.yaml`, as the generator writes it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClassRow {
    pub id: &'static str,
    /// The scheme the class belongs to.
    pub family: &'static str,
    pub region: &'static str,
    pub waters: &'static str,
    /// `motor_vessel`, `pushed_convoy`, `pushed_barge`, `towed_barge`, `coupled_unit`, `atb`,
    /// `pusher`, `towboat` or `tug`.
    pub formation: &'static str,
    /// Catalogue data. These do not filter a class list (Cargo is never a class property).
    pub cargo_modes: &'static [&'static str],
    pub loa_m: Option<(f64, f64)>,
    pub beam_m: Option<(f64, f64)>,
    pub draught_m: Option<(f64, f64)>,
    /// The class's bridge clearance, the air draft it allows.
    pub air_draft_m: Option<(f64, f64)>,
    pub dwt_t: Option<(f64, f64)>,
    pub n_abreast: Option<i64>,
    pub n_in_line: Option<i64>,
    pub confidence: &'static str,
    pub source: &'static str,
    pub notes: &'static str,
}

include!("vessel_classes_generated.rs");

/// The Formation kind of a `.vdesign` file that a class formation word stands for.
pub fn formation_kind(class_formation: &str) -> Option<&'static str> {
    match class_formation {
        "motor_vessel" => Some("self_propelled"),
        "pushed_barge" => Some("pushed_barge"),
        "pushed_convoy" | "coupled_unit" => Some("pushed_convoy"),
        _ => None,
    }
}

impl ClassRow {
    /// The Formation kind of a vessel in this class, when the file format has one.
    pub fn kind(&self) -> Option<&'static str> {
        formation_kind(self.formation)
    }

    /// Length overall maximum (m).
    pub fn loa_max_m(&self) -> Option<f64> {
        self.loa_m.map(|(_, hi)| hi)
    }

    /// Beam maximum (m).
    pub fn beam_max_m(&self) -> Option<f64> {
        self.beam_m.map(|(_, hi)| hi)
    }

    /// The draught limit the class gives: the top of its draught range.
    pub fn draught_limit_m(&self) -> Option<f64> {
        self.draught_m.map(|(_, hi)| hi)
    }

    /// The air-draft limit the class gives: the bottom of its air-draft range, the lowest bridge
    /// clearance.
    pub fn air_draft_limit_m(&self) -> Option<f64> {
        self.air_draft_m.map(|(lo, _)| lo)
    }

    /// Barges abreast and in line; a class that gives neither counts one.
    pub fn counts(&self) -> (i64, i64) {
        (self.n_abreast.unwrap_or(1), self.n_in_line.unwrap_or(1))
    }

    /// A vessel can be defined by this class: it has a length, a beam and a Formation kind.
    pub fn pickable(&self) -> bool {
        self.loa_max_m().is_some() && self.beam_max_m().is_some() && self.kind().is_some()
    }
}

/// The class with this id. An id the core used before issue #94 finds its class.
pub fn find(id: &str) -> Option<&'static ClassRow> {
    let id = LEGACY_IDS
        .iter()
        .find(|(old, _)| *old == id)
        .map_or(id, |(_, new)| *new);
    CLASSES.iter().find(|c| c.id == id)
}

/// Every scheme that has a pickable class, in the order of the catalogue.
pub fn schemes() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for c in CLASSES.iter().filter(|c| c.pickable()) {
        if !out.contains(&c.family) {
            out.push(c.family);
        }
    }
    out
}

/// The ladder of one scheme and one Formation kind, smallest class first.
pub fn ladder(scheme: &str, kind: &str) -> Vec<&'static ClassRow> {
    let mut list: Vec<&'static ClassRow> = CLASSES
        .iter()
        .filter(|c| c.pickable() && c.family == scheme && c.kind() == Some(kind))
        .collect();
    list.sort_by(|a, b| {
        let key = |c: &ClassRow| {
            (
                c.loa_max_m().unwrap_or(f64::NAN),
                c.beam_max_m().unwrap_or(f64::NAN),
            )
        };
        let (ka, kb) = (key(a), key(b));
        ka.0.total_cmp(&kb.0)
            .then(ka.1.total_cmp(&kb.1))
            .then(a.id.cmp(b.id))
    });
    list
}

/// Where a vessel sits on a ladder.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub class: &'static ClassRow,
    /// The vessel is longer or wider than the largest class of the scheme.
    pub beyond_largest: bool,
}

/// The class a vessel of this length and beam sits in, within one scheme and Formation kind. `None`
/// when the scheme has no pickable class of that kind. `current` is the class the Option holds: when
/// it holds the vessel and is as small as the smallest class that does (equal maxima), it stays, so
/// picking a class and reading it back give the same class.
pub fn place(
    scheme: &str,
    kind: &str,
    loa_m: f64,
    beam_m: f64,
    current: Option<&str>,
) -> Option<Placement> {
    let list = ladder(scheme, kind);
    let largest = *list.last()?;
    let holds = |c: &ClassRow| {
        c.loa_max_m().is_some_and(|m| m >= loa_m) && c.beam_max_m().is_some_and(|m| m >= beam_m)
    };
    let Some(first) = list.iter().copied().find(|c| holds(c)) else {
        return Some(Placement {
            class: largest,
            beyond_largest: true,
        });
    };
    let keep = current.and_then(find).filter(|cur| {
        holds(cur)
            && cur.family == scheme
            && cur.loa_max_m() == first.loa_max_m()
            && cur.beam_max_m() == first.beam_max_m()
    });
    Some(Placement {
        class: keep.unwrap_or(first),
        beyond_largest: false,
    })
}
