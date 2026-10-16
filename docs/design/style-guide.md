# OVCD app style guide

How the desktop app looks. It gives the app the same look as the website: a 1930s construction
blueprint. The blueprint field is Prussian blue with white ink. A gilt frame and a cream mat sit around
it.

The guide restyles the app. It does not change what the app does. The layout stays as the Views round
made it (issue #229): the Input panel at the left, Views with two levels of tabs, Outputs below,
hideable side panels and Fit. The drawings follow the Drawing Format Standard. The app does not restyle
what is on a drawing sheet, only the frame around it.

The website uses the same tokens: `website/tokens.css` and `website/site.css`.

## 1 Principles

1. **The drawing is the hero.** The sheet is black on white, as the standard prints it, in both themes.
   The app never tints, recolours or inverts a drawing.
2. **Show the working.** Every number has its unit. Every check has its value, limit, margin and
   citation.
3. **Colour is never the only signal.** Pass, fail, warning, input source and "out of date" each have a
   word and a shape as well as a colour.
4. **Plain and calm.** One ink, one accent, thin lines, small corners. Motion only where it explains.
5. **Craft, not replacement.** The app is an instrument for the designer. The designer decides.

## 2 Colour tokens

Two themes:

- **Light (cream).** The cream mat, with Prussian-blue ink, like a blue-line print.
- **Dark (blueprint).** The blueprint field, with white ink and a gilt accent.

The app follows the operating system's setting. The user can override it (section 9). The `:root`
block in `app/src/app.css` holds these tokens; this section is their source.

```css
:root {
  /* The four colours of the blueprint, the same in both themes */
  --blueprint: #24496c;       /* the blueprint field: splash, app icon, dark theme */
  --blueprint-deep: #1a3753;  /* the darker edge of the paper */
  --white-ink: #eef6f6;       /* line work and lettering on the blueprint */
  --gilt: #a78654;            /* the frame. Decoration only: frames, ornament rules */
  --gilt-mark: #8a6a35;       /* gilt that carries meaning (app-bar rule, selected marker) */
  --cream: #f6edd8;           /* the mat */

  /* Surfaces */
  --paper: #f6edd8;           /* window ground */
  --surface: #fbf6ea;         /* panels */
  --surface-2: #efe4c9;       /* insets: tab rail, panel heads, table heads, code */
  --sheet: #ffffff;           /* the drawing sheet. Never tinted. */
  --sheet-ink: #1c2326;       /* text and lines on the sheet, both themes */
  --sheet-ink-2: #4f585d;     /* captions on the sheet, both themes */

  /* Ink */
  --ink: #1c3550;             /* main text, headings */
  --ink-2: #3c5268;           /* secondary text */
  --ink-3: #56687a;           /* units, hints. Never below 11 px */
  --rule: #dccfae;            /* hairlines and borders (not text) */
  --rule-strong: #8c7b56;     /* input borders (3:1 on surfaces) */

  /* Accent */
  --accent: #24496c;          /* links, focus ring, active level-2 tab, primary button */
  --accent-ink: #eef6f6;      /* text on --accent */
  --accent-soft: #dfe6ea;     /* selected row, active level-2 tab ground, pressed toggle */
  --gilt-ink: #765825;        /* gilt as text */

  /* Status: always with a word and a shape */
  --pass: #2a6339;   --pass-soft: #e3eedc;
  --fail: #9e2a1c;   --fail-soft: #f7e0d6;
  --warn: #73500b;   --warn-soft: #f2e2bd;
  --stale: #56687a;           /* out-of-date results: struck through, with "out of date" */

  /* Input source tags */
  --src-typed: var(--accent);
  --src-default: var(--ink-3);
  --src-class: #5a3f86;       /* the value comes from the chosen Vessel Class */

  color-scheme: light;
}

/* Dark: the blueprint field. The same block sits under
   @media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { ... } } */
:root[data-theme="dark"] {
  --paper: #24496c;  --surface: #2a5278;  --surface-2: #1a3753;
  --ink: #eef6f6;  --ink-2: #d2e2e8;  --ink-3: #b9cfda;
  --rule: #4b7192;  --rule-strong: #95b4c6;
  --accent: #e6cd96;  --accent-ink: #1a3753;  --accent-soft: #2f577c;
  --gilt-ink: #e6cd96;  --gilt-mark: #d9bb80;
  --pass: #b9efc3;  --pass-soft: #1f4a4b;
  --fail: #ffc6ba;  --fail-soft: #4e3448;
  --warn: #ffe3a3;  --warn-soft: #43473e;
  --stale: #b9cfda;  --src-class: #e0d4ff;
  color-scheme: dark;
}
```

| Token | Use | Never |
|---|---|---|
| `--blueprint`, `--white-ink` | The blueprint field and its ink: splash, app icon | Behind the drawing sheet |
| `--gilt` | Frames and ornament rules | Text, or any mark that carries meaning |
| `--gilt-mark`, `--gilt-ink` | Gilt that carries meaning (app-bar rule, selected marker); gilt as text | |
| `--paper` | Window ground | Behind a drawing |
| `--surface`, `--surface-2` | Panels; insets, tab rail, panel heads | |
| `--sheet`, `--sheet-ink` | The drawing sheet and what is lettered on it | Tinted or themed |
| `--ink`, `--ink-2`, `--ink-3` | Text: main, secondary, units and hints | `--ink-3` below 11 px |
| `--accent` | Links, focus ring, active level-2 tab, primary button, typed source | Status |
| `--pass`, `--fail`, `--warn` | Check verdicts and input messages | Without the word and the shape |
| `--stale` | Out-of-date results, struck through, with "out of date" | |

**From the old tokens.** `--navy` → `--ink` (headings) or `--accent` (links, active state);
`--gold` → `--gilt-mark` (rules) or `--warn` (notices); `--cream` and `--bar` → `--surface-2`;
`--dim` → `--ink-3`; `--line` → `--rule`. Hard-coded colours in the components (the source tags, the
verdicts, the banners, the stale tag) become tokens. After the restyle, no component has a colour
literal, except the drawing sheet's own SVG.

## 3 Contrast (WCAG 2.2 AA)

Text pairs need 4.5:1. Lines and borders that carry meaning need 3:1. Every pair passes in both
themes. `node app/scripts/contrast.mjs` checks the tokens in `app/src/app.css` and exits 1 on a
failure; `npm run build` runs it.

| Foreground on background | Kind | Light | Dark |
|---|---|---|---|
| `ink` on `paper` | text | 10.76 | 8.53 |
| `ink` on `surface` | text | 11.63 | 7.43 |
| `ink` on `surface-2` | text | 9.92 | 11.16 |
| `ink-2` on `paper` | text | 6.94 | 7.03 |
| `ink-2` on `surface` | text | 7.49 | 6.13 |
| `ink-2` on `surface-2` | text | 6.39 | 9.20 |
| `ink-3` on `paper` | text | 4.93 | 5.79 |
| `ink-3` on `surface` | text | 5.32 | 5.05 |
| `ink-3` on `surface-2` | text | 4.54 | 7.58 |
| `accent` on `paper` | text | 8.03 | 6.03 |
| `accent` on `surface` | text | 8.68 | 5.25 |
| `accent` on `surface-2` | text | 7.40 | 7.89 |
| `accent-ink` on `accent` | text | 8.53 | 7.89 |
| `ink` on `accent-soft` | text | 9.94 | 6.90 |
| `accent` on `accent-soft` | text | 7.42 | 4.88 |
| `pass` on `pass-soft` | text | 5.96 | 7.57 |
| `fail` on `fail-soft` | text | 5.92 | 7.36 |
| `warn` on `warn-soft` | text | 5.69 | 7.59 |
| `ink` on `warn-soft` | text | 9.79 | 8.67 |
| `pass` on `surface` | text | 6.62 | 6.29 |
| `fail` on `surface` | text | 6.95 | 5.45 |
| `warn` on `surface` | text | 6.76 | 6.51 |
| `src-class` on `surface` | text | 7.82 | 5.83 |
| `stale` on `surface` | text | 5.32 | 5.05 |
| `sheet-ink` on `sheet` | text | 15.94 | 15.94 |
| `sheet-ink-2` on `sheet` | text | 7.28 | 7.28 |
| `gilt-ink` on `paper` | text | 5.64 | 6.03 |
| `gilt-ink` on `surface` | text | 6.10 | 5.25 |
| `white-ink` on `blueprint` | text | 8.53 | 8.53 |
| `white-ink` on `blueprint-deep` | text | 11.16 | 11.16 |
| `cream` on `blueprint` | text | 8.03 | 8.03 |
| `rule-strong` on `surface` | line | 3.83 | 3.73 |
| `rule-strong` on `paper` | line | 3.55 | 4.29 |
| `accent` on `paper` | line | 8.03 | 6.03 |
| `gilt-mark` on `paper` | line | 4.30 | 5.07 |
| `gilt-mark` on `surface` | line | 4.64 | 4.41 |

`--rule` is a hairline between areas that are already apart; it carries no meaning, so it has no
contrast target. A border that marks a control (an input box, a toggle) uses `--rule-strong`.

## 4 Typography

| Font | Job | Weights |
|---|---|---|
| **Barlow Semi Condensed** | All UI text: labels, numbers, tabs, buttons, tables. It is the drawings' own font. | 400, 500, 600 |
| **Source Serif 4** | Panel group titles and dialog titles | 400, 600, italic 400 (splash) |
| **IBM Plex Mono** | File names, code, command lines | 400 |
| **Architects Daughter** | Display only: view titles and the splash. Capitals, slanted (`font-style: oblique 10deg`), letter-spaced. | 400 |

```css
--font-ui: "Barlow Semi Condensed", system-ui, sans-serif;
--font-serif: "Source Serif 4", Georgia, serif;
--font-mono: "IBM Plex Mono", ui-monospace, Menlo, monospace;
--font-hand: "Architects Daughter", "Barlow Semi Condensed", sans-serif;
```

- **The hand lettering is for display only.** It is never used for inputs, numbers, tables, messages
  or body text.
- **The app bundles its fonts.** It runs offline, and never loads Google Fonts or any other web font
  service at run time. The font files live in `app/src/fonts/`, each with its SIL Open Font License in
  `app/public/licenses/`. The drawing font stays where the drawing crate vendors it
  (`crates/ovcd-draw/standard/fonts/`).
- **Scale (px):** 11 tags and captions · 12 panel heads · 13 UI body · 14 tabs and buttons · 16 group
  titles (serif) · 20 dialog titles (serif) · 24, 32, 48 splash.
- **Line height:** 1.35 in the UI, 1.6 for reading text.
- **Numbers:** `font-variant-numeric: tabular-nums`, right-aligned, the unit after the number in
  `--ink-3`.

## 5 Space, lines, corners

- **Space:** a 4 px step: `--sp-1` 4, `--sp-2` 8, `--sp-3` 12, `--sp-4` 16, `--sp-5` 24, `--sp-6` 32,
  `--sp-7` 48.
- **Lines:** thin 1 px, thick 2 px, extra thick 4 px (`--lw-thin`, `--lw-thick`, `--lw-extra`): the
  drawing standard's 1 : 2 : 4. Thin for hairlines and inputs. Thick for the app-bar rule and table
  heads. Extra thick only for the top of a dialog.
- **Corners:** 3 px on controls (`--radius`), 6 px on dialogs (`--radius-lg`). Panels and the drawing
  canvas have square corners.
- **Shadows:** none in the panels. One soft shadow on dialogs only.

## 6 Icons

24 px grid, 1.5 px single stroke, round caps and joins, no fill, `currentColor`. They are drawn like plan
lines. Status icons differ by **shape** as well as colour: pass is a circle with a tick, fail a square
with a cross, warning a triangle, out of date a struck clock. Every icon button has a text label or an
`aria-label`. Where text fits, the button keeps its word.

## 7 Components

**App bar.** 40 px high, on `--paper`, with a 2 px `--gilt-mark` rule under it (the frame). From the
left: the mark (the 32 px app drawing) and "OVCD"; the file buttons (New, Open, Save, Save as); About;
then, at the right, the panel toggles, the theme control, the file name with "(unsaved)", "Defaults:"
and "Engine x.y.z" in `--ink-3`. The file name uses `--font-mono`.

**Panel toggles.** "Input panel" and "Outputs panel" keep their words and their `aria-pressed`
state. Pressed means the panel is in sight: `--accent-soft` ground and an `--accent` border. Not
pressed: transparent with a `--rule-strong` border. The view refits after a panel moves, as now.

**Panel heads.** 12 px capitals, letter-spaced 0.08 em, `--ink-2` on `--surface-2`, a thin `--rule` under.

**Tabs, two levels.** Level 1 (Drawings · Stability & Propulsion · BOQ & Finance): 14 px, weight 600,
`--ink-2`. The active tab is `--ink` with a 3 px `--ink` rule under it. Level 2 (the views of the
group): 13 px chips with a `--rule-strong` border; the active chip has `--accent-soft` ground and an
`--accent` border. The view tools (zoom, Fit) sit at the right end of the level-2 row. Roles, keyboard
and `aria-selected` stay as they are.

**Buttons.** Default: `--surface` ground, `--rule-strong` border, `--ink` text, 3 px corners, 14 px.
Primary (one per dialog): `--accent` ground, `--accent-ink` text. Hover: 120 ms fade to
`--surface-2`. Focus: a 2 px `--accent` ring, offset 2 px, on every focusable thing.

**Inputs.** The label, then a source tag with a word and a shape:

| Source | Tag | Colour |
|---|---|---|
| Typed by the user | `● typed` | `--src-typed` |
| Profile default | `○ default` | `--src-default` |
| From the Vessel Class | `◆ class` | `--src-class` |

A typed value shows "reset", as now. Number boxes are right-aligned and tabular, with the unit after the
box. Box: `--surface` ground, 1 px `--rule-strong` border. Focus: 2 px `--accent` ring. Error: 2 px
`--fail` border, then `✕` and one sentence with the limit. Warning: `▲` and one sentence in `--warn`.
Help text and tooltips keep their words.

**Outputs.** The name at the left, the number at the right, the unit after it in `--ink-3`. Out of date:
the number struck through in `--stale`, a tag "out of date", and the banner says why.

**Checks.** One row each. A verdict chip with a word and a shape on a soft ground: `✓ PASS`
(`--pass` on `--pass-soft`), `✕ FAIL` (`--fail` on `--fail-soft`), `▲ WARN` (`--warn` on
`--warn-soft`). Then the check in plain words, value against limit and margin, and the citation at the
right in `--ink-3`.

**Compare table.** Head row on `--surface-2` with a 2 px `--ink` rule under it. Rows divided by thin
`--rule` lines. The active Option's column has `--accent-soft` ground. A value that differs between
Options stays bold (the signal that is not colour), on `--warn-soft`. Numbers right-aligned and
tabular.

**Banners.** Error: `--fail` text on `--fail-soft`, with `✕`. Warning and notices: `--warn` on
`--warn-soft`, with `▲`. The words stay as they are. The "Dismiss" buttons stay.

**Charts** (Power and speed curves, Intact stability). Plot ground `--surface`. Axes and grid in
`--rule`, axis labels in `--ink-2`, tick numbers in `--ink-3`, all in `--font-ui`. The main curve is
2 px `--ink`. A second series is dashed and in `--accent`, and has a label on the line as well as in the
legend. Criteria are dashed lines with their words. Areas are hatched, not only tinted. A click readout
is a hollow circle with its values. Chart colours come from the tokens at draw time, so a theme change
redraws the chart in the new colours.

**The drawing canvas.** The sheet (`--sheet`) sits on a `--surface-2` mat with a 1 px `--rule-strong`
border and square corners. It fits the panel, as now. The zoom state sits at the lower left in 11 px
`--ink-3` ("fitted", "zoomed"). What is on the sheet is the standard's business, not the app's.

**View titles.** Over each view, a title in the yard form: the view name in hand lettering, capitals,
slanted, letter-spaced, 16 px, with a double underline (1 px lines, 2 px apart) in `--ink`. Beneath it,
in 11 px `--font-ui` capitals, the scale or state when the app has it ("SCALE 1:500", "FITTED"). The
app shows only what it already knows; it does not work out a scale.

**Title-block device.** A box of thin ruled cells, each with a small hand-lettered label above its value
(as at the foot of the splash). Use it for the splash and for the About dialog. Do not use it for
results.

**Dialogs.** `--surface` ground, 6 px corners, a 4 px `--gilt-mark` rule along the top, one soft shadow.
Title 20 px Source Serif 4, weight 600, `--ink`. The veil behind is `--blueprint-deep` at 55 %.

**The first-run notice.** It keeps its words, its button "I understand", its `data-testid` and its
behaviour (once per user, and again from About). It takes the dialog style, with the four facts in a
title-block (Version, Licence, "Before you start"), and the scope line `▲ Concept design. Results are
not verified.` in `--warn` on `--warn-soft`.

**Empty and placeholder views.** A short line in `--ink-3`, with the hand-lettered view title above it.
For example: "The bill of quantities comes in a later phase. See the roadmap."

## 8 App icon and splash

**App icon.** The section drawing on the blueprint field in a gilt frame. The files come from the brand
set:

- macOS (`icon.icns`, and `icon.png`): the macOS-grid version (rounded square with a margin).
- Windows (`icon.ico`) and the 32 px and 128 px PNGs: the full-bleed square version.
- At 16 px and 32 px the drawing is simplified (fewer lines); use the 16 and 32 px masters for those
  sizes, not a scaled-down 1024.

The icons live in `app/src-tauri/icons/`, at the sizes `tauri.conf.json` lists.

**Splash (loading screen).** It shows while the engine starts, and goes away when the first result
arrives. It is the blueprint field in a gilt frame on the cream mat. At the left: the mark, "OVCD" in
hand lettering and "Open Vessel Concept Design" underlined; under them, one line in Source Serif 4: "An
open instrument for the concept and preliminary design of inland and coastal vessels." At the right: the
midship-section drawing in white ink with its leaders. Along the foot: a title block with Version
(the app version), State ("Starting the engine"), Before you start (the notice words) and Licence
(Apache 2.0). The splash is the same in both themes. It never blocks a click once the app is ready.

## 9 Light and dark

- The app follows the operating system's setting (`prefers-color-scheme`).
- A theme control in the app bar overrides it: **Auto**, **Light**, **Dark**. It sets
  `data-theme="light"` or `data-theme="dark"` on `<html>`, or removes it for Auto.
- The choice is kept between runs, like the panel state. The default is Auto.
- The drawing sheet, the splash and the app icon are the same in both themes.

## 10 Motion

- Results: no animation. New numbers replace old ones at once.
- Panels and tab content: 200 ms, `cubic-bezier(0.2, 0, 0, 1)`. The view refits at the end.
- Hover: 120 ms fade. Focus and pressed: at once.
- The one decorative motion: on the splash, the drawing fades in once (0.9 s). It never loops.
- `prefers-reduced-motion`: everything happens at once.

## 11 Voice

Simplified Technical English: short sentences, active voice, one word for one meaning, the words of
`GLOSSARY.md`. Say what happened, why, and what to do. The restyle does not change the app's words,
except where this guide gives new ones (empty views, the splash).

| Case | Write | Do not write |
|---|---|---|
| Input error | The draught limit is more than the moulded depth (4.20 m). Make it less, or make the hull deeper. | Invalid input! Constraint violation in field draught_limit. |
| Out of range | This hull is outside the calibrated range. Treat the result as a question, not an answer. | Results may be slightly less accurate. |
| Empty view | The bill of quantities comes in a later phase. See the roadmap. | Coming soon! Stay tuned |
| Scope | OVCD is a concept design tool. Its results are not verified. | Class-ready designs in seconds. |

## 12 Craft, not replacement

**Do:** show the working (rule, edition, paragraph, inputs, margin); use the words of the drawing
office (lines plan, offsets, station, fair, body plan); leave the decision with the designer; use real
drawing furniture (borders, title blocks, scale bars) where it helps the reader.

**Do not:** say OVCD replaces naval architects, draughtsmen, yards or class; say "approved",
"certified", "compliant" or "class-ready" about a result; use robots, sparkles or chat bubbles in any
image; age the drawings with sepia filters or fake stains; copy a real yard's, museum's or society's
mark, plan or lettering.
