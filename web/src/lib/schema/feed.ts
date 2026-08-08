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
    /**
     * The same measure over one year, which the department publishes alongside it.
     *
     * Carried so the smoothing is a visible choice. The two agree wherever agreement means
     * anything: of the 534 districts printing non-zero on both, 44 point opposite ways and none
     * of those has both magnitudes above 0.05. Every disagreement is a district sitting on zero.
     */
    progress_effect_size_one_year: maybeNum,
    per_enrolled_pupil: maybeNum,
    per_equivalent_pupil: maybeNum,
    /** The federal part of `per_equivalent_pupil`. The two parts add to the whole. */
    per_equivalent_pupil_federal: maybeNum,
    per_equivalent_pupil_state_local: maybeNum,
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
    /** Federal money as a share of operating spending: the median, the maximum, and the tail. */
    median_federal_share: num,
    max_federal_share: num,
    federal_share_above_tenth: z.number().int().nonnegative(),
    /** Federal share against attainment, holding poverty constant — and the raw figure beside it. */
    federal_share_vs_performance: num,
    federal_share_vs_performance_raw: num,
    /**
     * Districts whose two growth measures print non-zero values pointing opposite ways.
     *
     * Over `growth_measures_determinate`, not over all districts: a printed `0.00` covers
     * anything in (-0.005, 0.005) and has no sign to disagree about. Counting those as negative
     * turns 44 into 76, which is the first number this site computed and nearly published.
     */
    growth_measures_disagree: z.number().int().nonnegative(),
    growth_measures_determinate: z.number().int().nonnegative(),
    /** Of the disagreements, those where both magnitudes exceed 0.05. It is zero. */
    growth_measures_disagree_materially: z.number().int().nonnegative(),
    growth_measure_agreement: num,
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

/**
 * One tax year of a district's property tax base and the charge on it, from Table SD-1.
 *
 * The Department of Taxation's table, not the Department of Education's — the local half of the
 * formula measured by the other half of the state. Where the two overlap they agree: SD-1's
 * effective Class I rate matches the District Profile Report's for all 606 districts carrying
 * both, to 0.01 mills.
 *
 * Two years are carried because the mechanism only exists as a change. H.B. 920's reduction
 * factors roll an effective rate back as valuation rises and cannot roll it below twenty mills,
 * so what a reappraisal does to a district depends on which side of that floor it is on — and one
 * year cannot show that.
 */
export const PropertyTaxYearSchema = z
  .object({
    tax_year: z.number().int(),
    /** Class I: residential and agricultural, which carry their own reduction factor. */
    class1_value: num,
    /** Class II: commercial, industrial, mineral, railroad. */
    class2_value: num,
    /** Public utility tangible property — neither class, and not reduced. */
    public_utility_value: num,
    total_value: num,
    agricultural_value: num,
    residential_value: num,
    commercial_value: num,
    industrial_value: num,
    mineral_value: num,
    railroad_value: num,
    /** Effective Class I operating millage, after reduction factors. */
    class1_rate: num,
    class2_rate: num,
    class1_taxes_charged: num,
    class2_taxes_charged: num,
    /** Both classes, excluding joint vocational operating levies. */
    real_property_taxes_charged: num,
    public_utility_taxes_charged: num,
    value_per_pupil: num,
    /**
     * The pupil count Table SD-1 divides by, which is not the funding formula's.
     *
     * The two departments publish the same taxable valuation — multiply the profile report's
     * `assessed_valuation_per_pupil` by its enrolled ADM and `total_value` comes back to 1.000
     * for all 606 districts. The pupil counts differ: Columbus is 43,019 to the Department of
     * Education and 71,947 here, Youngstown 4,322 against 9,655. Taxation counts children
     * residing in the district; Education counts the ones it teaches, and the gap is charter,
     * voucher and open-enrolment-out students.
     */
    adm: num,
  })
  .strict();

/**
 * What the mechanism the Fair School Funding Plan replaced would charge this district today.
 *
 * `regime-diff` holds the plan's own base cost fixed and swaps the local share for the
 * charge-off: a flat statutory millage, uniform statewide, against assessed valuation. A
 * counterfactual at current inputs, not a reconstruction of any year the charge-off governed.
 *
 * It belongs beside the property tax because it *was* a property tax calculation, and because
 * its documented failure is a millage fact: the rate was uniform while H.B. 920 made effective
 * rates anything but, so a district whose own rate had fallen below it was charged for revenue
 * it could not collect.
 */
export const RegimeCounterfactualSchema = z
  .object({
    /** The statutory rate the counterfactual runs at — the charge-off's terminal 23 mills. */
    charge_off_mills: num,
    /** Deemed local share per pupil: the rate against valuation. */
    charge_off_local_share: maybeNum,
    /** Local capacity per pupil as the plan measures it. `null` where the minimum share binds. */
    local_capacity: maybeNum,
    /** Base cost aid per pupil under the charge-off, floored at zero — it had no minimum. */
    aid_charge_off: maybeNum,
    /** Base cost aid per pupil as the plan computes it. */
    aid_fsfp: maybeNum,
    /** Plan minus charge-off. Positive means the district gained by the change. */
    difference: maybeNum,
    /** What the one aligned component fails to explain. Zero is the expected answer. */
    residual: maybeNum,
    /** Whether the charge-off would run past the whole base cost it is subtracted from. */
    exceeds_base_cost: z.boolean(),
    /** Effective Class I mills short of the rate it would be charged at. The phantom revenue. */
    mills_short_of_charge_off: maybeNum,
  })
  .strict();

/**
 * H.B. 920 run against one district by the `millage` crate, rather than described.
 *
 * Every other property-tax figure in the feed is a published column copied across. These are
 * computed, which is why they can say three things no published column states: how much of the
 * voted rate the reduction factors have taken, what those factors alone predict for the current
 * tax year, and how far the charged rate departs from that prediction.
 *
 * The residual is the field worth reading. Reduction factors reach neither new construction nor
 * newly voted millage — the statute exempts both — so the gap between predicted and observed is
 * not error. It is the millage the factors do not touch, and its sign says which kind.
 */
export const MillageAnalysisSchema = z
  .object({
    /** The tax year `observed_rate` and `predicted_rate` describe. */
    tax_year: z.number().int(),
    /** Effective Class I rate the prior year — the base the prediction runs from. */
    prior_rate: num,
    /** Effective Class I rate this year, as Table SD-1 publishes it. */
    observed_rate: num,
    /** What reduction factors alone predict, held at the statutory floor. */
    predicted_rate: num,
    /** `observed_rate - predicted_rate`, in mills. What the factors cannot account for. */
    residual: num,
    /** Whether the floor is what stopped the reduction. */
    at_floor: z.boolean(),
    /** Fraction of the voted rate H.B. 920 has taken. `null` without a profile row. */
    cumulative_reduction: maybeNum,
    /** What one mill raises per pupil — over SD-1's own ADM, matching `value_per_pupil`. */
    yield_per_mill_per_pupil: num,
  })
  .strict();

/**
 * Where a district's operating money went in FY2025, per pupil, by function.
 *
 * The report card's spending file — a different source and basis from the audited actuals in
 * `finances`, and a per-pupil figure rather than a total. The two answer different questions and
 * adding them would double-count, which is why they are separate blocks and separate cards.
 *
 * `classroom_instruction` and `nonclassroom` are the department's own roll-ups and partition
 * operating spending exactly; the named functions sit inside one or the other.
 */
export const SpendingByFunctionSchema = z
  .object({
    /** Unweighted ADM — the headcount denominator, not the need-weighted one. */
    adm: num,
    operating_per_pupil: num,
    classroom_instruction: num,
    nonclassroom: num,
    instruction: num,
    pupil_support: num,
    instructional_staff_support: num,
    general_admin: num,
    school_admin: num,
    operations_maintenance: num,
    pupil_transportation: num,
    other_support: num,
    food_service: num,
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
    /** TY2023 and TY2024, oldest first. Empty where the district is absent from SD-1. */
    property_tax: z.array(PropertyTaxYearSchema),
    /** `null` for the two districts with no report-card spending row. */
    spending_by_function: SpendingByFunctionSchema.nullable(),
    /** The state's share of base cost alone, before every categorical. */
    base_cost_state_share: num,
    /** Targeted assistance, special education, DPIA, English learner, gifted, career-technical. */
    categorical_funding: num,
    formula_aid_per_pupil: num,
    realized_aid_per_pupil: num,
    guarantee: num,
    on_guarantee: z.boolean(),
    /** At or below the statutory floor, so reduction factors have stopped operating. */
    at_millage_floor: z.boolean(),
    /** Above it by less than a twentieth of a mill, where the binary stops meaning anything. */
    near_millage_floor: z.boolean(),
    at_minimum_state_share: z.boolean(),
    valuation_per_pupil: maybeNum,
    effective_class1_millage: maybeNum,
    /** The rate voters approved, which is not the rate anyone pays. */
    voted_operating_millage: maybeNum,
    /** The calculator run against this district. `null` without two tax years. */
    millage: MillageAnalysisSchema.nullable(),
    /** What the replaced mechanism would charge. `null` without a valuation. */
    regime: RegimeCounterfactualSchema.nullable(),
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

/**
 * One state's school finance, from the Census Bureau's Annual Survey of School System Finances.
 *
 * Money is in **thousands of dollars**, as the survey reports it. Enrolment is a headcount.
 */
export const StateFinanceSchema = z
  .object({
    fips: z.string().regex(/^\d{2}$/),
    name: z.string(),
    systems: z.number().int().positive(),
    enrollment: num,
    total_revenue: num,
    federal_revenue: num,
    state_revenue: num,
    local_revenue: num,
    /** Zero for the twelve states whose districts are funded by a parent government. */
    property_tax_revenue: num,
    parent_government_revenue: num,
    current_spending: num,
  })
  .strict();

/**
 * Where Ohio sits among the states — the only federal source in this feed.
 *
 * Every other figure here is Ohio describing itself, which means the corpus could restate the
 * *DeRolph* claim about over-reliance on local property tax and never test it. "Too heavily"
 * needs something to compare against.
 */
export const NationalSchema = z
  .object({
    fiscal_year: z.number().int(),
    states: z.array(StateFinanceSchema).length(51),
    ohio_local_rank: z.number().int().positive(),
    ohio_state_rank: z.number().int().positive(),
    ohio_spending_rank: z.number().int().positive(),
    /** Over `independent_states`, not over all 51 — see `StateFinanceSchema.property_tax_revenue`. */
    ohio_property_tax_rank: z.number().int().positive(),
    independent_states: z.number().int().positive(),
    national_local_share: num,
    national_state_share: num,
    national_spending_per_pupil: num,
  })
  .strict();

/** Statewide aggregates, so a district can be positioned without recomputing. */
export const StatewideSchema = z
  .object({
    districts: z.number().int().positive(),
    on_guarantee: z.number().int().nonnegative(),
    at_millage_floor: z.number().int().nonnegative(),
    near_millage_floor: z.number().int().nonnegative(),
    /** The rate voters approved, statewide median. */
    median_voted_millage: num,
    /** The rate anyone pays, statewide median. The gap between the two is H.B. 920. */
    median_effective_millage: num,
    /** Median of the per-district ratio — deliberately not one minus the ratio of the medians. */
    median_millage_reduction: num,
    /** What one mill raises per pupil: the median and the two ends of the range. */
    median_yield_per_mill: num,
    min_yield_per_mill: num,
    max_yield_per_mill: num,
    /** Median taxable value per pupil on Table SD-1's denominator, not the profile report's. */
    median_sd1_value_per_pupil: num,
    /** Districts whose effective rate is below the rate the charge-off would deem them able. */
    below_charge_off_rate: z.number().int().nonnegative(),
    /** Districts the charge-off would leave with no base cost aid at all. */
    charge_off_exceeds_base_cost: z.number().int().nonnegative(),
    /** Median change in base cost aid per pupil, charge-off to plan. */
    median_regime_difference: num,
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
    /** Where Ohio sits among the states. `null` if the Census fixture is absent. */
    national: NationalSchema.nullable(),
    districts: z.array(DistrictSchema).min(1),
  })
  .strict();

export type DistrictOutcome = z.infer<typeof DistrictOutcomeSchema>;
export type OutcomeStatewide = z.infer<typeof OutcomeStatewideSchema>;
export type FinanceYear = z.infer<typeof FinanceYearSchema>;
export type BaseCostBuildUp = z.infer<typeof BaseCostBuildUpSchema>;
export type PropertyTaxYear = z.infer<typeof PropertyTaxYearSchema>;
export type MillageAnalysis = z.infer<typeof MillageAnalysisSchema>;
export type RegimeCounterfactual = z.infer<typeof RegimeCounterfactualSchema>;
export type SpendingByFunction = z.infer<typeof SpendingByFunctionSchema>;
export type District = z.infer<typeof DistrictSchema>;
export type Statewide = z.infer<typeof StatewideSchema>;
export type StateFinance = z.infer<typeof StateFinanceSchema>;
export type National = z.infer<typeof NationalSchema>;
export type Deflator = z.infer<typeof DeflatorSchema>;
export type PolicyShape = z.infer<typeof PolicyShapeSchema>;
export type Checkpoint = z.infer<typeof CheckpointSchema>;
export type ForecastCheckpoint = z.infer<typeof ForecastCheckpointSchema>;
export type ProjectionMeta = z.infer<typeof ProjectionMetaSchema>;
export type Bundle = z.infer<typeof BundleSchema>;
