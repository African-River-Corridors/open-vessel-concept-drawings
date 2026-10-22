// The platform is chosen once, at start: Tauri when the page runs inside the desktop app, else Web.
// The web build defines __OVCD_WEB__ as true and the desktop build as false, so each bundle holds only
// its own adapter (the desktop bundle has no WebAssembly; the web bundle has no Tauri code).
import type { Platform } from "./types";

declare const __OVCD_WEB__: boolean;
export type { Platform, DrawingFormat, DrawingView, ModelFormat } from "./types";

let current: Platform | null = null;

export async function initPlatform(): Promise<Platform> {
  if (current) return current;
  if (__OVCD_WEB__) {
    current = await (await import("./web")).createWebPlatform();
  } else {
    current = (await import("./tauri")).tauriPlatform;
  }
  return current;
}

/** The chosen platform. Call `initPlatform()` first (main.ts does). */
export function platform(): Platform {
  if (!current) throw new Error("the platform is not ready");
  return current;
}
