/** The statewide view: the three structural facts, and the one chart that shows the first. */
import { barChart } from "./chart.js";
import { count, millions, money, pct } from "./format.js";
/** Quintiles of assessed valuation per pupil, poorest first. */
export function wealthQuintiles(districts) {
    const withValuation = districts
        .filter((d) => d.valuation_per_pupil != null)
        .sort((a, b) => a.valuation_per_pupil - b.valuation_per_pupil);
    const size = Math.floor(withValuation.length / 5);
    return Array.from({ length: 5 }, (_, i) => 
    // The last quintile takes the remainder, so no district is dropped by integer division.
    withValuation.slice(i * size, i === 4 ? withValuation.length : (i + 1) * size));
}
/** Share of each quintile funded by the guarantee. */
export function guaranteeRateByQuintile(districts) {
    return wealthQuintiles(districts).map((group, index) => {
        const onGuarantee = group.filter((d) => d.on_guarantee).length;
        const share = group.length === 0 ? 0 : onGuarantee / group.length;
        const label = ["Poorest fifth", "Second", "Third", "Fourth", "Wealthiest fifth"][index];
        return {
            label,
            value: share,
            direct: pct(share, 0),
            hover: `${label}: ${onGuarantee} of ${group.length} districts on the guarantee, median valuation ${money(group[Math.floor(group.length / 2)]?.valuation_per_pupil ?? 0)} per pupil`,
        };
    });
}
/** Render the statewide view. */
export function renderStatewide(bundle) {
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
      <div class="chartwrap" data-chart="quintiles">${barChart(bars, { max: 1 })}</div>
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
        <tr><th>At the 20-mill floor</th>
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
        ${pct(s.minimum_state_share, 0)} in this model, not the 5% the Fair School Funding Plan
        was enacted with — each biennial budget sets it.</p>
    </div>`;
}
