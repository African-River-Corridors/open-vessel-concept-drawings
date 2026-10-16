<script lang="ts">
  import type { Choice, InputSpec, Message, StoredInput, VesselClassSummary } from "./api";

  // How an Option is defined: by typed dimensions, or by a standard Vessel Class (issue #94). The
  // schemes, the classes and what picking a class sets all come from the schema, and the class the
  // vessel sits in comes from Rust. Length and beam decide the class, so there is no "modified"
  // label: the panel shows the class the vessel sits in. Picking a class sets length, beam and
  // Formation; the other inputs are drawn with a "class" badge where they came from it.
  let { schemeSpec, classSpec, inputs, summary, messages, onEdit }: {
    schemeSpec: InputSpec | undefined;
    classSpec: InputSpec | undefined;
    inputs: Record<string, StoredInput>;
    summary: VesselClassSummary | null | undefined;
    messages: Message[];
    onEdit: (name: string, value: unknown) => void;
  } = $props();

  let scheme = $derived((inputs["vessel_scheme"]?.value ?? null) as string | null);
  let current = $derived((inputs["vessel_class"]?.value ?? null) as string | null);
  /** The user asked for a standard class but has not picked a scheme yet. */
  let asking = $state(false);
  let byClass = $derived(scheme !== null || asking);
  let kind = $derived(((inputs["formation"]?.value ?? {}) as { kind?: string }).kind);
  let classes = $derived(
    ((classSpec?.choices ?? []) as Choice[]).filter((c) => c.scheme === scheme && c.formation === kind),
  );
  const KINDS: Record<string, string> = {
    self_propelled: "self-propelled",
    pushed_barge: "pushed barge",
    pushed_convoy: "pushed convoy",
  };

  function range(r: [number, number] | null | undefined, unit: string): string {
    if (!r) return "none";
    return r[0] === r[1] ? `${r[0]} ${unit}` : `${r[0]} to ${r[1]} ${unit}`;
  }
</script>

<div class="vc" data-input="vessel_class">
  <div class="modes" role="radiogroup" aria-label="How the vessel is defined">
    <label>
      <input
        type="radio"
        name="vc-mode"
        data-testid="vessel-mode-typed"
        checked={!byClass}
        onchange={() => { asking = false; if (scheme !== null) onEdit("vessel_scheme", null); }}
      />
      Typed dimensions
    </label>
    <label>
      <input
        type="radio"
        name="vc-mode"
        data-testid="vessel-mode-class"
        checked={byClass}
        onchange={() => (asking = true)}
      />
      Standard class
    </label>
  </div>

  {#if byClass}
    <div class="field">
      <span class="label">Class Scheme</span>
      <select
        aria-label="Class Scheme"
        data-testid="vessel-scheme"
        value={scheme ?? ""}
        onchange={(e) => { if (e.currentTarget.value) { asking = false; onEdit("vessel_scheme", e.currentTarget.value); } }}
      >
        {#if scheme === null}<option value="">Choose a Class Scheme</option>{/if}
        {#each schemeSpec?.choices ?? [] as s (s.value)}
          <option value={s.value}>{s.label}</option>
        {/each}
      </select>
    </div>
    {#if scheme !== null}
      <div class="field">
        <span class="label">Class</span>
        <select
          aria-label="Vessel Class"
          data-testid="vessel-class"
          value={current ?? ""}
          onchange={(e) => { if (e.currentTarget.value) onEdit("vessel_class", e.currentTarget.value); }}
        >
          {#if current === null || !classes.some((c) => c.value === current)}
            <option value={current ?? ""}>{current ?? "Choose a class"}</option>
          {/if}
          {#each classes as c (c.value)}
            <option value={c.value}>{c.label}</option>
          {/each}
        </select>
        {#if classes.length === 0}
          <div class="msg note">No class of this scheme for a {KINDS[kind ?? ""] ?? kind} Formation.</div>
        {/if}
      </div>
    {/if}
    {#if summary}
      <div class="sits" data-testid="vessel-sits-in">
        The vessel sits in <strong>{summary.id}</strong> <span class="badge"><span aria-hidden="true">◆ </span>class</span>
        <div class="dim">
          Up to {summary.length_max_m} m long and {summary.beam_max_m} m wide.
          Draught {range(summary.draught_m, "m")}; air draft {range(summary.air_draft_m, "m")}.
          Length and beam decide the class; the draught and air-draft limits are filled from it.
        </div>
      </div>
    {/if}
  {/if}

  {#each messages as m}
    <div class="msg {m.severity}" data-severity={m.severity}>{m.text}</div>
  {/each}
</div>

<style>
  .vc { margin: var(--sp-2) 0 10px; }
  .modes { display: flex; gap: var(--sp-3); font-size: var(--fs-13); margin-bottom: 6px; }
  .modes label { display: flex; gap: 4px; align-items: center; }
  .field { margin: 6px 0; }
  .label { display: block; font-size: var(--fs-12); color: var(--ink-2); }
  select { width: 100%; padding: 3px 5px; font: inherit; color: var(--ink); background: var(--surface); border: var(--lw-thin) solid var(--rule-strong); border-radius: var(--radius); box-sizing: border-box; }
  select:focus-visible { outline-offset: 0; }
  .sits { margin-top: 6px; font-size: var(--fs-13); }
  .dim { font-size: var(--fs-12); color: var(--ink-3); margin-top: 2px; }
  .badge { font-size: var(--fs-11); padding: 0 5px; border-radius: var(--radius); text-transform: uppercase; letter-spacing: 0.03em; color: var(--src-class); white-space: nowrap; }
  .msg { font-size: var(--fs-12); margin-top: 2px; }
  .msg.error { color: var(--fail); }
  .msg.error::before { content: "✕ " / ""; }
  .msg.warning { color: var(--warn); }
  .msg.warning::before { content: "▲ " / ""; }
  .msg.note { color: var(--ink-3); }
</style>
