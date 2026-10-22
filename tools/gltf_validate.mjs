#!/usr/bin/env node
// The Khronos glTF validator on the `.glb` of every fixture (issue #460).
//
//   node tools/gltf_validate.mjs <path to ovcd>
//
// Each fixture Option is written with `ovcd export --format gltf` and checked with the `gltf-validator`
// package (pinned in the CI job). Any error fails; so does a fixture that writes nothing (a pushed
// convoy has no model: it is the one allowed exception). Warnings and infos are printed.
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import validator from "gltf-validator";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const bin = process.argv[2];
if (!bin) {
  console.error("usage: node tools/gltf_validate.mjs <path to ovcd>");
  process.exit(2);
}
const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "ovcd-gltf-"));
const dirs = ["crates/ovcd-format/tests/fixtures", "crates/ovcd-format/tests/legacy"];
const allowed = new Set(["pushed_convoy.vdesign"]);
let checked = 0;
let bad = 0;
for (const dir of dirs) {
  for (const f of fs.readdirSync(path.join(root, dir)).filter((n) => n.endsWith(".vdesign")).sort()) {
    const file = path.join(root, dir, f);
    // Each Option by name: `--hash` lists them.
    const list = spawnSync(bin, ["export", "--format", "gltf", "--hash", file], { encoding: "utf8" });
    const names = list.stdout.split("\n").filter((l) => l.length > 0).map((l) => l.split("\t"));
    for (const [name, hash] of names) {
      if (hash === "-") {
        if (!allowed.has(f)) {
          console.log(`${f} / ${name}: no model: ${hash}`);
          bad++;
        }
        continue;
      }
      const out = path.join(tmp, `${f}-${name}.glb`.replace(/[^A-Za-z0-9_.-]/g, "_"));
      const r = spawnSync(bin, ["export", "--format", "gltf", "--option", name, "-o", out, file], { encoding: "utf8" });
      if (r.status !== 0) {
        console.log(`${f} / ${name}: ovcd exited ${r.status}: ${r.stderr.trim()}`);
        bad++;
        continue;
      }
      const report = await validator.validateBytes(new Uint8Array(fs.readFileSync(out)), { uri: out });
      const { numErrors, numWarnings } = report.issues;
      console.log(`${f} / ${name}: ${numErrors} errors, ${numWarnings} warnings`);
      for (const m of report.issues.messages.filter((x) => x.severity <= 1).slice(0, 10)) {
        console.log(`   ${m.code} ${m.pointer}: ${m.message}`);
      }
      checked++;
      bad += numErrors > 0 ? 1 : 0;
    }
  }
}
console.log(`${checked} files checked, ${bad} problems`);
process.exit(bad > 0 || checked === 0 ? 1 : 0);
