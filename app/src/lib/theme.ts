// The colour theme: Dark (the default), Light, or Auto, which follows the operating system. The choice
// is display state kept in local storage, per user. Storage can be empty or not available (a private
// window, blocked site data): then the theme is Dark.
export type ThemeChoice = "auto" | "light" | "dark";

export const THEME_KEY = "ovcd.theme";
export const DEFAULT_THEME: ThemeChoice = "dark";
const CHOICES: readonly ThemeChoice[] = ["auto", "light", "dark"];

export function loadTheme(): ThemeChoice {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    return CHOICES.find((c) => c === saved) ?? DEFAULT_THEME;
  } catch {
    return DEFAULT_THEME;
  }
}

export function saveTheme(choice: ThemeChoice) {
  try {
    localStorage.setItem(THEME_KEY, choice);
  } catch {
    // Not remembered; the theme still applies for this run.
  }
}

// Set `data-theme` on <html> for Light and Dark; remove it for Auto.
export function applyTheme(choice: ThemeChoice, root: Pick<HTMLElement, "setAttribute" | "removeAttribute"> = document.documentElement) {
  if (choice === "auto") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", choice);
}

// At start: apply the saved choice.
export function initTheme(): ThemeChoice {
  const choice = loadTheme();
  applyTheme(choice);
  return choice;
}
