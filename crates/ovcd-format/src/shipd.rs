//! The `shipd_parameters` input at the format's edge (issue #452, spec 0008 "ShipD-style hull"):
//! the object of the dataset's header names read into `ShipdParameters`, and the checks a Design
//! gets for it.

use ovcd_core::shipd::{NAMES, ShipdParameters};
use ovcd_core::text;
use ovcd_core::text::Text;
use serde_json::Value;

/// The parameters of a `shipd_parameters` value at the Option's length, beam and depth. `LOA`,
/// `Bd` and `Dd` come from the dimensions, not from the object; every other name is needed.
pub fn parameters_of(
    v: &Value,
    (loa, beam, depth): (f64, f64, f64),
) -> Result<ShipdParameters, Text> {
    let Some(o) = v.as_object() else {
        return Err(text!(
            "msg-shipd-no-parameters",
            "a ShipD Hull Form needs its parameters (`{input}`)",
            input = text::Arg::input("shipd_parameters")
        ));
    };
    let mut values = [0.0; 45];
    for (i, name) in NAMES.iter().enumerate() {
        values[i] = match *name {
            "LOA" => loa,
            "Bd" => beam / loa,
            "Dd" => depth / loa,
            _ => o.get(*name).and_then(Value::as_f64).ok_or_else(|| {
                text!(
                    "msg-shipd-missing",
                    "`{input}` needs the ShipD parameter {name}",
                    input = text::Arg::input("shipd_parameters"),
                    name = *name
                )
            })?,
        };
    }
    Ok(ShipdParameters(values))
}

/// The messages of an Option's ShipD parameters: one error per failed constraint, with its index
/// and name, and an error for a bulb (built with #453).
pub fn messages(v: &Value, dims: (f64, f64, f64)) -> Vec<Text> {
    let p = match parameters_of(v, dims) {
        Ok(p) => p,
        Err(e) => return vec![e],
    };
    let mut out = Vec::new();
    if p.0[31] != 0.0 || p.0[32] != 0.0 {
        out.push(text!(
            "msg-shipd-bulb",
            "`{input}`: a ShipD hull with a bulbous bow or stern is not built yet (bit_BB and bit_SB \
             must be 0)",
            input = text::Arg::input("shipd_parameters")
        ));
    }
    let (l, b, d) = dims;
    for f in ovcd_core::shipd::check(&p, l, b, d) {
        out.push(text!(
            "msg-shipd-constraint",
            "`{input}` fails ShipD constraint {index} ({name})",
            input = text::Arg::input("shipd_parameters"),
            index = f.index,
            name = f.name
        ));
    }
    out
}
