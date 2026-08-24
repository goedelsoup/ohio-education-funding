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
 * A dollar amount, or an em dash where there is no value.
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

/** A fraction as a percentage, with the same minus the dollar formatters use. */
export function pct(v: number | null | undefined, decimals = 1): string {
  if (v == null || !Number.isFinite(v)) return "—";
  const n = v * 100;
  const text = Math.abs(n).toFixed(decimals);
  return (n < 0 && Number(text) !== 0 ? "−" : "") + text + "%";
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
  return `<span class="fig">${parts.join("")}</span>`;
}
