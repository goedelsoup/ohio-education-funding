/**
 * The routes the chart gate visits, read off the build rather than written down.
 *
 * The list in `charts.spec.ts` was written by hand and said `/wiki/education-agency/toledo-city`
 * was "the only wiki route with a chart on it". Five wiki routes draw plots, and `/bounds` draws
 * one too; none of them was visited, so the text-size, frame and overlap tests had never run on
 * the TTAG plane whose y title ran 139 units into its neighbour (#608). A hand list is a claim
 * about the build that nothing checks, so this asks the build.
 *
 * # One route per family, not every route
 *
 * 2,433 district pages draw a chart, and they are four templates over 609 districts. Visiting all
 * of them would be the same four drawings with different numbers in, at 2,400 page loads a test.
 * So a route whose path is one of {@link FAMILIES} stands for its family, and the family is
 * visited at its representative. Every other route with a chart is its own page and is visited.
 *
 * `tests/dist/charted.spec.ts` holds the other half: that the families collapse what they claim
 * to, and that nothing else in the build is a family nobody has named.
 *
 * No Playwright import, so the dist project can read the same table.
 */

import { readdirSync, readFileSync } from "node:fs";
import { join, relative, sep } from "node:path";

/** The build this reads. Read when called, not at import: locally the build is the web server's. */
const DIST = join(import.meta.dirname, "../../dist");

/** What marks a document as drawing a chart: the class `draw` puts on every SVG it emits. */
const PLOT = /<svg\b[^>]*\bclass="plot"/;

/**
 * The templated routes, each with the instance the gate visits it at.
 *
 * Cleveland because the district suites already read it and it carries every section; Ottawa and
 * House 90 because they were the hand list's choices, so nothing that passed before moves.
 */
export const FAMILIES: readonly { pattern: RegExp; visit: string }[] = [
  { pattern: /^\/district\/\d{6}$/, visit: "/district/043786" },
  { pattern: /^\/district\/\d{6}\/finances$/, visit: "/district/043786/finances" },
  { pattern: /^\/district\/\d{6}\/outcome$/, visit: "/district/043786/outcome" },
  { pattern: /^\/district\/\d{6}\/taxes$/, visit: "/district/043786/taxes" },
  { pattern: /^\/county\/[a-z-]+$/, visit: "/county/ottawa" },
  { pattern: /^\/house\/\d{3}$/, visit: "/house/090" },
  { pattern: /^\/senate\/\d{2,3}$/, visit: "/senate/001" },
];

/**
 * The routes that draw their charts in the browser, after a fetch.
 *
 * The build cannot say these have a chart, because in the build they do not: the SVG arrives with
 * the scenario. Its stage was the narrow drawing in a 548px box at every desktop width until #609.
 */
export const CLIENT_DRAWN: readonly string[] = ["/scenario/reach"];

/** The address a built file is served at: `district/043786/finances.html` is `/district/043786/finances`. */
function routeOf(file: string): string {
  const path = relative(DIST, file).split(sep).join("/");
  const bare = path.replace(/(^|\/)index\.html$/, "").replace(/\.html$/, "");
  return `/${bare}`;
}

/** Every route in the build whose HTML carries a drawn chart, sorted. */
export function plottedRoutes(dir: string = DIST): string[] {
  const walk = (at: string): string[] =>
    readdirSync(at, { withFileTypes: true }).flatMap((entry) =>
      entry.isDirectory()
        ? walk(join(at, entry.name))
        : entry.name.endsWith(".html")
          ? [join(at, entry.name)]
          : [],
    );
  return walk(dir)
    .filter((file) => PLOT.test(readFileSync(file, "utf8")))
    .map(routeOf)
    .sort();
}

/** The family a route belongs to, or null for a page that is only itself. */
export function familyOf(route: string): (typeof FAMILIES)[number] | null {
  return FAMILIES.find((family) => family.pattern.test(route)) ?? null;
}

/**
 * The routes the chart gate visits: every built page with a chart, a family once at its
 * representative, and the browser-drawn routes after them.
 */
export function chartedRoutes(dir: string = DIST): string[] {
  const out = new Set<string>();
  for (const route of plottedRoutes(dir)) out.add(familyOf(route)?.visit ?? route);
  for (const route of CLIENT_DRAWN) out.add(route);
  return [...out];
}
