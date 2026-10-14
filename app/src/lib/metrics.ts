// What the bottom panel shows and how it rounds. Each value is read straight from the results
// object that `ovcd run --json` gives; nothing here computes a result. Rounding is one rule per
// quantity type, and the full value is always on hover.

export type Quantity = "tonnes" | "volume" | "length" | "ratio" | "fraction" | "count" | "text" | "flag";

export interface Metric {
  label: string;
  unit?: string;
  quantity: Quantity;
  /** Where the value lives in the results object. */
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
};

export const MAIN: Metric[] = [
  { label: "Cargo", unit: "t", quantity: "tonnes", path: ["results", "cargo_t"] },
  { label: "Governed by", quantity: "text", path: ["results", "governs"] },
  { label: "Cargo volume", unit: "m³", quantity: "volume", path: ["results", "cargo_volume_m3"] },
  { label: "Lightship", unit: "t", quantity: "tonnes", path: ["results", "lightship_t"] },
  { label: "Displacement", unit: "t", quantity: "tonnes", path: ["results", "displacement_t"] },
  { label: "Deadweight", unit: "t", quantity: "tonnes", path: ["results", "deadweight_t"] },
  { label: "Draught", unit: "m", quantity: "length", path: ["results", "draught_m"] },
  { label: "Maximum draught", unit: "m", quantity: "length", path: ["results", "draught_max_m"] },
  { label: "Light draught", unit: "m", quantity: "length", path: ["results", "light_draught_m"] },
  { label: "Freeboard", unit: "m", quantity: "length", path: ["results", "freeboard_m"] },
  { label: "Air draft, loaded", unit: "m", quantity: "length", path: ["results", "air_draft_loaded_m"] },
  { label: "Air draft, light", unit: "m", quantity: "length", path: ["results", "air_draft_light_m"] },
  { label: "Depth to draught (h/T)", quantity: "ratio", path: ["results", "h_over_T"] },
];

/** Shown only for an Option that has a tank arrangement (a gas Cargo). */
export const TANKS: Metric[] = [
  { label: "Tanks across", quantity: "count", path: ["arrangement", "n_wide"] },
  { label: "Tanks along", quantity: "count", path: ["arrangement", "n_long"] },
  { label: "Tank outer diameter", unit: "m", quantity: "length", path: ["arrangement", "tank_od_m"] },
  { label: "Side clearance", unit: "m", quantity: "length", path: ["arrangement", "side_clearance_m"] },
  { label: "Ruleset", quantity: "text", path: ["arrangement", "ruleset"] },
  { label: "Fill ratio", quantity: "fraction", path: ["arrangement", "fill_ratio"] },
  { label: "Depth optimised", quantity: "flag", path: ["arrangement", "depth_optimised"] },
];

export function valueAt(results: unknown, path: string[]): unknown {
  let v: unknown = results;
  for (const key of path) {
    if (v === null || typeof v !== "object") return undefined;
    v = (v as Record<string, unknown>)[key];
  }
  return v;
}

/** The value as a person reads it: rounded by quantity type, with thousands separators. */
export function show(m: Metric, v: unknown): string {
  if (v === null || v === undefined) return "–";
  if (typeof v === "boolean") return v ? "yes" : "no";
  if (typeof v === "number") {
    return v.toLocaleString("en-GB", {
      minimumFractionDigits: DECIMALS[m.quantity],
      maximumFractionDigits: DECIMALS[m.quantity],
    });
  }
  return String(v);
}

/** The value at full precision, for the hover text. */
export function exact(m: Metric, v: unknown): string {
  if (v === null || v === undefined) return "not applicable";
  return `${String(v)}${m.unit ? " " + m.unit : ""}`;
}

export function hasTanks(results: unknown): boolean {
  const a = valueAt(results, ["arrangement"]);
  return a !== null && a !== undefined && typeof a === "object";
}
