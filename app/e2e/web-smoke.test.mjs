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
const DIST = path.resolve(here, "../dist-web-test");
const CLI = process.env.OVCD_CLI;
assert.ok(CLI, "OVCD_CLI must be set (run e2e/run-web.sh)");
const SHOTS = process.env.OVCD_WEB_SHOTS;

const nativeHashes = () =>
  JSON.parse(execFileSync(CLI, ["run", "--json", FIXTURE], { encoding: "utf8", maxBuffer: 1 << 28 })).options.map(
    (o) => [o.name, o.results_hash],
  );

const chrome = (extra = {}) => ({
  browserName: "chrome",
  "wdio:enforceWebDriverClassic": true,
  "goog:chromeOptions": {
    ...(process.env.CHROME_BINARY ? { binary: process.env.CHROME_BINARY } : fs.existsSync("/Applications/Google Chrome.app") ? { binary: "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" } : {}),
    args: ["--headless=new", "--no-sandbox", "--disable-gpu", "--window-size=1500,950"],
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
    if (SHOTS) {
      fs.mkdirSync(SHOTS, { recursive: true });
      await new Promise((r) => setTimeout(r, 800));
      await b.saveScreenshot(path.join(SHOTS, "web-gas-self-propelled.png"));
    }
    // The Design is kept: a reload brings it back.
    await b.url(`${origin}/ovcd/app`);
    await until(b, "the restored Design", async () => (await b.$("header")).getText().then((t) => t.includes("gas_igc_2g_self_propelled.vdesign")));
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
