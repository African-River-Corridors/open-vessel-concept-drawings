<script lang="ts">
  // Settings: the per-user choices that change no result (Display Language, theme) and the Defaults
  // Profile, kept out of the app bar and the input panel. Escape, Close or a click outside closes it.
  import LanguageControl from "./LanguageControl.svelte";
  import ThemeControl from "./ThemeControl.svelte";
  import { t } from "./i18n.svelte";

  let { profile, onProfile, onClose }: {
    /** The Defaults Profile in use; null before the first Design loads. */
    profile: string | null;
    onProfile: (action: "load" | "builtin") => void;
    onClose: () => void;
  } = $props();

  let button: HTMLButtonElement;
  $effect(() => button?.focus());

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") onClose();
  }
</script>

<svelte:window onkeydown={onKey} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions (Escape closes it) -->
<div class="veil" onclick={(e) => e.target === e.currentTarget && onClose()}>
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="settings-title" data-testid="settings-dialog">
    <h2 id="settings-title">{t("app-settings")}</h2>

    <section>
      <h3>{t("app-language")}</h3>
      <LanguageControl />
    </section>

    <section>
      <h3>{t("app-theme")}</h3>
      <ThemeControl />
    </section>

    {#if profile !== null}
      <section>
        <h3>{t("app-profile")}</h3>
        <div class="pname" data-testid="settings-profile">{profile}</div>
        <div class="buttons">
          <button onclick={() => onProfile("load")}>{t("app-profile-load")}</button>
          <button onclick={() => onProfile("builtin")} disabled={profile === "built-in"}>{t("app-profile-builtin")}</button>
        </div>
      </section>
    {/if}

    <div class="foot">
      <button bind:this={button} data-testid="settings-close" onclick={onClose}>{t("app-settings-close")}</button>
    </div>
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
    background: var(--surface);
    color: var(--ink);
    border-top: 4px solid var(--gilt-mark);
    border-radius: var(--radius-lg, 6px);
    padding: 20px 26px;
    width: min(420px, calc(100vw - 32px));
    box-sizing: border-box;
    box-shadow: 0 8px 30px color-mix(in srgb, var(--blueprint-deep, #1a3753) 40%, transparent);
  }
  h2 { font-family: var(--font-serif); font-weight: 600; color: var(--ink); margin: 0 0 12px; font-size: 20px; }
  h3 { font-family: var(--font-serif); font-weight: 600; color: var(--ink); margin: 0 0 6px; font-size: var(--fs-14); }
  section { border-top: var(--lw-thin) solid var(--rule); padding: 10px 0; }
  .pname { font-size: var(--fs-13); }
  .buttons { display: flex; gap: var(--sp-1); margin-top: 6px; flex-wrap: wrap; }
  .buttons button { font-size: var(--fs-13); }
  .foot { display: flex; justify-content: flex-end; border-top: var(--lw-thin) solid var(--rule); padding-top: 12px; }
</style>
