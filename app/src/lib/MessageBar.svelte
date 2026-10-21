<script lang="ts" module>
  /** One message of the bar. `key` names it while it lasts; `dismiss` is given only where it is safe. */
  export interface BarMessage {
    key: string;
    severity: "error" | "warning" | "note";
    text: string;
    dismiss?: () => void;
  }
</script>

<script lang="ts">
  import { t } from "./i18n.svelte";

  // One line under the header for every message of the app (issue #436): the most severe first, a
  // count, previous and next, and a list that opens on demand. The messages come from the state each
  // time, so the bar never shows one that has gone.
  let { messages }: { messages: BarMessage[] } = $props();

  const ORDER = { error: 0, warning: 1, note: 2 } as const;
  const SHAPE = { error: "✕", warning: "▲", note: "○" } as const;

  let sorted = $derived([...messages].sort((a, b) => ORDER[a.severity] - ORDER[b.severity]));
  let index = $state(0);
  let open = $state(false);
  // A message that goes (fixed, dismissed, its Option deleted) moves the bar to the one now in its place.
  let at = $derived(Math.min(index, Math.max(sorted.length - 1, 0)));
  let current = $derived(sorted[at]);

  function step(by: number) {
    if (sorted.length > 0) index = (at + by + sorted.length) % sorted.length;
  }

  function key(e: KeyboardEvent) {
    if (e.key === "ArrowRight" || e.key === "ArrowDown") step(1);
    else if (e.key === "ArrowLeft" || e.key === "ArrowUp") step(-1);
    else if (e.key === "Escape") open = false;
    else return;
    e.preventDefault();
  }
</script>

{#if current}
  <div class="messages" data-testid="message-bar">
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="bar {current.severity}"
      role="group"
      aria-label={t("app-msg-bar")}
      onkeydown={key}
    >
      <span class="shape" aria-hidden="true">{SHAPE[current.severity]}</span>
      <span class="text" role={current.severity === "error" ? "alert" : "status"} title={current.text} data-testid="message-text">{current.text}</span>
      {#if sorted.length > 1}
        <span class="count" data-testid="message-count">{t("app-msg-count", { index: at + 1, count: sorted.length })}</span>
        <button type="button" class="nav" aria-label={t("app-msg-previous")} onclick={() => step(-1)}>&lsaquo;</button>
        <button type="button" class="nav" aria-label={t("app-msg-next")} onclick={() => step(1)}>&rsaquo;</button>
      {/if}
      <button type="button" aria-expanded={open} onclick={() => (open = !open)}>{t(open ? "app-msg-show-less" : sorted.length > 1 ? "app-msg-show-all" : "app-msg-show-more")}</button>
      {#if current.dismiss}<button type="button" onclick={current.dismiss}>{t("app-dismiss")}</button>{/if}
    </div>
    {#if open}
      <ul class="list" data-testid="message-list">
        {#each sorted as m, i (m.key)}
          <li class={m.severity} class:current={i === at}>
            <span class="shape" aria-hidden="true">{SHAPE[m.severity]}</span>
            <span class="full">{m.text}</span>
            {#if m.dismiss}<button type="button" onclick={m.dismiss}>{t("app-dismiss")}</button>{/if}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

<style>
  .messages { border-bottom: var(--lw-thin) solid var(--rule); font-size: var(--fs-13); }
  .bar { display: flex; align-items: center; gap: var(--sp-2); padding: 2px var(--sp-3); min-height: 24px; }
  .text { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .count { flex: none; font-variant-numeric: tabular-nums; }
  .bar button, .list button { flex: none; font-size: var(--fs-13); padding: 0 var(--sp-2); }
  .nav { min-width: 24px; }
  .shape { flex: none; }
  .error { background: var(--fail-soft); color: var(--fail); }
  .warning { background: var(--warn-soft); color: var(--warn); }
  .note { background: var(--surface-2); color: var(--ink); }
  .list { list-style: none; margin: 0; padding: 0; max-height: 40vh; overflow: auto; border-top: var(--lw-thin) solid var(--rule); }
  .list li { display: flex; align-items: baseline; gap: var(--sp-2); padding: 2px var(--sp-3); border-bottom: var(--lw-thin) solid var(--rule); }
  .list li.current .full { font-weight: 600; }
  .full { flex: 1; min-width: 0; overflow-wrap: anywhere; }
  @media (max-width: 600px) {
    .bar { padding: 2px var(--sp-2); gap: var(--sp-1); }
  }
</style>
