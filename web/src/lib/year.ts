/**
 * What year a figure is measured in, and how to say so.
 *
 * # Why this is a module and not a template literal
 *
 * Because the web layer carried about 190 four-digit year literals across 25 files, and one of
 * them was wrong in a way nothing could catch. `crates/bundle` wrote a provenance paragraph naming
 * every year in the feed; it said *"millage is TY2023"* while all 609 districts carried
 * `tax_year: 2024`. A sentence and a column cannot disagree in a way a compiler notices, and the
 * sentence is the half a reader sees.
 *
 * The feed now carries `series_years` — derived from the blocks themselves rather than typed — and
 * this reads it. Nothing on a page composes a year string.
 *
 * # Ohio reckons three ways, and they are eleven months apart
 *
 * | Reckoning | Span | Written |
 * |---|---|---|
 * | fiscal | July to June, named for the June | `FY2027` |
 * | tax | a calendar year of valuation and levy | `2024 tax year` |
 * | school | September to June | `2024-25` |
 *
 * A 2024 tax year funds an FY2025 budget. Rendering both as a bare `2024` invites a reader to set
 * them side by side as though they were the same period, which is the specific confusion the
 * district pages create: five reckonings on one page under a heading that names one of them.
 *
 * # Why the chip says the kind and not only the digits
 *
 * `FY2024` and `2024` differ by a prefix, which reads as a typography choice rather than a claim.
 * A tax year is therefore written **`2024 tax year`** in full wherever there is room, and the
 * `title` attribute carries the long form wherever there is not. Being slightly verbose is the
 * cost of the distinction being visible at all.
 */

import { loadFeed } from "./feed.ts";
import type { Bundle, SeriesYear } from "./types.ts";

/** The series keys this site asks for, so a typo is a type error rather than a missing chip. */
export type SeriesKey =
  | "formula"
  | "enrollment"
  | "outcome.performance"
  | "outcome.spending"
  | "profile"
  | "millage"
  | "property_tax"
  | "finances"
  | "history"
  | "national"
  | "appropriations"
  | "meal_program";

/**
 * The year block for a series, or `null` where the feed does not carry one.
 *
 * Null rather than a thrown error or a placeholder: a block can legitimately be absent — the
 * Census fixture and the meal-program extract both are, in a feed built without them — and a
 * missing chip is the honest rendering of "this data is not here" rather than a chip reading
 * "unknown" beside figures that are.
 */
export function seriesYear(series: SeriesKey, bundle?: Bundle): SeriesYear | null {
  const years = (bundle ?? loadFeed().bundle).series_years;
  return years.find((entry) => entry.series === series) ?? null;
}

/**
 * The short form, for a chip: `FY2027`, `2024`, `2024-25`.
 *
 * The tax year loses its words here and regains them in {@link yearTitle}, which is what the chip
 * hangs on its `title`. A chip is a few characters wide and a card header has one line.
 */
export function yearLabel(year: SeriesYear): string {
  return year.label;
}

/**
 * The long form, for a `title` and for a screen reader: what the reckoning is and who published it.
 *
 * This is the string that does the actual disambiguating. The chip is the affordance that says
 * there is something to disambiguate.
 */
export function yearTitle(year: SeriesYear): string {
  const reckoning = {
    fiscal: `fiscal year${year.label.includes("-") ? "s" : ""}, July to June`,
    tax: `tax year${year.label.includes("-") ? "s" : ""}, a calendar year of valuation and levy`,
    school: "school year, September to June",
  }[year.kind];
  return `${year.label} — ${reckoning}. Source: ${year.source}.`;
}

/**
 * The chip itself, as HTML, for the card renderers in `src/lib/*.ts`.
 *
 * Returns the empty string for a missing series rather than a placeholder, so a caller can
 * interpolate it unconditionally. See {@link seriesYear} for why absent is a real state.
 *
 * `<span>` and not `<abbr>`: the label is not an abbreviation of the title, it is a shorter
 * statement of the same fact, and `<abbr>` on a non-abbreviation is announced as one.
 */
export function yearChip(series: SeriesKey): string {
  const year = seriesYear(series);
  if (!year) return "";
  return (
    `<span class="year-chip" data-kind="${year.kind}" data-series="${year.series}" ` +
    `title="${escapeAttribute(yearTitle(year))}">${escapeAttribute(yearLabel(year))}</span>`
  );
}

/**
 * A chip naming two reckonings, for a card that genuinely mixes them.
 *
 * The scenario runner reads an FY2027 formula against FY2022 cost inputs; the report card
 * publishes 2024-25 attainment beside FY2025 spending. Picking one and printing it is the failure
 * this whole feature exists to stop, so a card that mixes says so: `FY2027 · costs FY2022`.
 *
 * `qualifier` is the word that distinguishes the second from the first — "costs", "spending",
 * "prior" — because "FY2027 · FY2022" tells a reader there are two years and not which is which.
 */
export function yearChipPair(
  primary: SeriesKey,
  secondary: SeriesKey,
  qualifier: string,
): string {
  const first = seriesYear(primary);
  const second = seriesYear(secondary);
  if (!first) return "";
  if (!second) return yearChip(primary);
  const title = `${yearTitle(first)} And ${qualifier}: ${yearTitle(second)}`;
  return (
    `<span class="year-chip" data-kind="${first.kind}" data-series="${first.series}" ` +
    `title="${escapeAttribute(title)}">${escapeAttribute(first.label)} · ${escapeAttribute(qualifier)} ` +
    `${escapeAttribute(second.label)}</span>`
  );
}

/**
 * Escape for an attribute value.
 *
 * Local rather than `format.ts`'s `escapeHtml` because that one is written for text content and
 * leaves the single quote alone, which is fine between tags and not fine inside an attribute this
 * module writes with double quotes — a `source` containing one would still be safe, but the rule
 * a reader has to check should not depend on which quote the template happened to use. Every
 * source string here is repository-authored; this is the boundary being real rather than assumed,
 * which is a distinction `prose.ts` had to learn once already.
 */
function escapeAttribute(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}
