// Unit tests for src/lib/theme.ts, with a fake local storage and a fake <html>. Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import { THEME_KEY, loadTheme, saveTheme, applyTheme, initTheme } from "../src/lib/theme.ts";

function fakeStorage(initial = {}) {
  const data = { ...initial };
  return {
    data,
    getItem: (k) => (k in data ? data[k] : null),
    setItem: (k, v) => { data[k] = String(v); },
  };
}
function fakeRoot() {
  const attrs = {};
  return {
    attrs,
    setAttribute: (k, v) => { attrs[k] = v; },
    removeAttribute: (k) => { delete attrs[k]; },
  };
}
const unavailable = () => ({
  getItem() { throw new Error("blocked"); },
  setItem() { throw new Error("blocked"); },
});

test("load: nothing saved is dark", () => {
  globalThis.localStorage = fakeStorage();
  assert.equal(loadTheme(), "dark");
});

test("load: a saved light or dark choice comes back", () => {
  for (const c of ["light", "dark", "auto"]) {
    globalThis.localStorage = fakeStorage({ [THEME_KEY]: c });
    assert.equal(loadTheme(), c);
  }
});

test("load: junk in storage is dark", () => {
  globalThis.localStorage = fakeStorage({ [THEME_KEY]: "purple" });
  assert.equal(loadTheme(), "dark");
});

test("save: writes the choice under ovd.theme", () => {
  globalThis.localStorage = fakeStorage();
  saveTheme("dark");
  assert.equal(globalThis.localStorage.data["ovd.theme"], "dark");
  assert.equal(loadTheme(), "dark");
});

test("storage unavailable: load gives dark, save does not throw", () => {
  globalThis.localStorage = unavailable();
  assert.equal(loadTheme(), "dark");
  assert.doesNotThrow(() => saveTheme("light"));
});

test("apply: light and dark set data-theme, auto removes it", () => {
  const root = fakeRoot();
  applyTheme("dark", root);
  assert.equal(root.attrs["data-theme"], "dark");
  applyTheme("light", root);
  assert.equal(root.attrs["data-theme"], "light");
  applyTheme("auto", root);
  assert.equal("data-theme" in root.attrs, false);
});

test("init: applies the saved choice to <html>", () => {
  const root = fakeRoot();
  globalThis.document = { documentElement: root };
  globalThis.localStorage = fakeStorage({ [THEME_KEY]: "dark" });
  assert.equal(initTheme(), "dark");
  assert.equal(root.attrs["data-theme"], "dark");
  delete globalThis.document;
});
