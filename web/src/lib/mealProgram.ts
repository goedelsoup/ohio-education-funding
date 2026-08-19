/**
 * The longest run of the count the formula's poverty weight is paid on, FY1998 to FY2014.
 *
 * # Why this is on the history route and not the outcomes one
 *
 * It looks like a poverty measure and the site already has one: `economically_disadvantaged`, from
 * the 2024-25 report card, on every district page. They are not the same thing and cannot be put
 * in one series. The report card's share is top-coded by community eligibility — a district where
 * every pupil eats free reports 100% whether its actual rate is 45% or 95% — and it is one year.
 * This is seventeen years of *applications approved*, which is the test R.C. 3317.03(B)(21) leaves
 * to the department and the count
 * [disadvantaged pupil impact aid](../../../.yidam/corpus/formula-component/fsfp-disadvantaged-pupil-impact-aid.yml)
 * is actually paid on.
 *
 * So it belongs where the other long series live, on a route whose whole premise is that its
 * figures are measured differently from the formula side and must not be read across to it.
 *
 * # The break at FY2010 is drawn, not annotated
 *
 * The denominator changes mid-series: `AdmCount` through FY2009, `CECount` — "the highest daily
 * number of students with access to the program" — from FY2010. That is not a redefinition at the
 * margins; it is a different population in the divisor, and the share steps up across it.
 *
 * A footnote saying so would be read after the eye has already drawn the line. So the two eras are
 * two separate series on the chart, `a` and `b`, with nothing joining them. A reader who wants the
 * seventeen-year trend has to decide to splice it, which is the decision this module exists to
 * make visible rather than to make for them.
 *
 * # The break at FY2012 is not drawn at all
 *
 * The second break is worse than the first, and the card handles it by stopping. From FY2012 the
 * department publishes MR-81 as three files and only one of them still counts applications:
 * community-eligibility sponsors collect none, because every child in those schools eats free. A
 * line joined across FY2011 falls thirteen points in three years and would read as poverty
 * collapsing during a period when the share of enrolment in those schools more than doubled.
 *
 * The chart therefore ends at FY2011. What follows is a band — the directly certified count as a
 * floor, what those schools may claim for as a ceiling — and the band brackets FY2011 rather than
 * sitting below it, so what the source will not settle is the *direction*. That is stated in prose
 * and in the table, and no line is drawn through it, because a line through a band is a claim.
 *
 * # Sponsors, not districts
 *
 * The population is meal-program sponsors: community schools and county boards of developmental
 * disabilities alongside traditional districts. It grows from 718 to 1,001 across the window,
 * mostly because community schools opened. The count is on the page for that reason — a rising
 * share over a changing population is two facts, and showing one without the other states the
 * wrong one.
 */

import { escapeHtml, pct } from "./format.ts";
import { seriesSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import type { MealProgramYear } from "./types.ts";
import { yearChip } from "./year.ts";
import { anchor } from "./section.ts";

/** The years the report is one file, and so the years a single share exists for. */
export function singleStream(meal: MealProgramYear[]): MealProgramYear[] {
  return meal.filter((y) => y.streams === 1 && y.share != null);
}

/** The years published as three files, which carry a band instead. */
export function splitStream(meal: MealProgramYear[]): MealProgramYear[] {
  return meal.filter((y) => y.streams > 1);
}

/** The fiscal year the denominator changes, or `null` if the series never crosses it. */
export function basisChange(meal: MealProgramYear[]): number | null {
  for (let i = 1; i < meal.length; i++) {
    if (meal[i]!.basis !== meal[i - 1]!.basis) return meal[i]!.fiscal_year;
  }
  return null;
}

/**
 * The share as two series, split at the basis change.
 *
 * `a` carries the `adm` era and `b` the `ce` era, each `null` outside its own. Two series rather
 * than one with a gap in it: a gap says "not measured", and this was measured — on something else.
 *
 * Only the years that have a share at all. The split era is absent from the chart rather than
 * drawn as a third series, because what it has is two numbers and a chart drawn from either of
 * them would be the assertion this whole module refuses to make.
 */
export function splitByBasis(meal: MealProgramYear[]): {
  year: number;
  a: number | null;
  b: number | null;
}[] {
  return singleStream(meal).map((y) => ({
    year: y.fiscal_year,
    a: y.basis === "adm" ? y.share! * 100 : null,
    b: y.basis === "ce" ? y.share! * 100 : null,
  }));
}

/** The meal-program poverty share, October by October. */
export function renderMealProgram(meal: MealProgramYear[]): string {
  const single = singleStream(meal);
  const split = splitStream(meal);
  if (single.length < 2) return "";

  const first = single[0]!;
  const last = single[single.length - 1]!;
  const change = basisChange(single);
  const end = split[split.length - 1];

  const chart = renderToString(
    seriesSpec(
      splitByBasis(meal),
      { a: "on ADM", b: "on CE" },
      (v) => `${v.toFixed(0)}%`,
      (p) => {
        const value = p.a ?? p.b;
        if (value == null) return `FY${p.year}`;
        return `FY${p.year}: ${value.toFixed(1)}% ${p.a != null ? "of ADM" : "of CE count"}`;
      },
    ),
  );

  return `
    <div class="card" id="meal-program" data-part="meal-program">
      <h2>${anchor("meal-program")}What the poverty weight is counted on${yearChip("meal_program")}</h2>
      <p class="note">Free and reduced-price lunch applications approved, as a share of the
        meal-program enrollment count, across every public sponsor in the Office for Child
        Nutrition's MR-81. It rose from ${pct(first.share!, 1)} in FY${first.fiscal_year} to
        ${pct(last.share!, 1)} in FY${last.fiscal_year} — fourteen Octobers, where the rest of this
        site has six years of anything.</p>

      <div class="scroll">${chart}</div>

      ${
        change == null
          ? ""
          : `<p class="note"><strong>The two lines are not one line.</strong> The denominator
        changes in FY${change}: through FY${change - 1} it is <code>AdmCount</code>, and from
        FY${change} it is <code>CECount</code> — the highest daily number of students with access
        to the program, which is neither ADM nor the count before it. The series is drawn in two
        pieces because joining them would assert a continuity the source does not have. The step
        across FY${change} is partly the definition moving, and nothing here can say how much.</p>`
      }

      ${
        end == null
          ? ""
          : `<p class="note"><strong>The line stops in FY${last.fiscal_year} and the report does
        not.</strong> From FY${split[0]!.fiscal_year} the department publishes MR-81 as three
        files, and only one of them counts applications: schools serving under community
        eligibility feed every child and collect no forms at all, so their approvals are zero by
        construction. By FY${end.fiscal_year} ${pct(end.without_applications, 0)} of the
        meal-program enrollment sat under such a sponsor, against
        ${pct(split[0]!.without_applications, 0)} three years earlier — and they are there because
        their poverty was high enough to qualify. Adding the three files gives
        ${pct(end.approved / end.enrollment, 1)} for FY${end.fiscal_year}, which would read as
        poverty collapsing.</p>

      <p class="note">What the source supports for those years is a range, not a figure. Its floor
        counts the children directly certified through SNAP, food assistance, foster care or a
        homeless roll — everyone who would have qualified by filing and did not appear on one of
        those lists is missing from it. Its ceiling is what those schools may claim for, which is
        the same count times the 1.6 the federal programme uses, capped at enrollment. For
        FY${end.fiscal_year} that is ${pct(end.floor, 1)} to ${pct(end.ceiling, 1)}. The band
        contains FY${last.fiscal_year}'s ${pct(last.share!, 1)}, so the thing this series cannot
        settle is not the level but the direction.</p>`
      }

      <p class="note">The population is <em>sponsors</em>, not districts: community schools and
        county boards of developmental disabilities are counted alongside traditional districts,
        and the sponsor count rises from ${first.sponsors} to ${(end ?? last).sponsors} across the
        window mostly because community schools opened. Nothing here may be read against the report
        card's economically-disadvantaged share, which is one year and top-coded by community
        eligibility, or against any figure on the formula side.</p>

      <div class="scroll"><table>
        <thead><tr><th>October</th><th class="tnum">Sponsors</th><th class="tnum">Enrollment</th>
          <th class="tnum">Approved</th><th class="tnum">Share</th><th>Counted on</th></tr></thead>
        <tbody>${meal
          .map(
            (y) => `<tr>
              <th>FY${y.fiscal_year}</th>
              <td class="tnum n">${y.sponsors}</td>
              <td class="tnum n">${Math.round(y.enrollment).toLocaleString("en-US")}</td>
              <td class="tnum n">${Math.round(y.approved).toLocaleString("en-US")}</td>
              <td class="tnum">${
                y.share == null ? `${pct(y.floor, 1)}–${pct(y.ceiling, 1)}` : pct(y.share, 1)
              }</td>
              <td>${escapeHtml(
                y.streams > 1
                  ? "CECount, three files"
                  : y.basis === "ce"
                    ? "CECount"
                    : "AdmCount",
              )}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>
    </div>`;
}
