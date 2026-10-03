/**
 * One Explained page, as a type: the eight sections #712's template requires, each a field.
 *
 * # Why the template is a type
 *
 * The plan's template has eight sections, and two of them are the ones a writer drops first: the
 * section that says where the page's comparison breaks, and the one that names the evidence. A
 * checklist in an issue does not stop either. A required field does — a topic module that leaves one
 * out does not compile, and `tests/unit/explained.spec.ts` refuses one that fills it with nothing.
 *
 * # Why the pages are modules and not corpus nodes
 *
 * `.yidam/decisions/explained-is-a-reading-not-a-record.yml` records it. In short: the corpus is an
 * evidence register with gates built for that register, and these pages are a reading of it. Each
 * one computes its numbers from the feed, the way `bounds.ts` does, and ends at the corpus nodes
 * that prove what it says ({@link Topic.howWeKnow}). The claim badges appear there and nowhere else.
 *
 * # Prose is HTML, and it is built, not typed
 *
 * Every {@link Prose} value is a string of HTML assembled from template literals and `format.ts`
 * calls. No number or year is typed into one: phase 1's source scan rejects a digit inside a prose
 * template, apart from the statute and bill numbers on its allowlist, because a typed figure is one
 * a regenerated panel makes wrong with nothing to notice (#702's `money(500)`).
 */

import type { SeriesKey } from "../year.ts";

/**
 * The runs of the bar's panel, in the order a reader meets them.
 *
 * A topic names one, and `nav()` groups the registry under these headings. Three, because #712's
 * plan asked for three or four and three hold the first wave: how the money is divided, why it
 * changes from year to year, and what the figures can and cannot show.
 */
export const GROUPS = [
  "How the money is split",
  "Why it moves",
  "What it does and does not show",
] as const;

/** One of {@link GROUPS}. */
export type Group = (typeof GROUPS)[number];

/**
 * A run of plain prose, as HTML.
 *
 * Inline markup only — `<strong>`, `<em>`, `<a>`, a year chip, a `<var>` — because the renderer
 * wraps it in its own `<p>` or `<li>`. A block element inside one would be invalid HTML.
 */
export type Prose = string;

/** Section 5: the worked example on one real district, and at most one chart. */
export interface Numbers {
  /**
   * The year the example's figures are in, as the chip on the section heading.
   *
   * Every figure on the page carries a year (`tests/dist/figures.spec.ts`). The example reads one
   * series, so one chip on the heading dates all of it.
   */
  chip: SeriesKey;
  /**
   * The example's result in one sentence, with its figure: the card's bold lead.
   *
   * A chart card on this site opens on a bold sentence holding a number
   * (`tests/dist/findings.spec.ts`), so that a reader who stops there has the answer. Here it is
   * the worked example's outcome, which the steps then derive.
   */
  finding: Prose;
  /** Who the example district is and why it was chosen: the rule, not a name typed by hand. */
  intro: Prose[];
  /** The steps, in order. Each is one sentence a reader can check against the one before it. */
  steps: Prose[];
  /**
   * One chart from an existing spec, rendered to markup, and what it shows.
   *
   * Optional. A picture of every district is often the clearest form of the point, and it is
   * never the only one: the steps carry the argument and the chart shows its spread.
   */
  chart?: { svg: string; caption: Prose };
  /** What the example shows, once the steps are done. */
  after: Prose[];
}

/** Section 6: the formula, written out, and where the law says it. */
export interface Rule {
  /**
   * The rule in notation, as HTML: `<var>` for a quantity and `<sub>` for an index.
   *
   * No maths library. #182 holds every page to zero external requests, and one formula per page
   * does not need a typesetter.
   */
  notation: string;
  /** The section of the Revised Code that states the rule, as written: "R.C. 3317.022". */
  statute: string;
  /** Where a reader can read it: the corpus node that vendors the section, or the code itself. */
  href: string;
  /** What each symbol in {@link notation} stands for, one item each. */
  symbols: Prose[];
}

/**
 * One line of section 7: a claim the page makes, and the evidence for it.
 *
 * The claim is written in the corpus's register and opens with its tag — `[verified]`,
 * `[inference]` — which the renderer turns into the same badge a corpus node shows.
 */
export interface Proof {
  claim: string;
  /** Corpus nodes, as `class/slug`. Each must exist; the renderer throws on one that does not. */
  nodes: string[];
  /** Keys of `crates/figures.json` that bind the figures the claim rests on. */
  figures?: string[];
}

/** Section 8: where to go next. */
export interface ReadNext {
  /** Slugs of other topics, two or three where they exist. The renderer throws on an unknown one. */
  topics: string[];
  /** The data page where a reader's own district shows this term: a district tab, not a widget. */
  data: { href: string; label: string; note: Prose };
}

/** What a topic is called and where it sits, before anything is computed. */
export interface TopicMeta {
  /** The address under `/explained/`, and the key every other topic names it by. */
  slug: string;
  /** Which run of the bar's panel it sits in. */
  group: Group;
  /**
   * Section 1: the question as a reader would type it. The `h1`, the `<title>` and the bar's label.
   *
   * Ends in `?` and runs no longer than {@link QUESTION_LIMIT}.
   */
  question: string;
}

/** The seven sections a topic computes. */
export interface TopicBody {
  /**
   * Section 2: two or three sentences, readable alone.
   *
   * `lead` is bold and `rest` follows it. Together they are the page's meta description, so they
   * run no longer than `DESCRIPTION_LIMIT` — see {@link description}.
   */
  shortAnswer: { lead: Prose; rest: Prose };
  /** Section 3: one comparison from common life, in one paragraph. */
  picture: Prose;
  /** Section 4: what the comparison gets wrong. Required: a picture with no limits misleads. */
  breaks: Prose[];
  numbers: Numbers;
  rule: Rule;
  /** Section 7: one item per claim, and at least one. */
  howWeKnow: Proof[];
  readNext: ReadNext;
}

/** A topic, whole. */
export type Topic = TopicMeta & TopicBody;

/**
 * A topic as the registry holds it: its name now, its sections on demand.
 *
 * The bar lists every topic on every page, and computing a worked example across the feed to print
 * one link would be the whole page's work for one line of it. So the name is data and the body is a
 * function, called by the page that renders it.
 */
export interface TopicModule extends TopicMeta {
  build: () => TopicBody;
}

/** The longest a question may run: the `h1` on one line at phone width, and a bar label. */
export const QUESTION_LIMIT = 70;

/** A topic, built. */
export function topic(module: TopicModule): Topic {
  const { build, ...meta } = module;
  return { ...meta, ...build() };
}

/** Markup to text, for a meta description and for the style checks. */
export function plain(html: string): string {
  return html
    .replace(/<span class="year-chip-def"[^>]*>.*?<\/span>/g, "")
    .replace(/<[^>]+>/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/\s+/g, " ")
    .replace(/\s+([.,;:?!)])/g, "$1")
    .trim();
}

/** The short answer as one line of text: the page's meta description. */
export function description(body: Pick<TopicBody, "shortAnswer">): string {
  return plain(`${body.shortAnswer.lead} ${body.shortAnswer.rest}`);
}

/**
 * A topic's stated identity, as two sides computed per district.
 *
 * Each topic's worked example states a relation — `paid = base + p × (computed − base)` — and the
 * page is only right if it holds on every district, not on the one it shows. A topic module exports
 * one of these beside itself, and its test runs {@link reconciles} over the year the page reads.
 */
export interface Identity<R extends { irn: string }> {
  /** The left side, as the payment report or the feed states it. */
  left: (row: R) => number;
  /** The right side, as the page's rule computes it. */
  right: (row: R) => number;
  /**
   * How far apart the two may be: half the published precision. A figure published to the dollar
   * reconciles within 0.5, and a tolerance wider than that would pass a rounding the page does not do.
   */
  tolerance: number;
}

/**
 * The districts, by IRN, on which an identity fails. Empty when it holds everywhere.
 *
 * Keyed on IRN because district names are not unique. A test pairs it with a doctored row that
 * must come back, so the guard is not one that every row passes by construction.
 */
export function reconciles<R extends { irn: string }>(
  identity: Identity<R>,
  districts: readonly R[],
): string[] {
  return districts
    .filter((row) => {
      const gap = Math.abs(identity.left(row) - identity.right(row));
      return !(gap <= identity.tolerance);
    })
    .map((row) => row.irn);
}
