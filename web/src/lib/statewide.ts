/** The statewide view: the three structural facts, and the one chart that shows the first. */

import type { Bar } from "./chart.ts";
import { barSpec, scatterSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import { count, escapeHtml, millions, money, pct } from "./format.ts";
import { realChange, series, type Basis } from "./real.ts";
import * as routes from "./routes.ts";
import type { TaxStatewide } from "./feed.ts";
import type { Bundle, District, National } from "./types.ts";
import { quintiles } from "./stats.ts";
import { yearChip, yearChipPair } from "./year.ts";
import { anchor } from "./section.ts";
import { medianTrace, pairs } from "./relationships.ts";

/**
 * The financial actuals, statewide, in both bases.
 *
 * # Why both, side by side
 *
 * Nominal, the story is a build-up and a partial drawdown that still ends above where it
 * started. In FY2020 dollars it ends **below**. Those two sentences support opposite arguments
 * about whether Ohio's districts are better placed than they were, and the difference is
 * entirely CPI — which rose 25.1% across the span. Showing one without the other would be
 * picking a side by omission, so this card shows both and names the index.
 */
export function renderStatewideFinances(bundle: Bundle, basis: Basis): string {
  const actuals = bundle.statewide.finances;
  if (actuals.length === 0) return "";
  const { years, converted, base } = series(bundle.deflator, actuals, basis);
  const first = years[0]!;
  const latest = years[years.length - 1]!;
  // Each statewide figure is a sum over the districts that reported the line, so it is null only
  // if none of them did — which has never happened, and would be a fact about the panel rather
  // than about the state. Handled rather than asserted: the card degrades to "—" instead of
  // reading a missing filing as a state with no cash.
  const held = years.filter((y) => y.ending_cash != null);
  const peak = held.reduce<(typeof held)[number] | null>(
    (a, b) => (a == null || b.ending_cash! > a.ending_cash! ? b : a),
    null,
  );

  const cashChange =
    latest.ending_cash == null || first.ending_cash == null
      ? null
      : latest.ending_cash - first.ending_cash;
  const yearsOfSpending =
    latest.ending_cash == null || !latest.total_expenditure
      ? null
      : latest.ending_cash / latest.total_expenditure;
  const realAid = realChange(bundle.deflator, actuals, (y) => y.state_aid);
  const firstAid = actuals[0]!.state_aid;
  const lastAid = actuals[actuals.length - 1]!.state_aid;
  const nominalAid = lastAid == null || firstAid == null || firstAid <= 0 ? null : lastAid / firstAid - 1;

  const bars: Bar[] = held.map((y) => ({
    label: `FY${y.fiscal_year}`,
    value: y.ending_cash!,
    hover: `FY${y.fiscal_year}: ${millions(y.ending_cash).replace("+", "")} held, ${millions(y.total_revenue).replace("+", "")} in, ${millions(y.total_expenditure).replace("+", "")} out`,
    ...(y.fiscal_year === peak?.fiscal_year || y.fiscal_year === latest.fiscal_year
      ? { direct: millions(y.ending_cash).replace("+", "") }
      : {}),
  }));

  const nominalFirstCash = actuals[0]!.ending_cash;
  const nominalLastCash = actuals[actuals.length - 1]!.ending_cash;
  const nominalCashChange =
    nominalLastCash == null || !nominalFirstCash ? null : nominalLastCash / nominalFirstCash - 1;

  const label = converted
    ? `FY${base} dollars`
    : basis === "real"
      ? "nominal — the index does not cover this panel"
      : "nominal";

  return `
    <div class="card" data-part="finances">
      <h2>${anchor("finances")}What districts actually received, spent, and hold — ${escapeHtml(label)}${yearChipPair("formula", "finances", "held")}</h2>
      <div class="tiles">
        <div class="tile"><div class="k">Cash held, FY${latest.fiscal_year}</div>
          <div class="v">${millions(latest.ending_cash).replace("+", "")}</div>
          <div class="n">${
            yearsOfSpending == null
              ? "no spending reported"
              : `${yearsOfSpending.toFixed(2)} years of spending at this rate`
          }</div></div>
        <div class="tile"><div class="k">Change since FY${first.fiscal_year}</div>
          <div class="v ${cashChange != null && cashChange < 0 ? "loss" : "gain"}">${millions(cashChange)}</div>
          <div class="n">${
            converted
              ? "in constant dollars, so this is purchasing power"
              : "before any price index"
          }</div></div>
        <div class="tile"><div class="k">State aid, FY${first.fiscal_year}–FY${latest.fiscal_year}</div>
          <div class="v ${(realAid ?? 0) < 0 ? "loss" : "gain"}">${
            realAid == null ? "—" : pct(realAid, 1)
          }</div>
          <div class="n">real; ${pct(nominalAid, 1)} nominal</div></div>
      </div>

      <div class="chartwrap" data-chart="statewide-cash">${renderToString(barSpec(bars), { label: `General fund cash held at 30 June, summed over the ${count(bundle.statewide.districts)} districts in this feed, by fiscal year, FY${first.fiscal_year} to FY${latest.fiscal_year}, ${label}` })}</div>
      <p class="note">General fund cash held at 30 June, summed over the
        ${count(bundle.statewide.districts)} districts in this feed.
        ${
          converted
            ? `Deflated with <strong>${bundle.deflator?.label ?? "an index"}</strong>. Nominally
               the balance ends ${pct(nominalCashChange, 0)}
               above FY${first.fiscal_year}; in constant dollars it ends
               ${pct(cashChange == null || !first.ending_cash ? null : cashChange / first.ending_cash, 0)}. Both are correct and they support
               opposite arguments, which is why this page will not show only one.`
            : `These are the figures as filed. The panel spans the sharpest price change in forty
               years, so switch to constant dollars before drawing a conclusion from the shape.`
        }</p>
      <p class="note">FY2021–FY2024 are the federal pandemic relief years: that money was booked
        in the general fund by some districts and separately by others, so a balance rising
        across them is not evidence about districts' own positions. These are audited actuals
        from districts' five-year forecast filings — not the FY${bundle.fiscal_year} calculator,
        and not comparable to it line for line.</p>
    </div>`;
}

/** Quintiles of assessed valuation per pupil, poorest first. */
export function wealthQuintiles(districts: District[]): District[][] {
  return quintiles(
    districts.filter((d) => d.valuation_per_pupil != null),
    (d) => d.valuation_per_pupil!,
  );
}

/** Share of each quintile funded by the guarantee. */
export function guaranteeRateByQuintile(districts: District[]): Bar[] {
  return wealthQuintiles(districts).map((group, index) => {
    const onGuarantee = group.filter((d) => d.on_guarantee).length;
    const share = group.length === 0 ? 0 : onGuarantee / group.length;
    const label = ["Poorest fifth", "Second", "Third", "Fourth", "Wealthiest fifth"][index]!;
    return {
      label,
      value: share,
      direct: pct(share, 0),
      hover: `${label}: ${onGuarantee} of ${group.length} districts on the guarantee, median valuation ${money(
        group[Math.floor(group.length / 2)]?.valuation_per_pupil ?? 0,
      )} per pupil`,
    };
  });
}

/**
 * The regime comparison's one directional sentence, with the direction read off the sign.
 *
 * The magnitude and the word for it were written separately: `money()` printed the figure and the
 * prose said "better off" whatever the figure was. The median has been negative since recognised
 * valuation was corrected, so this page read `−$47 per pupil better off` — a sentence asserting the
 * opposite of its own number. Aligning `money()`'s minus with the other three formatters made that
 * legible rather than causing it, and no formatter can fix it: the amount here is a magnitude and
 * the direction is a word chosen beside it, which is the only arrangement that cannot disagree.
 *
 * A magnitude that rounds away loses its direction with it — the rule `money` and `signedMoney`
 * already keep, for the same reason. "Worse off" on a figure that rounds to $0 is a claim the
 * figure does not support.
 */
export function renderRegimeDifference(median: number): string {
  const nowhere = "the median district is neither better nor worse off under the plan";
  if (Math.abs(median) < 0.5) return nowhere;
  const direction = median < 0 ? "worse" : "better";
  return `the median district is ${money(Math.abs(median))} per pupil ${direction} off under the plan`;
}

/**
 * The three structural facts, and the one chart that shows the first.
 *
 * Separate from {@link renderStatewideFinances} because that card is rendered twice — once per
 * dollar basis — and this one is not: nothing in it is a dollar series that a price index could
 * restate.
 */
export function renderStatewideStructure(bundle: Bundle, tax: TaxStatewide): string {
  /*
   * The two tax years the reduction-factor comparison spans, read off the feed.
   *
   * They were `TY2023 and TY2024`. The feed carries four tax years so that recognised valuation
   * can be reconstructed, and this sentence is about the last two — so the pair moves whenever
   * Table SD-1 gains a year, and the sentence would have gone on naming the old one.
   */
  const years = bundle.districts[0]?.property_tax.slice(-2) ?? [];
  const taxSpan =
    years.length === 2 ? `TY${years[0]!.tax_year} and TY${years[1]!.tax_year}` : "the two tax years";

  const s = bundle.statewide;
  const bars = guaranteeRateByQuintile(bundle.districts);

  /*
   * The wealth-neutrality cloud, and the two medians through it.
   *
   * The dots are what districts receive, split by whether the guarantee is what they receive it
   * under — which is the site's one genuinely two-way district split and is near enough balanced
   * (294 against 312) that neither half is a rounding error on the other. The lines are medians
   * per tenth of the valuation distribution: one of the formula's answer, one of the payment.
   * Their separation is the finding, and it was previously two numbers in a table.
   */
  const points = pairs(
    bundle.districts,
    (d) => d.valuation_per_pupil,
    (d) => d.realized_aid_per_pupil,
    (d, valuation, aid) =>
      `${d.name}: ${money(valuation)} valuation per pupil, ${money(aid)} state aid per pupil` +
      (d.on_guarantee ? ", on the guarantee" : ""),
  );
  const withValuation = bundle.districts.filter((d) => d.valuation_per_pupil != null);
  const realizedTrace = medianTrace(
    withValuation.map((d) => ({ x: d.valuation_per_pupil!, y: d.realized_aid_per_pupil })),
    10,
    "as received",
    "guarantee",
  );
  const formulaTrace = medianTrace(
    withValuation.map((d) => ({ x: d.valuation_per_pupil!, y: d.formula_aid_per_pupil })),
    10,
    "the formula",
    "formula",
  );
  const scatter = scatterSpec(
    points,
    {
      x: { label: "assessed valuation per pupil", format: (v) => money(v), log: true },
      y: { label: "state aid per pupil", format: (v) => money(v), log: true },
    },
    [formulaTrace, realizedTrace],
  );
  // Stated rather than left to the eye, at both ends of the wealth distribution. The gap is the
  // subject of the paragraph under the chart and a reader should not have to measure it off a line.
  const gapAt = (i: number) =>
    Math.round((realizedTrace.points[i]?.y ?? 0) - (formulaTrace.points[i]?.y ?? 0));
  const gapPoorest = gapAt(0);
  const gapWealthiest = gapAt(realizedTrace.points.length - 1);

  return `
    <div class="tiles">
      <div class="tile"><div class="k">State foundation aid</div>
        <div class="v">${millions(s.realized_aid_total).replace("+", "")}</div>
        <div class="n">FY${bundle.fiscal_year}, ${count(s.districts)} districts</div></div>
      <div class="tile"><div class="k">Paid by the guarantee</div>
        <div class="v">${millions(s.guarantee_total).replace("+", "")}</div>
        <div class="n">${pct(s.guarantee_total / s.realized_aid_total, 1)} of the total,
          to ${s.on_guarantee} districts</div></div>
      <div class="tile"><div class="k">Off the formula</div>
        <div class="v">${pct(s.on_guarantee / s.districts, 0)}</div>
        <div class="n">of districts are funded by the guarantee</div></div>
    </div>

    <div class="card" id="guarantee" data-part="guarantee">
      <h2>${anchor("guarantee")}Who is on the guarantee${yearChip("formula")}</h2>
      <p class="note">Districts grouped into fifths by assessed valuation per pupil, poorest on
        the left. The guarantee was written as transitional relief for districts losing
        students; the pattern it actually produces is a wealth gradient.</p>
      <div class="chartwrap" data-chart="quintiles">${renderToString(barSpec(bars, { max: 1 }), { label: `Share of districts on the guarantee by fifth of assessed valuation per pupil, poorest fifth first, FY${bundle.fiscal_year}` })}</div>
      <p class="note">Median valuation per pupil statewide is
        ${money(s.median_valuation_per_pupil)}.</p>
    </div>

    <div class="card" id="wealth-offset" data-part="wealth-offset">
      <h2>${anchor("wealth-offset")}Does state aid offset property wealth?${yearChipPair("formula", "profile", "valuation")}</h2>
      <p class="note">One dot per district: the assessed valuation each of its pupils stands on,
        against the state aid each of them receives. A compensating formula slopes down to the
        right, and this one does. The two lines are medians through ten equal-count groups of
        districts — one of what the formula computes, one of what is actually paid. Both scales
        are logarithmic, because valuation per pupil spans seventeen times and aid per pupil
        thirty; on a linear axis nine districts in ten sit in the left-hand third.</p>
      <div class="chartwrap" data-chart="wealth-offset">${renderToString(scatter, { label: `State aid per pupil against assessed valuation per pupil, one dot for each of ${count(points.length)} districts, with median lines for formula aid and aid as received, both axes logarithmic, FY${bundle.fiscal_year}` })}</div>
      <p class="note"><strong>The gap between the two lines is what the guarantee costs the
        equalisation.</strong> It is ${money(gapPoorest)} per pupil among the least wealthy tenth
        of districts and ${money(gapWealthiest)} among the wealthiest — the formula would pay the
        wealthy districts least, and the guarantee is what stops it. Which districts those are is
        the card above this one. Read as correlations against
        valuation per pupil, the formula alone reaches
        <strong>${s.wealth_neutrality_formula.toFixed(3)}</strong> and what districts receive
        reaches <strong>${s.wealth_neutrality_realized.toFixed(3)}</strong>; a perfectly
        compensating formula would be strongly negative.</p>
      <div class="scroll"><table><tbody>
        <tr><th>Aid vs. wealth — formula only</th>
            <td class="tnum">${s.wealth_neutrality_formula.toFixed(3)}</td></tr>
        <tr><th>Aid vs. wealth — as received</th>
            <td class="tnum">${s.wealth_neutrality_realized.toFixed(3)}</td></tr>
        <tr><th>Districts drawn</th>
            <td class="tnum">${count(points.length)}</td></tr>
      </tbody></table></div>
    </div>

    <div class="card" id="two-floors" data-part="two-floors">
      <h2>${anchor("two-floors")}Two floors${yearChipPair("formula", "millage", "millage")}</h2>
      <div class="scroll"><table><tbody>
        <tr><th>At or below the
            <a href="${routes.parameter("twenty-mill-floor")}">20-mill floor</a></th>
            <td>${s.at_millage_floor} districts (${pct(s.at_millage_floor / s.districts, 0)})</td></tr>
        <tr><th>Above it by under a twentieth of a mill</th>
            <td>${s.near_millage_floor} districts (${pct(s.near_millage_floor / s.districts, 0)})</td></tr>
        <tr><th>At the minimum state share of ${pct(s.minimum_state_share, 0)}</th>
            <td>${s.at_minimum_state_share} districts (${pct(s.at_minimum_state_share / s.districts, 0)})</td></tr>
        <tr><th>Median operating expenditure per pupil</th>
            <td>${money(s.median_operating_expenditure_per_pupil)}</td></tr>
      </tbody></table></div>
      <p class="note">At the <strong>20-mill floor</strong>, H.B. 920's tax reduction factors
        stop applying and rising property values reach district revenue — the only districts in
        Ohio for which that is true. At the <strong>minimum state share</strong>, the entire
        local capacity measure determines nothing: those districts receive a flat percentage of
        base cost however wealthy they are. That minimum is
        ${pct(s.minimum_state_share, 0)} in this model, not the 5% the
        <a href="${routes.wikiNode("funding-regime", "fair-school-funding-plan")}">Fair School
        Funding Plan</a> was enacted with — each biennial budget sets it.</p>
      <p class="note"><strong>The millage floor is visible in the tax record, not only in the
        statute.</strong> Across ${taxSpan},
        ${count(tax.rateFell.aboveFloor)} of the ${count(tax.districts.aboveFloor)} districts that
        began above the floor saw their effective Class I rate fall as reduction factors rolled it
        back; of the ${count(tax.districts.atFloor)} that began at it,
        ${count(tax.rateFell.atFloor)} did.
        ${pct(tax.reductionsAboveFloor, 1)} of every effective-rate reduction in Ohio happens above
        the floor, which is what it means to say reduction factors stop there — and why a
        reappraisal reaches one district's revenue and not another's.</p>
      <p class="note"><strong>And the size of it: the median district has lost
        ${pct(s.median_millage_reduction, 0)} of the millage its voters approved.</strong> The
        median voted current operating rate is
        ${s.median_voted_millage.toFixed(2)} mills; the median rate anyone pays is
        ${s.median_effective_millage.toFixed(2)}. Nobody decided that. It is reduction factors
        applied a hundredth of a mill at a time across the life of every levy, which is why a
        district can pass a levy, watch its tax base grow, and still return to the ballot.</p>
      <p class="note">Real property tax charged for current expenses is a median
        <strong>${pct(tax.medianChargeShare, 0)}</strong> of what a district spends on operations,
        and ${count(tax.chargedMoreThanSpent.length)} districts are charged more than they spend.
        A mill is the same rate everywhere and raises
        <strong>${money(s.min_yield_per_mill)} per pupil</strong> in the district where it raises
        least and <strong>${money(s.max_yield_per_mill)}</strong> where it raises most — a factor
        of ${Math.round(s.max_yield_per_mill / s.min_yield_per_mill)}, against a median of
        ${money(s.median_yield_per_mill)}. Comparing two districts' tax rates without that number
        compares effort to capacity. Each district's own figures are on its
        <a href="/districts">property tax page</a>, from the Department of Taxation's Table SD-1.</p>
      <p class="note"><strong>The mechanism this one replaced would charge half the state for
        revenue it cannot raise.</strong> Before the Fair School Funding Plan a district's own
        share was a flat 23 mills against its valuation, uniform statewide — and H.B. 920
        guarantees districts cannot all levy at one rate.
        ${count(s.below_charge_off_rate)} of ${count(s.districts)} have an effective Class I rate
        below 23 mills today, and ${count(s.charge_off_exceeds_base_cost)} have valuation high
        enough that the charge-off would take their whole base cost and leave nothing, having no
        minimum state share to stop at. Ohio's answer was a supplement rather than a fix. Against
        that counterfactual ${renderRegimeDifference(s.median_regime_difference)}.</p>
    </div>`;
}

/**
 * Where Ohio sits among the states.
 *
 * # The claim this finally tests
 *
 * *DeRolph* held that Ohio relied too heavily on local property tax. Every other figure on this
 * site is Ohio describing itself, so that holding could be restated and never checked — "too
 * heavily" is a comparison, and there was nothing to compare against. The Census Bureau's Annual
 * Survey of School System Finances is the third source and the first federal one, covering every
 * school system in the country on one set of definitions.
 *
 * The answer is that Ohio is unusual, and specifically in the way *DeRolph* said. It spends about
 * the national average per pupil and takes an exactly average share from Washington. What is
 * distinctive is who pays the rest.
 *
 * # Why the property tax rank is over 39 states and the others are over 51
 *
 * Twelve states fund schools through a parent city or county rather than through a district that
 * levies for itself. The survey attributes their property tax to the parent, so Massachusetts and
 * Virginia — which lean on property tax about as hard as anywhere — report zero. Local revenue
 * includes the parent appropriation and is comparable either way. Ranking all 51 on property tax
 * would have produced a confident and wrong answer, and nearly did.
 */
export function renderNational(national: National | null): string {
  if (!national) return "";

  const ohio = national.states.find((s) => s.name === "Ohio");
  if (!ohio) return "";

  const share = (part: number) => part / ohio.total_revenue;
  const perPupil = ohio.current_spending * 1_000 / ohio.enrollment;

  // Ordered by local share so Ohio's position is visible rather than asserted, and trimmed to the
  // extremes plus Ohio: fifty-one bars is a table, not a chart.
  const ranked = [...national.states].sort((a, b) => b.local_revenue / b.total_revenue - a.local_revenue / a.total_revenue);
  const shown = [...ranked.slice(0, 6), ...(ranked.slice(0, 6).some((s) => s.name === "Ohio") ? [] : [ohio])];

  const bars: Bar[] = shown.map((s) => ({
    label: s.name === "Ohio" ? "Ohio" : s.name,
    value: (s.local_revenue / s.total_revenue) * 100,
    direct: pct(s.local_revenue / s.total_revenue, 0),
    hover: `${s.name}: ${pct(s.local_revenue / s.total_revenue, 1)} local, ${pct(s.state_revenue / s.total_revenue, 1)} state`,
    current: s.name === "Ohio",
  }));

  const rows: [string, string, string, string][] = [
    [
      "Local share of school revenue",
      pct(share(ohio.local_revenue), 1),
      `${national.ohio_local_rank} of ${national.states.length}`,
      pct(national.national_local_share, 1),
    ],
    [
      "State share",
      pct(share(ohio.state_revenue), 1),
      `${national.ohio_state_rank} of ${national.states.length}`,
      pct(national.national_state_share, 1),
    ],
    [
      "Federal share",
      pct(share(ohio.federal_revenue), 1),
      "—",
      pct(
        national.states.reduce((a, s) => a + s.federal_revenue, 0) /
          national.states.reduce((a, s) => a + s.total_revenue, 0),
        1,
      ),
    ],
    [
      "Current spending per pupil",
      money(perPupil),
      `${national.ohio_spending_rank} of ${national.states.length}`,
      money(national.national_spending_per_pupil),
    ],
  ];

  return `
    <div class="card" id="national" data-part="national">
      <h2>${anchor("national")}Whether Ohio is unusual${yearChip("national")}</h2>
      <p class="note">Everything else on this site is Ohio describing itself. This is the Census
        Bureau counting every school system in the country on one set of definitions, for
        FY${national.fiscal_year} — the only figures here that can say whether what Ohio does is
        normal.</p>

      <div class="scroll"><table>
        <thead><tr><th></th><th class="tnum">Ohio</th><th>Rank</th>
          <th class="tnum">National</th></tr></thead>
        <tbody>${rows
          .map(
            ([label, value, rank, natl], i) => `<tr${i < 2 ? ' class="current"' : ""}>
              <th>${escapeHtml(label)}</th>
              <td class="tnum">${value}</td><td class="n">${rank}</td>
              <td class="tnum n">${natl}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>

      <div class="chartwrap" data-chart="local-share">${renderToString(barSpec(bars), { label: `Local share of school revenue by state, percent of total revenue, the states with the highest share and Ohio, FY${national.fiscal_year}`, description: `${shown.length} of ${national.states.length} states are drawn: those with the highest local share, and Ohio at rank ${national.ohio_local_rank}` })}</div>

      <p class="note"><strong>Ohio spends about what the country spends and raises it differently.
        </strong> Current spending per pupil is ${money(perPupil)} against a national
        ${money(national.national_spending_per_pupil)}, and the federal share is within a point of
        the national figure. What is distinctive is the split between the other two: Ohio is
        <strong>${national.ohio_local_rank}th highest of ${national.states.length}</strong> on the
        local share of school revenue and
        <strong>${national.ohio_state_rank}th</strong> on the state share — only
        ${count(national.states.length - national.ohio_state_rank)} states put in less. That is the
        <a href="${routes.wikiNode("litigation", "derolph-i-1997")}">DeRolph</a> holding, stated as
        a comparison for the first time on this site.</p>

      <p class="note">Among the ${count(national.independent_states)} states whose districts levy
        their own tax rather than being funded by a city or county, Ohio's property tax share of
        school revenue ranks <strong>${national.ohio_property_tax_rank}</strong>. That comparison
        excludes twelve states on purpose: where districts are dependent agencies the survey
        attributes the tax to the parent government, so Massachusetts and Virginia report
        <em>zero</em> school property tax while raising billions from it. Ranking all fifty-one
        would put two of the most property-tax-dependent states at the bottom.</p>

      <p class="note">FY${national.fiscal_year} is the peak year of federal pandemic relief, so
        the federal share here is inflated and the state and local shares are correspondingly
        deflated. That runs <em>against</em> this finding rather than for it — in an ordinary year
        Ohio's local share would be higher. It is also why the ${pct(share(ohio.federal_revenue), 1)}
        federal share of <em>revenue</em> on this row is not the
        <a href="/districts">4.2% federal share of operating spending</a> the report card gives for
        FY2025: different year, different denominator, and the difference between them is the
        relief cliff.</p>
    </div>`;
}
