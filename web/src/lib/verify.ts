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
import type { Bundle, Checkpoint, PolicyShape } from "./types.ts";

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
export function compare(bundle: Bundle, checkpoint: Checkpoint): Comparison {
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

/** The verdict for the whole feed. */
export interface Verification {
  ok: boolean;
  comparisons: Comparison[];
}

/** Run every checkpoint. */
export function verify(bundle: Bundle): Verification {
  const comparisons = bundle.checkpoints.map((c) => compare(bundle, c));
  return { ok: comparisons.every((c) => c.agrees), comparisons };
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
