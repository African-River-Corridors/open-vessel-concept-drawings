# Spec 0014: An articulated tug-barge for a liquefied-gas barge

Status: **agreed by the maintainer, 2026-10-31** (decisions D1–D23 taken with the maintainer,
2026-10-30/31). Builds on spec 0007 (Formations, the `atb` kind, tug sizing, Notch, Equivalent
Hull; ADR 0017), the sections generator of spec 0008, and the resistance methods of ADR 0023 and 0024.
Shares the stern-arrangement model with spec 0011 (#734). Scope per ADR 0011: concept and preliminary
design, never approval. Research: `docs/research/atb-gas-tug-barge.md`.

## Problem Statement

OVD can build an ATB: it sizes a Tug from the barge by fixed ratios, cuts a Notch, and powers the pair
as one Equivalent Hull. For a liquefied-gas barge on a coastal route that is not enough to choose
between an ATB and a self-propelled hull:

- **No coupler.** Nothing picks a coupler, checks it against the Tug's displacement, or says in what
  sea state the pair may stay connected. IMO MSC/Circ.866 2.2.4 asks that the master know the maximum
  sea state to disconnect and reconnect.
- **The power is not trusted.** OVD's ATB power is 0.29 to 0.43 of the published installed power of
  the three reference ATBs (`atb_powering.rs`). No LNG ATB is in the calibration. A wide, full gas
  barge is outside every resistance method's range (ADR 0024).
- **The Tug can be wrong for the barge.** The default Tug draught (B_tug / 2.33) ignores the barge's
  draught and the Waterway Limit. On the 140 × 29 m gas example it is 6.6 m against a laden barge
  draught of about 4.2 m. Built LNG ATB tugs draw 4.9 to 5.5 m.
- **No gas interface.** The IGC Code keeps accommodation, intakes and openings out of the cargo area,
  3 m clear of it, and B or 25 m from cargo vent exits. The Tug's house faces the barge's cargo area
  across the Notch. OVD checks none of this.
- **No gas path to the Tug.** Boil-off can be burnt on the barge, returned to a terminal, or sent to
  the Tug. A gas-fuelled Tug can be topped up from the cargo. OVD's gas balance knows only a
  self-propelled hull.
- **No Tug of its own.** The ATB Tug has no hull form, no GA and no stability check (0007 "Out of
  Scope").
- **No comparison.** A designer cannot see an ATB beside a self-propelled hull on the same cargo
  basis: power, speed, crew, fuel, capital cost.

## Solution

The `atb` Formation gains a **Coupler**, a **Gas Interface**, a **Tug gas supply**, a **Tug of its
own** (hull, GA, stability), an **Operability** check and a **comparison with the self-propelled
hull**. Its powering is calibrated on built ATBs, LNG ATBs included, and states its error. The
Formation can be held to an overall length, so the barge gives up length to the Tug. The barge-side
Notch belongs to this spec; the Tug's stern arrangement comes from spec 0011.

## User Stories

### Designer

1. As a designer, I want to turn a gas barge into an ATB and get a consistent Tug, Notch and Coupler,
   so that the pair is buildable.
2. As a designer, I want to hold the pair to the length my route allows, so that the barge gives up
   the length the Tug needs and I see the cargo lost.
3. As a designer, I want to start from a published tug or a tug I type, so that the barge fits a tug
   I can buy.
4. As a designer, I want the Coupler picked from a catalogue by Tug displacement, with its rated
   sea state, so that I know when the pair can push.
5. As a designer, I want the share of time the pair can stay connected on my route, from a wave
   climate I type, so that I can judge the schedule.
6. As a designer, I want the IGC separation between the Tug and the barge's cargo area and vents
   checked, so that the arrangement is right from the start.
7. As a designer, I want to choose where boil-off goes and how the gas Tug gets its gas, so that the
   gas balance and the cargo delivered follow.
8. As a designer, I want the Tug's hull, GA and stability, pushing and in tow, so that the Tug is a
   real vessel and not three numbers.
9. As a designer, I want the ATB and the self-propelled hull side by side, so that I can choose.

### Reviewer or expert

10. As a reviewer, I want the ATB power compared with built ATBs, with each residual stated, so that
    I know how far to trust it.
11. As a reviewer, I want each ATB and gas Rule cited to its paragraph and marked as a check or an
    approval item, so that I know what OVD checked and what class will ask for.

### Maintainer

12. As a maintainer, I want old files and today's ATB fixtures to keep their results hash, so that
    this work moves no saved design.

## Implementation Decisions

The maintainer's decisions are numbered D1–D23 (grill, 2026-10-30/31).

### Scope and home

- **D1** A new spec on top of 0007. It amends 0007's ATB defaults only where stated below.
- **D2** The ATB is a second option beside the self-propelled hull, on the same cargo basis, compared
  in the concept report.
- **D3** The first version covers everything in this spec, the Tug's GA and stability included.
- **D6** Split with spec 0011: this spec owns the barge Notch (depth, width, wing walls, coupler
  sockets, structure notes, the Tug-in-Notch fit) and the gas separation between Tug and barge, and
  amends the `atb.rs` ratios. Spec 0011 owns the arrangement model of every self-propelled stern
  block (engine room, steering gear, wheelhouse, accommodation, machinery mass, steel and cost); this
  spec calls it for the Tug. The ATB Tug's wheelhouse height over a laden barge uses 0011's sightline
  model; this spec supplies the barge obstructions and the coupled geometry.

### Terms (new in `GLOSSARY.md`)

**Coupler** (the mechanical connection between an ATB Tug and the barge Notch, with its degrees of
freedom and rating), **Rated Significant Wave Height**, **Formation Length Basis**, **Time Connected**,
**Gas Interface** (the IGC separation between the Tug and the barge's cargo area and vents).

### Inputs (format version 3; every input has a legacy value that keeps today's result)

| Input | Unit | Legacy | Default | Basis |
|---|---|---|---|---|
| `formation_length_basis` | `barge` or `formation` | `barge` | `formation` for a new `atb` Option | D17. `formation`: the typed length is the pair's; barge length = typed length − tug length + notch depth |
| `tug_draught_m` cap | m | none | min(ratio value, barge laden draught, Waterway Limit draught) | D7. A typed or preset Tug deeper than either gets a warning |
| `coupler` | catalogue id, `generic`, or none | none | the smallest catalogue Coupler whose rated tug displacement ≥ the Tug's displacement | D5 |
| `coupler_rated_tug_displacement_t`, `coupler_rated_hs_m`, `coupler_mass_t`, `coupler_envelope_m` | t, m, t, m | — | from the catalogue | Intercon published table (maker); generic values typed |
| `boil_off_destination` | `barge` (gas combustion unit or reliquefaction), `terminal`, `tug` | `barge` | `barge` | D9. `tug` carries a warning: no rule covers a tug-barge gas link |
| `tug_power_source` | as 0007 | `diesel_mechanical` | `gas_fuelled` when the barge's Cargo is gas | D16 (as 0007 decision 7) |
| `tug_gas_supply` | `own_tank`, `top_up_from_cargo`, `boil_off_link` | — | `top_up_from_cargo` | D21. Every value but `own_tank` carries the gas-link warning |
| `tug_gas_endurance_h` | h | — | 48 h at service power (estimate) | D23. Sizes the Tug's own fuel tank for time disconnected (in tow or alone). No source gives a figure; the default is marked as an estimate |
| `route_wave_climate` | table of Hs (and Tp) with time shares, or Hs percentiles | none | none (no check without it) | D11 |
| `tug_manning`, `barge_manning` | persons | none | none (typed) | D12. Barge unmanned by D8 |

### Calibration of ATB power (D10, D19)

- Keep the Equivalent Hull of 0007. Refit the ATB Formation allowance on the reference ATBs with the
  current default resistance method. Add the LNG ATBs with published power (the 4,000 m³ unit: 3,800
  kW, 10 kn) to `calibration/atb.yaml`; designs that publish power but no speed are listed and held
  out of the fit.
- Report each residual, the range, and the method's range check; status `estimate`.
- The ATB option enters the concept report once calibrated, with its error stated, whatever it is.

### Tug (D4, D7, D13–D16)

- **Source (D4):** sized from the power the pair needs (0007's loop), or a Reference Vessel preset,
  or typed. A preset or typed Tug fixes the Coupler and the Notch.
- **Draught (D7):** capped as in the inputs table.
- **Hull (D14):** a parametric tug hull from the sections generator (spec 0008), with tug parameters
  (bow fitted to the Notch, transom stern, skeg). A preset brings main dimensions only.
- **Arrangement:** spec 0011's stern-block model, plus the Coupler in the bow.
- **Power Source (D16):** gas-fuelled by default when the barge carries gas; propulsors as 0007.
- **Stability (D13):** IS Code 2008 Part A, and Part B 2.8 (towing and escort, MSC.415(97)),
  pushing and in tow, as Rules; ABS Marine Vessel Rules 5C-17-2 as a note until read.
- **GA (D15):** profile, main deck and wheelhouse plan, midship section, with the Coupler and the
  barge Notch drawn. The existing arrangement plan (#360) shows the pair.

### Barge (D17, D18, D22)

- **Length (D17):** with `formation_length_basis = formation`, the pair keeps the typed length and
  the barge is shorter by the Tug length less the Notch depth. The loop of 0007 then runs with the
  barge length changing; it stops on the same 1 % rule.
- **Stern (D18):** the barge drops the self-propelled stern block and keeps a small machinery space
  for generators and cargo plant (sized by spec 0011's model for an unmanned barge).
- **Tanks (D22):** keep the tank diameter and count; shorten the tanks to fit the shorter barge;
  report the cargo lost against the self-propelled option.
- **Notch (D6):** 0007's ratios stay as defaults; the Notch width also clears the Coupler envelope.
  Wing walls and coupler sockets get mass and an approval note (ABS 5-3-1/11.3).

### Gas Interface and gas balance (D8, D9, D21)

- **Checks across the Notch** (IGC 2014; our reading applies them to the Tug):
  1.2.24.9 (hazardous zone over the cargo area and 3 m fore and aft, to 2.4 m above the deck) against
  the Tug's bow and house front; 1.2.24.14 (6 m around a vent riser outlet); 3.2.1 (no accommodation,
  service space or control station in the cargo area); 3.2.4.1 (openings not facing the cargo area,
  4 % L, 3 to 5 m); 8.2.11.1 (cargo PRV vent exits at least B or 25 m, whichever is less, from any
  opening; less allowed below 90 m); 8.2.11.2 (other containment vents 10 m).
- **Gas balance of the pair:** cargo delivered = cargo loaded − boil-off handled on the barge or
  returned − gas taken by the Tug. The Tug's gas use comes from its installed power and the gas-genset
  data of 0007.
- A gas link to the Tug (`top_up_from_cargo`, `boil_off_link`) is shown as a warning that no rule
  covers it (research §3.4) and an approval item.

### Rules (D8)

| Set | Rules |
|---|---|
| `abs-barges` (2024) | `5-3-1/1.3` two vessels (note, existing); `5-3-1/1.1.3` combined length Lc (Measure); `5-3-1/5.3` barge strength on the barge alone (note); `5-3-1/9.3` still-water bending with and without the Tug (approval item); `5-3-1/11.1` connector lifetime loads by model test or multi-body analysis (approval item); `5-3-1/11.3` notch wing walls (note, existing); `5-2-5/1` IGC as appropriate to a non-propelled barge (note); `5-2-5/9.3` loading manual for a gas tug-barge with Lc > 65 m (approval item) |
| `imo-circ-866` (MSC/Circ.866, 1998) | type B status: two ships, barge unmanned (note); `2.2.2` the Tug able to tow the barge on a hawser (check: tow-mode stability and towing gear present); `2.2.4` maximum sea state to disconnect and reconnect known (the Coupler's rated Hs, shown) |
| `igc-2014` | the Gas Interface checks above, `block` for 3.2.1, `warn` for the rest |
| `is-code-2008` | the Tug's stability, Part A and Part B 2.8 |

DNV and BV follow when their clauses are read (research §3.1 gap). Every Rule cites edition and
paragraph and is a draft until a second reader verifies it.

### Operability (D11)

The Coupler's Rated Significant Wave Height against the typed route wave climate gives the Time
Connected (share of time Hs ≤ rated). No motions or connector loads until #731.

### Comparison (D2, D12)

Beside the self-propelled option on the same cargo basis: power and speed; capital cost (barge, Tug,
Coupler, Notch structure); crew from the typed manning; fuel per round trip; cargo delivered; the
pair's length. One table in the results and in the concept report.

## Testing Decisions

- **Old files:** every fixture opens with its old results hash; today's `atb_powering.rs` cases keep
  their values with the legacy inputs.
- **Calibration:** the refit table (in-sample and held-out, residuals) in the PR body; a test pins
  the allowance to `calibration/atb.yaml`.
- **Through interfaces:** the length basis (barge length = typed − tug + notch, to the bit); the
  draught cap; Coupler pick by displacement; Time Connected against a hand-summed scatter table; each
  Gas Interface check on a fixture that passes and one that fails; the gas balance closes to zero.
- **Tug stability:** IS Code criteria on a box tug against a hand calculation; tow-mode heeling lever
  against a hand calculation.
- **Drawings:** the conformance tests of spec 0003 for the Tug GA and the Coupler symbol.
- `ovd-core` stays `no_std`; the WebAssembly build runs in the gate.

## Out of Scope

- Connector lifetime loads, fatigue and FEA of the wing walls (approval items, notes only).
- Multi-body motions and seakeeping (#731).
- Dynamic positioning, manoeuvring, bunkering operations.
- Rigid ITBs (one ship, MSC/Circ.866 type A).
- Class approval (ADR 0011).

## Decisions taken and open questions

Taken with the maintainer, 2026-10-30/31: D1–D23 above. D23: the Tug's own gas tank lasts 48 hours at
service power while disconnected by default, marked as an estimate (no source gives a figure). No
decision is open.

## Build tickets (proposed; filed after this spec is agreed)

**Wave 1: calibration, Tug fit, Coupler**

1. ATB calibration: LNG ATBs in `calibration/atb.yaml`, refit the ATB allowance, residuals (relates #647, #359)
2. Formation length basis, shorter barge with shortened tanks, Tug draught cap (blocked by 1)
3. Coupler and its catalogue; Coupler mass in the Tug lightship (blocked by 2)

**Wave 2: gas and rules**

4. Gas Interface checks across the Notch (IGC 1.2.24, 3.2, 8.2.11) (blocked by 2)
5. ABS 5-3-1 and 5-2-5 ATB Rules and MSC/Circ.866 notes (blocked by 3, 4)
6. Boil-off destination, Tug gas supply and the pair's gas balance (blocked by 2)

**Wave 3: the Tug**

7. Parametric tug hull in the sections generator (blocked by 2)
8. Tug stability, IS Code Part A and Part B 2.8, pushing and in tow (blocked by 7 and the spec 0011 stern-model ticket)
9. Tug GA with the Coupler and Notch; wheelhouse over the laden barge via 0011's sightline (blocked by 3, 7 and the spec 0011 stern-model ticket)

**Wave 4: operability, comparison, report**

10. Route wave climate and Time Connected (blocked by 3)
11. ATB against the self-propelled hull: the comparison table (blocked by 1, 2, 3, 6)
12. Concept report: the ATB option beside the self-propelled option (blocked by 11, 10 and #727)
