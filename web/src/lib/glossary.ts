/**
 * Terms this site uses that mean something specific, and what they mean.
 *
 * # What earns an entry
 *
 * Not every piece of jargon. A term belongs here when **a reader who knows ordinary English gets
 * it wrong** — where the everyday reading and the Ohio reading differ, and the difference changes
 * the number. "Per equivalent pupil" sounds like a per-pupil figure and divides by a count that
 * has been weighted upward for need. "Effective millage" sounds like the rate voters approved and
 * is the rate after H.B. 920 reduced it.
 *
 * A term whose everyday reading is simply *vaguer* than the technical one does not earn an entry.
 * The list is short so that the dotted underlines stay meaningful; a page where every third word
 * is a term has taught the reader to ignore them.
 *
 * # Why the definitions link into the corpus
 *
 * Because the wiki is on the same site for exactly this reason. A tooltip has room for the
 * distinction and not for the evidence; the node has the evidence. Every entry that has a node
 * points at it, and the ones that do not are the ones worth writing next.
 *
 * # An entry is a call site, not a vocabulary
 *
 * An entry no page passes to {@link term} is a definition no reader can reach. #551 found seven:
 * `enrolled-pupil`, `twenty-mill-floor`, `guarantee`, `state-share`, `value-added`, `phase-in`
 * and `tax-year`, written in anticipation and never attached. They were retired rather than
 * attached. Where those words appear on a data page they are mostly already a link to the node
 * that defines them — the twenty-mill floor five times in `tax.ts`, the state share once — and a
 * tooltip beside a link to the same definition says the same thing twice. Git has the
 * definitions; an entry comes back with the call site that needs it.
 *
 * Three came back with #717, each with one: `guarantee` on the aid tile of a district on it,
 * `phase-in` in the Change tab's phase-in card, and `state-share` in the categoricals note, all
 * places the word sat as plain text. Theirs point at the Explained topic rather than the node, and
 * the topic's "How we know" carries the reader on to the node.
 */

import { escapeHtml } from "./format.ts";
import * as routes from "./routes.ts";

/** One defined term. */
export interface Term {
  /** What the definition says, in one or two sentences. Plain text; no markup. */
  definition: string;
  /** The corpus node that holds the evidence, as a site path. */
  href?: string;
  /** Link text, where the node's label is not the term itself. */
  hrefLabel?: string;
}

/**
 * The glossary, keyed by the slug a page passes to {@link term}.
 *
 * Keys are slugs rather than the display text so the same entry can be reached from "per
 * equivalent pupil", "equivalent pupils" and "need-weighted" without three copies of the
 * definition drifting apart.
 */
export const GLOSSARY: Record<string, Term> = {
  "equivalent-pupil": {
    definition:
      "Not a per-pupil figure in the ordinary sense. The denominator is a membership count " +
      "weighted upward for economically disadvantaged, English-learner and disability " +
      "enrollment, so a district serving more need divides by a larger number and reports less " +
      "spending per pupil than it does per child.",
    href: "/wiki/metric/expenditure-per-equivalent-pupil",
    hrefLabel: "Expenditure Per Equivalent Pupil",
  },
  "effective-millage": {
    definition:
      "The rate a district actually levies after H.B. 920's tax reduction factors, not the rate " +
      "its voters approved. The gap between the two is the whole H.B. 920 mechanism expressed as " +
      "a number.",
    href: "/wiki/metric/effective-operating-millage",
    hrefLabel: "Effective Operating Millage",
  },
  "base-cost": {
    definition:
      "What the state treats as the cost of educating one student before categorical additions. " +
      "Under the current plan it is built per district from staffing ratios and salary inputs, " +
      "so two districts of the same size can have different base costs.",
    href: "/wiki/parameter/base-cost-per-pupil",
    hrefLabel: "Base Cost Per Pupil",
  },
  "performance-index": {
    definition:
      "A district's tested students distributed across Ohio's achievement levels, weighted so " +
      "higher levels count for more. It tracks the economically disadvantaged share at −0.85, so " +
      "most of what it appears to say about a district is poverty.",
    href: "/wiki/metric/performance-index",
    hrefLabel: "Performance Index",
  },
  guarantee: {
    definition:
      "A district is on the guarantee when the formula computes less for it than an older " +
      "baseline, and the state pays the baseline instead. It says the formula falls short for " +
      "that district, not that the district is favored, though the money is real.",
    href: routes.explained("guarantee"),
    hrefLabel: "In plain terms",
  },
  "phase-in": {
    definition:
      "The share of the gap between what the formula computes and a district's older base that " +
      "is actually paid. A district below its base is held at the base, so a higher phase-in " +
      "does not reach it.",
    href: routes.explained("phase-in"),
    hrefLabel: "In plain terms",
  },
  "state-share": {
    definition:
      "The fraction of a district's base cost the state pays rather than the district, set by a " +
      "measure of local capacity and floored at a statutory minimum. A district at the minimum " +
      "is one the formula says could fund almost all of itself.",
    href: routes.explained("state-share"),
    hrefLabel: "In plain terms",
  },
  adm: {
    definition:
      "Average daily membership — the pupil count Ohio funds on. Two distinct counts wear the " +
      "name and they differ for every district; the formula uses each in different places.",
    href: "/wiki/metric/enrolled-adm",
    hrefLabel: "Enrolled ADM",
  },
};

/**
 * A term, its dotted underline, and the definition that hangs off it — as HTML.
 *
 * `text` is what the sentence needs to read; `slug` is which entry to attach. They differ often
 * enough that making the text the key would mean the same definition written several ways.
 *
 * Throws on an unknown slug rather than rendering the bare text. A silent fallback would let a
 * renamed entry quietly strip the definitions off a page and leave nothing to notice, which is
 * the same class of failure as a link that resolves to a plausible 404.
 */
export function term(slug: string, text: string): string {
  const entry = GLOSSARY[slug];
  if (!entry) {
    throw new Error(
      `no glossary entry for "${slug}" — add it to GLOSSARY or fix the slug at the call site`,
    );
  }

  const id = `def-${slug}`;
  const link = entry.href
    ? ` <a href="${entry.href}">${escapeHtml(entry.hrefLabel ?? "In the corpus")} →</a>`
    : "";

  return (
    `<span class="term-wrap">` +
    `<button type="button" class="term" aria-describedby="${id}">${escapeHtml(text)}</button>` +
    `<span class="term-def" id="${id}" role="note">${escapeHtml(entry.definition)}${link}</span>` +
    `</span>`
  );
}
