# Type C tanks

> Concept and preliminary design only. Not for approval. Draft: needs expert review.

**For:** gas containment engineer  ·  **Code:** `crates/ovcd-core/src/tanks.rs`, `pressure_vessel.rs`,
`barge_capacity.rs` (`TankDesign`, `gas_volumetric`, `dome_volume_m3`), `weights.rs` (tank lines),
`gas_arrangement.rs` (L/D check, OD cap), `cargo.rs` (cargo density); defaults in
`crates/ovcd-format/src/builtin_profile.json`; parameter mapping in `crates/ovcd-format/src/simulate.rs`
·  **Rule Sets:** none loaded yet. The IGC values in this area are constants in code with citations. Spec 0004 plans
them as Rules of the set `igc-2g` (4.23.2.1.2 minimum thickness, 4.23.2.1.3 weld factor, 4.23.3.1 allowable stress).

> **Changes made, 2026-10-20 and 21.** These fixes are on `main`, and the rows below show the new values:
> #313 S690QL1 at 256.7 MPa (TC-A24, TC-A37); #314 2:1 head area (TC-A08, TC-A47); #315 LPG density by carriage
> condition and the IGC 15.5.1 loading limit (TC-A48, TC-A52 to TC-A54); #316 dome rule (TC-A55, TC-A56); #317
> part 1 tank weight records (TC-A62, TC-A63).
>
> **Still open:** #317 Rules for the IGC 4.23.1.2 minimum design pressure and steel by design temperature (rows marked
> "Rule pending"); #374 the wall must include the IGC liquid pressure; #375 the head type as an input (a Design
> still gets hemispherical heads, so #314 moves no Design result yet). Code line numbers on changed rows are given
> by file only; recheck them at review.
>
> Calibration after the fixes: the river LPG tank wall is 15.5 mm against 13.1 mm printed (+18.3 %, outside the
> ±15 % band). This is an accepted miss: the paper uses the inland basis Rm/2.7. Its steel at the published thickness
> is 37.4 t against 34.4 t published.

OVCD is a concept tool (ADR 0011). This page says how OVCD sizes and weighs a Type C tank today. It is not a tank
design and it gives no verdict that an Administration or class can use.

## What OVCD computes here

For an Option with Cargo Containment `type_c_tanks` (a gas Cargo), each `simulate` result gives:

- `tank_outer_diameter_m`: the Tank Outer Diameter, to the outside of the insulation.
- `n_tanks_wide`, `n_tanks_total`, and the tank length (overall and cylinder) in the gas basis.
- `tank_inner_diameter_m` and `containment_inner_volume_m3`: the gross tank volume of all tanks, domes excluded.
- `cargo_volume_m3`: the gross volume times the fill ratio. The cargo mass is this volume times the cargo density.
- `tank_wall_thickness_mm`: the cylinder wall, and in `drawing.tank` the head wall.
- `drawing.tank` (not a result, not hashed): the LPG Carriage Condition, design pressure, design temperature, steel,
  allowable stress, insulation type and thickness, structure factor, where the wall came from (`formula` or
  `override`), the steel outer diameter and a statement of the diameter datum (`simulate.rs`, `tank_report`).
- Weights: tank shells, tank internals and supports, insulation (`WeightBreakdown`, `weights.rs`). They feed the
  lightship, the cost lines `type_c_tanks`, `tank_supports` and `insulation`, and the stability KG.
- Check `tank_length_over_diameter`: overall tank length over outer diameter, at most 7.
- The tank top height and the dome height, which set the air draft.

## How it works today

The order is the order of `barge_capacity.rs::gas_volumetric`. It loops up to 40 times, because the side clearance
depends on the tank volume through d(Vc) (IGC 2.4.1.1; see page 05).

1. **Defaults by Cargo.** A Design takes the tank values from the Defaults Profile tables keyed by Cargo Containment,
   Cargo and LPG Carriage Condition (`builtin_profile.json`, entries `tank_design_pressure_mpa` to
   `tank_structure_factor`). `simulate.rs::tank_design_of` and `tank_wall_of` turn them into a `TankDesign` and the
   wall override. A params key that is absent falls back to the engine's old values (TC-A69). Label: per row below.

2. **Tank outer diameter** (`tanks.rs` lines 99–102). Method.
   OD = (B − 2·c_s − (n_w − 1)·g_t) / n_w, with B the beam, c_s the side clearance, n_w tanks across and g_t the
   transverse gap (1.0 m). OD is to the outside of the insulation. The tank count across is the fewest that keeps
   OD at or below the cap (`barge_capacity.rs::tanks_wide_for_beam`).

3. **Steel diameter** (`tanks.rs` lines 125–131, 145). Method. D_s = OD − 2·t_ins. For a gas Design the wall formula
   takes D_s (`barge_capacity.rs` line 496, issue #292). The Python API still gives the formula the insulated OD
   (`wall_formula_on_steel_diameter` false).

4. **Wall thickness** (`pressure_vessel.rs::calculate_wall_thicknesses_with`, lines 244–286). Method, from the ASME
   VIII-1 shell formula form. OVCD does not hold ASME VIII-1, so the formulas are unverified (spec 0004, Appendix A).
   With P the design pressure (MPa), f the allowable stress (MPa), E the weld efficiency and R_o = D_s/2 (mm):
   - first pass: t₁ = P·R_o / (f·E − 0.6·P);
   - second pass: R_i = R_o − t₁, t_cyl = P·R_i / (f·E − 0.6·P);
   - hemispherical head: t_head = P·R_i / (2·f·E − 0.2·P); 2:1 head: t_head = P·(2·R_i) / (2·f·E − 0.2·P);
   - add the corrosion allowance, round up to 0.5 mm, then take at least the IGC minimum thickness.

   P is the design pressure only. The engine adds no liquid head. A typed wall thickness replaces both walls.

5. **Inner geometry** (`tanks.rs` lines 143–149). Method. R_so = D_s/2; R_i = R_so − t_cyl; head inner radius
   R_hi = R_so − t_head.

6. **Heads and cylinder length** (`tanks.rs` lines 164–192). Method.
   - Hemisphere: head length R_od = OD/2; inner volume (2/3)·π·R_hi³; steel area 2·π·R_so².
   - 2:1 ellipsoid (API only): head length OD/4; inner volume (1/3)·π·R_hi³; steel area 1.084·π·R_so².
   - Tank length L_t = (LOA − bow void − stern void − (n_l − 1)·g_l) / n_l, with g_l = 2.0 m. Cylinder length
     L_c = L_t − 2 × head length. A cylinder of zero or less is an error.

7. **Volume and cargo** (`tanks.rs` lines 205–211; `barge_capacity.rs` line 886). Method.
   V_tank = π·R_i²·L_c + 2·V_head. Cargo volume = n·V_tank × fill ratio. Cargo mass = cargo volume × density
   (`cargo.rs`, one density per Cargo). The dome is not in V_tank or the cargo volume.

8. **Dome** (`barge_capacity.rs` lines 467–469, 523, 561, 573). Method. A cylinder of diameter d_d and height h_d:
   V_dome = (π/4)·d_d²·h_d. It counts in Vc of each tank for d(Vc) only (IGC 2.4.1.1). The dome height adds to the
   tank top: h_top = double bottom + support height + OD + h_d. The dome has no weight in the model.

9. **Shell steel weight** (`tanks.rs` lines 213–222). Method.
   W_shell = n × [2·π·L_c·(R_so + R_i)/2·t_cyl + 2·A_head·t_head] × ρ_steel, with A_head the steel head area of step 6
   (outer steel surface). ρ_steel is the material's density (`barge_capacity.rs` line 501).

10. **Insulation weight** (`tanks.rs` lines 224–234). Method. W_ins = n × (2·π·R_od·L_c + 2·A_head,od) × t_ins ×
    ρ_ins, with the areas at the insulated outer face and ρ_ins = 60 kg/m³ for every insulation type.

11. **Tank total** (`weights.rs` lines 74–79). Method. W_tank = W_shell × structure factor. The part above the shell,
    W_shell × (factor − 1), is "tank internals and supports" and becomes the cost line `tank_supports`
    (`crates/ovcd-format/src/cost.rs` lines 79–81). Piping and outfitting are fractions of (hull steel + W_tank)
    (`weights.rs` lines 170–173; see page 02). In the stability KG, W_tank + W_ins sits at the tank centre
    (`gas_stability.rs` lines 709–716).

12. **Length over diameter** (`gas_arrangement.rs` lines 47, 257–274). Fixed in code. L_t / OD ≤ 7, both to the
    insulated face. A candidate over 7 fails `tank_length_over_diameter` and cannot win the arrangement search.

13. **Design temperature and insulation type.** Recorded only. They go to `drawing.tank` and change no number
    (`simulate.rs::tank_report`).

## Assumptions, rules and estimates

Profile line numbers are in `crates/ovcd-format/src/builtin_profile.json`. "API only" means the Python and `eval`
API, not a Design.

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| TC-A01 | Diameter datum | Tank Outer Diameter is to the outside of the insulation; clearances, tank top and drawing use it | Method | Issue #292; `GLOSSARY.md` "Tank Outer Diameter" | `tanks.rs` 99–102; `simulate.rs::tank_report` | Unknown |
| TC-A02 | Steel outer diameter | D_s = OD − 2·t_ins; wall formula and volume start from it | Method | Issue #292 | `tanks.rs` 125–131, 145; `barge_capacity.rs` 496 | Unknown |
| TC-A03 | Transverse gap between tanks | 1.0 m | Fixed in code | Original engine; no source | `models.rs` 189; `tanks.rs` 101 | Unknown |
| TC-A04 | Longitudinal gap between tanks | 2.0 m | Fixed in code | Original engine; no source | `models.rs` 190; `tanks.rs` 188–189 | Unknown |
| TC-A05 | Tank shape | One horizontal cylinder with two heads; no bilobe, no vertical tank | Fixed in code | Scope of the port | `tanks.rs` | Unknown; bilobe tanks fill a beam better |
| TC-A06 | Head type in a Design | Hemispherical always (no input) | Fixed in code | `BargeInputs` default; `gas_volumetric` does not set it | `models.rs` 194 | Unknown. Hemispherical heads are thin (half the shell) and long |
| TC-A07 | Hemispherical head | Length R_od; volume (2/3)πR_hi³; steel area 2πR_so² | Method | Sphere geometry | `tanks.rs` 166–169 | None known |
| TC-A08 | 2:1 ellipsoidal head (API only) | Length OD/4; volume (1/3)πR_hi³; steel area 1.380173·πR² = 4.336·R² at the steel outer surface | Method | Half an oblate spheroid (MathWorld "Oblate Spheroid", eq. 11). Fixed in #314: the old constant 1.084 was meant for D², not πR², so it was 21.5 % low (the true area is 27.3 % above the old one) | `tanks.rs` | Matches the cited formula. A Design still gets hemispherical heads (TC-A06, #375) |
| TC-A09 | Cylinder length | L_c = L_t − 2 × head length | Method | Geometry | `tanks.rs` 192 | None known for hemispheres |
| TC-A10 | Gross tank volume | πR_i²L_c + 2V_head; dome excluded | Method | Geometry | `tanks.rs` 205–209 | None known. Calibration: −0.1 % and +0.2 % |
| TC-A11 | Shell thickness formula | t = P·R/(f·E − 0.6·P), two passes from the steel outer radius | Method | ASME VIII-1 UG-27 form (standard not held; unverified, spec 0004 App. A). IGC 4.23.2.4 asks for "accepted pressure vessel design theory" | `pressure_vessel.rs` 155–168, 259–263 | Unknown until checked against the standard |
| TC-A12 | Hemispherical head thickness | t = P·R/(2·f·E − 0.2·P) | Method | Same as TC-A11. Code comment says UG-27 | `pressure_vessel.rs` 171–184, 268 | Unknown |
| TC-A13 | 2:1 head thickness (API only) | t = P·D/(2·f·E − 0.2·P) | Method | Same as TC-A11 | `pressure_vessel.rs` 187–200, 269 | Unknown |
| TC-A14 | Pressure in the wall formula | Design pressure (MARVS) only; no liquid head | Fixed in code | Profile basis: "The engine adds no liquid head" (line 162). IGC 4.23.2.2 asks for the design liquid pressure of 4.13.2 | `tanks.rs` 133 | Non-conservative: walls thin, most on low-pressure tanks |
| TC-A15 | Weld efficiency | 0.95 | Fixed in code | Matches IGC 4.23.2.1.3 (0.95 with the NDT of 6.5.6.5) | `models.rs` 200 | Conservative if a yard reaches 1.0 |
| TC-A16 | Corrosion allowance | 0.0 mm (304L, 316L); 0.5 mm (9Ni, S690QL1) | Fixed in code | Original engine; S690QL1 copies 9Ni. No source | `pressure_vessel.rs` 53, 64, 75, 91, 274–275 | Unknown |
| TC-A17 | Plate rounding | Up to the next 0.5 mm | Fixed in code | "standard plate sizing" (code comment); no source | `pressure_vessel.rs` 278–279 | Conservative (up to 0.5 mm) |
| TC-A18 | Minimum thickness after forming | 3 mm (304L, 316L); 5 mm (9Ni, S690QL1) | Fixed in code | Matches IGC 4.23.2.1.2 (5 mm C-Mn and nickel steels, 3 mm austenitic) | `pressure_vessel.rs` 52, 63, 74, 90, 282–283 | None known |
| TC-A19 | Thickness used for weight | The required thickness after rounding; no forming or mill tolerance added | Fixed in code | IGC 4.23.2.1.1: the calculated thickness is a minimum after forming, without negative tolerance | `tanks.rs` 139–140, 216–219 | Non-conservative (small) on weight |
| TC-A20 | Fixed wall override | Typed value replaces shell and head wall; 25 mm when the params key is absent | Fixed in code | Original engine; a saved file that stores 25 mm keeps it | `simulate.rs` 530–538; `barge_capacity.rs` 60 | Either way |
| TC-A21 | Allowable stress, 304L | 113 MPa | Estimate | Matches IGC 4.23.3.1 with standard minimums: min(485/3.5, 170/1.5) = 113.3 (derived). The code holds no Rm or Re; profile says "no source cited" | `pressure_vessel.rs` 50 | Close to the limit; non-conservative by 0.3 % at most |
| TC-A22 | Allowable stress, 316L | 113 MPa | Estimate | As TC-A21 | `pressure_vessel.rs` 61 | As TC-A21 |
| TC-A23 | Allowable stress, 9Ni | 230 MPa | Estimate | Matches IGC 4.23.3.1 with ASTM A553 Type I minimums: min(690/3, 585/1.5) = 230 (derived) | `pressure_vessel.rs` 72 | None known at those strengths |
| TC-A24 | Allowable stress, S690QL1 | 256.7 MPa = min(770/3, 690/1.5) | Sourced default | IGC 4.23.3.1; Rm and Re from the DESIGN 2006 river tank paper and an EN 10025-6 S690QL data sheet (#313). Input `tank_allowable_stress_basis`: `igc_4_23_3_1` (default) or `published_design` (285 MPa, kept for old files). A stress above the IGC limit gives a warning | `pressure_vessel.rs`; profile | Matches the IGC limit |
| TC-A25 | Steel density | 7.93 (304L), 7.98 (316L), 7.85 (9Ni, S690QL1) t/m³ | Fixed in code | Original engine; S690QL1 copies 9Ni (code comment says estimate) | `pressure_vessel.rs` 51, 62, 73, 89 | Small |
| TC-A26 | Design pressure, LNG | 0.4 MPa **(Rule pending, #317)** | Sourced default | Owners' sheets of two LNG bunker vessels (4 bar); a tank maker's 2018 presentation (3 to 5 barg) | Profile 127; wall formula | Unknown |
| TC-A27 | Design pressure, LNG and ethane | 0.4 MPa **(Rule pending, #317)** | Estimate | Taken as for LNG | Profile 133 | Unknown |
| TC-A28 | Design pressure, LPG fully pressurised | 1.77 MPa **(Rule pending, #317)** | Sourced default | Owner's Form C particulars of 16 pressurised LPG carriers: 17.65 to 17.7 barg | Profile 138–143 | Unknown |
| TC-A29 | Design pressure, LPG semi-pressurised | 1.0 MPa **(Rule pending, #317)** | Estimate | One trade-press report: 9.97 bar at −48 °C, condition not stated | Profile 145–150 | Unknown |
| TC-A30 | Design pressure, LPG fully refrigerated | 0.4 MPa **(Rule pending, #317)** | Estimate | Taken as for LNG | Profile 152–156 | May be below the IGC 4.23.1.2 minimum (see Questions) |
| TC-A31 | Design temperature, LNG and LNG/ethane | −163 °C **(Rule pending, #317)** | Sourced default | Owner's Form B particulars of a 2022 LNG bunker vessel | Profile 168–174; recorded only | No effect today |
| TC-A32 | Design temperature, LPG fully pressurised | 0 °C **(Rule pending, #317)** | Sourced default | Owner's Form C sheet of a 2007 pressurised LPG carrier (0 to 45 °C) | Profile 176–181; recorded only | No effect today |
| TC-A33 | Design temperature, LPG semi-pressurised | −48 °C **(Rule pending, #317)** | Estimate | One trade-press report | Profile 183–188; recorded only | No effect today |
| TC-A34 | Design temperature, LPG fully refrigerated | −48 °C **(Rule pending, #317)** | Estimate | Propane boils at −42.1 °C; margin | Profile 190–195; recorded only | No effect today |
| TC-A35 | Steel, LNG and LNG/ethane | 9Ni **(Rule pending, #317)** | Sourced default | A tank maker lists 9 % Ni or 304L for LNG | Profile 207–213 | Unknown; 304L is the other common choice |
| TC-A36 | Steel, LPG semi-pressurised | 9Ni **(Rule pending, #317)** | Estimate | No source | Profile 222–226 | Likely heavy and dear against built practice |
| TC-A37 | Steel, LPG fully pressurised | S690QL1 at 256.7 MPa **(steel-by-temperature Rule pending, #317)** | Sourced default | River LPG tank paper, DESIGN 2006; allowable stress per TC-A24 | Profile | See TC-A24 |
| TC-A38 | Steel, LPG fully refrigerated | 9Ni **(Rule pending, #317)** | Estimate | No source; profile says a low-temperature C-Mn steel would be lighter | Profile 228–232 | Likely heavy |
| TC-A39 | Allowable stress override | none (the material's) **(Rule pending, #317)** | Sourced default | Profile 237–243; spec 0004 plans an upper bound Rule (IGC 4.23.3.1) | `barge_capacity.rs` `TankDesign`; `pressure_vessel.rs` 254 | A typed value above the IGC limit is not refused today |
| TC-A40 | Insulation thickness, LNG and LNG/ethane | 330 mm | Estimate | One secondary report (330 mm panel); maker data not found | Profile 276–281 | Unknown. Public range 300 to 350 mm |
| TC-A41 | Insulation thickness, LPG fully pressurised | 0 mm | Sourced default | Owner's Form C sheet and GA of a 2007 pressurised LPG carrier: tanks not insulated | Profile 283–288 | None known |
| TC-A42 | Insulation thickness, LPG semi-pressurised and fully refrigerated | 200 mm | Estimate | One cold Type C design study (LCO2 at −35 °C, 200 mm PUF) | Profile 290–295 | Possibly thin for −48 °C |
| TC-A43 | Insulation type, LNG | panel | Estimate | One secondary report | Profile 248–253; recorded only | No effect today |
| TC-A44 | Insulation type, LPG fully pressurised | none | Sourced default | As TC-A41 | Profile 255–260; recorded only | No effect today |
| TC-A45 | Insulation type, LPG semi and refrigerated | pu_foam | Estimate | As TC-A42 | Profile 262–267; recorded only | No effect today |
| TC-A46 | Insulation density | 60 kg/m³, every type | Fixed in code | Original engine; no source | `models.rs` 203; `tanks.rs` 233 | Unknown. Public figures: perlite 40, lab PUF 96 kg/m³ |
| TC-A47 | Insulation volume | Insulated outer surface × thickness; 2:1 head at 4.336·R_od² | Method | Thin-layer approximation; head area per TC-A08 (#314) | `tanks.rs` | Conservative (a few %) for thick layers on small tanks |
| TC-A48 | Fill ratio | `fill_ratio` is the filling limit FL, 0.98. An LPG Cargo is loaded to LL = FL × ρR / ρL (IGC 15.5.1), never above FL. Reference temperature (IGC 15.1.3): fully pressurised = saturation temperature at the MARVS; semi-pressurised and fully refrigerated = the loading temperature, so LL = FL | Sourced default | IGC 15.3 (FL 98 %), 15.1.3 and 15.5.1 (#315). Inputs `lpg_loading_limit_basis` (`igc_15_1_3` default; `igc_15_5_2` gives a 45 °C reference when pressurised; `refrigerated_density` keeps old files) and `lpg_loading_temperature_c` | Profile; `simulate.rs`; `tanks.rs` | Matches IGC 15.5.1. Fully pressurised propane LL is about 85 % |
| TC-A49 | Cargo volume basis | Fill × gross volume without the dome | Method | IGC 15.1.1 says "relative to the total tank volume" | `tanks.rs` 211 | Conservative (small) if the dome counts in the total |
| TC-A50 | Cargo density, LNG | 450 kg/m³ | Fixed in code | Original engine (comments in the Python file) | `cargo.rs` 32; `barge_capacity.rs` 828, 886 | Unknown |
| TC-A51 | Cargo density, LNG/ethane | 500 kg/m³ | Fixed in code | Original engine | `cargo.rs` 43 | Unknown |
| TC-A52 | Cargo density, propane | Saturated liquid density at the loading temperature: 15 °C fully pressurised (estimate), −10 °C semi-pressurised (estimate), −42.1 °C fully refrigerated | Sourced default | NIST WebBook saturation tables (#315). Cargo per m³ of tank, fully pressurised: 569 → 432 kg | `cargo.rs`; profile | Matches the source at the stated temperatures |
| TC-A53 | Cargo density, butane | As TC-A52; fully refrigerated at −0.5 °C | Sourced default | NIST WebBook, n-butane (#315). Fully pressurised: 588 → 437 kg per m³ of tank | `cargo.rs`; profile | As TC-A52 |
| TC-A54 | Cargo density, LPG mix | As TC-A52; fully refrigerated at −34.3 °C (estimate) | Estimate | Ideal solution of the propane and butane values (#315). Fully pressurised: 577 → 440 kg per m³ of tank | `cargo.rs`; profile | Unknown; ideal-solution estimate |
| TC-A55 | Dome height above tank top | = dome diameter | Estimate | Dome rule (#316), two published points: a Ø2,500 mm dome 2,500 mm above the inside of the shell on an R7,300 lobe (Heo, Kang and Park 2017, Fig. 19); a Ø1,200 mm valve dome on a 4.73 m river LPG tank (DESIGN 2006). Old files keep 2.0 m | Profile; `barge_capacity.rs` | Unknown; two data points. A 110 × 11.4 m example moves from 2.0 to 1.64 m (air draft −0.36 m) |
| TC-A56 | Dome diameter | max(1.2 m, 0.17 × steel OD) | Estimate | Dome rule (#316), same two points. Both drawing views now draw the dome at this diameter | Profile; `barge_capacity.rs`; `ga.rs` | Small effect, through Vc |
| TC-A57 | Dome volume | (π/4)·d_d²·h_d, in Vc only; not in cargo, fill or weight | Method | IGC 2.4.1.1 (Vc includes domes) | `barge_capacity.rs` 467–469, 561 | Dome steel and fittings missing from weight: non-conservative (small) |
| TC-A58 | Tank top | db + support height + OD + dome height | Method | Geometry | `barge_capacity.rs` 573; `tanks.rs` 237 | None known |
| TC-A59 | Length over diameter | ≤ 7, insulated length over insulated OD | Fixed in code | OVCD Check. Origin ADN 2025 9.3.1.11.1(b), which states no datum; no IGC paragraph | `gas_arrangement.rs` 47, 257–274 | Unknown |
| TC-A60 | Outer diameter cap | 16 m | Estimate | Tool limit, not an IGC rule (profile 113–118) | `barge_capacity.rs` 46; `gas_arrangement.rs` 459–462 | Unknown |
| TC-A61 | Shell steel volume | Cylinder at mean surface × t; heads at outer steel surface × t | Method | Thin-shell approximation | `tanks.rs` 213–222 | Heads slightly conservative |
| TC-A62 | Structure factor, LPG fully pressurised | 1.12 | Estimate | One published uninsulated tank: 34.4 t steel over 30.6 t shell plate. Two tank weight records added, held out and not yet runnable: 0.115 t/m³ (155 t, 1,350 m³) and 0.096 t/m³ (240 t, 2,500 m³ bilobe) (#317 part 1) | Profile; `calibration/` | Unknown; one tank, in-sample |
| TC-A63 | Structure factor, all other rows | 1.30 | Estimate | Original engine. Not tuned yet: the tank weight records above come first (#317) | Profile | Non-conservative: model tanks weigh about half of published ones (see Evidence) |
| TC-A64 | Internals and supports | W_shell × (factor − 1); rings, saddles, chocks, dome and fittings not modelled apart | Method | Single-factor model | `weights.rs` 77; `cost.rs` 80 | Non-conservative (see TC-A63) |
| TC-A65 | Tank weight centre | Tank steel and insulation at the tank centre | Method | Symmetric cylinder | `gas_stability.rs` 709–716 | Small; the dome would raise it |
| TC-A66 | Saddle height | 0.5 m above the inner bottom | Estimate | Original engine | Profile 411–416; `tanks.rs` 237 | Unknown |
| TC-A67 | Tanks in line | 2 | Estimate | Original engine; sets tank length and so L/D | Profile 404–410 | Unknown |
| TC-A68 | Collision, flooding and heel loads on supports | Not computed | Fixed in code | IGC 4.15.1 (0.5 g fwd, 0.25 g aft), 4.15.2 (flooding) | none | Unknown; no saddle or chock quantities |
| TC-A69 | Engine fallback when a params key is absent | 0.5 MPa, 9Ni, 350 mm, structure factor 1.30, wall 25 mm | Fixed in code | Values before #292; used by the API and by files that leave a key out | `barge_capacity.rs` 60–66, 92–101; `simulate.rs` 530–575 | Either way |

## Inputs the user can change

All are Option inputs with defaults from the Defaults Profile (`builtin_profile.json`). The table rows depend on
Cargo Containment, Cargo and LPG Carriage Condition (see TC-A26 to TC-A48).

| Input | Default | Unit | Range (hard; soft) | Where the default comes from |
|---|---|---|---|---|
| `lpg_carriage_condition` | `fully_pressurised` | – | three values; LPG only | Profile 91–94. A file with none is read as fully pressurised, with a warning |
| `fill_ratio` | 0.98 | – | 0.1–1; 0.8–0.99 | IGC 15.3, the filling limit FL. LPG loads to the IGC 15.5.1 loading limit |
| `lpg_loading_limit_basis` | `igc_15_1_3` | – | three values; LPG only | #315 |
| `lpg_loading_temperature_c` | by carriage condition (TC-A52) | °C | LPG only | #315 |
| `tank_allowable_stress_basis` | `igc_4_23_3_1` | – | two values | #313 |
| `tank_design_pressure_mpa` | 0.4 / 1.77 / 1.0 / 0.4 by row | MPa | > 0, ≤ 5; 0.1–2.5 | Table rows TC-A26 to TC-A30 |
| `tank_design_temperature_c` | −163 / 0 / −48 / −48 by row | °C | > −273.15, ≤ 200; −170–60 | Table rows TC-A31 to TC-A34; recorded only |
| `tank_material` | 9Ni / S690QL1 by row | – | 304L, 316L, 9Ni, S690QL1 | Table rows TC-A35 to TC-A38 |
| `tank_allowable_stress_mpa` | none | MPa | > 0, ≤ 1000; 100–400 | none means the material's stress |
| `tank_insulation_type` | panel / none / pu_foam by row | – | – | Recorded only |
| `tank_insulation_thickness_mm` | 330 / 0 / 200 by row | mm | 0–1000; 0–400 | Table rows TC-A40 to TC-A42 |
| `tank_structure_factor` | 1.30 / 1.12 by row | – | 1–3; 1.0–1.6 | Table rows TC-A62, TC-A63 |
| `tank_wall_thickness_mm` | none (formula) | mm | > 0, ≤ 200; 10–50 | A typed value is an override for shell and heads |
| `max_tank_od_m` | 16.0 | m | > 0, ≤ 50; 4–20 | Tool limit |
| `tanks_in_line` | 2 | tanks | 1–50; 1–4 | Original engine |
| `tank_support_height_m` | 0.5 | m | 0–5; 0.2–1.5 | Original engine |
| `tank_dome_height_m` | from the dome rule (TC-A55) | m | 0–10; 0.5–3 | Dome rule, estimate (#316); old files keep 2.0 |
| `tank_dome_diameter_m` | max(1.2, 0.17 × steel OD) | m | 0–10; 0.5–3 | Dome rule, estimate (#316); old files keep 2.0 |

Not inputs today: head type, weld efficiency, corrosion allowance, minimum thickness, insulation density, tank gaps,
cargo density.

## Limits and known findings

- **No liquid head in the wall.** IGC 4.23.2.2 and 4.13.2.4 ask for Peq = P0 + (Pgd)max. OVCD uses P0 only (TC-A14).
  IGC 4.28.1.2 gives the guidance for Pgd (full tank, maximum cargo density at the design temperature). For an LNG
  tank of about 8 m diameter the static head alone (a = 1 g, Z = 8 m, ρ = 450 kg/m³) is about 0.035 MPa, about 9 %
  of 0.4 MPa (derived).
- **No minimum design pressure check** (IGC 4.23.1.2) and **no steel-by-temperature check**: #317.
- **No external pressure or buckling check** (IGC 4.23.2.3, 4.23.3.2). Formula walls on large low-pressure tanks are
  thin (about 8.5 mm on a 9 m LNG tank, derived), so buckling may govern.
- **Heads.** A Design always gets hemispherical heads. Torispherical heads, common on built tanks, are not modelled.
  The 2:1 head has a low area (#314) and its insulated length counts only half the insulation at each end.
- **Tank weight.** One structure factor stands for rings, saddles, wood, chocks, dome, sump, fittings and access.
  The model tank weighs about half of published marine tanks (Evidence). #317 adds calibration cases first.
- **No supports.** No saddles, wood, anti-flotation chocks or stops, and no collision, flooding or 30° heel loads
  (IGC 4.15, 4.13.9; IACS UI GC8). The cost line `tank_supports` is the structure factor surplus, not a saddle count.
- **Cargo density and fill.** One density per Cargo (refrigerated for LPG) and one 98 % fill: #315.
- **Dome.** Fixed 2.0 m × 2.0 m with no source (#316). The dome has no steel weight.
- **Design temperature and insulation type** change no number.
- **Single cylinder only.** No bilobe or trilobe tanks; two published bilobe records cannot run
  (`calibration/tank_cases.yaml`, `not_yet_runnable_tank_cases`).
- **API difference.** The Python API gives the wall formula the insulated diameter (issue #292). It waits for 0.3.
- Register findings (#121) from this area so far: S690QL1 stress (#313) and the 2:1 head area (#314). Spec 0004
  Appendix A lists the shell and head formulas as "no checkable source: unverified".

## Evidence

- **Tests** (`crates/ovcd-core/tests/`):
  - `tank_wall_formula.rs`: the two-pass shell formula gives 13.08 mm for the published river LPG tank
    (4,730 mm, 1.58 MPa, 285 MPa, E = 1.0); the paper prints 13.1 mm. The full chain gives 14.0 mm.
  - `tank_datum.rs`: the wall formula takes the steel diameter in a Design and the insulated diameter in the API;
    clearances measure to the steel; the steel mass follows the material.
  - `dome_in_vc.rs`: the default dome holds 2π m³, counts in Vc and not in the cargo or inner volume.
  - `pressure_vessel.rs`, `tanks.rs`: formula values, rounding, minimums, error paths, Python parity.
  - `crates/ovcd-format/tests/type_c_defaults.rs`: every profile row has a status and a basis; a sourced row has a
    source; defaults follow Cargo and carriage condition; old files keep their results; the Design fill is 0.98.
- **Calibration** (`calibration/results.md` section 5, bands in `calibration/tank_bands.yaml`). Two held-out cases:

  | Case | Figure | Published | Model | Error | Band |
  |---|---|---|---|---|---|
  | River LPG tank, 2006 paper (run with 2:1 heads) | Gross volume | 351.4 m³ | 351.1 m³ | −0.1 % | ±5 %, inside |
  | | Cylinder wall | 13.1 mm | 14.0 mm | +6.9 % | ±15 %, inside |
  | | Tank steel | 34.4 t | 36.2 t | +5.2 % | ±20 %, inside, **in-sample** (TC-A62) |
  | 5,000 m³ LCO2 design study, 2026 (run as 9Ni) | Gross volume | 5,000 m³ | 5,011 m³ | +0.2 % | ±10 %, inside |
  | | Cylinder wall | 50.0 mm | 56.0 mm | +12.0 % | ±25 %, inside |

  The steel case uses the 2:1 head with the low area (#314) and the 285 MPa stress (#313). Both fixes will move it.
- **Tank weight per m³ (derived, not a test).** Published empty weights of two marine LNG Type C tanks give 0.115
  t/m³ (1,350 m³, 155 t) and 0.096 t/m³ (2,500 m³ bilobe, 240 t) (`ovcd-rule-sources/type-c/NOTES-weight.md`). The
  engine's formulas for a 9.0 m × 25.2 m LNG tank (330 mm insulation, 0.4 MPa, 9Ni, factor 1.30) give 1,184 m³, 48 t
  of steel and 14 t of insulation: 0.052 t/m³.
- No evaluation harness case covers tank values (`evaluation/`).

## Questions for reviewers

1. OVCD sizes the wall on P0 only. Is Peq = P0 + (Pgd)max (IGC 4.23.2.2, 4.28.1) needed at concept stage, or is a
   fixed margin on P0 enough? Which accelerations suit an inland or coastal barge under 50 m?
2. IGC 4.23.1.2 with OVCD's inputs gives a minimum P0 of about 0.46 MPa for a 0.4 MPa fully refrigerated LPG tank in
   9Ni (8 m × 40 m, ρr 0.58, σm = 230 MPa; derived). Is 0.4 MPa a fair default for that row, or should it rise?
3. Is a single hemispherical head a fair default for small Type C cargo tanks, or should the default be
   torispherical or 2:1?
4. What weight per m³ would you expect for an insulated LNG Type C tank of 500 to 3,000 m³, without and with
   saddles, dome and fittings?
5. Is a corrosion allowance of 0.5 mm for 9 % Ni and S690QL1 steels, and 0 mm for austenitic steels, usual practice?
6. Should the thickness used for weight include a forming and mill tolerance (IGC 4.23.2.1.1)? How much?
7. Is 60 kg/m³ a fair installed density for PUF or panel insulation, including cladding and fixings?
8. Are the fixed gaps between tanks, 1.0 m across and 2.0 m along, fair for saddles, chocks and access?
9. Should L/D ≤ 7 apply to a coastal Type C tank under the IGC Code, or only under ADN? Which datum: steel or
   insulated?
10. Is 200 mm of PUF enough for propane at −42 °C to −48 °C in a semi-pressurised or refrigerated tank?
11. At what diameter and design pressure does external pressure or buckling govern a 9Ni or 304L LNG tank wall?
