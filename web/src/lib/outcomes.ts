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

import type { Bar } from "./chart.ts";
import { barSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import { count, escapeHtml, money, pct } from "./format.ts";
import type { Bundle, District, OutcomeStatewide } from "./types.ts";

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
      <div class="chartwrap" data-chart="poverty-quintiles">${renderToString(barSpec(bars, { max: 120 }))}</div>
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

/**
 * Where this district's achievement sits once composition is accounted for.
 *
 * # Why the raw number is not shown on its own
 *
 * A Performance Index is read as a verdict on a district, and at −0.846 against poverty it is
 * mostly a description of who lives there. The same discipline the statewide view applies to
 * every correlation applies here to a single figure: it is shown beside the median of the
 * districts with comparable poverty, so the comparison a reader makes by default is to
 * like-composed districts rather than to the state.
 *
 * The gap that produces is still not an effect. It is what is left after one crude control, over
 * a hundred-odd districts, in one year — which the copy says outright.
 *
 * The difference is signed but deliberately **not coloured**. The gain and loss hues are used on
 * this site for money, where the direction of "more" is not in dispute. This card states in its
 * own copy that the figure is neither an effect nor a judgement about the district's schools, and
 * painting it red would say the opposite, louder. The comparison view holds the same line.
 */
export function renderOutcomeContext(bundle: Bundle, district: District): string {
  const o = district.outcome;
  // Three districts have no report card and three have no reported poverty share. Either one
  // makes the comparison undefined, and an omitted card is better than an invented peer group.
  if (o?.performance_index == null || district.economically_disadvantaged == null) return "";

  const quintiles = povertyQuintiles(bundle.districts);
  const index = quintiles.findIndex((group) => group.some((d) => d.irn === district.irn));
  if (index < 0) return "";
  const peers = quintiles[index]!;
  const scores = peers.map((d) => d.outcome!.performance_index!).sort((a, b) => a - b);
  const median = scores[Math.floor(scores.length / 2)] ?? 0;
  const gap = o.performance_index - median;
  const povertyRange = peers
    .map((d) => d.economically_disadvantaged!)
    .sort((a, b) => a - b);
  const label = [
    "least poor fifth",
    "second-least-poor fifth",
    "middle fifth",
    "second-poorest fifth",
    "poorest fifth",
  ][index]!;

  return `
    <div class="card" id="comparable-poverty" data-part="comparable-poverty">
      <h2>Against districts with comparable poverty</h2>
      <div class="tiles">
        <div class="tile"><div class="k">This district</div>
          <div class="v">${o.performance_index.toFixed(1)}</div>
          <div class="n">Performance Index, 2024-25</div></div>
        <div class="tile"><div class="k">Median of its poverty fifth</div>
          <div class="v">${median.toFixed(1)}</div>
          <div class="n">${count(peers.length)} districts in the ${escapeHtml(label)},
            ${pct(povertyRange[0]!, 0)}–${pct(povertyRange[povertyRange.length - 1]!, 0)}
            economically disadvantaged</div></div>
        <div class="tile"><div class="k">Difference</div>
          <div class="v">${gap >= 0 ? "+" : "−"}${Math.abs(gap).toFixed(1)}</div>
          <div class="n">points, against like-composed districts</div></div>
      </div>
      <p class="note">Ohio's attainment measure tracks economic disadvantage at
        <strong>${coefficient(bundle.statewide.outcomes?.poverty_vs_performance ?? 0)}</strong>
        statewide, so comparing this district to the state median would mostly be comparing its
        poverty rate to the state's. The middle tile is the comparison worth making, and the
        difference between them is what is left after one crude control.</p>
      <p class="note">It is <strong>not</strong> an effect of anything. Districts are not assigned
        to a poverty fifth at random, the fifths are wide, and a single year of a measure that
        barely moves cannot separate a school's contribution from its intake. See
        <a href="/outcomes">what this data can and cannot say</a>.</p>
    </div>`;
}

/** The outcome block for a single district, for the district view. */

/**
 * What the two published growth measures say about the same district, and when they disagree.
 *
 * # Why this is on the page rather than in a footnote
 *
 * The department publishes value-added as a three-year average and over a single year. This site
 * has used the three-year one everywhere — in the spending-against-growth correlation, in the
 * outcome route, in the corpus's own claims — without ever saying that a second figure exists.
 *
 * The reassuring part is that they agree wherever agreement means anything. Of the 534 districts
 * printing a non-zero value on both, 44 point opposite ways, and **not one of the 44 has both
 * magnitudes above 0.05**: every disagreement is a district within 0.04 of zero on both measures,
 * which is no measured growth either way and a sign that is arbitrary. The smoothing choice never
 * reverses a district with real movement.
 *
 * It still belongs on the page, because the sign is what gets reported. A district printed at
 * +0.01 and -0.01 has two published answers to "is it adding value", and only one of them appears
 * anywhere else on this site.
 *
 * The count is over determinate pairs only. 72 districts print an exact `0.00` on one measure —
 * anything in (-0.005, 0.005) — and a bare `a > 0 != b > 0` silently reads every one of those as
 * negative and reports 76 instead of 44. That was the first number computed here.
 */
function growthNote(
  threeYear: number | null,
  oneYear: number | null,
  statewide: OutcomeStatewide | null,
): string {
  if (threeYear == null || oneYear == null) return "";

  // Both must print a non-zero value before "opposite" means anything at two decimals.
  const determinate = Math.abs(threeYear) >= 0.005 && Math.abs(oneYear) >= 0.005;
  const disagree = determinate && threeYear > 0 !== oneYear > 0;
  const context = statewide
    ? `Statewide the two correlate at ${statewide.growth_measure_agreement.toFixed(2)}, and of the
       ${count(statewide.growth_measures_determinate)} districts printing a non-zero value on
       both, ${count(statewide.growth_measures_disagree)} point opposite ways — none of them with
       both magnitudes above 0.05.`
    : "";

  return `<p class="note">${
    disagree
      ? `<strong>The two growth measures point opposite ways for this district.</strong> Over
         three years its effect size is ${threeYear.toFixed(2)}; over one year it is
         ${oneYear.toFixed(2)}. Both are published by the department for the same year, and only
         the three-year figure appears anywhere else on this site. Read it as neither: a district
         this close to zero on both measures has no growth signal in either direction, and the
         sign is the arbitrary part. ${context}`
      : !determinate
        ? `One of the two growth measures prints ${
            Math.abs(threeYear) < 0.005 ? "0.00 over three years" : "0.00 over one year"
          }, which at two decimals covers anything within half a hundredth of zero and has no
           direction to read. ${context}`
        : `Both growth measures point the same way here (${threeYear.toFixed(2)} over three years,
           ${oneYear.toFixed(2)} over one). ${context} This site uses the three-year figure
           throughout, because a single year of value-added is noisy enough that the department
           smooths it deliberately.`
  }</p>`;
}

export function renderDistrictOutcome(
  district: District,
  statewide: OutcomeStatewide | null,
): string {
  const o = district.outcome;
  if (!o) {
    return `<div class="card" id="outcomes" data-part="outcomes">
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
    <div class="card" id="outcomes" data-part="outcomes">
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
        <tr><th>Progress, three-year average<div class="n">The department's headline growth
              measure, and the one every correlation on this site uses.</div></th>
            <td class="tnum">${o.progress_effect_size == null ? "—" : o.progress_effect_size.toFixed(2)}</td></tr>
        <tr><th>Progress, one year<div class="n">The same measure without the smoothing, also
              published.</div></th>
            <td class="tnum">${
              o.progress_effect_size_one_year == null
                ? "—"
                : o.progress_effect_size_one_year.toFixed(2)
            }</td></tr>
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
      ${growthNote(o.progress_effect_size, o.progress_effect_size_one_year, statewide)}
      <p class="note">The two economically-disadvantaged shares on this page differ: the report
        card's is top-coded by community eligibility, the profile report's
        (${escapeHtml(pct(district.economically_disadvantaged, 1))}) is not.</p>
    </div>`;
}
