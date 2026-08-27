/**
 * The scenario builder: move a lever, see who it reaches.
 *
 * Everything here runs against the real 609-district panel in the browser, using the formula in
 * `policy.ts` — which does not get to run at all until it has reproduced the Rust-computed
 * checkpoints in the feed. See `verify.ts`.
 */

import { bin, type FanPoint } from "./chart.ts";
import { renderToString } from "./plot/client.ts";
import { fanSpec, histogramSpec } from "./plot/spec.ts";
import { count, escapeHtml, millions, money, pct, signedMoney } from "./format.ts";
import {
  applyAll,
  currentFormulaAid,
  currentLaw,
  totals,
  type Outcome,
  type Policy,
} from "./policy.ts";
import { forecastPath, growthPrior, statuteNote } from "./project.ts";
import type { Draft, Panel } from "./types.ts";
import * as routes from "./routes.ts";
import { anchor } from "./section.ts";

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
  phaseInGeneral: number;
  phaseInDpia: number;
  /**
   * The fiscal year to project enrollment to. Equal to the base year means "do not project".
   *
   * Not a policy lever, and deliberately not one: it changes what is being *asked*, not what
   * the state would do. {@link isCurrentLaw} ignores it for that reason.
   */
  horizon: number;
}

/**
 * What each lever will accept.
 *
 * # Why this is here rather than in the control that renders it
 *
 * The bounds used to live only in `ScenarioControls.astro`'s `SLIDERS`, as `min`/`max`/`step` on
 * five range inputs, and that made the *DOM* the validator: `readLevers` reads `input.value`, which
 * a browser has already clamped, so a hostile `?base=100` was corrected on its way through a
 * control rather than on its way in.
 *
 * That held for as long as every path read the controls. It stopped holding when `?draft=` gained
 * the ability to render before the first read of them — a draft's lever values are not on the
 * slider's step grid, so the draft path renders from the draft and skips `readLevers` deliberately.
 * `?draft=x&h=999999` then reached `forecastPath` with a million-year horizon and locked the tab.
 *
 * So the bounds are stated once, here, and both the control and the query string are held to them.
 * See {@link clampLevers}.
 */
export const LEVER_BOUNDS = {
  guaranteeArgument: { min: 0, max: 1, step: 0.05 },
  baseCostScale: { min: 0.8, max: 1.3, step: 0.01 },
  minimumStateShare: { min: 0.05, max: 0.3, step: 0.01 },
  phaseInGeneral: { min: 0, max: 1, step: 0.05 },
  phaseInDpia: { min: 0, max: 1, step: 0.05 },
} as const;

/** The two ends of the horizon, which are a property of the feed rather than of the control. */
export interface HorizonBound {
  /** The last observed year. Equal to it means "do not project". */
  base: number;
  /** The furthest year the feed carries a checkpoint for. */
  max: number;
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
    phaseInGeneral: 1,
    phaseInDpia: 1,
    horizon: baseYear + DEFAULT_HORIZON_YEARS,
  };
}

/**
 * Hold a partial set of levers to what the page can actually run.
 *
 * Applied to the query string and not to a draft's own values. A draft is priced by
 * `crates/project` and its `base-cost` provision is `1.0395` — deliberately off the slider's step
 * grid, which is the whole reason the draft path renders from the draft rather than from the
 * controls. Snapping or rejecting that would reintroduce the defect the draft path exists to
 * avoid. The query string has no such warrant: nothing computed it.
 *
 * Only the ends are enforced, never the step. A shared link carrying `base=1.0395` is a link to a
 * scenario this page can run and should keep running; one carrying `base=100` is not.
 *
 * A non-finite value is dropped rather than clamped, because there is no end of the range that
 * `NaN` was reaching for.
 */
export function clampLevers(levers: Partial<Levers>, horizon: HorizonBound): Partial<Levers> {
  const held: Partial<Levers> = { ...levers };
  for (const field of Object.keys(LEVER_BOUNDS) as (keyof typeof LEVER_BOUNDS)[]) {
    const value = held[field];
    if (value == null) continue;
    if (!Number.isFinite(value)) {
      delete held[field];
      continue;
    }
    const { min, max } = LEVER_BOUNDS[field];
    held[field] = Math.min(max, Math.max(min, value));
  }
  if (held.horizon != null) {
    if (!Number.isFinite(held.horizon)) delete held.horizon;
    // Rounded as well as bounded: the horizon indexes a loop over fiscal years, and a fractional
    // one runs to `through` without ever equalling it — `projectSeries` would return no projected
    // point for any district and the band would come back empty rather than wrong.
    else held.horizon = Math.min(horizon.max, Math.max(horizon.base, Math.round(held.horizon)));
  }
  return held;
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
    phaseInGeneral: levers.phaseInGeneral,
    phaseInDpia: levers.phaseInDpia,
  };
}

/**
 * The argument a guarantee rule actually takes, or `null` where it takes none.
 *
 * `as-enacted` and `removed` are nullary: {@link toPolicy} builds `{ kind }` and drops the number.
 * So two lever sets differing only in `guaranteeArgument` under one of those rules are the same
 * policy, and the retained-share control is hidden in exactly that state — a reader could not have
 * moved it, and cannot see what is supposed to differ.
 */
function guaranteeArgumentOf(rule: Policy["guarantee"]): number | null {
  switch (rule.kind) {
    case "rebase":
      return rule.factor;
    case "phase-out":
      return rule.remaining;
    default:
      return null;
  }
}

/**
 * Whether two sets of levers express the same policy.
 *
 * # Why this is one function and was two
 *
 * `isCurrentLaw` compared five fields and skipped `guaranteeArgument`; `matchesDraft` compared all
 * six. Both were asking "have the levers moved?" and they disagreed about which fields count, so
 * `?draft=hb-96-with-refreshed-inputs&arg=0.7` reported a departure from a bill it matched exactly
 * — the tiles were identical, and the control that supposedly differed was hidden.
 *
 * Comparing the *policies* rather than the levers settles it at the root: a field the formula never
 * reads cannot make two scenarios different, and there is no second list to keep in step.
 *
 * The horizon is outside this by construction — {@link toPolicy} does not carry it — which is the
 * behaviour both callers already wanted. Projecting further out changes what is being asked, not
 * what the state would do.
 */
function samePolicy(a: Levers, b: Levers): boolean {
  const x = toPolicy(a);
  const y = toPolicy(b);
  return (
    x.guarantee.kind === y.guarantee.kind &&
    guaranteeArgumentOf(x.guarantee) === guaranteeArgumentOf(y.guarantee) &&
    x.baseCostScale === y.baseCostScale &&
    x.minimumStateShare === y.minimumStateShare &&
    x.phaseInGeneral === y.phaseInGeneral &&
    x.phaseInDpia === y.phaseInDpia
  );
}

function isCurrentLaw(levers: Levers, model: number): boolean {
  return samePolicy(levers, defaultLevers(model));
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
 * # Where this sits, which is now between the two halves of the simulation
 *
 * It used to sit below the whole of it. It now sits below the simulation's three headline tiles
 * and above its distribution and tables — {@link RenderedScenario} is split for this. The part of
 * the old arrangement that mattered is kept: this card never appears above the figures it is a
 * forecast *of*, so a reader meets what the levers did before what they might do next, and there
 * is still deliberately no figure anywhere that adds the two together. A combined number would
 * inherit the forecast's error while wearing the simulation's precision.
 *
 * What changed is which simulation figures come first. The tiles are three numbers and a reader
 * who has just moved a lever is looking for them; the distribution, the most-affected table and
 * the underlying counts are the ones you read at rest, and a forecast is better company for those
 * than a wall of scrolling is.
 */
export function renderProjection(bundle: Panel, levers: Levers): string {
  const meta = bundle.projection;
  if (!meta) return "";
  const model = bundle.statewide.minimum_state_share;
  if (levers.horizon <= meta.base_year) {
    return `<div class="card" id="projection" data-part="projection">
      <h2>${anchor("projection")}At projected enrollment</h2>
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
  // The seam, not the first point: the forecast departs from the last observed year, and that is
  // the enrollment the projected one should be read against.
  const seam = path.filter((p) => p.observed).at(-1) ?? start;
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
    <div class="card" id="projection" data-part="projection">
      <h2>${anchor("projection")}At projected enrollment</h2>
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
          <div class="n">from ${count(Math.round(seam.adm))} observed in
            FY${seam.fiscalYear}</div></div>
      </div>

      <div class="chartwrap" data-chart="fan">${renderToString((w) => fanSpec(
        points,
        (v) => millions(v).replace("+", ""),
        (p) =>
          p.observed
            ? `FY${p.year}: ${millions(p.point).replace("+", "")} at published enrollment — exact`
            : `FY${p.year}: ${range(p.low, p.high)}, central ${millions(p.point).replace("+", "")}`,
        { width: w },
      ), { label: `Statewide total state aid by fiscal year at projected enrollment, FY${start.fiscalYear} to FY${end.fiscalYear}`, description: `Solid and exact through FY${seam.fiscalYear}, the last year of published enrollment; after it a dashed central estimate inside a band reaching ±${pct(width, 1)} at the horizon, on a vertical axis that does not start at zero` })}</div>
      <div class="legend">
        <span><i class="sw solid"></i> Observed enrollment, exact</span>
        <span><i class="sw anchor"></i> Last observed year</span>
        <span><i class="sw band"></i> Range at projected enrollment</span>
        <span><i class="sw dash"></i> Central estimate</span>
      </div>

      <p class="note">This is a <strong>forecast</strong>, and the card above it is not. The
        levers are held fixed and only enrollment moves: every district's enrolled ADM is carried
        forward from FY${meta.base_year} by a damped trend, and the FY${bundle.fiscal_year}
        formula is re-run at it. Nothing here projects the formula, the appropriation, or
        assessed valuation.</p>
      ${statuteNote(levers.horizon, meta.statute_ends)}
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

/**
 * The same levers, answered for one district.
 *
 * # Why this is not just the statewide table filtered to one row
 *
 * "What does this do to us" and "who does this reach" are different questions and the second one
 * is the one a policy argument turns on. A district page that showed only its own delta would let
 * a reader conclude a change is good because it is good for them, which is precisely the
 * reasoning the statewide incidence view exists to interrupt. So this card leads with the
 * district's own figure and then states, in the same card, how many districts move the other way
 * — and links to the distribution rather than summarising it away.
 */
export function renderDistrictScenario(bundle: Panel, levers: Levers, irn: string): string {
  const model = bundle.statewide.minimum_state_share;
  const district = bundle.districts.find((d) => d.irn === irn);
  if (!district) {
    return `<div class="card err" id="unknown-district" data-part="unknown-district"><p>No district with IRN ${escapeHtml(irn)} is in this feed.</p></div>`;
  }

  const outcomes = applyAll(bundle.districts, toPolicy(levers), model);
  const t = totals(outcomes);

  // What the settings did to the guarantee population, which the reached/unmoved split does not
  // say: a formula district can be unmoved because this lever does not touch it, a guarantee
  // district because nothing can touch it until the formula overtakes its frozen baseline.
  // `crates/scenario-delta` computes the same three counts and the feed's checkpoints carry them,
  // so the build has already checked this classification against the Rust before you see it.
  let liftedOff = 0;
  let pushedOn = 0;
  for (const [i, d] of bundle.districts.entries()) {
    const after = outcomes[i]!;
    if (d.on_guarantee && !after.onGuarantee) liftedOff++;
    else if (!d.on_guarantee && after.onGuarantee) pushedOn++;
  }
  const mine = outcomes.find((o) => o.irn === irn)!;

  if (isCurrentLaw(levers, model)) {
    return `<div class="card" id="current-law" data-part="current-law">
      <h2>${anchor("current-law")}Current law</h2>
      <p class="note">These are the settings the department's own FY${bundle.fiscal_year} model
        uses, so nothing moves. ${escapeHtml(district.name)} receives
        ${money(mine.realizedAid)}${
          district.on_guarantee
            ? `, of which ${money(mine.guarantee)} is the guarantee holding it above the
               ${money(mine.formulaAid)} the formula computes`
            : `, all of it computed by the formula`
        }. Move a lever.</p>
    </div>`;
  }

  // Where this district's change sits among the districts that moved at all. A rank is the honest
  // way to say "a lot" or "a little": the same dollar figure is a rounding error in Columbus and
  // a levy in a district of six hundred.
  const movers = outcomes
    .filter((o) => Math.abs(o.delta) > 0.5)
    .sort((a, b) => b.deltaPerPupil - a.deltaPerPupil);
  const rank = movers.findIndex((o) => o.irn === irn);
  const moved = Math.abs(mine.delta) > 0.5;

  return `
    <div class="tiles">
      <div class="tile"><div class="k">State aid under this scenario</div>
        <div class="v">${money(mine.realizedAid)}</div>
        <div class="n">against ${money(mine.baselineRealizedAid)} under current law</div></div>
      <div class="tile"><div class="k">Change</div>
        <div class="v ${mine.delta < 0 ? "loss" : mine.delta > 0 ? "gain" : ""}">${signedMoney(mine.delta)}</div>
        <div class="n">${signedMoney(mine.deltaPerPupil, 2)} per pupil</div></div>
      <div class="tile"><div class="k">Where it stands</div>
        <div class="v">${
          moved ? `${rank + 1} of ${movers.length}` : "unmoved"
        }</div>
        <div class="n">${
          moved
            ? `districts that move, ranked from largest gain to largest loss`
            : `this district's funding does not change under these settings`
        }</div></div>
    </div>

    <div class="card" id="moved-here" data-part="moved-here">
      <h2>${anchor("moved-here")}What moved for this district</h2>
      <div class="scroll"><table><tbody>
        <tr><th>Formula aid</th>
            <td>${money(currentFormulaAid(district))} → ${money(mine.formulaAid)}</td></tr>
        <tr><th>Guarantee</th>
            <td>${money(district.guarantee)} → ${money(mine.guarantee)}</td></tr>
        <tr><th>Total state aid</th>
            <td>${money(mine.baselineRealizedAid)} → ${money(mine.realizedAid)}</td></tr>
        <tr><th>On the guarantee</th>
            <td>${district.on_guarantee ? "yes" : "no"} → ${mine.onGuarantee ? "yes" : "no"}</td></tr>
      </tbody></table></div>
      <p class="note">${
        district.on_guarantee && !mine.onGuarantee
          ? `<strong>This scenario moves the district onto the formula.</strong> Its aid is now
             what the formula computes for it rather than what it received in FY2020.`
          : mine.onGuarantee && !district.on_guarantee
            ? `<strong>This scenario moves the district onto the guarantee.</strong> The formula
               now computes less for it than the FY2020 baseline, and the guarantee makes up the
               difference.`
            : `The district's relationship to the guarantee is unchanged by these settings.`
      }</p>
    </div>

    <div class="card" id="moved-elsewhere" data-part="moved-elsewhere">
      <h2>${anchor("moved-elsewhere")}And to everyone else</h2>
      <div class="scroll"><table><tbody>
        <tr><th>Districts reached</th><td>${t.gainers + t.losers} of ${t.districts}</td></tr>
        <tr><th>Up</th><td>${t.gainers}</td></tr>
        <tr><th>Down</th><td>${t.losers}</td></tr>
        <tr><th>Unmoved</th><td>${t.unmoved}</td></tr>
        <tr><th>Lifted off the guarantee<div class="n">The formula now computes more for them
          than their FY2020 baseline.</div></th><td>${liftedOff}</td></tr>
        <tr><th>Pushed onto it<div class="n">The formula now computes less.</div></th>
          <td>${pushedOn}</td></tr>
        <tr><th>Cost to the state</th><td class="${t.cost > 0 ? "gain" : t.cost < 0 ? "loss" : ""}">${millions(t.cost)}</td></tr>
      </tbody></table></div>
      <p class="note">A change that helps this district is not thereby a good change, and the
        count above is the reason: ${t.losers} district${t.losers === 1 ? "" : "s"} ${
          t.losers === 1 ? "receives" : "receive"
        } less under these settings. <a href="/scenario">The distribution across all
        ${t.districts}</a> — who gains, who loses, and how that falls across property wealth — is
        the statewide view.</p>
    </div>`;
}

/**
 * What a rendered scenario is made of, and where each half goes on the page.
 *
 * # Why this returns two strings rather than one
 *
 * The statewide page puts the {@link renderProjection} fan chart **between** them: the three
 * headline tiles sit directly under the levers, where a reader who has just moved one looks, and
 * the histogram and the two tables sit below the forecast. So the two halves land in two different
 * containers and cannot be one string.
 *
 * They are still computed in one pass. `applyAll` re-runs the formula over all 609 districts and
 * this function is called on every `input` event — on a slider drag that is once per frame — so
 * splitting it into two exported functions, each recomputing the outcomes, would double the work
 * in the hot path for a layout decision.
 */
export interface RenderedScenario {
  /** The tiles, and the current-law card when nothing has moved. Sits above the projection. */
  summary: string;
  /** The distribution, the most-affected table and the underlying counts. Sits below it. */
  detail: string;
}

/** Run the levers and render the result. */
export function renderScenario(bundle: Panel, levers: Levers): RenderedScenario {
  const model = bundle.statewide.minimum_state_share;
  if (isCurrentLaw(levers, model)) {
    return {
      summary: `<div class="card" id="current-law" data-part="current-law">
      <h2>${anchor("current-law")}Current law</h2>
      <p class="note">These are the settings the department's own FY${bundle.fiscal_year} model
        uses, so nothing moves. Total state foundation aid is
        ${millions(bundle.statewide.realized_aid_total).replace("+", "")} across
        ${count(bundle.statewide.districts)} districts, of which
        ${millions(bundle.statewide.guarantee_total).replace("+", "")} is the guarantee.
        Move a lever.</p>
    </div>`,
      // Nothing has moved, so there is nothing to distribute, rank, or account for. Returning an
      // empty string rather than an explanatory card is what lets the caller blank the container:
      // a stale "Most affected" left under the fan chart after a reset would be describing a
      // scenario the levers no longer hold.
      detail: "",
    };
  }

  const outcomes = applyAll(bundle.districts, toPolicy(levers), model);
  const t = totals(outcomes);

  // What the settings did to the guarantee population, which the reached/unmoved split does not
  // say: a formula district can be unmoved because this lever does not touch it, a guarantee
  // district because nothing can touch it until the formula overtakes its frozen baseline.
  // `crates/scenario-delta` computes the same three counts and the feed's checkpoints carry them,
  // so the build has already checked this classification against the Rust before you see it.
  let liftedOff = 0;
  let pushedOn = 0;
  for (const [i, d] of bundle.districts.entries()) {
    const after = outcomes[i]!;
    if (d.on_guarantee && !after.onGuarantee) liftedOff++;
    else if (!d.on_guarantee && after.onGuarantee) pushedOn++;
  }
  const deltas = outcomes
    .filter((o) => Math.abs(o.delta) > 0.5)
    .map((o) => o.deltaPerPupil);

  const summary = `
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
    </div>`;

  const detail = `
    <div class="card" id="distribution" data-part="distribution">
      <h2>${anchor("distribution")}How the change is distributed</h2>
      ${
        deltas.length > 0
          ? `<div class="chartwrap" data-chart="deltas">${renderToString((w) => histogramSpec(
              bin(deltas, 24),
              (v) => signedMoney(v),
              { width: w },
            ), { label: `Districts by change in state aid per pupil, for the ${count(deltas.length)} of ${count(t.districts)} districts these lever settings move against the FY${bundle.fiscal_year} model`, description: `Bins span ${signedMoney(Math.min(...deltas))} to ${signedMoney(Math.max(...deltas))} per pupil` })}</div>
        <div class="legend">
          <span><i class="sw loss"></i> Aid falls</span>
          <span><i class="sw gain"></i> Aid rises</span>
        </div>
        <p class="note">Districts by change in state aid per pupil. Bars are counts, not
          dollars — a tall bar near zero is many districts barely affected.</p>`
          : `<p class="note">No district's funding changes under these settings.</p>`
      }
    </div>

    <div class="card" id="most-affected" data-part="most-affected">
      <h2>${anchor("most-affected")}Most affected</h2>
      ${affectedTable(outcomes)}
    </div>

    <div class="card" id="moved-underneath" data-part="moved-underneath">
      <h2>${anchor("moved-underneath")}What moved underneath</h2>
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

  return { summary, detail };
}

/**
 * A draft's priced provisions, as lever positions.
 *
 * One function rather than two so the controls the page sets and the comparison it makes later
 * cannot disagree about what the draft says. `matchesDraft` is the reason that matters: a reader
 * who nudges a slider has stopped looking at the bill, and the page has to notice.
 */
export function draftLevers(draft: Draft, model: number, baseYear: number): Levers {
  const levers = defaultLevers(model, baseYear);
  for (const provision of draft.provisions) {
    const [rule, argument] = provision.proposed.split(":");
    switch (provision.lever) {
      case "guarantee":
        if (rule === "as-enacted" || rule === "removed" || rule === "rebase" || rule === "phase-out") {
          levers.guarantee = rule;
        }
        if (argument != null && Number.isFinite(Number(argument))) {
          levers.guaranteeArgument = Number(argument);
        }
        break;
      case "base-cost":
        levers.baseCostScale = Number(provision.proposed);
        break;
      case "min-share":
        levers.minimumStateShare = Number(provision.proposed);
        break;
      case "phase-in":
        levers.phaseInGeneral = Number(provision.proposed);
        break;
      case "phase-in-cat":
        levers.phaseInDpia = Number(provision.proposed);
        break;
      default:
        // An unpriced provision. It sets no lever, which is exactly why it has to be shown
        // separately rather than dropped — see `renderDraft`.
        break;
    }
  }
  return levers;
}

/**
 * Whether the controls still hold what the draft says.
 *
 * Delegates to {@link samePolicy}, which is what stops this and {@link isCurrentLaw} from
 * disagreeing about which fields count — they did, and the disagreement was visible as a departure
 * banner over figures identical to the bill's.
 */
export function matchesDraft(levers: Levers, draft: Draft, model: number, baseYear: number): boolean {
  return samePolicy(levers, draftLevers(draft, model, baseYear));
}

/**
 * What a draft's figure does not include, printed beside the figure.
 *
 * # Why this card exists, and why it is above the total rather than below it
 *
 * `crates/project` cannot produce a draft's cost without the provisions it failed to price:
 * `Priced` has no constructor that skips them. That guarantee stops at the process boundary. A
 * page that read the lever positions out of the feed and rendered the statewide total would show
 * a number for two of a bill's five clauses with nothing saying so — the same failure, one layer
 * up, and harder to notice because the page looks complete.
 *
 * So the unpriced provisions render with the total and not under it. Placement follows the
 * held-fixed card above the controls: this is a limit on what the reader is about to read, not a
 * footnote on what they got.
 *
 * # A moved lever is no longer the bill
 *
 * The levers are live. The moment one differs from what the draft sets, the figure below stops
 * being the draft's and the card says so rather than disappearing — a banner that vanished would
 * leave the reader with a number they still believe is the bill's, which is worse than no banner
 * at all.
 */
export function renderDraft(panel: Panel, levers: Levers, slug: string): string {
  const draft = panel.drafts.find((d) => d.slug === slug);
  /*
   * A slug this feed does not carry, which is what a shared link goes stale as.
   *
   * This returned `""`, so `/scenario?draft=hb-XXX` rendered a plain current-law page with
   * `&draft=hb-XXX` still in the address bar and nothing saying a bill had been asked for. That is
   * the failure this card exists to prevent, one level up: a reader who followed a link to a bill
   * meets figures they have every reason to read as the bill's.
   */
  if (!draft) {
    return `<div class="card err" id="draft" data-part="draft-unknown">
      <h2>${anchor("draft")}That bill is not in this feed</h2>
      <p class="note">This page was opened for a draft called
        <code>${escapeHtml(slug)}</code>, and this build of the feed carries no such bill — it was
        renamed, withdrawn, or the link predates it. <strong>The figures below are current law</strong>,
        not that bill's. <a href="/legislation">The bills this site does price</a> are on
        the legislation page.</p>
    </div>`;
  }

  const model = panel.statewide.minimum_state_share;
  const baseYear = panel.projection?.base_year ?? 0;
  const unpriced = draft.provisions.filter((p) => p.lever === "");
  const priced = draft.provisions.length - unpriced.length;
  const intact = matchesDraft(levers, draft, model, baseYear);
  const href = `${routes.wikiNode("draft-legislation", slug)}`;

  const departed = intact
    ? ""
    : `<p class="note err" data-part="draft-departed"><strong>These levers no longer match the
        draft.</strong> The figures below are yours rather than the bill's.
        <a href="?draft=${encodeURIComponent(slug)}">Put them back</a>.</p>`;

  const list = unpriced
    .map(
      (p) => `<li><strong>${escapeHtml(p.title)}</strong>
        <span class="tnum">${escapeHtml(p.authority)}</span><br />
        <span class="note">${escapeHtml(p.note)}</span></li>`,
    )
    .join("");

  /*
   * A draft nothing prices sets no lever, so the page below it is showing current law — which is
   * true and reads as false. Left alone, a reader lands on a bill and meets "nothing moves",
   * indistinguishable from a bill that costs nothing. See `Priced::cost` for the same distinction
   * one layer down, where it returns `None` rather than zero.
   */
  const nothingPriced =
    priced === 0
      ? `<p class="note err" data-part="draft-unpriceable"><strong>Nothing on this page is this
          bill.</strong> Not one of its ${count(draft.provisions.length)}
          provision${draft.provisions.length === 1 ? "" : "s"} sets a lever here, so the figures
          below are current law — which is what the controls say and not what the bill would do.
          There is no cost of zero to report; there is no cost.</p>`
      : "";

  /*
   * What the model cannot reach, in the three cases that read differently.
   *
   * The middle one is the defect this replaced. `nothingPriced` and this were emitted
   * independently, so a draft nothing prices got both — "There is no cost of zero to report; there
   * is no cost", immediately followed by "the total below is the cost of 0 clauses". The second
   * sentence was written assuming at least one clause priced, and it reported the zero the first
   * had just refused to report.
   *
   * So when nothing prices, the paragraph above has already said all of it and this contributes the
   * list alone.
   */
  const plural = unpriced.length === 1 ? "is" : "are";
  const missing =
    unpriced.length === 0
      ? `<p class="note" data-part="draft-complete">Every provision of this draft binds a lever, so
          the figures below are the whole of it rather than the part this model can reach. That is
          a property of a one-clause draft and not of drafts — a budget act moves special education
          weights, transportation and the scholarship deduction too, and none of those has a lever
          here.</p>`
      : priced === 0
        ? `<ul class="unpriced" data-part="draft-unpriced">${list}</ul>`
        : `<p class="note"><strong>${count(unpriced.length)} of this draft's
            ${count(draft.provisions.length)} provisions ${plural} not in any figure on this
            page.</strong> They set no lever, so moving the controls cannot express them and the
            total below is the cost of ${count(priced)}
            clause${priced === 1 ? "" : "s"}, not of the bill.</p>
          <ul class="unpriced" data-part="draft-unpriced">${list}</ul>`;

  return `<div class="card" id="draft" data-part="draft" data-draft="${escapeHtml(slug)}">
    <h2>${anchor("draft")}Opened from a draft</h2>
    <p class="note">The levers below are set to
      <a href="${href}">${escapeHtml(slug)}</a>, a bill that is not law.
      ${count(priced)} of its ${count(draft.provisions.length)} provisions bind a lever here.</p>
    ${nothingPriced}
    ${departed}
    ${missing}
  </div>`;
}
