<script lang="ts">
  // The first-run notice (a click-through, no warranty). It shows once per user and from About.
  let { onClose }: { onClose: () => void } = $props();
  let button: HTMLButtonElement;
  $effect(() => button?.focus());
</script>

<div class="veil">
  <div class="dialog" role="dialog" aria-modal="true" aria-labelledby="notice-title" data-testid="notice">
    <h2 id="notice-title">Before you start</h2>
    <p>OVCD is a concept design tool.</p>
    <p>Its results are not verified. Known engineering findings are under review.</p>
    <p>There is no warranty. OVCD is open source under the Apache License 2.0.</p>
    <button bind:this={button} onclick={onClose}>I understand</button>
  </div>
</div>

<style>
  .veil { position: fixed; inset: 0; background: rgba(20, 30, 50, 0.55); display: flex; align-items: center; justify-content: center; z-index: 10; }
  .dialog { background: #fff; border-top: 4px solid var(--gold); padding: 20px 26px; max-width: 460px; box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3); }
  h2 { font-family: var(--serif); color: var(--navy); margin: 0 0 10px; font-size: 20px; }
  p { margin: 8px 0; font-size: 15px; line-height: 1.45; }
  button { margin-top: 12px; padding: 6px 16px; font: inherit; }
</style>
