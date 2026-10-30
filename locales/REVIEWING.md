# Reviewing a translation

Thank you for helping. This guide is for a native speaker who reviews OVD's text in their own
language. You do not need to read code. You need a GitHub account, a text editor, Python 3 and
some knowledge of ships or boats. Marine knowledge matters most for the technical words.

OVD is a concept design tool. It is never used for approval. Clear words help a designer read a
result correctly, so your review makes the tool safer to use.

## What is here

```
locales/
  en/          the English source (do not edit it to translate)
    app.ftl      the app's buttons, headings and notes
    inputs.ftl   the names of the inputs, their choices and groups
    engine.ftl   the engine's checks, warnings, notes and check names
    drawing.ftl  drawing labels (English only for now)
  es/  pt/  zh-Hans/  ak/    one folder per language: app.ftl, inputs.ftl, engine.ftl
  terms.toml   the Term List: one word per language for each technical term
  REVIEWING.md this guide
```

The languages are Spanish (`es`, Latin American), Portuguese (`pt`, Brazilian), Simplified
Chinese (`zh-Hans`) and Twi (`ak`, Asante Twi spelling; CLDR files Twi under `ak`).

The English in `inputs.ftl`, `engine.ftl` and `drawing.ftl` is written from the program's source
by a tool. Never edit those English files. Edit only the file of your language.

Drawings stay English for now: the drawing standard allows only ASCII characters and one font.
So no language has a `drawing.ftl`.

## How a message looks

The files use [Fluent](https://projectfluent.org/). Each message is an id, an equals sign and the
text. The line above it says who checked it:

```
# status: draft
app-save = Guardar
```

- **Do not change the id** (the part before `=`).
- **Keep every `{ $name }` placeholder.** The app puts a number or a name there. You may move it
  to where your language needs it. `{ $draught } m` must keep `{ $draught }`.
- **Input names in placeholders.** Where the English has a placeholder in backticks, such as
  `` `{ $input }` ``, the app shows the name of an input there, in your language. Write the
  placeholder without the backticks: `{ $input } debe ser un número`.
- **Keep other backticks.** A word in backticks that is not a placeholder, such as
  `` `results_hash` `` or `` `ovd fill` ``, is a file key or a command. Keep it as it is.
- **Keep units and symbols as they are:** m, t, kn, kW, MPa, °, GM, GZ, KG, UKC.
- **Do not translate** rule ids, clause numbers or quoted rule text (for example "IGC 2.4.1.1"),
  names in capitals that the English keeps (OVD, IMO, IGC, CEMT), or words with underscores
  (`raked_box`, `pushed_convoy`): those are identifiers.
- **Numbers** arrive already written. The app gives them your language's decimal sign. Do not
  write numbers yourself.
- Some messages choose a form by number:

  ```
  app-sp-squat-points = · { $count ->
          [one] { $count } squat-limited point
         *[other] { $count } squat-limited points
      }
  ```

  Keep the shape. Use the plural forms your language has (Chinese and Twi often need only
  `*[other]`). The line with `*` is the default and must stay.

## Use the Term List

Open `terms.toml` first. It lists the technical words (draught, freeboard, heel, cofferdam,
saddle, deadweight and more) and the one word to use for each in your language. Each entry is
`[word, status, note]`.

- Use the listed word every time, so the same thing always has the same name.
- If a listed word is wrong, change it in `terms.toml` in the same pull request and say why. Set
  its status to `reviewed` and put your handle in the note.
- Where the note gives two choices, choose one and delete the other.
- **Twi:** most technical words have the status `needs-native` and keep the English word. If an
  established Twi word exists and marine people use it, write it in and set the status to
  `reviewed`. If no such word exists, keep the English word. **Never make up a new word** and
  present it as the standard term. An honest English word is better than an invented one.

## How to review

1. Read each English message in `locales/en/` and the message with the same id in your language.
2. Fix any message that is wrong, unclear or unnatural. Short and plain is best.
3. When a message is right, change its status line to

   ```
   # status: reviewed @your-github-handle a1b2c3d4
   ```

   where `a1b2c3d4` is the short hash of the English message. You do not need to work it out by
   hand. Run this command from the top folder of the repository, with your language tag:

   ```
   python3 tools/i18n_check.py --hashes es
   ```

   It prints one line for each English message: the id, then the status line to paste. Replace
   `@your-handle` with your GitHub handle.
4. A message that is missing from your file shows in English. You may add it, with a status line.
   Copy the id from the English file.
5. Run the check:

   ```
   python3 tools/i18n_check.py
   ```

   It must report `0 problem(s)`. It finds an id that is not in English, a placeholder that is
   missing or added, a status line that is missing, and a review that has lapsed.
6. Open a pull request with your changes and add the label `translation`. Say which files and
   which language you reviewed.

## When the English changes

If someone changes an English message after you reviewed it, your review no longer applies. The
check notices, because the short hash no longer matches, and reports a "lapsed review". This
command sets each lapsed message back to `draft`:

```
python3 tools/i18n_check.py --fix
```

Review those messages again when you can.

## Seeing your work

Pick your language in the header of the app, beside the Light and Dark buttons. The app keeps the
choice. To see your own changes before a release, run the desktop app from the source (see "Run
the desktop app" in the main `README.md`). The web version at
[africanrivercorridors.com/ovd/app](https://www.africanrivercorridors.com/ovd/app) shows a
language once a release includes it.

Look for text that is cut off or that overflows a button, a tab or a table. Long Spanish and
Portuguese words and Chinese line breaks are the usual places. If you find one, say where in your
pull request.

## Questions

Open an issue with the label `translation`, or ask in your pull request.
