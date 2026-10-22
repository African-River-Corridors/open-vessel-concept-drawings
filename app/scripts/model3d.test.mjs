// Unit tests for src/lib/model3d.ts. Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import { familyOf, kindLabelId, fixedView, orbitStep, FAMILY_TOKEN } from "../src/lib/model3d.ts";

test("every Space kind the arrangement places has a family and a label", () => {
  for (const kind of ["cargo_tank", "slop", "cofferdam", "pump_room", "double_bottom_ballast", "wing_ballast", "peak_ballast"]) {
    assert.ok(FAMILY_TOKEN[familyOf(kind)], kind);
    assert.match(kindLabelId(kind), /^app-3d-kind-[a-z-]+$/, kind);
  }
});

test("water ballast kinds share a family, and an unknown kind is a void", () => {
  assert.equal(familyOf("wing_ballast"), familyOf("peak_ballast"));
  assert.equal(familyOf("something_new"), "void");
  assert.equal(kindLabelId("something_new"), null);
});

test("the fixed views stand on the right axes with the right up", () => {
  const c = [10, 0, 2];
  assert.deepEqual(fixedView("profile", c, 30), { position: [10, 30, 2], up: [0, 0, 1] });
  assert.deepEqual(fixedView("plan", c, 30), { position: [10, 0, 32], up: [1, 0, 0] });
  assert.deepEqual(fixedView("body", c, 30), { position: [40, 0, 2], up: [0, 0, 1] });
});

test("an orbit step keeps the distance to the target and turns about z", () => {
  const target = [5, 1, 2];
  const p = [5 + 10, 1, 2 + 0.0001];
  const q = orbitStep(p, target, Math.PI / 2, 0);
  const dist = (a) => Math.hypot(a[0] - target[0], a[1] - target[1], a[2] - target[2]);
  assert.ok(Math.abs(dist(q) - dist(p)) < 1e-9);
  assert.ok(Math.abs(q[0] - 5) < 1e-3 && Math.abs(q[1] - 11) < 1e-3, `${q}`);
});

test("an orbit step stops short of the poles", () => {
  const q = orbitStep([0, 0, 10], [0, 0, 0], 0, -1);
  assert.ok(q[2] < 10 && q[2] > 9.9, `${q}`);
  const r = orbitStep([10, 0, 0], [0, 0, 0], 0, 5);
  assert.ok(r[2] > -10 && r[2] < -9.9, `${r}`);
});
