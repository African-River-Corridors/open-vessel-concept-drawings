// The desktop implementation: today's Tauri commands, unchanged.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Evaluation, Schema, Snapshot } from "../api";
import type { DeckItems, MeshGroup, Platform, PropulsorPart, SectionOutline, SpaceInfo, Waterline } from "./types";

/** What the webview gives for a raw response: an ArrayBuffer, or (when the IPC falls back to JSON) an array of bytes. */
function asBuffer(raw: ArrayBuffer | ArrayLike<number>): ArrayBuffer {
  if (raw instanceof ArrayBuffer) return raw;
  return Uint8Array.from(raw as ArrayLike<number>).buffer;
}

/** The raw bytes of `mesh_buffers`: two u32 counts, then the f32 positions and the u32 indices. */
function splitBuffers(raw: ArrayBuffer | ArrayLike<number>): { positions: Float32Array; indices: Uint32Array } {
  const buf = asBuffer(raw);
  const head = new DataView(buf, 0, 8);
  const np = head.getUint32(0, true);
  const ni = head.getUint32(4, true);
  return {
    positions: new Float32Array(buf.slice(8, 8 + 4 * np)),
    indices: new Uint32Array(buf.slice(8 + 4 * np, 8 + 4 * (np + ni))),
  };
}

export const tauriPlatform: Platform = {
  kind: "tauri",
  snapshot: () => invoke<Snapshot>("snapshot"),
  inputSchema: () => invoke<Schema>("input_schema"),
  evaluate: () => invoke<Evaluation>("evaluate"),
  newFile: () => invoke<Snapshot | null>("new_file"),
  openFile: () => invoke<Snapshot | null>("open_file"),
  onOpenedFromOutside: async (opened, failed) => {
    await listen("design-opened", () => opened());
    await listen<string>("design-open-failed", (e) => failed(e.payload));
  },
  save: () => invoke<Snapshot | null>("save"),
  saveAs: () => invoke<Snapshot | null>("save_as"),
  setInput: (option, name, value) => invoke<Snapshot>("set_input", { option, name, value }),
  resetInput: (option, name) => invoke<Snapshot>("reset_input", { option, name }),
  importOffsets: (option) => invoke<Snapshot | null>("import_offsets", { option }),
  setActiveOption: (option) => invoke<Snapshot>("set_active_option", { option }),
  setFormation: (option, kind, abreast, inLine) =>
    invoke<Snapshot>("set_formation", { option, kind, abreast, inLine }),
  addOption: () => invoke<Snapshot>("add_option"),
  duplicateOption: (option) => invoke<Snapshot>("duplicate_option", { option }),
  renameOption: (option, newName) => invoke<Snapshot>("rename_option", { option, newName }),
  deleteOption: (option) => invoke<Snapshot>("delete_option", { option }),
  loadProfile: () => invoke<Snapshot | null>("load_profile"),
  useBuiltInProfile: () => invoke<Snapshot>("use_built_in_profile"),
  noticeSeen: () => invoke<boolean>("notice_seen"),
  acceptNotice: () => invoke<void>("accept_notice"),
  exportDrawing: async () => false,
  exportModel: async () => false,
  meshes: async (option) => {
    const { positions, indices } = splitBuffers(await invoke<ArrayBuffer | number[]>("mesh_buffers", { option }));
    const g = await invoke<{ hash: string; groups: MeshGroup[] }>("mesh_groups", { option });
    return { positions, indices, groups: g.groups, hash: g.hash };
  },
  sectionAt: (option, x) => invoke<SectionOutline>("section_at", { option, x }),
  spaceInfo: (option, name) => invoke<SpaceInfo>("space_info", { option, name }),
  deckItems: (option) => invoke<DeckItems>("deck_items", { option }),
  propulsors: (option) => invoke<PropulsorPart[]>("propulsors", { option }),
  waterlines: (option) => invoke<Waterline[]>("waterlines", { option }),
};
