<script lang="ts">
  import { num, say, t, type TextIds } from "../i18n.svelte";

  // The ballast tanks of the hull and the ballast plan of each loading condition, exactly as the
  // engine gives them (`loading_conditions` of the results, issue #426). Numbers are rounded for
  // reading; the full value is on hover. Slack tanks have a free surface, which the stability view
  // already counts in GM; the moment is listed here so the plan can be read.
  interface Tank {
    name: string;
    kind: string;
    side: string;
    capacity_m3: number;
    capacity_t: number;
    lcg_m: number;
    tcg_m: number;
    vcg_m: number;
  }
  interface Fill {
    name: string;
    fill: number;
    mass_t: number;
    lcg_m: number;
    tcg_m: number;
    vcg_m: number;
    free_surface_moment_tm: number;
  }
  interface Condition {
    name: string;
    ballast_t: number;
    ballast: Fill[];
    plan_found: boolean | null;
  }
  interface LoadingConditions {
    ballast_tank_count: number;
    ballast_capacity_m3: number;
    ballast_capacity_t: number;
    ballast_tanks: Tank[];
    conditions: Condition[];
  }
  let { loading, ids }: { loading: LoadingConditions | null | undefined; ids: TextIds | null } = $props();
  /** Engine text (a condition name) in the Display Language. */
  const tr = (text: string) => say(text, ids);

  const KINDS: Record<string, string> = {
    double_bottom_ballast: "app-ballast-kind-double-bottom",
    wing_ballast: "app-ballast-kind-wing",
    peak_ballast: "app-ballast-kind-peak",
  };
  const SIDES: Record<string, string> = {
    port: "app-ballast-side-port",
    starboard: "app-ballast-side-starboard",
    centre: "app-ballast-side-centre",
  };
  /** A label from a Message Id, or the engine's own key when this view does not know it. */
  const named = (table: Record<string, string>, key: string) => (table[key] ? t(table[key]) : key);

  const planned = $derived((loading?.conditions ?? []).filter((c) => c.ballast.length > 0 || c.plan_found === false));
</script>

<div class="ballast" data-testid="ballast">
  {#if !loading || loading.ballast_tank_count === 0}
    <p class="note" data-testid="ballast-none">{t("app-ballast-none")}</p>
  {:else}
    <p class="summary" data-testid="ballast-summary">
      <strong>{t("app-ballast-count", { count: num(loading.ballast_tank_count, 0) })}</strong>
      <span class="dim">{t("app-ballast-capacity", { m3: num(loading.ballast_capacity_m3, 0), t: num(loading.ballast_capacity_t, 0) })}</span>
    </p>

    <table class="tanks" data-testid="ballast-tanks">
      <thead>
        <tr>
          <th>{t("app-ballast-col-tank")}</th>
          <th>{t("app-ballast-col-kind")}</th>
          <th>{t("app-ballast-col-side")}</th>
          <th class="num">{t("app-ballast-col-capacity-m3")}<span class="unit">&nbsp;(m³)</span></th>
          <th class="num">{t("app-ballast-col-capacity-t")}<span class="unit">&nbsp;(t)</span></th>
          <th class="num">{t("app-ballast-col-lcg")}<span class="unit">&nbsp;(m)</span></th>
          <th class="num">{t("app-ballast-col-tcg")}<span class="unit">&nbsp;(m)</span></th>
          <th class="num">{t("app-ballast-col-vcg")}<span class="unit">&nbsp;(m)</span></th>
        </tr>
      </thead>
      <tbody>
        {#each loading.ballast_tanks as k (k.name)}
          <tr>
            <th>{k.name}</th>
            <td>{named(KINDS, k.kind)}</td>
            <td>{named(SIDES, k.side)}</td>
            <td class="num" title={String(k.capacity_m3)}>{num(k.capacity_m3, 1)}</td>
            <td class="num" title={String(k.capacity_t)}>{num(k.capacity_t, 1)}</td>
            <td class="num" title={String(k.lcg_m)}>{num(k.lcg_m, 2)}</td>
            <td class="num" title={String(k.tcg_m)}>{num(k.tcg_m, 2)}</td>
            <td class="num" title={String(k.vcg_m)}>{num(k.vcg_m, 2)}</td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#each planned as c (c.name)}
      <h4 data-testid="ballast-condition">{t("app-ballast-of", { condition: tr(c.name), mass: num(c.ballast_t, 1) })}</h4>
      {#if c.plan_found === false}
        <p class="note fail">{t("app-ballast-no-plan")}</p>
      {/if}
      {#if c.ballast.length > 0}
        <table class="plan">
          <thead>
            <tr>
              <th>{t("app-ballast-col-tank")}</th>
              <th class="num">{t("app-ballast-col-fill")}</th>
              <th class="num">{t("app-ballast-col-mass")}<span class="unit">&nbsp;(t)</span></th>
              <th class="num">{t("app-ballast-col-lcg")}<span class="unit">&nbsp;(m)</span></th>
              <th class="num">{t("app-ballast-col-tcg")}<span class="unit">&nbsp;(m)</span></th>
              <th class="num">{t("app-ballast-col-vcg")}<span class="unit">&nbsp;(m)</span></th>
              <th class="num">{t("app-ballast-col-free-surface")}<span class="unit">&nbsp;(t·m)</span></th>
            </tr>
          </thead>
          <tbody>
            {#each c.ballast as f (f.name)}
              <tr>
                <th>{f.name}</th>
                <td class="num" title={String(f.fill)}>{num(f.fill, 2)}</td>
                <td class="num" title={String(f.mass_t)}>{num(f.mass_t, 1)}</td>
                <td class="num" title={String(f.lcg_m)}>{num(f.lcg_m, 2)}</td>
                <td class="num" title={String(f.tcg_m)}>{num(f.tcg_m, 2)}</td>
                <td class="num" title={String(f.vcg_m)}>{num(f.vcg_m, 2)}</td>
                <td class="num" title={String(f.free_surface_moment_tm)}>{num(f.free_surface_moment_tm, 1)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    {/each}
  {/if}
</div>

<style>
  .ballast { height: 100%; overflow: auto; }
  .summary { display: flex; gap: 10px; align-items: baseline; flex-wrap: wrap; margin: 0 0 8px; }
  .dim { color: var(--ink-3); font-size: 12px; }
  .note { color: var(--ink-3); }
  .note.fail { color: var(--fail); }
  table { border-collapse: collapse; font-size: 12px; display: block; overflow-x: auto; max-width: 100%; }
  th { text-align: left; font-weight: normal; padding: 3px 10px 3px 0; white-space: nowrap; }
  thead th { color: var(--ink-3); vertical-align: bottom; }
  td { padding: 3px 8px; }
  .num { text-align: right; font-variant-numeric: tabular-nums; }
  th.num { text-align: right; }
  .unit { color: var(--ink-3); }
  h4 { font-family: var(--font-serif); font-size: 13px; margin: 14px 0 4px; color: var(--ink); }
</style>
