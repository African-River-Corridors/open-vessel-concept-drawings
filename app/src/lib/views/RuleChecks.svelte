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
  let { checks }: { checks: Check[] } = $props();

  const COLUMNS: { label: string; cell: "result" | "name" | "message" }[] = [
    { label: "Result", cell: "result" },
    { label: "Check", cell: "name" },
    { label: "Message", cell: "message" },
  ];

  let ordered = $derived([...checks.filter((c) => !c.ok), ...checks.filter((c) => c.ok)]);
  let failing = $derived(checks.filter((c) => !c.ok).length);
</script>

<div class="rules" data-testid="rule-checks">
  {#if checks.length === 0}
    <p class="note">The engine reported no checks for this Option.</p>
  {:else}
    <p class="count" data-testid="rule-check-count">
      {failing} of {checks.length} check{checks.length === 1 ? "" : "s"} {failing === 1 ? "fails" : "fail"}
    </p>
    <div class="body">
      <table>
        <thead>
          <tr>
            {#each COLUMNS as col (col.cell)}<th>{col.label}</th>{/each}
          </tr>
        </thead>
        <tbody>
          {#each ordered as c, i (i)}
            <tr class:failed={!c.ok} data-testid="rule-check-row" data-ok={c.ok}>
              {#each COLUMNS as col (col.cell)}
                {#if col.cell === "result"}
                  <td><span class="tag {c.ok ? 'ok' : 'fail'}">{c.ok ? "PASS" : "FAIL"}</span></td>
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
  .note { color: var(--dim); }
  .body { flex: 1; min-height: 0; overflow: auto; border: 1px solid var(--line); }
  table { border-collapse: collapse; width: 100%; font-size: 12px; }
  thead th { position: sticky; top: 0; background: var(--cream); text-align: left; font-weight: 600; padding: 4px 10px; border-bottom: 1px solid var(--line); }
  td, tbody th { padding: 4px 10px; vertical-align: top; text-align: left; border-bottom: 1px solid #eee; }
  tbody th { font-weight: 600; white-space: nowrap; }
  tr.failed th { color: #7a1d12; }
  .message { color: var(--dim); }
  .tag { font-size: 10px; padding: 0 5px; border-radius: 3px; }
  .tag.ok { background: #e1f0e1; color: #245c24; }
  .tag.fail { background: #fbe4e1; color: #a02a1a; }
</style>
