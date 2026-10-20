<script lang="ts">
  import { focusInput } from "./focus";
  import { listParts, parts } from "./i18n.svelte";

  // Why the results are out of date: the inputs with a hard error, each a link to its field.
  let { blockers }: { blockers: { name: string; label: string }[] } = $props();
</script>

{#if blockers.length > 0}
  <p class="why" data-testid="stale-why">
    <span aria-hidden="true">▲</span>
    {#each parts("app-stale-why", ["inputs"]) as p}{#if typeof p === "string"}{p}{:else}{#each listParts(blockers.map((b) => b.label)) as q}{#if typeof q === "string"}{q}{:else}{@const b = blockers[Number(q.arg)]}<a
        href={"#" + b.name}
        onclick={(e) => { e.preventDefault(); focusInput(b.name); }}>{b.label}</a>{/if}{/each}{/if}{/each}
  </p>
{/if}

<style>
  .why { margin: 4px 0; font-size: var(--fs-12); color: var(--warn); }
  a { color: inherit; text-decoration: underline; cursor: pointer; }
</style>
