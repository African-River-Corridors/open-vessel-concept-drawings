# ShipD benchmark sample: source

The files here are a sample of the Ship-D dataset, kept to benchmark OVCD's ShipD-style hull
(issue #452, spec 0008 "Benchmark against the dataset").

- **Dataset:** Bagazinski, Noah; Ahmed, Faez, 2024, "Ship-D", https://doi.org/10.7910/DVN/MMGAUS,
  Harvard Dataverse, V1. Licence: CC0 1.0 (Creative Commons Zero, public domain dedication), as the
  Dataverse record states.
- **Paper:** Bagazinski, N. J., and Ahmed, F., "Ship-D: Ship Hull Dataset for Design Optimization
  Using Machine Learning", Proc. ASME IDETC/CIE 2023, Vol. 3A, paper V03AT03A028 (arXiv:2305.08279).
- **Files read from the dataset:** `Ship_D_Dataset/Constrained_Randomized_Set_2/` and
  `Constrained_Randomized_Set_3/`: `Input_Vectors`, and in `GeometricMeasures/`: `Volume`, `Area_WP`,
  `LCB`, `LCF`, `Ixx`, `Iyy` and `WL_Length` (each at the ten draughts T/Dd = 0.12 to 1.0), downloaded
  in their original CSV form (`?format=original`).
- **Sample:** `cargo run -p xtask -- shipd-sample <folder>` shuffles each set's rows with a fixed seed
  (452, xorshift64*) and takes, in that order, the first 100 hulls with no bulb that pass the 49
  constraints of `ovcd_core::shipd::check`.
- **`sample.csv`:** the set, the dataset row (from 0) and the 45 input values, under the dataset's
  header names.
- **`reference.csv`:** the set, the row, the quantity and the dataset's ten values, as the dataset
  gives them (divided by powers of LOA; LCB and LCF measured aft from the stem).
- **`report.md`:** written by `cargo run -p xtask -- shipd-benchmark`.

`NOTICE` thanks the authors.
