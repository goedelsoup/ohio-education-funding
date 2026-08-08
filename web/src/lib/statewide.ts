/** The statewide view: the three structural facts, and the one chart that shows the first. */

import type { Bar } from "./chart.ts";
import { barSpec } from "./plot/spec.ts";
import { renderToString } from "./plot/ssr.ts";
import { count, escapeHtml, millions, money, pct } from "./format.ts";
import { realChange, series, type Basis } from "./real.ts";
import * as routes from "./routes.ts";
import type { TaxStatewide } from "./feed.ts";
import type { Bundle, District } from "./types.ts";

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
  const peak = years.reduce((a, b) => (b.ending_cash > a.ending_cash ? b : a));

  const cashChange = latest.ending_cash - first.ending_cash;
  const realAid = realChange(bundle.deflator, actuals, (y) => y.state_aid);
  const nominalAid = actuals[actuals.length - 1]!.state_aid / actuals[0]!.state_aid - 1;

  const bars: Bar[] = years.map((y) => ({
    label: `FY${y.fiscal_year}`,
    value: y.ending_cash,
    hover: `FY${y.fiscal_year}: ${millions(y.ending_cash).replace("+", "")} held, ${millions(y.total_revenue).replace("+", "")} in, ${millions(y.total_expenditure).replace("+", "")} out`,
    ...(y.fiscal_year === peak.fiscal_year || y.fiscal_year === latest.fiscal_year
      ? { direct: millions(y.ending_cash).replace("+", "") }
      : {}),
  }));

  const label = converted
    ? `FY${base} dollars`
    : basis === "real"
      ? "nominal — the index does not cover this panel"
      : "nominal";

  return `
    <div class="card">
      <h2>What districts actually received, spent, and hold — ${escapeHtml(label)}</h2>
      <div class="tiles">
        <div class="tile"><div class="k">Cash held, FY${latest.fiscal_year}</div>
          <div class="v">${millions(latest.ending_cash).replace("+", "")}</div>
          <div class="n">${(latest.ending_cash / latest.total_expenditure).toFixed(2)} years of
            spending at this rate</div></div>
        <div class="tile"><div class="k">Change since FY${first.fiscal_year}</div>
          <div class="v ${cashChange < 0 ? "loss" : "gain"}">${millions(cashChange)}</div>
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

      <div class="chartwrap" data-chart="statewide-cash">${renderToString(barSpec(bars))}</div>
      <p class="note">General fund cash held at 30 June, summed over the
        ${count(bundle.statewide.districts)} districts in this feed.
        ${
          converted
            ? `Deflated with <strong>${bundle.deflator?.label ?? "an index"}</strong>. Nominally
               the balance ends ${pct(actuals[actuals.length - 1]!.ending_cash / actuals[0]!.ending_cash - 1, 0)}
               above FY${first.fiscal_year}; in constant dollars it ends
               ${pct(cashChange / first.ending_cash, 0)}. Both are correct and they support
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
  const withValuation = districts
    .filter((d) => d.valuation_per_pupil != null)
    .sort((a, b) => a.valuation_per_pupil! - b.valuation_per_pupil!);
  const size = Math.floor(withValuation.length / 5);
  return Array.from({ length: 5 }, (_, i) =>
    // The last quintile takes the remainder, so no district is dropped by integer division.
    withValuation.slice(i * size, i === 4 ? withValuation.length : (i + 1) * size),
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
 * The three structural facts, and the one chart that shows the first.
 *
 * Separate from {@link renderStatewideFinances} because that card is rendered twice — once per
 * dollar basis — and this one is not: nothing in it is a dollar series that a price index could
 * restate.
 */
export function renderStatewideStructure(bundle: Bundle, tax: TaxStatewide): string {
  const s = bundle.statewide;
  const bars = guaranteeRateByQuintile(bundle.districts);

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

    <div class="card">
      <h2>Who is on the guarantee</h2>
      <p class="note">Districts grouped into fifths by assessed valuation per pupil, poorest on
        the left. The guarantee was written as transitional relief for districts losing
        students; the pattern it actually produces is a wealth gradient.</p>
      <div class="chartwrap" data-chart="quintiles">${renderToString(barSpec(bars, { max: 1 }))}</div>
      <p class="note">Median valuation per pupil statewide is
        ${money(s.median_valuation_per_pupil)}.</p>
    </div>

    <div class="card">
      <h2>Does state aid offset property wealth?</h2>
      <div class="scroll"><table><tbody>
        <tr><th>Aid vs. wealth — formula only</th>
            <td class="tnum">${s.wealth_neutrality_formula.toFixed(3)}</td></tr>
        <tr><th>Aid vs. wealth — as received</th>
            <td class="tnum">${s.wealth_neutrality_realized.toFixed(3)}</td></tr>
      </tbody></table></div>
      <p class="note">Correlation between assessed valuation per pupil and state aid per pupil.
        A perfectly compensating formula would be strongly negative. The formula alone reaches
        ${s.wealth_neutrality_formula.toFixed(3)}; what districts actually receive is
        ${s.wealth_neutrality_realized.toFixed(3)}. <strong>The guarantee gives up part of the
        formula's equalization</strong>, because the districts it tops up are the ones the local
        capacity measure funds least.</p>
    </div>

    <div class="card">
      <h2>Two floors</h2>
      <div class="scroll"><table><tbody>
        <tr><th>At the <a href="${routes.parameter("twenty-mill-floor")}">20-mill floor</a></th>
            <td>${s.at_millage_floor} districts (${pct(s.at_millage_floor / s.districts, 0)})</td></tr>
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
        statute.</strong> Across TY2023 and TY2024,
        ${count(tax.rateFell.aboveFloor)} of the ${count(tax.districts.aboveFloor)} districts above
        the floor saw their effective Class I rate fall as reduction factors rolled it back; of the
        ${count(tax.districts.atFloor)} at the floor, ${count(tax.rateFell.atFloor)} did.
        ${pct(tax.reductionsAboveFloor, 1)} of every effective-rate reduction in Ohio happens above
        the floor, which is what it means to say reduction factors stop there — and why a
        reappraisal reaches one district's revenue and not another's.</p>
      <p class="note">Real property tax charged for current expenses is a median
        <strong>${pct(tax.medianChargeShare, 0)}</strong> of what a district spends on operations,
        and ${count(tax.chargedMoreThanSpent.length)} districts are charged more than they spend.
        Each district's own figures are on its
        <a href="/districts">property tax page</a>, from the Department of Taxation's Table SD-1.</p>
    </div>`;
}
