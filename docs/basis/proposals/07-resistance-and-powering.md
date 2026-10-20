# Proposals: 07 Resistance and powering

Not published. Companion to `docs/basis/07-resistance-and-powering.md`. Errors become findings for the register
(#121) only after maintainer triage. A bare file name such as `resistance.rs:380` is in `crates/ovcd-core/src/`.

Effect on results: **none** (text or new output only), **small** (under about 5 % on a power or speed value),
**large** (more than that, or a different governing limit).

## Errors

### E1. The limiting speed formula does not match the one-dimensional Schijf solution

- **Code:** `resistance.rs:380`: Fnh,lim = √(2/3) × (1 − n)^(1/3), with Fnh on the full depth h (`resistance.rs:381`).
- **Cited source:** "Schijf (1949) limiting speed in confined waterways" (`src/vessel_designer/core/resistance.py:10`).
  The paper was not read for this page. The check below is a derivation, not a quote.
- **Check:** one-dimensional continuity and Bernoulli for the return flow give the critical condition
  1 − n + Fnh²/2 − 1.5 Fnh^(2/3) = 0, with Fnh on the mean depth Ac / (surface width). At n = 0 its root is
  Fnh = 1.0, as it must be for open shallow water. The code gives 0.816 at n = 0.

  | n | Code Fnh,lim | 1-D solution |
  |---|---|---|
  | 0.00 | 0.816 | 1.000 |
  | 0.10 | 0.788 | 0.623 |
  | 0.20 | 0.758 | 0.475 |
  | 0.30 | 0.725 | 0.365 |

  The code is higher (less safe) for n above about 0.025. For the API default river (`resistance.rs:120-129`,
  n = 0.377) the code gives 10.8 kn. The 1-D solution gives 4.6 kn with the same n on the full depth, and 3.5 kn
  with Am = B·T (n = 0.443) on the mean depth. Squat governs that case at 2.5 kn, so the error is hidden there.
- **Effect:** large. The sweep cap (0.95 V_lim), the electric service-speed cap (0.9 V_lim) and the "channel"
  governing label all move. In the app, `limiting_speed_kn` and the length of each channel curve change, so
  saved results hashes change for Options with a channel.

### E2. The residuary method does not use its cited source, and it has a step at Fn = 0.20

- **Code:** `resistance.rs:349-355`: Rr = Rf × k(Fn), a three-piece fit with no source.
- **Cited source:** `src/vessel_designer/core/resistance.py:12` lists "Guldhammer-Harvald residuary resistance
  charts". The code does not look up those charts. The standing check calls the fit an "uncalibrated first-order
  estimate (RS1)" (`resistance.rs:733-734`).
- **Step:** k = 0.10 + 2.5 × 0.1^1.5 = 0.179 just below Fn = 0.20, and k = 0.150 at Fn = 0.20. Residuary
  resistance falls by 16 % at that point.
- **Form:** Cr is a multiple of Cf, so Cr changes with Reynolds number and with the viscosity choice. Under
  Froude's assumption Cr depends on Fn only. A reviewer must say whether this form is intended.
- **Effect:** small at Fn < 0.15 (most inland service speeds); the step is a local error near Fn = 0.20.

### E3. The midship area uses Cb, not Cm, and squat uses a different area

- **Code:** `resistance.rs:366`: Am = B × T × Cb. `squat.rs:142`: Am = B × T.
- **Source:** the midship section area is B × T × Cm by definition. No paragraph is cited in the code.
- **Effect:** large in confined water. With Cb = 0.85, n is 15 % low. V_lim (E1) is then higher and the
  blockage term β = 2 n Fnh² is lower. Both are non-conservative. One hull gets two blockage ratios in one run.

### E4. Electric installed power comes from the slowest environment, not the most demanding one

- **Code:** `electric_energy.rs:277-278`: installed = battery power of the environment with the lowest service
  speed × 1.20. The Python note says this environment "sizes propulsion"
  (`src/vessel_designer/core/resistance.py:180-183`).
- **Problem:** the rule is right only if the slowest leg also needs the most power. When the channel cap
  (0.9 V_lim) cuts the channel speed below the target, an open-sea leg at the full target speed can need more.
  With the code's V_lim the default case does not show this; with E1 fixed it can.
- **Effect:** latent today; large for the affected cases after E1 is fixed.

### E5. "Governed by power" is reported when the top of the speed grid governs

- **Code:** `resistance.rs:625-635`. With no next grid point and no channel limit, the label stays "power".
  The test `crates/ovcd-core/tests/resistance.rs:36-38` asserts "power" for the open sea at the grid top of
  12.0 kn.
- **Effect:** none on numbers. The label misleads a reader about what limits speed.

### E6. With no valid speed, the service speed is the first grid speed

- **Code:** `resistance.rs:637-640`. The reported resistance, power and fuel then belong to a speed that fails
  the power or squat test. The check `power_within_installed` fails, but squat failure gives no error check.
- **Effect:** none on the curves; the API service values can mislead.

### E7. SFC is applied to delivered power

- **Code:** `resistance.rs:428`: fuel = P_D × SFC. SFC is normally stated on engine (brake) power. No shaft,
  gearbox or electrical transmission loss sits between P_D and the engine.
- **Effect:** small for a direct drive, larger for diesel-electric. Fuel is low by the transmission efficiency.
  Non-conservative.

### E8. Comments claim links that the code does not make

- `src/vessel_designer/core/resistance.py:190-194` says the binding power "feeds `electric_barge` battery
  sizing, replacing its hard-coded 600 kW". No such module exists. The stern has a fixed count of battery units
  (`aft_arrangement.rs:26-27`), and the cost uses a fixed 620 kWh and 230 kW (`barge_cost.rs:51-52`).
- `resistance.rs:1-7` says "Nothing is fixed in the port". Many coefficients are fixed in code (page rows
  RP-A04 to RP-A28).
- **Effect:** none. A reader can trust a link that does not exist.

### E9. The calibration note and the app use different waterline lengths

- `calibration/results.md` section 3 says "Lwl = 0.97·LOA". The app sets Lwl = LOA (`simulate.rs:327`).
- **Effect:** small. The published 1.7-2.1 power ratio is for a different Lwl from the one the app uses.

### E10. Two values of g and of the knot in one calculation

- `resistance.rs:34-35` (9.81, 0.5144) and `squat.rs:20`, `squat.rs:136`, `waves.rs:16` (9.80665, 0.514444).
- **Effect:** negligible. Fix for consistency.

## References not checked

- **Lackenby (1963)** (`src/vessel_designer/core/resistance.py:11`): the code form δ = 1.4(T/h)^1.5 Fnh^2.5 is
  marked "fitted" (`resistance.rs:402`). No one has traced it to the paper. The β term has no source.
- **STAwave-1** (`waves.rs:3`): owned by page 08.
- **Genset data** (`aft_arrangement.rs:129-135`): a maker's product presentation, April 2023. Not read for this
  page.

## Gaps

1. **Convoy, tow and ATB powering.** A pushed convoy has no results (`run.rs:9-11`). A pushed barge has no
   pusher. ROADMAP stage 5 and spec 0001 (step 4, "convoy resistance and power") plan this.
2. **Propulsor model.** No wake, thrust deduction, open-water efficiency, nozzle, azimuth thruster or
   shallow-water loss. No bollard pull, though `calibration/pushers.yaml` holds bollard pull for 2 pushers.
3. **Installed power and margins in the app.** No P_B, sea margin, engine load limit (MCR fraction), or
   transmission and electrical losses.
4. **Viscous form factor, roughness and air resistance.** 1 + k = 1, ΔCF = 0, no wind on deck cargo or the
   raised wheelhouse.
5. **Validity ranges.** No check of Fn, Fnh, n, Cb, L/B or B/T against the range of each fit.
6. **Battery-electric sizing.** No battery capacity (kWh), depth of discharge, reserve, charge rate or hotel
   load. The number of battery units does not follow the energy per trip. The cost uses a fixed 620 kWh.
7. **Gas-fuelled option.** No electrical load from resistance, no gas consumption, no boil-off balance. The
   option is not in the Defaults Profile, so the app cannot select it.
8. **Current and route.** The current always opposes the vessel. No downstream leg, no current that varies
   along the route.
9. **Water temperature.** ν is fixed near 15 °C. Tropical rivers are warmer.
10. **Endurance.** Fuel use does not set fuel tank size or the 200 t stores weight (`models.rs:218`).
11. **Waves in the app.** The app never sets a wave height, so a coastal Service Area gets no added resistance.
12. **Evidence.** No built-vessel speed-power point. No evaluation case for resistance. `pushers.yaml` is not
    used by any calibration case.
13. **Glossary terms missing** (`GLOSSARY.md`): Service Speed, Limiting Speed, Blockage Ratio, Propulsive
    Efficiency, Delivered Power, Installed Power, Binding Environment. "Environment" (one channel section for
    resistance) can be confused with Service Area; propose "Waterway Section".

## Changes

| # | Change | Fixes | Effect on results | Order |
|---|---|---|---|---|
| C1 | Replace the limiting speed with the 1-D Schijf solution on the mean depth (or the reviewer's choice). Add a test at n = 0 (Fnh = 1). | E1 | Large; hashes change for Options with a channel | 1 |
| C2 | Use Am = B·T·Cm in resistance, with Cm an input (default near 1 for barges); use the same Am in squat. | E3 | Large in confined water | 1 (with C1) |
| C3 | Fix the comments and docstrings: residuary reference, battery sizing claim, "nothing is fixed". | E2 (text), E8 | None | 2 |
| C4 | Add a "speed grid" governing label; flag a service speed that fails squat. | E5, E6 | None | 2 |
| C5 | Size electric installed power on the highest battery power over all environments at their service speeds. | E4 | Latent; large after C1 for some cases | 2 |
| C6 | Add validity-range OVCD Checks for each fit (Fn, Fnh, n, Cb). | Gap 5 | None (warnings only) | 3 |
| C7 | Choose a sourced residuary method valid for full inland hulls; until then, make the fit continuous at Fn = 0.20. | E2 | Small (continuity); large (new method) | 4, after reviewer answer |
| C8 | Add (1 + k) and ΔCF as Estimate inputs, defaults from the chosen source. | Gap 4 | Large | 4 (with C7) |
| C9 | Report P_B = P_D / η_transmission and installed power with sea margin and MCR fraction as inputs; take fuel on P_B. | E7, gap 3 | New outputs; fuel small | 5 |
| C10 | Make Lwl an input or derive it from the hull form (page 01); align the calibration note. | E9 | Small to medium | 6 |
| C11 | Battery sizing: kWh from energy per trip, reserve and depth of discharge; drive the battery unit count and the cost `battery_kwh`. | Gap 6 | Large for electric Options | 7 |
| C12 | Gas-fuelled: electrical load from P_D plus hotel load; gas consumption; boil-off balance check; add `propulsion` to the Defaults Profile. | Gap 7 | New outputs | 7 |
| C13 | Convoy and pusher powering, with a propulsor and bollard-pull model calibrated on `pushers.yaml`. | Gaps 1, 2, 12 | New outputs | 8 (ROADMAP stage 5) |
| C14 | Downstream leg and a current per route section; water temperature input for ν; one g and one knot constant. | Gaps 8, 9, E10 | Small | Any time |
| C15 | Add an evaluation case and a built-vessel speed-power calibration case for resistance. | Gap 12 | None | With C1 |
