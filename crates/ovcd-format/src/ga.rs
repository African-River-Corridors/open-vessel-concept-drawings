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
/// The height of the notes strip at the bottom of the sheet.
const STRIP_MM: f64 = 18.0;
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
    containment: Containment,
}

enum Containment {
    /// A box hold or a sloped hopper hold.
    Hold {
        height: f64,
        top_width: f64,
        bottom_width: f64,
    },
    /// Type C tanks: horizontal cylinders with hemispherical heads.
    Tanks {
        od: f64,
        n_wide: usize,
        n_long: usize,
        side_clearance: f64,
        bottom_clearance: f64,
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
            gap_long: defaults.tank_gap_longitudinal,
            gap_across: defaults.tank_gap_transverse,
        }
    } else {
        Containment::Hold {
            height: geo("hold_height_m")?,
            top_width: geo("hold_top_width_m")?,
            bottom_width: geo("hold_bottom_width_m")?,
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
        Containment::Hold { height, .. } => {
            let (x0, x1) = (h.stern_void, h.loa - h.bow_void);
            c.polyline(
                "HIDDEN",
                &[
                    [x0, h.depth - height],
                    [x1, h.depth - height],
                    [x1, h.depth],
                    [x0, h.depth],
                ],
                true,
            );
        }
        Containment::Tanks {
            od,
            bottom_clearance,
            ..
        } => {
            if let Some((length, starts, _)) = h.tank_layout() {
                let r = od / 2.0;
                let yc = bottom_clearance + r;
                for x0 in starts {
                    let (xa, xb) = (x0 + r, x0 + length - r);
                    c.line("HIDDEN", [xa, yc + r], [xb, yc + r], None);
                    c.line("HIDDEN", [xa, yc - r], [xb, yc - r], None);
                    c.arc("HIDDEN", [xa, yc], r, 90.0, 270.0, None);
                    c.arc("HIDDEN", [xb, yc], r, 270.0, 90.0, None);
                }
            }
        }
    }
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
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
                for x0 in &starts {
                    for yc in &centres {
                        let (xa, xb) = (x0 + r, x0 + length - r);
                        c.line("HIDDEN", [xa, yc + r], [xb, yc + r], None);
                        c.line("HIDDEN", [xa, yc - r], [xb, yc - r], None);
                        c.arc("HIDDEN", [xa, *yc], r, 90.0, 270.0, None);
                        c.arc("HIDDEN", [xb, *yc], r, 270.0, 90.0, None);
                    }
                }
            }
        }
    }
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

fn midship_section(h: &Hull, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    let half = h.beam / 2.0;
    c.polyline(
        "OUTLINE",
        &[[-half, h.depth], [-half, 0.0], [half, 0.0], [half, h.depth]],
        false,
    );
    c.line("OUTLINE", [-half, h.depth], [half, h.depth], None);
    c.line("WATER", [-half, h.draught], [half, h.draught], None);
    if let Some(t) = h.light_draught {
        c.line("WATER", [-half, t], [half, t], Some("chain_double_dashed"));
    }
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
    match &h.containment {
        Containment::Hold {
            height,
            top_width,
            bottom_width,
        } => {
            let bottom = h.depth - height;
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
            if let Some((_, _, centres)) = h.tank_layout() {
                let yc = bottom_clearance + od / 2.0;
                for x in &centres {
                    c.circle("OUTLINE", [*x, yc], od / 2.0);
                }
                labels.push((
                    format!("SIDE CLEARANCE {}", metres(*side_clearance)),
                    [-half + side_clearance / 2.0, yc.min(h.depth)],
                ));
                labels.push((
                    format!("BOTTOM CLEARANCE {}", metres(*bottom_clearance)),
                    [0.0, bottom_clearance / 2.0],
                ));
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
