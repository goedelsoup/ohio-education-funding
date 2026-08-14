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
export const districtTaxes = (irn: string): string => `/district/${irn}/taxes`;

/**
 * A county, by its slug.
 *
 * Counties are a peer group rather than a boundary here — the department attributes each district
 * to one county and school district lines cross county lines freely — so the route is
 * `/county/…` singular, naming the page's subject rather than claiming an aggregate.
 */
export const county = (slug: string): string => `/county/${slug}`;

/**
 * An Ohio House district, by its number.
 *
 * Zero-padded to three characters, as the Census files number them: `/house/024`. Unpadded would
 * make `/house/24` and `/house/024` two paths for one district on a static host.
 */
export const houseDistrict = (number: string): string => `/house/${number}`;
/** And the Senate, on the same shape. */
export const senateDistrict = (number: string): string => `/senate/${number}`;
/** Either chamber's index. */
export const chamberIndex = (chamber: "house" | "senate"): string => `/${chamber}`;

/**
 * The path a built page is *served* at, from the path it was written to.
 *
 * `build.format` is `"file"`, so Astro writes `/district/043786` to `district/043786.html` — and
 * during the build `Astro.url.pathname` reports the second of those, because that is the file being
 * produced. Anything a page says about its own address therefore has to be put back: the canonical
 * link and `og:url` both name a URL a reader can visit, and `…/043786.html` is not one.
 *
 * This is not hypothetical tidying. The canonical link shipped as `…/district/043786.html` while
 * `sitemap-0.xml` listed `…/district/043786`, which is one page telling a crawler two different
 * things about which of its two addresses is the real one.
 */
export function canonicalPath(pathname: string): string {
  // `/index.html` is the site root, not a page called "index" — and it is the only page whose
  // served path is shorter than its name rather than the same minus an extension.
  if (pathname === "/index.html" || pathname === "/index") return "/";
  const stripped = pathname.endsWith(".html") ? pathname.slice(0, -".html".length) : pathname;
  // A trailing slash 404s on the static host, so it is never correct to leave one on.
  return stripped.length > 1 && stripped.endsWith("/") ? stripped.slice(0, -1) : stripped;
}

/**
 * The preview cards, which are routes like any other.
 *
 * Under `/og/` and mirroring the page routes below them, so the card for `/county/allen` is at
 * `/og/county/allen.png` and neither can be renamed without the other being obviously wrong.
 *
 * A district has five pages and one card. `/district/X/finances` and `/district/X/taxes` are the
 * same district and the preview says so; what distinguishes them in a feed is `og:title`, which
 * carries the page's own title. Rendering five near-identical images per district would be 3,045
 * of them rather than 609, for a difference no reader would see.
 */
export const og = {
  /** Any page that has not been given a card of its own. */
  default: (): string => "/og/default.png",
  district: (irn: string): string => `/og/district/${irn}.png`,
  county: (slug: string): string => `/og/county/${slug}.png`,
  chamberDistrict: (chamber: "house" | "senate", number: string): string =>
    `/og/${chamber}/${number}.png`,
  wikiClass: (className: string): string => `/og/wiki/${className}.png`,
  /**
   * A node's card — and a source's, since `source` is the fourteenth pseudo-class here exactly as
   * it is in the page routes above.
   */
  wikiNode: (className: string, node: string): string => `/og/wiki/${className}/${node}.png`,
  wikiSource: (slug: string): string => `/og/wiki/source/${slug}.png`,
  wikiDecision: (slug: string): string => `/og/wiki/decision/${slug}.png`,
  /** A top-level page — `/`, `/method`, `/data` — keyed by the slug in `src/lib/og/pages.ts`. */
  page: (slug: string): string => `/og/page/${slug}.png`,
} as const;

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

/**
 * A decision record.
 *
 * `decision`, singular, for the same reason `source` is: the segment names what one page is, not
 * the directory the file came out of. `/wiki/decisions/…` was the shape produced by an old bug
 * that read the `decisions/` subtree as an ontology class, and it is worth never building again.
 */
export const wikiDecision = (slug: string): string => `/wiki/decision/${slug}`;

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
