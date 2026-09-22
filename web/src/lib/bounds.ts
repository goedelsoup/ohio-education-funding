/**
 * The census of the modelled formula's bounds: every floor, ceiling and clamp, and who is on it.
 *
 * # Why this is a page and not a card on a node
 *
 * Every other drawing on this site is about a quantity — what a district is paid, what a component
 * costs, what a lever moves. This one is about the *shape* of the plan. `crates/project`'s census
 * walks all thirty-eight bounds the model can apply and asks, for each, how many of Ohio's
 * districts it is the operative term for: how often the `max` in R.C. 3317.011(F)(5)(b) is the
 * side that wins, rather than what it pays when it does.
 *
 * Thirty-eight edges at once is not a claim about any one component, so it would be filed wrongly
 * under any of them — see `PAGE_SERIES` in `corpusSeries.ts`, which is where the endpoint rule
 * that normally makes a node answer for a series' extremes is redirected to the plan node instead.
 *
 * # What is derived here and what is not
 *
 * Nothing on this page is typed. The whole of it — the ranking, the five family totals, the two
 * null results, the populations the counts are out of — is read off `crates/series.json`'s
 * `project/bounds-census`, which `crates/figures` computes from `project::bounds::census`. The
 * page can therefore be wrong only in the same way the crate is, and the crate's own test
 * (`the_bounds_that_cannot_bind_and_the_ones_that_never_have.rs`) pins the table bound by bound.
 *
 * The two null results are the reason for the derivation rather than a convenience. "One bound
 * cannot bind" and "none has never bound" are findings, and a hand-written page would keep saying
 * them after they stopped being true.
 */

import type { Rank } from "./chart.ts";
import {
  loadSeriesManifest,
  PAGE_SERIES,
  ranksOf,
  type ManifestSeries,
  type SeriesRow,
} from "./corpusSeries.ts";
import { count, escapeHtml } from "./format.ts";
import { compare } from "./order.ts";
import { rankSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import * as routes from "./routes.ts";
import { anchor } from "./section.ts";
import { yearChip, yearOf } from "./year.ts";

/** The one series this page draws. Named here because three things below look it up. */
export const SERIES_KEY = "project/bounds-census";

/** One of R.C. 3317's five families of provision, and how many bounds it contributes. */
export interface Family {
  label: string;
  bounds: number;
}

/** A population the counts are taken out of, and how many bounds are counted against it. */
export interface Population {
  of: number;
  bounds: number;
}

/** The census, as everything the page needs to say about it. */
export interface Census {
  series: ManifestSeries;
  /** The rows in the crate's own order, which is the ranking: most districts first. */
  rows: SeriesRow[];
  ranks: Rank[];
  /** By count of bounds, descending — the order the card reads them in. */
  families: Family[];
  /** The largest population any bound is counted against: every district in the model. */
  whole: number;
  /** The populations, largest first, so the ones that are not the whole are visible. */
  populations: Population[];
  most: SeriesRow;
  least: SeriesRow;
  /** Bounds the model can never reach, whatever the inputs. The chart marks these. */
  cannotBind: SeriesRow[];
  /** Bounds that can bind and do not, for any district in this panel. */
  neverBound: SeriesRow[];
  /** Bounds that are the operative term for exactly one district. */
  single: SeriesRow[];
  /** Where the endpoints are bound, because they are not bound on this page. */
  source: { route: string; node: string };
}

/**
 * Read the census out of the series manifest.
 *
 * Throws where the manifest does not carry it. That is the same rule `loadSeriesManifest` states
 * for an absent manifest: a page whose subject has gone missing must fail the build rather than
 * render an empty frame, because an empty frame reads as "there are no bounds".
 */
export function census(): Census {
  const series = loadSeriesManifest().series.find((s) => s.key === SERIES_KEY);
  if (!series) {
    throw new Error(
      `The series manifest carries no ${SERIES_KEY}, which is the whole subject of /bounds. ` +
        `Regenerate it: cargo run --manifest-path crates/Cargo.toml -p figures series > crates/series.json`,
    );
  }
  const source = PAGE_SERIES[SERIES_KEY];
  if (!source) {
    throw new Error(
      `${SERIES_KEY} is not in PAGE_SERIES, so nothing in the corpus answers for its endpoints ` +
        `and this page would be citing itself.`,
    );
  }

  const rows = series.rows;
  const tally = new Map<string, number>();
  for (const row of rows) {
    const family = row.group ?? "Unfamilied";
    tally.set(family, (tally.get(family) ?? 0) + 1);
  }
  /*
   * Ordered by size rather than by statute.
   *
   * The manifest carries the rows ranked, not in statutory order, so first appearance here would
   * order the families by their single largest bound — which is a fact about one provision
   * presented as a fact about a family. Size is both derived and the thing the card is about.
   */
  const families = [...tally]
    .map(([label, bounds]) => ({ label, bounds }))
    .sort((a, b) => b.bounds - a.bounds || compare(a.label, b.label));

  const counts = new Map<number, number>();
  for (const row of rows) {
    if (row.of === undefined) continue;
    counts.set(row.of, (counts.get(row.of) ?? 0) + 1);
  }
  const populations = [...counts]
    .map(([of, bounds]) => ({ of, bounds }))
    .sort((a, b) => b.of - a.of);
  const whole = populations[0]?.of ?? 0;

  return {
    series,
    rows,
    ranks: ranksOf(series),
    families,
    whole,
    populations,
    most: rows[0]!,
    least: rows[rows.length - 1]!,
    cannotBind: rows.filter((row) => row.marked != null),
    /* The second null result, and the harder one to see: a bound that *could* bind and does not
       for any district in this panel. It is the complement of the marked rows at zero, so it is
       derived rather than asserted — the day one appears, the card below says so on its own. */
    neverBound: rows.filter((row) => row.value === 0 && row.marked == null),
    single: rows.filter((row) => row.value === 1),
    source,
  };
}

/** `554 of 609`, or just `554` where the population is the whole model. */
function outOf(c: Census, row: SeriesRow): string {
  return row.of === undefined || row.of === c.whole
    ? count(row.value)
    : `${count(row.value)} of ${count(row.of)}`;
}

/**
 * The chart, and the two ends of it in words.
 *
 * Server-rendered, like every other static chart here — and on this page that is load-bearing
 * rather than conventional: `check:dist` reads the built HTML, and the endpoints it has to find
 * are inside this SVG.
 */
export function renderCensus(c: Census): string {
  const year = yearOf("formula");
  const naming = {
    label:
      `Every bound in the modelled formula, ranked by how many of Ohio's ${count(c.whole)} ` +
      `school districts each one is the operative term for${year ? `, ${year}` : ""}. ` +
      `Logarithmic scale, from ${count(c.most.value)} districts down to ${count(c.least.value)}.`,
  };
  const chart = renderToString(
    (width) =>
      rankSpec(c.ranks, { label: "Districts the bound is the operative term for", format: count }, {
        width,
      }),
    naming,
  );

  return `
    <div class="card" id="census" data-part="census">
      <h2>${anchor("census")}${count(c.rows.length)} bounds, and who is on each${yearChip("formula")}</h2>
      <p class="note">Each row is one floor, one ceiling or one clamp written into the funding
        formula, and its mark is the number of districts for which that bound — rather than the
        quantity it is bounding — is what actually decides the amount. The scale is logarithmic
        because the two ends of this census are ${count(c.most.value)} districts and
        ${count(c.least.value)}, and on a linear axis everything below about forty would be an
        indistinguishable stub against the axis.</p>
      <div class="chartwrap" data-chart="bounds-census">${chart}</div>
      <p class="note"><strong>${escapeHtml(c.most.label)}</strong> is the widest-reaching bound in
        the plan: it is the operative term for ${outOf(c, c.most)} districts, under
        ${escapeHtml(c.most.cites ?? "the statute")}. At the other end,
        <strong>${escapeHtml(c.least.label)}</strong> is operative for
        ${c.least.value === 0 ? "none of them" : outOf(c, c.least)}. Both figures, and the
        ${count(c.families.length)} family totals below, are pinned in
        <code>crates/figures.json</code> and quoted by <a href="${nodeHref(c)}">the corpus node for
        the plan itself</a> — which is where a claim about the whole plan is answered for, this
        page having no node of its own to bind it to.</p>
    </div>`;
}

/**
 * The corpus address of the node that answers for this series' endpoints.
 *
 * `PAGE_SERIES` holds the id, which is `class/node` — the form every corpus id takes — so the two
 * halves are split rather than stored separately. A malformed id would produce a link the
 * dist-link check fails on, which is the right place for it to be caught: `crossCheckSeries`
 * already requires the node to exist in the corpus before this renders at all.
 */
function nodeHref(c: Census): string {
  const [className, node] = c.source.node.split("/");
  return routes.wikiNode(className ?? "", node ?? "");
}

/** The five families, as counts. */
export function renderFamilies(c: Census): string {
  const rows = c.families
    .map(
      (family) => `<tr>
        <th>${escapeHtml(family.label)}</th>
        <td class="tnum">${count(family.bounds)}</td>
        <td class="tnum n">${((family.bounds / c.rows.length) * 100).toFixed(0)}%</td>
      </tr>`,
    )
    .join("");

  return `
    <div class="card" id="families" data-part="families">
      <h2>${anchor("families")}Where the bounds are${yearChip("formula")}</h2>
      <p class="note">The five parts of the formula do not carry equal numbers of edges.
        ${escapeHtml(c.families[0]!.label)} holds ${count(c.families[0]!.bounds)} of the
        ${count(c.rows.length)} on its own — the staffing ratios of R.C. 3317.011 are written as a
        minimum apiece, so each element of the base cost brings its own floor — while
        ${escapeHtml(c.families[c.families.length - 1]!.label)} has
        ${count(c.families[c.families.length - 1]!.bounds)}.</p>
      <div class="scroll"><table>
        <thead><tr><th>Family</th><th>Bounds</th><th>Share</th></tr></thead>
        <tbody>${rows}
          <tr class="current">
            <th>Every family</th>
            <td class="tnum">${count(c.rows.length)}</td>
            <td class="tnum">100%</td>
          </tr>
        </tbody>
      </table></div>
    </div>`;
}

/**
 * The census itself, as a table, grouped by family.
 *
 * The chart is ranked and this is not, deliberately: a reader who has seen the ranking wants to
 * know what is in each part of the formula, and a second copy of the same order would answer a
 * question they have already had answered. Within a family the rank order survives.
 *
 * `series-break` is the row-group divider `/wiki`'s meal-programme table introduced. It means "the
 * rows below are counted differently from the rows above" there and "the rows below are a
 * different part of the statute" here, and in both cases it is a `<tbody>` boundary rather than a
 * rule drawn over a continuous table — which is what makes it something a screen reader announces.
 */
export function renderTable(c: Census): string {
  const body = c.families
    .map((family) => {
      const rows = c.rows
        .filter((row) => (row.group ?? "") === family.label)
        .map(
          (row) => `<tr${row.marked ? ' class="current"' : ""}>
            <th>${escapeHtml(row.label)}${
              row.marked ? ` <span class="n">— ${escapeHtml(row.marked)}</span>` : ""
            }</th>
            <td class="n">${escapeHtml(row.cites ?? "")}</td>
            <td class="tnum">${count(row.value)}</td>
            <td class="tnum n">${row.of === undefined ? "" : count(row.of)}</td>
          </tr>`,
        )
        .join("");
      return (
        `<tbody><tr class="series-break"><th colspan="4">${escapeHtml(family.label)} — ` +
        `${count(family.bounds)} bounds</th></tr>${rows}</tbody>`
      );
    })
    .join("");

  return `
    <div class="card" id="table" data-part="table">
      <h2>${anchor("table")}The census${yearChip("formula")}</h2>
      <p class="note">Every bound the model applies, the provision it comes from, and the number of
        districts it is the operative term for. <strong>The last column is not always
        ${count(c.whole)}</strong>: a bound inside the transportation formula can only be operative
        for a district that formula runs for, and the same is true of the clawback. Reading a count
        against the wrong denominator is the mistake this column exists to prevent.</p>
      <div class="scroll"><table>
        <thead><tr><th>Bound</th><th>Provision</th><th>Districts</th><th>Out of</th></tr></thead>
        ${body}
      </table></div>
    </div>`;
}

/**
 * The two results that are absences.
 *
 * Both are rendered whichever way they come out. A card that appears only when it has something to
 * report is a card a reader cannot rely on: "no bound has ever failed to bind" and "this page did
 * not check" look identical when the second is silence.
 */
export function renderNulls(c: Census): string {
  const cannot =
    c.cannotBind.length === 0
      ? `<p class="note">Every bound in the census is reachable: for each of the
         ${count(c.rows.length)}, there is some district in this panel for which it is the
         operative term. That was not true when this census was first taken.</p>`
      : c.cannotBind
          .map(
            (row) => `<p class="note"><strong>${escapeHtml(row.label)}</strong>
             (${escapeHtml(row.cites ?? "the statute")}) is operative for
             <strong>no district in the state</strong>, and cannot be. It is a floor beneath a
             quantity that is never smaller than it — so it is written into the statute, it is
             modelled here, and it decides nothing. The chart above draws it at the foot of the
             scale rather than dropping it: a bound with no force is a finding, and a bar of
             length nought states nothing at all.</p>`,
          )
          .join("");

  const never =
    c.neverBound.length === 0
      ? `<p class="note"><strong>None.</strong> Every bound that can bind does bind, somewhere in
         the state, for at least one district. The thinnest margins are
         ${c.single
           .map((row) => `<strong>${escapeHtml(row.label)}</strong>`)
           .join(" and ")}, each of which is the operative term for exactly one district — which is
         the whole of the difference between a provision that does nothing and a provision that
         does something.</p>`
      : c.neverBound
          .map(
            (row) => `<p class="note"><strong>${escapeHtml(row.label)}</strong>
             (${escapeHtml(row.cites ?? "the statute")}) can bind and does not, for any district in
             this panel. Unlike the bound above it is not unreachable — a different year, or a
             different set of inputs, would put a district on it.</p>`,
          )
          .join("");

  /*
   * The headings count what is under them.
   *
   * A card headed "The bound that cannot bind" over a list of three is the same defect the body
   * of this module exists to avoid, one level up: the prose would go on asserting a finding the
   * data had stopped supporting. Zero keeps the plural, because the plural is what a heading over
   * an empty result has to be — "the bound that cannot bind" over "there is none" reads as a
   * contradiction rather than as a null result.
   */
  const bounds = (n: number) => (n === 1 ? "bound" : "bounds");

  return `
    <div class="card" id="cannot-bind" data-part="cannot-bind">
      <h2>${anchor("cannot-bind")}The ${bounds(c.cannotBind.length)} that cannot bind</h2>
      ${cannot}
    </div>

    <div class="card" id="never-bound" data-part="never-bound">
      <h2>${anchor("never-bound")}The ${bounds(c.neverBound.length)} that never have</h2>
      ${never}
    </div>`;
}
