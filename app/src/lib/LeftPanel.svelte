<script lang="ts">
  import type { OptionEvaluation, Schema, Snapshot } from "./api";
  import InputField from "./InputField.svelte";
  import VesselClass from "./VesselClass.svelte";

  // The left panel. The groups, their order, the labels, the choices and the ranges all come from
  // the schema. An input is drawn when the Option holds it, so the panel needs no rule for which
  // inputs a Cargo Containment uses: Rust sends only those that apply.
  let { snap, schema, evaluation, generation, onPick, onEdit, onReset, onFormation, onOption, onProfile }: {
    snap: Snapshot | null;
    schema: Schema | null;
    evaluation: OptionEvaluation | undefined;
    /** Changes when the fields must be drawn afresh. */
    generation: number;
    onPick: (name: string) => void;
    onEdit: (name: string, value: unknown) => void;
    onReset: (name: string) => void;
    onFormation: (kind: string, abreast: number | null, inLine: number | null) => void;
    onOption: (action: "add" | "duplicate" | "rename" | "delete", newName?: string) => void;
    onProfile: (action: "load" | "builtin") => void;
  } = $props();

  let option = $derived(snap?.design.options.find((o) => o.name === snap!.active_option));
  let groups = $derived(
    (schema?.groups ?? []).map((g) => ({
      name: g,
      inputs: (schema?.inputs ?? []).filter(
        (i) => i.group === g && option?.inputs[i.name] !== undefined
          && i.kind !== "vessel_scheme" && i.kind !== "vessel_class", // drawn by VesselClass
      ),
    })).filter((g) => g.inputs.length > 0 || g.name === VESSEL_CLASS_GROUP),
  );
  const VESSEL_CLASS_GROUP = "Vessel Class";
  const specOf = (name: string) => schema?.inputs.find((i) => i.name === name);
  let classMessages = $derived([...messagesFor("vessel_scheme"), ...messagesFor("vessel_class")]);

  let renaming = $state(false);
  let renameText = $state("");
  let confirmDelete = $state(false);
  let only = $derived((snap?.design.options.length ?? 0) <= 1);

  function messagesFor(name: string) {
    return (evaluation?.messages ?? []).filter((m) => m.inputs.includes(name));
  }
  function startRename() {
    renameText = snap?.active_option ?? "";
    renaming = true;
    confirmDelete = false;
  }
  function endRename(ok: boolean) {
    renaming = false;
    if (ok && renameText.trim() && renameText.trim() !== snap?.active_option) onOption("rename", renameText);
  }
  // Optionally-open groups are display state: closed groups stay closed across edits.
  let closed = $state<Record<string, boolean>>({});
</script>

<div class="panel">
  {#if snap}
    <section class="options">
      <h2>Option</h2>
      <select aria-label="Active Option" value={snap.active_option} onchange={(e) => onPick(e.currentTarget.value)}>
        {#each snap.design.options as o (o.name)}
          <option value={o.name}>{o.name}</option>
        {/each}
      </select>
      <div class="buttons">
        <button onclick={() => onOption("add")}>Add</button>
        <button onclick={() => onOption("duplicate")}>Duplicate</button>
        <button onclick={startRename}>Rename</button>
        <button onclick={() => (confirmDelete = true)} disabled={only} title={only ? "A Design needs at least one Option" : ""}>Delete</button>
      </div>
      {#if renaming}
        <div class="inline">
          <input
            type="text"
            aria-label="New Option name"
            bind:value={renameText}
            onkeydown={(e) => { if (e.key === "Enter") endRename(true); if (e.key === "Escape") endRename(false); }}
          />
          <button onclick={() => endRename(true)}>OK</button>
          <button onclick={() => endRename(false)}>Cancel</button>
        </div>
      {/if}
      {#if confirmDelete}
        <div class="inline confirm" role="alert">
          Delete "{snap.active_option}"?
          <button onclick={() => { confirmDelete = false; onOption("delete"); }}>Delete</button>
          <button onclick={() => (confirmDelete = false)}>Cancel</button>
        </div>
      {/if}
    </section>

    <section class="profile">
      <h2>Defaults Profile</h2>
      <div class="pname">{snap.profile}</div>
      <div class="buttons">
        <button onclick={() => onProfile("load")}>Load from file</button>
        <button onclick={() => onProfile("builtin")} disabled={snap.profile === "built-in"}>Built-in</button>
      </div>
    </section>

    {#each groups as g (g.name)}
      <details open={!closed[g.name]} ontoggle={(e) => (closed[g.name] = !e.currentTarget.open)}>
        <summary>{g.name}</summary>
        {#if schema?.group_notes?.[g.name]}<p class="gnote" data-testid="group-note">{schema.group_notes[g.name]}</p>{/if}
        {#if g.name === VESSEL_CLASS_GROUP}
          <VesselClass
            schemeSpec={specOf("vessel_scheme")}
            classSpec={specOf("vessel_class")}
            inputs={option!.inputs}
            summary={evaluation?.vessel_class}
            messages={classMessages}
            onEdit={(name, v) => onEdit(name, v)}
          />
        {/if}
        {#each g.inputs as spec (`${spec.name}|${snap.active_option}|${generation}`)}
          <InputField
            {spec}
            stored={option!.inputs[spec.name]}
            messages={messagesFor(spec.name)}
            onEdit={(v) => onEdit(spec.name, v)}
            onReset={() => onReset(spec.name)}
            {onFormation}
          />
        {/each}
      </details>
    {/each}
  {/if}
</div>

<style>
  .panel { padding: 10px 12px; }
  h2 { font-family: var(--serif); font-size: 14px; margin: 10px 0 4px; }
  select, input[type="text"] { width: 100%; padding: 4px 6px; font: inherit; box-sizing: border-box; }
  .buttons { display: flex; gap: 4px; margin-top: 6px; flex-wrap: wrap; }
  .inline { display: flex; gap: 4px; margin-top: 6px; align-items: center; font-size: 13px; }
  .confirm { background: #fbe4e1; padding: 4px 6px; }
  .pname { font-size: 13px; }
  details { border-top: 1px solid var(--line); margin-top: 10px; padding-top: 4px; }
  .gnote { margin: 4px 0; font-size: 12px; color: var(--dim); }
  summary { font-family: var(--serif); color: var(--navy); font-size: 15px; cursor: pointer; }
</style>
