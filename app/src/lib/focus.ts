/** Scroll to an input in the left panel, open its group if closed, and focus its field. */
export function focusInput(name: string) {
  const field = document.querySelector<HTMLElement>(`[data-input="${CSS.escape(name)}"]`);
  if (!field) return;
  const group = field.closest("details");
  if (group && !group.open) group.open = true;
  field.scrollIntoView({ block: "center" });
  field.querySelector<HTMLElement>("input, select")?.focus();
}
