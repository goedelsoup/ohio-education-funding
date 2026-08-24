/**
 * The same charts, drawn in the browser.
 *
 * Only the scenario routes load this. Their charts change when a slider moves, so there is
 * nothing to bake — and the alternative, a round trip per slider tick, is exactly what the
 * duplicated formula in `policy.ts` exists to avoid.
 *
 * The specifications come from `spec.ts`, identical to the ones `ssr.ts` renders at build time,
 * so the interactive copy of a chart cannot drift away from the static one. What differs is a
 * single line: where the document comes from. There is no `linkedom` here — the browser already
 * has a DOM — which is why this is a separate module and not a branch inside `ssr.ts`.
 *
 * The literal-colour guard is not repeated. A chart drawn in the browser re-renders on a theme
 * change like anything else, so the failure that guard exists to catch cannot happen here.
 */

import * as Plot from "@observablehq/plot";

import { applyNaming, BASE, type Drawing, type Naming, WIDTHS } from "./spec.ts";

/** One SVG, at one width. */
function draw(build: Drawing, naming: Naming, width: number): string {
  const spec = build(width);
  if (!spec) return "";
  const node = Plot.plot({ ...BASE, ...spec.options }) as unknown as Element;
  if (spec.hovers) {
    const marks = node.querySelectorAll(spec.hovers.selector);
    // Same check as the build path, for the same reason: tooltips attached by index to a mark
    // list Plot has reordered would label the wrong quantities and look correct doing it.
    if (marks.length === spec.hovers.text.length) {
      marks.forEach((mark, index) => {
        mark.setAttribute("data-hover", spec.hovers!.text[index]!);
      });
    }
  }
  applyNaming(node, naming);
  return node.outerHTML;
}

/**
 * Render a chart to a pair of SVGs, so the callers shared with the build path match.
 *
 * Both widths are drawn here too, rather than measuring the container and drawing the one that
 * fits. These charts are replaced on every slider tick, so a width read at render time would be
 * the width at that tick — and a reader who then rotates the phone, or drags a desktop window
 * narrow, would keep the layout chosen for the width they no longer have until they moved a lever
 * again. The stylesheet's container query has no such gap, and it is the same rule the 7,599
 * baked charts already answer to.
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
