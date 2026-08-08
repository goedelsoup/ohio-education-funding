/**
 * Where things live.
 *
 * One module so a route is renamed in one place rather than in the 40-odd templates that link to
 * it, and so the build-time link checker in `tests/` has something to enumerate against.
 *
 * Every path is absolute from the site root and carries no trailing slash. `build.format` is
 * `"file"`, so `/district/043786` is served by `district/043786.html` and adding a slash would
 * 404 on the static host.
 */

/** The district a route is about. IRN, always — 28 district names in the feed are not unique. */
export const district = (irn: string): string => `/district/${irn}`;
export const districtOutcome = (irn: string): string => `/district/${irn}/outcome`;
export const districtScenario = (irn: string): string => `/district/${irn}/scenario`;
export const districtFinances = (irn: string): string => `/district/${irn}/finances`;

/** A corpus node, by its class and file stem. */
export const wikiNode = (className: string, node: string): string =>
  `/wiki/${className}/${node}`;
/** A corpus class index, which doubles as the rendered ontology for that class. */
export const wikiClass = (className: string): string => `/wiki/${className}`;
/**
 * A catalog entry.
 *
 * Under `/wiki/` rather than at the top level: the sources are part of the same graph as the
 * nodes that cite them, and provenance in this domain is one hop from every numeric claim. It is
 * a fourteenth pseudo-class, not a separate section of the site.
 */
export const wikiSource = (slug: string): string => `/wiki/source/${slug}`;

/** Two districts side by side. Query parameters, because the pair is chosen by the reader. */
export function compare(a?: string, b?: string): string {
  if (!a || !b) return "/compare";
  return `/compare?a=${encodeURIComponent(a)}&b=${encodeURIComponent(b)}`;
}

/**
 * A metric, parameter, or other corpus concept referenced from a data page.
 *
 * The link out from a figure to what the figure *means* is the reason the wiki is worth
 * building; a glossary nobody arrives at from the number they were reading is a document museum.
 */
export const metric = (node: string): string => wikiNode("metric", node);
export const parameter = (node: string): string => wikiNode("parameter", node);
