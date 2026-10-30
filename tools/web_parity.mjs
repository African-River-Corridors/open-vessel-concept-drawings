#!/usr/bin/env node
// Hash parity (issue #245): every Option of every .vdesign fixture gets the same results hash from
// the wasm Design session as from native `ovd run --json`, and the same Rule Trace (issue #269),
// and the same mesh hash as native `ovd mesh --hash` (issue #456).
// No npm dependencies.
//
//   bash tools/build_web.sh && cargo build -p ovd-cli && node tools/web_parity.mjs
//
// OVD_BIN overrides the native binary (default target/debug/ovd).
import { readFileSync, readdirSync, existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const pkg = join(root, "app", "wasm-pkg");
const fixtures = join(root, "crates", "ovd-format", "tests", "fixtures");
const bin = process.env.OVD_BIN || join(root, "target", "debug", process.platform === "win32" ? "ovd.exe" : "ovd");

if (!existsSync(join(pkg, "ovd_web.js"))) throw new Error("no wasm build: run tools/build_web.sh first");
if (!existsSync(bin)) throw new Error(`no native ovd at ${bin}: run cargo build -p ovd-cli`);

const { default: init, Session, engine_version } = await import(join(pkg, "ovd_web.js"));
await init({ module_or_path: readFileSync(join(pkg, "ovd_web_bg.wasm")) });

// `ovd mesh --hash`: name -> mesh hash, or "-" when the Option has no meshes.
function nativeMeshes(file) {
  const r = spawnSync(bin, ["mesh", "--hash", file], { encoding: "utf8", maxBuffer: 1 << 24 });
  if (r.error) throw r.error;
  const out = new Map();
  for (const line of r.stdout.split("\n").filter((l) => l.length > 0)) {
    const [name, hash] = line.split("\t");
    out.set(name, hash);
  }
  return out;
}

function native(file) {
  const r = spawnSync(bin, ["run", "--json", file], { encoding: "utf8", maxBuffer: 1 << 28 });
  if (r.error) throw r.error;
  return JSON.parse(r.stdout);
}

// `ovd run --json` calls a convoy "not_yet_supported"; the session JSON calls it "not_available".
const webStatus = (s) => (s === "not_available" ? "not_yet_supported" : s);
const rows = [];
let bad = 0;
let compared = 0;
const EXPORT_FORMATS = ["gltf", "stl", "obj", "offsets-csv"];

// `ovd export --format <f> --hash`: name -> SHA-256 of the bytes, or "-".
function nativeGlb(file, format) {
  const r = spawnSync(bin, ["export", "--format", format, "--hash", file], { encoding: "utf8", maxBuffer: 1 << 24 });
  if (r.error) throw r.error;
  const out = new Map();
  for (const line of r.stdout.split("\n").filter((l) => l.length > 0)) {
    const [name, hash] = line.split("\t");
    out.set(name, hash);
  }
  return out;
}

let meshCompared = 0;
let glbCompared = 0;
const files = readdirSync(fixtures).filter((f) => f.endsWith(".vdesign")).sort();
if (files.length === 0) throw new Error("no fixtures found");

for (const f of files) {
  const path = join(fixtures, f);
  const nat = native(path);
  const s = new Session();
  s.open_text(f, readFileSync(path, "utf8"));
  const web = JSON.parse(s.evaluate());
  // Issue #456: the mesh hash of each Option, native against WebAssembly. Not the results hash.
  const natMesh = nativeMeshes(path);
  for (const o of web.options) {
    let wm;
    try {
      wm = s.mesh_hash(o.name);
    } catch {
      wm = "-";
    }
    const nm = natMesh.get(o.name) ?? "(missing)";
    const verdict = nm === wm ? "equal" : "MISMATCH";
    if (verdict === "MISMATCH") bad++;
    if (nm !== "-") meshCompared++;
    rows.push([f, `${o.name} (mesh)`, nm, wm, verdict]);
  }
  // Issues #460 and #461: the bytes of each export of each Option, native against WebAssembly.
  for (const format of EXPORT_FORMATS) {
    const natGlb = nativeGlb(path, format);
    for (const o of web.options) {
      let wg;
      try {
        wg = createHash("sha256").update(Buffer.from(s.export_model(o.name, format, ""))).digest("hex");
      } catch {
        wg = "-";
      }
      const ng = natGlb.get(o.name) ?? "(missing)";
      const verdict = ng === wg ? "equal" : "MISMATCH";
      if (verdict === "MISMATCH") bad++;
      if (ng !== "-") glbCompared++;
      rows.push([f, `${o.name} (${format})`, ng, wg, verdict]);
    }
  }
  s.free();
  if (nat.options.length !== web.options.length) {
    rows.push([f, "(option count)", String(nat.options.length), String(web.options.length), "MISMATCH"]);
    bad++;
    continue;
  }
  nat.options.forEach((n, i) => {
    const w = web.options[i];
    const wh = w.outcome.results_hash;
    const nh = n.results_hash;
    let verdict;
    if (nh !== undefined && wh !== undefined) {
      compared++;
      // The Rule Trace is outside the hash (spec 0004), so it is compared on its own.
      const sameTrace =
        JSON.stringify(n.results?.rule_trace) === JSON.stringify(w.outcome.results?.rule_trace) &&
        n.results?.rule_trace !== undefined;
      verdict = nh === wh && n.name === w.name && sameTrace ? "equal" : "MISMATCH";
    } else if (nh === undefined && wh === undefined && n.status === webStatus(w.outcome.status)) {
      verdict = `no hash (${n.status}) on both`;
    } else {
      verdict = "MISMATCH";
    }
    if (verdict === "MISMATCH") bad++;
    rows.push([f, n.name, nh ?? "-", wh ?? "-", verdict]);
  });
}

// A drawing export for a gas fixture: an SVG with the font embedded.
{
  const s = new Session();
  s.open_text("gas.vdesign", readFileSync(join(fixtures, "gas_igc_2g_self_propelled.vdesign"), "utf8"));
  const option = JSON.parse(s.snapshot()).active_option;
  const svg = s.drawing_svg(option, "side");
  const pdf = s.drawing_pdf(option, "plan");
  const dxf = s.drawing_dxf(option, "midship");
  const okSvg = svg.includes("<svg") && svg.includes("base64");
  const okPdf = pdf instanceof Uint8Array && Buffer.from(pdf.subarray(0, 5)).toString() === "%PDF-";
  const okDxf = dxf instanceof Uint8Array && Buffer.from(dxf).toString().includes("EOF");
  console.log(`drawing export (gas fixture): svg+font ${okSvg ? "ok" : "FAIL"}, pdf ${okPdf ? "ok" : "FAIL"}, dxf ${okDxf ? "ok" : "FAIL"}`);
  if (!(okSvg && okPdf && okDxf)) bad++;
  s.free();
}

// Issue #457: how long a 100 m barge's meshes take to reach the caller (first call, with the run).
{
  const s = new Session();
  s.open_text("dry.vdesign", readFileSync(join(fixtures, "dry_bulk_box_hold.vdesign"), "utf8"));
  const option = JSON.parse(s.snapshot()).active_option;
  s.set_input(option, "length_overall_m", "100");
  const t0 = performance.now();
  const positions = s.mesh_positions(option);
  const indices = s.mesh_indices(option);
  const groups = JSON.parse(s.mesh_groups(option));
  const ms = performance.now() - t0;
  const typed = positions instanceof Float32Array && indices instanceof Uint32Array;
  console.log(`meshes of a 100 m barge: ${ms.toFixed(0)} ms, ${indices.length / 3} triangles, ${groups.groups.length} groups, typed ${typed ? "ok" : "FAIL"}`);
  if (!typed) bad++;
  s.free();
}

console.log(`engine ${engine_version()}`);
const widths = [0, 1, 2, 3, 4].map((c) => Math.max(...rows.map((r) => String(r[c]).length)));
for (const r of rows) console.log(r.map((v, c) => String(v).padEnd(widths[c])).join("  "));
console.log(`${compared} hashes, ${meshCompared} mesh hashes and ${glbCompared} export hashes compared, ${bad} problems`);
if (bad > 0 || compared === 0 || meshCompared === 0 || glbCompared === 0) {
  console.error("web parity: FAIL");
  process.exit(1);
}
console.log("web parity: ok");
