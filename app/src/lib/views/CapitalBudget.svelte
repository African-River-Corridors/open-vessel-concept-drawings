<script lang="ts">
  // The capital budget of one Option, as the Design session gives it (the core's `barge_cost`): the
  // barge and tug breakdowns, subtotals, yard margin, delivery and unit total, in the rates' currency.
  // The app adds nothing up. Below the tables: which masses came from this design, which are the
  // core's defaults, so that the figures are not read as more than they are.
  interface Cost {
    currency: string;
    basis: string;
    barge: { item: string; amount: number }[];
    tug: { item: string; amount: number }[];
    barge_subtotal: number;
    tug_subtotal: number;
    yard_margin_fraction: number;
    yard_margin: number;
    delivery: number;
    unit_total: number;
    checks: { name: string; ok: boolean; message: string; severity: string }[];
    from_design: string[];
    core_default: string[];
  }
  let { cost, phase }: { cost: Cost | null | undefined; phase: string | undefined } = $props();

  /** The names of the core's budget items, as the input labels have them. An item not listed shows under the core's name. */
  const ITEM_LABELS: Record<string, string> = {
    hull_steel: "Hull steel",
    type_c_tanks: "Type C tanks",
    tank_supports: "Tank supports",
    insulation: "Insulation",
    cargo_system: "Cargo system",
    piping_machinery: "Piping and machinery",
    outfitting: "Outfitting",
    paint_anodes_misc: "Paint, anodes and miscellaneous",
    battery: "Battery",
    propulsion: "Propulsion",
    wheelhouse_accommodation: "Wheelhouse and accommodation",
  };
  const item = (id: string) => ITEM_LABELS[id] ?? id;

  /** An identifier as words for reading ("tank_shell_t" as "Tank shell"). */
  const words = (id: string) => {
    const text = id.replace(/_t$|_kwh$|_kw$/, "").replace(/_/g, " ");
    return text.charAt(0).toUpperCase() + text.slice(1);
  };
  const money = (v: number) => v.toLocaleString("en-GB", { maximumFractionDigits: 0 });
  const list = (ids: string[]) => ids.map(words).join(", ");
  let sections = $derived(
    cost
      ? [
          { title: "Barge", rows: cost.barge, subtotal: cost.barge_subtotal },
          { title: "Tug", rows: cost.tug, subtotal: cost.tug_subtotal },
        ]
      : [],
  );
  let failing = $derived((cost?.checks ?? []).filter((c) => !c.ok));
</script>

<div class="budget" data-testid="capital-budget">
  {#if phase !== "liquid_gas"}
    <div data-testid="capital-budget-wip">
      <h3 class="wip">Work in development</h3>
      <p class="note">The capital budget is computed for gas cargo only. A dry-bulk budget is work in development.</p>
    </div>
  {:else if !cost}
    <p class="note" data-testid="capital-budget-none">There is no capital budget for this Option, because the hull holds no tank arrangement to cost.</p>
  {:else}
    <p class="basis" data-testid="capital-budget-basis"><strong>{cost.basis}</strong> All amounts are in {cost.currency}.</p>
    <div class="tables">
      {#each sections as { title, rows, subtotal } (title)}
        <table data-testid="cost-{title.toLowerCase()}">
          <caption>{title}</caption>
          <tbody>
            {#each rows as r (r.item)}
              <tr><th scope="row">{item(r.item)}</th><td class="num">{money(r.amount)}</td></tr>
            {/each}
            <tr class="sub"><th scope="row">{title} subtotal</th><td class="num">{money(subtotal)}</td></tr>
          </tbody>
        </table>
      {/each}
      <table data-testid="cost-total">
        <caption>Unit</caption>
        <tbody>
          <tr><th scope="row">Barge subtotal</th><td class="num">{money(cost.barge_subtotal)}</td></tr>
          <tr><th scope="row">Tug subtotal</th><td class="num">{money(cost.tug_subtotal)}</td></tr>
          <tr><th scope="row">Yard margin ({(cost.yard_margin_fraction * 100).toLocaleString("en-GB", { maximumFractionDigits: 1 })}%)</th><td class="num">{money(cost.yard_margin)}</td></tr>
          <tr><th scope="row">Delivery</th><td class="num">{money(cost.delivery)}</td></tr>
          <tr class="sub total"><th scope="row">Unit total</th><td class="num" data-testid="cost-unit-total">{money(cost.unit_total)}</td></tr>
        </tbody>
      </table>
    </div>

    <p class="masses" data-testid="cost-masses">
      Mass from this design: {cost.from_design.length > 0 ? list(cost.from_design) : "none"}.
      {#if cost.core_default.length > 0}Core defaults, not from this design: {list(cost.core_default)}.{/if}
    </p>

    {#if failing.length > 0}
      <ul class="checks" data-testid="cost-checks">
        {#each failing as c (c.name)}
          <li class:warn={c.severity !== "error"}><span class="chip" aria-hidden="true">{c.severity === "error" ? "✕" : "▲"}</span> <strong>{c.severity === "error" ? "Check failed" : "Note"}: {words(c.name)}.</strong> {c.message}</li>
        {/each}
      </ul>
    {/if}
  {/if}
</div>

<style>
  .budget { height: 100%; overflow: auto; font-size: var(--fs-13); }
  .note { color: var(--ink-3); font-size: var(--fs-14); }
  .wip { font-family: var(--font-serif); color: var(--ink); font-size: var(--fs-16); margin: 0 0 var(--sp-1); }
  .basis { margin: 0 0 var(--sp-2); }
  .tables { display: grid; grid-template-columns: repeat(auto-fit, minmax(230px, 1fr)); gap: var(--sp-4); align-items: start; }
  table { border-collapse: collapse; width: 100%; }
  caption { text-align: left; font: 600 var(--fs-12) var(--font-ui); text-transform: uppercase; letter-spacing: 0.08em; color: var(--ink-2); background: var(--surface-2); border-bottom: var(--lw-thick) solid var(--ink); padding: var(--sp-1) 6px; }
  th, td { padding: 2px 6px; border-bottom: var(--lw-thin) solid var(--rule); text-align: left; font-weight: 400; }
  td.num { text-align: right; font-variant-numeric: tabular-nums; }
  tr.sub th, tr.sub td { font-weight: 600; border-top: var(--lw-thin) solid var(--rule-strong); }
  tr.total th, tr.total td { background: var(--surface-2); }
  .masses { margin: var(--sp-3) 0 6px; color: var(--ink-3); }
  .checks { margin: var(--sp-1) 0 0; padding: 0; list-style: none; }
  .checks li { color: var(--fail); background: var(--fail-soft); padding: var(--sp-1) var(--sp-2); margin-bottom: 2px; }
  .checks li.warn { color: var(--warn); background: var(--warn-soft); }
</style>
