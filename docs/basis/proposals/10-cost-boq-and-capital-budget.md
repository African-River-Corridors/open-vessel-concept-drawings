# Proposals: cost, BOQ and capital budget

Not published. Companion to `docs/basis/10-cost-boq-and-capital-budget.md`. Ids `CB-A..` refer to the
assumptions table on that page.

## Errors

Places where the code disagrees with itself, with its own labels or with what it tells the user.

**E1. Two steel masses in one Option.** The hull steel line prices the bottom-up steel (hull steel total + wing
plate + weather shield; `crates/ovcd-format/src/cost.rs` lines 76-78, `crates/ovcd-core/src/barge_cost.rs`
line 349). The lightship uses the top-down steel, 0.10 t/m³ × moulded volume
(`crates/ovcd-core/src/weights.rs` lines 163 and 176). The piping and outfitting masses are fractions of the
top-down steel (lines 170, 173). So the budget and the displacement rest on different steel. The difference is
`steel_delta` (line 167), which the budget does not show. Effect: large, sign unknown per design.

**E2. The basis line never changes.** The view always shows "Example rates — replace with yours."
(`crates/ovcd-format/src/vdesign.rs` line 241; `app/src/lib/views/CapitalBudget.svelte` line 66), and the
`fabrication_rates_are_estimates` note always shows (`barge_cost.rs` lines 392-398). After a user enters yard
quotes, the view still calls them example rates. Effect: none on numbers; misleads the reader.

**E3. The currency is not an input.** It is always the example file's `USD` (`cost.rs` lines 13-14, 38). A user
who enters rates in another currency gets a budget labelled USD. Effect: none on numbers; wrong unit label.

**E4. Tank internals are priced as supports.** The tank supports mass is shells × (tank structure factor − 1),
and the code comment says it holds "internal stiffeners, ring frames, support saddles/stools"
(`weights.rs` lines 76-77). The whole mass takes the supports rate (2,000 per t), not the tank shell rate
(10,000 per t). Stiffeners and ring frames inside a Type C tank are normally of the tank material. Effect:
small to medium, likely non-conservative.

**E5. The mass check is partial.** `masses_positive` tests only min(tank shell, hull steel) > 0
(`barge_cost.rs` lines 374-384). Its message, "mass-model inputs present", reads as a check of all masses.
Effect: none today (the gas path gives positive masses); a weak guard.

**E6. Tug in every gas budget.** `cost_of` adds the tug lines for any gas Option (`cost.rs` lines 151-160). It
does not check the Formation. To verify: a gas Option that is a self-propelled Hull or a towed barge still gets
a pusher tug in its budget. Effect: large for those Formations (tug subtotal 1,322,500 at example rates).

## Gaps

What a cost engineer would expect that OVCD does not do.

- **G1. No bill of quantities.** The BOQ view is a placeholder (`app/src/lib/MainPanel.svelte` lines 160-163).
  The capital budget shows amounts only, not quantity, unit and rate per line.
- **G2. No accuracy class.** The page and the app do not say how accurate the figure is.
- **G3. No contingency.** The yard margin (CB-A15) mixes engineering, class, trials and margin in one fraction.
  No design allowance, no estimating contingency, no escalation, no owner's costs (supervision, spares, first
  fill, financing).
- **G4. No price base date or index.** The example rates carry no date (CB-A02). A user cannot escalate them.
- **G5. Tug not sized.** Three constants (CB-A19 to CB-A21) stand for the tug. The power is not linked to the
  powering result (page 07).
- **G6. Lump sums do not scale.** Cargo system, paint, wheelhouse, tug outfitting and delivery are flat per unit
  (CB-A09, CB-A10, CB-A13, CB-A14, CB-A16).
- **G7. One tank shell rate for all materials.** 9 % Ni, 304L, 316L and S690QL1 take the same rate (CB-A04).
- **G8. Insulation priced per tonne.** At 60 kg/m³ the mass is small and a poor driver. Area × thickness is the
  usual basis (CB-A06, CB-A26).
- **G9. No budget for dry bulk or liquid Cargo** (CB-A48).
- **G10. Three copies of the default rates**: the example YAML, `builtin_profile.json` (lines 647-731) and the
  `LegacyFixed` list in `vdesign.rs`. A test keeps them equal, but each change needs three edits.
- **G11. Mass factors that drive cost are fixed in code** (CB-A34 to CB-A40, CB-A42). Each is a candidate input
  under #4. They belong to page 02 but move this page's result.
- **G12. No price evidence.** No calibration case has a published build cost.
- **G13. Glossary terms missing:** Capital Budget, Bill of Quantities (BOQ), Cost Rate, Lump Sum, Yard Margin,
  Unit (one barge plus its tug), Unit Total, Accuracy Class.

## Recommended accuracy class statement

Proposed text for the page and the app's Capital budget view:

> This is a Class 5 estimate in the AACE International cost estimate classification (Recommended Practice
> 18R-97). It is for concept screening only. The real cost can be as much as 50 % below or 100 % above this
> figure (low end −20 % to −50 %, high end +30 % to +100 %), and more with example rates. With yard quotes for the rates, it stays Class 5 until a BOQ exists.

Basis and source:

- The summary I read of AACE RP 18R-97 gives Class 5 as 0 % to 2 % project definition, for concept screening,
  with a low range of −20 % to −50 % and a high range of +30 % to +100 %. Class 4 is 1 % to 15 % definition,
  −15 % to −30 % and +20 % to +50 %. The ranges are at the 80 % confidence level. Source read: a secondary
  summary at epcland.com (<https://epcland.com/?p=52586>), not the AACE text. **Verify against the AACE text
  before publishing.**
- 18R-97 is written for the process industries. The generic matrix is RP 17R-97 (this is from prior knowledge,
  not from a text I read in this task). A reviewer should decide which practice to cite for shipbuilding.
- Why Class 5: the masses are parametric (fixed factors, a box hull), the tug is not sized, lump sums are flat,
  and the rates are examples. That is 0 % to 2 % definition.

## Changes

Each with its effect on results and its place in the order of work.

| # | Change | Fixes | Effect on results | Order |
|---|---|---|---|---|
| C1 | Add the accuracy class statement to the page, the view and the `cost` result | G2 | None | 1 |
| C2 | Make the basis line follow the rates: "Example rates" only while every rate is `default`; otherwise "User rates". Drop the always-on warning when no rate is a default | E2 | None | 2 |
| C3 | Make `currency` and `price_base_date` inputs in the Cost rates group; put a date on the example file | E3, G4 | None | 3 |
| C4 | Show quantity, unit and rate on each budget line (from `cost.masses` and the rates) | G1 (first step) | None | 4 |
| C5 | Add the tug only for Formations that have one; to do so, first verify E6 | E6 | Large for some Options | 5 |
| C6 | Price hull steel on one steel mass. Decide with page 02 whether it is the top-down or the bottom-up mass, and show `steel_delta` until both agree | E1 | Large | 6 |
| C7 | Split the tank structure mass: stiffeners and ring frames at the tank shell rate, saddles at the supports rate. Needs a split factor (Estimate) | E4 | Small to medium, up | 7 |
| C8 | Split the yard margin into engineering and class, yard overhead and profit, and a separate contingency line (input, default from the accuracy class) | G3 | Medium, up if contingency is added | 8 |
| C9 | Key the tank shell rate on the tank material (a table in the Defaults Profile, as for other tank defaults) | G7 | Medium | 9 |
| C10 | Price insulation per m² of insulated surface at the given thickness | G8 | Unknown | 10 |
| C11 | Scale the lump sums: cargo system by tank count and volume, paint by painted area | G6 | Medium | 11 |
| C12 | Size the tug from the powering result (page 07): installed kW and battery kWh | G5 | Large | 12, after page 07 work |
| C13 | Check all masses in `masses_positive` | E5 | None | Any time |
| C14 | Keep the default rates in one place (the profile) and generate the other two | G10 | None | Any time |
| C15 | Budgets for dry bulk and liquid Cargo | G9 | New result | Roadmap phase 6 |
| C16 | A full BOQ from the structural model | G1 | Large | Roadmap phase 4 |
| C17 | Add at least one build-cost calibration case with a published price and date | G12 | None (evidence) | When a public source exists |
