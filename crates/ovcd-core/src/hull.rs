//! Box barge hull geometry with bottom rakes at bow and stern. Port of `core/hull.py`.
//!
//! The raked box lives in [`hull_form`] in closed form; [`hull_form::generate`] writes the
//! Sections Table of the box and the raked box (issue #445), and later Hull Forms slot in beside it. [`HullGeometry`] is the Python name for it. Behaviour is copied exactly: every
//! `**` is `libm::pow`, Python's two-argument `min` keeps its first-of-equal rule, and every
//! expression keeps Python's evaluation order so the floating-point rounding is the same.

/// Hull Forms: the closed-form raked box, and the generators that write a Sections Table
/// (spec 0008, "Hull Forms"): [`hull_form::generate`].
pub mod hull_form {
    use crate::sections::{Section, SectionsError, SectionsTable};
    use alloc::vec;
    use alloc::vec::Vec;
    use core::fmt;
    use libm::{pow, sqrt};

    /// Box barge hull with bottom rakes (`HullGeometry` in Python). All lengths in metres.
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct RakedBox {
        /// Length overall (m)
        pub loa: f64,
        /// Moulded beam (m)
        pub beam: f64,
        /// Moulded depth (m)
        pub depth: f64,
        /// Horizontal length of bow rake (m)
        pub bow_rake_length: f64,
        /// Vertical rise of bow rake (m)
        pub bow_rake_rise: f64,
        /// Horizontal length of stern rake (m)
        pub stern_rake_length: f64,
        /// Vertical rise of stern rake (m)
        pub stern_rake_rise: f64,
    }

    /// Python `min(a, b)`: returns `b` only if `b < a`, so the first of two equal values wins.
    fn py_min(a: f64, b: f64) -> f64 {
        if b < a { b } else { a }
    }

    impl RakedBox {
        pub fn new(
            loa: f64,
            beam: f64,
            depth: f64,
            bow_rake_length: f64,
            bow_rake_rise: f64,
            stern_rake_length: f64,
            stern_rake_rise: f64,
        ) -> Self {
            Self {
                loa,
                beam,
                depth,
                bow_rake_length,
                bow_rake_rise,
                stern_rake_length,
                stern_rake_rise,
            }
        }

        /// Length of the parallel midbody (flat-bottom section).
        pub fn parallel_length(&self) -> f64 {
            self.loa - self.bow_rake_length - self.stern_rake_length
        }

        /// Displaced volume in one rake section at the given draft.
        fn rake_volume(&self, draft: f64, rake_length: f64, rake_rise: f64) -> f64 {
            if rake_length <= 0.0 {
                return 0.0;
            }
            if draft <= 0.0 {
                return 0.0;
            }
            if draft <= rake_rise {
                self.beam * pow(draft, 2.0) * rake_length / (2.0 * rake_rise)
            } else {
                self.beam * rake_length * (draft - rake_rise / 2.0)
            }
        }

        /// Length of the waterplane intersection in one rake section.
        fn rake_waterplane_length(&self, draft: f64, rake_length: f64, rake_rise: f64) -> f64 {
            if rake_length <= 0.0 {
                return 0.0;
            }
            if draft <= 0.0 {
                return 0.0;
            }
            if draft >= rake_rise {
                return rake_length;
            }
            draft * rake_length / rake_rise
        }

        /// First moment of rake displaced volume (for LCB).
        fn rake_first_moment_volume(
            &self,
            draft: f64,
            rake_length: f64,
            rake_rise: f64,
            x_start: f64,
            toward_bow: bool,
        ) -> f64 {
            if rake_length <= 0.0 || draft <= 0.0 {
                return 0.0;
            }
            let x_max = if rake_rise > 0.0 {
                py_min(rake_length, draft * rake_length / rake_rise)
            } else {
                rake_length
            };
            let a = draft;
            let b = if rake_length > 0.0 {
                rake_rise / rake_length
            } else {
                0.0
            };
            let s = if toward_bow { 1.0 } else { -1.0 };
            let xm = x_max;
            self.beam
                * (a * x_start * xm + a * s * pow(xm, 2.0) / 2.0
                    - b * x_start * pow(xm, 2.0) / 2.0
                    - b * s * pow(xm, 3.0) / 3.0)
        }

        /// Total displaced volume at the given draft (m^3).
        pub fn displaced_volume(&self, draft: f64) -> f64 {
            if draft <= 0.0 {
                return 0.0;
            }
            let v_mid = self.beam * py_min(draft, self.depth) * self.parallel_length();
            let v_bow = self.rake_volume(draft, self.bow_rake_length, self.bow_rake_rise);
            let v_stern = self.rake_volume(draft, self.stern_rake_length, self.stern_rake_rise);
            v_mid + v_bow + v_stern
        }

        /// Waterplane area at the given draft (m^2).
        pub fn waterplane_area(&self, draft: f64) -> f64 {
            if draft <= 0.0 {
                return 0.0;
            }
            let l_mid = self.parallel_length();
            let l_bow =
                self.rake_waterplane_length(draft, self.bow_rake_length, self.bow_rake_rise);
            let l_stern =
                self.rake_waterplane_length(draft, self.stern_rake_length, self.stern_rake_rise);
            self.beam * (l_mid + l_bow + l_stern)
        }

        /// Effective waterplane length at the given draft (m).
        pub fn waterplane_length(&self, draft: f64) -> f64 {
            let l_mid = self.parallel_length();
            let l_bow =
                self.rake_waterplane_length(draft, self.bow_rake_length, self.bow_rake_rise);
            let l_stern =
                self.rake_waterplane_length(draft, self.stern_rake_length, self.stern_rake_rise);
            l_mid + l_bow + l_stern
        }

        /// Second moment of waterplane area about the centreline (m^4).
        pub fn waterplane_inertia_transverse(&self, draft: f64) -> f64 {
            let l_wp = self.waterplane_length(draft);
            l_wp * pow(self.beam, 3.0) / 12.0
        }

        /// Second moment of waterplane area about the transverse centre (m^4).
        pub fn waterplane_inertia_longitudinal(&self, draft: f64) -> f64 {
            let l_wp = self.waterplane_length(draft);
            self.beam * pow(l_wp, 3.0) / 12.0
        }

        /// Vertical position of the centre of buoyancy above keel (m), by 100-strip integration.
        #[allow(non_snake_case)]
        pub fn KB(&self, draft: f64) -> f64 {
            if draft <= 0.0 {
                return 0.0;
            }
            let vol = self.displaced_volume(draft);
            if vol <= 0.0 {
                return 0.0;
            }
            let n: u32 = 100;
            let dz = draft / f64::from(n);
            let mut moment = 0.0;
            for i in 0..n {
                let z = (f64::from(i) + 0.5) * dz;
                let a_z = self.waterplane_area(z);
                moment += z * a_z * dz;
            }
            moment / vol
        }

        /// Longitudinal centre of buoyancy measured from the stern (m).
        #[allow(non_snake_case)]
        pub fn LCB(&self, draft: f64) -> f64 {
            if draft <= 0.0 {
                return self.loa / 2.0;
            }
            let vol = self.displaced_volume(draft);
            if vol <= 0.0 {
                return self.loa / 2.0;
            }
            let x_mid_start = self.stern_rake_length;
            let x_mid_center = x_mid_start + self.parallel_length() / 2.0;
            let v_mid = self.beam * py_min(draft, self.depth) * self.parallel_length();
            let moment_mid = v_mid * x_mid_center;
            // The stern rake's deep end is at its forward end, x = stern_rake_length, and it rises
            // aft toward x = 0 (issue #414: it was integrated from x = 0 forward, mirrored).
            let moment_stern = self.rake_first_moment_volume(
                draft,
                self.stern_rake_length,
                self.stern_rake_rise,
                self.stern_rake_length,
                false,
            );
            let x_bow_start = self.stern_rake_length + self.parallel_length();
            let moment_bow = self.rake_first_moment_volume(
                draft,
                self.bow_rake_length,
                self.bow_rake_rise,
                x_bow_start,
                true,
            );
            (moment_mid + moment_stern + moment_bow) / vol
        }

        /// Transverse metacentric radius BM = I_T / V (m).
        #[allow(non_snake_case)]
        pub fn BM_transverse(&self, draft: f64) -> f64 {
            let vol = self.displaced_volume(draft);
            if vol <= 0.0 {
                return 0.0;
            }
            self.waterplane_inertia_transverse(draft) / vol
        }

        /// Longitudinal metacentric radius BM_L = I_L / V (m).
        #[allow(non_snake_case)]
        pub fn BM_longitudinal(&self, draft: f64) -> f64 {
            let vol = self.displaced_volume(draft);
            if vol <= 0.0 {
                return 0.0;
            }
            self.waterplane_inertia_longitudinal(draft) / vol
        }

        /// Flat bottom plating area (m^2).
        pub fn bottom_area(&self) -> f64 {
            self.parallel_length() * self.beam
        }

        /// Total side plating area, both sides (m^2).
        pub fn side_area(&self) -> f64 {
            2.0 * self.loa * self.depth
        }

        /// Deck plating area (m^2).
        pub fn deck_area(&self) -> f64 {
            self.loa * self.beam
        }

        /// Bow rake plating area (m^2).
        pub fn bow_rake_area(&self) -> f64 {
            if self.bow_rake_length <= 0.0 {
                return 0.0;
            }
            let slope_length = sqrt(pow(self.bow_rake_length, 2.0) + pow(self.bow_rake_rise, 2.0));
            slope_length * self.beam
        }

        /// Stern rake plating area (m^2).
        pub fn stern_rake_area(&self) -> f64 {
            if self.stern_rake_length <= 0.0 {
                return 0.0;
            }
            let slope_length =
                sqrt(pow(self.stern_rake_length, 2.0) + pow(self.stern_rake_rise, 2.0));
            slope_length * self.beam
        }

        /// Total hull shell plating area (m^2).
        pub fn total_hull_surface_area(&self) -> f64 {
            self.bottom_area()
                + self.side_area()
                + self.deck_area()
                + self.bow_rake_area()
                + self.stern_rake_area()
        }
    }

    /// Which Hull Form, with its own parameters. All lengths in metres.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum HullFormKind {
        /// A rectangular box: stations at 0 and L.
        Box,
        /// A box with a straight bottom rake at each end: the keel rises from the base at the
        /// knuckle to the rise at the end.
        RakedBox {
            bow_rake_length_m: f64,
            bow_rake_rise_m: f64,
            stern_rake_length_m: f64,
            stern_rake_rise_m: f64,
        },
    }

    /// The main dimensions and the Hull Form.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct HullFormInputs {
        pub length_overall_m: f64,
        pub beam_m: f64,
        pub moulded_depth_m: f64,
        pub form: HullFormKind,
    }

    /// Why a Hull Form cannot write its sections.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum HullError {
        /// Length, beam or depth is not a positive finite number.
        Dimension,
        /// A rake length or rise is negative or not finite.
        NegativeRake,
        /// The two rakes together are longer than the hull.
        RakeLongerThanHull { rakes_m: f64, length_m: f64 },
        /// A rake rises above the moulded depth.
        RiseAboveDepth { rise_m: f64, depth_m: f64 },
        /// The sections the generator wrote were refused.
        Sections(SectionsError),
    }

    impl fmt::Display for HullError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match *self {
                HullError::Dimension => {
                    write!(f, "length, beam and moulded depth must be positive numbers")
                }
                HullError::NegativeRake => {
                    write!(f, "a rake length or rise must be 0 or more")
                }
                HullError::RakeLongerThanHull { rakes_m, length_m } => write!(
                    f,
                    "the bow and stern rakes ({rakes_m} m) are longer than the hull ({length_m} m)"
                ),
                HullError::RiseAboveDepth { rise_m, depth_m } => write!(
                    f,
                    "a rake rise of {rise_m} m is above the moulded depth of {depth_m} m"
                ),
                HullError::Sections(e) => write!(f, "{e}"),
            }
        }
    }

    fn positive(x: f64) -> bool {
        x.is_finite() && x > 0.0
    }

    fn non_negative(x: f64) -> bool {
        x.is_finite() && x >= 0.0
    }

    /// A rectangle half-section with its keel at `z_keel`.
    fn box_half(beam: f64, depth: f64, z_keel: f64) -> Vec<(f64, f64)> {
        vec![
            (0.0, z_keel),
            (beam / 2.0, z_keel),
            (beam / 2.0, depth),
            (0.0, depth),
        ]
    }

    /// Write the Sections Table of a Hull Form.
    pub fn generate(inputs: &HullFormInputs) -> Result<SectionsTable, HullError> {
        let (l, b, d) = (
            inputs.length_overall_m,
            inputs.beam_m,
            inputs.moulded_depth_m,
        );
        if !(positive(l) && positive(b) && positive(d)) {
            return Err(HullError::Dimension);
        }
        // Stations and keel heights, aft to forward.
        let keel: Vec<(f64, f64)> = match inputs.form {
            HullFormKind::Box => vec![(0.0, 0.0), (l, 0.0)],
            HullFormKind::RakedBox {
                bow_rake_length_m: bl,
                bow_rake_rise_m: br,
                stern_rake_length_m: sl,
                stern_rake_rise_m: sr,
            } => {
                if ![bl, br, sl, sr].iter().all(|&v| non_negative(v)) {
                    return Err(HullError::NegativeRake);
                }
                if bl + sl > l {
                    return Err(HullError::RakeLongerThanHull {
                        rakes_m: bl + sl,
                        length_m: l,
                    });
                }
                for rise in [br, sr] {
                    if rise > d {
                        return Err(HullError::RiseAboveDepth {
                            rise_m: rise,
                            depth_m: d,
                        });
                    }
                }
                let mut k = Vec::with_capacity(4);
                // A rake of no length is a vertical end: no knuckle station.
                k.push((0.0, if sl > 0.0 { sr } else { 0.0 }));
                if sl > 0.0 {
                    k.push((sl, 0.0));
                }
                if bl > 0.0 {
                    k.push((l - bl, 0.0));
                }
                k.push((l, if bl > 0.0 { br } else { 0.0 }));
                k
            }
        };
        let sections = keel
            .iter()
            .map(|&(x, z)| Section {
                x,
                points: box_half(b, d, z),
            })
            .collect();
        SectionsTable::new(sections, vec![d; keel.len()]).map_err(HullError::Sections)
    }

    impl RakedBox {
        /// The Sections Table of this raked box (its closed forms above are the generator's test).
        pub fn sections(&self) -> Result<SectionsTable, HullError> {
            generate(&HullFormInputs {
                length_overall_m: self.loa,
                beam_m: self.beam,
                moulded_depth_m: self.depth,
                form: HullFormKind::RakedBox {
                    bow_rake_length_m: self.bow_rake_length,
                    bow_rake_rise_m: self.bow_rake_rise,
                    stern_rake_length_m: self.stern_rake_length,
                    stern_rake_rise_m: self.stern_rake_rise,
                },
            })
        }
    }
}

/// The Python name for the raked box.
pub type HullGeometry = hull_form::RakedBox;
