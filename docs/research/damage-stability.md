# Research note: damage stability for OVD (issue #729)

**Research line.** *Searched:* the rule texts (IGC 2014 as amended, ADN 2025 Part 9, ES-TRIN 2025/1,
SOLAS II-1 as amended by MSC.421(98), ICLL 1988 Protocol reg. 27), OVD's own code, specs and ADRs, open
hydrostatics and damage-stability libraries, and published worked cases. *Exists:* OVD **already has a
working damage stability engine** (lost buoyancy on the Sections Table, `ovd-core/src/damage_stability.rs`,
#341) and case generators for ADN 9.3.2/9.3.3.15 and MARPOL I reg. 28 on liquid bulk hulls. No open
library does deterministic damage stability with rule criteria; one AGPL library does hydrostatics and GZ
from meshes. *Reusing:* OVD's solver, `DamageCriteria`, the `damage-case` rule object, the Spaces model
(ADR 0021) and its tests' oracle pattern. *Building because:* gas hulls have no damage cases, no
transverse subdivision in the cargo zone, and no way to float an intact Type C tank inside a flooded hold.
None of that exists anywhere we could reuse.

Sources are the maintainer's local copies of the free texts (IMO resolutions from wwwcdn.imo.org, ADN 2025 ECE/TRANS/351, ES-TRIN 2025/1 from cesni.eu), each with a recorded SHA-256, unless a URL is given.
"Inference" marks my reading, not the text.

## 1. Headline

1. **#729 is mostly a gas-hull job, not a new engine.** Spec 0006 and #341 built the solver and the
   liquid-bulk cases (commits 8248a22f, 54e0b1f4). The `igc/2.7/damage-survival` row is `not evaluated`
   with the gap "IGC 2.3 damage not modelled yet" (#663, f2d80b7e).
2. **The gas hull's compartments cannot carry damage cases today.** One hold `H1C` spans the whole cargo
   zone, with one double-bottom pair and one wing pair over the same length
   (`vessel_model.rs::spaces`, `compartments.rs::ballast_tanks`). With "any bulkhead within the damage is
   damaged", every damage floods the whole hold. The strip above the wing tanks, outboard of the hold, is
   in no Space at all, so a side damage would leave it buoyant (inference from the code).
3. **The solver floods boxes only.** `FloodedBox` is an axis-aligned box. A gas hold holds Type C tanks
   (cylinders, ADR 0021). An intact tank in a flooded hold keeps its buoyancy. A breached tank loses its
   contents (IGC 2.5.3). Neither can be expressed today.
4. **Probabilistic (SOLAS II-1 Part B-1) does not apply** to OVD's envelope. See §2.4.
5. **No published damage case with GZ figures for a barge was found** (the same gap #341 recorded).
   Closed forms and an independent integral stay the oracle; an external mesh code can add a cross-check.

## 2. The rules

### 2.1 IGC Code (MSC.370(93), amended to MSC.566(109))

Numbering note: the brief's "IGC 2.4–2.9" is the **1993 Code's** numbering. The 2014 Code (in force
2016) moved them: 2.3 damage assumptions, 2.4 tank location, 2.5 flood assumptions, 2.6 standard of
damage, 2.7 survival (source library notes). OVD's `igc-2g` set already uses 2014 numbers.

| Para | What it says (paraphrase) | OVD today |
|---|---|---|
| 2.3.1.1 | Side damage: long. min(⅓L^⅔, 14.5 m); transverse min(B/5, 11.5 m) inboard at the summer waterline; vertical unlimited | not modelled |
| 2.3.1.2 | Bottom damage: long. min(⅓L^⅔, 14.5 m) both zones; transverse min(B/6, 10 m) in the forward 0.3 L, min(B/6, 5 m) elsewhere; vertical min(B/15, 2 m) | not modelled. Note: **2 m**, not MARPOL's 6 m |
| 2.3.2.1 | A lesser damage that is worse must be assumed | not modelled (MARPOL path has the same gap) |
| 2.3.2.2 | Local damage anywhere in the cargo area to depth "d" (2.4.1) | not modelled |
| 2.4.1 | Tank location: 2G/2PG at least the bottom extent from the bottom and nowhere less than d (0.8 m for Vc ≤ 1,000 m³) from the shell | evaluated (`igc/2.4.1.2/*`) |
| 2.5.2 | Permeabilities: stores 0.60, accommodation 0.95, machinery 0.85, voids 0.95, hold spaces 0.95, consumable and other liquids 0 to 0.95 (partial fill consistent with the amount) | — |
| 2.5.3 | A damaged tank of liquid loses all its contents; salt water replaces them to the final waterplane | — |
| 2.5.4 | Transverse bulkheads closer than the side-damage length are non-existent; bulkheads bounding side or DB compartments are damaged when inside the extent; a step over 3 m is damaged | — |
| 2.5.5–2.5.7 | Minimise unsymmetrical flooding; no credit for valved equalisation; no progressive flooding through pipes | not modelled |
| 2.5.8 | No buoyancy for superstructure over a side damage | superstructures are not in the hydrostatics (spec 0008) |
| 2.6.1 | Standard of damage by ship type. **2G ≤ 150 m:** anywhere in the length, except either bulkhead bounding an aft machinery space. **2PG:** anywhere, except bulkheads further apart than the side-damage length. **3G < 80 m:** as 2PG and excluding an aft machinery space | — |
| 2.7.1 | Any stage of flooding: no downflooding opening immersed (2.7.1.1 as replaced by MSC.492(104)); heel ≤ 30°; intermediate residual stability as 2.7.2.1 | — |
| 2.7.2.1 | Final: range ≥ 20° beyond equilibrium, max GZ ≥ 0.1 m within it, area ≥ 0.0175 m.rad; the range may start anywhere from equilibrium to 25° (30° if no deck immersion). IACS UI GC17 narrows "openings closed weathertight" | — |
| 2.7.2.2 | Emergency power works | approval item |

Inference: on a narrow gas barge, B/5 side damage reaches through a 1 m wing into the hold and the tank
(2.4 m on a 12 m beam). So under IGC the tank is breached in most side cases, and 2.5.3 applies.

### 2.2 ADN 2025, type G (9.3.1.13 to 9.3.1.15)

- 9.3.1.13.1: damage stability proof required for every type G vessel. 9.3.1.13.4: most unfavourable
  loading condition, critical intermediate stages and the final stage.
- 9.3.1.15.1: **same extents as type C**: side 0.10 L (≥ 5.00 m) × 0.79 m (or the 9.3.4 distance less
  0.01 m) × unlimited; bottom 0.10 L × 3.00 m × 0.59 m; bulkheads in the extent damaged; bottom damage
  floods the compartments athwartships; permeability 95 % (a calculated average may be used); minima:
  engine room 85 %, accommodation 95 %, DB/fuel/ballast 0 or 95 % by function; engine room one-compartment.
- 9.3.1.15.2: intermediate GZ ≥ 0.03 m over 5°; final heel ≤ 12°; GZ ≥ 0.05 m and area ≥ 0.0065 m.rad up
  to the first non-weathertight opening, in any event to 27°; openings ≥ 0.10 m above the damage waterline.
- 9.3.1.11.2(a): a type G hull is a double hull (≥ 0.80 m side, ≥ 0.60 m DB; refrigerated tanks must
  be in double-hull holds), **or** a single hull with tanks ≥ 0.80 m from the side plating and ≥ 0.60 m
  from the bottom.

Inference: the 0.79 m side damage is 0.01 m short of the 0.80 m clearance, so under ADN the tank is
never breached; the wing (double hull) or the hold (single hull) floods. On a single-hull type G vessel
the hold floods around an intact tank, so the tank's buoyancy decides the result.

**Owner decision on record:** the gas `ADN_G` set was removed on 2026-10-15 (#175): "a ruleset with ADN's
name but not all of its rules would look like ADN compliance without being it." The river-only gas stack
today is `igc-2g`, `is2008-a`, `lr-inland-gas`, `es-trin-2025`, `ovd`. LR IWW Pt 4 Ch 4 1.7.1 points
type G stability to ADN (crude-oil research note 02 §5.2).

### 2.3 ES-TRIN 2025/1

- Art. 28.03 (craft **over 110 m**, not passenger vessels): damage stability by calculation; final stage
  by **lost buoyancy**, intermediate stages (25, 50, 75 %) by **added mass**, free to trim and sink.
  Side 0.10 L × 0.59 m × unlimited; bottom 0.10 L × 3.00 m × 0.39 m; bulkheads in the extent damaged;
  engine room one-compartment. Permeability 95 %, minima engine and service rooms 85 %, **cargo holds 70 %**,
  DB/fuel/ballast 0 or 95 %. Intermediate: heel ≤ 15°, GZ ≥ 0.02 m to 27°. Final: openings ≥ 0.10 m above
  the waterline, heel ≤ 12°, GZR ≥ 0.05 m and area ≥ 0.0065 m.rad to 27° (container figures differ).
  28.03(9): an ADN Part 9 damage calculation with a positive result is deemed proof.
- Below 110 m ES-TRIN asks no damage stability of a cargo vessel or barge (chapter 19 covers passenger
  vessels; out of OVD's scope). `es-trin-2025` 1.0.0 holds 28.04(1)(a) but not 28.03.

### 2.4 SOLAS II-1 probabilistic, and why it does not apply

- MSC.421(98), reg. 4.2.1.2: regulations 6 to 7-3 (probabilistic) apply to cargo ships of L ≥ 80 m, but
  may exclude ships shown to comply with the damage stability of MARPOL Annex I, the IBC Code, **the IGC
  Code**, ICLL reg. 27, or other IMO instruments.
- SOLAS ch. I reg. 3 excludes cargo ships under 500 GT and ships not propelled by mechanical means
  (from secondary reproductions of the text; the consolidated SOLAS is paywalled; not read in primary).
- So: an OVD gas vessel is either outside SOLAS (a barge, or under 500 GT) or excluded by IGC compliance.
  A dry-bulk or liquid barge is outside SOLAS. ADR 0011 already keeps "below the size where SOLAS applies
  in full". **Probabilistic damage stability has no customer in the envelope.** Inference: it would only
  matter if the envelope grew to self-propelled dry cargo ships ≥ 80 m on international voyages.

### 2.5 ICLL reg. 27 (MSC.143(77))

27(3): damage only for type A ships **over 150 m** with reduced freeboard; 27(14): barges may get type A
freeboard if they meet 27(2) and 27(3). Inference: inside OVD's envelope (mostly < 150 m) ICLL adds no
damage case. The 27(13) criteria (heel ≤ 15°/17°, GM > 0, residual 20°/0.1 m/0.0175) are listed for
completeness.

### 2.6 MARPOL I reg. 28 and NORMAM-202

Built for liquid bulk (#341). NORMAM 5.23.8/9 copies the MARPOL extents and criteria; its rows say
"ADN extents only" and stay `not evaluated`. Inference: the IGC case generator in this spec is the
MARPOL generator with other numbers (B/15 capped at 2 m, not 6 m), so NORMAM 5.23.8 can be evaluated
from the same code at no extra cost.

## 3. Method

| Choice | What the texts say | OVD today |
|---|---|---|
| Lost buoyancy (constant displacement, KG fixed) vs added weight (flood water as mass) | Same final position when solved exactly. ES-TRIN 28.03(3) names lost buoyancy for the final stage and added mass for intermediate stages. IGC and ADN do not name a method | Lost buoyancy, final stage, Newton solve of heel, trim and sinkage (`damage_stability.rs`) |
| Intermediate stages | ADN 9.3.1.15.2, ES-TRIN 28.03(5) and IGC 2.7.1.3 ask for them | not modelled |
| Deterministic vs probabilistic | Every rule in the envelope is deterministic (§2.4) | deterministic |
| Free surface in flooded spaces | Open to the sea: none at the final stage. Intermediate stages need it (ES-TRIN 28.03(4)(e): gross surface) | none |
| Permeability | A table per rule (IGC 2.5.2, ADN 9.3.1.15.1(c), ES-TRIN 28.03(4)(d)); liquids by function or filling | ADN 0/95 and MARPOL 95 for liquid spaces |
| Intact tank in a flooded hold | Not named in the texts; the tank is an intact volume, so it keeps its buoyancy (standard practice; inference) | cannot be expressed |

## 4. Open tools and libraries

| Tool | What it does | Licence | Verdict |
|---|---|---|---|
| OVD `damage_stability.rs` | Lost buoyancy, free trim, residual GZ, ADN and MARPOL criteria | Apache-2.0 (ours) | **Reuse.** Extend, do not replace |
| [navaltoolbox](https://github.com/NavalToolbox/navaltoolbox-lib) 0.9.3 (Rust/Python, Jul 2026) | Hydrostatics and GZ from STL/VTK, tanks, downflooding openings, scriptable criteria. **No damage or flooding module.** DTMB 5415 validation test | **AGPL-3.0** | **Test oracle only, outside the repo.** A damaged hull at 100 % permeability is the hull mesh less the flooded box, so its GZ is an independent check. Never link it into OVD (AGPL vs Apache-2.0) |
| [FreeCAD Ship](https://github.com/FreeCAD/freecad.ship) | Hydrostatics, GZ, tanks | LGPL-2.1 | Damaged stability was still on its roadmap when last checked (2026-08); not re-checked. Reject |
| [shipstab](https://pypi.org/project/shipstab/) 0.1.0 | Claims added-weight and lost-buoyancy damage | MIT | Alpha, one maintainer, no validation, the PyPI author field names an AI model. Reject as an oracle |
| OpenFOAM flooding studies | Time-domain CFD flooding | GPL | Out of scope (dynamic, not a rule check) |
| Commercial (GHS, Maxsurf Stability, NAPA, PIAS, DELFTship Pro) | Full deterministic and probabilistic damage | proprietary | Not reusable. A class reviewer will run one of these; OVD's output must be easy to compare |

## 5. Worked examples to verify against

- **Closed forms (have):** a wall-sided box with a centred flooded DB: sinkage = lost volume over the
  waterplane; GZ in closed form (`tests/damage_stability.rs`). Asymmetric case checked by an independent
  grid integral.
- **New closed forms (can derive):** a horizontal cylinder inside a flooded box hold at even keel (the
  circle-segment area is exact), and a symmetric two-tank hold. These check the intact-tank geometry.
- **Textbooks:** Biran and López-Pulido, *Ship Hydrostatics and Stability*, §11.3 compares lost buoyancy
  and added weight on a box barge (table of contents seen, numbers not read). Copyright text: cite the
  figures only after reading them; do not copy.
- **Published barge case with damage GZ figures:** none found (again). The #341 acceptance line stays open.
- **External cross-check:** navaltoolbox on a mesh with the flooded compartment cut out, at 100 %
  permeability, run outside the repo, numbers stored as a `sourced` fixture with the tool version.

## 6. What OVD's compartment model supports, and what it lacks

| Have | Lack, for a gas hull |
|---|---|
| Typed Spaces with kind, side, prism section or Type C tank shape (ADR 0021) | Transverse bulkheads in the cargo zone: one hold, one DB pair and one wing pair run its whole length |
| Clipping of a prism by the hull (`compartments::clip`) | The void outboard of the hold above the wing tanks is not a Space |
| Loading conditions with tank fillings (`loading_conditions.rs`) | A flooded shape other than a box; a tank as an intact body inside a flooded Space; a breached tank whose contents leave |
| `damage-case` and `marpol-damage-case` rule objects and the rule trace | Openings as points (only an intact `downflooding_angle_deg` input); a deck edge test for the 25°/30° choice |
| Aft machinery, cofferdam, service and peak Spaces | A way to mark the machinery space's bulkheads as not damaged (IGC 2.6.1.3, ADN one-compartment) |

## 7. Limits (ADR 0011)

Every output says: concept or preliminary check, final stage of flooding (until intermediate stages
land), no openings unless typed, no progressive flooding, not an approval. The rule rows that cannot be
evaluated stay `not evaluated` with a reason.
