/**
 * The scenario builder: move a lever, see who it reaches.
 *
 * Everything here runs against the real 609-district panel in the browser, using the formula in
 * `policy.ts` — which does not get to run at all until it has reproduced the Rust-computed
 * checkpoints in the feed. See `verify.ts`.
 */
import { bin, divergingHistogram } from "./chart.js";
import { count, escapeHtml, millions, money, pct, signedMoney } from "./format.js";
import { applyAll, currentLaw, totals } from "./policy.js";
/** The levers at their current-law positions. */
export function defaultLevers(modelMinimumStateShare) {
    return {
        guarantee: "as-enacted",
        guaranteeArgument: 0.5,
        baseCostScale: 1,
        minimumStateShare: modelMinimumStateShare,
        phaseInBaseCost: 1,
        phaseInCategorical: 1,
    };
}
/** Turn the controls into a policy. */
export function toPolicy(levers) {
    return {
        guarantee: levers.guarantee === "rebase"
            ? { kind: "rebase", factor: levers.guaranteeArgument }
            : levers.guarantee === "phase-out"
                ? { kind: "phase-out", remaining: levers.guaranteeArgument }
                : { kind: levers.guarantee },
        baseCostScale: levers.baseCostScale,
        minimumStateShare: levers.minimumStateShare,
        phaseInBaseCost: levers.phaseInBaseCost,
        phaseInCategorical: levers.phaseInCategorical,
    };
}
function isCurrentLaw(levers, model) {
    const base = defaultLevers(model);
    return (levers.guarantee === base.guarantee &&
        levers.baseCostScale === base.baseCostScale &&
        levers.minimumStateShare === base.minimumStateShare &&
        levers.phaseInBaseCost === base.phaseInBaseCost &&
        levers.phaseInCategorical === base.phaseInCategorical);
}
/** The controls. Rendered once; their values are read on every change. */
export function renderControls(model) {
    const slider = (id, label, min, max, step, value, note) => `
    <div class="lever">
      <label for="${id}">${escapeHtml(label)} <output id="${id}-out"></output></label>
      <input type="range" id="${id}" min="${min}" max="${max}" step="${step}" value="${value}">
      <div class="n">${note}</div>
    </div>`;
    return `
    <div class="levers">
      <div class="lever">
        <label for="lv-guarantee">Temporary transitional aid guarantee</label>
        <select id="lv-guarantee">
          <option value="as-enacted">As enacted — hold at FY2020</option>
          <option value="phase-out">Phase out</option>
          <option value="rebase">Re-base the FY2020 floor</option>
          <option value="removed">Remove entirely</option>
        </select>
        <div class="n">294 districts are held above what the formula computes for them.</div>
      </div>
      ${slider("lv-arg", "Guarantee retained", 0, 1, 0.05, 0.5, "Applies to phase-out and re-base.")}
      ${slider("lv-base", "Base cost", 0.8, 1.3, 0.01, 1, "A refresh of the FY2022 cost inputs is roughly +3%.")}
      ${slider("lv-min", "Minimum state share", 0.05, 0.3, 0.01, model, `Current model: ${pct(model, 0)}. The plan was enacted with 5%.`)}
      ${slider("lv-phase", "Phase-in, base cost", 0, 1, 0.05, 1, "Fraction of computed base cost aid appropriated.")}
      ${slider("lv-phase-cat", "Phase-in, categoricals", 0, 1, 0.05, 1, "Separate because Ohio's were: DPIA was phased in at 0% while the headline was 16.67%.")}
    </div>`;
}
function affectedTable(outcomes) {
    const moved = outcomes
        .filter((o) => Math.abs(o.delta) > 0.5)
        .sort((a, b) => Math.abs(b.deltaPerPupil) - Math.abs(a.deltaPerPupil))
        .slice(0, 12);
    if (moved.length === 0) {
        return `<p class="note">No district's funding changes under these settings.</p>`;
    }
    const rows = moved
        .map((o) => `<tr>
        <th>${escapeHtml(o.name)}</th>
        <td class="tnum ${o.delta > 0 ? "gain" : "loss"}">${signedMoney(o.deltaPerPupil)}</td>
        <td class="tnum ${o.delta > 0 ? "gain" : "loss"}">${millions(o.delta)}</td>
      </tr>`)
        .join("");
    return `<div class="scroll"><table>
    <thead><tr><th>District</th><th>Per pupil</th><th>Total</th></tr></thead>
    <tbody>${rows}</tbody></table></div>`;
}
/** Run the levers and render the result. */
export function renderScenario(bundle, levers) {
    const model = bundle.statewide.minimum_state_share;
    if (isCurrentLaw(levers, model)) {
        return `<div class="card">
      <h2>Current law</h2>
      <p class="note">These are the settings the department's own FY${bundle.fiscal_year} model
        uses, so nothing moves. Total state foundation aid is
        ${millions(bundle.statewide.realized_aid_total).replace("+", "")} across
        ${count(bundle.statewide.districts)} districts, of which
        ${millions(bundle.statewide.guarantee_total).replace("+", "")} is the guarantee.
        Move a lever.</p>
    </div>`;
    }
    const outcomes = applyAll(bundle.districts, toPolicy(levers), model);
    const t = totals(outcomes);
    const deltas = outcomes
        .filter((o) => Math.abs(o.delta) > 0.5)
        .map((o) => o.deltaPerPupil);
    return `
    <div class="tiles">
      <div class="tile"><div class="k">State aid</div>
        <div class="v ${t.cost > 0 ? "gain" : t.cost < 0 ? "loss" : ""}">${millions(t.cost)}</div>
        <div class="n">against ${millions(bundle.statewide.realized_aid_total).replace("+", "")}
          under current law</div></div>
      <div class="tile"><div class="k">Districts reached</div>
        <div class="v">${t.gainers + t.losers}</div>
        <div class="n">${t.gainers} up, ${t.losers} down</div></div>
      <div class="tile"><div class="k">Unmoved</div>
        <div class="v">${t.unmoved}</div>
        <div class="n">${pct(t.unmoved / t.districts, 0)} of districts</div></div>
    </div>

    <div class="card">
      <h2>How the change is distributed</h2>
      ${deltas.length > 0
        ? `<div class="chartwrap" data-chart="deltas">${divergingHistogram(bin(deltas, 24), (v) => signedMoney(v))}</div>
        <div class="legend">
          <span><i class="sw loss"></i> Aid falls</span>
          <span><i class="sw gain"></i> Aid rises</span>
        </div>
        <p class="note">Districts by change in state aid per pupil. Bars are counts, not
          dollars — a tall bar near zero is many districts barely affected.</p>`
        : `<p class="note">No district's funding changes under these settings.</p>`}
    </div>

    <div class="card">
      <h2>Most affected</h2>
      ${affectedTable(outcomes)}
    </div>

    <div class="card">
      <h2>What moved underneath</h2>
      <div class="scroll"><table><tbody>
        <tr><th>On the guarantee</th>
            <td>${bundle.statewide.on_guarantee} → ${t.onGuarantee}</td></tr>
        <tr><th>At the minimum state share</th>
            <td>${bundle.statewide.at_minimum_state_share} → ${t.atMinimumStateShare}</td></tr>
        <tr><th>Guarantee, total</th>
            <td>${money(bundle.statewide.guarantee_total)} → ${money(t.guarantee)}</td></tr>
        <tr><th>Formula aid, total</th><td>${money(t.formulaAid)}</td></tr>
        <tr><th>Realized aid, total</th><td>${money(t.realizedAid)}</td></tr>
      </tbody></table></div>
      <p class="note">This is a <strong>simulation</strong>, not a forecast: it re-runs the
        department's FY${bundle.fiscal_year} model with the levers moved, at published
        enrollment. It does not project anything, and it holds assessed valuation fixed —
        which the corpus cannot project from one observation per district. Local capacity is
        60% valuation, so a scenario where property values move is out of reach here.</p>
    </div>`;
}
