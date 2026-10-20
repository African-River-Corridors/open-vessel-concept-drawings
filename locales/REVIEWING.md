# Reviewing a translation

Thank you for helping. This guide is for a native speaker who reviews OVCD's text in their own
language. You do not need to read code. You need a GitHub account, a text editor, and some
knowledge of ships or boats. Marine knowledge matters most for the technical words.

OVCD is a concept design tool. It is never used for approval. Clear words help a designer read a
result correctly, so your review makes the tool safer to use.

## What is here

```
locales/
  en/          the English source (do not translate here)
    app.ftl      the app's buttons, headings and notes
    inputs.ftl   the names of the inputs and their choices
    engine.ftl   the engine's checks, warnings and notes
    drawing.ftl  drawing labels (English only for now)
  es/  pt/  zh-Hans/  ak/    one folder per language, the same files except drawing.ftl
  terms.toml   the Term List: one word per language for each technical term
```

The languages are Spanish (`es`, Latin American), Portuguese (`pt`, Brazilian), Simplified
Chinese (`zh-Hans`) and Twi (`ak`, Asante Twi spelling).

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
- **Keep units and symbols as they are:** m, t, kn, kW, °, GM, GZ, KG.
- **Do not translate** rule ids, clause numbers or quoted rule text (for example "IGC 4.23.1"),
  nor names in capitals that the English keeps (OVCD, IMO, CEMT).
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
saddle, deadweight and more) and the one word to use for each in your language.

- Use the listed word every time, so the same thing always has the same name.
- If a listed word is wrong, change it in `terms.toml` in the same pull request and say why.
- **Twi:** most technical words have status `needs-native` and keep the English word. If an
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

   where `a1b2c3d4` is the English's short hash. You do not need to work it out by hand: run

   ```
   python3 tools/i18n_check.py --hashes es
   ```

   (with your language tag) to print the status line for every message, ready to paste.
4. A message that is missing from your file shows in English. You may add it, with a status line.
5. Run `python3 tools/i18n_check.py`. It must report 0 problems. It finds a missing placeholder, an
   id that is not in English, and a status line that is missing.
6. Open a pull request with your changes. Say which files you reviewed.

## When the English changes

If someone changes an English message after you reviewed it, your review no longer applies. The
check notices (the short hash no longer matches), and `python3 tools/i18n_check.py --fix` sets that
message back to `draft`. Review it again when you can.

## Seeing your work

Run the app (see `app/README.md`, or the web version) and pick your language in the header. Look
for text that is cut off or overflows a button or a table. Long Spanish and Portuguese words and
Chinese line breaks are the usual places.

## Questions

Open an issue with the label `translation`, or ask in your pull request.
