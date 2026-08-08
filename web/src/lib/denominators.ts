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
  "census-fall-enrollment": {
    label: "Fall enrolment",
    source: "U.S. Census Bureau, F-33",
    note: "The federal survey's own count, on its own definitions, for every state.",
    field: "national.states[].enrollment",
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

  // Pupil counts themselves, not quantities over one.
  "districts[].adm": null,
  "districts[].current_year_adm": null,
  "districts[].spending_by_function.adm": null,
  "districts[].property_tax[].adm": null,
  "national.states[].enrollment": null,
  "projection.checkpoints[].adm": null,

  // Pupil counts as a series rather than a scalar.
  "districts[].adm_history": null,

  // Dimensionless: a change, and correlations whose parts each cancel their own denominator.
  "districts[].enrollment_change": null,
  "statewide.outcomes.enrolled_spending_vs_performance": null,
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
