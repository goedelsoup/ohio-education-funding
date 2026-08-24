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
import { compare } from "./order.ts";
import * as routes from "./routes.ts";
import { median } from "./stats.ts";
import { yearChip, yearOf } from "./year.ts";
import { anchor } from "./section.ts";
import { BOX_FROM, distributionSpec, type Drawing, draws } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";

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
   * `null` for a county with fewer than two reporting districts. `renderSpread` counts how many
   * that is off the feed rather than stating it here, so the figure cannot go stale. A ratio
   * needs two districts and a single-district county has no internal disparity to measure — that
   * is a real answer rather than a missing one, and the page says so.
   */
  valuationRatio: number | null;
  richest: District | null;
  poorest: District | null;
}

/** The middle value. Module-level because two renderers need it and a duplicate would drift. */
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
      const sorted = [...list].sort((a, b) => compare(a.name, b.name));
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
    .sort((a, b) => compare(a.name, b.name));
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
function renderSpread(c: County, statewide: Statewide, all: County[]): string {
  if (!c.richest || !c.poorest || c.valuationRatio == null) {
    // Counted off `all` rather than written down. "Four of Ohio's 88" was typed here, and both
    // halves of it are properties of the feed: a district joining or leaving one of these counties
    // moves the first, and the second is however many counties the department's attribution
    // produces — not a fact about Ohio's map.
    const unmeasurable = all.filter((other) => other.valuationRatio == null).length;
    return `
      <div class="card" id="spread" data-part="spread">
        <h2>${anchor("spread")}One district${yearChip("formula")}</h2>
        <p class="note">${escapeHtml(c.name)} County has a single district in the funding model,
          so it has no internal disparity to measure. ${count(unmeasurable)} of Ohio's
          ${count(all.length)} counties are in that position. The comparison this page is built for
          is between districts, so for ${escapeHtml(c.name)} the state distribution on
          <a href="/districts">the districts index</a> is the more useful view.</p>
      </div>`;
  }

  const rich = c.richest;
  const poor = c.poorest;
  const gap = rich.valuation_per_pupil! - poor.valuation_per_pupil!;

  /*
   * Whether this county's disparity is a large one.
   *
   * The card has always stated the ratio and never said what it was a lot *of*. The median county
   * is 2.1× and ten of the 84 measurable ones exceed 4×, so "5.5 times apart" is near the top of
   * the state and a reader had no way to know it from this page. The same form the district pages
   * use for the same question, over the counties rather than the districts.
   */
  const ratios = all
    .filter((other) => other.valuationRatio != null)
    .map((other) => ({
      value: other.valuationRatio!,
      hover: `${other.name}: ${other.valuationRatio!.toFixed(1)}× between its richest and poorest district`,
    }));
  // Read outside the drawing: `valuationRatio` is narrowed by the guard at the top of this
  // function, and a property narrowing does not survive into a closure that runs later.
  const marker = { value: c.valuationRatio!, label: c.name };
  const position: Drawing = (w) => distributionSpec(ratios, { width: w, marker });

  /*
   * Every district in the county, not just the two ends of it.
   *
   * The table below names the richest and the poorest, which is the comparison the card is built
   * on and was also the whole of what it drew. A county has six districts at the median and
   * thirty-one at the largest, and a pair of rows says nothing about whether the other twenty-nine
   * are spread evenly between them or bunched at one end — which is the difference between a
   * county with a disparity and a county with one unusual district.
   *
   * Dots rather than a box, because a box plot of six values is five statistics standing in for
   * six numbers a reader could simply have been shown.
   */
  const dots: Drawing = (w) =>
    distributionSpec(
      c.districts
        .filter((d) => d.valuation_per_pupil != null)
        .map((d) => ({
          value: d.valuation_per_pupil!,
          hover: `${d.name}: ${money(d.valuation_per_pupil!)} per pupil, ${money(d.realized_aid_per_pupil)} state aid per pupil`,
        })),
      { width: w },
    );

  return `
    <div class="card" id="spread" data-part="spread">
      <h2>${anchor("spread")}The spread inside ${escapeHtml(c.name)} County${yearChip("formula")}</h2>
      ${
        draws(dots)
          ? `<p class="note">Every district in the county by the assessed valuation each of its
             pupils stands on — one dot each, poorest on the left.${
               c.districts.filter((d) => d.valuation_per_pupil != null).length >= BOX_FROM
                 ? " The shaded box is the middle half of them."
                 : ""
             }</p>
             <div class="chartwrap" data-chart="county-spread">${renderToString(dots, { label: `Assessed valuation per pupil for each of the ${count(c.districts.filter((d) => d.valuation_per_pupil != null).length)} districts in ${c.name} County reporting a tax base, poorest at left, ${yearOf("formula")}` })}</div>
             <div class="scale">
               <span>${money(poor.valuation_per_pupil!)}</span>
               <span>${money(rich.valuation_per_pupil!)}</span>
             </div>`
          : ""
      }
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
      <p class="note">These four figures are the headline; <a href="${routes.compare(
        rich.irn,
        poor.irn,
      )}">the full comparison</a> puts the two side by side on every measure the panel carries.</p>
      ${
        draws(position)
          ? `<p class="note">Where that sits among the ${ratios.length} counties with more than one
             district reporting a tax base — one dot each, narrowest on the left, and the coloured
             rule is ${escapeHtml(c.name)}. The median is
             ${median(ratios.map((r) => r.value)).toFixed(1)}× apart.</p>
             <div class="chartwrap" data-chart="county-position">${renderToString(position, "presentational")}</div>
             <div class="scale">
               <span>${Math.min(...ratios.map((r) => r.value)).toFixed(1)}×</span>
               <span>${Math.max(...ratios.map((r) => r.value)).toFixed(1)}×</span>
             </div>`
          : ""
      }
      <p class="note">Two districts in the same county, <strong>${c.valuationRatio.toFixed(
        1,
      )} times</strong> apart on <a href="${routes.at(
        routes.districtTaxes(poor.irn),
        routes.SECTIONS.district.taxBase,
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
  const medianAid = median(c.districts.map((d) => d.realized_aid_per_pupil));

  return `
    <div class="card" id="roster" data-part="roster">
      <h2>${anchor("roster")}${count(c.districts.length)} district${c.districts.length === 1 ? "" : "s"}${yearChip("formula")}</h2>
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
      <p class="note">Median state aid here is ${money(medianAid)} per pupil,
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

/**
 * The median across every district in the feed, for the roster to compare a county against.
 *
 * Deferred to `stats.median` rather than sorted here. This was a third hand-rolled copy in a file
 * that already imports the shared one four lines above — and it took the upper-middle element,
 * which is the definition the whole workspace has now stopped using. See `stats.ts` on why.
 */
export function medianRealizedAid(districts: District[]): number {
  return median(districts.map((d) => d.realized_aid_per_pupil));
}

/** One county page: the spread, then the roster. */
export function renderCounty(
  c: County,
  statewide: Statewide,
  statewideMedianAid: number,
  /** Every county, so the spread card can say where this one's disparity sits among them. */
  all: County[],
): string {
  return `${renderSpread(c, statewide, all)}${renderRoster(c, statewide, statewideMedianAid)}`;
}
