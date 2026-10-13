//! The banned-maths rule (`clippy.toml`) covers every banned `f64` method for `f32` too. The core
//! is `f64`, but an `f32` transcendental would still call the platform's libm and break
//! bit-identical results across platforms.

const CLIPPY_TOML: &str = include_str!("../clippy.toml");

/// Method names banned for one float type, from `path = "<ty>::<name>"` entries.
fn banned(ty: &str) -> Vec<String> {
    let prefix = format!("path = \"{ty}::");
    let mut names: Vec<String> = CLIPPY_TOML
        .lines()
        .filter_map(|l| l.split_once(&prefix).map(|(_, rest)| rest))
        .filter_map(|rest| rest.split_once('"').map(|(name, _)| name.to_string()))
        .collect();
    names.sort();
    names
}

#[test]
fn every_banned_f64_method_is_banned_for_f32_too() {
    let f64s = banned("f64");
    assert!(f64s.len() >= 27, "the f64 list shrank: {f64s:?}");
    let f32s = banned("f32");
    let missing: Vec<&String> = f64s.iter().filter(|n| !f32s.contains(n)).collect();
    assert!(missing.is_empty(), "f32 methods not banned: {missing:?}");
}
