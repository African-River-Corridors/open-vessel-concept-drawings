<script lang="ts">
  // The Rule Trace of the Option (spec 0004, "Rule Trace"): one row per Rule per object, as the
  // engine gives it. Failures first, then warnings, passes, notes, not evaluated and not applicable;
  // the trace's order within each group. The app computes nothing: every field comes from the trace.
  // Units are the trace's. Only the table body scrolls, when the list is longer than the panel.
  import { t } from "../i18n.svelte";

  interface Row {
    rule: string | null;
    rule_sets: string[];
    title: string;
    citation: string;
    origin: string | null;
    rule_status: "verified" | "unverified";
    method_status: "verified" | "unverified" | null;
    object: string | null;
    value: number | boolean | string | null;
    unit: string;
    compare: "ge" | "gt" | "le" | "lt" | null;
    limit: number | null;
    margin: number | null;
    consequence: "block" | "warn" | "note" | null;
    outcome: Outcome;
    governing: boolean;
    sizes: boolean;
    gap: string | null;
    finding: string | null;
  }
  type Outcome = "pass" | "fail" | "warn" | "note" | "not evaluated" | "not applicable";
  export interface RuleTrace {
    rule_sets: string[];
    rows: Row[];
    verdict: { passes: boolean; failing_rows: number; blocking_not_evaluated: { rule: string; title: string }[]; summary: string };
  }

  let { trace }: { trace: RuleTrace | null } = $props();

  const COLUMNS = ["result", "rule", "object", "value", "limit", "margin", "citation", "status"] as const;

  // The display order of the outcomes, and each one's tag: a word and a shape, never colour only.
  const OUTCOMES: Record<Outcome, { rank: number; mark: string; label: string; tone: string }> = {
    fail: { rank: 0, mark: "✕", label: "app-fail", tone: "fail" },
    warn: { rank: 1, mark: "!", label: "app-rules-warn", tone: "warn" },
    pass: { rank: 2, mark: "✓", label: "app-pass", tone: "ok" },
    note: { rank: 3, mark: "i", label: "app-rules-note", tone: "plain" },
    "not evaluated": { rank: 4, mark: "○", label: "app-rules-not-evaluated", tone: "open" },
    "not applicable": { rank: 5, mark: "–", label: "app-rules-not-applicable", tone: "plain" },
  };
  const COMPARE = { ge: "≥", gt: ">", le: "≤", lt: "<" } as const;

  let rows = $derived(trace?.rows ?? []);
  let ordered = $derived(
    rows.map((r, i) => ({ r, i })).sort((a, b) => OUTCOMES[a.r.outcome].rank - OUTCOMES[b.r.outcome].rank || a.i - b.i).map((x) => x.r),
  );
  let failing = $derived(rows.filter((r) => r.outcome === "fail").length);

  /** A number as the trace gives it, at most 4 significant figures; text and yes/no as they are. */
  function show(v: number | boolean | string | null): string {
    if (v === null || v === undefined) return "";
    if (typeof v === "boolean") return t(v ? "app-rules-yes" : "app-rules-no");
    if (typeof v === "string") return v;
    return Number.isInteger(v) ? String(v) : String(Number(v.toPrecision(4)));
  }
  const unit = (r: Row) => (r.unit && r.unit !== "-" ? ` ${r.unit}` : "");
</script>

<div class="rules" data-testid="rule-checks">
  {#if !trace || rows.length === 0}
    <p class="note">{t("app-rules-none")}</p>
  {:else}
    <p class="count" data-testid="rule-check-count">
      {t("app-rules-trace-count", { failing, total: rows.length })}
    </p>
    <p class="summary" data-testid="rule-trace-summary">{trace.verdict.summary}</p>
    <div class="body">
      <table>
        <thead>
          <tr>
            {#each COLUMNS as col (col)}<th data-testid="rule-col-{col}">{t(`app-rules-col-${col}`)}</th>{/each}
          </tr>
        </thead>
        <tbody>
          {#each ordered as r, i (i)}
            {@const o = OUTCOMES[r.outcome]}
            <tr class={o.tone} data-testid="rule-check-row" data-outcome={r.outcome} data-ok={r.outcome !== "fail"} data-governing={r.governing}>
              <td data-testid="rule-cell-result">
                <span class="tag {o.tone}"><span aria-hidden="true">{o.mark}</span> {t(o.label)}</span>
                {#if r.governing}<span class="mark" title={t("app-rules-governing-hint")}>{t("app-rules-governing")}</span>{/if}
              </td>
              <th scope="row" data-testid="rule-cell-rule">
                <span class="title">{r.title}</span>
                {#if r.rule}<span class="id">{r.rule}</span>{/if}
                <span class="sets">{r.rule_sets.join(", ")}</span>
                {#if r.sizes}<span class="mark">{t("app-rules-sizes")}</span>{/if}
              </th>
              <td data-testid="rule-cell-object">{r.object ?? ""}</td>
              <td class="num" data-testid="rule-cell-value">{show(r.value)}{r.value !== null && typeof r.value === "number" ? unit(r) : ""}</td>
              <td class="num" data-testid="rule-cell-limit">{#if r.compare && r.limit !== null}{COMPARE[r.compare]} {show(r.limit)}{unit(r)}{/if}</td>
              <td class="num" data-testid="rule-cell-margin">{#if r.margin !== null}{r.margin > 0 ? "+" : ""}{show(r.margin)}{unit(r)}{/if}</td>
              <td class="cite" data-testid="rule-cell-citation">
                {r.citation}
                {#if r.finding}<span class="finding">{t("app-rules-finding", { finding: r.finding })}</span>{/if}
                {#if r.gap}<span class="gap">{r.gap}</span>{/if}
              </td>
              <td data-testid="rule-cell-status">
                {#if r.origin && r.origin !== "regulation" && r.origin !== "class"}
                  <span class="status">{t("app-rules-ovcd-check")}</span>
                {:else if r.origin}
                  <span class="status" class:unverified={r.rule_status !== "verified"}>{t(r.rule_status === "verified" ? "app-rules-verified" : "app-rules-unverified")}</span>
                {/if}
                {#if r.method_status === "unverified"}<span class="status unverified">{t("app-rules-method-unverified")}</span>{/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .rules { height: 100%; display: flex; flex-direction: column; }
  .count { margin: 0 0 2px; font-weight: 600; }
  .summary { margin: 0 0 6px; color: var(--ink-2); font-size: var(--fs-13); }
  .note { color: var(--ink-3); }
  .body { flex: 1; min-height: 0; overflow: auto; border: var(--lw-thin) solid var(--rule); }
  table { border-collapse: collapse; width: 100%; font-size: var(--fs-13); }
  thead th { position: sticky; top: 0; background: var(--surface-2); color: var(--ink); text-align: left; font-weight: 600; padding: var(--sp-1) 10px; border-bottom: var(--lw-thick) solid var(--ink); }
  td, tbody th { padding: var(--sp-1) 10px; vertical-align: top; text-align: left; border-bottom: var(--lw-thin) solid var(--rule); }
  tbody th { font-weight: 600; }
  tbody th span { display: block; }
  .id, .sets { font: var(--fs-11) var(--font-mono); color: var(--ink-3); font-weight: 400; }
  tr.fail .title { color: var(--fail); }
  .num { white-space: nowrap; font-variant-numeric: tabular-nums; }
  .cite { color: var(--ink-3); }
  .cite span, td .mark, .status { display: block; }
  .finding, .gap { font-size: var(--fs-11); }
  .tag { font: 600 var(--fs-11) var(--font-ui); padding: 1px 6px; border-radius: var(--radius); white-space: nowrap; display: inline-block; }
  .tag.ok { background: var(--pass-soft); color: var(--pass); }
  .tag.fail { background: var(--fail-soft); color: var(--fail); }
  .tag.warn { background: var(--warn-soft); color: var(--warn); }
  .tag.open { background: transparent; color: var(--ink-2); border: var(--lw-thin) dashed var(--ink-3); }
  .tag.plain { background: var(--surface-2); color: var(--ink-3); }
  .mark { font: 600 var(--fs-11) var(--font-ui); color: var(--accent-ink); margin-top: 2px; }
  .status { font-size: var(--fs-11); color: var(--ink-2); white-space: nowrap; }
  .status.unverified { color: var(--ink-3); font-style: italic; }
</style>
