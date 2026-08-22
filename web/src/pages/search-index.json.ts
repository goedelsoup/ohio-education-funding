/**
 * One flat index of everything on this site that a reader could be looking for.
 *
 * Emitted as a static file at build time — `output: "static"` turns this endpoint into
 * `dist/search-index.json` — so search is a fetch of one small document and a filter in the
 * browser, with nothing running behind the site. That is the same trade the rest of the platform
 * makes, and it is the only one available: a static host has no query endpoint to offer.
 *
 * Kept deliberately narrow. Enough to *find* the page — its title, its kind, a few searchable
 * aliases — and nothing to *read* on it, because the moment this carries prose it stops being a
 * few tens of kilobytes.
 */

import type { APIRoute } from "astro";

import { loadCorpus } from "../lib/corpus.ts";
import { counties } from "../lib/county.ts";
import { loadFeed, qualifiedName } from "../lib/feed.ts";
import * as routes from "../lib/routes.ts";

/** One findable thing. Field names are short because there are ~700 of these. */
export interface SearchEntry {
  /** Title, as it should appear in results. */
  t: string;
  /** URL. */
  u: string;
  /** Kind, for grouping and for the label on a result. */
  k: "district" | "county" | "seat" | "node" | "source" | "page";
  /** Extra searchable text — an IRN, a class name, a synonym. Not displayed verbatim. */
  a?: string;
}

export const GET: APIRoute = () => {
  const { alphabetical, bundle } = loadFeed();

  const entries: SearchEntry[] = [
    // `/statewide`, not `/`. This entry pointed at the front door for as long as it existed, so
    // searching for the name of a page navigated the reader away from it — the one failure a
    // search index can have that looks like a working result.
    { t: "Statewide", u: "/statewide", k: "page", a: "guarantee wealth neutrality floors summary" },
    { t: "Districts", u: "/districts", k: "page", a: "index list all table" },
    { t: "Outcomes", u: "/outcomes", k: "page", a: "performance index poverty achievement growth" },
    { t: "Scenario", u: "/scenario", k: "page", a: "levers simulation winners losers policy" },
    { t: "Compare districts", u: "/compare", k: "page", a: "two side by side peer" },
    { t: "Method", u: "/method", k: "page", a: "verification provenance sources not built" },
    { t: "Data", u: "/data", k: "page", a: "download csv json bundle feed" },
    { t: "The corpus", u: "/wiki", k: "page", a: "wiki nodes ontology" },
    { t: "Counties", u: "/counties", k: "page", a: "index list disparity spread" },
    { t: "House districts", u: routes.chamberIndex("house"), k: "page", a: "legislature seats roster" },
    { t: "Senate districts", u: routes.chamberIndex("senate"), k: "page", a: "legislature seats roster" },
    { t: "History", u: "/history", k: "page", a: "regimes timeline where the money came from" },
    { t: "Legislation", u: "/legislation", k: "page", a: "acts bills statute general assembly" },
  ];

  for (const d of alphabetical) {
    entries.push({ t: qualifiedName(d), u: routes.district(d.irn), k: "district", a: d.irn });
  }

  /*
   * The place-based routes, which were the whole of what this index could not find.
   *
   * 88 counties and 132 legislative seats — a fifth of the site — were reachable only by link or
   * by typing the URL. A reader looking for their county by name got nothing back, which reads as
   * "this site has no page for that" rather than as "search does not cover it".
   *
   * Titled exactly as the pages title themselves, so a result and its destination agree.
   */
  for (const county of counties(bundle.districts)) {
    entries.push({ t: `${county.name} County`, u: routes.county(county.slug), k: "county", a: county.slug });
  }
  for (const [chamber, seats] of [
    ["house", bundle.house_districts],
    ["senate", bundle.senate_districts],
  ] as const) {
    const label = chamber === "house" ? "House" : "Senate";
    for (const seat of seats) {
      entries.push({
        t: `${label} District ${seat.number}`,
        u: chamber === "house" ? routes.houseDistrict(seat.number) : routes.senateDistrict(seat.number),
        k: "seat",
        a: `${chamber} seat`,
      });
    }
  }

  const corpus = loadCorpus();
  for (const entry of corpus.classes) {
    entries.push({ t: entry.label, u: routes.wikiClass(entry.className), k: "node", a: entry.className });
  }
  for (const node of corpus.nodes) {
    // The class name is searchable alongside the label, so "legislation" finds every bill and
    // "metric" finds every measure without the reader knowing the site's URL scheme.
    entries.push({
      t: node.label,
      u: routes.wikiNode(node.className, node.name),
      k: "node",
      a: `${node.className} ${node.name}`,
    });
  }
  for (const source of corpus.sources) {
    entries.push({ t: source.title, u: routes.wikiSource(source.slug), k: "source", a: source.slug });
  }

  return new Response(JSON.stringify(entries), {
    headers: { "content-type": "application/json; charset=utf-8" },
  });
};
