/**
 * The shape of `data/bundle.json`, as `crates/bundle` writes it.
 *
 * The field names are the contract. `CONTRACT_VERSION` in that crate is bumped whenever any of
 * them changes meaning, and {@link REQUIRED_CONTRACT} below is what this page will render.
 */

/** The bundle contract this page understands. */
export const REQUIRED_CONTRACT = "4.0.0";

/**
 * The outcome side of a district, where the report card covers it.
 *
 * Two spending figures because the choice of divisor is the corpus's central finding about
 * this data: `per_equivalent_pupil` divides by a need-weighted count and is the department's
 * published number, `per_enrolled_pupil` divides by the headcount. Against an outcome that is
 * itself driven by composition, the first is substantially a composition proxy.
 *
 * `economically_disadvantaged` is the report card's, which is top-coded by community
 * eligibility. The untop-coded share is `District.economically_disadvantaged`.
 */
export interface DistrictOutcome {
  /** Ohio's attainment-level measure, 2024-25. */
  performance_index: number | null;
  performance_index_prior: number | null;
  performance_index_earliest: number | null;
  /** Ohio's growth measure, already a three-year average as published. */
  progress_effect_size: number | null;
  per_enrolled_pupil: number | null;
  per_equivalent_pupil: number | null;
  economically_disadvantaged: number | null;
  english_learner: number | null;
  students_with_disabilities: number | null;
}

/**
 * Statewide relationships between funding and outcomes.
 *
 * Every field is a correlation and none identifies an effect. The raw and controlled guarantee
 * figures are both present because showing one without the other states a confound as a finding.
 */
export interface OutcomeStatewide {
  districts: number;
  poverty_vs_performance: number;
  guarantee_vs_performance: number;
  guarantee_vs_performance_controlled: number;
  spending_vs_growth_controlled: number;
  weighted_spending_vs_performance: number;
  enrolled_spending_vs_performance: number;
  median_performance_on_guarantee: number;
  median_performance_on_formula: number;
}

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
  /**
   * Enrolled ADM for FY2024, FY2025, FY2026 — the years `projection.base_year` ends.
   *
   * Not nullable: a district without a history cannot be projected, and a page that quietly
   * dropped it would report a statewide total over a subset of the panel.
   */
  adm_history: [number, number, number];
  /** Achievement, growth, and need. `null` for the three districts with no report card. */
  outcome: DistrictOutcome | null;
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
  /** How the funding side relates to the outcome side. `null` if no district joined. */
  outcomes: OutcomeStatewide | null;
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

/**
 * A Rust-computed forecast this page must reproduce before it may draw a band.
 *
 * The same discipline as {@link Checkpoint}, applied to the harder half. Reproducing a simulation
 * checks one function; reproducing a forecast checks the projection, the prior, how the interval
 * compounds with the horizon, and the decision to re-run the formula at each end of the
 * enrollment band rather than scale the middle.
 */
export interface ForecastCheckpoint {
  label: string;
  policy: PolicyShape;
  fiscal_year: number;
  realized_aid: number;
  low: number;
  high: number;
  adm: number;
  on_guarantee: number;
}

/** How this feed's forecasts were made, and what their interval rests on. */
export interface ProjectionMeta {
  /** The last observed fiscal year. Everything past it is forecast. */
  base_year: number;
  /** The furthest year the checkpoints reach, and the furthest this page should offer. */
  horizon: number;
  method: "last-observed" | "cagr" | "damped" | "linear";
  damping: number;
  /**
   * Standard deviation of annual enrolled-ADM growth **across districts** — not within one.
   * Three observations cannot give a district's own variability.
   */
  sigma: number;
  z: number;
  prior_source: string;
  checkpoints: ForecastCheckpoint[];
}

/** The whole feed. */
export interface Bundle {
  contract_version: string;
  provenance: string;
  fiscal_year: number;
  statewide: Statewide;
  checkpoints: Checkpoint[];
  /** `null` disables the band: this feed cannot be projected. */
  projection: ProjectionMeta | null;
  districts: District[];
}
