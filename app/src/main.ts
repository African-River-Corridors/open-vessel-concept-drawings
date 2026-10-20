import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { initTheme } from "./lib/theme";
import { initLanguage } from "./lib/i18n.svelte";
import { startSplash } from "./lib/splash";
import { initPlatform } from "./lib/platform";

initTheme();
// The Display Language before anything is drawn, so the splash is in it too.
initLanguage();
// The splash covers the wait for the platform too (the web build loads the WebAssembly first).
startSplash();

// The platform (desktop or browser) is ready before the first screen draws.
export default initPlatform().then(() => mount(App, { target: document.getElementById("app")! }));
