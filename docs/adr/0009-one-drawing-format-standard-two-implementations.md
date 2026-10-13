# One Drawing Format Standard with two implementations: Python for agents, Rust for the app

Supersedes ADR 0003.

ADR 0003 sent General Arrangements through a separate Python drawing tool. The app is Rust, and it
must draw without a Python install. Agents still want a scriptable Python drawing tool.

## Decision

There is one **Drawing Format Standard**: a written specification of how drawings are described (a
data format, layers, symbols and title blocks). It has two implementations:

- Python, in the drawing tool's own repo, for agents and scripts.
- Rust, in a drawing crate in this workspace, for the app.

Both read the same standard and must give the same drawings.

## Consequences

- Extracting the Drawing Format Standard is a separate spec in the drawing tool's repo. It is out of
  scope for spec 0001.
- The calculation core still has no drawing dependency. Only the output layer draws.
- GA drawings and DXF in the app wait for the standard (see the roadmap).
