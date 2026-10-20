// The loading screen (markup in index.html). It is shown from page load until the first result
// arrives, then removed from the DOM. A timeout and any startup error remove it too, so it never
// blocks the app. The splash itself has pointer-events: none (splash.css).
import { parts, t } from "./i18n.svelte";

const TIMEOUT_MS = 15000;

export function startSplash(): void {
  const splash = document.getElementById("splash");
  if (!splash) return;
  const version = document.getElementById("splash-version");
  if (version) version.textContent = __APP_VERSION__;
  translate(splash);

  let done = false;
  // The header's file label shows once the first result is in; an error banner also ends the wait.
  const observer = new MutationObserver(() => check());
  const check = () => {
    if (document.querySelector("header .meta, [role=alert]")) finish();
  };
  const timer = window.setTimeout(finish, TIMEOUT_MS);

  function finish(): void {
    if (done) return;
    done = true;
    observer.disconnect();
    window.clearTimeout(timer);
    window.removeEventListener("error", finish);
    window.removeEventListener("unhandledrejection", finish);
    splash?.remove();
  }

  window.addEventListener("error", finish);
  window.addEventListener("unhandledrejection", finish);
  observer.observe(document.body, { childList: true, subtree: true });
  check();
}

/** The splash text in the Display Language. The markup holds the English, for the moment before this runs. */
function translate(splash: HTMLElement): void {
  for (const el of splash.querySelectorAll<HTMLElement>("[data-i18n]")) el.textContent = t(el.dataset.i18n!);
  if (splash.dataset.i18nAria) splash.setAttribute("aria-label", t(splash.dataset.i18nAria));
  const tagline = splash.querySelector<HTMLElement>("[data-i18n-tagline]");
  if (!tagline) return;
  tagline.replaceChildren(
    ...parts("app-splash-tagline", ["design"]).map((p) => {
      if (typeof p === "string") return document.createTextNode(p);
      const em = document.createElement("em");
      em.textContent = t("app-splash-tagline-design");
      return em;
    }),
  );
}
