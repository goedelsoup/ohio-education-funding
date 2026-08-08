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
import { loadFeed } from "../lib/feed.ts";
import * as routes from "../lib/routes.ts";

/** One findable thing. Field names are short because there are ~700 of these. */
export interface SearchEntry {
  /** Title, as it should appear in results. */
  t: string;
  /** URL. */
  u: string;
  /** Kind, for grouping and for the label on a result. */
  k: "district" | "node" | "source" | "page";
  /** Extra searchable text — an IRN, a class name, a synonym. Not displayed verbatim. */
  a?: string;
}

export const GET: APIRoute = () => {
  const { alphabetical } = loadFeed();

  const entries: SearchEntry[] = [
    { t: "Statewide", u: "/", k: "page", a: "guarantee wealth neutrality floors summary" },
    { t: "Districts", u: "/districts", k: "page", a: "index list all table" },
    { t: "Outcomes", u: "/outcomes", k: "page", a: "performance index poverty achievement growth" },
    { t: "Scenario", u: "/scenario", k: "page", a: "levers simulation winners losers policy" },
    { t: "Compare districts", u: "/compare", k: "page", a: "two side by side peer" },
    { t: "Method", u: "/method", k: "page", a: "verification provenance sources not built" },
    { t: "Data", u: "/data", k: "page", a: "download csv json bundle feed" },
    { t: "The corpus", u: "/wiki", k: "page", a: "wiki nodes ontology" },
  ];

  for (const d of alphabetical) {
    entries.push({ t: d.name, u: routes.district(d.irn), k: "district", a: d.irn });
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
