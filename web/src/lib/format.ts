/** Number and text formatting. Small enough to test exhaustively, and it has been wrong. */

/**
 * The number formatters, built once each.
 *
 * `toLocaleString(locale, options)` constructs a fresh `Intl.NumberFormat` for every call, and
 * this module is the hottest code in the build: a district page formats its own figures, and then
 * `strip` formats the same measure for all 609 districts twice over to decide which of them the
 * distribution is going to draw. About a million calls per build, measured at 16.0s against 0.39s
 * for the identical work through a cached formatter — a fortieth of the cost for the same string.
 *
 * Keyed by decimal places, which is the only thing that varies. Three entries live here in
 * practice: 0, 1 and 2.
 *
 * The output is the formatter's, not a reimplementation of it, so this is byte-for-byte what
 * `toLocaleString` produced — including the cases that make this file worth testing exhaustively:
 * a magnitude that rounds to nothing, a non-finite value, a negative zero.
 */
const FORMATTERS = new Map<number, Intl.NumberFormat>();

function decimal(decimals: number): Intl.NumberFormat {
  let formatter = FORMATTERS.get(decimals);
  if (!formatter) {
    formatter = new Intl.NumberFormat("en-US", {
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals,
    });
    FORMATTERS.set(decimals, formatter);
  }
  return formatter;
}

/**
 * A dollar amount, or an em dash where there is no value. Zero is `$0`, never the dash; a table
 * that would otherwise print a bare dash for a missing figure uses {@link reported}.
 *
 * The minus is U+2212 and sits outside the dollar sign, which is what `signedMoney` and
 * `millions` in this file already do. This one fell through to `toLocaleString`, so it printed
 * `$-47` while its neighbours printed `−$1,318`, and the two met in one sentence on /statewide
 * and in one chart tooltip on a district's finances. A reader is entitled to assume two figures
 * formatted differently were computed differently.
 *
 * A magnitude that rounds away loses the sign with it: `money(-0.4)` is `$0`, not `−$0`. Same
 * rule as `signedMoney`, and for the same reason — a sign on a quantity that rounded to nothing
 * is a claim the figure does not support.
 */
export function money(v: number | null | undefined, decimals = 0): string {
  if (v == null || !Number.isFinite(v)) return "—";
  const text = decimal(decimals).format(Math.abs(v));
  return (v < 0 && Number(text.replace(/,/g, "")) !== 0 ? "−" : "") + "$" + text;
}

/**
 * What a cell says when the source has no figure for it.
 *
 * Not "—", because "—" was also standing for zero. On `/districts` College Corner Local's row read
 * "—" under the guarantee, meaning $0, and "—" two cells along under valuation, meaning the source
 * had no value — and the district's own Dashboard wrote the second as "not reported" (#597).
 */
export const NOT_REPORTED = "not reported";

/**
 * `format(v)`, or {@link NOT_REPORTED} where there is no value.
 *
 * Zero is a value. The only test is the null branch every formatter here already has, so a zero
 * reaches `format` and is written `$0` or `0.0%`; a cell that means "nothing was paid" calls
 * `money` and gets `$0` rather than writing a dash of its own.
 */
export function reported(v: number | null | undefined, format: (v: number) => string): string {
  return v == null || !Number.isFinite(v) ? NOT_REPORTED : format(v);
}

/** A signed dollar amount, so a zero change reads as zero rather than as a gain. */
export function signedMoney(v: number | null | undefined, decimals = 0): string {
  if (v == null || !Number.isFinite(v)) return "—";
  if (Math.abs(v) < 0.5 * Math.pow(10, -decimals)) return money(0, decimals);
  return (v > 0 ? "+" : "−") + money(Math.abs(v), decimals);
}

/**
 * A large dollar amount at a readable scale.
 *
 * Switches to billions past a thousand million, because "$7281.2M" is a number a reader has to
 * count the digits of and "$7.28B" is one they can hold.
 */
export function millions(v: number | null | undefined): string {
  if (v == null || !Number.isFinite(v)) return "—";
  const sign = v < 0 ? "−" : v > 0 ? "+" : "";
  const magnitude = Math.abs(v);
  return magnitude >= 1_000_000_000
    ? `${sign}$${(magnitude / 1_000_000_000).toFixed(2)}B`
    : `${sign}$${(magnitude / 1_000_000).toFixed(1)}M`;
}

/**
 * A dollar amount at three significant figures, for the numbers a chart prints: `$83.5M`, `$10.5B`.
 *
 * One chart form printed money three ways. District bars wrote `$83,470,878`, `/statewide` wrote
 * `$10.48B`, and the wiki's scatter ends wrote `$18,184,179,141`. A label or an axis end exists to
 * be read at a glance, and nine digits cannot be. The full figure is still on the page, in the
 * tooltip and in the table. Those keep {@link money}.
 *
 * Under ten thousand it is {@link money}'s whole dollars. That is the per-pupil scale, where the
 * prose beside a chart writes `$4,229` and a label reading `$4.23K` would round the same figure a
 * second way.
 *
 * Same sign rule as `money`: the minus is U+2212 and sits outside the dollar sign.
 */
export function compactMoney(v: number | null | undefined): string {
  if (v == null || !Number.isFinite(v)) return "—";
  if (Math.abs(v) < 10_000) return money(v);
  return (v < 0 ? "−" : "") + "$" + COMPACT.format(Math.abs(v));
}

const COMPACT = new Intl.NumberFormat("en-US", { notation: "compact", maximumSignificantDigits: 3 });

/**
 * {@link compactMoney} for the labels of one axis, which are written in one style (#662).
 *
 * `compactMoney` keeps a figure under ten thousand whole so a label agrees with the prose beside
 * it, and that is right for a direct label. A tick set is read as a scale, and the rule split it:
 * `/statewide`'s wealth-offset axis read "$1,000" beside "$20K" and `/districts` "$5,000" beside
 * "$25K". So once any tick reaches ten thousand the whole set is compact.
 */
export function compactMoneyTicks(values: number[]): string[] {
  if (!values.some((v) => Number.isFinite(v) && Math.abs(v) >= 10_000)) return values.map(compactMoney);
  return values.map((v) =>
    !Number.isFinite(v) ? "—" : (v < 0 ? "−" : "") + "$" + COMPACT.format(Math.abs(v)),
  );
}

/**
 * A plain number to three significant figures, grouped, with a true minus.
 *
 * For an axis end on a ratio scale. `corpusSeries.ts` writes ratios to four places, which is the
 * right precision for a tooltip quoting a pinned coefficient. On an axis it printed `+2.0057` and
 * `62,603.6`, which are positions on a scale, not figures anybody will quote.
 */
export function sig(v: number | null | undefined): string {
  if (v == null || !Number.isFinite(v)) return "—";
  const text = SIG.format(Math.abs(v));
  return (v < 0 && !roundsToZero(text) ? "−" : "") + text;
}

const SIG = new Intl.NumberFormat("en-US", { maximumSignificantDigits: 3 });

/**
 * A fraction as a percentage, with the same minus the dollar formatters use.
 *
 * A positive share that rounds to nothing is written `<1%` (or `<0.1%` at one place), not `0%`. On
 * `/district/043802/taxes` the Agricultural and Railroad bars were drawn with width and labelled
 * `0%`, so the label and the bar disagreed. The bar was right. A negative share that rounds to
 * nothing keeps `money`'s rule and loses its sign.
 */
export function pct(v: number | null | undefined, decimals = 1): string {
  if (v == null || !Number.isFinite(v)) return "—";
  const n = v * 100;
  const text = Math.abs(n).toFixed(decimals);
  if (n > 0 && Number(text) === 0) return `<${(1 / 10 ** decimals).toFixed(decimals)}%`;
  return (n < 0 && Number(text) !== 0 ? "−" : "") + text + "%";
}

/**
 * A log error, with the sign on the front and a true minus rather than a hyphen.
 *
 * Written as a log error rather than converted to a percentage, which is
 * `.yidam/decisions/the-bias-published-beside-the-point.yml`'s call: these are means and sums of
 * `ln(point / actual)`, a percentage would be right only to first order, and the first order is
 * where all the small ones live.
 *
 * Four decimal places, because the smallest of the published figures is +0.00057 and `pct` would
 * render it as 0.1% — a figure a reader cannot tell from zero. It lives here rather than in
 * `bias.ts`, which is where it was written, because the district fan states one of these too and a
 * per-district card has no business importing /method's card to format a number.
 */
export function logError(v: number): string {
  return `${v < 0 ? "−" : "+"}${Math.abs(v).toFixed(4)}`;
}

/**
 * A plain number to a fixed number of places: grouped, and with the minus the formatters above use.
 *
 * This is what `toFixed` was being asked to do in the renderers, and `toFixed` does neither half
 * of it. It printed `-0.03` in the growth rows of 326 outcome pages, beside cells this file had
 * formatted as `−$1,318`, and it printed Columbus's Category 2 pupils as `8376.39` in the cell
 * beside `$1,115,773`. Millage, index values, effect sizes and FTE all come through here.
 *
 * Same sign rule as `money`: a magnitude that rounds away loses its sign, so `fixed(-0.001, 2)` is
 * `0.00`.
 */
export function fixed(v: number | null | undefined, places: number): string {
  if (v == null || !Number.isFinite(v)) return "—";
  const text = decimal(places).format(Math.abs(v));
  return (v < 0 && !roundsToZero(text) ? "−" : "") + text;
}

/**
 * `fixed` with its sign always written, for a difference or a coefficient.
 *
 * A zero after rounding carries no sign, which is `signedMoney`'s rule: `+0.0000` would claim a
 * direction the figure does not have.
 */
export function signed(v: number | null | undefined, places: number): string {
  if (v == null || !Number.isFinite(v)) return "—";
  const text = decimal(places).format(Math.abs(v));
  if (roundsToZero(text)) return text;
  return (v > 0 ? "+" : "−") + text;
}

function roundsToZero(text: string): boolean {
  return Number(text.replace(/,/g, "")) === 0;
}

/** A count with thousands separators. */
export function count(v: number): string {
  return WHOLE.format(v);
}

/** `count`'s formatter. Separate from {@link decimal}: it sets no minimum. */
const WHOLE = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });

/**
 * 1st, 2nd, 3rd, 4th … 11th, 12th, 13th … 21st.
 *
 * The teens are the whole difficulty, and getting them wrong put "2th percentile" on this page
 * once. It was caught by looking at the rendered output, not by any test — hence the tests.
 */
export function ordinal(n: number): string {
  const teens = n % 100;
  if (teens >= 11 && teens <= 13) return `${n}th`;
  const suffix = { 1: "st", 2: "nd", 3: "rd" }[n % 10] ?? "th";
  return `${n}${suffix}`;
}

/** Share of `values` strictly below `v`. */
export function percentileOf(sorted: number[], v: number): number {
  let below = 0;
  for (const x of sorted) {
    if (x < v) below++;
    else break;
  }
  return sorted.length === 0 ? 0 : below / sorted.length;
}

/** Escape text for interpolation into HTML. District names are data, not markup. */
export function escapeHtml(s: string): string {
  return s.replace(
    /[&<>"']/g,
    (c) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]!,
  );
}

/**
 * A figure as a compound: a value, the year it measures, and the basis it is in.
 *
 * # Why the annotation is a child and not a class
 *
 * A dollar amount here is meaningless without two more facts. Across FY2020-FY2025 prices rose
 * 25.1%; statewide cash balances end 9% ABOVE where they started in nominal dollars and 13% BELOW
 * in real ones, and those two sentences support opposite arguments. An unlabelled number is not an
 * incomplete figure, it is a wrong one.
 *
 * So the year is a required argument and the markup is three elements. Omitting the year is then a
 * type error rather than a forgotten adjective, which is the difference between a rule and a habit.
 *
 * # Where this is used, and where it is deliberately not
 *
 * The site renders roughly eight thousand money strings per sampled hundred pages, and **96% of
 * them already carry their year**: 52% sit in a card whose heading has a year chip, and 44% in a
 * table cell, where the design system puts the annotation on the column head rather than repeating
 * it down 609 rows. Wrapping those would be redundant work that made the markup heavier and the
 * page no clearer.
 *
 * The 4% that did not were tiles — a route's headline figures, outside any card, carrying no year
 * anywhere in the key or the note. Eleven of them across three modules. That is what this is for.
 */
export function fig(
  value: string,
  year: string,
  basis: "nominal" | "real" | null = null,
): string {
  const parts = [`<span class="fig-value">${escapeHtml(value)}</span>`];
  parts.push(`<span class="fig-year">${escapeHtml(year)}</span>`);
  if (basis !== null) {
    parts.push(`<span class="fig-basis" data-basis="${basis}">${basis}</span>`);
  }
  /*
   * Joined by a real space, not by the stylesheet's margin.
   *
   * Two adjacent inline spans concatenate with nothing between them in the accessible name and in
   * `textContent`: the tiles were announcing "$251,661TY2024", and the `/districts` column heads
   * would have named their sort buttons "State aid / pupilFY2027". The margin is a picture of a
   * gap and only a sighted reader gets it.
   *
   * The gap is therefore in the markup and `.fig-year` carries no `margin-left` — a word space at
   * this size is within a hundredth of an em of the .28 that rule used to set, so nothing moves.
   */
  return `<span class="fig">${parts.join(" ")}</span>`;
}
