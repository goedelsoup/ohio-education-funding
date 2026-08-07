/** The district view: what the formula computes for one district, and what it actually gets. */

import { fanChart, type FanPoint } from "./chart.ts";
import { count, escapeHtml, money, ordinal, pct, percentileOf, signedMoney } from "./format.ts";
import { renderDistrictOutcome } from "./outcomes.ts";
import { apply, currentLaw } from "./policy.ts";
import { forecastPath, growthPrior, observations } from "./project.ts";
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

/**
 * What a year of enrollment is worth to this district.
 *
 * # What this is not
 *
 * It is **not** the district's actual FY2026 state aid against its actual FY2027 state aid. The
 * department publishes one funding calculator at a time and replaces it rather than archiving
 * it, so this repository has the FY2027 model and no FY2026 one; there is no retrieved source
 * anywhere in it carrying a district's FY2026 foundation payment. A card that put a number under
 * "FY2026" would be inventing it.
 *
 * What it is instead is exact and, for the question people actually ask, more useful: the
 * **FY2027 formula held completely fixed and run at each year's enrolled ADM**. Every other
 * input — base cost, local capacity, the categoricals, the guarantee baseline — is identical
 * across the rows, so the difference between them is the enrollment channel and nothing else.
 * Comparing two years of published totals could not have said that, because the formula changed
 * between them too.
 */
function renderEnrollmentYears(bundle: Bundle, d: District): string {
  const meta = bundle.projection;
  if (!meta) return "";
  const model = bundle.statewide.minimum_state_share;
  const law = currentLaw(model);
  const history = observations(d, meta.base_year);

  const rows = history.map((o) => ({
    year: o.fiscalYear,
    adm: o.value,
    aid: apply(d, law, o.value, model).realizedAid,
  }));
  const latest = rows[rows.length - 1]!;
  const prior = rows[rows.length - 2]!;
  const change = latest.aid - prior.aid;
  const admChange = latest.adm - prior.adm;

  // The district's own band, from the same verified path the statewide fan uses — one district
  // instead of 606. For a district on the guarantee it collapses to a line, which is the point.
  const path = forecastPath(
    [d],
    law,
    meta.base_year + 6,
    meta.base_year,
    meta.method,
    meta.damping,
    growthPrior(bundle.districts, meta.z),
    model,
  );
  const end = path[path.length - 1]!;
  const insensitive = end.high - end.low < Math.max(1, end.realizedAid * 0.0005);
  const points: FanPoint[] = path.map((p) => ({
    year: p.fiscalYear,
    point: p.realizedAid,
    low: p.low,
    high: p.high,
    observed: p.observed,
  }));

  return `
    <div class="card">
      <h2>What a year of enrollment is worth here</h2>
      <div class="scroll"><table>
        <thead><tr>
          <th>Enrollment year</th><th>Enrolled ADM</th><th>State aid at that ADM</th>
        </tr></thead>
        <tbody>${rows
          .map(
            (r) => `<tr${r.year === meta.base_year ? ' class="current"' : ""}>
              <th>FY${r.year}${r.year === meta.base_year ? " — the model's own" : ""}</th>
              <td class="tnum">${count(Math.round(r.adm))}</td>
              <td class="tnum">${money(r.aid)}</td>
            </tr>`,
          )
          .join("")}</tbody>
      </table></div>
      <p class="note">Moving from FY${prior.year} to FY${latest.year} enrollment
        ${admChange < 0 ? "cost" : "gained"} this district
        ${count(Math.abs(Math.round(admChange)))} pupils and
        <strong class="${change < 0 ? "loss" : "gain"}">${signedMoney(change)}</strong>
        ${
          Math.abs(change) < 0.5 && d.on_guarantee
            ? `— nothing, because the guarantee holds its aid at a fixed dollar amount that
               enrollment does not enter.`
            : `, or ${signedMoney(latest.adm > 0 ? change / latest.adm : 0, 2)} per pupil.`
        }</p>
      <p class="note">These are <strong>not</strong> published FY${prior.year} and
        FY${latest.year} funding totals — the department publishes one calculator at a time and
        this repository holds the FY${bundle.fiscal_year} one, so no FY${prior.year} payment
        figure exists here to show. Every row is the FY${bundle.fiscal_year} formula held fixed
        and run at that year's enrolled ADM, which isolates the enrollment channel: two years of
        published totals could not, because the formula moved between them too.</p>

      <h3>Carried forward</h3>
      <div class="chartwrap" data-chart="district-fan">${fanChart(
        points,
        (v) => money(v),
        (p) =>
          p.observed
            ? `FY${p.year}: ${money(p.point)} at published enrollment — exact`
            : `FY${p.year}: ${money(p.low)} – ${money(p.high)}, central ${money(p.point)}`,
      )}</div>
      <p class="note">${
        insensitive
          ? `The band is flat. This district's state aid does not respond to its enrollment at
             all under current law, because the guarantee pays it a fixed amount and the
             formula — which does respond — is not what it is paid on.`
          : `The range, not the line, is the finding: at FY${end.fiscalYear} enrollment this
             district's aid is somewhere between ${money(end.low)} and ${money(end.high)}. The
             band is the cross-sectional spread of district enrollment growth, not this
             district's own history — three observations cannot give that.`
      }</p>
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

    ${renderEnrollmentYears(bundle, d)}

    <div class="card">
      <h2>Position among Ohio's ${bundle.statewide.districts} districts</h2>
      ${strip("Assessed valuation per pupil", d.valuation_per_pupil, valuations)}
      ${strip("Operating expenditure per pupil", d.operating_expenditure_per_pupil, expenditures)}
    </div>

    ${renderDistrictOutcome(d)}

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
