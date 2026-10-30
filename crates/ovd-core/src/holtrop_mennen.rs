//! Holtrop–Mennen calm-water resistance (issue #723, ADR 0023): the form-sensitive resistance
//! method for hulls finer than a bluff barge.
//!
//! Sources:
//! - J. Holtrop and G.G.J. Mennen, "An approximate power prediction method", International
//!   Shipbuilding Progress 29 (335), 1982, pp. 166-170 (`Variant::Hm1982`; its section 5 is the
//!   numerical example the tests check);
//! - J. Holtrop, "A statistical re-analysis of resistance and propulsion data", International
//!   Shipbuilding Progress 31 (363), 1984, pp. 272-276 (`Variant::Hm1984`: the 1984 form factor
//!   1 + k1 and the low-speed wave resistance R_W-A, valid for Fn up to 0.4).
//!
//! The bulb, transom, correlation and appendage terms are the 1982 ones in both variants: the 1984
//! paper keeps them.
//!
//! Where a hull gives no waterplane coefficient or centre of buoyancy, the estimates of OpenTNSim
//! (TU Delft, MIT licence, `opentnsim/energy/calculations.py` at commit cb6bdda, after Van
//! Koningsveld et al. 2023, Part IV, Eq 5.10 and Table 5.1) stand in: C_WP = (1 + 2 C_B) / 3 and
//! lcb = -13.5 + 19.4 C_P (per cent of L forward of mid-length). This module follows the
//! papers, not OpenTNSim, where the two differ: OpenTNSim uses the 1982 wave term with the 1984
//! form factor, and its c7 for B/L below 0.11 is overwritten by the next branch.
//!
//! Each method has a declared validity envelope ([`Envelope`]). Outside it the formulas still give
//! a number, and [`Envelope::outside`] names each parameter that is out of range.

use alloc::vec::Vec;
use libm::{cos, exp, log10, pow, sqrt};

/// Which paper's form factor and wave resistance to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    /// Holtrop and Mennen 1982: form factor with c12, c13; wave term with m2.
    Hm1982,
    /// Holtrop 1984: form factor with c14; low-speed wave term R_W-A with m4.
    Hm1984,
}

/// The hull as the method reads it. Lengths in m, areas in m², volume in m³.
#[derive(Debug, Clone, PartialEq)]
pub struct HullInputs {
    /// Waterline length L.
    pub lwl: f64,
    /// Moulded breadth B.
    pub beam: f64,
    /// Mean moulded draught T.
    pub draught: f64,
    /// Draught at the forward perpendicular T_F.
    pub draught_fwd: f64,
    /// Displacement volume ∇.
    pub volume_m3: f64,
    /// Longitudinal centre of buoyancy, per cent of L forward of 0.5 L (negative is aft).
    pub lcb_pct: f64,
    /// Midship section coefficient C_M.
    pub cm: f64,
    /// Waterplane coefficient C_WP.
    pub cwp: f64,
    /// Wetted surface of the hull S.
    pub wetted_surface_m2: f64,
    /// Transverse bulb area A_BT at the stem, and its centre above the keel h_B.
    pub bulb_area_m2: f64,
    pub bulb_centre_m: f64,
    /// Immersed transom area at rest A_T.
    pub transom_area_m2: f64,
    /// Wetted area of the appendages S_APP and their factor 1 + k2.
    pub appendage_area_m2: f64,
    pub appendage_factor: f64,
    /// Afterbody shape C_stern: -25 pram with gondola, -10 V sections, 0 normal, +10 U sections
    /// with a Hogner stern (1984 table).
    pub stern_shape: f64,
}

impl HullInputs {
    /// A hull with no bulb, transom or appendages and a normal afterbody.
    pub fn plain(lwl: f64, beam: f64, draught: f64, volume_m3: f64, cm: f64, cwp: f64) -> Self {
        let cp = volume_m3 / (lwl * beam * draught * cm);
        Self {
            lwl,
            beam,
            draught,
            draught_fwd: draught,
            volume_m3,
            lcb_pct: lcb_estimate(cp),
            cm,
            cwp,
            wetted_surface_m2: wetted_surface_estimate(lwl, beam, draught, volume_m3, cm, cwp, 0.0),
            bulb_area_m2: 0.0,
            bulb_centre_m: 0.0,
            transom_area_m2: 0.0,
            appendage_area_m2: 0.0,
            appendage_factor: 1.0,
            stern_shape: 0.0,
        }
    }

    /// Block coefficient ∇ / (L B T).
    pub fn cb(&self) -> f64 {
        self.volume_m3 / (self.lwl * self.beam * self.draught)
    }

    /// Prismatic coefficient C_B / C_M.
    pub fn cp(&self) -> f64 {
        self.cb() / self.cm
    }
}

/// The water and the speed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flow {
    /// Speed through the water (m/s).
    pub speed_ms: f64,
    /// Density (kg/m³).
    pub rho: f64,
    /// Kinematic viscosity (m²/s).
    pub nu: f64,
    /// Gravity (m/s²).
    pub g: f64,
}

/// Every intermediate of one evaluation, named as in the papers. Forces in kN.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Components {
    pub froude: f64,
    pub cp: f64,
    pub run_length_m: f64,
    pub form_factor: f64,
    pub cf: f64,
    pub r_f_kn: f64,
    pub r_app_kn: f64,
    pub c7: f64,
    pub entrance_angle_deg: f64,
    pub c1: f64,
    pub c3: f64,
    pub c2: f64,
    pub c5: f64,
    pub m1: f64,
    pub c15: f64,
    /// m2 (1982) or m4 (1984).
    pub m_hump: f64,
    pub lambda: f64,
    pub r_w_kn: f64,
    pub pb: f64,
    pub fni: f64,
    pub r_b_kn: f64,
    pub fnt: f64,
    pub r_tr_kn: f64,
    pub ca: f64,
    pub r_a_kn: f64,
    /// R_F (1 + k1) + R_APP + R_W + R_B + R_TR + R_A.
    pub total_kn: f64,
}

/// The estimate of lcb (per cent of L forward of mid-length) from C_P, OpenTNSim (Van Koningsveld
/// et al. 2023, Part IV, Table 5.1).
pub fn lcb_estimate(cp: f64) -> f64 {
    -13.5 + 19.4 * cp
}

/// The estimate of C_WP from C_B, OpenTNSim (Van Koningsveld et al. 2023, Part IV, Eq 5.10).
pub fn cwp_estimate(cb: f64) -> f64 {
    (1.0 + 2.0 * cb) / 3.0
}

/// The wetted surface of Holtrop and Mennen 1982.
pub fn wetted_surface_estimate(
    lwl: f64,
    beam: f64,
    draught: f64,
    volume_m3: f64,
    cm: f64,
    cwp: f64,
    bulb_area_m2: f64,
) -> f64 {
    let cb = volume_m3 / (lwl * beam * draught);
    lwl * (2.0 * draught + beam)
        * sqrt(cm)
        * (0.453 + 0.4425 * cb - 0.2862 * cm - 0.003467 * beam / draught + 0.3696 * cwp)
        + 2.38 * bulb_area_m2 / cb
}

/// ITTC-1957: C_F = 0.075 / (log10 Rn - 2)².
fn ittc_1957(speed_ms: f64, lwl: f64, nu: f64) -> f64 {
    0.075 / pow(log10(speed_ms * lwl / nu) - 2.0, 2.0)
}

/// The form factor 1 + k1.
fn form_factor(h: &HullInputs, cp: f64, lr: f64, variant: Variant) -> f64 {
    let (l, b, t) = (h.lwl, h.beam, h.draught);
    match variant {
        Variant::Hm1982 => {
            let tl = t / l;
            let c12 = if tl > 0.05 {
                pow(tl, 0.2228446)
            } else if tl > 0.02 {
                48.20 * pow(tl - 0.02, 2.078) + 0.479948
            } else {
                0.479948
            };
            let c13 = 1.0 + 0.003 * h.stern_shape;
            c13 * (0.93
                + c12
                    * pow(b / lr, 0.92497)
                    * pow(0.95 - cp, -0.521448)
                    * pow(1.0 - cp + 0.0225 * h.lcb_pct, 0.6906))
        }
        Variant::Hm1984 => {
            let c14 = 1.0 + 0.011 * h.stern_shape;
            0.93 + 0.487118
                * c14
                * pow(b / l, 1.06806)
                * pow(t / l, 0.46106)
                * pow(l / lr, 0.121563)
                * pow(l * l * l / h.volume_m3, 0.36486)
                * pow(1.0 - cp, -0.604247)
        }
    }
}

/// Resistance and its components at one speed.
pub fn components(h: &HullInputs, flow: Flow, variant: Variant) -> Components {
    let (l, b, t, vol) = (h.lwl, h.beam, h.draught, h.volume_m3);
    let Flow {
        speed_ms: v,
        rho,
        nu,
        g,
    } = flow;
    let q = 0.5 * rho * v * v; // dynamic pressure, N/m²
    let cb = h.cb();
    let cp = h.cp();
    let froude = v / sqrt(g * l);
    let lr = l * (1.0 - cp + 0.06 * cp * h.lcb_pct / (4.0 * cp - 1.0));

    let cf = ittc_1957(v, l, nu);
    let r_f = q * h.wetted_surface_m2 * cf;
    let k1 = form_factor(h, cp, lr, variant);
    let r_app = q * h.appendage_area_m2 * h.appendage_factor * cf;

    // Wave making and breaking.
    let bl = b / l;
    let c7 = if bl < 0.11 {
        0.229577 * pow(bl, 0.33333)
    } else if bl < 0.25 {
        bl
    } else {
        0.5 - 0.0625 / bl
    };
    let ie = 1.0
        + 89.0
            * exp(-pow(l / b, 0.80856)
                * pow(1.0 - h.cwp, 0.30484)
                * pow(1.0 - cp - 0.0225 * h.lcb_pct, 0.6367)
                * pow(lr / b, 0.34574)
                * pow(100.0 * vol / (l * l * l), 0.16302));
    let c1 = 2223105.0 * pow(c7, 3.78613) * pow(t / b, 1.07961) * pow(90.0 - ie, -1.37565);
    let abt = h.bulb_area_m2;
    let c3 = if abt > 0.0 {
        0.56 * pow(abt, 1.5) / (b * t * (0.31 * sqrt(abt) + h.draught_fwd - h.bulb_centre_m))
    } else {
        0.0
    };
    let c2 = exp(-1.89 * sqrt(c3));
    let c5 = 1.0 - 0.8 * h.transom_area_m2 / (b * t * h.cm);
    let lambda = if l / b < 12.0 {
        1.446 * cp - 0.03 * l / b
    } else {
        1.446 * cp - 0.36
    };
    let c16 = if cp < 0.8 {
        8.07981 * cp - 13.8673 * cp * cp + 6.984388 * cp * cp * cp
    } else {
        1.73014 - 0.7067 * cp
    };
    let m1 = 0.0140407 * l / t - 1.75254 * libm::cbrt(vol) / l - 4.79323 * b / l - c16;
    let l3v = l * l * l / vol;
    let c15_top = match variant {
        Variant::Hm1982 => 1727.0,
        Variant::Hm1984 => 1726.91,
    };
    let c15 = if l3v < 512.0 {
        -1.69385
    } else if l3v > c15_top {
        0.0
    } else {
        -1.69385 + (l / libm::cbrt(vol) - 8.0) / 2.36
    };
    let m_hump = match variant {
        Variant::Hm1982 => c15 * cp * cp * exp(-0.1 * pow(froude, -2.0)),
        Variant::Hm1984 => c15 * 0.4 * exp(-0.034 * pow(froude, -3.29)),
    };
    let r_w = c1
        * c2
        * c5
        * vol
        * rho
        * g
        * exp(m1 * pow(froude, -0.9) + m_hump * cos(lambda * pow(froude, -2.0)));

    // Bulb near the surface.
    let (pb, fni, r_b) = if abt > 0.0 {
        let pb = 0.56 * sqrt(abt) / (h.draught_fwd - 1.5 * h.bulb_centre_m);
        let fni = v / sqrt(g * (h.draught_fwd - h.bulb_centre_m - 0.25 * sqrt(abt)) + 0.15 * v * v);
        let r_b = 0.11 * exp(-3.0 * pow(pb, -2.0)) * pow(fni, 3.0) * pow(abt, 1.5) * rho * g
            / (1.0 + fni * fni);
        (pb, fni, r_b)
    } else {
        (0.0, 0.0, 0.0)
    };

    // Immersed transom.
    let (fnt, r_tr) = if h.transom_area_m2 > 0.0 {
        let fnt = v / sqrt(2.0 * g * h.transom_area_m2 / (b + b * h.cwp));
        let c6 = if fnt < 5.0 {
            0.2 * (1.0 - 0.2 * fnt)
        } else {
            0.0
        };
        (fnt, q * h.transom_area_m2 * c6)
    } else {
        (f64::INFINITY, 0.0)
    };

    // Model-ship correlation.
    let c4 = if h.draught_fwd / l <= 0.04 {
        h.draught_fwd / l
    } else {
        0.04
    };
    let ca = 0.006 * pow(l + 100.0, -0.16) - 0.00205
        + 0.003 * sqrt(l / 7.5) * pow(cb, 4.0) * c2 * (0.04 - c4);
    let r_a = q * h.wetted_surface_m2 * ca;

    let total = r_f * k1 + r_app + r_w + r_b + r_tr + r_a;
    Components {
        froude,
        cp,
        run_length_m: lr,
        form_factor: k1,
        cf,
        r_f_kn: r_f / 1000.0,
        r_app_kn: r_app / 1000.0,
        c7,
        entrance_angle_deg: ie,
        c1,
        c3,
        c2,
        c5,
        m1,
        c15,
        m_hump,
        lambda,
        r_w_kn: r_w / 1000.0,
        pb,
        fni,
        r_b_kn: r_b / 1000.0,
        fnt,
        r_tr_kn: r_tr / 1000.0,
        ca,
        r_a_kn: r_a / 1000.0,
        total_kn: total / 1000.0,
    }
}

/// The range of hull and speed parameters over which a method was fitted. A bound of `None` is not
/// declared.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Envelope {
    pub froude_max: Option<f64>,
    pub cp: (f64, f64),
    pub l_over_b: (f64, f64),
    pub b_over_t: (f64, f64),
}

/// Holtrop–Mennen for full hull forms (tankers and bulk carriers): Fn up to 0.24, C_P 0.73 to
/// 0.85, L/B 5.1 to 7.1, B/T 2.4 to 3.2. Ortigosa, López and García (2009), "Prediction of total
/// resistance coefficients using neural networks", Journal of Maritime Research VI(III), Table 1,
/// from the data of the 1982 paper.
pub const HOLTROP_FULL_FORM: Envelope = Envelope {
    froude_max: Some(0.24),
    cp: (0.73, 0.85),
    l_over_b: (5.1, 7.1),
    b_over_t: (2.4, 3.2),
};

/// Holtrop–Mennen for general cargo ships, from the same table: Fn up to 0.30, C_P 0.58 to 0.72,
/// L/B 5.3 to 8.0, B/T 2.4 to 4.0.
pub const HOLTROP_GENERAL_CARGO: Envelope = Envelope {
    froude_max: Some(0.30),
    cp: (0.58, 0.72),
    l_over_b: (5.3, 8.0),
    b_over_t: (2.4, 4.0),
};

/// The envelope that applies at a prismatic coefficient: the general cargo row below C_P 0.725
/// (the midpoint of the gap between the rows), the full-form row from there up.
pub fn holtrop_envelope(cp: f64) -> Envelope {
    if cp < 0.725 {
        HOLTROP_GENERAL_CARGO
    } else {
        HOLTROP_FULL_FORM
    }
}

/// One parameter outside an envelope: its name, value and range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutOfRange {
    pub name: &'static str,
    pub value: f64,
    pub low: f64,
    pub high: f64,
}

impl Envelope {
    /// The parameters outside the envelope, in the order Fn, C_P, L/B, B/T.
    pub fn outside(&self, froude: f64, cp: f64, l_over_b: f64, b_over_t: f64) -> Vec<OutOfRange> {
        let mut out = Vec::new();
        if let Some(top) = self.froude_max
            && froude > top
        {
            out.push(OutOfRange {
                name: "Fn",
                value: froude,
                low: 0.0,
                high: top,
            });
        }
        for (name, value, (low, high)) in [
            ("Cp", cp, self.cp),
            ("L/B", l_over_b, self.l_over_b),
            ("B/T", b_over_t, self.b_over_t),
        ] {
            if !(low..=high).contains(&value) {
                out.push(OutOfRange {
                    name,
                    value,
                    low,
                    high,
                });
            }
        }
        out
    }
}
