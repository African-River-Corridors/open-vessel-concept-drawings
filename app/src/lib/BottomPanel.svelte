<script lang="ts">
  import type { OptionEvaluation } from "./api";

  // Placeholder for ticket #75. Shows what the engine said about the active Option, and keeps the
  // last good results on screen, marked "out of date", while an input has a hard error.
  let { evaluation, stale, lastGood }: {
    evaluation: OptionEvaluation | undefined;
    stale: boolean;
    lastGood: OptionEvaluation | undefined;
  } = $props();

  let shown = $derived(stale ? lastGood : evaluation);
  let outcome = $derived(shown?.outcome);
</script>

<div class="panel">
  <h2>Outputs {#if stale}<span class="stale">out of date</span>{/if}</h2>
  {#each (evaluation?.messages ?? []).filter((m) => m.severity === "error") as m}
    <div class="msg error">{m.text}</div>
  {/each}
  {#if outcome?.status === "results"}
    <pre data-testid="results">{JSON.stringify(outcome.results, null, 2)}</pre>
  {:else if outcome && outcome.status !== "not_run"}
    <p>{outcome.message}</p>
  {:else if !evaluation}
    <p>Evaluating...</p>
  {/if}
</div>

<style>
  .panel { padding: 10px 14px; }
  h2 { font-family: var(--serif); font-size: 16px; margin: 0 0 8px; }
  .stale { background: #fdf1d3; color: #6b4a00; font: 12px var(--sans); padding: 1px 6px; border-radius: 3px; margin-left: 8px; }
  .msg.error { color: #a02a1a; font-size: 13px; }
  pre { font-size: 12px; margin: 0; white-space: pre-wrap; }
</style>
