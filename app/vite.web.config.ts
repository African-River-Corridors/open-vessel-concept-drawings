import { defineConfig } from "vite";
import { readFileSync } from "node:fs";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// The web build: static files for any static server. The page is served at /ovd/app (the deploy
// copies index.html to app.html) and its files live under /ovd/app-files/ (a folder named "app"
// next to app.html would make GitHub Pages redirect /ovd/app). Vite's base makes every URL in the
// built HTML and scripts an absolute /ovd/app-files/... path, including the .wasm and the font.
const { version } = JSON.parse(readFileSync(new URL("./package.json", import.meta.url), "utf8"));

export default defineConfig({
  plugins: [svelte()],
  base: "/ovd/app-files/",
  define: { __APP_VERSION__: JSON.stringify(version), __OVD_WEB__: true },
  build: { outDir: "dist-web", emptyOutDir: true, assetsInlineLimit: 0 },
  server: { fs: { allow: [".."] } },
});
