# The `.vdesign` file format

A Design is saved as one file with the extension `.vdesign`.

## Decisions

- Versioned JSON, in SI units **except speeds, which are in knots** (the inland-waterway convention):
  `max_speed_kn` and `channel_current_kn`. The `units` field holds `"SI"` and means this. Nothing else
  is non-SI. Storing speeds in m/s would be a format change (`format_version` 2 with a migration);
  it is not planned. Unit switching is only for display and input, in the app.
- A Design holds a named list of one or more Options, the engine version, and a results hash for each
  Option.
- An Option holds the typed `simulate` inputs, using glossary names, plus `hull_form` and `formation`.
- Required inputs: Cargo, Hull Form, length, beam, Formation and moulded depth. Every other input
  comes from the Defaults Profile.
- The file stores every input **as used**, and marks which ones came from a default. A later change to
  the Defaults Profile never changes the results of a saved file.
- Each validation range is hard (the input is refused) or soft (the input is accepted with a warning).
  Validation lives only in `ovcd-format`. The core adds no checks.
- The results hash is the SHA-256 of the canonical JSON of an Option's results. If a file opens on an
  engine version that gives a different hash, the tool warns that the results differ from the hash
  saved in the file. It cannot tell an engine change from a hand edit, so it does not blame either.

## Consequences

- A saved file gives the same results when opened again, on Windows and macOS.
- Moulded depth is required until a default rule by Cargo exists.
- A convoy Formation is valid in the file, but its results are not yet available.
