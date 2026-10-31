# Spec 0012: Damage stability for gas hulls, and one damage model for every hull

Status: **agreed with the maintainer, 2026-10-31** (D1–D21; D5 and D12 moved into phase 1, D11 with a roadmap note; the rest as recommended). Issue: #729. Related: #341 and spec 0006 (liquid bulk damage, built).
Research: `docs/research/damage-stability.md`.

Concept and preliminary design only, never approval (ADR 0011).

## 1. Problem

OVD checks damage stability only on a liquid bulk hull (spec 0006, #341: ADN 9.3.2/9.3.3.15 and MARPOL I
reg. 28). A gas hull has no damage cases, so `igc/2.7/damage-survival` is `not evaluated` on every gas
Option (#663). For a gas carrier or gas barge, damage survival is a main design driver: it sets wing
width, hold subdivision and draught. A concept report cannot show it.

The engine is there. What is missing is on the gas side:

1. **No subdivision in the cargo zone.** One hold `H1C`, one double-bottom pair and one wing pair run the
   whole cargo zone. Every damage floods the whole hold.
2. **An unmodelled strip.** The space outboard of the hold above the wing tanks is in no Space.
3. **Boxes only.** The solver floods axis-aligned boxes. It cannot float an intact Type C tank inside a
   flooded hold, nor empty a breached one.
4. **No IGC damage cases.** No extents, no standard of damage by ship type, no permeability table.

## 2. Decisions (design tree)

Round 1 (2026-10-31): every decision as recommended **except D5 and D12**, which the maintainer moved into phase 1, and D11, with a note. The table records the decision.

| # | Decision | Options | Decision and why |
|---|---|---|---|
| D1 | Where this lives | (a) new spec 0012 that generalises the engine; (b) a section of spec 0006 | **(a).** 0006 is a cargo spec and its damage part is built. Gas is a different cargo; the shared parts (flooded shapes, permeability tables, case table, drawings) serve every hull. 0006 keeps its rows; 0012 must not move a liquid result |
| D2 | Rule sets, first | (a) IGC 2.3–2.7 for gas on every Service Area; (b) also ADN 9.3.1.15 (type G) for river gas; (c) also ES-TRIN 28.03 (> 110 m); (d) NORMAM 5.23.8/9 from the IGC/MARPOL generator | **(a) first, then (c) and (d) as data rows.** IGC is already the gas basis (#175). (b) re-opens #175: see D3 |
| D3 | ADN type G damage for river-only gas | (a) not now, keep #175; (b) a new `adn-gas` set that lists all of ADN 9.3.1 with unmodelled rows `not evaluated`; (c) put 9.3.1.15 into `lr-inland-gas` (LR points type G stability to ADN) | **(b), in phase 2.** The full listing answers #175's worry: the trace shows what is not checked. Under ADN the tank is never breached (0.79 m vs 0.80 m), so ADN tests the wing and hold, which IGC does not |
| D4 | Probabilistic (SOLAS II-1 B-1) | (a) never inside ADR 0011's envelope; (b) a later spec | **(a).** B-1 excludes IGC-compliant ships, and barges and ships under 500 GT are outside SOLAS. A `not applicable` note in the trace says so |
| D5 | Method and stages | (a) lost buoyancy, final stage only; (b) plus intermediate stages by added mass | **Decided: final and intermediate stages together in phase 1.** Final stage by lost buoyancy; intermediate stages by added mass (ES-TRIN 28.03(3) names both). Details in D15–D18 |
| D6 | Gas hull subdivision | (a) one hold per tank in line, a watertight bulkhead midway between tank heads, DB and wings cut at the same bulkheads, the strip above the wings a void Space; (b) keep one hold | **(a).** With one hold the answer is always "sinks". The bulkhead positions come from `tanks::tank_layout`, the one position function (ADR 0021). It moves the Spaces table, not the capacity |
| D7 | Type C tanks in a flooded hold | (a) exact: the hold floods less the tank's immersed volume; the tank floods only when the extent reaches it; (b) an average hold permeability (allowed by IGC 2.5.2 note 1 and ADN); (c) ignore the tanks | **(a).** (b) puts the buoyancy in the wrong place as the hold heels; (c) sinks every hull |
| D8 | A breached tank's contents | (a) as the text: contents lost, sea water in (IGC 2.5.3), the cargo mass leaves the condition; (b) keep the cargo mass, as 0006's MARPOL path does | **(a) for IGC.** The text is plain. Flag 0006's choice as a follow-up question, not a change here |
| D9 | Permeability | A table per Rule Set keyed on `SpaceKind`; liquids by the condition's filling | **Data, not code:** IGC 2.5.2, ADN 9.3.1.15.1(c), ES-TRIN 28.03(4)(d). The ADN liquid set keeps 0/95 |
| D10 | Loading conditions damaged | (a) laden departure only (as 0006); (b) every standard condition | **(b) for gas.** IGC 2.5.1 says all conditions; gas has three. The case table grows by ×3 |
| D11 | Criteria not modelled | Openings (IGC 2.7.1.1), deck immersion (25°/30°), the 20° range start, emergency power | **Decided:** range from equilibrium (conservative); a deck-edge test; openings and emergency power `not evaluated` until an openings input exists. **Maintainer's note: add the openings input and the 2.7.1.1 test as a ROADMAP item** (done in the spec PR) |
| D12 | Outputs | Case table, floating position, GZ after damage, rule trace rows, a damage drawing | **Decided: all of them in phase 1, the damage drawing included.** Which cases are drawn: D19 |
| D13 | Verification | Closed forms; independent grid integral; external mesh oracle; published case | **Closed forms + grid (as #341) + new cylinder-in-hold closed form; navaltoolbox (AGPL) as an outside cross-check, never a dependency.** The published-case line stays open |
| D14 | Link to #341 | (a) #341 stays 0006's ticket; 0012 tickets refactor the solver under it; (b) close #341 into 0012 | **(a).** #341's open item (a published validation case) is shared; 0012's verification ticket closes it if one is found |

### Round 2 (decided 2026-10-31, all as recommended)

| # | Decision | Options | Decision and why |
|---|---|---|---|
| D15 | Intermediate stages: which | (a) 25, 50 and 75 % of the final flood, for every Rule Set (ES-TRIN 28.03(3) fractions); (b) finer steps | **(a).** The only rule in the set that names fractions; IGC says "any stage" |
| D16 | Flooding sequence | (a) every damaged Space fills together at the same fraction; (b) outer spaces (wing, DB) first, then the hold | **(a).** Simple, symmetric with the final stage; (b) needs flow paths the model does not hold |
| D17 | Free surface of flood water at intermediate stages | (a) gross surface (ES-TRIN 28.03(4)(e)); (b) surface × permeability | **(a).** It is the text, and conservative |
| D18 | A breached tank's cargo at intermediate stages | (a) the cargo stays aboard until the final stage; (b) it leaves in step with the inflow | **(a).** Conservative on sinkage and heel; IGC 2.5.3 speaks of the final plane only |
| D19 | Which damage cases the drawing shows | (a) the governing case per Rule Set and condition; (b) every case | **(a).** A few sheets, not hundreds; the table covers the rest |
| D20 | Spec 0006's MARPOL path keeps the oil in a breached tank. MARPOL I 28.4.1 says to account for "any outflow of liquids from damaged compartments" | (a) a follow-up ticket under #341: remove the oil per 28.4.1 (crude results move, design-impact table); (b) keep it and say in the trace that it is a conservative deviation | **(a).** It matches the text and D8 |
| D21 | Results hash | (a) damage block unhashed, as the liquid one is; the subdivision moves the gas Spaces table with a design-impact table; (b) hash the damage block | **(a).** Same as #341 |

## 3. Domain model

- **Flooded Volume** replaces `FloodedBox`: a Space shape (prism or tank) with a permeability, plus a list
  of **Intact Inclusions** (Spaces inside it that keep their buoyancy, such as a Type C tank). The box is
  the prism with four corners, so every liquid result stays the same.
- **Damage Extent**: a rule's assumed damage (side or bottom; long, deep, high; forward zone), from the
  Rule Set as data.
- **Damage Case**: one placed extent, the Spaces it floods, the tanks it breaches, the condition it
  damages.
- **Standard of Damage**: which bulkheads may be damaged (IGC 2.6.1 by ship type and length; ADN
  one-compartment engine room).
- **Permeability Table**: per Rule Set, keyed on `SpaceKind`.

## 4. Effects on the engine

| Area | Change |
|---|---|
| `damage_stability.rs` | Flooded Volumes of prism and tank shape; intact inclusions subtracted from a flooded hold; a breached tank removes its cargo mass (D8). The box path keeps its numbers (liquid results hash unchanged) |
| `vessel_model.rs`, `compartments.rs` | Gas holds per tank in line; DB and wings cut at hold bulkheads; the strip above the wings as a void Space (D6) |
| New `damage_cases` module | One generator for every rule with box extents: ADN (types C, N, G), ES-TRIN 28.03, IGC, MARPOL, NORMAM. The liquid generators move here unchanged |
| Loading conditions | Each gas condition is damaged (D10) |
| Rules | `igc-2g` 2.1.0: `igc/2.7/damage-survival` evaluated per IGC damage case; new object `igc-damage-case` |
| Drawings | Phase 2: plan and section per governing case, flooded Spaces hatched, damaged waterline |

## 5. Phasing

| Phase | Content |
|---|---|
| 1 | Flooded Volume refactor; gas subdivision; IGC cases, permeabilities, criteria; final **and intermediate** stages; case table, GZ and trace; **damage drawing**; verification |
| 2 | `adn-gas` (D3); ES-TRIN 28.03; NORMAM 5.23.8/9 |
| Roadmap | Openings input and the IGC 2.7.1.1 / ADN 0.10 m opening tests (maintainer's note on D11) |

## 6. Proposed tickets (draft, for the PR body)

1. Refactor: `FloodedBox` → Flooded Volume (prism, tank, intact inclusions). No result moves. — none
2. Gas hull subdivision: holds per tank in line, DB and wings cut, void strip Space. Spaces table moves. — none
3. Damage case generator module; move the ADN and MARPOL generators; add IGC extents and standard of damage. — 1
4. Permeability tables per Rule Set as data. — 3
5. Gas damage run, final stage: every condition, breached tanks lose cargo, case table and GZ in results; `igc-2g` 2.1.0 evaluates 2.7.2.1. — 2, 3, 4
6. Intermediate stages by added mass at 25, 50 and 75 % (D15–D18); IGC 2.7.1.2 and 2.7.1.3 evaluated. — 5
7. Damage case drawing (D19). — 5
8. Verification: cylinder-in-hold closed form, grid integral for a gas hold, intermediate-stage closed form, navaltoolbox cross-check fixture. — 1, 2, 6
9. (phase 2) `adn-gas` set; ES-TRIN 28.03 rows; NORMAM 5.23.8/9 evaluated. — 3, 4, 6
10. (follow-up under #341, spec 0006) MARPOL I 28.4.1: a breached cargo or slop tank loses its oil; crude results move with a design-impact table (D20). — 3

## 7. Not in scope

Probabilistic damage (D4); time-domain flooding; cross-flooding devices; oil outflow (0006); passenger
vessels; anti-flotation loads on tank supports (a later structure item).
