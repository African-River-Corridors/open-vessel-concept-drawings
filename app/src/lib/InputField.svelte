<script lang="ts">
  import type { InputSpec, Message, Ranges, StoredInput } from "./api";
  import { inputText, parseInput, say, schemaText, t } from "./i18n.svelte";

  // One input, drawn from its schema entry. It shows what Rust sent and sends back what the user
  // typed or chose. The number the user types stays in the box while it is typed: the box is filled
  // once and not rewritten, so "76." is not turned into "76".
  let { spec, stored, messages, onEdit, onReset, onFormation, onImportOffsets }: {
    spec: InputSpec;
    stored: StoredInput;
    messages: Message[];
    onEdit: (value: unknown) => void;
    onReset: () => void;
    onFormation: (kind: string, abreast: number | null, inLine: number | null) => void;
    onImportOffsets: () => void;
  } = $props();

  // The box is filled once. When Rust changes the value later (the Vessel Class moved, say) it is
  // refilled, unless the user is typing in it, so "76." is not turned into "76".
  function fill(el: HTMLInputElement, text: string) {
    el.value = text;
    return {
      update(next: string) {
        if (document.activeElement !== el) el.value = next;
      },
    };
  }

  function range(r: Ranges["hard"]): string {
    if (!r) return "";
    const parts: string[] = [];
    if (r.gt !== undefined) parts.push(`> ${r.gt}`);
    if (r.ge !== undefined) parts.push(`>= ${r.ge}`);
    if (r.lt !== undefined) parts.push(`< ${r.lt}`);
    if (r.le !== undefined) parts.push(`<= ${r.le}`);
    return parts.reduce((first, second) => t("app-range-and", { first, second }));
  }

  /** One selected Rule Set: id@version, its status and its unverified blocking Rules. */
  function setText(s: { id: string; version: string }): string {
    const known = spec.known_sets?.find((k) => k.id === s.id && k.version === s.version);
    return known
      ? `${s.id}@${s.version} (${known.status}, ${t("app-unverified-count", { count: known.unverified_blocking })})`
      : `${s.id}@${s.version}`;
  }

  function show(v: unknown): string {
    return v === null ? t("app-none") : String(v);
  }

  /** The Cargo groups of the moulded-depth rule, by the engine's group name. */
  const GROUP_IDS: Record<string, string> = {
    dry_bulk: "app-cargo-group-dry-bulk",
    liquid: "app-cargo-group-liquid",
    gas: "app-cargo-group-gas",
  };
  const SOURCE_IDS: Record<string, string> = { typed: "app-source-typed", default: "app-source-default", class: "app-source-class" };

  let help = $derived.by(() => {
    const lines: string[] = [];
    if (spec.default !== undefined) lines.push(t("app-help-default", { value: `${show(spec.default)}${spec.unit ? " " + spec.unit : ""}` }));
    if (spec.default_by) {
      const by = Object.entries(spec.default_by.values).map(([k, v]) => `${k} ${show(v)}`).join(", ");
      lines.push(t("app-help-default-by", { by: schemaText.defaultBy(spec.name, spec.default_by.by), values: by }));
    }
    if (spec.default_rule) {
      const r = spec.default_rule;
      const by = Object.entries(r.ratios).map(([g, v]) => `${GROUP_IDS[g] ? t(GROUP_IDS[g]) : g} ${v}`).join(", ");
      lines.push(t("app-help-default-rule", { ratios: by }));
      if (r.length_to_depth_warning_above !== null) lines.push(t("app-help-depth-warning", { limit: String(r.length_to_depth_warning_above) }));
      lines.push(t("app-help-rule-basis", { basis: r.basis }));
    }
    if (spec.hard) lines.push(t("app-help-hard-range", { range: range(spec.hard) }));
    if (spec.soft) lines.push(t("app-help-soft-range", { min: String(spec.soft.min), max: String(spec.soft.max) }));
    if (spec.basis) lines.push(t("app-help-basis", { basis: spec.basis }));
    return lines.join("\n");
  });

  function typed(text: string, send: (n: number) => void) {
    const n = parseInput(text);
    if (Number.isFinite(n)) send(n);
  }

  // The shape tells the three sources apart without colour; the word stays the accessible text.
  const SHAPES: Record<string, string> = { typed: "●", default: "○", class: "◆" };
  let hasError = $derived(messages.some((m) => m.severity === "error"));
  let label = $derived(schemaText.input(spec.name, spec.label));

  let value = $derived(stored.value);
  // Local display state, taken once: the field is drawn afresh (new key) when Rust changes a value.
  /* svelte-ignore state_referenced_locally */
  let showNumber = $state(stored.value !== null);

  let formation = $derived(
    (spec.kind === "formation" ? value : {}) as { kind: string; abreast?: number; in_line?: number },
  );
  /* svelte-ignore state_referenced_locally */
  let abreast = $state<number | null>(formation.abreast ?? null);
  /* svelte-ignore state_referenced_locally */
  let inLine = $state<number | null>(formation.in_line ?? null);
</script>

<div class="field" data-input={spec.name} title={help}>
  <div class="head">
    <span class="label">{label}</span>
    <span class="badge {stored.source}"><span class="shape" aria-hidden="true">{SHAPES[stored.source] ?? ""} </span>{SOURCE_IDS[stored.source] ? t(SOURCE_IDS[stored.source]) : stored.source}</span>
    {#if !spec.required && stored.source === "typed"}
      <button class="link" onclick={onReset} title={t("app-reset-title")}>{t("app-reset")}</button>
    {/if}
  </div>

  {#if spec.kind === "number"}
    <div class="row">
      {#if showNumber}
        <input
          type="text"
          inputmode="decimal"
          aria-label={label}
          class:invalid={hasError}
          use:fill={value === null ? "" : inputText(Number(value))}
          oninput={(e) => typed(e.currentTarget.value, (n) => onEdit(n))}
        />
        <span class="unit">{spec.unit && spec.unit !== "-" ? spec.unit : ""}</span>
      {/if}
      {#if spec.nullable}
        <label class="none">
          <input
            type="checkbox"
            checked={!showNumber}
            onchange={(e) => {
              showNumber = !e.currentTarget.checked;
              if (e.currentTarget.checked) onEdit(null);
            }}
          />
          {t("app-none")}
        </label>
      {/if}
    </div>
  {:else if spec.kind === "bool"}
    <label class="none">
      <input type="checkbox" aria-label={label} checked={value === true} onchange={(e) => onEdit(e.currentTarget.checked)} />
      {t(value === true ? "app-yes" : "app-no")}
    </label>
  {:else if spec.kind === "choice" || spec.kind === "cargo"}
    <select aria-label={label} class:invalid={hasError} value={String(value)} onchange={(e) => onEdit(e.currentTarget.value)}>
      {#each spec.choices ?? [] as c (c.value)}
        <option value={c.value}>{spec.kind === "cargo" && c.status === "development" ? t("app-cargo-development", { name: schemaText.choice(spec.name, c.value, c.label) }) : schemaText.choice(spec.name, c.value, c.label)}</option>
      {/each}
    </select>
  {:else if spec.kind === "formation"}
    <select
      aria-label={label}
      class:invalid={hasError}
      value={formation.kind}
      onchange={(e) => onFormation(e.currentTarget.value, e.currentTarget.value === "pushed_convoy" ? abreast : null, e.currentTarget.value === "pushed_convoy" ? inLine : null)}
    >
      {#each spec.choices ?? [] as c (c.value)}
        <option value={c.value}>{schemaText.choice(spec.name, c.value, c.label)}</option>
      {/each}
    </select>
    {#if formation.kind === "pushed_convoy"}
      <div class="row convoy">
        <label>
          {t("app-convoy-abreast")}
          <input
            type="text"
            inputmode="numeric"
            aria-label={t("app-convoy-abreast")}
            use:fill={abreast === null ? "" : String(abreast)}
            oninput={(e) => typed(e.currentTarget.value, (n) => { abreast = n; onFormation(formation.kind, abreast, inLine); })}
          />
        </label>
        <label>
          {t("app-convoy-in-line")}
          <input
            type="text"
            inputmode="numeric"
            aria-label={t("app-convoy-in-line")}
            use:fill={inLine === null ? "" : String(inLine)}
            oninput={(e) => typed(e.currentTarget.value, (n) => { inLine = n; onFormation(formation.kind, abreast, inLine); })}
          />
        </label>
      </div>
    {/if}
  {:else if spec.kind === "text"}
    <!-- Issue #429: a text (the currency, the price base date). An empty box is "none" where the input allows it. -->
    <div class="row">
      <input
        type="text"
        class="text"
        aria-label={label}
        class:invalid={hasError}
        use:fill={value === null ? "" : String(value)}
        oninput={(e) => onEdit(e.currentTarget.value === "" && spec.nullable ? null : e.currentTarget.value)}
      />
    </div>
  {:else if spec.kind === "parameter_set"}
    <!-- Issue #446: shown, not edited here; its editor comes with the ShipD Hull Form (#452). -->
    <span class="summary">{value === null ? t("app-none") : t("app-parameter-count", { count: Object.keys(value as object).length })}</span>
  {:else if spec.kind === "rule_sets"}
    <!-- Issue #274: shown, not edited here; the Rule Set commands come with #275. -->
    <span class="summary">{Array.isArray(value) ? value.map((s: { id: string; version: string }) => setText(s)).join("; ") : t("app-none")}</span>
  {:else if spec.kind === "openings_list"}
    <!-- Issue #592: shown, not edited here; the openings are typed in the file. -->
    <span class="summary">{Array.isArray(value) && value.length > 0 ? t("app-opening-count", { count: value.length }) : t("app-none")}</span>
  {:else if spec.kind === "offsets_table"}
    <!-- Issue #451: the station count, and an Import button that reads an offsets CSV. -->
    <div class="row">
      <span class="summary">{value === null ? t("app-none") : t("app-offset-station-count", { count: new Set((value as { station_x_m: number }[]).map((r) => r.station_x_m)).size })}</span>
      <button type="button" data-testid="import-offsets" onclick={onImportOffsets}>{t("app-import-offsets")}</button>
    </div>
  {/if}

  {#each messages as m}
    <div class="msg {m.severity}" data-severity={m.severity}>{say(m.text, null, m.msg)}</div>
  {/each}
</div>

<style>
  .field { margin: var(--sp-2) 0 10px; }
  .head { display: flex; align-items: baseline; gap: 6px; }
  .label { font-size: var(--fs-12); color: var(--ink-2); }
  .badge { font-size: var(--fs-11); padding: 0 5px; border-radius: var(--radius); text-transform: uppercase; letter-spacing: 0.03em; white-space: nowrap; }
  .badge.typed { color: var(--src-typed); }
  .badge.default { color: var(--src-default); }
  .badge.class { color: var(--src-class); }
  .link { background: none; border: none; color: var(--accent); text-decoration: underline; padding: 0; cursor: pointer; font-size: var(--fs-11); }
  .link:hover:not(:disabled) { background: none; }
  .row { display: flex; gap: 6px; align-items: center; }
  .summary { font-size: var(--fs-12); color: var(--ink-3); }
  .row.convoy label { display: flex; flex-direction: column; font-size: var(--fs-11); color: var(--ink-3); }
  input[type="text"], select { width: 100%; padding: 3px 5px; font: inherit; color: var(--ink); background-color: var(--surface); border: var(--lw-thin) solid var(--rule-strong); border-radius: var(--radius); box-sizing: border-box; }
  select { padding-right: var(--sp-5); }
  input[type="text"] { text-align: right; font-variant-numeric: tabular-nums; }
  input[type="text"].text { text-align: left; }
  input:focus-visible, select:focus-visible { outline-offset: 0; }
  input.invalid, select.invalid { border-width: var(--lw-thick); border-color: var(--fail); padding: 2px 4px; }
  select.invalid { padding-right: calc(var(--sp-5) - 1px); }
  .unit { min-width: 2.2em; font-size: var(--fs-12); color: var(--ink-3); }
  .none { font-size: var(--fs-12); display: flex; gap: 4px; align-items: center; white-space: nowrap; }
  .msg { font-size: var(--fs-12); margin-top: 2px; }
  /* The shape is generated, so it is not part of the message text; the empty alternative keeps it out of speech. */
  .msg.error { color: var(--fail); }
  .msg.error::before { content: "✕ " / ""; }
  .msg.warning { color: var(--warn); }
  .msg.warning::before { content: "▲ " / ""; }
</style>
