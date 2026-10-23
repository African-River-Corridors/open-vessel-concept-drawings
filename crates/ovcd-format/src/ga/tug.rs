//! The GA of a Formation's tug (issue #360, spec 0007 "Drawings": "The tug gets its own GA (it is a
//! Hull)"): one sheet with the side elevation above and the plan below, at one scale.
//!
//! It reads `drawing.tug` only: the size the sizing loop gave, the propulsors, the machinery spaces
//! and engines of the tug's sizing, and the wheelhouse (raised when the sightline lifts it). The
//! results carry no positions for the propellers, rudders and push knees, so they are drawn at the
//! drawing proportions below, as the stern of the GA views is: they show the arrangement, they are not
//! engineering data, and a note says so. x is forward from the tug's stern; z is up from its keel.

use super::*;

/// The gap on paper between the elevation and the plan, in millimetres.
const PLAN_GAP_MM: f64 = 30.0;
/// The propeller plane stands this many diameters forward of the stern.
const PROP_X_OF_D: f64 = 0.8;
/// A rudder behind its propeller: from this many diameters aft of the disc to `RUDDER_FWD_OF_D`.
const RUDDER_AFT_OF_D: f64 = 0.9;
const RUDDER_FWD_OF_D: f64 = 0.5;
/// The two flanking rudders ahead of each propeller: from `FLANK_AFT_OF_D` to `FLANK_FWD_OF_D`
/// diameters forward of the disc, a quarter diameter either side of the shaft.
const FLANK_AFT_OF_D: f64 = 0.5;
const FLANK_FWD_OF_D: f64 = 0.9;
/// A nozzle stands this share of the diameter outside the disc and is this share of it long.
const NOZZLE_RING_OF_D: f64 = 0.08;
const NOZZLE_LENGTH_OF_D: f64 = 0.5;
/// An azimuth unit's pod hangs with its axis this many diameters below the keel.
const POD_BELOW_KEEL_OF_D: f64 = 0.6;
/// The push knees: this high above the deck, this long and wide, at a quarter beam each side.
const KNEE_RISE_M: f64 = 1.5;
const KNEE_LENGTH_M: f64 = 0.6;
const KNEE_WIDTH_M: f64 = 0.4;
/// The engines stand on a double bottom of this height (as the tug's machinery centres take it).
const DOUBLE_BOTTOM_M: f64 = 0.5;

struct TugDraw {
    kind: String,
    length: f64,
    beam: f64,
    draught: f64,
    depth: f64,
    propulsor: String,
    count: u32,
    diameter: f64,
    /// (label, x aft, x forward) of the machinery spaces.
    zones: Vec<(String, f64, f64)>,
    /// (count, length, width, height) of the engines or gensets.
    engine: Option<(u32, f64, f64, f64)>,
    engine_room: (f64, f64),
    /// x aft, x forward, width, height, floor lowered, floor raised.
    wheelhouse: (f64, f64, f64, f64, f64, Option<f64>),
}

/// The message when an Option has no tug to draw.
pub const NO_TUG_GA: &str = "a tug GA is drawn for a Formation that includes its tug";

fn read_tug(results: &Value) -> Result<TugDraw, String> {
    let t = results
        .get("drawing")
        .and_then(|d| d.get("tug"))
        .ok_or_else(|| NO_TUG_GA.to_string())?;
    let n = |v: &Value, k: &str| {
        v.get(k)
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("the results have no drawing.tug.{k}"))
    };
    let zones = t["zones"]
        .as_array()
        .map(|z| {
            z.iter()
                .filter_map(|z| {
                    Some((
                        z["label"].as_str()?.to_string(),
                        z["x_aft_m"].as_f64()?,
                        z["x_fwd_m"].as_f64()?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default();
    let engine = t.get("engine").filter(|e| !e.is_null()).and_then(|e| {
        Some((
            e["count"].as_u64()? as u32,
            e["length_m"].as_f64()?,
            e["width_m"].as_f64()?,
            e["height_m"].as_f64()?,
        ))
    });
    let room = &t["engine_room_m"];
    let w = &t["wheelhouse"];
    Ok(TugDraw {
        kind: t["kind"].as_str().unwrap_or("pusher").to_string(),
        length: n(t, "length_m")?,
        beam: n(t, "beam_m")?,
        draught: n(t, "draught_m")?,
        depth: n(t, "depth_m")?,
        propulsor: t["propulsor_type"].as_str().unwrap_or("").to_string(),
        count: t["propulsor_count"].as_u64().unwrap_or(1).max(1) as u32,
        diameter: n(t, "propeller_diameter_m")?,
        zones,
        engine,
        engine_room: (
            room[0].as_f64().unwrap_or(0.0),
            room[1].as_f64().unwrap_or(0.0),
        ),
        wheelhouse: (
            n(w, "x_aft_m")?,
            n(w, "x_fwd_m")?,
            n(w, "width_m")?,
            n(w, "height_m")?,
            n(w, "floor_above_keel_m")?,
            w["floor_raised_above_keel_m"].as_f64(),
        ),
    })
}

impl TugDraw {
    fn shaft(&self) -> bool {
        self.propulsor.starts_with("shaft")
    }

    fn azimuth(&self) -> bool {
        self.propulsor.starts_with("azimuth")
    }

    fn ducted(&self) -> bool {
        matches!(self.propulsor.as_str(), "shaft_nozzle" | "azimuth_ducted")
    }

    fn prop_x(&self) -> f64 {
        PROP_X_OF_D * self.diameter
    }

    /// The height of the propeller axis above the keel: the tip at the keel, or the pod under it.
    fn axis_z(&self) -> f64 {
        if self.azimuth() {
            -POD_BELOW_KEEL_OF_D * self.diameter
        } else {
            self.diameter / 2.0
        }
    }

    /// The athwartships positions of `n` units spread evenly over the beam.
    fn spread(&self, n: u32) -> Vec<f64> {
        let step = self.beam / f64::from(n);
        (0..n)
            .map(|i| (f64::from(i) + 0.5) * step - self.beam / 2.0)
            .collect()
    }

    /// The engines' x extent, centred in the engine room.
    fn engine_x(&self, length: f64) -> (f64, f64) {
        let mid = (self.engine_room.0 + self.engine_room.1) / 2.0;
        (mid - length / 2.0, mid + length / 2.0)
    }

    fn label(&self) -> String {
        match self.kind.as_str() {
            "atb_tug" => label!("draw-tug-atb", "ATB TUG"),
            "towing_tug" => label!("draw-tug-towing", "TOWING TUG"),
            _ => label!("draw-tug-pusher", "PUSHER"),
        }
    }
}

fn rect(c: &mut Canvas, layer: &str, x0: f64, x1: f64, y0: f64, y1: f64) {
    c.polyline(layer, &[[x0, y0], [x1, y0], [x1, y1], [x0, y1]], true);
}

fn elevation(c: &mut Canvas, t: &TugDraw, labels: &mut Vec<(String, [f64; 2])>) {
    let (l, d) = (t.length, t.depth);
    c.polyline("OUTLINE", &[[0.0, d], [0.0, 0.0], [l, 0.0], [l, d]], false);
    c.line("OUTLINE", [0.0, d], [l, d], None);
    c.line("WATER", [0.0, t.draught], [l, t.draught], None);
    // The push knees at the bow (a pusher or an ATB tug; a towing tug has none).
    if t.kind != "towing_tug" {
        c.polyline(
            "OBJECT",
            &[
                [l - KNEE_LENGTH_M, d],
                [l - KNEE_LENGTH_M, d + KNEE_RISE_M],
                [l, d + KNEE_RISE_M],
                [l, d],
            ],
            false,
        );
        labels.push((
            label!("draw-push-knees", "PUSH KNEES"),
            [l, d + KNEE_RISE_M],
        ));
    }
    // The machinery spaces: a hidden bulkhead at the forward end of each.
    for (name, x0, x1) in &t.zones {
        c.line("HIDDEN", [*x1, 0.0], [*x1, d], None);
        labels.push((plain_capitals(name), [(x0 + x1) / 2.0, d * 0.75]));
    }
    if let Some((_, length, _, height)) = t.engine {
        let (x0, x1) = t.engine_x(length);
        let top = (DOUBLE_BOTTOM_M + height).min(d);
        rect(c, "HIDDEN", x0, x1, DOUBLE_BOTTOM_M, top);
    }
    // The propulsor, its shaft, nozzle and rudders.
    let (px, z, dia) = (t.prop_x(), t.axis_z(), t.diameter);
    c.line("OBJECT", [px, z - dia / 2.0], [px, z + dia / 2.0], None);
    labels.push((label!("draw-propeller", "PROPELLER"), [px, z + dia / 2.0]));
    if t.ducted() {
        let (r, h) = (
            dia / 2.0 + NOZZLE_RING_OF_D * dia,
            NOZZLE_LENGTH_OF_D * dia / 2.0,
        );
        rect(c, "OBJECT", px - h, px + h, z - r, z + r);
    }
    if t.azimuth() {
        // The leg from the hull bottom to the pod.
        c.line("OBJECT", [px, z], [px, 0.0], None);
    }
    if t.shaft() {
        if let Some((_, length, _, _)) = t.engine {
            c.centre_line([px, z], [t.engine_x(length).0, z]);
        }
        let (r0, r1) = (px - RUDDER_AFT_OF_D * dia, px - RUDDER_FWD_OF_D * dia);
        rect(c, "OBJECT", r0, r1, z - dia / 2.0, z + dia / 2.0);
        labels.push((label!("draw-rudder", "RUDDER"), [r0, z - dia / 2.0]));
        let (f0, f1) = (px + FLANK_AFT_OF_D * dia, px + FLANK_FWD_OF_D * dia);
        rect(c, "OBJECT", f0, f1, z - dia / 2.0, z + dia / 2.0);
        labels.push((
            label!("draw-flanking-rudders", "FLANKING RUDDERS"),
            [f1, z - dia / 2.0],
        ));
    }
    // The wheelhouse, lowered, and raised for the sightline.
    let (w0, w1, _, wh, floor, raised) = t.wheelhouse;
    rect(c, "OBJECT", w0, w1, floor, floor + wh);
    labels.push((label!("draw-wheelhouse", "WHEELHOUSE"), [w1, floor + wh]));
    if let Some(f) = raised.filter(|f| *f > floor + 1e-6) {
        rect(c, "CENTRE", w0, w1, f, f + wh);
        labels.push((label!("draw-wheelhouse-raised", "RAISED"), [w1, f + wh]));
    }
    // Dimensions.
    let bottom = t.axis_z().min(0.0) - dia / 2.0;
    if !c.try_dim_horizontal([0.0, bottom], [l, bottom], -DIM_OFFSET_MM) {
        labels.push((
            label!("draw-loa", "LOA {length}", length = metres(l)),
            [l, 0.0],
        ));
    }
    c.try_dim_vertical([0.0, 0.0], [0.0, d], DIM_OFFSET_MM);
    labels.push((
        label!(
            "draw-draught",
            "DRAUGHT {draught}",
            draught = metres(t.draught)
        ),
        [l, t.draught],
    ));
}

fn plan_view(c: &mut Canvas, t: &TugDraw, dz: f64, labels: &mut Vec<(String, [f64; 2])>) {
    let (l, h) = (t.length, t.beam / 2.0);
    let p = |x: f64, y: f64| [x, y + dz];
    c.polyline("OUTLINE", &[p(0.0, -h), p(l, -h), p(l, h), p(0.0, h)], true);
    if t.kind != "towing_tug" {
        for y in [-h / 2.0, h / 2.0] {
            let q = [
                p(l - KNEE_LENGTH_M, y - KNEE_WIDTH_M / 2.0),
                p(l, y + KNEE_WIDTH_M / 2.0),
            ];
            rect(c, "OBJECT", q[0][0], q[1][0], q[0][1], q[1][1]);
        }
    }
    for (_, _, x1) in &t.zones {
        c.line("HIDDEN", p(*x1, -h), p(*x1, h), None);
    }
    if let Some((n, length, width, _)) = t.engine {
        let (x0, x1) = t.engine_x(length);
        for y in t.spread(n) {
            rect(
                c,
                "HIDDEN",
                x0,
                x1,
                y - width / 2.0 + dz,
                y + width / 2.0 + dz,
            );
        }
        labels.push((
            label!("draw-engines", "ENGINES ({count})", count = n as usize),
            p(x1, t.spread(n)[0] + width / 2.0),
        ));
    }
    let (px, dia) = (t.prop_x(), t.diameter);
    let engine_aft = t.engine.map(|(_, length, _, _)| t.engine_x(length).0);
    for y in t.spread(t.count) {
        c.line("OBJECT", p(px, y - dia / 2.0), p(px, y + dia / 2.0), None);
        if t.shaft() {
            if let Some(x) = engine_aft {
                c.centre_line(p(px, y), p(x, y));
            }
            c.line(
                "OBJECT",
                p(px - RUDDER_AFT_OF_D * dia, y),
                p(px - RUDDER_FWD_OF_D * dia, y),
                None,
            );
            for side in [-0.25, 0.25] {
                c.line(
                    "OBJECT",
                    p(px + FLANK_AFT_OF_D * dia, y + side * dia),
                    p(px + FLANK_FWD_OF_D * dia, y + side * dia),
                    None,
                );
            }
        }
    }
    let (w0, w1, ww, _, _, _) = t.wheelhouse;
    rect(c, "OBJECT", w0, w1, dz - ww / 2.0, dz + ww / 2.0);
    let overshoot = 4.0 * c.k;
    c.centre_line(p(-overshoot, 0.0), p(l + overshoot, 0.0));
    if !c.try_dim_vertical(p(0.0, -h), p(0.0, h), DIM_OFFSET_MM) {
        labels.push((
            label!("draw-beam", "BEAM {beam}", beam = metres(t.beam)),
            p(l, h),
        ));
    }
}

fn tug_canvas(t: &TugDraw, n: u32) -> Canvas {
    let mut c = Canvas::new(n);
    let mut labels: Vec<(String, [f64; 2])> = Vec::new();
    elevation(&mut c, t, &mut labels);
    let bottom = t.axis_z().min(0.0) - t.diameter / 2.0;
    let dz = bottom - PLAN_GAP_MM * c.k - t.beam / 2.0;
    plan_view(&mut c, t, dz, &mut labels);
    let column = c.max[0] + 6.0 * c.k;
    c.label_column(column, &mut labels);
    c
}

/// The GA of the tug of an Option's results, when the Option includes it. `option` names the Option
/// in the title block.
pub fn tug_ga_for(results: &Value, option: Option<&str>) -> Result<Value, String> {
    let tug = read_tug(results)?;
    let mut hull = read_hull(results)?;
    hull.option_name = option.map(str::to_string);
    let what = tug.label();
    let title = label!("draw-view-tug-ga", "{tug} GA", tug = what.clone());
    let mut notes = vec![label!(
        "draw-note-dimensions",
        "{view}. DIMENSIONS IN METRES.",
        view = title.clone()
    )];
    notes.push(label!(
        "draw-note-tug",
        "{tug} {length} X {beam} X {draught} M; {count} X {propulsor}, DIAMETER {diameter} M.",
        tug = what.clone(),
        length = fixed(tug.length, 2),
        beam = fixed(tug.beam, 2),
        draught = fixed(tug.draught, 2),
        count = tug.count as usize,
        propulsor = plain_capitals(&tug.propulsor),
        diameter = fixed(tug.diameter, 2)
    ));
    notes.push(label!(
        "draw-note-tug-positions",
        "PROPELLERS, RUDDERS AND PUSH KNEES ARE DRAWN TO SHOW THE ARRANGEMENT, NOT CALCULATED."
    ));
    notes.push(label!(
        "draw-note-scope",
        "FOR CONCEPT AND PRELIMINARY DESIGN ONLY, NOT FOR APPROVAL."
    ));
    let mut fields = title_fields(&hull, &title, 1);
    fields["sheet_count"] = json!(1);
    if let Some(number) = fields["document_number"].as_str() {
        fields["document_number"] = json!(number.replace("-GA-1", "-TUG-1"));
    }
    let name = fields["supplementary_title"]
        .as_str()
        .and_then(|s| s.split(';').next())
        .unwrap_or("OPTION")
        .to_string();
    fields["supplementary_title"] = json!(format!("{name}; {what}"));
    place(notes, fields, RATIOS[0], &|n| tug_canvas(&tug, n)).map(|(document, _)| document)
}

/// As [`tug_ga_for`], from a run outcome.
pub fn tug_ga_named(outcome: &RunOutcome, option: Option<&str>) -> Result<Value, String> {
    match outcome {
        RunOutcome::Results { results, .. } => tug_ga_for(results, option),
        RunOutcome::NotAvailable(message) | RunOutcome::Failed(message) => Err(message.clone()),
    }
}
