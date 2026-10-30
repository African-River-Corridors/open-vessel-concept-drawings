<script lang="ts">
  import type { Evaluation, OptionEvaluation } from "./api";
  import StaleWhy from "./StaleWhy.svelte";
  import { HYDROSTATICS, MAIN, POWERING, TANKS, exact, hasHydrostatics, hasPowering, hasTanks, show, textIds, valueAt, type Metric } from "./metrics";
  import { checkName, num, say, t } from "./i18n.svelte";
  import { CARGO_ONLY_CHECKS, FULL_LOAD_ROWS, consumablesInLoad } from "./loadBasis";

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
    /** The results hash of the results shown (issue #666), or null with none. */
    hash: string | null;
    message: string | null;
    errors: string[];
  };

  /** What to show for one Option: its fresh results, else its last good ones (out of date). */
  function column(o: OptionEvaluation): Column {
    const errors = o.messages.filter((m) => m.severity === "error").map((m) => say(m.text, null, m.msg));
    if (o.outcome.status === "results") {
      return { name: o.name, stale: false, results: o.outcome.results, hash: o.outcome.results_hash, message: null, errors };
    }
    if (o.outcome.status === "not_run") {
      const prior = lastGood[o.name]?.outcome;
      return {
        name: o.name,
        stale: true,
        results: prior?.status === "results" ? prior.results : null,
        hash: prior?.status === "results" ? prior.results_hash : null,
        message: null,
        errors,
      };
    }
    return { name: o.name, stale: false, results: null, hash: null, message: say(o.outcome.message, null, o.outcome.msg), errors };
  }

  let columns = $derived((evaluation?.options ?? []).map(column));
  let current = $derived(columns.find((c) => c.name === active));
  let activeEval = $derived(evaluation?.options.find((o) => o.name === active));
  let rows = $derived<Metric[]>([
    ...MAIN,
    ...(columns.some((c) => hasTanks(c.results)) ? TANKS : []),
    ...(columns.some((c) => hasPowering(c.results)) ? POWERING : []),
    ...(columns.some((c) => hasHydrostatics(c.results)) ? HYDROSTATICS : []),
  ]);

  let checks = $derived.by(() => {
    const list =
      (valueAt(current?.results, ["checks"]) as { name: string; ok: boolean; message: string; severity?: string }[] | undefined) ?? [];
    // Issue #419: a failed check of severity "warning" does not gate the result, so it reads as a
    // warning, not a failure. A check with no severity is an error, as in the CLI summary.
    const warned = (c: { ok: boolean; severity?: string }) => !c.ok && c.severity === "warning";
    // Failures that gate first, then failed warnings, then passes; otherwise in the engine's order.
    return [...list.filter((c) => !c.ok && !warned(c)), ...list.filter(warned), ...list.filter((c) => c.ok)].map((c) => ({
      ...c,
      tone: c.ok ? "ok" : warned(c) ? "warn" : "fail",
    }));
  });
  const TAGS: Record<string, { mark: string; label: string }> = {
    ok: { mark: "✓", label: "app-pass" },
    warn: { mark: "!", label: "app-rules-warn" },
    fail: { mark: "✕", label: "app-fail" },
  };
  let warnings = $derived((valueAt(current?.results, ["warnings"]) as string[] | undefined) ?? []);
  // Issue #675: the gas tank arrangements the search tried, and why each lost.
  type Tried = {
    depth_m: number;
    n_wide: number;
    n_long: number;
    tank_od_m: number;
    side_clearance_m: number;
    wing_tank_width_m: number;
    cargo_volume_m3: number;
    outcome: "chosen" | "less_cargo" | "fails" | "not_built";
    reasons: { name: string; message: string }[];
  };
  const OUTCOMES: Record<Tried["outcome"], string> = {
    chosen: "app-tried-chosen",
    less_cargo: "app-tried-less-cargo",
    fails: "app-tried-fails",
    not_built: "app-tried-not-built",
  };
  let tried = $derived((valueAt(current?.results, ["fit_tried"]) as Tried[] | undefined) ?? []);
  let triedDepths = $derived(new Set(tried.map((a) => a.depth_m)).size > 1);
  let ids = $derived(textIds(current?.results));
  let softWarnings = $derived((activeEval?.messages ?? []).filter((m) => m.severity === "warning"));
  // Issue #668: the full load of the Outputs carries consumables, the freeboard checks do not.
  let consumables = $derived(consumablesInLoad(current?.results));

  function differs(m: Metric, c: Column): boolean {
    if (!current || c.name === current.name || !c.results || !current.results) return false;
    return JSON.stringify(valueAt(c.results, m.path)) !== JSON.stringify(valueAt(current.results, m.path));
  }
</script>

<div class="panel">
  {#if !evaluation}
    <p class="dim">{t("app-evaluating")}</p>
  {:else}
    <div class="cols">
      <section>
        <h2>
          {t("app-outputs")}
          {#if current?.stale}<span class="stale">{t("app-out-of-date")}</span>{/if}
        </h2>
        {#if current?.stale}<StaleWhy {blockers} />{/if}
        {#each current?.errors ?? [] as e}
          <div class="err"><span aria-hidden="true">✕</span> {e}</div>
        {/each}
        {#if current?.message}
          <p class="note" data-testid="engine-message">{current.message}</p>
        {:else if current?.results}
          <table data-testid="outputs">
            <tbody>
              {#each rows as m (m.label)}
                {@const v = valueAt(current.results, m.path)}
                {#if !(m.path[0] === "arrangement" && !hasTanks(current.results)) && !(m.path[0] === "powering" && !hasPowering(current.results)) && !(m.path[0] === "hydrostatics" && !hasHydrostatics(current.results))}
                  <tr>
                    <th>{t(m.label)}{#if consumables !== null && FULL_LOAD_ROWS.includes(m.label)}<span class="basis" data-testid="full-load-mark">&nbsp;†</span>{/if}</th>
                    <td class="num" class:text={m.quantity === "text"} title={exact(m, v)}><span class:old={current.stale}>{show(m, v, ids)}</span>{#if v !== null && v !== undefined && m.unit}<span class="unit">&nbsp;{m.unit}</span>{/if}</td>
                  </tr>
                {/if}
              {/each}
              {#if activeEval?.vessel_class?.reference_dwt_t}
                {@const r = activeEval.vessel_class.reference_dwt_t}
                <tr data-testid="class-reference-dwt">
                  <th title={t("app-class-reference-dwt-title")}>{t("app-class-reference-dwt", { class: activeEval.vessel_class.id })}</th>
                  <td class="num text">{r[0] === r[1] ? r[0] : t("app-range-to", { min: String(r[0]), max: String(r[1]) })}<span class="unit">&nbsp;t</span></td>
                </tr>
              {/if}
            </tbody>
          </table>
          {#if consumables !== null}
            <p class="note basis-note" data-testid="load-basis">† {t("app-load-basis-note", { consumables: num(consumables, 1) })}</p>
          {/if}
        {/if}
      </section>

      <section>
        <h2>{t("app-checks")}</h2>
        {#if checks.length === 0 && !current?.results}
          <p class="dim">{t("app-no-checks")}</p>
        {/if}
        <ul class="checks" data-testid="checks">
          {#each checks as c (c.name)}
            <li class={c.tone} data-testid="check" data-tone={c.tone}>
              <span class="tag"><span aria-hidden="true">{TAGS[c.tone].mark}</span> {t(TAGS[c.tone].label)}</span>
              <strong>{checkName(c.name, ids)}</strong>
              {#if c.message && c.message !== "OK"}<span class="msg">{say(c.message, ids)}</span>{/if}
              {#if consumables !== null && CARGO_ONLY_CHECKS.includes(c.name)}<span class="msg basis" data-testid="cargo-only-basis">({t("app-check-basis-cargo-only")})</span>{/if}
            </li>
          {/each}
        </ul>
        {#if tried.length > 0}
          <h2>{t("app-tried")}</h2>
          <table class="tried" data-testid="fit-tried">
            <thead>
              <tr>
                {#if triedDepths}<th>{t("app-tried-depth")}</th>{/if}
                <th>{t("app-tried-arrangement")}</th>
                <th class="num">{t("app-tried-tank-od")}</th>
                <th class="num">{t("app-tried-side-space")}</th>
                <th class="num">{t("app-tried-wing-tank")}</th>
                <th class="num">{t("app-tried-cargo-volume")}</th>
                <th class="outcome">{t("app-tried-outcome")}</th>
              </tr>
            </thead>
            <tbody>
              {#each tried as a}
                <tr class={a.outcome} data-testid="fit-tried-row" data-outcome={a.outcome}>
                  {#if triedDepths}<td class="num">{num(a.depth_m, 2)}</td>{/if}
                  <td>{a.n_wide} × {a.n_long}</td>
                  <td class="num">{a.outcome === "not_built" ? "–" : num(a.tank_od_m, 2)}</td>
                  <td class="num">{a.outcome === "not_built" ? "–" : num(a.side_clearance_m, 2)}</td>
                  <td class="num">{a.outcome === "not_built" ? "–" : num(a.wing_tank_width_m, 2)}</td>
                  <td class="num">{a.outcome === "not_built" ? "–" : num(a.cargo_volume_m3, 0)}</td>
                  <td class="outcome" title={a.outcome === "less_cargo" ? t("app-tried-less-cargo-title") : undefined}>{t(OUTCOMES[a.outcome])}</td>
                </tr>
                {#each a.reasons as r}
                  <tr class="reason {a.outcome}"><td colspan={triedDepths ? 7 : 6}>{say(r.message, ids)}</td></tr>
                {/each}
              {/each}
            </tbody>
          </table>
        {/if}
        {#if warnings.length > 0 || softWarnings.length > 0}
          <h2>{t("app-warnings")}</h2>
          <ul class="warnings" data-testid="warnings">
            {#each softWarnings as w}<li><span aria-hidden="true">▲</span> {say(w.text, null, w.msg)}</li>{/each}
            {#each warnings as w}<li><span aria-hidden="true">▲</span> {say(w, ids)}</li>{/each}
          </ul>
        {/if}
      </section>

      <section class="compare">
        <h2>{t("app-compare")}</h2>
        <table data-testid="compare">
          <thead>
            <tr>
              <th></th>
              {#each columns as c (c.name)}
                <th class:active={c.name === active}>
                  <button class="col" onclick={() => onActivate(c.name)} title={t("app-make-active")}>{c.name}</button>
                  {#if c.stale}<span class="stale">{t("app-out-of-date")}</span>{/if}
                </th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each rows as m (m.label)}
              <tr>
                <th>{t(m.label)}{#if m.unit}<span class="unit">&nbsp;({m.unit})</span>{/if}</th>
                {#each columns as c (c.name)}
                  {@const v = valueAt(c.results, m.path)}
                  <td class="num" class:text={m.quantity === "text"} class:differs={differs(m, c)} class:active={c.name === active} class:old={c.stale} title={c.results ? exact(m, v) : (c.message ?? "")}>
                    {c.results ? show(m, v, textIds(c.results)) : c.message ? "" : "–"}
                  </td>
                {/each}
              </tr>
            {/each}
            <!-- The results hash (issue #666): its first 12 characters, the full value on hover, so a
                 Design entered here can be checked against `ovcd run --json` of a published example. -->
            <tr data-testid="compare-hash">
              <th>{t("app-results-hash")}</th>
              {#each columns as c (c.name)}
                <td class="num text hash" class:active={c.name === active} class:old={c.stale} title={c.hash ?? ""}>{c.hash ? c.hash.slice(0, 12) : "–"}</td>
              {/each}
            </tr>
            {#if columns.some((c) => c.message)}
              <tr>
                <th>{t("app-status")}</th>
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
  .panel { padding: var(--sp-2) var(--sp-3); font-size: var(--fs-13); line-height: var(--lh-ui); }
  /* Issue #669: the columns follow the panel's own width, not the window's. Outputs is as wide as its
     table. Below the width the three need, Compare goes under Outputs and Checks; narrower still, one
     column. A table wider than its column scrolls inside it, so no column runs over the next. */
  .panel { container-type: inline-size; }
  .cols { display: grid; grid-template-columns: auto minmax(240px, 1.2fr) minmax(300px, 2fr); gap: var(--sp-5); align-items: start; }
  .cols > section { min-width: 0; overflow-x: auto; }
  @container (max-width: 960px) {
    .cols { grid-template-columns: auto minmax(0, 1fr); }
    .cols > .compare { grid-column: 1 / -1; }
  }
  @container (max-width: 560px) {
    .cols { grid-template-columns: minmax(0, 1fr); }
  }
  h2 { font: 600 var(--fs-12) var(--font-ui); text-transform: uppercase; letter-spacing: 0.08em; color: var(--ink-2); background: var(--surface-2); border-bottom: var(--lw-thin) solid var(--rule); margin: 0 0 var(--sp-2); padding: var(--sp-1) var(--sp-2); }
  .dim { color: var(--ink-3); }
  table { border-collapse: collapse; font-size: var(--fs-13); }
  th { text-align: left; font-weight: 400; color: var(--ink-2); padding: 2px 10px 2px 0; white-space: nowrap; }
  td { padding: 2px var(--sp-2); }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  td.text { text-align: left; white-space: normal; max-width: 170px; font-size: var(--fs-12); }
  .old { color: var(--stale); text-decoration: line-through; }
  .unit { color: var(--ink-3); }
  [data-testid="outputs"] tr { border-bottom: var(--lw-thin) solid var(--rule); }
  [data-testid="outputs"] th { padding-left: var(--sp-1); }
  thead th { color: var(--ink); text-align: right; background: var(--surface-2); border-bottom: var(--lw-thick) solid var(--ink); padding: var(--sp-1) var(--sp-2); font-weight: 600; }
  tbody tr { border-bottom: var(--lw-thin) solid var(--rule); }
  .hash { font-family: var(--font-mono); }
  th.active, td.active { background: var(--accent-soft); }
  td.differs { background: var(--warn-soft); color: var(--ink); font-weight: 700; }
  .col { background: none; border: none; font: inherit; font-weight: 600; cursor: pointer; color: var(--accent); padding: 0; text-decoration: underline; }
  .stale { background: var(--warn-soft); color: var(--warn); border: var(--lw-thin) solid var(--warn); font: var(--fs-11) var(--font-ui); padding: 1px 6px; border-radius: var(--radius); margin-left: 6px; text-transform: none; letter-spacing: 0; }
  .err { color: var(--fail); background: var(--fail-soft); font-size: var(--fs-12); margin-bottom: var(--sp-1); padding: var(--sp-1) var(--sp-2); }
  .note { font-size: var(--fs-12); color: var(--ink); }
  .checks, .warnings { list-style: none; margin: 0 0 var(--sp-3); padding: 0; font-size: var(--fs-12); }
  .checks li { margin: 0; padding: 3px 0; border-bottom: var(--lw-thin) solid var(--rule); }
  .warnings li { margin: 3px 0; color: var(--warn); }
  .tag { font: 600 var(--fs-11) var(--font-ui); padding: 1px 6px; border-radius: var(--radius); margin-right: var(--sp-1); white-space: nowrap; }
  .ok .tag { background: var(--pass-soft); color: var(--pass); }
  .fail .tag { background: var(--fail-soft); color: var(--fail); }
  .fail { color: var(--fail); }
  .warn .tag { background: var(--warn-soft); color: var(--warn); }
  .msg { color: var(--ink-3); margin-left: var(--sp-1); }
  .tried { width: 100%; margin-bottom: var(--sp-3); font-size: var(--fs-12); }
  .tried th { font-weight: 600; white-space: normal; }
  .tried td.outcome, .tried th.outcome { white-space: nowrap; }
  .tried tr.chosen td { color: var(--pass); font-weight: 600; }
  .tried tr.fails td.outcome, .tried tr.not_built td.outcome { color: var(--fail); }
  .tried tr.reason td { color: var(--fail); white-space: normal; padding-left: var(--sp-3); }
  .tried tr:has(+ tr.reason) { border-bottom: none; }
  .fail .msg { color: var(--fail); }
  .basis { color: var(--ink-3); }
  /* The note wraps to the table's width; it never widens the Outputs column. */
  .basis-note { color: var(--ink-2); margin: var(--sp-1) 0 0; width: 0; min-width: 100%; }
</style>
