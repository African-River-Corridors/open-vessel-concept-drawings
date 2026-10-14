<script lang="ts">
  import { onMount, tick } from "svelte";
  import type { Evaluation, OptionEvaluation } from "./api";
  import SpeedPower from "./views/SpeedPower.svelte";

  // The views, stacked in one scrolling panel. A dropdown at the top right jumps to a view and
  // follows the scroll. The hull views wait for the Drawing Format Standard (ADR 0009): their slots
  // draw nothing and say why.
  let { evaluation, active, lastGood }: {
    evaluation: Evaluation | null;
    active: string;
    lastGood: Record<string, OptionEvaluation>;
  } = $props();

  const VIEWS = [
    { id: "side", label: "Side elevation" },
    { id: "plan", label: "Plan" },
    { id: "midship", label: "Midship section" },
    { id: "speed-power", label: "Speed and power" },
  ];
  const HULL = ["side", "plan", "midship"];

  let scroller: HTMLDivElement;
  let current = $state(VIEWS[0].id);

  let outcome = $derived(evaluation?.options.find((o) => o.name === active)?.outcome);
  let results = $derived.by(() => {
    if (outcome?.status === "results") return outcome.results;
    const prior = lastGood[active]?.outcome;
    return outcome?.status === "not_run" && prior?.status === "results" ? prior.results : null;
  });
  let stale = $derived(outcome?.status === "not_run");
  let message = $derived(outcome && (outcome.status === "not_available" || outcome.status === "failed") ? outcome.message : null);

  interface Curve {
    name: string;
    limiting_speed_kn: number | null;
    blockage: number;
    points: { kn: number; kw: number; fuel_lph: number; squat_m: number; squat_limited: boolean }[];
  }
  let curves = $derived(((results as { curves?: Curve[] } | null)?.curves ?? []));

  function sectionTop(id: string): number {
    const el = scroller.querySelector<HTMLElement>(`[data-view="${id}"]`)!;
    // The panel is the offset parent, so this is the distance from the top of its content.
    return el.offsetTop;
  }

  function jump(id: string) {
    current = id;
    scroller.scrollTo({ top: sectionTop(id) - 40 });
  }

  /** The view in sight is the last one whose top has passed the top of the panel. */
  function follow() {
    let found = VIEWS[0].id;
    for (const v of VIEWS) if (sectionTop(v.id) - 60 <= scroller.scrollTop) found = v.id;
    // At the very bottom the last view counts even if it is short.
    if (scroller.scrollTop + scroller.clientHeight >= scroller.scrollHeight - 2) found = VIEWS[VIEWS.length - 1].id;
    current = found;
  }

  onMount(() => {
    tick().then(follow);
  });
</script>

<div class="panel" bind:this={scroller} onscroll={follow}>
  <header>
    <h2>Views</h2>
    <select aria-label="Jump to view" value={current} onchange={(e) => jump(e.currentTarget.value)}>
      {#each VIEWS as v (v.id)}
        <option value={v.id}>{v.label}</option>
      {/each}
    </select>
  </header>

  {#each VIEWS as v (v.id)}
    <section data-view={v.id} class:hull={HULL.includes(v.id)}>
      <h3>{v.label}</h3>
      {#if HULL.includes(v.id)}
        <div class="slot" data-testid="slot-{v.id}">
          Waits for the Drawing Format Standard. This view is drawn to that standard once it exists.
        </div>
      {:else if v.id === "speed-power"}
        {#if stale}<span class="stale">out of date</span>{/if}
        {#if message}
          <p class="note">{message}</p>
        {:else if curves.length === 0}
          <p class="note">No results to chart.</p>
        {:else}
          {#each curves as c (c.name)}
            <SpeedPower name={c.name} limitingSpeed={c.limiting_speed_kn} blockage={c.blockage} points={c.points} />
          {/each}
        {/if}
      {/if}
    </section>
  {/each}
</div>

<style>
  .panel { height: 100%; overflow-y: auto; position: relative; }
  header { position: sticky; top: 0; z-index: 2; display: flex; align-items: center; justify-content: space-between; padding: 6px 14px; background: rgba(255, 255, 255, 0.95); border-bottom: 1px solid var(--line); }
  h2 { font-family: var(--serif); font-size: 16px; margin: 0; }
  h3 { font-family: var(--serif); color: var(--navy); font-size: 15px; margin: 0 0 6px; }
  select { padding: 3px 6px; font: inherit; }
  section { padding: 14px; border-bottom: 1px solid var(--line); min-height: 180px; }
  .slot { border: 1px dashed var(--line); color: var(--dim); padding: 40px 12px; text-align: center; font-size: 13px; }
  .note { color: var(--dim); }
  .stale { background: #fdf1d3; color: #6b4a00; font: 11px var(--sans); padding: 1px 6px; border-radius: 3px; }
</style>
