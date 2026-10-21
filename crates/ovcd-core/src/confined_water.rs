//! Confined water: the midship section, the blockage ratio and the limiting speed of a vessel in a
//! channel (issues #396 and #397). Squat, resistance and the electric energy path all use these, so
//! one hull has one blockage ratio in one run.
//!
//! Source: G. Delefortrie, J. Verwilligen, M. Vantorre, E. Lataire, "Exact and Approximated
//! Solutions to the Critical Ship Speeds in Canals", 35th PIANC World Congress, Cape Town, 2024,
//! Paper 8, pp. 649-654. It proves the critical speeds of Schijf (1949): with the blockage ratio
//! m = A_M / Ω (midship section over channel section) and the Froude depth number
//! Fr_h = V / √(g h) on the water depth h, the first critical speed (section 2, and the summary in
//! section 4) is Fr_h,crit1 = [2 sin(arcsin(1 − m) / 3)]^(3/2).

/// The part of the limiting speed a vessel may use: an OVCD choice, not from a source. The speed
/// sweep stops here, and the service speed must stay at or below it.
pub const LIMITING_SPEED_MARGIN: f64 = 0.95;

/// The midship section area (m²): B × T. The midship coefficient is taken as 1 until it is an
/// input (#425); a barge midship section is close to a full rectangle.
pub fn midship_section_area(beam: f64, draught: f64) -> f64 {
    beam * draught
}

/// The blockage ratio m = A_M / Ω; 0 for no channel section.
pub fn blockage_ratio(midship_area: f64, channel_area: f64) -> f64 {
    if channel_area > 0.0 {
        midship_area / channel_area
    } else {
        0.0
    }
}

/// The first critical Froude depth number of Schijf, V / √(g h): [2 sin(arcsin(1 − m) / 3)]^(3/2).
/// It is 1 in open water (m = 0) and 0 for a channel the vessel fills (m ≥ 1).
pub fn critical_froude_depth(m: f64) -> f64 {
    if m >= 1.0 {
        return 0.0;
    }
    let m = m.max(0.0);
    libm::pow(2.0 * libm::sin(libm::asin(1.0 - m) / 3.0), 1.5)
}
