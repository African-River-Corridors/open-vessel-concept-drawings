<!-- unreleased -->
- CI: WebAssembly parity (`web-parity.yml`) is now a release gate only. `release.yml` calls it; it no longer runs on pull requests, on push or every day. It never failed on a parity mismatch, and it cost a web build on most pull requests. Start it by hand for a change to the web crate.
