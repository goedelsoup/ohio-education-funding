/**
 * One card per ontology class.
 *
 * A class page is both the list of its instances and the definition of what that kind of thing is,
 * so the card leads with the count and carries the first sentence of the definition under it.
 *
 * The source index is *not* here, unlike in `routes.ts` where `source` is a fourteenth
 * pseudo-class. It has no ontology entry to summarize — it is a catalog, not a class — so its card
 * is written in `src/lib/og/pages.ts` with the other written ones.
 */

import type { APIRoute, GetStaticPaths } from "astro";

import { loadCorpus, type OntologyClass } from "../../../lib/corpus.ts";
import { count } from "../../../lib/format.ts";
import type { Card } from "../../../lib/og/card.ts";
import { SITE } from "../../../lib/og/pages.ts";
import { respond } from "../../../lib/og/render.ts";
import { summarize } from "../../../lib/prose.ts";

export const getStaticPaths: GetStaticPaths = () =>
  loadCorpus().classes.map((entry) => ({
    params: { class: entry.className },
    props: { entry },
  }));

/** Exported for the unit suite. */
export function classCard(entry: OntologyClass): Card {
  const nodes = `${count(entry.nodes.length)} node${entry.nodes.length === 1 ? "" : "s"}`;
  return {
    eyebrow: `${SITE} · Wiki`,
    headline: entry.label,
    figure: nodes,
    // The class definition, cut to a card. `summarize` is the same function the page's
    // `<meta name="description">` uses, so the two cannot say different things about one class.
    //
    // 110 is the note limit `render` applies to a card that has a figure. Cutting to the same
    // length here means one ellipsis rather than a summary that is then truncated again.
    figureNote: summarize(entry.description, 110),
    meta: entry.foundationalType ? `Ontology class · ${entry.foundationalType}` : "Ontology class",
  };
}

export const GET: APIRoute = async ({ props }) =>
  respond(classCard((props as { entry: OntologyClass }).entry));
