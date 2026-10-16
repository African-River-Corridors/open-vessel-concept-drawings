import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// The web build: static files for any static server. The page is served at /ovcd/app (the deploy
// copies index.html to app.html) and its files live under /ovcd/app-files/ (a folder named "app"
// next to app.html would make GitHub Pages redirect /ovcd/app). Vite's base makes every URL in the
// built HTML and scripts an absolute /ovcd/app-files/... path, including the .wasm and the font.
export default defineConfig({
  plugins: [svelte()],
  base: "/ovcd/app-files/",
  define: { __OVCD_WEB__: true },
  build: { outDir: "dist-web", emptyOutDir: true, assetsInlineLimit: 0 },
  server: { fs: { allow: [".."] } },
});
