# Resistance and powering

> Concept and preliminary design only. Not for approval. Draft: needs expert review.
>
> Code state: written against `main` at commit `f6d446c` (2026-10-20).

**For:** Hydrodynamicist, marine engineer  ·  **Code:** `crates/ovd-core/src/resistance.rs`,
`electric_energy.rs`, `aft_arrangement.rs`, `squat.rs` (speed limit only), `waves.rs` (added resistance only);
`crates/ovd-format/src/simulate.rs` (the app path), `run.rs`  ·  **Rule Sets:** none. No Rule in any Rule Set
tests resistance or power. The methods have `method_status` unverified (spec 0004, "Measure (code)").

In the tables, a bare file name such as `resistance.rs:330` is in `crates/ovd-core/src/`. `simulate.rs` and
`run.rs` are in `crates/ovd-format/src/`. Line numbers are for the tree this page was written from.

## What OVD computes here

OVD has two paths. They do not give the same outputs.

**The app and `ovd run` (a saved Design).** For each Option with one Hull, `simulate` gives `curves`: one
list per environment ("Channel" if the Option has a channel depth and bottom width, and always "Deep water").
Each list has:

- `limiting_speed_kn`: the confined-water limiting speed ("Schijf"), null in deep water.
- `blockage`: the blockage ratio n = Am / Ac.
- `points`: for each speed, `kn` (over ground), `kw` (delivered power), `fuel_lph` (diesel, litres per hour),
  `squat_m` and `squat_limited`.

The app gives no service speed, no installed power and no energy per trip. A pushed convoy gets no results
("convoy results not yet available"). A pushed barge gets the curve of the barge hull alone, without its
pusher (`run.rs:9-11`).

**The Python and evaluation API (`resistance.compute`, `electric_energy.compute`, `aft_arrangement`).** This
path also gives:

- the service speed in each environment and what governs it (`power`, `squat` or `channel`);
- resistance, delivered power and fuel (l/h and l/nm) at the service speed;
- the binding environment (the one with the lowest service speed);
- for an electric barge: shaft and battery power, energy per km, per trip and per year, and installed power;
- for a self-propelled stern: thruster count, 40 ft battery units and, for the gas-fuelled option, the
  number and model of gas gensets (N+1) and the engine room and gas valve unit (GVU) room lengths.

## How it works today

1. **Speed grid.** Speeds run from a minimum to a maximum by a fixed step. The grid is built by repeated
   addition and then rounded to 0.1 kn, so a step of 0.25 kn gives 2.0, 2.2, 2.5, 2.8 (`resistance.rs:567-573`).
   The app uses 1.0 kn to `max_speed_kn` in steps of 0.5 kn (`simulate.rs:338-340`). Label: Fixed in code.
2. **Speed through water.** V = (speed over ground + current) × 0.514444 m/s per kn (1852/3600). The current always opposes
   the vessel (`resistance.rs:445-447`). Label: Fixed in code.
3. **Wetted surface.** S = Lwl × (B + 2T) × 1.02, unless the API caller gives S (`resistance.rs:308-313`,
   `660-664`). In the app, Lwl = LOA (`simulate.rs:327`). Label: Fixed in code.
4. **Friction.** ITTC 1957 line: Cf = 0.075 / (log10 Rn − 2)², Rn = V·Lwl/ν. Below Rn = 1e5, Cf = 0.008.
   ν = 1.19e-6 m²/s if the water density is at least 1.02 t/m³, else 1.14e-6 m²/s. Rf = ½ρV²S·Cf
   (`resistance.rs:315-344`). Label: Method.
5. **Residuary.** Rr = Rf × k(Fn), Fn = V/√(g·Lwl), with k = 0.10 below Fn 0.10; 0.10 + 2.5(Fn − 0.10)^1.5
   up to Fn 0.20; 0.15 + 8(Fn − 0.20)² above (`resistance.rs:346-357`). No source gives these coefficients.
   Label: Fixed in code (the method itself: Method, reference not matched; see the triage tickets.
6. **Shallow and confined water.** In an environment that is not marked deep water, the deep-water
   resistance Rf + Rr is multiplied by (1 + δ + β) (`resistance.rs:385-419`, `483-484`):
   - δ = 1.4 (T/h)^1.5 Fnh^2.5 for Fnh < 0.95, else δ = 5.0 ("Lackenby", coefficients "fitted", source not
     given);
   - β = 2 n Fnh², the blockage term, with n = Am/Ac, Am = B·T·Cb and Ac = (bottom width + surface width)/2 × h.
   Label: Fixed in code.
7. **Limiting speed.** Fnh,lim = √(2/3) × (1 − n)^(1/3), V_lim = Fnh,lim × √(g·h) (`resistance.rs:369-383`).
   The sweep stops at 0.95 × V_lim (`resistance.rs:575-581`). If Am ≥ Ac, V_lim = 0 and no cap applies.
   Label: Method (named "Schijf (1949)"; the formula does not match the one-dimensional Schijf solution, see
   the triage tickets).
8. **Added resistance in waves.** Only if the significant wave height is above zero: STAwave-1 for head seas,
   R = (1/16) ρ g Hs² B √(B/L_BWL), times cos² of the heading (`waves.rs:104-126`). The app never sets a wave
   height, so this term is zero there. Page 08 covers waves. Label: Method.
9. **Squat.** In a confined environment the squat module gives the dynamic under-keel clearance at each speed.
   A speed is "squat limited" if that clearance is below the squat safety margin, default 0.15 m
   (`resistance.rs:506-527`). Squat does not change the resistance; it only removes speeds from the service
   speed. Page 08 covers the squat methods.
10. **Power.** P_E = R_total × V (kW). P_D = P_E / η_D, with η_D the "propulsive efficiency", default 0.45
    (`resistance.rs:529-532`). A speed has "power available" if P_D ≤ the engine power (`resistance.rs:550`).
    There is no shaft, gear or generator loss, no sea margin and no engine load limit. Label: Method; η_D is an
    Estimate.
11. **Fuel.** Fuel (l/h) = P_D × SFC / (fuel density × 1000), with SFC = 195 g/kWh at every load and density
    0.845 kg/l. Fuel (l/nm) = fuel (l/h) / speed over ground (`resistance.rs:421-436`). Label: Fixed in code
    (in the app).
12. **Service speed (API only).** The highest grid speed below the cap that has power available and is not
    squat limited. If there is none, the first grid speed is used (`resistance.rs:587-598`, `637-640`). The
    label "squat" or "channel" comes from the next grid point; else "power" (`resistance.rs:624-635`). The
    binding environment is the one with the lowest service speed (`resistance.rs:249-260`).
13. **Electric energy (API only).** `electric_energy.compute` runs steps 1 to 11 with an engine power of
    1e12 kW, a design channel and, by default, an open-sea environment (`electric_energy.rs:181-222`). Then:
    P_battery = P_D / 0.92; energy per km = P_battery / (speed over ground in km/h); service speed = the
    lower of the 6.0 kn target, 0.9 × V_lim and the top of the curve; values at service speed by linear
    interpolation; energy per trip = energy per km × 102 km; energy per year = per trip × 2 × 300 trips;
    installed power = P_battery of the binding environment × 1.20 (`electric_energy.rs:225-278`).
    Label: Estimate (all defaults).
14. **Self-propelled stern.** The stern block holds up to 3 thrusters across the beam, 40 ft battery and
    accommodation units and a lifting wheelhouse (`aft_arrangement.rs:615-1029`). The thruster count, the
    battery unit count and the block length are layout values. No power, thrust or energy value sets them.
    Page 05 and page 02 cover the layout and the weights.
15. **Gas-fuelled stern (API only).** With `propulsion = "gas_fuelled"` the block has no battery units. It
    gets an engine room with gas gensets abreast and a GVU room forward of it. If the caller gives an
    electrical load, OVD picks the set and count from an 8-set catalogue so that N − 1 sets carry the load,
    with N ≥ 2 and the least installed kWe (`aft_arrangement.rs:223-246`). Fuel is assumed to come from the
    cargo boil-off; there is no fuel tank and no boil-off balance (`aft_arrangement.rs:940-946`). The app
    cannot select this option: `simulate` never sets `propulsion` (`barge_capacity.rs:760-762` reads it, and
    only `eval.rs:395-411` writes it).
16. **Push-tow and tug-barge systems.** OVD does not compute convoy, tow or ATB resistance or power. A
    pushed convoy has no results; a pushed barge is the barge hull alone (`run.rs:9-11`, `88-91`). The tug
    length, beam and notch depth in `models.rs:223-225` are for drawings only. `calibration/pushers.yaml`
    holds 10 published pushers; no code reads it yet.

## Assumptions, rules and estimates

| Id | Item | Value or method | Label | Source or basis | Where applied | If it is wrong |
|---|---|---|---|---|---|---|
| RP-A01 | Gravity (resistance, squat, waves) | 9.80665 m/s² | Fixed in code | Standard acceleration (3rd CGPM, 1901; source not held); one definition in `constants.rs` (issue #526) | `constants.rs` | Negligible |
| RP-A02 | Knot to m/s | 1852/3600 = 0.514444 | Fixed in code | The international nautical mile is 1852 m (source not held); one definition in `constants.rs` (issue #526) | `constants.rs` | Negligible |
| RP-A03 | Frictional line | ITTC 1957, Cf = 0.075/(log10 Rn − 2)² | Method | ITTC 1957 friction line, named in `core/resistance.py:9` | `resistance.rs:324-331` | Unknown; standard line |
| RP-A04 | Low-Rn fallback | Cf = 0.008 if Rn < 1e5 | Fixed in code | None stated ("laminar fallback") | `resistance.rs:327-329` | Not reached at real sizes |
| RP-A05 | Kinematic viscosity | 1.19e-6 m²/s (salt), 1.14e-6 m²/s (fresh) | Fixed in code | None stated; near 15 °C water | `resistance.rs:316-322` | Conservative in warm water (Cf too high) |
| RP-A06 | Salt/fresh switch | ρ ≥ 1.02 t/m³ is salt | Fixed in code | None stated | `resistance.rs:317` | Negligible |
| RP-A07 | Wetted surface | S = Lwl(B + 2T) × 1.02; 0 = compute | Fixed in code | Box barge: bottom plus two sides | `resistance.rs:308-313`, `660-664` | Conservative for shaped hulls (S too high) |
| RP-A08 | "Rake form factor" on S | 1.02 | Fixed in code | None stated | `resistance.rs:312` | Unknown |
| RP-A09 | Waterline length in the app | Lwl = LOA | Fixed in code | None stated | `simulate.rs:327` | Unknown: S too high, Fn too low |
| RP-A10 | Residuary method | Rr = Rf × k(Fn), three-piece fit | Method | `core/resistance.py:12` names Guldhammer-Harvald charts; the code does not use them | `resistance.rs:346-357` | Unknown |
| RP-A11 | Residuary ratio, Fn < 0.10 | k = 0.10 | Fixed in code | None stated | `resistance.rs:349-350` | Unknown |
| RP-A12 | Residuary ratio, 0.10 ≤ Fn < 0.20 | k = 0.10 + 2.5(Fn − 0.10)^1.5 | Fixed in code | None stated | `resistance.rs:351-352` | Unknown |
| RP-A13 | Residuary ratio, Fn ≥ 0.20 | k = 0.15 + 8(Fn − 0.20)² | Fixed in code | None stated. k falls from 0.179 to 0.150 at Fn = 0.20 | `resistance.rs:353-354` | Non-conservative just above Fn 0.20 |
| RP-A14 | Froude number length | Fn = V/√(g·Lwl) | Method | Standard | `resistance.rs:348`, `471` | n/a |
| RP-A15 | Viscous form factor, roughness, air, appendages | None (1 + k = 1, ΔCF = 0, no air, no appendage drag) | Fixed in code | Not modelled | `resistance.rs:478-480` | Non-conservative |
| RP-A16 | Midship section area | Am = B × T × Cb | Fixed in code | None stated; Cb used in place of Cm | `resistance.rs:364-367` | Non-conservative (n and V_lim) |
| RP-A17 | Channel section | Ac = (Wb + Ws)/2 × h, trapezoid | Method | Geometry | `resistance.rs:359-362` | n/a |
| RP-A18 | Channel surface width (app) | Ws = Wb + 2 × bank slope × h | Method | Geometry | `simulate.rs:220` | n/a |
| RP-A19 | Bank slope default | 3.0 H:V | Estimate | Profile, no basis text | `builtin_profile.json` `channel_bank_slope` | Unknown |
| RP-A20 | Limiting speed | Fnh,lim = √(2/3)(1 − n)^(1/3); V_lim = Fnh,lim √(g h) | Method | Named "Schijf (1949)", `core/resistance.py:10`; formula not traced to it | `resistance.rs:369-383` | Non-conservative (V_lim too high, see the triage tickets |
| RP-A21 | Depth used in the limit | Full depth h, not mean depth Ac/Ws | Fixed in code | None stated | `resistance.rs:381` | Non-conservative in trapezoid channels |
| RP-A22 | Hull fills channel | Am ≥ Ac or Ac ≤ 0: V_lim = 0, no cap | Fixed in code | Edge case | `resistance.rs:376-378`, `577-581` | Sweep runs uncapped |
| RP-A23 | Sweep cap | 0.95 × V_lim | Fixed in code | None stated | `resistance.rs:575-581` | Unknown |
| RP-A24 | Depth term ("Lackenby") | δ = 1.4(T/h)^1.5 Fnh^2.5, Fnh < 0.95 | Fixed in code | "fitted a = 1.4, b = 1.5, c = 2.5"; reference named Lackenby (1963), `core/resistance.py:11`; fit not traced | `resistance.rs:402-404` | Unknown |
| RP-A25 | Depth term near critical | δ = 5.0 for Fnh ≥ 0.95 | Fixed in code | None stated | `resistance.rs:405-406` | Unknown; a step in the curve |
| RP-A26 | Blockage term | β = 2 n Fnh² | Fixed in code | None stated | `resistance.rs:409-417` | Unknown |
| RP-A27 | How the terms combine | R = (Rf + Rr)(1 + δ + β) | Method | None stated | `resistance.rs:483-484` | Unknown |
| RP-A28 | Deep-water flag | Factor 1; no limit; no squat | Fixed in code | By flag, not by h/T | `resistance.rs:294-296`, `371-373`, `391-393` | Non-conservative if a "deep" case is shallow |
| RP-A29 | App deep-water case | h = 50 m, W = 1000 m, current 0, flag deep | Fixed in code | None stated | `simulate.rs:289-299` | Low |
| RP-A30 | API default environments | Open sea 50 m; river 6.5 m, 40/60 m wide, 1.0 kn; creek 8.0 m, 60/100 m, 0.5 kn | Estimate | Carried over from the original engine | `resistance.rs:107-141` | Unknown (examples only) |
| RP-A31 | Current direction | Always opposing: V_tw = V_og + current | Fixed in code | None stated | `resistance.rs:445-447` | Conservative for power; downstream leg not modelled |
| RP-A32 | Channel current default | 0.0 kn | Estimate | Profile, no basis text | `builtin_profile.json` `channel_current_kn` | Non-conservative on a river with current |
| RP-A33 | Water density (app) | 1.0 t/m³, one value for all environments | Estimate | Profile: fresh water, as the original engine | `simulate.rs` `rho`; `builtin_profile.json` | Small |
| RP-A34 | Added resistance in waves | STAwave-1, (1/16)ρgHs²B√(B/L_BWL) × cos²μ | Method | ITTC STAwave-1, named `waves.rs:3`; see page 08 | `waves.rs:119-126`; `resistance.rs:486-504` | Under-predicts for λ/L > 0.5 (code warning) |
| RP-A35 | Bow waterline length default | L_BWL = 0.2 × Lpp | Fixed in code | Blunt barge bow (code text) | `waves.rs:108` | Conservative (code text) |
| RP-A36 | Wave height in the app | 0 (never set) | Fixed in code | Rivers only in the app | `resistance.rs:98`; `simulate.rs:211-238` | Non-conservative for a coastal Service Area |
| RP-A37 | Effective power | P_E = R_total × V | Method | Standard | `resistance.rs:530` | n/a |
| RP-A38 | Delivered power | P_D = P_E / η_D; if η_D ≤ 0, P_D = P_E | Method | Standard definition | `resistance.rs:531-532` | n/a |
| RP-A39 | Propulsive efficiency η_D | 0.45 | Estimate | No basis in the profile; soft range 0.2-0.8 | `builtin_profile.json` `propulsive_efficiency`; `resistance.rs:55`; `simulate.rs:334` | Unknown; no shallow-water or propeller model |
| RP-A40 | Engine power (API default) | 2,400 kW | Estimate | None stated | `resistance.rs:54` | Sets service speed in the API only |
| RP-A41 | Engine power (app) | 1e6 kW: power never limits | Fixed in code | The app shows curves only | `simulate.rs:333` | n/a (no service speed in the app) |
| RP-A42 | Power margin in "power available" | None: P_D ≤ engine power | Fixed in code | No MCR fraction, sea margin or transmission loss | `resistance.rs:550`, `701-712` | Non-conservative |
| RP-A43 | Specific fuel consumption | 195 g/kWh at every load, on P_D | Fixed in code | None stated | `resistance.rs:56`, `428` | Non-conservative (no losses, no part-load rise) |
| RP-A44 | Fuel density | 0.845 kg/l, "marine diesel" | Fixed in code | None stated | `resistance.rs:57`, `429` | Small |
| RP-A45 | Fuel per distance | l/nm = l/h ÷ speed over ground | Method | Standard | `resistance.rs:430-434` | n/a |
| RP-A46 | Speed grid (app) | 1.0 kn to `max_speed_kn` (default 10), step 0.5 | Estimate | Profile: "the speed-power grid starts at 1 kn" | `simulate.rs:338-340` | n/a |
| RP-A47 | Speed grid (API) | 2.0 to 12.0 kn, step 0.5, rounded to 0.1 | Fixed in code | Python port | `resistance.rs:175-177`, `567-573` | Grid steps can skip a speed |
| RP-A48 | Service speed | Highest grid speed, under the cap, with power and squat clear | Method | None stated | `resistance.rs:587-598` | Resolution 0.5 kn |
| RP-A49 | No valid speed | Service speed = first grid speed | Fixed in code | Python port | `resistance.rs:637-640` | Reports a speed that fails power or squat |
| RP-A50 | What governs | From the next grid point; "channel" within 0.05 kn of 0.95 V_lim; else "power" | Fixed in code | None stated | `resistance.rs:624-635` | Says "power" when the grid top governs |
| RP-A51 | Binding environment | Lowest service speed; first of equal ones | Method | None stated | `resistance.rs:249-260` | see the triage tickets |
| RP-A52 | Squat safety margin | 0.15 m | Estimate | Profile: "carried over from the original engine" | `squat.rs:23`; `resistance.rs:101`, `526` | Unknown |
| RP-A53 | Squat speed search | Bisection, 0 to 20 kn, tolerance 0.01 kn | Fixed in code | None stated | `resistance.rs:608-618`; `squat.rs:271-313` | Capped at 20 kn |
| RP-A54 | Am in squat | B × T (resistance uses B × T × Cb) | Fixed in code | None stated | `squat.rs:142` | Two blockage ratios for one hull |
| RP-A55 | Resistance checks | Service ≤ 0.95 V_lim; P_D ≤ installed; 0 < n < 1; standing warning "first-order estimates (RS1)" | Fixed in code | Module checks, not in a Rule Set | `resistance.rs:680-739` | n/a |
| RP-A56 | Drivetrain efficiency (electric) | 0.92, battery to shaft | Estimate | None stated | `electric_energy.rs:78`, `234` | Unknown |
| RP-A57 | Propulsive coefficient (electric) | 0.45 | Estimate | None stated | `electric_energy.rs:77` | Unknown |
| RP-A58 | Target service speed (electric) | 6.0 kn | Estimate | None stated | `electric_energy.rs:79` | n/a |
| RP-A59 | Service cap (electric) | 0.9 × V_lim | Estimate | None stated | `electric_energy.rs:80`, `250-254` | Depends on RP-A20 |
| RP-A60 | Installed power margin (electric) | 1.20 × P_battery at service speed | Estimate | None stated | `electric_energy.rs:81`, `278` | Unknown |
| RP-A61 | Route (electric) | 102 km one way, 300 round trips a year | Estimate | Example values | `electric_energy.rs:82-83`, `259-261` | n/a |
| RP-A62 | Energy per km | P_battery / (V_og × 1.852) | Method | Standard | `electric_energy.rs:22`, `228-235` | n/a |
| RP-A63 | Values at service speed | Linear interpolation on the curve | Method | None stated | `electric_energy.rs:158-179` | Small |
| RP-A64 | Round trip | Both legs equal, current opposing on both | Fixed in code | None stated | `electric_energy.rs:261` | Conservative |
| RP-A65 | Electric channel defaults | h 4.0 m, 34/58 m wide, 0.5 kn, fresh; open sea 50 m, 1.025 t/m³ | Estimate | Example values | `electric_energy.rs:71-76`, `184-203` | n/a |
| RP-A66 | Battery and hotel load | Not modelled: no capacity, depth of discharge, reserve, hotel or auxiliary load | Fixed in code | Not modelled | `electric_energy.rs` (absent) | Non-conservative |
| RP-A67 | Battery units in the stern | 40 ft units: 1 (gas Cargo), 2 (dry bulk) | Estimate | Profile: "carried over from the original engine" | `aft_arrangement.rs:26-27`; `barge_capacity.rs:764-768` | Not sized from energy |
| RP-A68 | Thrusters | Up to 3; 2.60 m envelope; 0.60 m clearance; 3.50 m room | Estimate | Profile: "carried over from the original engine" | `aft_arrangement.rs:21-24`, `741-755` | Not sized from thrust |
| RP-A69 | Propulsion option | `battery_electric` (default) or `gas_fuelled`; app always battery-electric | Fixed in code | Issue #180 | `aft_arrangement.rs:94`; `barge_capacity.rs:760` | n/a |
| RP-A70 | Gas genset catalogue | 8 sets, 900-2,200 kWe, published engine size and mass | Fixed in code | Maker's medium-speed product presentation, April 2023 (cited in code, not read for this page) | `aft_arrangement.rs:129-217` | Unknown |
| RP-A71 | Genset envelope beyond engine | Alternator 1.90 × 1.90 m; base frame 0.25 m | Fixed in code | Marked ESTIMATE in code | `aft_arrangement.rs:113-127` | Unknown |
| RP-A72 | Genset choice | N+1: N − 1 sets carry the load, 2 ≤ N ≤ 6, least installed kWe | Method | Redundancy convention; no rule cited | `aft_arrangement.rs:221-246` | n/a |
| RP-A73 | Genset with no load given | 1,200 kWe set, 2 sets | Fixed in code | "smallest 8-cylinder set" | `aft_arrangement.rs:220`, `329-335` | Unknown |
| RP-A74 | Engine room spaces | Walkway 1.0 m; end clearance 1.2 m; GVU room 2.0 m | Fixed in code | Marked ESTIMATE in code | `aft_arrangement.rs:336-338`, `692-695` | Unknown |
| RP-A75 | Genset under deck | Genset height + 1.0 m ≤ moulded depth (warning) | Fixed in code | Marked ESTIMATE in code | `aft_arrangement.rs:917-926` | Unknown |
| RP-A76 | Gas fuel supply | Cargo boil-off, no fuel tank, no boil-off balance | Fixed in code | Code text cites IGC Code chapter 16 | `aft_arrangement.rs:940-946` | Non-conservative if boil-off is short |
| RP-A77 | Tug power and battery in the cost | 230 kW and 620 kWh, fixed | Fixed in code | "the engine does not size" the tug | `barge_cost.rs:51-52`; `cost.rs:72-73` | Unknown; not linked to resistance |
| RP-A78 | Stores (incl. diesel) | 200 t | Fixed in code | Not linked to fuel use | `models.rs:218` | Unknown |
| RP-A79 | Convoy, tow and ATB | No resistance or power; barge hull alone | Fixed in code | Not yet built (ROADMAP stage 5) | `run.rs:9-11`, `88-91` | Non-conservative for a convoy |

## Inputs the user can change

In the app and `.vdesign` file (Defaults Profile, `crates/ovd-format/src/builtin_profile.json`):

| Input | Default | Unit | Where the default comes from |
|---|---|---|---|
| `propulsive_efficiency` | 0.45 | - | Original engine; no basis text. Hard 0-1 (open), soft 0.2-0.8 |
| `max_speed_kn` | 10.0 | kn | Top of the speed grid. Soft 2-20 |
| `channel_depth_m` | none | m | No channel unless given |
| `channel_bottom_width_m` | none | m | No channel unless given |
| `channel_bank_slope` | 3.0 | H:V | No basis text |
| `channel_current_kn` | 0.0 | kn | No basis text |
| `channel_under_keel_clearance_m` | 0.5 | m | Sets the draught limit (page 08), not the resistance |
| `squat_safety_margin_m` | 0.15 | m | Original engine |
| `water_density_t_per_m3` | 1.0 | t/m³ | Fresh water |
| `block_coefficient` | 0.85 | - | Calibrated box barges 0.883-0.995 (page 01) |
| `thruster_count`, `thruster_envelope_m`, `thruster_clearance_m`, `thruster_room_length_m` | 3, 2.6, 0.6, 3.5 | -, m | Original engine; layout only |
| `battery_unit_count` | 1 (gas), 2 (dry bulk) | units | Original engine; layout only |
| `propulsion_rate_per_kw` | 1000 | per kW | Example cost rate (page 10); applies to the fixed 230 kW |

API only (`resistance.compute`, `electric_energy.compute`, `aft_arrangement`): engine power, SFC, fuel
density, wetted surface, all environment fields (including wave height, heading and period), the speed grid,
all `EnergyInputs` fields (drivetrain efficiency, target speed, safety fraction, power margin, route length,
trips per year), and `propulsion`, `electrical_load_kwe` and the genset fields.

## Limits and known findings

- **No convoy, tow or ATB powering.** A pushed convoy has no results. A pushed barge is computed without its
  pusher. The interaction of pusher and barges is not modelled.
- **No propeller or thruster model.** η_D is one number. There is no wake, thrust deduction, open-water
  efficiency, nozzle, bollard pull or loss of efficiency in shallow water.
- **No installed power in the app.** The app shows delivered power curves. It does not size an engine, a
  battery or a genset.
- **No margins.** No sea margin, engine load limit, transmission or electrical losses, or hotel load.
- **One fuel model.** Diesel at a fixed SFC. No gas fuel consumption, boil-off balance or energy per trip
  for the gas-fuelled option.
- **Validity ranges are not checked as OVD Checks for resistance.** The residuary fit, the depth term and
  the blockage term have no stated Fn, Fnh, n, L/B, B/T or Cb range. The code gives one standing warning
  (`resistance.rs:729-736`).
- **The limiting-speed formula does not match the one-dimensional Schijf solution.** see the triage tickets.
- **Findings register (#121):** no finding for this area is filed yet. The engineering basis triage tickets list candidates.

## Evidence

- **Parity tests only.** `crates/ovd-core/tests/resistance.rs`, `electric_energy.rs` and
  `gas_fuelled_stern.rs`, and `tests/core/test_resistance.py`, check that the Rust port gives the same
  numbers as the Python engine. They do not check the physics.
- **Calibration, design points.** `calibration/results.md` section 3 compares installed power of 14 Yangtze
  motor-vessel design points with OVD's deep-water delivered power. The ratio is 1.72 to 2.11. These points
  are model output of the source, not built ships (`calibration/README.md`, finding 3). A normal margin gives
  about 1.4; the gap is shallow water, under-predicted resistance, or both (finding 4). The note for that run
  says Lwl = 0.97 × LOA; the app uses Lwl = LOA.
- **No built-vessel speed-power point.** The calibration README says to hold power until one exists.
- **Pushers.** `calibration/pushers.yaml` lists 10 published pushers (installed power, draught, crew, convoy).
  No calibration case runs on them.
- **Evaluation harness.** No case in `evaluation/cases/` tests resistance or power.

## Questions for reviewers

1. Is the ITTC 1957 line with no form factor (1 + k = 1) acceptable for full, blunt inland hulls at
   Cb 0.85-0.99, or should OVD add (1 + k) and a roughness allowance?
2. Where do the residuary ratios 0.10, 2.5 × (Fn − 0.10)^1.5 and 0.15 + 8(Fn − 0.20)² come from? Is a
   ratio of Cr to Cf, which makes Cr depend on Reynolds number, a fair form for a barge?
3. The limiting speed uses √(2/3)(1 − n)^(1/3). Is that from Schijf (1949), and with which depth (full or
   mean)? The one-dimensional solution gives a Fnh limit of 0.62 at n = 0.1, not 0.79.
4. Should Am be B·T·Cm (about B·T for a barge) and not B·T·Cb?
5. Is δ = 1.4(T/h)^1.5 Fnh^2.5 a fair stand-in for Lackenby (1963)? Does that paper give a resistance
   factor or a speed loss? Is it right to add the blockage term β = 2 n Fnh² on top of it?
6. Is η_D = 0.45 a fair default for a ducted or azimuth-thruster inland vessel, in deep and in shallow water?
7. Which margins should OVD report between delivered and installed power (sea margin, engine load, losses)?
8. What method would you use first for a pushed convoy: a convoy resistance series, or a per-barge sum with
   a formation factor? Which published data would you check it against?
9. For the gas-fuelled option, what boil-off rate and gas consumption should OVD use to check that boil-off
   covers the fuel demand?
10. Is SFC = 195 g/kWh at every load acceptable for concept fuel figures, or should OVD use a part-load curve?
