// The pure parts of the 3D view (issue #458): which family a Space kind belongs to, where the camera
// stands for the three fixed views, and one step of a keyboard orbit. No three.js here, so node tests
// can run it. The axes are the hull's: x forward, y to starboard, z up.

/** The colour family of a Space (one token each in app.css, checked by scripts/contrast.mjs). */
export type Family = "cargo" | "ballast" | "void" | "machinery";

const FAMILIES: Record<string, Family> = {
  cargo_tank: "cargo",
  slop: "cargo",
  cargo_hold: "cargo",
  double_bottom_ballast: "ballast",
  wing_ballast: "ballast",
  peak_ballast: "ballast",
  cofferdam: "void",
  void: "void",
  peak_void: "void",
  pump_room: "machinery",
  machinery: "machinery",
};

/** The family of a Space kind; a kind the view does not know is a void, so it is still drawn. */
export function familyOf(kind: string): Family {
  return FAMILIES[kind] ?? "void";
}

/** The CSS token that holds the colour of a family. */
export const FAMILY_TOKEN: Record<Family, string> = {
  cargo: "--space-cargo",
  ballast: "--space-ballast",
  void: "--space-void",
  machinery: "--space-machinery",
};

/** The Message Id of a Space kind's name in the layer list, or null when the view has none. */
export function kindLabelId(kind: string): string | null {
  return kind in FAMILIES ? `app-3d-kind-${kind.replaceAll("_", "-")}` : null;
}

export type FixedView = "profile" | "plan" | "body";
export type Vec3 = [number, number, number];

/**
 * The mirror from model axes to view axes (spec 0009, "Axes and handedness"). The model axes (x
 * forward, y to starboard, z up) are left-handed; three.js is right-handed. The view puts every mesh
 * under one root group with this scale, so the picture is the vessel and not its mirror image; three.js
 * flips the front face for the negative determinant, so faces still face out.
 */
export const VIEW_MIRROR: Vec3 = [1, -1, 1];

/** A point in model axes, in the view's (right-handed) axes. */
export function toView(p: Vec3): Vec3 {
  return [p[0] * VIEW_MIRROR[0], p[1] * VIEW_MIRROR[1], p[2] * VIEW_MIRROR[2]];
}

/**
 * Where the camera stands for a fixed view of a hull of `size` (length, beam, depth) about `centre`,
 * and its up vector. Profile: from the starboard side. Plan: from above, the bow up the screen. Body
 * plan: from ahead. `distance` is how far back it stands. All in model axes: the view takes them
 * through `toView`.
 */
export function fixedView(view: FixedView, centre: Vec3, distance: number): { position: Vec3; up: Vec3 } {
  const [cx, cy, cz] = centre;
  switch (view) {
    case "profile":
      return { position: [cx, cy + distance, cz], up: [0, 0, 1] };
    case "plan":
      return { position: [cx, cy, cz + distance], up: [1, 0, 0] };
    case "body":
      return { position: [cx + distance, cy, cz], up: [0, 0, 1] };
  }
}

/**
 * One step of an orbit about `target`, as the arrow keys give it: `dAzimuth` turns about the z axis and
 * `dPolar` tips towards or away from the pole (both in radians). The polar angle stays between a little
 * above 0 and a little below pi, so the camera never flips over the pole.
 */
export function orbitStep(position: Vec3, target: Vec3, dAzimuth: number, dPolar: number): Vec3 {
  const [dx, dy, dz] = [position[0] - target[0], position[1] - target[1], position[2] - target[2]];
  const r = Math.hypot(dx, dy, dz);
  if (r === 0) return position;
  const eps = 0.01;
  const polar = Math.min(Math.PI - eps, Math.max(eps, Math.acos(dz / r) + dPolar));
  const azimuth = Math.atan2(dy, dx) + dAzimuth;
  return [
    target[0] + r * Math.sin(polar) * Math.cos(azimuth),
    target[1] + r * Math.sin(polar) * Math.sin(azimuth),
    target[2] + r * Math.cos(polar),
  ];
}

/**
 * The part of the core's shared buffers that one group uses, as buffers of its own (spec 0009, "The
 * 3D view"; issue #683). The core appends each part's vertices then its triangles, so a group's
 * vertices are one run, from its lowest index to its highest: the slice is that run, and the indices
 * are moved down by its start. Each group then gets its own bounding sphere, so transparent parts
 * sort and frustum culling works.
 */
export function groupSlice(
  positions: Float32Array,
  indices: Uint32Array,
  start: number,
  count: number,
): { positions: Float32Array; indices: Uint32Array } {
  const own = indices.subarray(start, start + count);
  if (own.length === 0) return { positions: new Float32Array(0), indices: new Uint32Array(0) };
  let lo = Infinity;
  let hi = -Infinity;
  for (const i of own) {
    if (i < lo) lo = i;
    if (i > hi) hi = i;
  }
  return { positions: positions.slice(3 * lo, 3 * (hi + 1)), indices: own.map((i) => i - lo) };
}

/**
 * Whether a layer is drawn with flat shading: the hull, the decks, the bulkheads, the Spaces, the
 * waterline planes and the Deck Items (boxes) are made of flat panels, so each face takes its own
 * normal and the knuckles show.
 * Any other layer is a curved part (tanks, domes, nozzles: later tickets of spec 0009), drawn smooth.
 */
export function flatShaded(layer: string): boolean {
  return ["hull", "deck", "bulkhead", "space", "waterline", "deck_item"].includes(layer);
}

/** The default camera's height above the horizon, in degrees (spec 0009, "The 3D view"). */
export const HOME_ELEVATION_DEG = 25;
/** The margin round the model in the default view: the model fills 1 / 1.1 of the half view. */
export const HOME_MARGIN = 1.1;

/**
 * The default camera of the 3D view, in model axes: from the aft-starboard quarter (45 degrees off the
 * centre line, abaft and to starboard), `HOME_ELEVATION_DEG` above the horizon, looking at the centre
 * of the box `min`..`max`, and just far enough back that every corner of the box is inside the view
 * with the margin `HOME_MARGIN` (vertical field of view `fovDeg`, width over height `aspect`).
 */
export function homeView(min: Vec3, max: Vec3, fovDeg: number, aspect: number): { position: Vec3; target: Vec3; up: Vec3 } {
  const target: Vec3 = [(min[0] + max[0]) / 2, (min[1] + max[1]) / 2, (min[2] + max[2]) / 2];
  const el = (HOME_ELEVATION_DEG * Math.PI) / 180;
  const az = Math.PI / 4;
  // From the target towards the camera: aft (x < 0), to starboard (y > 0), up.
  const back: Vec3 = [-Math.cos(el) * Math.cos(az), Math.cos(el) * Math.sin(az), Math.sin(el)];
  // The screen's right and up, seen from the camera.
  const right: Vec3 = [Math.sin(az), Math.cos(az), 0];
  const up: Vec3 = [
    back[1] * right[2] - back[2] * right[1],
    back[2] * right[0] - back[0] * right[2],
    back[0] * right[1] - back[1] * right[0],
  ];
  const tanV = Math.tan((fovDeg * Math.PI) / 360);
  const tanH = tanV * aspect;
  let distance = 0;
  for (const cx of [min[0], max[0]])
    for (const cy of [min[1], max[1]])
      for (const cz of [min[2], max[2]]) {
        const p = [cx - target[0], cy - target[1], cz - target[2]];
        const dot = (v: Vec3) => p[0] * v[0] + p[1] * v[1] + p[2] * v[2];
        // A corner at depth dot(back) towards the camera fits when its offset over its distance from
        // the camera is no more than the tangent of the half view, over the margin.
        const toward = dot(back);
        distance = Math.max(distance, toward + (Math.abs(dot(right)) * HOME_MARGIN) / tanH, toward + (Math.abs(dot(up)) * HOME_MARGIN) / tanV);
      }
  const position: Vec3 = [target[0] + back[0] * distance, target[1] + back[1] * distance, target[2] + back[2] * distance];
  return { position, target, up: [0, 0, 1] };
}
