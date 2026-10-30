# OVD crude oil — inland waterway rules for tank vessels and tank barges

Research note, 2026-10-20. Scope: crude oil (UN 1267, UN 3494) in inland tank vessels, tank
barges and push-tow tank barges. Seagoing rules are in a separate note.

## How to read this note

- **Rule id:** `<family>/<paragraph>/<slug>`. Families: `adn`, `es-trin`, `bv-nr217`, `lr-iww`,
  `normam-202`, `marpol-i` (comparison only).
- **Read:** "yes" = I read the primary text myself (file and page given). "secondary" = from a
  summary or another agent's note. "not verified" = not read.
- **Compute:** can a concept tool compute or check it from principal dimensions, tank layout,
  weights and stability?
  - **Y** = yes, from geometry or hydrostatics.
  - **P** = partly (needs a cargo-data input or a class scantling step).
  - **N** = no (equipment, operation or approval item; record as a checklist flag).
- Units: m, m³, kPa (gauge for tank pressures, absolute for vapour pressure, per ADN 1.2.1),
  m·rad.

### Sources read (local paths under `~/Dev/ovd-rule-sources/`)

| Source | Edition | File |
|---|---|---|
| ADN | 2025, ECE/TRANS/351 Vol. I and II, applicable 1 Jan 2025 | `adn/adn.pdf` (PDF page numbers below; printed page = PDF page − 24) |
| ES-TRIN | 2025/1 | `es-trin/ES_TRIN_2025_signed_en.pdf` |
| BV NR217 | June 2025 (Parts B and D) | `bv-nr217/217-NR_PartB_2025-06.pdf`, `217-NR_PartD_2025-06.pdf` |
| LR IWW Rules | November 2015 (LR Foundation, CC BY-ND) | `lr-inland/LR-IWW-Rules-Nov2015.pdf` |
| NORMAM-202/DPC | 1ª Revisão 2023, with Mod. 1 and Mod. 2 (to 2025-12-19) | `crude-oil/inland/normam-202/normam-202.pdf` (new download) |
| MARPOL Annex I | MEPC.117(52) | `crude-oil/seagoing/marpol-i/MEPC.117-52.pdf` (seagoing agent's download; reg. 19 read for comparison) |

---

## 1. ADN 2025 — Table C entries for crude oil

Source: ADN 2025, 3.2.3.2 Table C, PDF pp 212–216 (UN 1267) and 268–269 (UN 3494). Read: yes
(I read the rendered page images).

### 1.1 What the columns hold for crude oil

| Entry | (6) Vessel type | (7) Tank design | (8) Tank type | (9) Equipment | (10) HVV opening pressure kPa | (11) Max fill % | (12) Rel. density | (13) Sampling | (14) Pump room below deck | (15)/(16) T-class / expl. group | (17) Anti-ex | (18) Equipment | (19) Cones | (20) Remarks |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1267 PETROLEUM CRUDE OIL, PG I, II, III (6 rows, with and without remark 44) | * | * | * | * | * | * | blank | * | yes | T4³⁾ / II B⁴⁾ (II B3 rows with rem. 44) | yes | * | 1 (PG I, II), 0 (PG III) | 14; *see 3.2.3.3 (44 on II B3 rows) |
| 1267 … WITH MORE THAN 10% BENZENE (no IBP given), PG I–III | C | * | * | * | * | * | blank | * | yes | T4 / II B | yes | * | 1, 1, 0 | *see 3.2.3.3 (44) |
| 1267 … >10% BENZENE, INITIAL BOILING POINT (IBP) ≤ 60 °C, PG I–III | C | 1 (pressure tank) | 1 (independent) | — | — | 95 | blank | 1 | yes | T4 / II B | yes | PP, EP, EX, TOX, A | 1, 1, 0 | 43 (44) |
| 1267 … >10% BENZENE, 60 °C < IBP ≤ 85 °C, PG II–III | C | 2 (closed) | 2 (integral) | 3 (water spray) | 50 | 95 | blank | 2 | yes | T4 / II B | yes | PP, EP, EX, TOX, A | 1, 0 | 23; 38 (44) |
| 1267 … >10% BENZENE, 85 °C < IBP ≤ 115 °C, PG II–III | C | 2 | 2 | — | 50 | 95 | blank | 2 | yes | T4 / II B | yes | PP, EP, EX, TOX, A | 1, 0 | (44) |
| 1267 … >10% BENZENE, IBP > 115 °C, PG II–III | C | 2 | 2 | — | 35 | 95 | blank | 2 | yes | T4 / II B | yes | PP, EP, EX, TOX, A | 1, 0 | (44) |
| 3494 PETROLEUM SOUR CRUDE OIL, FLAMMABLE, TOXIC, PG I–III | C | * | * | * | * | 95 | blank | 1 (PG I), 2 (PG II, III) | **no** | T4 / II B (II B3 rem. 44) | yes | PP, EP, EX, TOX, A | 2, 2, 0 | 14; *see 3.2.3.3 (44) |

`*` = "determine by applying 3.2.3.3" (3.2.3.1, PDF p ~157 printed). Column (12) relative density is
blank for every crude-oil row. The density is a design input, and ADN 9.3.x.11.1(b) requires the
maximum relative density on the certificate of approval.

Remarks (3.2.3.1 column (20); read: yes):
- **14:** no carriage in type N for: self-ignition temperature ≤ 200 °C; flash point < 23 °C with
  explosion range > 15 percentage points; halogenated hydrocarbons; > 10% benzene; stabilized
  substances.
- **23:** pressure alarm at 40 kPa; water spray starts and runs until the pressure is 30 kPa.
- **38:** for 60 °C < IBP ≤ 85 °C (ASTM D86), the conditions are "identical to those … for an initial
  boiling point under or equal to 60 °C".
- **43:** "floater" may be precautionary.
- **44:** assign to this row only with measured data for the II B3/B2/B1 subgroup.

**Apparent conflict inside ADN (flag, do not resolve in code):** remark 38 sits on the
60–85 °C rows. Those rows show a closed integral tank at 50 kPa with water spray. The ≤ 60 °C rows
show a pressure tank. Read literally, remark 38 sends 60–85 °C crude to a pressure tank. Ask class
or the UNECE secretariat before relying on either reading.

### 1.2 The 3.2.3.3 decision for a crude oil (`*` rows)

Source: ADN 2025, 3.2.3.3 flowchart, PDF pp 273–274; Schemes A–C pp 275–277; column rules
pp 278–281. Read: yes (rendered images).

| Rule id | Criterion (as read) | Result | Compute |
|---|---|---|---|
| `adn/3.2.3.3/flowchart-type-c` | Any of these makes it type C: flash point < 23 °C with explosivity range > 15% at 20 °C; auto-ignition ≤ 200 °C; toxic (2.2.61); halogenated hydrocarbons; > 10% benzene; stabilized; N1 aquatic toxicity with Pv(50 °C) ≥ 1 kPa | Type C (Scheme A) | P (cargo data) |
| `adn/3.2.3.3/flowchart-n-closed-distinct` | Not C, and any of these: FP < 23 °C with N2; FP < 23 °C with floater/sinker; N1 with Pv50 < 1 kPa; **CMR Cat. 1A/1B** | Type N closed; tank walls distinct from hull (independent tank or integral tank in a double hull) | P |
| `adn/3.2.3.3/flowchart-n-closed-hull` | Not above, and flammable, FP < 23 °C, no other hazard | Type N closed; tank walls may form the hull (single hull) | P |
| `adn/3.2.3.3/flowchart-n-open` | Not above (e.g. FP ≥ 23 °C); with N2 or floater → open, walls distinct; else open, walls may form hull | Type N open (Scheme C: flame arrester when 23 °C ≤ FP ≤ 60 °C) | P |
| `adn/3.2.3.3/elevated-temp-tank-type` | Carriage temperature T: ≤ 80 °C integral tank allowed; 80 < T ≤ 115 °C independent tank (rem. 25 N / 26 C); > 115 °C independent tank | Tank type | Y (from carriage T) |
| `adn/3.2.3.3/scheme-a-type-c-valve` | Type C: IBP ≤ 60 °C and tank pressure at 50 °C > 50 kPa → pressure tank (400 kPa); 60 < IBP ≤ 85 °C → 50 kPa + water spray; 85 < IBP ≤ 115 °C → 50 kPa; IBP > 115 °C → 35 kPa; internal pressure at 50 °C ≤ 50 kPa → calculated (column (10) formula, rounded up to 5 kPa), at least 10 kPa | Col. (7), (9), (10) | P (needs IBP, vapour pressure) |
| `adn/3.2.3.3/scheme-b-type-n-closed-valve` | Type N closed, Class 3 FP < 23 °C: 175 ≤ Pv50 < 300 kPa → pressure tank (400 kPa), or 50 kPa with refrigeration; 150 ≤ Pv50 < 175 → 50 kPa; 110 ≤ Pv50 < 150 → 50 kPa (no spray) or 10 kPa with water spray; Pv50 < 110 → 10 kPa. CMR: 10 kPa (+ water spray if vapour pressure > 10 kPa by column (10) formula with va = 0.03) | Col. (9), (10) | P |
| `adn/3.2.3.3/col11-filling` | Type C 95%; type N 97% (95% if 175 < Pv50 < 300 kPa or molten) | Max fill | Y |
| `adn/3.2.3.3/col13-sampling` | Closed sampling for 6.1 PG I (so 3494 PG I); partly closed for all other type C and CMR in closed N | Col. (13) | N |
| `adn/3.2.3.3/col14-pump-room` | No pump room below deck for any substance with T in column (3b) (so not for 3494) | Col. (14) | Y (layout flag) |
| `adn/3.2.3.3/col17-anti-ex` | Anti-explosion protection if FP ≤ 60 °C, or heated to < 15 K below FP | Col. (17) | P |
| `adn/3.2.3.3/col9-heating` | "Possibility of heating" if melting point ≥ +15 °C (or > 0 °C and outside temperature ≤ melting point + 4 K); "heating system on board" if it must not solidify or must stay ≥ 15 K below FP | Col. (9) = 2 or 4 | P (needs pour point; read: yes, PDF p 278) |

**Practical result for most crude oils (my inference, not a rule text):** crude oil usually has
FP < 23 °C and is often classed CMR (benzene). That gives at least **closed type N with tank walls
distinct from the hull**. Benzene > 10%, H₂S toxicity (UN 3494), or N1 with Pv50 ≥ 1 kPa give
**type C**. Open type N is reachable only for a crude with FP ≥ 23 °C, no CMR, and no N1/N2/F.
That is uncommon. The tool must take FP, IBP, Pv50, benzene %, CMR, N-code, F, H₂S and pour point
as cargo inputs, and must default to **type C** when data are missing. ADN 3.2.3.1 says the
3.2.3.3 determination takes precedence over the mixture entries when data are insufficient.

---

## 2. ADN 2025 Part 9 — construction (type C = 9.3.2, type N = 9.3.3)

Read: yes for every row in this section. Pages: 9.3.2 PDF pp 565–606; 9.3.3 PDF pp 607–647;
9.3.4 PDF p 648.

### 2.1 Cargo area geometry (the concept drivers)

| Rule id | Citation (ADN 2025) | Requirement | Applies to | Compute |
|---|---|---|---|---|
| `adn/9.3.2.11.1/cargo-tank-max-volume` | 9.3.2.11.1(a); same table 9.3.3.11.1(a) | Max capacity of one cargo tank: LBH ≤ 600 m³ → 0.3·LBH; 600–3 750 m³ → 180 + (LBH − 600)·0.0635; > 3 750 m³ → 380 m³. L = overall hull length, B = extreme breadth, H = moulded depth in the cargo area (m). Note: the × sign is lost in the text extraction ("LBH0.3"). Continuity at 600 m³ (0.3·600 = 180) confirms the reading. | C and N (all lengths) | Y |
| `adn/9.3.3.11.1/trunk-equivalent-depth` | 9.3.3.11.1(a) | Type N trunk vessels: replace H by H′ = H + ht·(bt/B)·(lt/L) (formula garbled in extraction; symbols ht, bt, lt read) | N with trunk | Y (formula form "not verified" exactly) |
| `adn/9.3.4.1.1/alt-construction-tank-cap` | 9.3.4.1.1 | Capacity, tank length and the 9.3.2.11.7 side distance may be exceeded by the collision-energy method of 9.3.4. Absolute cap: 1 000 m³ per tank. 9.3.4 does **not** list 9.3.3.11.7 (type N 0.60 m). | C, N (G) | P (flag "9.3.4 study required") |
| `adn/9.3.2.11.1/cargo-tank-max-length` | 9.3.2.11.1(d); 9.3.3.11.1(d) | Cargo tank length ≤ 10.00 m if L ≤ 50 m; ≤ 0.20 L if L > 50 m. Not for independent cylindrical tanks with L/D ≤ 7. | C, N | Y |
| `adn/9.3.2.11.2/flush-deck-double-hull` | 9.3.2.11.2(a); 1.2.1 "Type C" | Cargo area is flush-deck double hull with wing spaces and double bottom, no trunk. Tanks do not extend beyond the deck. Independent tanks only in a hold bounded by double hull and double bottom. | C | Y |
| `adn/9.3.2.11.7/wing-width-min` | 9.3.2.11.7 | Side shell to longitudinal tank bulkhead ≥ 1.00 m. 0.80 m is allowed with: deck stringer +25%, side plating +15%, side longitudinals ≥ 0.15 m deep with ≥ 7.0 cm² face, webs ≤ 1.80 m spacing. | C, integral tanks | Y (P for the 0.80 m option) |
| `adn/9.3.2.11.7/double-bottom-depth` | 9.3.2.11.7 | Double-bottom mean depth ≥ 0.70 m, never < 0.60 m; 0.50 m under suction wells. | C | Y |
| `adn/9.3.2.11.8/hold-space-double-walls` | 9.3.2.11.8 | Tanks in a hold space: double walls ≥ 0.80 m apart; double bottom ≥ 0.60 m. | C, independent tanks | Y |
| `adn/9.3.3.11.7/wing-width-min` | 9.3.3.11.7 | Independent tanks or double hull with integral tanks: side ≥ 0.60 m; bottom ≥ 0.50 m (0.40 m under pump sumps); suction well to bottom structure ≥ 0.10 m. | N (where tank walls are distinct from hull) | Y |
| `adn/9.3.3.11.7/single-hull-allowed` | 1.2.1 type N sketches; 3.2.3.3; 9.3.3.11.7 | Type N has no general double-hull requirement. Single hull is allowed only when 3.2.3.3 says "walls may form vessel hull". | N | Y |
| `adn/9.3.2.11.3/cofferdam-width` | 9.3.2.11.3(a); 9.3.3.11.3(a) | Cofferdam ≥ 0.60 m between cargo tanks and accommodation, engine rooms, service spaces below deck outside the cargo area (or the vessel ends). Tanks in a hold: ≥ 0.50 m to hold end bulkhead (0.20 m for pressure tanks); an A-60 end bulkhead counts as a cofferdam. | C, N | Y |
| `adn/9.3.2.11.2/suction-well` | 9.3.2.11.2(c); 9.3.3.11.2(b) | Suction well ≤ 0.10 m³. | C, N | Y |
| `adn/9.3.2.11.2/pump-recess` | 9.3.2.11.2(e) | Deck recess for a cargo pump: ≤ 1.00 m deep; ≥ 6.00 m from accommodation/service openings; ≥ B/4 from side shell. | C | Y |
| `adn/9.3.2.11.5/ballast-only-wings` | 9.3.2.11.5; 9.3.3.11.5 | Wing spaces and double bottom in the cargo area hold ballast water only. Double bottoms may be oil-fuel tanks per 9.3.x.32 (≥ 0.6 m deep when hold spaces exist). | C, N | Y |
| `adn/9.3.2.11.10/access-openings` | 9.3.2.11.10; 9.3.3.11.9 | Openings ≥ 0.36 m², min side 0.50 m; tank circular openings ≥ 0.68 m diameter; clear width between stiffeners ≥ 0.50 m (0.45 m in double bottoms). This sets a practical lower bound on wing and DB size. | C, N | Y |
| `adn/9.3.2.26.2/residual-tank-cap` | 9.3.2.26.2 | Residual cargo tank ≤ 30 m³. | C (N same in 9.3.3.26, not re-read) | Y |

### 2.2 Distances from accommodation and openings

| Rule id | Citation | Requirement | Applies to | Compute |
|---|---|---|---|---|
| `adn/9.3.2.17.1/accommodation-outside-cargo-area` | 9.3.2.17.1; 9.3.3.17.1 | Accommodation and wheelhouse outside the cargo area, forward or aft of the planes that bound the below-deck cargo area. | C, N | Y |
| `adn/9.3.2.17.4/opening-setback-2m` | 9.3.2.17.4; 9.3.3.17.4 | Openable entrances, windows and other openings of superstructures and accommodation ≥ 2.00 m from the cargo area. | C, N | Y |
| `adn/9.3.2.16.1/engine-room-setback-2m` | 9.3.2.16.1; 9.3.3.16.1 | Engines outside the cargo area. Engine-room entrances and openings ≥ 2.00 m from the cargo area. Engine air intakes ≥ 2.00 m (9.3.2.31.2). | C, N | Y |
| `adn/9.3.2.12.4/vent-intake-6m` | 9.3.2.12.4(b)(i) | Accommodation/wheelhouse/service-space ventilation intakes ≥ 6.00 m from the cargo area and ≥ 2.00 m above deck. | C (N similar) | Y |
| `adn/9.3.2.22.4/hvv-outlet` | 9.3.2.22.4(e); 9.3.3.22.4(e) | High-velocity vent valve outlets ≥ 2.00 m above deck (1.00 m if no drive unit / no equipment within 1.00 m) and ≥ 6.00 m from accommodation, wheelhouse and service-space openings. | C, closed N | Y |
| `adn/9.3.2.25.1/deck-pump-6m` | 9.3.2.25.1; 9.3.3.25.1(c) | Cargo pumps on deck ≥ 6.00 m from accommodation/service-space openings. | C, N | Y |
| `adn/9.3.2.25.2/cargo-piping-b4` | 9.3.2.25.2(d),(e) | Deck cargo piping ≥ B/4 from the outer shell (not the shore connections). Shore connections ≥ 6.00 m from accommodation openings. No cargo piping below deck except in tanks and pump room. | C (N same, not re-read in full) | Y |
| `adn/9.3.2.10.3/protection-wall` | 9.3.2.10.3; 9.3.3.10.3 | Gas- and liquid-tight wall ≥ 1.00 m high, ≥ 1.00 m beyond the protected area, unless ≥ 12.00 m clear from HVV, shore connections, deck compressor and pressure-tank openings. | C, N with anti-ex cargo | Y |
| `adn/9.3.2.10.2/coamings` | 9.3.2.10.2 | Protective and spill coamings ≥ 0.075 m; protective coaming ≤ 0.60 m from the outer cofferdam bulkhead. | C, N | Y |
| `adn/9.3.2.10.4/sill-height` | 9.3.2.10.4 | Door sills and hatch/vent coamings of under-deck spaces ≥ 0.50 m above deck. | C, N | Y |
| `adn/9.3.2.22.1/tank-opening-height` | 9.3.2.22.1(b) | Tank openings > 0.10 m² and relief-device openings ≥ 0.50 m above deck. | C, N | Y |

### 2.3 Pump room, venting, vapour return, heating

| Rule id | Citation | Requirement | Applies to | Compute |
|---|---|---|---|---|
| `adn/9.3.2.17.6/pump-room-below-deck` | 9.3.2.17.6 | A cargo pump room below deck is allowed only with: cofferdam / A-60 / service or hold space to the engine room; exhaust outlets ≥ 6.00 m from accommodation openings; ≥ 30 air changes/h; O₂ alarm at 19.5% vol; gas detection at 20% LEL. Not allowed for UN 3494 (col. 14 "no"). | C (N similar, 9.3.3.17.6) | Y (layout), N (systems) |
| `adn/9.3.2.22.4/venting-closed` | 9.3.2.22.4(a)–(e); 9.3.3.22.4 "Closed Type N" | Each tank or group: connection for **vapour return ashore**; safe depressurization device; pressure/vacuum relief; detonation flame arrester at each tank connection; HVV design. | C, closed N | N (equipment); Y for HVV position |
| `adn/9.3.3.22.4/venting-open-n` | 9.3.3.22.4 "Open Type N", "Open Type N with flame arresters" | Open N: relief devices that prevent water ingress; with flame arrester: steady-burning arrester. No vapour return. | Open N | N |
| `adn/9.3.2.25.9/loading-rate` | 9.3.2.25.9 | Max loading/unloading flow per tank or group from venting design: gas flow = 1.25 × max loading flow; vapour density 50/50 vapour/air; 30% arrester clogging; over-pressure ≤ 1.15 × HVV opening pressure; vacuum ≤ design and ≤ 5 kPa. | C (N similar) | P |
| `adn/9.3.2.23.2/test-pressure` | 9.3.2.23.2–.3 | Cargo and residual tanks: test ≥ 1.3 × construction pressure. Cofferdams and open tanks ≥ 10 kPa. Cargo piping ≥ 1 000 kPa. Pressure tanks designed for 400 kPa (9.3.2.11.1(c)). | C (N similar) | P (feeds scantling design pressure) |
| `adn/9.3.2.28/water-spray` | 9.3.2.28 | When col. (9) = 3: deck water spray covering the whole cargo deck, ≥ 50 L/(m²·h), started from wheelhouse and deck, shore supply connection. | C, 60 < IBP ≤ 85 °C rows | Y (pump capacity = 0.05 m/h × deck area) |
| `adn/9.3.2.21.1/tank-instruments` | 9.3.2.21.1 | 95% fill mark; level gauge; level alarm at ≤ 90%; overflow sensor at ≤ 97.5%; vapour pressure gauge; temperature gauge if heated. | C, N | N |
| `adn/9.3.2.42/cargo-heating` | 9.3.2.42.1–.4 | Heating boiler on liquid fuel with FP > 55 °C, in the engine room or a separate below-deck space outside the cargo area. Cargo must not reach the boiler through a coil leak. | C (N similar) | N (Y for space allocation) |
| `adn/9.3.2.31.1/engine-fuel-fp` | 9.3.2.31.1 | Engines on fuel with FP > 55 °C, except LNG/other systems per ES-TRIN Ch. 30. | C, N | N |
| `adn/9.3.2.40.1/fire-main` | 9.3.2.40.1 | Two independent fire/ballast pumps, not in the same space; ≥ 3 hydrants in the cargo area; jet reach ≥ vessel breadth with two nozzles. | C (N similar) | P (pump sizing) |

### 2.4 What changes between open N, closed N and C (summary, read in 9.3.3 and 1.2.1)

| Item | Open N | Closed N | Type C |
|---|---|---|---|
| Hull in cargo area | single hull allowed if 3.2.3.3 permits | single hull or double hull per 3.2.3.3; trunk allowed | flush-deck double hull, no trunk |
| Min wing / DB | 0.60 / 0.50 m when tank walls are distinct | 0.60 / 0.50 m when distinct | 1.00 (0.80) / 0.70 mean, 0.60 min |
| Tank design | open (3 with flame arrester, 4 open) | closed (2), HVV 10–50 kPa | closed or pressure tank; HVV 35–50 kPa |
| Vapour return connection | not required | required | required |
| Gastight closures, venting piping rules | do not apply (9.3.3.22.6–.7) | apply | apply |
| Gas detection / ventilation / electrical | many provisions apply only near a shoreside assigned zone (9.3.3.10.6, .12.7, .52.11) | apply | apply |
| Damage-side extent | 0.59 m | 0.59 m | 0.79 m |
| Bottom damage vertical extent | 0.49 m | 0.49 m | 0.59 m |

---

## 3. ADN 2025 — stability and damage stability (9.3.x.13 to 9.3.x.15)

Read: yes. 9.3.2.13–.15 PDF pp 576–579; 9.3.3.13–.15 pp 616–619.

| Rule id | Citation | Requirement | Applies to | Compute |
|---|---|---|---|---|
| `adn/9.3.2.13.1/stability-proof` | 9.3.2.13.1; 9.3.3.13.1 | Intact and damage stability proof. Type N: **not required** for single-hull vessels with cargo tanks ≤ 0.70 B wide. | C all; N per text | Y |
| `adn/9.3.2.13.2/lightship-tolerance` | 9.3.2.13.2 | Lightship by inclining or mass calculation checked by a lightweight test within ±5%. | C, N | N (concept: margin) |
| `adn/9.3.2.13.3/all-densities-all-stages` | 9.3.2.13.3 | Intact stability for every stage of loading and unloading and the final condition, for **all relative densities** on the substance list; actual fillings of cargo, ballast and consumable tanks. | C, N | Y |
| `adn/9.3.2.14.1/intact-from-damage` | 9.3.2.14.1; 9.3.3.14.1 | Intact stability must satisfy what the damage calculation requires (N: for independent tanks and double hull). | C, N double hull | Y |
| `adn/9.3.2.14.2/intact-wide-tanks` | 9.3.2.14.2; 9.3.3.14.2 | If cargo tanks > 0.70 B wide: GZ ≥ 0.10 m within the positive range up to the first non-weathertight opening; area ≥ 0.024 m·rad up to that opening and in any case to < 27°; GM ≥ 0.10 m; **all free surfaces**, all stages. The stricter of .14.1 and .14.2 governs (9.3.2.14.3). | C, N | Y |
| `adn/9.3.2.15.1/side-damage-extent` | 9.3.2.15.1(a) | Side damage: longitudinal ≥ max(0.10 L, 5.00 m); transverse **0.79 m** inboard at max draught (or the 9.3.4 distance − 0.01 m); vertical: baseline upwards, no limit. | C | Y |
| `adn/9.3.3.15.1/side-damage-extent` | 9.3.3.15.1(a) | As above but transverse **0.59 m** (or 9.3.4 distance − 0.01 m). | N with independent tanks or double hull | Y |
| `adn/9.3.2.15.1/bottom-damage-extent` | 9.3.2.15.1(b); 9.3.3.15.1(b) | Bottom damage: longitudinal ≥ max(0.10 L, 5.00 m); transverse 3.00 m; vertical from base **0.59 m** (C) / **0.49 m** (N), sump excepted. Adjacent athwartship compartments flooded. | C, N | Y |
| `adn/9.3.2.15.1/two-compartment` | 9.3.2.15.1(c) | All bulkheads within the damage are damaged: survive flooding of two or more adjacent compartments lengthwise. Engine room: one-compartment standard. | C, N | Y |
| `adn/9.3.2.15.1/permeability` | 9.3.2.15.1(c) | 95% default; minima engine room 85%, accommodation 95%; DB, fuel, ballast 0% or 95% by function at max draught. | C, N | Y |
| `adn/9.3.2.15.2/intermediate-criteria` | 9.3.2.15.2 | Intermediate stages: GZ ≥ 0.03 m over a positive range ≥ 5°. | C, N | Y |
| `adn/9.3.2.15.2/final-criteria` | 9.3.2.15.2 | Final heel ≤ 12°. Beyond equilibrium: GZ ≥ 0.05 m and area ≥ 0.0065 m·rad up to the first non-watertight opening and ≤ 27°. Non-watertight openings ≥ 0.10 m above the damage waterline (9.3.2.15.1(c)). | C, N | Y |
| `adn/9.3.2.15.4/cross-flooding-15min` | 9.3.2.15.4 | Cross- or down-flooding equalization ≤ 15 min, if intermediate stability is proved. | C, N | P |

**Effect on concept design (my analysis of the read text):**
1. The side damage depth is 0.01 m less than the minimum wing width: 0.79 m against 0.80 m for
   C, 0.59 m against 0.60 m for N. The bottom damage depth matches the DB minimum less 0.01 m:
   0.59 m against 0.60 m, 0.49 m against 0.50 m. So the ADN damage case floods **ballast spaces
   only**, not cargo tanks, when the minimum widths are kept. A concept tool can model damage as
   lost buoyancy of wing and DB compartments.
2. Side damage of length max(0.10 L, 5 m) at any position, with bulkheads inside it damaged,
   floods **two adjacent wing compartments on one side**. The result is asymmetric. Short wing
   compartments, or wing and DB spaces that are separate on each side, limit the heel. Cross-flood
   ducts are an option only within the 15-minute limit.
3. Bottom damage 3.00 m wide floods adjacent athwartship DB compartments. A DB split on the
   centreline floods both sides, which is symmetric (sinkage, no heel).
4. The 12° heel limit and the 0.10 m clearance to openings set a **minimum freeboard to the
   first non-watertight opening** in the damaged condition. For a shallow river barge this, not
   the ES-TRIN 150 mm freeboard, usually sets the maximum draught (inference; verify per design).
5. Permeability of DB, fuel and ballast spaces is 0% or 95%, set by whether the space is full or
   empty at maximum draught. A wing tank that is empty at maximum draught floods at 95%. A full
   tank (0%) does not change buoyancy when it is damaged.

---

## 4. ES-TRIN 2025/1 — parts that affect tank vessels and barges

Read: yes (`es-trin/ES_TRIN_2025_signed_en.pdf`). ES-TRIN has **no tanker-specific stability
chapter**. Chapter 27 is "vessels carrying containers", not tankers. Tank-vessel stability comes
from ADN Part 9 and class.

| Rule id | Citation | Requirement | Applies to | Compute |
|---|---|---|---|---|
| `es-trin/3.02.3/stability-general` | Art. 3.02(3) | Stability shall correspond to the intended use (no numeric criteria). | All craft | N |
| `es-trin/3.03.1a/collision-bulkhead` | Art. 3.03(1)(a) | Collision bulkhead 0.04 L to 0.04 L + 2 m from the forward perpendicular at max draught; laden vessel keeps 100 mm residual safety clearance with the fore compartment flooded. 0.03 L allowed by calculation. | Self-propelled; pushers exempt (21.01(4)) | Y |
| `es-trin/3.03.1b/aft-peak-bulkhead` | Art. 3.03(1)(b) | L > 25 m: aft-peak bulkhead 1.4 m to 0.04 L + 2 m from the aft intersection of hull and max draught line (1 m by calculation). | L > 25 m | Y |
| `es-trin/4.01/safety-clearance` | Art. 4.01 | Safety clearance ≥ 300 mm; ≥ 500 mm to openings that cannot close spray-proof and weathertight. | All | Y |
| `es-trin/4.02/freeboard` | Art. 4.02 | Flush deck without sheer or superstructure: 150 mm. With sheer and superstructure: F = 150(1 − α) − (βv·Sev + βa·Sea)/15 mm, ≥ 0 mm. | All | Y |
| `es-trin/4.05/zone-4-reductions` | Art. 4.05 | Zone 4: safety clearance of doors/openings 150 mm (weathertight) or 200 mm (not); freeboard may be 0 mm. | Zone 4 waterways | Y |
| `es-trin/4.02.9/zone-1-2-salinity` | Art. 4.02(9) | Zones 1 and 2: salinity may be considered. Extra zone 1/2 requirements are national (EU Directive 2016/1629 Annex III, **not verified**). | Zones 1, 2 | P |
| `es-trin/5.06/min-speed` | Art. 5.06 | Vessels and convoys ≥ 13 km/h through the water (pushers solo exempt; estuary/port derogation). | Self-propelled, convoys | P (powering) |
| `es-trin/21.02/lighter-exemptions` | Art. 21.02(1)–(2) | Lighters without steering, accommodation or engine room: Chapters 5–7 and 15 do not apply. Ship-borne lighters L ≤ 40 m: collision bulkhead may be omitted if the front face carries 2.5 × the collision-bulkhead load. | Push-tow tank barges | Y |
| `es-trin/21.06/convoy-tests` | Art. 21.06 | Convoy manoeuvring tests (Arts 5.02–5.10) in the least favourable formation. | Pushers and pushed convoys | N |
| `es-trin/21.05.2/no-downstream-tow-86m` | Art. 21.05(2) | Craft with L > 86 m not authorised to tow downstream. | Towing | Y |
| `es-trin/28.03/damage-stability-110m` | Art. 28.03(3)–(9) | L > 110 m: damage stability with side damage ≥ 0.10 L × 0.59 m, bottom 0.10 L × 3.00 m × 0.39 m; intermediate heel ≤ 15°, GZ ≥ 0.02 m; final heel ≤ 12°, GZR ≥ 0.05 m, area ≥ 0.0065 m·rad to 27°. **Deemed met by a positive ADN Part 9 damage calculation (28.03(9)).** | L > 110 m (not passenger) | Y |
| `es-trin/28.04/redundancy-110m` | Art. 28.04(1)–(2) | L > 110 m: twin engines or single screw + independent bow thruster; radar; fixed bilge system. Derogation (2)(d): motor tankers built double hull per ADN 9.3.2.11.7 and 9.3.2.13–.15. | L > 110 m | P |

---

## 5. Class rules for inland tankers

### 5.1 BV NR217 (June 2025) — read: yes

| Rule id | Citation | Requirement | Applies to | Compute |
|---|---|---|---|---|
| `bv-nr217/PtD-Ch1-Sec3-3.2.1/side-tank-min-600` | Pt D Ch 1 Sec 3 [3.2.1] | Side tank width ≥ 600 mm (service notation Tanker). | All BV inland tankers | Y |
| `bv-nr217/PtD-Ch1-Sec3-2.3.1/centre-bhd-0.7B` | Pt D Ch 1 Sec 3 [2.3.1] | Non-DG tankers: tank breadth > 0.7 B normally needs a centre longitudinal bulkhead; else prove intact stability per Pt B Ch 2 Sec 2 [4]. DG tankers go to Pt D Ch 3. | Tanker | Y |
| `bv-nr217/PtD-Ch3-Sec3-3.2.7/dg-c-wing-1.0` | Pt D Ch 3 Sec 3 [3.2.7]–[3.2.10] | DG-C: side to inner side ≥ 1.00 m (**no 0.80 m option in this paragraph**, unlike ADN 9.3.2.11.7); DB mean ≥ 0.70 m, never < 0.60 m, 0.50 m under wells; hold-space tanks 0.80 / 0.60 m. | DG-C | Y |
| `bv-nr217/PtD-Ch3-Sec4-3.2.2/dg-n-wing-0.6` | Pt D Ch 3 Sec 4 [3.2.2] | DG-N: space to tank wall ≥ 0.60 m; bottom ≥ 0.50 m (0.40 m at sumps); well to bottom ≥ 0.10 m. Mirrors ADN 9.3.3.11.7. | DG-N | Y |
| `bv-nr217/PtB-Ch2-Sec2-2.3/free-surface` | Pt B Ch 2 Sec 2 [2.3.1]–[2.3.2] | Correct GM and the GZ curve for free surfaces in **all** loading conditions, at any filling level; not needed for a tank that is nominally full. | All | Y |
| `bv-nr217/PtB-Ch2-Sec2-4.1/intact-criteria` | Pt B Ch 2 Sec 2 [4.1.2]–[4.1.4] | Area ≥ 0.024 m·rad to the first non-weathertight opening and ≤ 27°; GZ ≥ 0.10 m; GM₀ ≥ 0.10 m (same numbers as ADN 9.3.x.14.2). | All (where applied) | Y |
| `bv-nr217/PtD-Ch2-Sec6-4/damage-tankers` | Pt D Ch 2 Sec 6 [4.1]–[4.2] | Notation "Damage stability", tankers: side damage ≥ max(0.10 LOA, 5 m), transverse **B2 − 0.01 m** (B2 = actual double-side width), bottom 3.00 m wide, vertical **D2 − 0.01 m** (D2 = actual DB height); criteria as ADN final stage (12°, 0.05 m, 0.0065 m·rad, 27°). | BV notation | Y |
| `bv-nr217/PtD-Ch1-Sec3-5.1.3/thermal-stress-90C` | Pt D Ch 1 Sec 3 [5.1.3] | Heated cargo above 90 °C: thermal stress calculation (water 0 °C, air 5 °C). | Heated crude | P |
| `bv-nr217/PtD-Ch1-Sec3-5.1.1/min-net-4.4` | Pt D Ch 1 Sec 3 [5.1.1] | Net thickness ≥ 4.4 mm in the cargo-tank area including cofferdams. | Tanker | P (steel weight) |
| `bv-nr217/PtD-Ch1-Sec3-6.5.2/no-tube-pillars` | Pt D Ch 1 Sec 3 [6.5.2] | No tubular pillars in tanks for flammable liquids. | Tanker | N |
| `bv-nr217/PtD-Ch1-Sec3-9.1.1/expansion-tank` | Pt D Ch 1 Sec 3 [9.1.1] | Each tank has an expansion trunk at about mid-length, ≥ 0.5 m above the tank top. | Tanker | Y |
| `bv-nr217/PtD-Ch1-Sec3-8.2.1/independent-tank-supports` | Pt D Ch 1 Sec 3 [8.1.3], [8.2.1] | Anti-flotation for an empty tank at damage draught; fore-aft seating force 0.30 P. | Independent tanks | P |

### 5.2 LR IWW Rules (Nov 2015) — read: yes (2015 edition; the July 2022 and July 2026 editions are not verified)

| Rule id | Citation | Requirement | Compute |
|---|---|---|---|
| `lr-iww/Pt4-Ch4-1.7.1/stability-per-adn` | Pt 4 Ch 4 1.7.1 | Intact and damage stability of type G, C, N "in accordance with recognized international stability requirements such as laid down in the ADN". | Y |
| `lr-iww/Pt4-Ch4-3.6.5/type-c-wing` | Pt 4 Ch 4 3.6.5 | Type C: side to tank bulkhead ≥ 1.00 m, 0.80 m with reinforcement; DB mean ≥ 0.70 m, ≥ 0.60 m (same as ADN). | Y |
| `lr-iww/Pt4-Ch4-3.7.1/type-n-wing` | Pt 4 Ch 4 3.7.1 | Type N double hull: side ≥ 0.60 m; bottom ≥ 0.50 m (0.40 m at pump suctions). | Y |
| `lr-iww/Pt4-Ch4-1.2.3/non-adn-flags` | Pt 4 Ch 4 1.2.3 | Ships for countries with rules other than ADN get "special consideration". | N |

### 5.3 Other class societies

- **DNV RU-INV, RINA inland, PRS inland, CCS inland:** not obtained (see SOURCES.md). Not verified.

---

## 6. River-specific national rules

### 6.1 Brazil — NORMAM-202/DPC (read: yes, new download)

Applies to **new** vessels (licence requested after 30 June 2004) that carry alcohol, petroleum and
derivatives on Brazilian inland waters. The definition of petroleum includes crude oil by reference
to MARPOL Annex I Appendix I (5.20 o). Barges ("balsas") are covered (5.20 d).

| Rule id | Citation | Requirement | Applies to | Compute |
|---|---|---|---|---|
| `normam-202/5.22.1/class-mandatory` | 5.22.1 | Class compulsory: self-propelled GT ≥ 500; non-propelled GT > 2 000. | New petroleum carriers | Y (GT estimate) |
| `normam-202/5.23.6a/double-hull-bottom` | 5.23.6 a) II) | Double bottom: cargo tank to bottom ≥ **max(B/15, 0.76 m)**. | All new petroleum carriers (except small fuel stations, 5.23.6 b)) | Y |
| `normam-202/5.23.6a/double-hull-side` | 5.23.6 a) II) | Double side: cargo tank to side ≥ **1.00 m**. | Same | Y |
| `normam-202/5.23.6a/sump` | 5.23.6 a) III) | Suction wells may enter the DB if < 100 L and the reduction in h is ≤ 25%. | Same | Y |
| `normam-202/5.23.7/stability` | 5.23.7 | Intact and damage stability; inclining test after build; damage stability for all expected loading, draught and trim conditions. | Same | Y |
| `normam-202/5.23.8/damage-extent` | 5.23.8 a)–g) | Side: longitudinal min(⅓·L^(2/3), 14.5 m); transverse min(B/5, 11.5 m); vertical unlimited. Bottom (FP to 0.3 L): min(⅓·L^(2/3), 14.5 m) × min(B/6, 10 m) × min(B/15, 6 m); elsewhere min(⅓·L^(2/3), 5 m) × min(B/6, 5 m) × min(B/15, 6 m). Two adjacent compartments; permeability 0.95 (stores 0.60, engine 0.85). Smaller damage must also pass if worse. | Same | Y |
| `normam-202/5.23.9/damage-criteria` | 5.23.9 | Positive GZ range ≥ 20° beyond equilibrium; area ≥ 0.0175 m·rad within that 20° or to downflooding; max residual GZ ≥ 0.1 m; final heel ≤ 25° (30° if the deck edge does not immerse). | Same | Y |
| `normam-202/5.23.3l/opening-setback-4m` | 5.23.3 l) | No accommodation or service spaces in the cargo area; front bulkhead windows fixed; openings to galleys, accommodation or spark sources ≥ **4 m** from the cargo area. | Same | Y |
| `normam-202/5.23.3a/engine-fuel-60C` | 5.23.3 a) | Engine fuel FP ≥ 60 °C; spark arresters. | Same | N |
| `normam-202/5.23.5a/sopep-200m3` | 5.23.5 a) | Shipboard oil pollution emergency plan if > 200 m³ of petroleum. | Same | N |

**Design effect:** the NORMAM side damage (B/5 deep) breaches a 1.0 m wing tank on any vessel
wider than 5 m. So, unlike ADN, the **cargo tank floods** in the damage case. The tool must model
cargo outflow and flooding of cargo tanks (MARPOL-style), not only ballast spaces.

### 6.2 Paraná–Paraguay Hidrovía (Argentina, Paraguay, Bolivia, Brazil, Uruguay)

- PNA Disposición DPAM RE.4 N° 01/2012, Agregado N° 1, sets tank-barge containment distances "w"
  and "h". A 2020 extension (Disposición 37/2020) quotes "h ≥ 0.61 m" for existing units. RIOCON II
  (Nov 2019 draft) is the regional rule for noxious liquids. **Secondary** (cira.org.ar summary).
  Primary text not obtained.
- A press report says PNA would require a 0.76 m double hull on barges. **Not verified.**
- A "Reglamento de Francobordo y Estabilidad" for Hidrovía vessels exists. **Not verified.**

### 6.3 China (CCS inland rules, China MSA statutory rules) — not obtained, not verified.

### 6.4 West Africa (Ghana Volta Lake, Niger/Benue) — no free national technical rule for inland tank vessels found. Not verified. LR Pt 4 Ch 4 1.2.3 and BV DG-C/DG-N (both ADN-based) are the likely class route (inference).

---

## 7. Conflicts and differences with MARPOL Annex I ("strictest rule governs")

MARPOL text read: reg. 19.3 and 19.6 (`MEPC.117-52.pdf`). Other MARPOL items below are marked.

| Topic | ADN (inland) | MARPOL Annex I | Strictest for coastal + river service |
|---|---|---|---|
| Wing width | C: ≥ 1.00 m (0.80 m reinforced); N: ≥ 0.60 m | ≥ 5 000 dwt: w = min(0.5 + DW/20 000, 2.0 m), ≥ 1.0 m (19.3.1). < 5 000 dwt: no wings needed if each tank ≤ 700 m³; else w = 0.4 + 2.4·DW/20 000, ≥ 0.76 m (19.6.2) | MARPOL for ≥ 5 000 dwt above ~10 000 dwt (w > 1.0 m). Below 5 000 dwt, ADN C (1.0 m) is stricter than MARPOL 0.76 m. ADN N (0.60 m) is never the strictest. |
| Double bottom | C: mean ≥ 0.70 m, ≥ 0.60 m; N: ≥ 0.50 m | ≥ 5 000 dwt: h = min(B/15, 2.0 m), ≥ 1.0 m. < 5 000 dwt: h = B/15, ≥ 0.76 m (19.6.1) | MARPOL for any B > 10.5 m (B/15 > 0.70 m), and always where its 0.76 m / 1.0 m minimum applies. NORMAM-202 adopts max(B/15, 0.76 m). |
| Cargo tank size | ≤ 380 m³ (LBH formula); length ≤ 0.20 L | < 5 000 dwt without wings: ≤ 700 m³. Reg. 26 tank-size limits: **not verified here** | ADN for the volume cap. |
| Damage extents | side 0.79 / 0.59 m deep, 0.10 L long; bottom 3.00 m × 0.59 / 0.49 m | Reg. 28 extents: **secondary** (NORMAM 5.23.8 uses the same min(⅓L^(2/3), 14.5 m), B/5, B/6, B/15 pattern) | MARPOL-type extents breach the cargo tanks; ADN extents do not. Run both. |
| Damage criteria | final heel ≤ 12°; GZ ≥ 0.05 m; area ≥ 0.0065 m·rad; to 27° | Reg. 28: 25°/30° heel, 20° range, 0.1 m GZ, 0.0175 m·rad (**secondary**, as copied in NORMAM 5.23.9, read) | Not ordered. ADN is stricter on heel (12° vs 25°); MARPOL is stricter on range and area. Check both. |
| Intact stability | Only if tanks > 0.70 B: GZ ≥ 0.10 m, area ≥ 0.024 m·rad, GM ≥ 0.10 m | Reg. 27 → IS Code 2008 Part A 2.2 (area 0.055 to 30°, etc.): **not verified here** (see seagoing note) | MARPOL/IS Code. |
| Venting | Closed tanks, HVV 10–50 kPa, vapour return connection, detonation arresters | Reg. not in Annex I; SOLAS II-2 inert gas for ≥ 8 000 dwt (seagoing note) | Both sets apply; they do not conflict. ADN has no inert-gas requirement for crude. |
| Engine fuel FP | > 55 °C (ADN 9.3.2.31.1) | — | NORMAM 60 °C is stricter in Brazil. |
| Opening setback | 2.00 m | — | NORMAM 4 m is stricter in Brazil. |

---

## 8. Ranked list — requirements that most change a crude-oil concept design

1. **Vessel type from cargo data** (`adn/3.2.3.3/flowchart-*`, Table C 1267/3494). Type C or N
   sets the whole cargo-area layout. Default to C without full cargo data. UN 3494 is always C.
2. **Wing width and double-bottom depth** (`adn/9.3.2.11.7/*`, `adn/9.3.3.11.7/*`,
   `normam-202/5.23.6a/*`, `marpol-i/19.3`, `19.6`). Take the maximum over the rule sets that
   apply: 1.00 m side and max(0.70 m mean, B/15, 0.76 m) bottom covers ADN C, NORMAM and MARPOL
   below 5 000 dwt.
3. **Cargo tank capacity and length caps** (`adn/9.3.2.11.1/cargo-tank-max-volume`,
   `…/cargo-tank-max-length`). For a large barge, 380 m³ per tank and 0.20 L per tank fix the
   number of tanks, the transverse bulkhead count and the steel weight. 9.3.4 can raise the cap to
   1 000 m³ only by a collision-energy study.
4. **Damage stability** (`adn/9.3.2.15.*`, `normam-202/5.23.8–9`). Under ADN this sets wing
   compartment length and side split (asymmetric flooding, 12° heel). Under NORMAM/MARPOL-type
   extents, cargo tanks flood, which can set the draught.
5. **Intact stability with free surfaces for all stages and all densities**
   (`adn/9.3.2.13.3`, `adn/9.3.2.14.2`, `bv-nr217/PtB-Ch2-Sec2-2.3`). Tanks wider than 0.70 B
   trigger the explicit criteria. BV expects a centre bulkhead in that case. This decides one
   tank across the beam, or two.
6. **Tank pressure class** (`adn/3.2.3.3/scheme-a-*`, `scheme-b-*`; Table C col. 7, 10). A
   volatile crude (IBP ≤ 60 °C with high Pv50) needs pressure tanks at 400 kPa. Otherwise it needs
   closed tanks at 35–50 kPa, with a test pressure of 1.3 ×. This affects scantlings and weight.
7. **Cofferdams and the accommodation/engine-room separation** (`adn/9.3.2.11.3`,
   `adn/9.3.2.17.*`, `adn/9.3.2.16.1`). 0.60 m cofferdams at both ends plus 2 m / 6 m setbacks fix
   the cargo-area length available inside L.
8. **Pump room** (`adn/9.3.2.17.6`, col. 14). Allowed below deck for UN 1267 with conditions; not
   for UN 3494. Otherwise use deck pumps ≥ 6 m from openings or deep-well pumps.
9. **Heating** (`adn/3.2.3.3/elevated-temp-tank-type`, `col9-heating`, `adn/9.3.2.42`,
   `bv-nr217/…/thermal-stress-90C`). Waxy crude above 80 °C forces independent tanks. Above
   90 °C, BV requires thermal-stress checks. Heating also needs a boiler space outside the cargo area.
10. **ES-TRIN freeboard and safety clearance** (`es-trin/4.01`, `4.02`, `4.05`), collision and
    aft-peak bulkhead positions (`es-trin/3.03`), lighter exemptions for push barges
    (`es-trin/21.02`), and the > 110 m rules (`es-trin/28.03–28.04`).

---

## 9. Gaps

- ADN source URL for the library copy is not verified (UNECE returns 403 to scripts). The content
  matches ADN 2025 (ECE/TRANS/351).
- Remark 38 against the IBP ≤ 60 °C / 60–85 °C rows of UN 1267 (> 10% benzene) looks
  contradictory. Needs an authoritative interpretation.
- ADN 1.6.7 transitional provisions (single-hull type N, existing vessels) not read.
- ADN 7.2.4 (filling, vapour return in operation) and 7.2.3.7 (degassing of crude-oil tanks) not
  read.
- MARPOL reg. 23, 26, 27 and 28 not read here. They are marked "secondary" or "not verified". See
  the seagoing note.
- DNV, RINA, CCS, PRS inland rules and the China MSA statutory rules: not obtained.
- Hidrovía (PNA RE.4 01/2012, RIOCON) primary texts: not obtained. West Africa: no technical rule
  found.
- NORMAM-202 intact stability criteria for tank vessels (in its stability chapter) not read; only
  the damage criteria of 5.23.9 were read.
- EU Directive 2016/1629 Annex III (zone 1/2 extra requirements) not read.
- Class scantlings (design pressure → plate thickness, steel weight) not extracted; only the rule
  pointers are given.
