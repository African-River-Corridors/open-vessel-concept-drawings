//! The banned-maths rule covers this crate too (ticket 135): `simulate` and the other modules here
//! compute the results that the results hash covers, so they follow the same rule as `ovcd-core`.
//! Clippy reads `clippy.toml` from each crate's own directory, so the list is kept in two files.

const FORMAT: &str = include_str!("../clippy.toml");
const CORE: &str = include_str!("../../ovcd-core/clippy.toml");

fn entries(toml: &str) -> Vec<&str> {
    toml.lines()
        .filter(|l| l.trim_start().starts_with("{ path ="))
        .collect()
}

#[test]
fn this_crate_bans_exactly_what_the_core_bans() {
    let (format, core) = (entries(FORMAT), entries(CORE));
    assert!(core.len() >= 54, "the core list shrank: {}", core.len());
    assert_eq!(
        format, core,
        "crates/ovcd-format/clippy.toml differs from crates/ovcd-core/clippy.toml"
    );
}

#[test]
fn only_the_drawing_geometry_is_allowed_to_use_the_banned_methods() {
    // Every `allow(clippy::disallowed_methods)` in this crate, by file. ga.rs draws; nothing else may.
    let sources = [
        ("eval.rs", include_str!("../src/eval.rs")),
        (
            "gas_arrangement.rs",
            include_str!("../src/gas_arrangement.rs"),
        ),
        ("gas_stability.rs", include_str!("../src/gas_stability.rs")),
        ("lib.rs", include_str!("../src/lib.rs")),
        ("profile.rs", include_str!("../src/profile.rs")),
        ("run.rs", include_str!("../src/run.rs")),
        ("session.rs", include_str!("../src/session.rs")),
        ("simulate.rs", include_str!("../src/simulate.rs")),
        ("vdesign.rs", include_str!("../src/vdesign.rs")),
    ];
    for (name, text) in sources {
        assert!(
            !text.contains("disallowed_methods"),
            "{name} lifts the banned-maths rule"
        );
    }
    let ga = include_str!("../src/ga.rs");
    assert_eq!(
        ga.matches("disallowed_methods").count(),
        1,
        "ga.rs: one allow, on the arc function"
    );
}
