<script lang="ts">
  import type { InputSpec, Message, Ranges, StoredInput } from "./api";

  // One input, drawn from its schema entry. It shows what Rust sent and sends back what the user
  // typed or chose. The number the user types stays in the box while it is typed: the box is filled
  // once and not rewritten, so "76." is not turned into "76".
  let { spec, stored, messages, onEdit, onReset, onFormation }: {
    spec: InputSpec;
    stored: StoredInput;
    messages: Message[];
    onEdit: (value: unknown) => void;
    onReset: () => void;
    onFormation: (kind: string, abreast: number | null, inLine: number | null) => void;
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
    return parts.join(" and ");
  }

  function show(v: unknown): string {
    return v === null ? "none" : String(v);
  }

  let help = $derived.by(() => {
    const lines: string[] = [];
    if (spec.default !== undefined) lines.push(`Default: ${show(spec.default)}${spec.unit ? " " + spec.unit : ""}`);
    if (spec.default_by) {
      const by = Object.entries(spec.default_by.values).map(([k, v]) => `${k} ${show(v)}`).join(", ");
      lines.push(`Default by ${spec.default_by.by}: ${by}`);
    }
    if (spec.default_rule) {
      const r = spec.default_rule;
      const NAMES: Record<string, string> = { dry_bulk: "dry bulk, container", liquid: "liquid", gas: "gas" };
      const by = Object.entries(r.ratios).map(([g, v]) => `${NAMES[g] ?? g} ${v}`).join(", ");
      lines.push(`Default when left out: the beam times a ratio by Cargo (${by}), to the millimetre`);
      if (r.length_to_depth_warning_above !== null) lines.push(`Warns when length over depth is above ${r.length_to_depth_warning_above}`);
      lines.push(`Rule basis: ${r.basis}`);
    }
    if (spec.hard) lines.push(`Hard range: ${range(spec.hard)}`);
    if (spec.soft) lines.push(`Soft range: ${spec.soft.min} to ${spec.soft.max}`);
    if (spec.basis) lines.push(`Basis: ${spec.basis}`);
    return lines.join("\n");
  });

  function typed(text: string, send: (n: number) => void) {
    const n = Number(text);
    if (text.trim() !== "" && Number.isFinite(n)) send(n);
  }

  // The shape tells the three sources apart without colour; the word stays the accessible text.
  const SHAPES: Record<string, string> = { typed: "●", default: "○", class: "◆" };
  let hasError = $derived(messages.some((m) => m.severity === "error"));

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
    <span class="label">{spec.label}</span>
    <span class="badge {stored.source}"><span class="shape" aria-hidden="true">{SHAPES[stored.source] ?? ""} </span>{stored.source}</span>
    {#if !spec.required && stored.source === "typed"}
      <button class="link" onclick={onReset} title="Go back to the default">reset</button>
    {/if}
  </div>

  {#if spec.kind === "number"}
    <div class="row">
      {#if showNumber}
        <input
          type="text"
          inputmode="decimal"
          aria-label={spec.label}
          class:invalid={hasError}
          use:fill={value === null ? "" : String(value)}
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
          none
        </label>
      {/if}
    </div>
  {:else if spec.kind === "bool"}
    <label class="none">
      <input type="checkbox" aria-label={spec.label} checked={value === true} onchange={(e) => onEdit(e.currentTarget.checked)} />
      {value === true ? "yes" : "no"}
    </label>
  {:else if spec.kind === "choice" || spec.kind === "cargo"}
    <select aria-label={spec.label} class:invalid={hasError} value={String(value)} onchange={(e) => onEdit(e.currentTarget.value)}>
      {#each spec.choices ?? [] as c (c.value)}
        <option value={c.value}>{c.label}</option>
      {/each}
    </select>
  {:else if spec.kind === "formation"}
    <select
      aria-label={spec.label}
      class:invalid={hasError}
      value={formation.kind}
      onchange={(e) => onFormation(e.currentTarget.value, e.currentTarget.value === "pushed_convoy" ? abreast : null, e.currentTarget.value === "pushed_convoy" ? inLine : null)}
    >
      {#each spec.choices ?? [] as c (c.value)}
        <option value={c.value}>{c.label}</option>
      {/each}
    </select>
    {#if formation.kind === "pushed_convoy"}
      <div class="row convoy">
        <label>
          Abreast
          <input
            type="text"
            inputmode="numeric"
            aria-label="Abreast"
            use:fill={abreast === null ? "" : String(abreast)}
            oninput={(e) => typed(e.currentTarget.value, (n) => { abreast = n; onFormation(formation.kind, abreast, inLine); })}
          />
        </label>
        <label>
          In line
          <input
            type="text"
            inputmode="numeric"
            aria-label="In line"
            use:fill={inLine === null ? "" : String(inLine)}
            oninput={(e) => typed(e.currentTarget.value, (n) => { inLine = n; onFormation(formation.kind, abreast, inLine); })}
          />
        </label>
      </div>
    {/if}
  {/if}

  {#each messages as m}
    <div class="msg {m.severity}" data-severity={m.severity}>{m.text}</div>
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
  .row.convoy label { display: flex; flex-direction: column; font-size: var(--fs-11); color: var(--ink-3); }
  input[type="text"], select { width: 100%; padding: 3px 5px; font: inherit; color: var(--ink); background: var(--surface); border: var(--lw-thin) solid var(--rule-strong); border-radius: var(--radius); box-sizing: border-box; }
  input[type="text"] { text-align: right; font-variant-numeric: tabular-nums; }
  input:focus-visible, select:focus-visible { outline-offset: 0; }
  input.invalid, select.invalid { border-width: var(--lw-thick); border-color: var(--fail); padding: 2px 4px; }
  .unit { min-width: 2.2em; font-size: var(--fs-12); color: var(--ink-3); }
  .none { font-size: var(--fs-12); display: flex; gap: 4px; align-items: center; white-space: nowrap; }
  .msg { font-size: var(--fs-12); margin-top: 2px; }
  /* The shape is generated, so it is not part of the message text; the empty alternative keeps it out of speech. */
  .msg.error { color: var(--fail); }
  .msg.error::before { content: "✕ " / ""; }
  .msg.warning { color: var(--warn); }
  .msg.warning::before { content: "▲ " / ""; }
</style>
