# Spec 0006: Stern block, propulsion, tugs, pushers and ATBs

Status: **agreed with the maintainer, 2026-10-20.** Decision record: ADR 0015. Research:
`docs/research/stern-and-tugs.md`. Parent issue: see "Build tickets".

## Problem Statement

A self-propelled hull in OVCD has one stern block: up to three azimuth thrusters, 40 ft battery and
accommodation units on deck and a lifting wheelhouse. Layout values set its length; no power, thrust or
energy value sets anything. It has no mass in the lightship, so cargo and draught are too high and the
light air draft is wrong. Every result floats on an even keel. One propulsive efficiency, 0.45, serves a
battery-electric azimuth barge and a shallow-draught pusher alike. The app shows delivered-power curves
and sizes no engine. A pushed convoy has no results; a pushed barge is the barge alone; an ATB tug is
three drawing numbers. The sightline check is a warning with a fixed margin over one height, though the
maintainer decided (#264) that it must block `fit`. The drawings show a square stern and circles for
thrusters.

Designers choose the stern form, the drive and the power source early, and size the tug with the barge.
OVCD cannot compare those choices today.

## Solution

Four design choices describe the aft end: the **Stern Form**, the **Propulsor**, the **Power Source**
and, for a Formation, the **Tug**. Each is an input with a few parameters. From them the engine derives:

- the stern geometry (closed-form corrections now; sections from the same parameters once ADR 0012
  lands);
- the machinery space and the stern block, laid out from the installed power;
- the mass and centres of every stern-block item, in the lightship, with trim;
- the power: an actuator-disc propulsor model gives efficiency, thrust and bollard pull, and the Power
  Source gives the transmission losses and the installed power;
- the wheelhouse height that the sightline Rule needs, and the air draft with the wheelhouse lowered;
- for a convoy, tow or ATB, a sized tug and the power of the whole Formation, as one equivalent hull.

New Rules from ES-TRIN 2025/1, SOLAS V/22 and the Rhine police regulations, and new OVCD Checks, go
into the rules engine (spec 0004). Old files keep today's arrangement and today's numbers.

## User Stories

### Designer

1. As a designer, I want to pick a tunnel, pram or transom stern, so that the hull suits the water depth.
2. As a designer, I want to pick shafts in nozzles, azimuth thrusters, Voith Schneider units or pump
   jets, and how many, so that I can compare drives on one hull.
3. As a designer, I want to pick diesel-mechanical, diesel-electric, battery-hybrid, battery-electric
   or gas-fuelled gensets, so that the engine room, the mass and the fuel or energy follow the choice.
4. As a designer, I want OVCD to size the installed power for my design speed, or to give the speed for
   a power I type, so that I can see the trade.
5. As a designer, I want the stern block to weigh what it should and to trim the vessel, so that cargo,
   draught, propeller immersion and air draft are honest.
6. As a designer, I want the wheelhouse raised just enough to meet the sightline Rule and lowered for
   the air draft, so that I see both numbers.
7. As a designer of a convoy, I want OVCD to size the pusher and give the convoy's power and speed, so
   that I can choose the number of barges.
8. As a designer of an ATB, I want the barge notch and the tug sized together, so that the pair is
   consistent.
9. As a designer, I want to start from a published tug (a Reference Vessel) and change it, so that I
   begin from something real.
10. As a designer, I want the drawings to show the stern shape, the propellers, nozzles and rudders, the
    engine room and the raised wheelhouse, so that the GA looks like a vessel.

### Reviewer or expert

11. As a reviewer, I want every default to say whether it is sourced or an estimate, and why, so that I
    know what to check.
12. As a reviewer, I want the propulsor model's factors calibrated on published bollard pull and power,
    with held-out cases, so that I can judge its error.
13. As a reviewer, I want each visibility, speed and redundancy Rule cited to its paragraph and shown in
    the Rule Trace, so that I can check it against the text.

### Maintainer

14. As a maintainer, I want old files to give the same results hash, so that this work moves no saved
    design.
15. As a maintainer, I want one format bump shared with #274 and ADR 0012, so that users migrate once.

## Implementation Decisions

### Terms

New glossary terms (in `GLOSSARY.md`): **Stern Block**, **Stern Form**, **Propulsor**, **Power
Source**, **Installed Power**, **Design Speed**, **Bollard Pull**, **Tug**, **Articulated Tug-Barge
(ATB)**, **Notch**, **Equivalent Hull**, **Blind Distance**, **Fixed Height**.

### Modules (ovcd-core, all `no_std`)

Each module has one entry function and plain structs in and out, so a test drives it through its
interface. None parses JSON.

| Module | Interface | Holds |
|---|---|---|
| `stern_form` | `geometry(&SternFormInputs, hull: &HullDims) -> SternGeometry` | volume removed by the stern run, tunnels or notch at a draught; its LCB moment; extra wetted surface; the profile, plan and section outlines for the drawings; the largest propeller the stern allows |
| `propulsor` | `performance(&PropulsorInputs, speeds, resistance) -> PropulsorCurve` and `bollard_pull(&PropulsorInputs, power_kw) -> f64` | the actuator disc, the per-type factor, wake and thrust deduction, blade-load check |
| `machinery` | `arrange(&MachineryInputs) -> Machinery` | engine and drive selection by Power Source, the machinery-space zones and lengths, the mass items with centres |
| `aft_arrangement` | `compute` (as today) | deck items: accommodation, battery or genset units on deck, wheelhouse; it now takes its below-deck length from `machinery` |
| `powering` | `compute(&PoweringInputs) -> Powering` | joins resistance, propulsor and transmission: delivered, shaft and installed power, service speed per environment, fuel or energy |
| `formation` | `compute(&FormationInputs) -> FormationResult` | tug sizing, the Equivalent Hull, coupling force, the Formation's powering |
| `weights` | gains `MassItem { label, mass_t, lcg_m, kcg_m }` | lightship items with centres; the stern block items join the hull items |
| `hydrostatics` | gains `trim(&items, &hull, draught)` | trim, aft and forward draught for each loading condition |

`resistance` is unchanged: `powering` and `formation` call it. The legacy chain (`propulsive_efficiency`
times the resistance curve) stays the path of a file that names no Propulsor.

### Inputs (`.vdesign`, format version 3)

All new inputs go into format version 3 (ADR 0007), which #274 opens and the ADR 0012 hull block
shares. Whichever ticket lands first opens version 3; if a release has already shipped version 3 when a
ticket here lands, the builder stops and asks the maintainer. Each input has a **legacy value**: the
value that reproduces today's result exactly, given to a file that lacks the input (the table of fixed
values in `vdesign.rs`, marked `default`, one warning per Option). A session and `ovcd fill --as-new`
use the Defaults Profile value instead.

"By phase" means a default keyed by Cargo phase (`KEYED_DEFAULTS`), as the battery unit count is today:
gas is `liquid_gas`, dry bulk is `dry_bulk`. "None" means the engine works the value out by the rule
given.

#### Stern and propulsion (Formation `self_propelled`, and the tug of any Formation)

| Input | Choices or unit | Legacy value | Default (new Option) | Status and basis |
|---|---|---|---|---|
| `stern_form` | `transom`, `pram`, `tunnel` | `transom` | by phase: gas `transom`, dry bulk `tunnel` | estimate: tunnel sterns suit intermediate and very shallow water, pram sterns high h/T (Rotteveel 2019 §2.3.1); azimuth thrusters need no tunnel (MARIN 2020) |
| `stern_run_length_m` | m, or none | 0 | none: transom 1.0 × depth; pram 1.6 × depth; tunnel Heuser L_H | tunnel: sourced (Heuser in Rotteveel 2019 §2.3.1, by shaft arrangement); transom and pram: estimate (pram uses the bow rake ratio of today's profile) |
| `stern_transom_height_m` | m above base, or none | depth (a square stern) | none: transom and pram 1.0 × design draught; tunnel 0.0 (the run ends at the transom with the skirt) | estimate: transom clear of the design waterline |
| `tunnel_crown_height_m` | m above base, or none | — (tunnel only) | none: shaft axis + D/2 + 0.10·D tip clearance, with the tunnel radius = D | radius sourced (Heuser); tip clearance estimate |
| `propulsor_type` | `unspecified`, `azimuth_open`, `azimuth_ducted`, `shaft_open`, `shaft_nozzle`, `voith_schneider`, `pump_jet` | `unspecified` (today's: the fixed efficiency and the thruster layout) | by phase: gas `azimuth_ducted`, dry bulk `shaft_nozzle` | estimate: a Z-drive LNG tanker (Greenstream) and twin ducted Rhine Class Va ships (DST/VLIZ model, CDS 110 and 135 tankers) |
| `propulsor_count` | 1 to 4 | — (`unspecified` uses `thruster_count`) | 2 | estimate: twin drive on the reference cargo ships and gas tanker |
| `propeller_diameter_m` | m, or none | — | none: the largest the stern allows (below) | method |
| `power_source` | `battery_electric`, `gas_fuelled`, `diesel_mechanical`, `diesel_electric`, `battery_hybrid` | `battery_electric` | by phase: gas `gas_fuelled`, dry bulk `diesel_mechanical` | maintainer decision 2026-10-20 |
| `installed_power_kw` | kW, or none | — (no service speed in the app, as today) | none: sized for the Design Speed | method |
| `design_speed_kn` | kn | — | 9.7 (18 km/h) self-propelled; 7.5 pushed; 11 ATB; 7.0 towed | 9.7 sourced (CDS 110 and 135 tanker design speed 18 km/h); 7.5 estimate (ES-TRIN 5.06 13 km/h plus margin); 11 sourced (Aveogan ATB, 11 kn); 7.0 estimate |
| `installed_power_margin` | ratio | — | 1.20 | estimate: the value `electric_energy` already uses |
| `bow_thruster` | `none`, `channel_jet`, `tunnel` | `none` | `channel_jet` | sourced: ES-TRIN 31.03(1) asks for a bow thruster on a motor cargo vessel operating alone (crew standard S2); channel jets fit shallow bows (Beveridge 1971) |
| `bow_thruster_power_kw` | kW, or none | — | none: 0.11 kW per m³ of L·B·T | sourced (Hekkenberg 2013 p.222) |
| `battery_energy_kwh` | kWh, or none (battery-electric and battery-hybrid) | — (unit count as today) | none: energy per trip × 1.25 reserve, at 80 % usable | reserve and usable share estimate (LTO usable 90 % sourced, Echandia; NMC not found) |
| `battery_unit_energy_kwh` | kWh per 40 ft unit | — | 5,000 | estimate: 70 kWh/m³ room density (estimate) × 76 m³ inside a 40 ft high-cube unit (ISO internal size) |
| `wheelhouse_type` | `elevating`, `fixed` | `elevating` | `elevating` | as today |
| `wheelhouse_eye_height_m` | m (exists) | 1.50 | 1.65 | sourced: ES-TRIN 7.02(3) |
| `sightline_method` | `margin`, `blind_distance` | `margin` (today's check) | `blind_distance` | the Rule needs a distance, not a margin (ES-TRIN 7.02(2)) |
| `stern_block_mass_included` | bool | false | true | WC-A29 fix; old files keep 0 t |

The existing aft block inputs (`thruster_*`, unit counts and sizes, `wheelhouse_*`) stay. `thruster_count`
and `propulsive_efficiency` apply only when `propulsor_type` is `unspecified`; the schema says so with
the same applies-to mechanism the Formation uses. `thruster_envelope_m` gets a rule for the other
azimuth types: none = 1.35 × D (estimate: nozzle and housing around the disc).

#### Formation and Tug (Formations other than `self_propelled`)

New Formation kinds: `atb` (one barge with a Notch and one ATB tug) and `towed_barge` (barges on a
hawser behind a towing tug). The existing `pushed_barge` and `pushed_convoy` take a pusher. This makes
the `atb` and `towed_barge` Vessel Classes pickable (`vessel_classes.rs` `formation_kind`).

| Input | Choices or unit | Legacy value | Default (new Option) | Status and basis |
|---|---|---|---|---|
| `tug_included` | bool | false (the barge alone, as today) | true | maintainer decision 2026-10-20 |
| `tug_preset` | a Reference Vessel id, or none | — | none | a preset fills the tug inputs below, marked `reference` (a new source marker beside `typed`, `default` and `class`) |
| `tug_length_m` | m, or none | — | none: pusher 19.5 × (P/1,400 kW)^0.67; ATB 0.28 × barge length; towing 23 × (P/3,000 kW)^0.45 | estimates fitted to the reference pushers and tugs (research §6, §7.3, §10); ATB ratio is the middle of 0.24–0.32 (Crowley sheets, derived) |
| `tug_beam_m` | m, or none | — | none: pusher 9.5 × (P/1,400 kW)^0.45; ATB 0.53 × barge beam; towing L/2.5 | estimates, same basis; ATB middle of 0.49–0.57 |
| `tug_draught_m` | m, or none | — | none: pusher 2.0, capped by the draught limit; ATB B/2.33; towing 0.42 × B | pusher estimate (reference pushers 1.4–2.65 m); ATB sourced mean tug B/T 2.33 (Yanakiev 2025 §4.1); towing estimate (Damen sheets) |
| `tug_*` stern and propulsion | the inputs of the table above, prefixed `tug_` | — | pusher: `tunnel`, `shaft_nozzle`, 2 (3 above 2,500 kW), `diesel_mechanical`; ATB tug: `transom`, `azimuth_ducted`, 2, `diesel_mechanical`; towing tug: `transom`, `azimuth_ducted`, 2, `diesel_mechanical` | estimates from the reference pushers (twin and triple screw in nozzles), ATB tugs and ASD tugs |
| `formation_resistance_allowance` | ratio | — | by Formation kind: pushed 1.00, ATB 1.05, towed 1.00 | ATB sourced as "rigidly linked … 5 % increase in resistance compared to equivalent ships" (Yanakiev 2025 Table 3), applied to ATBs as an estimate; others estimate, set by calibration |
| `notch_depth_m` | m, or none (ATB) | — | none: 0.50 × tug length | estimate: one deep-notch ATB (Legacy, derived) |
| `notch_width_m` | m, or none (ATB) | — | none: tug beam + 2 × 0.15 m | estimate: coupler clearance |

`models.rs` `tug_length`, `tug_beam` and `notch_depth` (drawing-only today) are replaced by these.

### Stern geometry (`stern_form`)

- **Transom:** the stern rake of today's raked box (`hull.rs`, `stern_rake_length` and `stern_rake_rise`)
  with length = `stern_run_length_m` and rise = depth − `stern_transom_height_m`. With the legacy values
  (length 0) it is today's square stern.
- **Pram:** the bottom rises from the base at the forward end of the run to the transom height along a
  parabola; removed volume and its moment are closed-form (a parabolic segment) at any draught.
- **Tunnel:** a transom-height-0 run whose bottom stays flat; each propulsor sits in a tunnel whose
  section at the propeller plane is a circular segment of radius D (Heuser) with its crown at
  `tunnel_crown_height_m`. The tunnel section tapers linearly to zero at the forward end of the run. The
  removed volume is the tunnel section area below the waterline times half the run length, per tunnel. Twin shafts sit at ±0.225·B (parallel, Heuser 0.45·B spacing) or ±0.14·B
  (converging, 0.28·B), chosen by `propulsor_type` (shaft: parallel; azimuth: converging).
- **Notch (ATB barge):** a full-depth rectangular cut of `notch_depth_m` × `notch_width_m` at the
  centreline of the transom.
- **Displacement:** when `stern_form` is not legacy, the default block coefficient is no longer the flat
  0.85: it is the raked-box volume (bow rake and Stern Form) at the design draught over L·B·T, worked out
  by `stern_form` with today's bow rake. A typed block coefficient is used as typed. A stored value in
  an old file is used as stored, so old results do not move.
- **LCB and wetted surface:** from the same removed volumes; the tunnel adds its curved surface.
- **Largest propeller:** D_max = `k_D` × design draught, with `k_D` 0.80 under a tunnel (estimate: the
  research's inland data run 0.57–0.93) and 0.65 at an open transom or pram stern (sourced: MAN, seagoing,
  "d/D < approximately 0.65"). An azimuth unit below a transom: D_max from the same 0.65 rule. A typed D
  above D_max is used and fails the OVCD Check "propeller fits the stern".
- **The sections table** (ADR 0012) later replaces the closed forms with sections the generator writes
  from the same inputs. The closed forms stay as the test of the generator, as the raked box does.

### Propulsor model (`propulsor`)

- **Actuator disc.** For each propulsor at ship speed V: thrust T = R_total / (n × (1 − t)); advance
  speed V_A = V(1 − w); disc area A = πD²/4; thrust loading C_T = T / (½ρ A V_A²); ideal efficiency
  η_i = 2 / (1 + √(1 + C_T)). A ducted propulsor uses the ducted disc, in which the duct carries a share
  1 − τ of the thrust (τ = 0.75, estimate: duct lift "can be 30 %" of the propeller's thrust, Rotteveel
  §2.2.2), giving η_i = 2 / (1 + √(1 + τ·C_T)). The builder cites the derivation from a free text or marks
  the Measure method `unverified`.
- **Losses.** η_D = κ_type × η_i × η_R × η_H, with η_R 0.98 and η_H = (1 − t)/(1 − w) = 1.0 for twin
  screw (sourced, MAN), and κ_type the factor of the Propulsor type. Wake w: transom and pram 0.20, tunnel
  0.30 (estimates in the MAN range 0.20–0.45, below the DST tunnel nominal wake 0.457 at h/T 3), with
  t = w so that η_H = 1.
- **Bollard pull** (V = 0): T₀ = κ_b,type × (2ρA)^(1/3) × P_D^(2/3) per unit, the momentum limit of a
  disc; the duct's extra thrust is inside κ_b. With ρ = 1,025 kg/m³ and κ_b = 0.77 this is the Wärtsilä
  open-propeller estimate 0.9 × (D·P_D)^(2/3) kN (derived). Bollard pull in tonnes = n × T₀ / 9.81.
- **Factors κ_type and κ_b,type** are Defaults Profile values, status `estimate`, fitted in the calibration ticket so
  that the bollard pull per kW of the in-sample reference tugs falls in the research bands (κ_b), and
  that the installed power of the in-sample ships at their stated speed falls in its band (κ), with the
  rest held out:

  | Type | Band to meet (t per 100 kW, derived from reference vessels) |
  |---|---|
  | `shaft_nozzle` | 1.65–1.80 |
  | `azimuth_ducted` | 1.55–1.85 |
  | `azimuth_open`, `shaft_open` | 1.20–1.45 (formula only, no vessel) |
  | `voith_schneider` | 1.25–1.35 |
  | `pump_jet` | no band; κ estimate, flagged `unverified` |

  The ASD 2312 / ASD 2811 pair (same power, 2.8 m and 2.4 m propellers, 70 t and 60 t) must come out in
  the right order and within 10 %.
- **Blade load.** P_D per unit / A ≤ 400 kW/m² open, 500 kW/m² ducted (Veth rating basis): an OVCD Check,
  warn.
- **Pump jet.** A pump jet uses the same disc with D = its inlet diameter and its own κ. The inlet
  diameter is an input; its default comes from a table by power that the builder reads from the maker's
  size range (Schottel product guide p.21), each row `sourced`, or `estimate` where the guide gives none.
  The method is `unverified`.

### Power and transmission (`powering`)

- P_D at each speed from the propulsor curve. Shaft power P_S = P_D / η_S, η_S 0.99 for a shaft line
  (sourced, MAN) and 0.95 for an azimuth unit (estimate: two gear sets; MARIN says it is lower).
- Power Source losses, from shaft to the engine or the battery:
  - diesel-mechanical: gearbox 0.97 (estimate);
  - diesel-electric and gas-fuelled gensets: electric chain loss 15 % at 1 MW or less, 9.4 % at 5 MW,
    interpolated on log power and held flat outside (sourced, Wärtsilä article, a vendor source);
  - battery-electric: 0.92, today's drivetrain value (estimate);
  - battery-hybrid: diesel-mechanical in transit; the battery leg at 0.92.
- **Installed Power** P_B = P_needed at the Design Speed in deep water at the laden draught ×
  `installed_power_margin`, rounded up to the engine table (below), unless typed.
- **Service speed:** the highest speed where P_needed ≤ P_B / margin, per environment (deep water and the
  design channel), with the squat and channel caps of today's resistance method.
- **Fuel and energy:** diesel at today's 195 g/kWh on P_B load; battery energy per trip as
  `electric_energy` does today, now with the Propulsor's η_D; gas fuel stays on boil-off as today.
- The app shows installed power, bollard pull and service speed per environment, beside today's curves.

### Machinery space and mass (`machinery`)

- **Layout** (Hekkenberg 2013 §4.2.4, sourced as a method): the stern block length is the longest of
  (a) the below-deck machinery run, (b) the deck items and (c) the stern run where the propulsors sit.
  - Below deck, aft to forward: the aft peak with the steering gear (1.5 m, estimate), the drive room
    (azimuth: today's thruster room; shaft: the shaft run inside the stern), the engine room (A 1.0 m +
    engine + gearbox + C 0.8 m, A and C estimates), the genset or battery room when the Power Source
    has one, the gas valve unit room (gas-fuelled, as today).
  - Engines abreast, one per shaft; gensets abreast, N+1 (as today's gas rule).
  - Engine length from a sourced table (Cat 3512C and 3516C, Cummins QSK60, Everllence 12V/16V/20V
    175D; research §4.2): the first engine whose rating covers P_B per engine. Below 955 kW per engine
    the length is 2.59 m × (P / 955 kW)^(1/3) (estimate). Gearbox 1.2 m; with a PTI motor 2.8 m
    (sourced range, Reintjes).
  - Battery room (battery-electric below deck, battery-hybrid): volume = energy / 70 kWh/m³ (estimate),
    deck height of the hull, full beam less 1.0 m each side.
  - Deck units and the wheelhouse as today (`aft_arrangement`).
- **Mass items** (each with LCG at its zone centre and KCG as given):

  | Item | Mass | KCG | Status |
  |---|---|---|---|
  | Diesel engines | 4.9 kg/kW | double bottom + engine height / 2 | sourced (Hekkenberg) |
  | Gearboxes | 2 kg/kW | shaft axis | sourced |
  | Gensets (diesel-electric) | 9 kg/kVA, kVA = kW / 0.8 | as engines | sourced; power factor estimate |
  | Gas gensets | today's published engine mass + alternator 9 kg/kVA less the engine share | as engines | engine sourced (maker table); rest estimate |
  | Switchboards, converters | 1 kg/kVA (≥ 0.3 t); 0.9 t + 2 kg/kVA | deck − 1 m | sourced |
  | Electric motors | 6 kg/kW | shaft axis | sourced |
  | PTI motor and converter | 8 kg/kWe | shaft axis | derived (Reintjes) |
  | Ducted azimuth unit | 1.5 × D³ t per unit | base + D | sourced (Hekkenberg) |
  | Shaft line, open azimuth, Voith Schneider, pump jet | VSP 22 kg/kW; pump jet by the sourced formula; open azimuth 1.5 × D³; shaft line 1.0 t per m of shaft (estimate) | shaft axis | VSP derived (Voith release); pump jet sourced; shaft estimate |
  | Rudders and steering gear | 1 t per m of D per rudder; 0.5 t per twin set | base + D | sourced; two rudders per shaft (sourced, Class Va practice) |
  | Bow thruster | 5.4 kg/kW | base + 1 m | sourced |
  | Small items, piping, cabling | 1 kg/kW; 3 t + 2.5 kg/kW | double bottom | sourced |
  | Batteries | 6 kg/kWh installed | room or unit centre | sourced (Corvus string 5.96) |
  | Accommodation units | 0.17 t/m³ of unit volume | unit centre | sourced |
  | Wheelhouse; lift column | 8 t; 5 t | lowered centre; deck | sourced (one ship) |

- The items go into the lightship when `stern_block_mass_included` is true. The lightship box factor k
  was fitted on pushed barges with no stern block (page 02), so the hull steel is not counted twice.
- A tug's lightship is its own: hull steel by today's box rate on its own L·B·D, plus its machinery.

### Trim (`hydrostatics`)

- LCG of every lightship item (hull steel at the centre of its own volume with the Stern Form removed,
  tanks and holds at their zones, the stern block items as above) and of the cargo.
- For each loading condition: trim = Δ × (LCG − LCB) / MTC, with MTC from the waterplane's longitudinal
  inertia (box with the stern and bow cut, closed form), then the aft and forward draughts.
- Results gain `trim_m`, `draught_aft_m` and `draught_fwd_m` per condition only when
  `stern_block_mass_included` is true. A file without it gets no new field, so its hash is unchanged.
- The propeller immersion check and the sightline use the aft draught and the trimmed waterline.

### Wheelhouse, sightline and air draft

- **Blind Distance** (a Measure, per loading condition): the eye at floor + `wheelhouse_eye_height_m`
  with the wheelhouse raised to the height the Rule needs (up to the maximum lift); every obstruction
  forward of it (deck units, cargo top, tank domes, coaming, the bow; for a pusher, the convoy's barges
  and their cargo) as a height and a forward edge; the trimmed waterline. Blind Distance =
  max over obstructions of x_o · z_e / (z_e − z_o) − x_b (research §9.2, derived).
- **Lift needed:** the smallest lift that brings the Blind Distance within the governing limit of the
  selected sets; the result reports it, the raised eye and floor, and whether the maximum lift is
  enough.
- **Fixed Height and air draft:** with the wheelhouse lowered (ES-TRIN 1.01, 4.29), as today.
- `sightline_method: margin` (legacy) keeps today's warning check exactly. With `blind_distance` the
  Rules below apply.
- **`fit` blocks on the sightline** for every file, as the maintainer decided on #264. This is a
  tracked design change (design-impact table and changelog): a candidate that fails is not feasible.

### Formations (`formation`)

- **Tug sizing:** the Formation's resistance at its Design Speed sets the tug's P_B (with the margin);
  P_B sets the tug's dimensions by the ratios above unless typed or preset; the tug gets its own stern
  block, machinery and lightship. The loop runs at most five times and stops when P_B moves less than
  1 % (the tug's own resistance is part of the Formation's).
- **Equivalent Hull:**
  - pushed: length = barges in line × barge length + tug length; beam = barges abreast × barge beam;
    draught = barge draught; block coefficient = (Σ displaced volumes) / (L·B·T);
  - ATB: length = barge length + tug length − notch depth; beam and draught of the barge;
  - towed: the barges and the tug as separate hulls, resistances added.
  The resistance of the Equivalent Hull, times `formation_resistance_allowance`, is the Formation's.
  The method is `unverified`.
- **Thrust** comes from the tug's propulsor model, with the tug's own draught and diameter.
- **Coupling force** (pushed): F_SB = 270 × P_B × L_S / B_S × 10⁻³ kN (ES-TRIN ESI-III-6 §2.1), reported
  as a note.
- **Convoy totals:** cargo, lightship and deadweight of every barge, and the tug's; the convoy result
  replaces today's "convoy results not yet available".
- **Presets:** Reference Vessel tugs from `calibration/` (pushers, tugs and ATB tugs with power,
  dimensions and propulsor on a stated basis) are listed in the catalogue; a preset fills the tug inputs
  marked `reference`.

### Rules (spec 0004)

New Rule Sets, each a draft until its Rules are verified by a second reader (`docs/rules/codification.md`):

| Set | Default in | Rules |
|---|---|---|
| `es-trin-2025` (family `es-trin`, ES-TRIN 2025/1) | `river-only` and `coastal-and-river` | `es-trin/7.02(2)/blind-zone-unladen`: Blind Distance (light condition) ≤ 250 m, block. `es-trin/5.06(1)/speed-13kmh`: speed through water at installed power, deep water, laden ≥ 13 km/h, block; not for a pusher alone. `es-trin/28.04(1)(a)/propulsion-redundancy`: a check Rule — length ≤ 110 m, or two independent engines of equal power and a bow thruster, or one screw and a bow thruster with its own power; block. `es-trin/7.02(3)/field-of-view` and `es-trin/7.12(2)/elevating-wheelhouse-stability`: `evaluated: false`, gaps "no window geometry" and "no raised-wheelhouse stability case". `es-trin/ESI-III-6.2.1/coupling-force`: note. |
| `solas-v22` (family `solas`, SOLAS V/22, MSC.99(73)) | `coastal-and-river` | `solas/V.22.1.1/blind-zone`: Blind Distance (every loading condition) ≤ min(2 L, 500 m), a formula limit. The rule applies to ships of 45 m and more; spec 0004 cannot express a numeric Applicability yet, so the Rule is `warn` for every length until Applicability Rules exist, with a finding. The ±10° side check is a gap. |
| `rheinschpv` (family `rheinschpv`, Rhine Police Regulations §1.07) | none (optional) | `rheinschpv/1.07(2)/blind-zone-laden`: Blind Distance (laden) ≤ 350 m, block when selected. |
| `abs-barges` notes for ATB (family `abs-barges`, 2024) | `coastal-and-river`, ATB only | `abs-barges/5-3-1.1.3/two-vessels` and `abs-barges/5-3-1.11.3/notch-wing-walls`: notes (approval items). |

OVCD Checks (set `ovcd`):

| Rule id | Measure and limit | Consequence | Origin |
|---|---|---|---|
| `ovcd/propulsion/propeller-fits-stern` | D ≤ D_max of the Stern Form | warn | method-validity |
| `ovcd/propulsion/immersion-light` | shaft-axis submergence at the light aft draught ≥ 0.76·D open, 0.71·D ducted; not for a tunnel stern | warn | method-validity (Beveridge 1971, a guide) |
| `ovcd/propulsion/blade-load` | P_D per disc area ≤ 400 kW/m² open, 500 ducted | warn | method-validity (Veth rating basis) |
| `ovcd/powering/installed-covers-design-speed` | P_B ≥ power needed at the Design Speed | block | physical |
| `ovcd/powering/shallow-water-validity` | h/T ≥ 2.0 in the design channel | warn | method-validity (Zhang 2023, Raven limit) |
| `ovcd/machinery/fits-under-deck` | engine or genset height + 1.0 m ≤ depth (today's genset check, all engines) | warn | physical |
| `ovcd/machinery/fits-beam` | engines or gensets abreast fit the beam | block | physical |
| `ovcd/formation/tug-proportions` | tug ratios inside the calibrated range | warn | method-validity |
| `ovcd/wheelhouse/lift-enough` | lift needed ≤ maximum lift | block | physical |

The existing aft-block checks become `ovcd` Rules in #272 as planned.

**Measures** added to the registry (#269): `wheelhouse.blind_distance(condition)`,
`powering.speed_through_water_kmh(condition, environment)`, `propulsion.redundancy_28_04` (bool),
`propulsion.diameter_over_max`, `propulsion.axis_submergence_ratio(condition)`, `propulsion.blade_load`,
`powering.installed_over_needed`, `formation.coupling_force`, `machinery.fits_under_deck` and the rest
named above. Their methods are `unverified` until the methods are read and checked (the actuator disc,
the Equivalent Hull, the Blind Distance geometry).

**Applicability** gains `formation_kind`, `propulsor_type`, `stern_form` and `power_source` as
enumerated inputs (spec 0004 allows any enumerated input the schema names).

### Calibration and evaluation

- `calibration/propulsion_cases.yaml`: the 24 vessels of research §10, each with its source file and
  hash, and the published figures: bollard pull (tugs and pushers), installed power at a stated design
  speed (inland ships, ATBs), pusher power and convoy size. Held-out cases are marked, as
  `tank_cases.yaml` does.
- Bands in `calibration/bands.yaml`: bollard pull ±10/15/20 %, installed power ±15/20/30 %, at
  high/medium/low confidence (estimate; the maintainer may change them).
- The fit writes κ_type and the Formation allowances to `calibration/factors.yaml`; the Defaults Profile
  takes them with status `estimate` and the fit as basis.
- Evaluation harness (#278): each new Rule gets cases just inside and just outside its limit (Blind
  Distance at 249 and 251 m; speed at 12.9 and 13.1 km/h; a 110.1 m single-screw vessel with and without
  a bow thruster), keyed by rule id; mutation of the rule data (#279) must flip them.
- The design-impact check must show no change for every existing fixture, except the #264 `fit` change,
  which is listed.

### Drawings (`ovcd-format` `ga.rs`, `ovcd-draw`)

- **Side elevation:** the stern outline of the Stern Form (rake, pram curve, tunnel crown as a hidden
  line with the skirt); propellers as a disc outline with the shaft line (centre line) to the gearbox;
  nozzles as a section around the disc; rudders behind each disc; azimuth units as leg and pod; the
  engines and gensets as hidden rectangles in the engine room; the raised wheelhouse as a thin chain
  outline marked "RAISED"; the governing sightline as a thin chain ray from the eye over the obstruction
  to the water, labelled with the Blind Distance; the Fixed Height dimension.
- **Plan:** shafts, propellers and rudders; tunnel outlines (hidden); engine footprints (hidden); the bow
  thruster; the ATB Notch.
- **Stern section** (new view `stern-section`): a transverse section at the propeller plane, looking
  forward: the hull section with the tunnels, the propeller discs and nozzles, the design and light
  waterlines, and the shaft-axis submergence dimension.
- **Formation arrangement plan** (new view `arrangement-plan`, for convoys, tows and ATBs): the barges and
  the tug in plan with overall dimensions and coupling points, and the profile sightline over the convoy.
  The tug gets its own GA (it is a Hull).
- New symbols (propeller, nozzle, rudder, azimuth unit, Voith Schneider unit, pump-jet inlet, coupler)
  go into the Drawing Format Standard symbol library first (ADR 0009), then into `ovcd-draw`.
- Labels go into the Text Library (spec 0005, #310).

### App

- Two new input groups: "Stern and propulsion" and "Tug". Inputs show their Defaults Profile status.
- Results show installed power, bollard pull, service speed per environment, the machinery space, the
  stern block mass, trim, the wheelhouse lift needed and the Fixed Height; for a Formation, the tug and
  the convoy totals.

### Messages

New check names, warnings and notes carry Message Ids beside the English, outside the results hash, as
#309 built them (spec 0005), and their English goes into the Text Library. Drawing labels follow #310;
the input labels and choices follow the app UI strings of #308. The catalogue check (#321) must pass.

## Testing Decisions

- **Old files:** every v1 and v2 fixture opens with its old results hash and one warning per Option for
  the new inputs (the #264 `fit` change aside, which the design-impact table lists).
- **Modules through their interfaces:** `stern_form` against a numerical integral of the same shape
  (volume, LCB) at several draughts; the legacy Stern Form against today's raked box to the bit;
  `propulsor` against the Wärtsilä bollard-pull estimate for open propellers, and C_T → 0 giving η_i → 1;
  `machinery` mass sums against hand sums of the sourced rates; trim against the legacy
  `calculate_trim` for a box.
- **Calibration** as above, with held-out cases reported in the PR body.
- **Rules:** evaluation cases per rule id; the loader refuses each new set file with a broken field.
- **Drawings:** the conformance tests of spec 0003 for the new entities and views.
- `ovcd-core` stays `no_std`; the WebAssembly build runs in the gate.

## Out of Scope

- Mechanical LNG dual-fuel engines (no free data below 2 MW).
- The Zhang et al. (2023) shallow-water convoy method (a later ticket, filed).
- Open-water charts (Wageningen B-series, Ka nozzles); wake from model tests.
- Tug stability (IS Code Part B 2.8, 46 CFR 173.095), coupler loads and sea-state limits.
- Approval items: notch wing-wall analysis, connector lifetime loads (notes only).
- Ocean-only service (ADR 0011).
- Damage stability of the stern block spaces.

## Decisions taken and open questions

Taken with the maintainer, 2026-10-20:

1. All three waves (self-propelled stern, pusher and convoy, ATB) are in 0.2.x.
2. The stern is parametric now; the ADR 0012 sections generator reuses the same parameters.
3. Power from an actuator-disc model with one calibrated factor per Propulsor type.
4. Convoys and ATBs use one Equivalent Hull with the existing resistance method; Zhang 2023 later.
5. The tug is sized from the power its Formation needs, with Reference Vessel presets.
6. Power Sources: battery-electric and gas-fuelled (kept), diesel-mechanical, diesel-electric and
   battery-hybrid (new). Mechanical LNG waits.
7. A new Option's defaults go by Cargo: gas gets gas-fuelled gensets and ducted azimuth thrusters, dry
   bulk gets diesel-mechanical twin shafts in nozzles under a tunnel stern.
8. LCG and trim are added.

Open (for the build, not blocking):

- The SOLAS V/22 45 m threshold waits for numeric Applicability (spec 0004 out of scope); the Rule is
  `warn` until then.
- The calibration bands are estimates; the maintainer may change them in `bands.yaml`.
- Engine-room length has no built-ship case; measuring scaled general arrangements is a follow-up.

## Build tickets

Filed after this spec is agreed; the list and order are in the parent issue. Wave 1 is the
self-propelled stern, wave 2 the pusher, convoy and tow, wave 3 the ATB.
