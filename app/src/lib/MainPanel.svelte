<script lang="ts">
    import type { Evaluation, HullViews, OptionEvaluation } from "./api";
  import StaleWhy from "./StaleWhy.svelte";
  import HullView from "./HullView.svelte";
  import SpeedPower from "./views/SpeedPower.svelte";
  import Stability from "./views/Stability.svelte";
  import RuleChecks from "./views/RuleChecks.svelte";
  import CapitalBudget from "./views/CapitalBudget.svelte";
  import { say, t, type TextIds } from "./i18n.svelte";

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

  // Two levels of tabs: a group, then a view of that group. Each label is a Message Id. The panel never scrolls: a drawing is
  // fitted to the space it has. Switching a tab starts the new view fitted again.
  const GROUPS = [
    {
      id: "drawings",
      label: "app-group-drawings",
      views: [
        { id: "side", label: "app-view-side" },
        { id: "plan", label: "app-view-plan" },
        { id: "midship", label: "app-view-midship" },
      ],
    },
    {
      id: "stability-propulsion",
      label: "app-group-stability-propulsion",
      views: [
        { id: "stability", label: "app-view-stability" },
        { id: "speed-power", label: "app-view-speed-power" },
        { id: "rules", label: "app-view-rules" },
      ],
    },
    {
      id: "boq-finance",
      label: "app-group-boq-finance",
      views: [
        { id: "boq", label: "app-view-boq" },
        { id: "capital", label: "app-view-capital" },
      ],
    },
  ];
  const HULL = ["side", "plan", "midship"];
  /** Views that show results of the Option, and so can be out of date. */
  const FROM_RESULTS = [...HULL, "stability", "speed-power", "rules", "capital"];
  type HullKey = "side" | "plan" | "midship";

  let groupId = $state(GROUPS[0].id);
  let current = $state(GROUPS[0].views[0].id);
  let group = $derived(GROUPS.find((g) => g.id === groupId) ?? GROUPS[0]);
  let currentLabel = $derived(t(group.views.find((v) => v.id === current)?.label ?? ""));

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
  let message = $derived(outcome && (outcome.status === "not_available" || outcome.status === "failed") ? say(outcome.message, null, outcome.msg) : null);
  /** The engine's Message Ids of the shown results (ADR 0014). */
  let ids = $derived(((results as { text_ids?: TextIds } | null)?.text_ids ?? null));

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
    <h2>{t("app-views")}</h2>
    <div class="tabs" role="tablist" aria-label={t("app-view-groups")}>
      {#each GROUPS as g (g.id)}
        <button type="button" role="tab" aria-selected={g.id === groupId} class:on={g.id === groupId} onclick={() => pickGroup(g.id)}>{t(g.label)}</button>
      {/each}
    </div>
    {#if stale && FROM_RESULTS.includes(current)}<span class="stale">{t("app-out-of-date")}</span>{/if}
  </header>
  <div class="tabs sub" role="tablist" aria-label={t("app-views-of", { group: t(group.label) })}>
    {#each group.views as v (v.id)}
      <button type="button" role="tab" aria-selected={v.id === current} class:on={v.id === current} onclick={() => (current = v.id)}>{t(v.label)}</button>
    {/each}
  </div>
  {#if stale}<div class="whyrow"><StaleWhy {blockers} /></div>{/if}

  <div class="view" data-view={current}>
    <!-- The view title, in the yard form (style guide section 7). The app has no scale to show, and the
         zoom state of a drawing sits with its canvas, so nothing goes beneath the name. -->
    <h3 class="title" data-testid="view-title">{currentLabel}</h3>
    <div class="content">
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
          <p class="note">{say(views.message, ids)}</p>
        {:else}
          <p class="note">{t("app-no-results-draw")}</p>
        {/if}
      </div>
    {:else if current === "speed-power"}
      <div class="charts">
        {#if message}
          <p class="note">{message}</p>
        {:else if curves.length === 0}
          <p class="note">{t("app-no-results-chart")}</p>
        {:else}
          {#each curves as c (c.name)}
            <SpeedPower name={say(c.name, ids)} limitingSpeed={c.limiting_speed_kn} blockage={c.blockage} points={c.points} {designSpeed} option={active} />
          {/each}
        {/if}
      </div>
    {:else if current === "stability"}
      {#if message}
        <p class="note">{message}</p>
      {:else if !results}
        <p class="note">{t("app-no-results-show")}</p>
      {:else}
        <!-- New results build a new chart, fitted. -->
        {#key JSON.stringify([active, stabilityOf(results)])}
          <Stability {ids} stability={stabilityOf(results)} marks={(results as { stability_marks?: never }).stability_marks} />
        {/key}
      {/if}
    {:else if current === "rules"}
      {#if message}
        <p class="note">{message}</p>
      {:else if !results}
        <p class="note">{t("app-no-results-show")}</p>
      {:else}
        <RuleChecks {ids} checks={(results as { checks?: { name: string; ok: boolean; message: string }[] }).checks ?? []} />
      {/if}
    {:else if current === "boq"}
      <div data-testid="placeholder-boq">
        <p class="note">{t("app-boq-later")}</p>
      </div>
    {:else if current === "capital"}
      {#if message}
        <p class="note">{message}</p>
      {:else if !results}
        <p class="note">{t("app-no-results-show")}</p>
      {:else}
        <CapitalBudget {ids} cost={(results as { cost?: never }).cost} phase={(results as { phase?: string }).phase} />
      {/if}
    {/if}
    </div>
  </div>
</div>

<style>
  .panel { height: 100%; display: flex; flex-direction: column; overflow: hidden; }
  header { display: flex; align-items: flex-end; gap: var(--sp-4); padding: var(--sp-2) var(--sp-4) 0; background: var(--surface-2); border-bottom: var(--lw-thin) solid var(--rule); }
  h2 { font-family: var(--font-serif); font-size: var(--fs-16); margin: 0; align-self: center; padding-bottom: var(--sp-2); }
  .tabs { display: flex; gap: var(--sp-1); }
  /* Level 1: 14 px, 600, the active tab in --ink with a 3 px rule under it. */
  .tabs button {
    background: none; border: 0; border-bottom: 3px solid transparent; border-radius: 0; cursor: pointer;
    padding: var(--sp-1) var(--sp-3) var(--sp-2); margin-bottom: -1px;
    font-size: var(--fs-14); font-weight: 600; color: var(--ink-2);
  }
  .tabs button:hover { background: none; color: var(--ink); }
  .tabs button.on { color: var(--ink); border-bottom-color: var(--ink); }
  /* Level 2: 13 px chips; the active chip has the soft accent ground and an accent border. */
  .tabs.sub { flex-wrap: wrap; padding: var(--sp-2) var(--sp-4); background: var(--surface); border-bottom: var(--lw-thin) solid var(--rule); }
  .tabs.sub button {
    border: var(--lw-thin) solid var(--rule-strong); border-radius: 12px; margin: 0;
    padding: 2px var(--sp-3); font-size: var(--fs-13); font-weight: 400; color: var(--ink);
  }
  .tabs.sub button:hover { background: var(--surface-2); }
  .tabs.sub button.on { background: var(--accent-soft); border-color: var(--accent); color: var(--ink); }
  .whyrow { padding: 0 var(--sp-4); background: var(--warn-soft); border-bottom: var(--lw-thin) solid var(--rule); }
  .view { flex: 1; min-height: 0; padding: 10px 14px; box-sizing: border-box; display: flex; flex-direction: column; }
  /* View title: hand lettering, capitals, slanted, letter-spaced, 16 px, double underline (two 1 px
     lines, 2 px apart). It takes its height from the view; the content below fits what is left. */
  .title {
    flex: none; align-self: flex-start; margin: 0 0 var(--sp-3); padding-bottom: 2px;
    font-family: var(--font-hand); font-size: var(--fs-16); font-weight: 400; font-style: oblique 10deg;
    text-transform: uppercase; letter-spacing: 0.1em; line-height: var(--lh-tight); color: var(--ink);
    border-bottom: var(--lw-thin) solid var(--ink); box-shadow: 0 3px 0 0 var(--ink);
  }
  .content { flex: 1; min-height: 0; }
  .slot { height: 100%; }
  .charts { height: 100%; display: flex; flex-direction: column; gap: 10px; }
  .note { color: var(--ink-3); }
  .stale { margin-left: auto; align-self: center; margin-bottom: var(--sp-1); background: var(--warn-soft); color: var(--warn); border: var(--lw-thin) solid var(--warn); font: var(--fs-11) var(--font-ui); padding: 1px 6px; border-radius: var(--radius); }
</style>
