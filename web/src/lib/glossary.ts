/**
 * Terms this site uses that mean something specific, and what they mean.
 *
 * # What earns an entry
 *
 * Not every piece of jargon. A term belongs here when **a reader who knows ordinary English gets
 * it wrong** — where the everyday reading and the Ohio reading differ, and the difference changes
 * the number. "Per equivalent pupil" sounds like a per-pupil figure and divides by a count that
 * has been weighted upward for need. "Effective millage" sounds like the rate voters approved and
 * is the rate after H.B. 920 reduced it. "On the guarantee" sounds like a benefit and is the
 * statement that the formula computes less for the district than it used to get.
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
 */

import { escapeHtml } from "./format.ts";

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
  "enrolled-pupil": {
    definition:
      "A headcount denominator: spending divided by pupils, with no weighting. Comparable across " +
      "districts in a way the need-weighted figure beside it is not.",
    href: "/wiki/metric/per-pupil-operating-expenditure",
    hrefLabel: "Per-Pupil Operating Expenditure",
  },
  "effective-millage": {
    definition:
      "The rate a district actually levies after H.B. 920's tax reduction factors, not the rate " +
      "its voters approved. The gap between the two is the whole H.B. 920 mechanism expressed as " +
      "a number.",
    href: "/wiki/metric/effective-operating-millage",
    hrefLabel: "Effective Operating Millage",
  },
  "twenty-mill-floor": {
    definition:
      "The limit below which reduction factors may not push a district's effective operating " +
      "rate. At the floor, growth in property value reaches revenue; above it, it does not.",
    href: "/wiki/parameter/twenty-mill-floor",
    hrefLabel: "Twenty-Mill Floor",
  },
  guarantee: {
    definition:
      "A district is on the guarantee when the formula computes less for it than its FY2020 " +
      "baseline, and the state pays the baseline instead. It is a statement about the formula " +
      "falling short, not about the district being favoured — though the money is real.",
    href: "/wiki/formula-component/temporary-transitional-aid-guarantee",
    hrefLabel: "Temporary Transitional Aid Guarantee",
  },
  "state-share": {
    definition:
      "The fraction of a district's base cost the state pays rather than the district, set by a " +
      "measure of local capacity and floored at a statutory minimum. A district at the minimum " +
      "is one the formula says can fund itself.",
    href: "/wiki/metric/state-share-percentage",
    hrefLabel: "State Share Percentage",
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
  "value-added": {
    definition:
      "Ohio's growth measure: observed achievement against what the state's model predicted from " +
      "each student's own prior scores. The figure used here is the three-year average, which is " +
      "the department's headline form.",
    href: "/wiki/metric/progress-value-added",
    hrefLabel: "Progress (Value-Added)",
  },
  "phase-in": {
    definition:
      "The fraction of the plan's computed formula amount that is actually appropriated. A " +
      "district funded at 100% of the phase-in is funded at 100% of a figure priced from FY2022 " +
      "salary inputs.",
    href: "/wiki/parameter/fsfp-phase-in-percentage",
    hrefLabel: "FSFP Phase-In Percentage",
  },
  adm: {
    definition:
      "Average daily membership — the pupil count Ohio funds on. Two distinct counts wear the " +
      "name and they differ for every district; the formula uses each in different places.",
    href: "/wiki/metric/enrolled-adm",
    hrefLabel: "Enrolled ADM",
  },
  "tax-year": {
    definition:
      "A calendar year of valuation and levy. The revenue a tax year raises reaches a district " +
      "in the following fiscal year, so a 2024 tax year and an FY2024 budget are eleven months " +
      "apart.",
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
