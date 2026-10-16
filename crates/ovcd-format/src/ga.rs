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
use ovcd_core::models::BargeInputs;
use ovcd_core::pyfmt::fixed;
use serde_json::{Value, json};

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
/// The ratios of the standard (clauses 7.3.1 and 7.3.4), smallest denominator first.
const RATIOS: [u32; 20] = [
    1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500, 1000, 1250, 2000, 2500, 5000, 10000, 20000, 50000,
    100000,
];
/// Paper distance of the dimension line from the geometry, in millimetres.
const DIM_OFFSET_MM: f64 = 12.0;
/// Room that a dimension or label needs beyond the geometry on its side, in millimetres.
const DIM_ROOM_MM: f64 = 6.0;
/// Spacing of stacked labels, in millimetres.
const LABEL_STEP_MM: f64 = 6.0;
/// A safe width for one character of label text at height 3.5, in millimetres. The labels are capital
/// letters, digits and a few signs, which average 2.3 mm; the window only needs to be generous.
const CHAR_MM: f64 = 2.5;
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
/// Half the base of a dome in the midship section, as a fraction of the tank diameter.
const DOME_HALF_BASE_OF_OD: f64 = 0.15;
/// Half the base of a dome in the elevation, as a fraction of one tank's length, and its floor in
/// metres.
const DOME_HALF_BASE_OF_LENGTH: f64 = 0.06;
const DOME_HALF_BASE_MIN_M: f64 = 1.2;

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
    containment: Containment,
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
    let containment = if results.get("phase").and_then(Value::as_str) == Some("liquid_gas") {
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
            wing_width: num_at(results, &["drawing", "wing_tank_width_m"]),
            wing_gap: num_at(results, &["drawing", "wing_tank_gap_m"]),
            wing_ratio: num_at(results, &["drawing", "wing_tank_height_ratio"]),
            gap_long: defaults.tank_gap_longitudinal,
            gap_across: defaults.tank_gap_transverse,
        }
    } else {
        let drawing = |key: &str| need(results, &["drawing", key]);
        Containment::Hold {
            top_width: geo("hold_top_width_m")?,
            bottom_width: geo("hold_bottom_width_m")?,
            inner_bottom: drawing("double_bottom_m")?,
            coaming: drawing("hatch_coaming_height_m")?,
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
                let top = (self.tank_bottom()? + ratio * od).min(self.depth);
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

    /// The dome height of the gas tanks, if the results have a positive one.
    fn dome(&self) -> Option<f64> {
        match &self.containment {
            Containment::Tanks { dome: Some(d), .. } if *d > 0.0 => Some(*d),
            _ => None,
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
    /// point when it is right of the target, to the left otherwise.
    fn leader(&mut self, text: &str, target: [f64; 2], label_at: [f64; 2]) {
        self.extend(target);
        let width = (text.chars().count() as f64 * CHAR_MM + 2.0) * self.k;
        self.extend([
            label_at[0]
                + if label_at[0] >= target[0] {
                    width
                } else {
                    -width
                },
            label_at[1] + 4.0 * self.k,
        ]);
        self.extend([label_at[0], label_at[1] - 4.0 * self.k]);
        self.entities.push(json!({"type": "leader", "layer": "ANNO", "from": {"point_m": pt(target)}, "label_at_m": pt(label_at), "text": text}));
    }

    /// Labels in a column at `x`, one under another from the highest target, each at least one
    /// step below the one above, so that the texts do not touch. Leaders run level or downward.
    fn label_column(&mut self, x: f64, labels: &mut [(String, [f64; 2])]) {
        labels.sort_by(|a, b| {
            b.1[1]
                .partial_cmp(&a.1[1])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut previous = f64::INFINITY;
        for (text, target) in labels.iter() {
            let y = target[1].min(previous - LABEL_STEP_MM * self.k);
            self.leader(text, *target, [x, y]);
            previous = y;
        }
    }
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
        labels.push(("DOUBLE BOTTOM".to_string(), [x_fwd, db]));
    }
    if let Some((_, _, top)) = h.wing_tank() {
        c.polyline("HIDDEN", &[[x0, db], [x0, top], [x1, top], [x1, db]], false);
        labels.push(("WBT (BOTH SIDES)".to_string(), [x1, (db + top) / 2.0]));
    }
    if let (Some(rise), Some((length, starts, _))) = (h.dome(), h.tank_layout()) {
        let r = od / 2.0;
        let layer = if yc + r >= h.depth {
            "OBJECT"
        } else {
            "HIDDEN"
        };
        for x in &starts {
            let mid = x + length / 2.0;
            let half = (DOME_HALF_BASE_OF_LENGTH * length)
                .max(DOME_HALF_BASE_MIN_M)
                .min(length / 2.0 - r);
            if half > 0.0 {
                c.dome(layer, mid, yc + r, half, rise);
            }
        }
        // named at the last dome, nearest the label column
        if let Some(x) = starts.last() {
            labels.push(("DOME".to_string(), [x + length / 2.0, yc + r + rise]));
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
/// forward equipment zone, each named by a leader above the hull.
fn zones(c: &mut Canvas, h: &Hull, view: View) {
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
    let above = hi + 10.0 * c.k;
    if let Some((cofferdam, aft)) = h.separation
        && cofferdam > 0.0
    {
        let start = x0 - aft;
        for x in [start, start + cofferdam] {
            let (a, b) = across(x);
            c.line("HIDDEN", a, b, None);
        }
        let mid = start + cofferdam / 2.0;
        // the text stands to the left of the label point, over the stern, well clear of the first tank
        c.leader("COFFERDAM", [mid, hi], [mid - 8.0 * c.k, above]);
    }
    if h.bow_void > 0.0 {
        let mid = h.loa - h.bow_void / 2.0;
        // the text stands to the right of the label point, over the bow
        c.leader("FWD EQUIP.", [mid, hi], [mid + 2.0 * c.k, above]);
    }
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
                for x0 in starts {
                    let (xa, xb) = (x0 + r, x0 + length - r);
                    c.stadium_split(("OBJECT", "HIDDEN"), (xa, xb, yc, r), h.depth);
                }
            }
        }
    }
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
    elevation_gas_detail(&mut c, h, &mut labels);
    zones(&mut c, h, View::Elevation);
    if !c.try_dim_horizontal([0.0, 0.0], [h.loa, 0.0], -DIM_OFFSET_MM) {
        labels.push((format!("LOA {}", metres(h.loa)), [h.loa, 0.0]));
    }
    if !c.try_dim_vertical([0.0, 0.0], [0.0, h.depth], DIM_OFFSET_MM) {
        labels.push((format!("DEPTH {}", metres(h.depth)), [h.loa, h.depth]));
    }
    // labels for draught and freeboard, to the right of the bow
    let mid_free = (h.draught + h.depth) / 2.0;
    labels.push((
        format!("FREEBOARD {}", metres(h.freeboard())),
        [h.x_at_height(mid_free), mid_free],
    ));
    labels.push((
        format!("DRAUGHT {}", metres(h.draught)),
        [h.x_at_height(h.draught), h.draught],
    ));
    if let Some(t) = h.light_draught {
        labels.push((
            format!("LIGHT DRAUGHT {}", metres(t)),
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
                for x0 in &starts {
                    for yc in &centres {
                        let (xa, xb) = (x0 + r, x0 + length - r);
                        if tank_y >= h.depth {
                            // the widest part stands above the deck: the whole outline is in view
                            c.stadium("OBJECT", xa, xb, *yc, r);
                        } else {
                            // The widest part is under the deck, so the outline is hidden. Where the
                            // tank's top stands above the deck, the part above is in view, and from
                            // above it is narrower: a sphere or cylinder cut at the deck height.
                            c.stadium("HIDDEN", xa, xb, *yc, r);
                            let dy = h.depth - tank_y;
                            if dy < r {
                                c.stadium("OBJECT", xa, xb, *yc, chord_half(r, dy));
                            }
                        }
                    }
                }
            }
        }
    }
    zones(&mut c, h, View::Plan);
    let overshoot = 4.0 * c.k;
    c.centre_line([-overshoot, 0.0], [h.loa + overshoot, 0.0]);
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
    if !c.try_dim_horizontal([0.0, y0], [h.loa, y0], -DIM_OFFSET_MM) {
        labels.push((format!("LOA {}", metres(h.loa)), [h.loa, y0]));
    }
    if !c.try_dim_vertical([0.0, y0], [0.0, y1], DIM_OFFSET_MM) {
        labels.push((format!("BEAM {}", metres(h.beam)), [h.loa, y1]));
    }
    let column = h.loa + 6.0 * c.k;
    c.label_column(column, &mut labels);
    c
}

/// Half the chord of a circle of radius `r` at distance `d` (under `r`) from its centre.
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
        labels.push(("DOUBLE BOTTOM".to_string(), [-half + 0.15 * half, db / 2.0]));
    }
    // wing ballast tanks (the wall inboard and the top) and the void between each and the cargo tank
    if let Some((w, gap, top)) = h.wing_tank() {
        for side in [-1.0, 1.0] {
            let (outer, wall, edge) = (side * half, side * (half - w), side * (half - w - gap));
            c.polyline("OBJECT", &[[outer, top], [wall, top], [wall, db]], false);
            if gap > 0.0 {
                c.polyline("ZONE", &[[wall, top], [edge, top], [edge, db]], false);
            }
        }
        let mid = (db + top) / 2.0;
        labels.push(("WBT (BOTH SIDES)".to_string(), [-half + w / 2.0, mid]));
        if gap > 0.0 {
            labels.push((
                "VOID (BOTH SIDES)".to_string(),
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
            labels.push(("CRADLE".to_string(), [left[0], (left[1] + db) / 2.0]));
        }
    }
    // a dome on top of each tank
    if let Some(rise) = h.dome() {
        let half_base = DOME_HALF_BASE_OF_OD * od;
        let base = yc + (r * r - half_base * half_base).sqrt();
        let sag = yc + r + rise - base;
        for (i, x) in centres.iter().enumerate() {
            c.dome("OBJECT", *x, base, half_base, sag);
            if i == 0 {
                labels.push(("DOME".to_string(), [*x, base + sag]));
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
            labels.push((format!("INNER BOTTOM {}", metres(bottom)), [-half, bottom]));
            labels.push((
                format!("COAMING {}", metres(*coaming)),
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
                    format!("SIDE CLEARANCE {}", metres(*side_clearance)),
                    [(-half + edge.max(-half)) / 2.0, y],
                ));
                // Bottom clearance: the gap between the keel and the tank bottom, under the
                // nearest tank. The core stands the tank on double bottom + support height, which
                // is not always the clearance it asks for; both are given when they differ.
                let text = if (bottom - bottom_clearance).abs() < 1e-9 {
                    format!("BOTTOM CLEARANCE {}", metres(*bottom_clearance))
                } else {
                    format!(
                        "TANK BOTTOM {} (CLEARANCE {})",
                        metres(bottom),
                        metres(*bottom_clearance)
                    )
                };
                labels.push((text, [centres[0], bottom / 2.0]));
            }
        }
    }
    let overshoot = 4.0 * c.k;
    c.centre_line([0.0, -overshoot], [0.0, h.depth + overshoot]);
    if !c.try_dim_horizontal([-half, 0.0], [half, 0.0], -DIM_OFFSET_MM) {
        labels.push((format!("BEAM {}", metres(h.beam)), [-half, 0.0]));
    }
    // depth on the right, with draught and freeboard where there is room, otherwise as labels
    if !c.try_dim_vertical([half, 0.0], [half, h.depth], -2.0 * DIM_OFFSET_MM) {
        labels.push((format!("DEPTH {}", metres(h.depth)), [-half, h.depth]));
    }
    if !c.try_dim_vertical([half, 0.0], [half, h.draught], -DIM_OFFSET_MM) {
        labels.push((format!("DRAUGHT {}", metres(h.draught)), [-half, h.draught]));
    }
    if !c.try_dim_vertical([half, h.draught], [half, h.depth], -DIM_OFFSET_MM) {
        labels.push((
            format!("FREEBOARD {}", metres(h.freeboard())),
            [-half, (h.draught + h.depth) / 2.0],
        ));
    }
    if let Some(t) = h.light_draught {
        labels.push((format!("LIGHT DRAUGHT {}", metres(t)), [-half, t]));
    }
    // labels run to the left of the hull
    let column = -half - 2.0 * DIM_OFFSET_MM * c.k;
    c.label_column(column, &mut labels);
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

/// The frame of the viewport: the drawing space less the strip of notes.
fn frame() -> (f64, f64, f64, f64) {
    (SHEET.0, SHEET.1, SHEET.2, SHEET.3 - STRIP_MM)
}

fn fits(window: [f64; 4], n: u32) -> bool {
    let k = 1000.0 / f64::from(n);
    let (_, _, fw, fh) = frame();
    (window[2] - window[0]) * k <= fw + 1e-9 && (window[3] - window[1]) * k <= fh + 1e-9
}

/// Build one view at the largest scale at which everything fits.
fn document(h: &Hull, title: &str, build: fn(&Hull, u32) -> Canvas) -> Result<Value, String> {
    for n in RATIOS {
        let canvas = build(h, n);
        let window = window_of(&canvas);
        if !fits(window, n) {
            continue;
        }
        let (fx, fy, fw, fh) = frame();
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
        ]
        .iter()
        .map(|n| json!({"name": n}))
        .collect();
        let mut notes = vec![format!("{title}. DIMENSIONS IN METRES.")];
        let what = format!("{}; {}", h.vessel, h.cargo);
        if what.len() > 2
            && what.chars().all(|c| (' '..='~').contains(&c))
            && what.chars().count() <= 100
        {
            notes.push(what);
        }
        return Ok(json!({
            "standard_version": "1.0",
            "sheet": {"size": "A3", "orientation": "landscape"},
            "layers": layers,
            "status": "concept",
            "strips": [{"id": "notes", "edge": "bottom", "size_mm": STRIP_MM, "content": "notes"}],
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
        side_elevation: document(&hull, "SIDE ELEVATION", side_elevation)?,
        plan: document(&hull, "PLAN", plan)?,
        midship_section: document(&hull, "MIDSHIP SECTION", midship_section)?,
    })
}
