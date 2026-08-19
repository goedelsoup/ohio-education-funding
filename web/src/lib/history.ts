/**
 * The long view: Ohio's school revenue as the Census Bureau measured it, FY2009 to FY2022.
 *
 * # Why this is a separate page and not a card on the front one
 *
 * Every other figure on this site is one fiscal year of the Fair School Funding Plan, computed
 * for the 609 traditional districts in the department's own calculator. This is a different
 * measurement of a different population: roughly 950 Ohio agencies a year — community schools and
 * educational service centres included — surveyed by the federal government, on the Bureau's
 * enrollment count rather than ADM, with revenue classified the Bureau's way.
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
 * 2. **The rate held and the gap grew.** State aid closes a stable share of the distance between
 *    the poorest and richest quartiles of districts, while that distance itself grew by two
 *    thirds — so the part no level of government closes grew with it. A page showing only the
 *    percentage would report that as stability, which is why the residual is drawn in dollars.
 *
 * Both endpoints flatter the federal column: FY2009-FY2011 carry the ARRA tail and FY2022 is the
 * ESSER peak, so the federal contribution is roughly double its ordinary size at each end. The
 * card says so, because a reader taking the federal line at face value would draw the wrong
 * conclusion from exactly the years that are easiest to notice.
 *
 * # Real dollars are not optional here
 *
 * CPI-U rose 37% across this panel. A gap growing from $5,674 to $9,590 per pupil in nominal
 * terms is a different claim from the same gap in constant dollars, and only one of them is the
 * finding. The dollar series is therefore shown on both bases, through the same {@link Basis}
 * toggle every other financial view uses, and a year the index cannot cover is dropped rather
 * than shown un-deflated in a column labelled real.
 */

import type { SeriesPoint } from "./chart.ts";
import { escapeHtml, money, pct } from "./format.ts";
import { seriesSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import { convert, type Basis } from "./real.ts";
import type { Bundle, Deflator, HistoryYear } from "./types.ts";
import { yearChip } from "./year.ts";
import { anchor } from "./section.ts";

/** What neither level of government closes — the part a district actually experiences. */
export function residual(year: HistoryYear): number {
  return year.gap_per_pupil - year.state_closes_per_pupil - year.federal_closes_per_pupil;
}

/** State aid's share of the gap it is measured against. */
export function stateShareOfGap(year: HistoryYear): number {
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
  for (let y = history[0]!.fiscal_year; y <= history[history.length - 1]!.fiscal_year; y++) {
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
    return { year, a: y ? a(y) : null, b: y ? b(y) : null };
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
export function renderRevenueMix(history: HistoryYear[]): string {
  if (history.length < 2) return "";

  const first = history[0]!;
  const last = history[history.length - 1]!;
  const points = withGaps(
    history,
    (y) => y.local_share * 100,
    (y) => y.state_share * 100,
  );

  const chart = renderToString(
    seriesSpec(
      points,
      { a: "local", b: "state" },
      (v) => `${v.toFixed(0)}%`,
      (p) =>
        p.a == null
          ? `FY${p.year}: not published`
          : `FY${p.year}: ${p.a.toFixed(1)}% local, ${(p.b ?? 0).toFixed(1)}% state`,
    ),
  );

  const missing = gaps(history);

  return `
    <div class="card" id="revenue-mix" data-part="revenue-mix">
      <h2>${anchor("revenue-mix")}Where the money came from${yearChip("history")}</h2>
      <p class="note">Local, state and federal revenue as shares of the total, across every
        comparable Ohio school system the Census Bureau surveyed. The state share fell from
        ${pct(first.state_share, 1)} in FY${first.fiscal_year} to ${pct(last.state_share, 1)} in
        FY${last.fiscal_year}, and the local share rose to meet it.</p>

      <div class="scroll">${chart}</div>

      <p class="note">The federal line is left off the chart and kept in the table below, because
        it is the one series whose ends cannot be read at face value: FY2009 through FY2011 carry
        the federal stimulus tail and FY${last.fiscal_year} is the pandemic relief peak, so the
        federal share is roughly double its ordinary size at both ends of this window.
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

  const first = restated[0]!;
  const last = restated[restated.length - 1]!;
  const points = withGaps(
    restated,
    (y) => y.gap_per_pupil,
    (y) => residual(y),
  );

  const chart = renderToString(
    seriesSpec(
      points,
      { a: "gap", b: "unclosed" },
      (v) => money(v),
      (p) =>
        p.a == null
          ? `FY${p.year}: not published`
          : `FY${p.year}: ${money(p.a)} gap, ${money(p.b ?? 0)} of it closed by nobody`,
    ),
  );

  const rateHeld =
    Math.abs(stateShareOfGap(last) - stateShareOfGap(first)) < 0.06 ? "held" : "moved";

  return `
    <div class="card" data-part="equity-gap">
      <h2>${anchor("equity-gap")}Whom it reached${yearChip("history")}</h2>
      <p class="note">Districts sorted by local revenue per pupil and cut into quarters. The gap is
        the distance between the poorest quarter and the richest; the second line is what neither
        state nor federal aid closes, which is the part a district actually experiences.
        ${basis === "real" ? `In FY${base} dollars.` : "In the dollars of each year."}</p>

      <div class="scroll">${chart}</div>

      <p class="note">State aid's share of the gap ${rateHeld} — ${pct(stateShareOfGap(first), 0)}
        in FY${first.fiscal_year} against ${pct(stateShareOfGap(last), 0)} in
        FY${last.fiscal_year} — while the gap itself grew from ${money(first.gap_per_pupil)} to
        ${money(last.gap_per_pupil)}. A formula doing the same proportional job against a larger problem
        leaves districts further apart every year, and the percentage is the thing that looks
        stable.</p>

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

/** What this page is measuring, said before any of it. */
export function renderProvenance(bundle: Bundle): string {
  const history = bundle.history;
  if (history.length === 0) return "";
  const first = history[0]!;
  const last = history[history.length - 1]!;

  return `
    <div class="card" id="what-this-is" data-part="what-this-is">
      <h2>${anchor("what-this-is")}What this is, and what it is not</h2>
      <p class="note">These figures are the U.S. Census Bureau's Annual Survey of School System
        Finances, FY${first.fiscal_year} through FY${last.fiscal_year}. They are not the state's
        funding formula and they do not reconcile with it. The survey counts about
        ${last.districts} comparable Ohio systems a year — community schools and educational
        service centres among them — on its own enrollment count and its own revenue
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
