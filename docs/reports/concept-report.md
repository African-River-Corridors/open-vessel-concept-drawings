# Concept design report from a run

`tools/concept_report.py` turns one Option of an `ovd run --json` output into a concept design report
(issue #727). The report is an [Open Report Designer](https://github.com/African-River-Corridors/open-report-designer)
(ORD) document; `ord render` makes PDF, HTML and Word from it.

Every figure comes from the run. The script types no number, so a report cannot say something the run
does not.

## Use

```
ovd run --json examples/gas-140x29-coastal-lng/gas-140x29-coastal-lng.vdesign > run.json
python3 tools/concept_report.py run.json --option "C deep" --ref "A basis" -o report.json
python3 tools/concept_report.py run.json --option "C deep" -o report.json --render out/ --ord ord
```

`--ref` puts a reference Option beside the main one in the particulars table. `--render` also runs
`ord check` and `ord render`.

## What the report holds

1. **The design at a glance.** Main particulars, with the basis of each value.
2. **Lightship and hull steel.** The lightship by item; the items add to the lightship (checked). Both
   hull steel figures OVD prints, top-down and bottom-up, and the gap between them.
3. **Hydrostatics and stability.** A hydrostatic table by draught with Cb, and the loading conditions.
   The note says whether every IS Code 2008 A.2.2 criterion passes, from the trace.
4. **Powering and gas balance.** The method is named as the engine runs it: ITTC-1957 friction times a
   bluff-barge Cr/Cf fit on the Froude number. It has no hull-form term.
5. **How the rules were applied.** A compliance matrix (rule, clause, requirement, value, limit,
   outcome, verified or not), one row per governing or warning Rule with its worst outcome. Then a short
   paragraph on each of those Rules, built from its trace lines, and the Rules not evaluated.
6. **Appendix: run provenance.** Engine version, commit and results hashes.

## Use it from a project script

A project's own narrative and trade studies stay with the project. Import the module and call the
parts: `particulars`, `lightship`, `steel`, `hydrostatics`, `stability`, `gz_chart`, `powering`,
`compliance`, `rule_paragraphs` (with an optional note per rule id), `not_evaluated`, `provenance`,
`ga_figures` (runs `ovd draw`, needs `rsvg-convert` and Pillow) and `render`. `build` puts the standard
parts together; pass `assets` to add the GZ chart and GA views.

## Tests

`tests/test_concept_report.py` builds each part from the stored run of the gas example and checks it
against that run. The ORD check runs when `ord` is on the path.
