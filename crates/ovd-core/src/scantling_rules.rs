//! The scantling Rules of ABS Steel Barges at the current edition (issue #740, spec 0013): the Rule
//! Set `abs-steel-barges`, as the panel model (`hull_panels`) reads it.
//!
//! **Concept and preliminary design only (ADR 0011). Not a structural approval.**
//!
//! A Rule is data (ADR 0013): a formula shape coded in `rule_trace::formula_value` and its
//! coefficients, here and in `crates/ovd-format/rules/abs-steel-barges/1.0.0.json`. A test in
//! `ovd-format` holds the two equal, coefficient by coefficient, so the file a reviewer reads is the
//! data the model uses. The core keeps its own copy because it is `no_std` and does not parse JSON.
//!
//! Edition: the Rules for Building and Classing Steel Barges of January 2026 are Part 3 January 2026
//! and Part 5 July 2025 (Notices and General Information, January 2026, Table 1). Every paragraph was
//! read in the July 2025 Rules; the January 2026 Rule Change Notice (Table 3) changes none of the
//! paragraphs below (its Part 3 changes are 3-2-3/15, 3-2-5/3, 3-2-6/2 commentary, 3-2-8 and 3-2-9).

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// The Rule Set id and version.
pub const RULE_SET: &str = "abs-steel-barges";
pub const RULE_SET_VERSION: &str = "1.0.0";

/// The edition every Rule cites.
pub const EDITION: &str = "ABS Rules for Building and Classing Steel Barges, January 2026 (Part 3 January 2026, Part 5 July 2025)";

/// One scantling Rule: its id (`abs-steel-barges/<paragraph>/<slug>`, the paragraph written with a
/// dot for the slash, as the Rule Set ids allow, `3-2-2.3.1` for 3-2-2/3.1), the paragraph in ABS
/// notation, the formula shape and its coefficients. A Rule whose
/// shape is `value` is a constant (`kind: limit` in the Rule Set file), its coefficient `value`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScantlingRule {
    pub id: &'static str,
    pub paragraph: &'static str,
    pub shape: &'static str,
    pub coefficients: &'static [(&'static str, f64)],
}

const fn rule(
    id: &'static str,
    paragraph: &'static str,
    shape: &'static str,
    coefficients: &'static [(&'static str, f64)],
) -> ScantlingRule {
    ScantlingRule {
        id,
        paragraph,
        shape,
        coefficients,
    }
}

/// The section modulus coefficients of 3-2-6/5.3 and 5.5 and 3-2-7/3.3 (k = 7.8 for a stiffener,
/// 4.74 for a primary member), one Rule per member kind with its c.
const STIFFENER_K: f64 = 7.8;
const PRIMARY_K: f64 = 4.74;

/// Every scantling Rule of the set, in the order the panel model applies them.
pub static RULES: &[ScantlingRule] = &[
    rule(
        "abs-steel-barges/3-1-1.3.1/rule-length",
        "3-1-1/3.1",
        "value",
        &[("value", 0.96)],
    ),
    rule(
        "abs-steel-barges/3-1-1.3.1/rule-length-waterline",
        "3-1-1/3.1",
        "value",
        &[("value", 0.85)],
    ),
    rule(
        "abs-steel-barges/3-2-1.3.1/hull-girder-modulus",
        "3-2-1/3.1",
        "abs-smr",
        &[
            ("c1_min", 4.11),
            ("l_a", 45.0),
            ("qa", 16.33),
            ("qb", 15.47),
            ("qc", 7.77),
            ("l_div", 100.0),
            ("l_b", 95.0),
            ("c1_max", 10.75),
            ("l_c", 300.0),
            ("l_d", 350.0),
            ("l_div_d", 150.0),
            ("p", 1.5),
            ("c2", 0.01),
            ("cb_add", 0.7),
        ],
    ),
    rule(
        "abs-steel-barges/3-2-2.3.1/side-shell",
        "3-2-2/3.1",
        "linear-ls-split",
        &[
            ("a1", 0.07),
            ("b1", 0.007),
            ("c1", 0.0),
            ("a2", 0.03),
            ("b2", 0.007),
            ("c2", 6.0),
            ("x_max", 150.0),
            ("strict", 0.0),
        ],
    ),
    rule(
        "abs-steel-barges/3-2-2.3.1.1/side-shell-reduction",
        "3-2-2/3.1.1",
        "value",
        &[("value", 0.5)],
    ),
    rule(
        "abs-steel-barges/3-2-2.3.1.1/side-shell-reduction-breadth",
        "3-2-2/3.1.1",
        "value",
        &[("value", 0.2)],
    ),
    rule(
        "abs-steel-barges/3-2-2.3.5.1/bottom-shell",
        "3-2-2/3.5.1",
        "linear-ls-split-capped",
        &[
            ("a1", 0.045),
            ("b1", 0.007),
            ("c1", 1.8),
            ("a2", 0.08),
            ("b2", 0.007),
            ("c2", -2.5),
            ("x_max", 123.0),
            ("x_cap", 225.0),
        ],
    ),
    rule(
        "abs-steel-barges/3-2-2.5.1/shell-ends",
        "3-2-2/5.1",
        "linear-ls-split-capped",
        &[
            ("a1", 0.055),
            ("b1", 0.007),
            ("c1", 1.0),
            ("a2", 0.015),
            ("b2", 0.01),
            ("c2", 2.2),
            ("x_max", 76.0),
            ("x_cap", 225.0),
        ],
    ),
    rule(
        "abs-steel-barges/3-2-3.2.1.2(a)/strength-deck",
        "3-2-3/2.1.2(a)",
        "linear-ls-split",
        &[
            ("a1", 0.0),
            ("b1", 0.009),
            ("c1", 2.4),
            ("a2", 0.0),
            ("b2", 0.006),
            ("c2", 4.7),
            ("x_max", 760.0),
            ("strict", 0.0),
        ],
    ),
    rule(
        "abs-steel-barges/3-2-3.2.1.2(b)/strength-deck",
        "3-2-3/2.1.2(b)",
        "deck-b",
        &[("l_max", 183.0), ("c1", 48.76), ("k", 26.0), ("c2", 8681.0)],
    ),
    rule(
        "abs-steel-barges/3-2-3.2.7/deck-ends",
        "3-2-3/2.7",
        "linear-ls",
        &[("a", 0.0), ("b", 0.01), ("c", 0.0)],
    ),
    rule(
        "abs-steel-barges/5-1-2.5.17/inner-bottom",
        "5-1-2/5.17",
        "linear-ls-less",
        &[("a", 0.037), ("b", 0.009), ("c", -0.5), ("less", 1.0)],
    ),
    rule(
        "abs-steel-barges/3-2-7.3.1/tank-bulkhead-plating",
        "3-2-7/3.1",
        "plate-sqrt-head",
        &[
            ("d", 254.0),
            ("c", 2.5),
            ("t_min", 6.5),
            ("e", 150.0),
            ("f", 2.5),
            ("a_max", 2.0),
            ("k_long", 1.0),
            ("a_min", 1.0),
            ("k1", 3.075),
            ("k2", 2.077),
            ("k3", 0.272),
        ],
    ),
    rule(
        "abs-steel-barges/3-2-6.5.1/watertight-bulkhead-plating",
        "3-2-6/5.1",
        "plate-sqrt-head",
        &[
            ("d", 290.0),
            ("c", 1.5),
            ("t_min", 6.0),
            ("e", 200.0),
            ("f", 2.5),
            ("a_max", 2.0),
            ("k_long", 1.0),
            ("a_min", 1.0),
            ("k1", 3.075),
            ("k2", 2.077),
            ("k3", 0.272),
        ],
    ),
    rule(
        "abs-steel-barges/5-2-1.3/tank-head",
        "5-2-1/3",
        "tank-head",
        &[
            ("h_short", 1.22),
            ("l_short", 61.0),
            ("k", 0.02),
            ("l_long", 122.0),
            ("h_long", 2.44),
        ],
    ),
    rule(
        "abs-steel-barges/3-2-6.5.3/bulkhead-head",
        "3-2-6/5.3",
        "bulkhead-head",
        &[("d_max", 6.10), ("k", 0.8), ("c", 1.22)],
    ),
    rule(
        "abs-steel-barges/3-2-4.3.11.1/web-in-tanks",
        "3-2-4/3.11.1",
        "web-minimum",
        &[
            ("a", 0.01),
            ("c", 3.0),
            ("t_cap", 11.5),
            ("l_floor", 29.0),
            ("l_min", 75.0),
            ("d1", 36.0),
            ("c1", 7.0),
            ("d2", 20.0),
            ("c2", 5.33),
        ],
    ),
    rule(
        "abs-steel-barges/3-2-4.3.11.2/web-clear-of-tanks",
        "3-2-4/3.11.2",
        "web-minimum",
        &[
            ("a", 0.01),
            ("c", 3.0),
            ("t_cap", 11.5),
            ("l_floor", 29.0),
            ("l_min", 75.0),
            ("d1", 36.0),
            ("c1", 5.5),
            ("d2", 60.0),
            ("c2", 6.35),
        ],
    ),
    rule(
        "abs-steel-barges/5-1-2.5.1.1/center-girder",
        "5-1-2/5.1.1",
        "linear-ls",
        &[("a", 0.056), ("b", 0.0), ("c", 5.5)],
    ),
    rule(
        "abs-steel-barges/5-1-2.5.1.2/center-girder-ends",
        "5-1-2/5.1.2",
        "value",
        &[("value", 0.85)],
    ),
    rule(
        "abs-steel-barges/5-1-2.5.5/side-girders",
        "5-1-2/5.5",
        "linear-ls",
        &[("a", 0.036), ("b", 0.0), ("c", 4.5)],
    ),
    rule(
        "abs-steel-barges/5-1-2.5.5/side-girder-spacing",
        "5-1-2/5.5",
        "value",
        &[("value", 4.6)],
    ),
    rule(
        "abs-steel-barges/5-1-2.5.7.1/solid-floors",
        "5-1-2/5.7.1",
        "linear-ls",
        &[("a", 0.036), ("b", 0.0), ("c", 6.0)],
    ),
    rule(
        "abs-steel-barges/5-1-1.3.1/deck-beam-head",
        "5-1-1/3.1",
        "linear-ls-split",
        &[
            ("a1", 0.02),
            ("b1", 0.0),
            ("c1", 0.76),
            ("a2", 0.0),
            ("b2", 0.0),
            ("c2", 2.9),
            ("x_max", 107.0),
            ("strict", 1.0),
        ],
    ),
    rule(
        "abs-steel-barges/5-1-1.3.1/deck-longitudinals",
        "5-1-1/3.1",
        "sm-chsl2-hull-k",
        &[
            ("k", STIFFENER_K),
            ("p", 1.709),
            ("q", 0.651),
            ("hull_k", 1.0),
        ],
    ),
    rule(
        "abs-steel-barges/5-2-2.3.7/bottom-longitudinals",
        "5-2-2/3.7",
        "sm-chsl2",
        &[("factor", 1.0), ("k", STIFFENER_K), ("c", 1.34)],
    ),
    rule(
        "abs-steel-barges/5-2-2.3.5/inner-bottom-longitudinals",
        "5-2-2/3.5",
        "sm-chsl2",
        &[("factor", 0.85), ("k", STIFFENER_K), ("c", 1.30)],
    ),
    rule(
        "abs-steel-barges/3-2-7.3.3/tank-boundary-longitudinals",
        "3-2-7/3.3",
        "sm-chsl2",
        &[("factor", 1.0), ("k", STIFFENER_K), ("c", 1.0)],
    ),
    rule(
        "abs-steel-barges/3-2-7.3.3/longitudinal-bulkhead-stiffeners",
        "3-2-7/3.3",
        "sm-chsl2",
        &[("factor", 1.0), ("k", STIFFENER_K), ("c", 0.90)],
    ),
    rule(
        "abs-steel-barges/5-2-1.7/side-longitudinals",
        "5-2-1/7",
        "sm-chsl2",
        &[("factor", 1.0), ("k", STIFFENER_K), ("c", 1.25)],
    ),
    rule(
        "abs-steel-barges/3-2-6.5.3/bulkhead-stiffeners",
        "3-2-6/5.3",
        "sm-chsl2",
        &[("factor", 1.0), ("k", STIFFENER_K), ("c", 0.56)],
    ),
    rule(
        "abs-steel-barges/3-2-6.5.5/bulkhead-webs",
        "3-2-6/5.5",
        "sm-chsl2",
        &[("factor", 1.0), ("k", PRIMARY_K), ("c", 1.0)],
    ),
    rule(
        "abs-steel-barges/5-2-1.11/side-webs",
        "5-2-1/11",
        "sm-chsl2",
        &[("factor", 1.0), ("k", PRIMARY_K), ("c", 1.75)],
    ),
    rule(
        "abs-steel-barges/5-2-1.11/deck-transverses",
        "5-2-1/11",
        "sm-chsl2",
        &[("factor", 1.0), ("k", PRIMARY_K), ("c", 2.0)],
    ),
];

/// The Rule of an id. Panics on an id that is not in [`RULES`]: the ids are fixed in the code.
pub fn get(id: &str) -> &'static ScantlingRule {
    RULES
        .iter()
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("no scantling Rule `{id}`"))
}

/// A coefficient of a Rule. Panics on a name the Rule does not hold.
pub fn coefficient(id: &str, name: &str) -> f64 {
    get(id)
        .coefficients
        .iter()
        .find(|(k, _)| *k == name)
        .map(|(_, v)| *v)
        .unwrap_or_else(|| panic!("scantling Rule `{id}` has no coefficient `{name}`"))
}

/// The value of a formula Rule at the given operands, by its shape (`rule_trace::formula_value`).
/// Panics when the shape or an operand is missing: the table and the shapes are fixed in the code
/// and tested.
pub fn value(id: &str, operands: &[(&str, f64)]) -> f64 {
    let r = get(id);
    let coefficients: Vec<(String, f64)> = r
        .coefficients
        .iter()
        .map(|(k, v)| (k.to_string(), *v))
        .collect();
    crate::rule_trace::formula_value(r.shape, &coefficients, |name| {
        operands
            .iter()
            .find(|(k, _)| *k == name)
            .map(|(_, v)| *v)
            .ok_or_else(|| format!("scantling Rule `{id}` needs operand `{name}`"))
    })
    .unwrap_or_else(|e| panic!("{e}"))
}
