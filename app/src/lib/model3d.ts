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
