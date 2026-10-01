/**
 * One card per corpus node, one per catalog source, and one per decision record.
 *
 * All three live here because all three live at `/wiki/<class>/<name>` on the site — `source` and
 * `decision` are the fourteenth and fifteenth pseudo-classes, and splitting the endpoint would put
 * one route family in three files for no reason a reader of any of them would see.
 *
 * The note is the node's own first sentence, through the same `snippet` the page's
 * `<meta name="description">` uses. A corpus node has no number to lead with; what it has is a
 * definition, and the definition is the thing worth carrying into a feed.
 */

import type { APIRoute, GetStaticPaths } from "astro";

import { type Decision, FROM_CATALOG, FROM_DECISION, loadCorpus, type Node, type Source } from "../../../../lib/corpus.ts";
import { count } from "../../../../lib/format.ts";
import type { Card } from "../../../../lib/og/card.ts";
import { SITE } from "../../../../lib/og/pages.ts";
import { respond } from "../../../../lib/og/render.ts";
import { snippet, sourceSnippet } from "../../../../lib/prose.ts";

export const getStaticPaths: GetStaticPaths = () => {
  const corpus = loadCorpus();
  return [
    ...corpus.nodes.map((node) => ({
      params: { class: node.className, node: node.name },
      props: { node, source: null, decision: null },
    })),
    ...corpus.sources.map((source) => ({
      params: { class: "source", node: source.slug },
      props: { node: null, source, decision: null },
    })),
    ...corpus.decisions.map((decision) => ({
      params: { class: "decision", node: decision.slug },
      props: { node: null, source: null, decision },
    })),
  ];
};

/** Exported for the unit suite, which renders the longest label in the corpus. */
export function nodeCard(node: Node, className: string): Card {
  return {
    eyebrow: `${SITE} · Library`,
    headline: node.label,
    // The node's own lead, not a truncation of its body. `summary` is capped at 50 words and
    // carries no markdown, which is exactly what this card can render.
    figureNote: snippet(node.summary, node.className),
    // What the page is, in the words the Library's own headings use. An edge count ("24 links into
    // and out of the corpus") measured the graph, which is not what a feed reader is deciding on.
    meta: className,
  };
}

/** Exported for the unit suite. */
export function sourceCard(source: Source): Card {
  return {
    eyebrow: `${SITE} · Library source`,
    headline: source.title,
    // What the source contains, not its `**Source.** … **Type.**` field list (#593).
    figureNote: sourceSnippet(source.body, FROM_CATALOG),
    meta: `Cited by ${count(source.citedBy.length)} Library ${source.citedBy.length === 1 ? "entry" : "entries"}`,
  };
}

/**
 * Exported for the unit suite.
 *
 * The headline is the title the page carries — the slug as words, see `decisionTitle`. The meta
 * line leads with the withdrawals where there are any: a record that has been corrected is the more
 * interesting of the two kinds, and a card that says so is doing the same job the page's own
 * banner does.
 */
export function decisionCard(decision: Decision): Card {
  return {
    eyebrow: `${SITE} · Decision record`,
    headline: decision.title,
    figureNote: snippet(decision.summary, FROM_DECISION),
    meta:
      decision.corrections > 0
        ? `${count(decision.corrections)} claim${
            decision.corrections === 1 ? "" : "s"
          } since withdrawn or superseded`
        : `Cited by ${count(decision.citedBy.length)} page${
            decision.citedBy.length === 1 ? "" : "s"
          }`,
  };
}

export const GET: APIRoute = async ({ props, params }) => {
  const { node, source, decision } = props as {
    node: Node | null;
    source: Source | null;
    decision: Decision | null;
  };
  if (decision) return respond(decisionCard(decision));
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
