// What the bottom panel shows and how it rounds. Each value is read straight from the results
// object that `ovcd run --json` gives; nothing here computes a result. Rounding is one rule per
// quantity type, and the full value is always on hover.
import { num, say, t, type TextIds } from "./i18n.svelte";

export type Quantity =
  | "tonnes"
  | "volume"
  | "length"
  | "ratio"
  | "fraction"
  | "count"
  | "text"
  | "flag"
  | "power"
  | "speed"
  | "energy"
  | "coefficient"
  | "area";

export interface Metric {
  /** The Message Id of its label. */
  label: string;
  unit?: string;
  quantity: Quantity;
  /** Where the value lives in the results object. A key `name=X` picks the list item named X. */
  path: string[];
}

const DECIMALS: Record<Quantity, number> = {
  tonnes: 1,
  volume: 0,
  length: 2,
  ratio: 2,
  fraction: 2,
  count: 0,
  text: 0,
  flag: 0,
  power: 0,
  speed: 1,
  energy: 0,
  coefficient: 3,
  area: 0,
};

export const MAIN: Metric[] = [
  { label: "app-metric-cargo", unit: "t", quantity: "tonnes", path: ["results", "cargo_t"] },
  { label: "app-metric-governs", quantity: "text", path: ["results", "governs"] },
  { label: "app-metric-cargo-volume", unit: "m³", quantity: "volume", path: ["results", "cargo_volume_m3"] },
  { label: "app-metric-lightship", unit: "t", quantity: "tonnes", path: ["results", "lightship_t"] },
  { label: "app-metric-displacement", unit: "t", quantity: "tonnes", path: ["results", "displacement_t"] },
  { label: "app-metric-deadweight", unit: "t", quantity: "tonnes", path: ["results", "deadweight_t"] },
  { label: "app-metric-draught", unit: "m", quantity: "length", path: ["results", "draught_m"] },
  { label: "app-metric-draught-max", unit: "m", quantity: "length", path: ["results", "draught_max_m"] },
  { label: "app-metric-light-draught", unit: "m", quantity: "length", path: ["results", "light_draught_m"] },
  { label: "app-metric-freeboard", unit: "m", quantity: "length", path: ["results", "freeboard_m"] },
  { label: "app-metric-air-draft-loaded", unit: "m", quantity: "length", path: ["results", "air_draft_loaded_m"] },
  { label: "app-metric-air-draft-light", unit: "m", quantity: "length", path: ["results", "air_draft_light_m"] },
  { label: "app-metric-depth-to-draught", quantity: "ratio", path: ["results", "h_over_T"] },
  // Issue #275: the Service Area, the Rule Sets and what was not checked.
  { label: "app-metric-scope", quantity: "text", path: ["rule_trace", "scope"] },
];

/** Shown only for an Option that has a tank arrangement (a gas Cargo). */
export const TANKS: Metric[] = [
  { label: "app-metric-tanks-across", quantity: "count", path: ["arrangement", "n_wide"] },
  { label: "app-metric-tanks-along", quantity: "count", path: ["arrangement", "n_long"] },
  { label: "app-metric-tank-od", unit: "m", quantity: "length", path: ["arrangement", "tank_od_m"] },
  { label: "app-metric-side-clearance", unit: "m", quantity: "length", path: ["arrangement", "side_clearance_m"] },
  { label: "app-metric-ruleset", quantity: "text", path: ["arrangement", "ruleset"] },
  { label: "app-metric-fill-ratio", quantity: "fraction", path: ["arrangement", "fill_ratio"] },
  { label: "app-metric-depth-optimised", quantity: "flag", path: ["arrangement", "depth_optimised"] },
];

/** Shown only for an Option with a specified Propulsor: its `powering` block (issue #353). */
export const POWERING: Metric[] = [
  { label: "app-metric-installed-power", unit: "kW", quantity: "power", path: ["powering", "installed_power_kw"] },
  { label: "app-metric-bollard-pull", unit: "t", quantity: "tonnes", path: ["powering", "bollard_pull_t"] },
  {
    label: "app-metric-service-speed-deep",
    unit: "kn",
    quantity: "speed",
    path: ["powering", "environments", "name=Deep water", "service_speed_kn"],
  },
  {
    label: "app-metric-service-speed-channel",
    unit: "kn",
    quantity: "speed",
    path: ["powering", "environments", "name=Channel", "service_speed_kn"],
  },
  { label: "app-metric-battery-energy", unit: "kWh", quantity: "energy", path: ["powering", "battery_energy_kwh"] },
];

/** The hull's upright hydrostatics at the laden draught, from its sections (issue #448). */
export const HYDROSTATICS: Metric[] = [
  { label: "app-metric-block-coefficient", quantity: "coefficient", path: ["hydrostatics", "cb"] },
  { label: "app-metric-midship-coefficient", quantity: "coefficient", path: ["hydrostatics", "cm"] },
  { label: "app-metric-prismatic-coefficient", quantity: "coefficient", path: ["hydrostatics", "cp"] },
  { label: "app-metric-waterplane-coefficient", quantity: "coefficient", path: ["hydrostatics", "cwp"] },
  { label: "app-metric-waterline-length", unit: "m", quantity: "length", path: ["hydrostatics", "l_wl_m"] },
  { label: "app-metric-kb", unit: "m", quantity: "length", path: ["hydrostatics", "kb_m"] },
  { label: "app-metric-bm-transverse", unit: "m", quantity: "length", path: ["hydrostatics", "bm_t_m"] },
  { label: "app-metric-km-transverse", unit: "m", quantity: "length", path: ["hydrostatics", "km_t_m"] },
  { label: "app-metric-lcb", unit: "m", quantity: "length", path: ["hydrostatics", "lcb_m"] },
  { label: "app-metric-lcf", unit: "m", quantity: "length", path: ["hydrostatics", "lcf_m"] },
  { label: "app-metric-waterplane-area", unit: "m²", quantity: "area", path: ["hydrostatics", "waterplane_area_m2"] },
  { label: "app-metric-wetted-surface", unit: "m²", quantity: "area", path: ["hydrostatics", "wetted_surface_m2"] },
  { label: "app-metric-tpc", unit: "t/cm", quantity: "tonnes", path: ["hydrostatics", "tpc_t"] },
  { label: "app-metric-mct", unit: "t·m/cm", quantity: "tonnes", path: ["hydrostatics", "mct_1cm_tm"] },
];

export function valueAt(results: unknown, path: string[]): unknown {
  let v: unknown = results;
  for (const key of path) {
    if (v === null || typeof v !== "object") return undefined;
    if (Array.isArray(v) && key.startsWith("name=")) {
      const name = key.slice(5);
      v = v.find((item) => item !== null && typeof item === "object" && (item as { name?: unknown }).name === name);
      continue;
    }
    v = (v as Record<string, unknown>)[key];
  }
  return v;
}

export function hasPowering(results: unknown): boolean {
  const p = valueAt(results, ["powering"]);
  return p !== null && p !== undefined && typeof p === "object";
}

/** The value as a person reads it: rounded by quantity type, with thousands separators. */
export function show(m: Metric, v: unknown, ids?: TextIds | null): string {
  if (v === null || v === undefined) return "–";
  if (typeof v === "boolean") return t(v ? "app-yes" : "app-no");
  if (typeof v === "number") {
    return num(v, DECIMALS[m.quantity]);
  }
  return typeof v === "string" ? say(v, ids) : String(v);
}

/** The engine's `text_ids` of a results object: its English texts with their Message Ids. */
export function textIds(results: unknown): TextIds | null {
  return (valueAt(results, ["text_ids"]) as TextIds | undefined) ?? null;
}

/** The value at full precision, for the hover text. */
export function exact(m: Metric, v: unknown): string {
  if (v === null || v === undefined) return t("app-not-applicable");
  return `${String(v)}${m.unit ? " " + m.unit : ""}`;
}

export function hasHydrostatics(results: unknown): boolean {
  const h = valueAt(results, ["hydrostatics"]);
  return h !== null && h !== undefined && typeof h === "object";
}

export function hasTanks(results: unknown): boolean {
  const a = valueAt(results, ["arrangement"]);
  return a !== null && a !== undefined && typeof a === "object";
}
