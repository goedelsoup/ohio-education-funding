/**
 * The outcome view: what districts achieve, and how little of it the funding side explains.
 *
 * This is the axis the corpus spent three phases building and the platform did not show. Its
 * job is not to display the Performance Index — it is to make the two mistakes this data
 * invites hard to make:
 *
 * 1. **Reading a correlation with poverty as a correlation with something else.** Poverty
 *    explains the Performance Index at −0.846. Most other district-level correlates are that
 *    variable in disguise, and the guarantee is the sharpest case: +0.187 raw, +0.035 held.
 *    Every association here is shown with its controlled figure beside it, never alone.
 * 2. **Quoting a per-pupil number without its denominator.** The published spending figure
 *    divides by a need-weighted count. Against a composition-driven outcome that is mostly a
 *    composition proxy, which is why the same numerator gives −0.004 one way and −0.355 the
 *    other.
 */

import { barChart, type Bar } from "./chart.ts";
import { count, escapeHtml, money, pct } from "./format.ts";
import type { Bundle, District } from "./types.ts";

/** A correlation, signed and to three places. */
function coefficient(v: number): string {
  return (v >= 0 ? "+" : "−") + Math.abs(v).toFixed(3);
}

/** Districts grouped into fifths by poverty, poorest last. */
export function povertyQuintiles(districts: District[]): District[][] {
  const withBoth = districts
    .filter((d) => d.economically_disadvantaged != null && d.outcome?.performance_index != null)
    .sort((a, b) => a.economically_disadvantaged! - b.economically_disadvantaged!);
  const size = Math.floor(withBoth.length / 5);
  return Array.from({ length: 5 }, (_, i) =>
    // The last group takes the remainder, so integer division drops no district.
    withBoth.slice(i * size, i === 4 ? withBoth.length : (i + 1) * size),
  );
}

/** Median Performance Index in each poverty fifth. */
export function performanceByPoverty(districts: District[]): Bar[] {
  const labels = [
    "Least poor fifth",
    "Second",
    "Third",
    "Fourth",
    "Poorest fifth",
  ];
  return povertyQuintiles(districts).map((group, index) => {
    const scores = group
      .map((d) => d.outcome!.performance_index!)
      .sort((a, b) => a - b);
    const median = scores[Math.floor(scores.length / 2)] ?? 0;
    const poverty = group
      .map((d) => d.economically_disadvantaged!)
      .sort((a, b) => a - b);
    const medianPoverty = poverty[Math.floor(poverty.length / 2)] ?? 0;
    return {
      label: labels[index]!,
      value: median,
      direct: median.toFixed(1),
      hover: `${labels[index]}: ${group.length} districts, median Performance Index ${median.toFixed(1)}, median economic disadvantage ${pct(medianPoverty, 0)}`,
    };
  });
}

/** Render the outcome view. */
export function renderOutcomes(bundle: Bundle): string {
  const o = bundle.statewide.outcomes;
  if (!o) {
    return `<div class="card"><p class="note">This feed carries no outcome data.</p></div>`;
  }
  const bars = performanceByPoverty(bundle.districts);
  const withoutReportCard = bundle.statewide.districts - o.districts;

  return `
    <div class="tiles">
      <div class="tile"><div class="k">Poverty vs achievement</div>
        <div class="v">${coefficient(o.poverty_vs_performance)}</div>
        <div class="n">economic disadvantage against the Performance Index</div></div>
      <div class="tile"><div class="k">Guarantee vs achievement</div>
        <div class="v">${coefficient(o.guarantee_vs_performance)}</div>
        <div class="n">before controlling for poverty</div></div>
      <div class="tile"><div class="k">…holding poverty constant</div>
        <div class="v">${coefficient(o.guarantee_vs_performance_controlled)}</div>
        <div class="n">what is left of it</div></div>
    </div>

    <div class="card">
      <h2>Poverty is most of what the Performance Index measures</h2>
      <p class="note">Districts in fifths by economically disadvantaged share, least poor on the
        left. Median Performance Index in each.</p>
      <div class="chartwrap" data-chart="poverty-quintiles">${barChart(bars, { max: 120 })}</div>
      <p class="note">At <strong>${coefficient(o.poverty_vs_performance)}</strong>, economic
        disadvantage explains about ${pct(o.poverty_vs_performance ** 2, 0)} of the variance in
        Ohio's attainment measure. Any other district-level variable correlated with it will
        appear to predict achievement, and mostly will not be.</p>
    </div>

    <div class="card">
      <h2>The guarantee is that trap, exactly</h2>
      <div class="scroll"><table><tbody>
        <tr><th>Median Performance Index, districts on the guarantee</th>
            <td class="tnum">${o.median_performance_on_guarantee.toFixed(1)}</td></tr>
        <tr><th>Median Performance Index, districts on the formula</th>
            <td class="tnum">${o.median_performance_on_formula.toFixed(1)}</td></tr>
        <tr><th>Guarantee status against the Performance Index</th>
            <td class="tnum">${coefficient(o.guarantee_vs_performance)}</td></tr>
        <tr><th>The same, holding poverty constant</th>
            <td class="tnum">${coefficient(o.guarantee_vs_performance_controlled)}</td></tr>
      </tbody></table></div>
      <p class="note">Guaranteed districts do score higher. They score higher because they are
        <strong>less poor</strong> — the same wealth gradient that decides who is on the
        guarantee in the first place, arriving on the outcome side as an achievement effect that
        is not there. Held against poverty the association is
        <strong>${coefficient(o.guarantee_vs_performance_controlled)}</strong>.</p>
      <p class="note">This does not say the guarantee is harmless or useless. It says the
        guarantee's beneficiaries were already going to test better, and nothing in this data
        speaks to what their funding did for them.</p>
    </div>

    <div class="card">
      <h2>The same numerator, two denominators, two answers</h2>
      <div class="scroll"><table><tbody>
        <tr><th>Spending per <em>need-weighted</em> pupil vs achievement</th>
            <td class="tnum">${coefficient(o.weighted_spending_vs_performance)}</td></tr>
        <tr><th>Spending per <em>enrolled</em> pupil vs achievement</th>
            <td class="tnum">${coefficient(o.enrolled_spending_vs_performance)}</td></tr>
        <tr><th>Spending per enrolled pupil vs <em>growth</em>, holding poverty</th>
            <td class="tnum">${coefficient(o.spending_vs_growth_controlled)}</td></tr>
      </tbody></table></div>
      <p class="note">One district's total operating expenditure, divided two ways. The
        department publishes the first: a count weighted upward for disadvantage, English
        learners, and disability. Dividing a spending figure by a need index and then correlating
        it against a need-driven outcome measures the weighting more than the spending.</p>
      <p class="note">And the sign flips with the outcome measure. Against attainment the
        relationship is negative; against <strong>growth</strong>, holding poverty constant, it
        is <strong>${coefficient(o.spending_vs_growth_controlled)}</strong>. Same districts, same
        control, same year. An outcome-based adequacy standard has to pick one, and the choice
        decides which districts are found wanting.</p>
    </div>

    <div class="card">
      <h2>What this cannot tell you</h2>
      <p class="note">Every figure here is a correlation over ${count(o.districts)} districts, and
        none identifies an effect. Districts are not assigned to the guarantee at random — they
        are on it because their FY2020 funding exceeded what the formula now computes, which is
        itself a function of wealth and enrollment history.</p>
      <p class="note">${withoutReportCard} of the ${count(bundle.statewide.districts)} districts
        in the funding model have no report card and are absent from everything above. They are
        the three smallest in Ohio.</p>
    </div>`;
}

/** The outcome block for a single district, for the district view. */
export function renderDistrictOutcome(district: District): string {
  const o = district.outcome;
  if (!o) {
    return `<div class="card">
      <h2>Outcomes</h2>
      <p class="note">No report card is published for this district. It is one of the three
        smallest in Ohio, and it is outside every outcome figure on this site.</p>
    </div>`;
  }
  const series = [
    ["2022-23", o.performance_index_earliest],
    ["2023-24", o.performance_index_prior],
    ["2024-25", o.performance_index],
  ] as const;

  return `
    <div class="card">
      <h2>Outcomes</h2>
      <div class="scroll"><table><tbody>
        ${series
          .map(
            ([year, value]) =>
              `<tr><th>Performance Index, ${year}</th><td class="tnum">${
                value == null ? "—" : value.toFixed(1)
              }</td></tr>`,
          )
          .join("")}
        <tr><th>Progress (value-added effect size)</th>
            <td class="tnum">${o.progress_effect_size == null ? "—" : o.progress_effect_size.toFixed(2)}</td></tr>
        <tr><th>Operating spending per enrolled pupil</th>
            <td>${money(o.per_enrolled_pupil)}</td></tr>
        <tr><th>Operating spending per need-weighted pupil</th>
            <td>${money(o.per_equivalent_pupil)}</td></tr>
        <tr><th>Economically disadvantaged (report card)</th>
            <td>${o.economically_disadvantaged == null ? "—" : `${o.economically_disadvantaged.toFixed(1)}%`}</td></tr>
        <tr><th>English learners</th>
            <td>${o.english_learner == null ? "—" : `${o.english_learner.toFixed(1)}%`}</td></tr>
        <tr><th>Students with disabilities</th>
            <td>${o.students_with_disabilities == null ? "—" : `${o.students_with_disabilities.toFixed(1)}%`}</td></tr>
      </tbody></table></div>
      <p class="note">The Performance Index is close to a fixed district trait across these three
        years, which is why a change in funding is unlikely to show up in it. Progress is the
        measure that moves. Both are 2024-25; the spending figures are FY2025.</p>
      <p class="note">The two economically-disadvantaged shares on this page differ: the report
        card's is top-coded by community eligibility, the profile report's
        (${escapeHtml(pct(district.economically_disadvantaged, 1))}) is not.</p>
    </div>`;
}
