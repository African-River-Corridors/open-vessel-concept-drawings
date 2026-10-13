# Tauri desktop app with an HTML/TypeScript UI; the same core in WebAssembly for the web later

The first product for non-coders is a desktop app for Windows and macOS. We build it with Tauri: a
Rust backend that calls `ovcd-format`, and an HTML and TypeScript user interface. The app holds no
calculation logic of its own. It reads a Design and gets results from the one core entry point.

The web version comes later. It uses the same Rust core compiled to WebAssembly, so a browser runs the
same engine as the app. `ovcd-core` has no file, clock or environment access for this reason.

## Consequences

- Units are display-only and belong in the UI, so there is no units crate. Files and the core are SI.
- The earlier static web simulator (Pyodide) cannot load the compiled package. It is removed. A new
  web version on the WebAssembly core is on the roadmap.
- The command-line tool and the app are built in parallel on the same core.
