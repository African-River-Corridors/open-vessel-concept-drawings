# Research note: stern blocks, propulsion, tugs, pushers and ATBs

Retrieved 2026-10-20. For spec 0006 and ADR 0015. Concept and preliminary design only; never approval.

## How to read this note

- **sourced**: the value is in the cited text, which we read.
- **derived**: plain arithmetic on sourced values.
- **estimate**: our own judgement; the basis is given.
- File paths are in the rule-source library (`~/Dev/ovcd-rule-sources/`, outside the repo, ADR 0001).
  The stern-and-tug files sit in `stern-tug/<area>/<folder>/`, one download per folder. The index
  `stern-tug/SOURCES.md` gives the URL, the retrieval date and the SHA-256 of each of the 172 files.
- A figure with no stated basis was left out. A source we could not read is named in "Gaps", never cited.

## 1. What OVCD does today

- **Stern block** (`aft_arrangement.rs`): a thruster room (up to three azimuth thrusters, 2.60 m
  envelope), 40 ft battery and accommodation units on deck, and a lifting wheelhouse. A gas-fuelled
  option (API only) adds an engine room of gas gensets and a gas valve unit room. No power, thrust or
  energy value sets any of it.
- **Mass:** the stern block has no mass in the lightship (`docs/basis` page 02, WC-A29). Every result
  floats on an even keel.
- **Powering:** one propulsive efficiency, 0.45, for every vessel. The app shows delivered-power curves
  only; it sizes no engine, battery or genset.
- **Formations:** a pushed convoy has no results; a pushed barge is the barge hull alone. The tug length,
  beam and notch depth in `models.rs` are for drawings only. `calibration/pushers.yaml` is read by no code.
- **Sightline:** a warning check with a fixed 0.50 m margin over one obstruction height. The maintainer
  decided on #264 that it must block `fit`.
- **Drawings:** a square stern, circles for thrusters, the wheelhouse lowered only.

## 2. Stern forms

### 2.1 Why inland sterns have tunnels

- Shallow draught limits the propeller diameter, so inland propellers are heavily loaded and prone to
  cavitation. `propulsion/tud-inland-stern` (Rotteveel 2019, TU Delft), §2.2.2 p.13–14. **sourced**
- The tunnel and its skirt (apron) stop air reaching the propeller. Ventilation is worst at bollard pull
  and in acceleration. Same, §2.2.3 p.14. **sourced**
- Tunnels add resistance. Same place. **sourced**

### 2.2 Tunnel geometry (Heuser, as quoted by Rotteveel 2019, §2.3.1 p.17)

| Item | Relation | Status |
|---|---|---|
| Tunnel radius over the propeller | = propeller diameter D | sourced |
| Stern run length L_H, single screw | 1.9·D + 1.85·B | sourced |
| L_H, twin screw, converging shafts | 2.5·D + 1.65·B | sourced |
| L_H, twin screw, parallel shafts | 3.2·D + 1.45·B | sourced |
| Twin shaft spacing | 0.28·B converging, 0.45·B parallel | sourced |
| Skirt and tunnel-end immersion | not printed in the thesis | gap |

- In the Class Va study (110 × 11.40 × 3.50 m), stern length, athwartships propeller position and
  diameter matter most. Summary p.vii. **sourced**
- In shallow water (h/T = 1.5): make D as large as possible, move it inboard, use an S-curved tunnel top.
  §7.2.3 p.112; `propulsion/ccr-deboer-lowwater` slide 19. **sourced**
- A bilge that turns inward before the skirt saves about 5 % resistance. §9.1.2 p.143. **sourced**

### 2.3 Which stern for which water

- Pram sterns suit high h/T; tunnel sterns suit intermediate and very shallow water. Rotteveel §2.3.1
  p.19, citing von der Stein. **sourced**
- For very shallow water: one propeller, or two close to the centreline. Same. **sourced**
- Pram barges may need skegs for course keeping and berthing. Same. **sourced**
- A twin-screw stern gives inclined inflow and an efficiency loss. Rotteveel §2.2.2 p.14. **sourced**
- An adjustable tunnel lets D exceed the part-load draught without air inlet; about 10 % fuel saving
  laden (claim, not built at full scale in 2009). `propulsion/ccr-tunnelschuerze` p.2. **sourced**

### 2.4 Propeller diameter against draught

| Case | D (m) | T (m) | D/T (derived) | Source |
|---|---|---|---|---|
| Class Va motor ship, twin ducted, full draught | 1.60 | 2.80 | 0.57 | `propulsion/vliz-414403` Table 1 |
| Same ship, reduced draught | 1.60 | 2.00 | 0.80 | same |
| Shallow pushboat, 4 screws | 1.30 | 1.40 | 0.93 | `propulsion/ccr-deboer-lowwater` slide 20 |
| Small pusher, single screw in nozzle | 1.20 | 1.96–2.40 | 0.50–0.61 | `propulsion/damen-pushboat-06541` p.1 |
| Seagoing bulk carrier or tanker | — | — | below about 0.65 | `propulsion/man-basic-principles` PDF p.18 |

So inland tunnel sterns run D/T of about 0.6 to 0.95 (derived from the rows above).

### 2.5 Immersion against air drawing

- Open propeller at zero advance: shaft submergence of about 0.76·D avoids air drawing (an extrapolation
  of model data). `propulsion/beveridge-bow-thrusters-1971` PDF p.20. **sourced** (a bow-thruster
  report; we use it as a guide for main propellers, **estimate**).
- Ducted unit: no free-surface effect down to 0.71·D to the axis. Same page. **sourced**
- Pump jet (Schottel SPJ): full thrust at 150–750 mm minimum immersion, by model. Online press release
  (not saved; read on the web). **sourced** as text.

### 2.6 Wake in a tunnel stern

DST model test, single-screw tunnel ship 86.0 × 11.36 × 2.5 m (`propulsion/dst-bericht-1495` §3.1 and
plates 9–11): nominal wake 0.457 at h/T = 3.0, 0.552 at h/T = 2.0, 0.65 at h/T = 1.5. **sourced**

## 3. Propulsors

### 3.1 Efficiency basics (seagoing; use with care inland)

`propulsion/man-basic-principles` PDF pp.14–18, all **sourced**: open-water efficiency 0.35–0.75; hull
efficiency 0.95–1.05 for a twin-screw conventional aftbody; relative rotative efficiency about 0.98 twin
screw; shaft efficiency about 0.99 (0.96–0.995); a larger diameter gives a higher efficiency; a CPP is
slightly less efficient than an FPP (larger hub).

### 3.2 Ducted propellers (Kort, 19A)

- Duct lift can be up to 30 % of the propeller's thrust. Rotteveel §2.2.2 p.14. **sourced**
- Bollard pull estimate (Wärtsilä Encyclopedia, "Bollard pull", read online):
  BP [t] = 0.9 · k2 · (D·P_D)^(2/3) / 9.81, with k2 = 1.2–1.4 for a nozzle. **sourced**
  Check on the small pusher above (D 1.2 m, 441 kW): 7.6 t against the sheet's 7.8 t. **derived**
- The rudder and the tunnel behind a nozzle both cut the nozzle thrust on a pusher.
  `propulsion/marin-asd-vs-shaftline` (Schot and Rotteveel 2020) p.14. **sourced**

### 3.3 Azimuth Z-drives and L-drives

- Blade-load limit (Veth): 400 kW/m² of disc area open, 500 kW/m² ducted, so
  D_min = √(4P/(π·q)). `propulsion/veth-azimuth-thrusters` p.83 footnote. **sourced**; the formula is
  **derived**. The Schottel SRP table fits 370–415 kW/m² (**derived**, `propulsion/schottel-product-guide`
  p.8).
- On a pushboat convoy (CFD): ASD is slightly better above 6 m water depth; shaftlines are better in
  shallow water; the azimuth diameter is smaller because the unit must turn. `marin-asd-vs-shaftline`
  p.14. **sourced**
- Mass: a ducted rudder propeller is W = 1.5·D³ t (scatter 1.3–1.8), from the Schottel range.
  `machinery-wheelhouse/tud-hekkenberg-2013` (Hekkenberg 2013, TU Delft) p.222. **sourced**

### 3.4 Voith Schneider, pump jets, twin and triple screw

- Voith Schneider tractors pull about 1.3 t per 100 kW (Bess 1.28, Baut 1.34; derived from
  `vessels/bube-bess-spec`, `vessels/bube-baut-spec`). No free open-water efficiency or draught data was
  found (gap).
- Pump jets: Schottel SPJ 50–3,500 kW, flush with the hull (protected in grounding).
  `propulsion/schottel-product-guide` PDF p.21. **sourced** A 40.3 × 9.5 m, T 1.0 m reference vessel
  with 2 × SPJ 82RD: `propulsion/dialnet-6769415` Table 1. **sourced** High-speed waterjets match
  propellers only above about 25 kn (`propulsion/hamiltonjet-designers-manual` p.2, **sourced**), so a
  pump jet is a shallow-water choice, not an efficiency choice (**estimate**).
- Twin screw spreads the load and adds redundancy (Rotteveel §2.2.2). Shallow pushboats use up to four
  screws (`ccr-deboer-lowwater` slide 20). **sourced**
- ES-TRIN 28.04(1)(a): a craft longer than 110 m needs two independent engines of equal power and a
  bow thruster, or one screw with a bow thruster able to propel the craft on its own.
  `es-trin/ES_TRIN_2025_signed_en.txt` Article 28.04. **sourced**

### 3.5 Bollard pull per kW (all ratios **derived** from stated BP and power)

| Propulsor | Vessels (t per 100 kW) | Band used (estimate) |
|---|---|---|
| Twin FPP in nozzles | Stan Tug 1606 1.79; Stan Tug 1907 1.68; Demi-K 1.70; pushboat 06541 1.74 | 1.65–1.80 |
| ASD / Z-drive | ASD 2312 1.84; ASD 2811 1.58; ASD 3212 1.63; ASD 2810 1.65; RAmparts 2500 1.56–1.58; Capt. Joseph Bisso 1.67; Aveogan 1.76 | 1.55–1.85 |
| Electric ASD | ASD 2111 E 1.59; eWolf 1.51 | — |
| Voith Schneider | Bess 1.28; Baut 1.34 | 1.25–1.35 |
| Shallow-draught pushers | CDS 4115 about 1.15; SDP 4600 1.29; RApide 4600-Z3 1.30 | small props at low draught |
| Open FPP | no vessel data | 1.2–1.45 (Wärtsilä formula with k2 = 1) |

The ASD 2312 and ASD 2811 have the same power (3,804 and 3,806 kW) and pull 70 t and 60 t with 2.8 m
and 2.4 m propellers (`vessels/damen-asd-tug-2312`, `vessels/damen-asd-tug-2811`). This pair tests the
diameter effect in any propulsor model.

## 4. Power sources and machinery

### 4.1 Mass per kW (Hekkenberg 2013, Appendix B, pp.221–225, all **sourced** unless marked)

| Item | Value |
|---|---|
| High-speed diesel, dry | 4.9 kg/kW (Cat, Cummins, Everllence sheets give 3.0–5.9, **derived**) |
| Gearbox | 2 kg/kW |
| Small engine-room items | 1 kg/kW of propulsion power |
| Piping, ducting, cabling | 3 t + 2.5 kg/kW |
| Gensets | 9 kg/kVA |
| Switchboards | 1 kg/kVA, at least 300 kg |
| Frequency converters | 900 kg + 2 kg/kVA |
| Electric motors | 6 kg/kW (range 5–10) |
| Bow thruster | 5.4 kg/kW; default power 0.11 kW per m³ of L·B·T |
| Rudders | 1 t per m of propeller diameter per rudder; steering machine 0.5 t for twin rudders |
| Pump jet | W [kg] = 0.003·P² + 6.0829·P − 138.17, P in kW |
| Accommodation, steel included | 0.17 t/m³ |
| Wheelhouse; raising column | 8 t; 5 t (one reference ship) |

Other parts:
- Hybrid motor on the gearbox (PTI/PTO) with its converter: 7.5–8.1 kg/kWe (**derived**,
  `machinery-wheelhouse/reintjes-hybrid-2025` pp.3–4).
- Batteries: NMC energy type 5.96 kg/kWh and 212.5 kWh/m³ at string level (Corvus page, **sourced**);
  LTO 12.1–17.4 kg/kWh (Echandia sheets, **sourced**), usable 90 % (Echandia, **sourced**).
- LNG dual-fuel medium-speed: 11.8–13.8 kg/kW for the L35/44DF, 3 MW and up (**derived**,
  `machinery-wheelhouse/everllence-l35-44df-prop` p.2). No free data for dual-fuel engines below 2 MW (gap).
- Diesel-electric losses: about 6.1 % at 18 MW HV, 9.4 % at 5 MW LV, up to 15 % below 1 MW (Wärtsilä
  Insights article, read online, vendor source). **sourced**

Totals (**estimate**, sums of the sourced items, excluding shafts, rudders and fuel):
- diesel-mechanical: about 10.4 kg/kW + 3 t;
- diesel-electric: about 24 kg/kW + 4.2 t (with kVA = kW / 0.8, our assumption).

### 4.2 Engine room length

No free text gives engine-room length as a function of power. Hekkenberg (2013) §4.2.4 pp.47–48 gives a
layout method (**sourced** as method): the aft ship is the longer of the accommodation and the engine
room; the engine room is the space aft of the engines (A), the engines and gearboxes, the genset and the
wheelhouse-column foundation if they do not fit beside (B), and a clearance to the forward bulkhead (C).
Tanks, pumps and switchboards fit round the main equipment.

Engine lengths (**sourced** from maker sheets in `machinery-wheelhouse/`):

| Engine | Power | Length |
|---|---|---|
| Cat 3512C (V12) | 955–1,903 kW | 2.59 m |
| Cat 3516C (V16) | 1,492–2,525 kW | 3.64–3.76 m |
| Cummins QSK60 (V16) | 1,491–2,013 kW | 3.29 m |
| Everllence 12V / 16V / 20V 175D | 1,740–4,400 kW | 2.90 / 3.42 / 3.94 m |
| Reintjes WAF-RHS gearbox | — | 1.09–1.28 m; 2.39–3.22 m with a PTI motor |

For the 175D family, length = 1.34 + 0.13 × cylinders (m) fits all three sizes (**derived**).

### 4.3 Fuel and battery spaces

- ES-TRIN 8.05(3)–(4): no fuel tank forward of the collision bulkhead, aft of the aft-peak bulkhead or
  directly above engines. **sourced**
- ES-TRIN 10.11: accumulators with more than 2.0 kW charging power go in a special room; lithium-ion
  rooms of 20 kWh or more need A60 boundaries and mechanical ventilation to open deck; no accumulators
  in the wheelhouse, accommodation or holds. **sourced**
- Battery room volume: no source gives a room-level figure. **estimate**: 70–85 kWh/m³ (NMC energy),
  35–50 kWh/m³ (LTO), from 2.5–3 times the string volume.

## 5. Rudders and bow thrusters

- Rhine Class Va ship: two ducted propellers, four fish-tail rudders (two per screw), 2.372 m² each
  (`propulsion/vliz-414403` §2.1 and Table 1, **sourced**); total about 3.1 % of L_PP·T (**derived**).
- Inland rudders turn to 60/80° asymmetric (`propulsion/damen-inland-rudders-de` p.3). **sourced**
- River towboats: flanking rudders sit forward of the propellers for astern work; a typical twin-screw
  towboat has four flanking and two steering rudders (US patent 6178907 and a trade article, read
  online). **sourced** as text. The CDS 4115 pusher has six rudders and four flanking rudders
  (`vessels/cd-4115-sd-pusher-brochure`). **sourced**
- Azimuth drives need no rudders (MARIN). **sourced**
- A tunnel bow thruster needs about one duct diameter below the waterline and one above the keel
  (Beveridge 1971 p.20, **sourced**), so it needs about 2·D of draught (**estimate**); shallow inland
  bows use channel jets. Veth Jet: about 11 kg thrust per kW (`propulsion/veth-bow-thrusters` p.4).
  **sourced**
- ES-TRIN Article 31.03 (crew standard S2): a motor cargo vessel operating alone needs a bow thruster
  worked from the steering position; a pusher needs coupling winches unless the lead craft has a bow
  thruster it can work. **sourced**

## 6. Tugs and pushers

### 6.1 Rule frame (ES-TRIN 2025/1 and BV NR217)

- Pusher: "a vessel specially built to propel a pushed convoy" (ES-TRIN 1.01, 1.9). A pusher with one
  craft coupled to permit guided articulation is also a rigid convoy (1.01, 2.4). **sourced**
- ES-TRIN 5.06(1): vessels and convoys reach at least 13 km/h through the water; not mandatory for a
  pusher alone. 5.04: tests at no less than 70 % of tonnage. **sourced**
- ES-TRIN 21.01(2): cable coupling needs at least two special winches or equivalent; 21.05(2): no
  downstream towing for craft longer than 86 m; 21.06: convoys are tested in the least favourable
  formation. **sourced**
- Coupling force (ES-TRIN ESI-III-6 §2, and BV NR217 Pt B Ch 7 Sec 6 [2.4.4]):
  F_SB = 270 · P_B · L_S / B_S · 10⁻³ kN (pusher to pushed craft). **sourced**
- BV NR217 Pt B Ch 7 Sec 6 [2.1.2]: the pushing device is at least two thirds of the pusher breadth.
  **sourced**

### 6.2 Sizes and power

- Rhine push boats: up to 40 × 15 m, draught about 1.90 m; the big ones, pushing up to six barges, have
  about 4,000 kW. `tugs/uliege-pushed-barge-thesis` pp.14–15 (citing a 2018 study). **sourced**
  L/B about 2.7, B/T about 8 (**derived**).
- Low-water concept pusher: 30 × 11 m, T 1.4 m, 3 × 700 kW, nozzles of 1.5 m; "more than four fully
  loaded" barges at usual convoy speed. Same thesis pp.74–76 (citing a 2009 study). **sourced**
- Mississippi: 5,000–6,999 hp towboats push 15-barge tows on the locking rivers (Waterways Journal,
  9 Aug 2024, read online). **sourced** as trade text.
- Pushers with both power and convoy size: CDS 4115 (3,990 kW, 12 barges, 281 × 48 m), SDP 4600
  (5,369 kW derived, 16 barges of 2,500 t), RApide 4600-Z3 (5,010 ekW derived, 35 barges of over
  2,000 t). See §9.

### 6.3 Conventional and ASD tugs

- Harbour and ASD tugs: L/B 1.9–2.8 (**derived** from the Damen sheets in `vessels/`).
- IS Code 2008 Part B 2.8 (towing, MSC.415(97)): the heeling lever uses BP·C_T, with C_T = 0.5 for
  conventional non-azimuth units and 0.90/(1 + l/L_LL) for azimuth units. **sourced** BV NR217 Pt D Ch 1
  Sec 7 [2.2.2] gives the inland equivalent. **sourced**
- 46 CFR 173.095 (US towline pull): GM ≥ N·(P·D)^(2/3)·s·h / (K·Δ·(f/B)), K = 13.93 metric. **sourced**

## 7. ATB and ITB systems

### 7.1 Treatment: one vessel or two

- ABS Steel Barges 2024, Part 5 Ch 3 Sec 1 (`abs-barges/10-barge-rules-jan24.txt` pp.267–273), all
  **sourced**: the tug and the barge are classed as two separate vessels (1.3); an ATB barge uses the
  parameters of the barge alone for strength and equipment (5.3, 7.7.3); a rigid ITB uses the combined
  length Lc, 96–97 % of the total waterline length (1.1.3); notch connections need a wing-wall
  torsional-shear analysis (11.3); the connector design uses lifetime loads from model tests or
  multi-body analysis (11.1.1). No separate current ABS ATB guide was found.
- 46 CFR 10.107: an ATB allows independent fore-and-aft pitch; a push-mode ITB moves as one with the
  barge. **sourced**
- USCG NVIC 2-81 Change 1 (`tugs/nvic-2-81-ch1-rosap`): dual-mode ITB tug and barge are separate vessels
  for inspection (§3.c(2)); disconnection within about five minutes is acceptable (Encl. 1 §5.d(1)(b));
  the designer sets the disconnect and reconnect sea states (§5.e(2)); the guidance does not apply to
  river routes (§2). **sourced** Change 2 (2023, adds ATBs) could not be read (HTTP 403).

### 7.2 Couplers

- JAK: pin couplers for tugs of 350–10,000 hp; socket plates welded to both sides of the barge notch; a
  bow "blister" on the tug. `tugs/jak-atb-coupler`. **sourced**
- Intercon: rams engage rows of teeth in the notch sides (`vessels/crowley-atb-12842-40085`); one
  degree of freedom, pitch about a transverse axis (MarineLink, read online). **sourced**
- Linkage classes: rigid, semi-rigid (articulated), flexible, loose; later generations use deeper notches
  and push for more of the time at sea. `tugs/yanakiev-2025-atb-review` (open-access review) §4.2 and
  Table 1. **sourced**

### 7.3 Proportions (ratios **derived** from builder sheets in `vessels/` and `tugs/`)

| Unit | L_tug / L_barge | B_tug / B_barge | Tug power |
|---|---|---|---|
| Legacy / 750-1 | 0.25 | 0.57 | 16,000 hp |
| Aveogan / Oliver Leavitt (AT-12842) | 0.32 | 0.49 | 6,700 hp |
| Aurora / Qamun (AT-10848) | 0.30 | 0.52 | 4,560 hp |
| 550 Class | 0.26 | 0.57 | 9,280 hp |
| Kim M. Bouchard | 0.24 | — | 10,000 hp |
| 8k and 10k bbl shallow-draft | 0.37–0.42 | 0.64 | — |

- Notch depth: about 0.5 × L_tug on one deep-notch ocean ATB (Legacy: barge 600 ft, coupled 674 ft,
  tug 148 ft). **estimate**, one data point. No rule or guide sets a notch ratio; no sheet gives notch
  width or depth in figures (gap).
- Review statistics (Yanakiev 2025 §4.1, **sourced**): tug L/B mean 3.19 (1.81–4.58), tug B/T mean 2.33;
  92 % of ATB barges are 105–203 m; barge L/B mean 5.76.
- Powering (Yanakiev 2025, **sourced**): ATB speed 9–12 kn, about half that of ships; rigidly linked
  units have about 5 % more resistance than equivalent ships (Table 3); shallow-water resistance rises
  0.2–0.3 times at h/T = 2.0 and 1.0–3.5 times at h/T = 1.2 (§2.1, citing model tests).
- The Aurora sheet gives 97 short tons bollard pull at 4,560 hp, which is far above every other tug
  (2.6 t/100 kW, derived). It is probably copied from the Aveogan sheet. Not used.

## 8. Convoy powering

- Zhang, Ringsberg and Thies (2023), `tugs/zhang-2023-inland-power`: R_T = ½ρV²S·C_T + R_S + R_BANK, with
  a depth-dependent friction line, a Millward shallow-water form factor and a bank term. Error "5.2 % for
  single vessels and 8 % for pusher-barge convoys"; valid for Frh 0.1–0.75; the common Raven correction
  works only above h/T = 2.0. **sourced**
- Holtrop, Mennen and van Terwisga (1990) covers L/B up to 7.25 and B/T up to 10 (Hekkenberg §4,
  **sourced**). In shallow water, L/h matters more than hull-form details (same, citing a 2000 study).
- Not read (gaps): Howe's formula text (ASCE papers are paywalled), DST/VBD convoy series (only cited
  second-hand).
- ES-TRIN ESI-III-6 uses 270 and 80 N/kW to turn installed power into a design thrust "while ensuring
  adequate levels of safety". These are safety values, not efficiency data. **sourced**

## 9. Wheelhouse, sightline and air draft

### 9.1 Rules

- ES-TRIN 7.02(2): the obstructed view ahead of the vessel, unladen with half its supplies and no
  ballast, shall not exceed 250 m. 7.02(3): field of view at least 240°, at least 140° in the forward
  semicircle; eye height 1.65 m above the wheelhouse floor. **sourced** ES-TRIN 2025/1 has no
  "1.5 × length" rule.
- ES-TRIN 19.01(5) passenger vessels: two lengths or 250 m, whichever is less. 29.06(2) high-speed
  craft: one length. **sourced**
- ES-TRIN 7.12: an elevating wheelhouse shall not endanger stability, can be entered in any position,
  and has an independent emergency lowering system. It sets no lift range. **sourced**
- ES-TRIN 1.01, 4.29 "fixed height": waterline to highest point with every mobile installation lowered.
  **sourced**
- Rhine Police Regulations §1.07(2): cargo or trim shall not restrict the view more than 350 m ahead of
  the bow; 500 m with radar and video (1.07(3)). `machinery-wheelhouse/de-rheinschpv-1-07`. **sourced**
- SOLAS V/22 (Res. MSC.99(73), `machinery-wheelhouse/imo-msc99-73`), ships of 45 m and over: the sea
  surface view is not obscured by more than two ship lengths or 500 m, whichever is less, forward of the
  bow to 10° either side, in all conditions of draught, trim and deck cargo (22.1.1); blind sectors at
  most 10° each and 20° in total (22.1.2). **sourced**

### 9.2 Eye height from the rule (**derived**, similar triangles)

With heights above the waterline of the stated loading condition, x measured forward from the eye:

    blind distance ahead of the bow = x_o · z_e / (z_e − z_o) − x_b
    z_e,min = max over obstructions of z_o · (x_b + B_max) / (x_b + B_max − x_o)
    z_floor,min = z_e,min − 1.65 m (ES-TRIN)

z_e is the eye height, z_o and x_o the top and forward edge of an obstruction (cargo, coaming, a
barge bow), x_b the distance to the bow (the convoy bow for a pusher), B_max the allowed blind length.

### 9.3 Air draft

- AGN Annex III Table 1 (`machinery-wheelhouse/be-fdfa-agn-amended` p.26): minimum height under bridges
  IV 5.25 or 7.00 m; Va/Vb 5.25, 7.00 or 9.10 m; VIa/VIb 7.00 or 9.10 m; VIc/VII 9.10 m. These include a
  safety clearance of about 0.30 m (footnote 2). **sourced**
- Existing E-waterway heights (UNECE Blue Book Rev.3, `machinery-wheelhouse/unece-blue-book-rev3`):
  Rhine below Mainz 9.10 m, Upper Rhine 7.00, Moselle 6.17, Main 6.00, upper Danube 4.61–6.00, Belgian
  E 01 5.25. **sourced**
- Elevating wheelhouse data (datum not stated in any source): eye height 11 m on two 19 m pushers that
  pass under bridges below 4 m lowered (`vessels/baird-demi-k-pusher`, `vessels/cd-riverbuster-1909-
  brochure`); a pusher bridge that elevates "up to 16 metres" (`vessels/cd-river-pusher-2010-web`); a
  US towboat eyeline of 39 ft 3 in (`vessels/workboat-bowling-green`). **sourced** as text.

## 10. Reference vessels (calibration candidates)

43 vessels with published particulars are in `stern-tug/vessels/` (full table in the library notes).
These 24 have power and dimensions on a stated basis and go to `calibration/` (spec 0006):

| Group | Vessel or design | L × B × T (m) | Power | Propulsion | Use |
|---|---|---|---|---|---|
| Inland cargo | CDS 110 Tanker | 110.00 × 11.45 × 3.30 | 2 × 500 kW | twin shaft, nozzles 1.52 m | power vs speed (18 km/h design) |
| Inland cargo | CDS 135 Tanker / Gambler | 135 × 11.45 × 3.31–3.50 | 2 × 709 kW | twin shaft, nozzles 1.62 m | power scaling at fixed beam |
| Inland cargo | CDS Dry Cargo 110 | 110.00 × 11.45 × 3.75 | 2 × 500 kW diesel or e-motor | twin shaft, nozzles 1.50 m | diesel vs electric stern |
| Inland cargo | Factofour | 135 × 17.20 × 4.00 | 2 × 1,350 kW | n/s | upper power bound |
| Inland cargo | LNG Greenstream | 110 × 11.4 × 3.46 | 4 × 285 kW LNG gensets, 2 × 500 kW Z-drive motors | Z-drive in nozzle | gas-electric stern |
| Pusher | CDS 4115 / Veronica V | 41.00 × 15.50 × 1.83–2.29 | 3 × 1,330 kW | 3 shafts, nozzles 1.87 m, 6 + 4 flanking rudders | BP 450 kN; convoy 12 barges |
| Pusher | SDP 4600 | 45.6 × 16.5 × 2.10 | 7,200 bhp | 3 azimuth in nozzles, diesel-electric | BP 69 t; 16 barges |
| Pusher | RApide 4600-Z3 | 45.6 × 16.5 × 2.1–2.65 | 3 × 1,670 ekW | 3 L-drives in nozzles, diesel-electric | BP 65 t; 35 barges |
| Pusher | Riverbuster 1909 | 19.00 × 9.08 × 1.90 | 1,500 kW | 2 Veth L-drives | BP 25 t; air draft 4.20 m |
| Pusher | Demi-K | 19.25 × 9.08 × 1.9 | 2 × 646 kW | 2 FPP in 1.6 m nozzles | BP 22 t; eye 11 m |
| Pusher | Bowling Green | 51.8 × 15.2 | 6,600 hp | 3 shafts, 2.54 m FPP | US towboat; eyeline 12.0 m |
| Pusher | Arne Christiansen | 36.6 × 10.4 | 3,200 hp | 2 Z-drives, 1.90 m | short-stern layout |
| Tug | Stan Tug 1606 | 16.76 × 5.94 × 2.25 | 894 kW | twin FPP, nozzles 1.35 m | BP 16.0 t |
| Tug | Stan Tug 1907 | 19.54 × 7.74 × 2.95 | 1,790 kW | twin FPP, nozzles 1.80 m | BP 30.0 t |
| Tug | ASD 2312 | 22.81 × 12.03 × 5.60 | 3,804 kW | 2 Z-drives, 2.8 m | BP 70 t |
| Tug | ASD 2811 | 28.57 × 11.43 × 4.65 | 3,806 kW | 2 Z-drives, 2.4 m | BP 60 t |
| Tug | ASD 3212 | 32.70 × 12.82 × 5.90 | 5,050 kW | 2 Z-drives, 3.0 m | BP 82.4 t |
| Tug | RSD 2513 Electric | 24.73 × 13.13 × 6.50 | 3,700 kW, 2,782 kWh | 2 L-drives, 3.0 m | battery tug, BP 70 t |
| Tug | ASD 2111 Electric | 21.02 × 10.81 × 5.25 | about 4,394 kW, about 4,820 kWh | 2 pods, 2.8 m | battery tug, BP 70 t |
| Tug | Bess | 36.00 × 12.00 × 5.00 | 5,450 bhp | 2 VSP | BP 52 t |
| Tug | Baut | 40.55 × 14.30 × 6.00 | 6,880 kW | 2 VSP | BP 92 t |
| ATB | Aveogan / Oliver Leavitt | tug 39.0 × 12.8; barge 121.9 × 25.9 | 6,700 hp | 2 Z-drives, 2.8 m | BP 88 t; 11 kn; Intercon C-50 |
| ATB | 550 Class | tug 37.1 × 12.8; barge 145.4 × 22.6 | 9,280 hp | twin FPP | 12 kn |
| ATB | Kim M. Bouchard | tug 45.7 × 15.2; barge 191.4 long | 10,000 hp | twin shaft, nozzles, 3.76 m | deep notch |

Imperial values were converted with 1 ft = 0.3048 m (**derived**). Open conflicts kept from the sources:
CDS 135 Tanker power (2 × 709 against 2 × 895 kW), CDS 2010 pusher draught (2.5 against 1.8 m), the
Aurora bollard pull.

No source gives an engine-room or stern-block length in metres. The proxies are one hull sold with
diesel or electric drive (CDS Dry Cargo 110), the 110 and 135 m tankers at one beam, and layouts with
gensets on deck (Greenstream, H2 Barge 1).

## 11. Gaps

- Engine-room or stern-block length of built ships (could be measured from scaled general arrangements).
- Heuser's skirt-immersion values; Voith Schneider efficiency and draught data; small LNG dual-fuel
  engines; battery-room density.
- Numeric notch depth and width; coupler sea-state ratings; Articouple, Artubar and Bludworth sheets.
- Howe's formula; the DST/VBD convoy series; NVIC 02-81 Change 2; a US list of lowest bridge clearances;
  clearances for European classes I–III.
- Blocked by bot checks (not bypassed): a towing-company site, a navy manual site, a journal mirror.
