// The desktop implementation: today's Tauri commands, unchanged.
import { invoke } from "@tauri-apps/api/core";
import type { Evaluation, Schema, Snapshot } from "../api";
import type { Platform } from "./types";

export const tauriPlatform: Platform = {
  kind: "tauri",
  snapshot: () => invoke<Snapshot>("snapshot"),
  inputSchema: () => invoke<Schema>("input_schema"),
  evaluate: () => invoke<Evaluation>("evaluate"),
  newFile: () => invoke<Snapshot | null>("new_file"),
  openFile: () => invoke<Snapshot | null>("open_file"),
  save: () => invoke<Snapshot | null>("save"),
  saveAs: () => invoke<Snapshot | null>("save_as"),
  setInput: (option, name, value) => invoke<Snapshot>("set_input", { option, name, value }),
  resetInput: (option, name) => invoke<Snapshot>("reset_input", { option, name }),
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
};
