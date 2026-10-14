# Rust core with PyO3 bindings and a Rust CLI; port with identical results before fixing

Status: done. Removed in ticket 29: the parity harness, the Reference Results, the parity case inputs, the frozen Python copy, the hidden `ovcd eval` command and the `xtask` regenerate task were deleted. The last commit that still has them is `ce96e66cf0016df3bb36bed1cdc0ae998372564e` (`git show ce96e66cf0016df3bb36bed1cdc0ae998372564e:parity/README.md`; the frozen Python runs from a checkout of that commit with Python 3.12 and numpy).

The calculation core moves from Python to Rust. People who design barges but do not write code need a
free, offline, installable app, and a compiled core is the base for it. The same core later runs in a
web page through WebAssembly.

The port copies Python's results first, and fixes nothing. If we also fixed known engineering errors
during the port, nobody could tell a port error from a deliberate change. A parity harness (spec 0001)
runs fixed cases through the frozen Python and stores Reference Results. The Rust code must match
them: floats within a relative and absolute tolerance of 1e-9, everything discrete exactly.

## Decisions

- One Cargo workspace. `ovcd-core` is pure calculation (`no_std` plus `alloc`, no I/O, builds for
  WebAssembly). `ovcd-format` holds the `.vdesign` format, defaults, validation and the `eval`
  dispatch. `ovcd-cli` is the `ovcd` binary. `ovcd-py` holds the PyO3 bindings, built by maturin into
  the `vessel-designer` wheel. `xtask` holds developer tasks.
- Transcendental maths in the core uses the pure-Rust `libm` crate, and a lint bans the standard
  library versions. Output is then bit-identical on every platform.
- The Python package keeps its public API and calls the Rust core. Until every entry point is at
  parity, the shipped package stays pure Python. One switch pull request then moves it to Rust.
- When the port is complete, the Reference Results and the frozen Python are deleted.

## Consequences

- The tolerance is never loosened to make a case pass.
- Known engineering findings wait until after the port.
- Python users keep `pip install vessel-designer`, now with ready-built wheels.
