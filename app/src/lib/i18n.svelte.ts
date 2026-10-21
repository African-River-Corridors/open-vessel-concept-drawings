// The only door to the Text Library (spec 0005, ADR 0015). Every string the app shows comes through
// here, in the Display Language, with English as the fallback. Language is display state only: it
// changes no input and no result, and it is never sent to Rust.
import { FluentBundle, FluentResource, type FluentVariable } from "@fluent/bundle";

export interface Language {
  /** The tag of the Text Library folder. */
  tag: string;
  /** The language's own name, shown in the picker. */
  name: string;
  /** The locale numbers are formatted in. */
  numbers: string;
}

/** English is the source. The others are drafts until a native speaker reviews them (locales/REVIEWING.md). */
export const LANGUAGES: Language[] = [
  { tag: "en", name: "English", numbers: "en-GB" },
  { tag: "es", name: "Español", numbers: "es-419" },
  { tag: "pt", name: "Português (Brasil)", numbers: "pt-BR" },
  { tag: "zh-Hans", name: "简体中文", numbers: "zh-Hans-CN" },
  { tag: "ak", name: "Twi (Akan)", numbers: "ak-GH" },
];
const SOURCE = LANGUAGES[0];

/** Where the choice is kept, per user. It is never saved in a Design and never sent to Rust. */
export const LANGUAGE_KEY = "ovcd.language";

// Every .ftl file of the Text Library, built into the bundle: the web app needs no fetch.
const FILES = import.meta.glob("../../../locales/*/*.ftl", { query: "?raw", import: "default", eager: true }) as Record<
  string,
  string
>;

function bundle(tag: string): FluentBundle {
  // No isolation marks: the English text must stay byte for byte what it was.
  const b = new FluentBundle(tag, { useIsolating: false });
  for (const [path, text] of Object.entries(FILES)) {
    if (path.split("/").at(-2) === tag) b.addResource(new FluentResource(text));
  }
  return b;
}

const BUNDLES = new Map(LANGUAGES.map((l) => [l.tag, bundle(l.tag)]));

/** The Display Language. Reading it in a template makes the template follow a change. */
const state = $state({ language: SOURCE });

export function language(): Language {
  return state.language;
}

export function setLanguage(tag: string) {
  state.language = LANGUAGES.find((l) => l.tag === tag) ?? SOURCE;
  // The browser picks the right glyphs for the language (Chinese, say) from this.
  document.documentElement.lang = state.language.tag;
}

/**
 * The language to start in: the user's kept choice, else the first system language that the app
 * has, else English. Twi is filed under `ak` (CLDR maps `tw` to `ak`); any Chinese but the
 * traditional script is Simplified Chinese.
 */
export function startLanguage(kept: string | null, system: readonly string[]): string {
  if (kept && LANGUAGES.some((l) => l.tag === kept)) return kept;
  for (const wanted of system) {
    const w = wanted.toLowerCase();
    const base = w.split("-")[0];
    if (base === "zh") {
      if (!/-(hant|tw|hk|mo)\b/.test(w)) return "zh-Hans";
    } else if (base === "ak" || base === "tw" || base === "twi") {
      return "ak";
    } else if (LANGUAGES.some((l) => l.tag === base)) {
      return base;
    }
  }
  return SOURCE.tag;
}

/** At start: the kept choice or the system language. Storage can be missing (a private window). */
export function initLanguage() {
  let kept: string | null = null;
  try {
    kept = localStorage.getItem(LANGUAGE_KEY);
  } catch {
    // No storage: the system language applies.
  }
  setLanguage(startLanguage(kept, navigator.languages ?? [navigator.language]));
}

/** The user picked a language: show it now and keep the choice. */
export function chooseLanguage(tag: string) {
  setLanguage(tag);
  try {
    localStorage.setItem(LANGUAGE_KEY, state.language.tag);
  } catch {
    // Not kept; the language still applies for this run.
  }
}

function format(b: FluentBundle | undefined, id: string, args?: Record<string, FluentVariable>): string | null {
  const m = b?.getMessage(id);
  return m?.value ? b!.formatPattern(m.value, args) : null;
}

/** An app message in the Display Language; English when the language lacks it; the id if English does. */
export function t(id: string, args?: Record<string, FluentVariable>): string {
  return format(BUNDLES.get(state.language.tag), id, args) ?? format(BUNDLES.get(SOURCE.tag), id, args) ?? id;
}

/** One argument of an engine message, as the engine sends it (ovcd-core `text::Arg`). */
export type EngineArg = string | number | { num: string; value: number } | { cite: string } | { input: string } | EngineMsg;
/** A Message Id with its arguments, as the engine sends it beside its English (ADR 0015). */
export interface EngineMsg {
  id: string;
  args?: Record<string, EngineArg>;
  /** The English, on a message that is an argument of another. */
  en?: string;
}
/** The `text_ids` of a result: each English text of the result, with its Message Id. */
export type TextIds = Record<string, EngineMsg>;

/**
 * Engine text in the Display Language. The engine's own English is the source: it is shown as it is
 * in English, and whenever the Display Language has no message for it. `msg` is the text's Message
 * Id where the engine sent one with it; otherwise the text is looked up in the result's `text_ids`.
 */
export function say(text: string, ids?: TextIds | null, msg?: EngineMsg | null): string {
  const m = msg ?? ids?.[text];
  if (!m || state.language.tag === SOURCE.tag) return text;
  return format(BUNDLES.get(state.language.tag), m.id, engineArgs(m.args ?? {}, ids)) ?? text;
}

/** Text the engine sends under a derived id (an input label, a choice): the English unless translated. */
export function engineLabel(id: string, english: string): string {
  if (state.language.tag === SOURCE.tag) return english;
  return format(BUNDLES.get(state.language.tag), id) ?? english;
}

/**
 * A check's name in the Display Language. The name is a key of the result, so it is the engine's own
 * text: its Message Id in `text_ids` where it has one, else the derived id `check-name-<slug>`.
 */
export function checkName(name: string, ids?: TextIds | null): string {
  if (ids?.[name]) return say(name, ids);
  return engineLabel(`check-name-${slug(name)}`, name);
}

/** A Message Id piece from an English name or a key, as `crates/ovcd-format/tests/i18n_inputs.rs` makes it. */
export function slug(s: string): string {
  return s.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
}

/** The schema's text in the Display Language: a group, its note, an input label, a choice. */
export const schemaText = {
  group: (g: string) => engineLabel(`group-${slug(g)}`, g),
  note: (g: string, note: string) => engineLabel(`group-note-${slug(g)}`, note),
  input: (name: string, label: string) => engineLabel(`input-${slug(name)}`, label),
  defaultBy: (name: string, by: string) => engineLabel(`input-${slug(name)}-default-by`, by),
  choice: (input: string, value: string, label: string) => engineLabel(`choice-${slug(input)}-${slug(value)}`, label),
};

/** The English label of each input, from the schema: an engine message names an input by its key. */
let inputLabels: Record<string, string> = {};

export function setInputLabels(labels: Record<string, string>) {
  inputLabels = labels;
}

/** An input named in an engine message, as its label in the Display Language. */
function inputLabel(key: string): string {
  if (key === "abreast" || key === "formation.abreast") return t("app-convoy-abreast");
  if (key === "in_line" || key === "formation.in_line") return t("app-convoy-in-line");
  return schemaText.input(key, inputLabels[key] ?? key);
}

function engineArgs(args: Record<string, EngineArg>, ids?: TextIds | null): Record<string, FluentVariable> {
  const out: Record<string, FluentVariable> = {};
  for (const [name, a] of Object.entries(args)) {
    if (typeof a === "string" || typeof a === "number") out[name] = a;
    else if ("num" in a) out[name] = decimalSign(a.num);
    else if ("cite" in a) out[name] = a.cite;
    else if ("input" in a) out[name] = inputLabel(a.input);
    else out[name] = say(a.en ?? a.id, ids, a);
  }
  return out;
}

/** The Display Language's decimal sign: a comma for Portuguese, a point for the others. */
function decimal(): string {
  return new Intl.NumberFormat(state.language.numbers).formatToParts(1.5).find((p) => p.type === "decimal")?.value ?? ".";
}

/** The engine's number text with the Display Language's decimal sign: the digits stay as computed. */
function decimalSign(text: string): string {
  const sign = decimal();
  return sign === "." ? text : text.replace(".", sign);
}

/** A stored number as an input box shows it: the same digits, with the Display Language's decimal sign. */
export function inputText(value: number): string {
  return decimalSign(String(value));
}

/**
 * A number as typed in an input box: the Display Language's decimal sign or a point. The stored value
 * is the same number in every language. Anything else is not a number (NaN).
 */
export function parseInput(text: string): number {
  const t = text.trim();
  if (t === "") return NaN;
  return Number(decimal() === "," ? t.replace(",", ".") : t);
}

/** A piece of a formatted message: plain text, or the place of one argument. */
export type Part = string | { arg: string };

/**
 * An app message cut at its arguments, for a template that draws an argument as markup (a link, a
 * bold name). Each argument named in `names` comes back as `{ arg }` in its place in the sentence,
 * so the language keeps its own word order.
 */
export function parts(id: string, names: string[], args: Record<string, FluentVariable> = {}): Part[] {
  const marked = { ...args, ...Object.fromEntries(names.map((n) => [n, `\u0001${n}\u0001`])) };
  return t(id, marked)
    .split("\u0001")
    .map((p, i) => (i % 2 === 1 ? { arg: p } : p))
    .filter((p) => p !== "");
}

/** Items joined as a list in the Display Language: "a, b and c" (`and`) or "a, b, c" (`unit`). */
export function list(items: string[], kind: "and" | "unit" = "and"): string {
  return listFormat(kind).format(items);
}

/** The same list, cut into items (by index) and the separators between them. */
export function listParts(items: string[]): Part[] {
  return listFormat("and")
    .formatToParts(items.map((_, i) => `\u0001${i}`))
    .map((p) => (p.type === "element" ? { arg: p.value.slice(1) } : p.value));
}

function listFormat(kind: "and" | "unit") {
  return new Intl.ListFormat(state.language.numbers, { type: kind === "and" ? "conjunction" : "unit" });
}

/** Chart tick numbers in the Display Language: as many decimals as the step between ticks needs. */
export function ticks(values: number[]): string[] {
  const step = values.length > 1 ? Math.abs(values[1] - values[0]) : 1;
  const digits = Math.min(6, (String(Number(step.toPrecision(6))).split(".")[1] ?? "").length);
  return values.map((v) => num(v, 0, digits));
}

/** A number for reading, in the Display Language's locale: `digits` decimals, with thousands separators. */
export function num(value: number, digits: number, maxDigits = digits): string {
  return value.toLocaleString(state.language.numbers, { minimumFractionDigits: digits, maximumFractionDigits: maxDigits });
}
