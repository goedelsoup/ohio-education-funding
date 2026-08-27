/**
 * The funding formula, re-derived in the browser.
 *
 * This is a second implementation of `crates/project/src/policy.rs`. Two implementations of the
 * same formula is normally a bad trade — they drift, and the one nobody runs is the one that is
 * wrong. It is acceptable here for one reason: **the feed carries Rust-computed checkpoints and
 * this page refuses to render a scenario until it reproduces every one of them**, against the
 * real 609-district panel, on every load. See `verify.ts`.
 *
 * So the Rust stays authoritative and the TypeScript has to prove it agrees. If someone edits
 * one and not the other, the page says so instead of showing a plausible wrong number.
 *
 * Keep this file a line-for-line mirror of `apply()` in the Rust. Any cleverness here is a
 * liability.
 */

import type { PanelDistrict } from "./types.ts";

/** What happens to the temporary transitional aid guarantee. */
export type GuaranteeRule =
  | { kind: "as-enacted" }
  | { kind: "removed" }
  | { kind: "rebase"; factor: number }
  | { kind: "phase-out"; remaining: number };

/** A set of levers. */
export interface Policy {
  guarantee: GuaranteeRule;
  /** Multiplier on aggregate base cost. */
  baseCostScale: number;
  /** Minimum state share of base cost. Set by each biennial budget, not permanent law. */
  minimumStateShare: number;
  /**
   * How far the district moves from its FY2020 funding base toward the computed amount, for
   * every core foundation component except DPIA.
   *
   * Not a fraction of computed aid. R.C. 3317.022 pays `base + pct × (computed − base)`, so at
   * 0% a district receives its FY2020 base rather than nothing.
   */
  phaseInGeneral: number;
  /** The same interpolation for DPIA, against its own FY2019 base. */
  phaseInDpia: number;
}

/** What a policy does to one district. */
export interface Outcome {
  irn: string;
  name: string;
  adm: number;
  formulaAid: number;
  realizedAid: number;
  guarantee: number;
  baselineRealizedAid: number;
  onGuarantee: boolean;
  atMinimumStateShare: boolean;
  /** Change against current law, in dollars. */
  delta: number;
  /** Change against current law, per current-year pupil. */
  deltaPerPupil: number;
}

/**
 * Below this, a change in a district's aid is floating-point residue rather than a change.
 *
 * `formulaAid` interpolates through `base + pct × (computed − base)`, which at `pct = 1` is not
 * exactly `computed` in binary floating point — so an identity run leaves deltas around 1e-9 and a
 * bare `!== 0` would report all 609 districts moved.
 *
 * Exported because the page filters on it too. `totals()` counted at this bound while the
 * histogram, the most-affected table and the district's rank each counted at half a dollar, so
 * "Districts reached" and the histogram's "the N of 609 districts these lever settings move" were
 * two counts of one quantity. No lever setting reachable from the controls made them disagree —
 * the steps are too coarse to produce a sub-dollar delta — but one shared constant is cheaper than
 * the argument that they cannot.
 */
export const MOVED = 0.005;

/** Statewide totals for a set of outcomes. */
export interface Totals {
  districts: number;
  onGuarantee: number;
  atMinimumStateShare: number;
  realizedAid: number;
  formulaAid: number;
  guarantee: number;
  /** Change in total state aid against current law. */
  cost: number;
  gainers: number;
  losers: number;
  unmoved: number;
}

/** Current law: the identity, which reproduces the department's own FY2027 model. */
export function currentLaw(modelMinimumStateShare: number): Policy {
  return {
    guarantee: { kind: "as-enacted" },
    baseCostScale: 1,
    minimumStateShare: modelMinimumStateShare,
    phaseInGeneral: 1,
    phaseInDpia: 1,
  };
}

/** Formula aid under current law: base cost share plus every categorical. */
export function currentFormulaAid(d: PanelDistrict): number {
  return d.base_cost_state_share + d.categorical_funding;
}

/**
 * Realized aid under current law.
 *
 * Summed from the components rather than taken from `realized_aid_per_pupil × adm`, which would
 * lose cents to the round trip and make the identity only approximately the identity.
 */
export function currentRealizedAid(d: PanelDistrict): number {
  return currentFormulaAid(d) + d.guarantee;
}

/**
 * Apply a policy to one district at a given current-year enrolled ADM.
 *
 * `modelMinimumStateShare` is the minimum the *published model* was computed under — 10% for
 * FY2027 — and is distinct from `policy.minimumStateShare`, which is the one being proposed.
 * The two are only equal under current law.
 */
export function apply(
  d: PanelDistrict,
  p: Policy,
  currentYearAdm: number,
  modelMinimumStateShare: number,
): Outcome {
  const baseCostPerPupil = d.base_cost_per_pupil * p.baseCostScale;
  const floorPerPupil = baseCostPerPupil * p.minimumStateShare;

  const modelledSharePerPupil =
    d.current_year_adm > 0 ? d.base_cost_state_share / d.current_year_adm : 0;
  const increasePerPupil = baseCostPerPupil - d.base_cost_per_pupil;

  // A district at the minimum state share has its local capacity censored by the floor: all
  // that is known is that it exceeds `1 − minimum` of base cost, not by how much. It is held at
  // the floor, which understates the gain for any whose capacity is only just above.
  const censored = d.at_minimum_state_share;
  const residualPerPupil = modelledSharePerPupil + increasePerPupil;
  const atMinimum = censored || residualPerPupil < floorPerPupil;

  const admRatio = d.current_year_adm > 0 ? currentYearAdm / d.current_year_adm : 1;

  let baseCostAid: number;
  if (censored) {
    baseCostAid =
      d.base_cost_state_share *
      admRatio *
      p.baseCostScale *
      (p.minimumStateShare / modelMinimumStateShare);
  } else if (residualPerPupil < floorPerPupil) {
    baseCostAid = floorPerPupil * currentYearAdm;
  } else {
    // Dollar-for-dollar per pupil, not proportional: local capacity does not move when base
    // cost does, so the state's residual absorbs the whole per-pupil increase.
    baseCostAid =
      d.base_cost_state_share * admRatio + increasePerPupil * currentYearAdm;
  }

  // The categoricals priced in the statewide average base cost move with it. Special education,
  // English learners and career-technical are each `weight × $8,241.61 × count × state share`, so
  // a lever that raises base cost per pupil raises them mechanically. The department's own
  // simulator holds them fixed — correctly, for a tool that changes one district at a time — and
  // this site does not, because a policy lever here moves all 609 at once. Mirrors
  // `project::policy::apply`; the reference checkpoints are what keep the two honest.
  const denominated = d.base_cost_denominated_categoricals;
  const categoricals =
    d.categorical_funding - denominated + denominated * p.baseCostScale;

  // The phase-in, as R.C. 3317.022 writes it:
  //
  //     funding base
  //       + [(general components − general funding base) × general phase-in %]
  //       + [(DPIA − DPIA funding base)                  × DPIA phase-in %]
  //
  // Two interpolations against two slices of one published base, not two multipliers on
  // computed aid. At 100% on both dials the bases cancel and this is the department's own
  // number, which is what keeps `currentLaw` the identity.
  const dpiaComputed = d.dpia_funding * admRatio;
  const generalComputed =
    baseCostAid + (categoricals * admRatio - dpiaComputed);

  const formulaAid =
    d.general_funding_base +
    p.phaseInGeneral * (generalComputed - d.general_funding_base) +
    (d.dpia_funding_base + p.phaseInDpia * (dpiaComputed - d.dpia_funding_base));

  // `[H2] − [I1]`, published for every district. This used to be
  // `d.on_guarantee ? currentRealizedAid(d) : 0`, which gave the 315 formula districts no floor.
  const baseline = d.guarantee_floor;
  let heldAt: number;
  switch (p.guarantee.kind) {
    case "as-enacted":
      heldAt = baseline;
      break;
    case "rebase":
      heldAt = baseline * p.guarantee.factor;
      break;
    case "phase-out":
      heldAt = formulaAid + p.guarantee.remaining * Math.max(0, baseline - formulaAid);
      break;
    case "removed":
      heldAt = 0;
      break;
  }
  const realizedAid = Math.max(formulaAid, heldAt);
  const baselineRealizedAid = currentRealizedAid(d);
  const delta = realizedAid - baselineRealizedAid;

  return {
    irn: d.irn,
    name: d.name,
    adm: currentYearAdm,
    formulaAid,
    realizedAid,
    guarantee: Math.max(0, realizedAid - formulaAid),
    baselineRealizedAid,
    onGuarantee: realizedAid > formulaAid + 0.005,
    atMinimumStateShare: atMinimum,
    delta,
    deltaPerPupil: currentYearAdm > 0 ? delta / currentYearAdm : 0,
  };
}

/** Apply a policy across the panel at modelled enrollment. */
export function applyAll(
  districts: PanelDistrict[],
  p: Policy,
  modelMinimumStateShare: number,
): Outcome[] {
  return districts.map((d) =>
    apply(d, p, d.current_year_adm, modelMinimumStateShare),
  );
}

/** Aggregate a set of outcomes. */
export function totals(outcomes: Outcome[]): Totals {
  let realizedAid = 0;
  let formulaAid = 0;
  let guarantee = 0;
  let baseline = 0;
  let onGuarantee = 0;
  let atMinimumStateShare = 0;
  let gainers = 0;
  let losers = 0;
  for (const o of outcomes) {
    realizedAid += o.realizedAid;
    formulaAid += o.formulaAid;
    guarantee += o.guarantee;
    baseline += o.baselineRealizedAid;
    if (o.onGuarantee) onGuarantee++;
    if (o.atMinimumStateShare) atMinimumStateShare++;
    if (o.delta > MOVED) gainers++;
    else if (o.delta < -MOVED) losers++;
  }
  return {
    districts: outcomes.length,
    onGuarantee,
    atMinimumStateShare,
    realizedAid,
    formulaAid,
    guarantee,
    cost: realizedAid - baseline,
    gainers,
    losers,
    unmoved: outcomes.length - gainers - losers,
  };
}
