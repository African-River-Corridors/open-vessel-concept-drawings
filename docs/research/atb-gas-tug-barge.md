# Research note: an articulated tug-barge (ATB) for a coastal liquefied-gas barge

Research for spec 0014 (`docs/specs/0014-atb-gas-tug-barge.md`), issue to follow.

**Research line.**
*Searched:* OVD's own code, specs, research note and calibration for ATBs; coupler makers' pages
(Intercon, JAK, Articouple, Bludworth); ABS Steel Barges 2024 Part 5 Ch 2 Sec 5 and Part 5 Ch 3 Sec 1;
IMO MSC/Circ.866; IGC 2014 (MSC.370(93)); NVIC 2-81; built LNG bunker ATBs (spec sheets and reviews);
the 2025 open-access ATB review (Yanakiev, Garbatov, Georgiev); Gulf of Guinea wave-climate papers.
*Exists:* OVD already has the `atb` Formation, a tug sized from the barge, a Notch cut in the sections
table, an ATB Equivalent Hull with a 1.05 allowance, a coupling-force note, ABS notes for "two vessels"
and "notch wing walls", and three reference ATBs. ABS has full ATB classification text and a
liquefied-gas tank barge section. Intercon publishes coupler ratings by tug displacement and Hs.
Four LNG bunker ATBs are built or designed in the US, all with diesel tugs.
*Reusing:* OVD `formation`, `atb`, `stern_form::apply_notch`, `propulsor`, `machinery`, the rules
engine and the reference-vessel calibration pattern; ABS 5-3-1 and 5-2-5 text already in the rule
source library; the Intercon ratings table as the first coupler catalogue.
*Building because:* nothing in OVD picks a coupler, checks a sea state, applies the gas separation
between a tug and a gas barge, compares an ATB with a self-propelled hull, or calibrates ATB power.
OVD's own test shows its ATB power is 0.29 to 0.43 of the published installed power of the reference
ATBs, so a number for a gas ATB cannot ship without that calibration.

## 1. What OVD models today (origin/main 5a0fb99a)

| Item | Where | State |
|---|---|---|
| Formation kind `atb`, `tug_included` | spec 0007, #357 (closed) | built |
| Tug from the barge: L = 0.28 L_barge, B = 0.53 B_barge, T = B_tug / 2.33 | `crates/ovd-core/src/atb.rs`, #362 (closed) | built; ratios are estimates (Crowley sheets, Yanakiev §4.1) |
| Notch: depth 0.50 L_tug, width B_tug + 2 × 0.15 m, full-depth slot in the sections table | `atb.rs`, `stern_form::apply_notch`, #465 | built; one data point (Legacy) |
| Equivalent Hull: L_barge + L_tug − notch depth; barge beam and draught | `formation.rs` | built; method `unverified` |
| Formation allowance ATB 1.05 | Defaults Profile | `sourced` (Yanakiev Table 2, "rigidly linked … 5 % increase in resistance") applied as an estimate |
| Tug sizing loop to the Formation's power | #359 (open) | part built (#615); ES-TRIN 5.06 Measure open |
| ATB rule notes: ABS 5-3-1/1.3 two vessels; 5-3-1/11.3 notch wing walls | #363 (closed) | notes only (approval items) |
| ATB drawings (arrangement plan, notch) | #363, #360 | built |
| Reference ATBs: Aveogan, 550 Class, Kim M. Bouchard | `calibration/atb.yaml` | three cases, barge depths estimated |
| ATB power against the references | `crates/ovd-format/tests/atb_powering.rs` | **model gives 0.29–0.43 of published installed power (0.31–0.46 with #649 margins); the test band is 0.2–0.6** |
| Barge resistance method | ADR 0024, #736 | `barge_form_factor` default; range L/B 7.6–16.3, Cp 0.77–0.88, Fn ≤ 0.137 |
| Out of scope in 0007 | spec 0007 "Out of Scope" | tug stability (IS Code B 2.8, 46 CFR 173.095), coupler loads and sea-state limits, notch wing-wall analysis, connector lifetime loads |

**Gaps.** No coupler. No sea state. No tug intact stability in tow or push. No gas interface (where
the tug's openings are against the barge's cargo area and vent mast). No boil-off-to-tug fuel path.
No ATB-versus-self-propelled comparison. ATB power uncalibrated against any gas ATB. The tug draught
ratio ignores the barge's draught and the channel.

**A check on a wide, shallow gas barge (inference, from the ratios above).** On a barge of
L/B ≈ 4.8 and laden draught ≈ 0.15 B, the default tug draught B_tug/2.33 comes out about 1.5 times the
barge's laden draught. Built LNG ATB tugs draw 16–18 ft (4.9–5.5 m) against barge draughts of
13–25 ft (§4). The ratio needs a cap at the barge draught, or at the channel limit, or both.

## 2. Couplers

| System | How it connects | Published rating | Source |
|---|---|---|---|
| Intercon (Model 21, 34, 50, 64) | Two rams on the tug engage toothed or wave "helmets" in the notch sides; one degree of freedom, pitch about a transverse axis | Max tug displacement 400 / 1,250 / 2,300 / 4,200 LT; **max significant wave height 16.4 ft (21") and 23.0 ft (34", 50", 64")**; engagement force 100,000–300,000 lb; installed weight 24,000–365,000 lb; envelope up to 36 × 12 × 9 ft | [intercon.com/models](https://intercon.com/models/) (maker; read 2026-10-30) |
| Intercon (general) | "Engagement allowed regardless of draft or trim"; "USCG approved for Dual Mode under NAVIC 2-81" | — | [intercon.com/advantages](https://intercon.com/advantages/) (maker) |
| JAK (Beacon Finland) | Pneumatic (or hydraulic) pins from the tug bow into socket plates welded to both notch sides; bow "blister" on the tug; optional HYDRALOK lock | "For ATB Tugs with 350 to 10 000 horsepower"; size picked from VTT model tests per tug/barge/sea state; no Hs published | JAK brochure 2023 (`ovcd-rule-sources/stern-tug/tugs/jak-atb-coupler`, SHA-256 in SOURCES-tugs.md) |
| Bludworth-Cook | Hydraulic caliper at the tug bow on a vertical bar at the notch apex; pads on the tug sides in a deep notch | none found | Flanagan, SNAME Maritime Technology Conference 2005 (slides) |
| Articouple (Taisei) | Hydraulic rams; "unlimited-draft-connection" | none found from the maker (site certificate error). Forum claim of 15 ft seas: **not used** | Yanakiev 2025 §1, §4.2 |
| Fleet count (US flag, 2017, MARAD table quoted in trade press) | Intercon ≈ 100, JAK 37, Bludworth 23, Articouple 22 | — | secondary; not verified at MARAD |

**Notch geometry.** No rule sets a notch ratio. No maker sheet read gives a notch depth or width in
figures. OVD's 0.50 L_tug is one deep-notch ocean ATB (derived). The Intercon envelope (beam 14–36 ft)
fixes the coupler space in the tug, not the notch. Built LNG ATBs: tug 128 ft on a 324 ft barge
(combined length not stated) and tug 128 ft with a 372 ft combined length on a barge whose length is
not stated (§4); neither gives the notch. **Gap kept.**

## 3. Rules and class

### 3.1 One vessel or two

- **IMO MSC/Circ.866 (1 June 1998)**, Guidelines for the application of safety standards to pusher
  tug-barge combinations. Type B: "The connection system should permit the pusher to move in
  relation to the barge" and to be disconnected "by one person in less than 5 minutes" (1.2.1–1.2.2).
  Type B "could be applied … as to two separate ships" (2.2.1); the pusher must be able to tow the
  barge on a hawser (2.2.2); the master must know "the maximum sea states at which the pusher can
  routinely disconnect and safely reconnect" (2.2.4). Type A (rigid, or remote-controlled from the
  barge) is one ship sized on the whole combination (2.1.1). The cargo codes listed in 2.2.3 do not
  include the IGC Code. (Copy read: Turkish maritime authority's hosted PDF.)
- **ABS Steel Barges 2024, 5-3-1/1.3**: "The tug and the barge are to be classed as two separate
  vessels". Tug: ✠A1 Towing Vessel ATB (dual mode, full compliance with MVR Part 5C Ch 17,
  including intact stability in tow, 5C-17-2). Barge: ✠A1 Barge, ATB. Combined length Lc (1.1.3).
- **ABS 5-3-1/5.3**: an ATB barge's longitudinal strength uses the barge alone. **7.3**: barge scantlings
  on barge length L; a ship-like barge may use MVR Part 5C or the LGC Guide instead. **7.7.3**: anchor
  equipment on the barge alone. **9.3**: still-water bending moment and shear with and without the tug.
- **ABS 5-3-1/11.1**: connector design on "maximum lifetime connector loads", from a scale-model test
  programme (RAOs, worst heading, long-term extreme) or a multi-body motion analysis; fatigue life not
  less than the barge's; a service record may replace the calculation (11.1.2). **11.3**: FEA of the
  structure in way of the connection; for a notch, wing walls checked for torsional shear and primary
  and secondary shear and bending.
- **USCG NVIC 2-81** Change 1 (read): dual-mode tug and barge are separate vessels for inspection;
  disconnection in about five minutes. Change 2 (July 2023) adds the term ATB; **not read** (HTTP 403).
- **DNV, BV.** Not read to the clause. DNV has "Tug"/"Pusher" notations (RU-SHIP Pt 5 Ch 6, from a
  secondary listing). BV NR467 has a section titled "Integrated Tug / Barge Combination" (search
  snippet only). **Gap**: read both before a Rule Set is written; ABS is the only text read in full.

### 3.2 A liquefied-gas barge pushed by a tug

- **ABS Steel Barges 2024, 5-2-5/1**: liquefied-gas barges "are to comply with the … IGC Code as
  appropriate, or other national standard, as applicable to the non-propelled status of the vessel";
  "For manned barges, consideration is to be given for full compliance with the code"; the flag and
  port Administrations decide. **5-2-5/3**: manned and unmanned "as established by the flag
  Administration"; notation Liquefied Gas Tank Barge with a cargo list. **5-2-5/9.3**: tug-barge
  combinations carrying liquefied gas with Lc > 65 m need a loading manual held by the tug's master.
- **ABS Guide for Liquefied Gas Tank Barges with Remote Control and Monitoring of Essential Systems
  (2019)**: for barges "unattended and controlled primarily from the towing vessel's navigation
  bridge" (press release; guide text not read).
- **IGC 2014, 1.1.1**: the Code applies to "ships regardless of their size". SOLAS Chapter I
  regulation 3 excludes ships not propelled by mechanical means (secondary copies read, not the IMO
  text). So for an unpowered barge the IGC Code applies through class and flag, not by SOLAS itself
  (inference from the two texts).
- **Built practice.** Every LNG ATB found is classed to the IGC Code by choice: the 4,000 m³ unit
  "built to the IMO International Gas Carrier Code"; the 5,000 m³ design lists "ABS LNG Tank Barge,
  IMO, SOLAS IGC Code, OCIMF, SIGTTO"; the 6,000 m³ design: barge "A1 Liquefied Gas Tank Barge, ATB,
  Unrestricted Ocean Service, GCU, RELIQ, LNG Bunkering", tug "A1 Towing Vessel, AMS, ACCU, UWILD".

### 3.3 Hazardous-area separation between the tug and the barge (IGC 2014)

The tug's deckhouse sits at the barge's aft end, so the IGC accommodation rules read across the notch.
None of these texts mentions a tug; applying them to the tug is our reading (inference).

| Clause | Requirement | Effect on an ATB |
|---|---|---|
| 1.2.24.9 | Hazardous: open deck over the cargo area and 3 m fore and aft of it, up to 2.4 m above the weather deck | the barge's aft cargo-area boundary must sit at least 3 m forward of the notch apex, or the tug bow is in a zone |
| 1.2.24.14 | Hazardous: 6 m radius cylinder of unlimited height around a vent riser outlet | vent mast placement against the tug's wheelhouse and intakes |
| 3.2.1 | No accommodation, service space or control station within the cargo area | the tug, and any barge deckhouse, stay aft of the cargo area |
| 3.2.4.1 | Entrances, air inlets, openings not facing the cargo area; on the side at least 4 % L, not less than 3 m, need not exceed 5 m from the end facing the cargo area | the tug's forward house front faces the cargo area across the notch |
| 3.2.4.2, 3.2.5 | Windows facing the cargo area fixed; A-60, wheelhouse A-0; wheelhouse doors and windows allowed if the wheelhouse can be made gas-tight quickly | ATB tug wheelhouse front |
| 8.2.11.1 | Cargo PRV vent exits at least B or 25 m, whichever is less, from the nearest air intake or opening to accommodation, service spaces, control stations; less permitted below 90 m length | sets the vent mast's distance from the tug's openings; the 5,000 m³ design uses a "Slewing Vent Mast for adjustable Hazardous Zones" |
| 8.2.11.2 | Other containment vent outlets at least 10 m from those openings | as above |
| 16.2–16.4 | Use of cargo vapour as fuel: no visible flame, exhaust below 535 °C, ventilated gas-consumer spaces, double-wall supply | applies if boil-off is sent to the tug |

### 3.4 Boil-off to the tug

- A US patent (8,967,174, Moran and Elliott Bay Design Group) covers an articulated conduit carrying
  boil-off gas from an LNG barge to its tug across the relative motion; trade press notes there are
  "not (yet) specific regulations" for a gas connection between a tug and a barge in an ATB (MarineLog).
- ABS gave an AiP in 2012 to an LNG ATB concept with dual-fuel tug engines burning the gas (trade press).
- **No built LNG ATB found burns cargo boil-off in its tug.** The built and designed units handle
  boil-off on the barge (sub-cooler, gas combustion unit, pressure accumulation) and run diesel tugs.

## 4. Built and designed LNG ATBs (reference vessels)

| Unit | Barge | Tug | Power | Speed | Cargo | Source |
|---|---|---|---|---|---|---|
| 4,000 m³ LNG bunker ATB (2021) | 324 × 64 × 32.5 ft depth (98.7 × 19.5 × 9.9 m) | 128 × 42 × 21 ft depth | 2 × 1,900 kW (2 × 2,548 hp), Z-drives; diesel | 10 kn | 4,000 m³; DP1+; IGC; ABS | Baird Maritime vessel review, 11 May 2021 |
| 5,000 m³ LNG bunker ATB design | combined 372 ft, breadth 64 ft, design draft 13'4", max loaded 16' | 128 × 40 ft, draft 16 ft, displacement 984 t, GRT < 300 | 2 × 3,350 hp GE Tier IV, 2 Schottel azimuth; diesel | not stated | 2 × 2,550 m³ Type C, 4 bar; BOG 0.20 %/day; GCU 0.5 t/h, sub-cooler 0.5 t/h | Crowley spec sheet V3 (2024) |
| 6,000 m³ LNG bunker ATB design | 350 × 62 ft, max draft 25 ft | 128 × 42 ft, draft 18 ft | 2 × Wabtec 8L250MDC, 2 × Schottel SRP560FP; diesel | not stated | 1 × 6,100 m³ Type C bi-lobe; unrestricted ocean | Crowley spec sheet V6 (2025) |
| 5,400–5,500 m³ units (two in service, Florida) | — | — | — | — | — | trade press; particulars not read |
| 12,000 m³ LNG bunker ATB design | — | — | — | — | ABS design approval, July 2026 | trade press snippet; not read |

The pattern: tug 39 m, beam 12–13 m, draught 4.9–5.5 m, about 4–5 MW, on barges of 4,000–6,000 m³
and 19–20 m beam. No reference unit reaches 20,000 m³ or a 29 m beam. **The largest built LNG ATB is
about a fifth of a 20,000 m³ barge; any tug sizing for that barge extrapolates** (inference).

## 5. Tug sizing, power and operability

- **OVD's ATB power is low on the reference ATBs**: 0.29–0.43 of published installed power
  (`atb_powering.rs`, on the `barge_fit` method of the time). The published power also covers shallow
  water, seaway, bollard pull and the owner's margin, so part of the gap is margin, not error.
- **The barge is outside every method's range.** `barge_form_factor` covers L/B 7.6–16.3, Cp
  0.77–0.88, Fn ≤ 0.137 (ADR 0024). A wide gas barge at L/B ≈ 4.8, Cp ≈ 0.96 is outside on two
  counts; the ATB Equivalent Hull is longer (L/B ≈ 5.5) but still outside. No open data cover a short,
  wide, full barge (ADR 0024).
- **A cross-check by the Admiralty coefficient (inference, not a method).** Scaling the 4,000 m³ unit's
  3.8 MW at 10 kn by displacement^(2/3) to a combination of about twice its size gives roughly 6 MW
  installed at 10 kn. The barge-alone figures OVD gives for a 20,000 m³ hull are 1.7–2.7 MW at
  8.7–9.7 kn. The two differ by about two times; installed margins explain part of it. This is the
  strongest reason to calibrate before a number goes in a report.
- **Coupler size from tug displacement.** Intercon rates by tug displacement (to 4,200 LT for the 64").
  The 5,000 m³ design's tug displaces 984 t. A tug for a 20,000 m³ barge will sit well inside the 64"
  rating (inference).
- **Wave climate, Gulf of Guinea.** Mean Hs about 1.3–1.6 m and mean period 10.4–10.9 s over 1979–2005
  (Dahunsi et al., JMSE 2022, abstract); swell from the south-west, energy highest May–September
  (Dingong et al., African J. Marine Science 2025, abstract); Sekondi records: zero-crossing periods
  under 8 s "essentially absent" (older IGY study, abstract). The rated Hs of the larger Intercon
  couplers (7.0 m) is far above the mean. **Inference:** coupler Hs will rarely limit operability on
  this coast; long-period swell (pitch response of the pair, connector fatigue, and bar crossings at
  river mouths) is the likelier limit. No motions model exists in OVD (#731).

## 6. Economics against a self-propelled hull

| Measure | Tug-barge | Ship | Source |
|---|---|---|---|
| Crew | 7–11 (ATB 7–12) | 17–25 (25+) | Yanakiev 2025 Table 2 (Kaskin) and §4.3 |
| Capital cost | 691–711 USD/DWT, 25,000 DWT barge, 7,000 BHP tug | 1,300–1,500 USD/DWT, 25,000 DWT ship | Yanakiev Table 2 (Kaskin); year not stated |
| Fuel | up to 5 % higher (rigidly linked, +5 % resistance) | — | Yanakiev Table 2 |
| 2005 US comparison | ATB $77.5 M, 12,000 hp, 13 kn, crew 12 | tanker $90 M, 12,900 hp, 15 kn, crew 21 | Flanagan, SNAME 2005 |

- The crew saving rests on US rules (tug tonnage thresholds, uninspected tugs, Jones Act trade).
  Yanakiev §5 states the reduction "is made possible under specific regulatory frameworks". Outside
  the US, MSC/Circ.866 lets a flag treat the barge as a separate, unmanned ship, so the manning is the
  tug's minimum safe manning set by the flag (inference: no West African flag rule on ATB manning was
  found; **gap**).
- Other levers: tug and barge built in different, smaller yards; the barge on barge rules (lighter
  steel, fewer safety systems when unmanned); one tug can serve more than one barge, so a barge can
  wait at a terminal as storage while the tug works (drop-and-swap); repairs split between yards.
- Against: the coupler and notch structure cost; lower propulsive efficiency than a large slow
  propeller (Yanakiev Table 3).

## 7. Sources

- OVD: `docs/specs/0007-stern-propulsion-and-tugs.md`, `docs/research/stern-and-tugs.md` §7, §10, §11,
  `calibration/atb.yaml`, `crates/ovd-core/src/atb.rs`, `formation.rs`,
  `crates/ovd-format/tests/atb_powering.rs`, ADR 0011, 0017, 0024.
- [Intercon models](https://intercon.com/models/), [Intercon advantages](https://intercon.com/advantages/).
- JAK ATB Coupling System brochure 2023, Beacon Finland (local copy, hash in the source library).
- ABS Rules for Building and Classing Steel Barges 2024, 5-2-5 and 5-3-1 (local copy).
- IMO MSC/Circ.866, 1 June 1998 ([hosted copy](https://denizcilik.uab.gov.tr/uploads/pages/gemi-operasyonlari/yak24.pdf)).
- IGC Code 2014, MSC.370(93), 1.1.1, 1.2.24, 3.2, 8.2.11, 16.2–16.4 (local copy).
- USCG NVIC 2-81 Change 1 (local copy); Change 2 (2023) not read.
- Yanakiev, Garbatov, Georgiev (2025), *Advances of Articulated Tug–Barge Transport in Enhancing
  Shipping Efficiency*, JMSE 13(8) 1451, CC BY 4.0 ([pdf](https://mdpi-res.com/d_attachment/jmse/jmse-13-01451/article_deploy/jmse-13-01451.pdf)).
- Flanagan (2005), *Modern ATB: the Jones Act Solution*, SNAME Maritime Technology Conference (slides).
- [Baird Maritime vessel review, 4,000 m³ LNG ATB](https://www.bairdmaritime.com/commercial-mariner/ships-ports/vessel-review-q-ocean-service-q-lng-4000-first-us-flagged-offshore-lng-bunkering-atb-to-serve-southeastern-coastal-clients).
- [Crowley 5,000 m³ LNG bunker ATB spec sheet V3](https://www.crowley.com/wp-content/uploads/sites/7/2024/10/24-05505-5000m3-BunkerATB-Spec-Sheet-V3.pdf);
  [Crowley 6K LNG bunker ATB spec sheet V6](https://www.crowley.com/wp-content/uploads/sites/7/2026/07/25-06490-478-x-62-6K-LNG-Bunker-ATB-AT-12842-B-35062-Spec-Sheet-V6-11-03-25.pdf).
- [MarineLog: boil-off gas conduit patent](https://www.marinelog.com/inland-coastal/coastal/moran-towing-patents-technology-to-use-lng-boil-off-gas-in-atb-tug/);
  [ABS LG tank barge guide press release](https://ww2.eagle.org/en/news/press-room/abs-new-guide-liquefied-gas-tank-barges.html).
- Dahunsi et al. (2022), JMSE (abstract via [DOAJ](https://doaj.org/article/c4e8bbf9fc0b4f2eb3d6d9432458f99a)).

## 8. Gaps to close in the build, not by asking

DNV and BV ATB clauses; NVIC 2-81 Change 2; a notch depth and width in figures; Articouple and
Bludworth ratings; a published power and speed for an LNG ATB above 6,000 m³; West African flag
manning practice for ATB tugs; a point hindcast (ERA5) of Hs and Tp on the route.
