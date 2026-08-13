/**
 * One card per corpus node, and one per catalog source.
 *
 * Both live here because both live at `/wiki/<class>/<name>` on the site — `source` is the
 * fourteenth pseudo-class, and splitting the endpoint would put one route family in two files for
 * no reason a reader of either would see.
 *
 * The figure line is the node's own first sentence, run through the same `summarize` the page's
 * `<meta name="description">` uses. A corpus node has no number to lead with; what it has is a
 * definition, and the definition is the thing worth carrying into a feed.
 */

import type { APIRoute, GetStaticPaths } from "astro";

import { loadCorpus, type Node, type Source } from "../../../../lib/corpus.ts";
import { count } from "../../../../lib/format.ts";
import type { Card } from "../../../../lib/og/card.ts";
import { SITE } from "../../../../lib/og/pages.ts";
import { respond } from "../../../../lib/og/render.ts";
import { summarize } from "../../../../lib/prose.ts";

export const getStaticPaths: GetStaticPaths = () => {
  const corpus = loadCorpus();
  return [
    ...corpus.nodes.map((node) => ({
      params: { class: node.className, node: node.name },
      props: { node, source: null },
    })),
    ...corpus.sources.map((source) => ({
      params: { class: "source", node: source.slug },
      props: { node: null, source },
    })),
  ];
};

/** Exported for the unit suite, which renders the longest label in the corpus. */
export function nodeCard(node: Node, className: string): Card {
  const edges = node.out.length + node.in.length;
  return {
    eyebrow: `${SITE} · ${className}`,
    headline: node.label,
    figureNote: summarize(node.description, 190),
    meta:
      // A corpus is a graph and the edge count is the only honest measure of how much of it a node
      // is load-bearing for. Nodes with no edges exist and say so rather than showing "0 links".
      edges > 0
        ? `${count(edges)} link${edges === 1 ? "" : "s"} into and out of the corpus`
        : "A corpus node",
  };
}

/** Exported for the unit suite. */
export function sourceCard(source: Source): Card {
  return {
    eyebrow: `${SITE} · Wiki source`,
    headline: source.title,
    figureNote: summarize(source.body, 190),
    meta: `Cited by ${count(source.citedBy.length)} node${source.citedBy.length === 1 ? "" : "s"}`,
  };
}

export const GET: APIRoute = async ({ props, params }) => {
  const { node, source } = props as { node: Node | null; source: Source | null };
  if (source) return respond(sourceCard(source));
  return respond(nodeCard(node!, label(params.class ?? "")));
};

/**
 * A class name as a card reads it: `formula-component` → `Formula component`.
 *
 * Read off the slug rather than from `corpus.byClass`, because the class's own `label` is plural
 * ("Formula components") and the eyebrow is naming what *this* node is one of.
 */
function label(className: string): string {
  const words = className.replace(/-/g, " ");
  return words.charAt(0).toUpperCase() + words.slice(1);
}
