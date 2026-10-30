// Which side panels are shown. The choice is display state kept in local storage, per user. Storage
// can be empty or not available (a private window, blocked site data): then both panels are shown.
export interface PanelChoice {
  input: boolean;
  outputs: boolean;
}

export const PANELS_KEY = "ovd.panels";
const SHOWN: PanelChoice = { input: true, outputs: true };

export function loadPanels(): PanelChoice {
  try {
    const saved = JSON.parse(localStorage.getItem(PANELS_KEY) ?? "null");
    return {
      input: saved?.input !== false,
      outputs: saved?.outputs !== false,
    };
  } catch {
    return { ...SHOWN };
  }
}

export function savePanels(choice: PanelChoice) {
  try {
    localStorage.setItem(PANELS_KEY, JSON.stringify(choice));
  } catch {
    // Not remembered; the panels still work.
  }
}
