//! Box barge hull geometry with bottom rakes at bow and stern. Port of `core/hull.py`.
//!
//! The raked box is the only Hull Form, and lives in [`hull_form`] so later Hull Forms slot in
//! beside it. [`HullGeometry`] is the Python name for it. Behaviour is copied exactly: every
//! `**` is `libm::pow`, Python's two-argument `min` keeps its first-of-equal rule, and every
//! expression keeps Python's evaluation order so the floating-point rounding is the same.

/// Hull Forms. Only the raked box exists today.
pub mod hull_form {
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
            let moment_stern = self.rake_first_moment_volume(
                draft,
                self.stern_rake_length,
                self.stern_rake_rise,
                0.0,
                true,
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
}

/// The Python name for the raked box.
pub type HullGeometry = hull_form::RakedBox;
