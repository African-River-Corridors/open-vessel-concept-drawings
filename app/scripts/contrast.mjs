// Contrast check for the app tokens (docs/design/style-guide.md, section 3).
// Reads the tokens from a CSS file (default src/app.css), checks every pair of the guide's table in the
// light and the dark theme (text 4.5:1, lines 3:1), prints a table, exits 1 on any failure.
// It also checks that the OS-dark block and the data-theme="dark" block hold the same values.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));

// [foreground, background, kind]
export const PAIRS = [
  ["ink", "paper", "text"], ["ink", "surface", "text"], ["ink", "surface-2", "text"],
  ["ink-2", "paper", "text"], ["ink-2", "surface", "text"], ["ink-2", "surface-2", "text"],
  ["ink-3", "paper", "text"], ["ink-3", "surface", "text"], ["ink-3", "surface-2", "text"],
  ["accent", "paper", "text"], ["accent", "surface", "text"], ["accent", "surface-2", "text"],
  ["accent-ink", "accent", "text"], ["ink", "accent-soft", "text"], ["accent", "accent-soft", "text"],
  ["pass", "pass-soft", "text"], ["fail", "fail-soft", "text"], ["warn", "warn-soft", "text"], ["ink", "warn-soft", "text"],
  ["pass", "surface", "text"], ["fail", "surface", "text"], ["warn", "surface", "text"],
  ["src-class", "surface", "text"], ["stale", "surface", "text"],
  ["sheet-ink", "sheet", "text"], ["sheet-ink-2", "sheet", "text"],
  ["gilt-ink", "paper", "text"], ["gilt-ink", "surface", "text"],
  ["white-ink", "blueprint", "text"], ["white-ink", "blueprint-deep", "text"], ["cream", "blueprint", "text"],
  ["rule-strong", "surface", "line"], ["rule-strong", "paper", "line"], ["accent", "paper", "line"],
  ["gilt-mark", "paper", "line"], ["gilt-mark", "surface", "line"],
];
const TARGET = { text: 4.5, line: 3.0 };

function declarations(body) {
  const out = {};
  for (const m of body.matchAll(/--([a-z0-9-]+)\s*:\s*([^;]+);/g)) out[m[1]] = m[2].trim();
  return out;
}

// Blocks whose selector matches `selector` exactly, in the file, merged in order.
function blocks(css, selector) {
  const stripped = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const out = {};
  const re = new RegExp("(^|[}\\s])" + selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + "\\s*\\{([^}]*)\\}", "g");
  for (const m of stripped.matchAll(re)) Object.assign(out, declarations(m[2]));
  return out;
}

export function parseTokens(css) {
  const light = blocks(css, ":root");
  const darkAttr = blocks(css, ':root[data-theme="dark"]');
  const darkMedia = blocks(css, ':root:not([data-theme="light"])');
  return { light, darkAttr, darkMedia };
}

function resolveValue(map, name, seen = new Set()) {
  let v = map[name];
  if (v === undefined || seen.has(name)) return undefined;
  seen.add(name);
  const ref = v.match(/^var\(--([a-z0-9-]+)\)$/);
  return ref ? resolveValue(map, ref[1], seen) : v;
}

function rgb(hex) {
  const h = hex.replace("#", "");
  if (!/^[0-9a-fA-F]{6}$/.test(h)) throw new Error(`not a #rrggbb colour: ${hex}`);
  return [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16));
}
function luminance(hex) {
  const [r, g, b] = rgb(hex).map((c) => {
    const s = c / 255;
    return s <= 0.03928 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}
export function ratio(a, b) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

export function check(css) {
  const { light, darkAttr, darkMedia } = parseTokens(css);
  const dark = { ...light, ...darkAttr };
  const rows = [];
  const problems = [];
  // The OS-dark block may be absent (Auto shows light while #248 is in progress); when present, it must match.
  const media = Object.keys(darkMedia).length > 0;
  for (const key of media ? new Set([...Object.keys(darkAttr), ...Object.keys(darkMedia)]) : []) {
    if (darkAttr[key] !== darkMedia[key]) {
      problems.push(`dark blocks differ on --${key}: data-theme="dark" has ${darkAttr[key]}, the OS-dark block has ${darkMedia[key]}`);
    }
  }
  for (const [fg, bg, kind] of PAIRS) {
    const row = { fg, bg, kind, target: TARGET[kind] };
    for (const [theme, map] of [["light", light], ["dark", dark]]) {
      const f = resolveValue(map, fg);
      const b = resolveValue(map, bg);
      if (f === undefined || b === undefined) {
        row[theme] = NaN;
        problems.push(`${theme}: token --${f === undefined ? fg : bg} is missing`);
        continue;
      }
      row[theme] = ratio(f, b);
      if (row[theme] < TARGET[kind]) {
        problems.push(`${theme}: ${fg} on ${bg} (${kind}) is ${row[theme].toFixed(2)}, needs ${TARGET[kind]}`);
      }
    }
    rows.push(row);
  }
  return { rows, problems };
}

export function format(rows) {
  const w = Math.max(...rows.map((r) => `${r.fg} on ${r.bg}`.length));
  const cell = (r, t) => (Number.isNaN(r[t]) ? "  n/a " : r[t].toFixed(2).padStart(6)) + (r[t] >= r.target ? "  ok  " : " FAIL ");
  const lines = [`${"pair".padEnd(w)}  kind   need   light        dark`];
  for (const r of rows) {
    lines.push(`${`${r.fg} on ${r.bg}`.padEnd(w)}  ${r.kind.padEnd(5)}  ${r.target.toFixed(1)}  ${cell(r, "light")} ${cell(r, "dark")}`);
  }
  return lines.join("\n");
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const path = process.argv[2] ? resolve(process.argv[2]) : resolve(here, "../src/app.css");
  const { rows, problems } = check(readFileSync(path, "utf8"));
  console.log(format(rows));
  if (problems.length) {
    console.error("\nContrast check failed:\n" + problems.map((p) => "  " + p).join("\n"));
    process.exit(1);
  }
  console.log(`\nContrast check passed: ${rows.length} pairs, light and dark.`);
}
