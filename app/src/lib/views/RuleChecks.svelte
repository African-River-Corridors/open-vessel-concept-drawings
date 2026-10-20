<script lang="ts">
  // Every check the engine reports for the Option, as it gives them: result, name and message.
  // Failures come first, then passes, each group in the engine's order. The app computes nothing.
  // Value, limit, margin and citation columns come with the rules ontology (#176): add a column to
  // COLUMNS then. Only the table body scrolls, when the list is longer than the panel.
  interface Check {
    name: string;
    ok: boolean;
    message: string;
  }
  import { t } from "../i18n.svelte";

  let { checks }: { checks: Check[] } = $props();

  const COLUMNS: { label: string; cell: "result" | "name" | "message" }[] = [
    { label: "app-rules-result", cell: "result" },
    { label: "app-rules-check", cell: "name" },
    { label: "app-rules-message", cell: "message" },
  ];

  let ordered = $derived([...checks.filter((c) => !c.ok), ...checks.filter((c) => c.ok)]);
  let failing = $derived(checks.filter((c) => !c.ok).length);
</script>

<div class="rules" data-testid="rule-checks">
  {#if checks.length === 0}
    <p class="note">{t("app-rules-none")}</p>
  {:else}
    <p class="count" data-testid="rule-check-count">
      {t("app-rules-count", { failing, total: checks.length })}
    </p>
    <div class="body">
      <table>
        <thead>
          <tr>
            {#each COLUMNS as col (col.cell)}<th>{t(col.label)}</th>{/each}
          </tr>
        </thead>
        <tbody>
          {#each ordered as c, i (i)}
            <tr class:failed={!c.ok} data-testid="rule-check-row" data-ok={c.ok}>
              {#each COLUMNS as col (col.cell)}
                {#if col.cell === "result"}
                  <td><span class="tag {c.ok ? 'ok' : 'fail'}"><span aria-hidden="true">{c.ok ? "✓" : "✕"}</span> {t(c.ok ? "app-pass" : "app-fail")}</span></td>
                {:else if col.cell === "name"}
                  <th scope="row">{c.name}</th>
                {:else}
                  <td class="message">{c.message}</td>
                {/if}
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .rules { height: 100%; display: flex; flex-direction: column; }
  .count { margin: 0 0 6px; font-weight: 600; }
  .note { color: var(--ink-3); }
  .body { flex: 1; min-height: 0; overflow: auto; border: var(--lw-thin) solid var(--rule); }
  table { border-collapse: collapse; width: 100%; font-size: var(--fs-13); }
  thead th { position: sticky; top: 0; background: var(--surface-2); color: var(--ink); text-align: left; font-weight: 600; padding: var(--sp-1) 10px; border-bottom: var(--lw-thick) solid var(--ink); }
  td, tbody th { padding: var(--sp-1) 10px; vertical-align: top; text-align: left; border-bottom: var(--lw-thin) solid var(--rule); }
  tbody th { font-weight: 600; white-space: nowrap; }
  tr.failed th { color: var(--fail); }
  .message { color: var(--ink-3); }
  .tag { font: 600 var(--fs-11) var(--font-ui); padding: 1px 6px; border-radius: var(--radius); white-space: nowrap; }
  .tag.ok { background: var(--pass-soft); color: var(--pass); }
  .tag.fail { background: var(--fail-soft); color: var(--fail); }
</style>
