import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { initTheme } from "./lib/theme";
import { startSplash } from "./lib/splash";
import { initPlatform } from "./lib/platform";

initTheme();
// The splash covers the wait for the platform too (the web build loads the WebAssembly first).
startSplash();

// The platform (desktop or browser) is ready before the first screen draws.
export default initPlatform().then(() => mount(App, { target: document.getElementById("app")! }));
