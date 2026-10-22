<!-- unreleased -->
- Hull Forms (issue #452, spec 0008): the `shipd` Hull Form is built, without bulbs. `ovcd-core::shipd` is a Sections Table generator written for OVCD from the Ship-D parameter and constraint definitions (Bagazinski and Ahmed; the dataset doi:10.7910/DVN/MMGAUS, CC0 1.0); each definition settled from the dataset's documents is recorded in the module header, and no code is copied. `shipd_parameters` (the dataset's header names; LOA, Bd and Dd come from the Option's length, beam and depth) reaches the hull; a Design's checks name each failed constraint by index and name, and refuse a bulb (#453). The Defaults Profile's parameters are a shallow, nearly flat-bottomed hull of the benchmark sample. A ShipD Design is cut at 41 stations and 20 height levels, so it stays fast; in the IS Code loading conditions a run still takes about 21 s (#567). The benchmark (`cargo run -p xtask -- shipd-benchmark`, `calibration/shipd/`): 100 hulls each of the dataset's Constrained Randomized Sets 2 and 3, at a finer cut; 95 % of Set 2 and 98 % of Set 3 pass, Set 2 exactly on the spec's 95 % line. New fixture `shipd_dry_bulk.vdesign`. Old files keep their hash; the design-impact table shows the new case.

<!-- design-impact -->
### ticket/452-shipd-hull (2026-10-22)

Between `main ae4ba7f` and `this branch 2bc82c6`.

| Case | Quantity | Before | After | Change |
|---|---|---|---|---|
| sweep hull shipd 80x14 | `result` | fill: warning: Option 'sweep hull shipd 80x14': `service_area` was not stored in the file; wrote in the Defaults Profile value river-only, marked default \| warning: Option 'sweep hull shipd 80x14': `rule_sets` was not stored in the file; wrote in the Defaults Profile value [{"id":"is2008-a","version":"2.0.0","hash":"7dfa155b0d58383f73d35b9b0c037c89dbd928ad4bf22fe93a0a32b1eaee744f","source":"default"},{"id":"es-trin-2025","version":"1.0.0","hash":"a37a0c98b24d77e205b4cc04d4803b2de27912ba81232cc695a1984ef8e58d6a","source":"default"},{"id":"ovcd","version":"2.1.0","hash":"7daa76a269bcad107d2e87ca16df70669aa985aab05473296555b853124cce96","source":"default"}], marked default \| ovcd fill: 1 error(s); nothing written | computed | changed |
