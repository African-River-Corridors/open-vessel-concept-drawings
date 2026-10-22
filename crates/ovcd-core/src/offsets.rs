//! The typed offsets table (spec 0008, "Offsets table"): one row per point, `station_x_m, y_m,
//! z_m`, the points of each station from the keel on the centre line round to the deck on the
//! centre line, stations in ascending x. [`from_rows`] reads it into a Sections Table and
//! [`to_rows`] writes one out, so a table survives the round trip.
//!
//! A station is the run of rows that share one x. A table whose stations differ in point count is
//! resampled: each station is re-cut to the largest count by arc length, with its own points kept,
//! and [`import`] says so.

use crate::sections::{Section, SectionsError, SectionsTable};
use alloc::vec::Vec;
use core::fmt;
use libm::sqrt;

/// One point of the offsets table (m).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OffsetRow {
    pub station_x_m: f64,
    pub y_m: f64,
    pub z_m: f64,
}

/// Why an offsets table is refused. Stations are counted from 0, aft to forward.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OffsetsError {
    /// A value is not a finite number (the row, from 0).
    NotFinite { row: usize },
    /// Fewer than two stations.
    TooFewStations { count: usize },
    /// A station lies aft of the one before it.
    XDescending { station: usize },
    /// A station has fewer than two points.
    TooFewPoints { station: usize },
    /// A station does not start and end on the centre line (y = 0).
    OffCentreLine { station: usize },
    /// A point lies to port of the centre line.
    NegativeY { station: usize },
    /// Two edges of a station cross.
    SelfCrossing { station: usize },
}

impl OffsetsError {
    /// The message, with its Message Id (stations and rows counted from 1).
    pub fn text(&self) -> crate::text::Text {
        use crate::text;
        match *self {
            OffsetsError::NotFinite { row } => text!(
                "msg-offsets-not-finite",
                "row {row} of the offsets table holds a value that is not a number",
                row = row + 1
            ),
            OffsetsError::TooFewStations { count: n } => text!(
                "msg-offsets-too-few-stations",
                "the offsets table needs at least 2 stations, not {n}",
                n = n
            ),
            OffsetsError::XDescending { station } => text!(
                "msg-offsets-x-descending",
                "station {station} of the offsets table lies aft of the station before it; give the stations in ascending x",
                station = station + 1
            ),
            OffsetsError::TooFewPoints { station } => text!(
                "msg-offsets-too-few-points",
                "station {station} of the offsets table needs at least 2 points",
                station = station + 1
            ),
            OffsetsError::OffCentreLine { station } => text!(
                "msg-offsets-off-centre-line",
                "station {station} of the offsets table does not start and end on the centre line (y = 0)",
                station = station + 1
            ),
            OffsetsError::NegativeY { station } => text!(
                "msg-offsets-negative-y",
                "station {station} of the offsets table has a point with y below 0",
                station = station + 1
            ),
            OffsetsError::SelfCrossing { station } => text!(
                "msg-offsets-self-crossing",
                "station {station} of the offsets table crosses itself",
                station = station + 1
            ),
        }
    }
}

impl fmt::Display for OffsetsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

/// A table read by [`import`]: the Sections Table, and the stations it resampled (empty when every
/// station had the same point count).
#[derive(Debug, Clone, PartialEq)]
pub struct Imported {
    pub table: SectionsTable,
    /// The stations re-cut to the largest point count, and that count.
    pub resampled: Vec<usize>,
    pub points: usize,
}

/// The main dimensions an offsets hull gives (spec 0008): L the x range, B twice the largest y, D
/// the lowest deck edge height.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dimensions {
    pub length_m: f64,
    pub beam_m: f64,
    pub depth_m: f64,
}

/// Read an offsets table into a Sections Table (see [`import`] for the resampling it may do).
pub fn from_rows(rows: &[OffsetRow]) -> Result<SectionsTable, OffsetsError> {
    import(rows).map(|i| i.table)
}

/// Read an offsets table: group the rows into stations, check them, resample unequal stations,
/// and build the Sections Table. The deck edge height of each station is the highest point at its
/// largest half-breadth.
pub fn import(rows: &[OffsetRow]) -> Result<Imported, OffsetsError> {
    if let Some(row) = rows
        .iter()
        .position(|r| !(r.station_x_m.is_finite() && r.y_m.is_finite() && r.z_m.is_finite()))
    {
        return Err(OffsetsError::NotFinite { row });
    }
    let mut stations: Vec<(f64, Vec<(f64, f64)>)> = Vec::new();
    for r in rows {
        match stations.last_mut() {
            Some((x, pts)) if *x == r.station_x_m => pts.push((r.y_m, r.z_m)),
            _ => stations.push((r.station_x_m, alloc::vec![(r.y_m, r.z_m)])),
        }
    }
    if stations.len() < 2 {
        return Err(OffsetsError::TooFewStations {
            count: stations.len(),
        });
    }
    for (i, (x, pts)) in stations.iter().enumerate() {
        if i > 0 && *x < stations[i - 1].0 {
            return Err(OffsetsError::XDescending { station: i });
        }
        if pts.len() < 2 {
            return Err(OffsetsError::TooFewPoints { station: i });
        }
        if pts.iter().any(|p| p.0 < 0.0) {
            return Err(OffsetsError::NegativeY { station: i });
        }
        if pts[0].0 != 0.0 || pts[pts.len() - 1].0 != 0.0 {
            return Err(OffsetsError::OffCentreLine { station: i });
        }
    }
    let n = stations.iter().map(|s| s.1.len()).max().unwrap_or(0);
    let reference = stations.iter().position(|s| s.1.len() == n).unwrap_or(0);
    let fractions = arc_fractions(&stations[reference].1);
    let mut resampled = Vec::new();
    let sections: Vec<Section> = stations
        .iter()
        .enumerate()
        .map(|(i, (x, pts))| {
            let points = if pts.len() == n {
                pts.clone()
            } else {
                resampled.push(i);
                resample(pts, &fractions)
            };
            Section { x: *x, points }
        })
        .collect();
    let depths = sections.iter().map(|s| deck_edge(&s.points)).collect();
    let table = SectionsTable::new(sections, depths).map_err(|e| match e {
        SectionsError::SelfCrossing { section } => OffsetsError::SelfCrossing { station: section },
        SectionsError::XDescending { section } => OffsetsError::XDescending { station: section },
        SectionsError::OffCentreLine { section } => {
            OffsetsError::OffCentreLine { station: section }
        }
        SectionsError::NegativeY { section, .. } => OffsetsError::NegativeY { station: section },
        SectionsError::TooFewPoints { section } => OffsetsError::TooFewPoints { station: section },
        SectionsError::TooFewSections { count } => OffsetsError::TooFewStations { count },
        _ => OffsetsError::NotFinite { row: 0 },
    })?;
    Ok(Imported {
        table,
        resampled,
        points: n,
    })
}

/// Write a Sections Table as offsets rows: every point of every Section, aft to forward.
pub fn to_rows(table: &SectionsTable) -> Vec<OffsetRow> {
    table
        .sections
        .iter()
        .flat_map(|s| {
            s.points.iter().map(move |&(y, z)| OffsetRow {
                station_x_m: s.x,
                y_m: y,
                z_m: z,
            })
        })
        .collect()
}

/// L, B and D of a Sections Table read as an offsets hull.
pub fn dimensions(table: &SectionsTable) -> Dimensions {
    let ss = &table.sections;
    let beam = ss
        .iter()
        .flat_map(|s| s.points.iter().map(|p| p.0))
        .fold(0.0, f64::max);
    let depth = table
        .depth_at_side
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    Dimensions {
        length_m: ss[ss.len() - 1].x - ss[0].x,
        beam_m: 2.0 * beam,
        depth_m: depth,
    }
}

/// The deck edge of a half-section: the highest point at its largest half-breadth.
fn deck_edge(pts: &[(f64, f64)]) -> f64 {
    let y_max = pts.iter().map(|p| p.0).fold(0.0, f64::max);
    pts.iter()
        .filter(|p| p.0 == y_max)
        .map(|p| p.1)
        .fold(f64::NEG_INFINITY, f64::max)
}

/// The arc length to each point as a fraction of the whole outline (0 at the keel, 1 at the deck);
/// an outline of no length takes even steps.
fn arc_fractions(pts: &[(f64, f64)]) -> Vec<f64> {
    let mut s = alloc::vec![0.0; pts.len()];
    for i in 1..pts.len() {
        let (a, b) = (pts[i - 1], pts[i]);
        s[i] = s[i - 1] + sqrt((b.0 - a.0) * (b.0 - a.0) + (b.1 - a.1) * (b.1 - a.1));
    }
    let total = s[pts.len() - 1];
    let last = (pts.len() - 1) as f64;
    s.iter()
        .enumerate()
        .map(|(i, &v)| {
            if total > 0.0 {
                v / total
            } else {
                i as f64 / last
            }
        })
        .collect()
}

/// Re-cut a station of fewer points to the reference's `fractions`. Each of its own points takes the
/// reference point nearest in arc fraction (in order, keel and deck at the ends), so its corners
/// stay; between two kept points the new ones follow the reference's spacing along the outline.
fn resample(pts: &[(f64, f64)], fractions: &[f64]) -> Vec<(f64, f64)> {
    let (m, n) = (pts.len(), fractions.len());
    let own = arc_fractions(pts);
    // The reference index each kept point takes: increasing, with room for the points after it.
    let mut at = alloc::vec![0usize; m];
    for k in 1..m {
        let lo = at[k - 1] + 1;
        let hi = n - (m - k);
        let nearest = (lo..=hi)
            .min_by(|&a, &b| {
                (fractions[a] - own[k])
                    .abs()
                    .total_cmp(&(fractions[b] - own[k]).abs())
            })
            .unwrap_or(lo);
        at[k] = if k == m - 1 { n - 1 } else { nearest };
    }
    let mut out = Vec::with_capacity(n);
    for k in 0..m - 1 {
        let (a, b) = (pts[k], pts[k + 1]);
        let (fa, fb) = (fractions[at[k]], fractions[at[k + 1]]);
        let span = (at[k + 1] - at[k]) as f64;
        for (step, &f) in fractions[at[k]..at[k + 1]].iter().enumerate() {
            let u = if fb > fa {
                (f - fa) / (fb - fa)
            } else {
                step as f64 / span
            };
            out.push((a.0 + u * (b.0 - a.0), a.1 + u * (b.1 - a.1)));
        }
    }
    out.push(pts[m - 1]);
    out
}
