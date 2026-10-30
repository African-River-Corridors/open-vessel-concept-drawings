<script lang="ts">
  import { onMount } from "svelte";
  import { platform, type DeckItem, type DeckItems, type Meshes, type PropulsorPart, type SectionOutline, type SpaceInfo, type Waterline } from "../platform";
  import { watchTheme } from "../chartTheme";
  import { FAMILY_TOKEN, PRESETS, VIEW_MIRROR, closedSolid, familyOf, fixedView, flatShaded, groupSlice, homeView, kindLabelId, presetOf, presetState, seaPlane, orbitStep, toView, wheelhouseDraw, type FixedView, type Preset, type Vec3, type WheelhouseShown } from "../model3d";
  import { num, t } from "../i18n.svelte";

  // The 3D view of the Vessel Model (spec 0008, issue #458). The core builds the meshes; this view only
  // draws them (three.js, loaded when the view first opens) and holds no geometry of its own. Layers:
  // the hull, the decks, the bulkheads and one per kind of Space. Click a Space (or pick it in the list)
  // to read its volume and centroid, as the core gives them. Colours come from the CSS tokens, read again
  // when the theme changes. It opens on the outside of the vessel (spec 0009, "The 3D view"): the hull
  // opaque and flat shaded, its edges drawn, the Spaces off, from the aft-starboard quarter. A sea plane
  // lies at the shown loading condition's waterplane, and the hull below it takes the wet colour.
  // Spec 0009 adds the tanks, their domes, the weather shield and the Deck Items; three View Presets
  // (Exterior on open, Cargo, Arrangement); a half-model cut and a deck cut, whose cut faces take
  // `--model-cut`; the wheelhouse lowered, raised or both; and a pick panel for each new part.
  let { option }: { option: string } = $props();

  type Layer = { id: string; label: string };
  let host = $state<HTMLDivElement>();
  let status = $state<"loading" | "ready" | "no-webgl" | "failed">("loading");
  let problem = $state("");
  let spaces = $state<SpaceInfo[]>([]);
  let visible = $state<Record<string, boolean>>({ hull: true, deck: true, bulkhead: true });
  let selected = $state<SpaceInfo | null>(null);
  // The Deck Items and the wheelhouse's lift, from the core (spec 0009).
  let deck = $state<DeckItems>({ items: [], wheelhouse: null });
  let selectedItem = $state<DeckItem | null>(null);
  // The propulsor parts, from the core's Propulsor Layout (spec 0009).
  let propParts = $state<PropulsorPart[]>([]);
  let selectedProp = $state<PropulsorPart | null>(null);
  // The cuts and the hull's translucency a preset sets.
  let halfCut = $state(false);
  let deckCut = $state(false);
  let hullTranslucent = $state(false);
  let wheelhouseShown = $state<WheelhouseShown>("lowered");
  /** The layers of the model's groups, in the order they come. */
  let groupLayers = $state<string[]>([]);
  // The section plane and the waterline (issue #459).
  let sectionOn = $state(false);
  let sectionX = $state(0);
  let length = $state(0);
  let waterlines = $state<Waterline[]>([]);
  let waterline = $state("");
  const shownWaterline = $derived(waterlines.find((w) => w.name === waterline) ?? null);
  /** Kinds of Space present, in the order the core lists them. */
  let kinds = $derived([...new Set(spaces.map((s) => s.kind))]);
  const layers = $derived<Layer[]>([
    { id: "hull", label: t("app-3d-layer-hull") },
    { id: "deck", label: t("app-3d-layer-deck") },
    { id: "bulkhead", label: t("app-3d-layer-bulkhead") },
    ...(groupLayers.includes("dome") ? [{ id: "dome", label: t("app-3d-layer-dome") }] : []),
    ...(groupLayers.includes("weather_shield") ? [{ id: "weather_shield", label: t("app-3d-layer-weather-shield") }] : []),
    ...(groupLayers.includes("deck_item") ? [{ id: "deck_item", label: t("app-3d-layer-deck-item") }] : []),
    ...(groupLayers.includes("propulsor") ? [{ id: "propulsor", label: t("app-3d-layer-propulsor") }] : []),
  ]);
  /** A propulsor part's name: its part, numbered as the layout numbers it. */
  const partName = (p: PropulsorPart) => t(`app-3d-part-${p.part.replaceAll("_", "-")}`) + (p.index === null ? "" : ` ${p.index + 1}`);
  /** The Propulsor type's name, as the inputs name it; the default stern block's thrusters are ducted azimuth units. */
  const typeName = (key: string) => t(key === "unspecified" ? "app-3d-propulsor-default-thrusters" : `choice-propulsor-type-${key.replaceAll("_", "-")}`);
  /** The preset the toggles match, or null: "Custom". */
  let preset = $derived(presetOf({ visible, halfCut, deckCut, sea: waterline !== "", hullTranslucent }, kinds, waterlines.length > 0));
  function choose(p: Preset) {
    const s = presetState(p, kinds);
    visible = { ...visible, ...s.visible };
    halfCut = s.halfCut;
    deckCut = s.deckCut;
    hullTranslucent = s.hullTranslucent;
    waterline = s.sea ? (waterlines[0]?.name ?? "") : "";
  }
  const hasWheelhouse = $derived(deck.items.some((d) => d.kind === "wheelhouse_raised"));
  const itemName = (d: DeckItem) => t(`app-3d-item-${d.kind.replaceAll("_", "-")}`);
  /** A gas Design's hold is the space around its Type C tanks (IGC term; spec 0009). */
  let hasTanks = $derived(spaces.some((s) => s.tank));
  const kindName = (kind: string) => {
    if (kind === "cargo_hold" && hasTanks) return t("app-3d-kind-hold-around-tanks");
    const id = kindLabelId(kind);
    return id ? t(id) : kind;
  };
  const sideName = (side: string) => t(`app-3d-side-${side}`);
  /** A Space layer is on, so the hull and the decks are drawn translucent to show it, unless the hull is
   * cut in half (it shows the inside opaque); the Arrangement preset makes it translucent too. */
  let innerOn = $derived(hullTranslucent || (!halfCut && kinds.some((k) => visible[`kind:${k}`])));

  // Set by onMount, called by the template.
  let apply: () => void = () => {};
  let selectByName: (name: string | null) => void = () => {};
  let drawSection: () => void = () => {};
  let selectItem: (id: string | null) => void = () => {};
  let selectProp: (id: string | null) => void = () => {};
  let goTo: (v: FixedView) => void = () => {};
  let reset: () => void = () => {};
  let onKey: (e: KeyboardEvent) => void = () => {};

  $effect(() => {
    // Any change of a layer, or of the translucency, is drawn.
    void visible;
    void innerOn;
    void waterline;
    void halfCut;
    void deckCut;
    void wheelhouseShown;
    apply();
  });
  $effect(() => {
    // The plane and its outline follow the switch and the slider.
    void sectionOn;
    void sectionX;
    drawSection();
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
        deck = await p.deckItems(option);
        propParts = await p.propulsors(option);
        groupLayers = [...new Set(data.groups.map((g) => g.layer))];
        waterlines = await p.waterlines(option);
        // The view opens on the Exterior preset (spec 0009, D4): the outside of the vessel, on the sea.
        choose("exterior");
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
      renderer.localClippingEnabled = true;
      // Keeps everything at or behind x = constant (three.js clips where n.p + c < 0).
      const plane = new THREE.Plane(new THREE.Vector3(-1, 0, 0), 1e9);
      // The half-model cut keeps model y >= 0 (the port half goes): under the mirrored root that is
      // view y <= 0. The deck cut keeps the hull below its deck edge (spec 0009, "Cuts").
      const half = new THREE.Plane(new THREE.Vector3(0, -1, 0), 1e9);
      const deckPlane = new THREE.Plane(new THREE.Vector3(0, 0, -1), 1e9);
      const canvas = renderer.domElement;
      canvas.tabIndex = 0;
      canvas.dataset.testid = "model3d-canvas";
      canvas.setAttribute("aria-label", t("app-3d-canvas-label"));
      host.appendChild(canvas);

      const token = (name: string) => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
      const scene = new THREE.Scene();
      const camera = new THREE.PerspectiveCamera(35, 1, 0.1, 5000);
      camera.up.set(0, 0, 1);
      // Lights (spec 0009): sky and ground from the theme, a key light from the aft-starboard quarter and
      // a weaker fill from port. No ambient flood, so the faces of the hull read apart.
      const sky = new THREE.HemisphereLight(0xffffff, 0xffffff, 1.6);
      sky.position.set(0, 0, 1);
      scene.add(sky);
      const key = new THREE.DirectionalLight(0xffffff, 2.0);
      const fill = new THREE.DirectionalLight(0xffffff, 0.6);
      scene.add(key, fill, key.target, fill.target);

      // One mesh per group, each on its own slice of the core's buffers (issue #683). The root holds
      // them in model axes and mirrors them into the view's right-handed axes (spec 0009, issue #682).
      const meshes = new Map<string, InstanceType<typeof THREE.Mesh>>();
      const edges: InstanceType<typeof THREE.LineSegments>[] = [];
      const space = new Map<InstanceType<typeof THREE.Object3D>, string>();
      // What a click picks: a Space (a tank's dome picks its tank), a Deck Item or a propulsor part.
      const pickable = new Map<InstanceType<typeof THREE.Object3D>, { item: boolean; prop?: boolean; id: string }>();
      // The inside faces of each closed solid, drawn in `--model-cut` where a cut opens it.
      const backs = new Map<string, InstanceType<typeof THREE.Mesh>>();
      const root = new THREE.Group();
      root.scale.set(...VIEW_MIRROR);
      scene.add(root);
      for (const g of data.groups) {
        const part = groupSlice(data.positions, data.indices, g.start, g.count);
        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute("position", new THREE.BufferAttribute(part.positions, 3));
        geometry.setIndex(new THREE.BufferAttribute(part.indices, 1));
        // A tank is a Space but curved: smooth, like the domes (spec 0009).
        const flat = flatShaded(g.layer) && !(g.layer === "space" && spaces.find((s) => s.name === g.id)?.tank);
        if (!flat) geometry.computeVertexNormals();
        const clips = g.layer === "hull" ? [plane, half, deckPlane] : [plane, half];
        const material = new THREE.MeshStandardMaterial({ side: THREE.DoubleSide, flatShading: flat, roughness: 0.85, metalness: 0, clippingPlanes: clips });
        const mesh = new THREE.Mesh(geometry, material);
        mesh.userData = { layer: g.layer, id: g.id };
        root.add(mesh);
        meshes.set(`${g.layer}:${g.id}`, mesh);
        if (g.layer === "space") space.set(mesh, g.id);
        if (g.layer === "space") pickable.set(mesh, { item: false, id: g.id });
        if (g.layer === "dome") pickable.set(mesh, { item: false, id: g.id.replace(/_dome$/, "") });
        if (g.layer === "deck_item") pickable.set(mesh, { item: true, id: g.id });
        if (g.layer === "propulsor") pickable.set(mesh, { item: false, prop: true, id: g.id });
        if (closedSolid(g.layer)) {
          const back = new THREE.Mesh(geometry, new THREE.MeshBasicMaterial({ side: THREE.BackSide, clippingPlanes: clips }));
          back.userData = { layer: g.layer, id: g.id, cutFace: true };
          back.visible = false;
          root.add(back);
          backs.set(`${g.layer}:${g.id}`, back);
        }
        // The hull's edges: where its faces meet at 30 degrees or more, clipped like the faces.
        if (g.layer === "hull") {
          const line = new THREE.LineSegments(new THREE.EdgesGeometry(geometry, 30), new THREE.LineBasicMaterial({ clippingPlanes: [plane, half, deckPlane] }));
          line.userData = { layer: g.layer, id: g.id, edges: true };
          root.add(line);
          edges.push(line);
        }
      }
      // The hull below the shown waterplane, in the wet colour: the hull's own geometry again, each copy
      // clipped to its side of the plane (issue #684). No waterline shown: the dry copy keeps it all.
      const above = new THREE.Plane(new THREE.Vector3(0, 0, 1), 1e9);
      const below = new THREE.Plane(new THREE.Vector3(0, 0, -1), -1e9);
      const wet: InstanceType<typeof THREE.Mesh>[] = [];
      for (const mesh of meshes.values()) {
        if (mesh.userData.layer !== "hull") continue;
        (mesh.material as InstanceType<typeof THREE.MeshStandardMaterial>).clippingPlanes = [plane, half, deckPlane, above];
        const copy = new THREE.Mesh(mesh.geometry, (mesh.material as InstanceType<typeof THREE.MeshStandardMaterial>).clone());
        (copy.material as InstanceType<typeof THREE.MeshStandardMaterial>).clippingPlanes = [plane, half, deckPlane, below];
        copy.userData = { ...mesh.userData, wet: true };
        root.add(copy);
        wet.push(copy);
      }
      const hullBox = new THREE.Box3();
      for (const mesh of meshes.values()) if (mesh.userData.layer === "hull") hullBox.union(mesh.geometry.boundingBox ?? (mesh.geometry.computeBoundingBox(), mesh.geometry.boundingBox!));
      // The deck edge height amidships, from the core, for the deck cut (the hull's top until it comes).
      let deckZ = hullBox.max.z;
      platform()
        .sectionAt(option, (hullBox.min.x + hullBox.max.x) / 2)
        .then((s: SectionOutline) => {
          deckZ = s.depth_at_side_m;
          paint();
        })
        .catch(() => {});
      // The sea plane, set to the shown waterline in paint().
      const seaGeometry = new THREE.BufferGeometry();
      const sea = new THREE.Mesh(seaGeometry, new THREE.MeshStandardMaterial({ side: THREE.DoubleSide, flatShading: true, roughness: 0.6, metalness: 0, transparent: true, opacity: 0.45, depthWrite: false }));
      sea.userData = { layer: "sea", id: "sea" };
      sea.renderOrder = 1;
      root.add(sea);
      let seaFor = "";
      // The box of the model, in model axes.
      const modelBox = new THREE.Box3().setFromArray(data.positions);
      const centre = modelBox.getCenter(new THREE.Vector3());
      const size = modelBox.getSize(new THREE.Vector3());
      const reach = Math.max(size.x, size.y, size.z) * 1.1;
      // The key light from the aft-starboard quarter and above, the fill from port, both in model axes.
      const at = (dx: number, dy: number, dz: number): Vec3 => toView([centre.x + dx * reach, centre.y + dy * reach, centre.z + dz * reach]);
      key.position.set(...at(-0.6, 0.8, 1));
      fill.position.set(...at(0.3, -1, 0.4));
      key.target.position.set(...at(0, 0, 0));
      fill.target.position.set(...at(0, 0, 0));
      length = modelBox.max.x;
      sectionX = Math.round(length / 2);
      const controls = new OrbitControls(camera, canvas);
      controls.enableDamping = false;

      const kindOf = (name: string) => spaces.find((s) => s.name === name)?.kind ?? "void";
      // The sea plane and the split of the hull at the shown waterline.
      const setSea = () => {
        const w = shownWaterline;
        sea.visible = w !== null;
        for (const m of wet) m.visible = w !== null && visible.hull !== false;
        if (!w) {
          above.set(new THREE.Vector3(0, 0, 1), 1e9);
          seaFor = "";
          delete canvas.dataset.sea;
          return;
        }
        const [x0, x1] = [hullBox.min.x, hullBox.max.x];
        if (seaFor !== w.name) {
          const s = seaPlane(x0, x1, w.draught_aft_m, w.draught_fwd_m, reach);
          seaGeometry.setAttribute("position", new THREE.BufferAttribute(s.positions, 3));
          seaGeometry.setIndex(new THREE.BufferAttribute(s.indices, 1));
          seaGeometry.computeBoundingSphere();
          seaFor = w.name;
        }
        // The plane through the waterline (model and view share x and z): keep z above it, or below.
        const start = new THREE.Vector3(x0, 0, w.draught_aft_m);
        const normal = new THREE.Vector3(-(w.draught_fwd_m - w.draught_aft_m) / (x1 - x0), 0, 1).normalize();
        above.setFromNormalAndCoplanarPoint(normal, start);
        below.setFromNormalAndCoplanarPoint(normal.clone().negate(), start);
        // For the end-to-end tests: the drawn plane's height at the hull's ends, read from its corners.
        const p = seaGeometry.getAttribute("position");
        const [a, f] = [0, 1].map((i) => root.localToWorld(new THREE.Vector3(p.getX(i), p.getY(i), p.getZ(i))));
        const zAt = (x: number) => a.z + ((f.z - a.z) * (x - a.x)) / (f.x - a.x);
        canvas.dataset.sea = JSON.stringify({ name: w.name, aft: zAt(x0), fwd: zAt(x1) });
      };
      const colour = (mesh: InstanceType<typeof THREE.Mesh>) => {
        const { layer, id } = mesh.userData as { layer: string; id: string };
        if (layer === "space") return token(FAMILY_TOKEN[familyOf(kindOf(id))]);
        if (layer === "hull") return token("--model-hull");
        if (layer === "waterline") return token("--accent");
        if (layer === "deck") return token("--model-deck");
        // A dome is part of its tank: the cargo colour (spec 0009).
        if (layer === "dome") return token(FAMILY_TOKEN.cargo);
        if (layer === "deck_item") return token(id.startsWith("wheelhouse") ? "--model-wheelhouse" : "--model-deck-item");
        if (layer === "weather_shield") return token("--model-deck");
        if (layer === "propulsor") return token("--model-propulsor");
        return token("--ink-2");
      };
      const picked = (layer: string, id: string) =>
        (layer === "space" && selected?.name === id) || (layer === "dome" && selected?.name === id.replace(/_dome$/, "")) || (layer === "deck_item" && selectedItem?.id === id) || (layer === "propulsor" && selectedProp?.id === id);
      function paint() {
        half.constant = halfCut ? 0 : 1e9;
        deckPlane.constant = deckCut ? deckZ - 1e-3 : 1e9;
        const cutOn = halfCut || deckCut || sectionOn;
        renderer.setClearColor(new THREE.Color(token("--paper") || "#f6edd8"));
        sky.color.set(token("--surface") || "#fbf6ea");
        sky.groundColor.set(token("--surface-2") || "#efe4c9");
        for (const mesh of meshes.values()) {
          const { layer, id } = mesh.userData as { layer: string; id: string };
          const m = mesh.material as InstanceType<typeof THREE.MeshStandardMaterial>;
          m.color.set(colour(mesh));
          const wh = layer === "deck_item" ? wheelhouseDraw(id, wheelhouseShown) : { on: true, opacity: 1 };
          const on =
            layer === "space"
              ? visible[`kind:${kindOf(id)}`] !== false
              : layer === "waterline"
                ? id === waterline
                : visible[layer] !== false && wh.on;
          // The deck cut lifts the deck and the weather shield off (spec 0009).
          mesh.visible = on && !(deckCut && (layer === "deck" || layer === "weather_shield"));
          // Opaque, unless a Space layer is on: then the hull and the decks let it show through.
          const translucent = innerOn && (layer === "hull" || layer === "deck");
          m.transparent = translucent || layer === "space" || layer === "waterline" || wh.opacity < 1;
          m.opacity = translucent ? (layer === "hull" ? 0.2 : 0.6) : layer === "space" ? 0.85 : layer === "waterline" ? 0.55 : wh.opacity;
          m.depthWrite = !translucent && wh.opacity === 1;
          m.emissive.set(picked(layer, id) ? token("--gilt-mark") : "#000000");
          m.emissiveIntensity = 0.55;
          // Where a cut opens a closed solid, its inside faces show in the cut colour.
          const back = backs.get(`${layer}:${id}`);
          const side = cutOn && back ? THREE.FrontSide : THREE.DoubleSide;
          if (m.side !== side) {
            m.side = side;
            m.needsUpdate = true;
          }
          if (back) {
            back.visible = mesh.visible && cutOn && !translucent && wh.opacity === 1;
            (back.material as InstanceType<typeof THREE.MeshBasicMaterial>).color.set(token("--model-cut"));
          }
        }
        for (const line of edges) {
          line.visible = visible.hull !== false;
          (line.material as InstanceType<typeof THREE.LineBasicMaterial>).color.set(token("--model-edge"));
        }
        setSea();
        (sea.material as InstanceType<typeof THREE.MeshStandardMaterial>).color.set(token("--model-sea"));
        for (const m of wet) {
          const dry = meshes.get(`${m.userData.layer}:${m.userData.id}`)!.material as InstanceType<typeof THREE.MeshStandardMaterial>;
          const mat = m.material as InstanceType<typeof THREE.MeshStandardMaterial>;
          mat.color.set(token("--model-hull-wet"));
          mat.transparent = dry.transparent;
          mat.opacity = dry.opacity;
          mat.depthWrite = dry.depthWrite;
        }
        for (const c of outline.children) ((c as InstanceType<typeof THREE.Line>).material as InstanceType<typeof THREE.LineBasicMaterial>).color.set(token("--accent"));
        controls.update();
        renderer.render(scene, camera);
        // What is drawn, for the end-to-end tests: the groups on, by layer.
        const shown = [...meshes.values()].filter((m) => m.visible).map((m) => `${m.userData.layer}:${m.userData.id}`);
        if (sea.visible) shown.push("sea");
        canvas.dataset.shown = JSON.stringify(shown);
        // The cuts and the wheelhouse drawn, for the end-to-end tests (issue #689).
        canvas.dataset.cuts = JSON.stringify({ half: half.constant, deck: deckPlane.constant, cutFaces: [...backs.values()].filter((b) => b.visible).length });
        const whOpacity = (id: string) => {
          const m = meshes.get(`deck_item:${id}`);
          return m ? { visible: m.visible, opacity: (m.material as InstanceType<typeof THREE.MeshStandardMaterial>).opacity } : null;
        };
        canvas.dataset.wheelhouse = JSON.stringify({ lowered: whOpacity("wheelhouse_lowered"), raised: whOpacity("wheelhouse_raised") });
        // The colours drawn, for the end-to-end tests: the theme switch repaints them (issue #684).
        const hex = (m: InstanceType<typeof THREE.Material>) => `#${(m as InstanceType<typeof THREE.MeshStandardMaterial>).color.getHexString()}`;
        const first = (layer: string) => [...meshes.values()].find((m) => m.userData.layer === layer);
        canvas.dataset.colours = JSON.stringify({
          hull: first("hull") ? hex(first("hull")!.material as InstanceType<typeof THREE.Material>) : null,
          wet: wet[0] ? hex(wet[0].material as InstanceType<typeof THREE.Material>) : null,
          sea: hex(sea.material as InstanceType<typeof THREE.Material>),
          deck: first("deck") ? hex(first("deck")!.material as InstanceType<typeof THREE.Material>) : null,
          edge: edges[0] ? hex(edges[0].material as InstanceType<typeof THREE.Material>) : null,
        });
        // Where four points of the model's box land on the screen (x and y from -1 to 1, y up) and how
        // far each is from the camera, for the handedness check of the end-to-end tests (spec 0009).
        const probe = (p: Vec3) => {
          const w = root.localToWorld(new THREE.Vector3(...p));
          const s = w.clone().project(camera);
          return [s.x, s.y, w.distanceTo(camera.position)];
        };
        const [mc, lo, hi] = [modelBox.getCenter(new THREE.Vector3()), modelBox.min, modelBox.max];
        canvas.dataset.probe = JSON.stringify({
          bow: probe([hi.x, mc.y, mc.z]),
          stern: probe([lo.x, mc.y, mc.z]),
          port: probe([mc.x, lo.y, mc.z]),
          starboard: probe([mc.x, hi.y, mc.z]),
        });
        // The material of each layer's first group, and the draw calls of the last frame, for the
        // end-to-end tests (issue #683).
        const materials: Record<string, { opacity: number; transparent: boolean }> = {};
        for (const mesh of meshes.values()) {
          const m = mesh.material as InstanceType<typeof THREE.MeshStandardMaterial>;
          materials[mesh.userData.layer] ??= { opacity: m.opacity, transparent: m.transparent };
        }
        canvas.dataset.materials = JSON.stringify(materials);
        canvas.dataset.calls = String(renderer.info.render.calls);
      }
      apply = paint;

      // For the end-to-end tests (issue #683): draw one part alone into an off-screen target and count
      // its pixels on each side of the section plane's trace, the bow side being the screen's right in
      // the profile view. A part the plane clips has none on the bow side.
      const coverage = (part: "hull" | "edges" | "spaces") => {
        const chosen = (o: InstanceType<typeof THREE.Object3D>) =>
          part === "hull" ? o.userData.layer === "hull" && !o.userData.edges : part === "edges" ? o.userData.edges === true : o.userData.layer === "space";
        const parts = root.children.filter(chosen);
        if (parts.length === 0) return null;
        const was = new Map(root.children.map((o) => [o, o.visible]));
        for (const o of root.children) o.visible = chosen(o) && (was.get(o) ?? false);
        const [w, h] = [canvas.width, canvas.height];
        const target = new THREE.WebGLRenderTarget(w, h);
        const clear = renderer.getClearColor(new THREE.Color());
        renderer.setClearColor(clear, 0);
        renderer.setRenderTarget(target);
        renderer.render(scene, camera);
        const pixels = new Uint8Array(4 * w * h);
        renderer.readRenderTargetPixels(target, 0, 0, w, h, pixels);
        renderer.setRenderTarget(null);
        renderer.setClearColor(clear, 1);
        target.dispose();
        for (const [o, v] of was) o.visible = v;
        const cut = root.localToWorld(new THREE.Vector3(plane.constant < 1e8 ? plane.constant : modelBox.max.x, centre.y, centre.z)).project(camera);
        const cutPx = ((cut.x + 1) / 2) * w;
        let kept = 0;
        let beyond = 0;
        for (let i = 0; i < w * h; i++) {
          if (pixels[4 * i + 3] === 0) continue;
          const x = i % w;
          if (x < cutPx - 2) kept++;
          else if (x > cutPx + 2) beyond++;
        }
        paint();
        return { kept, beyond };
      };
      (canvas as unknown as { ovcdCoverage: typeof coverage }).ovcdCoverage = coverage;

      // The section plane: the clip, and the outline the core gives at that x, drawn in the plane.
      const outline = new THREE.Group();
      root.add(outline);
      let asked = 0;
      const clear = () => {
        for (const c of [...outline.children]) {
          (c as InstanceType<typeof THREE.Line>).geometry.dispose();
          outline.remove(c);
        }
        delete canvas.dataset.section;
      };
      drawSection = () => {
        plane.constant = sectionOn ? sectionX : 1e9;
        if (!sectionOn) {
          asked++;
          clear();
          paint();
          return;
        }
        const mine = ++asked;
        platform()
          .sectionAt(option, sectionX)
          .then((s: SectionOutline) => {
            if (mine !== asked) return;
            clear();
            const drawn: [number, number][][] = [];
            for (const ring of s.rings) {
              const xyz = new Float32Array(ring.length * 3);
              ring.forEach(([y, z], i) => xyz.set([s.x_m, y, z], i * 3));
              const g = new THREE.BufferGeometry();
              g.setAttribute("position", new THREE.BufferAttribute(xyz, 3));
              const line = new THREE.LineLoop(g, new THREE.LineBasicMaterial({ color: token("--accent") }));
              line.userData = { outline: true };
              outline.add(line);
              drawn.push(ring.map((_, i) => [xyz[i * 3 + 1], xyz[i * 3 + 2]]));
            }
            // What is drawn, as the f32 the screen holds, for the end-to-end tests.
            canvas.dataset.section = JSON.stringify({ x: s.x_m, rings: drawn });
            paint();
          })
          .catch(() => {});
        paint();
      };

      // The camera stands at `position` (model axes) and looks at the centre of the model.
      function look(position: Vec3, up: Vec3) {
        camera.up.set(...toView(up));
        camera.position.set(...toView(position));
        controls.target.set(...toView([centre.x, centre.y, centre.z]));
        controls.update();
        paint();
      }
      reset = () => {
        const h = homeView(modelBox.min.toArray() as Vec3, modelBox.max.toArray() as Vec3, camera.fov, camera.aspect);
        look(h.position, h.up);
      };
      goTo = (v) => {
        const f = fixedView(v, [centre.x, centre.y, centre.z], reach);
        look(f.position, f.up);
      };
      selectItem = (id) => {
        selectedItem = id ? (deck.items.find((d) => d.id === id) ?? null) : null;
        selected = null;
        selectedProp = null;
        paint();
      };
      selectProp = (id) => {
        selectedProp = id ? (propParts.find((d) => d.id === id) ?? null) : null;
        selected = null;
        selectedItem = null;
        paint();
      };
      selectByName = (name) => {
        selected = name ? (spaces.find((s) => s.name === name) ?? null) : null;
        selectedItem = null;
        selectedProp = null;
        // A Space picked in the list is shown: its kind's layer goes on.
        if (selected) visible[`kind:${selected.kind}`] = true;
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
        const hit = ray.intersectObjects([...pickable.keys()].filter((o) => o.visible), false)[0];
        const what = hit ? pickable.get(hit.object) : undefined;
        if (what?.item) selectItem(what.id);
        else if (what?.prop) selectProp(what.id);
        else selectByName(what ? what.id : null);
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
      resize();
      reset();

      cleanup = () => {
        stopTheme();
        ro.disconnect();
        canvas.removeEventListener("pointerdown", onDown);
        canvas.removeEventListener("pointerup", onUp);
        controls.dispose();
        clear();
        seaGeometry.dispose();
        (sea.material as InstanceType<typeof THREE.Material>).dispose();
        for (const m of wet) (m.material as InstanceType<typeof THREE.Material>).dispose();
        for (const b of backs.values()) (b.material as InstanceType<typeof THREE.Material>).dispose();
        for (const m of [...meshes.values(), ...edges]) {
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

<div class="model" data-testid="model3d" data-status={status} data-problem={problem || undefined} data-preset={preset ?? "custom"}>
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
        <div class="bar presets" role="group" aria-label={t("app-3d-presets")}>
          <span class="dim">{t("app-3d-presets")}</span>
          {#each PRESETS as p (p)}
            <button type="button" class:on={preset === p} aria-pressed={preset === p} data-testid="preset-{p}" onclick={() => choose(p)}>{t(`app-3d-preset-${p}`)}</button>
          {/each}
          {#if preset === null}<span class="custom" data-testid="preset-custom">{t("app-3d-preset-custom")}</span>{/if}
          <label><input type="checkbox" data-testid="cut-half" bind:checked={halfCut} /> {t("app-3d-cut-half")}</label>
          <label><input type="checkbox" data-testid="cut-deck" bind:checked={deckCut} /> {t("app-3d-cut-deck")}</label>
        </div>
        {#if hasWheelhouse}
          <div class="bar presets" role="group" aria-label={t("app-3d-wheelhouse")}>
            <span class="dim">{t("app-3d-wheelhouse")}</span>
            {#each ["lowered", "raised", "both"] as const as w (w)}
              <button type="button" class:on={wheelhouseShown === w} aria-pressed={wheelhouseShown === w} data-testid="wheelhouse-{w}" onclick={() => (wheelhouseShown = w)}>{t(`app-3d-wheelhouse-${w}`)}</button>
            {/each}
            {#if deck.wheelhouse}
              <span class="dim" data-testid="wheelhouse-lift" data-needed={deck.wheelhouse.needed_m} data-allowed={deck.wheelhouse.allowed_m} data-air={deck.wheelhouse.air_draft_light_m}>
                {deck.wheelhouse.needed_m === null
                  ? t("app-3d-lift-used", { used: num(deck.wheelhouse.used_m, 2), allowed: num(deck.wheelhouse.allowed_m, 2) })
                  : t("app-3d-lift-needed", { needed: num(deck.wheelhouse.needed_m, 2), allowed: num(deck.wheelhouse.allowed_m, 2) })}{#if deck.wheelhouse.air_draft_light_m !== null}; {t("app-3d-air-draft-light", { lowered: num(deck.wheelhouse.air_draft_light_m, 2), raised: num(deck.wheelhouse.air_draft_navigating_light_m ?? deck.wheelhouse.air_draft_light_m, 2) })}{/if}
              </span>
            {/if}
          </div>
        {/if}
        <div class="bar tools">
          <label><input type="checkbox" data-testid="section-on" bind:checked={sectionOn} /> {t("app-3d-section")}</label>
          {#if sectionOn}
            <input type="range" data-testid="section-x" min="0" max={length} step="0.1" bind:value={sectionX} aria-label={t("app-3d-section-x")} />
            <span class="dim" data-testid="section-x-value">{t("app-3d-section-at", { x: num(sectionX, 1) })}</span>
          {/if}
          {#if waterlines.length > 0}
            <label class="wl">{t("app-3d-waterline")}
              <select data-testid="waterline-select" bind:value={waterline}>
                <option value="">{t("app-3d-waterline-none")}</option>
                {#each waterlines as w (w.name)}<option value={w.name}>{w.name}</option>{/each}
              </select>
            </label>
            {#if shownWaterline}
              <span class="dim" data-testid="waterline-info" data-draught={shownWaterline.draught_m} data-aft={shownWaterline.draught_aft_m} data-fwd={shownWaterline.draught_fwd_m} data-trim={shownWaterline.trim_m}>
                {t("app-3d-waterline-draughts", { mean: num(shownWaterline.draught_m, 2), aft: num(shownWaterline.draught_aft_m, 2), fwd: num(shownWaterline.draught_fwd_m, 2), trim: num(shownWaterline.trim_m, 2) })}
              </span>
            {/if}
          {/if}
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
      {#if spaces.length === 0 && deck.items.length === 0 && propParts.length === 0}
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
            {#each deck.items as d (d.id)}
              <tr class:on={selectedItem?.id === d.id}>
                <th><button type="button" class="link" data-testid="deck-item-{d.id}" onclick={() => selectItem(d.id)}>{d.id}</button></th>
                <td>{itemName(d)}</td>
                <td class="num dim">–</td>
              </tr>
            {/each}
            {#each propParts as q (q.id)}
              <tr class:on={selectedProp?.id === q.id}>
                <th><button type="button" class="link" data-testid="propulsor-{q.id}" onclick={() => selectProp(q.id)}>{q.id}</button></th>
                <td>{partName(q)}</td>
                <td class="num dim">–</td>
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
          {#if selected.tank}
            <dt>{t("app-3d-tank-od")}</dt><dd data-testid="space-info-tank-od" data-value={selected.tank.outer_diameter_m}>{num(selected.tank.outer_diameter_m, 2)} m</dd>
            <dt>{t("app-3d-tank-length")}</dt><dd data-testid="space-info-tank-length" data-value={selected.tank.length_m}>{num(selected.tank.length_m, 2)} m</dd>
            <dt>{t("app-3d-tank-head")}</dt><dd data-testid="space-info-tank-head">{t(`app-3d-head-${selected.tank.head_type.replaceAll("_", "-")}`)}, {num(selected.tank.head_length_m, 2)} m</dd>
          {/if}
        </dl>
      {:else if selectedItem}
        <dl class="info" data-testid="deck-item-info">
          <dt>{t("app-3d-col-name")}</dt><dd data-testid="deck-item-info-id">{selectedItem.id}</dd>
          <dt>{t("app-3d-col-kind")}</dt><dd>{itemName(selectedItem)}{selectedItem.kind.endsWith("_unit") ? ` (${selectedItem.label})` : ""}</dd>
          <dt>{t("app-3d-item-size")}</dt><dd data-testid="deck-item-info-size">{num(selectedItem.x_fwd_m - selectedItem.x_aft_m, 2)} × {num(selectedItem.y_stbd_m - selectedItem.y_port_m, 2)} × {num(selectedItem.z_top_m - selectedItem.z_base_m, 2)} m</dd>
          <dt>{t("app-3d-item-base")}</dt><dd data-testid="deck-item-info-base" data-value={selectedItem.z_base_m}>{num(selectedItem.z_base_m, 2)} m</dd>
          <dt>{t("app-3d-item-top")}</dt><dd data-testid="deck-item-info-top" data-value={selectedItem.z_top_m}>{num(selectedItem.z_top_m, 2)} m</dd>
          {#if selectedItem.kind.startsWith("wheelhouse") && deck.wheelhouse}
            <dt>{t("app-3d-item-lift")}</dt><dd data-testid="deck-item-info-lift">{deck.wheelhouse.needed_m === null ? "–" : num(deck.wheelhouse.needed_m, 2)} / {num(deck.wheelhouse.used_m, 2)} / {num(deck.wheelhouse.allowed_m, 2)} m</dd>
          {/if}
        </dl>
      {:else if selectedProp}
        <dl class="info" data-testid="propulsor-info">
          <dt>{t("app-3d-col-name")}</dt><dd data-testid="propulsor-info-id">{selectedProp.id}</dd>
          <dt>{t("app-3d-col-kind")}</dt><dd>{partName(selectedProp)}</dd>
          {#if selectedProp.part === "bow_thruster"}
            <dt>{t("app-3d-propulsor-tunnel-diameter")}</dt><dd data-testid="propulsor-info-diameter" data-value={selectedProp.tunnel_diameter_m}>{num(selectedProp.tunnel_diameter_m ?? 0, 2)} m</dd>
            <dt>{t("app-3d-propulsor-axis")}</dt><dd data-testid="propulsor-info-axis" data-value={selectedProp.z_m}>{num(selectedProp.z_m, 2)} m</dd>
          {:else}
            <dt>{t("app-3d-propulsor-type")}</dt><dd data-testid="propulsor-info-type">{typeName(selectedProp.propulsor_type)}</dd>
            <dt>{t("app-3d-propulsor-diameter")}</dt><dd data-testid="propulsor-info-diameter" data-value={selectedProp.diameter_m}>{num(selectedProp.diameter_m, 2)} m</dd>
            <dt>{t("app-3d-propulsor-axis")}</dt><dd data-testid="propulsor-info-axis" data-value={selectedProp.axis_z_m}>{num(selectedProp.axis_z_m, 2)} m</dd>
          {/if}
          {#if selectedProp.part === "rudder"}
            <dt>{t("app-3d-propulsor-rudder-size")}</dt><dd data-testid="propulsor-info-rudder">{num(selectedProp.span_m ?? 0, 2)} × {num(selectedProp.chord_m ?? 0, 2)} m</dd>
          {/if}
          <dt>{selectedProp.part === "bow_thruster" ? t("app-3d-propulsor-bow-power") : t("app-3d-propulsor-power")}</dt><dd data-testid="propulsor-info-power" data-value={selectedProp.unit_power_kw}>{selectedProp.unit_power_kw === null ? "–" : `${num(selectedProp.unit_power_kw, 0)} kW`}</dd>
          <dt>{t("app-3d-propulsor-mass-centre")}</dt><dd data-testid="propulsor-info-mass">{selectedProp.mass_item}: {num(selectedProp.lcg_m, 2)}, {num(selectedProp.kcg_m, 2)} m</dd>
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
  .presets button.on { background: var(--accent-soft); border-color: var(--accent); }
  .presets label { display: flex; align-items: center; gap: 6px; }
  .custom { color: var(--ink-2); font-style: italic; }
  .tools label { display: flex; align-items: center; gap: 6px; }
  .tools select { background: var(--surface); color: var(--ink); border: 1px solid var(--rule-strong); border-radius: var(--radius); }
  .tools input[type="range"] { width: 200px; }
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
