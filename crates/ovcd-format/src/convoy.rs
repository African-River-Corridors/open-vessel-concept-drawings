//! The totals of a pushed convoy (issue #358, spec 0007, "Convoy totals").
//!
//! A pushed convoy is `abreast` x `in_line` identical barges. The Option's inputs describe one
//! barge, and `simulate` runs that barge as a pushed barge. This module adds the `convoy` block to
//! the result: the barge, and the totals of every barge (cargo, lightship, deadweight, the overall
//! length and beam). The Waterway Limit checks of the lock (`add_lock_checks`) are on the overall
//! size of a self-propelled hull, a pushed barge or a convoy. The pusher is not in the totals yet
//! (#359).

use crate::text_ids::TextIds;
use crate::vdesign::DesignOption;
use ovcd_core::checks::Check;
use ovcd_core::text;
use ovcd_core::text::Arg;
use serde_json::{Map, Value, json};

/// The barges abreast and in line of a pushed convoy Option; `None` for any other Formation.
pub(crate) fn counts(option: &DesignOption) -> Option<(u64, u64)> {
    let formation = &option.input("formation")?.value;
    if formation["kind"] != "pushed_convoy" {
        return None;
    }
    Some((
        formation["abreast"].as_u64()?,
        formation["in_line"].as_u64()?,
    ))
}

fn lock_limit(option: &DesignOption, name: &str) -> Option<f64> {
    option.input(name).and_then(|i| i.value.as_f64())
}

/// Add the `convoy` block to the result of one barge.
pub(crate) fn add_totals(abreast: u64, in_line: u64, v: &mut Value) {
    let n = (abreast * in_line) as f64;
    let barge = |key: &str| v["results"][key].as_f64();
    let loa = v["dims"]["loa_m"].as_f64();
    let beam = v["dims"]["beam_m"].as_f64();
    let total = |x: Option<f64>| x.map_or(Value::Null, |x| json!(n * x));
    let overall_length = loa.map(|l| in_line as f64 * l);
    let overall_beam = beam.map(|b| abreast as f64 * b);
    let mut block = Map::new();
    block.insert("abreast".into(), json!(abreast));
    block.insert("in_line".into(), json!(in_line));
    block.insert("barges".into(), json!(abreast * in_line));
    block.insert(
        "barge".into(),
        json!({
            "cargo_t": barge("cargo_t"),
            "lightship_t": barge("lightship_t"),
            "deadweight_t": barge("deadweight_t"),
            "displacement_t": barge("displacement_t"),
            "cargo_volume_m3": barge("cargo_volume_m3"),
        }),
    );
    block.insert(
        "totals".into(),
        json!({
            "cargo_t": total(barge("cargo_t")),
            "lightship_t": total(barge("lightship_t")),
            "deadweight_t": total(barge("deadweight_t")),
            "displacement_t": total(barge("displacement_t")),
            "cargo_volume_m3": total(barge("cargo_volume_m3")),
            "overall_length_m": overall_length,
            "overall_beam_m": overall_beam,
        }),
    );
    // The pusher joins the totals with #359.
    block.insert("pusher_included".into(), json!(false));

    if let Some(o) = v.as_object_mut() {
        o.insert("convoy".into(), Value::Object(block));
    }
}

/// Make the `cost` block of a pushed convoy the budget of every barge (issue #552). The block the
/// budget gives is one barge with its tug. The barge items and their subtotal are multiplied by the
/// number of barges; the tug stays as it is, one tug for the whole convoy (the pusher joins with #359);
/// the yard margin is the same fraction of the new subtotals (worked out from the rounded subtotals, so it can differ by a unit from a figure worked out from the unrounded ones); the delivery stays one lump. The block of
/// one barge is kept beside it as `one_barge`, so a reader can see the basis. A budget that is `null`
/// stays `null`. `barges` is the count.
pub(crate) fn scale_cost(barges: u64, cost: &mut Value) {
    if !cost.is_object() {
        return;
    }
    let one_barge = cost.clone();
    // One barge is the budget as it is: no figure is worked out again, so none is rounded again.
    if barges <= 1 {
        cost["barges"] = json!(barges);
        cost["one_barge"] = one_barge;
        return;
    }
    let n = barges as f64;
    let whole = |v: &Value| v.as_f64();
    let (Some(barge_sub), Some(tug_sub), Some(fraction), Some(delivery)) = (
        whole(&cost["barge_subtotal"]),
        whole(&cost["tug_subtotal"]),
        whole(&cost["yard_margin_fraction"]),
        whole(&cost["delivery"]),
    ) else {
        return;
    };
    let scaled_sub = barge_sub * n;
    // The core's own rounding: Python's round, half to even, to a whole amount.
    let margin = (fraction * (scaled_sub + tug_sub)).round_ties_even();
    let total = scaled_sub + tug_sub + margin + delivery;
    if let Some(items) = cost["barge"].as_array_mut() {
        for item in items {
            if let Some(a) = item["amount"].as_f64() {
                item["amount"] = json!((a * n) as i64);
            }
        }
    }
    cost["barge_subtotal"] = json!(scaled_sub as i64);
    cost["yard_margin"] = json!(margin as i64);
    cost["unit_total"] = json!(total as i64);
    cost["barges"] = json!(barges);
    cost["one_barge"] = one_barge;
}

/// The Waterway Limit checks of the lock, on the overall size of the Formation: a self-propelled
/// hull and a pushed barge are one hull; a pushed convoy is `abreast` x `in_line` barges. An ATB or
/// a tow is not checked yet. Nothing is added when the Option gives no lock.
pub(crate) fn add_lock_checks(option: &DesignOption, v: &mut Value) {
    let (abreast, in_line) = match counts(option) {
        Some(c) => c,
        None => match option.input("formation").map(|i| i.value["kind"].as_str()) {
            Some(Some("self_propelled" | "pushed_barge")) => (1, 1),
            _ => return,
        },
    };
    let loa = v["dims"]["loa_m"].as_f64();
    let beam = v["dims"]["beam_m"].as_f64();
    let overall_length = loa.map(|l| in_line as f64 * l);
    let overall_beam = beam.map(|b| abreast as f64 * b);
    let mut checks = Vec::new();
    if let (Some(limit), Some(length)) = (lock_limit(option, "lock_length_m"), overall_length) {
        checks.push(Check::new(
            "within lock length",
            length <= limit,
            text!(
                "check-lock-length",
                "overall length {length} m <= lock length {limit} m",
                length = Arg::fixed(length, 2),
                limit = Arg::py(limit),
            ),
        ));
    }
    if let (Some(limit), Some(width)) = (lock_limit(option, "lock_width_m"), overall_beam) {
        checks.push(Check::new(
            "within lock width",
            width <= limit,
            text!(
                "check-lock-width",
                "overall beam {width} m <= lock width {limit} m",
                width = Arg::fixed(width, 2),
                limit = Arg::py(limit),
            ),
        ));
    }
    let mut ids = TextIds::default();
    ids.checks(&checks);
    let listed: Vec<Value> = checks
        .iter()
        .map(
            |c| json!({ "name": c.name, "ok": c.ok, "message": c.message, "severity": c.severity }),
        )
        .collect();
    let new_ids = ids.finish(&Value::Array(listed.clone()));
    if checks.is_empty() {
        return;
    }
    if let Some(o) = v.as_object_mut() {
        if let Some(Value::Array(all)) = o.get_mut("checks") {
            all.extend(listed);
        }
        if let (Some(Value::Object(have)), Value::Object(add)) = (o.get_mut("text_ids"), new_ids) {
            for (k, val) in add {
                have.entry(k).or_insert(val);
            }
        }
    }
}
