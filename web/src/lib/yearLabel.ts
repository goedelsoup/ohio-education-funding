/**
 * The one place a fiscal year is written from a number, for code that cannot import `year.ts`.
 *
 * `year.ts` reads the feed and so opens `node:fs`; `plot/spec.ts` is bundled into the browser by
 * the scenario runner. The fan chart's foot wrote `FY${year}` inline as a result, a fourth spelling
 * of the year in the one module every chart passes through (#610). This holds the form, and
 * `year.ts`'s {@link fiscalSpan} is built from it, so the two cannot drift.
 */

/** A fiscal year as the site writes it: `FY2027`. */
export function fiscalYear(year: number): string {
  return `FY${year}`;
}
