// The only door to the Rust side. Every function forwards to a Tauri command; none computes.
import { invoke } from "@tauri-apps/api/core";

export type Source = "typed" | "default";
export interface StoredInput {
  value: unknown;
  source: Source;
}
export interface DesignOption {
  name: string;
  inputs: Record<string, StoredInput>;
  results_hash?: string;
}
export interface Snapshot {
  engine_version: string;
  profile: string;
  design: { options: DesignOption[] };
  file_name: string;
  path: string | null;
  dirty: boolean;
  title: string;
  active_option: string;
  /** Options whose stored results hash differs from this engine's, at open. */
  hash_changed: string[];
}
export interface Choice {
  value: string;
  label: string;
}
export interface HardRange {
  gt?: number;
  ge?: number;
  lt?: number;
  le?: number;
}
export interface Ranges {
  unit?: string;
  hard?: HardRange;
  soft?: { min: number; max: number };
  basis?: string;
}
export interface InputSpec extends Ranges {
  name: string;
  label: string;
  group: string;
  required: boolean;
  kind: "number" | "bool" | "choice" | "cargo" | "formation";
  nullable?: boolean;
  choices?: Choice[];
  default?: unknown;
  default_by?: { by: string; values: Record<string, unknown> };
  convoy?: { abreast: Ranges; in_line: Ranges };
}
export interface Schema {
  profile: string;
  engine_version: string;
  groups: string[];
  inputs: InputSpec[];
}
export interface Message {
  severity: "error" | "warning";
  scope: "input" | "cross_check" | "option";
  inputs: string[];
  text: string;
}
/** The hull views of a computed Option, drawn in Rust to the Drawing Format Standard, as SVG text. */
export type HullViews = { side: string; plan: string; midship: string } | { message: string };
export type Outcome =
  | { status: "results"; results: Record<string, unknown>; results_hash: string; views?: HullViews }
  | { status: "not_available" | "failed"; message: string }
  | { status: "not_run" };
export interface OptionEvaluation {
  name: string;
  messages: Message[];
  outcome: Outcome;
}
export interface Evaluation {
  options: OptionEvaluation[];
  /** Messages about the file as a whole, which no Option owns. */
  file_messages: Message[];
}

export const snapshot = () => invoke<Snapshot>("snapshot");
export const inputSchema = () => invoke<Schema>("input_schema");
export const evaluate = () => invoke<Evaluation>("evaluate");
/** Each of these returns null when the user cancels a dialog. */
export const newFile = () => invoke<Snapshot | null>("new_file");
export const openFile = () => invoke<Snapshot | null>("open_file");
export const save = () => invoke<Snapshot | null>("save");
export const saveAs = () => invoke<Snapshot | null>("save_as");
export const setInput = (option: string, name: string, value: unknown) =>
  invoke<Snapshot>("set_input", { option, name, value });
export const resetInput = (option: string, name: string) =>
  invoke<Snapshot>("reset_input", { option, name });
export const setActiveOption = (option: string) =>
  invoke<Snapshot>("set_active_option", { option });
export const setFormation = (
  option: string,
  kind: string,
  abreast: number | null,
  inLine: number | null,
) => invoke<Snapshot>("set_formation", { option, kind, abreast, inLine });
export const addOption = () => invoke<Snapshot>("add_option");
export const duplicateOption = (option: string) => invoke<Snapshot>("duplicate_option", { option });
export const renameOption = (option: string, newName: string) =>
  invoke<Snapshot>("rename_option", { option, newName });
export const deleteOption = (option: string) => invoke<Snapshot>("delete_option", { option });
/** null when the user cancels the file dialog; rejects with the reason when the profile is refused. */
export const loadProfile = () => invoke<Snapshot | null>("load_profile");
export const useBuiltInProfile = () => invoke<Snapshot>("use_built_in_profile");
