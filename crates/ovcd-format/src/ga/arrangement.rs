//! The arrangement plan of a Formation (issue #360, spec 0007 "Drawings"): one sheet with the barges
//! and the tug in plan, with the overall dimensions and the coupling points, and below it the profile
//! in the light condition with the tug's sightline over the convoy.
//!
//! It reads the results only: the barge (`dims`, `results`), the barges abreast and in line
//! (`convoy`), the tug (`formation.tug`), the Notch of an ATB (`drawing.atb`) and the sightline
//! (`formation.sightline`). x is forward from the stern of the tug (from the stern of the aft barges
//! when there is no tug); in plan y is to port, in profile z is up from the light waterline.

use super::*;

/// The gap on paper between the plan and the profile, in millimetres: room for the overall beam
/// dimension under the plan and the labels.
const PROFILE_GAP_MM: f64 = 30.0;
/// The radius of a coupling point mark, in millimetres of paper.
const COUPLING_MM: f64 = 1.2;
/// How far the drawn sightline runs ahead of the bow, as a share of the overall length: a longer
/// Blind Distance is cut there, and the note gives its value.
const RAY_REACH: f64 = 0.2;

/// What the arrangement plan draws.
struct Arrangement {
    hull: Hull,
    abreast: u64,
    in_line: u64,
    /// The tug: length, beam, draught, depth.
    tug: Option<(f64, f64, f64, f64)>,
    /// The aft end of the aft barges from the tug's stern.
    barge_x0: f64,
    /// The sightline: the eye x, the eye and the floor raised above the water, the bow x, the
    /// Blind Distances light and laden, the limit.
    sight: Option<Sight>,
}

struct Sight {
    eye_x: f64,
    eye_raised: f64,
    floor_raised: f64,
    bow_x: f64,
    light: f64,
    laden: f64,
    limit: f64,
}

/// The message when an Option has no arrangement plan.
pub const NO_ARRANGEMENT_PLAN: &str =
    "an arrangement plan is drawn for a pushed convoy, or a pushed barge or an ATB with its tug";

fn read_sight(results: &Value) -> Option<Sight> {
    let s = results.get("formation")?.get("sightline")?;
    let n = |k: &str| s.get(k).and_then(Value::as_f64);
    Some(Sight {
        eye_x: n("eye_x_m")?,
        eye_raised: n("eye_raised_above_water_m")?,
        floor_raised: n("floor_raised_above_water_m")?,
        bow_x: n("bow_x_m")?,
        light: n("blind_distance_light_m")?,
        laden: n("blind_distance_laden_m")?,
        limit: n("limit_m")?,
    })
}

fn read_arrangement(results: &Value) -> Result<Arrangement, String> {
    let formation = results
        .get("formation")
        .filter(|f| f["tug_included"] == true);
    if let Some(f) = formation {
        if let Some(e) = f.get("error").and_then(Value::as_str) {
            return Err(e.to_string());
        }
        if f.get("barges_towed").is_some() {
            return Err(NO_ARRANGEMENT_PLAN.to_string());
        }
    }
    let convoy = results.get("convoy");
    if convoy.is_none() && formation.is_none() {
        return Err(NO_ARRANGEMENT_PLAN.to_string());
    }
    let hull = read_hull(results)?;
    let count = |k: &str| convoy.and_then(|c| c[k].as_u64()).unwrap_or(1);
    let tug = formation.and_then(|f| {
        let t = &f["tug"];
        Some((
            t["length_m"].as_f64()?,
            t["beam_m"].as_f64()?,
            t["draught_m"].as_f64()?,
            t["depth_m"].as_f64()?,
        ))
    });
    let notch = hull.atb.as_ref().map_or(0.0, |a| a.notch_depth);
    // The tug's stern at x = 0; an ATB tug's bow stands the coupler's gap off the Notch's end wall.
    let gap = if hull.atb.is_some() {
        ATB_COUPLER_GAP_M
    } else {
        0.0
    };
    let barge_x0 = tug.map_or(0.0, |t| t.0 + gap - notch);
    Ok(Arrangement {
        abreast: count("abreast"),
        in_line: count("in_line"),
        tug,
        barge_x0,
        sight: read_sight(results),
        hull,
    })
}

impl Arrangement {
    fn overall_length(&self) -> f64 {
        self.barge_x0 + self.in_line as f64 * self.hull.loa
    }

    fn overall_beam(&self) -> f64 {
        let barges = self.abreast as f64 * self.hull.beam;
        self.tug.map_or(barges, |t| barges.max(t.1))
    }

    /// The light draught of a barge: the light waterline, else the design one.
    fn barge_light_draught(&self) -> f64 {
        self.hull.light_draught.unwrap_or(self.hull.draught)
    }
}

/// The barges in plan, the tug, the coupling points, the centre line and the overall dimensions.
fn plan_part(c: &mut Canvas, a: &Arrangement, labels: &mut Vec<(String, [f64; 2])>) {
    let h = &a.hull;
    let half = a.abreast as f64 * h.beam / 2.0;
    let mut couplings: Vec<[f64; 2]> = Vec::new();
    for i in 0..a.in_line {
        let x0 = a.barge_x0 + i as f64 * h.loa;
        for j in 0..a.abreast {
            let y0 = -half + j as f64 * h.beam;
            let y1 = y0 + h.beam;
            match (&h.atb, i) {
                // The barge of an ATB, with its Notch at the transom.
                (Some(n), 0) => {
                    let w = n.notch_width / 2.0;
                    c.polyline(
                        "OUTLINE",
                        &[
                            [x0, y0],
                            [x0 + h.loa, y0],
                            [x0 + h.loa, y1],
                            [x0, y1],
                            [x0, w],
                            [x0 + n.notch_depth, w],
                            [x0 + n.notch_depth, -w],
                            [x0, -w],
                        ],
                        true,
                    );
                }
                _ => c.polyline(
                    "OUTLINE",
                    &[[x0, y0], [x0 + h.loa, y0], [x0 + h.loa, y1], [x0, y1]],
                    true,
                ),
            }
            if h.bow_rake > 0.0 {
                let xr = x0 + h.loa - h.bow_rake;
                c.line("OBJECT", [xr, y0], [xr, y1], None);
            }
            if h.stern_rake > 0.0 {
                let xr = x0 + h.stern_rake;
                c.line("OBJECT", [xr, y0], [xr, y1], None);
            }
        }
        // A joint between two rows of barges: a coupling point on each side line it crosses.
        if i > 0 {
            for j in 0..=a.abreast {
                couplings.push([x0, -half + j as f64 * h.beam]);
            }
        }
    }
    if let Some((length, beam, _, _)) = a.tug {
        let tb = beam / 2.0;
        match &h.atb {
            // The ATB tug in the Notch, with its pointed bow and the coupler bars.
            Some(n) => {
                let bow = a.barge_x0 + n.notch_depth - ATB_COUPLER_GAP_M;
                let stern = bow - length;
                let shoulder = bow - (length * ATB_BOW_TAPER).min(beam);
                c.polyline(
                    "OBJECT",
                    &[
                        [stern, -tb],
                        [shoulder, -tb],
                        [bow, 0.0],
                        [shoulder, tb],
                        [stern, tb],
                    ],
                    true,
                );
                for y in [-tb / 2.0, tb / 2.0] {
                    c.line("OBJECT", [bow, y], [bow + ATB_COUPLER_GAP_M, y], None);
                    couplings.push([bow + ATB_COUPLER_GAP_M, y]);
                }
            }
            // A pusher: a box with its square bow on the aft barges, coupled at its push knees.
            None => {
                c.polyline(
                    "OBJECT",
                    &[[0.0, -tb], [length, -tb], [length, tb], [0.0, tb]],
                    true,
                );
                for y in [-tb / 2.0, tb / 2.0] {
                    couplings.push([length, y]);
                }
            }
        }
    }
    let r = COUPLING_MM * c.k;
    for p in &couplings {
        c.circle("OBJECT", *p, r);
    }
    // The label points at the forward coupling on the port side, clear of the other leaders.
    let front = couplings.iter().copied().max_by(|p, q| {
        (p[0], p[1])
            .partial_cmp(&(q[0], q[1]))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if let Some(p) = front {
        labels.push((
            label!(
                "draw-coupling-points",
                "COUPLING POINTS ({count})",
                count = couplings.len()
            ),
            [p[0] + r, p[1] + r],
        ));
    }
    let length = a.overall_length();
    let beam = a.overall_beam();
    let overshoot = 4.0 * c.k;
    c.centre_line([-overshoot, 0.0], [length + overshoot, 0.0]);
    let y_low = -beam / 2.0;
    if !c.try_dim_horizontal([0.0, y_low], [length, y_low], -DIM_OFFSET_MM) {
        labels.push((
            label!(
                "draw-overall-length",
                "OVERALL LENGTH {length}",
                length = metres(length)
            ),
            [length, y_low],
        ));
    }
    if !c.try_dim_vertical(
        [a.barge_x0, -half],
        [a.barge_x0, half],
        DIM_OFFSET_MM + a.barge_x0 / c.k,
    ) {
        labels.push((
            label!(
                "draw-overall-beam",
                "OVERALL BEAM {beam}",
                beam = metres(2.0 * half)
            ),
            [0.0, half],
        ));
    }
    // The barge's own length, on the first barge, above the plan.
    let top = beam / 2.0;
    c.try_dim_horizontal([a.barge_x0, top], [a.barge_x0 + h.loa, top], DIM_OFFSET_MM);
}

/// The profile in the light condition, `dz` below the plan: the barges and the tug at their own
/// light draughts, the light waterline, the wheelhouse and the sightline.
fn profile_part(c: &mut Canvas, a: &Arrangement, dz: f64, labels: &mut Vec<(String, [f64; 2])>) {
    let h = &a.hull;
    let t = a.barge_light_draught();
    let p = |x: f64, z: f64| [x, z + dz];
    for i in 0..a.in_line {
        let x0 = a.barge_x0 + i as f64 * h.loa;
        let (keel, deck) = (-t, h.depth - t);
        let mut pts = vec![p(x0, deck)];
        pts.push(p(x0 + h.stern_rake.max(0.0), keel));
        pts.push(p(x0 + h.loa - h.bow_rake.max(0.0), keel));
        pts.push(p(x0 + h.loa, deck));
        c.polyline("OUTLINE", &pts, true);
        // The cargo top above the deck (a coaming, tank domes or a cargo top), as the air draft
        // gives it: the sightline passes over it.
        if let Some(top) = h.air_top {
            let top = top - t;
            if top > deck + 1e-6 {
                let (xa, xb) = (x0 + h.stern_void, x0 + h.loa - h.bow_void);
                c.polyline(
                    "OBJECT",
                    &[p(xa, deck), p(xa, top), p(xb, top), p(xb, deck)],
                    false,
                );
            }
        }
    }
    let length = a.overall_length();
    let mut water_end = length;
    if let Some((tl, _, td, tdepth)) = a.tug {
        let x0 = 0.0;
        c.polyline(
            "OBJECT",
            &[
                p(x0, -td),
                p(x0 + tl, -td),
                p(x0 + tl, tdepth - td),
                p(x0, tdepth - td),
            ],
            true,
        );
        if let Some(s) = &a.sight {
            let (wx0, wx1) = (
                s.eye_x - ovcd_core::aft_arrangement::DEFAULT_WHEELHOUSE_LENGTH_M,
                s.eye_x,
            );
            let wh = ovcd_core::aft_arrangement::DEFAULT_WHEELHOUSE_HEIGHT_M;
            let floor = tdepth - td;
            c.polyline(
                "OBJECT",
                &[
                    p(wx0, floor),
                    p(wx1, floor),
                    p(wx1, floor + wh),
                    p(wx0, floor + wh),
                ],
                true,
            );
            if s.floor_raised > floor + 1e-6 {
                let f = s.floor_raised;
                c.polyline(
                    "CENTRE",
                    &[p(wx0, f), p(wx1, f), p(wx1, f + wh), p(wx0, f + wh)],
                    true,
                );
                labels.push((label!("draw-wheelhouse-raised", "RAISED"), p(wx1, f + wh)));
                // The column of the lift.
                c.line("HIDDEN", p(wx0 + 0.5, floor + wh), p(wx0 + 0.5, f), None);
            }
            // The sightline from the raised eye to the water: cut at RAY_REACH of the overall length
            // ahead of the bow when the Blind Distance is longer.
            if s.light.is_finite() {
                let x_hit = s.bow_x + s.light;
                let x_end = x_hit.min(s.bow_x + RAY_REACH * length);
                let z_end = s.eye_raised * (x_hit - x_end) / (x_hit - s.eye_x);
                c.line("CENTRE", p(s.eye_x, s.eye_raised), p(x_end, z_end), None);
                water_end = water_end.max(x_end);
                labels.push((
                    label!("draw-sightline-light", "SIGHTLINE, LIGHT"),
                    p(x_end, z_end),
                ));
            }
        }
    }
    c.line(
        "WATER",
        p(0.0, 0.0),
        p(water_end, 0.0),
        Some("chain_double_dashed"),
    );
    labels.push((
        label!("draw-light-waterline", "LIGHT WATERLINE"),
        p(water_end, 0.0),
    ));
}

fn arrangement_canvas(a: &Arrangement, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
    plan_part(&mut c, a, &mut labels);
    let h = &a.hull;
    let top = a
        .sight
        .as_ref()
        .map_or(0.0, |s| s.eye_raised.max(s.floor_raised + 3.4))
        .max(h.air_top.unwrap_or(h.depth) - a.barge_light_draught())
        .max(h.depth);
    let dz = -a.overall_beam() / 2.0 - PROFILE_GAP_MM * c.k - top;
    profile_part(&mut c, a, dz, &mut labels);
    let column = c.max[0] + 6.0 * c.k;
    c.label_column(column, &mut labels);
    c
}

/// The arrangement plan of an Option's results: a pushed convoy, or a pushed barge or an ATB with its
/// tug. `option` names the Option in the title block.
pub fn arrangement_plan_for(results: &Value, option: Option<&str>) -> Result<Value, String> {
    let mut a = read_arrangement(results)?;
    a.hull.option_name = option.map(str::to_string);
    let title = label!("draw-view-arrangement-plan", "ARRANGEMENT PLAN");
    let mut notes = vec![label!(
        "draw-note-arrangement",
        "{view}. DIMENSIONS IN METRES. PROFILE IN THE LIGHT CONDITION.",
        view = title.clone()
    )];
    notes.push(label!(
        "draw-note-barges",
        "{abreast} ABREAST BY {in_line} IN LINE.",
        abreast = a.abreast as usize,
        in_line = a.in_line as usize
    ));
    if let Some(s) = &a.sight {
        notes.push(label!(
            "draw-note-blind-distance",
            "BLIND DISTANCE AHEAD OF THE BOW: LIGHT {light} M, LADEN {laden} M, LIMIT {limit} M.",
            light = fixed(s.light, 1),
            laden = fixed(s.laden, 1),
            limit = fixed(s.limit, 0)
        ));
    }
    notes.push(label!(
        "draw-note-scope",
        "FOR CONCEPT AND PRELIMINARY DESIGN ONLY, NOT FOR APPROVAL."
    ));
    notes.extend(failed_notes(&a.hull.failed_checks));
    let mut fields = title_fields(&a.hull, &title, 1);
    fields["sheet_count"] = json!(1);
    if let Some(number) = fields["document_number"].as_str() {
        fields["document_number"] = json!(number.replace("-GA-1", "-AP-1"));
    }
    place(notes, fields, RATIOS[0], &|n| arrangement_canvas(&a, n)).map(|(document, _)| document)
}

/// As [`arrangement_plan_for`], from a run outcome.
pub fn arrangement_plan_named(outcome: &RunOutcome, option: Option<&str>) -> Result<Value, String> {
    match outcome {
        RunOutcome::Results { results, .. } => arrangement_plan_for(results, option),
        RunOutcome::NotAvailable(message) | RunOutcome::Failed(message) => Err(message.clone()),
    }
}
