/**
 * The shape of `data/bundle.json`, as `crates/bundle` writes it.
 *
 * The field names are the contract. `CONTRACT_VERSION` in that crate is bumped whenever any of
 * them changes meaning, and {@link REQUIRED_CONTRACT} below is what this page will render.
 */

/** The bundle contract this page understands. */
export const REQUIRED_CONTRACT = "2.0.0";

/** One district, as the feed carries it. */
export interface District {
  irn: string;
  name: string;
  /** Base cost enrolled ADM: the greater of the three-year average and the current year. */
  adm: number;
  /** Current-year enrolled ADM, FY2026. The denominator the state share is paid on. */
  current_year_adm: number;
  base_cost_per_pupil: number;
  aggregate_base_cost: number;
  /** The state's share of base cost alone, before every categorical. */
  base_cost_state_share: number;
  /** Targeted assistance, special education, DPIA, English learner, gifted, career-technical. */
  categorical_funding: number;
  formula_aid_per_pupil: number;
  realized_aid_per_pupil: number;
  guarantee: number;
  on_guarantee: boolean;
  at_millage_floor: boolean;
  at_minimum_state_share: boolean;
  valuation_per_pupil: number | null;
  effective_class1_millage: number | null;
  operating_expenditure_per_pupil: number | null;
  economically_disadvantaged: number | null;
  /** FY2024 to FY2026. FY2026 is partly departmental estimate. */
  enrollment_change: number | null;
}

/** Statewide aggregates, so a district can be positioned without recomputing. */
export interface Statewide {
  districts: number;
  on_guarantee: number;
  at_millage_floor: number;
  at_minimum_state_share: number;
  median_valuation_per_pupil: number;
  median_operating_expenditure_per_pupil: number;
  wealth_neutrality_formula: number;
  wealth_neutrality_realized: number;
  guarantee_total: number;
  realized_aid_total: number;
  minimum_state_share: number;
}

/** A policy in the shape the feed serializes it. */
export interface PolicyShape {
  guarantee: "as-enacted" | "removed" | "rebase" | "phase-out";
  guarantee_argument: number;
  base_cost_scale: number;
  minimum_state_share: number;
  phase_in_base_cost: number;
  phase_in_categorical: number;
}

/**
 * A Rust-computed result this page must reproduce before it may compute its own.
 *
 * See {@link ../src/verify.ts}. The scenario builder re-derives the funding formula in
 * TypeScript so a slider does not need a round trip; these are what stop that second
 * implementation from drifting away from the first one unnoticed.
 */
export interface Checkpoint {
  label: string;
  policy: PolicyShape;
  cost: number;
  realized_aid: number;
  gainers: number;
  losers: number;
  unmoved: number;
  on_guarantee: number;
}

/** The whole feed. */
export interface Bundle {
  contract_version: string;
  provenance: string;
  fiscal_year: number;
  statewide: Statewide;
  checkpoints: Checkpoint[];
  districts: District[];
}
