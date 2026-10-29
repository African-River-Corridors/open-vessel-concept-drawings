<script lang="ts">
  import { applyTheme, loadTheme, saveTheme, type ThemeChoice } from "./theme";
  import { t } from "./i18n.svelte";

  // Dark is the default; Auto follows the operating system. The choice is kept between runs.
  const CHOICES: { id: ThemeChoice; label: string }[] = [
    { id: "auto", label: "app-theme-auto" },
    { id: "light", label: "app-theme-light" },
    { id: "dark", label: "app-theme-dark" },
  ];

  let choice = $state<ThemeChoice>(loadTheme());

  function pick(next: ThemeChoice) {
    choice = next;
    saveTheme(next);
    applyTheme(next);
  }
</script>

<div class="theme" role="group" aria-label={t("app-theme")}>
  {#each CHOICES as c (c.id)}
    <button type="button" aria-pressed={choice === c.id} onclick={() => pick(c.id)}>{t(c.label)}</button>
  {/each}
</div>

<style>
  .theme { display: inline-flex; }
  button { background: transparent; font-size: var(--fs-13); border-radius: 0; margin-left: -1px; }
  button:first-child { border-radius: var(--radius) 0 0 var(--radius); margin-left: 0; }
  button:last-child { border-radius: 0 var(--radius) var(--radius) 0; }
  button[aria-pressed="true"],
  button[aria-pressed="true"]:hover:not(:disabled) { background: var(--accent-soft); border-color: var(--accent); position: relative; z-index: 1; }
  button:focus-visible { position: relative; z-index: 2; }
</style>
