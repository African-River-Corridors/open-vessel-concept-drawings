// The only door to the Rust side. Every function forwards to the platform adapter (Tauri commands on the
// desktop, the WebAssembly session in the browser); none computes.
import { platform } from "./platform";
export type { DrawingFormat, DrawingView, ModelFormat } from "./platform";
import type { DrawingFormat, DrawingView, ModelFormat } from "./platform";
import type { EngineMsg } from "./i18n.svelte";

/** Where an input came from: the designer, the Defaults Profile, or the Vessel Class. */
export type Source = "typed" | "default" | "class";
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
  /** What changed when an old (version 1) file was read as the current inputs, at open. */
  migration_notes: string[];
  /** The Message Id of each migration note, where it has one (ADR 0015). */
  migration_note_msgs?: (EngineMsg | null)[];
}
export interface Choice {
  value: string;
  label: string;
  /** A Vessel Class choice: its scheme, the Formation kind it gives and what picking it sets. */
  scheme?: string;
  formation?: string | null;
  /** A Cargo choice: `development` while the Cargo is work in development (issue #334). */
  status?: string;
}
/** The Vessel Class an Option sits in, and its reference deadweight (shown beside the cargo, never an input). */
export interface VesselClassSummary {
  id: string;
  scheme: string;
  formation: string;
  length_max_m: number | null;
  beam_max_m: number | null;
  draught_m: [number, number] | null;
  air_draft_m: [number, number] | null;
  reference_dwt_t: [number, number] | null;
  confidence: string;
  source: string;
  notes: string;
  /** The vessel is longer or wider than the largest class of its scheme. */
  beyond_largest: boolean;
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
  /** The Cargo Containments the input is used for. */
  applies_to?: string[];
  /** The Formation kinds the input is used for. The panel needs neither: Rust sends only the inputs an Option uses. */
  applies_to_formation?: string[];
  /** Issue #347: used only when another input holds one of these values (the panel needs none: Rust drops the input). */
  applies_when?: { input: string; values: string[] };
  /** Whether a plain or keyed default is sourced (with its source) or an estimate. */
  default_status?: "sourced" | "estimate";
  default_source?: string;
  kind: "number" | "bool" | "choice" | "cargo" | "formation" | "vessel_scheme" | "vessel_class" | "parameter_set" | "offsets_table" | "openings_list" | "text" | "rule_sets";
  nullable?: boolean;
  /** For `rule_sets`: each built-in Rule Set with its status and unverified blocking Rules. */
  known_sets?: { id: string; version: string; status: string; unverified_blocking: number }[];
  choices?: Choice[];
  default?: unknown;
  default_by?: { by: string; values: Record<string, unknown> };
  /** A default that is a rule, not a value: moulded depth is a ratio of the Cargo group times the beam. */
  default_rule?: {
    form: string;
    by: string;
    ratios: Record<string, number>;
    /** Which group each Cargo is in. */
    cargo_group: Record<string, string>;
    length_to_depth_warning_above: number | null;
    basis: string;
    slenderness_basis: string;
  };
  convoy?: { abreast: Ranges; in_line: Ranges };
}
export interface Schema {
  profile: string;
  engine_version: string;
  groups: string[];
  /** A note to show under a group's name, by group name. */
  group_notes?: Record<string, string>;
  inputs: InputSpec[];
}
export interface Message {
  severity: "error" | "warning";
  scope: "input" | "cross_check" | "option";
  inputs: string[];
  text: string;
  /** The Message Id of `text`, where the engine gives one (ADR 0015). */
  msg?: EngineMsg | null;
}
/** The hull views of a computed Option, drawn in Rust to the Drawing Format Standard, as SVG text. */
export type HullViews = { side: string; plan: string; midship: string } | { message: string };
export type Outcome =
  | { status: "results"; results: Record<string, unknown>; results_hash: string; views?: HullViews }
  | { status: "not_available" | "failed"; message: string; msg?: EngineMsg | null }
  | { status: "not_run" };
export interface OptionEvaluation {
  name: string;
  messages: Message[];
  outcome: Outcome;
  /** null for typed dimensions. */
  vessel_class?: VesselClassSummary | null;
}
export interface Evaluation {
  options: OptionEvaluation[];
  /** Messages about the file as a whole, which no Option owns. */
  file_messages: Message[];
}

export const snapshot = () => platform().snapshot();
export const inputSchema = () => platform().inputSchema();
export const evaluate = () => platform().evaluate();
/** Each of these returns null when the user cancels a dialog. */
export const newFile = () => platform().newFile();
export const openFile = () => platform().openFile();
export const save = () => platform().save();
export const saveAs = () => platform().saveAs();
export const setInput = (option: string, name: string, value: unknown) =>
  platform().setInput(option, name, value);
export const resetInput = (option: string, name: string) => platform().resetInput(option, name);
export const importOffsets = (option: string) => platform().importOffsets(option);
export const setActiveOption = (option: string) => platform().setActiveOption(option);
export const setFormation = (
  option: string,
  kind: string,
  abreast: number | null,
  inLine: number | null,
) => platform().setFormation(option, kind, abreast, inLine);
export const addOption = () => platform().addOption();
export const duplicateOption = (option: string) => platform().duplicateOption(option);
export const renameOption = (option: string, newName: string) => platform().renameOption(option, newName);
export const deleteOption = (option: string) => platform().deleteOption(option);
/** null when the user cancels the file dialog; rejects with the reason when the profile is refused. */
export const loadProfile = () => platform().loadProfile();
export const useBuiltInProfile = () => platform().useBuiltInProfile();
/** The first-run notice: has it been accepted, and remember that it was. */
export const noticeSeen = () => platform().noticeSeen();
export const acceptNotice = () => platform().acceptNotice();
/** True in the browser build. */
export const isWeb = () => platform().kind === "web";
/** Save a drawing view as a download. false where the platform cannot (the desktop app, for now). */
export const exportDrawing = (option: string, view: DrawingView, format: DrawingFormat) =>
  platform().exportDrawing(option, view, format);
export const exportModel = (option: string, format: ModelFormat) => platform().exportModel(option, format);
