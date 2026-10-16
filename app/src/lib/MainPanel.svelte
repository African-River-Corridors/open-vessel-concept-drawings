<script lang="ts">
    import type { Evaluation, HullViews, OptionEvaluation } from "./api";
  import StaleWhy from "./StaleWhy.svelte";
  import HullView from "./HullView.svelte";
  import SpeedPower from "./views/SpeedPower.svelte";
  import Stability from "./views/Stability.svelte";
  import RuleChecks from "./views/RuleChecks.svelte";

  // The hull views are SVG drawn in Rust to the Drawing Format Standard (ADR 0009, spec 0003); the
  // panel only places and zooms them.
  let { evaluation, active, lastGood, blockers, designSpeed }: {
    evaluation: Evaluation | null;
    active: string;
    lastGood: Record<string, OptionEvaluation>;
    blockers: { name: string; label: string }[];
    /** The active Option's `max_speed_kn` input, from the Design session. */
    designSpeed: number | null;
  } = $props();

  // Two levels of tabs: a group, then a view of that group. The panel never scrolls: a drawing is
  // fitted to the space it has. Switching a tab starts the new view fitted again.
  const GROUPS = [
    {
      id: "drawings",
      label: "Drawings",
      views: [
        { id: "side", label: "Side elevation" },
        { id: "plan", label: "Plan" },
        { id: "midship", label: "Cross-section" },
      ],
    },
    {
      id: "stability-propulsion",
      label: "Stability & Propulsion",
      views: [
        { id: "stability", label: "Intact stability" },
        { id: "speed-power", label: "Power and speed curves" },
        { id: "rules", label: "Rule check calculations" },
      ],
    },
    {
      id: "boq-finance",
      label: "BOQ & Finance",
      views: [
        { id: "boq", label: "BOQ" },
        { id: "capital", label: "Capital budget" },
      ],
    },
  ];
  const HULL = ["side", "plan", "midship"];
  /** Views that show results of the Option, and so can be out of date. */
  const FROM_RESULTS = [...HULL, "stability", "speed-power", "rules"];
  type HullKey = "side" | "plan" | "midship";

  let groupId = $state(GROUPS[0].id);
  let current = $state(GROUPS[0].views[0].id);
  let group = $derived(GROUPS.find((g) => g.id === groupId) ?? GROUPS[0]);
  let currentLabel = $derived(group.views.find((v) => v.id === current)?.label ?? "");

  function pickGroup(id: string) {
    groupId = id;
    current = GROUPS.find((g) => g.id === id)!.views[0].id;
  }

  let outcome = $derived(evaluation?.options.find((o) => o.name === active)?.outcome);
  let results = $derived.by(() => {
    if (outcome?.status === "results") return outcome.results;
    const prior = lastGood[active]?.outcome;
    return outcome?.status === "not_run" && prior?.status === "results" ? prior.results : null;
  });
  const stabilityOf = (r: unknown) => (r as { stability?: never }).stability;
  let stale = $derived(outcome?.status === "not_run");
  /** The views of the Option: its own, or the last good ones while the Design has a hard error. */
  let views = $derived.by((): HullViews | null => {
    if (outcome?.status === "results") return outcome.views ?? null;
    const prior = lastGood[active]?.outcome;
    return outcome?.status === "not_run" && prior?.status === "results" ? (prior.views ?? null) : null;
  });
  const drawing = (key: HullKey): string | null => (views && key in views ? (views as Record<HullKey, string>)[key] : null);
  let message = $derived(outcome && (outcome.status === "not_available" || outcome.status === "failed") ? outcome.message : null);

  interface Curve {
    name: string;
    limiting_speed_kn: number | null;
    blockage: number;
    points: { kn: number; kw: number; fuel_lph: number; squat_m: number; squat_limited: boolean }[];
  }
  let curves = $derived(((results as { curves?: Curve[] } | null)?.curves ?? []));
</script>

<div class="panel" data-testid="views-panel">
  <header>
    <h2>Views</h2>
    <div class="tabs" role="tablist" aria-label="View groups">
      {#each GROUPS as g (g.id)}
        <button type="button" role="tab" aria-selected={g.id === groupId} class:on={g.id === groupId} onclick={() => pickGroup(g.id)}>{g.label}</button>
      {/each}
    </div>
    {#if stale && FROM_RESULTS.includes(current)}<span class="stale">out of date</span>{/if}
  </header>
  <div class="tabs sub" role="tablist" aria-label="Views of {group.label}">
    {#each group.views as v (v.id)}
      <button type="button" role="tab" aria-selected={v.id === current} class:on={v.id === current} onclick={() => (current = v.id)}>{v.label}</button>
    {/each}
  </div>
  {#if stale}<div class="whyrow"><StaleWhy {blockers} /></div>{/if}

  <div class="view" data-view={current}>
    {#if HULL.includes(current)}
      <div class="slot" data-testid="slot-{current}">
        {#if drawing(current as HullKey)}
          <!-- A new key starts the view fitted again. -->
          {#key current}
            <HullView svg={drawing(current as HullKey) ?? ""} label={currentLabel} />
          {/key}
        {:else if message}
          <p class="note">{message}</p>
        {:else if views && "message" in views}
          <p class="note">{views.message}</p>
        {:else}
          <p class="note">No results to draw.</p>
        {/if}
      </div>
    {:else if current === "speed-power"}
      <div class="charts">
        {#if message}
          <p class="note">{message}</p>
        {:else if curves.length === 0}
          <p class="note">No results to chart.</p>
        {:else}
          {#each curves as c (c.name)}
            <SpeedPower name={c.name} limitingSpeed={c.limiting_speed_kn} blockage={c.blockage} points={c.points} {designSpeed} option={active} />
          {/each}
        {/if}
      </div>
    {:else if current === "stability"}
      {#if message}
        <p class="note">{message}</p>
      {:else if !results}
        <p class="note">No results to show.</p>
      {:else}
        <!-- New results build a new chart, fitted. -->
        {#key JSON.stringify([active, stabilityOf(results)])}
          <Stability stability={stabilityOf(results)} marks={(results as { stability_marks?: never }).stability_marks} />
        {/key}
      {/if}
    {:else if current === "rules"}
      {#if message}
        <p class="note">{message}</p>
      {:else if !results}
        <p class="note">No results to show.</p>
      {:else}
        <RuleChecks checks={(results as { checks?: { name: string; ok: boolean; message: string }[] }).checks ?? []} />
      {/if}
    {:else if current === "boq"}
      <p class="note" data-testid="placeholder-boq">The bill of quantities comes in a later phase.</p>
    {:else if current === "capital"}
      <p class="note" data-testid="placeholder-capital">The capital budget is coming soon.</p>
    {/if}
  </div>
</div>

<style>
  .panel { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
  header { display: flex; align-items: center; gap: 14px; padding: 6px 14px 0; border-bottom: 1px solid var(--line); }
  h2 { font-family: var(--serif); font-size: 16px; margin: 0; align-self: center; padding-bottom: 6px; }
  .tabs { display: flex; gap: 2px; }
  .tabs button { background: none; border: 1px solid transparent; border-bottom: 0; border-radius: 4px 4px 0 0; padding: 4px 12px; cursor: pointer; color: var(--dim); }
  .tabs button.on { color: var(--navy); font-weight: 600; background: #fff; border-color: var(--line); margin-bottom: -1px; padding-bottom: 5px; }
  .tabs.sub { padding: 6px 14px; border-bottom: 1px solid var(--line); }
  .tabs.sub button { border: 1px solid var(--line); border-radius: 12px; padding: 2px 12px; }
  .tabs.sub button.on { background: var(--cream); border-color: var(--gold); margin: 0; padding-bottom: 2px; }
  .whyrow { padding: 0 14px; background: #fdf1d3; border-bottom: 1px solid var(--line); }
  .view { flex: 1; min-height: 0; padding: 10px 14px; box-sizing: border-box; }
  .slot { height: 100%; }
  .charts { height: 100%; display: flex; flex-direction: column; gap: 10px; }
  .note { color: var(--dim); }
  .stale { margin-left: auto; align-self: center; background: #fdf1d3; color: #6b4a00; font: 11px var(--sans); padding: 1px 6px; border-radius: 3px; }
</style>
