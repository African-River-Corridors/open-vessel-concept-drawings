# Spec 0005: Languages

Status: **agreed with the maintainer, 2026-10-20.** Decision record: ADR 0015 (amends ADR 0008). Issue: #307.

## Problem Statement

OVD is English only. Its users include engineers in Ghana, Latin America and China. They read
Spanish, Portuguese, Simplified Chinese and Twi, and many read English as a second language.

The English is spread through the code. The app holds about 300 strings in its Svelte files. The
engine writes its check messages, warnings and stability notes with `format!()`, and that English is
inside the results hash. The input labels, choices and group notes come from Rust. The drawings carry
about 35 English labels. Nothing marks which text a user reads and which text is an identifier.

## Solution

A **Text Library** of Fluent files, one folder per language, read by the app, the web app and the
Rust tests. The engine adds a **Message Id** with typed arguments beside each English message. The
app formats the id in the **Display Language** the user picked and falls back to English. Results,
their hash and saved files do not change. Four draft languages come first: Spanish (`es`),
Portuguese (`pt`), Simplified Chinese (`zh-Hans`) and Twi (`ak`).

## User Stories

1. As a Ghanaian engineer, I want the app in Twi where good Twi terms exist, and in English where
   they do not, so that I am never shown an invented word.
2. As an engineer in Brazil, I want numbers with a decimal comma, so that I read them as I write them.
3. As a reviewer, I want a saved file to be the same in every language, so that a translation can
   never change a result.
4. As a volunteer translator, I want one plain-text file per area with the English beside each id,
   and a status line on each message, so that I can review without reading code.
5. As a maintainer, I want CI to catch a missing key, an unused key or a wrong placeholder in
   seconds, so that the library stays whole.
6. As a reader of a rule check, I want the rule id, the citation and the quoted rule text in the
   language of the rule, so that I can check it against the source.

## Implementation Decisions

### Where text reaches a user (inventory, 2026-10-20)

| Source | Where | Size | Phase |
|---|---|---|---|
| App chrome: buttons, tabs, headings, notes, table heads, chart axes | `app/src/**/*.svelte`, `metrics.ts` | about 300 strings | 1 |
| Input labels, choices, groups, group notes | `ovd-format` `vdesign.rs` (`spec(...)`, `CHOICE_LABELS`), sent in the input schema | about 115 inputs, 60 choices | 1 |
| Check names and messages | `ovd-core` (`Check`, about 50 sites), `ovd-format` `simulate.rs` | about 70 | 1 |
| Warnings | `ovd-core` result structs, about 12 constants and 8 inline | about 20 | 1 |
| Stability: condition and criterion names, limitations | `gas_stability.rs` | about 15 | 1 |
| Names in results: `governs`, vessel, cargo, curve names | `simulate.rs`, `cargo.rs` | about 25 | 1 |
| Session messages: range errors, cross-checks, migration notes | `vdesign.rs` `Report`, `session.rs` | about 70 | 1 |
| Capital budget: item names, basis, cost checks | app `CapitalBudget.svelte`, `barge_cost.rs` | about 20 | 1 |
| Drawing labels, notes, view names, status line | `ovd-format` `ga.rs`, `ovd-draw` `furniture.rs` | about 45 | 1 (English only) |
| Desktop dialogs, window title | `app/src-tauri`, `ovd-shell` `state.rs` | about 10 | 1 |
| `ovd run` summary lines, CLI help | `ovd-format` `run.rs`, `ovd-cli` `main.rs` | about 30 | 2 |
| Errors: engine `PyException`, `Result<_, String>`, Python exceptions | core, format, shell, py | about 150 | not translated |
| Website and docs | `website/` | | 3 |

### The Text Library

```
locales/
  en/app.ftl        app chrome
  en/inputs.ftl     input labels, choices, groups, group notes
  en/engine.ftl     checks, warnings, stability, names, session messages
  en/drawing.ftl    drawing labels (English only, phase 1)
  es/ pt/ zh-Hans/ ak/   the same files except drawing.ftl, each message with a status
  terms.toml        the Term List
  REVIEWING.md      the reviewer guide
```

- English (`en`) is the source. Every other file holds a subset of the English ids.
- **Ids** are lower-case kebab case with an area prefix: `app-`, `input-`, `choice-`, `group-`,
  `check-`, `warn-`, `stab-`, `name-`, `msg-`, `draw-`. Input ids derive from the schema:
  `input-<name>`, `choice-<input>-<value>`, `group-<slug of the group name>`. An id is never reused
  for another meaning.
- **Placeholders** are named Fluent variables (`{ $draught }`). A message is a whole sentence, never
  a fragment joined to another.
- **Status.** Each message in a translation has a comment line above it:
  `# status: draft`, or `# status: reviewed @handle a1b2c3d4`, where the eight hex digits are the
  start of the SHA-256 of the English message source. If the English changes, the review has lapsed.

### The engine (Rust)

- `ovd-core` gets a `text` module: `Msg { id: &'static str, args: Vec<(&'static str, Arg)> }`
  and `Arg = Num { value: f64, text: String } | Count(i64) | Text(String) | Cite(String)`.
  `Num.text` is the number as the engine prints it today (`pyfmt`), so a translation shows the same
  digits with the language's decimal sign.
- A macro `text!(id, "English {a} …", a = …)` returns the English `String` and the `Msg` from one
  call. The English is the same `format!` string as today, so it is byte-identical by construction.
- The result structs carry the `Msg` beside the English (`Check.msg`, warnings as `Text`).
- `simulate` writes a results key **`text_ids`**: an object from each English text in the results to
  its `{ "id": …, "args": { … } }`. `text_ids` joins `UNHASHED_KEYS` in `run.rs` and in
  `tools/design_impact.py` (the existing test keeps the two lists equal).
- Session messages (`Message`) and outcome messages get an optional `msg` field. They are not hashed.
- No Rust function takes a language. Rust formats Fluent only in tests (the English test below) and,
  in phase 2, in the CLI.

### The front end

- One module, `app/src/lib/i18n.ts`, is the only door to the Text Library:
  - `t(id, args?)` formats an app id;
  - `say(text, ids?)` translates an engine English text through `text_ids` (or a `msg`), else
    returns the English;
  - `num(value, digits)` and `numText(text)` format numbers in the Display Language;
  - `language` (a Svelte store) and `setLanguage(tag)`.
- The `.ftl` files are bundled with Vite (`?raw`) into both builds, so the web app needs no fetch.
- **Fallback:** a message missing in the Display Language falls back to English; an id missing in
  English (a bug the check catches) shows the engine's English text, or the id in the app.
- **Language selection:** a picker in Settings (app bar), beside the theme control. The choice is kept per
  user (local storage; a Tauri setting later). The first default is the system language if it is one
  of the five, else English. `<html lang>` follows the choice, so browsers pick the right CJK glyphs.
- **Numbers:** results use `Intl.NumberFormat` with the language's locale: `en-GB`, `es-419`,
  `pt-BR`, `zh-Hans-CN`, `ak-GH`. Only `pt-BR` uses the decimal comma. An input box shows its stored
  number with the language's decimal sign and accepts that sign when typed; the stored value is the
  same number.

### Fonts and text width

- The bundled fonts were checked after #300: the woff2 files have the same character maps as the
  TTFs they came from. No glyph was lost. None of them has ɛ, ɔ, Ɛ, Ɔ or CJK; all have the Spanish
  and Portuguese letters.
- The UI font stacks gain system fallbacks: `system-ui`, `"Segoe UI"`, `"Noto Sans"`,
  `"DejaVu Sans"` for ɛ and ɔ, and `"PingFang SC"`, `"Microsoft YaHei"`, `"Noto Sans CJK SC"`,
  `"Source Han Sans SC"` for Chinese. A bundled subset font (by `unicode-range`) can replace the
  fallback later, if the screenshots show a mixed look that hurts reading.
- The UI lets text wrap. Table heads and tabs take `overflow-wrap: anywhere` for long Portuguese and
  Spanish words; Chinese uses `line-break: strict`. The screenshots of phase 1 are the overflow check.
- **Drawings** stay English, in Barlow, with the decimal point. The Standard's width table (104
  characters) and its ASCII character set (clause 11.6.2) would both need CJK and ɛ/ɔ entries, a CJK
  font embedded in the PDF and named in the DXF, and wider leader stacks. An issue on the Standard
  asks for this. Until then `drawing.ftl` has English only and the catalogue check skips it.

### The Term List

`locales/terms.toml` lists the controlled naval-architecture terms, each with its English, a pointer
to `GLOSSARY.md` where the term is defined there, and per language the chosen term, a status and a
note. A Twi entry with no established term keeps the English term and has the status `needs-native`.
The check validates the file's shape. A translator uses the listed term every time.

### The catalogue check

`tools/i18n_check.py`, standard library only, run in the `test` job and by `tools/gates.sh`:

- every id used in the app source (`t("…")` literals) and in the Rust sources (`text!("…"` literals)
  is in English; every English id is used, except the derived `input-`, `choice-` and `group-` ids,
  which a Rust test checks against the input schema;
- every translated id is in English (no orphans);
- each translated message has the same placeholders as its English;
- each translated message has a status line, and a `reviewed` one matches the English hash.
  `--fix` sets lapsed reviews back to `draft`.

It reads only text files and finishes in well under a second.

### Tests

- **Results do not depend on language.** A Rust test: `text_ids` is excluded from the hash, and the
  hash of every fixture Option is unchanged. A web e2e test: evaluate and save a Design in each
  language; the results hashes and the saved bytes are identical.
- **English agrees.** A Rust test formats every `text_ids` entry of every fixture and example with
  `locales/en/engine.ftl` (`fluent-bundle`, a dev-dependency) and compares it with the engine's
  English text.
- **English output is unchanged.** The existing tests that pin English text, the fixture hashes and
  the web hash parity job stay green with no edit.
- **Screenshots.** The e2e screenshot set runs in each language, light and dark.

### Pull requests

1. This spec and ADR 0015 (docs).
2. The Text Library and `i18n.ts`, with every app string moved to `app.ftl`, English only, and the
   catalogue check.
3. Engine messages: `text!`, `text_ids`, session `msg`, input labels; the English test.
4. Drawing labels into `drawing.ftl` (English), with no change to drawing bytes.
5. The four draft languages, the picker, fonts, screenshots and the reviewer guide.

## Out of Scope

- Translated drawings (they wait for the Standard).
- CLI and Python output in other languages (phase 2: a `--lang` flag and `OVD_LANG`).
- The website and docs (phase 3: mkdocs-material language support, one folder per language).
- Right-to-left languages.
- A translation platform (Weblate and similar). The files are plain text in the repo; a platform can
  come later if volunteers ask for one.

## Open questions for the maintainer

1. Spanish numbers use `es-419` (decimal point). Several South American countries use a decimal
   comma. Is a separate number-format setting worth adding later?
2. A bundled fallback font for ɛ and ɔ, if the system fallback looks poor in the screenshots.

## Further Notes

- Twi has few established technical terms. The draft keeps English for naval-architecture terms and
  lists them in the Term List for a native speaker with marine knowledge.
- Spec 0004's Rule Trace will carry Message Ids for its labels, not for its rule ids or citations.
