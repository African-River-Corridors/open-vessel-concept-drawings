<script lang="ts">
  // The capital budget of one Option, as the Design session gives it (the core's `barge_cost`; a tank barge's
  // cargo tank items are lump sums, issue #334): the
  // barge and tug breakdowns, subtotals, yard margin, delivery and unit total, in the rates' currency.
  // A Formation with no tug (self-propelled, issue #403) has an empty tug breakdown: it is not shown.
  // The app adds nothing up. Below the tables: which masses came from this design, which are the
  // core's defaults, so that the figures are not read as more than they are.
  interface Cost {
    currency: string;
    basis: string;
    /** The date the prices are as of (issue #429): null for current estimates. */
    price_base_date?: string | null;
    /** Which rates (issue #404): "example" while every rate is the example file's, "user" once one is typed. */
    rates?: "example" | "user";
    barge: { item: string; amount: number }[];
    tug: { item: string; amount: number }[];
    barge_subtotal: number;
    tug_subtotal: number;
    yard_margin_fraction: number;
    yard_margin: number;
    delivery: number;
    unit_total: number;
    checks: { name: string; ok: boolean; message: string; severity: string }[];
    /** The share of the tank structure surplus priced at the tank shell rate (issue #402): an assumption. */
    tank_internals?: { fraction: number; status: string; source: string | null } | null;
    from_design: string[];
    core_default: string[];
    /** The two hull steel figures (issue #401): the lightship's top-down one, the budget's bottom-up one. */
    steel?: { top_down_t: number; bottom_up_t: number; delta_t: number } | null;
  }
  import { checkName, list as join, num, say, t, type TextIds } from "../i18n.svelte";

  let { cost, phase, ids }: { cost: Cost | null | undefined; phase: string | undefined; ids: TextIds | null } = $props();

  /** The core's budget items, by id (Message Ids `app-cost-item-…`). An item not listed shows under the core's name. */
  const ITEMS = [
    "hull_steel",
    "tank_coating",
    "cargo_pumps",
    "cargo_piping",
    "cargo_heating",
    "vents_gauging",
    "type_c_tanks",
    "tank_supports",
    "insulation",
    "cargo_system",
    "piping_machinery",
    "outfitting",
    "paint_anodes_misc",
    "battery",
    "propulsion",
    "wheelhouse_accommodation",
  ];
  const item = (id: string) => (ITEMS.includes(id) ? t(`app-cost-item-${id.replaceAll("_", "-")}`) : id);

  /** An identifier as words for reading ("tank_shell_t" as "Tank shell"). */
  const words = (id: string) => {
    const text = id.replace(/_t$|_kwh$|_kw$/, "").replace(/_/g, " ");
    return text.charAt(0).toUpperCase() + text.slice(1);
  };
  /** A cost check's name: translated where the Display Language has it, else as words. */
  const costCheck = (name: string) => {
    const said = checkName(name);
    return said === name ? words(name) : said;
  };
  const money = (v: number) => num(v, 0);
  const list = (ids: string[]) => join(ids.map(words), "unit");
  let sections = $derived(
    cost
      ? [
          { key: "barge", caption: "app-cost-barge", total: "app-cost-barge-subtotal", rows: cost.barge, subtotal: cost.barge_subtotal },
          { key: "tug", caption: "app-cost-tug", total: "app-cost-tug-subtotal", rows: cost.tug, subtotal: cost.tug_subtotal },
        ].filter((s) => s.rows.length > 0)
      : [],
  );
  let failing = $derived((cost?.checks ?? []).filter((c) => !c.ok));
</script>

<div class="budget" data-testid="capital-budget">
  {#if phase !== "liquid_gas" && phase !== "liquid_bulk"}
    <div data-testid="capital-budget-wip">
      <h3 class="wip">{t("app-work-in-development")}</h3>
      <p class="note">{t("app-cost-wip")}</p>
    </div>
  {:else if !cost}
    <p class="note" data-testid="capital-budget-none">{t("app-cost-none")}</p>
  {:else}
    <p class="basis" data-testid="capital-budget-basis"><strong>{cost.rates ? t(`app-cost-basis-${cost.rates}`) : say(cost.basis, ids)}</strong> {cost.rates ? t(cost.price_base_date ? "app-cost-prices-as-of" : "app-cost-prices-current", { date: cost.price_base_date ?? "" }) : ""} {t("app-cost-currency", { currency: cost.currency })}</p>
    <div class="tables">
      {#each sections as { key, caption, total, rows, subtotal } (key)}
        <table data-testid="cost-{key}">
          <caption>{t(caption)}</caption>
          <tbody>
            {#each rows as r (r.item)}
              <tr><th scope="row">{item(r.item)}</th><td class="num">{money(r.amount)}</td></tr>
              {#if key === "barge" && r.item === "hull_steel" && cost.steel}
                <tr class="steel" data-testid="cost-steel-delta"><td colspan="2">{t("app-cost-steel-delta", { top_down: num(cost.steel.top_down_t, 1), bottom_up: num(cost.steel.bottom_up_t, 1), delta: num(cost.steel.delta_t, 1) })}</td></tr>
              {/if}
            {/each}
            <tr class="sub"><th scope="row">{t(total)}</th><td class="num">{money(subtotal)}</td></tr>
          </tbody>
        </table>
      {/each}
      <table data-testid="cost-total">
        <caption>{t("app-cost-unit")}</caption>
        <tbody>
          <tr><th scope="row">{t("app-cost-barge-subtotal")}</th><td class="num">{money(cost.barge_subtotal)}</td></tr>
          {#if cost.tug.length > 0}<tr><th scope="row">{t("app-cost-tug-subtotal")}</th><td class="num">{money(cost.tug_subtotal)}</td></tr>{/if}
          <tr><th scope="row">{t("app-cost-yard-margin", { percent: num(cost.yard_margin_fraction * 100, 0, 1) })}</th><td class="num">{money(cost.yard_margin)}</td></tr>
          <tr><th scope="row">{t("app-cost-delivery")}</th><td class="num">{money(cost.delivery)}</td></tr>
          <tr class="sub total"><th scope="row">{t("app-cost-unit-total")}</th><td class="num" data-testid="cost-unit-total">{money(cost.unit_total)}</td></tr>
        </tbody>
      </table>
    </div>

    <p class="masses" data-testid="cost-masses">
      {t("app-cost-from-design", { masses: cost.from_design.length > 0 ? list(cost.from_design) : t("app-none") })}
      {#if cost.core_default.length > 0}{t("app-cost-core-defaults", { masses: list(cost.core_default) })}{/if}
    </p>

    {#if cost.tank_internals}
      <p class="note" data-testid="cost-tank-internals">{t("app-cost-internals", { percent: num(cost.tank_internals.fraction * 100, 0, 1) })}</p>
    {/if}

    {#if failing.length > 0}
      <ul class="checks" data-testid="cost-checks">
        {#each failing as c (c.name)}
          <li class:warn={c.severity !== "error"}><span class="chip" aria-hidden="true">{c.severity === "error" ? "✕" : "▲"}</span> <strong>{t(c.severity === "error" ? "app-cost-check-failed" : "app-cost-note", { check: costCheck(c.name) })}</strong> {say(c.message, ids)}</li>
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
  tr.steel td { color: var(--ink-3); font-size: var(--fs-12); }
  .masses { margin: var(--sp-3) 0 6px; color: var(--ink-3); }
  .checks { margin: var(--sp-1) 0 0; padding: 0; list-style: none; }
  .checks li { color: var(--fail); background: var(--fail-soft); padding: var(--sp-1) var(--sp-2); margin-bottom: 2px; }
  .checks li.warn { color: var(--warn); background: var(--warn-soft); }
</style>
