//! The hull steel of a double-hull barge built up from its structural members (issue #722).
//!
//! **Concept and preliminary design only (ADR 0011). This is not a structural approval.** Each
//! member is one record: where it sits in the hull (x, y and z bounds), its plate thickness, its
//! stiffeners (direction, spacing and a named section) and the primary members that support it.
//! The scantlings are the minimum scantlings of the ABS Rules for Building and Classing Steel Barges
//! (2021 edition, the free archive copy), cited per member. The weight is the sum over the records:
//! plate area x thickness x density, plus the stiffeners and the primary members, plus one named
//! allowance for brackets, welds and small items. No structure factor is used.
//!
//! What it is not: no finite element analysis, no buckling check, no fatigue, no corrosion margin
//! beyond what the rule formulas hold, no openings or lightening holes, no local strengthening
//! under the tank saddles, no machinery seats, no superstructure. Those are follow-up tickets.
//!
//! The rule length L is taken as the length overall (an Estimate: ABS 3-1-1/3 takes it on the
//! summer load line). Ordinary-strength steel throughout (Q = 1, q = 1). Unrestricted service:
//! ABS gives special consideration for restricted service (3-2-1 Table note), not modelled.

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use libm::{ceil, fabs, floor, pow, sqrt};

/// The rule edition every scantling here is taken from.
pub const RULE_EDITION: &str = "ABS Rules for Building and Classing Steel Barges, 2021";

/// The allowance for brackets, collars, welds and small items, as a share (%) of the net member
/// steel. An Estimate: no primary source was found that gives one figure. Weld metal alone is
/// about 1 to 3 % of the structure at the design stage (a secondary source; see the PR of issue
/// #722), and brackets, chocks and collars are not in the members. Open question for the
/// maintainer on issue #722.
pub const BRACKETS_WELDS_ALLOWANCE_PCT: f64 = 5.0;

/// Default spacing of the longitudinals (mm). An Estimate inside the 610 to 915 mm range that
/// ABS 5-1-2/5.17 uses for inner-bottom plating.
pub const DEFAULT_LONGITUDINAL_SPACING_MM: f64 = 700.0;

/// Default spacing of the floors and web frames (m): the largest that ABS 5-1-2/5.7.1 allows for
/// the solid floors of a longitudinally framed double bottom.
pub const DEFAULT_WEB_SPACING_M: f64 = 3.66;

/// Height of a tank vent or overflow above the deck (m): the 760 mm standard height of ABS 3-2-10
/// Table 3. It sets the design head of the ballast tanks (ABS 5-2-2/3.7, 5-2-2/7).
pub const OVERFLOW_HEIGHT_M: f64 = 0.76;

/// What the panel model needs to know about the hull. Distances in metres; x from the aft end,
/// y from the centreline (the model is symmetric), z from the keel.
#[derive(Debug, Clone, PartialEq)]
pub struct DoubleHullSpec {
    pub loa_m: f64,
    pub beam_m: f64,
    pub depth_m: f64,
    /// Block coefficient for the hull girder section modulus (ABS 3-1-1/31).
    pub block_coefficient: f64,
    /// The cargo zone, where the double side (wing tanks) runs.
    pub cargo_zone_aft_m: f64,
    pub cargo_zone_fwd_m: f64,
    /// Double bottom height; it runs the full length.
    pub double_bottom_m: f64,
    /// Width of the double side (each side) in the cargo zone; 0 for a single side.
    pub wing_width_m: f64,
    /// Height of the wing tank top above the keel. Taken at most D.
    pub wing_top_m: f64,
    /// Total width of the deck cut-outs at D in the cargo zone (tanks standing through the deck).
    pub deck_opening_width_m: f64,
    /// Total width of the cut-outs in the wing deck (the plate at the wing tank top).
    pub wing_deck_opening_width_m: f64,
    /// Number of tanks (holds) along the cargo zone: it sets the hold bulkheads.
    pub holds_along: usize,
    /// Number of tanks abreast: it sets the span of the deck transverses.
    pub tanks_abreast: usize,
    /// Hull steel density (t/m3).
    pub steel_density: f64,
}

/// Spacings and allowances the designer chooses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScantlingChoices {
    pub longitudinal_spacing_mm: f64,
    pub web_spacing_m: f64,
    pub allowance_pct: f64,
}

impl Default for ScantlingChoices {
    fn default() -> Self {
        Self {
            longitudinal_spacing_mm: DEFAULT_LONGITUDINAL_SPACING_MM,
            web_spacing_m: DEFAULT_WEB_SPACING_M,
            allowance_pct: BRACKETS_WELDS_ALLOWANCE_PCT,
        }
    }
}

/// Which way a member's stiffeners run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StiffenerDirection {
    Longitudinal,
    Transverse,
    Vertical,
    None,
}

impl StiffenerDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Longitudinal => "longitudinal",
            Self::Transverse => "transverse",
            Self::Vertical => "vertical",
            Self::None => "none",
        }
    }
}

/// A stiffener or primary member section: a flat bar or a tee, without its attached plate.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub name: String,
    pub web_mm: (f64, f64),
    /// Flange width and thickness (mm); (0, 0) for a flat bar.
    pub flange_mm: (f64, f64),
}

impl Section {
    fn flat(h: f64, t: f64) -> Self {
        Self {
            name: format!("FB {h:.0}x{t:.0}"),
            web_mm: (h, t),
            flange_mm: (0.0, 0.0),
        }
    }

    fn tee(h: f64, tw: f64, bf: f64, tf: f64) -> Self {
        Self {
            name: format!("T {h:.0}x{tw:.1}+{bf:.0}x{tf:.0}"),
            web_mm: (h, tw),
            flange_mm: (bf, tf),
        }
    }

    /// The cross-section area of the section alone (cm2).
    pub fn area_cm2(&self) -> f64 {
        (self.web_mm.0 * self.web_mm.1 + self.flange_mm.0 * self.flange_mm.1) / 100.0
    }

    /// The mass per metre of the section alone (t/m) at a steel density (t/m3).
    pub fn mass_per_m(&self, density: f64) -> f64 {
        self.area_cm2() * 1e-4 * density
    }

    /// The smaller section modulus (cm3) of the section with its attached plate: thickness
    /// `plate_mm`, effective width `width_mm` (ABS 3-1-2/13.5.1: at most one frame space for a
    /// stiffener).
    pub fn sm_with_plate_cm3(&self, plate_mm: f64, width_mm: f64) -> f64 {
        // Parts as (area mm2, centroid mm above the plate's outer face, own inertia mm4).
        let (h, tw) = self.web_mm;
        let (bf, tf) = self.flange_mm;
        let parts = [
            (
                width_mm * plate_mm,
                plate_mm / 2.0,
                width_mm * pow(plate_mm, 3.0) / 12.0,
            ),
            (h * tw, plate_mm + h / 2.0, tw * pow(h, 3.0) / 12.0),
            (bf * tf, plate_mm + h + tf / 2.0, bf * pow(tf, 3.0) / 12.0),
        ];
        let area: f64 = parts.iter().map(|p| p.0).sum();
        let na = parts.iter().map(|p| p.0 * p.1).sum::<f64>() / area;
        let inertia: f64 = parts.iter().map(|p| p.2 + p.0 * pow(p.1 - na, 2.0)).sum();
        let top = plate_mm + h + tf - na;
        let far = if top > na { top } else { na };
        inertia / far / 1000.0
    }
}

/// The stiffener series the model picks from: flat bars within the web ratio of ABS 3-1-2/13.5.2
/// (15 without a flange) and tees within it (50 with a flange). The dimensions are a chosen series
/// (an Estimate); their properties are worked out from the geometry, not read from a catalogue.
pub fn stiffener_series() -> Vec<Section> {
    let mut v = alloc::vec![
        Section::flat(100.0, 7.0),
        Section::flat(120.0, 8.0),
        Section::flat(150.0, 10.0),
        Section::flat(180.0, 12.0),
        Section::flat(200.0, 14.0),
    ];
    for (h, tw) in [
        (200.0, 9.0),
        (250.0, 10.0),
        (300.0, 11.0),
        (350.0, 11.0),
        (400.0, 12.0),
        (450.0, 12.0),
        (500.0, 13.0),
        (550.0, 13.0),
        (600.0, 14.0),
    ] {
        let bf = floor(0.3 * h / 10.0 + 0.5) * 10.0;
        v.push(Section::tee(h, tw, bf, tw + 3.0));
    }
    v.sort_by(|a, b| a.area_cm2().total_cmp(&b.area_cm2()));
    v
}

/// The lightest section of the series with at least `sm_cm3` on the given plate; the largest when
/// none is enough (the result then says so in the member's note).
pub fn pick_stiffener(sm_cm3: f64, plate_mm: f64, spacing_mm: f64) -> (Section, bool) {
    let series = stiffener_series();
    for s in &series {
        if s.sm_with_plate_cm3(plate_mm, spacing_mm) >= sm_cm3 {
            return (s.clone(), true);
        }
    }
    (series.last().cloned().expect("a series"), false)
}

/// A primary member (floor-like web with a face plate) sized for a section modulus: web depth and
/// thickness given by the rule's proportions, the flange the least (in 1 mm steps of thickness,
/// width 10 x thickness, within the 14 x thickness of ABS 3-2-4/3.5) that gives the modulus with
/// the effective attached plate.
pub fn design_primary(
    sm_cm3: f64,
    web_depth_mm: f64,
    web_mm: f64,
    plate_mm: f64,
    eff_mm: f64,
) -> Section {
    for tf in 0..=60 {
        let tf = tf as f64;
        let s = if tf == 0.0 {
            Section::flat(web_depth_mm, web_mm)
        } else {
            Section::tee(web_depth_mm, web_mm, 10.0 * tf, tf)
        };
        if s.sm_with_plate_cm3(plate_mm, eff_mm) >= sm_cm3 {
            return s;
        }
    }
    Section::tee(web_depth_mm, web_mm, 600.0, 60.0)
}

/// One structural member of the hull: a record of the panel table.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelMember {
    pub member: String,
    /// x bounds from the aft end, y bounds from the centreline (one side; `count` says how many),
    /// z bounds from the keel (m).
    pub x_m: (f64, f64),
    pub y_m: (f64, f64),
    pub z_m: (f64, f64),
    /// How many of these there are (two sides, n floors).
    pub count: usize,
    /// Plate area of all of them (m2) and its thickness (mm).
    pub plate_area_m2: f64,
    pub plate_mm: f64,
    pub stiffener_direction: StiffenerDirection,
    pub stiffener_spacing_mm: f64,
    pub stiffener: Option<Section>,
    /// Total length of the stiffeners of all of them (m).
    pub stiffener_length_m: f64,
    /// The primary members that support this panel: their spacing (m) and section.
    pub primary_spacing_m: Option<f64>,
    pub primary: Option<String>,
    pub plate_t: f64,
    pub stiffeners_t: f64,
    pub total_t: f64,
    /// The rule paragraphs that set the scantlings.
    pub rule: String,
    /// "rule" when the rule formulas set the scantlings; "estimate" when a value is chosen.
    pub basis: &'static str,
    pub note: String,
}

/// The hull girder check of ABS 3-2-1/3.1 on the midship section.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HullGirder {
    /// Required section modulus SMR (cm2-m), with K = 1 (no still-water bending moment given).
    pub smr_cm2m: f64,
    /// Section modulus to the deck and to the bottom as built (cm2-m).
    pub sm_deck_cm2m: f64,
    pub sm_bottom_cm2m: f64,
    /// Thickness added to the deck and to the bottom over 0.4L amidships to meet SMR (mm).
    pub deck_added_mm: f64,
    pub bottom_added_mm: f64,
    /// Neutral axis above the keel (m) and moment of inertia (m4).
    pub neutral_axis_m: f64,
    pub inertia_m4: f64,
}

/// The panel model of one hull.
#[derive(Debug, Clone, PartialEq)]
pub struct HullPanelModel {
    pub members: Vec<PanelMember>,
    pub hull_girder: HullGirder,
    pub net_t: f64,
    pub allowance_pct: f64,
    pub allowance_t: f64,
    pub total_t: f64,
    pub rule_edition: &'static str,
}

/// C1 of ABS 3-2-1/3.1.
fn c1(l: f64) -> f64 {
    if l < 45.0 {
        4.11
    } else if l < 95.0 {
        16.33 * pow(l / 100.0, 2.0) - 15.47 * (l / 100.0) + 7.77
    } else if l <= 300.0 {
        10.75 - pow((300.0 - l) / 100.0, 1.5)
    } else if l < 350.0 {
        10.75
    } else {
        10.75 - pow((l - 350.0) / 150.0, 1.5)
    }
}

/// SMR of ABS 3-2-1/3.1 with K = 1 (cm2-m).
pub fn required_section_modulus(l: f64, b: f64, cb: f64) -> f64 {
    c1(l) * 0.01 * l * l * b * (cb + 0.7)
}

/// k of ABS 3-2-6/5.1 and 3-2-7/3.1 for a panel of aspect ratio a.
fn panel_k(a: f64) -> f64 {
    if a > 2.0 {
        1.0
    } else {
        let a = if a < 1.0 { 1.0 } else { a };
        (3.075 * sqrt(a) - 2.077) / (a + 0.272)
    }
}

/// Plating of a tank boundary, ABS 3-2-7/3.1 (mm): t = s k sqrt(q h)/254 + 2.5, at least 6.5 and
/// s/150 + 2.5.
pub fn tank_boundary_plate_mm(s_mm: f64, aspect: f64, h_m: f64) -> f64 {
    let t = s_mm * panel_k(aspect) * sqrt(h_m.max(0.0)) / 254.0 + 2.5;
    t.max(6.5).max(s_mm / 150.0 + 2.5)
}

/// Plating of a watertight bulkhead, ABS 3-2-6/5.1 (mm): t = s k sqrt(q h)/290 + 1.5, at least 6
/// and s/200 + 2.5.
pub fn watertight_bulkhead_plate_mm(s_mm: f64, aspect: f64, h_m: f64) -> f64 {
    let t = s_mm * panel_k(aspect) * sqrt(h_m.max(0.0)) / 290.0 + 1.5;
    t.max(6.0).max(s_mm / 200.0 + 2.5)
}

/// SM = 7.8 c h s l^2 (cm3), the stiffener formula of ABS 3-2-6/5.3, 3-2-7/3.3, 5-2-1/7, 5-2-2/3.5.
pub fn stiffener_sm(c: f64, h: f64, s_m: f64, l: f64) -> f64 {
    7.8 * c * h * s_m * l * l
}

/// SM = 4.74 c h s l^2 (cm3), the primary member formula of ABS 3-2-6/5.5, 3-2-7/3.5, 5-2-1/11.
pub fn primary_sm(c: f64, h: f64, s_m: f64, l: f64) -> f64 {
    4.74 * c * h * s_m * l * l
}

/// The scantling head h of ABS 5-2-1/3 for a tank whose overflow runs to the deck: h1 = ht + ha,
/// ht from the point to the deck at side, ha the larger of ha1 (by L) and the vent setting (none).
fn tank_head(l: f64, depth: f64, z: f64) -> f64 {
    let ha1 = if l <= 61.0 {
        1.22
    } else if l <= 122.0 {
        0.02 * l
    } else {
        2.44
    };
    (depth - z).max(0.0) + ha1
}

/// The watertight bulkhead head of ABS 3-2-6/5.3 and 5.5: the distance to the deck, taken as
/// 0.8 x it + 1.22 where it is under 6.10 m.
fn bulkhead_head(depth: f64, z: f64) -> f64 {
    let d = (depth - z).max(0.0);
    if d < 6.10 { 0.8 * d + 1.22 } else { d }
}

/// Web thickness of girders, chords and transverses, ABS 3-2-4/3.11 (mm): 1 mm per 100 mm of depth
/// plus 3 (at most 11.5), and not less than L/36 + 7.0 in tanks or L/36 + 5.5 clear of tanks
/// (L >= 75 m; the L < 75 m forms otherwise).
pub fn web_thickness_mm(l: f64, depth_mm: f64, in_tank: bool) -> f64 {
    let by_depth = (0.01 * depth_mm + 3.0).min(11.5);
    let l1 = l.max(29.0);
    let floor_t = match (in_tank, l >= 75.0) {
        (true, true) => l1 / 36.0 + 7.0,
        (true, false) => l1 / 20.0 + 5.33,
        (false, true) => l1 / 36.0 + 5.5,
        (false, false) => l1 / 60.0 + 6.35,
    };
    by_depth.max(floor_t)
}

/// Round a plate up to the next 0.5 mm, as plate is ordered.
fn order_mm(t: f64) -> f64 {
    ceil(t * 2.0 - 1e-9) / 2.0
}

struct Builder<'a> {
    spec: &'a DoubleHullSpec,
    ch: ScantlingChoices,
    members: Vec<PanelMember>,
}

/// A stiffened panel to add: geometry, plate, stiffeners.
struct Panel {
    member: String,
    x: (f64, f64),
    y: (f64, f64),
    z: (f64, f64),
    count: usize,
    area_each: f64,
    plate_mm: f64,
    dir: StiffenerDirection,
    /// Stiffener count per panel and the length of each.
    stiffeners: Vec<(Section, f64)>,
    primary_spacing: Option<f64>,
    primary: Option<String>,
    rule: String,
    basis: &'static str,
    note: String,
}

impl Builder<'_> {
    fn add(&mut self, p: Panel) {
        let rho = self.spec.steel_density;
        let plate_area = p.area_each * p.count as f64;
        let plate_t = plate_area * p.plate_mm / 1000.0 * rho;
        let mut st_t = 0.0;
        let mut st_len = 0.0;
        for (s, len) in &p.stiffeners {
            st_t += s.mass_per_m(rho) * len * p.count as f64;
            st_len += len * p.count as f64;
        }
        // The heaviest stiffener names the member's stiffener.
        let stiffener = p
            .stiffeners
            .iter()
            .map(|(s, _)| s)
            .max_by(|a, b| a.area_cm2().total_cmp(&b.area_cm2()))
            .cloned();
        self.members.push(PanelMember {
            member: p.member,
            x_m: p.x,
            y_m: p.y,
            z_m: p.z,
            count: p.count,
            plate_area_m2: plate_area,
            plate_mm: p.plate_mm,
            stiffener_direction: p.dir,
            stiffener_spacing_mm: if p.stiffeners.is_empty() {
                0.0
            } else {
                self.ch.longitudinal_spacing_mm
            },
            stiffener,
            stiffener_length_m: st_len,
            primary_spacing_m: p.primary_spacing,
            primary: p.primary,
            plate_t,
            stiffeners_t: st_t,
            total_t: plate_t + st_t,
            rule: p.rule,
            basis: p.basis,
            note: p.note,
        });
    }
}

/// The number of stiffeners across a panel of width `w_m` at spacing `s_mm`.
fn n_across(w_m: f64, s_mm: f64) -> usize {
    let n = floor(w_m * 1000.0 / s_mm + 1e-9);
    if n < 1.0 { 0 } else { n as usize }
}

/// Build the panel model of a double-hull barge.
pub fn double_hull_panels(spec: &DoubleHullSpec, ch: &ScantlingChoices) -> HullPanelModel {
    let l = spec.loa_m;
    let lr = l.min(225.0);
    let b = spec.beam_m;
    let d = spec.depth_m;
    let db = spec.double_bottom_m.clamp(0.0, d);
    let s = ch.longitudinal_spacing_mm;
    let sm = s / 1000.0;
    let web = ch.web_spacing_m;
    let lc = (spec.cargo_zone_fwd_m - spec.cargo_zone_aft_m).max(0.0);
    let w = spec.wing_width_m.clamp(0.0, b / 2.0);
    let zw = spec.wing_top_m.clamp(db, d);
    let mid = (0.3 * l, 0.7 * l);
    let ends_len = 0.6 * l;

    // --- Rule plate thicknesses ---
    // Bottom shell, ABS 3-2-2/3.5.1 (longitudinal framing), and at the ends 3-2-2/5.1.
    let t_bottom_mid = if lr <= 123.0 {
        0.045 * lr + 0.007 * s + 1.8
    } else {
        0.08 * lr + 0.007 * s - 2.5
    };
    let t_shell_end = if l <= 76.0 {
        0.055 * lr + 0.007 * s + 1.0
    } else {
        0.015 * lr + 0.01 * s + 2.2
    };
    // Side shell, ABS 3-2-2/3.1, less 0.5 mm with longitudinal bulkheads within 0.2B (3.1.1).
    let mut t_side_mid = if l <= 150.0 {
        0.07 * l + 0.007 * s
    } else {
        0.03 * l + 0.007 * s + 6.0
    };
    if w > 0.0 && w <= 0.2 * b {
        t_side_mid -= 0.5;
    }
    // Strength deck, ABS 3-2-3/1.1.2 (a) and (b), longitudinal beams; ends 3-2-3/1.7.
    let ta = if s <= 760.0 {
        0.009 * s + 2.4
    } else {
        0.006 * s + 4.7
    };
    let lb = l.min(183.0);
    let tb = s * (lb + 48.76) / (26.0 * lb + 8681.0);
    let t_deck_mid = ta.max(tb);
    let t_deck_end = 0.01 * s;
    // Inner bottom, ABS 5-2-2/3.3: 5-1-2/5.17 (less 1 mm, longitudinal framing) and 3-2-7/3.1
    // with the head of 5-2-1/3 (the double bottom is a ballast tank).
    let t_ib = (0.037 * lr + 0.009 * s - 0.5 - 1.0).max(tank_boundary_plate_mm(
        s,
        web / sm,
        tank_head(l, d, db),
    ));

    // --- Hull girder, ABS 3-2-1/3.1, on the midship section ---
    let smr = required_section_modulus(l, b, spec.block_coefficient);
    let deck_w = (b - spec.deck_opening_width_m).max(0.0);
    let wing_deck_w = if zw < d && w > 0.0 {
        (b - spec.wing_deck_opening_width_m)
            .max(0.0)
            .min(2.0 * w)
            .max(0.0)
    } else {
        0.0
    };
    let t_ws = |z: f64| tank_boundary_plate_mm(s, web / sm, tank_head(l, d, z));
    let t_inner_side = t_ws(db);
    let t_wing_deck = t_ws(zw);
    // Side girders of the double bottom, ABS 5-1-2/5.5: at most 4.6 m apart.
    let n_side_girders = (ceil((b / 2.0) / 4.6 - 1e-9) - 1.0).max(0.0) as usize;
    let t_cg = 0.056 * l + 5.5;
    let t_sg = 0.036 * l + 4.5;

    // Longitudinal stiffeners, chosen against the rule SM with the rule plate.
    let ha_ovf = 2.0 / 3.0 * OVERFLOW_HEIGHT_M;
    let ha_wing = ha_ovf.max(0.91);
    let l_floor = web.max(1.83); // ABS 5-1-2/5.13: l not under 1.83 m for L >= 61 m
    let bottom_long =
        pick_stiffener(stiffener_sm(1.34, d + ha_ovf, sm, l_floor), t_bottom_mid, s).0;
    let ib_long = pick_stiffener(
        stiffener_sm(1.0, tank_head(l, d, db), sm, web)
            .max(0.85 * stiffener_sm(1.30, d + ha_ovf, sm, l_floor)),
        t_ib,
        s,
    )
    .0;
    // Deck beams, ABS 5-1-1/3.1: h = 2.9 m (L >= 107) or 0.02L + 0.76; c by the hull girder k,
    // taken first as k = 1 (c = 0.945) and not revised (an Estimate).
    let h_deck = if l >= 107.0 { 2.9 } else { 0.02 * l + 0.76 };
    let c_deck = 1.0 / (1.709 - 0.651);
    let deck_long = pick_stiffener(stiffener_sm(c_deck, h_deck, sm, web), t_deck_mid, s).0;

    // Side and inner side longitudinals, one per stiffener level (ABS 5-2-2/7 -> 5-2-1/7, c = 1.25,
    // and 3-2-7/3.3, c = 0.90, for the longitudinal bulkhead).
    let levels = |z0: f64, z1: f64| -> Vec<f64> {
        let n = n_across(z1 - z0, s);
        (1..=n)
            .map(|i| z0 + i as f64 * sm)
            .filter(|z| *z < z1)
            .collect()
    };
    let side_levels = levels(0.0, d);
    let side_longs: Vec<Section> = side_levels
        .iter()
        .map(|z| {
            pick_stiffener(
                stiffener_sm(1.25, (d - z) + ha_wing, sm, web),
                t_side_mid,
                s,
            )
            .0
        })
        .collect();
    let inner_levels = if w > 0.0 { levels(db, zw) } else { Vec::new() };
    let inner_longs: Vec<Section> = inner_levels
        .iter()
        .map(|z| {
            pick_stiffener(
                stiffener_sm(0.90, tank_head(l, d, *z), sm, web),
                t_inner_side,
                s,
            )
            .0
        })
        .collect();
    let wing_deck_long = pick_stiffener(
        stiffener_sm(1.0, tank_head(l, d, zw), sm, web),
        t_wing_deck,
        s,
    )
    .0;

    // Section of the midship 0.4L: (area m2, z m, own inertia m4) for every longitudinal part.
    let section = |t_deck: f64, t_bot: f64| -> (f64, f64) {
        let mut parts: Vec<(f64, f64, f64)> = Vec::new();
        parts.push((b * t_bot / 1000.0, 0.0, 0.0));
        parts.push((
            n_across(b, s) as f64 * bottom_long.area_cm2() * 1e-4,
            0.1,
            0.0,
        ));
        if db > 0.0 {
            parts.push((b * t_ib / 1000.0, db, 0.0));
            parts.push((
                n_across(b, s) as f64 * ib_long.area_cm2() * 1e-4,
                db - 0.1,
                0.0,
            ));
            let a_cg = db * t_cg / 1000.0;
            parts.push((a_cg, db / 2.0, a_cg * db * db / 12.0));
            let a_sg = 2.0 * n_side_girders as f64 * db * t_sg / 1000.0;
            parts.push((a_sg, db / 2.0, a_sg * db * db / 12.0));
        }
        let a_side = 2.0 * d * t_side_mid / 1000.0;
        parts.push((a_side, d / 2.0, a_side * d * d / 12.0));
        for (z, st) in side_levels.iter().zip(&side_longs) {
            parts.push((2.0 * st.area_cm2() * 1e-4, *z, 0.0));
        }
        if w > 0.0 && zw > db {
            let h = zw - db;
            let a = 2.0 * h * t_inner_side / 1000.0;
            parts.push((a, db + h / 2.0, a * h * h / 12.0));
            for (z, st) in inner_levels.iter().zip(&inner_longs) {
                parts.push((2.0 * st.area_cm2() * 1e-4, *z, 0.0));
            }
            parts.push((wing_deck_w * t_wing_deck / 1000.0, zw, 0.0));
            parts.push((
                n_across(wing_deck_w, s) as f64 * wing_deck_long.area_cm2() * 1e-4,
                zw - 0.1,
                0.0,
            ));
        }
        parts.push((deck_w * t_deck / 1000.0, d, 0.0));
        parts.push((
            n_across(deck_w, s) as f64 * deck_long.area_cm2() * 1e-4,
            d - 0.1,
            0.0,
        ));
        let area: f64 = parts.iter().map(|p| p.0).sum();
        let na = parts.iter().map(|p| p.0 * p.1).sum::<f64>() / area;
        let i: f64 = parts.iter().map(|p| p.2 + p.0 * pow(p.1 - na, 2.0)).sum();
        (na, i)
    };
    let sm_of = |t_deck: f64, t_bot: f64| {
        let (na, i) = section(t_deck, t_bot);
        // m3 -> cm2-m: x 1e4.
        (i / (d - na) * 1e4, i / na * 1e4, na, i)
    };
    let mut deck_add = 0.0;
    let mut bottom_add = 0.0;
    for _ in 0..400 {
        let (sd, sb, _, _) = sm_of(t_deck_mid + deck_add, t_bottom_mid + bottom_add);
        if sd >= smr && sb >= smr {
            break;
        }
        if sd < smr {
            deck_add += 0.5;
        }
        if sb < smr {
            bottom_add += 0.5;
        }
    }
    let (sd, sb, na, inertia) = sm_of(t_deck_mid + deck_add, t_bottom_mid + bottom_add);
    let hull_girder = HullGirder {
        smr_cm2m: smr,
        sm_deck_cm2m: sd,
        sm_bottom_cm2m: sb,
        deck_added_mm: deck_add,
        bottom_added_mm: bottom_add,
        neutral_axis_m: na,
        inertia_m4: inertia,
    };

    let mut bld = Builder {
        spec,
        ch: *ch,
        members: Vec::new(),
    };
    let floors_desc = format!(
        "solid floors {:.1} mm x {:.2} m",
        order_mm(0.036 * l + 6.0),
        db
    );
    let webs_desc = |what: &str| format!("{what} at {web:.2} m");
    let n_b = n_across(b, s);

    // --- Outer shell and deck, midship 0.4L and ends (tapered: the mean of the two) ---
    let t_bm = order_mm(t_bottom_mid + bottom_add);
    let t_be = order_mm((t_bottom_mid + bottom_add + t_shell_end) / 2.0);
    for (name, x, len, t, rule) in [
        (
            "bottom shell, midship 0.4L",
            mid,
            0.4 * l,
            t_bm,
            "ABS 3-2-2/3.5.1; 3-2-1/3.1 hull girder",
        ),
        (
            "bottom shell, ends",
            (0.0, l),
            ends_len,
            t_be,
            "ABS 3-2-2/5.1, tapered from 3-2-2/3.5.1",
        ),
    ] {
        bld.add(Panel {
            member: name.into(),
            x,
            y: (0.0, b / 2.0),
            z: (0.0, 0.0),
            count: 1,
            area_each: len * b,
            plate_mm: t,
            dir: StiffenerDirection::Longitudinal,
            stiffeners: alloc::vec![(bottom_long.clone(), n_b as f64 * len)],
            primary_spacing: Some(web),
            primary: Some(floors_desc.clone()),
            rule: format!("{rule}; longitudinals 5-2-2/3.7 (c = 1.34)"),
            basis: "rule",
            note: if bottom_add > 0.0 {
                format!("{bottom_add:.1} mm added amidships for the hull girder section modulus")
            } else {
                String::new()
            },
        });
    }
    let t_sm = order_mm(t_side_mid);
    let t_se = order_mm((t_side_mid + t_shell_end) / 2.0);
    for (name, x, len, t, rule) in [
        (
            "side shell, midship 0.4L",
            mid,
            0.4 * l,
            t_sm,
            "ABS 3-2-2/3.1",
        ),
        (
            "side shell, ends",
            (0.0, l),
            ends_len,
            t_se,
            "ABS 3-2-2/5.1, tapered from 3-2-2/3.1",
        ),
    ] {
        bld.add(Panel {
            member: name.into(),
            x,
            y: (b / 2.0, b / 2.0),
            z: (0.0, d),
            count: 2,
            area_each: len * d,
            plate_mm: t,
            dir: StiffenerDirection::Longitudinal,
            stiffeners: side_longs.iter().map(|st| (st.clone(), len)).collect(),
            primary_spacing: Some(web),
            primary: Some(webs_desc("side web frames")),
            rule: format!("{rule}; longitudinals 5-2-2/7 and 5-2-1/7 (c = 1.25)"),
            basis: "rule",
            note: String::new(),
        });
    }
    // Deck: in the cargo zone the strips outside the tank cut-outs; full beam in the end zones.
    let deck_mid_w = deck_w;
    let t_dm = order_mm(t_deck_mid + deck_add);
    let t_de = order_mm((t_deck_mid + deck_add + t_deck_end) / 2.0);
    let ends_area = (l - lc) * b + (lc - 0.4 * l).max(0.0) * deck_w;
    let ends_long_len =
        (l - lc) * n_b as f64 + (lc - 0.4 * l).max(0.0) * n_across(deck_w, s) as f64;
    bld.add(Panel {
        member: "strength deck, midship 0.4L".into(),
        x: mid,
        y: (0.0, b / 2.0),
        z: (d, d),
        count: 1,
        area_each: 0.4 * l * deck_mid_w,
        plate_mm: t_dm,
        dir: StiffenerDirection::Longitudinal,
        stiffeners: alloc::vec![(deck_long.clone(), n_across(deck_w, s) as f64 * 0.4 * l)],
        primary_spacing: Some(web),
        primary: Some(webs_desc("deck transverses")),
        rule: "ABS 3-2-3/1.1.2(a)(b); 3-2-1/3.1 hull girder; beams 5-1-1/3.1".into(),
        basis: "rule",
        note: format!(
            "width {deck_w:.2} m outside the tank cut-outs{}",
            if deck_add > 0.0 {
                format!("; {deck_add:.1} mm added for the hull girder section modulus")
            } else {
                String::new()
            }
        ),
    });
    bld.add(Panel {
        member: "strength deck, ends".into(),
        x: (0.0, l),
        y: (0.0, b / 2.0),
        z: (d, d),
        count: 1,
        area_each: ends_area,
        plate_mm: t_de,
        dir: StiffenerDirection::Longitudinal,
        stiffeners: alloc::vec![(deck_long.clone(), ends_long_len)],
        primary_spacing: Some(web),
        primary: Some(webs_desc("deck transverses")),
        rule: "ABS 3-2-3/1.7, tapered from 3-2-3/1.1.2".into(),
        basis: "rule",
        note: "full beam in the end zones; the strips outside the cut-outs in the rest of the cargo zone".into(),
    });

    // --- Double bottom: inner bottom, girders, floors (full length) ---
    if db > 0.0 {
        bld.add(Panel {
            member: "inner bottom".into(),
            x: (0.0, l),
            y: (0.0, b / 2.0),
            z: (db, db),
            count: 1,
            area_each: l * b,
            plate_mm: order_mm(t_ib),
            dir: StiffenerDirection::Longitudinal,
            stiffeners: alloc::vec![(ib_long.clone(), n_b as f64 * l)],
            primary_spacing: Some(web),
            primary: Some(floors_desc.clone()),
            rule: "ABS 5-2-2/3.3 (5-1-2/5.17, 3-2-7/3.1); longitudinals 5-2-2/3.5".into(),
            basis: "rule",
            note: "ballast tank boundary; head per 5-2-1/3".into(),
        });
        bld.add(Panel {
            member: "centre girder".into(),
            x: (0.0, l),
            y: (0.0, 0.0),
            z: (0.0, db),
            count: 1,
            area_each: 0.75 * l * db + 0.25 * l * db * 0.85,
            plate_mm: order_mm(t_cg),
            dir: StiffenerDirection::None,
            stiffeners: Vec::new(),
            primary_spacing: None,
            primary: None,
            rule: "ABS 5-1-2/5.1.1 and 5.1.2 (85 % at the ends, weighed as thinner area)".into(),
            basis: "rule",
            note: String::new(),
        });
        if n_side_girders > 0 {
            bld.add(Panel {
                member: "side girders".into(),
                x: (0.0, l),
                y: (0.0, b / 2.0),
                z: (0.0, db),
                count: 2 * n_side_girders,
                area_each: l * db,
                plate_mm: order_mm(t_sg),
                dir: StiffenerDirection::None,
                stiffeners: Vec::new(),
                primary_spacing: None,
                primary: None,
                rule: "ABS 5-1-2/5.5 (at most 4.6 m apart)".into(),
                basis: "rule",
                note: format!("{n_side_girders} each side"),
            });
        }
        let n_floor = (floor(l / web) as usize).max(1);
        let floor_stiff = Section::flat(100.0, 7.0);
        bld.add(Panel {
            member: "solid floors".into(),
            x: (0.0, l),
            y: (0.0, b / 2.0),
            z: (0.0, db),
            count: n_floor,
            area_each: b * db,
            plate_mm: order_mm(0.036 * l + 6.0),
            dir: StiffenerDirection::Vertical,
            stiffeners: alloc::vec![(floor_stiff, n_b as f64 * db)],
            primary_spacing: Some(web),
            primary: None,
            rule: "ABS 5-1-2/5.7.1 (spacing at most 3.66 m; a stiffener at every longitudinal)".into(),
            basis: "rule",
            note: "no lightening holes; floor stiffener FB 100x7 is an Estimate (the smallest of the series)".into(),
        });
    }

    // --- Double side (cargo zone): inner side, wing deck, wing web frames ---
    if w > 0.0 && zw > db && lc > 0.0 {
        let h = zw - db;
        bld.add(Panel {
            member: "inner side (longitudinal bulkhead)".into(),
            x: (spec.cargo_zone_aft_m, spec.cargo_zone_fwd_m),
            y: (b / 2.0 - w, b / 2.0 - w),
            z: (db, zw),
            count: 2,
            area_each: lc * h,
            plate_mm: order_mm(t_inner_side),
            dir: StiffenerDirection::Longitudinal,
            stiffeners: inner_longs.iter().map(|st| (st.clone(), lc)).collect(),
            primary_spacing: Some(web),
            primary: Some(webs_desc("wing web frames")),
            rule: "ABS 3-2-7/3.1 plating, 3-2-7/3.3 stiffeners (c = 0.90), head 5-2-1/3".into(),
            basis: "rule",
            note: "wing ballast tank boundary".into(),
        });
        if wing_deck_w > 0.0 {
            bld.add(Panel {
                member: "wing tank top".into(),
                x: (spec.cargo_zone_aft_m, spec.cargo_zone_fwd_m),
                y: (b / 2.0 - w, b / 2.0),
                z: (zw, zw),
                count: 1,
                area_each: lc * wing_deck_w,
                plate_mm: order_mm(t_wing_deck),
                dir: StiffenerDirection::Longitudinal,
                stiffeners: alloc::vec![(
                    wing_deck_long.clone(),
                    n_across(wing_deck_w, s) as f64 * lc
                )],
                primary_spacing: Some(web),
                primary: Some(webs_desc("wing web frames")),
                rule: "ABS 3-2-7/3.1 plating, 3-2-7/3.3 stiffeners, head 5-2-1/3".into(),
                basis: "rule",
                note: format!("total width {wing_deck_w:.2} m, both sides"),
            });
        }
        let n_web = (floor(lc / web) as usize).max(1);
        let tw = web_thickness_mm(l, w * 1000.0, true);
        bld.add(Panel {
            member: "wing web frames".into(),
            x: (spec.cargo_zone_aft_m, spec.cargo_zone_fwd_m),
            y: (b / 2.0 - w, b / 2.0),
            z: (db, zw),
            count: 2 * n_web,
            area_each: w * h,
            plate_mm: order_mm(tw),
            dir: StiffenerDirection::None,
            stiffeners: Vec::new(),
            primary_spacing: Some(web),
            primary: None,
            rule: "ABS 3-2-4/3.11.1 web thickness in tanks".into(),
            basis: "rule",
            note: "plate diaphragms across the double side, no openings".into(),
        });
    }

    // --- Side web frames clear of the double side: above the wing tank top in the cargo zone, and
    // from the inner bottom to the deck in the end zones ---
    let side_web = |span: f64, z_mid: f64| {
        let depth_mm = (125.0 * span).max(2.5 * 200.0);
        let tw = web_thickness_mm(l, depth_mm, false);
        let smr_w = primary_sm(1.75, (d - z_mid) + ha_wing, web, span);
        design_primary(
            smr_w,
            depth_mm,
            order_mm(tw),
            t_side_mid,
            (web * 1000.0).min(span * 1000.0 / 3.0),
        )
    };
    let add_side_webs = |bld: &mut Builder, name: &str, x: (f64, f64), z: (f64, f64), n: usize| {
        if n == 0 || z.1 <= z.0 {
            return;
        }
        let span = z.1 - z.0;
        let sec = side_web(span, (z.0 + z.1) / 2.0);
        bld.add(Panel {
            member: name.into(),
            x,
            y: (b / 2.0, b / 2.0),
            z,
            count: 2 * n,
            area_each: 0.0,
            plate_mm: 0.0,
            dir: StiffenerDirection::None,
            stiffeners: alloc::vec![(sec.clone(), span)],
            primary_spacing: Some(web),
            primary: Some(sec.name.clone()),
            rule: "ABS 5-2-1/11 (c = 1.75), proportions 3-2-4/3.9(iii), web 3-2-4/3.11.2".into(),
            basis: "rule",
            note:
                "design head to the deck plus the 0.91 m of 5-2-2/9 (an Estimate for a void space)"
                    .into(),
        });
    };
    let n_cz = if lc > 0.0 {
        (floor(lc / web) as usize).max(1)
    } else {
        0
    };
    let z_cz = if w > 0.0 { zw } else { db };
    add_side_webs(
        &mut bld,
        "side web frames, cargo zone",
        (spec.cargo_zone_aft_m, spec.cargo_zone_fwd_m),
        (z_cz, d),
        n_cz,
    );
    let n_ends = floor((l - lc) / web) as usize;
    add_side_webs(
        &mut bld,
        "side web frames, end zones",
        (0.0, l),
        (db, d),
        n_ends,
    );

    // --- Deck transverses, full length ---
    let n_strips = spec.tanks_abreast.max(1) as f64 + 1.0;
    let span = if spec.deck_opening_width_m > 0.0 {
        deck_w / n_strips
    } else {
        b / n_strips
    };
    if span > 0.0 {
        let depth_mm = (58.5 * span).max(2.0 * deck_long.web_mm.0);
        let tw = order_mm(web_thickness_mm(l, depth_mm, false));
        let sec = design_primary(
            primary_sm(2.0, h_deck, web, span),
            depth_mm,
            tw,
            t_deck_mid,
            (web * 1000.0).min(span * 1000.0 / 3.0),
        );
        let n = (floor(l / web) as usize).max(1);
        let len_each = (lc * deck_w + (l - lc) * b) / l;
        bld.add(Panel {
            member: "deck transverses".into(),
            x: (0.0, l),
            y: (0.0, b / 2.0),
            z: (d - depth_mm / 1000.0, d),
            count: n,
            area_each: 0.0,
            plate_mm: 0.0,
            dir: StiffenerDirection::None,
            stiffeners: alloc::vec![(sec.clone(), len_each)],
            primary_spacing: Some(web),
            primary: Some(sec.name.clone()),
            rule: "ABS 5-2-1/11 (c = 2.0), head 5-1-1/3.1, proportions 3-2-4/3.9(i), web 3-2-4/3.11.2".into(),
            basis: "rule",
            note: format!("span {span:.2} m between supports (an Estimate: deck strips split by the tanks abreast)"),
        });
    }

    // --- Transverse bulkheads: the hold bulkheads of the cargo zone, a collision bulkhead and an aft
    // peak bulkhead. Watertight bulkheads (the holds are not tanks), horizontal stiffeners between
    // vertical webs at the web spacing ---
    let n_bhd = spec.holds_along + 1 + 2;
    let hb = d - db;
    if hb > 0.0 {
        // Plating in strakes one web spacing high, each at its lower edge (3-2-6/5.1).
        let mut area_t = 0.0;
        let mut stiff: Vec<(Section, f64)> = Vec::new();
        let n_rows = n_across(hb, s);
        for i in 0..n_rows {
            let z = db + (i as f64 + 1.0) * sm;
            let st = pick_stiffener(stiffener_sm(0.56, bulkhead_head(d, z), sm, web), 6.0, s).0;
            stiff.push((st, b));
        }
        let mut z = db;
        while z < d - 1e-9 {
            let top = (z + web).min(d);
            area_t += (top - z) * watertight_bulkhead_plate_mm(s, web / sm, d - z);
            z = top;
        }
        let t_mean = area_t / hb;
        let n_vweb = (floor(b / web) as usize).max(1);
        let vweb = {
            let depth_mm = 125.0 * hb;
            let tw = order_mm(web_thickness_mm(l, depth_mm, false));
            design_primary(
                primary_sm(1.0, bulkhead_head(d, db + hb / 2.0), web, hb),
                depth_mm,
                tw,
                t_mean,
                (web * 1000.0).min(hb * 1000.0 / 3.0),
            )
        };
        stiff.push((vweb.clone(), hb * n_vweb as f64));
        bld.add(Panel {
            member: "transverse bulkheads".into(),
            x: (0.0, l),
            y: (0.0, b / 2.0),
            z: (db, d),
            count: n_bhd,
            area_each: b * hb,
            plate_mm: libm::round(t_mean * 10.0) / 10.0,
            dir: StiffenerDirection::Transverse,
            stiffeners: stiff,
            primary_spacing: Some(web),
            primary: Some(format!("vertical webs {}", vweb.name)),
            rule: "ABS 3-2-6/5.1 plating, 5.3 stiffeners (c = 0.56), 5.5 webs; proportions 3-2-7/3.5.3".into(),
            basis: "rule",
            note: format!(
                "{} hold bulkheads, a collision and an aft peak bulkhead; plate thickness is the area mean of strakes {web:.2} m high",
                spec.holds_along + 1
            ),
        });
    }

    let net: f64 = bld.members.iter().map(|m| m.total_t).sum();
    let allowance = net * ch.allowance_pct / 100.0;
    HullPanelModel {
        members: bld.members,
        hull_girder,
        net_t: net,
        allowance_pct: ch.allowance_pct,
        allowance_t: allowance,
        total_t: net + allowance,
        rule_edition: RULE_EDITION,
    }
}

/// The double-hull spec of a gas hull from its converged arrangement (the `BargeInputs` and
/// `TankResult` of `GasBasis`). The wing tank top, double bottom and cut-outs are the ones the
/// weights already use (`weights::calculate_weights_with`).
pub fn gas_hull_spec(
    inputs: &crate::models::BargeInputs,
    tanks: &crate::models::TankResult,
    block_coefficient: f64,
) -> DoubleHullSpec {
    let db = inputs.double_bottom_height.unwrap_or(0.0);
    let od = tanks.outer_diameter;
    let r = od / 2.0;
    let bottom = crate::tanks::tank_bottom(db, inputs.tank_support_height);
    let axis = crate::tanks::tank_axis_z(bottom, od);
    let chord = |z: f64| {
        let dy = z - axis;
        if fabs(dy) < r {
            2.0 * sqrt(r * r - dy * dy)
        } else {
            0.0
        }
    };
    let wing_top = bottom + inputs.wing_tank_height_ratio * od;
    let stern = inputs.stern_non_tank_length.unwrap_or(0.0);
    let bow = inputs.bow_non_tank_length.unwrap_or(0.0);
    let n_wide = tanks.n_tanks_wide.max(1) as f64;
    DoubleHullSpec {
        loa_m: inputs.loa,
        beam_m: inputs.beam,
        depth_m: inputs.depth,
        block_coefficient,
        cargo_zone_aft_m: stern,
        cargo_zone_fwd_m: inputs.loa - bow,
        double_bottom_m: db,
        wing_width_m: inputs.wing_tank_width.unwrap_or(0.0).max(0.0),
        wing_top_m: wing_top,
        deck_opening_width_m: n_wide * chord(inputs.depth),
        wing_deck_opening_width_m: n_wide * chord(wing_top),
        holds_along: tanks.n_tanks_long.max(1) as usize,
        tanks_abreast: tanks.n_tanks_wide.max(1) as usize,
        steel_density: inputs.hull_steel_density,
    }
}
