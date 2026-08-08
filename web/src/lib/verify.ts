/**
 * Checking this page's arithmetic against the Rust that is authoritative for it.
 *
 * `policy.ts` re-derives the funding formula so a slider does not need a round trip. That is a
 * duplicated implementation, and the way it stays honest is that the feed carries results the
 * Rust computed and this runs every one of them through the TypeScript before the scenario
 * builder is allowed to render. Against the real 609-district panel — not a sample.
 *
 * The tolerance is tight on purpose: a dollar across seven billion. Anything looser would
 * absorb a real disagreement in a lever that only moves a few districts.
 */

import { apply, totals, type GuaranteeRule, type Policy } from "./policy.ts";
import { forecast, growthPrior } from "./project.ts";
import type {
  Panel,
  Checkpoint,
  ForecastCheckpoint,
  PolicyShape,
} from "./types.ts";

/** Largest total-dollar disagreement treated as arithmetic noise rather than a defect. */
export const TOLERANCE = 1.0;

/** What a single checkpoint comparison found. */
export interface Comparison {
  label: string;
  agrees: boolean;
  /** Human-readable list of the fields that differ. Empty when they agree. */
  differences: string[];
}

/** Turn a serialized policy back into one this page can run. */
export function toPolicy(shape: PolicyShape): Policy {
  let guarantee: GuaranteeRule;
  switch (shape.guarantee) {
    case "removed":
      guarantee = { kind: "removed" };
      break;
    case "rebase":
      guarantee = { kind: "rebase", factor: shape.guarantee_argument };
      break;
    case "phase-out":
      guarantee = { kind: "phase-out", remaining: shape.guarantee_argument };
      break;
    default:
      guarantee = { kind: "as-enacted" };
  }
  return {
    guarantee,
    baseCostScale: shape.base_cost_scale,
    minimumStateShare: shape.minimum_state_share,
    phaseInBaseCost: shape.phase_in_base_cost,
    phaseInCategorical: shape.phase_in_categorical,
  };
}

/** Run one checkpoint's policy and compare every reported field. */
export function compare(bundle: Panel, checkpoint: Checkpoint): Comparison {
  const model = bundle.statewide.minimum_state_share;
  const policy = toPolicy(checkpoint.policy);
  const outcomes = bundle.districts.map((d) =>
    apply(d, policy, d.current_year_adm, model),
  );
  const t = totals(outcomes);

  const differences: string[] = [];
  const near = (name: string, ours: number, theirs: number) => {
    if (Math.abs(ours - theirs) > TOLERANCE) {
      differences.push(`${name}: page ${ours.toFixed(2)}, feed ${theirs.toFixed(2)}`);
    }
  };
  const same = (name: string, ours: number, theirs: number) => {
    if (ours !== theirs) differences.push(`${name}: page ${ours}, feed ${theirs}`);
  };

  near("cost", t.cost, checkpoint.cost);
  near("realized aid", t.realizedAid, checkpoint.realized_aid);
  same("gainers", t.gainers, checkpoint.gainers);
  same("losers", t.losers, checkpoint.losers);
  same("unmoved", t.unmoved, checkpoint.unmoved);
  same("districts on the guarantee", t.onGuarantee, checkpoint.on_guarantee);

  return { label: checkpoint.label, agrees: differences.length === 0, differences };
}

/**
 * Largest total-dollar disagreement in a *forecast* treated as noise.
 *
 * Looser than {@link TOLERANCE} by three orders of magnitude, and still tight: a forecast sums
 * 606 exponentials, and `Math.exp` and Rust's `f64::exp` are not required to agree in the last
 * bit. A thousand dollars across seven billion is 1.4 parts in ten million — far below any
 * disagreement that would mean the two implementations had actually diverged, and far above the
 * drift that transcendental rounding can produce.
 */
export const FORECAST_TOLERANCE = 1_000.0;

/** Run one forecast checkpoint's policy and horizon, and compare every reported field. */
export function compareForecast(
  bundle: Panel,
  checkpoint: ForecastCheckpoint,
): Comparison {
  const meta = bundle.projection;
  const differences: string[] = [];
  if (!meta) {
    return {
      label: checkpoint.label,
      agrees: false,
      differences: ["the feed carries forecasts but no projection block to run them with"],
    };
  }

  const effect = forecast(
    bundle.districts,
    toPolicy(checkpoint.policy),
    checkpoint.fiscal_year,
    meta.base_year,
    meta.method,
    meta.damping,
    growthPrior(bundle.districts, meta.z),
    bundle.statewide.minimum_state_share,
  );

  const near = (name: string, ours: number, theirs: number) => {
    if (Math.abs(ours - theirs) > FORECAST_TOLERANCE) {
      differences.push(`${name}: page ${ours.toFixed(2)}, feed ${theirs.toFixed(2)}`);
    }
  };
  near("realized aid", effect.realizedAid, checkpoint.realized_aid);
  // Both ends, not just the middle. A page that reproduced the point and got the band wrong
  // would be the exact failure this axis exists to prevent, and it would look correct.
  near("low end of the band", effect.low, checkpoint.low);
  near("high end of the band", effect.high, checkpoint.high);
  near("projected ADM", effect.adm, checkpoint.adm);
  if (effect.onGuarantee !== checkpoint.on_guarantee) {
    differences.push(
      `districts on the guarantee: page ${effect.onGuarantee}, feed ${checkpoint.on_guarantee}`,
    );
  }

  return { label: checkpoint.label, agrees: differences.length === 0, differences };
}

/** The verdict for the whole feed. */
export interface Verification {
  ok: boolean;
  comparisons: Comparison[];
  /** The forecast half, checked separately so one can fail without disabling the other. */
  forecasts: Comparison[];
}

/** Run every checkpoint, simulation and forecast alike. */
export function verify(bundle: Panel): Verification {
  const comparisons = bundle.checkpoints.map((c) => compare(bundle, c));
  const forecasts = (bundle.projection?.checkpoints ?? []).map((c) =>
    compareForecast(bundle, c),
  );
  return { ok: comparisons.every((c) => c.agrees), comparisons, forecasts };
}

/**
 * A feed with no checkpoints is unverified, not verified.
 *
 * Vacuous truth is the failure mode this guards: `every()` over an empty list is `true`, and a
 * feed that lost its checkpoints would otherwise pass silently and let the scenario builder run
 * unchecked.
 */
export function isVerified(v: Verification): boolean {
  return v.ok && v.comparisons.length > 0;
}

/**
 * Whether the band may be drawn.
 *
 * Deliberately separate from {@link isVerified}: the simulation and the forecast are different
 * claims, and a projection that failed its checkpoints should cost the reader the band, not the
 * scenario builder. The reverse also holds — a broken formula makes any forecast built on it
 * meaningless, so this requires both.
 */
export function isForecastVerified(v: Verification): boolean {
  return isVerified(v) && v.forecasts.length > 0 && v.forecasts.every((c) => c.agrees);
}
