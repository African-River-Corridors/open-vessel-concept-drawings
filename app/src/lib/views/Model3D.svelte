<script lang="ts">
  import { onMount } from "svelte";
  import { platform, type Meshes, type SpaceInfo } from "../platform";
  import { watchTheme } from "../chartTheme";
  import { FAMILY_TOKEN, familyOf, fixedView, kindLabelId, orbitStep, type FixedView } from "../model3d";
  import { num, t } from "../i18n.svelte";

  // The 3D view of the Vessel Model (spec 0008, issue #458). The core builds the meshes; this view only
  // draws them (three.js, loaded when the view first opens) and holds no geometry of its own. Layers:
  // the hull, the decks, the bulkheads and one per kind of Space. Click a Space (or pick it in the list)
  // to read its volume and centroid, as the core gives them. Colours come from the CSS tokens, read again
  // when the theme changes.
  let { option }: { option: string } = $props();

  type Layer = { id: string; label: string };
  let host = $state<HTMLDivElement>();
  let status = $state<"loading" | "ready" | "no-webgl" | "failed">("loading");
  let problem = $state("");
  let spaces = $state<SpaceInfo[]>([]);
  let visible = $state<Record<string, boolean>>({ hull: true, deck: true, bulkhead: true });
  let selected = $state<SpaceInfo | null>(null);
  /** Kinds of Space present, in the order the core lists them. */
  let kinds = $derived([...new Set(spaces.map((s) => s.kind))]);
  const layers = $derived<Layer[]>([
    { id: "hull", label: t("app-3d-layer-hull") },
    { id: "deck", label: t("app-3d-layer-deck") },
    { id: "bulkhead", label: t("app-3d-layer-bulkhead") },
  ]);
  const kindName = (kind: string) => {
    const id = kindLabelId(kind);
    return id ? t(id) : kind;
  };
  const sideName = (side: string) => t(`app-3d-side-${side}`);
  /** An inner layer is on, so the hull is drawn translucent. */
  let innerOn = $derived(visible.deck || visible.bulkhead || kinds.some((k) => visible[`kind:${k}`]));

  // Set by onMount, called by the template.
  let apply: () => void = () => {};
  let selectByName: (name: string | null) => void = () => {};
  let goTo: (v: FixedView) => void = () => {};
  let reset: () => void = () => {};
  let onKey: (e: KeyboardEvent) => void = () => {};

  $effect(() => {
    // Any change of a layer, or of the translucency, is drawn.
    void visible;
    void innerOn;
    apply();
  });

  onMount(() => {
    let stop = false;
    let cleanup = () => {};
    (async () => {
      let data: Meshes;
      try {
        const p = platform();
        data = await p.meshes(option);
        const names = data.groups.filter((g) => g.layer === "space").map((g) => g.id);
        spaces = await Promise.all(names.map((n) => p.spaceInfo(option, n)));
        for (const s of spaces) visible[`kind:${s.kind}`] ??= true;
      } catch (e) {
        problem = String(e);
        status = "failed";
        return;
      }
      if (stop || !host) return;
      const THREE = await import("three");
      const { OrbitControls } = await import("three/examples/jsm/controls/OrbitControls.js");
      if (stop || !host) return;
      let renderer: InstanceType<typeof THREE.WebGLRenderer>;
      try {
        renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
      } catch {
        status = "no-webgl";
        return;
      }
      status = "ready";
      const canvas = renderer.domElement;
      canvas.tabIndex = 0;
      canvas.dataset.testid = "model3d-canvas";
      canvas.setAttribute("aria-label", t("app-3d-canvas-label"));
      host.appendChild(canvas);

      const token = (name: string) => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
      const scene = new THREE.Scene();
      const camera = new THREE.PerspectiveCamera(35, 1, 0.1, 5000);
      camera.up.set(0, 0, 1);
      scene.add(new THREE.AmbientLight(0xffffff, 1.4));
      const sun = new THREE.DirectionalLight(0xffffff, 2.2);
      sun.position.set(-30, -50, 80);
      scene.add(sun);

      // One mesh per group, all on the one vertex buffer of the core.
      const positions = new THREE.BufferAttribute(data.positions, 3);
      const meshes = new Map<string, InstanceType<typeof THREE.Mesh>>();
      const space = new Map<InstanceType<typeof THREE.Object3D>, string>();
      const root = new THREE.Group();
      scene.add(root);
      for (const g of data.groups) {
        if (g.layer === "waterline") continue;
        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute("position", positions);
        geometry.setIndex(new THREE.BufferAttribute(data.indices.subarray(g.start, g.start + g.count), 1));
        geometry.computeVertexNormals();
        const material = new THREE.MeshStandardMaterial({ side: THREE.DoubleSide, roughness: 0.8, metalness: 0 });
        const mesh = new THREE.Mesh(geometry, material);
        mesh.userData = { layer: g.layer, id: g.id };
        root.add(mesh);
        meshes.set(`${g.layer}:${g.id}`, mesh);
        if (g.layer === "space") space.set(mesh, g.id);
      }
      const box = new THREE.Box3().setFromObject(root);
      const centre = box.getCenter(new THREE.Vector3());
      const size = box.getSize(new THREE.Vector3());
      const reach = Math.max(size.x, size.y, size.z) * 1.1;
      const controls = new OrbitControls(camera, canvas);
      controls.enableDamping = false;

      const kindOf = (name: string) => spaces.find((s) => s.name === name)?.kind ?? "void";
      const colour = (mesh: InstanceType<typeof THREE.Mesh>) => {
        const { layer, id } = mesh.userData as { layer: string; id: string };
        if (layer === "space") return token(FAMILY_TOKEN[familyOf(kindOf(id))]);
        if (layer === "hull") return token("--ink-3");
        if (layer === "deck") return token("--rule-strong");
        return token("--ink-2");
      };
      function paint() {
        renderer.setClearColor(new THREE.Color(token("--paper") || "#f6edd8"));
        for (const mesh of meshes.values()) {
          const { layer, id } = mesh.userData as { layer: string; id: string };
          const m = mesh.material as InstanceType<typeof THREE.MeshStandardMaterial>;
          m.color.set(colour(mesh));
          const on =
            layer === "space" ? visible[`kind:${kindOf(id)}`] !== false : visible[layer] !== false;
          mesh.visible = on;
          const translucent = layer === "hull" && innerOn;
          m.transparent = translucent || layer === "space" || layer === "deck";
          m.opacity = translucent ? 0.2 : layer === "space" ? 0.85 : layer === "deck" ? 0.6 : 1;
          m.depthWrite = !translucent;
          m.emissive.set(layer === "space" && selected?.name === id ? token("--gilt-mark") : "#000000");
          m.emissiveIntensity = 0.55;
        }
        controls.update();
        renderer.render(scene, camera);
        // What is drawn, for the end-to-end tests: the groups on, by layer.
        const shown = [...meshes.values()].filter((m) => m.visible).map((m) => `${m.userData.layer}:${m.userData.id}`);
        canvas.dataset.shown = JSON.stringify(shown);
      }
      apply = paint;

      function look(position: [number, number, number], up: [number, number, number]) {
        camera.up.set(...up);
        camera.position.set(...position);
        controls.target.copy(centre);
        controls.update();
        paint();
      }
      const home = (): [number, number, number] => [centre.x - reach * 0.5, centre.y + reach * 0.8, centre.z + reach * 0.5];
      reset = () => look(home(), [0, 0, 1]);
      goTo = (v) => {
        const f = fixedView(v, [centre.x, centre.y, centre.z], reach);
        look(f.position, f.up);
      };
      selectByName = (name) => {
        selected = name ? (spaces.find((s) => s.name === name) ?? null) : null;
        paint();
      };
      onKey = (e) => {
        const step = Math.PI / 36;
        const keys: Record<string, [number, number]> = {
          ArrowLeft: [step, 0],
          ArrowRight: [-step, 0],
          ArrowUp: [0, -step],
          ArrowDown: [0, step],
        };
        const d = keys[e.key];
        if (!d) return;
        e.preventDefault();
        const [x, y, z] = orbitStep(
          [camera.position.x, camera.position.y, camera.position.z],
          [controls.target.x, controls.target.y, controls.target.z],
          d[0],
          d[1],
        );
        camera.position.set(x, y, z);
        paint();
      };

      // A click that did not drag picks the nearest visible Space.
      const ray = new THREE.Raycaster();
      let down: [number, number] | null = null;
      const onDown = (e: PointerEvent) => (down = [e.clientX, e.clientY]);
      const onUp = (e: PointerEvent) => {
        if (!down || Math.hypot(e.clientX - down[0], e.clientY - down[1]) > 4) return;
        const r = canvas.getBoundingClientRect();
        ray.setFromCamera(new THREE.Vector2(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1), camera);
        const hit = ray.intersectObjects([...space.keys()].filter((o) => o.visible), false)[0];
        selectByName(hit ? (space.get(hit.object) ?? null) : null);
      };
      canvas.addEventListener("pointerdown", onDown);
      canvas.addEventListener("pointerup", onUp);
      controls.addEventListener("change", () => renderer.render(scene, camera));

      const resize = () => {
        if (!host) return;
        const w = Math.max(host.clientWidth, 100);
        const h = Math.max(host.clientHeight, 100);
        renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
        renderer.setSize(w, h, false);
        canvas.style.width = "100%";
        canvas.style.height = "100%";
        camera.aspect = w / h;
        camera.updateProjectionMatrix();
        paint();
      };
      const ro = new ResizeObserver(resize);
      ro.observe(host);
      const stopTheme = watchTheme(paint);
      reset();
      resize();

      cleanup = () => {
        stopTheme();
        ro.disconnect();
        canvas.removeEventListener("pointerdown", onDown);
        canvas.removeEventListener("pointerup", onUp);
        controls.dispose();
        for (const m of meshes.values()) {
          m.geometry.dispose();
          (m.material as InstanceType<typeof THREE.Material>).dispose();
        }
        renderer.dispose();
        canvas.remove();
      };
    })();
    return () => {
      stop = true;
      cleanup();
    };
  });
</script>

<div class="model" data-testid="model3d" data-status={status}>
  {#if status === "failed"}
    <p class="note">{t("app-3d-failed", { why: problem })}</p>
  {:else}
    <div class="stage">
      {#if status === "no-webgl"}
        <p class="note" data-testid="model3d-no-webgl">{t("app-3d-no-webgl")}</p>
      {:else}
        <div class="host" bind:this={host} onkeydown={onKey} role="application" aria-label={t("app-3d-canvas-label")}></div>
        {#if status === "loading"}<p class="note over">{t("app-3d-loading")}</p>{/if}
        <div class="bar">
          <button type="button" data-testid="model3d-reset" onclick={() => reset()}>{t("app-3d-reset")}</button>
          <button type="button" data-testid="model3d-view-profile" onclick={() => goTo("profile")}>{t("app-3d-view-profile")}</button>
          <button type="button" data-testid="model3d-view-plan" onclick={() => goTo("plan")}>{t("app-3d-view-plan")}</button>
          <button type="button" data-testid="model3d-view-body" onclick={() => goTo("body")}>{t("app-3d-view-body")}</button>
          <span class="hint">{t("app-3d-hint")}</span>
        </div>
      {/if}
    </div>
    <aside class="side">
      {#if status !== "no-webgl"}
        <h4>{t("app-3d-layers")}</h4>
        <ul class="layers" data-testid="model3d-layers">
          {#each layers as l (l.id)}
            <li><label><input type="checkbox" data-testid="layer-{l.id}" bind:checked={visible[l.id]} onchange={() => apply()} /> {l.label}</label></li>
          {/each}
          {#each kinds as k (k)}
            <li><label><input type="checkbox" data-testid="layer-kind-{k}" bind:checked={visible[`kind:${k}`]} onchange={() => apply()} /> <span class="chip" style="background: var({FAMILY_TOKEN[familyOf(k)]})"></span>{kindName(k)}</label></li>
          {/each}
          <li class="later"><label><input type="checkbox" disabled data-testid="layer-structure" /> {t("app-3d-layer-structure")} <span class="dim">({t("app-3d-structure-later")})</span></label></li>
        </ul>
      {/if}
      <h4>{t("app-3d-spaces")}</h4>
      {#if spaces.length === 0}
        <p class="note">{t("app-3d-no-spaces")}</p>
      {:else}
        <table data-testid="model3d-spaces">
          <thead>
            <tr><th>{t("app-3d-col-name")}</th><th>{t("app-3d-col-kind")}</th><th class="num">{t("app-3d-col-volume")}<span class="unit">&nbsp;(m³)</span></th></tr>
          </thead>
          <tbody>
            {#each spaces as s (s.name)}
              <tr class:on={selected?.name === s.name}>
                <th><button type="button" class="link" data-testid="space-{s.name}" onclick={() => selectByName(s.name)}>{s.name}</button></th>
                <td>{kindName(s.kind)}</td>
                <td class="num" title={String(s.volume_m3)}>{num(s.volume_m3, 1)}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
      {#if selected}
        <dl class="info" data-testid="space-info">
          <dt>{t("app-3d-col-name")}</dt><dd data-testid="space-info-name">{selected.name}</dd>
          <dt>{t("app-3d-col-kind")}</dt><dd>{kindName(selected.kind)}, {sideName(selected.side)}</dd>
          <dt>{t("app-3d-col-volume")}</dt><dd data-testid="space-info-volume" data-value={selected.volume_m3}>{num(selected.volume_m3, 1)} m³</dd>
          <dt>{t("app-3d-col-centroid")}</dt><dd data-testid="space-info-centroid">{num(selected.lcg_m, 2)}, {num(selected.tcg_m, 2)}, {num(selected.vcg_m, 2)} m</dd>
        </dl>
      {:else if status !== "no-webgl"}
        <p class="note">{t("app-3d-pick-hint")}</p>
      {/if}
    </aside>
  {/if}
</div>

<style>
  .model { height: 100%; display: flex; gap: 12px; min-height: 0; }
  .stage { flex: 3; min-width: 0; display: flex; flex-direction: column; gap: 6px; position: relative; }
  .host { flex: 1; min-height: 160px; background: var(--paper); border: 1px solid var(--rule); box-sizing: border-box; overflow: hidden; }
  .host:focus-within { outline: 2px solid var(--gilt-mark); outline-offset: 1px; }
  .host :global(canvas) { display: block; outline: none; }
  .over { position: absolute; top: 8px; left: 8px; margin: 0; }
  .bar { display: flex; flex-wrap: wrap; gap: 6px; align-items: center; font-size: 12px; }
  .bar button { padding: 1px 10px; background: var(--surface); color: var(--ink); border: 1px solid var(--rule-strong); border-radius: var(--radius); cursor: pointer; }
  .hint { color: var(--ink-3); margin-left: auto; }
  .side { flex: 1; min-width: 230px; max-width: 340px; overflow: auto; font-size: 12px; }
  h4 { font-family: var(--font-serif); font-size: 13px; margin: 0 0 4px; color: var(--ink); }
  .layers { list-style: none; margin: 0 0 12px; padding: 0; display: grid; gap: 2px; }
  .layers label { display: flex; align-items: center; gap: 6px; }
  .chip { width: 12px; height: 12px; border-radius: 2px; display: inline-block; border: 1px solid var(--rule-strong); }
  .later { color: var(--ink-3); }
  .dim { color: var(--ink-3); }
  .note { color: var(--ink-3); }
  table { border-collapse: collapse; width: 100%; }
  th { text-align: left; font-weight: normal; padding: 2px 8px 2px 0; white-space: nowrap; }
  thead th { color: var(--ink-3); }
  td { padding: 2px 6px; }
  .num { text-align: right; font-variant-numeric: tabular-nums; }
  th.num { text-align: right; }
  .unit { color: var(--ink-3); }
  tr.on { background: var(--accent-soft); }
  .link { background: none; border: 0; padding: 0; color: var(--accent); cursor: pointer; font: inherit; text-decoration: underline; }
  .info { display: grid; grid-template-columns: auto 1fr; gap: 2px 10px; margin: 10px 0 0; }
  .info dt { color: var(--ink-3); }
  .info dd { margin: 0; font-variant-numeric: tabular-nums; }
</style>
