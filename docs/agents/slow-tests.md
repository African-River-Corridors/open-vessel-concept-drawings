# Slow tests: left out of the PR gate, run nightly

The `test` job of every PR runs the fast gate (`tools/gates.sh`) with the `pr` profile. The tests
below are left out of it and run in the `nightly` workflow (`.github/workflows/nightly.yml`), every
day and on any PR that changes this list, the nextest profiles or the gate script. No test is
deleted. Run every test locally with `GATES_NEXTEST_PROFILE=nightly bash tools/gates.sh`.

The `test` check is split (issue #561). `ci.yml` runs the gate in four parallel jobs: `gate-lint`, `gate-rust` (four
partitions of the Rust tests), `gate-impact` and `gate-python`. The job named `test` waits for all of them and
passes only if each passed. `tools/gates.sh` takes `GATES_STEPS` (a list of step groups) and `GATES_PARTITION`
(for example `2/4`) to run one part; with neither set it runs everything, as before.

A test moves here only by its measured time, and never if it proves a gate: the evaluation harness
false-pass check, the fixture and results hashes, WebAssembly parity, the release guard and the
embargo check, or the design-impact check.

`tests/test_gates_slow_list.py` checks that this table, the `pr` filter in `.config/nextest.toml` and
the tests marked `@pytest.mark.slow` name the same tests.

Times: measured on 2026-10-21 on a developer machine (Apple silicon, 1,185 Rust tests in 326 s of
test time; pytest 285 s in one process). A CI runner is about as fast for pytest and up to two times
slower for Rust.

| Test | Measured | Why it can wait for the nightly run |
|---|---|---|
| `ovd-core::gas_arrangement_feasible no_arrangement_labelled_feasible_has_a_failing_check` | 29.0 s | A property sweep of the gas arrangement search over many hulls. The search's own unit and fixture tests stay in the PR gate, and so do the fixture hashes, which move if a feasibility verdict moves. |
| `ovd-format::ga a_sweep_of_hull_sizes_always_draws_a_valid_sheet` | 26.0 s | A drawing property sweep over hull sizes. Drawings are not in the results hash; the drawing cross-check job still runs on every PR. |
| `ovd-core::gas_arrangement property_the_smallest_viable_length_is_a_length_that_passes` | 11.5 s | A property sweep of the smallest viable length. The fit's unit tests and the fixture hashes stay in the PR gate. |
| `ovd-format::aft_block_inputs each_input_moves_the_aft_block_in_the_drawing_object` | 11.1 s | Moves each of the aft block inputs and checks the drawing object. Drawing data only, not in the results hash. |
| `ovd-format::light_angle_of_max_gz a_light_failure_with_another_criterion_failing_stays_an_error` | 217 s on the CI runner (the JUnit of run 39742410265, with other tests running beside it) | Runs the arrangement search on a 60 x 20 m hull where no candidate passes, so it tries every one. It checks that a light-condition warning is not given when another criterion fails too. The rule itself, and the same rule on feasible hulls, stay in the PR gate (`light_angle_of_max_gz` has 3 other tests); the fixture hashes still move if a verdict moves. |
| `tests/test_evaluation_runner.py::test_the_mutants_file_is_what_the_generator_writes` | 165.9 s | Regenerates `evaluation/mutants.yaml` from the Python oracle and the base cases, and checks the file is current. The file changes only when `evaluation/` changes (3 commits so far), so the PR `test` job still runs this check whenever a PR changes `evaluation/`, and on every run that is not a PR. The false-pass check itself (`evaluation/run.py --check` and `test_the_real_corpus_has_no_false_pass_and_its_results_are_current`) stays in every PR. |
