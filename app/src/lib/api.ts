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
export interface InputSpec {
  name: string;
  group: string;
  required: boolean;
  kind: string;
  unit?: string;
  basis?: string;
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
export type Outcome =
  | { status: "results"; results: Record<string, unknown>; results_hash: string }
  | { status: "not_available" | "failed"; message: string }
  | { status: "not_run" };
export interface OptionEvaluation {
  name: string;
  messages: Message[];
  outcome: Outcome;
}
export interface Evaluation {
  options: OptionEvaluation[];
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
