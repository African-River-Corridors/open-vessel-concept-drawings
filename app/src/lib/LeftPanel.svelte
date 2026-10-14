<script lang="ts">
  import type { Evaluation, OptionEvaluation, Schema, Snapshot } from "./api";

  // Placeholder for ticket #74: the option picker and the required number inputs, drawn from the
  // schema. It only shows what Rust sent and sends back what the user typed.
  let { snap, schema, evaluation, generation, onPick, onEdit }: {
    snap: Snapshot | null;
    /** Changes when a different Design is loaded, so the fields are drawn afresh. */
    generation: number;
    schema: Schema | null;
    evaluation: OptionEvaluation | undefined;
    onPick: (name: string) => void;
    onEdit: (name: string, value: number) => void;
  } = $props();

  let option = $derived(snap?.design.options.find((o) => o.name === snap!.active_option));
  let numbers = $derived((schema?.inputs ?? []).filter((i) => i.required && i.kind === "number"));

  /** Fill a field once. After that the user's typing owns it, so "76." is not rewritten to "76". */
  function fill(el: HTMLInputElement, text: string) {
    el.value = text;
  }

  function typed(name: string, text: string) {
    const n = Number(text);
    if (text.trim() !== "" && Number.isFinite(n)) onEdit(name, n);
  }
  function messagesFor(name: string) {
    return (evaluation?.messages ?? []).filter((m) => m.inputs.includes(name));
  }
</script>

<div class="panel">
  <h2>Inputs</h2>
  {#if snap}
    <label>
      Option
      <select value={snap.active_option} onchange={(e) => onPick(e.currentTarget.value)}>
        {#each snap.design.options as o (o.name)}
          <option value={o.name}>{o.name}</option>
        {/each}
      </select>
    </label>
    {#each numbers as spec (`${spec.name}|${snap.active_option}|${generation}`)}
      {@const stored = option?.inputs[spec.name]}
      <label>
        {spec.name}
        <span class="row">
          <input
            type="text"
            inputmode="decimal"
            use:fill={stored ? String(stored.value) : ""}
            oninput={(e) => typed(spec.name, e.currentTarget.value)}
          />
          <span class="unit">{spec.unit ?? ""}</span>
        </span>
      </label>
      {#each messagesFor(spec.name) as m}
        <div class="msg {m.severity}">{m.text}</div>
      {/each}
    {/each}
    <p class="note">The full input form arrives with the left-panel ticket.</p>
  {/if}
</div>

<style>
  .panel { padding: 10px 12px; }
  h2 { font-family: var(--serif); font-size: 16px; margin: 0 0 8px; }
  label { display: block; font-size: 12px; color: var(--dim); margin: 8px 0 2px; }
  .row { display: flex; gap: 6px; align-items: center; }
  input, select { width: 100%; padding: 4px 6px; font: inherit; color: var(--ink); }
  .unit { min-width: 2em; }
  .msg { font-size: 12px; margin-top: 2px; }
  .msg.error { color: #a02a1a; }
  .msg.warning { color: #8a6000; }
  .note { color: var(--dim); font-size: 12px; margin-top: 16px; }
</style>
