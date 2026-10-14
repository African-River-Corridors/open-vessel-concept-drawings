<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "./lib/api";
  import type { Evaluation, Schema, Snapshot, OptionEvaluation } from "./lib/api";
  import LeftPanel from "./lib/LeftPanel.svelte";
  import MainPanel from "./lib/MainPanel.svelte";
  import BottomPanel from "./lib/BottomPanel.svelte";
  import Notice from "./lib/Notice.svelte";

  // Display state only: the Design, its checks and its results all come from Rust.
  let snap = $state<Snapshot | null>(null);
  let schema = $state<Schema | null>(null);
  let evaluation = $state<Evaluation | null>(null);
  /** The last Option evaluation that had results, per Option name, shown "out of date" on a hard error. */
  let lastGood = $state<Record<string, OptionEvaluation>>({});
  let error = $state<string | null>(null);
  let bannerDismissed = $state(false);
  let generation = $state(0);
  /** The first-run notice: open on first start, and again from About. */
  let noticeOpen = $state(false);
  let noticeIsFirstRun = $state(false);

  let leftWidth = $state(340);
  let bottomHeight = $state(260);

  const EVALUATE_PAUSE_MS = 150;
  let pending: ReturnType<typeof setTimeout> | undefined;
  let queue: Array<{ send: () => Promise<Snapshot>; redraw: boolean }> = [];

  async function guard<T>(work: () => Promise<T>): Promise<T | undefined> {
    try {
      error = null;
      return await work();
    } catch (e) {
      error = String(e);
      return undefined;
    }
  }

  async function runEvaluation() {
    const result = await api.evaluate();
    for (const o of result.options) {
      if (o.outcome.status === "results") lastGood[o.name] = o;
    }
    evaluation = result;
  }

  /** Take a new Design state: forget the last good results of a different Design. */
  async function adopt(next: Snapshot | null | undefined, newDesign = false, redraw = newDesign) {
    if (!next) return;
    if (newDesign) {
      lastGood = {};
      bannerDismissed = false;
    }
    if (redraw) generation += 1;
    snap = next;
    await runEvaluation();
  }

  async function fileAction(action: () => Promise<Snapshot | null>, newDesign: boolean) {
    await guard(async () => {
      await flush();
      const next = await action();
      if (next) await adopt(next, newDesign);
    });
  }

  /** Send queued edits and evaluate, now. */
  async function flush() {
    clearTimeout(pending);
    pending = undefined;
    const work = queue;
    queue = [];
    if (work.length === 0) return;
    let last: Snapshot | undefined;
    for (const w of work) last = await w.send();
    await adopt(last, false, work.some((w) => w.redraw));
  }

  /** An edit: send it after a pause in typing (about 150 ms), then evaluate the whole Design. */
  function edit(send: () => Promise<Snapshot>, redraw = false) {
    queue.push({ send, redraw });
    clearTimeout(pending);
    pending = setTimeout(() => guard(flush), EVALUATE_PAUSE_MS);
  }

  /** Option and profile commands: send what is queued, run the command, evaluate. */
  async function command(run: () => Promise<Snapshot | null>, refetchSchema = false) {
    await guard(async () => {
      await flush();
      const next = await run();
      if (!next) return;
      if (refetchSchema) schema = await api.inputSchema();
      await adopt(next, false, true);
    });
  }

  function optionAction(action: "add" | "duplicate" | "rename" | "delete", newName?: string) {
    const name = snap?.active_option ?? "";
    if (action === "add") command(api.addOption);
    else if (action === "duplicate") command(() => api.duplicateOption(name));
    else if (action === "rename") command(() => api.renameOption(name, newName ?? ""));
    else command(() => api.deleteOption(name));
  }

  function onKey(e: KeyboardEvent) {
    if (!(e.metaKey || e.ctrlKey)) return;
    const k = e.key.toLowerCase();
    if (k === "s") {
      e.preventDefault();
      fileAction(e.shiftKey ? api.saveAs : api.save, false);
    } else if (k === "o") {
      e.preventDefault();
      fileAction(api.openFile, true);
    } else if (k === "n") {
      e.preventDefault();
      fileAction(api.newFile, true);
    }
  }

  async function closeNotice() {
    noticeOpen = false;
    if (noticeIsFirstRun) {
      noticeIsFirstRun = false;
      await guard(api.acceptNotice);
    }
  }

  onMount(() => {
    guard(async () => {
      if (!(await api.noticeSeen())) {
        noticeIsFirstRun = true;
        noticeOpen = true;
      }
      schema = await api.inputSchema();
      await adopt(await api.snapshot(), true);
    });
  });

  function drag(axis: "x" | "y") {
    return (down: PointerEvent) => {
      const target = down.currentTarget as HTMLElement;
      target.setPointerCapture(down.pointerId);
      const move = (e: PointerEvent) => {
        if (axis === "x") leftWidth = Math.min(Math.max(e.clientX, 200), window.innerWidth - 300);
        else bottomHeight = Math.min(Math.max(window.innerHeight - e.clientY, 100), window.innerHeight - 160);
      };
      const up = () => {
        target.removeEventListener("pointermove", move);
        target.removeEventListener("pointerup", up);
      };
      target.addEventListener("pointermove", move);
      target.addEventListener("pointerup", up);
    };
  }

  let active = $derived(snap?.active_option ?? "");
  let activeEval = $derived(evaluation?.options.find((o) => o.name === active));
  /** The inputs of the active Option that have a hard error, with their schema labels. */
  let blockers = $derived.by(() => {
    const names = new Set<string>();
    for (const m of activeEval?.messages ?? []) if (m.severity === "error") m.inputs.forEach((n) => names.add(n));
    return [...names].map((name) => ({ name, label: schema?.inputs.find((i) => i.name === name)?.label ?? name }));
  });
</script>

<svelte:window onkeydown={onKey} />

{#if noticeOpen}<Notice onClose={closeNotice} />{/if}

<div class="shell">
  <header>
    <button onclick={() => fileAction(api.newFile, true)}>New</button>
    <button onclick={() => fileAction(api.openFile, true)}>Open</button>
    <button onclick={() => fileAction(api.save, false)}>Save</button>
    <button onclick={() => fileAction(api.saveAs, false)}>Save as</button>
    <button onclick={() => (noticeOpen = true)}>About</button>
    <span class="grow"></span>
    {#if snap}
      <span class="meta">{snap.file_name}{snap.dirty ? " (unsaved)" : ""}</span>
      <span class="meta">Defaults: {snap.profile}</span>
      <span class="meta" data-testid="engine-version">Engine {snap.engine_version}</span>
    {/if}
  </header>

  {#if error}
    <div class="banner error" role="alert">
      {error}
      <button onclick={() => (error = null)}>Dismiss</button>
    </div>
  {/if}
  {#each evaluation?.file_messages ?? [] as m}
    <div class="banner {m.severity === 'error' ? 'error' : 'warn'}" role="status">{m.text}</div>
  {/each}
  {#if snap && snap.hash_changed.length > 0 && !bannerDismissed}
    <div class="banner warn" role="status">
      Results changed since this file was saved. Options affected: {snap.hash_changed.join(", ")}.
      <button onclick={() => (bannerDismissed = true)}>Dismiss</button>
    </div>
  {/if}

  <div class="body" style:grid-template-columns="{leftWidth}px 6px 1fr" style:grid-template-rows="1fr 6px {bottomHeight}px">
    <section class="left"><LeftPanel {snap} {schema} {generation} evaluation={activeEval}
      onPick={(name) => command(() => api.setActiveOption(name))}
      onEdit={(name, value) => {
        const option = active;
        // A number being typed stays in its box; a choice, switch or "none" redraws the form.
        edit(() => api.setInput(option, name, value), typeof value !== "number");
      }}
      onReset={(name) => { const option = active; edit(() => api.resetInput(option, name), true); }}
      onFormation={(kind, abreast, inLine) => {
        const option = active;
        edit(() => api.setFormation(option, kind, abreast, inLine), false);
      }}
      onOption={optionAction}
      onProfile={(a) => command(a === "load" ? api.loadProfile : api.useBuiltInProfile, true)} /></section>
    <div class="handle v" role="separator" aria-orientation="vertical" onpointerdown={drag("x")}></div>
    <section class="main"><MainPanel {evaluation} {active} {lastGood} {blockers} /></section>
    <div class="handle h" role="separator" aria-orientation="horizontal" onpointerdown={drag("y")}></div>
    <section class="bottom"><BottomPanel {evaluation} {active} {lastGood} {blockers} onActivate={(name) => command(() => api.setActiveOption(name))} /></section>
  </div>
</div>

<style>
  .shell { display: flex; flex-direction: column; height: 100vh; }
  header { display: flex; gap: 6px; align-items: center; padding: 6px 10px; border-bottom: 1px solid var(--line); background: var(--bar); }
  .grow { flex: 1; }
  .meta { color: var(--dim); font-size: 12px; margin-left: 12px; }
  .banner { padding: 6px 10px; font-size: 13px; }
  .banner.error { background: #fbe4e1; color: #7a1d12; }
  .banner.warn { background: #fdf1d3; color: #6b4a00; }
  .body { flex: 1; display: grid; min-height: 0; }
  .body > section { overflow: auto; min-width: 0; min-height: 0; }
  .left { grid-column: 1; grid-row: 1 / 4; }
  .handle.v { grid-column: 2; grid-row: 1 / 4; cursor: col-resize; background: var(--line); }
  .main { grid-column: 3; grid-row: 1; }
  .body > .main { overflow: hidden; }
  .handle.h { grid-column: 3; grid-row: 2; cursor: row-resize; background: var(--line); }
  .bottom { grid-column: 3; grid-row: 3; }
  .handle:hover { background: var(--gold); }
</style>
