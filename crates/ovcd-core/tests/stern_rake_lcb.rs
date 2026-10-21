//! Seam: the LCB of the raked box's stern rake (issue #414): a prism and a wedge by hand.
//!
//! A box 100 m long, 10 m wide, with a stern rake 10 m long rising 4 m, at a draught of 2 m. The
//! rake bottom rises from the base at x = 10 m to 4 m at x = 0, so at 2 m draught the water reaches
//! x = 10 - 10 x 2 / 4 = 5 m. Prism (parallel body): 90 x 10 x 2 = 1,800 m3 at x = 55 m. Stern
//! wedge: from x = 5 to 10 m, a triangle of section 2 m deep at x = 10 m to 0 at x = 5 m, so
//! 0.5 x 5 x 2 x 10 = 50 m3 at its centroid x = 10 - 5 / 3 = 8.333 m. LCB = (1,800 x 55 + 50 x
//! 8.333) / 1,850 = 53.739 m.
//!
//! Label: property. The hand case is worked above.

use ovcd_core::hull::HullGeometry;

#[test]
fn the_stern_rake_lcb_is_the_prism_and_wedge_hand_case() {
    let hull = HullGeometry::new(100.0, 10.0, 6.0, 0.0, 0.0, 10.0, 4.0);
    let v = hull.displaced_volume(2.0);
    assert!((v - 1850.0).abs() < 1e-9, "{v}");
    let wedge_x = 10.0 - 5.0 / 3.0;
    let hand = (1800.0 * 55.0 + 50.0 * wedge_x) / 1850.0;
    assert!(
        (hull.LCB(2.0) - hand).abs() < 1e-9,
        "{} {hand}",
        hull.LCB(2.0)
    );
    // The same hull turned about: a bow rake of the same shape gives the mirror LCB.
    let bow = HullGeometry::new(100.0, 10.0, 6.0, 10.0, 4.0, 0.0, 0.0);
    assert!(
        (bow.LCB(2.0) - (100.0 - hand)).abs() < 1e-9,
        "{}",
        bow.LCB(2.0)
    );
}
