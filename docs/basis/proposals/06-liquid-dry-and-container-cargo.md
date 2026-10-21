# Proposals: 06 Liquid, dry bulk and container cargo

Not published. Companion to `docs/basis/06-liquid-dry-and-container-cargo.md`. Ids `CG-A<nn>` refer to the
assumptions table of that page. Code state: 2026-10-20.

## Errors (code against its cited source, or against its own basis)

These are candidates for the findings register (#121) after maintainer triage.

### E1. Bulk GM check does not run for the dense cargo case, and runs at the wrong load when it does

- **Code:** `crates/ovcd-core/src/barge_capacity.rs` line 1002 (`props.phase == "dry_bulk" && draft > 0.0 &&
  !overloaded`) and lines 1003 to 1015; `crates/ovcd-format/src/simulate.rs` lines 744 and 759 (the Design carries
  `min(volume, weight)` cargo, and `capacity_checks` passes on whatever `gm_positive` the core made).
- **Source:** IS Code 2008 (MSC.267(85)) Part A 2.2.4: initial GM0 not less than 0.15 m. The code uses the same
  0.15 m but cites nothing.
- **Disagreement:** the GM is computed only for the volume-bound load, and only if that load floats. For bauxite in a
  box hold the volume-bound load is often heavier than the hull floats (hand calculation on the page: LOA 100, B 16,
  D 6 gives about 9,090 t against 8,160 t at D). Then no GM check appears at all, and the Design reports a
  weight-bound cargo with no stability figure. When the check does run, it is for a heavier cargo than the Design
  reports.
- **Effect on results:** none on capacity; large on what the user is told about stability (a missing check).

### E2. LPG cargo density ignores the LPG Carriage Condition

- **Code:** `crates/ovcd-core/src/cargo.rs` lines 51 to 82 (one density per LPG Cargo, fully refrigerated values);
  `barge_capacity.rs` line 828 (density read with no carriage condition); `builtin_profile.json` line 92 (default
  condition `fully_pressurised`).
- **Basis:** the Python twin's own comment (`src/vessel_designer/core/cargo.py` lines 37 to 45) says 581 kg/m³ is the
  refrigerated value and 489 to 493 kg/m³ is propane at about +25 °C. Its cited sources were not read in this review,
  so this is an internal inconsistency, not a checked source error.
- **Effect:** large. A fully pressurised LPG Option (the default) reads about 16 % too much cargo mass, and the draught
  and weight-bound limit follow.

### E3. Free surface moment treats the volume fill as a height fraction

- **Code:** `crates/ovcd-core/src/tanks.rs` line 280 (`h = fill_ratio * 2.0 * r`).
- **Basis:** geometry. For a horizontal circular tank at 98 % volume fill, the liquid height is about 95 % of the
  diameter, and the free surface chord is about 0.89 r. The code gives a chord of about 0.56 r, so `chord³` and the
  moment are about a quarter of the true value. No source is cited for the formula.
- **Reach:** only `hydrostatics::calculate_stability` (line 132), which only the Python API calls
  (`crates/ovcd-format/src/eval.rs` line 1544). A Design does not use it; the gas stability of a Design uses its own
  free surface (page 03).
- **Effect:** none on Designs; large on API users of `calculate_stability` (GM_fluid too high, non-conservative).

### E4. Default block coefficient sits below its own stated basis for box barges

- **Code:** `crates/ovcd-format/src/builtin_profile.json` lines 52 to 58 (default 0.85; basis "calibrated box barges
  span 0.883 to 0.995"); `calibration/factors.yaml` (`block_coefficient_box_barge_on_loa` 0.928).
- **Disagreement:** the default for a box barge is below every calibrated box barge. Calibration results use 0.928;
  a Design uses 0.85. The deadweight evidence on the page therefore does not describe the default Design.
- **Effect:** large on box barge capacity (about −9 % weight-bound displacement at the same draught; about 700 t on
  the LOA 100 × B 16 × D 6 example). Conservative for cargo, non-conservative for light air draft.

### E5. Comment and value disagree on the LPG mix density

- **Code:** `src/vessel_designer/core/cargo.py` lines 65 to 66 say butane 601 kg/m³; `cargo.rs` line 76 holds
  588.4, which is the blend with butane at 600 (the database value, line 65). With 601 the blend is 588.8.
- **Effect:** none on results (value is consistent with the database); fix the comment.

## Gaps

- **G1. No liquid cargo other than liquefied gas.** No product, chemical or vegetable oil Cargo; no integral or
  independent tank model for them; no ADN tank vessel types; no tank capacity limit (ADN 2025 9.3.2.11.1(a): the
  table by L × B × H, at most 380 m³ per tank above 3,750 m³); no tank vessel double hull (9.3.2.11.7: side 1.00 m,
  or 0.80 m with reinforcement; mean double bottom 0.70 m, never below 0.60 m); no cofferdam rule for these cargoes.
  The calibration path lists seven double-skin tank barges it cannot run. The profile has an empty `liquid` depth
  group.
- **G2. No container stowage.** No container Cargo, no TEU from bays × rows × tiers, no stack weight or average
  container mass, no lashing or visibility limit, no container stability (ES-TRIN 2025/1 chapter 27, articles 27.01
  to 27.04, secured and non-secured containers). The core `"container"` hold type and the calibrated container k
  (0.1711) are unreachable. Catalogue TEU figures are dropped by the generated core table.
- **G3. Dry bulk has one Cargo.** No stowage factor table, no light or heavy cargo distinction, no angle of repose or
  cargo shift. A user cannot type a density.
- **G4. Stern block mass is missing from the dry bulk lightship** (CG-A40). It is also missing in every capacity path
  that uses the k factor.
- **G5. No double hull or double bottom rule check for holds.** The wing void and double bottom are free inputs.
  ADN 2025 9.1.0.91.2 (side 0.80 m, 0.60 m with reinforcement) and 9.1.0.91.3 (double bottom at least 0.50 m) exist
  for double-hull dry cargo vessels.
- **G6. Deadweight equals cargo.** No stores, fuel, water or crew allowance in the dry bulk Design path.
  (`models.rs` line 218 has `stores_weight` 200 t, but no code in `crates/ovcd-core/src` reads it.)
- **G7. No utilisation against a Vessel Class deadweight.** The class reference deadweight is only shown.
- **G8. Gas loading limit (IGC 15.5.1) not applied.** Already listed in spec 0004 as a later Rule.
- **G9. `cargo_density_override` is a dead field** in `BargeInputs` (`models.rs` lines 110, 205): the API accepts it
  and nothing reads it.
- **G10. Glossary terms missing:** cargo zone, volume-bound and weight-bound cargo, stowage factor, TEU, deadweight,
  wing void, hold double bottom, end bulkhead allowance, filling limit and loading limit. `GLOSSARY.md` also defines
  **Moulded Depth** twice.

## Proposed changes, in order of work

| # | Change | Effect on results | Notes |
|---|---|---|---|
| 1 | Run `bulk_gm` at the cargo the Design reports (weight- or volume-bound), always, in `simulate.rs` after line 744; keep it a warning until a Rule Set exists (fixes E1) | None on capacity; adds a GM figure and check to every dry bulk Option | Small change. Needs the `ovcd` Rule Set id later (`is2008/A.2.2.4/gm0`) |
| 2 | Make LPG density follow the carriage condition: a `default_by_table` density in the Defaults Profile, keyed like the tank tables, each row sourced (fixes E2) | Large for pressurised LPG (cargo about −16 %) | Design change: needs a design-impact table and a CHANGELOG entry |
| 3 | Fix the free surface liquid height: solve the circular segment for the volume fill (fixes E3) | None on Designs; large on the API `calculate_stability` | Python parity: the change must go in both twins, or the parity test breaks |
| 4 | Box barge Cb default by containment: 0.928 for `box_hold` and `sloped_hopper_hold`, 0.85 kept for gas (fixes E4) | Large for dry bulk capacity (about +9 % weight-bound displacement) | Keyed default already supported (`KEYED_DEFAULTS`); old files keep their stored value |
| 5 | Add the stern block mass to the lightship of a self-propelled hull (G4) | Small to medium (tens of tonnes) | Needs masses for the 40 ft units, thrusters and wheelhouse, with sources |
| 6 | Make `bulk_steel_rate_per_cbm`, `bulk_structure_factor`, piping and outfitting factors inputs, or drop the hopper build-up for a calibrated k (CG-A34 to A37) | None at defaults | The build-up reads about 2× the box barge lightships; calibration of a real hopper barge is needed first |
| 7 | Add a dry bulk Cargo table (density and stowage factor), and a user density input (G3) | None at defaults | Each value needs a published source |
| 8 | Add a stores and consumables allowance to the dry bulk deadweight (G6) | Small (cargo falls by the allowance) | Default could reuse `stores_weight`, with a basis |
| 9 | Add ADN double hull checks as an optional Rule Set (`adn-dry`, `adn-tank`) with 9.1.0.91 and 9.3.2.11.7 to .8 (G5, part of G1) | None unless selected | Spec 0004 shape: `sizes` Rules on wing void and double bottom |
| 10 | Container cargo: a `container_hold` containment, TEU = bays × rows × tiers from the hold size, a mean TEU mass, and the ES-TRIN chapter 27 stability check (G2) | New results only | Calibrate with the container k case and catalogue TEU figures |
| 11 | Liquid cargo: a product Cargo and an integral tank hold, with the ADN tank capacity table (G1) | New results only | Calibrate with the seven tank barge cases already listed |
| 12 | Remove `cargo_density_override` or wire it (G9); fix the LPG mix comment (E5); add the glossary terms (G10) | None | Housekeeping |

Order reason: 1 to 3 fix wrong or missing outputs at low cost; 4 and 5 move capacity and need design-impact review;
6 to 8 need sources; 9 to 11 are new capability.
