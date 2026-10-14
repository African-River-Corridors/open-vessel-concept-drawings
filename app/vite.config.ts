import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri expects a fixed dev port and does not want Vite to clear the screen.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  // The font file lives in the drawing crate's vendored standard, one folder up from the app.
  server: { port: 1420, strictPort: true, watch: { ignored: ["**/src-tauri/**"] }, fs: { allow: [".."] } },
});
