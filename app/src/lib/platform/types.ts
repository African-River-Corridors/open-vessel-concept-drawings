// The platform interface: every call the front end makes to the Rust side, and the file actions.
// Two implementations: Tauri (desktop) and Web (the ovcd-web WebAssembly session). Neither holds
// any calculation, validation or default logic.
import type { Evaluation, Schema, Snapshot } from "../api";

export type DrawingView = "side" | "plan" | "midship";
export type DrawingFormat = "svg" | "pdf" | "dxf";
export type ModelFormat = "gltf" | "stl" | "obj" | "offsets-csv";

/** Issue #457: one named part of the Vessel Model meshes, `count` indices from `start`. */
export interface MeshGroup {
  layer: "hull" | "deck" | "bulkhead" | "space" | "dome" | "weather_shield" | "deck_item" | "propulsor" | "waterline";
  id: string;
  start: number;
  count: number;
}

/** The meshes of an Option: x y z per vertex (m, hull axes), three indices per triangle. */
export interface Meshes {
  positions: Float32Array;
  indices: Uint32Array;
  groups: MeshGroup[];
  /** The mesh hash of the parity check (not the results hash). */
  hash: string;
}

/** The hull's section outline at an x: the polygons of the full section, [y, z] in metres. */
export interface SectionOutline {
  x_m: number;
  depth_at_side_m: number;
  rings: [number, number][][];
}

/** A Space as the core gives it. */
export interface SpaceInfo {
  name: string;
  kind: string;
  side: string;
  volume_m3: number;
  lcg_m: number;
  tcg_m: number;
  vcg_m: number;
  /** A Type C tank's Tank Layout values (spec 0009); absent for any other Space. */
  tank?: { outer_diameter_m: number; length_m: number; head_type: string; head_length_m: number };
}

/** A Deck Item as the core gives it (spec 0009): its mesh group id, kind and box (y to starboard). */
export interface DeckItem {
  id: string;
  kind: "accommodation_unit" | "battery_unit" | "wheelhouse_lowered" | "wheelhouse_raised" | "lift_column";
  label: string;
  x_aft_m: number;
  x_fwd_m: number;
  y_port_m: number;
  y_stbd_m: number;
  z_base_m: number;
  z_top_m: number;
}

/** A part of the Propulsor Layout (spec 0009), as the pick panel shows it: centre in mesh axes (y to
 * starboard), and its propulsor's type, diameter, axis height and power (a bow thruster's own power). */
export interface PropulsorPart {
  id: string;
  part: "propulsor" | "nozzle" | "leg" | "rudder" | "bow_thruster";
  index: number | null;
  propulsor_type: string;
  diameter_m: number;
  axis_z_m: number;
  unit_power_kw: number | null;
  x_m: number;
  y_m: number;
  z_m: number;
  mass_item: string;
  lcg_m: number;
  kcg_m: number;
  span_m?: number;
  chord_m?: number;
  tunnel_diameter_m?: number;
}

/** The Deck Items of an Option and its wheelhouse's lift (null without a stern block). */
export interface DeckItems {
  items: DeckItem[];
  wheelhouse: {
    needed_m: number | null;
    used_m: number;
    allowed_m: number;
    air_draft_light_m: number | null;
    air_draft_navigating_light_m: number | null;
  } | null;
}

/** The waterline of one floating position, and the index of its group in `Meshes.groups`. */
export interface Waterline {
  name: string;
  draught_m: number;
  draught_aft_m: number;
  draught_fwd_m: number;
  trim_m: number;
  group: number | null;
}

export interface Platform {
  /** "tauri" for the desktop app, "web" for the browser build. */
  readonly kind: "tauri" | "web";
  snapshot(): Promise<Snapshot>;
  inputSchema(): Promise<Schema>;
  evaluate(): Promise<Evaluation>;
  /** Each of these returns null when the user cancels a dialog. */
  newFile(): Promise<Snapshot | null>;
  openFile(): Promise<Snapshot | null>;
  /**
   * Issue #671: call `opened` when the app is given a Design from outside (Finder, "Open With",
   * the command line) and has loaded it, or `failed` with the reason when it could not. The web
   * build is never given a file this way.
   */
  onOpenedFromOutside(opened: () => void, failed: (reason: string) => void): Promise<void>;
  save(): Promise<Snapshot | null>;
  saveAs(): Promise<Snapshot | null>;
  setInput(option: string, name: string, value: unknown): Promise<Snapshot>;
  resetInput(option: string, name: string): Promise<Snapshot>;
  /** Issue #451: pick an offsets CSV and import it into the Option; null when the user cancels. */
  importOffsets(option: string): Promise<Snapshot | null>;
  setActiveOption(option: string): Promise<Snapshot>;
  setFormation(option: string, kind: string, abreast: number | null, inLine: number | null): Promise<Snapshot>;
  addOption(): Promise<Snapshot>;
  duplicateOption(option: string): Promise<Snapshot>;
  renameOption(option: string, newName: string): Promise<Snapshot>;
  deleteOption(option: string): Promise<Snapshot>;
  loadProfile(): Promise<Snapshot | null>;
  useBuiltInProfile(): Promise<Snapshot>;
  noticeSeen(): Promise<boolean>;
  acceptNotice(): Promise<void>;
  /** Save one drawing view as a file. Resolves false when this platform cannot (the desktop app, for now). */
  exportDrawing(option: string, view: DrawingView, format: DrawingFormat): Promise<boolean>;
  /** Issue #460: the Option's Vessel Model as a glTF binary, saved as a file. */
  exportModel(option: string, format: ModelFormat, below?: string): Promise<boolean>;
  /** Issue #457: the Vessel Model of an Option for the 3D view. Vertex data stays typed. */
  meshes(option: string): Promise<Meshes>;
  sectionAt(option: string, x: number): Promise<SectionOutline>;
  spaceInfo(option: string, name: string): Promise<SpaceInfo>;
  /** Spec 0009: the Deck Items and the wheelhouse's lift, from the core. */
  deckItems(option: string): Promise<DeckItems>;
  /** The propulsor parts of an Option (empty without a stern block). */
  propulsors(option: string): Promise<PropulsorPart[]>;
  waterlines(option: string): Promise<Waterline[]>;
}
