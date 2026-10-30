// Issue #668: which load each laden number of the bottom panel is for. With the IS Code loading
// conditions, the Outputs draught and freeboard are for the full load: the cargo and the consumables
// (stores and fuel). The two freeboard checks take the cargo alone. Both are right for their own
// load, so the panel names the load instead of showing two numbers for one quantity with no reason.
// Nothing here computes a result: the consumables are read from the results as the engine gives them.

/** The Outputs rows that are for the full load, by Message Id. */
export const FULL_LOAD_ROWS = ["app-metric-draught", "app-metric-freeboard"];

/** The checks that take the cargo alone, by the name in the results. */
export const CARGO_ONLY_CHECKS = ["freeboard ok", "freeboard at payload"];

/** The consumables in the laden Outputs (t), or null when the results carry none. */
export function consumablesInLoad(results: Record<string, unknown> | null | undefined): number | null {
  const loading = results?.["loading_conditions"] as { consumables_t?: unknown } | null | undefined;
  const c = loading?.consumables_t;
  return typeof c === "number" && c > 0 ? c : null;
}
