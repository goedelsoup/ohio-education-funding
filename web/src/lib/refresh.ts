/**
 * What a cost-input refresh actually delivers, decomposed into the two channels it travels.
 *
 * # The defect this replaces
 *
 * `/scenario`'s held-fixed card carried a worked example, typed:
 *
 * > A 3.1% refresh moves $113.0M of base cost aid and about $25.2M more through those three.
 *
 * `$113.0M` is `3.1% × $3,645.9M`, the panel's total base cost state share — a proportional
 * product. `policy.ts` rejects proportionality in as many words: *"Dollar-for-dollar per pupil, not
 * proportional: local capacity does not move when base cost does, so the state's residual absorbs
 * the whole per-pupil increase."* Run the model at 3.1% and base cost aid rises $265.7M.
 *
 * The two numbers were also different vintages. `$113.0M` came from
 * `.yidam/corpus/scenario/ACTIONS.md`'s pre-fix table, which reported what the site delivered
 * *before* `base_cost_scale` reached the denominated categoricals; `$25.2M` was recomputed after.
 * The sentence beside them — "until recently this page showed only the first number" — was false,
 * because the page never showed `$113.0M` at all.
 *
 * A reader could catch it with the slider directly below: at +3% the tile read `+$163.1M` against
 * the card's `$138.2M`.
 *
 * # Why the scale comes from a draft rather than from prose
 *
 * The old sentence named 3.1% — the FY2018-to-FY2022 restatement, which is a fact about a refresh
 * the current model has *already absorbed*, since H.B. 96 holds cost inputs at FY2022. Meanwhile
 * the lever's own note said "roughly +3%" and the two drafts that price a refresh both set
 * `1.0395`. Three numbers for one quantity, none of them saying which refresh it was.
 *
 * So the scale is read from the feed's own priced provision. It is the refresh a reader can open
 * with `?draft=`, the figures are computed by the same `applyAll` the tiles use, and nothing here
 * is typed twice.
 *
 * # Why the split is two runs and not an apportionment
 *
 * The guarantee is a `max`, so the two channels are not additive in the general case: a district
 * whose base cost aid rises past its guarantee floor delivers the whole increase, and one still
 * under the floor delivers none of it. Scaling a total by the ratio of the inputs would produce a
 * number with the right units and no referent. Running the formula twice — once with the
 * denominated categoricals moving and once with them held — measures what the second channel adds
 * on top of the first, which is what the sentence claims.
 *
 * Build-time only. Nothing in the browser needs this.
 */

import { applyAll, currentLaw, totals } from "./policy.ts";
import type { District, Draft, PanelDistrict } from "./types.ts";

/** A refresh, priced. */
export interface RefreshEffect {
  /** The multiplier on base cost, as the draft states it. `1.0395` is a 3.95% refresh. */
  scale: number;
  /** Realized aid delivered through base cost alone, in dollars. */
  throughBaseCost: number;
  /** What the base-cost-denominated categoricals add on top of it. */
  throughCategoricals: number;
  /** The slug of the draft whose provision sets the scale. */
  slug: string;
}

/** Total realized aid across the panel at one base cost scale. */
function delivered(districts: PanelDistrict[], scale: number, model: number): number {
  return totals(applyAll(districts, { ...currentLaw(model), baseCostScale: scale }, model))
    .realizedAid;
}

/**
 * Price the refresh the feed carries, or `null` where it carries none.
 *
 * `null` is not a failure: a feed with no `base-cost` provision has no refresh to illustrate, and
 * the card renders the sentence that does not depend on one. Inventing a scale to fill the gap is
 * the shape of defect this module exists to remove.
 */
export function refreshEffect(
  districts: PanelDistrict[],
  drafts: Draft[],
  model: number,
): RefreshEffect | null {
  const candidates = drafts
    .flatMap((draft) => draft.provisions.map((provision) => ({ draft, provision })))
    .filter(({ provision }) => provision.lever === "base-cost");
  /*
   * A one-clause draft first, where there is one.
   *
   * Both bills in the feed price the same restatement, and one of them is five clauses of which
   * this is the first. Naming that bill in a sentence about a refresh names a bill that *contains*
   * a refresh, and a reader following the link meets four other provisions. The single-clause draft
   * is the refresh, so it is the honest referent — and where the feed carries no such draft, the
   * bill is still true, just broader.
   */
  const priced =
    candidates.find(({ draft }) => draft.provisions.length === 1) ?? candidates[0];
  if (!priced) return null;

  const scale = Number(priced.provision.proposed);
  if (!Number.isFinite(scale) || scale <= 0) return null;

  /*
   * The same panel with the denominated categoricals zeroed, which is what "held fixed" means in
   * `apply`: `categoricals = categorical_funding − denominated + denominated × scale`, so a
   * denominated total of zero leaves the published figure alone at every scale.
   *
   * The baseline is the same either way — at `scale = 1` the two terms cancel — so one baseline
   * serves both differences.
   */
  const held = districts.map((d) => ({ ...d, base_cost_denominated_categoricals: 0 }));

  const baseline = delivered(districts, 1, model);
  const throughBaseCost = delivered(held, scale, model) - baseline;
  const both = delivered(districts, scale, model) - baseline;

  return {
    scale,
    throughBaseCost,
    throughCategoricals: both - throughBaseCost,
    slug: priced.draft.slug,
  };
}

/**
 * The three totals the held-fixed card names, summed from the feed.
 *
 * Every one was a typed literal — `812_500_000`, `1_890_000_000`, `45_700_000` — beside two more
 * that were wrong. `crates/project/tests/what_a_scenario_holds_fixed.rs` asserts the first and third
 * to within a million, so they were *checked*; they were still restated in a second language on the
 * other side of a process boundary, which is the arrangement that produced the two wrong ones.
 */
export interface HeldFixed {
  /** Categoricals priced in base cost per pupil and inside foundation funding: the lever moves these. */
  denominated: number;
  /** DPIA and targeted assistance, the indices whose numerator and denominator cancel. */
  indexDriven: number;
  /** Preschool special education's weighted half, denominated the same way and outside foundation funding. */
  preschoolWeighted: number;
}

/** Sum them. Build-time: `preschool_special_education` is not on the browser's slim panel. */
export function heldFixed(districts: District[]): HeldFixed {
  let denominated = 0;
  let indexDriven = 0;
  let preschoolWeighted = 0;
  for (const d of districts) {
    denominated += d.base_cost_denominated_categoricals;
    indexDriven += d.categoricals.dpia + d.categoricals.targeted_assistance;
    // The weighted half is the total less the flat component, which is how the Rust separates it.
    preschoolWeighted += d.preschool_special_education.total - d.preschool_special_education.flat_component;
  }
  return { denominated, indexDriven, preschoolWeighted };
}
