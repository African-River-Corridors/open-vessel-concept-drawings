//! The offsets table at the format's edge (issue #451, spec 0008 "Offsets table"): the
//! `hull_offsets` input as rows, the CSV the import reads and the export writes, and the main
//! dimensions an `offsets` Option takes from its table.

use ovd_core::offsets::{Dimensions, OffsetRow};
use ovd_core::text;
use ovd_core::text::Text;
use serde_json::{Value, json};

/// The CSV header of an offsets table.
pub const CSV_HEADER: &str = "station_x_m,y_m,z_m";

/// The rows of a `hull_offsets` value: an array of `{station_x_m, y_m, z_m}`.
pub fn rows_of(v: &Value) -> Result<Vec<OffsetRow>, Text> {
    let Some(list) = v.as_array() else {
        return Err(text!(
            "msg-offsets-no-table",
            "an offsets Hull Form needs its offsets table (`{input}`)",
            input = text::Arg::input("hull_offsets")
        ));
    };
    list.iter()
        .enumerate()
        .map(|(i, r)| {
            let get = |k: &str| r.get(k).and_then(Value::as_f64);
            match (get("station_x_m"), get("y_m"), get("z_m")) {
                (Some(x), Some(y), Some(z)) => Ok(OffsetRow {
                    station_x_m: x,
                    y_m: y,
                    z_m: z,
                }),
                _ => Err(text!(
                    "msg-offsets-bad-row",
                    "row {row} of the offsets table needs the numbers station_x_m, y_m and z_m",
                    row = i + 1
                )),
            }
        })
        .collect()
}

/// The `hull_offsets` value of `rows`.
pub fn rows_json(rows: &[OffsetRow]) -> Value {
    Value::Array(
        rows.iter()
            .map(|r| json!({ "station_x_m": r.station_x_m, "y_m": r.y_m, "z_m": r.z_m }))
            .collect(),
    )
}

/// Read an offsets CSV: UTF-8, comma-separated, the header `station_x_m,y_m,z_m`, one row per
/// point; blank lines (between stations) are allowed.
pub fn parse_csv(text_in: &str) -> Result<Vec<OffsetRow>, Text> {
    let mut lines = text_in
        .lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim()))
        .filter(|(_, l)| !l.is_empty());
    match lines.next() {
        Some((_, h)) if h.replace(' ', "") == CSV_HEADER => {}
        _ => {
            return Err(text!(
                "msg-offsets-csv-header",
                "an offsets CSV starts with the header {header}",
                header = CSV_HEADER
            ));
        }
    }
    lines
        .map(|(n, l)| {
            let cells: Vec<Option<f64>> = l.split(',').map(|c| c.trim().parse().ok()).collect();
            match cells[..] {
                [Some(x), Some(y), Some(z)] => Ok(OffsetRow {
                    station_x_m: x,
                    y_m: y,
                    z_m: z,
                }),
                _ => Err(text!(
                    "msg-offsets-csv-line",
                    "line {line} of the offsets CSV needs three numbers: station_x_m, y_m, z_m",
                    line = n
                )),
            }
        })
        .collect()
}

/// Write rows as an offsets CSV, with a blank line between stations.
pub fn to_csv(rows: &[OffsetRow]) -> String {
    let mut out = String::from(CSV_HEADER);
    out.push('\n');
    for (i, r) in rows.iter().enumerate() {
        if i > 0 && rows[i - 1].station_x_m != r.station_x_m {
            out.push('\n');
        }
        out.push_str(&format!("{},{},{}\n", r.station_x_m, r.y_m, r.z_m));
    }
    out
}

/// The main dimensions of a `hull_offsets` value, when it is a valid table.
pub fn dimensions_of(v: &Value) -> Option<Dimensions> {
    let rows = rows_of(v).ok()?;
    let table = ovd_core::offsets::from_rows(&rows).ok()?;
    Some(ovd_core::offsets::dimensions(&table))
}
