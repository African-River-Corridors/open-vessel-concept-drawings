// Unit tests for src/lib/chartTheme.ts, with fakes for the computed style, the observer and the media
// query. Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import { readTokens, dashFor, DASHES, watchTheme, placeLabels } from "../src/lib/chartTheme.ts";

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

// Issue #667: the GZ curve names at the right edge of the chart.
const apart = (ys, step) => {
  const placed = ys.filter((y) => y !== null).sort((a, b) => a - b);
  return placed.every((y, i) => i === 0 || y - placed[i - 1] >= step - 1e-9);
};

test("placeLabels: names far apart stay where they want to be", () => {
  assert.deepEqual(placeLabels([40, 80, 120], 10, 200, 13), [40, 80, 120]);
});

test("placeLabels: four curves that end at the same GZ near the top get four readable names", () => {
  // The demo of issue #667: four names wanting one baseline just under the top edge of a short plot.
  const ys = [15, 15.5, 14, 16];
  const at = placeLabels(ys, 12, 110, 13);
  assert.ok(at.every((y) => y !== null), "all four fit");
  assert.ok(apart(at, 13), `at least one step apart: ${at}`);
  assert.ok(at.every((y) => y >= 12 && y <= 110), "inside the plot");
});

test("placeLabels: names that meet at the bottom edge move up, and keep their order down the chart", () => {
  const ys = [100, 105, 108];
  const at = placeLabels(ys, 0, 110, 13);
  assert.ok(apart(at, 13));
  assert.ok(at.every((y) => y <= 110));
  assert.ok(at[0] < at[1] && at[1] < at[2], "order kept");
});

test("placeLabels: a plot too short for every name keeps the first ones and drops the rest", () => {
  const at = placeLabels([20, 21, 22, 23], 10, 30, 13);
  assert.equal(at.filter((y) => y !== null).length, 2, "room for two names in 20 px at 13 px a name");
  assert.notEqual(at[0], null);
  assert.notEqual(at[1], null);
  assert.ok(apart(at, 13));
  assert.deepEqual(placeLabels([20], 30, 10, 13), [null], "no room at all");
});
