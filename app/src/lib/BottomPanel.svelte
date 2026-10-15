<script lang="ts">
  import type { Evaluation, OptionEvaluation } from "./api";
  import StaleWhy from "./StaleWhy.svelte";
  import { MAIN, TANKS, exact, hasTanks, show, valueAt, type Metric } from "./metrics";

  // Outputs of the active Option, its checks and warnings, and a compare table of every Option.
  // The numbers are read from the engine's results object. While the active Option has a hard
  // error its last good results stay on screen, marked "out of date".
  let { evaluation, active, lastGood, blockers, onActivate }: {
    evaluation: Evaluation | null;
    active: string;
    lastGood: Record<string, OptionEvaluation>;
    blockers: { name: string; label: string }[];
    onActivate: (name: string) => void;
  } = $props();

  type Column = {
    name: string;
    stale: boolean;
    results: Record<string, unknown> | null;
    message: string | null;
    errors: string[];
  };

  /** What to show for one Option: its fresh results, else its last good ones (out of date). */
  function column(o: OptionEvaluation): Column {
    const errors = o.messages.filter((m) => m.severity === "error").map((m) => m.text);
    if (o.outcome.status === "results") {
      return { name: o.name, stale: false, results: o.outcome.results, message: null, errors };
    }
    if (o.outcome.status === "not_run") {
      const prior = lastGood[o.name]?.outcome;
      return {
        name: o.name,
        stale: true,
        results: prior?.status === "results" ? prior.results : null,
        message: null,
        errors,
      };
    }
    return { name: o.name, stale: false, results: null, message: o.outcome.message, errors };
  }

  let columns = $derived((evaluation?.options ?? []).map(column));
  let current = $derived(columns.find((c) => c.name === active));
  let activeEval = $derived(evaluation?.options.find((o) => o.name === active));
  let rows = $derived<Metric[]>([...MAIN, ...(columns.some((c) => hasTanks(c.results)) ? TANKS : [])]);

  let checks = $derived.by(() => {
    const list = (valueAt(current?.results, ["checks"]) as { name: string; ok: boolean; message: string }[] | undefined) ?? [];
    // Failures first, otherwise in the engine's order.
    return [...list.filter((c) => !c.ok), ...list.filter((c) => c.ok)];
  });
  let warnings = $derived((valueAt(current?.results, ["warnings"]) as string[] | undefined) ?? []);
  let softWarnings = $derived((activeEval?.messages ?? []).filter((m) => m.severity === "warning"));

  function differs(m: Metric, c: Column): boolean {
    if (!current || c.name === current.name || !c.results || !current.results) return false;
    return JSON.stringify(valueAt(c.results, m.path)) !== JSON.stringify(valueAt(current.results, m.path));
  }
</script>

<div class="panel">
  {#if !evaluation}
    <p class="dim">Evaluating...</p>
  {:else}
    <div class="cols">
      <section>
        <h2>
          Outputs
          {#if current?.stale}<span class="stale">out of date</span>{/if}
        </h2>
        {#if current?.stale}<StaleWhy {blockers} />{/if}
        {#each current?.errors ?? [] as e}
          <div class="err">{e}</div>
        {/each}
        {#if current?.message}
          <p class="note" data-testid="engine-message">{current.message}</p>
        {:else if current?.results}
          <table data-testid="outputs">
            <tbody>
              {#each rows as m (m.label)}
                {@const v = valueAt(current.results, m.path)}
                {#if !(m.path[0] === "arrangement" && !hasTanks(current.results))}
                  <tr>
                    <th>{m.label}</th>
                    <td class="num" class:text={m.quantity === "text"} title={exact(m, v)}>{show(m, v)}{#if v !== null && v !== undefined && m.unit}<span class="unit">&nbsp;{m.unit}</span>{/if}</td>
                  </tr>
                {/if}
              {/each}
              {#if activeEval?.vessel_class?.reference_dwt_t}
                {@const r = activeEval.vessel_class.reference_dwt_t}
                <tr data-testid="class-reference-dwt">
                  <th title="The class's published figure, for comparison. It is never an input.">Class reference deadweight ({activeEval.vessel_class.id})</th>
                  <td class="num text">{r[0] === r[1] ? r[0] : `${r[0]} to ${r[1]}`}<span class="unit">&nbsp;t</span></td>
                </tr>
              {/if}
            </tbody>
          </table>
        {/if}
      </section>

      <section>
        <h2>Checks</h2>
        {#if checks.length === 0 && !current?.results}
          <p class="dim">No checks without results.</p>
        {/if}
        <ul class="checks" data-testid="checks">
          {#each checks as c (c.name)}
            <li class={c.ok ? "ok" : "fail"}>
              <span class="tag">{c.ok ? "PASS" : "FAIL"}</span>
              <strong>{c.name}</strong>
              {#if c.message && c.message !== "OK"}<span class="msg">{c.message}</span>{/if}
            </li>
          {/each}
        </ul>
        {#if warnings.length > 0 || softWarnings.length > 0}
          <h2>Warnings</h2>
          <ul class="warnings" data-testid="warnings">
            {#each softWarnings as w}<li>{w.text}</li>{/each}
            {#each warnings as w}<li>{w}</li>{/each}
          </ul>
        {/if}
      </section>

      <section class="compare">
        <h2>Compare</h2>
        <table data-testid="compare">
          <thead>
            <tr>
              <th></th>
              {#each columns as c (c.name)}
                <th class:active={c.name === active}>
                  <button class="col" onclick={() => onActivate(c.name)} title="Make this the active Option">{c.name}</button>
                  {#if c.stale}<span class="stale">out of date</span>{/if}
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each rows as m (m.label)}
              <tr>
                <th>{m.label}{#if m.unit}<span class="unit">&nbsp;({m.unit})</span>{/if}</th>
                {#each columns as c (c.name)}
                  {@const v = valueAt(c.results, m.path)}
                  <td class="num" class:text={m.quantity === "text"} class:differs={differs(m, c)} class:active={c.name === active} title={c.results ? exact(m, v) : (c.message ?? "")}>
                    {c.results ? show(m, v) : c.message ? "" : "–"}
                  </td>
                {/each}
              </tr>
            {/each}
            {#if columns.some((c) => c.message)}
              <tr>
                <th>Status</th>
                {#each columns as c (c.name)}<td class="note">{c.message ?? ""}</td>{/each}
              </tr>
            {/if}
          </tbody>
        </table>
      </section>
    </div>
  {/if}
</div>

<style>
  .panel { padding: 8px 14px; }
  .cols { display: grid; grid-template-columns: minmax(220px, 1fr) minmax(240px, 1.2fr) minmax(300px, 2fr); gap: 20px; align-items: start; }
  h2 { font-family: var(--serif); font-size: 14px; margin: 0 0 6px; }
  .dim { color: var(--dim); }
  table { border-collapse: collapse; font-size: 12px; }
  th { text-align: left; font-weight: normal; color: var(--dim); padding: 2px 10px 2px 0; white-space: nowrap; }
  td { padding: 2px 8px; }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  td.text { text-align: left; white-space: normal; max-width: 170px; font-size: 11px; }
  .unit { color: var(--dim); }
  thead th { color: var(--ink); text-align: right; }
  thead th.active { border-bottom: 2px solid var(--gold); }
  td.active { background: #f7f3e6; }
  td.differs { background: #fdf1d3; font-weight: 600; }
  .col { background: none; border: none; font: inherit; font-weight: 600; cursor: pointer; color: var(--navy); padding: 0; text-decoration: underline; }
  .stale { background: #fdf1d3; color: #6b4a00; font: 11px var(--sans); padding: 1px 6px; border-radius: 3px; margin-left: 6px; }
  .err { color: #a02a1a; font-size: 12px; margin-bottom: 4px; }
  .note { font-size: 12px; color: var(--ink); }
  .checks, .warnings { list-style: none; margin: 0 0 10px; padding: 0; font-size: 12px; }
  .checks li, .warnings li { margin: 3px 0; }
  .tag { font-size: 10px; padding: 0 5px; border-radius: 3px; margin-right: 4px; }
  .ok .tag { background: #e1f0e1; color: #245c24; }
  .fail .tag { background: #fbe4e1; color: #a02a1a; }
  .fail { color: #7a1d12; }
  .msg { color: var(--dim); margin-left: 4px; }
  .fail .msg { color: #7a1d12; }
</style>
