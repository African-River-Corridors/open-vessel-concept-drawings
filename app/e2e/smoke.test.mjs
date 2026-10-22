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

/**
 * Click Open on a fixture (or any path) and wait for it to load. The file before may have the same
 * name and outputs, so the name and the outputs alone do not show that this open is done. An open
 * draws the form afresh (each field is keyed on the open), so mark the fields first and wait until
 * none of the marked ones is left (issue #470: a click on the old form was lost).
 */
async function open(file, expectName) {
  await browser.execute(() => document.querySelectorAll(".field[data-input]").forEach((e) => e.setAttribute("data-before-open", "")));
  answer("open.txt", file);
  await (await browser.$("button=Open")).click();
  await until(`${expectName} to open`, async () => (await fileLabel()).includes(expectName));
  await until("the form of the opened file", () =>
    browser.execute(() => document.querySelector(".field[data-input]") !== null && document.querySelector("[data-before-open]") === null),
  );
  await until("outputs", async () => (await browser.$("[data-testid=outputs]")).isExisting());
}

/** Untick "none" on a number input, and wait until its box is there to type in. */
async function untickNone(name, label) {
  await until(`the ${label} box`, async () => {
    if (await (await field(label)).isExisting()) return true;
    const none = await browser.$(`[data-input="${name}"] .none input`);
    if ((await none.isExisting()) && (await none.isSelected())) await none.click();
    return false;
  });
}

async function type(label, text) {
  // The panel can re-render a field between finding it and typing into it (a new snapshot replaces
  // the node): find it again, at most twice, on a stale element.
  for (let attempt = 0; ; attempt++) {
    try {
      const input = await field(label);
      await input.waitForExist({ timeout: 15000, timeoutMsg: `the ${label} input to be there` });
      await input.setValue(text);
      return;
    } catch (e) {
      if (attempt >= 2 || !/stale element/i.test(String(e))) throw e;
    }
  }
}

describe("OVCD app smoke tests", () => {
  before(async () => {
    browser = await remote({
      hostname: "127.0.0.1",
      port: 4444,
      path: "/",
      logLevel: "warn",
      // webdriverio 10 throws when a selector matches many elements; 9 took the first. The tests rely on that.
      strictSelectors: false,
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
    assert.match(await cargoOutput(), /^2,346\.2/);
    assert.match(await outputsText(), /Displacement\s*2,679\.8/);
    assert.equal(await (await browser.$("[data-testid=engine-version]")).getText(), `Engine ${appVersion}`);
  });

  const HULL_TABS = { side: "Side elevation", plan: "Plan", midship: "Cross-section" };
  // The views, by top tab. Every view is checked for scrolling and photographed.
  const GROUPS = [
    ["Drawings", ["Side elevation", "Plan", "Cross-section"]],
    ["Stability & Propulsion", ["Intact stability", "Ballast tanks", "Power and speed curves", "Rule check calculations"]],
    ["BOQ & Finance", ["BOQ", "Capital budget"]],
  ];
  const tab = (label) => browser.$(`//button[@role="tab" and normalize-space()="${label}"]`);
  /** The text of each element in a list, in order. */
  async function texts(list) {
    const out = [];
    for (let i = 0; i < list.length; i++) out.push(await list[i].getText());
    return out;
  }
  const slug = (label) => label.toLowerCase().replace(/[^a-z]+/g, "-").replace(/^-|-$/g, "");

  /** Show a view: its top tab, then its sub-tab. */
  async function show(group, view) {
    await (await tab(group)).click();
    await (await tab(view)).click();
    await until(`the ${view} tab to be selected`, async () => (await (await tab(view)).getAttribute("aria-selected")) === "true");
  }

  /** The text of the SVG drawn in a hull view (shows that view first). */
  async function hullText(key) {
    await show("Drawings", HULL_TABS[key]);
    await until(`the ${key} drawing`, async () => (await browser.$$(`[data-testid=slot-${key}] svg`)).length === 1);
    return browser.execute((k) => document.querySelector(`[data-testid=slot-${k}] svg`).textContent, key);
  }

  // Screenshots for review: when OVCD_E2E_SHOTS names a folder, save one per named point there.
  const SHOTS = process.env.OVCD_E2E_SHOTS;
  async function shot(name) {
    if (!SHOTS) return;
    fs.mkdirSync(SHOTS, { recursive: true });
    await browser.saveScreenshot(path.join(SHOTS, `${name}.png`));
  }

  /** Wait until the page can be photographed: fonts loaded, nothing mid-layout. */
  async function settle() {
    await browser.execute(() => {
      window.__fontsReady = false;
      document.fonts.ready.then(() => { window.__fontsReady = true; });
    });
    await until("the fonts to load", async () => browser.execute(() => window.__fontsReady === true));
    await browser.pause(400);
  }
  const setTheme = async (name) => {
    await (await browser.$(`button=${name}`)).click();
    const want = name === "Auto" ? null : name.toLowerCase();
    await until(`the ${name} theme`, async () => (await browser.execute(() => document.documentElement.dataset.theme ?? null)) === want);
  };

  /** How far the Views panel's content reaches past its box: more than 0 means it scrolls. */
  const overflow = () =>
    browser.execute(() => {
      const p = document.querySelector("[data-testid=views-panel]");
      return p.scrollHeight - p.clientHeight;
    });

  const toggle = (what, shown) => browser.$(`button[aria-label="${shown ? "Hide" : "Show"} the ${what} panel"]`);
  /** Hide or show a side panel through its toggle in the header. */
  async function setPanel(what, shown) {
    const button = await toggle(what, !shown);
    if (await button.isExisting()) await button.click();
    await until(`the ${what} panel to be ${shown ? "shown" : "hidden"}`, async () => (await toggle(what, shown).isExisting()));
  }
  const sideDisplayed = (cls) => browser.execute((c) => getComputedStyle(document.querySelector(`.body > section.${c}`)).display !== "none", cls);
  // The sheet is sized, not scaled (#297), so read the zoom state text: "scale(1)" stands for fitted.
  const sheetScale = () => browser.execute(() => (document.querySelector("[data-testid=zoom-state]")?.textContent === "fitted" ? "scale(1)" : "zoomed"));

  it("draws the side elevation, plan and midship section, dimensioned from the results", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    const t = { side: await hullText("side"), plan: await hullText("plan"), midship: await hullText("midship") };
    // LOA 76.5 m, beam 11.4 m, depth 3.5 m: the dimension texts are computed from the geometry in Rust.
    assert.match(t.side, /76\.50/);
    assert.match(t.plan, /76\.50/);
    assert.match(t.plan, /11\.40/);
    assert.match(t.midship, /11\.40/);
    assert.match(t.midship, /3\.50/);
    assert.match(t.side, /SIDE ELEVATION/);
  });

  it("redraws the views when an input changes, and draws tanks for a gas Cargo", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    await hullText("plan");
    await type("Beam", "12");
    await until("the plan to show the new beam", async () =>
      /12\.00/.test(await browser.execute(() => document.querySelector("[data-testid=slot-plan] svg")?.textContent ?? "")));
    await open(path.join(FIXTURES, "gas_igc_2g_self_propelled.vdesign"), "gas_igc_2g_self_propelled.vdesign");
    await show("Drawings", "Cross-section");
    await until("the gas views", async () =>
      /LPG|Gas barge/.test(await browser.execute(() => document.querySelector("[data-testid=slot-midship] svg")?.textContent ?? "")));
    const circles = await browser.execute(() => document.querySelectorAll("[data-testid=slot-midship] svg circle[data-layer=OUTLINE]").length);
    assert.equal(circles, 1, "one tank across");
  });

  it("shows two levels of tabs, with no jump list, and plain placeholders for later views", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    assert.equal((await browser.$$('select[aria-label="Jump to view"]')).length, 0, "the jump list is gone");
    const top = await browser.$$('[role=tablist][aria-label="View groups"] button');
    assert.deepEqual(await texts(top), GROUPS.map(([g]) => g));
    for (const [group, views] of GROUPS) {
      await (await tab(group)).click();
      const subs = await browser.$$(`[role=tablist][aria-label="Views of ${group}"] button`);
      assert.deepEqual(await texts(subs), views, `the views of ${group}`);
      // Picking a top tab selects its first view.
      assert.equal(await subs[0].getAttribute("aria-selected"), "true");
    }
    await show("Stability & Propulsion", "Rule check calculations");
    assert.ok(await browser.$("[data-testid=rule-checks]").isExisting());
    await show("BOQ & Finance", "BOQ");
    assert.ok(await browser.$("[data-testid=placeholder-boq]").isExisting());
    await show("BOQ & Finance", "Capital budget");
    assert.ok(await browser.$("[data-testid=capital-budget]").isExisting());
  });

  for (const [cargo, fixture] of [["gas", "gas_igc_2g_exact.vdesign"], ["dry-bulk", "dry_bulk_box_hold.vdesign"]]) {
    it(`no view scrolls, with the side panels shown or hidden (${cargo})`, async () => {
      await open(path.join(FIXTURES, fixture), fixture);
      try {
        for (const hidden of [false, true]) {
          await setPanel("Input", !hidden);
          await setPanel("Outputs", !hidden);
          for (const [group, views] of GROUPS) {
            for (const view of views) {
              await show(group, view);
              if (group === "Drawings") await until(`the ${view} drawing`, async () => (await browser.$$("[data-testid^=slot-] svg")).length === 1);
              await browser.pause(300);
              assert.ok((await overflow()) <= 0, `${view} scrolls (${cargo}, panels ${hidden ? "hidden" : "shown"})`);
              // The panels-shown views are in the 248 set; keep one panels-hidden shot per tab group.
              if (hidden && view === views[0]) await shot(`229-${cargo}-${slug(view)}-panels-hidden`);
            }
          }
        }
      } finally {
        await setPanel("Input", true);
        await setPanel("Outputs", true);
      }
    });
  }

  it("zooms a drawing, and Fit, a new view, a resized panel or a hidden panel return it to fitted", async () => {
    await open(path.join(FIXTURES, "gas_igc_2g_exact.vdesign"), "gas_igc_2g_exact.vdesign");
    const zoomIn = async () => {
      await (await browser.$('button[aria-label="Zoom in"]')).click();
      await until("the drawing to be zoomed", async () => !/scale\(1\)/.test(await sheetScale()));
    };
    const fitted = (why) => until(`fitted after ${why}`, async () => /scale\(1\)/.test(await sheetScale()));
    await hullText("plan");
    await fitted("opening the plan");
    await zoomIn();
    await shot("229-gas-plan-zoomed");
    await (await browser.$('button[aria-label="Fit to the panel"]')).click();
    await fitted("Fit");

    await zoomIn();
    await show("Drawings", "Cross-section");
    await show("Drawings", "Plan");
    await fitted("switching view");

    await zoomIn();
    await show("Stability & Propulsion", "Intact stability");
    await show("Drawings", "Plan");
    await fitted("switching top tab");

    try {
      await zoomIn();
      await setPanel("Outputs", false);
      await fitted("hiding the Outputs panel");
      await zoomIn();
      await setPanel("Input", false);
      await fitted("hiding the Input panel");
      await zoomIn();
      await setPanel("Input", true);
      await fitted("showing the Input panel");
    } finally {
      await setPanel("Input", true);
      await setPanel("Outputs", true);
    }

    await zoomIn();
    const size = await browser.getWindowSize();
    try {
      await browser.setWindowSize(size.width - 200, size.height - 150);
      await fitted("resizing the window");
    } finally {
      await browser.setWindowSize(size.width, size.height);
    }
  });

  it("hides and shows each side panel, and remembers the choice", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    const stored = () => browser.execute(() => localStorage.getItem("ovcd.panels"));
    // Both are shown by default.
    assert.ok(await toggle("Input", true).isExisting());
    assert.ok(await toggle("Outputs", true).isExisting());
    assert.equal(await sideDisplayed("left"), true);
    assert.equal(await sideDisplayed("bottom"), true);
    try {
      await setPanel("Input", false);
      assert.equal(await sideDisplayed("left"), false);
      assert.equal(await sideDisplayed("bottom"), true, "the other panel stays");
      await setPanel("Outputs", false);
      assert.equal(await sideDisplayed("bottom"), false);
      assert.deepEqual(JSON.parse(await stored()), { input: false, outputs: false });

      // A restart of the page reads the choice back.
      await browser.refresh();
      await until("the app to draw again", async () => (await browser.$("header")).isExisting());
      await until("the Input toggle to offer Show", async () => (await toggle("Input", false)).isExisting());
      assert.ok(await toggle("Outputs", false).isExisting());
      assert.equal(await sideDisplayed("left"), false);

      await setPanel("Input", true);
      await setPanel("Outputs", true);
      assert.deepEqual(JSON.parse(await stored()), { input: true, outputs: true });
    } finally {
      await setPanel("Input", true);
      await setPanel("Outputs", true);
    }
  });

  it("falls back to both panels shown when the stored choice is unreadable", async () => {
    await browser.execute(() => localStorage.setItem("ovcd.panels", "not json"));
    await browser.refresh();
    await until("the app to draw again", async () => (await browser.$("header")).isExisting());
    assert.ok(await toggle("Input", true).isExisting());
    assert.ok(await toggle("Outputs", true).isExisting());
  });

  // Power and speed curves: one chart and one readout per channel environment.
  const charts = () => browser.$$("[data-testid=speed-power-chart]");
  const readout = async (i) => {
    const el = (await browser.$$("[data-testid=speed-power-readout]"))[i];
    return { kn: Number(await el.getAttribute("data-kn")), design: (await el.getAttribute("data-design")) === "true", text: await el.getText() };
  };
  /** Click near the left or right end of a chart's plot area (the nearest computed point is chosen). */
  async function clickEnd(i, end) {
    const over = (await browser.$$("[data-testid=speed-power-chart] .u-over"))[i];
    const { width } = await over.getSize();
    await over.click({ x: Math.round(end === "left" ? 4 - width / 2 : width / 2 - 4), y: 0 });
  }
  const designSpeedSelected = (n) =>
    until("the design speed to be selected in every chart", async () => {
      for (let i = 0; i < n; i++) {
        const r = await readout(i);
        if (!(r.kn === 10 && r.design)) return false;
      }
      return true;
    });

  for (const [cargo, fixture] of [["gas", "gas_igc_2g_exact.vdesign"], ["dry-bulk", "dry_bulk_box_hold.vdesign"]]) {
    it(`selects the design speed in the power and speed curves, and a click selects another point (${cargo})`, async () => {
      await open(path.join(FIXTURES, fixture), fixture);
      await show("Stability & Propulsion", "Power and speed curves");
      await until("the chart", async () => (await charts()).length >= 1);
      assert.equal((await browser.$$("button=Reset zoom")).length, 0, "no zoom");
      await designSpeedSelected(1);
      const first = await readout(0);
      assert.match(first.text, /speed 10\.00 kn/);
      assert.match(first.text, /power [\d,]+ kW/);
      assert.match(first.text, /fuel [\d,.]+ l\/h/);
      assert.match(first.text, /squat [\d.]+ m/);
      assert.match(first.text, /squat-limited (yes|no)/);

      await clickEnd(0, "left");
      await until("the first point", async () => (await readout(0)).kn === 1);
      const low = await readout(0);
      assert.equal(low.design, false);
      assert.notEqual(low.text, first.text, "the readout shows the clicked point");
      await shot(`230-${cargo}-power-and-speed-curves-clicked`);

      // An input change selects the design speed again.
      await type("Beam", "12");
      await designSpeedSelected(1);
    });
  }

  it("shows one readout per channel environment, and the design speed follows the Maximum speed input", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    // These two inputs are "none" to start with: untick "none", then type the value.
    // 120 m wide, so the Schijf limit (about 12 kn, issue #397) lets the channel curve reach the 10 kn
    // design speed. It was 40 m, where the limit now stops the curve below it.
    for (const [name, label, value] of [["channel_depth_m", "Channel depth", "6"], ["channel_bottom_width_m", "Channel bottom width", "120"]]) {
      await untickNone(name, label);
      await type(label, value);
    }
    await show("Stability & Propulsion", "Power and speed curves");
    await until("two charts", async () => (await charts()).length === 2);
    await designSpeedSelected(2);
    // Clicking in one chart leaves the other alone.
    await clickEnd(0, "left");
    await until("the first chart's new point", async () => (await readout(0)).kn === 1);
    assert.equal((await readout(1)).kn, 10);
    assert.ok((await overflow()) <= 0, "two charts fit the panel");
    await shot("230-dry-bulk-power-and-speed-curves-two-environments");

    // A new Maximum speed is the new design speed (the curves now end there).
    await type("Maximum speed", "8");
    await until("the new design speed", async () => {
      const r = await readout(0);
      return r.kn === 8 && r.design;
    });
  });

  it("selects the design speed again after an Option switch", async () => {
    await open(path.join(FIXTURES, "two_options.vdesign"), "two_options.vdesign");
    await show("Stability & Propulsion", "Power and speed curves");
    await until("the chart", async () => (await charts()).length >= 1);
    await clickEnd(0, "left");
    await until("the clicked point", async () => (await readout(0)).kn === 1);
    const picker = await browser.$('select[aria-label="Active Option"]');
    await picker.selectByVisibleText("Option 2");
    await until("the design speed of Option 2", async () => {
      const r = await readout(0);
      return r.kn === 10 && r.design;
    });
    // Leave the file as it opened: the next test opens the same file name and expects Base active.
    await picker.selectByVisibleText("Base");
    await until("Base active again", async () => (await picker.getValue()) === "Base");
  });

  it("draws a GZ curve for each loading condition of a gas Design, with the governing one marked, and zooms with Fit", async () => {
    await open(path.join(FIXTURES, "gas_igc_2g_exact.vdesign"), "gas_igc_2g_exact.vdesign");
    await show("Stability & Propulsion", "Intact stability");
    await until("the GZ chart", async () => (await browser.$$("[data-testid=stability-chart] canvas")).length >= 1);
    // One legend entry per condition, with its GM and max GZ from the engine.
    const items = await browser.$$("[data-testid=stability-chart] .legend .item");
    assert.ok(items.length >= 2, "a curve per loading condition");
    for (let i = 0; i < items.length; i++) assert.match(await items[i].getText(), /GM -?[\d.]+ m · max GZ -?[\d.]+ m at \d+°/i);
    // The governing condition in the legend is the one the table marks.
    const governing = await browser.$("[data-testid=stability-chart] .item.governing-item");
    const row = await browser.$("tr[data-governing=true] th");
    const name = await governing.getAttribute("data-condition");
    assert.ok((await row.getText()).toLowerCase().includes(name.toLowerCase()), `the table marks ${name} as governing`);
    assert.equal((await browser.$$("[data-testid=stability-chart] .item.governing-item")).length, 1);
    assert.equal((await browser.$$("[data-testid=stability-wip]")).length, 0);

    const fitButton = await browser.$('button[aria-label="Fit to the panel"]');
    assert.equal(await fitButton.isEnabled(), false, "fitted to start with");
    await (await browser.$('button[aria-label="Zoom in"]')).click();
    await until("Fit to be offered", async () => await fitButton.isEnabled());
    await shot("231-gas-intact-stability-zoomed");
    await fitButton.click();
    await until("fitted again", async () => !(await fitButton.isEnabled()));
    // Another view and back starts fitted.
    await (await browser.$('button[aria-label="Zoom in"]')).click();
    await show("Stability & Propulsion", "Rule check calculations");
    await show("Stability & Propulsion", "Intact stability");
    await until("fitted after switching view", async () => !(await (await browser.$('button[aria-label="Fit to the panel"]')).isEnabled()));
  });

  it("says that stability is work in development for a dry-bulk Design", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    await show("Stability & Propulsion", "Intact stability");
    await until("the work-in-development state", async () => (await browser.$$("[data-testid=stability-wip]")).length === 1);
    assert.match(await browser.$("[data-testid=stability-wip]").getText(), /^Work in development\s+Stability is computed for gas cargo only\. Dry-bulk stability is work in development\.$/);
    assert.equal((await browser.$$("[data-testid=stability-chart]")).length, 0, "no empty chart");
  });

  // The Rule Trace (spec 0004, #276): one row per Rule per object, failures first. A minimum
  // freeboard just under the hull depth leaves no cargo, so an OVCD Check fails and each table has
  // a failing row.
  const RANK = { fail: 0, warn: 1, pass: 2, note: 3, "not evaluated": 4, "not applicable": 5 };
  for (const [cargo, fixture, freeboard] of [["gas", "gas_igc_2g_exact.vdesign", "4.3"], ["dry-bulk", "dry_bulk_box_hold.vdesign", "3.4"]]) {
    it(`shows the Rule Trace in the Rule check calculations table, failures first (${cargo})`, async () => {
      await open(path.join(FIXTURES, fixture), fixture);
      await type("Minimum freeboard (safety clearance)", freeboard);
      await show("Stability & Propulsion", "Rule check calculations");
      await until("a failing row", async () => (await browser.$$('[data-testid=rule-check-row][data-outcome="fail"]')).length > 0);
      assert.deepEqual(await texts(await browser.$$("[data-testid=rule-checks] thead th")),
        ["Result", "Rule", "Object", "Value", "Limit", "Margin", "Citation", "Status"]);
      const rows = await browser.execute(() =>
        [...document.querySelectorAll("[data-testid=rule-check-row]")].map((tr) => ({
          outcome: tr.dataset.outcome,
          governing: tr.dataset.governing === "true",
          gap: tr.querySelector("[data-testid=rule-cell-citation] .gap")?.textContent ?? "",
        })),
      );
      const ranks = rows.map((r) => RANK[r.outcome]);
      assert.ok(ranks.every((r) => r !== undefined), "every outcome has a tag");
      assert.deepEqual(ranks, [...ranks].sort((a, b) => a - b), "failures first, then warnings, passes, notes, not evaluated, not applicable");
      const notEvaluated = rows.filter((r) => r.outcome === "not evaluated");
      assert.ok(notEvaluated.length > 0, "a not evaluated row");
      assert.ok(notEvaluated.every((r) => r.gap.length > 0), "a not evaluated row says what the engine lacks");
      // Dry bulk evaluates no limit Rule yet (its stability is first-order), so only gas has one.
      if (cargo === "gas") assert.ok(rows.some((r) => r.governing), "a governing row");
      const failing = rows.filter((r) => r.outcome === "fail").length;
      assert.match(await browser.$("[data-testid=rule-check-count]").getText(), new RegExp(`^${failing} of ${rows.length} rule checks? fails?$`));
      assert.match(await browser.$("[data-testid=rule-trace-summary]").getText(), /fails \d+ blocking Rule check/);
      assert.ok((await overflow()) <= 0, "the panel does not scroll");
      for (const theme of ["Light", "Dark"]) {
        await setTheme(theme);
        await settle();
        await shot(`234-${cargo}-rule-trace-${theme.toLowerCase()}`);
      }
      await setTheme("Auto");
    });
  }

  const unitTotal = async () => Number((await browser.$("[data-testid=cost-unit-total]").getText()).replace(/,/g, ""));

  it("shows the capital budget of a gas Design with its basis, labels and where the masses come from", async () => {
    await open(path.join(FIXTURES, "gas_igc_2g_exact.vdesign"), "gas_igc_2g_exact.vdesign");
    await show("BOQ & Finance", "Capital budget");
    await until("the budget", async () => (await browser.$$("[data-testid=cost-unit-total]")).length === 1);
    assert.equal((await browser.$$("[data-testid=capital-budget-wip]")).length, 0);
    assert.match(await browser.$("[data-testid=capital-budget-basis]").getText(), /^Example rates . replace with yours\. Current estimates\. All amounts are in [A-Z]{3}\.$/);
    for (const table of ["barge", "tug", "total"]) assert.ok(await browser.$(`[data-testid=cost-${table}]`).isExisting(), table);
    // Plain names, as the input labels have them.
    const barge = await browser.$("[data-testid=cost-barge]").getText();
    for (const label of ["Type C tanks", "Piping and machinery", "Paint, anodes and miscellaneous"]) assert.ok(barge.includes(label), label);
    assert.ok((await browser.$("[data-testid=cost-tug]").getText()).includes("Wheelhouse and accommodation"));
    assert.ok((await unitTotal()) > 0);
    const masses = await browser.$("[data-testid=cost-masses]").getText();
    assert.match(masses, /^Mass from this design: .*Tank shell/);
    assert.match(masses, /Core defaults, not from this design: .*Battery/);
    assert.ok((await overflow()) <= 0, "the panel does not scroll");

    // Editing a cost rate changes the total.
    const before = await unitTotal();
    await type("Hull steel rate", "5000");
    await until("the total to change", async () => (await unitTotal()) !== before);
    // Put the rate back: the next test opens the same file name and must not meet this edit.
    await type("Hull steel rate", "2500");
    await until("the total to return", async () => (await unitTotal()) === before);
  });

  it("says that the capital budget is work in development for a dry-bulk Design", async () => {
    await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
    await show("BOQ & Finance", "Capital budget");
    await until("the work-in-development state", async () => (await browser.$$("[data-testid=capital-budget-wip]")).length === 1);
    assert.match(
      await browser.$("[data-testid=capital-budget-wip]").getText(),
      /^Work in development\s+The capital budget is computed for gas cargo and crude oil only\. A dry-bulk budget is work in development\.$/,
    );
    assert.equal((await browser.$$("[data-testid=cost-unit-total]")).length, 0, "no budget table");
  });

  it("shows the capital budget of a crude oil Design with its cargo tank items, and tags the Cargo as work in development", async () => {
    await open(path.join(FIXTURES, "crude_integral_tanks_pushed.vdesign"), "crude_integral_tanks_pushed.vdesign");
    await show("BOQ & Finance", "Capital budget");
    await until("the budget", async () => (await browser.$$("[data-testid=cost-unit-total]")).length === 1);
    assert.equal((await browser.$$("[data-testid=capital-budget-wip]")).length, 0);
    const barge = await browser.$("[data-testid=cost-barge]").getText();
    for (const row of ["Hull steel", "Tank coating", "Cargo pumps", "Cargo piping", "Cargo heating", "Vents and gauging"]) assert.ok(barge.includes(row), row);
    // The Cargo picker tags a Cargo that is still in development.
    const options = await browser.$$('select[aria-label="Cargo"] option');
    const labels = await texts(options);
    assert.ok(labels.includes("Crude oil (work in development)"), labels.join(" | "));
    assert.ok(labels.every((l) => l === "Crude oil (work in development)" || !l.includes("work in development")), "only crude oil is tagged");
  });

  it("lists the ballast tanks and the ballast plan of a Design in the IS Code conditions (issue #426)", async () => {
    await open(path.join(FIXTURES, "crude_integral_tanks_coastal.vdesign"), "crude_integral_tanks_coastal.vdesign");
    await show("Stability & Propulsion", "Ballast tanks");
    await until("the ballast tanks", async () => (await browser.$$("[data-testid=ballast-tanks] tbody tr")).length === 6);
    assert.match(await browser.$("[data-testid=ballast-summary]").getText(), /^6 ballast tanks\s+Capacity /);
    const headings = await texts(await browser.$$("[data-testid=ballast-condition]"));
    assert.ok(headings.some((h) => h.startsWith("Ballast, departure: ") && h.endsWith(" t of ballast")), headings.join(" | "));
    assert.equal((await browser.$$("[data-testid=ballast-none]")).length, 0);
  });

  it("says there are no ballast tanks to show for a Design saved before the IS Code conditions (issue #426)", async () => {
    await open(path.join(FIXTURES, "gas_igc_2g_exact.vdesign"), "gas_igc_2g_exact.vdesign");
    await show("Stability & Propulsion", "Ballast tanks");
    await until("the note", async () => (await browser.$$("[data-testid=ballast-none]")).length === 1);
    assert.equal((await browser.$$("[data-testid=ballast-tanks]")).length, 0);
  });

  for (const [cargo, fixture] of [["gas", "gas_igc_2g_exact.vdesign"], ["dry-bulk", "dry_bulk_box_hold.vdesign"]]) {
    it(`says the bill of quantities comes in a later phase (${cargo})`, async () => {
      await open(path.join(FIXTURES, fixture), fixture);
      await show("BOQ & Finance", "BOQ");
      const text = await browser.$("[data-testid=placeholder-boq]").getText();
      assert.equal(text, "The bill of quantities comes in a later phase. See the roadmap.");
      assert.equal(await browser.$("[data-testid=view-title]").getText(), "BOQ", "the view title is over the placeholder");
    });
  }

  it("has a Cost rates group with its basis, 17 inputs and the example values", async () => {
    await open(path.join(FIXTURES, "gas_igc_2g_exact.vdesign"), "gas_igc_2g_exact.vdesign");
    const group = await browser.$("//details[summary[normalize-space()='Cost rates']]");
    assert.ok(await group.isExisting(), "the group is in the left panel");
    assert.match(await group.$("[data-testid=group-note]").getText(), /^Example rates . replace with yours\.$/);
    assert.equal((await group.$$(".field")).length, 17);
    await until("the example rate", async () => (await (await field("Hull steel rate")).getValue()) === "2500");
    assert.equal(await (await field("Yard margin")).getValue(), "0.1");
    await browser.execute(() => document.querySelector("[data-testid=group-note]").scrollIntoView({ block: "start" }));
    await browser.pause(300);
    await shot("233-gas-cost-rates-inputs");
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
    await show("Drawings", "Plan");
    await type("Beam", "-1");
    await until("the error under the field", async () => (await browser.$$(".field .msg.error")).length > 0);
    assert.match(await browser.$(".field .msg.error").getText(), /^Beam/);
    assert.doesNotMatch(await browser.$(".field .msg.error").getText(), /`/);
    await until("the out-of-date mark", async () => (await browser.$$(".stale")).length > 0);
    assert.match(await browser.$(".stale").getText(), /out of date/);
    // The outputs header says why and where to look.
    assert.match(await browser.$('[data-testid="stale-why"]').getText(), /Not recalculated: fix the error under Beam/);
    // The last good numbers stay on screen.
    assert.match(await cargoOutput(), /^2,346\.2/);
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

  // The screenshot set of spec 248: every view of every tab, gas and dry bulk, in light and in dark,
  // plus the first-run notice and one error state. The theme is set with the app-bar control.
  for (const theme of ["Light", "Dark"]) {
    const t = theme.toLowerCase();
    for (const [cargo, fixture] of [["gas", "gas_igc_2g_exact.vdesign"], ["dry-bulk", "dry_bulk_box_hold.vdesign"]]) {
      it(`takes the ${t} screenshot set of every view (${cargo})`, async () => {
        await setTheme(theme);
        try {
          await open(path.join(FIXTURES, fixture), fixture);
          for (const [group, views] of GROUPS) {
            for (const view of views) {
              await show(group, view);
              if (group === "Drawings") await until(`the ${view} drawing`, async () => (await browser.$$("[data-testid^=slot-] svg")).length === 1);
              if (view === "Power and speed curves") await until("the chart", async () => (await charts()).length >= 1);
              if (view === "Intact stability" && cargo === "gas") await until("the GZ chart", async () => (await browser.$$("[data-testid=stability-chart] canvas")).length >= 1);
              await settle();
              await shot(`248-${t}-${cargo}-${slug(view)}`);
            }
          }
        } finally {
          await setTheme("Auto");
        }
      });
    }

    it(`takes the ${t} screenshots of the first-run notice and of an error`, async () => {
      await setTheme(theme);
      try {
        await (await browser.$("button=About")).click();
        await until("the notice", async () => (await browser.$("[data-testid=notice]")).isExisting());
        await settle();
        await shot(`248-${t}-notice`);
        await (await browser.$("button=I understand")).click();
        await until("the notice to close", async () => !(await browser.$("[data-testid=notice]").isExisting()));

        await open(path.join(FIXTURES, "dry_bulk_box_hold.vdesign"), "dry_bulk_box_hold.vdesign");
        await show("Drawings", "Plan");
        await type("Beam", "-1");
        await until("the error under the field", async () => (await browser.$$(".field .msg.error")).length > 0);
        await until("the out-of-date mark", async () => (await browser.$$(".stale")).length > 0);
        await settle();
        await shot(`248-${t}-error`);
      } finally {
        await setTheme("Auto");
      }
    });
  }

  it("switches the theme from the app bar: Dark sets data-theme, Auto removes it", async () => {
    const theme = () => browser.execute(() => document.documentElement.dataset.theme ?? null);
    await (await browser.$("button=Dark")).click();
    await until("the dark theme", async () => (await theme()) === "dark");
    await (await browser.$("button=Auto")).click();
    await until("the theme attribute to go", async () => (await theme()) === null);
  });

  it("switches the language from the app bar, keeps the choice on reload, and back to English", async () => {
    const lang = () => browser.execute(() => document.documentElement.lang);
    await (await browser.$("[data-testid=language]")).selectByAttribute("value", "es");
    await until("the header in Spanish", async () => (await browser.$("button=Abrir")).isExisting());
    assert.equal(await lang(), "es");
    await browser.refresh();
    await until("Spanish after a reload", async () => (await browser.$("button=Abrir")).isExisting());
    assert.equal(await lang(), "es");
    await (await browser.$("[data-testid=language]")).selectByAttribute("value", "en");
    await until("the header in English", async () => (await browser.$("button=Open")).isExisting());
    assert.equal(await lang(), "en");
  });
});
