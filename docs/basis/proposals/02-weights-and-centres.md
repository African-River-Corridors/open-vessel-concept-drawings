# Proposals: 02 Weights and centres

Not published. Companion to `docs/basis/02-weights-and-centres.md`. Ids `WC-Ann` refer to the assumptions table
on that page. Errors become findings for the register (#121) after maintainer triage.

## Errors

Each error gives the code location and the text it disagrees with.

### WC-E1. Default Cb is outside the range its own basis cites

- **Code:** `crates/ovd-format/src/builtin_profile.json:52`, `block_coefficient` default 0.85, used for every Hull
  (`crates/ovd-format/src/simulate.rs:667`).
- **Cited source:** the same entry's basis says "calibrated box barges span 0.883 to 0.995".
  `calibration/factors.yaml` gives the fitted box-barge Cb as 0.928, and `calibration/README.md` ("Corrections
  applied") lists "Box-barge Cb = 0.928 on LOA" as applied. The harness uses 0.928
  (`src/ovd/calibration.py:161`); the app does not.
- **Effect:** large. For a box-hold barge, displacement at a given draught is 8.4 % low (0.85 / 0.928). The
  weight-bound cargo falls by more than that, because the lightship stays the same. The calibration bands
  describe the 0.928 runs, so they do not cover what a user sees with the default.

### WC-E2. Lightship and cost use two different steel weights

- **Code:** lightship uses `steel_top_down` (`crates/ovd-core/src/weights.rs:163`, `176`). The cost masses use
  `hull_steel_total + wing_plate + weather_shield`, which is `steel_bottom_up`
  (`crates/ovd-format/src/cost.rs:76-78`).
- **Text it disagrees with:** the doc comment of `calculate_weights` (`weights.rs:42-43`) says hull steel "is
  estimated from plating areas and thicknesses, plus a structure factor". That is the bottom-up figure, which the
  lightship does not use. `models.rs:137` says piping is a fraction of "(hull + tank weight)"; the code uses the
  top-down steel.
- **Effect:** large for cost (page 10). In the parity case (`crates/ovd-core/tests/weights.rs:57-58`) the vessel
  floats 865.8 t of hull steel but is costed for 568.7 t, 34 % less.

### WC-E3. `stores_weight` is an input that does nothing

- **Code:** `crates/ovd-core/src/models.rs:139-140` documents `stores_weight` as "Diesel, fresh water,
  provisions (tonnes)", default 200 t (`models.rs:218`). `weights.rs:193-194` sets `stores` to 0 and says
  "Consumables are deadweight". No deadweight item carries them: deadweight is the cargo
  (`simulate.rs:797`, `1081`).
- **Source:** IS Code 2008 (MSC.267(85)), Introduction 2.23, defines the lightship condition without
  consumables and stores. Part B 3.4.1.2 lists the cargo ship loading conditions with full stores and fuel, and
  with 10 % remaining. OVD puts the stores in neither the lightship nor the loading conditions.
- **Effect:** small to medium. Cargo is high by the consumables mass. The draught at that cargo is low by the same
  mass. KG ignores them.

### WC-E4. Gas hull steel KG uses D, the steel mass uses D_m

- **Code:** `steel_top_down` is a rate × LOA × B × D_m, where D_m can be above D (the wing plate level,
  `weights.rs:157-162`). Its KG is 0.5 × D (`crates/ovd-core/src/gas_stability.rs:715`).
- **Text it disagrees with:** `weights.rs:152` says "Moulded depth = wing plate level". The KG item does not
  follow.
- **Effect:** small, non-conservative. KG is low when the wing plate is above D. With the default wing tank height
  ratio (0.66) and a tank wider than the depth, this happens often.

### WC-E5. Legacy KG and LCG items do not sum to the lightship

- **Code:** `crates/ovd-core/src/hydrostatics.rs:83`, `91`, `180`, `193` weigh `hull_steel_total` and
  `tank_shells`. The lightship (`weights.rs:176`) uses `steel_top_down` and `tank_total` (shells plus internals).
  The wing plate, the weather shield and the tank internals have no centre.
- **Text it disagrees with:** the `calculate_stability` comment says "weighted average of component centres"
  (`hydrostatics.rs:80`). The gas path checks the same sum (`gas_stability.rs:805`).
- **Effect:** none on `simulate` results (parity interface only, `crates/ovd-format/src/eval.rs:1544`, `1570`).
  It matters if a caller uses the legacy chain for KG or trim.

## Gaps

### WC-G1. The stern block has no mass

The self-propelled stern block sets the stern void (`barge_capacity.rs:782-786`) but adds no mass to the
lightship (WC-A29). The box k was fitted on pushed barges, which have no stern block. The genset engine mass is
already known for a gas-fuelled block (`aft_arrangement.rs:1025`) and is not used. The dry result gives no warning;
only the gas stability result names it (`gas_stability.rs:798-803`).
Direction: cargo and laden draught are non-conservative. The light draught is too small, so the light air draft
is too large, which is conservative for the bridge gate. KG is unknown.

### WC-G2. No LCG, trim or TCG in the main result

`simulate` floats every condition on an even keel (WC-A48). The stern block, the bow void and the tank zone are
not centred, so trim can be large on a self-propelled Hull. The legacy `calculate_trim` exists but is not used.

### WC-G3. Sloped-hopper build-up has no calibration case

The hopper Hull keeps the build-up from the original engine (WC-A28). Applied to box barges it gave 31–111 % high
(`calibration/README.md`, pass 1). No hopper barge with a published lightship is in `calibration/cases.yaml`.

### WC-G4. Gas lightship factors have no calibration case

The 0.10 t/m³ rate, 4 % piping and 3.5 % outfitting (WC-A01, A04, A05) have no source and no case. Gas and tank
barges are in `not_yet_runnable_cases` (`calibration/cases.yaml:1321`).

### WC-G5. Deadweight is cargo only

No fuel, fresh water, stores, crew or ballast (WC-A35). Compare ABS Rules for Building and Classing Steel Barges
(2024), 3-1-1/45: deadweight is the displacement at the summer load line, in water of specific gravity 1.025, less
the lightweight. OVD's deadweight is at the user's water density (1.0 by default) and at the governing draught.
The result name is the same; the meaning is narrower.

### WC-G6. No deck barge or pontoon model

They run as box barges in calibration only (WC-A27). Out of sample, pontoon lightship is 27 % low on average and
deadweight 26 % high (`calibration/results.md`, summary). The app has no deck or pontoon Cargo Containment.

### WC-G7. Lightship KG is mid-depth, not deck level

OVD puts dry lightship KG at 0.5 × D (`barge_capacity.rs:704`) and gas hull steel at 0.5 × D. IS Code 2008
(MSC.267(85)), Part B 8.6, says a pontoon need not be inclined if a conservative lightship KG is assumed, and that
KG can be assumed at the main deck. OVD has no option for that conservative value.

### WC-G8. Box-barge Cb fit depends on k

Six of the seven Cb points have no observed lightship and use k × L·B·D (`calibration.py:132`). An error in k moves
Cb. The two are not independent.

### WC-G9. Container k is unreachable

`calibration/factors.yaml` has a container k (one case), but the app has no container hold
(`crates/ovd-format/src/run.rs:172-176`, `simulate.rs:193-203`).

### WC-G10. No weight margin

No growth or design margin is added to the lightship at concept stage.

### WC-G11. Glossary terms missing or doubled

- Missing: **Lightship**, **Deadweight**, **Stern Block**, **Lightship Factor** (k), **Weight Breakdown**,
  **Top-down Steel** and **Bottom-up Steel**.
- `GLOSSARY.md` has two **Moulded Depth** entries with different text. Merge them.
- **Lightship** should say it follows IS Code 2008 Introduction 2.23 in intent, but has no consumables model.

## Changes

| # | Change | Effect on results | Order |
|---|---|---|---|
| WC-C1 | Make the default Cb depend on the Cargo Containment: box hold 0.928 (fitted), Type C tanks 0.85 until a gas case exists. Fix the profile basis text. (WC-E1) | Large: box-barge deadweight up by about 10 % | 1 |
| WC-C2 | Fix the `calculate_weights` doc comment and the `piping_factor` comment to say what the code does. (WC-E2) | None | 1 |
| WC-C3 | Use one steel figure for the lightship, the KG and the cost, or show both with the delta beside the cost. Decide which with a gas case. (WC-E2) | Large on cost; none on lightship if top-down stays | 2 |
| WC-C4 | Move the gas factors (WC-A01, A04, A05, A09, A10, A13) and the hopper factors (WC-A16, A19, A21) into the Defaults Profile as Estimates (#4). | None | 2 |
| WC-C5 | Add the stern block as lightship items with mass and centres: genset mass from the existing table, other units as Estimates. Add a dry-path warning until then. (WC-G1) | Large on self-propelled cargo and draught | 3 |
| WC-C6 | Remove `stores_weight`, or make it a deadweight item in the laden and light conditions as IS Code Part B 3.4.1.2 describes. (WC-E3, WC-G5) | Small to medium | 4 |
| WC-C7 | Put the top-down steel KG at 0.5 × D_m. (WC-E4) | Small; KG up | 5 |
| WC-C8 | Add LCG for each lightship item and the cargo, and trim, to the main result. Reuse the legacy item positions as a start. (WC-G2, WC-E5) | Medium; new results | 6 |
| WC-C9 | Find published lightships for hopper barges and gas barges; add them as cases. Until then, label the hopper and gas lightship "uncalibrated" in the result. (WC-G3, WC-G4) | Unknown until cases exist | 6 |
| WC-C10 | Add an option for lightship KG at the main deck (IS Code Part B 8.6), default on for deck barges and pontoons when they exist. (WC-G7) | Medium; GM down | 7 |
| WC-C11 | Fit k and Cb jointly, or fit Cb only on cases with an observed lightship. (WC-G8) | Small; Cb may move | 7 |
| WC-C12 | Add a deck/pontoon Cargo Containment with its own k once enough cases exist. (WC-G6) | Large for those Hulls | 8 |
| WC-C13 | Add the glossary terms and merge the two Moulded Depth entries. (WC-G11) | None | 1 |
| WC-C14 | Decide whether a concept-stage lightship margin belongs in the Defaults Profile. (WC-G10) | Small to medium | 8 |
