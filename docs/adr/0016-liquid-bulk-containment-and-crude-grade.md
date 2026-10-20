# Liquid bulk containment as typed Spaces; the ADN vessel type comes from the crude grade

Status: **accepted**, 2026-10-20, with spec 0006.

## Context

OVCD carries liquefied gas in Type C tanks and dry bulk in holds. Its only liquid cargo is gas. Crude oil tank barges
and small coastal tankers need rectangular cargo tanks inside a double hull, ballast spaces, cofferdams, slop tanks and
a pump arrangement, plus rules from ADN, MARPOL Annex I and ICLL. The codebase review (`docs/research/crude-oil/04`)
found that a new cargo phase is silently routed down the gas path and drawn as a dry hold today.

ADN 2025 does not give one vessel type for crude oil. Its 3.2.3.3 scheme gives type N or C from the crude's
properties (CMR, benzene, H₂S, aquatic toxicity), and UN 3494 is always type C (`docs/research/crude-oil/02`).

## Decision

1. **A cargo class enum first.** The cargo phase becomes a Rust enum before any crude code lands, so every branch on
   cargo is a `match` the compiler checks. No result changes.
2. **One deep module, `liquid_bulk`, returns typed Spaces** (cargo tank, slop, wing ballast, double bottom, cofferdam,
   pump room) with boxes, volumes and centroids. It is the seed of the roadmap phase 2 compartment model. Integral and
   independent tanks are two adapters at one seam. A new oil is a data row, not a code path.
3. **Crude oil is one Cargo with a Crude Grade.** The grade's flags give the ADN Tank Vessel Type by the 3.2.3.3
   scheme. The user can override it. With unknown flags the type is C.
4. **Rules read Measures from the module's result**, never from its inside.

## Consequences

- New Rule Sets `adn-tank`, `marpol-i` and `icll-type-a`, and four additions to spec 0004 (spec 0006 §5.2).
- The gas Rule Sets must apply only to Type C containment, or a crude Option gets gas rows.
- New results keys stay in the liquid path, so saved hashes of gas and dry bulk files do not move.
- Damage stability and oil outflow start as `not evaluated` rows, so crude stays "work in development" until they
  exist.

## Rejected

- **A crude-specific path beside the gas path.** It repeats layout, clipping and free-surface logic in four callers,
  and roadmap phase 2 would build a second compartment model.
- **One fixed vessel type for crude.** ADN does not give one; a fixed type is either unsafe (N) or overweight for
  sweet low-benzene crude (C).
