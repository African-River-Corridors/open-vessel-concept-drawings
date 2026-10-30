# A barge form factor method, checked against barge towing-tank data

Status: **proposed**, 2026-10-30. Issue #735, after ADR 0023 (#723). The default does not change in
this decision; the switch is a question for the maintainer (see "Proposal").

## Context

ADR 0023 added Holtrop-Mennen beside the bluff-barge fit. On wide, shallow, full barges the two differ
by 1.7 to 2.0 times in power, and neither had been checked against barge data. The maintainer wants to
test hulls with a lower block coefficient, so OVD needs a method that ranks hull forms and is known to
hold somewhere on barges.

## Research

`calibration/resistance_results.md` has the research line (searched, exists, reusing, building
because) and the per-case table. In short: two open sources give barge resistance as numbers, Zentari
(2023, pushed convoys of Europa-type barges, deep and shallow water) and Tabaczek and Zawislak (2018,
an inland motor cargo vessel in shallow water). Together they give 22 model-scale points
(`calibration/resistance_cases.csv`): Cb 0.77 to 0.88, L/B 7.6 to 16.3, B/T 4.1 to 8.1, Fn 0.05 to
0.14, h/T 1.2 to deep. No open data cover a short, wide, full barge (L/B under 5).

## Decision

1. A fourth value of `resistance_method`, **`barge_form_factor`**:
   CT = CF(ITTC-1957) x (1 + k1 + dk + w(Fn)), with
   - 1 + k1 the Holtrop 1984 form factor (the hull as `holtrop_mennen` reads it);
   - dk = 0.644 (T/h)^1.72, Millward (1989), the increase of k in shallow water (0 in deep water);
   - w(Fn) = residuary_ratio(Fn) - residuary_ratio(0), the Froude-number rise of the bluff-barge fit,
     for the waves (0 below Fn 0.10).
   OVD's Lackenby and blockage factor stays on top, as for every method. No constant is fitted.
2. Its declared range is the range of the data (`BARGE_FORM_FACTOR_ENVELOPE`): Fn up to 0.137,
   Cp 0.768 to 0.877, L/B 7.59 to 16.29, B/T 4.05 to 8.12 (and h/T from 1.2, documented, not checked).
   Rule `ovd/resistance/method-in-range` (Rule Set `ovd` 3.1.0, not yet released) now also applies to
   it and warns outside.
3. `barge_fit` stays the default. No saved Design, fixture or example result moves.

## Evidence

| Range | barge_fit | holtrop_mennen | barge_form_factor |
|---|---|---|---|
| Deep, pusher + 2 or 4 barges (6 points) | -25 to -7 % | -17 to +18 % | -25 to 0 % |
| Deep, pusher + 1 barge (3) | -39 to -34 % | -35 to -30 % | -36 to -31 % |
| Shallow, h/T 1.2 to 2.0 (13) | -51 to -24 % | -49 to -18 % | -28 to -2 % |
| All 22, mean absolute | 31.9 % | 25.9 % | 17.4 % |

Tests: `crates/ovd-core/tests/resistance_calibration.rs` (the table; every case inside the envelope;
the new method has the smallest mean error over all cases and in shallow water; the barge fit is low on
every case) and `crates/ovd-core/tests/resistance_method.rs` (a finer hull needs less power from Cb
0.97 to 0.80; the range check; the Millward term).

## Proposal (for the maintainer)

- **Which method holds where.** Deep water, Cp 0.77 to 0.85, L/B 8 to 16: Holtrop-Mennen and
  `barge_form_factor` are both within about 25 %; Holtrop's wave term is erratic at L/B 16. Shallow water
  (h/T 1.2 to 2): only `barge_form_factor` is within 30 %. The bluff-barge fit is low everywhere it was
  tested, by 7 to 51 %.
- **Refit the barge fit?** Not in place: it moves every stored result and has no form term to refit.
  The data say its form factor (1.10) is too low, and the Holtrop form factor tracks the barge data
  (deep, single-row convoy within 2 %; motor cargo vessel 1.298 against 1.33). `barge_form_factor` is
  that refit, behind the input.
- **Shallow water.** Millward's dk should be added (here, inside `barge_form_factor`). A follow-up could
  apply it to every method; it would move shallow-water results of every Design.
- **Default.** Propose `barge_form_factor` as the default for **new** Designs, with saved Designs kept
  on `barge_fit` through the legacy value, once the maintainer agrees. It ranks hull forms and is the
  closest of the three to barge tests. It still reads about 17 % low on average.

## Not done

- No data for L/B under 7.6 or Fn above 0.14: the coastal LNG concept (L/B 3 to 4.5, Fn about 0.2) is
  outside the range at every Cb, and the warning says so.
- The h/T lower bound of 1.2 is in the documentation, not in the check (the envelope has no depth term).
- Full scale: no barge trial was found, so the correlation allowance is untested.
- Translations of the new choice label (English only).
