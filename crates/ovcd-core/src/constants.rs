//! Constants shared by the resistance, squat and wave modules.

/// Standard acceleration of gravity, m/s^2: the 3rd CGPM (1901) value. Source not held in this
/// repository.
pub const GRAVITY: f64 = 9.80665;

/// Metres per second in one knot: 1 kn = 1852 m per 3600 s (the international nautical mile is
/// 1852 m). Source not held in this repository.
pub const KNOTS_TO_MS: f64 = 1852.0 / 3600.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resistance_squat_and_waves_share_one_definition() {
        assert_eq!(crate::resistance::GRAVITY, GRAVITY);
        assert_eq!(crate::squat::GRAVITY, GRAVITY);
        assert_eq!(crate::waves::GRAVITY, GRAVITY);
        assert_eq!(crate::resistance::KNOTS_TO_MS, KNOTS_TO_MS);
        assert_eq!(KNOTS_TO_MS, 1852.0 / 3600.0);
    }
}
