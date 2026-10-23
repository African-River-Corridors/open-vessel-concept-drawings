<!-- unreleased -->
- Tests: `tools/check_fixture_hashes.py` also checks the website examples (`website/assets/examples/*.vdesign`) when it runs with no fixtures folder, so the pull request gate fails when a results change leaves an example with a stale hash. `lrg-gas-89.vdesign` is re-stamped: its stored hash was stale on main. No result moves.
