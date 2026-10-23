<!-- unreleased -->
- Docs (#602): ADR 0020 records the solver tolerance stop. Each equilibrium solve stops at the first step whose residual is within a tolerance (1e-13 of the target volume, 1e-11 m of B - G). This replaces the "no early stop" rule of spec 0008, which gets an amendment note. No result moves.
