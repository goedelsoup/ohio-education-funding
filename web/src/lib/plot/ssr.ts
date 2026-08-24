/**
 * Observable Plot, rendered to SVG at build time.
 *
 * # Why server-side and not in the browser
 *
 * Plot needs a DOM. The usual answer is to ship it and draw on load, which would make it this
 * site's first runtime dependency — roughly 100 KB gzipped of Plot and d3 on every page — and
 * would mean the charts are the one part of a document that arrives empty. Both are avoidable:
 * `linkedom` supplies a DOM here, Plot draws into it during `astro build`, and what reaches the
 * reader is plain SVG inside the HTML. Plot stays a devDependency for every page that uses this.
 *
 * The scenario routes are the deliberate exception. Their charts change when a slider moves, so
 * those pages import Plot for real through `client.ts` — same specifications, different document.
 *
 * Nothing in this module may be imported by client code: `linkedom` is 200 KB and there is no
 * reason for a browser to have a second DOM implementation inside its first one.
 */

import * as Plot from "@observablehq/plot";
import { parseHTML } from "linkedom";

import { applyNaming, BASE, type Drawing, type Naming, WIDTHS } from "./spec.ts";

/** Anything that looks like a baked-in colour: `#abc`, `#aabbcc`, `rgb(…)`, `hsl(…)`. */
const LITERAL_COLOUR = /(#[0-9a-f]{3,8}\b|\brgba?\(|\bhsla?\()/i;

/**
 * Refuse to emit a chart carrying a literal colour.
 *
 * A hex code in build-time SVG is a chart that ignores the theme, and the mistake is invisible to
 * whoever makes it: they are looking at light mode, where the literal is very nearly right. The
 * failure only shows up for a reader on a dark page, which is not a reader this build has. So it
 * stops the build instead.
 *
 * Plot's own `<style>` block is excluded. It sets type, display and overflow, and the one colour
 * in it is a `--plot-background` custom property this platform never uses.
 */
function ensureThemeable(svg: string): string {
  const found = svg.replace(/<style>[\s\S]*?<\/style>/g, "").match(LITERAL_COLOUR);
  if (found) {
    throw new Error(
      `A chart was rendered with the literal colour ${found[0]}. Build-time SVG cannot re-render ` +
        `when the reader switches theme, so every colour must be a var(--…) reference into the ` +
        `stylesheet. Use the SERIES and INK tokens in src/lib/plot/tokens.ts.`,
    );
  }
  return svg;
}

/**
 * Put `data-hover` on each rendered mark, in data order.
 *
 * Plot emits marks in the order it is given them, so index alignment holds — but that is an
 * assumption about someone else's renderer, so it is checked rather than trusted. A mismatch
 * means Plot dropped, filtered or reordered something, and tooltips attached by index would then
 * label the wrong quantities: worse than having none, because they would look right.
 *
 * Applied here rather than through Plot's `title` channel, which produces the browser's native
 * tooltip — slow to appear, unstyleable, and unable to hold the two-clause sentences these charts
 * need.
 */
export function attachHovers(root: Element, selector: string, text: string[]): void {
  const marks = root.querySelectorAll(selector);
  if (marks.length !== text.length) {
    throw new Error(
      `The hover layer does not line up with what Plot drew: ${text.length} values against ` +
        `${marks.length} marks matching "${selector}". Attaching them by index would label the ` +
        `wrong quantities.`,
    );
  }
  marks.forEach((mark, index) => {
    (mark as Element).setAttribute("data-hover", text[index]!);
  });
}

/** One SVG, at one width. */
function draw(build: Drawing, naming: Naming, width: number): string {
  const spec = build(width);
  if (!spec) return "";
  const { document } = parseHTML("<!doctype html><html><body></body></html>");
  const node = Plot.plot({ ...BASE, ...spec.options, document }) as unknown as Element;
  if (spec.hovers) attachHovers(node, spec.hovers.selector, spec.hovers.text);
  applyNaming(node, naming);
  return ensureThemeable(node.outerHTML);
}

/**
 * Render a chart to a pair of SVGs, for putting straight into a document.
 *
 * # Why two
 *
 * A static SVG has one layout and the stylesheet used to make it fit by scaling it. At 375px that
 * scale is 0.46, which took the axis text to about 4.6px — see {@link WIDTHS} for why enlarging
 * the type instead does not work. So the drawing is laid out twice, at the two widths it is
 * actually shown at, and `app.css` picks with a container query. Both are in the document; the
 * one the reader is not looking at is `display: none`, which also takes it out of the
 * accessibility tree, so a screen reader is offered one chart rather than the same chart twice.
 *
 * `null` renders to nothing. A spec builder returns null when the data cannot support the form it
 * was asked for — a fan chart of one observation, say — and an empty string is the honest output:
 * better than an axis with a single mark on it, which would read as a finding. That decision is
 * about the data and never about the width, so it is taken once, on the wide drawing, and the
 * narrow one is not asked.
 */
export function renderToString(build: Drawing, naming: Naming): string {
  const wide = draw(build, naming, WIDTHS.wide);
  if (!wide) return "";
  const narrow = draw(build, naming, WIDTHS.narrow);
  return (
    `<div class="chart-pair">` +
    `<div class="chart-at" data-at="narrow">${narrow}</div>` +
    `<div class="chart-at" data-at="wide">${wide}</div>` +
    `</div>`
  );
}
