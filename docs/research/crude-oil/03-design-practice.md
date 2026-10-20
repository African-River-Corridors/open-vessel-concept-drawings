# OVCD crude oil cargo — 03 Design practice and published data

Scope: crude oil tank barges (inland, push-tow, ATB) and small coastal and river-sea tankers (about 500 to 12,000 dwt).
Written 2026-10-20 for OVCD (public, open-source). Simplified Technical English.

**How to read this file**

- Each number has a source tag, for example [S7]. The source list is at the end.
- **(est.)** = my estimate or my calculation from source figures. The method is given next to it.
- **(unverified)** = seen only in a search-engine extract. I could not open the page. Do not calibrate on it.
- PDFs are saved under `~/Dev/ovcd-rule-sources/crude-oil/practice/<dir>/` and logged in
  `~/Dev/ovcd-rule-sources/crude-oil/SOURCES.md`, section "Design practice and reference vessels". Web pages
  (no PDF) are cited by URL only.
- Rule texts that other agents own (ADN, MARPOL, ES-TRIN, CFR) are quoted here only where they set a design
  number that practice follows. See those agents' notes for the full rules.

---

## 1. Cargo properties that drive the design

### 1.1 Density

| Grade (example) | API gravity | SG at 15.6 °C (est., SG = 141.5 / (131.5 + API)) | Source of API |
|---|---|---|---|
| Peregrino (Brazil), heavy | 14.3 | 0.970 | [S1] |
| Roncador Heavy (Brazil) | 19.8 | 0.935 | [S1] |
| Doba Blend (Chad) | 20.5 | 0.931 (source gives 0.9308) | [S2] |
| Johan Sverdrup (Norway), medium | 28.3 | 0.886 | [S1] |
| Uganda (Lake Albert), waxy | about 30 | 0.876 | [S3] |
| Azeri Light | 35.3 | 0.848 | [S1] |
| Bakken (US), light | 43.5 | 0.809 | [S1] |
| Gudrun Blend (Norway), extra light | 52.9 | 0.767 | [S1] |
| Algerian Condensate (for the bound) | 68.6 | 0.707 | [S1] |

- Crude oils in the Equinor assay list run from 14.3 to 52.9 °API, so SG about 0.97 to 0.77 [S1].
  Condensates go higher (62.5 to 68.6 °API) [S1].
- Class practice for tank structure:
  - ABS Steel Barges 2024, 5-2-1/3: the scantling head uses ρ = 1.0 where the cargo SG is 1.05 or less.
    It uses the actual SG only above 1.05 [S20]. So every crude oil designs as SG 1.0 on an ABS barge.
  - Designer practice (Marine Engineering Bureau, river-sea tankers): "crude oil and petroleum products with
    density up to 1.015 t/m³" (RST12, RST12C, RST43C) and "0.7 up to 1.015 t/m³" (RST27M) [S30].
    A fuel-oil barge design states a density range of 0.92 to 1.015 t/m³ (ROB20) [S30].
- **Recommendation for OVCD (est.):** use SG 1.015 to 1.025 for tank scantlings and SG 0.85 (nominal) with a
  range of 0.77 to 0.97 for capacity and stability. Check stability at both ends of the range.
- Capacity ratio in built vessels (cargo m³ per tonne dwt): 1.00 to 1.27 m³/t (table in section 3, est.).
  A ratio near 1.18 means the vessel is full by volume at about SG 0.85 (est.).
- **US inland barges are volume-rich.** A typical 30,000 bbl (4,770 m³) barge carries 3,300 t [S28]. That is
  0.69 t/m³ if full (est.). With crude at SG 0.85 the barge reaches its draught before its tanks are full.
  Example: PBL 3422 (297.5 × 54 ft) carried 20,260.6 bbl (3,221 m³) of crude at 9.5 ft (2.90 m) draught [S11].
  This is about 68 % of the nominal 30,000 bbl (est.). So **partial fill is the normal loaded state** on these
  barges. This drives free surface and sloshing (section 5).

### 1.2 Viscosity, pour point and cargo heating

- Pour point varies widely:
  - Doba Blend: −4 °C pour point; viscosity 360 cSt at 40 °C, 185 cSt at 50 °C, 21 cSt at 100 °C [S2]
    (secondary report of a 2004 assay).
  - Uganda crude: pour point about 40 °C. The export pipeline keeps it at not less than 50 °C [S3].
- Pumpability: for efficient discharge the kinematic viscosity "should not exceed 250 centistokes and never in
  excess of 600 centistokes" [S16, MEPC.81(43)].
- Wax: if the cargo falls below its cloud point, expect sludge in the tanks [S16]. Approximate cloud point from
  pour point x: Cloud point °C = 20.2 (10^(0.00708x − 0.1157714)) + 8. Example: pour point +1 °C gives about
  24 °C [S16].
- For crude oil washing of paraffinic crude, keep the wash oil at least 10 °C above the cloud point [S16].
- Operating rule used by P&I clubs: keep cargo at least 10 °C above the pour point for the full voyage
  (Standard Club guidance, seen via search only; the PDF now redirects) (unverified). Britannia P&I says crude
  "traditionally" is heated in carriage, and high ROB and waxy sludge follow from heat loss [S17].
- **When heating is needed (est.):** when the lowest expected cargo temperature (sea or river water, plus
  voyage time) can fall below pour point + 10 °C, or when viscosity at that temperature exceeds 250 cSt.
- **Heating systems seen in built vessels:**
  - In-tank coils with a thermal-oil or steam boiler: river-sea tankers list "cargo heating: coils" and
    "thermal boilers" [S30]. B. No. 255 (ATB crude barge) had a "cargo thermal fluid heater" [S10].
  - Deck-mounted diesel-fired thermal-fluid heater on US inland barges: Bollinger hot-oil barges, 6 million
    BTU/h heater per 30,000 bbl barge [S24]; Conrad asphalt barges, 8 million BTU/h per 30,000 bbl [S25].
  - Deck-mounted heat exchangers (no in-tank coils) with steam, hot water or thermal oil (Framo type). Seen on
    a tanker in a Britannia case [S17].
- **Installed heat per m³ of cargo (est.):**

| Vessel | Cargo | Installed heat | Cargo m³ | kW/m³ (est.) | Basis |
|---|---|---|---|---|---|
| Bollinger 30,000 bbl hot-oil barge (2004) | hot oil, 6 tanks | 6 MMBTU/h = 1.76 MW | 4,770 | 0.37 | [S24] |
| Conrad PTC 2001/2002 (2020) | asphalt | 8 MMBTU/h = 2.34 MW | 4,770 | 0.49 | [S25] |
| MEB RST27 (river-sea, crude to 60 °C) | crude/products | boilers 2 × 2.5 t/h | 8,274 | 0.35 | [S30]; 1 t/h steam taken as 0.57 MW (latent heat at about 7 bar) |
| MEB RST12 (crude to 50 °C) | crude/products | boilers 2 × 2.0 t/h | 8,075 | 0.28 | [S30]; same basis |
| MEB RST05 | crude/products | boilers 2 × 1.0 t/h | 4,427 | 0.26 | [S30]; same basis |
| MEB RST11 | crude/products | boilers 2 × 1.0 t/h | 7,042 | 0.16 | [S30]; same basis |
| MEB RT63 (tanker-pusher, flash > 60 °C) | fuel oil | 2 × 800 kW | 2,435 | 0.66 | [S30] |

  These are totals with both boilers. Boilers also serve other users. Treat them as upper bounds.
- **Heat needed to hold temperature is much lower than installed heat.** A 7,360 m³ tank with RMH 45 fuel oil
  at about 55 °C (sea 5 °C, air 2 °C) lost 164 kW, of which 41.5 kW through the bottom [S31]. That is about
  0.022 kW/m³ (est.). The same study sizes coils for a heat-up from 44 to 66 °C in 4 days [S31]. That needs
  about 0.12 kW/m³ (est.: SG 0.99, cp 1.9 kJ/kg·K). Coil area was 82 m², or 0.011 m²/m³ (est.) [S31].
  Small barge tanks have more surface per m³, so expect higher loss per m³ (est.).
- **OVCD method (est.):** Q = Σ(U·A·ΔT) for hold, plus m·cp·ΔT / t for heat-up. Size the heater for the larger
  of heat-up and "heat while you discharge". Use the installed-heat table above as a sanity band
  (0.15 to 0.65 kW/m³).

### 1.3 Vapour pressure: venting, vapour return and inerting

- Bakken crude (AFPM survey, about 1,400 samples): RVP at 38 °C maximum 15.4 psia (106 kPa); vapour pressure at
  50 °C maximum 16.72 psia (115 kPa); flash point −59 to 50 °C; initial boiling point 2.2 to 66.9 °C;
  dissolved flammable gas up to 12.0 liquid-volume % [S4].
- US Coast Guard: RVP (ASTM D323) sets the cargo grade under 46 CFR. Tank barges get different structural
  standards when cargo RVP exceeds 25 psia (USCG Proceedings 2015, seen via search) (unverified).
  46 CFR 30.10-15: Grade D flash 80 to 150 °F; Grade E flash 150 °F or above (Cornell LII text).
- Built venting arrangements:
  - CTOW 330 (US, 2026): USCG-approved vapour recovery; 304 stainless vent header; closed gauging; high-velocity
    P/V valve at 3.0 psig (20.7 kPa) pressure and 0.5 psig (3.4 kPa) vacuum [S26].
  - Bollinger hot-oil barges: vapour pressure limit 3.0 psi [S24].
  - Changhang Barge 64011 series (Yangtze, 2026): oil vapour collection system to GB 20951-2020, and inert gas
    [S27].
- Inerting:
  - SOLAS II-2/4.5.5 (MSC.365(93)) requires a fixed inert gas system on tankers of 8,000 dwt and above built on or
    after 1 January 2016 (see `seagoing/solas/MSC.365-93.pdf`).
  - Coasters below that size often fit nitrogen anyway: 9 of 10 Uni-Tankers sheets under 13,000 dwt show "N2"
    or "flue gas"; Amak Swan (3,476 dwt, 2005) shows "no" [S12].
- Inland (ADN): UN 1267 crude oil rows are split by vapour pressure and boiling point and by type N or C; crude
  with more than 10 % benzene and UN 3494 sour crude go to type C with closed tanks [ADN Table C; inland-rules
  agent]. Vapour pressure at 50 °C therefore selects the tank type, the P/V valve setting and whether a vapour
  return line is needed.

### 1.4 H2S in sour crude

- Bakken vapour space: most H2S readings were below the OSHA STEL; one sample reached 23,000 ppm [S4].
- USCG Safety Alert 02-14 (Jan 2014): a Coast Guard member was seriously injured by H2S during an exam of a tank
  vessel with Grade E sour crude. It points to ISGOTT 2.3.6.5 (SCBA when a vapour-free atmosphere is not
  guaranteed) [web: dco.uscg.mil alert 0214; PDF did not download] (unverified).
- Coating test media include H2S: the PSPC-COT gas test uses 200 ± 20 ppm H2S; the immersion test oil carries
  5 ± 1 ppm w/w H2S [S18].
- Design effects (est.): closed gauging and closed sampling; gas detection for H2S; vapour return; toxic-cargo
  arrangements under ADN for UN 3494 (type C) [ADN]; coating rated for sour crude.

### 1.5 Temperature limits for coatings and structure

- PSPC-COT (MSC.288(87)) tests coatings at 60 ± 3 °C (gas phase) and 60 ± 2 °C (immersion) [S18].
- Built designs state carriage temperatures of up to 50 °C (RST12, RST25) or up to 60 °C (RST27, RST27M,
  RST43C) [S30].
- Structure:
  - ABS barges 5-2-1/1.3: give a temperature distribution when heated cargo is above 80 °C in tanks not on the
    sea; above 80 °C steel temperature, give thermal stress calculations (air 5 °C, sea 0 °C) [S20].
  - BV NR217 Pt D Ch 1 Sec 3 [5.1.3]: thermal stress calculation needed if carriage temperature exceeds 90 °C
    (water 0 °C, air 5 °C) [S21].
- Coating makers' ratings: Jotun product pages (search extract) give crude-oil resistance up to 120 °C (Tankguard
  Storage) and sour crude up to 90 °C (Tankguard HB Classic) (unverified; page needs login).
- **Practice band (est.):** design crude carriage at 50 to 60 °C. That stays inside the PSPC-COT test
  temperature and below the class thermal-stress triggers (80 to 90 °C).

---

## 2. Typical arrangements

### 2.1 Number and layout of cargo tanks

- River-sea tankers about 140 × 16.7 m: 6 cargo tanks + 2 slop tanks, 2 or 3 segregations (RST27, RST12, RST25,
  RST22M) [S30]. Six tanks over 16.7 m beam implies 3 pairs, port and starboard, on a centreline bulkhead (est.).
- Coastal product/chemical tankers 3,500 to 13,000 dwt: 10 to 16 cargo tanks including 2 slop tanks; number of
  segregations is often equal to the number of tanks (Uni-Tankers fleet) [S12].
- River tank barges (non-self-propelled): 8 tanks (Danube 1635T, 76.5 m), 10 + 2 slops, 12 + 2 slops, and 14 tanks
  (ROB21, 85 m) [S30].
- US inland 297.5 × 54 ft barges: 6 tanks (Bollinger hot-oil barge) [S24]; 8 compartments in 2 segregations
  (CTOW 330) [S26]. Kirby fleet records show 6 carbon-steel tanks on a 297 × 54 × 12 ft barge (unverified).
- Larger US barges: 14 tanks (LPB-32684, 326 × 84 ft) [S9]; 16 tanks (B. No. 255, 468 ft) [S10].
- Rhine tankers 110 × 11.45 m: 8 tanks (Generation) [S29a]; 6 tanks (Green Rhine) [S29b].
- Centreline bulkhead rule: BV NR217 Pt D Ch 1 Sec 3 [2.3.1]: where tank breadth exceeds 0.7 B, cargo tanks are
  normally to have centre longitudinal bulkheads; if not, give proof of intact stability [S21]. ADN gives extra
  stability criteria when cargo tanks are wider than 0.70 B (9.3.x.14.2) [ADN].

### 2.2 Double hull widths

Rule minima (the built widths are not published in the sources I found; see Gaps):

| Regime | Side (wing) width w | Double bottom h | Source |
|---|---|---|---|
| MARPOL I/19.3, oil tanker ≥ 5,000 dwt | 0.5 + DW/20,000, max 2.0 m, min 1.0 m | B/15, max 2.0 m, min 1.0 m | MEPC.117(52) |
| MARPOL I/19.6, oil tanker < 5,000 dwt | 0.4 + 2.4 DW/20,000, min 0.76 m (or no wings, if each tank ≤ 700 m³) | B/15, min 0.76 m | MEPC.117(52) |
| 33 CFR 157.10d(d), inland / limited short protected coastwise, < 10,000 dwt | min 61 cm (2 ft) | min 61 cm (2 ft) | Cornell LII text |
| ADN 9.3.2.11.7, type C integral tanks | min 1.00 m (0.80 m with stated reinforcement) | mean min 0.70 m, never < 0.60 m | ADN 2025 |
| ADN 9.3.3.11.7, type N double hull | min 0.60 m | min 0.50 m (0.40 m at pump sumps) | ADN 2025 |

- Example (est.): Amak Swan, 3,476 dwt, B 14.1 m [S12] → w = 0.82 m, h = 0.94 m by MARPOL I/19.6.
  RST27, 7,030 dwt, B 16.7 m [S30] → w = 1.0 m (minimum governs), h = 1.11 m.
- All US inland tank barges for bulk liquids are double hull (Kirby 10-K 2017) [web: sec.gov Kirby 10-K].
  US crude barges commonly have "type II/III" hulls under 46 CFR 32.63 [S24, S25].

### 2.3 Cargo pumps, stripping and slop tanks

- **Coasters: one deepwell pump per tank, deck-driven.** Uni-Tankers sheets list pump count equal to or close to
  tank count, for example Lillo Swan 12 × 300 + 2 × 100 m³/h for 14 tanks; Mona Swan 14 × 300 m³/h for 16 tanks
  [S12].
- **River-sea tankers:** 6 × 150 or 6 × 200 m³/h for 6 + 2 tanks (one per cargo tank) [S30]. Older designs use
  2 × 450 m³/h (RST05, RST11) [S30], which suggests a pump-room or common-line layout (est.).
- **Rhine tankers:** electrically driven deepwell pumps on deck, Green Rhine 4 × 200 + 2 × 120 m³/h [S29b].
- **US inland barges:** diesel-driven deck pumps.
  - Bollinger hot-oil barge: one Byron Jackson 3-stage deepwell 12 in pump and one IMO deepwell screw pump; each
    pump can discharge any tank; 530 hp diesels [S24].
  - CTOW 330: four diesel-driven pumps, about 4,000 bbl/h at 250 ft head [S26].
  - LPB-32684: four diesel engines in a centreline pump house drive four cargo pumps; up to four products at a
    time to port or starboard manifolds [S9].
- Screw pumps suit viscous cargo [S17].
- Slop tanks: 2 slop tanks are standard on river-sea and coastal tankers [S12, S30]. Slop volume on MEB designs:
  154 m³ (004ROB05, 3.3 % of cargo), 256 m³ (RST22, 3.3 %), 280 m³ (RST27 series), 340 m³ (ROB20, 5 %) [S30].
- Tank-wash and slop-tank pump: 80 to 100 m³/h on MEB designs [S30].

### 2.4 Cargo piping, manifold and venting

- MEB designs: 2 manifolds amidships + 1 aft starboard on the 3-grade versions (RST27M, RST43C) [S30].
- CTOW 330: two transverse cargo headers about 120 ft (37 m) forward of the stern; spool-piece cross-connection
  of the two segregations; spectacle flanges and dedicated vapour headers [S26].
- LPB-32684: manifolds port and starboard [S9].
- Venting: see 1.3.

### 2.5 Ballast

- River-sea tankers carry large ballast volumes: 0.44 to 0.62 m³ per m³ of cargo (est., from 001RST02 3,665 m³,
  RST11 3,215, RST25 4,349, ST19 2,310, RST05 1,950) [S30].
- River barges carry less: 0.18 to 0.28 m³/m³ (est., from ROB07 856 m³, 004ROB05 1,009, 005ROB04 1,600) [S30].
- Rhine tanker Generation: 1,320 m³ ballast for 3,040 m³ cargo (0.43) [S29a].
- US inland barges: no ballast data in the sources found (gap).

### 2.6 Cargo tank coating

- Coastal tankers in one owner fleet: Marineline (Amak Swan, Lillo Swan, Rio Fyn, Selandia Swan, Selena ATK,
  Katelina, Mona Swan), epoxy (Bothnia), stainless steel (Fandia Swan, Tina M) [S12].
- Crude oil tankers of 5,000 dwt and above fall under PSPC-COT (MSC.288(87)) [S18]. A maker's application guide
  for an epoxy COT system is at [S19].
- Danger: coating failure leads to through-bulkhead corrosion. B. No. 255 had 50 % hull coating failure; crude
  leaked through a corroded bulkhead into a void forepeak and exploded [S10].

---

## 3. Published reference vessels

**Full data** = LOA, beam, depth, draught, deadweight or cargo tonnes, cargo m³ and tank count all stated by the
source. 16 vessels/designs have full data (marked ✔). Basis of m³: "incl. slops" or "98 %" as the source states;
"nominal" where the source gives barrels without a fill basis. bbl → m³ at 0.158987.

### 3.1 Inland and river tank barges / tankers

| # | Vessel / design (builder or designer) | Region / type | LOA m | B m | D m | T m | DWT or cargo t | Cargo m³ (basis) | Tanks | Lightship t | Power kW | Notes | Src |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 ✔ | 1635T/550A modernisation (MEB) | Danube, non-self-prop. | 76.50 | 11.02 | 3.90 | 3.0 | 1,907 cargo | 2,022.4 | 8 | about 600 (est., light draught 0.77 m) | — | products flash > 60 °C | [S30] |
| 2 ✔ | ROB21 (MEB) | river, non-self-prop., small draught | 85.0 | 17.20 | 2.60 | 1.80 (DWL); dwt table to 2.35 | 1,801 at 1.80 m; 2,555 at 2.35 m | 3,257 (98 %) | 14 | about 710 (est., dwt curve extrapolated to zero) | — | 7 × 100 m³/h pumps | [S30] |
| 3 ✔ | RT63 (MEB) | river tanker-pusher | 85.70 | 17.20 | 2.60 | 1.80; table to 2.35 | 1,310 at 1.80 m; 2,064 at 2.35 m | 2,435 | 10 | about 1,130 (est., same method) | 2 × 746 | thermal boilers 2 × 800 kW | [S30] |
| 4 ✔ | ROB20 (MEB) | river O 2.0, non-self-prop. | 118.79 | 22.80 | 4.0 (scantling) | 2.75 | 6,000 | 6,870 (incl. 340 slop) | 12 + 2 | — | — | fuel oil, SG 0.92–1.015 | [S30] |
| 5 ✔ | 004ROB05 (MEB) | river-sea, non-self-prop., crude | 93.0 | 16.70 | 5.0 | 3.60 | 4,324 cargo | 4,620 (incl. 154 slop) | 12 + 2 | — | — | 2 × 300 m³/h | [S30] |
| 6 ✔ | 005ROB04 (MEB) | river-sea, non-self-prop., crude | 121.80 | 16.80 | 5.0 | 3.60 | 5,125 cargo | 5,780 (incl. slops) | 10 + 2 | — | bow thr. 257 | 2 × 400 m³/h | [S30] |
| 7 ✔ | ROB07 (MEB) | river-sea, non-self-prop., crude | 95.23 | 16.80 | 5.0 | 3.60 (SWL) | 4,403 dwt | 4,670 (incl. slops) | 12 + 2 | — | — | 5 × 150 m³/h | [S30] |
| 8 | Project 16802 (Vympel) | Russian river, fuel oil, non-self-prop. | 86.3 | 17.0 | 3.3 | 2.6–2.95 by basin | 2,490–2,970 cargo | 3,300 | — | about 745 (est., displacement minus cargo, all basins) | — | no heating | [S32] |
| 9 ✔ | CDS Tanker 110, mts Generation (Concordia Damen) | Rhine, self-prop. | 110 | 11.45 | 4.9 | 3.3 (1.2 min) | 2,868 cargo | 3,040 | 8 | — | not stated (hybrid) | ballast 1,320 m³ | [S29a] |
| 10 | Green Rhine (Peters Shipyards, 2013) | Rhine, self-prop. | 110.00 | 11.44 | — | 3.46 | 2,877 t | 3,127 | 6 | — | gensets 4 × 300; Z-drives 2 × 500 | double hull; 6 elec. pumps | [S29b] |
| 11 | PBL 3422 (2013, Savage) | US inland, crude | 90.7 (297.5 ft) | 16.5 (54 ft) | — | 2.9 (9.5 ft) actual | — | 3,221 actual load (20,260.6 bbl) | ≥ 3 | — | — | 1,616 GRT | [S11] |
| 12 | Horizon Maritime hot-oil barges (Bollinger, 2005) | US inland | 90.68 (297.5 ft) | 16.46 | 3.96 (13 ft) + 1.07 trunk | — | — | 4,770 nominal (30,000 bbl) | 6 | — | 2 × 530 hp pump diesels | heater 6 MMBTU/h | [S24] |
| 13 | PTC 2001 / 2002 (Conrad, 2020) | US inland, asphalt | 90.68 | 16.46 | 3.66 (12 ft) | — | — | 4,770 nominal | — | — | 99 kW gen. | heater 8 MMBTU/h; double skin | [S25] |
| 14 | CTOW 330 (C&C Marine, 2026) | US, bunker | 90.5 | 16.46 | 4.88 (16 ft) | — | — | 4,452 nominal (28,000 bbl) | 8 (2 segr.) | — | side thruster 450 | P/V 3.0/0.5 psig | [S26] |
| 15 | Generic 30,000 bbl inland barge (Kirby) | US inland | 90.68 (297.5 ft) | 16.46 | 3.66 (12 ft) | — | 3,300 t cargo "typical" | 4,770 nominal | — | — | — | — | [S28], dims [S24, S25] |
| 16 | Balsa "Lumpsum" (Grupo Atem) | Amazon, fuel | 108.16 | 30 | — | — | — | 12,000 (12 million L) | — | — | — | — | [S8] |
| 17 | Changhang Barge 64011–64015 (Hubei Jiangrun; CCS) | Yangtze, crude and products, pushed | 67.25 | — | — | — | about 4,200 cargo | — | — | — | — | VRU, inert gas, own pumps | [S27] |

### 3.2 Sea-going tank barges and ATBs

| # | Vessel | LOA m | B m | D m | T m | Cargo m³ (basis) | Tanks | Notes | Src |
|---|---|---|---|---|---|---|---|---|---|
| 18 | Crowley B-20850, 10,000 bbl shallow-draft ATB barge | 63.40 (208 ft) | 15.24 | 2.74 (9 ft) | — | 1,590 diesel "max available" (10,000 bbl); 5,000 bbl gasoline | — | tug 77'8" × 32 ft, 4 ft draught | [S6] |
| 19 | Crowley B-18350, 8,000 bbl ATB barge | 55.78 (183 ft) | 15.24 | 2.74 | — | 1,272 diesel (8,000 bbl); 3,000 bbl gasoline | — | same tug | [S7] |
| 20 | Crowley LPB-32684, petroleum barge | 99.36 (326 ft) | 25.60 | 8.53 | 5.89 | 11,320 (71,200 bbl) | 14 | 5,765 GT ITC; centreline pump house, 4 diesel pumps; ABS A1 Oil Tank Barge | [S9] |
| 21 | B. No. 255 (1979, double-hulled 1999), crude ATB barge | 142.65 (468 ft) | 26.06 | 11.61 (38.1 ft, listed as "draft"; probably depth, est.) | — | 29,953 (188,400 bbl, USCG documented) | 16 | 9,834 GT; thermal fluid heater | [S10] |

### 3.3 River-sea and coastal tankers (self-propelled)

| # | Vessel / design | LOA m | B m | D m | T m | DWT t | Cargo m³ (basis) | Tanks (cargo/slop) | Power kW (ME) | Heating / IG | Src |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 22 ✔ | RST27 (MEB; built by Krasnoye Sormovo and Oka yards per PortNews, https://en.portnews.ru/news/135470) | 140.85 | 16.70 | 6.0 | 4.20 sea / 3.60 river | 7,030 / 5,428 | 8,274 (incl. slops) | 6 / 2 | 2 × 1,200 | coils; boilers 2 × 2.5 t/h; cargo to 60 °C | [S30] |
| 23 ✔ | RST12 (MEB) | 141.0 | 16.98 | 5.50 | 4.17 / 3.60 | 7,183 / 5,632 | 8,075 (incl. slops) | 6 / 2 | 2 × 1,200 | coils; boilers 2 × 2.0 t/h; to 50 °C | [S30] |
| 24 ✔ | RST25 (MEB) | 139.99 | 16.60 | 5.50 | 4.175 / 3.60 | 6,710 / 5,229 | 6,990 (incl. slops) | 6 / 2 | 2 × 1,200 | boilers 2 × 2 t/h; to 50 °C | [S30] |
| 25 ✔ | 005RST01 (MEB) | 138.70 | 16.76 | 6.0 (+1.0 trunk) | 4.30 / 3.60 | 6,444 / 4,726 | 7,196 | 6 | 2 × 1,080 | coils; boilers 2 × 1.2 t/h | [S30] |
| 26 ✔ | RST05 (MEB; PRS "Crude oil tanker") | 118.87 | 13.24 | 5.80 | 4.3 / 3.6 | 3,955 / 2,868 | 4,427 (incl. slops) | 8 / 2 | 2 × 567 | boilers 2 × 1.0 t/h | [S30] |
| 27 ✔ | RST11 (MEB) | 131.64 | 16.75 | 5.50 | 4.477 / 3.60 | 6,586 / 4,677 | 7,042 (incl. slops) | 8 / 2 | 2 × 736 | boilers 2 × 1.0 t/h | [S30] |
| 28 ✔ | ST19 (MEB) | 114.40 | 16.30 | 6.10 | 4.16 / 3.60 | 4,320 / 3,270 | 4,150 (incl. slops) | 10 / 2 | 2 × 1,020 | — | [S30] |
| 29 ✔ | RST43C (MEB; ABS restricted service) | 149.95 | 17.30 | 7.50 | 5.59 | 11,053 | 11,447 (98 %, incl. slops) | 8 / 2 | 2 × 1,600 | coils; to 60 °C | [S30] |
| 30 | Amak Swan (2005, BV), oil tanker | 92.86 | 14.10 | — | 5.70 | 3,476 | 4,085 | 12 / 2 | — | IG: no; Marineline | [S12] |
| 31 | Lillo Swan (2007, BV) | 99.90 | 15.00 | — | 6.04 | 4,696 | 5,087 | 12 / 2 | — | N2; Marineline | [S12] |
| 32 | Rio Fyn (2009, BV) | 105.50 | 16.80 | — | 6.28 | 5,717 | 6,290 | 10 / 2 | — | N2; Marineline | [S12] |
| 33 | Selandia Swan (2017, BV) | 99.99 | 16.60 | — | 6.70 | 6,122 | 6,121 | 10 / 2 | — | N2; Marineline | [S12] |
| 34 | Selena ATK (2021, BV) | 99.99 | 16.60 | — | 6.70 | 6,043 | 6,133 | 8 / 2 | — | N2; Marineline | [S12] |
| 35 | Tina M (2024, BV) | 99.99 | 16.60 | — | 6.70 | 6,057 | 6,293 | 12 / 2 | — | N2; stainless | [S12] |
| 36 | Bothnia (2013, GL) | 118.16 | 18.50 | — | 7.90 | 8,985 | 9,545 | 10 / 2 | — | N2; epoxy | [S12] |
| 37 | Katelina (2014, DNV) | 118.16 | 18.50 | — | 7.90 | 8,933 | 9,504 | 10 / 2 | — | flue gas; Marineline | [S12] |
| 38 | Mona Swan (2006, ABS) | 129.75 | 19.60 | — | 8.00 | 11,348 | 12,396 | 14 / 2 | — | N2; Marineline | [S12] |
| 39 | Fandia Swan (2017, NK) | 126.76 | 20.80 | — | 8.10 | 12,662 | 13,850 | 14 / 2 | — | N2; stainless | [S12] |

Notes on the tables:
- Uni-Tankers sheets give LBP and gross tonnage too (see the PDFs). They give no depth and no power.
- Uni-Tankers vessels are chemical/oil (product) tankers, not crude carriers. They are hull-form and capacity
  calibration cases only. MEB designs 22–29 and barges 5–7 state crude oil in their purpose [S30].
- I left out Celsius Middelfart (19,997 dwt) as above the size range; the sheet is saved [S12].
- MEB gives many more projects (RST22, RST22M, RST07, RST12C, RST27M, RST34 combined, ST21, ROB17, RT18). Their
  data are on the same pages [S30].

---

## 4. Weight and cost data

### 4.1 Lightship and steel weight

No source in this search gives a split of steel, outfit and machinery for a tank barge or small tanker.
Lightship estimates from published hydrostatics (all est.):

| Vessel | Method | Lightship t (est.) | t per m³ cargo | t per t dwt |
|---|---|---|---|---|
| Vympel 16802 river fuel-oil barge | displacement 3,715 t − cargo 2,970 t (and 3,235 − 2,490 in M basin; both give 745 t) [S32] | 745 | 0.23 | 0.25 |
| MEB ROB21 river barge | linear dwt-vs-draught table, 1,390 t/m, zero dwt at T = 0.51 m [S30] | about 710 | 0.22 | 0.28 |
| MEB 1635T Danube barge | light draught 0.77 m × about 855 t/m (1,907 t over 2.23 m) [S30] | about 600–660 | 0.30–0.33 | 0.31–0.35 |
| MEB RT63 tanker-pusher (self-prop.) | dwt table, 1,359 t/m, zero dwt at T = 0.83 m [S30] | about 1,130 | 0.46 | 0.55 |

- **Band for OVCD (est.):** non-self-propelled river tank barge lightship 0.22 to 0.33 t per m³ of cargo.
  Self-propelled shallow-draught tanker about 0.45 t/m³. These include outfit; the barges have little machinery.
- A Kirby fleet record (search extract only) gives a 297 × 54 × 12 ft, 30,481 bbl barge with 2,515 t at 7 ft,
  3,456 t at 9 ft and 4,935 t at 12 ft (unverified; site refused connection). If confirmed, it would give a
  lightship near 750 short tons (est.). Do not calibrate on it until read from the record or a COI.
- Outfit for pumps, heating and piping: no published weights found. Use maker data at the next stage.
  Proxies (est.): one deck-mounted diesel-fired thermal-fluid heater of 6 to 8 MMBTU/h per 30,000 bbl barge
  [S24, S25]; 2 to 4 diesel pump sets per barge [S24, S26, S9].

### 4.2 Cost

| Contract | Data | Derived (est.) | Src |
|---|---|---|---|
| Kirby / Trinity, 2003 | 16 double-hull 30,000 bbl inland black oil barges, about $29 million | $1.81 M per barge; $60/bbl; $380/m³ (2003 USD) | [web: marinelink 2003] |
| Kirby, 2014 | 29 × 30,000 bbl inland tank barges, about $81 million, delivery 2014; total 870,000 bbl (gCaptain) or 830,000 bbl (WorkBoat) | $2.79 M per barge; $93–98/bbl; $585–615/m³; about $846 per t of typical cargo (3,300 t [S28]) (2014 USD) | [web: gCaptain, WorkBoat 2014] |

- Context: Kirby tied the 2014 order to growth in crude oil and condensate movements [web: gCaptain 2014].
- No published cost was found for coastal tankers of 500 to 10,000 dwt, or for heated crude barges (gap).

---

## 5. Stability and structure points at concept stage

### 5.1 Free surface with many tanks

- Correct GM and GZ for free surface at every filling level; full tanks may be ignored (BV NR217 Pt B Ch 2 Sec 2
  [2.3]) [S21].
- MARPOL I/27 (oil tankers ≥ 5,000 dwt): GM0 ≥ 0.15 m at all stages including liquid transfer; assume ballast
  tanks slack; meet it by design, not by procedures alone (MEPC.117(52)).
- ADN 9.3.3.13.3: proof of intact stability for all stages of loading and unloading and for all cargo relative
  densities on the substance list [ADN].
- Hydrostatics (standard result): free-surface moment of a rectangular tank goes with b³. One centreline
  bulkhead makes two tanks of b/2, so the total moment falls to 1/4. Two longitudinal bulkheads (three tanks of
  b/3) cut it to 1/9.
- US inland barges load to draught, not to volume (section 1.1). So most cargo tanks are slack in the loaded
  condition. OVCD should compute free surface for all slack tanks in every loaded case, not only in transit
  stages (est.).

### 5.2 Effect of the centreline bulkhead

- Required in practice when tank breadth > 0.7 B (BV NR217 Pt D Ch 1 Sec 3 [2.3.1]) [S21]; extra criteria for
  tanks > 0.70 B in ADN 9.3.x.14.2 (GZ ≥ 0.10 m; area ≥ 0.024 m·rad up to the first non-weathertight opening or
  27°) [ADN].
- It also supports deck and bottom, and it sets the segregation layout (pairs of tanks, port and starboard).
- Risk: an unequal port and starboard fill gives a list. Load and discharge pairs together (est.).

### 5.3 Loading sequences

- MARPOL I/27.3 allows only "simple supplementary operational procedures", approved, that list which tanks may be
  slack (MEPC.117(52)).
- BV NR217 needs a loading manual with permitted still-water bending moments and shear forces, and a loading
  instrument where required [S21, Pt B Ch 2 / Ch 3].
- OVCD should model at least: departure full, arrival, ballast, and 25/50/75 % intermediate stages of a
  sequential discharge, with the worst free surface (est.).

### 5.4 Sloshing at partial fills

- IACS CSR trigger (oil tankers ≥ 150 m): assess sloshing impact where the effective sloshing breadth exceeds
  0.56 B or the effective length exceeds 0.13 L, at any fill from 0.05 to 0.95 of tank height [S22].
- The inland rules read for this task (BV NR217, ABS barges) have no sloshing check for oil tanks [S20, S21].
- Concept-stage check (est.): flag any cargo tank with length > 0.13 L or breadth > 0.56 B. For US-type barges
  that run slack (section 1.1), consider swash bulkheads or shorter tanks.

### 5.5 Longitudinal strength of long barges

- BV NR217 Pt B Ch 3 Sec 2 [2]: closed-form still-water bending moment estimates for non-propelled cargo carriers.
  They are not valid for L > 135 m, unusual designs, or non-homogeneous loading other than the standard cases;
  then a direct calculation is needed [S21].
- Partial loading in alternate tanks and draught-limited loading both give non-homogeneous load. Use direct
  calculation of the SWBM from the tank plan for all barges (est.).
- Heated cargo adds thermal stress: ABS above 80 °C [S20]; BV above 90 °C [S21] (section 1.5).
- Corrosion and coating loss reduce section modulus and allow cargo to leak into voids (B. No. 255) [S10].

---

## 6. Main design drivers (summary)

1. **Density range 0.77 to 0.97 (SG)** with scantlings at SG 1.0 to 1.025. Capacity ratio 1.0 to 1.27 m³/t.
2. **Draught-limited partial fill** on volume-rich inland barges → free surface and sloshing in the loaded case.
3. **Pour point and viscosity** → heating (0.15 to 0.65 kW/m³ installed in built vessels; hold losses about
   0.02 kW/m³ in a large tank) and screw or deepwell pumps.
4. **Vapour pressure (up to about 115 kPa at 50 °C for light crude)** → tank type, P/V setting, vapour return,
   inerting threshold (8,000 dwt seagoing).
5. **H2S** in sour crude → closed operations, gas detection, coating selection, ADN type C.
6. **Double hull minima** (0.61 m US inland, 0.60/0.50 m ADN N, 1.0/0.70 m ADN C, 0.76 m or more MARPOL).
7. **Tank breadth > 0.7 B → centreline bulkhead.**

## 7. Gaps

- No published as-built double-hull widths for any reference vessel.
- No steel / outfit / machinery weight split; lightship only by estimate (4 cases). No lightship for any US
  30,000 bbl barge from a source I could open (Kirby vessel records refused connection).
- No vessel-level data for Niger Delta crude barges; Amazon and Yangtze entries are partial.
- No published heating-coil area or heat-load design basis for an inland crude barge.
- No cost data for coastal tankers or heated barges; barge costs are 2003 and 2014 USD only.
- Grades A–C (RVP) definitions in 46 CFR not read here; USCG "RVP > 25 psia" statement and Safety Alert 02-14
  seen only through search.
- Leads not pursued: USACE "Waterborne Transportation Lines of the US, Vol. 3 Vessel Characteristics" (light and
  loaded draughts for every US tank barge; ROSAP download blocked a script); Kirby COI PDFs; Environment Canada
  oil properties catalogue (PDF blocked a script).

---

## Sources

Saved PDFs (`~/Dev/ovcd-rule-sources/crude-oil/practice/`, logged with sha256 in `crude-oil/SOURCES.md`):

- [S4] AFPM, *A Survey of Bakken Crude Oil Characteristics* (2014), EPA NRT copy — `afpm-crude-survey/`
- [S5] NOAA, Bakken first responders guide (2016) — `noaa-bakken-guide/` (background only)
- [S6] Crowley Engineering, 10,000 bbl shallow-draft ATB spec sheet (2024) — `crowley-10k-atb/`
- [S7] Crowley Engineering, 8,000 bbl ATB spec sheet (2024) — `crowley-8k-atb/`
- [S8] Grupo Atem, MME workshop presentation on Amazon low water (2024) — `grupo-atem-estiagem-2024/`
- [S9] Crowley Engineering, LPB-32684 326 × 84 ft petroleum barge spec sheet (2025) — `crowley-lpb-32684/`
- [S10] NTSB MAB-19/07, Buster Bouchard / B. No. 255 (2019) — `ntsb-mab1907-b255/`
- [S11] NTSB MAB-21/06, Savage Voyager / PBL 3422 (2021) — `ntsb-mab2106-pbl3422/`
- [S12] Uni-Tankers owner fact sheets (11 vessels) — `uni-tankers-*/`
- [S13] NTSB MAB-15/12, tank barge Nash (2015) — `ntsb-mab1512/` (single-hull tank barge, not crude; not used in tables)
- [S16] IMO Res. MEPC.81(43), COW manual Section 9 — `imo-mepc-81-43-cow/`
- [S17] Britannia P&I, Risk Watch Vol 22 No 1 (2025) — `britannia-risk-watch-22-1/`
- [S18] IMO Res. MSC.288(87), PSPC for cargo oil tanks — `imo-msc-288-87-pspc-cot/`
- [S19] AkzoNobel International, Interline 704HS cargo oil tank guidelines — `akzonobel-interline-704hs-cot/`
- [S22] Türk Loydu circular S-P-01/20, sloshing impact assessment — `turk-loydu-s-p-01-20/`
- [S28] Kirby Corporation investor roadshow, Feb 2018 — `kirby-roadshow-2018-02/`

Already in the library (read, not downloaded again):

- [S20] ABS Rules for Building and Classing Steel Barges 2024 — `../abs-barges/10-barge-rules-jan24.pdf` (5-2-1/1.3, 5-2-1/3)
- [S21] BV NR217 Inland Navigation Vessels, June 2025, Parts B and D — `../bv-nr217/`
- ADN 2025 — `../adn/adn.pdf` (9.3.2.11.7, 9.3.3.11.7, 9.3.3.13, 9.3.x.14.2, Table C UN 1267 / 3494)
- MEPC.117(52) revised MARPOL Annex I — `crude-oil/seagoing/marpol-i/` (reg. 19, 27)
- MSC.365(93) SOLAS II-2/4.5.5 — `crude-oil/seagoing/solas/`

Web pages (no PDF; retrieved 2026-10-20):

- [S1] Equinor crude oil assays list — https://www.equinor.com/energy/crude-oil-assays
- [S2] Oil & Gas Journal, "ChevronTexaco assays Doba crude" (2004) — https://ogj.com/general-interest/companies/article/17231967/chevron-texaco-assays-doba-crude (via search extract; secondary)
- [S3] EACOP project pages (crude ≥ 50 °C; pour point about 40 °C) — https://eacop.com/faqs/ , https://www.eacop.com/?p=23225 (via search extract)
- [S24] MarineLink, "Bollinger to design, build five DH hot oil barges" (25 Oct 2004) — https://marinelink.com/news/bollinger-design-barges317362
- [S25] Maritime Executive, Conrad delivers two asphalt barges to Parker Towing (2020) — https://maritime-executive.com/corporate/conrad-shipyard-delivers-two-inland-asphalt-barges-to-parker-towing
- [S26] Baird Maritime, vessel review CTOW 330 (23 Apr 2026) — https://www.bairdmaritime.com/amp/story/shipping/tankers/vessel-review-ctow-330-bunkering-barge-with-28000-barrel-capacity-and-enhanced-safety-features
- [S27] Baird Maritime, Changhang Barge Yangtze oil transport (6 Oct 2026) — https://www.bairdmaritime.com/shipping/tankers/changhang-barge-yangtze-oil-transport
- [S29a] RINA *The Naval Architect*, "Damen hybrid-electric tanker takes to the Rhine" — https://rina.org.uk/publications/the-naval-architect/damen-hybrid-electric-tanker-takes-to-the-rhine
- [S29b] Scheepvaartwest, Green Rhine (ENI 02335378) — https://scheepvaartwest.be/CMS/index.php/tankers/4918-green-rhine-eni-02335378
- [S30] Marine Engineering Bureau project pages, oil tankers — index https://meb.com.ua/projects_tnkrs.html ; pages `https://meb.com.ua/oil/<PROJECT>-e.html` (RST27, RST27M, RST12, RST12C, RST25, RST05, RST11, ST19, RST43C, 005RST01, 001RST02, RST22, ROB20, ROB21, RT63, 004ROB05, 005ROB04, ROB07, 1635T)
- [S31] Magazinović, "Multi-level coil bundle for efficient cargo tank heating", *J. Marine Science and Application* 20(3), 2021, doi:10.1007/s11804-021-00215-0 — https://html.rhhz.net/jmsa/html/20210306.htm
- [S32] Vympel Design Bureau, Project 16802 fuel-oil barge — https://vympel.ru/en/projects/suda-tekhnicheskogo-flota-barzhi/nalivnaya-barzha-gruzopodemnostyu-2500-t-proekt-16802/
- Kirby 10-K for 2017 (fleet, double hull, heating) — https://www.sec.gov/Archives/edgar/data/56047/000114036118010152/form10k.htm
- Kirby 2003 order — https://www.marinelink.com/news/industries-construct301271 ; 2014 order — https://gcaptain.com/kirby-triples-2014-new-tank-barge-capacity/ , https://www.workboat.com/coastal-inland-waterways/kirby-to-build-29-more-30000-bbl-tank-barges
- 33 CFR 157.10d and 46 CFR 30.10-15, 32.63-5 — https://www.law.cornell.edu/cfr/text/33/157.10d (and sister pages)
