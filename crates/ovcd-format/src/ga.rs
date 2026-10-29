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

mod arrangement;

mod tug;

pub use arrangement::{NO_ARRANGEMENT_PLAN, arrangement_plan_for, arrangement_plan_named};
pub use tug::{NO_TUG_GA, tug_ga_for, tug_ga_named};

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

/// What a convoy gets in place of views: its results are the totals of one barge's, and the
/// arrangement plan of a convoy comes with the pusher (#360).
pub const CONVOY_NO_VIEWS: &str = "no drawings of a pushed convoy yet";

/// The views of an Option that was run. A convoy, or an Option that cannot be computed, gives no
/// views and the same message as the outputs panel.
pub fn ga_views_of(outcome: &RunOutcome) -> Result<GaViews, String> {
    ga_views_named(outcome, None)
}

/// As [`ga_views_of`], with the name of the Option for the title block of each sheet.
pub fn ga_views_named(outcome: &RunOutcome, option: Option<&str>) -> Result<GaViews, String> {
    match outcome {
        RunOutcome::Results { results, .. } if results.get("convoy").is_some() => {
            Err(CONVOY_NO_VIEWS.to_string())
        }
        RunOutcome::Results { results, .. } => ga_views_for(results, option),
        RunOutcome::NotAvailable(message) | RunOutcome::Failed(message) => Err(message.clone()),
    }
}

// ---- the sheet ---------------------------------------------------------------------------------

/// The drawing space of A3 landscape (clause 6.3), in millimetres.
const SHEET: (f64, f64, f64, f64) = (20.0, 10.0, 390.0, 277.0);
/// The height of the title block (ISO 7200 template, clause 14.3): it stands in the bottom-right
/// corner of the drawing space, and the notes and the viewport stand above it (clause 14.10).
const TITLE_BLOCK_MM: f64 = 56.0;
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
/// The gap between the tug's bow and the end wall of the Notch, where the coupler is drawn (m), and
/// the share of the tug's length that its bow tapers over.
const ATB_COUPLER_GAP_M: f64 = 0.3;
const ATB_BOW_TAPER: f64 = 0.12;

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
    /// The tunnel stern, when the Option has one.
    tunnel_stern: Option<TunnelStern>,
    /// The Blind Distance light and laden, and the limit.
    blind: Option<(f64, f64, f64)>,
    /// The Option's name, for the title block (`None` when the caller gives none).
    option_name: Option<String>,
    /// The Rule Sets the Option was checked against (`rule_trace.rule_sets`), as `id@version`.
    rule_sets: Vec<String>,
    /// The draught of the "Ballast, departure" condition, when the results have the IS Code loading
    /// conditions (`loading_conditions`).
    ballast_draught: Option<f64>,
    /// The top of the structure above the keel, loaded: the draught and the loaded air draft.
    air_top: Option<f64>,
    /// The number of holds of a dry bulk hull (`drawing.hold_count`); 0 for a gas hull.
    holds: usize,
    /// The hull's own starboard half-section at mid-length, keel to deck edge, as [y, z]
    /// (`drawing.midship_outline`, issue #450); `None` for the rectangle.
    midship_outline: Option<Vec<[f64; 2]>>,
    /// The forward separation and the service space of a gas hull (`drawing.separation`).
    fwd_separation: Option<(f64, f64)>,
    bow_rake: f64,
    stern_rake: f64,
    bow_void: f64,
    stern_void: f64,
    /// The gas separation forward of the stern block (`drawing.separation`): the cofferdam and the
    /// aft separation, whose end is the start of the cargo zone. `None` for dry bulk.
    separation: Option<(f64, f64)>,
    /// The aft block of a self-propelled hull (`drawing.aft_arrangement`); `None` for a pushed
    /// barge. Its x is measured from the stern, as the hull's x is.
    aft: Option<Aft>,
    /// The names of the checks that fail (`results.checks` with `ok` false), as plain capitals.
    failed_checks: Vec<String>,
    /// The Notch of an ATB barge and its tug (`drawing.atb`, issue #363); `None` for any other
    /// Formation.
    atb: Option<AtbDraw>,
    containment: Containment,
}

/// The Notch of an ATB barge and the tug that sits in it, as the views draw them.
struct AtbDraw {
    notch_depth: f64,
    notch_width: f64,
    /// The tug, when the Option includes it: length, beam, draught, depth.
    tug: Option<(f64, f64, f64, f64)>,
}

fn read_atb(results: &Value) -> Option<AtbDraw> {
    let a = results.get("drawing")?.get("atb")?;
    let tug = a.get("tug").filter(|t| !t.is_null()).and_then(|t| {
        Some((
            t.get("length_m")?.as_f64()?,
            t.get("beam_m")?.as_f64()?,
            t.get("draught_m")?.as_f64()?,
            t.get("depth_m")?.as_f64()?,
        ))
    });
    Some(AtbDraw {
        notch_depth: a.get("notch_depth_m")?.as_f64()?,
        notch_width: a.get("notch_width_m")?.as_f64()?,
        tug,
    })
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
    /// The wheelhouse raised for the sightline (`wheelhouse_floor_raised_m` and
    /// `wheelhouse_top_raised_m`), when it is lifted (issue #356).
    wheelhouse_raised: Option<(f64, f64)>,
    block_top: f64,
}

/// The tunnel stern, from `drawing.stern` (issue #356): the tunnel crown along the run, and the
/// athwartships extent of each tunnel at the transom.
struct TunnelStern {
    /// (x, height above the keel) of the crown, from the transom to the end of the run.
    crown: Vec<[f64; 2]>,
    /// (y of the port edge, y of the starboard edge) of each tunnel.
    tunnels: Vec<(f64, f64)>,
}

fn read_tunnel_stern(results: &Value) -> Option<TunnelStern> {
    if results["geometry"]["stern_form"] != "tunnel" {
        return None;
    }
    let stern = results.get("drawing")?.get("stern")?;
    let points = |v: &Value| -> Option<Vec<[f64; 2]>> {
        v.as_array()?
            .iter()
            .map(|p| Some([p.get(0)?.as_f64()?, p.get(1)?.as_f64()?]))
            .collect()
    };
    let crown = points(stern.get("profile")?)?;
    let tunnels = stern
        .get("tunnel_sections")?
        .as_array()?
        .iter()
        .filter_map(|t| {
            let pts = points(t)?;
            let lo = pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
            let hi = pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
            (lo.is_finite() && hi > lo).then_some((lo, hi))
        })
        .collect();
    (crown.len() > 1).then_some(TunnelStern { crown, tunnels })
}

/// The Blind Distances of the sightline, light and laden, and their limit (`sightline`, issue #356).
fn read_blind(results: &Value) -> Option<(f64, f64, f64)> {
    let s = results.get("sightline")?;
    Some((
        s.get("blind_distance_light_m")?.as_f64()?,
        s.get("blind_distance_laden_m")?.as_f64()?,
        s.get("limit_m")?.as_f64()?,
    ))
}

/// Text in the plain capitals a note or a field allows: an em dash becomes a hyphen, an underscore a
/// space, any other character outside printable ASCII is dropped, and the spaces are squeezed.
fn plain_capitals(name: &str) -> String {
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
            plain_capitals(name)
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
        wheelhouse_raised: n("wheelhouse_lift_m")
            .filter(|lift| *lift > 0.0)
            .and_then(|_| {
                Some((
                    n("wheelhouse_floor_raised_m")?,
                    n("wheelhouse_top_raised_m")?,
                ))
            }),
        block_top: n("block_top_lowered_m")?,
    })
}

/// A Space of a liquid bulk hull as the drawing reads it (`drawing.liquid_tanks.spaces`): the
/// views draw its extent and its cross-section corners, nothing of their own.
struct DrawSpace {
    name: String,
    /// The Space kind key: `cargo_tank`, `slop`, `cofferdam`, `pump_room`, `double_bottom_ballast`,
    /// `wing_ballast`.
    kind: String,
    /// `port`, `starboard` or `centre`.
    side: String,
    x_aft: f64,
    x_fwd: f64,
    /// The cross-section corners, (y, z) in metres.
    section: Vec<[f64; 2]>,
}

impl DrawSpace {
    fn is_tank(&self) -> bool {
        self.kind == "cargo_tank" || self.kind == "slop"
    }

    fn mid_x(&self) -> f64 {
        (self.x_aft + self.x_fwd) / 2.0
    }

    /// The least and greatest (y, z) of the section.
    fn extent(&self) -> ([f64; 2], [f64; 2]) {
        let mut lo = [f64::INFINITY; 2];
        let mut hi = [f64::NEG_INFINITY; 2];
        for p in &self.section {
            for i in 0..2 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        }
        (lo, hi)
    }

    /// The name without its side letter: `3P` is tank `3`.
    fn number(&self) -> String {
        match self.name.strip_suffix(['P', 'S']) {
            Some(n) => n.to_string(),
            None => self.name.clone(),
        }
    }
}

fn read_draw_spaces(results: &Value) -> Result<Vec<DrawSpace>, String> {
    let list = results["drawing"]["liquid_tanks"]["spaces"]
        .as_array()
        .ok_or("the results have no `drawing.liquid_tanks.spaces`")?;
    draw_spaces_of(list)
}

/// The ballast Spaces of a dry bulk hull (`drawing.ballast_spaces`); none when the results have no
/// IS Code loading conditions (issue #426).
fn read_ballast_spaces(results: &Value) -> Result<Vec<DrawSpace>, String> {
    match results["drawing"]["ballast_spaces"].as_array() {
        Some(list) => draw_spaces_of(list),
        None => Ok(Vec::new()),
    }
}

fn draw_spaces_of(list: &[Value]) -> Result<Vec<DrawSpace>, String> {
    list.iter()
        .map(|v| {
            let text = |key: &str| {
                v[key]
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| format!("a Space has no `{key}`"))
            };
            let number = |key: &str| {
                v[key]
                    .as_f64()
                    .ok_or_else(|| format!("a Space has no `{key}`"))
            };
            let section = v["section_m"]
                .as_array()
                .ok_or("a Space has no `section_m`")?
                .iter()
                .map(|p| match (p[0].as_f64(), p[1].as_f64()) {
                    (Some(y), Some(z)) => Ok([y, z]),
                    _ => Err("a Space section corner is not a pair of numbers".to_string()),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(DrawSpace {
                name: text("name")?,
                kind: text("kind")?,
                side: text("side")?,
                x_aft: number("x_aft_m")?,
                x_fwd: number("x_fwd_m")?,
                section,
            })
        })
        .collect()
}

enum Containment {
    /// Integral cargo tanks of a liquid bulk hull: the typed Spaces.
    Liquid {
        spaces: Vec<DrawSpace>,
        /// Cargo heating is fitted (the lightship has a heating item).
        heating: bool,
    },
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
        /// The double bottom, wing and peak ballast tanks (`drawing.ballast_spaces`), if the
        /// results have them.
        ballast: Vec<DrawSpace>,
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
        /// The weather shield over the tanks: plate thickness (mm) and clearance from the tank top
        /// to the shield top (`drawing`), if the results have them (issue #407).
        shield_thickness: Option<f64>,
        shield_clearance: Option<f64>,
        /// `drawing.tank.overall_tank_length_m`: the core's length of one tank, heads included.
        tank_length: f64,
        /// The gaps between tanks the core laid them out with (`drawing.tank`).
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
                shield_thickness: num_at(results, &["drawing", "weather_shield_thickness_mm"]),
                shield_clearance: num_at(results, &["drawing", "weather_shield_clearance_m"]),
                tank_length: need(results, &["drawing", "tank", "overall_tank_length_m"])?,
                gap_long: need(results, &["drawing", "tank", "tank_gap_longitudinal_m"])?,
                gap_across: need(results, &["drawing", "tank", "tank_gap_transverse_m"])?,
            }
        }
        CargoClass::DryBulk => {
            let drawing = |key: &str| need(results, &["drawing", key]);
            Containment::Hold {
                top_width: geo("hold_top_width_m")?,
                bottom_width: geo("hold_bottom_width_m")?,
                inner_bottom: drawing("double_bottom_m")?,
                coaming: drawing("hatch_coaming_height_m")?,
                ballast: read_ballast_spaces(results)?,
            }
        }
        CargoClass::LiquidBulk => Containment::Liquid {
            spaces: read_draw_spaces(results)?,
            heating: results["liquid"]["lightship_items"]
                .as_array()
                .is_some_and(|items| {
                    items.iter().any(|i| {
                        i["name"] == "heating" && i["mass_t"].as_f64().is_some_and(|m| m > 0.0)
                    })
                }),
        },
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
        tunnel_stern: read_tunnel_stern(results),
        blind: read_blind(results),
        option_name: None,
        rule_sets: results["rule_trace"]["rule_sets"]
            .as_array()
            .map(|list| {
                list.iter()
                    .filter_map(|v| v.as_str().map(plain_capitals))
                    .collect()
            })
            .unwrap_or_default(),
        ballast_draught: results["loading_conditions"]["conditions"]
            .as_array()
            .and_then(|list| {
                list.iter()
                    .find(|c| c["role"] == "ballast_departure")
                    .and_then(|c| c["draught_m"].as_f64())
            })
            .filter(|t| *t > 0.0 && *t < depth),
        air_top: num_at(results, &["results", "air_draft_loaded_m"])
            .filter(|a| *a > 0.0)
            .map(|a| draught + a),
        holds: num_at(results, &["drawing", "hold_count"]).map_or(0, |n| n.max(0.0) as usize),
        midship_outline: results["drawing"]["midship_outline"]
            .as_array()
            .map(|list| {
                list.iter()
                    .filter_map(|p| Some([p.get(0)?.as_f64()?, p.get(1)?.as_f64()?]))
                    .collect()
            }),
        fwd_separation: num_at(results, &["drawing", "separation", "fwd_separation_m"]).zip(
            num_at(results, &["drawing", "separation", "service_space_m"]),
        ),
        bow_rake: num_at(results, &["geometry", "bow_rake_length_m"])
            .unwrap_or(0.0)
            .max(0.0),
        stern_rake: num_at(results, &["geometry", "stern_rake_length_m"])
            .unwrap_or(0.0)
            .max(0.0),
        bow_void: geo("bow_void_length_m")?,
        stern_void: geo("stern_void_length_m")?,
        separation: num_at(results, &["drawing", "separation", "cofferdam_m"]).zip(num_at(
            results,
            &["drawing", "separation", "aft_separation_m"],
        )),
        aft: read_aft(results),
        failed_checks: failed_checks(results),
        atb: read_atb(results),
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
            Containment::Hold { .. } | Containment::Liquid { .. } => None,
        }
    }

    /// The tank centre height above the keel, for the section and the elevation alike.
    fn tank_centre_y(&self) -> Option<f64> {
        match &self.containment {
            Containment::Tanks { od, .. } => Some(self.tank_bottom()? + od / 2.0),
            Containment::Hold { .. } | Containment::Liquid { .. } => None,
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

    /// The ballast waterline, when it stands apart from the light and the loaded waterlines on the
    /// paper (at least 1 mm from each at the scale `k`, metres per millimetre).
    fn ballast_line(&self, k: f64) -> Option<f64> {
        let t = self.ballast_draught?;
        let apart = |other: f64| (t - other).abs() >= k;
        (apart(self.draught) && self.light_draught.is_none_or(apart)).then_some(t)
    }

    /// The weather shield over the tanks, as the weights carry it (`weights.rs`): (the level its
    /// sides stand on, its top). It stands on the wing plate, or on the deck where the Moulded Depth is
    /// higher; its top is the tank top plus the clearance. `None` without a shield, without a wing
    /// tank, with a wing tank as high as the tank, or when the top is not above the deck (issue #407).
    fn weather_shield(&self) -> Option<(f64, f64)> {
        let Containment::Tanks {
            od,
            wing_ratio: Some(ratio),
            shield_thickness: Some(t),
            shield_clearance: Some(clearance),
            ..
        } = &self.containment
        else {
            return None;
        };
        let (_, _, wing_top) = self.wing_tank()?;
        let top = self.tank_bottom()? + od + clearance;
        (*t > 0.0 && *ratio < 1.0 && top > self.depth).then_some((wing_top.max(self.depth), top))
    }

    /// The double bottom of a gas hull (zero: none).
    fn gas_double_bottom(&self) -> f64 {
        match &self.containment {
            Containment::Tanks { double_bottom, .. } => *double_bottom,
            Containment::Hold { .. } | Containment::Liquid { .. } => 0.0,
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
    /// the athwartships centre of each tank). The length is the core's (`TankResult`); the tanks
    /// follow each other along the cargo zone with the longitudinal gap, and sit across the beam
    /// less the side clearances with the transverse gap (the core's layout, `tanks.rs`).
    fn tank_layout(&self) -> Option<(f64, Vec<f64>, Vec<f64>)> {
        let Containment::Tanks {
            od,
            n_wide,
            n_long,
            side_clearance,
            tank_length,
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
        let length = *tank_length;
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
    if let Some((foot, top)) = h.weather_shield() {
        // The enclosure over the tanks: not structural, so dashed (issue #407).
        c.polyline(
            "ZONE",
            &[[x0, foot], [x0, top], [x1, top], [x1, foot]],
            false,
        );
        labels.push((label!("draw-weather-shield", "WEATHER SHIELD"), [x1, top]));
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
    // Issue #409: the service space between the cofferdam and the cargo zone, and the forward
    // separation, the end-bulkhead space ahead of the last tank, from the separation result.
    if let (Some((cofferdam, aft)), Some((fwd, service))) = (h.separation, h.fwd_separation) {
        if service > 0.0 {
            let mid = x0 - aft + cofferdam + service / 2.0;
            // above the hull: under it the cofferdam's label and the LOA dimension stand
            c.leader(
                &label!("draw-service-space", "SERVICE SPACE"),
                [mid, hi],
                [mid + 2.0 * c.k, hi + 6.0 * c.k],
            );
        }
        if fwd > 0.0 && x1 + fwd < h.loa {
            let (a, b) = across(x1 + fwd);
            c.line("HIDDEN", a, b, None);
            labels.push((
                label!("draw-fwd-separation", "FWD SEPARATION"),
                [x1 + fwd / 2.0, hi],
            ));
        }
    }
    if h.bow_void > 0.0 {
        let mid = h.loa - h.bow_void / 2.0;
        // named in the column of the bow labels, with the others, so that the texts keep apart
        labels.push((label!("draw-fwd-equip", "FWD EQUIP."), [mid, hi]));
    }
}

/// The tunnel stern, hidden under the hull: in elevation the tunnel crown along the run, in plan the
/// outline of each tunnel from the transom to the end of the run (issue #356).
fn tunnel_stern(c: &mut Canvas, h: &Hull, view: View) {
    let Some(t) = &h.tunnel_stern else {
        return;
    };
    match view {
        View::Elevation => c.polyline("HIDDEN", &t.crown, false),
        View::Plan => {
            let run = t.crown.last().map_or(0.0, |p| p[0]);
            for (lo, hi) in &t.tunnels {
                c.polyline(
                    "HIDDEN",
                    &[[0.0, *lo], [run, *lo], [run, *hi], [0.0, *hi]],
                    true,
                );
            }
        }
    }
}

/// The bulkheads between the holds of a dry bulk hull: `holds - 1` of them, evenly spaced along the
/// cargo zone (the volume model counts the end bulkheads and these, `n_holds + 1` in all). In
/// elevation they stand from the inner bottom to the deck; in plan they cross the hold.
fn hold_bulkheads(c: &mut Canvas, h: &Hull, view: View, labels: &mut Vec<(String, [f64; 2])>) {
    let Containment::Hold {
        top_width,
        inner_bottom,
        ..
    } = &h.containment
    else {
        return;
    };
    if h.holds < 2 {
        return;
    }
    let (x0, x1) = (h.stern_void, h.loa - h.bow_void);
    for i in 1..h.holds {
        let x = x0 + (x1 - x0) * i as f64 / h.holds as f64;
        match view {
            View::Elevation => c.line("HIDDEN", [x, *inner_bottom], [x, h.depth], None),
            View::Plan => c.line("HIDDEN", [x, -top_width / 2.0], [x, top_width / 2.0], None),
        }
    }
    // named at the middle of the forward hold, nearest the column of labels
    let mid = x1 - (x1 - x0) / (2.0 * h.holds as f64);
    let y = match view {
        View::Elevation => (inner_bottom + h.depth) / 2.0,
        View::Plan => top_width / 4.0,
    };
    labels.push((
        label!("draw-holds", "HOLDS ({count})", count = h.holds),
        [mid, y],
    ));
}

/// The most a label point is moved along its row to clear a neighbour, in millimetres of paper. The
/// leader then slants instead of running straight up.
const ROW_SHIFT_MAX_MM: f64 = 40.0;

/// Labels that stand above the hull in rows, each over its target. A label takes the lowest row where its
/// text can stand clear of the texts already there, moved along the row (the leader slants) by no more than
/// `ROW_SHIFT_MAX_MM`; a row past the last is empty, so the search ends. A label whose leader would run
/// through the text of a lower row goes up a row first. `targets` are (text, target point); the leaders run up
/// from `base`. The text stands to the left of the label point in the forward half of the hull, so that it
/// keeps clear of the column of labels beyond the bow.
fn labels_in_rows(c: &mut Canvas, mut targets: Vec<(String, [f64; 2])>, base: f64, loa: f64) {
    targets.sort_by(|a, b| a.1[0].total_cmp(&b.1[0]));
    let gap = LABEL_CLEAR_MM * c.k;
    let reach = ROW_SHIFT_MAX_MM * c.k;
    // the text spans (x from, x to) of each row
    let mut rows: Vec<Vec<(f64, f64)>> = Vec::new();
    for (text, target) in targets {
        let width = (LEADER_TEXT_GAP_MM + text_width_mm(&text)) * c.k;
        let left = target[0] > loa / 2.0;
        // the label point, and the span of the text, for a shift along the row
        let span_at = |shift: f64| {
            let point = if left {
                target[0] - 2.0 * c.k - shift
            } else {
                target[0] + 2.0 * c.k + shift
            };
            let span = if left {
                (point - width, point)
            } else {
                (point, point + width)
            };
            (point, span)
        };
        // The shifts to try on a row: none, then just clear of each text already there.
        let place = |r: usize| -> Option<f64> {
            let spans: &[(f64, f64)] = rows.get(r).map_or(&[], Vec::as_slice);
            let mut shifts = vec![0.0];
            for sp in spans {
                let natural = span_at(0.0).0;
                shifts.push(if left {
                    natural - (sp.0 - gap)
                } else {
                    sp.1 + gap - natural
                });
            }
            shifts.sort_by(f64::total_cmp);
            shifts
                .into_iter()
                .filter(|s| *s >= 0.0 && *s <= reach)
                .find(|s| {
                    let (_, span) = span_at(*s);
                    spans
                        .iter()
                        .all(|sp| sp.1 + gap <= span.0 || span.1 + gap <= sp.0)
                })
        };
        let leader_clear = |r: usize| {
            rows.iter().take(r).all(|lower| {
                lower
                    .iter()
                    .all(|sp| target[0] < sp.0 - gap || target[0] > sp.1 + gap)
            })
        };
        let (row, shift) = (0..=rows.len())
            .find_map(|r| place(r).filter(|_| leader_clear(r)).map(|s| (r, s)))
            .or_else(|| (0..=rows.len()).find_map(|r| place(r).map(|s| (r, s))))
            .unwrap_or((rows.len(), 0.0));
        let (point, span) = span_at(shift);
        if row >= rows.len() {
            rows.resize(row + 1, Vec::new());
        }
        rows[row].push(span);
        c.leader(
            &text,
            target,
            [point, base + (row as f64 + 1.0) * LABEL_STEP_MM * c.k],
        );
    }
}

/// The names of a liquid bulk hull's Spaces, in elevation and plan. The tank positions (plan), the
/// cofferdams, the pump room and the slop tanks stand over the hull in rows; the wing ballast (plan)
/// and the double bottom (elevation) go in the column. Each label is at its Space.
fn liquid_labels(c: &mut Canvas, h: &Hull, view: View, labels: &mut Vec<(String, [f64; 2])>) {
    let Containment::Liquid { spaces, .. } = &h.containment else {
        return;
    };
    let top = match view {
        View::Elevation => h.depth.max(h.air_top.unwrap_or(0.0)),
        View::Plan => h.beam / 2.0,
    };
    let mut above: Vec<(String, [f64; 2])> = Vec::new();
    for sp in spaces {
        let (lo, hi) = sp.extent();
        let (y, z) = ((lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0);
        let (x, edge) = match view {
            View::Elevation => (sp.mid_x(), hi[1]),
            View::Plan => (sp.mid_x(), hi[0]),
        };
        let centre = match view {
            View::Elevation => [x, z],
            View::Plan => [x, y],
        };
        match sp.kind.as_str() {
            "cargo_tank" if view == View::Plan && sp.side != "starboard" => above.push((
                label!("draw-tank", "TANK {number}", number = sp.number()),
                [x, top],
            )),
            "slop" if view == View::Plan && sp.side != "starboard" => {
                above.push((label!("draw-slop-tank", "SLOP TANK"), [x, top]))
            }
            "cofferdam" => above.push((label!("draw-cofferdam", "COFFERDAM"), [x, top.min(edge)])),
            "pump_room" => above.push((label!("draw-pump-room", "PUMP ROOM"), [x, top.min(edge)])),
            "wing_ballast" if view == View::Plan && sp.side == "port" => {
                labels.push((label!("draw-wbt-both-sides", "WBT (BOTH SIDES)"), centre));
            }
            "double_bottom_ballast" if view == View::Elevation && sp.side == "port" => {
                labels.push((
                    label!("draw-double-bottom", "DOUBLE BOTTOM"),
                    [sp.x_fwd - 3.0 * c.k, z],
                ))
            }
            _ => {}
        }
    }
    labels_in_rows(c, above, top + 2.0 * c.k, h.loa);
}

/// The ballast tanks of a dry bulk hull (`drawing.ballast_spaces`): the wing tanks hatched in plan,
/// the double bottom named in elevation. The section draws them in [`hold_ballast_section`].
fn hold_ballast(c: &mut Canvas, h: &Hull, view: View, labels: &mut Vec<(String, [f64; 2])>) {
    let Containment::Hold { ballast, .. } = &h.containment else {
        return;
    };
    for sp in ballast {
        let (lo, hi) = sp.extent();
        match (sp.kind.as_str(), view) {
            ("wing_ballast", View::Plan) => {
                c.hatch(
                    "HATCH",
                    &[
                        [sp.x_aft, lo[0]],
                        [sp.x_fwd, lo[0]],
                        [sp.x_fwd, hi[0]],
                        [sp.x_aft, hi[0]],
                    ],
                    "lines",
                );
                if sp.side == "port" {
                    labels.push((
                        label!("draw-wbt-both-sides", "WBT (BOTH SIDES)"),
                        [sp.mid_x(), (lo[0] + hi[0]) / 2.0],
                    ));
                }
            }
            ("double_bottom_ballast", View::Elevation) if sp.side == "port" => labels.push((
                label!("draw-double-bottom", "DOUBLE BOTTOM"),
                [sp.x_fwd - 3.0 * c.k, (lo[1] + hi[1]) / 2.0],
            )),
            _ => {}
        }
    }
}

/// The double bottom and wing ballast tanks of a dry bulk hull at the middle of its cargo zone,
/// hatched and named, as the liquid bulk section does.
fn hold_ballast_section(c: &mut Canvas, h: &Hull, labels: &mut Vec<(String, [f64; 2])>) {
    let Containment::Hold { ballast, .. } = &h.containment else {
        return;
    };
    let cut = (h.stern_void + h.loa - h.bow_void) / 2.0;
    for sp in ballast.iter().filter(|s| {
        (s.kind == "double_bottom_ballast" || s.kind == "wing_ballast")
            && s.x_aft <= cut
            && s.x_fwd > cut
    }) {
        c.polyline("OUTLINE", &sp.section, true);
        c.hatch("HATCH", &sp.section, "lines");
        let (lo, hi) = sp.extent();
        let mid = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        match sp.kind.as_str() {
            "double_bottom_ballast" if sp.side == "port" => {
                labels.push((label!("draw-double-bottom", "DOUBLE BOTTOM"), mid));
            }
            "wing_ballast" if sp.side == "port" => {
                labels.push((label!("draw-wbt-both-sides", "WBT (BOTH SIDES)"), mid));
            }
            _ => {}
        }
    }
}

/// The typical section of a liquid bulk hull, at the first cargo tank: the cargo tanks, with the
/// centreline bulkhead where the tank is split, the double bottom and the double side hatched.
fn liquid_section(
    c: &mut Canvas,
    spaces: &[DrawSpace],
    heating: bool,
    labels: &mut Vec<(String, [f64; 2])>,
) {
    let Some(first) = spaces
        .iter()
        .filter(|s| s.kind == "cargo_tank")
        .map(|s| s.x_aft)
        .min_by(|a, b| a.total_cmp(b))
    else {
        return;
    };
    let tank_here = |s: &DrawSpace| s.kind == "cargo_tank" && s.x_aft == first;
    let ballast_here = |s: &DrawSpace| {
        (s.kind == "double_bottom_ballast" || s.kind == "wing_ballast")
            && s.x_aft <= first
            && s.x_fwd > first
    };
    for sp in spaces.iter().filter(|s| tank_here(s) || ballast_here(s)) {
        let (lo, hi) = sp.extent();
        let layer = if tank_here(sp) { "OBJECT" } else { "OUTLINE" };
        c.polyline(layer, &sp.section, true);
        if ballast_here(sp) {
            c.hatch("HATCH", &sp.section, "lines");
        }
        let mid = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        match sp.kind.as_str() {
            "cargo_tank" if sp.side != "starboard" => {
                let split = sp.side == "port";
                labels.push((
                    if split {
                        label!("draw-cargo-tank-pair", "CARGO TANKS (P AND S)")
                    } else {
                        label!("draw-cargo-tank", "CARGO TANK")
                    },
                    mid,
                ));
                if split {
                    labels.push((
                        label!("draw-centreline-bulkhead", "CENTRELINE BULKHEAD"),
                        [0.0, mid[1]],
                    ));
                }
                if heating {
                    labels.push((
                        label!("draw-cargo-heating", "CARGO HEATING FITTED"),
                        [mid[0], lo[1] + 0.25 * (hi[1] - lo[1])],
                    ));
                }
            }
            "double_bottom_ballast" if sp.side == "port" => {
                labels.push((label!("draw-double-bottom", "DOUBLE BOTTOM"), mid));
            }
            "wing_ballast" if sp.side == "port" => {
                labels.push((label!("draw-wbt-both-sides", "WBT (BOTH SIDES)"), mid));
            }
            _ => {}
        }
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
            // The wheelhouse raised for the sightline: a thin chain outline, named RAISED. Only an
            // Option with a sightline result (the new stern) draws it: the drawings of a file saved
            // before stay as they were.
            if let (Some((floor, top)), true) = (a.wheelhouse_raised, h.blind.is_some()) {
                c.polyline(
                    "CENTRE",
                    &[
                        [a.wheelhouse_x0, floor],
                        [a.wheelhouse_x1, floor],
                        [a.wheelhouse_x1, top],
                        [a.wheelhouse_x0, top],
                    ],
                    true,
                );
                c.leader(
                    &label!("draw-wheelhouse-raised", "RAISED"),
                    [a.wheelhouse_x1, top],
                    [a.wheelhouse_x1 + 4.0 * k, top + 6.0 * k],
                );
            }
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
    // The Notch of an ATB barge, and the tug in it (issue #363): the Notch's end wall is a hidden
    // line (the Notch is on the centre line), the tug a side outline at its own draught.
    if let Some(a) = &h.atb {
        c.line(
            "HIDDEN",
            [a.notch_depth, 0.0],
            [a.notch_depth, h.depth],
            None,
        );
        if let Some((length, _, draught, depth)) = a.tug {
            let (bow, keel) = (a.notch_depth - ATB_COUPLER_GAP_M, h.draught - draught);
            c.polyline(
                "OBJECT",
                &[
                    [bow - length, keel],
                    [bow, keel],
                    [bow, keel + depth],
                    [bow - length, keel + depth],
                ],
                true,
            );
        }
    }
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
    if let Some(t) = h.ballast_line(c.k) {
        c.line(
            "WATER",
            [h.stern_x_at_height(t), t],
            [h.x_at_height(t), t],
            Some("dashed"),
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
        Containment::Liquid { spaces, .. } => {
            // every Space under the deck: its extent in x and its height range
            for sp in spaces
                .iter()
                .filter(|sp| sp.side != "starboard" || !sp.is_tank())
            {
                if sp.kind == "double_bottom_ballast" || sp.kind == "wing_ballast" {
                    continue;
                }
                let (lo, hi) = sp.extent();
                c.polyline(
                    "HIDDEN",
                    &[
                        [sp.x_aft, lo[1]],
                        [sp.x_fwd, lo[1]],
                        [sp.x_fwd, hi[1]],
                        [sp.x_aft, hi[1]],
                    ],
                    true,
                );
            }
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
    hold_bulkheads(&mut c, h, View::Elevation, &mut labels);
    hold_ballast(&mut c, h, View::Elevation, &mut labels);
    liquid_labels(&mut c, h, View::Elevation, &mut labels);
    tunnel_stern(&mut c, h, View::Elevation);
    // Issue #409: the top of the structure and the loaded air draft under it.
    if let Some(top) = h.air_top {
        c.line("ZONE", [0.0, top], [h.loa, top], None);
        labels.push((
            label!(
                "draw-air-draft",
                "AIR DRAFT {draught}",
                draught = metres(top - h.draught)
            ),
            [h.loa, top],
        ));
    }
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
    if let Some(t) = h.ballast_line(c.k) {
        labels.push((
            label!(
                "draw-ballast-draught",
                "BALLAST DRAUGHT {draught}",
                draught = metres(t)
            ),
            [h.x_at_height(t), t],
        ));
    }
    c.label_column(h.loa + 6.0 * c.k, &mut labels);
    c
}

/// The tug in the Notch, in plan: its outline with a pointed bow, the bow at the end of the Notch, and
/// the coupler as two short bars across the gap between the tug's bow and the Notch's end wall
/// (plain lines: the standard's symbol library has no coupler yet).
fn atb_tug_plan(c: &mut Canvas, a: &AtbDraw) {
    let Some((length, beam, _, _)) = a.tug else {
        return;
    };
    let (d, half) = (a.notch_depth, beam / 2.0);
    // The tug's bow stands a little off the end wall: the coupler's gap.
    let bow = d - ATB_COUPLER_GAP_M;
    let stern = bow - length;
    let shoulder = bow - (length * ATB_BOW_TAPER).min(2.0 * half);
    c.polyline(
        "OBJECT",
        &[
            [stern, -half],
            [shoulder, -half],
            [bow, 0.0],
            [shoulder, half],
            [stern, half],
        ],
        true,
    );
    for y in [-half / 2.0, half / 2.0] {
        c.line("OBJECT", [bow, y], [d, y], None);
    }
}

fn plan(h: &Hull, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    let (y0, y1) = (-h.beam / 2.0, h.beam / 2.0);
    match &h.atb {
        // The barge of an ATB: the Notch is cut at the transom (issue #363), full depth.
        Some(a) => {
            let (d, w) = (a.notch_depth, a.notch_width / 2.0);
            c.polyline(
                "OUTLINE",
                &[
                    [0.0, y0],
                    [h.loa, y0],
                    [h.loa, y1],
                    [0.0, y1],
                    [0.0, w],
                    [d, w],
                    [d, -w],
                    [0.0, -w],
                ],
                true,
            );
            atb_tug_plan(&mut c, a);
        }
        None => c.polyline(
            "OUTLINE",
            &[[0.0, y0], [h.loa, y0], [h.loa, y1], [0.0, y1]],
            true,
        ),
    }
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
        Containment::Liquid { spaces, .. } => {
            for sp in spaces {
                let (lo, hi) = sp.extent();
                match sp.kind.as_str() {
                    // the wing ballast tanks are hatched; the rest stands under the deck
                    "wing_ballast" => c.hatch(
                        "HATCH",
                        &[
                            [sp.x_aft, lo[0]],
                            [sp.x_fwd, lo[0]],
                            [sp.x_fwd, hi[0]],
                            [sp.x_aft, hi[0]],
                        ],
                        "lines",
                    ),
                    "double_bottom_ballast" => {}
                    _ => c.polyline(
                        "HIDDEN",
                        &[
                            [sp.x_aft, lo[0]],
                            [sp.x_fwd, lo[0]],
                            [sp.x_fwd, hi[0]],
                            [sp.x_aft, hi[0]],
                        ],
                        true,
                    ),
                }
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
    hold_bulkheads(&mut c, h, View::Plan, &mut labels);
    hold_ballast(&mut c, h, View::Plan, &mut labels);
    liquid_labels(&mut c, h, View::Plan, &mut labels);
    tunnel_stern(&mut c, h, View::Plan);
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
    // the weather shield: its sides stand on the wing plate (or the deck), its top spans the beam
    if let Some((foot, top)) = h.weather_shield() {
        c.polyline(
            "ZONE",
            &[[-half, foot], [-half, top], [half, top], [half, foot]],
            false,
        );
        labels.push((
            label!("draw-weather-shield", "WEATHER SHIELD"),
            [-half, top],
        ));
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

/// The hull's outline in the midship section, from the port deck edge round the keel to the
/// starboard deck edge: the hull's own half-section mirrored (issue #450), else the rectangle. A
/// point on a straight line between its neighbours is left out, so a flat bottom is one line.
fn midship_outline(h: &Hull) -> Vec<[f64; 2]> {
    let half = h.beam / 2.0;
    let Some(starboard) = h.midship_outline.as_ref().filter(|p| p.len() >= 2) else {
        return vec![[-half, h.depth], [-half, 0.0], [half, 0.0], [half, h.depth]];
    };
    let mut pts: Vec<[f64; 2]> = starboard.iter().rev().map(|p| [-p[0], p[1]]).collect();
    pts.extend(starboard.iter().skip(usize::from(starboard[0][0] == 0.0)));
    let mut out: Vec<[f64; 2]> = Vec::with_capacity(pts.len());
    for p in pts {
        if out.last() == Some(&p) {
            continue;
        }
        if let [.., a, b] = out[..] {
            let cross = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
            if cross == 0.0 {
                out.pop();
            }
        }
        out.push(p);
    }
    out
}

fn midship_section(h: &Hull, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    let half = h.beam / 2.0;
    c.polyline("OUTLINE", &midship_outline(h), false);
    section_deck_line(&mut c, h);
    c.line("WATER", [-half, h.draught], [half, h.draught], None);
    if let Some(t) = h.light_draught {
        c.line("WATER", [-half, t], [half, t], Some("chain_double_dashed"));
    }
    if let Some(t) = h.ballast_line(c.k) {
        c.line("WATER", [-half, t], [half, t], Some("dashed"));
    }
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
    match &h.containment {
        Containment::Hold {
            top_width,
            bottom_width,
            inner_bottom,
            coaming,
            ..
        } => {
            let bottom = *inner_bottom;
            // the inner bottom across the hull; the hold side runs on up to the coaming top, where
            // the volume model takes the top width
            c.line("OUTLINE", [-half, bottom], [half, bottom], None);
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
                    [-top_width / 2.0, h.depth + coaming],
                    [-bottom_width / 2.0, bottom],
                    [bottom_width / 2.0, bottom],
                    [top_width / 2.0, h.depth + coaming],
                ],
                false,
            );
            hold_ballast_section(&mut c, h, &mut labels);
        }
        Containment::Liquid { spaces, heating } => {
            liquid_section(&mut c, spaces, *heating, &mut labels);
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
    if let Some(t) = h.ballast_line(c.k) {
        labels.push((
            label!(
                "draw-ballast-draught",
                "BALLAST DRAUGHT {draught}",
                draught = metres(t)
            ),
            [-half, t],
        ));
    }
    // Issue #409: the top of the structure and the loaded air draft under it, on the right.
    if let Some(top) = h.air_top {
        c.line("ZONE", [-half, top], [half, top], None);
        if !c.try_dim_vertical([half, h.draught], [half, top], -3.0 * DIM_OFFSET_MM) {
            labels.push((
                label!(
                    "draw-air-draft",
                    "AIR DRAFT {draught}",
                    draught = metres(top - h.draught)
                ),
                [-half, top],
            ));
        }
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
    (SHEET.0, SHEET.1, SHEET.2, SHEET.3 - TITLE_BLOCK_MM - strip)
}

fn fits(window: [f64; 4], n: u32, strip: f64) -> bool {
    let k = 1000.0 / f64::from(n);
    let (_, _, fw, fh) = frame(strip);
    (window[2] - window[0]) * k <= fw + 1e-9 && (window[3] - window[1]) * k <= fh + 1e-9
}

/// The width of the supplementary title of the title block, less its padding (iso7200: 110 mm, 1 mm
/// each side), and the height of its text.
const SUPPLEMENTARY_MM: f64 = 108.0;
const SUPPLEMENTARY_TEXT_MM: f64 = 3.5;
/// The same for the document number (iso7200: 70 mm at 5 mm).
const NUMBER_MM: f64 = 68.0;
const NUMBER_TEXT_MM: f64 = 5.0;

fn fits_mm(text: &str, height: f64, width: f64) -> bool {
    ovcd_draw::rules::text_width(text, height).is_some_and(|w| w <= width)
}

/// The Option name as the title block shows it: plain capitals, or "OPTION" for none.
fn shown_name(option: Option<&str>) -> String {
    let name = option.map(plain_capitals).unwrap_or_default();
    if name.is_empty() {
        "OPTION".to_string()
    } else {
        name
    }
}

/// Can the drawings name this Option? Its name, in the capitals of the title block, must fit the
/// supplementary title on one line (clause 14.6: a text is never cut, shortened or made smaller to
/// fit). If not, the message names the field and how much of this name fits (issue #660).
pub fn option_name_fits(name: &str) -> Result<(), String> {
    let shown = shown_name(Some(name));
    if fits_mm(&shown, SUPPLEMENTARY_TEXT_MM, SUPPLEMENTARY_MM) {
        return Ok(());
    }
    let fit = (1..=name.chars().count())
        .take_while(|&n| {
            let part: String = name.chars().take(n).collect();
            fits_mm(
                &shown_name(Some(&part)),
                SUPPLEMENTARY_TEXT_MM,
                SUPPLEMENTARY_MM,
            )
        })
        .last()
        .unwrap_or(0);
    Err(format!(
        "The Option name '{name}' is too wide for the drawing title block, so the drawings cannot show it. Use a shorter name: the first {fit} characters of this one fit."
    ))
}

/// The document number of a sheet (spec 0003): `<OPTION>-<KIND>-<n>`. When that is too wide for its
/// cell, the Option part is a short id of the name, `OPTION-<6 hex digits of its SHA-256>`, the
/// same on every run; the full name stays in the supplementary title (issue #660).
fn document_number(name: &str, kind: &str, sheet: u32) -> String {
    let slug: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let number = format!("{slug}-{kind}-{sheet}");
    if fits_mm(&number, NUMBER_TEXT_MM, NUMBER_MM) {
        return number;
    }
    use sha2::{Digest, Sha256};
    let id: String = Sha256::digest(name.as_bytes())[..3]
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect();
    format!("OPTION-{id}-{kind}-{sheet}")
}

/// The given fields of the title block (clause 14.4.1): the view, the Option and the sheet number
/// of three, with `kind` the document type in the number. The scale, the status and the sheet size
/// are derived by the standard. `what` follows the name in the supplementary title when both fit.
fn title_fields(
    h: &Hull,
    title: &str,
    kind: &str,
    sheet: u32,
    what: &str,
) -> Result<Value, String> {
    if let Some(name) = h.option_name.as_deref() {
        option_name_fits(name)?;
    }
    let name = shown_name(h.option_name.as_deref());
    let mut fields = json!({
        "owner": label!("draw-owner", "OPEN VESSEL CONCEPT DESIGN"),
        "title": title,
        "document_number": document_number(&name, kind, sheet),
        "document_type": "GA",
        "sheet_number": sheet,
        "sheet_count": 3,
    });
    // 110 mm wide at 3.5 mm: about 40 capitals fit.
    let supplementary = format!("{name}; {what}");
    if fits_mm(&supplementary, SUPPLEMENTARY_TEXT_MM, SUPPLEMENTARY_MM) {
        fields["supplementary_title"] = json!(supplementary);
    } else {
        fields["supplementary_title"] = json!(name);
    }
    Ok(fields)
}

/// The note that names the Rule Sets the Option was checked against, when it fits its line.
fn rule_sets_note(sets: &[String]) -> Option<String> {
    if sets.is_empty() {
        return None;
    }
    let list = sets.join(", ").replace('@', " ");
    let note = label!("draw-note-rule-sets", "RULE SETS: {sets}.", sets = list);
    if note_fits(&note) {
        return Some(note);
    }
    Some(label!(
        "draw-note-rule-sets-count",
        "{count} RULE SETS.",
        count = sets.len()
    ))
}

/// Build one view at the largest scale at which everything fits, no larger than 1:`from` (the common
/// scale of the three views, issue #409). Gives the document and the ratio it was drawn at.
fn document(
    h: &Hull,
    title: &str,
    build: fn(&Hull, u32) -> Canvas,
    from: u32,
    sheet: u32,
) -> Result<(Value, u32), String> {
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
    if let Some(sets) = rule_sets_note(&h.rule_sets) {
        notes.push(sets);
    }
    if let Some((light, laden, limit)) = h.blind {
        notes.push(label!(
            "draw-note-blind-distance",
            "BLIND DISTANCE AHEAD OF THE BOW: LIGHT {light} M, LADEN {laden} M, LIMIT {limit} M.",
            light = fixed(light, 1),
            laden = fixed(laden, 1),
            limit = fixed(limit, 0)
        ));
    }
    notes.push(label!(
        "draw-note-scope",
        "FOR CONCEPT AND PRELIMINARY DESIGN ONLY, NOT FOR APPROVAL."
    ));
    notes.extend(failed_notes(&h.failed_checks));
    place(
        notes,
        title_fields(h, title, "GA", sheet, &plain_capitals(&h.vessel))?,
        from,
        &|n| build(h, n),
    )
}

/// Lay one canvas out on an A3 sheet with its notes and title block fields, at the largest scale at
/// which it fits, no larger than 1:`from`. Gives the document and the ratio it was drawn at.
fn place(
    notes: Vec<String>,
    fields: Value,
    from: u32,
    build: &dyn Fn(u32) -> Canvas,
) -> Result<(Value, u32), String> {
    let strip = strip_mm(notes.len());
    for n in RATIOS.into_iter().filter(|n| *n >= from) {
        let canvas = build(n);
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
        return Ok((
            json!({
                "standard_version": "1.0",
                "sheet": {"size": "A3", "orientation": "landscape"},
                "layers": layers,
                "status": "concept",
                "title_block": {"template": "iso7200", "fields": fields},
                "strips": [{"id": "notes", "edge": "bottom", "size_mm": strip, "content": "notes"}],
                "notes": notes,
                "viewports": [{
                    "id": "main",
                    "frame": {"x_mm": fx, "y_mm": fy, "width_mm": fw, "height_mm": fh},
                    "window": {"xmin_m": window[0], "ymin_m": window[1], "xmax_m": window[2], "ymax_m": window[3]},
                    "scale": n,
                    "scale_bar": {},
                    "entities": canvas.entities,
                }],
            }),
            n,
        ));
    }
    Err("the hull does not fit any scale of the standard on an A3 sheet".to_string())
}

/// The three views of an Option's results (the `simulate` result object).
///
/// The side elevation and the plan, which show the same length, are drawn at one scale: the largest
/// at which both fit (issue #409), so that a length scaled off one view is the same off the other.
/// The typical section is a detail of the cargo containment: at the scale of the hull it would be a
/// few millimetres wide, so it keeps the largest scale at which it fits and says so in its scale
/// bar.
pub fn ga_views(results: &Value) -> Result<GaViews, String> {
    ga_views_for(results, None)
}

/// As [`ga_views`], with the name of the Option for the title block of each sheet.
pub fn ga_views_for(results: &Value, option: Option<&str>) -> Result<GaViews, String> {
    let mut hull = read_hull(results)?;
    hull.option_name = option.map(str::to_string);
    let elevation_title = label!("draw-view-side-elevation", "SIDE ELEVATION");
    let plan_title = label!("draw-view-plan", "PLAN");
    // A view that needs a smaller scale than the other sets the scale for both; drawing a view at a
    // smaller scale can change its labels, so the search runs until the two agree.
    let mut common = RATIOS[0];
    let (elevation, plan_view) = loop {
        let e = document(&hull, &elevation_title, side_elevation, common, 1)?;
        let p = document(&hull, &plan_title, plan, common, 2)?;
        if e.1 == p.1 {
            break (e.0, p.0);
        }
        common = e.1.max(p.1);
    };
    let (section, _) = document(
        &hull,
        &label!("draw-view-midship-section", "TYPICAL SECTION"),
        midship_section,
        RATIOS[0],
        3,
    )?;
    Ok(GaViews {
        side_elevation: elevation,
        plan: plan_view,
        midship_section: section,
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

    fn gas_results() -> Value {
        let text = include_str!("../tests/fixtures/gas_igc_2g_self_propelled.vdesign");
        let design = crate::check(text, &crate::Profile::built_in())
            .design
            .expect("a valid Design");
        match crate::run_design(&design).remove(0).outcome {
            crate::RunOutcome::Results { results, .. } => results,
            other => panic!("no results: {other:?}"),
        }
    }

    #[test]
    fn the_drawn_tank_is_as_long_as_the_cores_tank() {
        // Issue #406: the drawing lays the tanks out from the results' own tank length and gaps.
        let results = gas_results();
        let hull = read_hull(&results).unwrap();
        let (length, starts, _) = hull.tank_layout().expect("a tank layout");
        let tank = &results["drawing"]["tank"];
        assert_eq!(Some(length), tank["overall_tank_length_m"].as_f64());
        let gap = tank["tank_gap_longitudinal_m"].as_f64().unwrap();
        for pair in starts.windows(2) {
            assert!((pair[1] - pair[0] - (length + gap)).abs() < 1e-9);
        }
        // The tanks fill the cargo zone: the last one ends where the zone does.
        let end = starts.last().unwrap() + length;
        assert!((end - (hull.loa - hull.bow_void)).abs() < 1e-6, "{end}");
    }

    #[test]
    fn results_without_the_tank_lengths_are_refused_not_drawn_from_a_stand_in() {
        let mut results = gas_results();
        results["drawing"]["tank"]
            .as_object_mut()
            .unwrap()
            .remove("overall_tank_length_m");
        let e = read_hull(&results).err().expect("refused");
        assert!(e.contains("overall_tank_length_m"), "{e}");
    }
}
