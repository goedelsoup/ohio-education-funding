/**
 * The longest run of the count the formula's poverty weight is paid on, FY2001 to FY2011.
 *
 * # Why this is on the history route and not the outcomes one
 *
 * It looks like a poverty measure and the site already has one: `economically_disadvantaged`, from
 * the 2024-25 report card, on every district page. They are not the same thing and cannot be put
 * in one series. The report card's share is top-coded by community eligibility — a district where
 * every pupil eats free reports 100% whether its actual rate is 45% or 95% — and it is one year.
 * This is eleven years of *applications approved*, which is the test R.C. 3317.03(B)(21) leaves to
 * the department and the count
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
 * eleven-year trend has to decide to splice it, which is the decision this module exists to make
 * visible rather than to make for them.
 *
 * # Sponsors, not districts
 *
 * The population is meal-program sponsors: community schools and county boards of developmental
 * disabilities alongside traditional districts. It grows from 730 to 949 across the window, mostly
 * because community schools opened. The count is on the page for that reason — a rising share over
 * a changing population is two facts, and showing one without the other states the wrong one.
 */

import { escapeHtml, pct } from "./format.ts";
import { seriesSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import type { MealProgramYear } from "./types.ts";

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
 */
export function splitByBasis(meal: MealProgramYear[]): {
  year: number;
  a: number | null;
  b: number | null;
}[] {
  return meal.map((y) => ({
    year: y.fiscal_year,
    a: y.basis === "adm" ? y.share * 100 : null,
    b: y.basis === "ce" ? y.share * 100 : null,
  }));
}

/** The meal-program poverty share, October by October. */
export function renderMealProgram(meal: MealProgramYear[]): string {
  if (meal.length < 2) return "";

  const first = meal[0]!;
  const last = meal[meal.length - 1]!;
  const change = basisChange(meal);

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
    <div class="card">
      <h2>What the poverty weight is counted on</h2>
      <p class="note">Free and reduced-price lunch applications approved, as a share of the
        meal-program enrollment count, across every public sponsor in the Office for Child
        Nutrition's MR-81. It rose from ${pct(first.share, 1)} in FY${first.fiscal_year} to
        ${pct(last.share, 1)} in FY${last.fiscal_year} — eleven Octobers, where the rest of this
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

      <p class="note">The population is <em>sponsors</em>, not districts: community schools and
        county boards of developmental disabilities are counted alongside traditional districts,
        and the sponsor count rises from ${first.sponsors} to ${last.sponsors} across the window
        mostly because community schools opened. Nothing here may be read against the report
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
              <td class="tnum">${pct(y.share, 1)}</td>
              <td>${escapeHtml(y.basis === "ce" ? "CECount" : "AdmCount")}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>
    </div>`;
}
