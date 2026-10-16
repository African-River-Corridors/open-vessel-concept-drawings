// The browser implementation: the ovcd-web WebAssembly session, running in this page. No server sees
// a Design. Errors thrown by the session are strings; they become rejected promises with the same text.
import init, { Session } from "../../../wasm-pkg/ovcd_web.js";
import wasmUrl from "../../../wasm-pkg/ovcd_web_bg.wasm?url";
import type { Evaluation, Schema, Snapshot } from "../api";
import type { DrawingFormat, DrawingView, Platform } from "./types";

const LAST_DESIGN = "ovcd.lastDesign";
const NOTICE = "ovcd.noticeSeen";
const DEFAULT_NAME = "design.vdesign";

/** Storage may be missing or throw (private window, blocked site data): every use is guarded. */
function store(): Storage | null {
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}
function readKey(key: string): string | null {
  try {
    return store()?.getItem(key) ?? null;
  } catch {
    return null;
  }
}
function writeKey(key: string, value: string): boolean {
  try {
    store()!.setItem(key, value);
    return true;
  } catch {
    return false;
  }
}

let session: Session;

/** Run a call into the session: parse its JSON, turn a thrown string into a rejection. */
async function call<T>(run: () => string): Promise<T> {
  return JSON.parse(run()) as T;
}

function follow(snap: Snapshot): Snapshot {
  try {
    document.title = snap.title;
  } catch {
    /* no document title to set */
  }
  rememberDesign(snap);
  return snap;
}

/** Keep the last Design in browser storage, so a reload brings it back. */
function rememberDesign(snap: Snapshot) {
  try {
    writeKey(LAST_DESIGN, JSON.stringify({ name: snap.file_name, text: session.save_text() }));
  } catch {
    /* a Design that cannot be saved right now is not remembered */
  }
}

async function change(run: () => string): Promise<Snapshot> {
  return follow(await call<Snapshot>(run));
}

function download(name: string, data: BlobPart, type: string) {
  const url = URL.createObjectURL(new Blob([data], { type }));
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  a.style.display = "none";
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 10000);
}

/** Ask for a file with the browser's picker. null when the user cancels. */
function pickFile(accept: string): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = accept;
    input.dataset.testid = "file-picker";
    // Present but out of sight, so a test can fill it in.
    input.style.cssText = "position:fixed;left:0;top:0;width:1px;height:1px;opacity:0;";
    const done = (file: File | null) => {
      input.remove();
      resolve(file);
    };
    input.addEventListener("change", () => done(input.files?.[0] ?? null));
    input.addEventListener("cancel", () => done(null));
    document.body.append(input);
    input.click();
  });
}

function confirmDiscard(): boolean {
  return !session.is_dirty() || window.confirm("The Design has unsaved changes. Discard them?");
}

async function currentSnapshot(): Promise<Snapshot> {
  return JSON.parse(session.snapshot()) as Snapshot;
}

async function saveNamed(name: string): Promise<Snapshot> {
  const text = session.save_text();
  download(name, text, "application/json");
  return follow(await call<Snapshot>(() => session.saved(name)));
}

function fileNameOf(snap: Snapshot): string {
  return snap.file_name.endsWith(".vdesign") ? snap.file_name : DEFAULT_NAME;
}

export async function createWebPlatform(): Promise<Platform> {
  await init({ module_or_path: wasmUrl });
  session = new Session();

  // Bring back the last Design, if there is one. Any failure leaves the starter Design.
  try {
    const raw = readKey(LAST_DESIGN);
    if (raw) {
      const last = JSON.parse(raw) as { name: string; text: string };
      session.open_text(last.name, last.text);
    }
  } catch {
    /* ignore: the starter Design stays */
  }

  window.addEventListener("beforeunload", (e) => {
    try {
      if (session.is_dirty()) {
        e.preventDefault();
        e.returnValue = "";
      }
    } catch {
      /* nothing to guard */
    }
  });

  const platform: Platform = {
    kind: "web",
    snapshot: async () => follow(await currentSnapshot()),
    inputSchema: () => call<Schema>(() => session.input_schema()),
    evaluate: () => call<Evaluation>(() => session.evaluate()),
    newFile: async () => (confirmDiscard() ? change(() => session.new_design()) : null),
    openFile: async () => {
      if (!confirmDiscard()) return null;
      const file = await pickFile(".vdesign");
      if (!file) return null;
      const text = await file.text();
      return change(() => session.open_text(file.name, text));
    },
    save: async () => saveNamed(fileNameOf(await currentSnapshot())),
    saveAs: async () => {
      const name = window.prompt("File name", fileNameOf(await currentSnapshot()));
      if (name === null || name.trim() === "") return null;
      const clean = name.trim();
      return saveNamed(clean.endsWith(".vdesign") ? clean : `${clean}.vdesign`);
    },
    setInput: (option, name, value) => change(() => session.set_input(option, name, JSON.stringify(value))),
    resetInput: (option, name) => change(() => session.reset_input(option, name)),
    setActiveOption: (option) => change(() => session.set_active_option(option)),
    setFormation: (option, kind, abreast, inLine) =>
      change(() => session.set_formation(option, kind, abreast ?? undefined, inLine ?? undefined)),
    addOption: () => change(() => session.add_option()),
    duplicateOption: (option) => change(() => session.duplicate_option(option)),
    renameOption: (option, newName) => change(() => session.rename_option(option, newName)),
    deleteOption: (option) => change(() => session.delete_option(option)),
    loadProfile: async () => {
      const file = await pickFile(".json");
      if (!file) return null;
      const text = await file.text();
      return change(() => session.load_profile_text(text));
    },
    useBuiltInProfile: () => change(() => session.use_built_in_profile()),
    noticeSeen: async () => readKey(NOTICE) === "1",
    acceptNotice: async () => {
      writeKey(NOTICE, "1");
    },
    exportDrawing: async (option: string, view: DrawingView, format: DrawingFormat) => {
      const base = `${fileNameOf(await currentSnapshot()).replace(/\.vdesign$/, "")}-${option}-${view}`;
      if (format === "svg") download(`${base}.svg`, session.drawing_svg(option, view), "image/svg+xml");
      else if (format === "pdf") download(`${base}.pdf`, new Uint8Array(session.drawing_pdf(option, view)), "application/pdf");
      else download(`${base}.dxf`, new Uint8Array(session.drawing_dxf(option, view)), "application/dxf");
      return true;
    },
  };
  // The smoke test reads results through this; only a build in test mode has it.
  if (import.meta.env.MODE === "test") (window as unknown as { __ovcdPlatform: Platform }).__ovcdPlatform = platform;
  return platform;
}
