<script lang="ts">
  import { num, t } from "../i18n.svelte";

  // The totals of a pushed convoy, as the core gives them (`convoy` of the results, issue #551): the
  // barge, and the sum over every barge of the convoy. The app adds nothing up. The pusher is not in
  // the totals until the pusher sizing is, and the view says so while the core says so.
  interface Figures {
    cargo_t: number | null;
    lightship_t: number | null;
    deadweight_t: number | null;
    displacement_t: number | null;
    cargo_volume_m3: number | null;
    overall_length_m?: number | null;
    overall_beam_m?: number | null;
  }
  interface ConvoyResults {
    abreast: number;
    in_line: number;
    barges: number;
    barge: Figures;
    totals: Figures;
    pusher_included: boolean;
  }
  let { convoy, dims }: { convoy: ConvoyResults; dims: { loa_m?: number; beam_m?: number } | null | undefined } = $props();

  type Row = { id: string; label: string; unit: string; dp: number; barge: number | null | undefined; total: number | null | undefined };
  const rows = $derived<Row[]>([
    { id: "cargo", label: "app-convoy-cargo", unit: "t", dp: 1, barge: convoy.barge.cargo_t, total: convoy.totals.cargo_t },
    { id: "lightship", label: "app-convoy-lightship", unit: "t", dp: 1, barge: convoy.barge.lightship_t, total: convoy.totals.lightship_t },
    { id: "deadweight", label: "app-convoy-deadweight", unit: "t", dp: 1, barge: convoy.barge.deadweight_t, total: convoy.totals.deadweight_t },
    { id: "displacement", label: "app-convoy-displacement", unit: "t", dp: 1, barge: convoy.barge.displacement_t, total: convoy.totals.displacement_t },
    { id: "volume", label: "app-convoy-cargo-volume", unit: "m³", dp: 1, barge: convoy.barge.cargo_volume_m3, total: convoy.totals.cargo_volume_m3 },
    { id: "length", label: "app-convoy-overall-length", unit: "m", dp: 2, barge: dims?.loa_m, total: convoy.totals.overall_length_m },
    { id: "beam", label: "app-convoy-overall-beam", unit: "m", dp: 2, barge: dims?.beam_m, total: convoy.totals.overall_beam_m },
  ]);
  const cell = (v: number | null | undefined, dp: number) => (v === null || v === undefined ? "–" : num(v, dp));
</script>

<div class="convoy" data-testid="convoy">
  <p class="summary" data-testid="convoy-summary">
    <strong>{t("app-convoy-formation", { abreast: convoy.abreast, in_line: convoy.in_line, barges: convoy.barges })}</strong>
  </p>
  <table data-testid="convoy-table">
    <thead>
      <tr>
        <th>{t("app-convoy-quantity")}</th>
        <th class="num">{t("app-convoy-one-barge")}</th>
        <th class="num">{t("app-convoy-total")}</th>
      </tr>
    </thead>
    <tbody>
      {#each rows as r (r.id)}
        <tr data-row={r.id}>
          <th>{t(r.label)}<span class="unit">&nbsp;({r.unit})</span></th>
          <td class="num" data-cell="barge" title={r.barge === null || r.barge === undefined ? "" : String(r.barge)}>{cell(r.barge, r.dp)}</td>
          <td class="num" data-cell="total" title={r.total === null || r.total === undefined ? "" : String(r.total)}>{cell(r.total, r.dp)}</td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if !convoy.pusher_included}
    <p class="note" data-testid="convoy-pusher-note">{t("app-convoy-pusher-note")}</p>
  {/if}
</div>

<style>
  .convoy { height: 100%; overflow: auto; }
  .summary { margin: 0 0 8px; }
  table { border-collapse: collapse; font-size: 13px; }
  th { text-align: left; font-weight: normal; padding: 3px 14px 3px 0; white-space: nowrap; }
  thead th { color: var(--ink-3); }
  td { padding: 3px 8px; }
  .num { text-align: right; font-variant-numeric: tabular-nums; }
  th.num { text-align: right; }
  .unit { color: var(--ink-3); }
  .note { color: var(--ink-3); margin-top: 10px; }
</style>
