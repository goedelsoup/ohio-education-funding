/**
 * Counties as peer groups.
 *
 * # Why a county page, when the corpus is about districts
 *
 * Ohio's funding argument is a claim about *neighbours*. `DeRolph` was not that Ohio spends too
 * little on average; it was that two districts a few miles apart, whose children are equally
 * Ohio's, are funded by tax bases that differ by a factor of ten. A statewide distribution shows
 * that as a spread, and a spread is abstract. A county shows it as a list of places a reader can
 * drive between.
 *
 * So these pages lead with **within-county dispersion** rather than with county totals. The
 * headline of a county page is the ratio between its richest and poorest district, because that is
 * the quantity the litigation was about and it is invisible on both of the pages that already
 * exist — the district page shows one district, and the districts index shows all 609 at once.
 *
 * # What a county is here, and what it is not
 *
 * The department attributes each district to exactly one county, and this inherits that. It is a
 * simplification: Ohio school district boundaries follow historical township lines and cross
 * county lines freely, so a district's pupils, valuation and voters are not all inside the county
 * named on its row.
 *
 * That makes these pages **comparisons, not aggregations**. Summing 31 Cuyahoga districts and
 * calling the result Cuyahoga's school funding would be wrong in a way no caveat repairs, because
 * some of those districts' tax bases are in Geauga and Summit. Comparing 31 districts that the
 * department groups together is fine, and it is what a reader wanting to know how their district
 * sits against its neighbours is asking for.
 *
 * The one place this file sums is enrolment and dollars *of the districts listed*, always labelled
 * as such. Every ratio and rank is computed district-by-district and then summarised, so no figure
 * on the page depends on the county being a real boundary.
 */

import type { District, Statewide } from "./types.ts";
import { count, escapeHtml, money, pct } from "./format.ts";
import * as routes from "./routes.ts";

/** A county's districts, with the dispersion measures the page is built on. */
export interface County {
  name: string;
  slug: string;
  districts: District[];
  /** Enrolment of the districts listed. Not "the county's pupils" — see the module note. */
  adm: number;
  /** How many of the listed districts are held up by the guarantee, at the floor, at the minimum. */
  onGuarantee: number;
  atMillageFloor: number;
  atMinimumStateShare: number;
  /**
   * Richest over poorest on assessed valuation per pupil, among districts that report it.
   *
   * `null` for a county with fewer than two reporting districts, which is four of the 88. A ratio
   * needs two districts and a single-district county has no internal disparity to measure — that
   * is a real answer rather than a missing one, and the page says so.
   */
  valuationRatio: number | null;
  richest: District | null;
  poorest: District | null;
}

/** `Van Wert` becomes `van-wert`; the slug is the URL and must round-trip through the router. */
export function slugify(county: string): string {
  return county
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}

/** Group the feed by the department's county attribution, richest county first by district count. */
export function counties(districts: District[]): County[] {
  const groups = new Map<string, District[]>();
  for (const d of districts) {
    const list = groups.get(d.county);
    if (list) list.push(d);
    else groups.set(d.county, [d]);
  }

  return [...groups.entries()]
    .map(([name, list]) => {
      const sorted = [...list].sort((a, b) => a.name.localeCompare(b.name));
      const withValuation = sorted
        .filter((d) => d.valuation_per_pupil != null && d.valuation_per_pupil > 0)
        .sort((a, b) => a.valuation_per_pupil! - b.valuation_per_pupil!);
      const poorest = withValuation[0] ?? null;
      const richest = withValuation[withValuation.length - 1] ?? null;

      return {
        name,
        slug: slugify(name),
        districts: sorted,
        adm: sorted.reduce((a, d) => a + d.adm, 0),
        onGuarantee: sorted.filter((d) => d.on_guarantee).length,
        atMillageFloor: sorted.filter((d) => d.at_millage_floor).length,
        atMinimumStateShare: sorted.filter((d) => d.at_minimum_state_share).length,
        valuationRatio:
          withValuation.length >= 2 && poorest!.valuation_per_pupil! > 0
            ? richest!.valuation_per_pupil! / poorest!.valuation_per_pupil!
            : null,
        richest: withValuation.length >= 2 ? richest : null,
        poorest: withValuation.length >= 2 ? poorest : null,
      };
    })
    .sort((a, b) => a.name.localeCompare(b.name));
}

/**
 * The headline: what separates the richest district in this county from the poorest.
 *
 * Two districts in the same county, often sharing a border, with tax bases per pupil that differ
 * by whatever this ratio is. In Cuyahoga it is over eight. That is the whole of `DeRolph` in one
 * sentence, and it is a sentence about places rather than about a distribution.
 *
 * The state share is shown beside it because it is the formula's answer to the disparity, and
 * reading the two together is the only way to see how much of the gap the state closes.
 */
function renderSpread(c: County, statewide: Statewide): string {
  if (!c.richest || !c.poorest || c.valuationRatio == null) {
    return `
      <div class="card" data-part="spread">
        <h2>One district</h2>
        <p class="note">${escapeHtml(c.name)} County has a single district in the funding model,
          so it has no internal disparity to measure. Four of Ohio's 88 counties are in that
          position. The comparison this page is built for is between districts, so for
          ${escapeHtml(c.name)} the state distribution on
          <a href="/districts">the districts index</a> is the more useful view.</p>
      </div>`;
  }

  const rich = c.richest;
  const poor = c.poorest;
  const gap = rich.valuation_per_pupil! - poor.valuation_per_pupil!;

  return `
    <div class="card" data-part="spread">
      <h2>The spread inside ${escapeHtml(c.name)} County</h2>
      <div class="scroll"><table>
        <thead><tr><th>District</th><th class="tnum">Valuation per pupil</th>
          <th class="tnum">State aid per pupil</th><th class="tnum">Pupils</th></tr></thead>
        <tbody>
          <tr class="current"><th><a href="${routes.district(rich.irn)}">${escapeHtml(
            rich.name,
          )}</a><div class="n">The most property wealth per pupil here</div></th>
            <td class="tnum">${money(rich.valuation_per_pupil!)}</td>
            <td class="tnum">${money(rich.realized_aid_per_pupil)}</td>
            <td class="tnum">${count(Math.round(rich.adm))}</td></tr>
          <tr class="current"><th><a href="${routes.district(poor.irn)}">${escapeHtml(
            poor.name,
          )}</a><div class="n">The least</div></th>
            <td class="tnum">${money(poor.valuation_per_pupil!)}</td>
            <td class="tnum">${money(poor.realized_aid_per_pupil)}</td>
            <td class="tnum">${count(Math.round(poor.adm))}</td></tr>
        </tbody>
      </table></div>
      <p class="note">Two districts in the same county, <strong>${c.valuationRatio.toFixed(
        1,
      )} times</strong> apart on <a href="${routes.at(
        routes.districtTaxes(poor.irn),
        routes.SECTIONS.taxBase,
      )}">the tax base each pupil stands on</a> — ${money(gap)} per pupil of assessed valuation. Every mill ${escapeHtml(poor.name)} levies raises
        ${(1 / c.valuationRatio).toFixed(2)} of what the same mill raises in
        ${escapeHtml(rich.name)}, which is the disparity <a href="${routes.wikiNode(
          "litigation",
          "derolph-i-1997",
        )}">DeRolph</a> was brought over and the reason the formula has an equalisation term at
        all.</p>
      <p class="note">${
        poor.realized_aid_per_pupil > rich.realized_aid_per_pupil
          ? `The formula answers it here: ${escapeHtml(poor.name)} receives
             ${money(poor.realized_aid_per_pupil - rich.realized_aid_per_pupil)} more state aid per
             pupil than ${escapeHtml(rich.name)}. Whether that closes the gap is a different
             question from whether it points the right way — ${money(gap)} of valuation per pupil
             is not repaired by ${money(
               poor.realized_aid_per_pupil - rich.realized_aid_per_pupil,
             )} of aid.`
          : `<strong>The formula does not answer it here.</strong> ${escapeHtml(rich.name)} has the
             larger tax base <em>and</em> receives ${money(
               rich.realized_aid_per_pupil - poor.realized_aid_per_pupil,
             )} more state aid per pupil than ${escapeHtml(poor.name)}. That is usually the
             guarantee, which holds a district at a historical amount regardless of what the
             formula now computes: ${
               rich.on_guarantee
                 ? `${escapeHtml(rich.name)} is on it.`
                 : `neither district here is on it, so the cause is elsewhere — most often a
                    difference in enrolment trend or in the categorical mix.`
             }`
      } Statewide, ${count(statewide.on_guarantee)} districts are held above their formula
        amount.</p>
    </div>`;
}

/** Every district in the county, on the measures a neighbour comparison turns on. */
function renderRoster(c: County, statewide: Statewide, statewideMedianAid: number): string {
  const median = (values: number[]): number => {
    const sorted = [...values].sort((a, b) => a - b);
    return sorted.length === 0 ? 0 : sorted[Math.floor(sorted.length / 2)]!;
  };
  const medianAid = median(c.districts.map((d) => d.realized_aid_per_pupil));

  return `
    <div class="card" data-part="roster">
      <h2>${count(c.districts.length)} district${c.districts.length === 1 ? "" : "s"}</h2>
      <div class="scroll"><table>
        <thead><tr>
          <th>District</th><th class="tnum">Pupils</th>
          <th class="tnum">Valuation per pupil</th><th class="tnum">State aid per pupil</th>
          <th class="tnum">Spending per pupil</th><th>Status</th>
        </tr></thead>
        <tbody>
          ${c.districts
            .map((d) => {
              const flags = [
                d.on_guarantee ? "on the guarantee" : "",
                d.at_millage_floor ? "at the 20-mill floor" : "",
                d.at_minimum_state_share ? "at the minimum state share" : "",
              ].filter(Boolean);
              return `<tr>
                <th><a href="${routes.district(d.irn)}">${escapeHtml(d.name)}</a></th>
                <td class="tnum">${count(Math.round(d.adm))}</td>
                <td class="tnum">${
                  d.valuation_per_pupil == null ? "—" : money(d.valuation_per_pupil)
                }</td>
                <td class="tnum">${money(d.realized_aid_per_pupil)}</td>
                <td class="tnum">${
                  d.operating_expenditure_per_pupil == null
                    ? "—"
                    : money(d.operating_expenditure_per_pupil)
                }</td>
                <td class="n">${flags.length === 0 ? "on formula" : escapeHtml(flags.join(" · "))}</td>
              </tr>`;
            })
            .join("")}
        </tbody>
      </table></div>
      <p class="note">${count(Math.round(c.adm))} pupils across the districts listed, and
        <strong>that is a sum of these districts rather than a count of the county's children</strong>
        — Ohio school district boundaries cross county lines, so some of these pupils live
        elsewhere and some of the county's live in districts on another page. The department
        attributes each district to one county and this page inherits that attribution; it is a
        peer group, not a boundary.</p>
      <p class="note">The median district here receives ${money(medianAid)} per pupil in state aid,
        against ${money(statewideMedianAid)} statewide.${
          c.onGuarantee > 0
            ? ` ${count(c.onGuarantee)} of ${count(c.districts.length)} ${
                c.onGuarantee === 1 ? "is" : "are"
              } on the guarantee, against ${pct(
                statewide.on_guarantee / statewide.districts,
                0,
              )} of the state.`
            : ` None is on the guarantee, which is unusual: ${pct(
                statewide.on_guarantee / statewide.districts,
                0,
              )} of Ohio districts are.`
        }${
          c.atMillageFloor > 0
            ? ` ${count(c.atMillageFloor)} ${
                c.atMillageFloor === 1 ? "is" : "are"
              } at or below the 20-mill floor, where H.B. 920's reduction factors stop operating and
              a reappraisal raises revenue again.`
            : ""
        }</p>
    </div>`;
}

/** The median across every district in the feed, for the roster to compare a county against. */
export function medianRealizedAid(districts: District[]): number {
  const sorted = districts.map((d) => d.realized_aid_per_pupil).sort((a, b) => a - b);
  return sorted.length === 0 ? 0 : sorted[Math.floor(sorted.length / 2)]!;
}

/** One county page: the spread, then the roster. */
export function renderCounty(
  c: County,
  statewide: Statewide,
  statewideMedianAid: number,
): string {
  return `${renderSpread(c, statewide)}${renderRoster(c, statewide, statewideMedianAid)}`;
}
