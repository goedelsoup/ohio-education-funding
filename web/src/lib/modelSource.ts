/**
 * The publication the model year is computed from, and the date it carries.
 *
 * A reader citing a figure needs to say when the data dates from (#595). The answer is the date on
 * the department's funding calculator, and that date is stated in exactly one place: the catalog
 * entry describing the calculator, which says it is "dated 16 December 2025". This reads it from
 * there rather than repeating it, for the reason `year.ts` reads every year label off the data — a
 * date typed into the footer would stay put the day the calculator is re-fetched, and nothing would
 * notice.
 *
 * The entry is found from the feed's own model year, so a bundle built for another year looks for
 * that year's calculator and fails the build if the corpus has not catalogued it.
 */

import { loadCorpus, type Source } from "./corpus.ts";
import { wikiSource } from "./routes.ts";

export interface ModelSource {
  /** The catalog entry's slug, which is also its page under `/wiki/source/`. */
  slug: string;
  href: string;
  /** The date as the entry writes it: `16 December 2025`. */
  date: string;
  /** The same date as `YYYY-MM-DD`, for `<time datetime>`. */
  iso: string;
}

const MONTHS = [
  "January", "February", "March", "April", "May", "June",
  "July", "August", "September", "October", "November", "December",
];

/** `dated 16 December 2025`, the form every dated catalog entry uses. */
const DATED = new RegExp(`\\bdated (\\d{1,2}) (${MONTHS.join("|")}) (\\d{4})\\b`, "g");

/** The catalog slug of the department's funding calculator for a fiscal year: `dew-fy27-…`. */
const calculatorSlug = (fiscalYear: number): string =>
  `dew-fy${String(fiscalYear % 100).padStart(2, "0")}-funding-calculator`;

/**
 * The one date an entry gives itself.
 *
 * Exactly one, and a throw otherwise: an entry that came to mention a second dated edition would
 * make "the first match" a guess about which one the model reads.
 */
export function datedOf(source: Pick<Source, "slug" | "body">): Pick<ModelSource, "date" | "iso"> {
  const matches = [...source.body.matchAll(DATED)];
  if (matches.length !== 1) {
    throw new Error(
      `catalog/${source.slug}.md must say "dated <day> <Month> <year>" exactly once to date the model; found ${matches.length}`,
    );
  }
  const [, day, month, year] = matches[0]!;
  const mm = String(MONTHS.indexOf(month!) + 1).padStart(2, "0");
  return { date: `${Number(day)} ${month} ${year}`, iso: `${year}-${mm}-${day!.padStart(2, "0")}` };
}

/** The calculator the model year is read from. Throws if the corpus does not catalogue it. */
export function modelSource(fiscalYear: number): ModelSource {
  const slug = calculatorSlug(fiscalYear);
  const source = loadCorpus().sources.find((entry) => entry.slug === slug);
  if (!source) {
    throw new Error(`The FY${fiscalYear} model has no catalog entry: expected .yidam/catalog/${slug}.md`);
  }
  return { slug, href: wikiSource(slug), ...datedOf(source) };
}
