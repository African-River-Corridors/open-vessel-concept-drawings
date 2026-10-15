# Scope: concept and preliminary design for inland and coastal vessels, never approval

The project will grow from concept drawings into stability, strength, construction drawings, quantities,
cost and voyage simulation. Before that work starts, it must say what it claims and for which vessels.
The answer sets which rule sets the engine holds, how results are worded, and what "trusted" means.

## Decision

- **Design stage:** concept design and preliminary design. Never class approval, never a stability booklet
  for an approved vessel, never construction approval.
- **Vessel envelope:** inland and coastal vessels below the size where SOLAS applies in full, and tug-barge
  systems: pushed barges and convoys, towed barges, and articulated tug-barges (ATB), with the tugs that
  drive them.
- **Seagoing cargo ships under the IACS Common Structural Rules are out of scope until after version 1.0.**
  Widening the envelope needs a new ADR.
- **Every result says what it is:** the rule set and edition used, each rule checked with its citation, and
  a plain statement that the output is a concept or preliminary design, valid inside the calibrated range.

## Consequences

- The rules engine holds the rule sets that fit this envelope first: the IMO Intact Stability Code 2008,
  IGC for gas cargo, ES-TRIN and similar inland rules, and inland class rules for barges.
- A class reviewer or a lender's engineer can rely on a specific released version and its verification and
  validation report, never on `main`.
- The app, the README and every report carry the scope statement.
