// The platform interface: every call the front end makes to the Rust side, and the file actions.
// Two implementations: Tauri (desktop) and Web (the ovcd-web WebAssembly session). Neither holds
// any calculation, validation or default logic.
import type { Evaluation, Schema, Snapshot } from "../api";

export type DrawingView = "side" | "plan" | "midship";
export type DrawingFormat = "svg" | "pdf" | "dxf";
export type ModelFormat = "gltf";

export interface Platform {
  /** "tauri" for the desktop app, "web" for the browser build. */
  readonly kind: "tauri" | "web";
  snapshot(): Promise<Snapshot>;
  inputSchema(): Promise<Schema>;
  evaluate(): Promise<Evaluation>;
  /** Each of these returns null when the user cancels a dialog. */
  newFile(): Promise<Snapshot | null>;
  openFile(): Promise<Snapshot | null>;
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
  exportModel(option: string, format: ModelFormat): Promise<boolean>;
}
