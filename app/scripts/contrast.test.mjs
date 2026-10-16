// Tests for scripts/contrast.mjs: the real app.css passes; a CSS file with a failing pair exits 1.
import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const script = join(here, "contrast.mjs");
const appCss = join(here, "../src/app.css");
const run = (...args) => spawnSync(process.execPath, [script, ...args], { encoding: "utf8" });

test("the real app.css passes and prints the pairs", () => {
  const r = run();
  assert.equal(r.status, 0, r.stdout + r.stderr);
  assert.match(r.stdout, /ink on paper/);
  assert.match(r.stdout, /gilt-mark on surface/);
});

test("a failing text pair in the light theme exits 1 and names the pair", () => {
  const dir = mkdtempSync(join(tmpdir(), "contrast-"));
  const css = readFileSync(appCss, "utf8").replace("--ink-3: #56687a;", "--ink-3: #b0b8c0;");
  assert.notEqual(css, readFileSync(appCss, "utf8"));
  const bad = join(dir, "bad.css");
  writeFileSync(bad, css);
  const r = run(bad);
  assert.equal(r.status, 1);
  assert.match(r.stderr, /light: ink-3 on paper/);
});

test("a failing line pair in the dark theme exits 1", () => {
  const dir = mkdtempSync(join(tmpdir(), "contrast-"));
  const css = readFileSync(appCss, "utf8").replaceAll("--rule-strong: #95b4c6;", "--rule-strong: #4b7192;");
  const bad = join(dir, "bad-dark.css");
  writeFileSync(bad, css);
  const r = run(bad);
  assert.equal(r.status, 1);
  assert.match(r.stderr, /dark: rule-strong on surface/);
});

test("dark blocks that disagree exit 1", () => {
  const dir = mkdtempSync(join(tmpdir(), "contrast-"));
  // An OS-dark block whose --pass differs from the data-theme="dark" block.
  const dark = readFileSync(appCss, "utf8").match(/:root\[data-theme="dark"\] \{([^}]*)\}/)[1];
  const css = readFileSync(appCss, "utf8") + `\n@media (prefers-color-scheme: dark) {\n  :root:not([data-theme="light"]) {${dark.replace("--pass: #b9efc3;", "--pass: #b9efc4;")}}\n}\n`;
  const bad = join(dir, "bad-sync.css");
  writeFileSync(bad, css);
  const r = run(bad);
  assert.equal(r.status, 1);
  assert.match(r.stderr, /dark blocks differ on --pass/);
});
