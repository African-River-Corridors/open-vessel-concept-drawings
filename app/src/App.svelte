<script lang="ts">
  import { onMount } from "svelte";
  import * as api from "./lib/api";
  import type { Evaluation, Schema, Snapshot, OptionEvaluation } from "./lib/api";
  import LeftPanel from "./lib/LeftPanel.svelte";
  import MainPanel from "./lib/MainPanel.svelte";
  import BottomPanel from "./lib/BottomPanel.svelte";
  import Notice from "./lib/Notice.svelte";
  import ThemeControl from "./lib/ThemeControl.svelte";
  import { loadPanels, savePanels } from "./lib/panels";
  import { list, say, schemaText, setInputLabels, t } from "./lib/i18n.svelte";

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
  /** The browser build: shows a label and a link to the desktop app. */
  const web = api.isWeb();

  let leftWidth = $state(340);
  let bottomHeight = $state(260);
  // Both side panels are shown unless the user hid one last time.
  const saved = loadPanels();
  let showInput = $state(saved.input);
  let showOutputs = $state(saved.outputs);
  $effect(() => savePanels({ input: showInput, outputs: showOutputs }));

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
      if (schema) setInputLabels(Object.fromEntries(schema.inputs.map((i) => [i.name, i.label])));
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
      setInputLabels(Object.fromEntries(schema.inputs.map((i) => [i.name, i.label])));
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
  /** The active Option's design speed, from the Design session's inputs. */
  let designSpeed = $derived.by(() => {
    const v = snap?.design.options.find((o) => o.name === active)?.inputs.max_speed_kn?.value;
    return typeof v === "number" ? v : null;
  });
  let activeEval = $derived(evaluation?.options.find((o) => o.name === active));
  /** The inputs of the active Option that have a hard error, with their schema labels. */
  let blockers = $derived.by(() => {
    const names = new Set<string>();
    for (const m of activeEval?.messages ?? []) if (m.severity === "error") m.inputs.forEach((n) => names.add(n));
    return [...names].map((name) => {
      const label = schema?.inputs.find((i) => i.name === name)?.label;
      return { name, label: label ? schemaText.input(name, label) : name };
    });
  });
</script>

<svelte:window onkeydown={onKey} />

{#if noticeOpen}<Notice onClose={closeNotice} />{/if}

<div class="shell">
  <header>
    <!-- The 32 px OVCD mark, drawn in the text colour so it reads in both themes. -->
    <svg class="mark" viewBox="0 0 64 64" width="32" height="32" aria-hidden="true" focusable="false">
      <g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round">
        <path d="M5.5 34 H58.5" stroke-width="3" />
        <path d="M32 43.5 C24.5 43.5 19 37 18.8 14.4" stroke-width="3" />
        <path d="M32 48 C37.8 45.8 44.6 34 45.6 12.8" stroke-width="3" />
        <path d="M32 48 H18 Q10 48 10 40 V16 M32 48 H46 Q54 48 54 40 V16" stroke-width="5" />
      </g>
    </svg>
    <span class="brand">OVCD</span>
    <button onclick={() => fileAction(api.newFile, true)}>{t("app-new")}</button>
    <button onclick={() => fileAction(api.openFile, true)}>{t("app-open")}</button>
    <button onclick={() => fileAction(api.save, false)}>{t("app-save")}</button>
    <button onclick={() => fileAction(api.saveAs, false)}>{t("app-save-as")}</button>
    <button onclick={() => (noticeOpen = true)}>{t("app-about")}</button>
    {#if web}<span class="web-badge" data-testid="web-label">{t("app-web-version")}</span> <a href="/ovcd#install">{t("app-download-desktop")}</a>{/if}
    <span class="grow"></span>
    <button type="button" aria-label={t(showInput ? "app-input-panel-hide" : "app-input-panel-show")} aria-pressed={showInput} onclick={() => (showInput = !showInput)}>{t("app-input-panel")}</button>
    <button type="button" aria-label={t(showOutputs ? "app-outputs-panel-hide" : "app-outputs-panel-show")} aria-pressed={showOutputs} onclick={() => (showOutputs = !showOutputs)}>{t("app-outputs-panel")}</button>
    <ThemeControl />
    {#if snap}
      <span class="meta file">{snap.dirty ? t("app-file-unsaved", { file: snap.file_name }) : snap.file_name}</span>
      <span class="meta">{t("app-header-profile", { profile: snap.profile })}</span>
      <span class="meta" data-testid="engine-version">{t("app-engine-version", { version: snap.engine_version })}</span>
    {/if}
  </header>

  {#if error}
    <div class="banner error" role="alert">
      <span class="msg">{error}</span>
      <button onclick={() => (error = null)}>{t("app-dismiss")}</button>
    </div>
  {/if}
  {#each evaluation?.file_messages ?? [] as m}
    <div class="banner {m.severity === 'error' ? 'error' : 'warn'}" role="status">{say(m.text, null, m.msg)}</div>
  {/each}
  {#each snap?.migration_notes ?? [] as note}
    <div class="banner warn" role="status" data-testid="migration-note">{note}</div>
  {/each}
  {#if snap && snap.hash_changed.length > 0 && !bannerDismissed}
    <div class="banner warn" role="status">
      {t("app-hash-changed", { options: list(snap.hash_changed, "unit") })}
      <button onclick={() => (bannerDismissed = true)}>{t("app-dismiss")}</button>
    </div>
  {/if}

  <div class="body" style:grid-template-columns={showInput ? `${leftWidth}px 6px 1fr` : "0 0 1fr"} style:grid-template-rows={showOutputs ? `1fr 6px ${bottomHeight}px` : "1fr 0 0"}>
    <section class="left" hidden={!showInput}><LeftPanel {snap} {schema} {generation} evaluation={activeEval}
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
    <div class="handle v" hidden={!showInput} role="separator" aria-orientation="vertical" onpointerdown={drag("x")}></div>
    <section class="main"><MainPanel {evaluation} {active} {lastGood} {blockers} {designSpeed} /></section>
    <div class="handle h" hidden={!showOutputs} role="separator" aria-orientation="horizontal" onpointerdown={drag("y")}></div>
    <section class="bottom" hidden={!showOutputs}><BottomPanel {evaluation} {active} {lastGood} {blockers} onActivate={(name) => command(() => api.setActiveOption(name))} /></section>
  </div>
</div>

<style>
  .shell { display: flex; flex-direction: column; height: 100vh; }
  header {
    display: flex; flex-wrap: wrap; gap: var(--sp-2); align-items: center; box-sizing: border-box;
    min-height: 40px; padding: 0 var(--sp-3);
    background: var(--paper); color: var(--ink);
    border-bottom: var(--lw-thick) solid var(--gilt-mark);
  }
  .mark { color: var(--ink); flex: none; }
  .brand { font-weight: 600; font-size: var(--fs-14); letter-spacing: 0.08em; margin-right: var(--sp-2); }
  header button { background: var(--surface); }
  header button:hover:not(:disabled) { background: var(--surface-2); }
  header button[aria-pressed] { background: transparent; }
  header button[aria-pressed="true"],
  header button[aria-pressed="true"]:hover:not(:disabled) { background: var(--accent-soft); border-color: var(--accent); }
  .grow { flex: 1; }
  .meta { color: var(--ink-3); font-size: var(--fs-12); margin-left: var(--sp-2); }
  .meta.file { font-family: var(--font-mono); }
  header :global(.web-badge) {
    font-size: var(--fs-11); text-transform: uppercase; letter-spacing: 0.08em; padding: 1px var(--sp-2);
    background: var(--surface-2); color: var(--ink-2); border: var(--lw-thin) solid var(--rule-strong); border-radius: var(--radius);
  }
  header :global(a) { color: var(--accent); font-size: var(--fs-13); }
  .banner {
    display: flex; align-items: baseline; gap: var(--sp-2); padding: var(--sp-1) var(--sp-3); font-size: var(--fs-13);
    border-bottom: var(--lw-thin) solid var(--rule);
  }
  .banner::before { flex: none; }
  .banner .msg { flex: 1; }
  .banner button { margin-left: var(--sp-2); font-size: var(--fs-13); padding: 1px var(--sp-2); }
  .banner.error { background: var(--fail-soft); color: var(--fail); }
  .banner.error::before { content: "\2715"; }
  .banner.warn { background: var(--warn-soft); color: var(--warn); }
  .banner.warn::before { content: "\25B2"; }
  .body { flex: 1; display: grid; min-height: 0; }
  .body > section { overflow: auto; min-width: 0; min-height: 0; }
  .left { grid-column: 1; grid-row: 1 / 4; }
  .handle.v { grid-column: 2; grid-row: 1 / 4; cursor: col-resize; background: var(--rule); }
  .main { grid-column: 3; grid-row: 1; }
  .body > .main { overflow: hidden; }
  .handle.h { grid-column: 3; grid-row: 2; cursor: row-resize; background: var(--rule); }
  .bottom { grid-column: 3; grid-row: 3; }
  .body > [hidden] { display: none; }
  .handle:hover { background: var(--gilt-mark); }
</style>
