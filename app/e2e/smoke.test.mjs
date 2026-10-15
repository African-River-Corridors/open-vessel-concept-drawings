// End-to-end smoke tests (spec 0002, ticket 78): a few journeys on the built app, driven through
// Tauri's WebDriver (`tauri-driver`). Run by `e2e/run.sh`, which starts the driver and sets
// OVCD_TEST_DIALOG_DIR: a debug build answers its native dialogs from files in that folder, since
// WebDriver cannot click a native dialog (see `test_answer` in src-tauri/src/lib.rs).
import { after, before, beforeEach, describe, it } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { remote } from "webdriverio";

const here = import.meta.dirname;
// The engine version the app shows: read from the Tauri config, so a version bump needs no test edit.
const appVersion = JSON.parse(fs.readFileSync(path.join(here, "..", "src-tauri", "tauri.conf.json"), "utf8")).version;
const FIXTURES = path.resolve(here, "../../crates/ovcd-format/tests/fixtures");
const APP = path.resolve(here, process.env.OVCD_APP_BINARY ?? "../../target/debug/ovcd-app");
const DIALOGS = process.env.OVCD_TEST_DIALOG_DIR;
assert.ok(DIALOGS, "OVCD_TEST_DIALOG_DIR must be set (run e2e/run.sh)");
const CONFIG = process.env.OVCD_TEST_CONFIG_DIR;
assert.ok(CONFIG, "OVCD_TEST_CONFIG_DIR must be set (run e2e/run.sh)");
const WORK = fs.mkdtempSync(path.join(os.tmpdir(), "ovcd-e2e-work-"));

const answer = (file, text) => fs.writeFileSync(path.join(DIALOGS, file), text);
const forget = (file) => fs.rmSync(path.join(DIALOGS, file), { force: true });

let browser;

const until = (what, check, timeout = 15000) =>
  browser.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });

const fileLabel = async () => (await browser.$$("header .meta"))[0].getText();
const outputsText = async () => (await browser.$("[data-testid=outputs]")).getText();
const cargoOutput = async () => (await browser.$("[data-testid=outputs] tr:first-child td")).getText();
const field = (label) => browser.$(`input[aria-label="${label}"]`);

/** Click Open on a fixture (or any path) and wait for it to load. */
async function open(file, expectName) {
  answer("open.txt", file);
  await (await browser.$("button=Open")).click();
  await until(`${expectName} to open`, async () => (await fileLabel()).includes(expectName));
  await until("outputs", async () => (await browser.$("[data-testid=outputs]")).isExisting());
}

async function type(label, text) {
  const input = await field(label);
  await input.setValue(text);
}

describe("OVCD app smoke tests", () => {
  before(async () => {
    browser = await remote({
      hostname: "127.0.0.1",
      port: 4444,
      path: "/",
      logLevel: "warn",
      capabilities: { "wdio:enforceWebDriverClassic": true, "tauri:options": { application: APP } },
    });
    try {
      await until("the app to draw", async () => (await browser.$("header")).isExisting(), 60000);
    } catch (e) {
      // Say what the webview shows, so a CI failure is readable.
      const peek = async (what, f) => console.error(`e2e diagnostics: ${what}:`, await f().catch((x) => `(failed: ${x.message})`));
      await peek("handles", () => browser.getWindowHandles());
      await peek("url", () => browser.getUrl());
      await peek("title", () => browser.getTitle());
      await peek("state", () =>
        browser.execute(() => ({
          ready: document.readyState,
          tauri: typeof window.__TAURI_INTERNALS__,
          resources: performance.getEntriesByType("resource").map((r) => `${r.name} ${Math.round(r.duration)}ms ${r.transferSize}b`),
          app: document.getElementById("app")?.innerHTML.slice(0, 200),
          errors: window.__e2eErrors ?? null,
        })),
      );
      await peek("fresh import", () =>
        browser.executeAsync((done) => {
          const src = document.querySelector("script[type=module]")?.getAttribute("src");
          const seen = [];
          window.addEventListener("error", (e) => seen.push(String(e.message)));
          window.addEventListener("unhandledrejection", (e) => seen.push(String(e.reason)));
          import(`${src}?probe=${Date.now()}`).then(
            () => setTimeout(() => done({ result: "module ran", seen, app: document.getElementById("app")?.innerHTML.slice(0, 200) }), 1500),
            (err) => done({ result: `module failed: ${err} ${(err && err.stack) || ""}`, seen }),
          );
        }),
      );
      await peek("source", async () => (await browser.getPageSource()).slice(0, 600));
      throw e;
    }
  });

  after(async () => {
    await browser?.deleteSession();
  });

  beforeEach(() => {
    for (const f of ["open.txt", "save.txt", "profile.txt", "discard.txt"]) forget(f);
  });

  it("shows the first-run notice on first start only, and again from About", async () => {
    const notice = await browser.$("[data-testid=notice]");
    await until("the notice", async () => notice.isExisting());
    const text = await notice.getText();
    assert.match(text, /concept design tool/);
    assert.match(text, /not verified/);
    assert.match(text, /no warranty/i);
    assert.equal(fs.readdirSync(CONFIG).length, 0, "nothing is remembered before the click");
    await (await browser.$("button=I understand")).click();
    await until("the notice to close", async () => !(await browser.$("[data-testid=notice]").isExisting()));
    assert.equal(fs.readdirSync(CONFIG).length, 1, "the click is remembered in the config folder");
    // Help: the notice is reachable again.
    await (await browser.$("button=About")).click();
    await until("the notice from About", async () => (await browser.$("[data-testid=notice]")).isExisting());
    await (await browser.$("button=I understand")).click();
    await until("the notice to close", async () => !(await browser.$("[data-testid=notice]").isExisting()));
  });

  it("opens an example file and shows its outputs", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    assert.match(await cargoOutput(), /^2,038\.5/);
    assert.match(await outputsText(), /Displacement\s*2,372\.1/);
    assert.equal(await (await browser.$("[data-testid=engine-version]")).getText(), `Engine ${appVersion}`);
  });

  /** The text of the SVG drawn in each hull view slot. */
  const hullTexts = () =>
    browser.execute(() =>
      Object.fromEntries(
        ["side", "plan", "midship"].map((k) => [k, document.querySelector(`[data-testid=slot-${k}] svg`)?.textContent ?? null]),
      ),
    );
  const viewsDrawn = async () => (await hullTexts()).midship !== null && (await hullTexts()).plan !== null && (await hullTexts()).side !== null;

  it("draws the side elevation, plan and midship section, dimensioned from the results", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    await until("the three views to be drawn", viewsDrawn);
    const t = await hullTexts();
    // LOA 76.5 m, beam 11.4 m, depth 3.5 m: the dimension texts are computed from the geometry in Rust.
    assert.match(t.side, /76\.50/);
    assert.match(t.plan, /76\.50/);
    assert.match(t.plan, /11\.40/);
    assert.match(t.midship, /11\.40/);
    assert.match(t.midship, /3\.50/);
    assert.match(t.side, /SIDE ELEVATION/);
    // The front end only places the drawing: one SVG in each slot, and the zoom buttons.
    for (const slot of ["side", "plan", "midship"]) {
      assert.equal((await browser.$$(`[data-testid=slot-${slot}] svg`)).length, 1);
    }
  });

  it("redraws the views when an input changes, and draws tanks for a gas Cargo", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    await until("the three views to be drawn", viewsDrawn);
    await type("Beam", "12");
    await until("the plan to show the new beam", async () => /12\.00/.test((await hullTexts()).plan ?? ""));
    await open(path.join(FIXTURES, "gas_igc_2g_self_propelled.vdesign"), "gas_igc_2g_self_propelled.vdesign");
    await until("the gas views", async () => /LPG|Gas barge/.test((await hullTexts()).midship ?? ""));
    const circles = await browser.execute(() => document.querySelectorAll("[data-testid=slot-midship] svg circle[data-layer=OUTLINE]").length);
    assert.equal(circles, 1, "one tank across");
  });

  it("changes an output when an input is edited", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    const before = await cargoOutput();
    await type("Beam", "12");
    await until("the cargo to change", async () => (await cargoOutput()) !== before);
    assert.match(await fileLabel(), /unsaved/);
  });

  it("shows the error and marks the outputs out of date for a value outside the hard range", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    await type("Beam", "-1");
    await until("the error under the field", async () => (await browser.$$(".field .msg.error")).length > 0);
    assert.match(await browser.$(".field .msg.error").getText(), /^Beam/);
    assert.doesNotMatch(await browser.$(".field .msg.error").getText(), /`/);
    await until("the out-of-date mark", async () => (await browser.$$(".stale")).length > 0);
    assert.match(await browser.$(".stale").getText(), /out of date/);
    // The outputs header says why and where to look.
    assert.match(await browser.$('[data-testid="stale-why"]').getText(), /Not recalculated: fix the error under Beam/);
    // The last good numbers stay on screen.
    assert.match(await cargoOutput(), /^2,038\.5/);
  });

  it("switches Option from the picker and from the compare table", async () => {
    await open(path.join(FIXTURES, "two_options.vdesign"), "two_options.vdesign");
    const picker = await browser.$('select[aria-label="Active Option"]');
    assert.equal(await picker.getValue(), "Base");
    await picker.selectByVisibleText("Option 2");
    await until("Option 2 active", async () => (await picker.getValue()) === "Option 2");
    await (await browser.$("button.col=Base")).click();
    await until("Base active", async () => (await picker.getValue()) === "Base");
  });

  it("saves and re-opens the same Design", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    await type("Beam", "12");
    await until("the cargo to change", async () => /unsaved/.test(await fileLabel()));
    const saved = path.join(WORK, "round-trip.vdesign");
    answer("save.txt", saved);
    await (await browser.$("button=Save as")).click();
    await until("the file to be saved", async () => fs.existsSync(saved) && !/unsaved/.test(await fileLabel()));
    const first = fs.readFileSync(saved, "utf8");
    assert.equal(JSON.parse(first).options[0].inputs.beam_m.value, 12);

    // Start over, then open what was saved.
    await (await browser.$("button=New")).click();
    await until("a new Design", async () => (await fileLabel()).startsWith("Untitled"));
    await open(saved, "round-trip.vdesign");
    assert.equal(await (await field("Beam")).getValue(), "12");

    // Saving it again writes the same text.
    await (await browser.$("button=Save")).click();
    await until("the save", async () => !/unsaved/.test(await fileLabel()));
    assert.equal(fs.readFileSync(saved, "utf8"), first);
  });
});
