// The only door to the Text Library (spec 0005, ADR 0014). Every string the app shows comes through
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

export const LANGUAGES: Language[] = [{ tag: "en", name: "English", numbers: "en-GB" }];
const SOURCE = LANGUAGES[0];

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
  document.documentElement.lang = state.language.tag;
}

function format(b: FluentBundle | undefined, id: string, args?: Record<string, FluentVariable>): string | null {
  const m = b?.getMessage(id);
  return m?.value ? b!.formatPattern(m.value, args) : null;
}

/** An app message in the Display Language; English when the language lacks it; the id if English does. */
export function t(id: string, args?: Record<string, FluentVariable>): string {
  return format(BUNDLES.get(state.language.tag), id, args) ?? format(BUNDLES.get(SOURCE.tag), id, args) ?? id;
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

/** A number for reading, in the Display Language's locale: `digits` decimals, with thousands separators. */
export function num(value: number, digits: number, maxDigits = digits): string {
  return value.toLocaleString(state.language.numbers, { minimumFractionDigits: digits, maximumFractionDigits: maxDigits });
}
