//! The GA view builder (spec 0003): one Option's results to three drawing documents.
//!
//! The documents follow the Drawing Format Standard (JSON). This module needs nothing from the
//! drawing crate: it writes JSON, and the standard's own checks decide whether the documents are
//! valid. It reads the results that `simulate` gives and the core's own geometry (lengths, voids,
//! rake, hold and tank sizes). It calculates no capacity, weight or draught, and it never changes a
//! result.
//!
//! Each document is one A3 landscape sheet with one viewport, a bottom strip of notes, the status
//! `concept` and no title block. The window of the viewport is the extent of everything drawn, with
//! room for dimensions and labels, and the scale is `fit`: the standard picks the largest scale in
//! its list at which the window fits the frame. Dimensions are computed by the standard from the
//! geometry they point at, so the numbers on a drawing are the numbers in the results.
//!
//! What a view shows:
//! - Side elevation: hull outline with the bow rake, deck line, design waterline, light waterline,
//!   the hold or the tanks in dashed hidden lines, dimensions LOA and depth, labels for draught and
//!   freeboard.
//! - Plan: hull outline, the hold opening or the tank outlines, centre line, dimensions LOA and beam.
//! - Midship section: the hold (box or sloped hopper) or the tank circles with side and bottom
//!   clearance, waterlines, dimensions beam, depth, draught and freeboard.

use crate::run::RunOutcome;
use ovcd_core::cargo::CargoClass;
use ovcd_core::models::BargeInputs;
use ovcd_core::pyfmt::fixed;
use serde_json::{Value, json};

/// A drawing label with its Message Id in `locales/en/drawing.ftl` (spec 0005). Drawings stay
/// English for now: the Drawing Format Standard allows printable ASCII only (clause 11.6.2), so
/// only the English is used.
macro_rules! label {
    ($id:literal, $($rest:tt)*) => {
        ovcd_core::text!($id, $($rest)*).en
    };
}

/// The three views of one Option, as drawing documents.
#[derive(Debug, Clone, PartialEq)]
pub struct GaViews {
    pub side_elevation: Value,
    pub plan: Value,
    pub midship_section: Value,
}

impl GaViews {
    /// The views in order, with the file-name stem of each.
    pub fn all(&self) -> [(&'static str, &Value); 3] {
        [
            ("side-elevation", &self.side_elevation),
            ("plan", &self.plan),
            ("midship-section", &self.midship_section),
        ]
    }
}

/// The views of an Option that was run. A convoy, or an Option that cannot be computed, gives no
/// views and the same message as the outputs panel.
pub fn ga_views_of(outcome: &RunOutcome) -> Result<GaViews, String> {
    match outcome {
        RunOutcome::Results { results, .. } => ga_views(results),
        RunOutcome::NotAvailable(message) | RunOutcome::Failed(message) => Err(message.clone()),
    }
}

// ---- the sheet ---------------------------------------------------------------------------------

/// The drawing space of A3 landscape (clause 6.3), in millimetres.
const SHEET: (f64, f64, f64, f64) = (20.0, 10.0, 390.0, 277.0);
/// The height of the notes strip at the bottom of the sheet: the heading, two notes and the status
/// line on its last line (clauses 14.9 and 14.10.1).
const STRIP_MM: f64 = 21.0;
/// The pitch of the notes in a strip (clause 14.10.1).
const NOTE_PITCH_MM: f64 = 4.5;
/// The most characters of a note: the schema allows 120. 120 capitals at height 2.5 are well under
/// the width of the strip (390 mm).
const NOTE_MAX_CHARS: usize = 120;
/// The text height of a note (clause 14.10.1), in millimetres.
const NOTE_TEXT_MM: f64 = 2.5;
/// The ratios of the standard (clauses 7.3.1 and 7.3.4), smallest denominator first.
const RATIOS: [u32; 20] = [
    1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500, 1000, 1250, 2000, 2500, 5000, 10000, 20000, 50000,
    100000,
];
/// Paper distance of the dimension line from the geometry, in millimetres.
const DIM_OFFSET_MM: f64 = 12.0;
/// Room that a dimension or label needs beyond the geometry on its side, in millimetres.
const DIM_ROOM_MM: f64 = 6.0;
/// Spacing of stacked labels, in millimetres: the text height and a gap of 2.5 mm.
const LABEL_STEP_MM: f64 = 6.0;
/// The text height of a leader label (clause 12.6), in millimetres.
const LABEL_TEXT_MM: f64 = 3.5;
/// The gap from the label point to the text of a leader (half a unit, clause 12.6).
const LEADER_TEXT_GAP_MM: f64 = 0.9;
/// Clear room kept round every label text in the window, in millimetres. With the window's own 3 mm
/// it keeps a label 6.5 mm inside the frame, clear of the centring marks, which reach 5 mm into the
/// drawing space (clause 6.4) wherever a label stands along the side.
const LABEL_CLEAR_MM: f64 = 3.5;
/// The most passes of the swap that uncrosses the leaders of a column.
const UNCROSS_PASSES: usize = 100;
/// Room kept under the drawing for the scale bar, in millimetres.
const BAR_ROOM_MM: f64 = 26.0;
/// The shortest dimension line that has room for its arrowheads, in millimetres (clause 12.4.3 asks
/// for two arrow lengths, 7.2 mm on A3).
const MIN_DIM_MM: f64 = 8.0;

// Drawing proportions. The core has no value for these: they only set how a detail is drawn, as the
// earlier Python GA drawer did, and are not engineering data.
/// A tank cradle wraps this many degrees of the tank's underside.
const CRADLE_WRAP_DEG: f64 = 120.0;
/// A cradle stands this far outside the tank envelope, in metres.
const CRADLE_CLEARANCE_M: f64 = 0.05;
/// A thruster is drawn as a circle: in elevation a fraction of the thruster room length (at most
/// 1.1 m) standing 0.2 m above the keel, in plan a fraction (at most 1.3 m).
const THRUSTER_R_ELEVATION_OF_ROOM: f64 = 0.30;
const THRUSTER_R_ELEVATION_MAX_M: f64 = 1.1;
const THRUSTER_LIFT_M: f64 = 0.2;
const THRUSTER_R_PLAN_OF_ROOM: f64 = 0.32;
const THRUSTER_R_PLAN_MAX_M: f64 = 1.3;

// ---- the geometry that the views draw ------------------------------------------------------------

/// What the results give about the hull and its cargo containment.
struct Hull {
    vessel: String,
    cargo: String,
    loa: f64,
    beam: f64,
    depth: f64,
    draught: f64,
    light_draught: Option<f64>,
    bow_rake: f64,
    stern_rake: f64,
    bow_void: f64,
    stern_void: f64,
    cargo_zone: f64,
    /// The gas separation forward of the stern block (`drawing.separation`): the cofferdam and the
    /// aft separation, whose end is the start of the cargo zone. `None` for dry bulk.
    separation: Option<(f64, f64)>,
    /// The aft block of a self-propelled hull (`drawing.aft_arrangement`); `None` for a pushed
    /// barge. Its x is measured from the stern, as the hull's x is.
    aft: Option<Aft>,
    /// The names of the checks that fail (`results.checks` with `ok` false), as plain capitals.
    failed_checks: Vec<String>,
    containment: Containment,
}

/// The values of the core's aft arrangement that the views draw.
struct Aft {
    zones: Vec<(String, f64, f64)>,
    thruster_offsets: Vec<f64>,
    thruster_room: f64,
    container_x0: f64,
    container_width: f64,
    container_top: f64,
    container_labels: Vec<String>,
    unit_width: f64,
    side_gap: f64,
    rows: usize,
    per_row: usize,
    row_length: f64,
    row_gap: f64,
    wheelhouse_x0: f64,
    wheelhouse_x1: f64,
    wheelhouse_width: f64,
    wheelhouse_floor: f64,
    wheelhouse_top: f64,
    block_top: f64,
}

/// The names of the failed error checks, in the plain capitals a note allows: an em dash becomes a hyphen,
/// an underscore a space, and any other character outside printable ASCII is dropped.
fn failed_checks(results: &Value) -> Vec<String> {
    let Some(checks) = results.get("checks").and_then(Value::as_array) else {
        return Vec::new();
    };
    checks
        .iter()
        // A failed warning does not make the design fail (issue #418); no severity is an error.
        .filter(|c| {
            c.get("ok").and_then(Value::as_bool) == Some(false)
                && c.get("severity").and_then(Value::as_str) != Some("warning")
        })
        .map(|c| {
            let name = c.get("name").and_then(Value::as_str).unwrap_or("CHECK");
            let plain: String = name
                .chars()
                .filter_map(|ch| match ch {
                    '\u{2014}' | '\u{2013}' | '\u{2212}' => Some('-'),
                    '_' => Some(' '),
                    ch if (' '..='~').contains(&ch) => Some(ch.to_ascii_uppercase()),
                    _ => None,
                })
                .collect();
            plain.split_whitespace().collect::<Vec<_>>().join(" ")
        })
        .collect()
}

fn read_aft(results: &Value) -> Option<Aft> {
    let a = results.get("drawing")?.get("aft_arrangement")?;
    if !a.is_object() {
        return None;
    }
    let n = |key: &str| a.get(key).and_then(Value::as_f64);
    let list = |key: &str| -> Option<Vec<f64>> {
        a.get(key)?.as_array()?.iter().map(Value::as_f64).collect()
    };
    let zones = a
        .get("zones")?
        .as_array()?
        .iter()
        .map(|z| {
            Some((
                z.get(0)?.as_str()?.to_string(),
                z.get(1)?.as_f64()?,
                z.get(2)?.as_f64()?,
            ))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Aft {
        zones,
        thruster_offsets: list("thruster_offsets_m")?,
        thruster_room: n("thruster_room_length_m")?,
        container_x0: n("container_x0_m")?,
        container_width: n("container_total_width_m")?,
        container_top: n("container_top_above_keel_m")?,
        container_labels: a
            .get("container_labels")?
            .as_array()?
            .iter()
            .map(|t| t.as_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()?,
        unit_width: n("container_unit_width_m")?,
        side_gap: n("container_side_gap_m")?,
        rows: n("container_rows")? as usize,
        per_row: n("container_per_row")? as usize,
        row_length: n("container_row_length_m")?,
        row_gap: n("container_row_gap_m")?,
        wheelhouse_x0: n("wheelhouse_x0_m")?,
        wheelhouse_x1: n("wheelhouse_x1_m")?,
        wheelhouse_width: n("wheelhouse_width_m")?,
        wheelhouse_floor: n("wheelhouse_floor_lowered_m")?,
        wheelhouse_top: n("wheelhouse_top_lowered_m")?,
        block_top: n("block_top_lowered_m")?,
    })
}

enum Containment {
    /// A box hold or a sloped hopper hold.
    Hold {
        top_width: f64,
        bottom_width: f64,
        /// The hold floor: the inner bottom, in metres above the keel (`drawing.double_bottom_m`).
        /// The core's `hold_height_m` runs from here to the coaming top, so `depth - hold_height`
        /// is not the floor.
        inner_bottom: f64,
        /// The hatch coaming above the deck (`drawing.hatch_coaming_height_m`).
        coaming: f64,
    },
    /// Type C tanks: horizontal cylinders with hemispherical heads.
    Tanks {
        od: f64,
        n_wide: usize,
        n_long: usize,
        side_clearance: f64,
        bottom_clearance: f64,
        /// `drawing.double_bottom_m` and `drawing.tank_support_height_m`: the tank bottom is their
        /// sum above the keel (the core's `h_top` is this + diameter + dome).
        double_bottom: f64,
        support_height: f64,
        /// `drawing.dome_height_m`, if the results have it.
        dome: Option<f64>,
        /// `drawing.tank.dome_diameter_m` (issue #316); the results must give it when there is a
        /// dome (issue #406).
        dome_diameter: Option<f64>,
        /// The head length over the tank radius (issue #375): 1 for a hemispherical head, 0.5 for a
        /// 2:1 semi-ellipsoidal one (`drawing.tank.head_type`).
        head_ratio: f64,
        /// The wing ballast tank: width per side, gap to the cargo tank and height as a fraction of
        /// the tank diameter (`drawing`), if the results have them.
        wing_width: Option<f64>,
        wing_gap: Option<f64>,
        wing_ratio: Option<f64>,
        gap_long: f64,
        gap_across: f64,
    },
}

fn num_at(v: &Value, path: &[&str]) -> Option<f64> {
    let mut cur = v;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_f64()
}

fn need(v: &Value, path: &[&str]) -> Result<f64, String> {
    num_at(v, path).ok_or_else(|| format!("the results have no `{}`", path.join(".")))
}

fn text_at(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

fn read_hull(results: &Value) -> Result<Hull, String> {
    let loa = need(results, &["dims", "loa_m"])?;
    let beam = need(results, &["dims", "beam_m"])?;
    let depth = need(results, &["dims", "depth_m"])?;
    let draught = need(results, &["results", "draught_m"])?;
    let geo = |key: &str| need(results, &["geometry", key]);
    let phase = text_at(results, "phase");
    let class = CargoClass::from_key(&phase)
        .map_err(|_| format!("the results give an unknown Cargo Class `{phase}`"))?;
    let containment = match class {
        CargoClass::Gas => {
            let defaults = BargeInputs::new(loa, beam, depth, draught, 0.0);
            let arrangement = |key: &str| need(results, &["arrangement", key]);
            Containment::Tanks {
                od: geo("tank_outer_diameter_m")?,
                n_wide: arrangement("n_wide")? as usize,
                n_long: arrangement("n_long")? as usize,
                side_clearance: geo("side_clearance_m")?,
                bottom_clearance: geo("bottom_clearance_m")?,
                double_bottom: need(results, &["drawing", "double_bottom_m"])?,
                support_height: need(results, &["drawing", "tank_support_height_m"])?,
                dome: num_at(results, &["drawing", "dome_height_m"]),
                // A dome is drawn at the core's diameter in force (issue #316); with a dome, the
                // results must give it (issue #406: no drawing proportion stands in for it).
                dome_diameter: match num_at(results, &["drawing", "dome_height_m"]) {
                    Some(h) if h > 0.0 => {
                        Some(need(results, &["drawing", "tank", "dome_diameter_m"])?)
                    }
                    _ => None,
                },
                head_ratio: match results["drawing"]["tank"]["head_type"].as_str() {
                    Some("ellipsoidal_2to1") => 0.5,
                    _ => 1.0,
                },
                wing_width: num_at(results, &["drawing", "wing_tank_width_m"]),
                wing_gap: num_at(results, &["drawing", "wing_tank_gap_m"]),
                wing_ratio: num_at(results, &["drawing", "wing_tank_height_ratio"]),
                gap_long: defaults.tank_gap_longitudinal,
                gap_across: defaults.tank_gap_transverse,
            }
        }
        CargoClass::DryBulk => {
            let drawing = |key: &str| need(results, &["drawing", key]);
            Containment::Hold {
                top_width: geo("hold_top_width_m")?,
                bottom_width: geo("hold_bottom_width_m")?,
                inner_bottom: drawing("double_bottom_m")?,
                coaming: drawing("hatch_coaming_height_m")?,
            }
        }
        CargoClass::LiquidBulk => {
            return Err("a liquid bulk Cargo has no drawing yet".to_string());
        }
    };
    for (name, value) in [
        ("loa", loa),
        ("beam", beam),
        ("depth", depth),
        ("draught", draught),
    ] {
        if !(value.is_finite() && value > 0.0) {
            return Err(format!(
                "the results give {name} = {value}; a view needs a positive length"
            ));
        }
    }
    Ok(Hull {
        vessel: text_at(results, "vessel"),
        cargo: text_at(results, "cargo"),
        loa,
        beam,
        depth,
        draught,
        light_draught: num_at(results, &["results", "light_draught_m"])
            .filter(|t| *t > 0.0 && *t < depth),
        bow_rake: num_at(results, &["geometry", "bow_rake_length_m"])
            .unwrap_or(0.0)
            .max(0.0),
        stern_rake: num_at(results, &["geometry", "stern_rake_length_m"])
            .unwrap_or(0.0)
            .max(0.0),
        bow_void: geo("bow_void_length_m")?,
        stern_void: geo("stern_void_length_m")?,
        cargo_zone: geo("cargo_zone_length_m")?,
        separation: num_at(results, &["drawing", "separation", "cofferdam_m"]).zip(num_at(
            results,
            &["drawing", "separation", "aft_separation_m"],
        )),
        aft: read_aft(results),
        failed_checks: failed_checks(results),
        containment,
    })
}

impl Hull {
    fn freeboard(&self) -> f64 {
        self.depth - self.draught
    }

    /// The bottom of the hull at length `x` from the stern, in metres above the keel: flat, rising
    /// along the bow rake and the stern rake (the core's rake: a straight line up to the depth).
    fn x_at_height(&self, y: f64) -> f64 {
        // the bow end: the keel is flat up to `loa - bow_rake`, then rises to `depth` at `loa`
        if self.bow_rake > 0.0 {
            (self.loa - self.bow_rake) + self.bow_rake * (y / self.depth)
        } else {
            self.loa
        }
    }

    /// The stern end of the hull at height `y` above the keel: square, or raked up to the depth.
    fn stern_x_at_height(&self, y: f64) -> f64 {
        if self.stern_rake > 0.0 {
            self.stern_rake * (1.0 - y / self.depth)
        } else {
            0.0
        }
    }

    /// The tank bottom above the keel: double bottom plus support height, as the core stands the
    /// tank. `None` for a hold.
    fn tank_bottom(&self) -> Option<f64> {
        match &self.containment {
            Containment::Tanks {
                double_bottom,
                support_height,
                ..
            } => Some(double_bottom + support_height),
            Containment::Hold { .. } => None,
        }
    }

    /// The tank centre height above the keel, for the section and the elevation alike.
    fn tank_centre_y(&self) -> Option<f64> {
        match &self.containment {
            Containment::Tanks { od, .. } => Some(self.tank_bottom()? + od / 2.0),
            Containment::Hold { .. } => None,
        }
    }

    /// The wing ballast tank: (width per side, gap to the cargo tank, top). The top is `ratio` of
    /// the tank diameter above the tank bottom, kept under the deck. `None` without a width, or
    /// when the width does not fit in the side clearance. The core takes the wing tank width from
    /// the default 1 m side clearance and does not work it out again once the clearance is known,
    /// so width and gap can add up to more than the clearance: the gap drawn is then cut to the
    /// room that is left, so that no line enters a tank.
    fn wing_tank(&self) -> Option<(f64, f64, f64)> {
        match &self.containment {
            Containment::Tanks {
                od,
                side_clearance,
                wing_width: Some(w),
                wing_gap,
                wing_ratio: Some(ratio),
                ..
            } if *w > 0.0 && *w < *side_clearance => {
                // The wing plate level, as the weights carry it: not clamped at the deck (issue
                // #407). Above the Moulded Depth the side shell runs up to it.
                let top = self.tank_bottom()? + ratio * od;
                let gap = wing_gap.unwrap_or(0.0).max(0.0).min(side_clearance - w);
                Some((*w, gap, top))
            }
            _ => None,
        }
    }

    /// The double bottom of a gas hull (zero: none).
    fn gas_double_bottom(&self) -> f64 {
        match &self.containment {
            Containment::Tanks { double_bottom, .. } => *double_bottom,
            Containment::Hold { .. } => 0.0,
        }
    }

    /// The dome: its rise above the tank top and half its base, from the core's dome height and
    /// diameter in force (issue #316); `None` with no dome.
    fn dome(&self) -> Option<(f64, f64)> {
        match &self.containment {
            Containment::Tanks {
                dome: Some(rise),
                dome_diameter: Some(d),
                ..
            } if *rise > 0.0 && *d > 0.0 => Some((*rise, d / 2.0)),
            _ => None,
        }
    }

    /// The head length over the tank radius: 1 for hemispherical heads, 0.5 for 2:1 ones.
    fn head_ratio(&self) -> f64 {
        match &self.containment {
            Containment::Tanks { head_ratio, .. } => *head_ratio,
            _ => 1.0,
        }
    }

    /// The tanks along and across: (length of one tank overall, the x of each tank's stern end,
    /// the athwartships centre of each tank). The layout is the core's (`tanks.rs`): the cargo zone
    /// shared between the tanks with the longitudinal gap, the beam less the side clearances shared
    /// with the transverse gap.
    fn tank_layout(&self) -> Option<(f64, Vec<f64>, Vec<f64>)> {
        let Containment::Tanks {
            od,
            n_wide,
            n_long,
            side_clearance,
            gap_long,
            gap_across,
            ..
        } = &self.containment
        else {
            return None;
        };
        let (n_wide, n_long) = (*n_wide, *n_long);
        if n_wide == 0 || n_long == 0 {
            return None;
        }
        let length = (self.cargo_zone - (n_long as f64 - 1.0) * gap_long) / n_long as f64;
        let starts = (0..n_long)
            .map(|i| self.stern_void + i as f64 * (length + gap_long))
            .collect();
        let available = self.beam - 2.0 * side_clearance;
        let centres = (0..n_wide)
            .map(|j| -available / 2.0 + od / 2.0 + j as f64 * (od + gap_across))
            .collect();
        Some((length, starts, centres))
    }
}

// ---- a canvas of entities in model metres, with the extent of what was drawn -----------------------

struct Canvas {
    /// Metres of model space for one millimetre of paper.
    k: f64,
    entities: Vec<Value>,
    min: [f64; 2],
    max: [f64; 2],
}

fn pt(p: [f64; 2]) -> Value {
    json!([p[0], p[1]])
}

impl Canvas {
    fn new(denominator: u32) -> Canvas {
        Canvas {
            k: f64::from(denominator) / 1000.0,
            entities: Vec::new(),
            min: [f64::INFINITY; 2],
            max: [f64::NEG_INFINITY; 2],
        }
    }

    fn extend(&mut self, p: [f64; 2]) {
        self.min = [self.min[0].min(p[0]), self.min[1].min(p[1])];
        self.max = [self.max[0].max(p[0]), self.max[1].max(p[1])];
    }

    fn line(&mut self, layer: &str, a: [f64; 2], b: [f64; 2], line_type: Option<&str>) {
        if a == b {
            return;
        }
        self.extend(a);
        self.extend(b);
        let mut e = json!({"type": "line", "layer": layer, "from_m": pt(a), "to_m": pt(b)});
        if let Some(t) = line_type {
            e["line_type"] = json!(t);
        }
        self.entities.push(e);
    }

    fn polyline(&mut self, layer: &str, points: &[[f64; 2]], closed: bool) {
        let mut pts: Vec<[f64; 2]> = Vec::new();
        for p in points {
            if pts.last() != Some(p) {
                pts.push(*p);
            }
        }
        if closed && pts.len() > 1 && pts.first() == pts.last() {
            pts.pop();
        }
        if pts.len() < 2 {
            return;
        }
        for p in &pts {
            self.extend(*p);
        }
        self.entities.push(json!({"type": "polyline", "layer": layer, "points_m": pts.iter().map(|p| pt(*p)).collect::<Vec<_>>(), "closed": closed}));
    }

    fn circle(&mut self, layer: &str, c: [f64; 2], r: f64) {
        self.extend([c[0] - r, c[1] - r]);
        self.extend([c[0] + r, c[1] + r]);
        self.entities
            .push(json!({"type": "circle", "layer": layer, "centre_m": pt(c), "radius_m": r}));
    }

    /// An arc counter-clockwise from `start` to `end` degrees.
    // Drawing geometry, not a result: this only widens the drawing's extent, and no capacity, weight,
    // draught or hash depends on it. The banned-maths rule (clippy.toml) is lifted for this one
    // function; the standard-library `sin` and `cos` are fine here.
    #[allow(clippy::disallowed_methods)]
    fn arc(
        &mut self,
        layer: &str,
        c: [f64; 2],
        r: f64,
        start: f64,
        end: f64,
        line_type: Option<&str>,
    ) {
        let sweep = (end - start).rem_euclid(360.0);
        for a in [start, end].into_iter().chain(
            [0.0, 90.0, 180.0, 270.0]
                .into_iter()
                .filter(|a| (a - start).rem_euclid(360.0) <= sweep),
        ) {
            let rad = a.to_radians();
            self.extend([c[0] + r * rad.cos(), c[1] + r * rad.sin()]);
        }
        let mut e = json!({"type": "arc", "layer": layer, "centre_m": pt(c), "radius_m": r, "start_deg": start, "end_deg": end});
        if let Some(t) = line_type {
            e["line_type"] = json!(t);
        }
        self.entities.push(e);
    }

    /// A horizontal stadium (two lines and two half-circle ends) from `xa` to `xb` about height `yc`.
    fn stadium(&mut self, layer: &str, xa: f64, xb: f64, yc: f64, r: f64) {
        self.line(layer, [xa, yc + r], [xb, yc + r], None);
        self.line(layer, [xa, yc - r], [xb, yc - r], None);
        self.arc(layer, [xa, yc], r, 90.0, 270.0, None);
        self.arc(layer, [xb, yc], r, 270.0, 90.0, None);
    }

    /// A tank outline in elevation or plan from `xa` to `xb` (the head joints) about height `yc`, with
    /// heads `ratio x r` long (issue #375): the stadium for hemispherical heads (`ratio` 1), and
    /// half-ellipse ends drawn as polylines for 2:1 heads.
    fn tank_outline(&mut self, layer: &str, (xa, xb, yc, r): (f64, f64, f64, f64), ratio: f64) {
        if ratio == 1.0 {
            self.stadium(layer, xa, xb, yc, r);
            return;
        }
        self.line(layer, [xa, yc + r], [xb, yc + r], None);
        self.line(layer, [xa, yc - r], [xb, yc - r], None);
        for (x, side) in [(xa, -1.0), (xb, 1.0)] {
            let pts = half_ellipse(x, yc, ratio * r, r, side);
            self.polyline(layer, &pts, false);
        }
    }

    /// As [`Canvas::tank_outline`], split at height `level` like [`Canvas::stadium_split`].
    fn tank_outline_split(
        &mut self,
        layers: (&str, &str),
        (xa, xb, yc, r): (f64, f64, f64, f64),
        ratio: f64,
        level: f64,
    ) {
        if ratio == 1.0 {
            self.stadium_split(layers, (xa, xb, yc, r), level);
            return;
        }
        let (above, below) = layers;
        let layer_of = |y: f64| if y >= level { above } else { below };
        self.line(layer_of(yc + r), [xa, yc + r], [xb, yc + r], None);
        self.line(layer_of(yc - r), [xa, yc - r], [xb, yc - r], None);
        for (x, side) in [(xa, -1.0), (xb, 1.0)] {
            let pts = half_ellipse(x, yc, ratio * r, r, side);
            // runs of points on one side of the level, joined at the crossing
            let mut run: Vec<[f64; 2]> = vec![pts[0]];
            for w in pts.windows(2) {
                let (p, q) = (w[0], w[1]);
                if (p[1] >= level) != (q[1] >= level) {
                    let t = (level - p[1]) / (q[1] - p[1]);
                    let cross = [p[0] + t * (q[0] - p[0]), level];
                    run.push(cross);
                    self.polyline(layer_of(p[1]), &run, false);
                    run = vec![cross];
                }
                run.push(q);
            }
            let last = run[run.len() - 1];
            let first = run[0];
            self.polyline(layer_of((first[1] + last[1]) / 2.0), &run, false);
        }
    }

    /// The same stadium split at height `level`: the part at or above it on `above`, the part
    /// below on `below`. Lines lie wholly on one side; the end arcs are split at the angle where
    /// the circle meets `level`.
    fn stadium_split(
        &mut self,
        (above, below): (&str, &str),
        (xa, xb, yc, r): (f64, f64, f64, f64),
        level: f64,
    ) {
        let s = (level - yc) / r;
        let layer_of = |y: f64| if y >= level { above } else { below };
        self.line(layer_of(yc + r), [xa, yc + r], [xb, yc + r], None);
        self.line(layer_of(yc - r), [xa, yc - r], [xb, yc - r], None);
        if s >= 1.0 || s <= -1.0 {
            // the level is at or beyond a pole: the whole arc is on one side
            let layer = layer_of(yc);
            self.arc(layer, [xa, yc], r, 90.0, 270.0, None);
            self.arc(layer, [xb, yc], r, 270.0, 90.0, None);
            return;
        }
        // the angle (degrees, in -90..90) on the right half of a circle where y = level
        let a = libm::asin(s).to_degrees();
        // left end: above is 90 to 180 - a, below is 180 - a to 270
        self.arc(above, [xa, yc], r, 90.0, 180.0 - a, None);
        self.arc(below, [xa, yc], r, 180.0 - a, 270.0, None);
        // right end: below is 270 to a, above is a to 90
        self.arc(below, [xb, yc], r, 270.0, a, None);
        self.arc(above, [xb, yc], r, a, 90.0, None);
    }

    /// A hatch over a closed boundary.
    fn hatch(&mut self, layer: &str, boundary: &[[f64; 2]], pattern: &str) {
        for p in boundary {
            self.extend(*p);
        }
        self.entities.push(json!({"type": "hatch", "layer": layer, "boundary_m": boundary.iter().map(|p| pt(*p)).collect::<Vec<_>>(), "pattern": pattern}));
    }

    /// A dome: a quadratic curve from `(x - half, base)` to `(x + half, base)` whose apex is at
    /// `base + rise` (the control point is at twice the rise).
    fn dome(&mut self, layer: &str, x: f64, base: f64, half: f64, rise: f64) {
        let steps = 16;
        let pts: Vec<[f64; 2]> = (0..=steps)
            .map(|i| {
                let t = f64::from(i) / f64::from(steps);
                let u = 1.0 - t;
                [
                    u * u * (x - half) + 2.0 * u * t * x + t * t * (x + half),
                    (u * u + t * t) * base + 2.0 * u * t * (base + 2.0 * rise),
                ]
            })
            .collect();
        self.polyline(layer, &pts, false);
    }

    fn centre_line(&mut self, a: [f64; 2], b: [f64; 2]) {
        self.extend(a);
        self.extend(b);
        self.entities.push(
            json!({"type": "centre_line", "layer": "CENTRE", "from_m": pt(a), "to_m": pt(b)}),
        );
    }

    /// A horizontal dimension; a negative offset puts the line below the points.
    fn dim_horizontal(&mut self, from: [f64; 2], to: [f64; 2], offset_mm: f64) {
        self.extend(from);
        self.extend(to);
        let y = (from[1] + to[1]) / 2.0 + offset_mm * self.k;
        self.extend([from[0], y + offset_mm.signum() * DIM_ROOM_MM * self.k]);
        self.entities.push(json!({"type": "dimension", "layer": "DIMENSIONS", "kind": "linear", "orientation": "horizontal", "from": {"point_m": pt(from)}, "to": {"point_m": pt(to)}, "offset_mm": offset_mm}));
    }

    /// A vertical dimension; a positive offset puts the line to the left of the points.
    fn dim_vertical(&mut self, from: [f64; 2], to: [f64; 2], offset_mm: f64) {
        self.extend(from);
        self.extend(to);
        let x = (from[0] + to[0]) / 2.0 - offset_mm * self.k;
        self.extend([x - offset_mm.signum() * DIM_ROOM_MM * self.k, from[1]]);
        self.entities.push(json!({"type": "dimension", "layer": "DIMENSIONS", "kind": "linear", "orientation": "vertical", "from": {"point_m": pt(from)}, "to": {"point_m": pt(to)}, "offset_mm": offset_mm}));
    }

    /// Does a length have room for a dimension at this scale?
    fn dim_fits(&self, length_m: f64) -> bool {
        length_m / self.k >= MIN_DIM_MM
    }

    /// A horizontal dimension if the length has room for its arrowheads at this scale.
    fn try_dim_horizontal(&mut self, from: [f64; 2], to: [f64; 2], offset_mm: f64) -> bool {
        let fits = self.dim_fits((to[0] - from[0]).abs());
        if fits {
            self.dim_horizontal(from, to, offset_mm);
        }
        fits
    }

    /// A vertical dimension if the length has room for its arrowheads at this scale.
    fn try_dim_vertical(&mut self, from: [f64; 2], to: [f64; 2], offset_mm: f64) -> bool {
        let fits = self.dim_fits((to[1] - from[1]).abs());
        if fits {
            self.dim_vertical(from, to, offset_mm);
        }
        fits
    }

    /// A leader to `target` with its label at `label_at`; the text stands to the right of the label
    /// point when it is right of the target, to the left otherwise. The window takes in the text
    /// box of the standard (clause 11.7), not an estimate of it.
    fn leader(&mut self, text: &str, target: [f64; 2], label_at: [f64; 2]) {
        self.extend(target);
        let right = label_at[0] >= target[0];
        let reach = (LEADER_TEXT_GAP_MM + text_width_mm(text) + LABEL_CLEAR_MM) * self.k;
        let half = (LABEL_TEXT_MM / 2.0 + LABEL_CLEAR_MM) * self.k;
        self.extend([
            label_at[0] + if right { reach } else { -reach },
            label_at[1] + half,
        ]);
        self.extend([label_at[0], label_at[1] - half]);
        self.entities.push(json!({"type": "leader", "layer": "ANNO", "from": {"point_m": pt(target)}, "label_at_m": pt(label_at), "text": text}));
    }

    /// Labels in a column at `x`, with the text standing outside the column and the leaders running
    /// in to the targets. The labels keep their order by target height and a gap of one step from
    /// the real text height; each stands as near its target as the gap allows (the least-squares
    /// spread of `spread_slots`). Where two leaders still cross, the two labels swap places: a swap
    /// of two crossing leaders always makes their total length shorter, so the loop ends, and when
    /// it ends no two leaders of the column cross.
    fn label_column(&mut self, x: f64, labels: &mut [(String, [f64; 2])]) {
        labels.sort_by(|a, b| {
            b.1[1]
                .partial_cmp(&a.1[1])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let wanted: Vec<f64> = labels.iter().map(|l| l.1[1]).collect();
        let slots = spread_slots(&wanted, LABEL_STEP_MM * self.k);
        let mut order: Vec<usize> = (0..labels.len()).collect();
        // order[i] is the label that stands at slot i
        for _ in 0..UNCROSS_PASSES {
            let mut swapped = false;
            for i in 0..order.len() {
                for j in i + 1..order.len() {
                    let a = (labels[order[i]].1, [x, slots[i]]);
                    let b = (labels[order[j]].1, [x, slots[j]]);
                    if segments_cross(a, b) {
                        order.swap(i, j);
                        swapped = true;
                    }
                }
            }
            if !swapped {
                break;
            }
        }
        for (slot, &label) in order.iter().enumerate() {
            let (text, target) = &labels[label];
            self.leader(text, *target, [x, slots[slot]]);
        }
    }
}

/// The width of a label text on paper, in millimetres, from the width table of the standard
/// (clause 11.7.1). A character that the table lacks takes the widest capital's room.
fn text_width_mm(text: &str) -> f64 {
    ovcd_draw::rules::text_width_exact(text, LABEL_TEXT_MM)
        .unwrap_or(text.chars().count() as f64 * LABEL_TEXT_MM)
}

/// The heights of the label slots for targets at heights `wanted` (highest first): each slot is at
/// least `step` below the one above, and as near its target as that allows, in the least-squares
/// sense. Slot `i + 1` less slot `i` is at most `-step`, so with `z = slot + i * step` the `z` run
/// down, and the nearest such `z` come from pooling neighbours that break the order.
fn spread_slots(wanted: &[f64], step: f64) -> Vec<f64> {
    // blocks of (sum of z, count)
    let mut blocks: Vec<(f64, f64)> = Vec::new();
    for (i, w) in wanted.iter().enumerate() {
        blocks.push((w + i as f64 * step, 1.0));
        while blocks.len() > 1 {
            let n = blocks.len();
            let (last, before) = (blocks[n - 1], blocks[n - 2]);
            if last.0 / last.1 <= before.0 / before.1 {
                break;
            }
            blocks.truncate(n - 2);
            blocks.push((last.0 + before.0, last.1 + before.1));
        }
    }
    let mut z = Vec::with_capacity(wanted.len());
    for (sum, count) in blocks {
        for _ in 0..count as usize {
            z.push(sum / count);
        }
    }
    z.iter()
        .enumerate()
        .map(|(i, z)| z - i as f64 * step)
        .collect()
}

/// Do two leader segments (target first, label point second) cross, or touch other than at a common
/// target point?
fn segments_cross(a: ([f64; 2], [f64; 2]), b: ([f64; 2], [f64; 2])) -> bool {
    if (a.0[0] - b.0[0]).abs() < 1e-9 && (a.0[1] - b.0[1]).abs() < 1e-9 {
        return false;
    }
    let side = |o: [f64; 2], p: [f64; 2], q: [f64; 2]| {
        (p[0] - o[0]) * (q[1] - o[1]) - (p[1] - o[1]) * (q[0] - o[0])
    };
    let (d1, d2) = (side(b.0, b.1, a.0), side(b.0, b.1, a.1));
    let (d3, d4) = (side(a.0, a.1, b.0), side(a.0, a.1, b.1));
    d1 * d2 <= 0.0 && d3 * d4 <= 0.0
}

fn metres(x: f64) -> String {
    format!("{} m", fixed(x, 2))
}

// ---- the three views ---------------------------------------------------------------------------

/// The hull outline in profile: stern rake, keel, bow rake (open at the deck).
fn profile_points(h: &Hull) -> Vec<[f64; 2]> {
    let mut pts = vec![[0.0, h.depth]];
    pts.push(if h.stern_rake > 0.0 {
        [h.stern_rake, 0.0]
    } else {
        [0.0, 0.0]
    });
    if h.bow_rake > 0.0 {
        pts.push([h.loa - h.bow_rake, 0.0]);
        pts.push([h.loa, h.depth]);
    } else {
        pts.push([h.loa, 0.0]);
        pts.push([h.loa, h.depth]);
    }
    pts
}

/// The gas detail of the side elevation, from `drawing`: the double bottom and the wing ballast tanks
/// behind the tanks (hidden), and a dome on each tank. A dome is in view when the tank top is above
/// the deck, hidden when it is under it.
fn elevation_gas_detail(c: &mut Canvas, h: &Hull, labels: &mut Vec<(String, [f64; 2])>) {
    let (Some(yc), Containment::Tanks { od, .. }) = (h.tank_centre_y(), &h.containment) else {
        return;
    };
    let (x0, x1, db) = (h.stern_void, h.loa - h.bow_void, h.gas_double_bottom());
    if db > 0.0 {
        let x_fwd = h.x_at_height(db);
        c.line("HIDDEN", [h.stern_x_at_height(db), db], [x_fwd, db], None);
        labels.push((label!("draw-double-bottom", "DOUBLE BOTTOM"), [x_fwd, db]));
    }
    if let Some((_, _, top)) = h.wing_tank() {
        if top > h.depth {
            // The wing plate stands above the deck, so it is in view (issue #407).
            c.line("HIDDEN", [x0, db], [x0, h.depth], None);
            c.line("HIDDEN", [x1, h.depth], [x1, db], None);
            c.polyline(
                "OBJECT",
                &[[x0, h.depth], [x0, top], [x1, top], [x1, h.depth]],
                false,
            );
        } else {
            c.polyline("HIDDEN", &[[x0, db], [x0, top], [x1, top], [x1, db]], false);
        }
        labels.push((
            label!("draw-wbt-both-sides", "WBT (BOTH SIDES)"),
            [x1, (db + top) / 2.0],
        ));
    }
    if let (Some((rise, half_base)), Some((length, starts, _))) = (h.dome(), h.tank_layout()) {
        let r = od / 2.0;
        let layer = if yc + r >= h.depth {
            "OBJECT"
        } else {
            "HIDDEN"
        };
        for x in &starts {
            let mid = x + length / 2.0;
            let half = half_base.min(length / 2.0 - h.head_ratio() * r);
            if half > 0.0 {
                c.dome(layer, mid, yc + r, half, rise);
            }
        }
        // named at the last dome, nearest the label column
        if let Some(x) = starts.last() {
            labels.push((
                label!("draw-dome", "DOME"),
                [x + length / 2.0, yc + r + rise],
            ));
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum View {
    Elevation,
    Plan,
}

/// The longitudinal zones, in elevation and plan: the limits of the cargo zone (dashed ZONE lines,
/// not physical), the cofferdam of a gas hull (its two bulkheads, hidden under the deck) and the
/// forward equipment zone, each named by a leader (the forward zone's goes in the caller's column of labels).
fn zones(c: &mut Canvas, h: &Hull, view: View, labels: &mut Vec<(String, [f64; 2])>) {
    let (lo, hi) = match view {
        View::Elevation => (0.0, h.depth),
        View::Plan => (-h.beam / 2.0, h.beam / 2.0),
    };
    let across = |x: f64| ([x, lo], [x, hi]);
    let (x0, x1) = (h.stern_void, h.loa - h.bow_void);
    for x in [x0, x1] {
        if x > 0.0 && x < h.loa {
            let (a, b) = across(x);
            c.line("ZONE", a, b, None);
        }
    }
    if let Some((cofferdam, aft)) = h.separation
        && cofferdam > 0.0
    {
        let start = x0 - aft;
        for x in [start, start + cofferdam] {
            let (a, b) = across(x);
            c.line("HIDDEN", a, b, None);
        }
        let mid = start + cofferdam / 2.0;
        // The label stands under the hull, its text to the right of the label point, so that it
        // keeps clear of the aft block and the first tank above the deck, and of the thrusters'
        // label, whose text stands to the left of the stern.
        c.leader(
            &label!("draw-cofferdam", "COFFERDAM"),
            [mid, lo],
            [mid + 2.0 * c.k, lo - 6.0 * c.k],
        );
    }
    if h.bow_void > 0.0 {
        let mid = h.loa - h.bow_void / 2.0;
        // named in the column of the bow labels, with the others, so that the texts keep apart
        labels.push((label!("draw-fwd-equip", "FWD EQUIP."), [mid, hi]));
    }
}

/// The aft block of a self-propelled hull, from `drawing.aft_arrangement`; nothing for a pushed
/// barge. Layers: the units and thrusters are equipment (EQUIP); the wheelhouse is part of the
/// vessel (OBJECT, like the tanks); the zone bulkheads are below the deck (HIDDEN in elevation).
/// The wheelhouse is drawn lowered, as the air draft is measured; the raised position is not drawn,
/// because the standard's only chain line type already stands for the light waterline.
fn aft_block(c: &mut Canvas, h: &Hull, view: View) {
    let Some(a) = &h.aft else {
        return;
    };
    let k = c.k;
    let wh_mid = (a.wheelhouse_x0 + a.wheelhouse_x1) / 2.0;
    let room_mid = a.thruster_room / 2.0;
    let unit_row_mid = a.container_x0 + a.row_length / 2.0;
    match view {
        View::Elevation => {
            // zone bulkheads: the forward end of each zone, under the deck (not where the
            // cofferdam's first bulkhead is already)
            for (_, _, x_fwd) in &a.zones {
                let at_cofferdam = h
                    .separation
                    .is_some_and(|(_, aft)| (x_fwd - (h.stern_void - aft)).abs() < 1e-9);
                if !at_cofferdam {
                    c.line("HIDDEN", [*x_fwd, 0.0], [*x_fwd, h.depth], None);
                }
            }
            // the container units on deck: one box for each row (the units of a row are abreast)
            if a.container_top > h.depth {
                for row in 0..a.rows {
                    let x0 = a.container_x0 + row as f64 * (a.row_length + a.row_gap);
                    c.polyline(
                        "EQUIP",
                        &[
                            [x0, h.depth],
                            [x0 + a.row_length, h.depth],
                            [x0 + a.row_length, a.container_top],
                            [x0, a.container_top],
                        ],
                        true,
                    );
                }
            }
            // the wheelhouse, lowered
            c.polyline(
                "OBJECT",
                &[
                    [a.wheelhouse_x0, a.wheelhouse_floor],
                    [a.wheelhouse_x1, a.wheelhouse_floor],
                    [a.wheelhouse_x1, a.wheelhouse_top],
                    [a.wheelhouse_x0, a.wheelhouse_top],
                ],
                true,
            );
            // the thrusters, in the steering flat
            let r =
                (a.thruster_room * THRUSTER_R_ELEVATION_OF_ROOM).min(THRUSTER_R_ELEVATION_MAX_M);
            if !a.thruster_offsets.is_empty() && r > 0.0 {
                c.circle("EQUIP", [room_mid, r + THRUSTER_LIFT_M], r);
                c.leader(
                    &label!(
                        "draw-thrusters",
                        "THRUSTERS ({count})",
                        count = a.thruster_offsets.len()
                    ),
                    [room_mid, THRUSTER_LIFT_M],
                    [room_mid + 2.0 * k, -6.0 * k],
                );
            }
            // Labels above the block; the lower one is the one further from the stern end. The
            // block top, a height above the keel, is named on the label of the part that makes it.
            let top = a.block_top.max(h.depth);
            let block_top = metres(a.block_top);
            let (units, cab) = if (a.wheelhouse_top - a.block_top).abs() < 1e-9 {
                (
                    container_label(a),
                    label!(
                        "draw-wheelhouse-block-top",
                        "WHEELHOUSE, BLOCK TOP {height}",
                        height = block_top
                    ),
                )
            } else {
                (
                    label!(
                        "draw-containers-block-top",
                        "CONTAINERS ({abreast} x {rows}), BLOCK TOP {height}",
                        abreast = a.per_row,
                        rows = a.rows,
                        height = block_top
                    ),
                    label!("draw-wheelhouse", "WHEELHOUSE"),
                )
            };
            c.leader(
                &units,
                [unit_row_mid, a.container_top.max(h.depth)],
                [unit_row_mid - 5.0 * k, top + 6.0 * k],
            );
            c.leader(
                &cab,
                [wh_mid, a.wheelhouse_top],
                [wh_mid - 5.0 * k, top + 12.0 * k],
            );
        }
        View::Plan => {
            // the container units, from the arrangement: `per_row` abreast, `rows` along
            let per_row = a.per_row.max(1);
            let y0 = a.container_width / 2.0 - a.side_gap;
            for i in 0..a.container_labels.len() {
                let (row, col) = (i / per_row, i % per_row);
                let x0 = a.container_x0 + row as f64 * (a.row_length + a.row_gap);
                let yc = y0 - col as f64 * (a.unit_width + a.side_gap) - a.unit_width / 2.0;
                c.polyline(
                    "EQUIP",
                    &[
                        [x0, yc - a.unit_width / 2.0],
                        [x0 + a.row_length, yc - a.unit_width / 2.0],
                        [x0 + a.row_length, yc + a.unit_width / 2.0],
                        [x0, yc + a.unit_width / 2.0],
                    ],
                    true,
                );
            }
            // the wheelhouse footprint
            let hw = a.wheelhouse_width / 2.0;
            c.polyline(
                "OBJECT",
                &[
                    [a.wheelhouse_x0, -hw],
                    [a.wheelhouse_x1, -hw],
                    [a.wheelhouse_x1, hw],
                    [a.wheelhouse_x0, hw],
                ],
                true,
            );
            // the thrusters abreast in the steering flat
            let r = (a.thruster_room * THRUSTER_R_PLAN_OF_ROOM).min(THRUSTER_R_PLAN_MAX_M);
            if r > 0.0 {
                for off in &a.thruster_offsets {
                    c.circle("EQUIP", [room_mid, *off], r);
                }
                if let Some(low) = a.thruster_offsets.iter().copied().reduce(f64::min) {
                    c.leader(
                        &label!(
                            "draw-thrusters",
                            "THRUSTERS ({count})",
                            count = a.thruster_offsets.len()
                        ),
                        [room_mid, low - r],
                        [room_mid + 2.0 * k, -h.beam / 2.0 - 6.0 * k],
                    );
                }
            }
            let top = h.beam / 2.0;
            c.leader(
                &container_label(a),
                [unit_row_mid, y0],
                [unit_row_mid - 5.0 * k, top + 6.0 * k],
            );
            c.leader(
                &label!("draw-wheelhouse", "WHEELHOUSE"),
                [wh_mid, hw],
                [wh_mid - 5.0 * k, top + 12.0 * k],
            );
        }
    }
}

/// The label of the container units: how many abreast and how many rows, from the arrangement.
fn container_label(a: &Aft) -> String {
    label!(
        "draw-containers",
        "CONTAINERS ({abreast} x {rows})",
        abreast = a.per_row,
        rows = a.rows
    )
}

fn side_elevation(h: &Hull, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    c.polyline("OUTLINE", &profile_points(h), false);
    c.line("OUTLINE", [0.0, h.depth], [h.loa, h.depth], None);
    // waterlines
    c.line(
        "WATER",
        [h.stern_x_at_height(h.draught), h.draught],
        [h.x_at_height(h.draught), h.draught],
        None,
    );
    if let Some(t) = h.light_draught {
        c.line(
            "WATER",
            [h.stern_x_at_height(t), t],
            [h.x_at_height(t), t],
            Some("chain_double_dashed"),
        );
    }
    // the hold or the tanks, hidden under the deck
    match &h.containment {
        Containment::Hold {
            inner_bottom,
            coaming,
            ..
        } => {
            let (x0, x1) = (h.stern_void, h.loa - h.bow_void);
            // the hold's ends and sides under the deck; the floor is the inner bottom line below
            c.polyline(
                "HIDDEN",
                &[
                    [x0, *inner_bottom],
                    [x0, h.depth],
                    [x1, h.depth],
                    [x1, *inner_bottom],
                ],
                false,
            );
            // the inner bottom runs the length of the hull
            c.line(
                "HIDDEN",
                [h.stern_x_at_height(*inner_bottom), *inner_bottom],
                [h.x_at_height(*inner_bottom), *inner_bottom],
                None,
            );
            // the hatch coaming stands above the deck over the hold
            c.polyline(
                "OUTLINE",
                &[
                    [x0, h.depth],
                    [x0, h.depth + coaming],
                    [x1, h.depth + coaming],
                    [x1, h.depth],
                ],
                false,
            );
        }
        Containment::Tanks { od, .. } => {
            if let (Some((length, starts, _)), Some(yc)) = (h.tank_layout(), h.tank_centre_y()) {
                let r = od / 2.0;
                // The part of a tank above the deck is in plain view (OBJECT: a secondary object
                // line, the hull profile being the primary outline); the part below it is hidden.
                let head = h.head_ratio() * r;
                for x0 in starts {
                    let (xa, xb) = (x0 + head, x0 + length - head);
                    c.tank_outline_split(
                        ("OBJECT", "HIDDEN"),
                        (xa, xb, yc, r),
                        h.head_ratio(),
                        h.depth,
                    );
                }
            }
        }
    }
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
    elevation_gas_detail(&mut c, h, &mut labels);
    aft_block(&mut c, h, View::Elevation);
    zones(&mut c, h, View::Elevation, &mut labels);
    if !c.try_dim_horizontal([0.0, 0.0], [h.loa, 0.0], -DIM_OFFSET_MM) {
        labels.push((
            label!("draw-loa", "LOA {length}", length = metres(h.loa)),
            [h.loa, 0.0],
        ));
    }
    if !c.try_dim_vertical([0.0, 0.0], [0.0, h.depth], DIM_OFFSET_MM) {
        labels.push((
            label!("draw-depth", "DEPTH {depth}", depth = metres(h.depth)),
            [h.loa, h.depth],
        ));
    }
    // labels for draught and freeboard, to the right of the bow
    let mid_free = (h.draught + h.depth) / 2.0;
    labels.push((
        label!(
            "draw-freeboard",
            "FREEBOARD {freeboard}",
            freeboard = metres(h.freeboard())
        ),
        [h.x_at_height(mid_free), mid_free],
    ));
    labels.push((
        label!(
            "draw-draught",
            "DRAUGHT {draught}",
            draught = metres(h.draught)
        ),
        [h.x_at_height(h.draught), h.draught],
    ));
    if let Some(t) = h.light_draught {
        labels.push((
            label!(
                "draw-light-draught",
                "LIGHT DRAUGHT {draught}",
                draught = metres(t)
            ),
            [h.x_at_height(t), t],
        ));
    }
    c.label_column(h.loa + 6.0 * c.k, &mut labels);
    c
}

fn plan(h: &Hull, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    let (y0, y1) = (-h.beam / 2.0, h.beam / 2.0);
    c.polyline(
        "OUTLINE",
        &[[0.0, y0], [h.loa, y0], [h.loa, y1], [0.0, y1]],
        true,
    );
    if h.bow_rake > 0.0 {
        c.line(
            "OBJECT",
            [h.loa - h.bow_rake, y0],
            [h.loa - h.bow_rake, y1],
            None,
        );
    }
    if h.stern_rake > 0.0 {
        c.line("OBJECT", [h.stern_rake, y0], [h.stern_rake, y1], None);
    }
    match &h.containment {
        Containment::Hold {
            top_width,
            bottom_width,
            ..
        } => {
            let (x0, x1) = (h.stern_void, h.loa - h.bow_void);
            c.polyline(
                "OBJECT",
                &[
                    [x0, -top_width / 2.0],
                    [x1, -top_width / 2.0],
                    [x1, top_width / 2.0],
                    [x0, top_width / 2.0],
                ],
                true,
            );
            if (bottom_width - top_width).abs() > 1e-9 {
                c.polyline(
                    "HIDDEN",
                    &[
                        [x0, -bottom_width / 2.0],
                        [x1, -bottom_width / 2.0],
                        [x1, bottom_width / 2.0],
                        [x0, bottom_width / 2.0],
                    ],
                    true,
                );
            }
        }
        Containment::Tanks { od, .. } => {
            if let Some((length, starts, centres)) = h.tank_layout() {
                let r = od / 2.0;
                let tank_y = h.tank_centre_y().unwrap_or(0.0);
                let ratio = h.head_ratio();
                for x0 in &starts {
                    for yc in &centres {
                        let (xa, xb) = (x0 + ratio * r, x0 + length - ratio * r);
                        if tank_y >= h.depth {
                            // the widest part stands above the deck: the whole outline is in view
                            c.tank_outline("OBJECT", (xa, xb, *yc, r), ratio);
                        } else {
                            // The widest part is under the deck, so the outline is hidden. Where the
                            // tank's top stands above the deck, the part above is in view, and from
                            // above it is narrower: a sphere or cylinder cut at the deck height (a
                            // 2:1 head cut there is a smaller ellipse of the same shape).
                            c.tank_outline("HIDDEN", (xa, xb, *yc, r), ratio);
                            let dy = h.depth - tank_y;
                            if dy < r {
                                c.tank_outline("OBJECT", (xa, xb, *yc, chord_half(r, dy)), ratio);
                            }
                        }
                    }
                }
            }
        }
    }
    aft_block(&mut c, h, View::Plan);
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
    zones(&mut c, h, View::Plan, &mut labels);
    let overshoot = 4.0 * c.k;
    c.centre_line([-overshoot, 0.0], [h.loa + overshoot, 0.0]);
    if !c.try_dim_horizontal([0.0, y0], [h.loa, y0], -DIM_OFFSET_MM) {
        labels.push((
            label!("draw-loa", "LOA {length}", length = metres(h.loa)),
            [h.loa, y0],
        ));
    }
    if !c.try_dim_vertical([0.0, y0], [0.0, y1], DIM_OFFSET_MM) {
        labels.push((
            label!("draw-beam", "BEAM {beam}", beam = metres(h.beam)),
            [h.loa, y1],
        ));
    }
    let column = h.loa + 6.0 * c.k;
    c.label_column(column, &mut labels);
    c
}

/// Half the chord of a circle of radius `r` at distance `d` (under `r`) from its centre.
/// The points of a half ellipse from the top `(x, yc + ry)` to the bottom `(x, yc - ry)`, bulging
/// by `rx` towards `side` (-1 aft, +1 forward): the outline of a 2:1 head (issue #375).
fn half_ellipse(x: f64, yc: f64, rx: f64, ry: f64, side: f64) -> Vec<[f64; 2]> {
    let steps = 24;
    (0..=steps)
        .map(|i| {
            let a = core::f64::consts::PI * f64::from(i) / f64::from(steps);
            [x + side * rx * libm::sin(a), yc + ry * libm::cos(a)]
        })
        .collect()
}

fn chord_half(r: f64, d: f64) -> f64 {
    (r * r - d * d).sqrt()
}

/// The x of the left edge of a circle at height `y` (its centre if `y` is outside the circle).
fn circle_left_edge(xc: f64, yc: f64, r: f64, y: f64) -> f64 {
    let dy = y - yc;
    if dy.abs() >= r {
        xc
    } else {
        xc - (r * r - dy * dy).sqrt()
    }
}

/// The gas detail of the midship section, from `drawing`: the double bottom with its hatch, the wing
/// ballast tanks and the voids beside the cargo tanks, a cradle under each tank and the domes.
/// Anything the results leave out (`null`) is left out. The labels go to the left column and stand
/// for both sides.
fn section_gas_detail(
    c: &mut Canvas,
    h: &Hull,
    centres: &[f64],
    labels: &mut Vec<(String, [f64; 2])>,
) {
    let (Some(yc), Containment::Tanks { od, .. }) = (h.tank_centre_y(), &h.containment) else {
        return;
    };
    let (half, r, db) = (h.beam / 2.0, od / 2.0, h.gas_double_bottom());
    // double bottom: the inner bottom line across the hull, and a line hatch in the band below it
    if db > 0.0 {
        c.line("OBJECT", [-half, db], [half, db], None);
        c.hatch(
            "HATCH",
            &[[-half, 0.0], [half, 0.0], [half, db], [-half, db]],
            "lines",
        );
        labels.push((
            label!("draw-double-bottom", "DOUBLE BOTTOM"),
            [-half + 0.15 * half, db / 2.0],
        ));
    }
    // wing ballast tanks (the wall inboard and the top) and the void between each and the cargo tank
    if let Some((w, gap, top)) = h.wing_tank() {
        for side in [-1.0, 1.0] {
            let (outer, wall, edge) = (side * half, side * (half - w), side * (half - w - gap));
            c.polyline("OBJECT", &[[outer, top], [wall, top], [wall, db]], false);
            // Above the deck the side shell runs up to the wing plate (issue #407).
            if top > h.depth {
                c.line("OBJECT", [outer, h.depth], [outer, top], None);
            }
            if gap > 0.0 {
                c.polyline("ZONE", &[[wall, top], [edge, top], [edge, db]], false);
            }
        }
        let mid = (db + top) / 2.0;
        labels.push((
            label!("draw-wbt-both-sides", "WBT (BOTH SIDES)"),
            [-half + w / 2.0, mid],
        ));
        if gap > 0.0 {
            labels.push((
                label!("draw-void-both-sides", "VOID (BOTH SIDES)"),
                [-half + w + gap / 2.0, mid],
            ));
        }
    }
    // a cradle under each tank: the underside arc, a little outside the tank, and its two legs
    let radius = r + CRADLE_CLEARANCE_M;
    let (a0, a1) = (270.0 - CRADLE_WRAP_DEG / 2.0, 270.0 + CRADLE_WRAP_DEG / 2.0);
    for (i, x) in centres.iter().enumerate() {
        let on = |deg: f64| {
            [
                x + radius * libm::cos(deg.to_radians()),
                yc + radius * libm::sin(deg.to_radians()),
            ]
        };
        let (left, right) = (on(a0), on(a1));
        c.arc("OBJECT", [*x, yc], radius, a0, a1, None);
        c.line("OBJECT", left, [left[0], db], None);
        c.line("OBJECT", right, [right[0], db], None);
        if i == 0 {
            labels.push((
                label!("draw-cradle", "CRADLE"),
                [left[0], (left[1] + db) / 2.0],
            ));
        }
    }
    // a dome on top of each tank
    if let Some((rise, half_base)) = h.dome() {
        let half_base = half_base.min(0.95 * r);
        let base = yc + (r * r - half_base * half_base).sqrt();
        let sag = yc + r + rise - base;
        for (i, x) in centres.iter().enumerate() {
            c.dome("OBJECT", *x, base, half_base, sag);
            if i == 0 {
                labels.push((label!("draw-dome", "DOME"), [*x, base + sag]));
            }
        }
    }
}

/// The deck line of the section. A tank that stands through the deck cuts it: the line stops at the
/// tank circle and starts again past it, so no segment lies inside a circle. (Not done in the
/// elevation: there the deck edge is the near side of the hull and passes in front of the tank.)
fn section_deck_line(c: &mut Canvas, h: &Hull) {
    let half = h.beam / 2.0;
    let mut cuts: Vec<(f64, f64)> = Vec::new();
    if let (Containment::Tanks { od, .. }, Some(yc), Some((_, _, centres))) =
        (&h.containment, h.tank_centre_y(), h.tank_layout())
    {
        let r = od / 2.0;
        let dy = h.depth - yc;
        if dy.abs() < r {
            let w = (r * r - dy * dy).sqrt();
            cuts = centres.iter().map(|x| (x - w, x + w)).collect();
        }
    }
    let mut x = -half;
    for (a, b) in cuts {
        c.line("OUTLINE", [x, h.depth], [a.max(x), h.depth], None);
        x = x.max(b);
    }
    c.line("OUTLINE", [x, h.depth], [half, h.depth], None);
}

fn midship_section(h: &Hull, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    let half = h.beam / 2.0;
    c.polyline(
        "OUTLINE",
        &[[-half, h.depth], [-half, 0.0], [half, 0.0], [half, h.depth]],
        false,
    );
    section_deck_line(&mut c, h);
    c.line("WATER", [-half, h.draught], [half, h.draught], None);
    if let Some(t) = h.light_draught {
        c.line("WATER", [-half, t], [half, t], Some("chain_double_dashed"));
    }
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
    match &h.containment {
        Containment::Hold {
            top_width,
            bottom_width,
            inner_bottom,
            coaming,
        } => {
            let bottom = *inner_bottom;
            // the inner bottom across the hull, and the coaming above the deck on each side
            c.line("OUTLINE", [-half, bottom], [half, bottom], None);
            for side in [-1.0, 1.0] {
                c.line(
                    "OUTLINE",
                    [side * top_width / 2.0, h.depth],
                    [side * top_width / 2.0, h.depth + coaming],
                    None,
                );
            }
            labels.push((
                label!(
                    "draw-inner-bottom",
                    "INNER BOTTOM {height}",
                    height = metres(bottom)
                ),
                [-half, bottom],
            ));
            labels.push((
                label!(
                    "draw-coaming",
                    "COAMING {height}",
                    height = metres(*coaming)
                ),
                [-top_width / 2.0, h.depth + coaming],
            ));
            c.polyline(
                "OUTLINE",
                &[
                    [-top_width / 2.0, h.depth],
                    [-bottom_width / 2.0, bottom],
                    [bottom_width / 2.0, bottom],
                    [top_width / 2.0, h.depth],
                ],
                false,
            );
        }
        Containment::Tanks {
            od,
            side_clearance,
            bottom_clearance,
            ..
        } => {
            if let (Some((_, _, centres)), Some(yc), Some(bottom)) =
                (h.tank_layout(), h.tank_centre_y(), h.tank_bottom())
            {
                for x in &centres {
                    c.circle("OUTLINE", [*x, yc], od / 2.0);
                }
                section_gas_detail(&mut c, h, &centres, &mut labels);
                // Side clearance: a point in the gap between the hull side and the nearest tank,
                // at the tank's centre height, or just below the deck when the centre is above it.
                let y = yc.min(h.depth) - 1.5 * c.k;
                let edge = circle_left_edge(centres[0], yc, od / 2.0, y);
                labels.push((
                    label!(
                        "draw-side-clearance",
                        "SIDE CLEARANCE {clearance}",
                        clearance = metres(*side_clearance)
                    ),
                    [(-half + edge.max(-half)) / 2.0, y],
                ));
                // Bottom clearance: the gap between the keel and the tank bottom, under the
                // nearest tank. The core stands the tank on double bottom + support height, which
                // is not always the clearance it asks for; both are given when they differ.
                let text = if (bottom - bottom_clearance).abs() < 1e-9 {
                    label!(
                        "draw-bottom-clearance",
                        "BOTTOM CLEARANCE {clearance}",
                        clearance = metres(*bottom_clearance)
                    )
                } else {
                    label!(
                        "draw-tank-bottom",
                        "TANK BOTTOM {height} (CLEARANCE {clearance})",
                        height = metres(bottom),
                        clearance = metres(*bottom_clearance)
                    )
                };
                labels.push((text, [centres[0], bottom / 2.0]));
            }
        }
    }
    let overshoot = 4.0 * c.k;
    c.centre_line([0.0, -overshoot], [0.0, h.depth + overshoot]);
    if !c.try_dim_horizontal([-half, 0.0], [half, 0.0], -DIM_OFFSET_MM) {
        labels.push((
            label!("draw-beam", "BEAM {beam}", beam = metres(h.beam)),
            [-half, 0.0],
        ));
    }
    // depth on the right, with draught and freeboard where there is room, otherwise as labels
    if !c.try_dim_vertical([half, 0.0], [half, h.depth], -2.0 * DIM_OFFSET_MM) {
        labels.push((
            label!("draw-depth", "DEPTH {depth}", depth = metres(h.depth)),
            [-half, h.depth],
        ));
    }
    if !c.try_dim_vertical([half, 0.0], [half, h.draught], -DIM_OFFSET_MM) {
        labels.push((
            label!(
                "draw-draught",
                "DRAUGHT {draught}",
                draught = metres(h.draught)
            ),
            [-half, h.draught],
        ));
    }
    if !c.try_dim_vertical([half, h.draught], [half, h.depth], -DIM_OFFSET_MM) {
        labels.push((
            label!(
                "draw-freeboard",
                "FREEBOARD {freeboard}",
                freeboard = metres(h.freeboard())
            ),
            [-half, (h.draught + h.depth) / 2.0],
        ));
    }
    if let Some(t) = h.light_draught {
        labels.push((
            label!(
                "draw-light-draught",
                "LIGHT DRAUGHT {draught}",
                draught = metres(t)
            ),
            [-half, t],
        ));
    }
    // The labels stand in one column to the left of the hull. The right side holds the dimensions,
    // and a leader from beyond them would run through their lines and figures.
    c.label_column(-half - 2.0 * DIM_OFFSET_MM * c.k, &mut labels);
    c
}

// ---- the documents -------------------------------------------------------------------------------

/// The window that holds a canvas, with a little room all round and the scale bar's room below.
fn window_of(c: &Canvas) -> [f64; 4] {
    let room = 3.0 * c.k;
    [
        c.min[0] - room,
        c.min[1] - BAR_ROOM_MM.max(4.0) * c.k,
        c.max[0] + room,
        c.max[1] + room,
    ]
}

/// The notes for a design that fails checks: none, or one note on one line. The standard has no
/// note of several lines and no line that continues a note (clauses 11.3.3 and 14.10.1), and a note
/// is never cut or made smaller (clause 14.6). So the note names as many checks as fit, in order,
/// and then says how many more there are: `DESIGN FAILS 6 CHECKS: A, B AND 4 MORE`. It fits when it
/// has at most `NOTE_MAX_CHARS` characters (the schema) and its width, with the number in front of
/// it, is within the strip (clause 14.6). If not even the first name fits, the count stands alone.
fn failed_notes(failed: &[String]) -> Vec<String> {
    if failed.is_empty() {
        return Vec::new();
    }
    let count = failed.len();
    for shown in (1..=count).rev() {
        let checks = failed[..shown].join(", ");
        let note = match (count, count - shown) {
            (1, _) => label!(
                "draw-fails-one",
                "DESIGN FAILS {count} CHECK: {checks}",
                count = count,
                checks = checks
            ),
            (_, 0) => label!(
                "draw-fails",
                "DESIGN FAILS {count} CHECKS: {checks}",
                count = count,
                checks = checks
            ),
            (_, more) => label!(
                "draw-fails-more",
                "DESIGN FAILS {count} CHECKS: {checks} AND {more} MORE",
                count = count,
                checks = checks,
                more = more
            ),
        };
        if note_fits(&note) {
            return vec![note];
        }
    }
    if count == 1 {
        vec![label!(
            "draw-fails-one-count",
            "DESIGN FAILS {count} CHECK",
            count = count
        )]
    } else {
        vec![label!(
            "draw-fails-count",
            "DESIGN FAILS {count} CHECKS",
            count = count
        )]
    }
}

/// Does a note fit its line in the strip? At most `NOTE_MAX_CHARS` characters, and a width, after
/// a number of two digits and the full stop, of at most the strip width less `2 * 2` mm (clause
/// 14.6), by the width table of the standard (clause 11.7.1) at the note height.
fn note_fits(note: &str) -> bool {
    note.chars().count() <= NOTE_MAX_CHARS
        && ovcd_draw::rules::text_width(&format!("99. {note}"), NOTE_TEXT_MM)
            .is_some_and(|w| w <= SHEET.2 - 4.0)
}

/// The height of the strip of notes (clause 14.10.1): the heading, the notes at one pitch each, and
/// the status line on the last line; never under the usual strip.
fn strip_mm(notes: usize) -> f64 {
    (12.0 + NOTE_PITCH_MM * notes as f64).max(STRIP_MM)
}

/// The frame of the viewport: the drawing space less the strip of notes.
fn frame(strip: f64) -> (f64, f64, f64, f64) {
    (SHEET.0, SHEET.1, SHEET.2, SHEET.3 - strip)
}

fn fits(window: [f64; 4], n: u32, strip: f64) -> bool {
    let k = 1000.0 / f64::from(n);
    let (_, _, fw, fh) = frame(strip);
    (window[2] - window[0]) * k <= fw + 1e-9 && (window[3] - window[1]) * k <= fh + 1e-9
}

/// Build one view at the largest scale at which everything fits.
fn document(h: &Hull, title: &str, build: fn(&Hull, u32) -> Canvas) -> Result<Value, String> {
    let mut notes = vec![label!(
        "draw-note-dimensions",
        "{view}. DIMENSIONS IN METRES.",
        view = title
    )];
    let what = format!("{}; {}", h.vessel, h.cargo);
    if what.len() > 2
        && what.chars().all(|c| (' '..='~').contains(&c))
        && what.chars().count() <= 100
    {
        notes.push(what);
    }
    notes.extend(failed_notes(&h.failed_checks));
    let strip = strip_mm(notes.len());
    for n in RATIOS {
        let canvas = build(h, n);
        let window = window_of(&canvas);
        if !fits(window, n, strip) {
            continue;
        }
        let (fx, fy, fw, fh) = frame(strip);
        let layers: Vec<Value> = [
            "OUTLINE",
            "OBJECT",
            "HIDDEN",
            "CENTRE",
            "DIMENSIONS",
            "ANNO",
            "TEXT",
            "WATER",
            "HATCH",
            "ZONE",
            "EQUIP",
        ]
        .iter()
        .map(|n| json!({"name": n}))
        .collect();
        return Ok(json!({
            "standard_version": "1.0",
            "sheet": {"size": "A3", "orientation": "landscape"},
            "layers": layers,
            "status": "concept",
            "strips": [{"id": "notes", "edge": "bottom", "size_mm": strip, "content": "notes"}],
            "notes": notes,
            "viewports": [{
                "id": "main",
                "frame": {"x_mm": fx, "y_mm": fy, "width_mm": fw, "height_mm": fh},
                "window": {"xmin_m": window[0], "ymin_m": window[1], "xmax_m": window[2], "ymax_m": window[3]},
                "scale": "fit",
                "scale_bar": {},
                "entities": canvas.entities,
            }],
        }));
    }
    Err("the hull does not fit any scale of the standard on an A3 sheet".to_string())
}

/// The three views of an Option's results (the `simulate` result object).
pub fn ga_views(results: &Value) -> Result<GaViews, String> {
    let hull = read_hull(results)?;
    Ok(GaViews {
        side_elevation: document(
            &hull,
            &label!("draw-view-side-elevation", "SIDE ELEVATION"),
            side_elevation,
        )?,
        plan: document(&hull, &label!("draw-view-plan", "PLAN"), plan)?,
        midship_section: document(
            &hull,
            &label!("draw-view-midship-section", "MIDSHIP SECTION"),
            midship_section,
        )?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(n: usize, long: bool) -> Vec<String> {
        (0..n)
            .map(|i| {
                if long {
                    format!("A VERY LONG CHECK NAME FOR THE TEST NUMBER {i}")
                } else {
                    format!("CHECK {i}")
                }
            })
            .collect()
    }

    #[test]
    fn no_failed_checks_make_no_note() {
        assert!(failed_notes(&[]).is_empty());
    }

    #[test]
    fn one_failed_check_is_named_in_the_singular() {
        assert_eq!(
            failed_notes(&["STABILITY - LADEN".to_string()]),
            ["DESIGN FAILS 1 CHECK: STABILITY - LADEN"]
        );
    }

    #[test]
    fn a_few_failed_checks_are_all_named_in_order() {
        assert_eq!(
            failed_notes(&names(3, false)),
            ["DESIGN FAILS 3 CHECKS: CHECK 0, CHECK 1, CHECK 2"]
        );
    }

    #[test]
    fn many_failed_checks_make_one_note_that_counts_the_rest() {
        for count in [8, 20, 60] {
            let notes = failed_notes(&names(count, true));
            assert_eq!(notes.len(), 1);
            let note = &notes[0];
            assert!(note.starts_with(&format!("DESIGN FAILS {count} CHECKS: A VERY LONG")));
            assert!(note.contains(" AND ") && note.ends_with(" MORE"), "{note}");
            assert!(note_fits(note), "{note}");
            // one name more would not have fitted
            let shown = note.matches(" NUMBER ").count();
            let more = count - shown;
            assert!(note.ends_with(&format!("AND {more} MORE")), "{note}");
            let next = format!(
                "{}, {}",
                &note[..note.rfind(" AND ").unwrap()],
                names(count, true)[shown]
            );
            assert!(!note_fits(&format!("{next} AND {} MORE", more - 1)) || more == 1);
        }
    }

    #[test]
    fn a_name_too_long_for_the_line_gives_the_count_alone() {
        let long = "X".repeat(200);
        assert_eq!(
            failed_notes(&[long.clone(), long]),
            ["DESIGN FAILS 2 CHECKS"]
        );
    }
}
