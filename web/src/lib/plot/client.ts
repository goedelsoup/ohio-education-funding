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

import { BASE, type Spec } from "./spec.ts";

/** Render a specification to an SVG string, so the callers shared with the build path match. */
export function renderToString(spec: Spec | null): string {
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
  return node.outerHTML;
}
