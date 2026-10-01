/**
 * One card per ontology class.
 *
 * A class page is both the list of its instances and the definition of what that kind of thing is,
 * so the card carries the first sentence of the definition and counts the instances in plain words:
 * "12 formula components", never "12 nodes" or the class's UFO stereotype, which are the corpus's
 * vocabulary and not a reader's (#593).
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
import { snippet } from "../../../lib/prose.ts";

export const getStaticPaths: GetStaticPaths = () =>
  loadCorpus().classes.map((entry) => ({
    params: { class: entry.className },
    props: { entry },
  }));

/** Exported for the unit suite. */
export function classCard(entry: OntologyClass): Card {
  return {
    eyebrow: `${SITE} · Library`,
    headline: entry.label,
    // The class definition's first sentence. `snippet` is the same function the page's
    // `<meta name="description">` uses, so the two cannot say different things about one class.
    figureNote: snippet(entry.description, entry.className),
    meta: `${count(entry.nodes.length)} ${entry.label.toLowerCase()}`,
  };
}

export const GET: APIRoute = async ({ props }) =>
  respond(classCard((props as { entry: OntologyClass }).entry));
