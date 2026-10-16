// Unit tests for src/lib/chartTheme.ts, with fakes for the computed style, the observer and the media
// query. Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import { readTokens, dashFor, DASHES, watchTheme } from "../src/lib/chartTheme.ts";

test("readTokens reads each token by its name and trims it", () => {
  const style = { getPropertyValue: (n) => ({ "--ink": " #123456 ", "--surface": "#fff", "--font-ui": ' "Barlow Semi Condensed", sans-serif' })[n] ?? "" };
  const t = readTokens(style);
  assert.equal(t.ink, "#123456");
  assert.equal(t.surface, "#fff");
  assert.equal(t.fontUi, '"Barlow Semi Condensed", sans-serif');
  assert.equal(t.accent, "", "a token that is not set is empty");
});

test("readTokens follows a change of the style", () => {
  let ink = "#111";
  const style = { getPropertyValue: (n) => (n === "--ink" ? ink : "") };
  assert.equal(readTokens(style).ink, "#111");
  ink = "#eee";
  assert.equal(readTokens(style).ink, "#eee");
});

test("the first dash pattern is solid and the others differ", () => {
  assert.deepEqual(dashFor(0), []);
  const keys = DASHES.map((d) => d.join(","));
  assert.equal(new Set(keys).size, keys.length);
  assert.deepEqual(dashFor(DASHES.length), dashFor(0), "the patterns repeat");
});

test("watchTheme calls back for a data-theme change and for a colour-scheme change, and stops", () => {
  let calls = 0;
  let observed;
  let disconnected = false;
  const listeners = new Set();
  const observer = { observe: (root, opts) => { observed = opts; }, disconnect: () => { disconnected = true; } };
  const media = { addEventListener: (_t, fn) => listeners.add(fn), removeEventListener: (_t, fn) => listeners.delete(fn) };
  const stop = watchTheme(() => { calls++; }, {}, () => observer, media);
  assert.deepEqual(observed, { attributes: true, attributeFilter: ["data-theme"] });
  for (const fn of listeners) fn();
  assert.equal(calls, 1);
  stop();
  assert.equal(disconnected, true);
  assert.equal(listeners.size, 0);
});
