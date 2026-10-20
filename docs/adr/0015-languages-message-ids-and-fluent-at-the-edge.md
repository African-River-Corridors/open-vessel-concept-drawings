# Languages: message ids from the engine, Fluent at the edge

OVCD's users read English, Spanish, Portuguese, Simplified Chinese and Twi. ADR 0008 chose Fluent and
left open how the engine's English reaches a translation. Today the engine writes free English with
`format!()`, and that English is inside the results hash: check names and messages, warnings, the
stability limitations and the vessel and cargo names are all hashed. So the engine must not change
its English, and a translation must not depend on matching English text. This ADR settles both and
amends ADR 0008.

## Decision

- **One format: Fluent.** Every user-facing string lives in a Fluent (`.ftl`) file in the **Text
  Library** (`locales/<language>/`). The front end formats with `@fluent/bundle`; Rust uses
  `fluent-bundle` where it formats. Both are the reference implementations of one syntax, so one
  file serves both. ICU MessageFormat has no maintained `no_std` or small Rust implementation and is
  harder for volunteers to read. A flat key-value file has no plurals, so it forces joined
  fragments, which do not translate.
- **The engine emits a Message Id with typed arguments beside its English.** `ovcd-core` stays
  `no_std` and does no translation. Where it writes a message it also records the Message Id and its
  arguments: a number (its value, and its text as the engine prints it), a count, a plain text, or
  a clause citation. One macro writes both from one call, so the English cannot drift from the id.
- **The English stays the engine's own, and stays hashed.** The results keep their English text,
  byte for byte. The ids travel in a new results key, `text_ids`, that maps each English text to its
  Message Id and arguments. `text_ids` is outside the results hash, as `drawing` and `cost` are.
  So no stored hash moves, and the design-impact check sees no change.
- **The edge translates.** The app, the web app and, later, the CLI look up an English text in
  `text_ids`, format the id in the Display Language, and fall back to the English text when the id or
  the language has no message. A missing key never shows an id or a blank.
- **Language is a presentation choice only.** No engine function takes a language. The Display
  Language is a per-user setting (the app's settings, later a CLI flag), never stored in a
  `.vdesign` file, never an input. A test proves that the results, their hash and a saved file are
  the same in every language.
- **Not translated:** rule ids, clause citations and quoted rule text (they stay in the language of
  the source rule), unit symbols (SI, plus knots per ADR 0007), file names, Option names and other
  text the user typed, and developer-facing errors (Python exceptions, CLI argument errors), which
  stay English so that they can be searched.
- **Numbers follow the Display Language in the app; drawings keep the decimal point.** The app shows
  the engine's number text with the language's decimal sign (a comma for Portuguese). Drawings follow
  the Drawing Format Standard, which uses the decimal point.
- **Drawings stay English in this phase.** The vendored Drawing Format Standard allows only printable
  ASCII and one font (clause 11.6), and that font has no ɛ, ɔ or CJK glyphs. Drawing labels get
  Message Ids now, rendered in English. Translated drawings wait for a change to the Standard.
- **Every translated message has a status:** `draft` or `reviewed`, with the reviewer's handle and a
  short hash of the English it was reviewed against. Machine or agent translation is a draft only.
  When the English changes, the review lapses to `draft`. Where a language has no established term,
  the message keeps the English term, and the **Term List** records it for a native reviewer.
- **Twi is `ak` (Akan).** CLDR, ICU and the browsers file Twi under `ak`, and CLDR maps `tw` to `ak`,
  so `ak` gets plural rules and number formats with no code of ours. The language is shown as
  "Twi (Akan)" and the text follows Asante Twi spelling.

## Considered options

- **Translate the English text at the edge by matching it.** No engine change, but every message
  with a number in it becomes a pattern, and any English edit breaks a translation silently.
- **Replace the engine's English with ids.** Cleanest, but the hashed English would move, every
  stored hash would change, and the frozen-text tests of spec 0001 would all be rewritten.
- **Put the language in the `.vdesign` file.** A file is shared between people who read different
  languages; the reader's own setting must win, so a stored language would be noise.

## Consequences

- One macro call per message site in the core, and one English test that formats every `text_ids`
  entry with the English Fluent file and compares it to the engine's English.
- A fast catalogue check in CI: missing keys, unused keys, placeholder mismatch, and lapsed reviews.
- The Rule Trace of spec 0004 carries Message Ids for its labels; its rule ids and citations are not
  translated.
- Website and docs translation is a later phase (mkdocs-material language support).
