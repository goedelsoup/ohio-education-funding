/** The district view: what the formula computes for one district, and what it actually gets. */

import { escapeHtml, money, ordinal, pct, percentileOf } from "./format.ts";
import type { Bundle, District } from "./types.ts";

function strip(label: string, value: number | null, sorted: number[]): string {
  if (value == null) return `<p class="note">${escapeHtml(label)}: not reported.</p>`;
  const p = percentileOf(sorted, value);
  return `
    <div class="strip-row">
      <div class="strip-head">
        <span>${escapeHtml(label)}</span>
        <strong class="tnum">${money(value)}</strong>
      </div>
      <div class="strip"><div class="marker" style="left:calc(${(p * 100).toFixed(1)}% - 1.5px)"></div></div>
      <div class="scale">
        <span>${money(sorted[0] ?? 0)}</span>
        <span>${ordinal(Math.round(p * 100))} percentile</span>
        <span>${money(sorted[sorted.length - 1] ?? 0)}</span>
      </div>
    </div>`;
}

/** Render one district. */
export function renderDistrict(bundle: Bundle, d: District): string {
  const valuations = bundle.districts
    .map((x) => x.valuation_per_pupil)
    .filter((v): v is number => v != null)
    .sort((a, b) => a - b);
  const expenditures = bundle.districts
    .map((x) => x.operating_expenditure_per_pupil)
    .filter((v): v is number => v != null)
    .sort((a, b) => a - b);

  const formulaPP = d.formula_aid_per_pupil;
  const realizedPP = d.realized_aid_per_pupil;
  const guaranteePP = realizedPP - formulaPP;
  const total = Math.max(realizedPP, 1);
  const formulaWidth = (formulaPP / total) * 100;
  const guaranteeWidth = (guaranteePP / total) * 100;

  const flags: string[] = [
    d.on_guarantee ? "Funded by the guarantee, not the formula" : "On formula",
  ];
  if (d.at_millage_floor) flags.push("At the 20-mill floor");
  if (d.at_minimum_state_share) flags.push("At the minimum state share");
  if (d.enrollment_change != null && d.enrollment_change < 0) {
    flags.push(`Enrollment down ${pct(-d.enrollment_change)} FY2024→FY2026`);
  }

  return `
    <div class="tiles">
      <div class="tile"><div class="k">Base cost / pupil</div>
        <div class="v">${money(d.base_cost_per_pupil)}</div>
        <div class="n">what the plan says it costs</div></div>
      <div class="tile"><div class="k">State aid / pupil</div>
        <div class="v">${money(realizedPP)}</div>
        <div class="n">${
          d.on_guarantee
            ? money(guaranteePP) + " of it from the guarantee"
            : "all from the formula"
        }</div></div>
      <div class="tile"><div class="k">Enrolled ADM</div>
        <div class="v">${d.adm.toLocaleString("en-US", { maximumFractionDigits: 0 })}</div>
        <div class="n">base cost enrolled</div></div>
    </div>

    <div class="card">
      <h2>Where the state aid comes from</h2>
      <div class="barwrap">
        <div class="bar" role="img" aria-label="Formula aid ${money(formulaPP)} per pupil, guarantee ${money(guaranteePP)} per pupil">
          <div class="seg formula ${guaranteeWidth <= 0 ? "only" : ""}" style="width:${formulaWidth}%"></div>
          ${guaranteeWidth > 0 ? `<div class="seg guarantee" style="width:${guaranteeWidth}%"></div>` : ""}
        </div>
        <div class="legend">
          <span><i class="sw formula"></i> Formula ${money(formulaPP)}/pupil</span>
          ${guaranteeWidth > 0 ? `<span><i class="sw guarantee"></i> Guarantee ${money(guaranteePP)}/pupil</span>` : ""}
        </div>
      </div>
      <p class="note">${
        d.on_guarantee
          ? `The formula computes ${money(formulaPP)} per pupil. This district receives
             ${money(realizedPP)} because the temporary transitional aid guarantee holds it at
             what it received in <strong>FY2020</strong> — a year Ohio froze funding under the
             Bridge formula rather than computing it. The formula produces
             <strong>${pct(formulaPP / realizedPP, 0)}</strong> of that level.`
          : `This district is funded by the formula, so an increase in its computed base cost
             reaches it in full — unlike the ${bundle.statewide.on_guarantee} districts held on
             the guarantee.`
      }</p>
      <div class="flags">${flags
        .map((f) => `<span class="flag">${escapeHtml(f)}</span>`)
        .join("")}</div>
    </div>

    <div class="card">
      <h2>Position among Ohio's ${bundle.statewide.districts} districts</h2>
      ${strip("Assessed valuation per pupil", d.valuation_per_pupil, valuations)}
      ${strip("Operating expenditure per pupil", d.operating_expenditure_per_pupil, expenditures)}
    </div>

    <div class="card">
      <h2>Detail</h2>
      <div class="scroll"><table><tbody>
        <tr><th>Base cost per pupil</th><td>${money(d.base_cost_per_pupil, 2)}</td></tr>
        <tr><th>Aggregate base cost</th><td>${money(d.aggregate_base_cost)}</td></tr>
        <tr><th>State share of base cost</th><td>${money(d.base_cost_state_share)}</td></tr>
        <tr><th>Categorical funding</th><td>${money(d.categorical_funding)}</td></tr>
        <tr><th>Formula aid per pupil</th><td>${money(formulaPP, 2)}</td></tr>
        <tr><th>Guarantee per pupil</th><td>${money(guaranteePP, 2)}</td></tr>
        <tr><th>Guarantee, total</th><td>${money(d.guarantee)}</td></tr>
        <tr><th>Assessed valuation per pupil</th><td>${money(d.valuation_per_pupil)}</td></tr>
        <tr><th>Effective Class 1 millage</th><td>${
          d.effective_class1_millage == null ? "—" : d.effective_class1_millage.toFixed(2)
        }</td></tr>
        <tr><th>Operating expenditure per pupil</th><td>${money(d.operating_expenditure_per_pupil)}</td></tr>
        <tr><th>Economically disadvantaged</th><td>${pct(d.economically_disadvantaged)}</td></tr>
        <tr><th>Enrolled ADM FY2026</th><td>${d.current_year_adm.toLocaleString("en-US", { maximumFractionDigits: 0 })}</td></tr>
        <tr><th>Enrollment change FY2024→FY2026</th><td>${pct(d.enrollment_change)}</td></tr>
      </tbody></table></div>
      <p class="note">FY2026 enrolled ADM is partly a departmental estimate: the calculator is
        published before that fiscal year closes.</p>
    </div>`;
}
