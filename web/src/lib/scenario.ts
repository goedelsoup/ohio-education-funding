/**
 * The scenario builder: move a lever, see who it reaches.
 *
 * Everything here runs against the real 609-district panel in the browser, using the formula in
 * `policy.ts` — which does not get to run at all until it has reproduced the Rust-computed
 * checkpoints in the feed. See `verify.ts`.
 */

import { bin, divergingHistogram, fanChart, type FanPoint } from "./chart.ts";
import { count, escapeHtml, millions, money, pct, signedMoney } from "./format.ts";
import { applyAll, currentLaw, totals, type Outcome, type Policy } from "./policy.ts";
import { forecastPath, growthPrior } from "./project.ts";
import type { Bundle } from "./types.ts";

/**
 * The default horizon, in years past the last observation.
 *
 * Six, which lands on FY2032 — one of the years the feed carries a checkpoint for, so the first
 * thing a reader sees is a year the page has proved it computes correctly.
 */
export const DEFAULT_HORIZON_YEARS = 6;

/** The state the controls hold. */
export interface Levers {
  guarantee: "as-enacted" | "removed" | "rebase" | "phase-out";
  guaranteeArgument: number;
  baseCostScale: number;
  minimumStateShare: number;
  phaseInBaseCost: number;
  phaseInCategorical: number;
  /**
   * The fiscal year to project enrollment to. Equal to the base year means "do not project".
   *
   * Not a policy lever, and deliberately not one: it changes what is being *asked*, not what
   * the state would do. {@link isCurrentLaw} ignores it for that reason.
   */
  horizon: number;
}

/** The levers at their current-law positions. */
export function defaultLevers(
  modelMinimumStateShare: number,
  baseYear = 2026,
): Levers {
  return {
    guarantee: "as-enacted",
    guaranteeArgument: 0.5,
    baseCostScale: 1,
    minimumStateShare: modelMinimumStateShare,
    phaseInBaseCost: 1,
    phaseInCategorical: 1,
    horizon: baseYear + DEFAULT_HORIZON_YEARS,
  };
}

/** Turn the controls into a policy. */
export function toPolicy(levers: Levers): Policy {
  return {
    guarantee:
      levers.guarantee === "rebase"
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

function isCurrentLaw(levers: Levers, model: number): boolean {
  const base = defaultLevers(model);
  return (
    levers.guarantee === base.guarantee &&
    levers.baseCostScale === base.baseCostScale &&
    levers.minimumStateShare === base.minimumStateShare &&
    levers.phaseInBaseCost === base.phaseInBaseCost &&
    levers.phaseInCategorical === base.phaseInCategorical
  );
}

/** The controls. Rendered once; their values are read on every change. */
export function renderControls(model: number, projection: ProjectionControl): string {
  const slider = (
    id: string,
    label: string,
    min: number,
    max: number,
    step: number,
    value: number,
    note: string,
  ) => `
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
      ${
        projection.available
          ? slider(
              "lv-horizon",
              "Project enrollment to",
              projection.baseYear,
              projection.maxYear,
              1,
              projection.baseYear + DEFAULT_HORIZON_YEARS,
              `FY${projection.baseYear} is the last observed year — leave it there for no forecast.`,
            )
          : ""
      }
    </div>`;
}

/** What the horizon control needs to know about the feed it is offering years from. */
export interface ProjectionControl {
  available: boolean;
  baseYear: number;
  maxYear: number;
}

function affectedTable(outcomes: Outcome[]): string {
  const moved = outcomes
    .filter((o) => Math.abs(o.delta) > 0.5)
    .sort((a, b) => Math.abs(b.deltaPerPupil) - Math.abs(a.deltaPerPupil))
    .slice(0, 12);
  if (moved.length === 0) {
    return `<p class="note">No district's funding changes under these settings.</p>`;
  }
  const rows = moved
    .map(
      (o) => `<tr>
        <th>${escapeHtml(o.name)}</th>
        <td class="tnum ${o.delta > 0 ? "gain" : "loss"}">${signedMoney(o.deltaPerPupil)}</td>
        <td class="tnum ${o.delta > 0 ? "gain" : "loss"}">${millions(o.delta)}</td>
      </tr>`,
    )
    .join("");
  return `<div class="scroll"><table>
    <thead><tr><th>District</th><th>Per pupil</th><th>Total</th></tr></thead>
    <tbody>${rows}</tbody></table></div>`;
}

/** A dollar range, written so neither end reads as the subordinate one. */
function range(low: number, high: number): string {
  return `${millions(low).replace("+", "")} – ${millions(high).replace("+", "")}`;
}

/**
 * The forecast half: the same policy at projected enrollment.
 *
 * # Why the range is the headline and the point is the footnote
 *
 * Everywhere else on this page the big number is a point estimate. Here it is a range, and that
 * inversion is the whole design. A forecast rendered as "$7.13B (±3.4%)" is read as $7.13B; the
 * interval becomes a disclaimer that the eye skips. Rendered as "$6.89B – $7.38B" it cannot be,
 * because there is no single number to take away.
 *
 * The same rule sets the chart: the band is a mark rather than shading, its centre line is
 * dashed, and only the two bounds are direct-labelled. See {@link fanChart}.
 *
 * This card is always **below** the simulation, and there is deliberately no figure anywhere
 * that adds the two together — a combined number would inherit the forecast's error while
 * wearing the simulation's precision.
 */
export function renderProjection(bundle: Bundle, levers: Levers): string {
  const meta = bundle.projection;
  if (!meta) return "";
  const model = bundle.statewide.minimum_state_share;
  if (levers.horizon <= meta.base_year) {
    return `<div class="card">
      <h2>At projected enrollment</h2>
      <p class="note">Not projected. Move <em>Project enrollment to</em> past
        FY${meta.base_year} to carry every district's enrolled ADM forward and re-run these
        levers against it.</p>
    </div>`;
  }

  const prior = growthPrior(bundle.districts, meta.z);
  const path = forecastPath(
    bundle.districts,
    toPolicy(levers),
    levers.horizon,
    meta.base_year,
    meta.method,
    meta.damping,
    prior,
    model,
  );
  const end = path[path.length - 1]!;
  const start = path[0]!;
  const width = (end.high - end.low) / (2 * end.realizedAid);

  // The same horizon under current law, so the reader can see what the guarantee is doing to
  // the *uncertainty* rather than only to the level. This is the corpus finding that motivated
  // the whole axis, and it is invisible unless both widths are on screen at once.
  const reference = forecastPath(
    bundle.districts,
    currentLaw(model),
    levers.horizon,
    meta.base_year,
    meta.method,
    meta.damping,
    prior,
    model,
  );
  const referenceEnd = reference[reference.length - 1]!;
  const referenceWidth =
    (referenceEnd.high - referenceEnd.low) / (2 * referenceEnd.realizedAid);
  const moved = Math.abs(width - referenceWidth) > 0.0005;

  const points: FanPoint[] = path.map((p) => ({
    year: p.fiscalYear,
    point: p.realizedAid,
    low: p.low,
    high: p.high,
    observed: p.observed,
  }));

  return `
    <div class="card">
      <h2>At projected enrollment</h2>
      <div class="tiles">
        <div class="tile wide"><div class="k">Total state aid, FY${end.fiscalYear}</div>
          <div class="v range">${range(end.low, end.high)}</div>
          <div class="n">Central estimate ${millions(end.realizedAid).replace("+", "")}.
            One path through the band, not the answer.</div></div>
        <div class="tile"><div class="k">Band half-width</div>
          <div class="v">±${pct(width, 1)}</div>
          <div class="n">${
            moved
              ? `current law at the same horizon is ±${pct(referenceWidth, 1)}`
              : "the same as current law at this horizon"
          }</div></div>
        <div class="tile"><div class="k">Projected enrolled ADM</div>
          <div class="v">${count(Math.round(end.adm))}</div>
          <div class="n">from ${count(Math.round(start.adm))} in FY${start.fiscalYear}</div></div>
      </div>

      <div class="chartwrap" data-chart="fan">${fanChart(
        points,
        (v) => millions(v).replace("+", ""),
        (p) =>
          p.observed
            ? `FY${p.year}: ${millions(p.point).replace("+", "")} at published enrollment — exact`
            : `FY${p.year}: ${range(p.low, p.high)}, central ${millions(p.point).replace("+", "")}`,
      )}</div>
      <div class="legend">
        <span><i class="sw band"></i> Range of total state aid</span>
        <span><i class="sw dash"></i> Central estimate</span>
        <span><i class="sw anchor"></i> Last observed year, no forecast</span>
      </div>

      <p class="note">This is a <strong>forecast</strong>, and the card above it is not. The
        levers are held fixed and only enrollment moves: every district's enrolled ADM is carried
        forward from FY${meta.base_year} by a damped trend, and the FY${bundle.fiscal_year}
        formula is re-run at it. Nothing here projects the formula, the appropriation, or
        assessed valuation.</p>
      <p class="note">The band is not this district's — or this state's — own historical
        variability. Three observations cannot give that. It is the ${escapeHtml(
          meta.prior_source,
        )}, σ = ${pct(prior.sigma, 2)} a year, widened by the square root of the horizon because
        growth errors compound. Both ends are computed by re-running the whole formula at that
        enrollment, not by scaling the middle — the guarantee is a
        <code>max</code>, and a district can be on formula at one end of the band and on the
        guarantee at the other.</p>
    </div>`;
}

/** Run the levers and render the result. */
export function renderScenario(bundle: Bundle, levers: Levers): string {
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
      ${
        deltas.length > 0
          ? `<div class="chartwrap" data-chart="deltas">${divergingHistogram(
              bin(deltas, 24),
              (v) => signedMoney(v),
            )}</div>
        <div class="legend">
          <span><i class="sw loss"></i> Aid falls</span>
          <span><i class="sw gain"></i> Aid rises</span>
        </div>
        <p class="note">Districts by change in state aid per pupil. Bars are counts, not
          dollars — a tall bar near zero is many districts barely affected.</p>`
          : `<p class="note">No district's funding changes under these settings.</p>`
      }
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
