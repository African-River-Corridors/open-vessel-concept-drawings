# Contributing to open-vessel-concept-drawings

Thank you for helping. Bug reports, fixes, calibration data and new features are all welcome.

## Set up

Python 3.12 or later. With [uv](https://docs.astral.sh/uv/):

```
git clone https://github.com/African-River-Corridors/open-vessel-concept-drawings
cd open-vessel-concept-drawings
uv venv
uv pip install -e '.[dev]'
```

Or with plain pip: `python -m venv .venv && .venv/bin/pip install -e '.[dev]'`.

The install builds a small Rust extension with maturin, so you need the Rust toolchain (`rustup` reads
`rust-toolchain.toml`). A user who installs a built wheel needs no compiler. The release workflow
(`.github/workflows/release.yml`) builds and tests wheels and `ovcd` binaries; it publishes nothing.

## Run the tests

```
OVCD_PYTHON=.venv/bin/python bash tools/gates.sh   # the fast gate: Rust and Python
python tools/release_guard.py
```

Both must pass. CI runs them on every push and pull request. The gate needs the pinned Rust
toolchain (`rust-toolchain.toml`; rustup installs it on first use) and fails if any tool is
missing. For the Rust loop alone, `cargo test` needs no Python.

The Rust core was ported from the Python core with identical results (spec 0001, complete). The
port scaffolding is gone; the spec says where to find it in the history. Every pull request also
reports whether designs moved (`tools/design_impact.py`, see `CHANGELOG.md`).

## Rules for the core

- The calculation core is Rust (`crates/ovcd-core`). `src/vessel_designer/` is a thin layer: it keeps
  the public imports, names and result types, and calls the compiled core through
  `vessel_designer/_bridge.py`. `tests/test_public_api.py` enforces the public API. The core
  stays pure calculation, no I/O; `no_std` and the WebAssembly build check that.
- Every new calculation comes with tests: a golden case (a worked number with its source), a property
  (an invariant), and a feasibility check (the checks fire when they should).
- Cite the source of every number — standard, table and page, or the reference vessel.
- Use the words in `GLOSSARY.md`. If you need a new term, add it there.

## Pull requests

1. Open an issue first for anything bigger than a small fix, so we can agree the approach.
2. Fork, branch from `main`, keep the change focused.
3. Make sure tests and the release guard pass.
4. Open a pull request using the template. Say what changed and how you checked it.

Every pull request from outside the maintainers needs approval from a code owner
(see `.github/CODEOWNERS`) before it merges.

## Licence of contributions

This project is licensed under Apache-2.0. Unless you say otherwise, any contribution you
intentionally submit is licensed under Apache-2.0, as set out in section 5 of the licence.
There is no Contributor Licence Agreement and no Developer Certificate of Origin sign-off to make.

## Conduct

Please follow the [code of conduct](CODE_OF_CONDUCT.md).
