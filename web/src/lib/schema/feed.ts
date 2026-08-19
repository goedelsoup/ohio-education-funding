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
 *
 * Both are **fractions**, as every share in this feed is. They were not always: the report card
 * publishes 0 to 100 and the bundle passed that through until contract `35.0.0`, so two fields of
 * the same name sat in one document 100× apart, both `maybeNum` and neither saying which it was.
 * `sharesAreFractions` in `tests/unit/schema.spec.ts` is what keeps that from coming back.
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
    /**
     * Share of taxable value the charge-off reaches, after the reappraisal phase-in.
     *
     * The charge-off applied to **recognized valuation**, not to total taxable value: a
     * revaluation's inflationary increase is phased in over three years. One where the phase-in
     * has finished, below one where it has not.
     */
    recognized_share: num,
    /** Tax year the district's county last reappraised or updated. */
    reappraisal_year: num,
    /** Per pupil the charge-off would be overstated by if run on total taxable value. */
    overstated_by: maybeNum,
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
    /**
     * The county the department attributes the district to.
     *
     * One county per district — the department's own simplification, since school district
     * boundaries cross county lines freely and the calculator picks one anyway. Good enough to
     * group peers by, and not good enough to sum into a figure called the county's, which is why
     * `/county/…` compares districts rather than reporting county totals.
     */
    county: z.string().min(1),
    /**
     * The payments outside foundation funding: the performance supplement and the two enrolment
     * supplements.
     *
     * `[H] Foundation Funding` is base cost plus the six categoricals, and the guarantee holds a
     * district at it. These sit in `[R] Total State Support` instead, so nothing cushions a fall
     * in either — a district that drops a star, or slips below 3% growth, loses the money.
     */
    supplements: z
      .object({
        /** $13 a pupil times the greater of the two ratings, for districts clearing any of three routes. */
        performance: num,
        performance_eligible: z.boolean(),
        stars: maybeNum,
        progress: maybeNum,
        /** $40 a pupil, every district, no test. */
        base_funding: num,
        /** The three-year enrolment change the 3% test is applied to. */
        enrollment_change: num,
        growth_eligible: z.boolean(),
        /** $250 on every pupil — not the pupils gained — for a district that cleared 3%. */
        growth: num,
        /** What clearing it would have paid a district that did not. `null` where it did. */
        growth_forgone: maybeNum,
      })
      .strict(),
    /**
     * Where this district sits among America's, on federal definitions.
     *
     * Every other figure in this feed is Ohio describing itself, which cannot say whether Ohio is
     * unusual. This is 10,382 comparable school districts in every state on the Census Bureau's
     * own definitions.
     *
     * Three caveats. The year is **FY2022** against the model's FY2027. The denominator is the
     * **federal** fall membership, not Ohio's ADM — never show a per-pupil figure from here beside
     * one from the funding model without saying so. And the comparison set excludes charter
     * agencies and non-unified districts; `null` for the one Ohio K-8 district it leaves out.
     */
    national: z
      .object({
        local_share: num,
        local_share_percentile: num,
        revenue_per_pupil: num,
        revenue_per_pupil_percentile: num,
        spending_per_pupil: num,
        spending_per_pupil_percentile: num,
      })
      .strict()
      .nullable(),
    /**
     * The guarantee's machinery, and the second hold-harmless stacked on it.
     *
     * The guarantee is the FY2021 funding base less an **open-enrolment clawback** less foundation
     * funding — the clawback charges the full statewide average base cost per pupil for every FTE
     * lost beyond a threshold, and reaches 43 districts. Above it sits a second hold-harmless
     * against a larger FY2021 base that includes transportation, reaching 144 districts of which
     * 17 are not on the guarantee at all.
     */
    transition: z
      .object({
        funding_base: num,
        open_enrollment_prior: num,
        open_enrollment_current: num,
        open_enrollment_threshold: num,
        open_enrollment_adjustment: num,
        fy21_funding_base: num,
        transition_supplement: num,
      })
      .strict(),
    /**
     * Preschool special education: a flat $4,000 a pupil plus the six weights at half, prorated.
     *
     * The flat component is 69% of the program and is not reduced by the state share, so for most
     * of what this pays the wealthiest district and the poorest are funded identically. And the
     * proration no longer fits: the sheet carries a $147.5m appropriation limit beside the factor,
     * and at the stated factor the program totals $148.4m.
     */
    preschool_special_education: z
      .object({
        adm: z.tuple([num, num, num, num, num, num]),
        aid: z.tuple([num, num, num, num, num, num]),
        total: num,
        /** What the flat $4,000 component alone is worth. */
        flat_component: num,
        /** What the program would pay without the proration. */
        unprorated: num,
      })
      .strict(),
    /**
     * Transportation, the largest thing outside foundation funding.
     *
     * $726m, plus $183m of special education transportation — transportation alone is larger than
     * special education, making it the second-largest single program in Ohio's school funding.
     * Two competing rate bases with the district paid the greater, a 50% state minimum share
     * against the formula's 10%, two supplements rewarding opposite things, its own guarantee, and
     * a proration factor meaning the appropriation did not cover the special education line.
     */
    transportation: z
      .object({
        public_riders: num,
        /** Weighted double. */
        nonpublic_riders: num,
        /** Weighted one and a half. */
        community_riders: num,
        weighted_riders: num,
        /** The two competing bases. The district is paid the greater, and which one is invisible. */
        per_rider_base: num,
        per_mile_base: num,
        paid_on_miles: z.boolean(),
        /** The state share after the 50% floor, which binds for most of the state. */
        effective_state_share: num,
        school_bus: num,
        mass_transit: num,
        other: num,
        efficiency: num,
        density: num,
        efficiency_index: num,
        district_density: num,
        /** A second transitional guarantee, on a FY2021 base. */
        fy21_base: num,
        guarantee: num,
        total: num,
        special_education: num,
        /** What the special education line would have been without the proration. */
        special_education_unprorated: num,
      })
      .strict(),
    /**
     * The Ohio House districts this district lies in, largest share first.
     *
     * Usually one — 270 of 609 districts sit inside a single House district — and up to eleven,
     * which is Columbus. Derived from census blocks rather than published; see the feed's
     * top-level `house_districts` for what that supports.
     */
    house_districts: z.array(
      z.object({ number: z.string().min(1), share: num }).strict(),
    ),
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
    /**
     * The part of `categorical_funding` priced in the statewide average base cost per pupil.
     *
     * Special education, English learners and career-technical — each `weight × $8,241.61 ×
     * count × state share`. A base cost lever moves these too, and `apply()` scales them. Sent
     * from the bundle rather than re-derived here so the Rust and TypeScript implementations
     * cannot disagree about which programs count.
     */
    base_cost_denominated_categoricals: num,
    /**
     * Special education's six weighted categories: ADM and the aid each produces.
     *
     * The weights span a factor of sixteen — 0.2435 for Category 1 against 3.9554 for Category 6
     * — and the money runs against them. Category 6 is 15% of pupils statewide and 48% of the
     * program; Category 2 is 65% of pupils and 34%.
     */
    special_education: z
      .object({
        adm: z.tuple([num, num, num, num, num, num]),
        aid: z.tuple([num, num, num, num, num, num]),
      })
      .strict(),
    /**
     * Career-technical's five categories, plus associated services at a sixth weight.
     *
     * Weighted against a **career-technical** base cost of $9,855.62 rather than the $8,241.61
     * every other weighted categorical uses, so a CTE weight and a special education weight are
     * not on the same scale.
     */
    career_technical: z
      .object({
        fte: z.tuple([num, num, num, num, num]),
        aid: z.tuple([num, num, num, num, num]),
        associated_services: num,
      })
      .strict(),
    /**
     * English learners' three categories, whose weights **descend** — 0.2104, 0.1577, 0.1053.
     *
     * Category 1 is the most recently arrived learner and is funded at twice Category 3, so the
     * program tapers as need persists. Every other weighted categorical runs the other way.
     */
    english_learners: z
      .object({
        adm: z.tuple([num, num, num]),
        aid: z.tuple([num, num, num]),
      })
      .strict(),
    /**
     * DPIA: a blend of two poverty counts, and an index that is **squared**.
     *
     * Aid scales with the square of relative poverty, so a district at twice the state's rate
     * scores four times the index rather than twice.
     */
    dpia: z
      .object({
        economically_disadvantaged_adm: num,
        directly_certified_adm: num,
        weighted_adm: num,
        percentage: num,
        index: num,
      })
      .strict(),
    /**
     * Targeted assistance: two tiers that measure different things and add.
     *
     * The capacity tier pays 0.8% of the shortfall below the median district's *total* weighted
     * wealth, phased by district size. The wealth tier is a rate against wealth per **resident**
     * pupil, paid on **enrolled** pupils — two counts, one line apart in the same formula.
     */
    targeted_assistance: z
      .object({
        property_valuation: num,
        federal_gross_income: num,
        weighted_wealth: num,
        capacity_index: num,
        capacity_amount: num,
        wealth_per_pupil: num,
        wealth_index: num,
        wealth_amount: num,
        resident_adm: num,
        supplement_eligible: z.boolean(),
      })
      .strict(),
    /**
     * Gifted: two per-pupil amounts and three kinds of unit, with floors and a cap.
     *
     * A unit is a headcount entitlement priced like a salary. Coordinator units are floored at 0.5
     * and capped at 8; specialist units are floored at 0.3 in each band. So a district that
     * identifies no gifted pupils still draws $93,993 before its state share — gifted is the one
     * categorical with a floor rather than a proportion.
     */
    gifted: z
      .object({
        identification: num,
        referral: num,
        fte_k8: num,
        fte_9_12: num,
        coordinator_units: num,
        coordinator_aid: num,
        specialist_k8_units: num,
        specialist_k8_aid: num,
        specialist_9_12_units: num,
        specialist_9_12_aid: num,
        entirely_on_the_floor: z.boolean(),
      })
      .strict(),
    /**
     * `[a] Enrolled ADM` — the pupil count four of the six categoricals are paid on.
     *
     * Not `adm`, which averages three years, and not `current_year_adm`. It equals the latter in
     * 608 of 609 districts and differs in Akron by fifty pupils, so anything computed per pupil
     * from a categorical amount belongs over this one.
     */
    categorical_adm: num,
    /**
     * The same total, as its six parts.
     *
     * It was a residual for eight phases — core foundation funding less the state share of base
     * cost, exact and uninterrogable. The six behave nothing alike: targeted assistance is
     * equalisation and is zero for 135 districts; DPIA tracks poverty. A page showing the sum
     * cannot say which kind of money a district is getting.
     */
    categoricals: z
      .object({
        targeted_assistance: num,
        special_education: num,
        dpia: num,
        english_learners: num,
        gifted: num,
        career_technical: num,
      })
      .strict(),
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
    /** Districts receiving nothing from targeted assistance — it is equalisation and switches off. */
    districts_without_targeted_assistance: z.number().int().nonnegative(),
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
/**
 * One of Ohio's 99 House districts, with school funding apportioned across it.
 *
 * # These are estimates and nothing publishes them
 *
 * The department computes funding per school district and stops. No House district is a unit of
 * account anywhere in Ohio's funding system, and 339 of 609 school districts straddle two or more
 * of them — so a House district figure has to be derived, by splitting each school district across
 * the House districts it overlaps in proportion to under-18 population from the 2020 census.
 *
 * The one guarantee is that the split is exact in aggregate: every school district's shares sum to
 * one, so the 99 House districts sum to the statewide total to the cent. Everything else is an
 * estimate, and any page showing one says so.
 */
export const HouseDistrictSchema = z
  .object({
    number: z.string().min(1),
    adm: num,
    realized_aid: num,
    base_cost_state_share: num,
    categorical_funding: num,
    guarantee: num,
    districts_on_guarantee: z.number().int().nonnegative(),
    districts_at_minimum_state_share: z.number().int().nonnegative(),
    districts_wholly_inside: z.number().int().nonnegative(),
    members: z.array(
      z
        .object({
          irn: z.string().min(1),
          name: z.string().min(1),
          /** How much of the *school district* is here. */
          share: num,
          /** How much of *this House district's* pupils that school district provides. */
          share_of_house_district: num,
          adm: num,
          realized_aid: num,
          wholly_inside: z.boolean(),
        })
        .strict(),
    ),
  })
  .strict();

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
    /**
     * What the policy did to the guarantee population, which `unmoved` does not say.
     *
     * A formula district can be unmoved because the lever pulled does not touch it; a guarantee
     * district is unmoved because nothing pulled can touch it until the formula overtakes its
     * frozen baseline. These three separate the two cases, and the page must reproduce them —
     * which means reproducing both runs rather than only the perturbed one.
     */
    held_throughout: z.number().int().nonnegative(),
    lifted_off: z.number().int().nonnegative(),
    pushed_on: z.number().int().nonnegative(),
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

/**
 * One year of the Census Bureau's survey of Ohio school systems.
 *
 * The only part of the feed that reaches before FY2020, and the only part not measured by the
 * department's own formula. Roughly 950 agencies a year rather than 609 districts, on the
 * Bureau's enrollment count rather than ADM, and every figure here computed over the subset the
 * survey marks comparable. It does not reconcile with the FY2027 model and is not meant to.
 */
export const HistoryYearSchema = z
  .object({
    fiscal_year: z.number().int(),
    districts: z.number().int().positive(),
    /** Shares of total revenue. They sum to 1 up to rounding. */
    local_share: num,
    state_share: num,
    federal_share: num,
    /** Mean local revenue per pupil, poorest and richest quartile of districts. */
    poorest_local_per_pupil: num,
    richest_local_per_pupil: num,
    /** The gap between them, and how much of it each level of government closes. */
    gap_per_pupil: num,
    state_closes_per_pupil: num,
    federal_closes_per_pupil: num,
  })
  .strict();

/**
 * One October of the free and reduced-price lunch report, MR-81.
 *
 * The third population and the third enrollment count in this feed, and the only series whose
 * denominator changes inside itself — `adm` through FY2009, `ce` from FY2010. `basis` is required
 * on every row rather than inferred from the year, because a consumer that has to know the
 * cutover to read a row is a consumer that will get it wrong once.
 *
 * A share, not a count and not a dollar. Nothing here may be compared to the formula side, which
 * counts 609 traditional districts on ADM; `sponsors` here includes community schools and county
 * boards of developmental disabilities.
 *
 * # Why `share` is nullable
 *
 * From FY2012 the department publishes MR-81 as three files, and only one of them still counts
 * applications. Community-eligibility sponsors collect none at all — every child eats free — so
 * their approval columns are zero by construction, and adding the three streams gives a share that
 * falls thirteen points in three years for a reason that is not poverty. Those Octobers carry
 * `floor` and `ceiling` and no share. `streams` says which kind of year a row is, and it is the
 * field to branch on rather than the year.
 */
export const MealProgramYearSchema = z
  .object({
    fiscal_year: z.number().int(),
    /** Public sponsors the year is computed over, after excluding published corruption. */
    sponsors: z.number().int().positive(),
    /**
     * The denominator, summed over those sponsors.
     *
     * Present so the share can be checked and so `denominators.ts` can see this block at all —
     * its guard walks field names, and `share` is not one it recognises.
     */
    enrollment: num,
    /** Approvals, summed over those sponsors. Short by the community stream from FY2012. */
    approved: num,
    /**
     * Directly certified children under community eligibility, and zero before FY2012.
     *
     * Not an approval. Direct certification reaches families already on SNAP, TANF, foster care or
     * a homeless roll; an application reaches anyone under the income line who files one.
     */
    identified: num,
    /** `approved` over `enrollment`, and `null` for the Octobers published as three files. */
    share: num.nullable(),
    /** The lowest share the source supports: approvals plus directly certified children. */
    floor: num,
    /** The highest: what every sponsor may claim for, capped at enrollment school by school. */
    ceiling: num,
    /**
     * The share of the October's enrollment under sponsors that collect no applications.
     *
     * Zero through FY2011 and a sixth by FY2014. The size of the hole in `approved`, and it grows
     * because community eligibility is open to schools whose poverty is already high.
     */
    without_applications: num,
    /** How many files the October was published as. One through FY2011, three from FY2012. */
    streams: z.number().int().positive(),
    /** Which denominator that is. Not inferred from the year — see above. */
    basis: z.enum(["adm", "ce"]),
  })
  .strict();

/**
 * One fiscal year of what the General Assembly appropriated to the department.
 *
 * An **input** to the funding system, where every other block here is an output of it. It is what
 * was set aside, not what any district received: an appropriation is a ceiling, and the formula's
 * proration factor exists because at least one line has been a residual claimant. Differencing
 * this against a payment produces a number that means nothing.
 *
 * `source` names which publication answers for the year — `workbook` for the greenbooks and
 * budget workbooks, `catalog` for the four years only the Catalog of Budget Line Items reaches
 * (FY2006-07 and FY2012-13). Where both speak they agree to the cent, so this is not a confidence
 * signal; it is so a reader can see that four years rest on a different document.
 */
export const AppropriationYearSchema = z
  .object({
    fiscal_year: z.number().int(),
    /** Everything the department was appropriated, excluding the property tax reimbursements. */
    enacted: num,
    /** The formula's own lines: GRF 200550 (200501 before FY2006) and Lottery 200612. */
    foundation_funding: num,
    /** How many line items the total is over. */
    items: z.number().int().positive(),
    source: z.enum(["workbook", "catalog", "act"]),
  })
  .strict();

/**
 * One appropriation line the department is funded through, and the act that created it.
 *
 * `general_assembly` is `null` for roughly half the lines, because the Catalog's legal basis cites
 * only their current authority. Carried as unknown rather than filled from an earlier edition with
 * the same number: a line item number is reused, so inheriting an origin down a number would
 * attribute one programme's founding act to another's.
 *
 * `discontinued` is the publisher's own label and does **not** distinguish abolition from
 * consolidation — a line folded into another is discontinued too.
 */
export const AppropriationLineSchema = z
  .object({
    fund: z.string().min(1),
    ali: z.string().min(1),
    name: z.string().min(1),
    /** The act as the Catalog writes it, or empty when it names none. */
    established_by: z.string(),
    general_assembly: z.number().int().nullable(),
    /** The year that General Assembly convened. `null` alongside `general_assembly`. */
    convened: z.number().int().nullable(),
    discontinued: z.boolean(),
  })
  .strict();

/** The whole feed. */
/**
 * What year one block of the feed is measured in.
 *
 * Ohio reckons three ways and they do not line up: a tax year is a calendar year whose revenue
 * reaches the district in the *following* fiscal year, a school year straddles two calendar years
 * and is published as `2024-25`, and a fiscal year runs July to June and is named for the June.
 * Every one of those is "2024" to somebody.
 *
 * `label` is a string and not a number because a school year has no single number, and `kind` is
 * carried separately so a consumer never has to infer the reckoning from the shape of the label.
 */
export const SeriesYearSchema = z
  .object({
    series: z.string().min(1),
    kind: z.enum(["fiscal", "tax", "school"]),
    label: z.string().min(1),
    source: z.string().min(1),
  })
  .strict();

export const BundleSchema = z
  .object({
    contract_version: z.string().min(1),
    provenance: z.string().min(1),
    /**
     * The year the *formula* computes, and the year of nothing else on a district page.
     *
     * See {@link SeriesYearSchema}: a district page shows this beside a 2024 tax year, a 2024-25
     * report card, an FY2022 Census survey and a forecast reaching back to FY2020.
     */
    fiscal_year: z.number().int(),
    /** What year every other block is measured in, by series key. Sorted by key. */
    series_years: z.array(SeriesYearSchema).min(1),
    statewide: StatewideSchema,
    checkpoints: z.array(CheckpointSchema),
    /** `null` disables the band: this feed cannot be projected. */
    projection: ProjectionMetaSchema.nullable(),
    /** `null` means the feed can only be shown in nominal dollars. */
    deflator: DeflatorSchema.nullable(),
    /** Where Ohio sits among the states. `null` if the Census fixture is absent. */
    national: NationalSchema.nullable(),
    /** The survey year by year, oldest first. Empty if the panel is absent. */
    history: z.array(HistoryYearSchema),
    /** What the General Assembly appropriated, by fiscal year, oldest first. Empty if absent. */
    appropriations: z.array(AppropriationYearSchema),
    /** The lines themselves, with the act that created each. Empty if absent. */
    appropriation_lines: z.array(AppropriationLineSchema),
    /** The meal-program poverty share by October, oldest first. Empty if absent. */
    meal_program: z.array(MealProgramYearSchema),
    house_districts: z.array(HouseDistrictSchema),
    /** The same for the Senate: 33 seats, each exactly three House districts. */
    senate_districts: z.array(HouseDistrictSchema),
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
export type HistoryYear = z.infer<typeof HistoryYearSchema>;
export type MealProgramYear = z.infer<typeof MealProgramYearSchema>;
export type AppropriationYear = z.infer<typeof AppropriationYearSchema>;
export type AppropriationLine = z.infer<typeof AppropriationLineSchema>;
export type Deflator = z.infer<typeof DeflatorSchema>;
export type PolicyShape = z.infer<typeof PolicyShapeSchema>;
export type Checkpoint = z.infer<typeof CheckpointSchema>;
export type ForecastCheckpoint = z.infer<typeof ForecastCheckpointSchema>;
export type ProjectionMeta = z.infer<typeof ProjectionMetaSchema>;
export type SeriesYear = z.infer<typeof SeriesYearSchema>;
export type Bundle = z.infer<typeof BundleSchema>;
