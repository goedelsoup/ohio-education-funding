/**
 * The long view: Ohio's school revenue as the Census Bureau measured it, FY2009 to FY2024.
 *
 * # Why this is a separate page and not a card on the front one
 *
 * Every other figure on this site is one fiscal year of the Fair School Funding Plan, computed
 * for the 609 traditional districts in the department's own calculator. This is a different
 * measurement of a different population: the six hundred or so comparable Ohio school systems
 * the federal government surveys a year — community schools, joint vocational districts and
 * educational service centres excluded — on the Bureau's enrollment count rather than ADM, with
 * revenue classified the Bureau's way.
 *
 * The two do not reconcile, and putting them on one page would invite a reader to subtract one
 * from the other. So the history lives on its own route, says on its face what it is measuring,
 * and links to the catalog record that says what the survey can be trusted for.
 *
 * # What the series actually shows, and what it does not
 *
 * Two findings, both already computed and tested in `crates/dispersion`:
 *
 * 1. **The state share fell and the local share rose.** State revenue went from roughly 46% of
 *    school revenue to roughly 34% across the panel. That is the H.B. 920 story arriving from
 *    outside Ohio's own accounting.
 * 2. **The rate held while the gap grew.** State aid closed a stable share of the distance
 *    between the poorest and richest quartiles of districts — between 40% and 49% in every year
 *    from FY2012 to FY2024 — while that distance itself grew by 84%. A page showing only the
 *    percentage would report thirteen years of stability, which is why the residual is drawn in
 *    dollars: it is what a district experiences and it grows with the gap.
 *
 * FY2023 and FY2024 read as a break in this series until the measure took an enrolment floor.
 * They were a five-pupil island district entering the survey's comparable population; see
 * `dispersion::ohio_panel::MIN_ENROLMENT` and `doctrine/equity`'s revisions.
 *
 * The prose beside the chart is computed rather than written, so it says "held" or "moved"
 * according to what the panel does.
 *
 * Both ends flatter the federal column: FY2009-FY2012 carry the ARRA tail and FY2020 onward
 * carries ESSER, so the federal contribution is well above its ordinary size at each end. The card
 * says so, with the peaks found in the series rather than written (#705), because a reader taking the federal line at face value would draw the wrong
 * conclusion from exactly the years that are easiest to notice.
 *
 * # Real dollars are not optional here
 *
 * CPI-U rose 45% across this panel. A gap growing from $5,364 to $10,527 per pupil in nominal
 * terms is a different claim from the same gap in constant dollars, and only one of them is the
 * finding. The dollar series is therefore shown on both bases, through the same {@link Basis}
 * toggle every other financial view uses, and a year the index cannot cover is dropped rather
 * than shown un-deflated in a column labelled real.
 */

import type { SeriesPoint } from "./chart.ts";
import { escapeHtml, money, pct } from "./format.ts";
import { renderRegimeKey, type RegimeEvent } from "./legislation.ts";
import { seriesSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import { convert, type Basis } from "./real.ts";
import type { Bundle, Deflator, HistoryYear } from "./types.ts";
import { yearChip } from "./year.ts";

/**
 * The first year of the band the "held or moved" sentence is computed over.
 *
 * Not the panel's own first year. FY2009-FY2011 are the ARRA years, where federal money displaced
 * state gap-closing and the state share of the gap sits at its lowest in the series — a verdict
 * that included them would be reading the stimulus arriving and leaving rather than the formula.
 * The corpus states the band from here for the same reason; see `doctrine/equity`.
 */
const BAND_FROM = 2012;

/**
 * The two federal relief windows, inclusive: ARRA's tail, and ESSER's.
 *
 * The years between them are the federal share's ordinary size. The windows are the programmes'
 * reach into the panel, not where the share happens to peak — the peak is found in the series by
 * {@link federalPeak}, because the note once named the panel's last year as the pandemic peak when
 * FY2022 was (#705).
 */
const STIMULUS: readonly [number, number] = [2009, 2012];
const RELIEF: readonly [number, number] = [2020, Infinity];

/** The year with the largest federal share within `[from, to]`, or null if none falls there. */
export function federalPeak(
  history: HistoryYear[],
  [from, to]: readonly [number, number] = [-Infinity, Infinity],
): HistoryYear | null {
  return history
    .filter((y) => y.fiscal_year >= from && y.fiscal_year <= to)
    .reduce<HistoryYear | null>((m, y) => (m == null || y.federal_share > m.federal_share ? y : m), null);
}

/** The federal note: its ordinary range, both relief peaks, and where the panel leaves it. */
export function federalNote(history: HistoryYear[]): string {
  const ordinary = history
    .filter((y) => y.fiscal_year > STIMULUS[1] && y.fiscal_year < RELIEF[0])
    .map((y) => y.federal_share * 100);
  const stimulus = federalPeak(history, STIMULUS);
  const relief = federalPeak(history, RELIEF);
  const last = lastOf(history);
  const peaks = [
    stimulus && `to ${pct(stimulus.federal_share, 1)} in FY${stimulus.fiscal_year} on stimulus money`,
    relief && `to ${pct(relief.federal_share, 1)} in FY${relief.fiscal_year} on pandemic relief`,
  ].filter(Boolean);
  const range =
    ordinary.length > 0
      ? `ran ${Math.round(Math.min(...ordinary))}–${Math.round(Math.max(...ordinary))}% in ordinary years, `
      : "";
  const still =
    relief != null && relief !== last ? `, and was still ${pct(last.federal_share, 1)} in FY${last.fiscal_year}` : "";
  return peaks.length === 0 ? "" : `The federal share ${range}rose ${peaks.join(" and ")}${still}.`;
}
import { anchor } from "./section.ts";
import { firstOf, lastOf } from "./ends.ts";

/** What neither level of government closes — the part a district actually experiences. */
export function residual(year: HistoryYear): number {
  return year.gap_per_pupil - year.state_closes_per_pupil - year.federal_closes_per_pupil;
}

/** State aid's share of the gap it is measured against. */
function stateShareOfGap(year: HistoryYear): number {
  return year.gap_per_pupil > 0 ? year.state_closes_per_pupil / year.gap_per_pupil : 0;
}

/**
 * Years the panel skips, as fiscal years.
 *
 * FY2014 is missing from the Bureau's archive under every naming the other years use. Naming it
 * is the difference between a chart with a hole in it and a chart a reader assumes is complete.
 */
export function gaps(history: HistoryYear[]): number[] {
  if (history.length === 0) return [];
  const present = new Set(history.map((y) => y.fiscal_year));
  const out: number[] = [];
  for (let y = firstOf(history).fiscal_year; y <= lastOf(history).fiscal_year; y++) {
    if (!present.has(y)) out.push(y);
  }
  return out;
}

/**
 * The series, with a `null` at every year the panel skips.
 *
 * The gap has to reach the chart as a year with no value rather than as an absent year, or the
 * line is drawn straight across it and asserts a measurement nobody made.
 */
export function withGaps(
  history: HistoryYear[],
  a: (y: HistoryYear) => number | null,
  b: (y: HistoryYear) => number | null,
): SeriesPoint[] {
  const byYear = new Map(history.map((y) => [y.fiscal_year, y]));
  const missing = gaps(history);
  const years = [...history.map((y) => y.fiscal_year), ...missing].sort((x, y) => x - y);
  return years.map((year) => {
    const y = byYear.get(year);
    return { at: year, a: y ? a(y) : null, b: y ? b(y) : null };
  });
}

/**
 * Restate a year's dollar figures into `base` dollars, or drop it.
 *
 * Dropping is the right answer rather than falling back to nominal: a real-terms series with one
 * un-deflated point in it is wrong in a way nothing on the page would show.
 */
export function inBase(
  deflator: Deflator | null,
  year: HistoryYear,
  base: number,
  basis: Basis,
): HistoryYear | null {
  if (basis === "nominal") return year;
  if (!deflator) return null;
  const at = (value: number) => convert(deflator, value, year.fiscal_year, base);
  const gap = at(year.gap_per_pupil);
  const state = at(year.state_closes_per_pupil);
  const federal = at(year.federal_closes_per_pupil);
  const poorest = at(year.poorest_local_per_pupil);
  const richest = at(year.richest_local_per_pupil);
  if (gap == null || state == null || federal == null || poorest == null || richest == null) {
    return null;
  }
  return {
    ...year,
    gap_per_pupil: gap,
    state_closes_per_pupil: state,
    federal_closes_per_pupil: federal,
    poorest_local_per_pupil: poorest,
    richest_local_per_pupil: richest,
  };
}

/** Where the money came from, year by year. */
export function renderRevenueMix(history: HistoryYear[], events: RegimeEvent[] = []): string {
  if (history.length < 2) return "";

  const first = firstOf(history);
  const last = lastOf(history);
  const points = withGaps(
    history,
    (y) => y.local_share * 100,
    (y) => y.state_share * 100,
  );

  const chart = renderToString(
    (w) =>
      seriesSpec(
        points,
        { a: "local", b: "state" },
        // To the place the prose beside it uses: "state 35%" sat next to a sentence saying 34.5%.
        (v) => `${v.toFixed(1)}%`,
        (p) =>
          p.a == null
            ? `FY${p.at}: not published`
            : `FY${p.at}: ${p.a.toFixed(1)}% local, ${(p.b ?? 0).toFixed(1)}% state`,
        // Census shares, and state revenue there is not foundation aid: neither line takes the
        // formula's hue or the guarantee's (#706).
        { width: w, tick: (year) => `FY${year}`, hues: { a: "plain", b: "plain-strong" }, events },
      ),
  { label: `Local and state shares of total school revenue across the Census Bureau's Ohio school systems, percent, FY${first.fiscal_year} to FY${last.fiscal_year}`, description: "The axis is truncated to the range of the two shares rather than starting at zero" },
  );

  const missing = gaps(history);

  return `
    <div class="card" id="revenue-mix" data-part="revenue-mix">
      <h2>${anchor("revenue-mix")}Where the money came from${yearChip("history")}</h2>
      <p class="note"><strong>The state share ${last.state_share < first.state_share ? "fell" : "rose"} from
        ${pct(first.state_share, 1)} in FY${first.fiscal_year} to ${pct(last.state_share, 1)} in
        FY${last.fiscal_year}, and the local share ${last.local_share < first.local_share ? "fell" : "rose"} from
        ${pct(first.local_share, 1)} to ${pct(last.local_share, 1)}, with federal money covering the
        rest.</strong>
        Local, state and federal revenue as shares of the total, across every comparable Ohio
        school system the Census Bureau surveyed.</p>

      <div class="chartwrap" data-chart="revenue-mix">${chart}</div>
      ${renderRegimeKey(events, first.fiscal_year, last.fiscal_year)}

      <p class="note">The federal line is left off the chart and kept in the table below, because
        it is the one series whose ends cannot be read at face value. ${federalNote(history)}
        ${missing.length > 0 ? `FY${missing.join(", FY")} ${missing.length === 1 ? "is" : "are"} absent from the Bureau's archive, and the line breaks rather than bridging it.` : ""}</p>

      <div class="scroll"><table>
        <thead><tr><th>Year</th><th class="tnum">Systems</th><th class="tnum">Local</th>
          <th class="tnum">State</th><th class="tnum">Federal</th></tr></thead>
        <tbody>${history
          .map(
            (y) => `<tr>
              <th>FY${y.fiscal_year}</th>
              <td class="tnum n">${y.districts}</td>
              <td class="tnum">${pct(y.local_share, 1)}</td>
              <td class="tnum">${pct(y.state_share, 1)}</td>
              <td class="tnum n">${pct(y.federal_share, 1)}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>
    </div>`;
}

/** The gap between the poorest and richest quartiles, and what closes it. */
export function renderEqualization(
  history: HistoryYear[],
  deflator: Deflator | null,
  base: number | null,
  basis: Basis,
  events: RegimeEvent[] = [],
): string {
  if (history.length < 2) return "";

  const restated =
    basis === "real" && base != null
      ? history.map((y) => inBase(deflator, y, base, basis)).filter((y): y is HistoryYear => y != null)
      : history;
  if (restated.length < 2) {
    return `<div class="card" data-part="equity-gap"><h2>${anchor("equity-gap")}Whom it reached</h2>
      <p class="note">The price index does not cover this panel, so these figures cannot be shown
        in constant dollars.</p></div>`;
  }

  const first = firstOf(restated);
  const last = lastOf(restated);
  const points = withGaps(
    restated,
    (y) => y.gap_per_pupil,
    (y) => residual(y),
  );

  /*
   * Both bases' values, so the nominal and the constant-dollar drawings share one frame (#706). On
   * a domain of its own each drawing rescaled to fit its line, and the switch showed two lines of
   * nearly the same shape.
   */
  const both = [
    history,
    base == null
      ? []
      : history.map((y) => inBase(deflator, y, base, "real")).filter((y): y is HistoryYear => y != null),
  ].flatMap((years) => years.flatMap((y) => [y.gap_per_pupil, residual(y)]));

  const chart = renderToString(
    (w) =>
      seriesSpec(
        points,
        { a: "gap", b: "unclosed" },
        (v) => money(v),
        (p) =>
          p.a == null
            ? `FY${p.at}: not published`
            : `FY${p.at}: ${money(p.a)} gap, ${money(p.b ?? 0)} of it closed by nobody`,
        {
          width: w,
          tick: (year) => `FY${year}`,
          // A local-revenue gap and what no aid closes: neither line is formula aid (#706).
          hues: { a: "plain", b: "plain-strong" },
          span: both,
          unit: basis === "real" && base != null ? `in constant FY${base} dollars` : "in the dollars of each year",
          events,
        },
      ),
  { label: `Gap in local revenue per pupil between the poorest and richest quarter of districts, and the part neither state nor federal aid closes, FY${first.fiscal_year} to FY${last.fiscal_year}, ${basis === "real" && base != null ? `in FY${base} dollars` : "in the dollars of each year"}`, description: "The axis is truncated to the range of the two series rather than starting at zero, and the line breaks at any year the panel skips" },
  );

  /*
   * "Held" is the band, not the endpoints.
   *
   * This compared the panel's first year to its last and called a move anything over six points.
   * Two things were wrong with it. The panel opens in FY2009-FY2011, where federal money displaced
   * state gap-closing and the state share is at its lowest — so an endpoint comparison measures
   * the stimulus arriving and leaving rather than the formula. And it was a binary sitting half a
   * point from its own threshold, which would have flipped the sentence on a fixture revision that
   * changed nothing about Ohio.
   *
   * The finding is a band across FY2012 onward. This is that band's whole spread, against a bar
   * wide enough to admit the year-to-year variation the series has always had and narrow enough
   * to fail if the rate ever starts trending.
   */
  const banded = history.filter((y) => y.fiscal_year >= BAND_FROM);
  const band = banded.map(stateShareOfGap);
  const bandFirst = Math.min(...banded.map((y) => y.fiscal_year));
  const rateHeld = Math.max(...band) - Math.min(...band) < 0.1 ? "held" : "moved";

  return `
    <div class="card" data-part="equity-gap">
      <h2>${anchor("equity-gap")}Whom it reached${yearChip("history")}</h2>
      <p class="note"><strong>The gap between the poorest and richest quarter of districts
        ${last.gap_per_pupil < first.gap_per_pupil ? "narrowed" : "grew"} from ${money(first.gap_per_pupil)}
        per pupil in FY${first.fiscal_year} to ${money(last.gap_per_pupil)} in
        FY${last.fiscal_year}.</strong> Districts sorted by local revenue per pupil and cut into
        quarters. The gap is
        the distance between the poorest quarter and the richest; the second line is what neither
        state nor federal aid closes, which is the part a district actually experiences.
        ${basis === "real" ? `In FY${base} dollars.` : "In the dollars of each year."}</p>

      <div class="chartwrap" data-chart="equity-gap">${chart}</div>
      ${renderRegimeKey(events, first.fiscal_year, last.fiscal_year)}

      <p class="note">State aid's share of the gap ${rateHeld} — ${pct(Math.min(...band), 0)} to
        ${pct(Math.max(...band), 0)} in every year from FY${bandFirst} — while the gap itself grew
        from
        ${money(first.gap_per_pupil)} to ${money(last.gap_per_pupil)}. ${
          rateHeld === "held"
            ? `A formula doing the same proportional job against a larger problem leaves districts
        further apart every year, and the percentage is the thing that looks stable.`
            : `A formula closing a smaller proportion of a larger gap leaves districts further apart
        for both reasons at once, and the residual below is where the two compound.`
        }</p>

      <div class="scroll"><table>
        <thead><tr><th>Year</th><th class="tnum">Poorest quarter</th>
          <th class="tnum">Richest quarter</th><th class="tnum">Gap</th>
          <th class="tnum">State closes</th><th class="tnum">Federal closes</th>
          <th class="tnum">Unclosed</th></tr></thead>
        <tbody>${restated
          .map(
            (y) => `<tr>
              <th>FY${y.fiscal_year}</th>
              <td class="tnum n">${money(y.poorest_local_per_pupil)}</td>
              <td class="tnum n">${money(y.richest_local_per_pupil)}</td>
              <td class="tnum">${money(y.gap_per_pupil)}</td>
              <td class="tnum">${money(y.state_closes_per_pupil)}</td>
              <td class="tnum n">${money(y.federal_closes_per_pupil)}</td>
              <td class="tnum">${money(residual(y))}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>
    </div>`;
}

/**
 * What the two survey cards are measuring, and what they are not.
 *
 * # Why it comes after them (#549)
 *
 * It opened the page, ahead of the first chart, on the argument that a reader should know whose
 * measurement this is before reading any of it. The lead under the `h1` already says so — "measured
 * by the federal government rather than by the state" — so what this card added in first position
 * was the detail: the survey's name, its population, and why it does not reconcile with the
 * formula. That is apparatus in the three-tier order #549 set, and it now follows the two cards it
 * qualifies, drawn in the apparatus register.
 */
export function renderProvenance(bundle: Bundle): string {
  const history = bundle.history;
  if (history.length === 0) return "";
  const first = firstOf(history);
  const last = lastOf(history);

  return `
    <div class="card" data-card="apparatus" id="what-this-is" data-part="what-this-is">
      <h2>${anchor("what-this-is")}What this is, and what it is not</h2>
      <p class="note">The two cards above are drawn from the U.S. Census Bureau's Annual Survey
        of School System Finances, FY${first.fiscal_year} through FY${last.fiscal_year}. They are not the state's
        funding formula and they do not reconcile with it. The survey counts about
        ${last.districts} comparable Ohio systems a year — not counting community schools, joint
        vocational districts or educational service centers — on its own enrollment count and its own revenue
        classification, where everything else on this site is the Department of Education and
        Workforce's FY${bundle.fiscal_year} model of ${bundle.statewide.districts} traditional
        districts.</p>
      <p class="note">Which is the point of showing it. It is the only measurement here that
        reaches before FY2020, and the only one Ohio did not produce about itself. What it can be
        trusted for is in its <a href="/wiki/source/census-f33-school-system-finances">catalog
        record</a>.</p>
    </div>`;
}

/** Placeholder for a feed carrying no history, so the route is never a blank page. */
export function renderAbsent(): string {
  return `
    <div class="card" id="no-panel" data-part="no-panel">
      <h2>${anchor("no-panel")}No historical panel in this feed</h2>
      <p class="note">The feed carries no <code>history</code> block, so there is nothing to draw.
        That is a feed problem rather than a rendering one:
        <code>${escapeHtml("cargo run -p bundle")}</code> builds it from
        <code>crates/dispersion/fixtures/f33-ohio-panel.csv</code>.</p>
    </div>`;
}
