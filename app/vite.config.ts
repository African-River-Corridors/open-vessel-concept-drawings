import { defineConfig } from "vite";
import { readFileSync } from "node:fs";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri expects a fixed dev port and does not want Vite to clear the screen.
const { version } = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8"));

export default defineConfig({
  // The desktop bundle holds the Tauri adapter only (the web build sets __OVD_WEB__ to true).
  define: { __APP_VERSION__: JSON.stringify(version), __OVD_WEB__: false },
  plugins: [svelte()],
  clearScreen: false,
  // The font file lives in the drawing crate's vendored standard, one folder up from the app.
  server: { port: 1420, strictPort: true, watch: { ignored: ["**/src-tauri/**"] }, fs: { allow: [".."] } },
});
