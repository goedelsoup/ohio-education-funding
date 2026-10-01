/**
 * The same charts, drawn in the browser.
 *
 * Only the scenario routes load this. Their charts change when a slider moves, so there is
 * nothing to bake — and the alternative, a round trip per slider tick, is exactly what the
 * duplicated formula in `policy.ts` exists to avoid.
 *
 * The drawing is `draw` in `spec.ts`, the function `ssr.ts` renders at build time, so the
 * interactive copy of a chart cannot drift away from the static one. What differs is where the
 * document comes from — there is no `linkedom` here, the browser already has a DOM — and what a
 * misaligned hover layer does, which is why this is a separate module and not a branch inside
 * `ssr.ts`.
 *
 * The literal-colour guard is not repeated. A chart drawn in the browser re-renders on a theme
 * change like anything else, so the failure that guard exists to catch cannot happen here.
 */

import { openToKeyboard } from "../chart.ts";
import { draw, type Drawing, type Naming, pair, type Renderer } from "./spec.ts";

/**
 * Ignored rather than thrown: a chart that cannot pair its marks should lose its second channel,
 * not take the page down with it. The build path throws, so a misalignment cannot reach a reader
 * in the first place.
 */
const BROWSER: Renderer = { onMisaligned: () => {} };

/** One SVG, at one width. */
function drawn(build: Drawing, naming: Naming, width: number): string {
  const node = draw(build, naming, width, BROWSER);
  if (!node) return "";
  // Unlike the baked charts, these are only ever produced by a browser that is running this
  // module — so the tab stop can be part of the markup rather than added to it afterwards, and a
  // scenario re-render carries it without the scenario script having to remember.
  openToKeyboard(node);
  return node.outerHTML;
}

/**
 * Render a chart at every one of `WIDTHS`, so the callers shared with the build path match.
 *
 * All three widths are drawn here too, rather than measuring the container and drawing the one that
 * fits. These charts are replaced on every slider tick, so a width read at render time would be
 * the width at that tick — and a reader who then rotates the phone, or drags a desktop window
 * narrow, would keep the layout chosen for the width they no longer have until they moved a lever
 * again. The stylesheet's container query has no such gap, and it is the same rule the 7,599
 * baked charts already answer to.
 */
export function renderToString(build: Drawing, naming: Naming): string {
  return pair((width) => drawn(build, naming, width));
}
