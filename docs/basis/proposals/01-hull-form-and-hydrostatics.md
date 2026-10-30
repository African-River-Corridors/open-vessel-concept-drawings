# Proposals: hull form and hydrostatics

Not published. Companion to `../01-hull-form-and-hydrostatics.md`. Maintainer triage moves the errors to the
findings register (#121).

## Errors

Each error gives the code location and the source it disagrees with. Where the source is geometry, the check is a
worked example.

### E1. LCB of the stern rake is mirrored (legacy path)

- **Code:** `crates/ovd-core/src/hull.rs` `LCB`, lines 204-210, calls `rake_first_moment_volume` (lines 93-121) for
  the stern rake with `x_start = 0` and `toward_bow = true`. That function integrates a water depth `T − b·u` that is
  largest at `x_start`. At the stern the bottom is **highest** at `x = 0` and the water depth is largest at the
  forward end of the rake, `x = l_stern`.
- **Source:** geometry. Worked example, the test hull `HullGeometry::new(60, 10, 4, 6, 2, 4, 1.5)` of
  `crates/ovd-core/tests/hull.rs`, at `T = 3 m`: stern rake volume 90 m³ (both ways). Correct first moment 200 m⁴,
  centroid 2.22 m from the stern. Code: 160 m⁴, centroid 1.78 m. LCB is too far aft.
- **Effect:** none on the main path (rakes are zero there). Legacy `calculate_trim` gives too much bow-up trim for a
  hull with a stern rake. Small.

### E2. Block coefficient basis differs from ES-TRIN

- **Code:** `simulate.rs:741` and `:751`, `barge_capacity.rs:905`: `∇ = Cb × L × B × T` with `L` the length overall.
  The profile basis of `block_coefficient` does not say "on the length overall"; `calibration/factors.yaml` does
  (`block_coefficient_box_barge_on_loa`).
- **Source:** ES-TRIN 2025/1, Chapter 1, Article 1.01, 4.9 defines the block coefficient on the length of waterline,
  the breadth of waterline and the draught (4.18, 4.21, 4.23 define those).
- **Effect:** a user who types a `Cb` from a source on `Lwl` gets a draught too small by `Lwl / L`: for a 1.6 D bow
  rake (rise D) on an 80 m, 4 m deep hull at 3 m draught, `Lwl` is 78.4 m, so about 2 %. More for shorter hulls
  or a stern rake. Non-conservative. Small to medium.

### E3. Default Cb is outside its own basis, and the evidence does not back it

- **Code:** `builtin_profile.json` `block_coefficient`: default 0.85; basis "calibrated box barges span 0.883 to
  0.995; widened down for shaped gas hulls".
- **Source:** `calibration/factors.yaml` `block_coefficient_box_barge_on_loa`: 0.928 (n = 7). `calibration/results.md`
  reports the held-out deadweight with the fitted 0.928, not with 0.85.
- **Effect:** for a box or hopper barge, the draught is 9 % too deep and the weight-bound cargo too low. No source
  gives 0.85 for a gas hull. Large for capacity; conservative for draught.

### E4. Midship area uses Cb as the midship coefficient

- **Code:** `crates/ovd-core/src/resistance.rs:365-367` `midship_section_area = B × T × Cb`. Used in the Schijf
  limit (line 375) and the blockage term (line 411). `squat.rs:142` uses `B × T`.
- **Source:** the midship coefficient is a different coefficient from the block coefficient (ES-TRIN 4.9 defines
  `Cb` on volume). No source in the repo gives `Am = Cb B T`.
- **Effect:** for a box barge (`Cm` near 1) and `Cb` 0.85, the blockage is 15 % low and the Schijf limiting speed is
  too high. Non-conservative. The two modules disagree on one quantity. Medium. Cross-reference area 07 and 08.

### E5. Cb counts twice in the legacy stability and trim

- **Code:** `hydrostatics.rs:71` and `:172`: displacement = raked-box volume × `ρ` × `Cb`. The raked box already
  takes out the rake volume.
- **Source:** definition of `Cb` (ES-TRIN 4.9): the ratio of the displaced volume to `L B T`. Applying it to a volume
  that is already reduced counts the form twice.
- **Effect:** none on the main path (no rakes). Legacy path with rakes: displacement low, free surface correction
  per tonne high. Small.

### E6. The convoy beam check cannot fail

- **Code:** `vessel_standards.rs:92-94` sets `unit_beam_m = beam / n_abreast`; `validate_catalogue` (lines 203-208)
  then tests `|unit_beam × n_abreast − beam| > 0.05`, which is always false.
- **Source:** none needed: the check is built from the value it checks.
- **Effect:** none on results. A catalogue error in a convoy beam is never caught. Small.

### E7. Depth-rule basis text disagrees with the generated table

- **Code:** profile `depth_to_beam_ratio.basis`: dry bulk "0.22 to 0.44 (inland and pushed)", liquid "0.22 to 0.38".
- **Source:** `catalogue/depth_rule.md` (generated): dry bulk inland 0.22 to 0.49; liquid all 0.33 to 0.57, inland
  0.33 to 0.38. The liquid midpoint would be 0.355 on the inland range, not 0.30.
- **Effect:** none today (no liquid Cargo). Text only. Small.

## Gaps

- **G1. No sections table.** ADR 0012 decides it; nothing is built. Every hull is a box with one `Cb`. Damage
  stability, strength, LCB and trim all wait for it (#274).
- **G2. The drawn hull and the hydrostatic hull differ.** The drawing shows a bow rake of `1.6 D` with a rise of `D`
  (`ga_adapter.rs:204`, `models.rs:232`). That shape alone gives `Cb ≈ 1 − 1.6 D / L + 0.8 T / L` on the length
  overall: about 0.95 for `L` 80 m, `D` 4 m, `T` 3 m. The hydrostatics use 0.85. Nothing links them.
- **G3. BM and KB do not follow Cb.** `BM = B² / (12 T)` uses the box volume; the displacement uses the Cb volume.
  For a hull with a full waterplane BM should be `B² / (12 Cb T)`. Direction conservative for GM, but the two
  quantities rest on two different hulls.
- **G4. No form coefficients other than Cb.** No `Cm`, `Cp` or `Cwp`. An expert expects at least `Cm` (for
  blockage) and `Cwp` (for BM and TPC).
- **G5. No hydrostatic table.** TPC, MCT, KM, LCB and LCF against draught are standard at concept stage.
- **G6. No trim on the main path.** The hull floats level; freeboard and air draft ignore trim.
- **G7. One length.** No `Lwl` or `Lpp`. Squat and resistance take the length overall.
- **G8. Class maxima as design dimensions.** Some schemes let LOA run under the maximum (the GB 38030 rows say up to
  10 % under). Picking a class always builds at the maximum.
- **G9. Pushed convoys.** No hydrostatics per barge or per convoy yet (`run.rs:91`).
- **G10. Sources not read.** Barrass II, ICORELS and the `Cb ≥ 0.3` validity floor have no source text in
  `~/Dev/ovd-rule-sources/`. They need a source before their labels can rise.
- **G11. Glossary.** Missing terms: Block Coefficient (and its length basis), Draught, Length Overall, Length of
  Waterline, Freeboard. `GLOSSARY.md` defines Moulded Depth twice.

## Changes

| # | Change | Effect on results | Order |
|---|---|---|---|
| C1 | Say "on the length overall" in the `block_coefficient` basis and the input label. Warn when a typed `Cb` comes with a source on `Lwl` (E2). | None | 1 |
| C2 | Regenerate the D/B basis ranges from `depth_rule.md` (E7). | None | 2 |
| C3 | Check the convoy beam against an independent published unit beam, or drop the check (E6). | None | 3 |
| C4 | Fix the stern-rake moment (E1): integrate with the depth `T − h_r + b·u` from the stern. Add an independent test (prism and wedge centroids). | None on the main path; small on the legacy path | 4 |
| C5 | Stop applying `Cb` to the raked volume in the legacy stability and trim (E5), or document it as a calibration factor. | Small, legacy path | 5 |
| C6 | Set the default `Cb` by Cargo Containment: box and hopper hold 0.93 (fitted, `factors.yaml`); Type C tanks an Estimate with its basis stated. Needs a maintainer decision (E3). | Large: draught about 9 % shallower for box barges, more weight-bound cargo | 6 |
| C7 | Add a midship coefficient input (default an Estimate near 1 for barges). Use one midship area in squat and resistance (E4). | Medium: lower Schijf speed limits, more resistance in confined water | 7 |
| C8 | Make BM and KB consistent with the displacement until the sections table exists: divide the waterplane inertia by the Cb volume, `BM = Cwp_I × L B³ / (12 Cb L B T)`, with a waterplane inertia factor `Cwp_I` (1 for a box) as a new input (G3, G4). | Small to medium on GM | 8 |
| C9 | Build the ADR 0012 sections table with the raked box as the first generator. Derive `Cb`, KB, BM, LCB and LCF by integration; `Cb` becomes a result, not an input (G1, G2, G6). | Large | 9 |
| C10 | Output a hydrostatic table against draught (G5). | None on existing results; new outputs | 10 |
