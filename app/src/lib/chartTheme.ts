// Colours and fonts for the uPlot charts, read from the CSS tokens (docs/design/style-guide.md,
// section 7) each time the chart draws, so a theme change needs only a redraw. No colour literal lives
// in a chart.
import type uPlot from "uplot";

export interface ChartTokens {
  surface: string;
  rule: string;
  ink: string;
  ink2: string;
  ink3: string;
  accent: string;
  fail: string;
  gilt: string;
  fontUi: string;
}

/** The token that each field reads. */
const SOURCE: Record<keyof ChartTokens, string> = {
  surface: "--surface",
  rule: "--rule",
  ink: "--ink",
  ink2: "--ink-2",
  ink3: "--ink-3",
  accent: "--accent",
  fail: "--fail",
  gilt: "--gilt-mark",
  fontUi: "--font-ui",
};

/** Read the tokens from a computed style. A token that is not set gives an empty string. */
export function readTokens(style: Pick<CSSStyleDeclaration, "getPropertyValue">): ChartTokens {
  const out = {} as ChartTokens;
  for (const key of Object.keys(SOURCE) as (keyof ChartTokens)[]) out[key] = style.getPropertyValue(SOURCE[key]).trim();
  return out;
}

/** The tokens now in force on <html>. */
export function currentTokens(): ChartTokens {
  return readTokens(getComputedStyle(document.documentElement));
}

/** Dash patterns, in CSS pixels, that tell series apart without colour. Index 0 is solid. */
export const DASHES: number[][] = [[], [7, 4], [2, 3], [9, 3, 2, 3], [13, 4]];

/** The dash pattern for the n-th series (n from 0); the patterns repeat after the last. */
export function dashFor(n: number): number[] {
  return DASHES[((n % DASHES.length) + DASHES.length) % DASHES.length];
}

interface Observable {
  observe(target: unknown, options: { attributes: boolean; attributeFilter: string[] }): void;
  disconnect(): void;
}
interface MediaList {
  addEventListener(type: "change", fn: () => void): void;
  removeEventListener(type: "change", fn: () => void): void;
}

/**
 * Call `onChange` when the theme changes: the `data-theme` attribute of <html> (the theme control) or
 * the operating system's colour scheme (Auto). Returns a function that stops watching.
 */
export function watchTheme(
  onChange: () => void,
  root: unknown = document.documentElement,
  makeObserver: (fn: () => void) => Observable = (fn) => new MutationObserver(fn),
  media: MediaList = matchMedia("(prefers-color-scheme: dark)"),
): () => void {
  const observer = makeObserver(onChange);
  observer.observe(root, { attributes: true, attributeFilter: ["data-theme"] });
  media.addEventListener("change", onChange);
  return () => {
    observer.disconnect();
    media.removeEventListener("change", onChange);
  };
}

/**
 * uPlot draws an axis title and the tick numbers with one colour. The style guide wants the title in
 * `--ink-2` and the numbers in `--ink-3`: these two hooks set the colour of each text by its font
 * (the title font is bold) while the axes draw. Put `titleInk` in `drawClear` and `restoreInk` in
 * `drawAxes`.
 */
export function axisInkHooks(tokens: () => ChartTokens): { drawClear: (u: uPlot) => void; drawAxes: (u: uPlot) => void } {
  const originals = new WeakMap<CanvasRenderingContext2D, CanvasRenderingContext2D["fillText"]>();
  return {
    drawClear(u) {
      const ctx = u.ctx;
      if (originals.has(ctx)) return;
      const original = ctx.fillText.bind(ctx);
      originals.set(ctx, original);
      ctx.fillText = (text: string, x: number, y: number, maxWidth?: number) => {
        const t = tokens();
        ctx.fillStyle = /\bbold\b|\b[6-9]00\b/.test(ctx.font) ? t.ink2 : t.ink3;
        if (maxWidth === undefined) original(text, x, y);
        else original(text, x, y, maxWidth);
      };
    },
    drawAxes(u) {
      const ctx = u.ctx;
      if (!originals.has(ctx)) return;
      // Remove the own property: the prototype's fillText is the real one again.
      delete (ctx as unknown as { fillText?: unknown }).fillText;
      originals.delete(ctx);
    },
  };
}

/** A hatching pattern of diagonal lines in `colour`, for areas that are tinted as well. */
export function hatchPattern(ctx: CanvasRenderingContext2D, colour: string, k: number): CanvasPattern | null {
  const size = Math.round(8 * k);
  const tile = document.createElement("canvas");
  tile.width = tile.height = size;
  const t = tile.getContext("2d");
  if (!t) return null;
  t.strokeStyle = colour;
  t.lineWidth = k;
  t.beginPath();
  t.moveTo(0, size);
  t.lineTo(size, 0);
  t.moveTo(-size / 2, size / 2);
  t.lineTo(size / 2, -size / 2);
  t.moveTo(size / 2, size + size / 2);
  t.lineTo(size + size / 2, size / 2);
  t.stroke();
  return ctx.createPattern(tile, "repeat");
}

/** `font` strings for uPlot and the canvas. */
export const uiFont = (tokens: ChartTokens, px: number, weight = 400) => `${weight} ${px}px ${tokens.fontUi || "sans-serif"}`;
