<script lang="ts">
  import { focusInput } from "./focus";

  // Why the results are out of date: the inputs with a hard error, each a link to its field.
  let { blockers }: { blockers: { name: string; label: string }[] } = $props();
</script>

{#if blockers.length > 0}
  <p class="why" data-testid="stale-why">
    <span aria-hidden="true">▲</span> Not recalculated: fix the error under
    {#each blockers as b, i (b.name)}{i > 0 ? (i === blockers.length - 1 ? " and " : ", ") : " "}<a
        href={"#" + b.name}
        onclick={(e) => { e.preventDefault(); focusInput(b.name); }}>{b.label}</a>{/each}
  </p>
{/if}

<style>
  .why { margin: 4px 0; font-size: var(--fs-12); color: var(--warn); }
  a { color: inherit; text-decoration: underline; cursor: pointer; }
</style>
