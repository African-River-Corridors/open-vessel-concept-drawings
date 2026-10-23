# ShipD benchmark sample: source

The files here are a sample of the Ship-D dataset, kept to benchmark OVCD's ShipD-style hull
(issues #452 and #453, spec 0008 "Benchmark against the dataset").

- **Dataset:** Bagazinski, Noah; Ahmed, Faez, 2024, "Ship-D", https://doi.org/10.7910/DVN/MMGAUS,
  Harvard Dataverse, V1. Licence: CC0 1.0 (Creative Commons Zero, public domain dedication), as the
  Dataverse record states.
- **Paper:** Bagazinski, N. J., and Ahmed, F., "Ship-D: Ship Hull Dataset for Design Optimization
  Using Machine Learning", Proc. ASME IDETC/CIE 2023, Vol. 3A, paper V03AT03A028 (arXiv:2305.08279).
- **Files read from the dataset:** `Ship_D_Dataset/Constrained_Randomized_Set_2/`,
  `Constrained_Randomized_Set_3/` and `Constrained_Randomized_Set_1/`: `Input_Vectors`, and in `GeometricMeasures/`: `Volume`, `Area_WP`,
  `LCB`, `LCF`, `Ixx`, `Iyy` and `WL_Length` (each at the ten draughts T/Dd = 0.12 to 1.0), downloaded
  in their original CSV form (`?format=original`).
- **Sample:** `cargo run -p xtask -- shipd-sample <folder>` shuffles each set's rows with a fixed seed
  (452, xorshift64*) and takes, in that order, the first 100 hulls that pass the 49 constraints of
  `ovcd_core::shipd::check`: with no bulb for Sets 2 and 3, with a bulb at the bow, the stern or
  both for Set 1. The sets are drawn in the order 2, 3, 1 from one generator, so adding Set 1 left
  the first two unchanged.
- **`sample.csv`:** the set, the dataset row (from 0) and the 45 input values, under the dataset's
  header names.
- **`reference.csv`:** the set, the row, the quantity and the dataset's ten values, as the dataset
  gives them (divided by powers of LOA; LCB and LCF measured aft from the stem).
- **`artefacts.csv`:** the sampled hulls where the dataset's code departs from the definitions, with
  the cause (`stern_ellipse` or `bulb_crossing`, described in `report.md`). Each was found by running
  the dataset's code locally against OVCD's reading; the code is not kept here and not run in CI. The
  test `each_listed_artefact_misses_by_the_definitions_and_the_dataset_code_reading_reproduces_it`
  holds the evidence (issue #648).
- **`report.md`:** written by `cargo run -p xtask -- shipd-benchmark`.

`NOTICE` thanks the authors.
