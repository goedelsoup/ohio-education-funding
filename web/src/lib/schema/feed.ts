/**
 * The shape of `data/bundle.json`, as `crates/bundle` writes it — and the single definition of it.
 *
 * # Why this replaced a hand-written interface
 *
 * These types used to be an `interface Bundle` in `types.ts`, and `loadFeed` did
 * `JSON.parse(raw) as Bundle`. A cast is not a check: if the Rust renamed a field, dropped one, or
 * changed a number to a string, nothing anywhere noticed. The feed would parse, the types would
 * agree, and 609 pages would render `undefined` and `NaN` — quietly, because a missing number
 * formats as an em dash and an em dash looks deliberate.
 *
 * Now the shape is declared once, in zod, and the TypeScript types are inferred from it. The
 * build *parses* rather than casts, so a feed that does not match stops the build.
 *
 * # Why the objects are strict
 *
 * `.strict()` rejects unknown keys, which sounds unhelpful — a new field in the Rust is not an
 * error in itself. It is here, because this file is a hand-maintained mirror of a Rust struct and
 * the only failure that matters is the two drifting apart. A field appearing that this does not
 * know about is exactly that signal, arriving at the moment it is cheap to act on. The contract
 * version catches deliberate breaks; strictness catches the accidental ones.
 *
 * # Where the real authority is
 *
 * Still the Rust. This is a mirror with a tripwire on it, not a source of truth: `CONTRACT_VERSION`
 * in `crates/bundle` is what declares a breaking change, CI diffs the committed feed against a
 * freshly generated one, and the checkpoints in the feed are what prove the *arithmetic* agrees.
 * This layer only proves the *shape* does.
 */

import { z } from "astro/zod";

/**
 * A number.
 *
 * No `.finite()`: zod 4 rejects `NaN` and `Infinity` from `z.number()` already, and asking twice
 * is deprecated. The guard those would have provided — a non-number reaching a formatter as a
 * quantity — is still in force.
 */
const num = z.number();
/** A number the feed may legitimately not have for a district. */
const maybeNum = z.number().nullable();

/**
 * The outcome side of a district, where the report card covers it.
 *
 * Two spending figures because the choice of divisor is the corpus's central finding about this
 * data: `per_equivalent_pupil` divides by a need-weighted count and is the department's published
 * number, `per_enrolled_pupil` divides by the headcount. Against an outcome that is itself driven
 * by composition, the first is substantially a composition proxy.
 *
 * `economically_disadvantaged` is the report card's, which is top-coded by community eligibility.
 * The untop-coded share is `District.economically_disadvantaged`.
 */
export const DistrictOutcomeSchema = z
  .object({
    /** Ohio's attainment-level measure, 2024-25. */
    performance_index: maybeNum,
    performance_index_prior: maybeNum,
    performance_index_earliest: maybeNum,
    /** Ohio's growth measure, already a three-year average as published. */
    progress_effect_size: maybeNum,
    per_enrolled_pupil: maybeNum,
    per_equivalent_pupil: maybeNum,
    economically_disadvantaged: maybeNum,
    english_learner: maybeNum,
    students_with_disabilities: maybeNum,
  })
  .strict();

/**
 * Statewide relationships between funding and outcomes.
 *
 * Every field is a correlation and none identifies an effect. The raw and controlled guarantee
 * figures are both present because showing one without the other states a confound as a finding.
 */
export const OutcomeStatewideSchema = z
  .object({
    districts: z.number().int().nonnegative(),
    poverty_vs_performance: num,
    guarantee_vs_performance: num,
    guarantee_vs_performance_controlled: num,
    spending_vs_growth_controlled: num,
    weighted_spending_vs_performance: num,
    enrolled_spending_vs_performance: num,
    median_performance_on_guarantee: num,
    median_performance_on_formula: num,
  })
  .strict();

/** One closed fiscal year of a district's general fund. Audited actuals. */
export const FinanceYearSchema = z
  .object({
    fiscal_year: z.number().int(),
    /** Unrestricted grants-in-aid: state foundation money as the district books it. */
    state_aid: num,
    /** Property tax plus income tax — the local levy yield actually collected. */
    local_tax: num,
    total_revenue: num,
    total_expenditure: num,
    /** Cash balance at 30 June. What the district holds. */
    ending_cash: num,
  })
  .strict();

/**
 * The base cost build-up: all twenty-two elements of R.C. 3317.011, for one district.
 *
 * The only figures in this feed that this repository *computes* rather than reads. Everything else
 * per-district is the department's published model passed through; these come from
 * `crates/foundation` running the statutory staffing ratios against the district's own grade-band
 * enrollment, priced at the FY2027 statewide factor set.
 *
 * `published_aggregate` and `residual` travel with them on purpose. Claiming to reproduce a number
 * is worth nothing without showing the difference, and the difference here is about a dollar on
 * figures in the millions — twenty-two elements each rounded where the department rounds.
 */
export const BaseCostBuildUpSchema = z
  .object({
    // A — teacher base cost, R.C. 3317.011(D).
    classroom_teachers: num,
    special_teachers: num,
    substitutes: num,
    professional_development: num,
    teachers: num,
    // B — student support, R.C. 3317.011(E).
    counselors: num,
    librarians: num,
    wellness: num,
    academic_cocurricular: num,
    safety: num,
    supplies: num,
    technology: num,
    student_support: num,
    // C — district leadership and accountability, R.C. 3317.011(F).
    superintendent: num,
    treasurer: num,
    other_administrators: num,
    fiscal_support: num,
    emis: num,
    leadership_support: num,
    itc: num,
    district_leadership: num,
    // D — building leadership and operation, R.C. 3317.011(G).
    building_leadership_staff: num,
    building_support: num,
    building_operation: num,
    building_leadership: num,
    // E — athletics, R.C. 3317.011(H).
    athletic_cocurricular: num,
    funded_classroom_teachers: num,
    funded_special_teachers: num,
    computed_aggregate: num,
    published_aggregate: num,
    residual: num,
  })
  .strict();

/** One district, as the feed carries it. */
export const DistrictSchema = z
  .object({
    /** Six digits, always. 28 of the 609 *names* are shared, so this is the only safe key. */
    irn: z.string().regex(/^\d{6}$/, "an IRN is six digits"),
    name: z.string().min(1),
    /** Base cost enrolled ADM: the greater of the three-year average and the current year. */
    adm: num,
    /** Current-year enrolled ADM, FY2026. The denominator the state share is paid on. */
    current_year_adm: num,
    base_cost_per_pupil: num,
    aggregate_base_cost: num,
    /** How that aggregate is assembled — the one thing here that is computed, not quoted. */
    base_cost_build_up: BaseCostBuildUpSchema,
    /** The state's share of base cost alone, before every categorical. */
    base_cost_state_share: num,
    /** Targeted assistance, special education, DPIA, English learner, gifted, career-technical. */
    categorical_funding: num,
    formula_aid_per_pupil: num,
    realized_aid_per_pupil: num,
    guarantee: num,
    on_guarantee: z.boolean(),
    at_millage_floor: z.boolean(),
    at_minimum_state_share: z.boolean(),
    valuation_per_pupil: maybeNum,
    effective_class1_millage: maybeNum,
    operating_expenditure_per_pupil: maybeNum,
    economically_disadvantaged: maybeNum,
    /** FY2024 to FY2026. FY2026 is partly departmental estimate. */
    enrollment_change: maybeNum,
    /**
     * Enrolled ADM for FY2024, FY2025, FY2026 — the years `projection.base_year` ends.
     *
     * A tuple of exactly three, not an array: a district without a full history cannot be
     * projected, and a page that quietly dropped it would report a statewide total over a subset
     * of the panel.
     */
    adm_history: z.tuple([num, num, num]),
    /** Achievement, growth, and need. `null` for the three districts with no report card. */
    outcome: DistrictOutcomeSchema.nullable(),
    /**
     * Six closed fiscal years of **actuals**, oldest first. Empty where no filing was found.
     *
     * The only figures in this feed that are a record rather than a model. They come from the
     * district's own five-year forecast filing, not from the funding calculator, and the two are
     * differently constructed. Never render one as a check on the other.
     */
    finances: z.array(FinanceYearSchema),
  })
  .strict();

/** Statewide aggregates, so a district can be positioned without recomputing. */
export const StatewideSchema = z
  .object({
    districts: z.number().int().positive(),
    on_guarantee: z.number().int().nonnegative(),
    at_millage_floor: z.number().int().nonnegative(),
    at_minimum_state_share: z.number().int().nonnegative(),
    median_valuation_per_pupil: num,
    median_operating_expenditure_per_pupil: num,
    wealth_neutrality_formula: num,
    wealth_neutrality_realized: num,
    guarantee_total: num,
    realized_aid_total: num,
    minimum_state_share: num,
    /** How the funding side relates to the outcome side. `null` if no district joined. */
    outcomes: OutcomeStatewideSchema.nullable(),
    /**
     * Closed fiscal years of actuals, summed over the districts in this feed.
     *
     * Summed in Rust so the page and the feed cannot disagree about which districts are in the
     * total. The panel behind it covers 660 reporting bodies including joint vocational
     * districts; this is the 609 the feed carries.
     */
    finances: z.array(FinanceYearSchema),
  })
  .strict();

/**
 * A price index, so any year of the panel can be restated in another year's dollars.
 *
 * The choice of index is a claim, so the label travels with the numbers: CPI-U is a general
 * consumer index and school costs are majority compensation, for which the Employment Cost Index
 * would be better and has shorter coverage. Any real-dollar figure must name it.
 */
export const DeflatorSchema = z
  .object({
    label: z.string().min(1),
    points: z.array(z.object({ fiscal_year: z.number().int(), index: num }).strict()),
  })
  .strict();

/** A policy in the shape the feed serializes it. */
export const PolicyShapeSchema = z
  .object({
    guarantee: z.enum(["as-enacted", "removed", "rebase", "phase-out"]),
    guarantee_argument: num,
    base_cost_scale: num,
    minimum_state_share: num,
    phase_in_base_cost: num,
    phase_in_categorical: num,
  })
  .strict();

/**
 * A Rust-computed result this page must reproduce before it may compute its own.
 *
 * See `verify.ts`. The scenario builder re-derives the funding formula in TypeScript so a slider
 * does not need a round trip; these are what stop that second implementation from drifting away
 * from the first one unnoticed.
 */
export const CheckpointSchema = z
  .object({
    label: z.string().min(1),
    policy: PolicyShapeSchema,
    cost: num,
    realized_aid: num,
    gainers: z.number().int().nonnegative(),
    losers: z.number().int().nonnegative(),
    unmoved: z.number().int().nonnegative(),
    on_guarantee: z.number().int().nonnegative(),
  })
  .strict();

/**
 * A Rust-computed forecast this page must reproduce before it may draw a band.
 *
 * The same discipline as {@link CheckpointSchema}, applied to the harder half. Reproducing a
 * simulation checks one function; reproducing a forecast checks the projection, the prior, how the
 * interval compounds with the horizon, and the decision to re-run the formula at each end of the
 * enrollment band rather than scale the middle.
 */
export const ForecastCheckpointSchema = z
  .object({
    label: z.string().min(1),
    policy: PolicyShapeSchema,
    fiscal_year: z.number().int(),
    realized_aid: num,
    low: num,
    high: num,
    adm: num,
    on_guarantee: z.number().int().nonnegative(),
  })
  .strict();

/** How this feed's forecasts were made, and what their interval rests on. */
export const ProjectionMetaSchema = z
  .object({
    /** The last observed fiscal year. Everything past it is forecast. */
    base_year: z.number().int(),
    /** The furthest year the checkpoints reach, and the furthest this page should offer. */
    horizon: z.number().int(),
    method: z.enum(["last-observed", "cagr", "damped", "linear"]),
    damping: num,
    /**
     * Standard deviation of annual enrolled-ADM growth **across districts** — not within one.
     * Three observations cannot give a district's own variability.
     */
    sigma: num,
    z: num,
    prior_source: z.string().min(1),
    checkpoints: z.array(ForecastCheckpointSchema),
  })
  .strict();

/** The whole feed. */
export const BundleSchema = z
  .object({
    contract_version: z.string().min(1),
    provenance: z.string().min(1),
    fiscal_year: z.number().int(),
    statewide: StatewideSchema,
    checkpoints: z.array(CheckpointSchema),
    /** `null` disables the band: this feed cannot be projected. */
    projection: ProjectionMetaSchema.nullable(),
    /** `null` means the feed can only be shown in nominal dollars. */
    deflator: DeflatorSchema.nullable(),
    districts: z.array(DistrictSchema).min(1),
  })
  .strict();

export type DistrictOutcome = z.infer<typeof DistrictOutcomeSchema>;
export type OutcomeStatewide = z.infer<typeof OutcomeStatewideSchema>;
export type FinanceYear = z.infer<typeof FinanceYearSchema>;
export type BaseCostBuildUp = z.infer<typeof BaseCostBuildUpSchema>;
export type District = z.infer<typeof DistrictSchema>;
export type Statewide = z.infer<typeof StatewideSchema>;
export type Deflator = z.infer<typeof DeflatorSchema>;
export type PolicyShape = z.infer<typeof PolicyShapeSchema>;
export type Checkpoint = z.infer<typeof CheckpointSchema>;
export type ForecastCheckpoint = z.infer<typeof ForecastCheckpointSchema>;
export type ProjectionMeta = z.infer<typeof ProjectionMetaSchema>;
export type Bundle = z.infer<typeof BundleSchema>;
