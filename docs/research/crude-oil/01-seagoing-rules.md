# 01 — Sea-going and coastal rules for crude oil tankers and tank barges (500–10,000 dwt)

Scope: rules that set concept-design limits for small crude oil tankers (coasters) and sea-going or
coastal tank barges, including tug-barge units. Date of research: 2026-10-20.

Source files are in `~/Dev/ovcd-rule-sources/crude-oil/seagoing/` and logged in
`~/Dev/ovcd-rule-sources/crude-oil/SOURCES.md`. Some texts were already in the library
(`../icll/`, `../is-code-2008/`, `../abs-barges/`, `../type-c/class/`).

## How to read this file

- **Rule id**: `<family>/<paragraph>/<slug>`.
- **Read**: "read" = I read the primary text in the file named. "secondary" = I saw it only in a
  summary or reproduction. "not verified" = from memory or a source I could not open.
- **Compute**: "yes" = OVCD can compute it from principal dimensions, tank layout, weights and
  hydrostatics. "partial" = OVCD can compute a proxy or a check, but the full rule needs detail
  that a concept tool does not have. "no" = arrangement, equipment or operation; record as a flag.
- **Edition basis**: MARPOL Annex I = revised Annex I, Res. MEPC.117(52) (adopted 15 Oct 2004, in
  force 1 Jan 2007), plus MEPC.141(54) (2006), MEPC.248(66) (2014) and MEPC.330(76) (2021). The
  consolidated 2022 edition is paywalled. I did not collect every later amendment. Treat clause
  numbers as MEPC.117(52) numbering, with the MEPC.248(66) renumbering of 28.6 → 28.7 applied.
- **Delivery dates**: every rule below assumes a **new ship** (delivered after 1 Jan 2010,
  constructed after 1 Jan 2016). Older-ship branches are left out.

## Key applicability facts (read these first)

1. **An oil tank barge is an oil tanker under MARPOL Annex I.** Reg. 1.5 defines an oil tanker as
   "a ship constructed or adapted primarily to carry oil in bulk". Nothing in Annex I excludes
   non-self-propelled ships. Reg. 28.5 names "the person in charge of a non-self-propelled oil
   tanker". (read: MEPC.117(52))
2. **The UNSP barge exemption does not cover oil barges.** MEPC.330(76) Reg. 1.40.2 defines an
   unmanned non-self-propelled barge as one that "carries no oil". A crude barge cannot use it. (read)
3. **SOLAS does not apply to non-self-propelled ships or to cargo ships below 500 GT.** SOLAS Ch I
   Reg. 3(a). (secondary — consolidated SOLAS is paywalled.) A crude barge gets its fire and
   arrangement rules from class and flag (for example ABS 5-2-3, 46 CFR 32), which copy the SOLAS
   tanker layout rules. A self-propelled coaster of 500 GT and above on international voyages gets
   SOLAS II-2 in full.
4. **SOLAS II-2 "tanker" rules apply to crude** (flashpoint ≤ 60 °C, Reid vapour pressure below
   atmospheric). SOLAS II-2/1.6.1 (read: MSC.99(73)).
5. **Load lines apply to barges.** ICLL Reg. 2(1) covers "lighters, barges or other ships without
   independent means of propulsion" (read: MSC.143(77)). The 1966 Convention applies to ships of
   24 m and above on international voyages (Art. 5, not verified).
6. **Reg. 26 does not apply to new tankers.** Reg. 26.7: not for tankers delivered on or after
   1 Jan 2010. Reg. 23 replaces it. (read)
7. **Crude oil above 900 kg/m³ at 15 °C is "heavy grade oil"** (Reg. 21.2.1). On a tanker of 600–
   5,000 dwt it removes the single-side option of Reg. 19.6 (see `marpol-i/21.4.2`). (read)

## A. MARPOL Annex I — construction (Chapter 4)

### marpol-i/19.1/600-dwt-threshold
- **Citation**: MARPOL Annex I Reg. 19.1, MEPC.117(52).
- **Requires**: Reg. 19 (double hull / double bottom) applies to oil tankers of **≥ 600 dwt**
  delivered on or after 6 July 1996.
- **Applicability**: all oil tankers ≥ 600 dwt, self-propelled or not, crude and product.
- **Compute**: yes (dwt switch).
- **Read**: read.

### marpol-i/19.3.1/wing-tank-width (≥ 5,000 dwt)
- **Citation**: Reg. 19.2, 19.3.1.
- **Requires**: wing tanks or spaces over the full cargo tank length, full depth of side (or from
  double-bottom top to uppermost deck). Cargo tanks inboard of the side shell by at least
  `w = min(0.5 + DW/20,000, 2.0)` m, with `w ≥ 1.0` m. Measured at right angles to the side shell.
- **Applicability**: oil tankers ≥ 5,000 dwt. In the 5,000–10,000 dwt band, `w` = 0.75–1.0 m, so
  the **1.0 m minimum governs** across the whole band.
- **Compute**: yes.
- **Read**: read.

### marpol-i/19.3.2/double-bottom-height (≥ 5,000 dwt)
- **Citation**: Reg. 19.3.2.
- **Requires**: `h = min(B/15, 2.0)` m, with `h ≥ 1.0` m, from cargo tank bottom to moulded bottom
  shell, at right angles to the bottom shell. For B = 15–30 m, `h` = 1.0–2.0 m.
- **Applicability**: oil tankers ≥ 5,000 dwt.
- **Compute**: yes.
- **Read**: read.

### marpol-i/19.3.3/bilge-transition
- **Citation**: Reg. 19.3.3; UI 42 (Deutsche Flagge reproduction of MARPOL UIs).
- **Requires**: where `h ≠ w`, distance `w` governs above `1.5h` from the baseline. UI 42: applies
  over the entire tank length.
- **Applicability**: ≥ 5,000 dwt.
- **Compute**: yes (section geometry).
- **Read**: read (Reg.); UI read in flag-state reproduction.

### marpol-i/19.3.4/ballast-capacity-in-double-hull
- **Citation**: Reg. 19.3.4; UI 43.
- **Requires**: for crude tankers ≥ 20,000 dwt only, wing + double-bottom + peak tanks hold at
  least the Reg. 18 segregated ballast. Not binding below 20,000 dwt.
- **Applicability**: crude ≥ 20,000 dwt; product ≥ 30,000 dwt. **Not in OVCD's size range.**
- **Compute**: yes, but not needed in range.
- **Read**: read.

### marpol-i/19.3.5/suction-well-clearance
- **Citation**: Reg. 19.3.5.
- **Requires**: suction wells may enter the double bottom if well bottom to shell ≥ `0.5h`.
- **Compute**: yes (check).
- **Read**: read.

### marpol-i/19.3.6/piping-segregation
- **Citation**: Reg. 19.3.6.
- **Requires**: ballast piping must not pass through cargo tanks; cargo piping must not pass
  through ballast tanks (short fully welded lengths excepted).
- **Compute**: no (flag).
- **Read**: read.

### marpol-i/19.4/mid-deck-alternative
- **Citation**: Reg. 19.4.1–19.4.3.
- **Requires**: double bottom may be omitted if `1.1·hc·ρc·g + p ≤ dn·ρs·g` (hc = cargo height on
  the bottom shell, m; ρc = max cargo density, kg/m³; p = P/V valve set pressure, Pa; dn = minimum
  operating draught, m; ρs = seawater density, kg/m³; g = 9.81 m/s²). Any horizontal partition at
  ≥ `min(B/6, 6 m)` and ≤ `0.6D` above base. Wing tanks still per 19.3.1.
- **Applicability**: ≥ 5,000 dwt (alternative to 19.3).
- **Compute**: yes. Note: for shallow-draught coastal ships `dn` is small, so this is rarely viable.
- **Read**: read.

### marpol-i/19.6.1/small-tanker-double-bottom (600–5,000 dwt)
- **Citation**: Reg. 19.6.1.
- **Requires**: either full 19.3 + 19.4 compliance, **or** at least a double bottom with
  `h = B/15` m, `h ≥ 0.76` m. No 2.0 m cap is stated for this branch. In the bilge area the cargo
  tank boundary runs parallel to the midship flat bottom (figure 3).
- **Applicability**: oil tankers 600 to < 5,000 dwt.
- **Compute**: yes.
- **Read**: read.

### marpol-i/19.6.2/small-tanker-700m3-or-wing-tanks (600–5,000 dwt)
- **Citation**: Reg. 19.6.2.
- **Requires**: with the 19.6.1 double bottom, **either** every cargo tank ≤ **700 m³**, **or** wing
  tanks per 19.3.1 with `w = 0.4 + 2.4·DW/20,000` m, `w ≥ 0.76` m. For 600–5,000 dwt this gives
  w = 0.47–1.0 m → 0.76 m governs up to 3,000 dwt, then rises to 1.0 m at 5,000 dwt.
  UI 44: wing tanks used for this rule may carry oil (not heavy grade oil) if each tank ≤ 700 m³.
- **Applicability**: oil tankers 600 to < 5,000 dwt. **Crude > 900 kg/m³ → see 21.4.2: wing tanks
  are then mandatory.**
- **Compute**: yes (tank volume check or w check).
- **Read**: read (Reg.); UI read in flag-state reproduction.

### marpol-i/21.4.2/heavy-crude-double-hull (600–5,000 dwt)
- **Citation**: Reg. 21.1, 21.2.1, 21.4.2.
- **Requires**: a tanker 600–5,000 dwt carrying heavy grade oil (crude with ρ15 > 900 kg/m³) must
  have **both** the 19.6.1 double bottom **and** wing tanks per 19.3.1 with `w` per 19.6.2. The
  700 m³ option is closed.
- **Applicability**: 600–5,000 dwt, crude with ρ15 > 900 kg/m³ (also HFO, bitumen).
- **Compute**: yes (cargo density switch).
- **Read**: read.

### marpol-i/19.7/no-oil-forward-of-collision-bulkhead
- **Citation**: Reg. 19.7.
- **Requires**: no oil forward of the SOLAS II-1/11 collision bulkhead. A ship that needs no
  collision bulkhead (for example a barge) uses a plane placed **as if** it were one. SOLAS II-1/11
  position is 0.05L–0.08L (or 0.05L+3 m) from FP for cargo ships (not verified, paywalled).
- **Applicability**: all Reg. 19 tankers, barges included.
- **Compute**: yes (fore end of cargo block).
- **Read**: read (Reg. 19.7); collision-bulkhead position not verified.

### marpol-i/ui-41/independent-tanks-count-as-double-hull
- **Citation**: MARPOL I Unified Interpretation 41 (to Reg. 19).
- **Requires**: oil tankers with independent tanks are double-hull tankers if the tank boundaries
  keep the Reg. 19 distances to bottom and side shell.
- **Compute**: yes.
- **Read**: read (flag-state reproduction of IMO UIs, not IMO original).

### marpol-i/22/pump-room-double-bottom
- **Citation**: Reg. 22 (in force 1 Jan 2007).
- **Requires**: pump-room double bottom with `h = min(B/15, 2)` m, `h ≥ 1` m, above baseline.
  Not needed if the pump-room floor is already that high (e.g. gondola stern), or if flooding the
  pump-room does not disable ballast or cargo pumping.
- **Applicability**: oil tankers ≥ 5,000 dwt constructed on or after 1 Jan 2007.
- **Compute**: yes (if OVCD models a pump-room); most small tankers use deepwell pumps and no
  pump-room.
- **Read**: read.

### marpol-i/23.3.1/mean-oil-outflow (≥ 5,000 dwt)
- **Citation**: Reg. 23.1–23.10; MEPC.122(52) explanatory notes.
- **Requires**: `OM = (0.4·OMS + 0.6·OMB)/C ≤ 0.015` for C ≤ 200,000 m³ (C = cargo volume at 98 %).
  `OMS = C3·Σ PS(i)·OS(i)`; `OMB = 0.7·OMB(0) + 0.3·OMB(−2.5 m tide)`;
  `OMB(t) = Σ PB(i)·OB(i)·CDB(i)`. PS and PB come from the tabulated damage probabilities
  (Reg. 23.8.3, 23.9.3) and closed-form `PSy`, `PBz`. `OB(i)` from hydrostatic balance:
  `hc = ((ds + tc − Zl)·ρs − 1000p/g)/ρn`, with `ρn = 1000·DWT/C`, p ≥ 5 kPa if inert gas fitted,
  OB ≥ 1 % of tank volume if bounded by the bottom shell. `CDB` = 0.6 over a non-oil space, 1.0 on
  the shell. Permeability 0.99. Piping < 0.30Bs from side or < 0.30Ds from bottom needs valves at
  the tank (23.11.1).
- **Applicability**: oil tankers ≥ 5,000 dwt delivered on or after 1 Jan 2010, barges included
  (MEPC.122(52) Part C uses a tank barge as its worked example).
- **Compute**: **yes** — fully defined by tank box coordinates (Xa, Xf, Zl, Zu, y, Yp, Ys, z), L,
  Bs, BB, Ds, ds, DWT, C. Good candidate for an OVCD module.
- **Read**: read (full tables are in MEPC.117(52) pp. on Reg. 23).

### marpol-i/23.3.2/small-tanker-tank-length (< 5,000 dwt)
- **Citation**: Reg. 23.3.2; MEPC.122(52) Part B §3.
- **Requires**: each cargo tank length ≤ max(10 m, value below):
  - no longitudinal bulkhead: `(0.5·bi/B + 0.1)·L`, max `0.2L`;
  - centreline bulkhead: `(0.25·bi/B + 0.15)·L`;
  - two or more longitudinal bulkheads: wing tanks `0.2L`; centre tanks `0.2L` if bi/B ≥ 0.2,
    else as the first two cases.
  - `bi` = min distance from side to the tank's outer longitudinal bulkhead at summer freeboard.
  MEPC.122(52) §3.1: for < 5,000 dwt, tank size is governed by the **700 m³** limit of 19.6.2 and
  this length limit.
- **Applicability**: oil tankers < 5,000 dwt delivered on or after 1 Jan 2010.
- **Compute**: yes.
- **Read**: read.

### marpol-i/26/tank-size-limits-old-ships
- **Citation**: Reg. 26.2–26.4, 26.7.
- **Requires**: hypothetical outflow ≤ max(30,000 m³, 400·∛DW), ≤ 40,000 m³, plus tank length
  limits (same formulas as 23.3.2). **Does not apply to tankers delivered on or after 1 Jan 2010.**
- **Compute**: not needed for new designs.
- **Read**: read.

### marpol-i/18.2/segregated-ballast-draught
- **Citation**: Reg. 18.1, 18.2, 18.5.
- **Requires**: SBT so that in every ballast condition `dm ≥ 2.0 + 0.02L` m, trim by stern
  ≤ 0.015L, propeller fully immersed. For L < 150 m: "to the satisfaction of the Administration".
- **Applicability**: **mandatory only for crude tankers ≥ 20,000 dwt** (product ≥ 30,000 dwt).
  Not mandatory in OVCD's range. Reg. 18.9 lets a smaller tanker qualify as SBT voluntarily; that
  earns the slop-tank reduction in 29.2.3.2.
- **Compute**: yes (ballast-condition draught and trim check). Useful as a design target even
  where not mandatory.
- **Read**: read.

### marpol-i/18.13/protective-location-PL
- **Citation**: Reg. 18.12–18.15.
- **Requires**: ΣPAc + ΣPAs ≥ J·Lt·(B + 2D). Applies only to ≥ 20,000 dwt crude tankers that do
  not meet Reg. 19. **Out of range and superseded by Reg. 19.**
- **Read**: read.

### marpol-i/27.1/intact-stability-all-conditions
- **Citation**: Reg. 27.1–27.3; UI 55.1 and 55.2.
- **Requires**: for any operating draught, worst cargo/ballast loading including **intermediate
  stages of liquid transfer**, ballast tanks assumed slack:
  - in port: GM0 (free-surface corrected, at 0°) ≥ 0.15 m;
  - at sea: area 0–30° ≥ 0.055 m·rad; area 0–40° (or 0–θf) ≥ 0.09 m·rad; area 30–40° (or 30–θf)
    ≥ 0.03 m·rad; GZ ≥ 0.20 m at ≥ 30°; GZmax at > 30° preferably, ≥ 25°; GM0 ≥ 0.15 m.
  - Must be met **by design** (27.2); operational procedures only for combination carriers.
  - UI 55.1 gives two proof methods: (.1) all cargo tanks at the fill level that maximises
    (vertical moment of volume + free-surface inertia at 0°), density from available cargo dwt at
    the displacement of minimum KM, full consumables, 1 % ballast, max ballast free surface; or
    (.2) systematic sweep: draught light ballast → scantling, consumables 97/50/10 %, cargo/ballast
    in ≥ 20 steps from 1 % to 99 %, densities lowest → highest, no slack-tank limits allowed.
- **Applicability**: oil tankers **≥ 5,000 dwt** delivered on or after 1 Feb 2002. Below 5,000 dwt
  the general IS Code criteria apply (see `is-code/A2.2`); class may still apply 27 (ABS barge rules
  3-6-1 apply the liquid-transfer appendix to all tank barges).
- **Compute**: **yes** — UI 55.1.1 is a closed algorithm OVCD can run from tank geometry and
  hydrostatics. This is the binding constraint on beam and on wide centre tanks (no centreline
  bulkhead → large free surface).
- **Read**: read (Reg.); UI read in flag-state reproduction.

### marpol-i/28.2/damage-stability-extents
- **Citation**: Reg. 28.1–28.4, 28.7 (ex 28.6).
- **Requires**: survive side and bottom damage in all actual partial and full load conditions
  (ballast-only excluded):
  - side: longitudinal `min(L^(2/3)/3, 14.5 m)`; transverse `min(B/5, 11.5 m)`; vertical from
    baseline up without limit;
  - bottom, 0.3L from FP: `min(L^(2/3)/3, 14.5 m)` × `min(B/6, 10 m)` × `min(B/15, 6 m)`;
  - bottom, elsewhere: `min(L^(2/3)/3, 5 m)` × `min(B/6, 5 m)` × `min(B/15, 6 m)`;
  - **L ≤ 150 m: damage between adjacent transverse bulkheads only, except the machinery space**
    (one-compartment standard; bulkheads spaced closer than the side-damage length count as
    absent). For L ≤ 100 m, the Administration may relax.
  - raking damage (28.7): only ≥ 20,000 dwt — out of range.
  - permeabilities: stores 0.60, accommodation 0.95, machinery 0.85, voids 0.95, liquids 0–0.95;
    free surface at 5° heel.
- **Applicability**: oil tankers ≥ **150 GT**, barges included (28.5 names non-self-propelled
  tankers).
- **Compute**: yes, given a compartment model.
- **Read**: read.

### marpol-i/28.3/damage-survival-criteria
- **Citation**: Reg. 28.3.
- **Requires**: final waterline below progressive-flooding openings (air pipes, weathertight
  closures); heel ≤ 25° (≤ 30° if no deck-edge immersion); residual GZ range ≥ 20° beyond
  equilibrium, GZmax ≥ 0.1 m within that range, area ≥ 0.0175 m·rad; intermediate stages to the
  Administration's satisfaction; no credit for cross-flooding needing valves.
- **Compute**: yes.
- **Read**: read.

### marpol-i/28.6/stability-instrument
- **Citation**: Reg. 28.6 (inserted by MEPC.248(66), in force 1 Jan 2016); waiver Reg. 3.6.
- **Requires**: every oil tanker carries a stability instrument that checks intact and damage
  stability. Waiver possible for dedicated service with few load cases, remote verification,
  approved loading range, or limiting KG/GM curves.
- **Applicability**: all oil tankers (no size floor stated in 28.6). ABS barge rules 3-6-1/7 apply
  the same to tank barges.
- **Compute**: no (equipment flag). OVCD can output limiting KG/GM curves as a design product.
- **Read**: read.

### marpol-i/29.2.3/slop-tank-capacity
- **Citation**: Reg. 29.1, 29.2.3, 29.3; waiver Reg. 3.4.
- **Requires**: slop tank(s) ≥ **3 %** of oil cargo capacity; Administration may accept 2 % (closed
  washing loop, or SBT or COW fitted), 1.5 % (SBT/COW + closed loop). ≥ 2 slop tanks only for
  ≥ 70,000 dwt. Reg. 3.4 waiver: voyages ≤ 72 h and within 50 nm of land, domestic trade, all oily
  mixtures retained for shore reception.
- **Applicability**: oil tankers ≥ **150 GT**.
- **Compute**: yes (capacity fraction; reduces net cargo volume).
- **Read**: read.

### marpol-i/33.1/crude-oil-washing
- **Citation**: Reg. 33.1–33.2; SOLAS II-2/4.5.5.1.3.
- **Requires**: COW system on crude tankers ≥ 20,000 dwt. If fitted voluntarily on a smaller ship,
  it must meet the safety parts of the COW Specifications, and SOLAS II-2/4.5.5.1.3 then requires
  an **inert gas system and fixed tank washing machines**.
- **Applicability**: crude ≥ 20,000 dwt mandatory. **Out of range, but voluntary COW triggers IG.**
- **Compute**: no (flag).
- **Read**: read.

### marpol-i/12A/oil-fuel-tank-protection
- **Citation**: Reg. 12A (MEPC.141(54), ships delivered on or after 1 Aug 2010).
- **Requires**: if aggregate oil fuel ≥ 600 m³: fuel tanks above bottom shell by
  `h = min(B/20, 2.0)`, ≥ 0.76 m; inboard of side by `w = 0.4 + 2.4·C/20,000`, ≥ 1.0 m
  (0.76 m for tanks < 500 m³). Individual fuel tank ≤ 2,500 m³.
- **Applicability**: ships with ≥ 600 m³ fuel. Most coasters and all barges carry less → usually
  not binding.
- **Compute**: yes (fuel volume switch).
- **Read**: read.

## B. SOLAS Chapter II-2 — tankers (self-propelled, ≥ 500 GT, international)

### solas-ii-2/1.6.1/tanker-scope
- **Citation**: SOLAS II-2/1.6.1, MSC.99(73) (2000 amendments, in force 1 Jul 2002).
- **Requires**: "tanker" requirements apply to crude oil and petroleum products with flashpoint
  ≤ 60 °C (closed cup) and RVP below atmospheric. Products > 60 °C → 1.6.4 lighter regime.
- **Compute**: yes (cargo property switch).
- **Read**: read.

### solas-ii-2/4.5.1.1/cargo-block-forward-of-machinery
- **Citation**: SOLAS II-2/4.5.1.1.
- **Requires**: cargo pump-rooms, cargo and slop tanks and cofferdams **forward** of machinery
  spaces. Cargo and slop tanks isolated from machinery spaces by cofferdams, cargo pump-rooms, oil
  bunker tanks or ballast tanks. Pump-room recess into a Cat. A machinery space: deckhead ≤ D/3
  above keel (≤ D/2 for ≤ 25,000 dwt if justified).
- **Compute**: yes (longitudinal zoning check; cofferdam length is a weight/length item).
- **Read**: read.

### solas-ii-2/4.5.1.2/accommodation-aft
- **Citation**: SOLAS II-2/4.5.1.2, 4.5.1.3.
- **Requires**: accommodation, control stations, service spaces aft of the cargo area and
  isolating spaces. A single deck or bulkhead failure must not let cargo gas in. Administration may
  permit them forward; non-Cat-A machinery forward is allowed with isolation.
- **Compute**: yes (zoning check).
- **Read**: read.

### solas-ii-2/4.5.1.5/navigation-position-over-cargo
- **Citation**: SOLAS II-2/4.5.1.5.
- **Requires**: a navigation position above the cargo area only if necessary, with an open space
  ≥ 2 m high between it and the cargo deck.
- **Compute**: yes (geometry check; relevant to forward-wheelhouse or pusher designs).
- **Read**: read.

### solas-ii-2/4.5.1.6/spill-coaming
- **Citation**: SOLAS II-2/4.5.1.6.
- **Requires**: keep deck spills away from accommodation, e.g. continuous coaming ≥ 300 mm side to
  side.
- **Compute**: no (flag; small weight item).
- **Read**: read.

### solas-ii-2/4.5.2.1/openings-facing-cargo
- **Citation**: SOLAS II-2/4.5.2.1–4.5.2.3.
- **Requires**: doors, air inlets and openings to accommodation, service, control and machinery
  spaces must not face the cargo area. On the sides they sit at least `max(0.04L, 3 m)`, need not
  exceed 5 m, from the end facing cargo. Windows in that zone fixed; A-60 except wheelhouse.
- **Compute**: yes (deckhouse layout check).
- **Read**: read.

### solas-ii-2/4.5.3.4/vent-outlet-height-distance
- **Citation**: SOLAS II-2/4.5.3.4.1.
- **Requires**: cargo vent outlets for loading/discharge: free-flow ≥ **6 m** above the cargo deck
  (or gangway within 4 m) and ≥ **10 m** horizontally from air intakes, ignition sources, windlass;
  or high-velocity (≥ 30 m/s) ≥ **2 m** above deck and ≥ 10 m away.
- **Compute**: yes (deck layout check; 10 m radius strongly constrains short ships).
- **Read**: read.

### solas-ii-2/11.6.2/pv-valve-outlets
- **Citation**: SOLAS II-2/11.6.1–11.6.3.
- **Requires**: thermal-breathing P/V outlets ≥ 2 m above cargo deck and ≥ 5 m from air intakes
  and ignition sources. Secondary full-flow relief or per-tank pressure sensors. High-level alarm or
  overflow control. P/V breakers: no overpressure above tank test pressure; vacuum ≤ 700 mm WG.
- **Compute**: partial (positions yes; tank design pressure is a structural input).
- **Read**: read.

### solas-ii-2/4.5.4.1/pump-room-ventilation
- **Citation**: SOLAS II-2/4.5.4.1.
- **Requires**: cargo pump-room mechanical exhaust ventilation, ≥ 20 air changes/h, non-sparking
  fans.
- **Compute**: no (flag).
- **Read**: read.

### solas-ii-2/4.5.5.1.2/inert-gas-8000-dwt
- **Citation**: SOLAS II-2/4.5.5.1.2, inserted by MSC.365(93) (in force 1 Jan 2016); 4.5.5.4.2.
- **Requires**: tankers **≥ 8,000 dwt** constructed on or after 1 Jan 2016 carrying cargoes under
  1.6.1 (crude included) need a **fixed inert gas system** (FSS Code ch. 15). For 8,000–20,000 dwt
  the Administration may accept equivalent arrangements. Any tanker using COW needs IG
  (4.5.5.1.3). With IG: double-hull spaces get IG connections (4.5.5.1.4) and a closed ullage
  system (4.5.5.3.2).
- **Applicability**: self-propelled SOLAS tankers ≥ 8,000 dwt. Not SOLAS-mandatory for barges.
- **Compute**: yes (dwt switch → add IG plant weight, space and power).
- **Read**: read.

### solas-ii-2/10.8/deck-foam (flag only)
- **Citation**: SOLAS II-2/10.8, 10.9 (MSC.99(73)).
- **Requires**: fixed deck foam system on tankers; fixed fire-extinguishing system in cargo
  pump-rooms. Details in the FSS Code (not downloaded).
- **Compute**: no (weight allowance only).
- **Read**: located in text, not studied in detail.

### solas-ii-2/hazardous-areas
- **Citation**: SOLAS II-1/45.11 and IEC 60092-502.
- **Requires**: zone classification of the cargo area for electrical equipment.
- **Compute**: no.
- **Read**: **not verified** (IEC 60092-502 paywalled; II-1 text not held).

### solas-i/3/non-propelled-and-sub-500gt-excluded
- **Citation**: SOLAS Ch I Reg. 3(a).
- **Requires**: SOLAS does not apply to cargo ships < 500 GT or ships not propelled by mechanical
  means, unless a regulation says so.
- **Compute**: yes (switch).
- **Read**: **secondary** (consolidated SOLAS paywalled).

## C. Load Lines (ICLL 1966 / 1988 Protocol, MSC.143(77) Annex B text)

### icll/27.2/type-a-definition
- **Citation**: ICLL Annex I Reg. 27(2), 27(4), MSC.143(77).
- **Requires**: Type A = designed to carry only liquid cargo in bulk; high-integrity exposed deck
  with only small gasketed steel access openings to cargo tanks; low permeability of loaded cargo
  compartments. Type A ships take freeboard from table 28.1.
- **Compute**: yes (switch).
- **Read**: read.

### icll/28.1/type-a-tabular-freeboard
- **Citation**: Reg. 28 tables 28.1 (Type A) and 28.2 (Type B).
- **Requires**: tabular freeboard by length. **The two tables are identical up to L = 61 m**
  (e.g. L = 24 m: 200 mm; 50 m: 443 mm; 60 m: 573 mm). They diverge above that: L = 80 m: A 841 /
  B 887 mm; L = 100 m: A 1,135 / B 1,271 mm. Type B ships under 100 m also get the Reg. 29
  short-superstructure addition `7.5·(100−L)·(0.35 − E1/L)` mm; Type A does not.
- **Effect**: for coasters and barges under ~60 m, Type A gives **no tabular gain**; the gain comes
  from avoiding the Reg. 29 addition, and from the Type A deck-integrity regime. Above 80 m the gain
  is 50–140 mm. Corrections common to both: block coefficient `×(Cb+0.68)/1.36` for Cb > 0.68
  (Reg. 30, Cb capped at 1.0); depth `+(D − L/15)·R` mm, `R = L/0.48` for L < 120 m (Reg. 31).
- **Compute**: yes — OVCD can compute minimum freeboard, then maximum draught and dwt from D.
- **Read**: read.

### icll/27.3/type-a-damage-over-150m
- **Citation**: Reg. 27(3), 27(11)–(13).
- **Requires**: damage survival only for Type A ships over 150 m. **Out of range.**
- **Read**: read.

### icll/27.14/barge-freeboard
- **Citation**: Reg. 27(14).
- **Requires**: barges get freeboards under these rules. Tank barges that meet 27(2)–(3) may get
  Type A freeboard. Deck cargo only with Type B. **Unmanned** barges: regs 25, 26(3), 26(4) and 39
  (bow height) do not apply. Unmanned barges with only small gasketed steel openings on the
  freeboard deck: freeboard may be **25 % less**.
- **Applicability**: non-self-propelled. Unmanned condition matters (tug-barge units where crew
  live on the tug).
- **Compute**: yes (switch; strong effect on depth).
- **Read**: read.

### icll/39/minimum-bow-height
- **Citation**: Reg. 39(1)–(2).
- **Requires**: `Fb = (6075(L/100) − 1875(L/100)² + 200(L/100)³)·(2.08 + 0.609Cb − 1.603Cwf −
  0.0129(L/d1))` mm, with d1 = draught at 85 % D. Sheer over ≥ 0.15L or forecastle ≥ 0.07L.
- **Applicability**: self-propelled and manned barges; not unmanned barges (27(14)(b)).
- **Compute**: yes.
- **Read**: read.

## D. Intact Stability Code 2008 (MSC.267(85))

### is-code/A2.2/general-criteria
- **Citation**: IS Code Part A 2.2.1–2.2.4 (and 2.3 weather criterion).
- **Requires**: area 0–30° ≥ 0.055, 0–40° ≥ 0.09, 30–40° ≥ 0.03 m·rad; GZ ≥ 0.20 m at ≥ 30°;
  GZmax ≥ 25°; GM0 ≥ 0.15 m; plus severe wind and rolling (2.3).
- **Applicability**: ships ≥ 24 m (mandatory via SOLAS/ICLL). Tankers < 5,000 dwt use these
  without the Reg. 27 "all conditions incl. liquid transfer" sweep.
- **Compute**: yes.
- **Read**: read (library copy, scanned/OCR).

### is-code/A3.2/oil-tankers-defer-to-marpol-27
- **Citation**: IS Code Part A 3.2.
- **Requires**: oil tankers ≥ 5,000 dwt comply with MARPOL Annex I Reg. 27.
- **Read**: read.

### is-code/B3.1/free-surface
- **Citation**: IS Code Part B 3.1.2–3.1.6.
- **Requires**: free surface for any tank < 98 % full; nominally full cargo tanks still corrected
  at 98 % with inertia at 5° heel; consumables: worst transverse pair or centre tank per liquid;
  liquid transfer may use the actual fill at each stage.
- **Compute**: yes.
- **Read**: read.

### is-code/B2.2/pontoons-not-for-tank-barges
- **Citation**: IS Code Part B 2.2.1.
- **Requires**: the pontoon criteria apply only to unmanned, non-self-propelled units that carry
  **only deck cargo**, Cb ≥ 0.9, B/D > 3, no hatchways. **A crude tank barge does not qualify**;
  use A2.2 or MARPOL Reg. 27.
- **Read**: read.

## E. US federal rules for tank barges (coastal reference case)

### us-33cfr/157.10d(c)/double-hull-widths
- **Citation**: 33 CFR 157.10d(c), eCFR 2026-10-16 (OPA 90).
- **Requires**: double sides **and** double bottom for every tank vessel (barges included):
  ≥ 5,000 dwt: `w = min(0.5 + DWT/20,000, 2.0)`, ≥ 1.0 m; `h = min(B/15, 2.0)`, ≥ 1.0 m.
  < 5,000 dwt: `w = 0.4 + 2.4·DWT/20,000`, ≥ 0.76 m; `h = B/15`, ≥ 0.76 m.
  **No 700 m³ single-side option** (unlike MARPOL 19.6.2).
- **Applicability**: tank vessels in US waters, contract after 30 Jun 1990.
- **Compute**: yes.
- **Read**: read.

### us-33cfr/157.10d(d)/inland-short-coastwise-61cm
- **Citation**: 33 CFR 157.10d(d), (e)(2)(ii)–(iii), (f).
- **Requires**: < 10,000 dwt certificated only for inland or limited short protected coastwise
  routes: double side and double bottom ≥ **0.61 m** (2 ft), ≥ 46 cm passage between framing.
  No oil within the lesser of 7.62 m or 5 % L (≥ 0.61 m) of the headlog; box/trail barges 0.61 m.
  Cargo tank length must stop at least `w` from the stern.
- **Compute**: yes.
- **Read**: read.

### us-46cfr/172.065/damage-stability-oil-tank-vessels
- **Citation**: 46 CFR 172.060–172.070, eCFR 2026-09-25.
- **Requires**: mirrors MARPOL Reg. 28 extents and survival criteria (L ≤ 150 m: between adjacent
  bulkheads). 172.070: tank vessels ≥ 5,000 dwt meet MEPC.117(52) intact stability (Reg. 27).
- **Compute**: yes.
- **Read**: read.

### us-46cfr/172.050/subchapter-d-barge-hull-types
- **Citation**: 46 CFR 172.047–172.050.
- **Requires**: Type I/II barge hulls (for liquefied flammable gas or RVP > 25 psia cargo in
  independent tanks) survive 6 ft × 30 in side damage and 15 in bottom damage with GM ≥ 50 mm.
  **Normal crude (RVP < 25 psia) is outside this subpart.**
- **Read**: read.

### us-46cfr/32.60-10/segregation-grade-abcd
- **Citation**: 46 CFR 32.60-10, 32.60-20.
- **Requires**: cofferdams, pump-rooms or tanks between cargo tanks and quarters/ignition sources;
  cargo tank spaces to main deck; pump-room access from open deck. Crude is Grade A–C by RVP and
  flashpoint (46 CFR 30.10, **not verified**).
- **Compute**: partial (zoning).
- **Read**: read.

## F. Class rules and notations

| Society | Notation (oil) | Rule text | Read? |
|---|---|---|---|
| ABS | `Oil Tank Barge` | Steel Barges 2024, 5-2-3: arrangement per MARPOL regs 18, 19, 23, 29; cofferdams; accommodation aft; openings ≥ max(0.04L, 3 m) from cargo end. 3-6-1/3: all tank barges classed after 1 Jul 1997 meet MVR App. 3-3-A1 (liquid-transfer intact stability); damage stability per MARPOL I. 3-6-1/7: stability instrument (waivable). | read |
| ABS | `Oil Carrier` (ships) | MVR Part 5C-1/5C-2 | not read (cite) |
| BV | `oil tanker` (+ `ESP`, `flash point > 60°C`) | NR467 Part D Ch 7 (Jul 2026). Rule routing by size: < 500 GT → NR566; L < 65 m → NR600 for hull and arrangement; ≥ 65 m → Part B. App. 5 covers accidental oil outflow. | read (application table only) |
| DNV | `Tanker for oil` (+ `ESP`) | DNV-RU-SHIP Pt 5 Ch 5 | **not verified** (PDF not retrievable) |
| LR | `Double Hull Oil Tanker`, `ESP` | Rules for Ships Pt 4 Ch 9 | **not verified** (account needed) |

Sea-going oil barges at DNV and LR: notation names not verified.

## G. What OVCD can compute — summary

| Rule id | Compute | Main design lever |
|---|---|---|
| marpol-i/19.3.1, 19.3.2, 19.6.1, 19.6.2, 21.4.2 | yes | inner-hull geometry, cargo volume |
| marpol-i/23.3.1 (≥ 5,000 dwt) | yes | tank subdivision |
| marpol-i/23.3.2 (< 5,000 dwt) | yes | tank length / count |
| marpol-i/27.1 + UI 55.1 | yes | beam, longitudinal bulkheads, KG |
| marpol-i/28.2–28.3 | yes | bulkhead spacing, freeboard |
| marpol-i/29.2.3 | yes | net cargo volume |
| icll/28.1, 27.14, 30, 31, 39 | yes | depth vs draught vs dwt |
| solas-ii-2/4.5.1, 4.5.2, 4.5.3.4, 11.6.2 | yes (checks) | zoning, deckhouse, vents |
| solas-ii-2/4.5.5.1.2 | yes (switch) | IG plant weight and space |
| marpol-i/19.3.6, 22, 28.6, 33; solas 10.8, 4.5.4.1 | no (flags) | equipment |

## H. Ranked list — requirements that most change a concept design

1. **Double hull / double bottom (marpol-i/19.3, 19.6; 21.4.2; us-33cfr/157.10d).** Sets the inner
   hull. Below 5,000 dwt the designer chooses between a single side with every tank ≤ 700 m³ or wing
   tanks w ≥ 0.76 m; heavy crude (> 900 kg/m³) or US trade removes that choice. Above 5,000 dwt,
   w ≥ 1.0 m and h = B/15 (1.0–2.0 m). Biggest effect on cargo cubic, lightship steel and KG.
2. **Intact stability in all loading and transfer conditions (marpol-i/27.1 + UI 55.1;
   is-code/B3.1).** Forces a centreline (or more) longitudinal bulkhead or narrower tanks, and sets
   minimum beam for a given depth. Binding ≥ 5,000 dwt; class applies it to all tank barges (ABS).
3. **Freeboard (icll/28.1, 27.14, 30, 31, 39).** Sets depth for a target draught and dwt. Type A
   gains little below 61 m; the 25 % reduction for unmanned barges with small gasketed openings is
   the largest single freeboard lever for tug-barge units.
4. **Damage stability (marpol-i/28.2–28.3).** One-compartment standard for L ≤ 150 m; sets
   transverse bulkhead spacing (≥ side-damage length to count) and residual freeboard.
5. **Tank size and count (marpol-i/23.3.2, 19.6.2; 23.3.1 OM ≤ 0.015 for ≥ 5,000 dwt).** Sets
   number of transverse bulkheads and longitudinal subdivision; drives steel weight and pump count.
6. **Cargo-area zoning (solas-ii-2/4.5.1, 4.5.2; ABS 5-2-3; 46 CFR 32.60-10).** Cargo block forward
   of machinery, cofferdams, accommodation aft, openings ≥ max(0.04L, 3 m) from the cargo end. Costs
   length (cofferdam ≈ one frame space to ~1 m, not verified) and fixes the general arrangement.
7. **No oil forward of collision bulkhead (marpol-i/19.7; us-33cfr/157.10d(e)).** Sets fore end of
   the cargo block; costs 5–8 % of L.
8. **Slop tanks 3 % (2 % with SBT/COW) (marpol-i/29.2.3).** Directly reduces net cargo capacity;
   waivable for short domestic voyages.
9. **Vent mast clearances (solas-ii-2/4.5.3.4: 6 m high / 10 m away; 11.6.2: 2 m / 5 m).** On ships
   under ~60 m this can decide deckhouse and windlass positions.
10. **Inert gas ≥ 8,000 dwt (solas-ii-2/4.5.5.1.2).** Adds plant weight, space and power at the top
    of the range; also triggered by any voluntary COW.

## I. Gaps and next steps

- **Consolidated texts not held**: MARPOL 2022, SOLAS 2024, LL 2021 editions are paywalled. Later
  Annex I amendments after MEPC.248(66) (e.g. any change to Reg. 12A, 23, 27) were not collected —
  verify that no post-2016 change touches the rules above.
- **SOLAS Ch I Reg. 3** (non-propelled and < 500 GT exclusions) is secondary only.
- **SOLAS II-1/11** collision bulkhead position and **II-1/45.11 / IEC 60092-502** hazardous areas:
  not verified.
- **FSS Code ch. 15** (IG) and **COW Specifications** (A.446(XI) as amended): not downloaded.
- **DNV Pt 5 Ch 5** and **LR Pt 4 Ch 9**: not retrieved; notations in table F for DNV and LR are
  from memory → not verified. BV NR566 / NR600 (the small-ship rule sets BV routes to) not held.
- **ITB/ATB tug-barge combinations**: no IMO text found on whether a pinned ITB is one "ship" for
  MARPOL dwt thresholds and SOLAS. US practice (46 CFR, USCG NVICs) not collected.
- **Crude cargo grades** under 46 CFR 30.10 (Grade A–C by RVP and flashpoint): not verified.
- **Weather criterion** (IS Code A2.3) not restated here; OVCD already holds the IS Code.
