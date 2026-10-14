# Translations with Fluent: English, Simplified Chinese, Latin American Spanish

The app and the reports are translated with [Fluent](https://projectfluent.org/). The first three
languages are English, Simplified Chinese and Latin American Spanish.

## Decisions

- Message text lives in Fluent files, not in code.
- Fluent handles plurals and grammar per language, so messages are whole sentences with named
  variables, never joined fragments.
- Translations are display only. Files and core results stay language-neutral and SI.

## Consequences

- The core's check messages and warnings are English text that the tests compare exactly (the port's parity harness did too, until it was removed).
  A translation layer sits above the core and maps them. It is a later spec.
- A report crate (PDF with translations) is later work, not part of spec 0001.
