// Unit tests for src/lib/model3d.ts. Run: npm run test:unit
import test from "node:test";
import assert from "node:assert/strict";
import { familyOf, flatShaded, groupSlice, homeView, seaPlane, kindLabelId, fixedView, orbitStep, toView, VIEW_MIRROR, FAMILY_TOKEN } from "../src/lib/model3d.ts";

// The default camera for the 76.5 x 11.4 x 3.5 m box, field of view 35 degrees, aspect 1.6: what homeView
// printed (Label: snapshot).
const SNAPSHOT_HOME = [-19.018667, 57.268667, 39.516316];

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

test("the view mirrors the left-handed model axes once, in y (spec 0009, issue #682)", () => {
  // Label: property. With x forward and z up, a right-handed y points to port; the model's y points to
  // starboard, so the model axes are left-handed. One mirror (determinant -1) puts port at view y > 0.
  const [ex, ey, ez] = [toView([1, 0, 0]), toView([0, 1, 0]), toView([0, 0, 1])];
  const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
  const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
  assert.equal(dot(cross([1, 0, 0], [0, 1, 0]), [0, 0, 1]), 1, "the model axes as numbers");
  assert.equal(dot(cross(ex, ey), ez), -1, "one mirror: the determinant is -1");
  assert.deepEqual(toView([3, -2, 1]), [3, 2, 1], "a point to port has view y > 0");
  assert.equal(VIEW_MIRROR[0] * VIEW_MIRROR[1] * VIEW_MIRROR[2], -1);
});

test("a group's slice holds its own vertices and its triangles re-indexed (issue #683)", () => {
  // Label: property. Two parts appended as the core appends them: a triangle, then a quad.
  const positions = new Float32Array([0, 0, 0, 1, 0, 0, 0, 1, 0, 5, 5, 5, 6, 5, 5, 6, 6, 5, 5, 6, 5]);
  const indices = new Uint32Array([0, 1, 2, 3, 4, 5, 3, 5, 6]);
  const a = groupSlice(positions, indices, 0, 3);
  assert.deepEqual([...a.indices], [0, 1, 2]);
  assert.deepEqual([...a.positions], [0, 0, 0, 1, 0, 0, 0, 1, 0]);
  const b = groupSlice(positions, indices, 3, 6);
  assert.deepEqual([...b.indices], [0, 1, 2, 0, 2, 3]);
  assert.deepEqual([...b.positions], [5, 5, 5, 6, 5, 5, 6, 6, 5, 5, 6, 5]);
  // Each triangle has the same corners as in the shared buffers.
  for (let t = 0; t < 6; t++) {
    const k = b.indices[t];
    assert.deepEqual([...b.positions.slice(3 * k, 3 * k + 3)], [...positions.slice(3 * indices[3 + t], 3 * indices[3 + t] + 3)]);
  }
  assert.equal(groupSlice(positions, indices, 9, 0).indices.length, 0);
});

test("the flat parts are flat shaded, and a curved part is smooth (issue #683)", () => {
  for (const layer of ["hull", "deck", "bulkhead", "space", "waterline"]) assert.equal(flatShaded(layer), true, layer);
  assert.equal(flatShaded("tank"), false);
});

test("the default camera stands aft to starboard, 25 degrees up, and frames the box with 10 % margin (issue #683)", () => {
  // Label: snapshot (the position, as the code printed it) and property (the angles and the fit).
  const min = [0, -5.7, 0];
  const max = [76.5, 5.7, 3.5];
  const fov = 35;
  const aspect = 1.6;
  const { position, target, up } = homeView(min, max, fov, aspect);
  assert.deepEqual(target, [38.25, 0, 1.75]);
  assert.deepEqual(up, [0, 0, 1]);
  const d = position.map((v, i) => v - target[i]);
  const r = Math.hypot(...d);
  const deg = (x) => (x * 180) / Math.PI;
  assert.ok(Math.abs(deg(Math.asin(d[2] / r)) - 25) < 1e-9, "25 degrees above the horizon");
  assert.ok(d[0] < 0 && d[1] > 0 && Math.abs(-d[0] - d[1]) < 1e-9, "abaft and to starboard, on the 45 degree line");
  // Project the eight corners with a camera built here, not with the function's own basis.
  const f = d.map((v) => -v / r); // looking direction
  const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
  const norm = (a) => a.map((v) => v / Math.hypot(...a));
  const rt = norm(cross(f, [0, 0, 1]));
  const u = cross(rt, f);
  const tv = Math.tan((fov * Math.PI) / 360);
  let most = 0;
  for (const x of [min[0], max[0]])
    for (const y of [min[1], max[1]])
      for (const z of [min[2], max[2]]) {
        const p = [x - position[0], y - position[1], z - position[2]];
        const depth = p[0] * f[0] + p[1] * f[1] + p[2] * f[2];
        const sx = (p[0] * rt[0] + p[1] * rt[1] + p[2] * rt[2]) / depth / (tv * aspect);
        const sy = (p[0] * u[0] + p[1] * u[1] + p[2] * u[2]) / depth / tv;
        most = Math.max(most, Math.abs(sx), Math.abs(sy));
      }
  assert.ok(Math.abs(most - 1 / 1.1) < 1e-9, `the farthest corner fills 1 / 1.1 of the half view: ${most}`);
  assert.deepEqual(position.map((v) => Number(v.toFixed(6))), SNAPSHOT_HOME);
});

test("the sea plane passes through the draughts at the hull's ends, and is flat across (issue #684)", () => {
  // Label: property. A trimmed position: 1.92 m aft at x = 0, 2.51 m forward at x = 76.5.
  const s = seaPlane(0, 76.5, 1.92, 2.51, 40);
  assert.equal(s.z(0), 1.92);
  assert.equal(s.z(76.5), 2.51);
  const p = s.positions;
  const at = (i) => [p[3 * i], p[3 * i + 1], p[3 * i + 2]];
  for (const i of [0, 1, 2, 3]) assert.ok(Math.abs(at(i)[2] - s.z(at(i)[0])) < 1e-5, `corner ${i}: ${at(i)}`);
  assert.equal(at(0)[2], at(3)[2], "level across at the aft edge");
  assert.equal(at(1)[2], at(2)[2], "level across at the forward edge");
  assert.ok(at(0)[0] < 0 && at(1)[0] > 76.5 && at(0)[1] < 0 && at(3)[1] > 0, "it reaches past the hull");
  // The two triangles face up (counter-clockwise from above, in model axes).
  for (const [a, b, c] of [[0, 1, 2], [0, 2, 3]]) {
    const [u, v] = [at(b).map((x, k) => x - at(a)[k]), at(c).map((x, k) => x - at(a)[k])];
    assert.ok(u[0] * v[1] - u[1] * v[0] > 0);
  }
});
