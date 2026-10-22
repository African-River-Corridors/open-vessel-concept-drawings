// Web smoke test (issue #246): the built web app, in headless Chrome, served like the live site.
// Run by e2e/run-web.sh. Checks: the first-run notice, the starter Design evaluates, the gas example
// opens through the file picker and its results hash equals `ovcd run --json`, and a browser with
// storage blocked still loads.
import { after, before, describe, it } from "node:test";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { remote } from "webdriverio";
import { serve } from "./web-server.mjs";

const here = import.meta.dirname;
const FIXTURE = path.resolve(here, "../../crates/ovcd-format/tests/fixtures/gas_igc_2g_self_propelled.vdesign");
const BALLAST_FIXTURE = path.resolve(here, "../../crates/ovcd-format/tests/fixtures/crude_integral_tanks_coastal.vdesign");
const CONVOY_FIXTURE = path.resolve(here, "../../crates/ovcd-format/tests/fixtures/pushed_convoy.vdesign");
const DRY_FIXTURE = path.resolve(here, "../../crates/ovcd-format/tests/fixtures/dry_bulk_box_hold.vdesign");
const DIST = path.resolve(here, "../dist-web-test");
const CLI = process.env.OVCD_CLI;
assert.ok(CLI, "OVCD_CLI must be set (run e2e/run-web.sh)");
const SHOTS = process.env.OVCD_WEB_SHOTS;

const nativeHashes = () =>
  JSON.parse(execFileSync(CLI, ["run", "--json", FIXTURE], { encoding: "utf8", maxBuffer: 1 << 28 })).options.map(
    (o) => [o.name, o.results_hash],
  );

const nativeMeshHashes = (file) =>
  execFileSync(CLI, ["mesh", "--hash", file], { encoding: "utf8" })
    .split("\n")
    .filter((l) => l.length > 0)
    .map((l) => l.split("\t").slice(0, 2));

const chrome = (extra = {}) => ({
  browserName: "chrome",
  "wdio:enforceWebDriverClassic": true,
  "goog:chromeOptions": {
    ...(process.env.CHROME_BINARY ? { binary: process.env.CHROME_BINARY } : fs.existsSync("/Applications/Google Chrome.app") ? { binary: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" } : {}),
    args: ["--headless=new", "--no-sandbox", "--use-angle=swiftshader", "--enable-unsafe-swiftshader", "--ignore-gpu-blocklist", "--window-size=1500,950"],
    ...extra,
  },
});

let origin, server;
const sessions = [];
async function start(options = {}) {
  const b = await remote({ logLevel: "warn", strictSelectors: false, capabilities: chrome(options) });
  sessions.push(b);
  return b;
}
const until = (b, what, check, timeout = 30000) =>
  b.waitUntil(check, { timeout, interval: 100, timeoutMsg: `timed out waiting for: ${what}` });
const outputs = (b) => b.$("[data-testid=outputs]");

before(async () => {
  ({ server, origin } = await serve(DIST));
});
after(async () => {
  for (const b of sessions) await b.deleteSession().catch(() => {});
  server?.close();
});

describe("OVCD web smoke tests", () => {
  it("shows the notice, then the starter Design evaluated, with the web label and desktop link", async () => {
    const b = await start();
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
    await (await b.$("button=I understand")).click();
    await until(b, "the notice to close", async () => !(await (await b.$("[data-testid=notice]")).isExisting()));
    await until(b, "results", async () => (await outputs(b)).isExisting());
    assert.ok((await (await outputs(b)).getText()).length > 0, "outputs have text");
    // The style sheet sets the label in capitals.
    assert.equal((await (await b.$("[data-testid=web-label]")).getText()).toLowerCase(), "web version");
    assert.equal(await (await b.$("a=Download the desktop app")).getAttribute("href"), "/ovcd#install");
    // Reload: the notice was remembered.
    await b.url(`${origin}/ovcd/app`);
    await until(b, "results after reload", async () => (await outputs(b)).isExisting());
    assert.equal(await (await b.$("[data-testid=notice]")).isExisting(), false, "notice not shown again");
  });

  it("opens the gas example through the file picker; results hash equals ovcd run --json", async () => {
    const b = await start();
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
    await (await b.$("button=I understand")).click();
    await until(b, "results", async () => (await outputs(b)).isExisting());
    // Headless Chrome cancels a file chooser at once. Stop the app opening it, then fill the input in.
    await b.execute(() => {
      HTMLInputElement.prototype.click = function () {};
    });
    await (await b.$("button=Open")).click();
    const picker = await b.$("[data-testid=file-picker]");
    await picker.waitForExist({ timeout: 5000 });
    await picker.setValue(FIXTURE);
    await until(b, "the file name", async () => (await b.$("header")).getText().then((t) => t.includes("gas_igc_2g_self_propelled.vdesign")));
    await until(b, "results", async () => (await outputs(b)).isExisting());
    const ours = await b.execute(async () => {
      const ev = await window.__ovcdPlatform.evaluate();
      return ev.options.map((o) => [o.name, o.outcome.results_hash]);
    });
    assert.deepEqual(ours, nativeHashes());
    // Issue #457: the meshes reach the front end typed, with the mesh hash of `ovcd mesh --hash`.
    const meshes = await b.execute(async () => {
      const ev = await window.__ovcdPlatform.evaluate();
      return Promise.all(
        ev.options.map(async (o) => {
          const m = await window.__ovcdPlatform.meshes(o.name);
          return [o.name, m.hash, m.positions instanceof Float32Array && m.indices instanceof Uint32Array];
        }),
      );
    });
    assert.deepEqual(meshes, nativeMeshHashes(FIXTURE).map(([n, h]) => [n, h, true]));
    if (SHOTS) {
      fs.mkdirSync(SHOTS, { recursive: true });
      await new Promise((r) => setTimeout(r, 800));
      await b.saveScreenshot(path.join(SHOTS, "web-gas-self-propelled.png"));
    }
    // The Design is kept: a reload brings it back.
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the restored Design", async () => (await b.$("header")).getText().then((t) => t.includes("gas_igc_2g_self_propelled.vdesign")));
  });

  it("gives a 100 m barge's meshes, a section, a Space and the waterlines (issue #457)", async () => {
    const b = await start();
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
    await (await b.$("button=I understand")).click();
    await until(b, "results", async () => (await outputs(b)).isExisting());
    const r = await b.execute(async () => {
      const p = window.__ovcdPlatform;
      const option = (await p.snapshot()).active_option;
      await p.setInput(option, "length_overall_m", 100);
      const t0 = performance.now();
      const m = await p.meshes(option);
      const ms = performance.now() - t0;
      const space = m.groups.find((g) => g.layer === "space");
      const info = await p.spaceInfo(option, space.id);
      const section = await p.sectionAt(option, 50);
      const lines = await p.waterlines(option);
      return { ms, triangles: m.indices.length / 3, info, section, lines, groups: m.groups.length };
    });
    console.log(`web: a 100 m barge's meshes reached the front end in ${r.ms.toFixed(0)} ms (${r.triangles} triangles, ${r.groups} groups)`);
    assert.ok(r.triangles > 0 && r.info.volume_m3 > 0 && r.lines.length > 0);
    assert.ok(r.section.rings.length >= 1 && r.section.x_m === 50);
  });

  it("lists the ballast tanks and the ballast plan of a Design in the IS Code conditions (issue #426)", async () => {
    const b = await start();
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
    await (await b.$("button=I understand")).click();
    await until(b, "results", async () => (await outputs(b)).isExisting());
    await b.execute(() => {
      HTMLInputElement.prototype.click = function () {};
    });
    await (await b.$("button=Open")).click();
    const picker = await b.$("[data-testid=file-picker]");
    await picker.waitForExist({ timeout: 5000 });
    await picker.setValue(BALLAST_FIXTURE);
    await until(b, "the file name", async () => (await b.$("header")).getText().then((t) => t.includes("crude_integral_tanks_coastal.vdesign")));
    const tab = (label) => b.$(`//button[@role="tab" and normalize-space()="${label}"]`);
    await (await tab("Stability & Propulsion")).click();
    await (await tab("Ballast tanks")).click();
    await until(b, "the ballast tanks", async () => (await b.$$("[data-testid=ballast-tanks] tbody tr")).length === 6);
    assert.match(await (await b.$("[data-testid=ballast-summary]")).getText(), /^6 ballast tanks\s+Capacity /);
    const headings = [];
    for (const h of await b.$$("[data-testid=ballast-condition]")) headings.push(await h.getText());
    assert.ok(headings.some((h) => h.startsWith("Ballast, departure: ") && h.endsWith(" t of ballast")), headings.join(" | "));
    if (SHOTS) {
      fs.mkdirSync(SHOTS, { recursive: true });
      await new Promise((r) => setTimeout(r, 800));
      await b.saveScreenshot(path.join(SHOTS, "web-ballast-tanks.png"));
    }
  });

  it("shows the convoy totals of a pushed convoy, and no Convoy tab for any other Formation (issue #551)", async () => {
    const b = await start();
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
    await (await b.$("button=I understand")).click();
    await until(b, "results", async () => (await outputs(b)).isExisting());
    await b.execute(() => {
      HTMLInputElement.prototype.click = function () {};
    });
    const tab = (label) => b.$(`//button[@role="tab" and normalize-space()="${label}"]`);
    await (await tab("Drawings")).click();
    assert.equal(await (await tab("Convoy totals")).isExisting(), false, "the starter Design is not a convoy");
    await (await b.$("button=Open")).click();
    const picker = await b.$("[data-testid=file-picker]");
    await picker.waitForExist({ timeout: 5000 });
    await picker.setValue(CONVOY_FIXTURE);
    await until(b, "the file name", async () => (await b.$("header")).getText().then((t) => t.includes("pushed_convoy.vdesign")));
    await (await tab("Drawings")).click();
    await until(b, "the Convoy tab", async () => (await tab("Convoy totals")).isExisting());
    await (await tab("Convoy totals")).click();
    await until(b, "the totals", async () => (await b.$("[data-testid=convoy-table]")).isExisting());
    const convoy = await b.execute(async () => (await window.__ovcdPlatform.evaluate()).options[0].outcome.results.convoy);
    assert.match(await (await b.$("[data-testid=convoy-summary]")).getText(), new RegExp(`^${convoy.abreast} abreast, ${convoy.in_line} in line: ${convoy.barges} barges$`));
    const want = { cargo: ["cargo_t"], lightship: ["lightship_t"], deadweight: ["deadweight_t"], displacement: ["displacement_t"], volume: ["cargo_volume_m3"], length: ["overall_length_m"], beam: ["overall_beam_m"] };
    for (const [row, [key]] of Object.entries(want)) {
      const total = await (await b.$(`[data-row=${row}] [data-cell=total]`)).getAttribute("title");
      assert.equal(Number(total), convoy.totals[key], row);
    }
    for (const row of ["cargo", "lightship", "deadweight", "displacement", "volume"]) {
      const barge = await (await b.$(`[data-row=${row}] [data-cell=barge]`)).getAttribute("title");
      assert.equal(Number(barge), convoy.barge[want[row][0]], row);
    }
    assert.equal(await (await b.$("[data-testid=convoy-pusher-note]")).isExisting(), !convoy.pusher_included);
    if (SHOTS) {
      fs.mkdirSync(SHOTS, { recursive: true });
      await new Promise((r) => setTimeout(r, 500));
      await b.saveScreenshot(path.join(SHOTS, "web-convoy.png"));
    }
  });

  it("shows the 3D model: toggles every layer, picks a Space and reads its volume (issue #458)", async () => {
    const b = await start();
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
    await (await b.$("button=I understand")).click();
    await until(b, "results", async () => (await outputs(b)).isExisting());
    await b.execute(() => {
      HTMLInputElement.prototype.click = function () {};
    });
    await (await b.$("button=Open")).click();
    const picker = await b.$("[data-testid=file-picker]");
    await picker.waitForExist({ timeout: 5000 });
    await picker.setValue(BALLAST_FIXTURE);
    await until(b, "the file name", async () => (await b.$("header")).getText().then((t) => t.includes("crude_integral_tanks_coastal.vdesign")));
    const tab = (label) => b.$(`//button[@role="tab" and normalize-space()="${label}"]`);
    await (await tab("Drawings")).click();
    await (await tab("3D")).click();
    await until(b, "the model", async () => (await (await b.$("[data-testid=model3d]")).getAttribute("data-status")) !== "loading", 60000);
    const status = await (await b.$("[data-testid=model3d]")).getAttribute("data-status");
    assert.equal(status, "ready", "WebGL is available in the test browser");
    const shown = async () => JSON.parse(await (await b.$("[data-testid=model3d-canvas]")).getAttribute("data-shown"));
    const all = await shown();
    const layers = new Set(all.map((g) => g.split(":")[0]));
    for (const l of ["hull", "deck", "bulkhead", "space"]) assert.ok(layers.has(l), `a ${l} group is drawn: ${[...layers]}`);
    // Each layer toggle takes its groups off and puts them back.
    for (const [testid, layer] of [["layer-hull", "hull"], ["layer-deck", "deck"], ["layer-bulkhead", "bulkhead"]]) {
      await (await b.$(`[data-testid=${testid}]`)).click();
      await until(b, `${layer} off`, async () => !(await shown()).some((g) => g.startsWith(`${layer}:`)));
      await (await b.$(`[data-testid=${testid}]`)).click();
      await until(b, `${layer} on`, async () => (await shown()).some((g) => g.startsWith(`${layer}:`)));
    }
    const kindBoxes = await b.$$("[data-testid^=layer-kind-]");
    assert.ok(kindBoxes.length >= 3, "one toggle per kind of Space");
    await kindBoxes[0].click();
    await until(b, "a kind off", async () => (await shown()).filter((g) => g.startsWith("space:")).length < all.filter((g) => g.startsWith("space:")).length);
    await kindBoxes[0].click();
    assert.equal((await b.$("[data-testid=layer-structure]")).isEnabled !== undefined, true);
    assert.equal(await (await b.$("[data-testid=layer-structure]")).isEnabled(), false, "structure is disabled");
    // Pick a Space: its volume is the core's.
    const name = (await (await b.$("[data-testid=model3d-spaces] tbody th button")).getText()).trim();
    await (await b.$(`[data-testid="space-${name}"]`)).click();
    await until(b, "the Space info", async () => (await b.$("[data-testid=space-info]")).isExisting());
    const shownVolume = Number(await (await b.$("[data-testid=space-info-volume]")).getAttribute("data-value"));
    const core = await b.execute(async (n) => (await window.__ovcdPlatform.spaceInfo("Option 1", n)).volume_m3, name);
    assert.equal(shownVolume, core);
    if (SHOTS) {
      fs.mkdirSync(SHOTS, { recursive: true });
      for (const theme of ["Light", "Dark"]) {
        await (await b.$(`button=${theme}`)).click();
        await new Promise((r) => setTimeout(r, 800));
        await b.saveScreenshot(path.join(SHOTS, `web-3d-${theme.toLowerCase()}.png`));
      }
    }
  });

  it("gives the same results hashes and saved bytes in every language, and keeps the choice", async () => {
    const b = await start();
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
    await (await b.$("button=I understand")).click();
    await until(b, "results", async () => (await outputs(b)).isExisting());
    await b.execute(() => {
      HTMLInputElement.prototype.click = function () {};
      // Keep each download's bytes, so the saved file can be compared.
      window.__saved = [];
      URL.createObjectURL = (blob) => {
        window.__saved.push(blob);
        return "blob:kept";
      };
    });
    await (await b.$("button=Open")).click();
    const picker = await b.$("[data-testid=file-picker]");
    await picker.waitForExist({ timeout: 5000 });
    await picker.setValue(FIXTURE);
    await until(b, "the file name", async () => (await b.$("header")).getText().then((t) => t.includes("gas_igc_2g_self_propelled.vdesign")));
    const seen = {};
    for (const [tag, open] of [["es", "Abrir"], ["pt", "Abrir"], ["zh-Hans", "打开"], ["ak", "Bue"], ["en", "Open"]]) {
      await (await b.$("[data-testid=language]")).selectByAttribute("value", tag);
      await until(b, `the header in ${tag}`, async () => (await b.$(`button=${open}`)).isExisting());
      assert.equal(await b.execute(() => document.documentElement.lang), tag);
      seen[tag] = await b.execute(async () => {
        const ev = await window.__ovcdPlatform.evaluate();
        await window.__ovcdPlatform.save();
        const text = await window.__saved.at(-1).text();
        return { hashes: ev.options.map((o) => [o.name, o.outcome.results_hash]), text };
      });
    }
    for (const tag of ["es", "pt", "zh-Hans", "ak"]) {
      assert.deepEqual(seen[tag].hashes, seen.en.hashes, `results hashes in ${tag}`);
      assert.equal(seen[tag].text, seen.en.text, `saved file in ${tag}`);
    }
    assert.deepEqual(seen.en.hashes, nativeHashes());
    assert.ok(!seen.en.text.includes("language"), "the language is not in the file");
    // The choice is kept: after a reload the app is in the last language picked.
    await (await b.$("[data-testid=language]")).selectByAttribute("value", "pt");
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the app in Portuguese again", async () => (await b.$("button=Abrir")).isExisting());
    assert.equal(await b.execute(() => document.documentElement.lang), "pt");
    if (!SHOTS) return;
    // For review (spec 0005): each view in each language, light and dark, and any text that is wider
    // than its box (a tab, a table head, a button).
    const overflow = [];
    fs.mkdirSync(SHOTS, { recursive: true });
    const tabs = (sub) => b.$$(`[data-testid=views-panel] [role=tablist]${sub ? ".sub" : ":not(.sub)"} button`);
    for (const tag of ["en", "es", "pt", "zh-Hans", "ak"]) {
      await (await b.$("[data-testid=language]")).selectByAttribute("value", tag);
      for (const [i, theme] of [[1, "light"], [2, "dark"]]) {
        await (await b.$$(".theme button"))[i].click();
        for (let g = 0; g < (await tabs(false)).length; g++) {
          await (await tabs(false))[g].click();
          for (let v = 0; v < (await tabs(true)).length; v++) {
            await (await tabs(true))[v].click();
            await new Promise((r) => setTimeout(r, 600));
            const name = `311-${tag}-${theme}-${g}-${v}`;
            await b.saveScreenshot(path.join(SHOTS, `${name}.png`));
            const wide = await b.execute(() =>
              [...document.querySelectorAll("button, th, [role=tab], label, h2, h3, h4, td")]
                .filter((e) => e.offsetParent && e.scrollWidth > e.clientWidth + 1 && getComputedStyle(e).overflowX !== "auto")
                .map((e) => `${e.tagName}: ${e.textContent.trim().slice(0, 60)}`),
            );
            overflow.push(...wide.map((w) => `${name} | ${w}`));
          }
        }
      }
    }
    fs.writeFileSync(path.join(SHOTS, "311-overflow.txt"), `${[...new Set(overflow)].join("\n")}\n`);
  });

  it("shows every message in one line, steps through them, lists them all, and drops an Option's notes with it", async () => {
    const OLD = path.resolve(here, "../../crates/ovcd-format/tests/legacy/v1_adn_g_self_propelled.vdesign");
    const b = await start();
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
    await (await b.$("button=I understand")).click();
    await until(b, "results", async () => (await outputs(b)).isExisting());
    await b.execute(() => {
      HTMLInputElement.prototype.click = function () {};
    });
    await (await b.$("button=Open")).click();
    const picker = await b.$("[data-testid=file-picker]");
    await picker.waitForExist({ timeout: 5000 });
    await picker.setValue(OLD);
    const bar = () => b.$("[data-testid=message-bar]");
    await until(b, "the message bar", async () => (await bar()).isExisting());
    // Six notes of the changed old file (issue #436): one line, not six banners.
    const count = await (await b.$("[data-testid=message-count]")).getText();
    const total = Number(count.split(" of ")[1]);
    assert.ok(total >= 5, `at least five messages: ${count}`);
    assert.equal(count, `1 of ${total}`);
    const height = await b.execute(() => document.querySelector("[data-testid=message-bar]").getBoundingClientRect().height);
    assert.ok(height <= 32, `the bar is one line high (${height} px)`);
    const seen = new Set();
    for (let i = 0; i < total; i++) {
      seen.add(await (await b.$("[data-testid=message-text]")).getText());
      await (await b.$("button[aria-label='Next message']")).click();
    }
    assert.equal(seen.size, total, "next shows each message once");
    assert.equal(await (await b.$("[data-testid=message-count]")).getText(), `1 of ${total}`, "next wraps round");
    await (await b.$("button[aria-label='Previous message']")).click();
    assert.equal(await (await b.$("[data-testid=message-count]")).getText(), `${total} of ${total}`);
    await (await b.$("button=All")).click();
    assert.equal((await b.$$("[data-testid=message-list] li")).length, total, "the list shows them all");
    await (await b.$("button=Less")).click();
    assert.equal(await (await b.$("[data-testid=message-list]")).isExisting(), false);
    if (SHOTS) {
      // For review (issue #436): the bar and its open list, light and dark, desktop and phone width.
      fs.mkdirSync(SHOTS, { recursive: true });
      for (const [w, h, size] of [[1500, 950, "desktop"], [390, 844, "phone"]]) {
        await b.setWindowSize(w, h);
        for (const [i, theme] of [[1, "light"], [2, "dark"]]) {
          await (await b.$$(".theme button"))[i].click();
          await new Promise((r) => setTimeout(r, 300));
          await b.saveScreenshot(path.join(SHOTS, `436-${size}-${theme}-bar.png`));
          await (await b.$("button=All")).click();
          await b.saveScreenshot(path.join(SHOTS, `436-${size}-${theme}-list.png`));
          await (await b.$("button=Less")).click();
        }
      }
      await b.setWindowSize(1500, 950);
    }
    // Keep a copy, then delete the Option the notes are about: its notes go with it.
    await (await b.$("button=Duplicate")).click();
    await until(b, "two Options", async () => (await b.$$('select[aria-label="Active Option"] option')).length === 2);
    await (await b.$('select[aria-label="Active Option"]')).selectByIndex(0);
    await (await b.$("button=Delete")).click();
    await (await (await b.$(".confirm")).$("button=Delete")).click();
    await until(b, "one Option", async () => (await b.$$('select[aria-label="Active Option"] option')).length === 1);
    await until(b, "the notes to go", async () => !(await (await bar()).isExisting()));
  });

  it("shows a failed warning check as a warning, not a failure (issue #419)", async () => {
    // The dry bulk example is overloaded at its volume-bound load: `not overloaded` fails, and it is a
    // warning (issue #418), so it does not gate the result.
    const b = await start();
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
    await (await b.$("button=I understand")).click();
    await until(b, "results", async () => (await outputs(b)).isExisting());
    await b.execute(() => {
      HTMLInputElement.prototype.click = function () {};
    });
    await (await b.$("button=Open")).click();
    const picker = await b.$("[data-testid=file-picker]");
    await picker.waitForExist({ timeout: 5000 });
    await picker.setValue(DRY_FIXTURE);
    await until(b, "the file name", async () => (await b.$("header")).getText().then((t) => t.includes("dry_bulk_box_hold.vdesign")));
    await until(b, "the checks", async () => (await b.$$("[data-testid=checks] [data-testid=check]")).length > 0);
    const rows = await b.$$("[data-testid=checks] [data-testid=check]");
    const seen = [];
    for (const r of rows) seen.push([await r.getAttribute("data-tone"), await r.getText()]);
    const overloaded = seen.find(([, text]) => text.includes("not overloaded"));
    assert.ok(overloaded, `a "not overloaded" check: ${JSON.stringify(seen)}`);
    assert.equal(overloaded[0], "warn");
    assert.ok(overloaded[1].includes("WARN") && !overloaded[1].includes("FAIL"), overloaded[1]);
    // The warnings come after the failures that gate and before the passes.
    const tones = seen.map(([tone]) => tone);
    const order = { fail: 0, warn: 1, ok: 2 };
    assert.deepEqual([...tones].sort((x, y) => order[x] - order[y]), tones);
  });

  it("still loads when browser storage is blocked", async () => {
    // Blocking cookies and site data makes `localStorage` throw.
    const b = await start({ prefs: { "profile.default_content_setting_values.cookies": 2 } });
    await b.url(`${origin}/ovcd/app`);
    const blocked = await b.execute(() => {
      try {
        window.localStorage.getItem("x");
        return false;
      } catch {
        return true;
      }
    });
    assert.ok(blocked, "storage is blocked in this browser");
    // With no memory of the notice it shows on every load.
    await until(b, "the notice", async () => (await b.$("[data-testid=notice]")).isExisting());
    await (await b.$("button=I understand")).click();
    await until(b, "results", async () => (await outputs(b)).isExisting());
    assert.equal(await (await b.$("[data-testid=notice]")).isExisting(), false);
  });
});
