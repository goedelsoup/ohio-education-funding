/**
 * Every per-pupil figure in the feed, and which pupil count it divides by.
 *
 * # Why this file exists
 *
 * Six different pupil counts appear in this project's sources, and this site has shipped two
 * comparisons across them. The first put the report card's need-weighted spending beside the
 * profile report's headcount spending; the second printed Table SD-1's valuation per pupil
 * against a statewide median computed on the Department of Education's ADM, which is a different
 * number for 528 of 609 districts and a factor of 2.3 different for Youngstown.
 *
 * Neither was caught by a type, a schema or a test, because both figures are numbers, both are
 * per pupil, and both are correct. Only the pairing is wrong. The failure has no signature except
 * knowing what each denominator is — so that knowledge is written down here, and a test asserts
 * that no per-pupil field escapes it.
 *
 * # What the guard actually catches
 *
 * `tests/unit/denominators.spec.ts` walks the feed for anything that looks like a per-pupil
 * quantity and fails if it is not declared below. A new field added in Rust therefore stops the
 * build until someone states its denominator, which is the moment the question is cheap to
 * answer. It also checks the declared comparator pairs still share one.
 *
 * It cannot check prose. A sentence putting two figures side by side is still a human's
 * responsibility; what this removes is the excuse of not knowing.
 *
 * # One block is absent from this file on purpose
 *
 * `appropriations` carries no per-pupil figure and no pupil count, so the walk below never reaches
 * it and nothing here declares it. That is the design rather than an oversight: a statewide
 * appropriation divided by a pupil count would be a per-pupil figure on a *seventh* denominator,
 * rendered a scroll away from the formula's own per-pupil numbers on this site's longest-running
 * page.
 *
 * The temptation is specific and will arrive: "state aid per pupil, 2002 to 2027" is the obvious
 * chart to want from that series, and the obvious denominator to reach for is whichever ADM is
 * nearest to hand. An appropriation is also a ceiling rather than a payment, so the result would
 * be a per-pupil figure for money no pupil necessarily received. If a per-pupil appropriation is
 * ever wanted, it needs an entry in `DENOMINATORS` naming the count and the year it comes from,
 * like everything else here.
 */

/** The pupil counts the sources use. Distinct measures, not variants of one. */
export const DENOMINATORS = {
  "base-cost-adm": {
    label: "Base cost ADM",
    source: "Department of Education, FY2027 calculator",
    note: "Funded rather than enrolled: a three-year average, and the count the funding formula divides by.",
    field: "districts[].adm",
  },
  "enrolled-adm-fy24": {
    label: "Enrolled ADM, FY2024",
    source: "Department of Education, District Profile Report",
    note: "A headcount of pupils the district teaches. Differs from base cost ADM by a median 1.6% and by 27% at the extreme.",
    field: "districts[].adm_history[0]",
  },
  "unweighted-adm-fy25": {
    label: "Unweighted ADM, FY2025",
    source: "Department of Education, report card",
    note: "The report card's headcount. Not the same year or file as enrolled ADM FY2024.",
    field: "districts[].spending_by_function.adm",
  },
  "weighted-adm-fy25": {
    label: "Need-weighted ADM, FY2025",
    source: "Department of Education, report card",
    note: "Weighted upward for disadvantage, English learners and disability. The department's own headline per-pupil denominator, and against a composition-driven outcome it is substantially a composition proxy.",
    field: "not carried directly; implied by per_equivalent_pupil",
  },
  "sd1-adm": {
    label: "Table SD-1 ADM",
    source: "Department of Taxation",
    note: "Children resident in the district, including those attending community schools or using scholarships. 71,947 for Columbus against Education's 43,019.",
    field: "districts[].property_tax[].adm",
  },
  "categorical-enrolled-adm": {
    label: "Enrolled ADM, `[a]`",
    source: "Department of Education, FY2027 calculator, `ADM Data` column 3",
    note: "The count targeted assistance, gifted, career-technical and English learners are paid on. Equal to `[b3] FY26 Enrolled ADM` in 608 of 609 districts and fifty pupils apart in Akron — one number entered twice and corrected once.",
    field: "districts[].categorical_adm",
  },
  "ta-resident-adm": {
    label: "Resident ADM",
    source: "Department of Education, FY2027 calculator, `Targeted_Assistance`",
    note: "Enrolled ADM less pupils open-enrolling in, plus those open-enrolling out. Targeted assistance measures wealth per resident pupil and then pays on the enrolled count — two denominators, one line apart in the same formula.",
    field: "districts[].targeted_assistance.resident_adm",
  },
  "f33-fall-membership": {
    label: "Fall membership, `V33`",
    source: "NCES, School District Finance Survey (F-33), FY2022",
    note: "The federal count for a single district, five years before the model it sits beside. Not Ohio's enrolled ADM and not the report card's headcount — and the only denominator here that is not Ohio's own.",
    field: "districts[].national.revenue_per_pupil",
  },
  "census-fall-enrollment": {
    label: "Fall enrolment",
    source: "U.S. Census Bureau, F-33",
    note: "The federal survey's own count, on its own definitions, for every state.",
    field: "national.states[].enrollment",
  },
  "meal-program-count": {
    label: "Meal-program denominator, `AdmCount` then `CECount`",
    source: "Ohio DEW, Office for Child Nutrition, MR-81, FY2001-FY2011",
    note: "The only count here that changes definition inside its own series: `AdmCount` through FY2009, then `CECount` — 'the highest daily number of students with access to the program' — from FY2010, which is neither ADM nor the count before it. The population is sponsors, not districts: community schools and county boards of developmental disabilities are in, and the count rises from 730 to 949 across the window mostly because community schools opened. Every row carries its own `basis` so a reader cannot splice the two halves without being told.",
    field: "meal_program[].enrollment",
  },
  "f33-panel-membership": {
    label: "Fall membership, `V33`, per year",
    source: "U.S. Census Bureau / NCES, School District Finance Survey (F-33), FY2009-FY2022",
    note: "The same measure as `f33-fall-membership` and a different population: every comparable Ohio system in each year of the panel rather than one district in FY2022, so about 950 agencies including community schools and educational service centres. Nothing on the history route may be compared to a figure from the formula side, which counts 609 traditional districts on ADM.",
    field: "history[].poorest_local_per_pupil",
  },
} as const;

export type DenominatorKey = keyof typeof DENOMINATORS;

/**
 * Each per-pupil field in the feed, keyed by the path a walk of the JSON produces.
 *
 * `null` marks a field that *is* a pupil count rather than a quantity divided by one, and a
 * handful of ratios that are dimensionless because both their parts share a denominator — those
 * are the only per-pupil-looking figures that can be compared between districts without asking.
 */
export const FIELD_DENOMINATORS: Record<string, DenominatorKey | null> = {
  // The formula's own figures, all on the funded count.
  "districts[].base_cost_per_pupil": "base-cost-adm",
  "districts[].formula_aid_per_pupil": "base-cost-adm",
  "districts[].realized_aid_per_pupil": "base-cost-adm",

  // The profile report's, on enrolled ADM FY2024.
  "districts[].valuation_per_pupil": "enrolled-adm-fy24",
  "districts[].operating_expenditure_per_pupil": "enrolled-adm-fy24",

  // The report card's, on its two counts. Every figure in the spending-by-function block is per
  // pupil on the unweighted count, including the ones whose names do not say so — see
  // `BLOCK_DENOMINATORS`, which is what a walk keyed on field names cannot see.
  "districts[].spending_by_function.operating_per_pupil": "unweighted-adm-fy25",
  "districts[].spending_by_function.pupil_support": "unweighted-adm-fy25",
  "districts[].spending_by_function.pupil_transportation": "unweighted-adm-fy25",
  "districts[].outcome.per_enrolled_pupil": "unweighted-adm-fy25",
  "districts[].outcome.per_equivalent_pupil": "weighted-adm-fy25",
  "districts[].outcome.per_equivalent_pupil_federal": "weighted-adm-fy25",
  "districts[].outcome.per_equivalent_pupil_state_local": "weighted-adm-fy25",

  // Taxation's, on its own resident count.
  "districts[].property_tax[].value_per_pupil": "sd1-adm",
  "districts[].millage.yield_per_mill_per_pupil": "sd1-adm",

  // Statewide medians. Each must match the district field it is displayed against.
  "statewide.median_valuation_per_pupil": "enrolled-adm-fy24",
  "statewide.median_operating_expenditure_per_pupil": "enrolled-adm-fy24",
  "statewide.median_sd1_value_per_pupil": "sd1-adm",
  "statewide.median_yield_per_mill": "sd1-adm",
  "statewide.min_yield_per_mill": "sd1-adm",
  "statewide.max_yield_per_mill": "sd1-adm",

  // The Census comparison, on the federal survey's count.
  "national.national_spending_per_pupil": "census-fall-enrollment",

  // The per-district national comparison, on the survey's own count for that district.
  //
  // This is the denominator the site is most likely to get wrong next, because the figures look
  // exactly like the ones above them on the same page and are not on the same count, the same
  // definitions, or the same year. `operating_expenditure_per_pupil` is FY2024 on enrolled ADM
  // FY2024; `spending_per_pupil` here is FY2022 on federal fall membership. They are not
  // comparable and the card says so.
  "districts[].national.revenue_per_pupil": "f33-fall-membership",
  "districts[].national.spending_per_pupil": "f33-fall-membership",

  // The historical panel, on the survey's count for each of its years.
  //
  // Three of these were called `gap`, `state_closes` and `federal_closes` when they were written,
  // and the walk did not see them: they are dollars per pupil whose names did not say so. That is
  // this registry's failure mode rather than an exception to it, so the fields were renamed in the
  // Rust instead of exempted here. A per-pupil quantity that hides its denominator in a field
  // name is precisely the thing that shipped twice.
  "history[].poorest_local_per_pupil": "f33-panel-membership",
  "history[].richest_local_per_pupil": "f33-panel-membership",
  "history[].gap_per_pupil": "f33-panel-membership",
  "history[].state_closes_per_pupil": "f33-panel-membership",
  "history[].federal_closes_per_pupil": "f33-panel-membership",

  // The percentiles are dimensionless — a rank. `local_share` and its percentile are too, and are
  // deliberately absent: the walk does not consider them denominator-bearing, and declaring a
  // field it cannot see makes the table a claim about the feed that has stopped being checkable.
  "districts[].national.revenue_per_pupil_percentile": null,
  "districts[].national.spending_per_pupil_percentile": null,

  // Pupil counts themselves, not quantities over one.
  "districts[].adm": null,
  "districts[].current_year_adm": null,
  "districts[].spending_by_function.adm": null,
  "districts[].property_tax[].adm": null,
  "national.states[].enrollment": null,
  "projection.checkpoints[].adm": null,

  // Pupil counts as a series rather than a scalar.
  "districts[].adm_history": null,
  // Six special education ADM counts. Counts, not quantities over one — and the aid beside them
  // is a dollar total per category rather than a per-pupil figure, so neither carries a
  // denominator. The weights that connect them multiply a statewide average base cost per pupil,
  // which is the only per-pupil quantity in the calculation and is not district-specific.
  "districts[].special_education.adm": null,

  // Targeted assistance's own two counts, and the one quantity it expresses per pupil.
  //
  // `wealth_per_pupil` is the case this registry exists for. It is weighted wealth over *resident*
  // ADM, and it sits on the page beside a district's valuation per pupil, which is over enrolled
  // ADM FY2024, and beside its wealth *amount*, which was paid on the enrolled count. Three
  // adjacent figures, three denominators, and only the middle one's name admits to having one.
  "districts[].targeted_assistance.wealth_per_pupil": "ta-resident-adm",
  "districts[].targeted_assistance.resident_adm": null,
  "districts[].categorical_adm": null,

  // DPIA's counts. `percentage` is the blended count over enrolled ADM and `index` is that over
  // the statewide share and squared — both dimensionless, because the denominator cancels.
  "districts[].dpia.economically_disadvantaged_adm": null,
  "districts[].dpia.directly_certified_adm": null,
  "districts[].dpia.weighted_adm": null,

  // Category counts for the three remaining weighted programs. Counts, not quantities over one,
  // and the aid beside each is a dollar total per category rather than a per-pupil figure.
  "districts[].english_learners.adm": null,
  // Preschool pupils by category. Counts, not quantities over one — and a different population
  // from every other ADM here: these children are not in the enrolled ADM the formula funds, which
  // is why the program sits outside it.
  "districts[].preschool_special_education.adm": null,
  "districts[].career_technical.fte": null,
  "districts[].gifted.fte_k8": null,
  "districts[].gifted.fte_9_12": null,

  // Apportioned pupil counts. Counts rather than quantities over one — and estimates, unlike
  // every other count in this table, because a House district is not a unit of account in Ohio's
  // funding system and its pupils have been derived by splitting school districts across census
  // blocks. Nothing on the site divides by one of these, and nothing should: a per-pupil figure
  // over an apportioned denominator would carry the apportionment's error into a ratio that looks
  // like the department's.
  "house_districts[].adm": null,
  "house_districts[].members[].adm": null,
  "senate_districts[].adm": null,
  "senate_districts[].members[].adm": null,

  // The three-year enrolment change the growth supplement's 3% cliff is tested against.
  // Dimensionless: both ends are the same count in different years, so the denominator cancels.
  // It is not the same measure as `enrollment_change`, which is the one-year FY25-to-FY26 change
  // the projection is fitted from — two fields, both a fraction, both named for a change, and
  // three years apart in what they compare.
  "districts[].supplements.enrollment_change": null,

  // The guarantee's open-enrolment clawback. `prior` and `current` are FTE counts and the
  // threshold is measured in the same FTE, so none is a quantity over a pupil count. The
  // adjustment is dollars — and dollars per *lost* FTE, which is not a denominator anything else
  // on the site divides by and must never be shown as a per-pupil figure beside one that is.
  "districts[].transition.open_enrollment_prior": null,
  "districts[].transition.open_enrollment_current": null,
  "districts[].transition.open_enrollment_threshold": null,
  "districts[].transition.open_enrollment_adjustment": null,

  // Dimensionless: a change, and correlations whose parts each cancel their own denominator.
  "districts[].enrollment_change": null,
  "statewide.outcomes.enrolled_spending_vs_performance": null,

  // The meal-program count itself, which is the denominator rather than a quantity over one.
  // Everything else in that block is declared by `BLOCK_DENOMINATORS`.
  "meal_program[].enrollment": null,
};

/**
 * Blocks of the feed where every figure shares one denominator, whatever its field is called.
 *
 * `spending_by_function` is the case that matters: `instruction`, `food_service`,
 * `general_admin` and the rest are all per unweighted pupil, and none of their names says so. A
 * guard keyed on field names cannot see them, so the block is declared whole and the test checks
 * that anything it *does* see inside a declared block agrees with the block.
 */
export const BLOCK_DENOMINATORS: Record<string, DenominatorKey> = {
  "districts[].spending_by_function.": "unweighted-adm-fy25",
  // The second case of the same shape, and the reason `meal_program[].enrollment` exists at all.
  // `share` and `approved` are both over the meal-program count and neither name says so, so the
  // block is declared whole. `sponsors` is a count of sponsors rather than of pupils and the walk
  // does not reach it.
  "meal_program[].": "meal-program-count",
};

/**
 * Pairs the site renders next to each other, which must therefore share a denominator.
 *
 * Every entry is a place a district figure is shown against a statewide comparator, which is the
 * exact shape of the bug this file exists for. The pair that was wrong on 609 pages is the third
 * one: it read `median_valuation_per_pupil` until the divergence was measured.
 */
export const RENDERED_PAIRS: [string, string][] = [
  ["districts[].valuation_per_pupil", "statewide.median_valuation_per_pupil"],
  [
    "districts[].operating_expenditure_per_pupil",
    "statewide.median_operating_expenditure_per_pupil",
  ],
  ["districts[].property_tax[].value_per_pupil", "statewide.median_sd1_value_per_pupil"],
  ["districts[].millage.yield_per_mill_per_pupil", "statewide.median_yield_per_mill"],
  ["districts[].millage.yield_per_mill_per_pupil", "statewide.min_yield_per_mill"],
  ["districts[].millage.yield_per_mill_per_pupil", "statewide.max_yield_per_mill"],
];

/**
 * Per-pupil figures the site renders on the same card but must **not** be compared.
 *
 * {@link RENDERED_PAIRS} asserts that two figures shown together share a denominator. This is the
 * other half: pairs that are legitimately on different counts and appear near each other anyway,
 * recorded so that "these are adjacent and different" is a decision rather than an oversight.
 *
 * The targeted assistance card is the live case. It shows weighted wealth per *resident* pupil two
 * rows above an amount paid on *enrolled* pupils, because that is what the department's formula
 * does — the card's job is to make the discrepancy visible, not to resolve it. Every entry here
 * must be labelled on the page with the count it uses.
 */
export const DELIBERATELY_UNCOMPARABLE: [string, string][] = [
  ["districts[].targeted_assistance.wealth_per_pupil", "districts[].valuation_per_pupil"],
  ["districts[].targeted_assistance.resident_adm", "districts[].categorical_adm"],
  // The history route and the formula side. Not adjacent on any card — they are on separate
  // routes precisely so they cannot be — but recorded because the temptation to put the local
  // revenue gap beside a district's valuation per pupil will arrive, and they are a different
  // population, a different count, and a decade apart.
  ["history[].poorest_local_per_pupil", "districts[].valuation_per_pupil"],
  // The meal-program count against the two panels it sits beside on `/history`. Recorded because
  // the invitation here is stronger than the one above: MR-81 covers FY2001-FY2011 and the Census
  // panel starts in FY2009, so three years overlap and a reader will want to read across them.
  // They are different sponsors on different counts, and the meal-program denominator changes
  // definition inside those three years.
  ["meal_program[].enrollment", "history[].poorest_local_per_pupil"],
];
