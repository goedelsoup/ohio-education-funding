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
  /**
   * Weight on directly certified ADM in the DPIA count, against economically disadvantaged ADM.
   * Current law is 0.35 — H.B. 96's second-year blend. Zero is the count the formula used before
   * the act rewrote it.
   */
  dpiaDirectlyCertifiedWeight: number;
  /**
   * What the top of the supplemental targeted assistance scale pays, per pupil. **Current law is
   * zero**: R.C. 3317.0218 is repealed and the tier pays nothing while its eligibility test still
   * runs. Restoring it at the schedule it was last paid on is 750.
   */
  supplementalTopRate: number;
  /**
   * The minimum state share applied to transportation. Current law is 0.5. Separate from
   * `minimumStateShare` and acting on a separate programme — transportation sits outside core
   * foundation funding, so this moves `Outcome.transportation` and never `formulaAid`.
   */
  transportationFloor: number;
  /**
   * What happens to the formula transition supplement, `[K]`, when the guarantee moves.
   *
   * A second dial because they are different instruments in different law: the guarantee is
   * codified at R.C. 3317.019 and `[K]` is uncodified Section 265.225 of H.B. 110, extended to
   * FY2027 by H.B. 96. `[K]` tops a district up to its FY2021 base from a total that already
   * contains the guarantee, so retiring the guarantee with this left `"as-enacted"` moves 90.9%
   * of the apparent saving onto `[K]` rather than saving it.
   *
   * Mirrors two of the three values of `project::policy::Backstop`. The third, `rebased`, keeps
   * Section 265.225 and recomputes `[L1]` without the guarantee inside it; it prices a removal at
   * -$253.6M against -$79.8M here. It is absent because the feed carries `fy21_funding_base` and
   * not the guarantee inside it, so the minuend that reading changes does not reach the browser —
   * #490.
   */
  backstop: "as-enacted" | "repealed";
}

/**
 * Quantities a lever needs that are properties of the panel rather than of a district.
 *
 * Two of the three levers added for Stage 3 move a statewide statistic as a side effect of moving
 * a district's inputs, and one of those statistics divides the thing it is computed from. Mirrors
 * `project::policy::Statewide`.
 */
export interface Statewide {
  /** The percentage the DPIA index divides by, after any rescale the policy implies. */
  dpiaPercentage: number;
  /** The highest FY2019 targeted assistance wealth index, which tops the supplemental scale. */
  supplementalTopIndex: number;
}

/**
 * The constants the *published model* was computed under, as opposed to the ones being proposed.
 *
 * One object rather than four positional arguments. It began as a single `modelMinimumStateShare`
 * and grew three siblings when the Stage 3 levers arrived; four bare numbers of the same type in
 * a row is a place to transpose two of them and get a plausible wrong answer at every call site
 * at once.
 */
export interface Model {
  /** The minimum state share the published model used — 10% for FY2027. */
  minimumStateShare: number;
  /** The transportation floor in force — 50% for FY2027. */
  transportationFloor: number;
  /** The published statewide economically disadvantaged percentage. */
  dpiaPercentage: number;
  /** The published top of the supplemental scale. */
  supplementalTopIndex: number;
}

/** The two statewide quantities at their published values, which is current law's resolution. */
export function publishedStatewide(m: Model): Statewide {
  return {
    dpiaPercentage: m.dpiaPercentage,
    supplementalTopIndex: m.supplementalTopIndex,
  };
}

/** Read the model constants off a feed's statewide block. */
export function modelOf(w: {
  minimum_state_share: number;
  transportation_floor: number;
  dpia_statewide_percentage: number;
  supplemental_top_index: number;
}): Model {
  return {
    minimumStateShare: w.minimum_state_share,
    transportationFloor: w.transportation_floor,
    dpiaPercentage: w.dpia_statewide_percentage,
    supplementalTopIndex: w.supplemental_top_index,
  };
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
  /**
   * Transportation aid under the policy's floor.
   *
   * Outside `realizedAid`, deliberately: transportation is not core foundation funding, and
   * folding it in would change what every other figure on this site means.
   */
  transportation: number;
  /** Transportation under current law's floor, for the comparison. */
  baselineTransportation: number;
  /**
   * `[K]` the formula transition supplement, re-derived against this policy's own output.
   *
   * Outside `realizedAid` for the same reason transportation is, and inside total state support.
   * A lever that cuts realized aid or transportation is partly offset by this rising.
   */
  transitionSupplement: number;
  /** The same under current law, for the comparison. */
  baselineTransitionSupplement: number;
  /** Change in core foundation funding against current law, in dollars. */
  delta: number;
  /** Change against current law, per current-year pupil. */
  deltaPerPupil: number;
  /** Change across both channels — the figure a cost column should show. */
  totalDelta: number;
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
  /** Total transportation aid, which is outside `realizedAid`. */
  transportation: number;
  /** Total formula transition supplement, also outside `realizedAid`. */
  transitionSupplement: number;
  /** Change in total state support — realized aid and transportation — against current law. */
  cost: number;
  gainers: number;
  losers: number;
  unmoved: number;
}

/** Current law: the identity, which reproduces the department's own FY2027 model. */
export function currentLaw(m: Model): Policy {
  return {
    guarantee: { kind: "as-enacted" },
    baseCostScale: 1,
    minimumStateShare: m.minimumStateShare,
    phaseInGeneral: 1,
    phaseInDpia: 1,
    dpiaDirectlyCertifiedWeight: DPIA_BLEND,
    // Zero, and not a rate: the tier is repealed, so current law pays it nothing. The only lever
    // here whose identity is an absence.
    supplementalTopRate: 0,
    transportationFloor: m.transportationFloor,
    backstop: "as-enacted",
  };
}

/** H.B. 96's second-year DPIA blend: 65% economically disadvantaged, 35% directly certified. */
export const DPIA_BLEND = 0.35;

/** What each weighted disadvantaged pupil generates. */
export const DPIA_PER_PUPIL = 422;

/** The FY2019 wealth index the supplemental tier requires a district to exceed. */
export const TA_SUPPLEMENT_INDEX_THRESHOLD = 1.6;

/** The rate at the bottom of the supplemental scale, as a share of the rate at the top. */
export const TA_SUPPLEMENT_FLOOR_SHARE = 0.1;

/**
 * DPIA from a blended count, at a stated statewide index denominator.
 *
 * `d1 × $422 × (d1 / enrolled ADM / statewide)²`, with `d1` capped at the district's own enrolled
 * ADM. Mirrors `project::panel::Dpia::aid_from_count`.
 */
export function dpiaFromCount(
  weightedAdm: number,
  enrolledAdm: number,
  statewide: number,
): number {
  if (enrolledAdm <= 0 || statewide <= 0) return 0;
  const d1 = Math.min(weightedAdm, enrolledAdm);
  const d2 = d1 / enrolledAdm;
  return d1 * DPIA_PER_PUPIL * (d2 / statewide) ** 2;
}

/** The DPIA count at a given weight on directly certified ADM. */
export function dpiaCountAt(d: PanelDistrict, weight: number): number {
  return (
    (1 - weight) * d.dpia_econ_disadvantaged_adm +
    weight * d.dpia_directly_certified_adm
  );
}

/**
 * Resolve the two statewide quantities against a panel and a policy.
 *
 * The DPIA rescale is closed-form: aid scales as the inverse square of the denominator, so
 * holding the statewide total fixed while the count moves is `s' = s × √(total at s / target)`.
 * No search, and no tolerance to choose. Mirrors `project::policy::Statewide::under`.
 */
export function statewideUnder(
  districts: PanelDistrict[],
  p: Policy,
  m: Model,
): Statewide {
  // Current law's weight needs no rescale, and asking for one would introduce a residual where
  // the identity requires none.
  if (p.dpiaDirectlyCertifiedWeight === DPIA_BLEND) {
    return {
      dpiaPercentage: m.dpiaPercentage,
      supplementalTopIndex: m.supplementalTopIndex,
    };
  }

  const at = (weight: number): number => {
    let total = 0;
    for (const d of districts) {
      total += dpiaFromCount(
        dpiaCountAt(d, weight),
        d.current_year_adm,
        m.dpiaPercentage,
      );
    }
    return total;
  };
  const target = at(DPIA_BLEND);
  const moved = at(p.dpiaDirectlyCertifiedWeight);
  return {
    dpiaPercentage:
      moved > 0 && target > 0
        ? m.dpiaPercentage * Math.sqrt(moved / target)
        : m.dpiaPercentage,
    supplementalTopIndex: m.supplementalTopIndex,
  };
}

/**
 * The supplemental tier's per-pupil rate for one district, at a stated top of the scale.
 *
 * Linear from a tenth of `topRate` at the eligibility threshold of 1.6 to `topRate` at the
 * state's highest FY2019 wealth index. Mirrors
 * `project::panel::TargetedAssistance::supplemental_rate`.
 */
export function supplementalRate(
  d: PanelDistrict,
  topIndex: number,
  topRate: number,
): number {
  if (!d.supplement_eligible || topIndex <= TA_SUPPLEMENT_INDEX_THRESHOLD) {
    return 0;
  }
  const span =
    (d.supplemental_wealth_index - TA_SUPPLEMENT_INDEX_THRESHOLD) /
    (topIndex - TA_SUPPLEMENT_INDEX_THRESHOLD);
  return (
    topRate * (TA_SUPPLEMENT_FLOOR_SHARE + (1 - TA_SUPPLEMENT_FLOOR_SHARE) * span)
  );
}

/**
 * One district's transportation aid at a stated floor and enrollment.
 *
 * The gross is recovered in Rust and carried on the feed, because the department publishes
 * transportation net and dividing a published figure by a constant the browser would have to be
 * told separately is how two implementations come to disagree. Mirrors
 * `project::policy::transport_under`.
 */
export function transportUnder(
  d: PanelDistrict,
  currentYearAdm: number,
  floor: number,
): number {
  if (d.transportation_state_share <= 0 || d.transportation_gross <= 0) {
    return d.transportation_paid;
  }
  const admRatio =
    d.current_year_adm > 0 ? currentYearAdm / d.current_year_adm : 1;
  return (
    (d.transportation_gross * Math.max(d.transportation_state_share, floor) +
      d.transportation_guarantee) *
    admRatio
  );
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
 * `m` carries the constants the *published model* was computed under — a 10% minimum state share
 * and a 50% transportation floor for FY2027 — which are distinct from the ones on `p` being
 * proposed. They are only equal under current law.
 */
export function apply(
  d: PanelDistrict,
  p: Policy,
  w: Statewide,
  currentYearAdm: number,
  m: Model,
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
      (p.minimumStateShare / m.minimumStateShare);
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
  // DPIA, recomputed from the counts rather than read, so the blend is a lever. At current law
  // the published figure is used instead: the recomputation reproduces the department's column to
  // a rounding artefact rather than exactly, and that residual would leak into `currentLaw` and
  // stop it being the identity. Mirrors the same guard in `project::policy::apply`.
  const publishedDpia = d.dpia_funding * admRatio;
  const dpiaComputed =
    p.dpiaDirectlyCertifiedWeight === DPIA_BLEND
      ? publishedDpia
      : dpiaFromCount(
          dpiaCountAt(d, p.dpiaDirectlyCertifiedWeight),
          currentYearAdm,
          w.dpiaPercentage,
        );

  // The repealed supplemental tier, which pays nothing until a policy funds it. `[I]` is zero in
  // the model for every district, so this adds rather than replaces.
  const supplemental =
    supplementalRate(d, w.supplementalTopIndex, p.supplementalTopRate) *
    currentYearAdm;

  const generalComputed =
    baseCostAid + (categoricals * admRatio - publishedDpia) + supplemental;

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

  // Transportation, priced against its own floor and kept out of the sum above.
  const transportation = transportUnder(d, currentYearAdm, p.transportationFloor);
  const baselineTransportation = transportUnder(
    d,
    d.current_year_adm,
    m.transportationFloor,
  );

  // `[K]`, which R.C. 3317.019 does not govern: Section 265.225 tops the district up to its
  // FY2021 base from a total that already contains the guarantee, so it is computed after
  // realized aid and transportation and is what absorbs a cut to either.
  //
  // Both sides come from the same construction rather than one being read from the published
  // column, so the cent of drift in recovering transportation cancels instead of registering as
  // movement. Mirrors `project::policy::apply`.
  const transitionSupplement =
    p.backstop === "repealed"
      ? 0
      : Math.max(0, d.fy21_funding_base - (realizedAid + transportation));
  const baselineTransitionSupplement = Math.max(
    0,
    d.fy21_funding_base - (baselineRealizedAid + baselineTransportation),
  );

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
    transportation,
    baselineTransportation,
    transitionSupplement,
    baselineTransitionSupplement,
    delta,
    deltaPerPupil: currentYearAdm > 0 ? delta / currentYearAdm : 0,
    totalDelta:
      transportation +
      realizedAid +
      transitionSupplement -
      (baselineTransportation + baselineRealizedAid + baselineTransitionSupplement),
  };
}

/** Apply a policy across the panel at modelled enrollment. */
export function applyAll(
  districts: PanelDistrict[],
  p: Policy,
  m: Model,
): Outcome[] {
  // Resolved once against the whole panel: the DPIA index denominator is not a property of any
  // one district, and a per-district resolution would be a different number and a wrong one.
  const w = statewideUnder(districts, p, m);
  return districts.map((d) => apply(d, p, w, d.current_year_adm, m));
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
  let transportation = 0;
  let baselineTransportation = 0;
  let transitionSupplement = 0;
  let baselineTransitionSupplement = 0;
  for (const o of outcomes) {
    realizedAid += o.realizedAid;
    formulaAid += o.formulaAid;
    guarantee += o.guarantee;
    baseline += o.baselineRealizedAid;
    transportation += o.transportation;
    baselineTransportation += o.baselineTransportation;
    transitionSupplement += o.transitionSupplement;
    baselineTransitionSupplement += o.baselineTransitionSupplement;
    if (o.onGuarantee) onGuarantee++;
    if (o.atMinimumStateShare) atMinimumStateShare++;
    // Across both channels. Counting on `delta` alone would report a transportation-only policy
    // as moving nobody, which is the same defect as reporting its cost as zero.
    if (o.totalDelta > MOVED) gainers++;
    else if (o.totalDelta < -MOVED) losers++;
  }
  return {
    districts: outcomes.length,
    onGuarantee,
    atMinimumStateShare,
    realizedAid,
    formulaAid,
    guarantee,
    transportation,
    // Total state support, not realized aid: the transportation floor moves a channel core
    // foundation funding does not contain, and a cost that could not see it would agree with a
    // browser that had never implemented the lever.
    transitionSupplement,
    cost:
      realizedAid +
      transportation +
      transitionSupplement -
      (baseline + baselineTransportation + baselineTransitionSupplement),
    gainers,
    losers,
    unmoved: outcomes.length - gainers - losers,
  };
}
