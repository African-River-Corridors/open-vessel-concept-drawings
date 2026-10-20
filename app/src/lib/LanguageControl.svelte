<script lang="ts">
  import { LANGUAGES, chooseLanguage, language, t } from "./i18n.svelte";

  // The Display Language. It is display state only, kept per user: it changes no input, no result
  // and no saved file (ADR 0014). Each language is named in its own words.
</script>

<label class="lang">
  <span class="sr">{t("app-language")}</span>
  <select aria-label={t("app-language")} data-testid="language" value={language().tag} onchange={(e) => chooseLanguage(e.currentTarget.value)}>
    {#each LANGUAGES as l (l.tag)}
      <option value={l.tag} lang={l.tag}>{l.name}</option>
    {/each}
  </select>
</label>

<style>
  .lang { display: inline-flex; }
  select { font-size: var(--fs-13); }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); white-space: nowrap; }
</style>
