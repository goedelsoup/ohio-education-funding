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
 * # This module must never be imported by client code
 *
 * It imports `loadFeed`, and `feed.ts` opens `node:fs`. Importing it from `src/scripts/` puts the
 * build-time feed reader into a browser bundle: the comparison table stopped rendering entirely
 * the one time this was tried. Browser code reads `panel.series_years` directly — see the
 * four-line `yearOf` in `scripts/compare.ts` and the reason beside it.
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
  | "meal_program"
  | "casino";

/**
 * The year block for a series, or `null` where the feed does not carry one.
 *
 * Null rather than a thrown error or a placeholder: a block can legitimately be absent — the
 * Census fixture and the meal-program extract both are, in a feed built without them — and a
 * missing chip is the honest rendering of "this data is not here" rather than a chip reading
 * "unknown" beside figures that are.
 */
export function seriesYear(
  series: SeriesKey,
  // Structural rather than `Bundle`, so the browser-side comparison script can pass the trimmed
  // `Panel` it fetched. Both carry `series_years`; neither should have to carry the other's shape
  // to ask what year a figure is on.
  feed?: { series_years: SeriesYear[] },
): SeriesYear | null {
  const years = (feed ?? loadFeed().bundle).series_years;
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
 * A chip, its short label, and the reckoning that hangs off it — as HTML.
 *
 * # Why this is a button and not a span with a `title`
 *
 * It was a span with a `title`, and a `title` reaches a mouse and nothing else. It is not
 * announced by most screen readers, there is no way to bring it up from a keyboard, and on a
 * touch screen it does not exist at all — which is the width where a reader most needs telling
 * that this `2024` is a tax year and the `FY2025` two cards down is not. 12,449 chips in the
 * build carried the distinction where only a pointer could reach it, under a rule whose own
 * comment described an affordance it did not implement.
 *
 * So it takes the shape `.term` already uses on this site and for the same three reasons: a real
 * button, so it is focusable and tappable; the long form beside it in the markup, so nothing has
 * to be fetched or computed to show it; and no `title`, which would be announced on top of what
 * the button already says.
 *
 * # Why the long form is an `aria-label` and not an `aria-describedby`
 *
 * `.term` points at its definition by id, which works because a glossary slug appears once in a
 * sentence. A series does not: `yearChip("formula")` is written on eight cards of one district
 * page, so an id derived from the series would be duplicated eight times — and a counter would
 * make the markup depend on the order pages happen to be built in, which is a defect this
 * repository already has an open issue about.
 *
 * Naming the button directly needs no id at all. {@link yearTitle} opens with the label the chip
 * shows, so the visible text is a prefix of the accessible name and a reader who says "twenty
 * twenty-four" is still saying the name of the control. The panel beside it is `aria-hidden`,
 * because it is the same sentence again for eyes rather than a second fact.
 *
 * `<button>` and not `<abbr>`: the label is not an abbreviation of the title, it is a shorter
 * statement of the same fact, and `<abbr>` on a non-abbreviation is announced as one.
 *
 * Returns the empty string for a missing series rather than a placeholder, so a caller can
 * interpolate it unconditionally. See {@link seriesYear} for why absent is a real state.
 */
export function yearChip(series: SeriesKey): string {
  const year = seriesYear(series);
  if (!year) return "";
  return chip(
    { kind: year.kind, series: year.series },
    yearLabel(year),
    yearTitle(year),
  );
}

/**
 * The markup both chip forms share.
 *
 * `label` is what the chip shows; `says` is the whole of what it means, and it is written twice —
 * once as the button's name, once as the panel a sighted reader sees. That duplication is the
 * cost of the panel not needing an id; see {@link yearChip}.
 */
function chip(data: { kind: string; series: string }, label: string, says: string): string {
  return (
    `<span class="year-chip-wrap">` +
    `<button type="button" class="year-chip" data-kind="${data.kind}" ` +
    `data-series="${data.series}" aria-label="${escapeAttribute(says)}">` +
    `${escapeAttribute(label)}</button>` +
    `<span class="year-chip-def" aria-hidden="true">${escapeAttribute(says)}</span>` +
    `</span>`
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
  const says = `${yearTitle(first)} And ${qualifier}: ${yearTitle(second)}`;
  return chip(
    { kind: first.kind, series: first.series },
    `${first.label} · ${qualifier} ${second.label}`,
    says,
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

/**
 * A school year some number of years before the one given: `2024-25`, 1 → `2023-24`.
 *
 * The report card publishes three years of the Performance Index and the site labels all three.
 * Two of those labels were literals — `2022-23` and `2023-24` beside a derived `2024-25` — which
 * is the worst of both: the current year moves with the fixture and the two behind it do not, so
 * the column headings drift apart from each other rather than all going stale together.
 *
 * Returns the empty string for a label that is not `YYYY-YY`, rather than composing nonsense from
 * whatever it was handed.
 */
export function schoolYearBefore(label: string, years: number): string {
  const match = /^(\d{4})-(\d{2})$/.exec(label);
  if (!match) return "";
  const start = Number(match[1]) - years;
  return `${start}-${String((start + 1) % 100).padStart(2, "0")}`;
}

/**
 * The last year of a span label: `FY2024-FY2026` → `FY2026`, and `FY2027` → `FY2027`.
 *
 * Written for the sentences that name one end of a window rather than the window — *"a rolling
 * three-year average ending FY2026"* — which are the ones a chip cannot carry and which were
 * therefore still literals. The end of the enrolment span *is* the last year of that average, so
 * this is reading the fixture rather than asserting an offset from another year.
 *
 * The empty string for a school year, which is one year written with a hyphen: splitting `2024-25`
 * would yield `25`. A caller wanting the later half of a school year wants
 * {@link schoolYearBefore}'s arithmetic, not this.
 */
export function seriesSpanEnd(series: SeriesKey): string {
  const year = seriesYear(series);
  if (!year || year.kind === "school") return "";
  const parts = year.label.split("-");
  return parts[parts.length - 1] ?? "";
}

/**
 * The label for a series, or the empty string where the feed does not carry it.
 *
 * The common case at a call site that just needs the digits inside a sentence — `seriesYear(k)?.label ?? ""`
 * written once rather than at each of thirty places.
 */
export function yearOf(series: SeriesKey): string {
  return seriesYear(series)?.label ?? "";
}
