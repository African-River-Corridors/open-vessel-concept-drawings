import { mount } from "svelte";
import "./app.css";
import App from "./App.svelte";
import { initTheme } from "./lib/theme";
import { startSplash } from "./lib/splash";

initTheme();

export default mount(App, { target: document.getElementById("app")! });
startSplash();
