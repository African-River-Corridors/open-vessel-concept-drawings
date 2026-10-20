<script lang="ts">
  // The first-run notice (a click-through, no warranty). It shows once per user and from About.
  import { t } from "./i18n.svelte";

  let { onClose }: { onClose: () => void } = $props();
  let button: HTMLButtonElement;
  $effect(() => button?.focus());
</script>

<div class="veil">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="notice-title" data-testid="notice">
    <h2 id="notice-title">{t("app-notice-title")}</h2>
    <div class="tb">
      <div><span class="k">{t("app-notice-version")}</span><span class="v">{__APP_VERSION__}</span></div>
      <div><span class="k">{t("app-notice-licence")}</span><span class="v">Apache 2.0</span></div>
    </div>
    <p class="scope">▲ {t("app-notice-scope")}</p>
    <p>{t("app-notice-tool")}</p>
    <p>{t("app-notice-unverified")}</p>
    <p>{t("app-notice-warranty")}</p>
    <button bind:this={button} onclick={onClose}>{t("app-notice-accept")}</button>
  </div>
</div>

<style>
  .veil {
    position: fixed;
    inset: 0;
    background: color-mix(in srgb, var(--blueprint-deep, #1a3753) 55%, transparent);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10;
  }
  .dialog {
    background: var(--surface, #fbf6ea);
    color: var(--ink, #1c3550);
    border-top: 4px solid var(--gilt-mark, #8a6a35);
    border-radius: var(--radius-lg, 6px);
    padding: 20px 26px;
    max-width: 460px;
    box-shadow: 0 8px 30px color-mix(in srgb, var(--blueprint-deep, #1a3753) 40%, transparent);
  }
  h2 { font-family: var(--font-serif, Georgia, serif); font-weight: 600; color: var(--ink, #1c3550); margin: 0 0 12px; font-size: 20px; }
  p { margin: 8px 0; font-size: 15px; line-height: 1.45; }
  /* The title-block device: thin ruled cells with a small hand-lettered label over each value. */
  .tb { display: grid; grid-template-columns: 1fr 1fr; border: 1px solid var(--rule-strong, #8c7b56); }
  .tb > div { padding: 6px 10px; }
  .tb > div + div { border-left: 1px solid var(--rule-strong, #8c7b56); }
  .k {
    display: block;
    font: 400 11px var(--font-hand, "Architects Daughter", "Barlow Semi Condensed", sans-serif);
    font-style: oblique 10deg;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--ink-3, #56687a);
  }
  .v { font-size: 14px; font-weight: 500; font-variant-numeric: tabular-nums; }
  .scope {
    margin: 12px 0 4px;
    padding: 6px 10px;
    font-weight: 600;
    color: var(--warn, #73500b);
    background: var(--warn-soft, #f2e2bd);
  }
  button { margin-top: 12px; padding: 6px 16px; font: inherit; }
</style>
