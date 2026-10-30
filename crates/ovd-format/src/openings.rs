//! The `downflooding_openings` input at the format's edge (issue #592): the deck openings through
//! which progressive flooding may take place, as rows of a place along the hull, a place across it
//! and a height above the deck. The engine works the angle of down-flooding from them.

use ovd_core::hydrostatics::Opening;
use ovd_core::text;
use ovd_core::text::Text;
use serde_json::Value;

/// The numbers each row holds, in the order they are written.
pub const COLUMNS: [&str; 3] = ["x_m", "y_m", "height_above_deck_m"];

/// The openings of a `downflooding_openings` value: `null` or an empty list is none. A row is
/// `{x_m, y_m, height_above_deck_m}` and may carry a `name`.
pub fn openings_of(v: &Value) -> Result<Vec<Opening>, Text> {
    let Some(list) = v.as_array() else {
        return Ok(Vec::new());
    };
    list.iter()
        .enumerate()
        .map(|(i, r)| {
            let get = |k: &str| r.get(k).and_then(Value::as_f64).filter(|x| x.is_finite());
            match (get("x_m"), get("y_m"), get("height_above_deck_m")) {
                (Some(x), Some(y), Some(h)) => Ok(Opening {
                    x_m: x,
                    y_m: y,
                    height_above_deck_m: h,
                }),
                _ => Err(text!(
                    "msg-openings-bad-row",
                    "opening {row} needs the numbers x_m, y_m and height_above_deck_m",
                    row = i + 1
                )),
            }
        })
        .collect()
}

/// Whether `row` is an object of the three numbers (and optionally a `name` text).
pub fn is_row(row: &Value) -> bool {
    row.as_object().is_some_and(|o| {
        COLUMNS.iter().all(|c| {
            o.get(*c)
                .and_then(Value::as_f64)
                .is_some_and(f64::is_finite)
        }) && o
            .keys()
            .all(|k| COLUMNS.contains(&k.as_str()) || (k == "name" && o[k].is_string()))
    })
}

/// The problems of the openings against the hull: each outside the length, the breadth at the
/// deck, or with a negative height. One text each, naming the opening by its number.
pub fn range_problems(openings: &[Opening], (l, b): (f64, f64)) -> Vec<Text> {
    let mut out = Vec::new();
    for (i, o) in openings.iter().enumerate() {
        let row = i + 1;
        if !(0.0..=l).contains(&o.x_m) {
            out.push(text!(
                "msg-opening-x",
                "opening {row}: x_m = {value} is outside the hull (0 to {limit} m)",
                row = row,
                value = crate::vdesign::number(o.x_m),
                limit = crate::vdesign::number(l)
            ));
        }
        if o.y_m.abs() > b / 2.0 {
            out.push(text!(
                "msg-opening-y",
                "opening {row}: y_m = {value} is outside the breadth ({limit} m each side)",
                row = row,
                value = crate::vdesign::number(o.y_m),
                limit = crate::vdesign::number(b / 2.0)
            ));
        }
        if o.height_above_deck_m < 0.0 {
            out.push(text!(
                "msg-opening-height",
                "opening {row}: height_above_deck_m = {value} is below the deck",
                row = row,
                value = crate::vdesign::number(o.height_above_deck_m)
            ));
        }
    }
    out
}
