//! Building the committed CSV fixtures the calculator crates read.
//!
//! Each builder is a pure function from parsed rows to rows of strings, so it tests without a
//! network or a workbook. Only [`write_csv`] touches disk.
//!
//! # Why a CSV sits in the middle
//!
//! Retrieval is side-effecting and can fail; calculation must be deterministic and auditable.
//! The fixture is the seam. It is also legible in a diff, which a cached workbook is not — when
//! the department reposts a corrected file, the change shows up as changed numbers in a review
//! rather than as a silently different answer.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::io;
use std::path::Path;

use crate::conventions::{cell, cell_number, is_statewide_row, number, rows_by_key};

/// Format a value the way the fixtures are written: fixed decimals, trailing zeros trimmed,
/// blank for absent.
///
/// Blank rather than `0` for absent is load-bearing. A district with no reported valuation and
/// a district whose valuation is nil are different claims, and the calculators read the
/// difference as `Option<f64>`.
///
/// # A trailing zero is not a trailing zero
///
/// The trim only applies past a decimal point. Trimming the string form of an integer turns 10
/// into 1 and 30 into 3 — which is what the predecessor of this function did to the school
/// building count, silently, for every district that happened to have a multiple of ten.
#[must_use]
pub fn format_value(value: Option<f64>, places: usize) -> String {
    let Some(value) = value else {
        return String::new();
    };
    let rendered = format!("{value:.places$}");
    if rendered.contains('.') {
        rendered
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        rendered
    }
}

/// District names go into a comma-separated file, so commas are replaced rather than quoted.
#[must_use]
pub fn clean_name(raw: &str) -> String {
    raw.replace(',', " ").trim().to_string()
}

/// Columns of the FY2027 department-model fixture.
pub const FY27_HEADER: &[&str] = &[
    "irn",
    "district",
    "base_cost_enrolled_adm",
    "school_buildings",
    "adm_kindergarten",
    "adm_grades_1_3",
    "adm_grades_4_8_non_cte",
    "adm_grades_9_12_non_cte",
    "adm_cte",
    "adm_grades_9_12_total",
    "funded_classroom_teachers",
    "funded_special_teachers",
    "teacher_base_cost",
    "aggregate_base_cost",
    "base_cost_per_pupil",
    "temp_transitional_aid_guarantee",
    "enrolled_adm_fy24",
    "enrolled_adm_fy25",
    "enrolled_adm_fy26",
    "assessed_valuation_per_pupil_fy23",
    "core_foundation_funding",
    "base_cost_state_share",
    "total_state_support",
    "total_transfers",
    "service_center_charge",
    "other_adjustments",
    "net_state_funding",
    "capacity_per_pupil",
    "state_share_percentage",
    "valuation_ty25",
    "valuation_ty24",
    "valuation_ty23",
    "agi_ty24",
    "agi_ty23",
    "agi_ty22",
    "tax_returns",
    "federal_median_income",
    "statewide_median_income",
    "benchmark_ratio",
    "capacity_rate",
    "targeted_assistance",
    "special_education",
    "dpia",
    "english_learners",
    "gifted",
    "career_technical",
    "sped_adm_cat1",
    "sped_adm_cat2",
    "sped_adm_cat3",
    "sped_adm_cat4",
    "sped_adm_cat5",
    "sped_adm_cat6",
    "sped_aid_cat1",
    "sped_aid_cat2",
    "sped_aid_cat3",
    "sped_aid_cat4",
    "sped_aid_cat5",
    "sped_aid_cat6",
    "dpia_econ_disadvantaged_adm",
    "dpia_directly_certified_adm",
    "dpia_weighted_adm",
    "dpia_percentage",
    "dpia_index",
    "cte_fte_cat1",
    "cte_fte_cat2",
    "cte_fte_cat3",
    "cte_fte_cat4",
    "cte_fte_cat5",
    "cte_aid_cat1",
    "cte_aid_cat2",
    "cte_aid_cat3",
    "cte_aid_cat4",
    "cte_aid_cat5",
    "cte_associated_services",
    "el_adm_cat1",
    "el_adm_cat2",
    "el_adm_cat3",
    "el_aid_cat1",
    "el_aid_cat2",
    "el_aid_cat3",
    "gifted_adm_k6",
    "gifted_fte_k8",
    "gifted_fte_9_12",
    "gifted_identification",
    "gifted_referral",
    "gifted_professional_development",
    "gifted_coordinator_units",
    "gifted_coordinator_aid",
    "gifted_specialist_k8_units",
    "gifted_specialist_k8_aid",
    "gifted_specialist_9_12_units",
    "gifted_specialist_9_12_aid",
    "ta_open_enrollment_in",
    "ta_open_enrollment_out",
    "ta_fy19_wealth_index",
    "ta_fy19_enrolled_adm",
    "ta_fy19_total_adm",
    "ta_property_valuation",
    "ta_federal_gross_income",
    "ta_weighted_wealth",
    "ta_capacity_index",
    "ta_capacity_amount",
    "ta_wealth_per_pupil",
    "ta_wealth_index",
    "ta_wealth_amount",
    "ta_supplemental",
    "ta_supplement_eligible",
    "categorical_enrolled_adm",
    "county",
    "performance_stars",
    "performance_progress",
    "performance_progress_prior",
    "performance_eligible",
    "performance_supplement",
    "base_funding_supplement",
    "enrolled_adm_fy23",
    "enrollment_change_three_year",
    "growth_supplement_eligible",
    "enrollment_growth_supplement",
    "trans_public_riders",
    "trans_nonpublic_riders",
    "trans_community_riders",
    "trans_weighted_riders",
    "trans_mass_transit_riders",
    "trans_other_riders",
    "trans_bus_miles",
    "trans_assigned_buses",
    "trans_rider_capacity_target",
    "trans_efficiency_index",
    "trans_district_density",
    "trans_square_miles",
    "trans_reported_sped_cost",
    "trans_school_bus",
    "trans_mass_transit",
    "trans_other",
    "trans_efficiency",
    "trans_density",
    "trans_fy21_base",
    "trans_guarantee",
    "trans_total",
    "trans_special_education",
    // Order matters and is not checked by the length assertion: these eight are emitted before
    // the preschool block, and listing them after it put every value in the wrong column while
    // the row length stayed right.
    "funding_base",
    "funding_base_econ_dis",
    "open_enrollment_fte_prior",
    "open_enrollment_fte_current",
    "open_enrollment_threshold",
    "open_enrollment_adjustment",
    "fy21_funding_base",
    "formula_transition_supplement",
    "prek_sped_adm_cat1",
    "prek_sped_adm_cat2",
    "prek_sped_adm_cat3",
    "prek_sped_adm_cat4",
    "prek_sped_adm_cat5",
    "prek_sped_adm_cat6",
    "prek_sped_aid_cat1",
    "prek_sped_aid_cat2",
    "prek_sped_aid_cat3",
    "prek_sped_aid_cat4",
    "prek_sped_aid_cat5",
    "prek_sped_aid_cat6",
    "prek_sped_total",
];

/// Column positions in the department's `Base_Cost` sheet, whose header is on the fourth row.
/// Named here rather than inline so a layout change is one edit.
mod base_cost_columns {
    pub const NAME: usize = 1;
    /// `County Names`. Every categorical sheet carries it too, and they agree.
    pub const COUNTY: usize = 2;
    pub const BUILDINGS: usize = 3;
    pub const ADM: usize = 7;
    pub const KINDERGARTEN: usize = 8;
    pub const GRADES_1_3: usize = 9;
    pub const GRADES_4_8: usize = 10;
    pub const GRADES_9_12: usize = 11;
    pub const CAREER_TECHNICAL: usize = 12;
    pub const GRADES_9_12_TOTAL: usize = 13;
    pub const FUNDED_CLASSROOM: usize = 14;
    pub const FUNDED_SPECIAL: usize = 17;
    pub const TEACHER_COST: usize = 22;
    pub const AGGREGATE: usize = 57;
    pub const PER_PUPIL: usize = 58;
}

/// Column positions in `Summary_SFPR`.
mod summary_columns {
    pub const NAME: usize = 1;
    /// `[A] Base Cost Calculated` — the state share of base cost alone.
    pub const BASE_COST_SHARE: usize = 3;
    /// `[H] Core Foundation Funding Calc` — base cost share plus targeted assistance, special
    /// education, DPIA, English learner, gifted, and CTE. Formula aid, not base cost aid.
    pub const CORE: usize = 10;
    pub const GUARANTEE: usize = 11;
    /// `[R] Total State Support` — every state payment the report carries, including the ones
    /// the guarantee's base excludes: transportation, preschool special education, special
    /// education transportation, and the performance supplement.
    ///
    /// Carried because `CORE + GUARANTEE` is what a district is held at and `TOTAL` is what it
    /// is paid, and the corpus has been comparing the first against figures that behave like the
    /// second.
    pub const TOTAL_STATE_SUPPORT: usize = 20;
    /// `U - Total Transfers (S + T)` — educational service center charges plus other
    /// adjustments.
    ///
    /// Extracted to settle a question by measurement rather than by assertion: whether the
    /// FY2027 report carries the voucher and community school channel. It does not. Under the
    /// Fair School Funding Plan community and STEM students are funded directly rather than
    /// deducted, and these lines are small and are something else.
    pub const TOTAL_TRANSFERS: usize = 27;
    /// `S - Educational Service Center` — the service-centre charge alone.
    ///
    /// Separated from [`TOTAL_TRANSFERS`] so the corpus can bound the one place a voucher or
    /// community-school deduction could still be hiding. The calculator has no line named for
    /// either; the whole transfer channel is this column plus [`OTHER_ADJUSTMENTS`], and only
    /// the second is unlabelled.
    pub const SERVICE_CENTER: usize = 25;
    /// `T - Other Adjustments` — the residual, and the entire remaining hiding place.
    pub const OTHER_ADJUSTMENTS: usize = 26;
    /// `V - Net State Funding (R + U)` — total state support after transfers.
    pub const NET_STATE_FUNDING: usize = 28;
}

/// Column positions in `Detail_SFPR`, whose header is on the fourth row.
///
/// The sheet the corpus never read. It carries the local capacity measure as the department
/// computes it — `[b1] Per Pupil Capacity Amount` — for every district including the ones where
/// the minimum state share binds and recovering it by subtraction is impossible.
mod detail_columns {
    pub const IRN: usize = 0;
    /// `[b1] Per Pupil Capacity Amount` — R.C. 3317.017's blend, published rather than inferred.
    pub const CAPACITY_PER_PUPIL: usize = 5;
    /// `[b4] State Share Percentage`.
    pub const STATE_SHARE: usize = 8;
    /// `[B]` through `[G]` — the six categorical programs, each computed per district.
    ///
    /// The corpus has carried their sum as one number since genesis, inferred as core foundation
    /// funding less the state share of base cost. The inference is exact — `[A]` plus these six is
    /// `[H] Foundation Funding` to the cent — but it produces a lump, and the lump is 43% of
    /// formula aid against a base cost half this project decomposed into 22 statutory elements.
    pub const TARGETED_ASSISTANCE: usize = 10;
    pub const SPECIAL_EDUCATION: usize = 11;
    pub const DPIA: usize = 12;
    pub const ENGLISH_LEARNERS: usize = 13;
    pub const GIFTED: usize = 14;
    pub const CAREER_TECHNICAL: usize = 15;

    // The guarantee's own machinery, and the third FY2021 anchor beside it.
    //
    // `[I] Temporary Transitional Aid Guarantee` is not simply "hold the district at its old
    // amount". It is `funding base - open enrolment adjustment - foundation funding`, and the
    // middle term is a **clawback**: a guaranteed district whose open enrolment FTE has fallen by
    // more than `max(10% of last year, 20 FTE)` has its guarantee reduced by the statewide average
    // base cost per pupil for every FTE beyond that threshold. 43 districts, $5.1m withheld.
    //
    // `[K] Formula Transition Supplement` is a *second* hold-harmless on top of the first, against
    // a different and larger base: `max(FY21 funding base - (foundation funding + guarantee +
    // supplemental targeted assistance + transportation), 0)`. $63.6m to 144 districts, of which
    // 17 are not on the guarantee at all.
    /// `[H2] Funding Base` — the FY2021 amount the guarantee compares against.
    pub const FUNDING_BASE: usize = 17;
    /// `[H3]` — the DPIA part of it, which the phase-in dials separately.
    pub const FUNDING_BASE_ECON_DIS: usize = 18;
    /// `[h1]`/`[h2]` — open enrolment FTE, last year and this.
    pub const OPEN_ENROLLMENT_PRIOR: usize = 21;
    pub const OPEN_ENROLLMENT_CURRENT: usize = 22;
    /// `[I2]`/`[I1]` — the threshold a loss must exceed, and what it costs beyond it.
    pub const OPEN_ENROLLMENT_THRESHOLD: usize = 23;
    pub const OPEN_ENROLLMENT_ADJUSTMENT: usize = 24;
    /// `[L1]` — a FY2021 base that includes transportation, unlike `[H2]`.
    pub const FY21_FUNDING_BASE: usize = 28;
    /// `[K]` — what that base holds the district at.
    pub const FORMULA_TRANSITION_SUPPLEMENT: usize = 29;
}

/// Column positions in the `Special Edu` sheet, whose header is on the third row.
///
/// The second-largest categorical, $722m, and the first of the six to be decomposed. Ohio funds
/// special education in six weighted categories, and the weights are the whole of the policy:
/// 0.2435 for Category 1 against 3.9554 for Category 6, a range of sixteen. A Category 6 pupil is
/// funded at nearly four times the base cost of a pupil with no disability, and a Category 1 pupil
/// at a quarter of it. The categorical total says none of that.
mod special_education_columns {
    pub const IRN: usize = 0;
    /// Six ADM counts, Category 1 through 6, then the six aid amounts they produce.
    pub const FIRST_ADM: usize = 4;
    pub const FIRST_AID: usize = 10;
}

/// Column positions in the `DPIA` sheet, whose header is on the third row.
///
/// Disadvantaged Pupil Impact Aid, $525m, and the only categorical whose mechanism is neither a
/// weight times a count nor an equalisation off wealth. It is a **blend of two poverty counts**
/// scaled by an **index** of the district's poverty against the state's:
///
/// - `d1` weights the FY2025 economically disadvantaged ADM at 65% and the FY2026 directly
///   certified ADM at 35%. Two measures of the same thing that disagree — direct certification
///   is administrative and captures fewer children than the disadvantaged count does.
/// - `d2` is that blended count as a share of enrolled ADM.
/// - `d3` indexes `d2` against the statewide figure of 0.5334, so a district at the state average
///   scores one and the aid scales from there.
///
/// The per-pupil amount is $422. Nothing about that reaches a page showing one DPIA total.
mod dpia_columns {
    pub const IRN: usize = 0;
    /// `d1a` FY2025 economically disadvantaged ADM.
    pub const ECON_DISADVANTAGED_ADM: usize = 4;
    /// `d1b` FY2026 directly certified ADM.
    pub const DIRECTLY_CERTIFIED_ADM: usize = 5;
    /// `d1` the 65/35 blend of the two.
    pub const WEIGHTED_ADM: usize = 6;
    /// `d2` the blend as a share of enrolled ADM.
    pub const PERCENTAGE: usize = 7;
    /// `d3` that share indexed against the statewide one.
    pub const INDEX: usize = 8;
}

/// The DPIA blend weights and per-pupil amount, as the workbook states them.
pub const DPIA_BLEND: (f64, f64) = (0.65, 0.35);
/// The amount each weighted disadvantaged pupil generates.
pub const DPIA_PER_PUPIL: f64 = 422.0;
/// The statewide economically disadvantaged percentage `d3` indexes against.
pub const DPIA_STATEWIDE_PERCENTAGE: f64 = 0.533_380_310_606_710_3;

/// The statutory weight on each special education category, as the workbook states them.
///
/// Multiplied by the statewide average base cost per pupil to give the per-pupil amount, then by
/// the district's ADM in that category and its state share percentage. Held here rather than in a
/// calculator because they are parameters: they are set in statute and changed by budget bills,
/// and the corpus has no node for them.
pub const SPECIAL_EDUCATION_WEIGHTS: [f64; 6] = [0.2435, 0.6179, 1.4845, 1.9812, 2.6830, 3.9554];

/// The five career-technical weights and the associated-services weight applied to their sum.
pub const CTE_WEIGHTS: [f64; 5] = [0.6230, 0.5905, 0.2154, 0.1830, 0.1570];
/// Associated services, weighted against total CTE FTE rather than any one category.
pub const CTE_ASSOCIATED_WEIGHT: f64 = 0.0294;
/// `Detailed SFPR!L12` — the career-technical base cost per pupil the five weights multiply.
pub const CTE_BASE_COST_PER_PUPIL: f64 = 9855.62;

/// The three English learner weights, in the order the sheet gives them.
pub const ENGLISH_LEARNER_WEIGHTS: [f64; 3] = [0.2104, 0.1577, 0.1053];
/// `Base_Cost!E3` — the statewide average base cost per pupil the EL weights multiply.
pub const AVERAGE_BASE_COST_PER_PUPIL: f64 = 8241.61;

/// The gifted program's five prices and three unit divisors, as the sheet's formulas state them.
pub const GIFTED_IDENTIFICATION_PER_PUPIL: f64 = 24.0;
/// Against enrolled ADM rather than the K-6 count identification uses.
pub const GIFTED_REFERRAL_PER_PUPIL: f64 = 2.50;
/// One coordinator unit per this many enrolled pupils, floored at 0.5 units and capped at 8.
pub const GIFTED_COORDINATOR_DIVISOR: f64 = 3300.0;
/// Floored at 0.5 units and capped at 8, so the smallest and largest districts both bind.
pub const GIFTED_COORDINATOR_UNIT_FLOOR: f64 = 0.5;
/// The upper bound on coordinator units, reached at 26,400 enrolled pupils.
pub const GIFTED_COORDINATOR_UNIT_CAP: f64 = 8.0;
/// What one coordinator unit is worth before the state share.
pub const GIFTED_COORDINATOR_UNIT_PRICE: f64 = 85_776.0;
/// One intervention specialist unit per this many identified gifted pupils, floored at 0.3.
pub const GIFTED_SPECIALIST_DIVISOR: f64 = 140.0;
/// The minimum units a district draws in each band, however few gifted pupils it identifies.
pub const GIFTED_SPECIALIST_UNIT_FLOOR: f64 = 0.3;
/// The two bands are priced differently, K-8 above 9-12.
pub const GIFTED_SPECIALIST_K8_UNIT_PRICE: f64 = 89_378.0;
/// The 9-12 unit is priced $8,404 below the K-8 one.
pub const GIFTED_SPECIALIST_9_12_UNIT_PRICE: f64 = 80_974.0;

/// Targeted assistance's statewide constants and coefficients.
///
/// `MEDIAN_WEIGHTED_WEALTH` is computed by the sheet itself as the median of the district column;
/// the per-pupil median is stated. The sheet also publishes both **excluding North Bass and Middle
/// Bass**, at $362,326,436.80 and $253,337.38 — and neither formula references them. The exclusion
/// is displayed and unused.
pub const TA_MEDIAN_WEIGHTED_WEALTH: f64 = 392_151_306.632_980_76;
/// The median district's weighted wealth per resident pupil, which `[E]` indexes against.
pub const TA_MEDIAN_WEALTH_PER_PUPIL: f64 = 276_708.97;
/// Property valuation against federal adjusted gross income, in the blend that makes `[A]`.
pub const TA_WEALTH_BLEND: (f64, f64) = (0.6, 0.4);
/// The capacity tier pays this share of the shortfall below the median district's total wealth.
pub const TA_CAPACITY_RATE: f64 = 0.008;
/// Below this ADM the capacity tier pays nothing; between here and `TA_CAPACITY_RAMP_START` it
/// pays `TA_CAPACITY_SMALL_SHARE`; from there to `TA_CAPACITY_FULL_AT` the share ramps to one.
pub const TA_CAPACITY_MINIMUM_ADM: f64 = 200.0;
/// Between here and `TA_CAPACITY_FULL_AT` the share ramps linearly from 5% to 100%.
pub const TA_CAPACITY_RAMP_START: f64 = 400.0;
/// At and above this ADM the capacity tier is paid in full.
pub const TA_CAPACITY_FULL_AT: f64 = 600.0;
/// What a district between the minimum and the ramp receives of the capacity tier.
pub const TA_CAPACITY_SMALL_SHARE: f64 = 0.05;
/// The wealth tier's two coefficients. The second is 0.8 times the first, which is why the tier
/// cuts off at a wealth index of 0.8: that is exactly where the bracket reaches zero.
pub const TA_WEALTH_RATE: f64 = 0.014;
/// Applied to the district's own wealth per pupil; exactly 0.8 times `TA_WEALTH_RATE`.
pub const TA_WEALTH_OFFSET_RATE: f64 = 0.0112;
/// Below this wealth index the tier pays nothing, because the bracket has gone negative.
pub const TA_WEALTH_INDEX_FLOOR: f64 = 0.8;

/// The performance supplement's rate, per pupil per rating point.
pub const PERFORMANCE_SUPPLEMENT_PER_POINT: f64 = 13.0;
/// A district qualifies on any of three routes: stars above this, progress at or above
/// [`PERFORMANCE_PROGRESS_THRESHOLD`], or progress higher than the year before.
pub const PERFORMANCE_STAR_THRESHOLD: f64 = 3.5;
/// The progress-component route, which does not require the overall rating to be good at all.
pub const PERFORMANCE_PROGRESS_THRESHOLD: f64 = 3.0;
/// The unconditional supplement, per pupil, every district.
pub const BASE_FUNDING_SUPPLEMENT_PER_PUPIL: f64 = 40.0;
/// The growth supplement's rate and the three-year growth a district must reach to draw it.
pub const ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL: f64 = 250.0;
/// Three per cent over three years. A cliff, not a slope: below it the district draws nothing.
pub const ENROLLMENT_GROWTH_THRESHOLD: f64 = 0.03;

/// Transportation's two rate bases: per weighted rider, and per bus mile times a 180-day year.
pub const TRANSPORT_PER_RIDER: f64 = 1337.175;
/// The mile base, which 350 of 611 districts are actually paid on.
pub const TRANSPORT_PER_MILE: f64 = 6.867;
/// A school year, as the mile base counts it.
pub const TRANSPORT_SCHOOL_DAYS: f64 = 180.0;
/// Non-public riders count double and community or STEM school riders one and a half times.
pub const TRANSPORT_NONPUBLIC_WEIGHT: f64 = 2.0;
/// Community and STEM school riders.
pub const TRANSPORT_COMMUNITY_WEIGHT: f64 = 1.5;
/// Mass transit and other vehicle types are paid at these fractions of the rider rate.
pub const TRANSPORT_MASS_TRANSIT_RATE: f64 = 0.35;
/// Other vehicle types, type 5 and 6.
pub const TRANSPORT_OTHER_RATE: f64 = 0.50;
/// **Fifty per cent**, against the formula's ten. 440 of 611 districts sit on it.
pub const TRANSPORT_MINIMUM_STATE_SHARE: f64 = 0.5;
/// The efficiency supplement's ceiling, and the index band it ramps across.
pub const TRANSPORT_EFFICIENCY_CEILING: f64 = 0.15;
/// Below the lower bound nothing is paid; across the band the supplement ramps to its ceiling.
pub const TRANSPORT_EFFICIENCY_BAND: (f64, f64) = (1.0, 1.5);
/// The density supplement: `(28 - riders per square mile) / 100`, times the mile base, times 0.55.
pub const TRANSPORT_DENSITY_PIVOT: f64 = 28.0;
/// What fraction of the mile base the density supplement pays.
pub const TRANSPORT_DENSITY_RATE: f64 = 0.55;
/// The appropriation did not cover the computed special education transportation entitlement.
pub const TRANSPORT_SPED_PRORATION: f64 = 0.917_459_740_976_215;
/// The general proration factor, which is one this year and is a dial that has been used.
pub const TRANSPORT_PRORATION: f64 = 1.0;

/// Preschool special education pays this flat, per pupil, whatever the category.
pub const PREK_SPED_FLAT_PER_PUPIL: f64 = 4000.0;
/// And the school-age weights at half, against the statewide average base cost per pupil.
pub const PREK_SPED_WEIGHT_FRACTION: f64 = 0.5;
/// The stated proration factor, and the appropriation it was set against.
pub const PREK_SPED_PRORATION: f64 = 0.968_544_48;
/// The limit the factor was calibrated against, and which the program now exceeds.
pub const PREK_SPED_APPROPRIATION: f64 = 147_500_000.0;

/// A guaranteed district losing open enrolment FTE beyond the threshold is charged this per FTE.
///
/// It is the statewide **average base cost per pupil**, at full value — not the district's state
/// share of it. So the guarantee falls by more per departing pupil than the state was paying for
/// that pupil.
pub const OPEN_ENROLLMENT_CLAWBACK_PER_FTE: f64 = 8241.61;
/// The loss a district may absorb before the clawback applies: the greater of these two.
pub const OPEN_ENROLLMENT_THRESHOLD_FRACTION: f64 = 0.1;
/// The floor on that threshold, so a small district is not clawed back on a handful of FTE.
pub const OPEN_ENROLLMENT_THRESHOLD_FLOOR: f64 = 20.0;

/// Column positions in the `CTE` sheet, whose header is on the fourth row.
///
/// Five weighted categories and an associated-services weight applied to the sum. Mechanically the
/// same shape as special education — weight times count times a base cost times the state share —
/// with one difference that matters: **CTE is weighted against its own base cost per pupil**,
/// $9,855.62, where every other weighted categorical uses the general $8,241.61. A career-technical
/// pupil starts from a base 20% higher before any weight is applied.
mod cte_columns {
    pub const IRN: usize = 0;
    /// Five FTE counts, Category 1 through 5, then the five aid amounts they produce.
    pub const FIRST_FTE: usize = 4;
    pub const FIRST_AID: usize = 9;
    /// Associated services, the sum of all five FTE at a sixth weight.
    pub const ASSOCIATED_SERVICES: usize = 16;
}

/// Column positions in the `EL` sheet, whose header is on the fourth row.
///
/// Three categories, and the weights run **downward** — 0.2104, 0.1577, 0.1053. Category 1 is the
/// most recently arrived learner and is funded at twice Category 3, so the program pays most in a
/// pupil's first year and tapers. Every other weighted categorical in the plan runs the other way.
mod el_columns {
    pub const IRN: usize = 0;
    pub const FIRST_ADM: usize = 4;
    pub const FIRST_AID: usize = 8;
}

/// Column positions in the `Gifted` sheet, whose header is on the fifth row.
///
/// The one categorical that is not a weight times a count. Two per-pupil amounts — $24 against
/// K-6 enrolment for identification, $2.50 against all enrolment for referral — plus three kinds of
/// **unit**, each a headcount entitlement priced at a salary-like figure.
///
/// The units carry floors and a cap, and they are the whole of the policy:
///
/// - a coordinator unit per 3,300 pupils, **floored at 0.5 and capped at 8**;
/// - an intervention specialist unit per 140 identified gifted pupils in K-8, floored at 0.3;
/// - the same per 140 in grades 9-12, floored at 0.3, priced lower.
///
/// A district with no identified gifted pupils at all still draws 0.5 + 0.3 + 0.3 units, so gifted
/// funding has a floor no other categorical has. The cap binds from 26,400 pupils upward.
mod gifted_columns {
    pub const IRN: usize = 0;
    /// `[a1] Enrollment K-6`, the base for identification.
    pub const ADM_K6: usize = 4;
    /// `[F1] Identification` and `[F2] Referral`, both already net of the state share.
    pub const IDENTIFICATION: usize = 6;
    pub const REFERRAL: usize = 7;
    /// `[f1]`/`[f2]` — identified gifted FTE by grade band.
    pub const FTE_K8: usize = 8;
    pub const FTE_9_12: usize = 9;
    /// `[F3] Professional Development`, a hard-coded column rather than a computed one.
    pub const PROFESSIONAL_DEVELOPMENT: usize = 11;
    /// Units then dollars, for each of the three unit kinds.
    pub const COORDINATOR_UNITS: usize = 12;
    pub const COORDINATOR_AID: usize = 13;
    pub const SPECIALIST_K8_UNITS: usize = 14;
    pub const SPECIALIST_K8_AID: usize = 15;
    pub const SPECIALIST_9_12_UNITS: usize = 16;
    pub const SPECIALIST_9_12_AID: usize = 17;
}

/// Column positions in the `Targeted_Assistance` sheet, whose header is on the fifth row.
///
/// The largest categorical at $1.36bn, and the only one that is an **equalisation** rather than a
/// payment for a category of pupil. It has two additive tiers, and they measure different things:
///
/// - `[C]` the **capacity amount** — 0.8% of however far the district's *total* weighted wealth
///   falls below the statewide median district's, phased by size;
/// - `[F]` the **wealth amount** — a rate against weighted wealth *per resident pupil*.
///
/// Two features of the sheet's own formulas are invisible in any published total. First, the
/// capacity tier has a **size cliff**: a district under 200 ADM receives none of it, one between
/// 200 and 400 receives 5%, and the fraction ramps linearly to 100% only between 400 and 600.
/// Second, the wealth tier divides by *resident* ADM — enrolled less those open-enrolling in, plus
/// those open-enrolling out — and then multiplies by *enrolled* ADM. Two different pupil counts,
/// one line apart in the same formula.
mod targeted_assistance_columns {
    pub const IRN: usize = 0;
    /// `[a1]`/`[a2]` — open enrolment in and out, which turn enrolled ADM into resident ADM.
    pub const OPEN_ENROLLMENT_IN: usize = 4;
    pub const OPEN_ENROLLMENT_OUT: usize = 5;
    /// `[b]`/`[c]`/`[d]` — the FY2019 figures the supplemental tier's eligibility test uses.
    pub const FY19_WEALTH_INDEX: usize = 6;
    pub const FY19_ENROLLED_ADM: usize = 7;
    pub const FY19_TOTAL_ADM: usize = 8;
    /// `[A1]`/`[A2]` — property valuation and federal adjusted gross income, blended 60/40.
    pub const PROPERTY_VALUATION: usize = 9;
    pub const FEDERAL_GROSS_INCOME: usize = 10;
    /// `[A] District Weighted Wealth`.
    pub const WEIGHTED_WEALTH: usize = 11;
    /// `[B]`/`[C]` — the capacity tier.
    pub const CAPACITY_INDEX: usize = 12;
    pub const CAPACITY_AMOUNT: usize = 13;
    /// `[D]`/`[E]`/`[F]` — the wealth tier.
    pub const WEALTH_PER_PUPIL: usize = 14;
    pub const WEALTH_INDEX: usize = 15;
    pub const WEALTH_AMOUNT: usize = 16;
    /// `[H]`/`[I]` — the supplemental tier's eligibility flag and its amount.
    pub const SUPPLEMENT_ELIGIBLE: usize = 18;
    pub const SUPPLEMENTAL: usize = 19;
}

/// Column positions in the `Performance Supplement` sheet, whose header is on the third row.
///
/// **The only place Ohio's funding formula pays on measured outcomes.** $55.7m, and it is outside
/// foundation funding — it sits in `[R] Total State Support` and not in `[H]`, so the guarantee
/// does not hold a district at it and this corpus carried it as part of an unexplained remainder.
///
/// Three ways to qualify, and the amount scales with the rating rather than being flat: $13 per
/// pupil times the **greater** of the overall star rating and the progress component rating. So a
/// five-star district receives $65 a pupil and a qualifying 2.5-star district $32.50.
mod performance_columns {
    pub const IRN: usize = 0;
    /// `O1 Overall Performance Rating Stars` — 0 to 5 in half steps. One district reads `N/A`.
    pub const STARS: usize = 4;
    /// `O2`/`O3` — the progress component rating for 2023-24 and the year before it.
    pub const PROGRESS: usize = 5;
    pub const PROGRESS_PRIOR: usize = 6;
    /// `Q4. Overall Eligibility` — `Yes` on any of the three routes.
    pub const ELIGIBLE: usize = 7;
    pub const AMOUNT: usize = 8;
}

/// Column positions in the `Base_Enrollment Growth` sheet, whose header is on the fourth row.
///
/// Two payments that share a sheet and nothing else. The **base funding supplement** is $40 for
/// every pupil in every district, unconditional, $56.1m. The **enrollment growth supplement** is
/// $250 a pupil for districts whose enrolment rose at least 3% over three years — and it pays on
/// *every* pupil, not the new ones, so it is a cliff rather than a slope.
///
/// Worth reading beside the guarantee, which pays districts for enrolment they have **lost**. The
/// same formula pays a premium in both directions and the two have never been looked at together.
mod growth_columns {
    pub const IRN: usize = 0;
    /// `L Base Funding Supplement` — $40 times enrolled ADM, for everyone.
    pub const BASE_SUPPLEMENT: usize = 4;
    /// `M1B FY23 Enrolled ADM` — a fourth ADM year, which the panel did not hold.
    pub const ADM_FY23: usize = 6;
    /// `M1` — the three-year change, against which `M2` tests 3%.
    pub const CHANGE: usize = 7;
    pub const ELIGIBLE: usize = 8;
    pub const AMOUNT: usize = 9;
}

/// Column positions in the `Transportation` sheet, whose header is on the fifth row.
///
/// **$726m, plus $183m of special education transportation beside it.** Transportation alone is
/// larger than special education, which makes it the **second-largest single program in Ohio's
/// school funding** after targeted assistance — and all of it sat inside the remainder this corpus
/// carried between `[H] Foundation Funding` and `[R] Total State Support`.
///
/// It is the most elaborate component in the calculator and nothing about it resembles the
/// formula:
///
/// - **Two competing bases, and the district gets the greater.** Per weighted rider at $1,337.175,
///   or per bus mile at $6.867 times a 180-day year. **350 of 611 districts are paid on the mile
///   base**, so the `MAX` is not a formality — it flips for more than half the state.
/// - **Non-public riders count double and community-school riders one and a half times.** A
///   district transporting a private-school child is funded at twice the rate of its own pupil.
///   They are 4.5% of riders and 8.5% of weighted ridership.
/// - **The state minimum share is 50%**, against the formula's 10%. **440 of 611 districts sit on
///   it**, so for most of the state a district's own capacity does not determine its transportation
///   aid at all.
/// - **Two supplements pulling opposite ways.** An efficiency supplement pays up to 15% more for
///   filling buses; a density supplement pays sparse districts on `(28 - riders per square mile)`.
///   One rewards concentration and one compensates for its absence.
/// - **Its own guarantee.** `[F]` holds 38 districts at their FY2021 transportation funding, $24.8m.
///   That is a *second* transitional guarantee, separate from the one on foundation funding, and
///   the corpus has a node for only one of them.
/// - **Special education transportation is prorated at 0.91746** — the appropriation did not cover
///   the computed entitlement and every district's amount was scaled down to fit.
mod transportation_columns {
    pub const IRN: usize = 0;
    /// `[a1]`/`[a2]`/`[a3]` — public, non-public, and community or STEM school riders.
    pub const PUBLIC_RIDERS: usize = 3;
    pub const NONPUBLIC_RIDERS: usize = 4;
    pub const COMMUNITY_RIDERS: usize = 5;
    /// `[b]` — the three weighted 1, 2 and 1.5 respectively.
    pub const WEIGHTED_RIDERS: usize = 7;
    /// `[c]`/`[d]` — mass transit and other vehicle types, paid at 35% and 50% of the rider rate.
    pub const MASS_TRANSIT_RIDERS: usize = 8;
    pub const OTHER_RIDERS: usize = 9;
    /// `[e]` through `[h]` — the inputs to the mile base and the two supplements.
    pub const BUS_MILES: usize = 10;
    pub const ASSIGNED_BUSES: usize = 11;
    pub const RIDER_CAPACITY_TARGET: usize = 12;
    /// `[D2]`/`[E2]` — the two indices, published rounded to four places. Carried rather than
    /// recomputed: each is a rounded quotient of rounded inputs, and rebuilding the chain from
    /// the ends puts the error where the supplement is most sensitive to it.
    pub const EFFICIENCY_INDEX: usize = 22;
    pub const DISTRICT_DENSITY: usize = 24;
    pub const SQUARE_MILES: usize = 13;
    /// `[j]` — reported special education transportation cost, before proration.
    pub const REPORTED_SPED_COST: usize = 15;
    /// `[A]` through `[E]` — the four payments and the two supplements.
    pub const SCHOOL_BUS: usize = 18;
    pub const MASS_TRANSIT: usize = 19;
    pub const OTHER: usize = 20;
    pub const EFFICIENCY: usize = 23;
    pub const DENSITY: usize = 26;
    /// `[F1]`/`[F]` — the FY2021 base and the transportation guarantee it produces.
    pub const FY21_BASE: usize = 27;
    pub const GUARANTEE: usize = 28;
    /// `[G]`/`[J]` — the total, and special education transportation beside it.
    pub const TOTAL: usize = 29;
    pub const SPECIAL_EDUCATION: usize = 30;
}

/// Column positions in the `PSS_pec_Ed` sheet, whose header is on the fifth row.
///
/// Preschool special education, **$148m**, and the last line in the gap between `[H] Foundation
/// Funding` and `[R] Total State Support`.
///
/// # A flat amount and a half-weight, which nothing else in the formula combines
///
/// Each category pays `(ADM x $4,000) + (ADM x weight x average base cost x state share x 0.5)`,
/// and the whole is multiplied by a proration factor. So a preschool pupil generates a **flat
/// $4,000 regardless of category** — 69% of the program — plus a weighted amount at **half** the
/// school-age rate. The weights are the same six.
///
/// The state share applies only to the weighted half. The $4,000 is paid in full to every
/// district, which makes this the one component where the wealthiest district and the poorest are
/// funded identically for most of what they receive.
///
/// # The proration no longer fits the appropriation it was set against
///
/// The sheet carries its own **appropriation limit of $147,500,000** in a cell beside the factor,
/// which is what makes this program the clearest statement in the workbook of what a proration is:
/// a budget divided by an entitlement. At the stated factor of 0.96854448 the program totals
/// **$148,396,721** — **$896,721 over the limit**. The factor that would reach it is 0.96269183.
///
/// A third cell on the same sheet states a total of $146,708,228.07, which matches neither the
/// column above it nor the cap. Most likely the factor was calibrated against an earlier ADM
/// vintage and the counts were refreshed without recalibrating. This is a projection published
/// before the fiscal year, not an actual, so a later recalibration is expected — but as published
/// the three figures are mutually inconsistent, and that is worth recording rather than smoothing.
mod prek_sped_columns {
    pub const IRN: usize = 0;
    /// Six ADM counts, Category 1 through 6, then the six amounts they produce.
    pub const FIRST_ADM: usize = 3;
    pub const FIRST_AID: usize = 10;
    pub const TOTAL: usize = 16;
}

/// Column positions in the `Local_Capacity` sheet, whose header is on the second row.
///
/// The whole of R.C. 3317.017 worked step by step, with the statute's own labels — `[V1]`, `[I1]`,
/// `[C1]` through `[C7]`. Reading the `Valuation & Income` sheet instead and inferring the rest
/// produced a capacity 4.4% light, and every inferred input turned out to be the wrong one:
///
/// - The third term is **federal** median income, not the Ohio median the profile report
///   publishes. For Columbus that is $46,395 against $31,555.
/// - It carries an adjustment factor, as does the tax-return count, and both are applied before
///   the blend. Columbus's adjusted return count is 272,169 against a raw 142,349.
/// - The statewide figure the ratio divides by is the **federal** statewide median, $54,546.64,
///   not the median of district medians.
/// - The benchmark is published per row rather than needing reconstruction: 1.46504.
mod capacity_columns {
    pub const IRN: usize = 0;
    /// `[V2] TY25 Total Valuation` and the two years behind it.
    pub const VALUATION_TY25: usize = 3;
    pub const VALUATION_TY24: usize = 4;
    pub const VALUATION_TY23: usize = 5;
    /// `[I2]`/`[I3]` carry an adjustment factor; `[I4]` is raw.
    pub const AGI_TY24: usize = 7;
    pub const AGI_TY23: usize = 8;
    pub const AGI_TY22: usize = 9;
    /// `[I5] TY23 Federal Median Income with ADJ Factor`.
    pub const FEDERAL_MEDIAN_INCOME: usize = 11;
    /// `[I6] TY23 Number of State Tax Returns with ADJ Factor`.
    pub const TAX_RETURNS: usize = 12;
    /// `[I7] TY23 Statewide Federal Median Income`.
    pub const STATEWIDE_MEDIAN_INCOME: usize = 13;
    /// `[C5] Ratio Calculated in C4 for the 40th Highest District` — the benchmark, published.
    pub const BENCHMARK_RATIO: usize = 19;
    /// `[C6] Local Capacity Percentage`.
    pub const CAPACITY_RATE: usize = 20;
}

/// Column positions in the `ADM Data` sheet.
///
/// # The enrolled-ADM years are labelled twice, and once wrongly
///
/// `Base_Cost` carries the same three ADM columns, cell for cell, under the headers
/// `[b1] FY22`, `[b2] FY23`, `[b3] FY24`. `ADM Data` labels the identical values `[b1] FY24`,
/// `[b2] FY25`, `[b3] FY26`. They cannot both be right, and the arithmetic settles it: base
/// cost enrolled ADM is the three-year average of the three, and this is the **FY2027**
/// calculator, which R.C. 3317.011 funds on FY2024 through FY2026. The `Base_Cost` headers are
/// left over from an earlier vintage of the workbook.
///
/// This matters beyond tidiness. The first fixture built from this file carried the stale
/// labels, so every enrollment-trend figure in the corpus was named for the wrong pair of
/// years — and the later two of the three are themselves partly departmental estimate rather
/// than actual, which a label reading "FY2022 to FY2024" conceals entirely.
/// # And a seventh pupil count, which is the one the categoricals are paid on
///
/// `[a] Enrolled ADM` sits at column 3 of this sheet and is **not** `[b3] FY26 Enrolled ADM` at
/// column 28. For Akron they are 18,892.45 and 18,842.45 — fifty pupils apart. The four
/// categorical sheets all look up column 3; base cost averages columns 26 through 28.
///
/// So a district's aid is computed against one enrolled ADM for its base cost and a different one
/// for its targeted assistance, gifted, career-technical and English learner amounts, and no
/// published figure names either. This corpus carries both rather than picking.
mod adm_columns {
    /// `[a] Enrolled ADM` — the count the four categorical sheets are paid on.
    pub const CATEGORICAL_ENROLLED_ADM: usize = 3;
    pub const BUILDINGS_FY25: usize = 25;
    pub const ADM_FY24: usize = 26;
    pub const ADM_FY25: usize = 27;
    pub const ADM_FY26: usize = 28;
}

/// Column positions in the District Profile Report's `District Data` sheet.
mod profile_columns {
    pub const NAME: usize = 0;
    pub const IRN: usize = 1;
    pub const ENROLLED_ADM: usize = 4;
    pub const ECON_DISADVANTAGED: usize = 11;
    pub const VALUATION_PER_PUPIL: usize = 21;
    pub const CURRENT_OPERATING_MILLAGE: usize = 33;
    pub const EFFECTIVE_CLASS1_MILLAGE: usize = 34;
    pub const OPERATING_EXPENDITURE: usize = 46;
    pub const STATE_REVENUE: usize = 47;
    pub const LOCAL_REVENUE: usize = 49;
}

/// The worksheets [`build_fy27_model`] reads, named rather than positional.
///
/// Eight `&[Vec<String>]` arguments in a row is a mistake waiting to be made: every one has the
/// same type, so transposing two is silent and produces a fixture of plausible wrong numbers. The
/// sheets grew from four to eight as the calculator was taken apart, and at eight it stopped being
/// safe to pass them by position.
/// `Default` gives every sheet as empty, which is what a test exercising one of them wants; the
/// production call site in [`crate::rebuild`] names all twelve.
#[derive(Clone, Copy, Default)]
pub struct Fy27Sheets<'a> {
    /// `Base_Cost` — the statutory build-up's inputs.
    pub base_cost_rows: &'a [Vec<String>],
    /// `Summary_SFPR` — the payment report's headline lines.
    pub summary_rows: &'a [Vec<String>],
    /// `ADM Data` — the three enrolled-ADM years.
    pub adm_rows: &'a [Vec<String>],
    /// The District Profile Report's `District Data`, a different workbook.
    pub profile_rows: &'a [Vec<String>],
    /// `Detail_SFPR` — the six categoricals and the published capacity.
    pub detail_rows: &'a [Vec<String>],
    /// `Local_Capacity` — R.C. 3317.017 worked step by step.
    pub capacity_rows: &'a [Vec<String>],
    /// `Special Edu` — six weighted categories.
    pub special_education_rows: &'a [Vec<String>],
    /// `DPIA` — the blend and the squared index.
    pub dpia_rows: &'a [Vec<String>],
    /// `CTE` — five weighted categories against a career-technical base cost.
    pub cte_rows: &'a [Vec<String>],
    /// `EL` — three weighted categories, descending.
    pub el_rows: &'a [Vec<String>],
    /// `Gifted` — two per-pupil amounts and three kinds of unit.
    pub gifted_rows: &'a [Vec<String>],
    /// `Targeted_Assistance` — the two-tier equalisation.
    pub targeted_assistance_rows: &'a [Vec<String>],
    /// `Performance Supplement` — the one payment gated on measured outcomes.
    pub performance_rows: &'a [Vec<String>],
    /// `Base_Enrollment Growth` — $40 for every pupil, and $250 for a district that grew.
    pub growth_rows: &'a [Vec<String>],
    /// `Transportation` — two rate bases, two supplements, and a second guarantee.
    pub transportation_rows: &'a [Vec<String>],
    /// `PSS_pec_Ed` — preschool special education, a flat amount plus a half-weight.
    pub prek_sped_rows: &'a [Vec<String>],
}

/// Index a worksheet by IRN, taking every row whose key column holds one.
///
/// **Not** by skipping a fixed number of header rows. The department puts each categorical sheet's
/// header at whatever depth that sheet's statewide constants needed — the fourth row for `CTE` and
/// `EL`, the fifth for `Gifted` and `Targeted_Assistance` — and leaves blank rows above it, which
/// the workbook reader drops. Counting rows therefore means counting the blanks correctly too, and
/// the first attempt here was off by one on three of the four sheets. It lost exactly the first
/// district, alphabetically, and left the rest right: the fixture gained one row of empty
/// categorical columns out of 609 and nothing else looked wrong.
///
/// Matching on the key instead makes the header depth irrelevant. A header row's key cell says
/// `District IRN`, which is not a district key, so it filters itself out.
fn rows_by_irn(rows: &[Vec<String>], irn_column: usize) -> HashMap<&str, &Vec<String>> {
    rows_by_key(rows, irn_column).collect()
}

/// Join the department's FY2027 base cost and summary sheets with profile-report valuation.
#[must_use]
pub fn build_fy27_model(sheets: &Fy27Sheets<'_>) -> Vec<Vec<String>> {
    let Fy27Sheets {
        base_cost_rows,
        summary_rows,
        adm_rows,
        profile_rows,
        detail_rows,
        capacity_rows,
        special_education_rows,
        dpia_rows,
        cte_rows,
        el_rows,
        gifted_rows,
        targeted_assistance_rows,
        performance_rows,
        growth_rows,
        transportation_rows,
        prek_sped_rows,
    } = *sheets;
    use base_cost_columns as bc;

    let summary: Vec<(&str, &Vec<String>)> = rows_by_key(summary_rows, 0)
        .filter(|(_, row)| !is_statewide_row(row, summary_columns::NAME))
        .collect();

    // The three sheets the corpus never read, keyed by IRN. Their header rows sit at different
    // depths — `Detail_SFPR` on the fourth, tax returns on the second, valuation on the first —
    // so each is skipped to its own data start rather than to a shared one.
    let detail: HashMap<&str, &Vec<String>> = detail_rows
        .iter()
        .skip(4)
        .filter(|row| !cell(row, detail_columns::IRN).trim().is_empty())
        .map(|row| (cell(row, detail_columns::IRN).trim(), row))
        .collect();
    let special_education: HashMap<&str, &Vec<String>> = special_education_rows
        .iter()
        .skip(3)
        .filter(|row| !cell(row, special_education_columns::IRN).trim().is_empty())
        .map(|row| (cell(row, special_education_columns::IRN).trim(), row))
        .collect();
    let dpia: HashMap<&str, &Vec<String>> = dpia_rows
        .iter()
        .skip(3)
        .filter(|row| !cell(row, dpia_columns::IRN).trim().is_empty())
        .map(|row| (cell(row, dpia_columns::IRN).trim(), row))
        .collect();
    let capacity: HashMap<&str, &Vec<String>> = capacity_rows
        .iter()
        .skip(2)
        .filter(|row| !cell(row, capacity_columns::IRN).trim().is_empty())
        .map(|row| (cell(row, capacity_columns::IRN).trim(), row))
        .collect();
    // The four remaining categorical sheets. Their headers sit at two depths: `CTE` and `EL` on the
    // fourth row, `Gifted` and `Targeted_Assistance` on the fifth.
    let cte: HashMap<&str, &Vec<String>> = rows_by_irn(cte_rows, cte_columns::IRN);
    let el: HashMap<&str, &Vec<String>> = rows_by_irn(el_rows, el_columns::IRN);
    let gifted: HashMap<&str, &Vec<String>> = rows_by_irn(gifted_rows, gifted_columns::IRN);
    let targeted: HashMap<&str, &Vec<String>> =
        rows_by_irn(targeted_assistance_rows, targeted_assistance_columns::IRN);
    // The two supplements outside foundation funding.
    let performance: HashMap<&str, &Vec<String>> =
        rows_by_irn(performance_rows, performance_columns::IRN);
    let growth: HashMap<&str, &Vec<String>> = rows_by_irn(growth_rows, growth_columns::IRN);
    let transportation: HashMap<&str, &Vec<String>> =
        rows_by_irn(transportation_rows, transportation_columns::IRN);
    let prek_sped: HashMap<&str, &Vec<String>> =
        rows_by_irn(prek_sped_rows, prek_sped_columns::IRN);

    let valuation: Vec<(&str, &Vec<String>)> = profile_rows
        .iter()
        .skip(1)
        .map(|row| (cell(row, profile_columns::IRN).trim(), row))
        .collect();

    let adm_history: Vec<(&str, &Vec<String>)> = rows_by_key(adm_rows, 0).collect();

    let mut districts: Vec<(&str, &Vec<String>)> = rows_by_key(base_cost_rows, 0).collect();
    districts.sort_by(|a, b| a.0.cmp(b.0));

    let mut out = Vec::with_capacity(districts.len());
    for (irn, row) in districts {
        let Some((_, summary_row)) = summary.iter().find(|(key, _)| *key == irn) else {
            continue;
        };
        // A district present in the base cost sheet with no ADM is a placeholder row — a
        // district that closed, or one that exists only to carry a footnote.
        let Some(adm) = cell_number(row, bc::ADM).filter(|adm| *adm > 0.0) else {
            continue;
        };
        // Enrolled ADM comes from `ADM Data` rather than `Base_Cost`, because only that sheet
        // labels the years correctly. See the note on `adm_columns`.
        let history = adm_history
            .iter()
            .find(|(key, _)| *key == irn)
            .map(|(_, r)| *r);
        let adm_year = |column: usize| history.and_then(|row| cell_number(row, column));

        let valuation_per_pupil = valuation
            .iter()
            .find(|(key, _)| *key == irn)
            .and_then(|(_, profile)| cell_number(profile, profile_columns::VALUATION_PER_PUPIL));

        out.push(vec![
            irn.to_string(),
            clean_name(cell(row, bc::NAME)),
            format_value(Some(adm), 4),
            format_value(
                adm_year(adm_columns::BUILDINGS_FY25).or_else(|| cell_number(row, bc::BUILDINGS)),
                0,
            ),
            format_value(cell_number(row, bc::KINDERGARTEN), 4),
            format_value(cell_number(row, bc::GRADES_1_3), 4),
            format_value(cell_number(row, bc::GRADES_4_8), 4),
            format_value(cell_number(row, bc::GRADES_9_12), 4),
            format_value(cell_number(row, bc::CAREER_TECHNICAL), 4),
            format_value(cell_number(row, bc::GRADES_9_12_TOTAL), 4),
            format_value(cell_number(row, bc::FUNDED_CLASSROOM), 2),
            format_value(cell_number(row, bc::FUNDED_SPECIAL), 2),
            format_value(cell_number(row, bc::TEACHER_COST), 2),
            format_value(cell_number(row, bc::AGGREGATE), 2),
            format_value(cell_number(row, bc::PER_PUPIL), 2),
            // A district off the guarantee records 0, not blank: it is known to have none.
            format_value(
                Some(cell_number(summary_row, summary_columns::GUARANTEE).unwrap_or(0.0)),
                2,
            ),
            format_value(adm_year(adm_columns::ADM_FY24), 4),
            format_value(adm_year(adm_columns::ADM_FY25), 4),
            format_value(adm_year(adm_columns::ADM_FY26), 4),
            format_value(valuation_per_pupil, 2),
            format_value(cell_number(summary_row, summary_columns::CORE), 2),
            format_value(
                cell_number(summary_row, summary_columns::BASE_COST_SHARE),
                2,
            ),
            format_value(
                cell_number(summary_row, summary_columns::TOTAL_STATE_SUPPORT),
                2,
            ),
            format_value(
                Some(cell_number(summary_row, summary_columns::TOTAL_TRANSFERS).unwrap_or(0.0)),
                2,
            ),
            format_value(
                Some(cell_number(summary_row, summary_columns::SERVICE_CENTER).unwrap_or(0.0)),
                2,
            ),
            format_value(
                Some(cell_number(summary_row, summary_columns::OTHER_ADJUSTMENTS).unwrap_or(0.0)),
                2,
            ),
            format_value(
                cell_number(summary_row, summary_columns::NET_STATE_FUNDING),
                2,
            ),
            // The department's own local capacity, and the inputs it is built from. Absent for a
            // district missing from any of the three sheets, which is why every one is written
            // through `format_value` rather than defaulted to zero: a capacity of nothing and a
            // capacity nobody published are different claims and the calculators read them so.
            format_value(
                detail
                    .get(irn)
                    .and_then(|row| cell_number(row, detail_columns::CAPACITY_PER_PUPIL)),
                4,
            ),
            format_value(
                detail
                    .get(irn)
                    .and_then(|row| cell_number(row, detail_columns::STATE_SHARE)),
                // Ten places rather than six. The state share multiplies four of the six
                // categoricals, so rounding it costs a dollar or so per district on each — small,
                // and enough to stop a reproduction test from being exact.
                10,
            ),
        ]);

        let cap = capacity.get(irn);
        for (column, places) in [
            (capacity_columns::VALUATION_TY25, 2),
            (capacity_columns::VALUATION_TY24, 2),
            (capacity_columns::VALUATION_TY23, 2),
            (capacity_columns::AGI_TY24, 2),
            (capacity_columns::AGI_TY23, 2),
            (capacity_columns::AGI_TY22, 2),
            (capacity_columns::TAX_RETURNS, 4),
            (capacity_columns::FEDERAL_MEDIAN_INCOME, 4),
            (capacity_columns::STATEWIDE_MEDIAN_INCOME, 4),
            (capacity_columns::BENCHMARK_RATIO, 8),
            (capacity_columns::CAPACITY_RATE, 10),
        ] {
            let value = cap.and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }

        for column in [
            detail_columns::TARGETED_ASSISTANCE,
            detail_columns::SPECIAL_EDUCATION,
            detail_columns::DPIA,
            detail_columns::ENGLISH_LEARNERS,
            detail_columns::GIFTED,
            detail_columns::CAREER_TECHNICAL,
        ] {
            let value = detail.get(irn).and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, 2));
        }

        let sped = special_education.get(irn);
        for offset in 0..6 {
            let value = sped
                .and_then(|row| cell_number(row, special_education_columns::FIRST_ADM + offset));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, 4));
        }
        for offset in 0..6 {
            let value = sped
                .and_then(|row| cell_number(row, special_education_columns::FIRST_AID + offset));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, 2));
        }

        let dpia_row = dpia.get(irn);
        for (column, places) in [
            (dpia_columns::ECON_DISADVANTAGED_ADM, 4),
            (dpia_columns::DIRECTLY_CERTIFIED_ADM, 4),
            (dpia_columns::WEIGHTED_ADM, 4),
            (dpia_columns::PERCENTAGE, 8),
            (dpia_columns::INDEX, 8),
        ] {
            let value = dpia_row.and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }

        let cte_row = cte.get(irn);
        let mut columns: Vec<(usize, usize)> = (0..5)
            .map(|k| (cte_columns::FIRST_FTE + k, 9))
            .chain((0..5).map(|k| (cte_columns::FIRST_AID + k, 2)))
            .collect();
        columns.push((cte_columns::ASSOCIATED_SERVICES, 2));
        for (column, places) in columns {
            let value = cte_row.and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }

        let el_row = el.get(irn);
        for (column, places) in (0..3)
            .map(|k| (el_columns::FIRST_ADM + k, 9))
            .chain((0..3).map(|k| (el_columns::FIRST_AID + k, 2)))
        {
            let value = el_row.and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }

        let gifted_row = gifted.get(irn);
        for (column, places) in [
            (gifted_columns::ADM_K6, 9),
            (gifted_columns::FTE_K8, 9),
            (gifted_columns::FTE_9_12, 9),
            (gifted_columns::IDENTIFICATION, 2),
            (gifted_columns::REFERRAL, 2),
            (gifted_columns::PROFESSIONAL_DEVELOPMENT, 2),
            (gifted_columns::COORDINATOR_UNITS, 8),
            (gifted_columns::COORDINATOR_AID, 2),
            (gifted_columns::SPECIALIST_K8_UNITS, 8),
            (gifted_columns::SPECIALIST_K8_AID, 2),
            (gifted_columns::SPECIALIST_9_12_UNITS, 8),
            (gifted_columns::SPECIALIST_9_12_AID, 2),
        ] {
            let value = gifted_row.and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }

        let ta_row = targeted.get(irn);
        for (column, places) in [
            (targeted_assistance_columns::OPEN_ENROLLMENT_IN, 9),
            (targeted_assistance_columns::OPEN_ENROLLMENT_OUT, 9),
            (targeted_assistance_columns::FY19_WEALTH_INDEX, 9),
            (targeted_assistance_columns::FY19_ENROLLED_ADM, 4),
            (targeted_assistance_columns::FY19_TOTAL_ADM, 4),
            (targeted_assistance_columns::PROPERTY_VALUATION, 2),
            (targeted_assistance_columns::FEDERAL_GROSS_INCOME, 4),
            (targeted_assistance_columns::WEIGHTED_WEALTH, 4),
            (targeted_assistance_columns::CAPACITY_INDEX, 8),
            (targeted_assistance_columns::CAPACITY_AMOUNT, 2),
            (targeted_assistance_columns::WEALTH_PER_PUPIL, 2),
            (targeted_assistance_columns::WEALTH_INDEX, 8),
            (targeted_assistance_columns::WEALTH_AMOUNT, 2),
            (targeted_assistance_columns::SUPPLEMENTAL, 2),
        ] {
            let value = ta_row.and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }
        // The eligibility flag is the sheet's one non-numeric answer, and the department spells its
        // negative `N0` with a zero. Carried as one or nothing rather than as either spelling.
        let eligible = ta_row
            .map(|row| cell(row, targeted_assistance_columns::SUPPLEMENT_ELIGIBLE).trim())
            .is_some_and(|flag| flag.eq_ignore_ascii_case("yes"));
        out.last_mut()
            .expect("just pushed")
            .push(if eligible { "1" } else { "0" }.to_string());
        // The pupil count all four of those sheets are actually paid on, which is none of the three
        // years base cost averages. Carried last so the header stays append-only.
        out.last_mut().expect("just pushed").push(format_value(
            adm_year(adm_columns::CATEGORICAL_ENROLLED_ADM),
            9,
        ));
        // The county, from the same sheet the district's identity comes from. Ohio's 88 counties
        // are the unit almost every reader already has a mental model of, and until now the only
        // grouping the site offered was the whole state.
        out.last_mut()
            .expect("just pushed")
            .push(clean_name(cell(row, base_cost_columns::COUNTY)));

        // The performance supplement. The star rating reads `N/A` for one district, which
        // `cell_number` returns as `None` and `format_value` writes as empty — the right answer,
        // since a district with no rating did not score zero.
        let perf = performance.get(irn);
        for (column, places) in [
            (performance_columns::STARS, 2),
            (performance_columns::PROGRESS, 2),
            (performance_columns::PROGRESS_PRIOR, 2),
        ] {
            let value = perf.and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }
        let performance_eligible = perf
            .map(|row| cell(row, performance_columns::ELIGIBLE).trim())
            .is_some_and(|flag| flag.eq_ignore_ascii_case("yes"));
        out.last_mut()
            .expect("just pushed")
            .push(if performance_eligible { "1" } else { "0" }.to_string());
        let value = perf.and_then(|row| cell_number(row, performance_columns::AMOUNT));
        out.last_mut()
            .expect("just pushed")
            .push(format_value(value, 2));

        // The base funding supplement and the enrollment growth supplement.
        let grow = growth.get(irn);
        for (column, places) in [
            (growth_columns::BASE_SUPPLEMENT, 2),
            (growth_columns::ADM_FY23, 6),
            (growth_columns::CHANGE, 9),
        ] {
            let value = grow.and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }
        let growth_eligible = grow
            .map(|row| cell(row, growth_columns::ELIGIBLE).trim())
            .is_some_and(|flag| flag.eq_ignore_ascii_case("yes"));
        out.last_mut()
            .expect("just pushed")
            .push(if growth_eligible { "1" } else { "0" }.to_string());
        let value = grow.and_then(|row| cell_number(row, growth_columns::AMOUNT));
        out.last_mut()
            .expect("just pushed")
            .push(format_value(value, 2));

        // Transportation: the counts it is computed from, then the payments it produces.
        let trans = transportation.get(irn);
        use transportation_columns as tc;
        for (column, places) in [
            (tc::PUBLIC_RIDERS, 4),
            (tc::NONPUBLIC_RIDERS, 4),
            (tc::COMMUNITY_RIDERS, 4),
            (tc::WEIGHTED_RIDERS, 4),
            (tc::MASS_TRANSIT_RIDERS, 4),
            (tc::OTHER_RIDERS, 4),
            (tc::BUS_MILES, 4),
            (tc::ASSIGNED_BUSES, 4),
            (tc::RIDER_CAPACITY_TARGET, 4),
            (tc::EFFICIENCY_INDEX, 4),
            (tc::DISTRICT_DENSITY, 6),
            (tc::SQUARE_MILES, 4),
            (tc::REPORTED_SPED_COST, 2),
            (tc::SCHOOL_BUS, 2),
            (tc::MASS_TRANSIT, 2),
            (tc::OTHER, 2),
            (tc::EFFICIENCY, 2),
            (tc::DENSITY, 2),
            (tc::FY21_BASE, 2),
            (tc::GUARANTEE, 2),
            (tc::TOTAL, 2),
            (tc::SPECIAL_EDUCATION, 2),
        ] {
            let value = trans.and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }

        // The guarantee's machinery and the transition supplement, from the same detail sheet.
        for (column, places) in [
            (detail_columns::FUNDING_BASE, 2),
            (detail_columns::FUNDING_BASE_ECON_DIS, 2),
            (detail_columns::OPEN_ENROLLMENT_PRIOR, 6),
            (detail_columns::OPEN_ENROLLMENT_CURRENT, 6),
            (detail_columns::OPEN_ENROLLMENT_THRESHOLD, 2),
            (detail_columns::OPEN_ENROLLMENT_ADJUSTMENT, 2),
            (detail_columns::FY21_FUNDING_BASE, 2),
            (detail_columns::FORMULA_TRANSITION_SUPPLEMENT, 2),
        ] {
            let value = detail.get(irn).and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }

        // Preschool special education: six counts, six amounts, and the total they sum to.
        let prek = prek_sped.get(irn);
        for (column, places) in (0..6)
            .map(|k| (prek_sped_columns::FIRST_ADM + k, 6))
            .chain((0..6).map(|k| (prek_sped_columns::FIRST_AID + k, 2)))
            .chain(std::iter::once((prek_sped_columns::TOTAL, 2)))
        {
            let value = prek.and_then(|row| cell_number(row, column));
            out.last_mut()
                .expect("just pushed")
                .push(format_value(value, places));
        }
    }
    out
}

/// Columns of the district-profile fixture.
pub const PROFILE_HEADER: &[&str] = &[
    "irn",
    "district",
    "enrolled_adm_fy24",
    "econ_disadvantaged_pct_fy24",
    "assessed_valuation_per_pupil_fy23",
    "current_operating_millage_ty23",
    "effective_class1_millage_ty23",
    "operating_expenditure_per_pupil_fy24",
    "state_revenue_per_pupil_fy24",
    "local_revenue_per_pupil_fy24",
];

/// Reduce the District Profile Report's sixty columns to the ten the corpus uses.
#[must_use]
pub fn build_profile_extract(profile_rows: &[Vec<String>]) -> Vec<Vec<String>> {
    use profile_columns as p;
    let numeric = [
        p::ENROLLED_ADM,
        p::ECON_DISADVANTAGED,
        p::VALUATION_PER_PUPIL,
        p::CURRENT_OPERATING_MILLAGE,
        p::EFFECTIVE_CLASS1_MILLAGE,
        p::OPERATING_EXPENDITURE,
        p::STATE_REVENUE,
        p::LOCAL_REVENUE,
    ];
    profile_rows
        .iter()
        .skip(1)
        .filter(|row| !cell(row, p::IRN).trim().is_empty())
        .map(|row| {
            let mut record = Vec::with_capacity(PROFILE_HEADER.len());
            record.push(cell(row, p::IRN).trim().to_string());
            record.push(clean_name(cell(row, p::NAME)));
            record.extend(
                numeric
                    .iter()
                    .map(|index| format_value(number(cell(row, *index)), 4)),
            );
            record
        })
        .collect()
}

/// Columns of the 2024-25 report card fixture.
///
/// The two spending columns are the point of the file. `operating_expenditures_fy25` is a
/// dollar total, and the two ADM columns beside it are the two divisors the department
/// publishes for it. Anything computed per pupil from this fixture has to name which one it
/// used, because the answer changes.
pub const REPORT_CARD_HEADER: &[&str] = &[
    "irn",
    "district",
    "performance_index_2425",
    "performance_index_2324",
    "performance_index_2223",
    "unweighted_adm_fy25",
    "weighted_adm_fy25",
    "operating_expenditures_fy25",
    "exp_per_equivalent_pupil_fy25",
    "exp_per_equivalent_pupil_federal_fy25",
    "exp_per_equivalent_pupil_state_local_fy25",
    "progress_composite_2425",
    "progress_effect_size_2425",
    "progress_effect_size_1yr_2425",
    "econ_disadvantaged_pct_2425",
    "english_learner_pct_2425",
    "students_with_disabilities_pct_2425",
];

/// Column positions in the Achievement download's `Performance_Index` sheet.
mod achievement_columns {
    pub const IRN: usize = 0;
    pub const NAME: usize = 1;
    pub const PI_2425: usize = 6;
    pub const PI_2324: usize = 15;
    pub const PI_2223: usize = 16;
}

/// Column positions in `DISTRICT_SPENDING_PER_PUPIL`.
///
/// Columns 7 through 9 repeat the *statewide* figure on every row, identically. They are a
/// comparison band for the printed report card, not district data, and summing them would
/// produce a statewide average multiplied by 607.
mod spending_columns {
    pub const IRN: usize = 0;
    pub const EQUIVALENT: usize = 4;
    pub const FEDERAL: usize = 5;
    pub const STATE_AND_LOCAL: usize = 6;
}

/// Column positions in both Expanded List per-pupil sheets, which share a layout and differ
/// only in what column 3 divides by.
mod expanded_columns {
    pub const IRN: usize = 0;
    pub const ORG_TYPE: usize = 2;
    pub const ADM: usize = 3;
    pub const OPERATING_EXPENDITURES: usize = 4;
}

/// Column positions in `OVERALL_VALUE_ADDED_OVERVIEW`.
///
/// # Two growth numbers, and only one of them compares districts
///
/// `Overall Composite` is a precision-scaled statistic — a gain divided by its standard error —
/// so it grows with the number of tested students and correlates with enrollment at +0.24.
/// `Overall Effect Size` is the standardised gain and does not. Ranking districts on the
/// composite ranks them partly by size; every association in this corpus uses the effect size.
mod value_added_columns {
    pub const IRN: usize = 0;
    pub const COMPOSITE: usize = 5;
    pub const EFFECT_SIZE: usize = 6;
    /// On the `*_GAINS` sheets the effect size sits one column earlier than on the overview,
    /// because those sheets carry no star rating.
    pub const GAINS_EFFECT_SIZE: usize = 5;
}

/// Column positions in `District_Details`, which is in long form: one row per district per
/// student group, rather than one row per district.
mod details_columns {
    pub const IRN: usize = 0;
    pub const STUDENT_GROUP: usize = 4;
    pub const ENROLLMENT_PERCENT: usize = 6;
}

/// Student-group labels this corpus reads from `District_Details`.
///
/// # The economic-disadvantage share here is not the Cupp Report's
///
/// This one is top-coded: 87 districts report exactly 100.0% and 197 report 95% or more,
/// because community eligibility counts every student in a qualifying district as economically
/// disadvantaged. The Cupp Report's FY2024 measure puts 37 districts at 100%. They correlate at
/// only +0.823 and against the Performance Index they give -0.734 and -0.846 respectively — the
/// censored one weaker, as censoring predicts. Both are committed; neither substitutes for the
/// other.
const ECONOMIC_DISADVANTAGE: &str = "Economic Disadvantage";
/// Reported for about half of districts; the rest are suppressed at fewer than ten students.
const ENGLISH_LEARNER: &str = "English Learner";
/// Reported for every district.
const STUDENTS_WITH_DISABILITIES: &str = "Students with Disabilities";

/// Columns of the FY2025 expenditure-function fixture.
pub const FUNCTIONS_HEADER: &[&str] = &[
    "irn",
    "district",
    "unweighted_adm_fy25",
    "operating_expenditure_per_pupil_fy25",
    "instruction_per_pupil",
    "pupil_support_per_pupil",
    "instructional_staff_support_per_pupil",
    "classroom_instruction_per_pupil",
    "general_admin_per_pupil",
    "school_admin_per_pupil",
    "operations_maintenance_per_pupil",
    "pupil_transportation_per_pupil",
    "other_support_per_pupil",
    "food_service_per_pupil",
    "nonclassroom_per_pupil",
];

/// Column positions on the Expanded List's `Expenditure per Pupil` sheet.
///
/// Everything from `INSTRUCTION` onward is already a per-pupil dollar figure on the unweighted
/// denominator; `OPERATING_TOTAL` is a whole-district dollar total. Mixing the two is the
/// obvious error and the reason they are named apart here.
///
/// The department's two roll-ups partition operating spending exactly:
/// `CLASSROOM_INSTRUCTION` = instruction + pupil support + instructional staff support, and
/// `NONCLASSROOM` = the six administrative, plant, transport and food rows. The pair sums to
/// operating expenditure per pupil, which
/// [`crates/dispersion/tests/expenditure_functions_fy25.rs`](../../dispersion/tests/expenditure_functions_fy25.rs)
/// checks rather than assumes. Construction, debt and non-operating rows sit outside it and are
/// deliberately not carried here.
mod function_columns {
    pub const ADM: usize = 3;
    pub const OPERATING_TOTAL: usize = 4;
    pub const INSTRUCTION: usize = 7;
    pub const PUPIL_SUPPORT: usize = 8;
    pub const INSTRUCTIONAL_STAFF_SUPPORT: usize = 9;
    pub const CLASSROOM_INSTRUCTION: usize = 10;
    pub const GENERAL_ADMIN: usize = 11;
    pub const SCHOOL_ADMIN: usize = 12;
    pub const OPERATIONS_MAINTENANCE: usize = 13;
    pub const PUPIL_TRANSPORTATION: usize = 14;
    pub const OTHER_SUPPORT: usize = 15;
    pub const FOOD_SERVICE: usize = 16;
    pub const NONCLASSROOM: usize = 17;
}

/// Extract per-pupil operating spending by function for traditional districts.
///
/// Reads the `Expenditure per Pupil` sheet — the headcount-denominator one. The department also
/// publishes the identical layout over weighted ADM, and every figure here would be about a
/// fifth smaller on that sheet without a single function changing its share.
#[must_use]
pub fn build_function_extract(unweighted_rows: &[Vec<String>]) -> Vec<Vec<String>> {
    use function_columns as f;
    let per_pupil = [
        f::INSTRUCTION,
        f::PUPIL_SUPPORT,
        f::INSTRUCTIONAL_STAFF_SUPPORT,
        f::CLASSROOM_INSTRUCTION,
        f::GENERAL_ADMIN,
        f::SCHOOL_ADMIN,
        f::OPERATIONS_MAINTENANCE,
        f::PUPIL_TRANSPORTATION,
        f::OTHER_SUPPORT,
        f::FOOD_SERVICE,
        f::NONCLASSROOM,
    ];
    let mut out: Vec<Vec<String>> = rows_by_key(unweighted_rows, expanded_columns::IRN)
        .filter(|(_, row)| cell(row, expanded_columns::ORG_TYPE).trim() == PUBLIC_DISTRICT)
        .map(|(irn, row)| {
            let adm = number(cell(row, f::ADM));
            let total = number(cell(row, f::OPERATING_TOTAL));
            let mut record = vec![
                irn.to_string(),
                clean_name(cell(row, expanded_columns::IRN + 1)),
                format_value(adm, 4),
                format_value(
                    match (total, adm) {
                        (Some(t), Some(a)) if a > 0.0 => Some(t / a),
                        _ => None,
                    },
                    2,
                ),
            ];
            record.extend(
                per_pupil
                    .iter()
                    .map(|i| format_value(number(cell(row, *i)), 2)),
            );
            record
        })
        .collect();
    out.sort_by(|a, b| a[0].cmp(&b[0]));
    out
}

/// The Expanded List's label for a traditional district.
///
/// The file also carries 320 community schools, 49 JVSDs, 19 eschools and 8 STEM schools. The
/// report card's district files carry none of them, so an unfiltered join drops silently to the
/// intersection and looks like it worked.
const PUBLIC_DISTRICT: &str = "Public District";

/// Join the three 2024-25 report card publications on IRN.
///
/// Rows are emitted only for districts present in the achievement file with a Performance
/// Index, which is the rated population — the spending and enrollment files are wider.
#[must_use]
pub fn build_report_card_extract<'a>(
    achievement_rows: &'a [Vec<String>],
    spending_rows: &'a [Vec<String>],
    weighted_rows: &'a [Vec<String>],
    unweighted_rows: &'a [Vec<String>],
    value_added_rows: &'a [Vec<String>],
    one_year_gain_rows: &'a [Vec<String>],
    details_rows: &'a [Vec<String>],
) -> Vec<Vec<String>> {
    let spending: HashMap<&str, &Vec<String>> =
        rows_by_key(spending_rows, spending_columns::IRN).collect();

    // The Expanded List is keyed by IRN across every org type it covers, so the filter is part
    // of the lookup rather than a step after it. Without it the join drops silently to the
    // intersection and looks like it worked.
    let districts_only = |rows: &'a [Vec<String>]| -> HashMap<&'a str, &'a Vec<String>> {
        rows_by_key(rows, expanded_columns::IRN)
            .filter(|(_, row)| cell(row, expanded_columns::ORG_TYPE).trim() == PUBLIC_DISTRICT)
            .collect()
    };
    let weighted = districts_only(weighted_rows);
    let unweighted = districts_only(unweighted_rows);
    let value_added: HashMap<&str, &Vec<String>> =
        rows_by_key(value_added_rows, value_added_columns::IRN).collect();
    let one_year: HashMap<&str, &Vec<String>> =
        rows_by_key(one_year_gain_rows, value_added_columns::IRN).collect();

    // Long form: key on (IRN, student group) rather than IRN alone.
    let mut details: HashMap<(&str, &str), &Vec<String>> = HashMap::new();
    for (irn, row) in rows_by_key(details_rows, details_columns::IRN) {
        details.insert((irn, cell(row, details_columns::STUDENT_GROUP).trim()), row);
    }

    achievement_rows
        .iter()
        .skip(1)
        .filter(|row| !cell(row, achievement_columns::IRN).trim().is_empty())
        .filter(|row| !is_statewide_row(row, achievement_columns::NAME))
        .filter(|row| number(cell(row, achievement_columns::PI_2425)).is_some())
        .map(|row| {
            let irn = cell(row, achievement_columns::IRN).trim().to_string();
            let spend = spending.get(irn.as_str()).copied();
            let growth = value_added.get(irn.as_str()).copied();
            let gain = one_year.get(irn.as_str()).copied();
            let share = |group: &str| {
                details
                    .get(&(irn.as_str(), group))
                    .and_then(|r| number(cell(r, details_columns::ENROLLMENT_PERCENT)))
            };
            let wtd = weighted.get(irn.as_str()).copied();
            let unw = unweighted.get(irn.as_str()).copied();

            let from = |table: Option<&Vec<String>>, index: usize| {
                table.and_then(|r| number(cell(r, index)))
            };

            vec![
                irn.clone(),
                clean_name(cell(row, achievement_columns::NAME)),
                format_value(number(cell(row, achievement_columns::PI_2425)), 1),
                format_value(number(cell(row, achievement_columns::PI_2324)), 1),
                format_value(number(cell(row, achievement_columns::PI_2223)), 1),
                format_value(from(unw, expanded_columns::ADM), 4),
                format_value(from(wtd, expanded_columns::ADM), 4),
                format_value(from(wtd, expanded_columns::OPERATING_EXPENDITURES), 2),
                format_value(from(spend, spending_columns::EQUIVALENT), 2),
                format_value(from(spend, spending_columns::FEDERAL), 2),
                format_value(from(spend, spending_columns::STATE_AND_LOCAL), 2),
                format_value(from(growth, value_added_columns::COMPOSITE), 2),
                format_value(from(growth, value_added_columns::EFFECT_SIZE), 2),
                format_value(from(gain, value_added_columns::GAINS_EFFECT_SIZE), 2),
                format_value(share(ECONOMIC_DISADVANTAGE), 1),
                format_value(share(ENGLISH_LEARNER), 1),
                format_value(share(STUDENTS_WITH_DISABILITIES), 1),
            ]
        })
        .collect()
}

/// Write a fixture with LF endings, so git sees no spurious churn on a rebuild.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] if the directory cannot be created or the file written.
pub fn write_csv(path: &Path, header: &[&str], rows: &[Vec<String>]) -> io::Result<usize> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out = String::with_capacity(rows.len() * 160 + 256);
    out.push_str(&header.join(","));
    out.push('\n');
    for row in rows {
        // This writer does not quote, and every consumer of these fixtures splits on the comma.
        // A field carrying one shifts that row's remaining columns and nothing downstream can
        // tell — the MR-81 panel shipped 99 such rows on its first build, because Ohio has
        // sponsors called "Holy Trinity, Swanton Ele Sch" and "Edge Academy, The". Failing here
        // is the only place the problem is visible; by the time a calculator reads the file, the
        // corruption looks like data.
        assert!(
            !row.iter().any(|field| field.contains(',')),
            "a field contains a comma and this writer does not quote: {row:?}"
        );
        out.push_str(&row.join(","));
        out.push('\n');
    }
    fs::write(path, out)?;
    Ok(rows.len())
}

/// Where each fixture is written, relative to the repository root.
pub const FY27_FIXTURE: &str = "crates/foundation/fixtures/fy27-department-model.csv";
/// Where the district-profile fixture is written, relative to the repository root.
pub const PROFILE_FIXTURE: &str = "crates/dispersion/fixtures/cupp-fy24-district-data.csv";
/// Where the grade-band headcount fixture is written, relative to the repository root.
pub const GRADE_BANDS_FIXTURE: &str = "crates/foundation/fixtures/fy24-district-grade-bands.csv";

/// Columns of the grade-band fixture.
pub const GRADE_BANDS_HEADER: &[&str] = &[
    "irn",
    "district",
    "enrolled_adm_fy24",
    "headcount_kindergarten",
    "headcount_grades_1_3",
    "headcount_grades_4_8",
    "headcount_grades_9_12",
];

/// Column positions in the October headcount sheet, `fy24_hdcnt_dist`.
///
/// One column per individual grade, which is the whole reason the file is worth retrieving: the
/// funding calculator publishes ADM already collapsed into the formula's four bands, and this is
/// the only source that shows what went into them.
mod headcount_columns {
    pub const KINDERGARTEN: usize = 4;
    /// Grades 1 through 12 are consecutive from here.
    pub const GRADE_1: usize = 5;
}

/// A fixed-decimal rendering that keeps its trailing zeros.
///
/// [`format_value`] trims them, which is right for a figure whose precision varies. Enrolled ADM
/// is published to exactly four places and this fixture has always carried `847.8230`, so
/// trimming would rewrite every row of it.
fn format_fixed(value: Option<f64>, places: usize) -> String {
    value.map_or_else(String::new, |value| format!("{value:.places$}"))
}

/// Sum a run of consecutive grade columns, treating a suppressed count as unknown.
///
/// `None` if any grade in the band was withheld. A band summed over a `<10` is not a smaller
/// band, it is a band whose total is not known, and the difference lands hardest in exactly the
/// small districts where suppression happens.
fn band(row: &[String], first_grade: usize, grades: std::ops::Range<usize>) -> Option<f64> {
    let mut total = 0.0;
    for grade in grades {
        total += cell_number(row, first_grade + grade - 1)?;
    }
    Some(total)
}

/// Join October headcount by individual grade to the profile report's district list.
///
/// Name and enrolled ADM come from the profile report, headcounts from the October file. That is
/// why the fixture holds 606 districts rather than the headcount file's 609: the three the
/// profile report omits are the three smallest in Ohio, and their grade bands are real but place
/// against nothing else the corpus holds.
///
/// # `<10` is not zero, and the fixture this replaces said it was
///
/// Five districts have a grade whose count the department withholds because publishing it would
/// identify students. A band containing one is **blank** here, not a sum of the grades that were
/// published: Bloomfield-Mespo's grades 9-12 are 19, 13, 12 and a withheld count, and the
/// LibreOffice-derived fixture recorded 44 — asserting the fourth grade has no students when it
/// has between one and nine. The convention was already written down in
/// [`crate::conventions::SUPPRESSED`]; the pipeline that produced that fixture did not follow it.
#[must_use]
pub fn build_grade_bands(
    headcount_rows: &[Vec<String>],
    profile_rows: &[Vec<String>],
) -> Vec<Vec<String>> {
    use headcount_columns as h;

    // The profile workbook puts the district name in column 0 and the IRN in column 1 — the
    // opposite way round from every other sheet here, and the reason this join silently produced
    // nothing the first time it ran.
    let covered: Vec<(&str, &Vec<String>)> = profile_rows
        .iter()
        .skip(1)
        .map(|row| (cell(row, profile_columns::IRN).trim(), row))
        .filter(|(irn, _)| !irn.is_empty())
        .collect();

    let mut out: Vec<Vec<String>> = Vec::new();
    for (irn, row) in rows_by_key(headcount_rows, 0) {
        let Some((_, profile)) = covered.iter().find(|(key, _)| *key == irn) else {
            continue;
        };
        out.push(vec![
            irn.to_string(),
            clean_name(cell(profile, profile_columns::NAME)),
            format_fixed(cell_number(profile, profile_columns::ENROLLED_ADM), 4),
            format_value(cell_number(row, h::KINDERGARTEN), 0),
            format_value(band(row, h::GRADE_1, 1..4), 0),
            format_value(band(row, h::GRADE_1, 4..9), 0),
            format_value(band(row, h::GRADE_1, 9..13), 0),
        ]);
    }
    // Sorted by the composite name, which is the order this fixture has always been in.
    out.sort_by(|a, b| a[1].cmp(&b[1]));
    out
}

/// Where the per-district financial panel is written, relative to the repository root.
pub const FINANCE_FIXTURE: &str = "crates/project/fixtures/district-finances.csv";

/// Header of the financial panel: one row per district per fiscal year.
pub const FINANCE_HEADER: &[&str] = &[
    "irn",
    "name",
    "county",
    "fiscal_year",
    "unrestricted_aid",
    "restricted_aid",
    "property_tax",
    "income_tax",
    "property_tax_allocation",
    "total_revenue",
    "total_revenue_and_sources",
    "total_expenditure",
    "beginning_cash",
    "ending_cash",
];

/// Reduce a set of five-year-forecast filings to one row per district per **actual** fiscal year.
///
/// # Only actuals, and only once each
///
/// A filing carries three prior actuals and five forecast years; only the actuals are kept, for
/// the reason in [`crate::forecast`] — a submitted forecast is a treasurer's projection made
/// under incentives, not a measurement.
///
/// The pinned filings are three years apart so their actual windows tile without overlapping,
/// and if that ever stops being true the later filing wins: a restatement is the department's
/// most recent word on a closed year. The seam between two filings is checkable — one's last
/// ending cash balance is the next's first beginning balance, to the dollar — and
/// `crates/project` tests exactly that.
///
/// Districts are emitted in IRN order, then fiscal year, so the fixture diffs cleanly.
#[must_use]
pub fn build_finance_extract(filings: &[Vec<crate::forecast::Line>]) -> Vec<Vec<String>> {
    use std::collections::BTreeMap;

    // (irn, fiscal year) -> (name, county, line code -> amount). BTreeMap so the output order is
    // the key order rather than a hash order that changes between builds.
    type Cell = (String, String, BTreeMap<String, f64>);
    let mut panel: BTreeMap<(String, u16), Cell> = BTreeMap::new();

    for filing in filings {
        for line in filing {
            for (index, fiscal_year) in line.actual_years().into_iter().enumerate() {
                let entry = panel
                    .entry((line.irn.clone(), fiscal_year))
                    .or_insert_with(|| (line.name.clone(), line.county.clone(), BTreeMap::new()));
                entry.0 = line.name.clone();
                entry.1 = line.county.clone();
                entry.2.insert(line.code.clone(), line.actual[index]);
            }
        }
    }

    panel
        .into_iter()
        .map(|((irn, fiscal_year), (name, county, amounts))| {
            let mut row = vec![irn, name, county, fiscal_year.to_string()];
            for code in crate::forecast::EXTRACTED {
                row.push(format_value(
                    Some(amounts.get(*code).copied().unwrap_or(0.0)),
                    2,
                ));
            }
            row
        })
        .collect()
}

/// Where the SD-1 taxable value and taxes charged panel is written, relative to the root.
pub const SD1_FIXTURE: &str = "crates/dispersion/fixtures/sd1-district-taxes.csv";

/// Header of the SD-1 panel: one row per district per **tax** year.
///
/// Tax year, not fiscal year, and the column is named so that nothing joins the two by accident.
/// A tax year's charge is collected in the *following* calendar year, half in the first part of
/// it and half in July — so TY2024 money arrives across FY2025 and FY2026. See
/// [`build_sd1_extract`].
pub const SD1_HEADER: &[&str] = &[
    "irn",
    "district",
    "county",
    "tax_year",
    "agricultural_value",
    "residential_value",
    "class1_value",
    "mineral_value",
    "industrial_value",
    "commercial_value",
    "railroad_value",
    "class2_value",
    "real_property_value",
    "public_utility_value",
    "total_value",
    "class1_taxes_charged",
    "class2_taxes_charged",
    "real_property_taxes_charged",
    "real_property_taxes_charged_with_jvsd",
    "public_utility_taxes_charged",
    "class1_rate",
    "class2_rate",
    "real_property_millage",
    "public_utility_millage",
    "value_per_pupil",
    "adm",
];

/// Column positions on SD-1's per-district worksheets.
///
/// Both worksheets in a workbook carry the same 28 columns in the same order, and so do both
/// tax years — it is the *sheet names* that drift, not the layout. The banner above the header
/// is a row shorter in the TY2023 workbook, which is why nothing here indexes rows by number.
mod sd1_columns {
    pub const COUNTY: usize = 0;
    pub const IRN: usize = 2;
    pub const NAME: usize = 4;
    pub const AGRICULTURAL: usize = 5;
    pub const RESIDENTIAL: usize = 6;
    pub const CLASS1_VALUE: usize = 7;
    pub const MINERAL: usize = 8;
    pub const INDUSTRIAL: usize = 9;
    pub const COMMERCIAL: usize = 10;
    pub const RAILROAD: usize = 11;
    pub const CLASS2_VALUE: usize = 12;
    pub const REAL_VALUE: usize = 13;
    pub const PUBLIC_UTILITY_VALUE: usize = 14;
    pub const TOTAL_VALUE: usize = 15;
    pub const CLASS1_TAXES: usize = 16;
    pub const CLASS2_TAXES: usize = 17;
    pub const REAL_TAXES: usize = 18;
    pub const PUBLIC_UTILITY_TAXES: usize = 19;
    pub const CLASS1_RATE: usize = 21;
    pub const CLASS2_RATE: usize = 22;
    pub const REAL_MILLS: usize = 23;
    pub const PUBLIC_UTILITY_MILLS: usize = 24;
    pub const VALUE_PER_PUPIL: usize = 25;
    pub const ADM: usize = 27;
}

/// One tax year of SD-1, as its two per-district worksheets.
///
/// The department publishes the same districts twice in one workbook, differing in one thing:
/// whether the joint vocational school district's operating levy is counted in the taxes
/// charged. 501 of 611 districts are in a JVSD, so the choice moves most of the state — $513
/// million in TY2024 — and carrying only one of the two would make the fixture silently take a
/// side on a question its callers should be able to ask.
#[derive(Debug, Clone, Copy)]
pub struct Sd1Year<'a> {
    /// The tax year the charge was levied for.
    pub tax_year: u16,
    /// The `ExJVS…` worksheet: JVSD operating levies removed.
    pub excluding_jvsd: &'a [Vec<String>],
    /// The `SD1DAT…` worksheet: the same districts with the JVSD levy left in.
    pub including_jvsd: &'a [Vec<String>],
}

/// Reduce SD-1 to one row per district per tax year.
///
/// # What "taxes charged" is, and is not
///
/// It is a **levy**, not a receipt. Three consequences, each of which has produced a wrong
/// number somewhere:
///
/// - **It is gross of the credits the state reimburses.** The department's own note says the
///   figures "include taxes that have been reduced under various property tax programs that are
///   reimbursed to local school districts by the state" — the non-business credit, the
///   owner-occupancy credit and the homestead exemption. About a tenth of what this column calls
///   property tax is state money. Treating it as the local share overstates local effort.
/// - **It is a tax year, not a fiscal year.** A TY2024 charge is collected across calendar 2025,
///   half early and half in July, so it straddles FY2025 and FY2026. Dividing it by a
///   single-fiscal-year denominator counts money the district had not yet received — which is
///   largest exactly where a new levy has just passed.
/// - **Charged is not collected.** Delinquency means receipts run below the charge, and by a
///   district-specific amount this table cannot see.
///
/// Emitted in IRN then tax-year order, so the fixture diffs cleanly.
#[must_use]
pub fn build_sd1_extract(years: &[Sd1Year<'_>]) -> Vec<Vec<String>> {
    use sd1_columns as c;

    let value_columns = [
        c::AGRICULTURAL,
        c::RESIDENTIAL,
        c::CLASS1_VALUE,
        c::MINERAL,
        c::INDUSTRIAL,
        c::COMMERCIAL,
        c::RAILROAD,
        c::CLASS2_VALUE,
        c::REAL_VALUE,
        c::PUBLIC_UTILITY_VALUE,
        c::TOTAL_VALUE,
        c::CLASS1_TAXES,
        c::CLASS2_TAXES,
        c::REAL_TAXES,
    ];
    let rate_columns = [
        c::PUBLIC_UTILITY_TAXES,
        c::CLASS1_RATE,
        c::CLASS2_RATE,
        c::REAL_MILLS,
        c::PUBLIC_UTILITY_MILLS,
        c::VALUE_PER_PUPIL,
        c::ADM,
    ];

    let mut out: Vec<Vec<String>> = Vec::new();
    for year in years {
        // The JVSD-inclusive total, by IRN. Looked up rather than zipped: the two worksheets
        // happen to agree on row order today, and relying on that would break silently.
        let with_jvsd: Vec<(&str, &Vec<String>)> =
            rows_by_key(year.including_jvsd, c::IRN).collect();

        for (key, row) in rows_by_key(year.excluding_jvsd, c::IRN) {
            let irn = pad_irn(key);
            let mut record = vec![
                irn,
                clean_name(cell(row, c::NAME)),
                clean_name(cell(row, c::COUNTY)),
                year.tax_year.to_string(),
            ];
            record.extend(
                value_columns
                    .iter()
                    .map(|i| format_value(cell_number(row, *i), 0)),
            );
            record.push(format_value(
                with_jvsd
                    .iter()
                    .find(|(other, _)| *other == key)
                    .and_then(|(_, other)| cell_number(other, c::REAL_TAXES)),
                0,
            ));
            record.extend(
                rate_columns
                    .iter()
                    .map(|i| format_value(cell_number(row, *i), 4)),
            );
            out.push(record);
        }
    }
    out.sort_by(|a, b| a[0].cmp(&b[0]).then(a[3].cmp(&b[3])));
    out
}

/// Left-pad an Information Retrieval Number to the six digits every other fixture here uses.
///
/// The Department of Taxation stores the IRN as a number, so Manchester Local's `000442` arrives
/// as `442` and Ohio Valley's `061903` as `61903`. Joining that against the department of
/// education's zero-padded keys matches on neither, and the failure mode is an empty join rather
/// than an error.
fn pad_irn(key: &str) -> String {
    if key.len() >= 6 {
        key.to_string()
    } else {
        format!("{key:0>6}")
    }
}

/// Where the per-district casino distribution panel is written, relative to the root.
pub const CASINO_FIXTURE: &str = "crates/dispersion/fixtures/casino-district-distributions.csv";

/// Header of the casino panel: one row per district per **distribution**.
///
/// The grain is the payment, not the year. Money moves twice a year under R.C. 5753.03(D)(2) and
/// the halves are not interchangeable: the closure that took January-June 2020 down to a third of
/// a normal half-year is invisible in any annual figure that averages it with the half beside it.
///
/// `counties` is how many county funds a district was paid out of, and it is empty for the three
/// distributions the department published only as a statewide list. It is not a district's county
/// — a district appears once per county it has resident students in, which for the statewide
/// e-schools is all 88.
pub const CASINO_HEADER: &[&str] = &["irn", "district", "distribution", "counties", "amount"];

/// One published sheet of one distribution.
#[derive(Debug, Clone, Copy)]
pub struct CasinoSheet<'a> {
    /// The month the money was paid, `YYYY-MM`. January or August, and it is the identity of a
    /// distribution: the revenue half-year follows from it by statute.
    pub distribution: &'a str,
    /// What to call this sheet when a check on it fails.
    pub label: &'a str,
    /// Its rows.
    pub rows: &'a [Vec<String>],
}

/// Which of the two layouts the department used for a sheet.
///
/// Detected from the header row rather than declared per file, because the layout is a fact about
/// the sheet and the department changed it twice inside one series. Column *positions* are read
/// off the header too: the amount sits in `E` in most years and in `G` in the statewide layout and
/// in the one true-up year, and a fixed index would read the wrong column in three of eighteen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CasinoLayout {
    /// County code, county name, IRN, district name, amount. One row per district per county.
    ByCounty {
        county: usize,
        irn: usize,
        name: usize,
        amount: usize,
    },
    /// IRN, district name, amount. One row per district, statewide.
    ByDistrict {
        irn: usize,
        name: usize,
        amount: usize,
    },
}

/// The row the department puts at the foot of the data block, carrying its own total.
///
/// It is the reason a digit filter is not optional. The row's amount sits in the amount column
/// like every district's, so a parser that sums the column without checking what is in the key
/// column reports exactly twice the distribution — and $90 million for a half-year is not
/// obviously wrong to anyone who has not seen the series.
const CASINO_TOTAL_ROW: &str = "Total Distribution Amount";

/// Find the header row and what layout it describes.
fn casino_layout(rows: &[Vec<String>]) -> Option<(usize, CasinoLayout)> {
    for (index, row) in rows.iter().enumerate() {
        let Some(irn) = row.iter().position(|cell| cell.trim() == "IRN") else {
            continue;
        };
        // The amount column is the *last* header cell naming a distribution. In the true-up year
        // there are three, and the first two are the halves of the third.
        let Some(amount) = row
            .iter()
            .rposition(|cell| cell.trim().contains("istr") && cell.trim().contains("ution"))
        else {
            continue;
        };
        let county = row.iter().position(|cell| cell.trim() == "County Code");
        let layout = match county {
            Some(county) => CasinoLayout::ByCounty {
                county,
                irn,
                name: irn + 1,
                amount,
            },
            None => CasinoLayout::ByDistrict {
                irn,
                name: irn + 1,
                amount,
            },
        };
        return Some((index, layout));
    }
    None
}

/// The half-year a distribution paid for, from the month it was paid in.
///
/// R.C. 5753.03(D)(2) transfers the tax quarterly and pays districts twice a year, in January for
/// the half-year that ended in December and in August for the one that ended in June. Fifteen of
/// the eighteen sheets print the period in their title banner; three do not, and rather than leave
/// those three unlabelled the rule is applied to all of them and checked against every banner that
/// exists. A rule that is checked eighteen times a rebuild is not an assumption.
#[must_use]
pub fn casino_statutory_period(distribution: &str) -> Option<(String, String)> {
    let (year, month) = distribution.split_once('-')?;
    let year: i32 = year.parse().ok()?;
    match month {
        "01" => Some((format!("{}-07-01", year - 1), format!("{}-12-31", year - 1))),
        "08" => Some((format!("{year}-01-01"), format!("{year}-06-30"))),
        _ => None,
    }
}

/// The half-year a sheet says it is for, read out of its title banner.
///
/// `None` where the banner names only the month the money moved, which is what the four sheets of
/// the combined FY2016-FY2017 workbook do.
#[must_use]
pub fn casino_printed_period(rows: &[Vec<String>]) -> Option<(String, String)> {
    for row in rows.iter().take(8) {
        for cell in row {
            if let Some((from, rest)) = cell.split_once(" - ") {
                if let (Some(start), Some(end)) = (casino_date(from), casino_date(rest)) {
                    return Some((start, end));
                }
            }
        }
    }
    None
}

/// `Jun 30, 2015` and anything following it, as `2015-06-30`.
fn casino_date(text: &str) -> Option<String> {
    let text = text.trim();
    let (month, rest) = text.split_once(' ')?;
    let month = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .iter()
    .position(|name| name.eq_ignore_ascii_case(month))?
        + 1;
    let (day, rest) = rest.split_once(", ")?;
    let day: u32 = day.trim().parse().ok()?;
    let year: i32 = rest
        .split_whitespace()
        .next()?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()?;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// One district's share of one distribution, before the sheets are merged.
struct CasinoRow {
    name: String,
    counties: usize,
    amount: f64,
}

/// Reduce the published distribution sheets to one row per district per distribution.
///
/// # What the sheets are made to prove about themselves
///
/// Three checks, each of which has a way of failing that produces a plausible number rather than
/// an error, which is why each is a check and not a comment:
///
/// - **The foot total.** Every sheet prints its own `Total Distribution Amount`, and it must equal
///   the district rows summed. This is the property worth looking for in any new source: the file
///   states an aggregate beside the parts, so a column read from the wrong offset, a dropped
///   continuation page or a doubled row is caught by the source rather than by a reviewer's sense
///   of scale.
/// - **The banner.** Where a sheet names its half-year, it must be the half-year
///   [`casino_statutory_period`] derives from the payment month.
/// - **The overlap.** August 2015 is published twice, once by county and once statewide. The two
///   files are different layouts written by different reports, and they must agree on every
///   district and on the total.
///
/// # Two rules about which rows exist
///
/// Rows whose amount is zero are dropped. The statewide layout lists districts certified at no
/// students — 22 of the 1,066 in August 2015, closed community schools for the most part — and the
/// county layout omits them. Keeping them would make a district's presence in the panel depend on
/// which layout the department happened to use that year, which is a fact about the report and not
/// about the district.
///
/// A district's county rows are summed, and how many there were is kept. That count is the
/// apportionment R.C. 5753.11 describes rather than a duplicate: a district is paid out of every
/// county fund it has resident students in, which for a statewide e-school is all 88 of them.
///
/// # Errors
///
/// Returns the first check that fails, naming the sheet.
pub fn build_casino_extract(sheets: &[CasinoSheet<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut panel: BTreeMap<(String, String), CasinoRow> = BTreeMap::new();
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();

    for sheet in sheets {
        let (header, layout) = casino_layout(sheet.rows)
            .ok_or_else(|| format!("{}: no IRN header row", sheet.label))?;
        let (irn_column, name_column, amount_column, county_column) = match layout {
            CasinoLayout::ByCounty {
                county,
                irn,
                name,
                amount,
            } => (irn, name, amount, Some(county)),
            CasinoLayout::ByDistrict { irn, name, amount } => (irn, name, amount, None),
        };

        if let Some((start, end)) = casino_printed_period(sheet.rows) {
            let derived = casino_statutory_period(sheet.distribution).ok_or_else(|| {
                format!(
                    "{}: {} is not a payment month",
                    sheet.label, sheet.distribution
                )
            })?;
            if (start.clone(), end.clone()) != derived {
                return Err(format!(
                    "{}: banner says {start}..{end}, the payment month says {}..{}",
                    sheet.label, derived.0, derived.1
                ));
            }
        }

        let mut published: Option<f64> = None;
        let mut summed = 0.0;
        let mut fresh: BTreeMap<String, CasinoRow> = BTreeMap::new();

        for row in &sheet.rows[header + 1..] {
            let key = cell(row, irn_column).trim();
            if key.starts_with(CASINO_TOTAL_ROW)
                || cell(row, 0).trim().starts_with(CASINO_TOTAL_ROW)
            {
                published = cell_number(row, amount_column);
                continue;
            }
            if key.len() != 6 || !key.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            let Some(amount) = cell_number(row, amount_column) else {
                continue;
            };
            summed += amount;
            if amount == 0.0 {
                continue;
            }
            let entry = fresh.entry(key.to_string()).or_insert_with(|| CasinoRow {
                name: clean_name(cell(row, name_column)),
                counties: 0,
                amount: 0.0,
            });
            entry.amount += amount;
            if county_column.is_some() {
                entry.counties += 1;
            }
        }

        let published =
            published.ok_or_else(|| format!("{}: no {CASINO_TOTAL_ROW} row", sheet.label))?;
        if (published - summed).abs() >= 0.005 {
            return Err(format!(
                "{}: rows sum to {summed:.2}, the sheet says {published:.2}",
                sheet.label
            ));
        }

        match seen.insert(sheet.distribution.to_string(), sheet.label) {
            None => {
                for (irn, row) in fresh {
                    panel.insert((sheet.distribution.to_string(), irn), row);
                }
            }
            Some(first) => {
                for (irn, row) in fresh {
                    let key = (sheet.distribution.to_string(), irn.clone());
                    let held = panel.get(&key).ok_or_else(|| {
                        format!(
                            "{}: {irn} is paid in {} and absent from {first}",
                            sheet.label, sheet.distribution
                        )
                    })?;
                    if (held.amount - row.amount).abs() >= 0.005 {
                        return Err(format!(
                            "{} and {first} disagree about {irn} in {}: {:.2} against {:.2}",
                            sheet.label, sheet.distribution, row.amount, held.amount
                        ));
                    }
                }
            }
        }
    }

    Ok(panel
        .into_iter()
        .map(|((distribution, irn), row)| {
            vec![
                irn,
                row.name,
                distribution,
                if row.counties == 0 {
                    String::new()
                } else {
                    row.counties.to_string()
                },
                format_value(Some(row.amount), 2),
            ]
        })
        .collect())
}

/// Where the deflator's check fixture is written, relative to the repository root.
pub const CPI_FIXTURE: &str = "crates/connect/fixtures/cpi-u-june.tsv";
/// Where the 2024-25 report card fixture is written, relative to the repository root.
pub const REPORT_CARD_FIXTURE: &str =
    "crates/dispersion/fixtures/report-card-2425-district-data.csv";
/// Where the FY2025 expenditure-function fixture is written, relative to the repository root.
pub const FUNCTIONS_FIXTURE: &str = "crates/dispersion/fixtures/expenditure-functions-fy25.csv";

/// Reduce the Bureau's 2.7 MB all-series flat file to the one series and period the deflator
/// uses.
///
/// Committing the extract rather than the whole file is what makes the deflator's verification
/// hermetic: [`deflate`](../../deflate/) can claim its index points are checked against the
/// agency, and a test proves it without a network. Lines are kept in their published form so
/// [`crate::cpi::parse_series`] reads the extract and the original identically.
#[must_use]
pub fn build_cpi_extract(text: &str, series_id: &str, period: &str) -> String {
    let mut out = String::with_capacity(8 * 1024);
    let mut lines = text.lines();
    if let Some(header) = lines.next() {
        out.push_str(header);
        out.push('\n');
    }
    for line in lines {
        let mut fields = line.split('\t').map(str::trim);
        if fields.next() == Some(series_id) && fields.nth(1) == Some(period) {
            out.push_str(line.trim_end());
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conventions::STATEWIDE_ROW;

    /// Rows of a casino sheet in the county layout, banner and foot total included.
    fn casino_county_sheet(
        banner: &str,
        districts: &[(&str, &str, &str, &str)],
    ) -> Vec<Vec<String>> {
        let mut rows = vec![
            vec![
                String::new(),
                String::new(),
                "Ohio Department of Taxation".into(),
            ],
            vec![String::new(), String::new(), banner.into()],
            vec![
                "County Code".into(),
                "County Name".into(),
                "IRN".into(),
                "SD Name".into(),
                "Distrubution Amount".into(),
            ],
        ];
        let mut total = 0.0;
        for (county, name, irn, amount) in districts {
            total += amount.parse::<f64>().unwrap_or_default();
            rows.push(vec![
                (*county).to_string(),
                "SOMEWHERE".into(),
                (*irn).to_string(),
                (*name).to_string(),
                (*amount).to_string(),
            ]);
        }
        rows.push(vec![
            "Total Distribution Amount".into(),
            String::new(),
            String::new(),
            String::new(),
            format!("{total:.2}"),
        ]);
        rows
    }

    #[test]
    fn a_district_paid_from_three_counties_is_one_row_that_says_three() {
        let sheet = casino_county_sheet(
            "Jul 1, 2023 - Dec 31, 2023 County Student Distribution",
            &[
                ("01", "OHIO CONNECTIONS ACADEMY", "000236", "449.12"),
                ("02", "OHIO CONNECTIONS ACADEMY", "000236", "550.88"),
                ("03", "OHIO CONNECTIONS ACADEMY", "000236", "100.00"),
                ("01", "MANCHESTER LOCAL", "000442", "21076.75"),
            ],
        );
        let panel = build_casino_extract(&[CasinoSheet {
            distribution: "2024-01",
            label: "test",
            rows: &sheet,
        }])
        .expect("checks pass");
        assert_eq!(
            panel,
            vec![
                vec!["000236", "OHIO CONNECTIONS ACADEMY", "2024-01", "3", "1100"],
                vec!["000442", "MANCHESTER LOCAL", "2024-01", "1", "21076.75"],
            ]
        );
    }

    #[test]
    fn the_foot_total_is_what_catches_a_column_read_from_the_wrong_offset() {
        // The failure this stands against does not look like a failure. Sum the amount column
        // without excluding the row the department puts its own total on, and a half-year comes
        // out at exactly twice its size — which for this channel is $90m, a number with nothing
        // obviously wrong about it.
        let whole = casino_county_sheet(
            "Jan 1, 2023 - Jun 30, 2023 County Student Distribution",
            &[
                ("01", "MANCHESTER LOCAL", "000442", "21076.75"),
                ("02", "LOST TO THE PAGE BREAK", "000443", "500.00"),
            ],
        );
        // A page break drops a district. Nothing about the remaining rows is malformed and the
        // foot total does not move, so only the comparison between them notices.
        let torn: Vec<Vec<String>> = whole
            .iter()
            .filter(|row| crate::conventions::cell(row, 2) != "000443")
            .cloned()
            .collect();
        let failure = build_casino_extract(&[CasinoSheet {
            distribution: "2023-08",
            label: "the sheet that lost a row",
            rows: &torn,
        }])
        .expect_err("the sheet's own total disagrees");
        assert!(failure.contains("the sheet that lost a row"), "{failure}");
        assert!(failure.contains("21576.75"), "{failure}");
    }

    #[test]
    fn a_banner_that_disagrees_with_the_payment_month_is_a_failure_not_a_note() {
        // R.C. 5753.03(D)(2) pays in August for January-June. A sheet filed under the wrong month
        // parses perfectly and puts a half-year in the wrong fiscal year.
        let sheet = casino_county_sheet(
            "Jan 1, 2023 - Jun 30, 2023 County Student Distribution",
            &[("01", "MANCHESTER LOCAL", "000442", "21076.75")],
        );
        let failure = build_casino_extract(&[CasinoSheet {
            distribution: "2024-01",
            label: "misfiled",
            rows: &sheet,
        }])
        .expect_err("the banner and the month disagree");
        assert!(failure.contains("2023-01-01..2023-06-30"), "{failure}");
        assert!(failure.contains("2023-07-01"), "{failure}");
    }

    #[test]
    fn two_layouts_of_one_distribution_have_to_agree_district_by_district() {
        // August 2015 is published twice, by county and statewide. The check is worth having
        // because the two are generated by different reports: they agree today to the cent across
        // 1,044 districts, and a disagreement would mean one of the two was never what it said.
        let by_county = casino_county_sheet(
            "Jan 1, 2015 - Jun 30, 2015 County Student Distribution",
            &[
                ("01", "MANCHESTER LOCAL", "000442", "2000.00"),
                ("02", "MANCHESTER LOCAL", "000442", "1000.00"),
            ],
        );
        let statewide = vec![
            vec![
                String::new(),
                String::new(),
                String::new(),
                "Ohio Department of Taxation".into(),
            ],
            vec![
                String::new(),
                String::new(),
                String::new(),
                "August 2015".into(),
            ],
            vec![
                "IRN".into(),
                "School District Name".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "Distribution Amount".into(),
            ],
            vec![
                "000442".into(),
                "MANCHESTER LOCAL".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "3500.00".into(),
            ],
            vec![
                "Total Distribution Amount:".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "3500.00".into(),
            ],
        ];
        let failure = build_casino_extract(&[
            CasinoSheet {
                distribution: "2015-08",
                label: "by county",
                rows: &by_county,
            },
            CasinoSheet {
                distribution: "2015-08",
                label: "statewide",
                rows: &statewide,
            },
        ])
        .expect_err("3500 against 3000");
        assert!(failure.contains("000442"), "{failure}");
    }

    #[test]
    fn the_true_up_years_amount_is_the_last_column_and_not_the_first() {
        // August 2018 carries three amount columns: a January 2018 recalculation of $7,475.78, the
        // August 2018 calculation of $48.0m, and their total. The first sits in column E, which is
        // where every other year's amount is — so a fixed offset reports the half-year at seven
        // thousand dollars and nothing else notices.
        let rows = vec![
            vec![
                String::new(),
                String::new(),
                "Ohio Department of Taxation".into(),
            ],
            vec![
                String::new(),
                String::new(),
                "Jan 1, 2018 - Jun 30, 2018 County Student Distribution".into(),
            ],
            vec![
                "County Code".into(),
                "County Name".into(),
                "IRN".into(),
                "SD Name".into(),
                "January 2018 Calculated Distribution (A)".into(),
                "August 2018 Calculated Distribution (B)".into(),
                "Total Distribution".into(),
            ],
            vec![
                "01".into(),
                "ADAMS".into(),
                "000442".into(),
                "MANCHESTER LOCAL".into(),
                "3.21".into(),
                "20000.00".into(),
                "20003.21".into(),
            ],
            vec![
                "Total Distribution Amount".into(),
                String::new(),
                String::new(),
                String::new(),
                "3.21".into(),
                "20000.00".into(),
                "20003.21".into(),
            ],
        ];
        let panel = build_casino_extract(&[CasinoSheet {
            distribution: "2018-08",
            label: "the true-up",
            rows: &rows,
        }])
        .expect("checks pass");
        assert_eq!(panel[0][4], "20003.21");
    }

    #[test]
    fn a_zero_row_is_dropped_because_only_one_of_the_two_layouts_publishes_them() {
        // The statewide sheets list districts certified at no students — 22 of the 1,066 in August
        // 2015. The county sheets omit them. Keeping them would make a district's presence in the
        // panel a fact about which report the department ran that year.
        let sheet = casino_county_sheet(
            "Jul 1, 2023 - Dec 31, 2023 County Student Distribution",
            &[
                ("01", "MANCHESTER LOCAL", "000442", "21076.75"),
                ("01", "CLOSED ACADEMY", "000909", "0.00"),
            ],
        );
        let panel = build_casino_extract(&[CasinoSheet {
            distribution: "2024-01",
            label: "test",
            rows: &sheet,
        }])
        .expect("checks pass");
        assert_eq!(panel.len(), 1);
        assert_eq!(panel[0][0], "000442");
    }

    #[test]
    fn the_statutory_period_is_the_half_year_before_the_payment() {
        assert_eq!(
            casino_statutory_period("2024-01"),
            Some(("2023-07-01".into(), "2023-12-31".into()))
        );
        assert_eq!(
            casino_statutory_period("2020-08"),
            Some(("2020-01-01".into(), "2020-06-30".into()))
        );
        // Money moves in January and August and no other month; anything else is a filing error.
        assert_eq!(casino_statutory_period("2024-04"), None);
    }

    #[test]
    fn a_banner_is_read_and_a_month_only_headline_is_not_mistaken_for_one() {
        let dated = vec![vec![
            "Jan 1, 2015 - Jun 30, 2015 County Student Distribution".to_string(),
        ]];
        assert_eq!(
            casino_printed_period(&dated),
            Some(("2015-01-01".into(), "2015-06-30".into()))
        );
        // The four sheets of the combined workbook name the month the money moved and not the
        // half-year it was earned in. Guessing one from the other is what the statutory rule is
        // for, and it is not this function's job.
        let undated = vec![
            vec!["School District Distribution".to_string()],
            vec!["August 2015".to_string()],
        ];
        assert_eq!(casino_printed_period(&undated), None);
    }

    /// Ohio's two courts date an opinion differently, and one of the two has no closing bracket.
    ///
    /// The supreme court prints the date inside a parenthetical that ends `.)`. The courts of
    /// appeals print `Rendered on <date>` alone on a line. A reader that looks for `.)` first and
    /// only falls back to the line ending is right about the first and finds the *next* closing
    /// parenthesis anywhere in the document for the second — sixty-eight lines of holding, in the
    /// first appellate opinion this repository wired.
    #[test]
    fn a_date_ends_at_whichever_comes_first_of_the_bracket_and_the_line() {
        let supreme = "(No. 95-2066--Submitted September 10, 1996--Decided March 24, 1997.)\n\
                       Constitutional law—Education—Schools (and so on.)";
        assert_eq!(decided_on(supreme), "March 24, 1997");

        let appellate = "Rendered on March 29, 2024\n\nBOGGS, J.\n\
                         {¶ 1} On February 2, 2024, appellees filed the instant motion.\n\
                         The order is not final. (See R.C. 2505.02(B)(4).)";
        assert_eq!(decided_on(appellate), "March 29, 2024");

        // Neither marker present is an empty field rather than a guess.
        assert_eq!(decided_on("IN THE COURT OF APPEALS OF OHIO"), "");
    }

    fn row(width: usize, values: &[(usize, &str)]) -> Vec<String> {
        let mut row = vec![String::new(); width];
        for (index, value) in values {
            row[*index] = (*value).to_string();
        }
        row
    }

    /// Three banner rows, a header, then two districts — the department's layout.
    fn base_cost_rows() -> Vec<Vec<String>> {
        let district = |irn: &str, name: &str, buildings: &str| {
            row(
                59,
                &[
                    (0, irn),
                    (1, name),
                    (2, "Somewhere"),
                    (3, buildings),
                    (4, "2200"),
                    (6, "2100"),
                    (7, "2150.5"),
                    (8, "150"),
                    (9, "450"),
                    (10, "760"),
                    (11, "700"),
                    (12, "90"),
                    (13, "790"),
                    (14, "84.21"),
                    (17, "14.34"),
                    (22, "9500000.55"),
                    (57, "17400000.25"),
                    (58, "8091.16"),
                ],
            )
        };
        vec![
            row(1, &[(0, "Ohio Department of Education & Workforce")]),
            row(1, &[(0, "")]),
            row(1, &[(0, "Base Cost")]),
            row(4, &[(0, "District IRN"), (1, "District Names")]),
            district("043786", "Cleveland", "10"),
            district("049056", "Northern", "3"),
        ]
    }

    fn summary_rows() -> Vec<Vec<String>> {
        vec![
            row(4, &[(0, "District IRN"), (1, "District Names")]),
            row(
                12,
                &[
                    (0, "043786"),
                    (1, "Cleveland"),
                    (3, "4000000"),
                    (10, "10000000"),
                    (11, "0"),
                ],
            ),
            row(
                12,
                &[
                    (0, "049056"),
                    (1, "Northern"),
                    (3, "3000000"),
                    (10, "7000000"),
                    (11, "2500000"),
                ],
            ),
            // The aggregate the department ships in this sheet. Numeric IRN; must be excluded.
            row(
                12,
                &[
                    (0, "999999"),
                    (1, STATEWIDE_ROW),
                    (10, "999999999"),
                    (11, "888888888"),
                ],
            ),
        ]
    }

    /// The `ADM Data` sheet: a header, then one row per district. Only the columns the builder
    /// reads are populated.
    fn adm_rows() -> Vec<Vec<String>> {
        let district = |irn: &str, buildings: &str, fy24: &str, fy25: &str, fy26: &str| {
            row(
                29,
                &[
                    (0, irn),
                    (25, buildings),
                    (26, fy24),
                    (27, fy25),
                    (28, fy26),
                ],
            )
        };
        vec![
            row(2, &[(0, "District IRN"), (1, "District")]),
            district("043786", "10", "2200", "2150", "2100"),
            district("049056", "3", "2200", "2150", "2100"),
        ]
    }

    fn profile_rows() -> Vec<Vec<String>> {
        let district = |irn: &str, name: &str, valuation: &str| {
            row(
                50,
                &[
                    (0, name),
                    (1, irn),
                    (4, "2100"),
                    (11, "0.42"),
                    (21, valuation),
                    (33, "31.35"),
                    (34, "20"),
                    (46, "11986.62"),
                    (47, "6423.28"),
                    (49, "5277.43"),
                ],
            )
        };
        vec![
            row(2, &[(0, "District"), (1, "IRN")]),
            district("043786", "Cleveland Municipal", "184903.95"),
            district("049056", "Northern Local", "279983.24"),
        ]
    }

    fn model() -> Vec<Vec<String>> {
        build_fy27_model(&Fy27Sheets {
            base_cost_rows: &base_cost_rows(),
            summary_rows: &summary_rows(),
            adm_rows: &adm_rows(),
            profile_rows: &profile_rows(),
            ..Default::default()
        })
    }

    fn field<'a>(rows: &'a [Vec<String>], irn: &str, column: &str) -> &'a str {
        let index = FY27_HEADER.iter().position(|c| *c == column).unwrap();
        &rows.iter().find(|r| r[0] == irn).unwrap()[index]
    }

    #[test]
    fn emits_one_row_per_district_in_irn_order() {
        let rows = model();
        assert_eq!(
            rows.iter().map(|r| r[0].as_str()).collect::<Vec<_>>(),
            ["043786", "049056"]
        );
    }

    #[test]
    fn excludes_the_statewide_aggregate_row() {
        // Its IRN is numeric, so the digit filter keeps it; only the name check drops it.
        assert!(!model().iter().any(|r| r[0] == "999999"));
    }

    #[test]
    fn row_width_matches_the_declared_header() {
        for row in model() {
            assert_eq!(row.len(), FY27_HEADER.len());
        }
    }

    #[test]
    fn the_adm_history_is_named_for_the_years_the_adm_data_sheet_declares() {
        // Base_Cost labels the same three columns FY22/FY23/FY24. It is stale, and a fixture
        // built from those labels names every enrollment trend for the wrong pair of years.
        let rows = model();
        assert_eq!(field(&rows, "043786", "enrolled_adm_fy24"), "2200");
        assert_eq!(field(&rows, "043786", "enrolled_adm_fy25"), "2150");
        assert_eq!(field(&rows, "043786", "enrolled_adm_fy26"), "2100");
    }

    #[test]
    fn the_base_cost_state_share_is_kept_apart_from_total_formula_aid() {
        // Summary_SFPR column 3 is base cost alone; column 10 adds targeted assistance,
        // special education, DPIA, English learner, gifted, and CTE. A state-share lever acts
        // on the first and would be wrong applied to the second.
        let rows = model();
        assert_eq!(field(&rows, "049056", "base_cost_state_share"), "3000000");
        assert_eq!(field(&rows, "049056", "core_foundation_funding"), "7000000");
    }

    #[test]
    fn a_building_count_that_is_a_multiple_of_ten_keeps_its_zero() {
        // Trimming trailing zeros off an integer turned 10 buildings into 1. Nine districts
        // carried a wrong count in the committed fixture before this was caught.
        assert_eq!(field(&model(), "043786", "school_buildings"), "10");
        assert_eq!(field(&model(), "049056", "school_buildings"), "3");
    }

    #[test]
    fn joins_valuation_from_the_profile_report() {
        assert_eq!(
            field(&model(), "049056", "assessed_valuation_per_pupil_fy23"),
            "279983.24"
        );
    }

    #[test]
    fn carries_the_guarantee_and_core_funding() {
        let rows = model();
        assert_eq!(
            field(&rows, "049056", "temp_transitional_aid_guarantee"),
            "2500000"
        );
        assert_eq!(field(&rows, "049056", "core_foundation_funding"), "7000000");
    }

    #[test]
    fn a_district_with_no_guarantee_records_zero_not_blank() {
        // Blank would read as "unknown"; this district is known to have none.
        assert_eq!(
            field(&model(), "043786", "temp_transitional_aid_guarantee"),
            "0"
        );
    }

    #[test]
    fn skips_a_district_missing_from_the_summary_sheet() {
        let rows = build_fy27_model(&Fy27Sheets {
            base_cost_rows: &base_cost_rows(),
            summary_rows: &summary_rows()[..1],
            adm_rows: &adm_rows(),
            profile_rows: &profile_rows(),
            ..Default::default()
        });
        assert!(rows.is_empty());
    }

    #[test]
    fn skips_a_district_with_no_usable_adm() {
        let mut base = base_cost_rows();
        base[5][7] = "0".into();
        let rows = build_fy27_model(&Fy27Sheets {
            base_cost_rows: &base,
            summary_rows: &summary_rows(),
            adm_rows: &adm_rows(),
            profile_rows: &profile_rows(),
            ..Default::default()
        });
        assert_eq!(
            rows.iter().map(|r| r[0].as_str()).collect::<Vec<_>>(),
            ["043786"]
        );
    }

    #[test]
    fn commas_are_stripped_from_district_names() {
        let mut base = base_cost_rows();
        base[5][1] = "Northern Local, Perry".into();
        let rows = build_fy27_model(&Fy27Sheets {
            base_cost_rows: &base,
            summary_rows: &summary_rows(),
            adm_rows: &adm_rows(),
            profile_rows: &profile_rows(),
            ..Default::default()
        });
        assert!(!rows[1][1].contains(','));
        assert_eq!(rows[1][1], "Northern Local  Perry");
    }

    #[test]
    fn trailing_whitespace_is_stripped_from_district_names() {
        let mut base = base_cost_rows();
        base[5][1] = "Bellefontaine City ".into();
        let rows = build_fy27_model(&Fy27Sheets {
            base_cost_rows: &base,
            summary_rows: &summary_rows(),
            adm_rows: &adm_rows(),
            profile_rows: &profile_rows(),
            ..Default::default()
        });
        assert_eq!(rows[1][1], "Bellefontaine City");
    }

    #[test]
    fn the_profile_extract_reduces_to_the_declared_columns() {
        let rows = build_profile_extract(&profile_rows());
        assert_eq!(rows.len(), 2);
        for row in &rows {
            assert_eq!(row.len(), PROFILE_HEADER.len());
        }
        assert_eq!(rows[0][0], "043786");
        assert_eq!(rows[0][1], "Cleveland Municipal");
    }

    #[test]
    fn missing_profile_values_are_blank_not_zero() {
        let mut rows = profile_rows();
        rows[1][21] = "#N/A".into();
        let out = build_profile_extract(&rows);
        let index = PROFILE_HEADER
            .iter()
            .position(|c| *c == "assessed_valuation_per_pupil_fy23")
            .unwrap();
        assert_eq!(out[0][index], "");
    }

    #[test]
    fn formats_trim_only_past_a_decimal_point() {
        assert_eq!(format_value(Some(847.823), 4), "847.823");
        assert_eq!(format_value(Some(20.0), 4), "20");
        assert_eq!(format_value(Some(10.0), 0), "10");
        assert_eq!(format_value(Some(100.0), 0), "100");
        assert_eq!(format_value(None, 2), "");
    }

    #[test]
    fn the_cpi_extract_keeps_the_header_and_one_series() {
        let text = "series_id\tyear\tperiod\tvalue\n\
             CUUR0000SA0     \t2000\tM06\t       172.400\t\n\
             CUUR0000SA0     \t2000\tM05\t       171.300\t\n\
             CUUR0000SAF1    \t2000\tM06\t       167.900\t\n";
        let extract = build_cpi_extract(text, "CUUR0000SA0", "M06");
        let lines: Vec<&str> = extract.lines().collect();
        assert_eq!(lines.len(), 2, "header plus one matching observation");
        assert!(lines[0].starts_with("series_id"));
        assert!(lines[1].contains("172.400"));
    }

    #[test]
    fn writes_lf_endings_and_returns_a_row_count() {
        let dir = std::env::temp_dir().join(format!("edfund-fixture-{}", std::process::id()));
        let path = dir.join("nested/out.csv");
        let written = write_csv(
            &path,
            &["a", "b"],
            &[vec!["1".into(), "2".into()], vec!["3".into(), "4".into()]],
        )
        .unwrap();
        assert_eq!(written, 2);
        let raw = fs::read(&path).unwrap();
        assert!(
            !raw.windows(2).any(|w| w == b"\r\n"),
            "CRLF would churn the diff on every rebuild"
        );
        assert!(String::from_utf8_lossy(&raw).starts_with("a,b\n1,2\n"));
        let _ = fs::remove_dir_all(&dir);
    }
}

/// Where the Census F-33 state comparison is written, relative to the root.
pub const F33_FIXTURE: &str = "crates/dispersion/fixtures/census-f33-states.csv";

/// Header of the state comparison: one row per state, plus the District of Columbia.
///
/// Fifty-one rows out of fourteen thousand. The survey covers every school system in the United
/// States and the corpus needs exactly one thing from it that nothing else can supply — where
/// Ohio sits among the states — so the fixture is the aggregate rather than the panel.
pub const F33_HEADER: &[&str] = &[
    "fips",
    "state",
    "systems",
    "enrollment",
    "total_revenue",
    "federal_revenue",
    "state_revenue",
    "local_revenue",
    "property_tax_revenue",
    "parent_government_revenue",
    "current_spending",
];

/// FIPS code to state name. The survey carries codes only.
const STATE_NAMES: &[(&str, &str)] = &[
    ("01", "Alabama"),
    ("02", "Alaska"),
    ("04", "Arizona"),
    ("05", "Arkansas"),
    ("06", "California"),
    ("08", "Colorado"),
    ("09", "Connecticut"),
    ("10", "Delaware"),
    ("11", "District of Columbia"),
    ("12", "Florida"),
    ("13", "Georgia"),
    ("15", "Hawaii"),
    ("16", "Idaho"),
    ("17", "Illinois"),
    ("18", "Indiana"),
    ("19", "Iowa"),
    ("20", "Kansas"),
    ("21", "Kentucky"),
    ("22", "Louisiana"),
    ("23", "Maine"),
    ("24", "Maryland"),
    ("25", "Massachusetts"),
    ("26", "Michigan"),
    ("27", "Minnesota"),
    ("28", "Mississippi"),
    ("29", "Missouri"),
    ("30", "Montana"),
    ("31", "Nebraska"),
    ("32", "Nevada"),
    ("33", "New Hampshire"),
    ("34", "New Jersey"),
    ("35", "New Mexico"),
    ("36", "New York"),
    ("37", "North Carolina"),
    ("38", "North Dakota"),
    ("39", "Ohio"),
    ("40", "Oklahoma"),
    ("41", "Oregon"),
    ("42", "Pennsylvania"),
    ("44", "Rhode Island"),
    ("45", "South Carolina"),
    ("46", "South Dakota"),
    ("47", "Tennessee"),
    ("48", "Texas"),
    ("49", "Utah"),
    ("50", "Vermont"),
    ("51", "Virginia"),
    ("53", "Washington"),
    ("54", "West Virginia"),
    ("55", "Wisconsin"),
    ("56", "Wyoming"),
];

/// Aggregate the Census F-33 district panel to one row per state.
///
/// # Which systems are counted, and why the rule is enrollment rather than school level
///
/// The survey covers 14,106 school systems at six school levels: elementary-only, secondary-only,
/// unified, vocational and special, nonoperating, and education service agencies. States organise
/// differently — Ohio has 609 unified districts and no elementary-only ones, Illinois has
/// hundreds of both — so any rule stated in terms of school level would count different things in
/// different states, which is precisely what an interstate comparison must not do.
///
/// The rule here is **enrolment above zero**. It admits every system that teaches somebody and
/// excludes the two that would double count: 691 education service agencies, which report revenue
/// received *from* the districts they serve, and 121 nonoperating systems, which levy tax and pay
/// tuition elsewhere. Ohio's own ESCs are in the first group, and the corpus already knows that
/// channel — it is the `total_transfers` line that had to be ruled out as the voucher deduction.
///
/// # Property tax cannot be ranked across states, and local revenue can
///
/// Nine states — Alaska, Connecticut, the District of Columbia, Maryland, Massachusetts, North
/// Carolina, Rhode Island, Tennessee and Virginia — report **zero** school property tax revenue
/// while raising billions locally. They levy plenty; their school districts are dependent
/// agencies of a city or county, so the tax belongs to the parent government and reaches the
/// district as an appropriation. Virginia's parent contributions are 94% of its local school
/// revenue, the District of Columbia's 99%.
///
/// So `property_tax_revenue / total_revenue` is not a national ranking: it silently compares
/// states that report their own levy against states structurally unable to. Massachusetts funds
/// schools from property tax about as heavily as anywhere and scores zero.
///
/// `local_revenue` is the aggregate that survives the difference, because parent contributions
/// are inside it. Rank on that. `parent_government_revenue` travels beside the property tax
/// column so a consumer can see which structure a state has rather than having to know.
///
/// # Dollars are thousands, and enrolment is not
///
/// Every money column in the F-33 is reported in thousands of dollars. The fixture keeps the
/// survey's own unit rather than converting, so a reader comparing it against the published
/// tables is comparing like with like; consumers multiply. Enrolment is a headcount.
///
/// # Read by header, because the layout moved once already
///
/// This builder used a fixed column map, which was accurate for FY2022 and silently wrong for
/// FY2024: the Bureau dropped the `IDCENSUS` column, shifting every index after it by one. Every
/// name the builder wants still exists and still means the same thing, so positional indices
/// would have produced a complete, plausible, entirely wrong fixture — state codes read as unit
/// types, revenue read as enrolment. Names are looked up per file instead.
///
/// Note that these are the Bureau's `elsec` tables, whose names differ from the NCES `sdf`
/// school-district files the Ohio panel reads: property tax is `LOCRPROP` here and `T06` there.
///
/// # Errors
///
/// Returns a message naming the missing column if the layout moves again.
pub fn build_f33_states(rows: &[Vec<String>], label: &str) -> Result<Vec<Vec<String>>, String> {
    let head = rows.first().cloned().unwrap_or_default();
    let at = |name: &str| column(&head, name, label);
    let fips = at("FIPST")?;
    let school_level = at("SCHLEV")?;
    let enrolment = at("ENROLL")?;
    let spending = at("TCURSPND")?;
    let columns = [
        at("TOTALREV")?,
        at("TFEDREV")?,
        at("TSTREV")?,
        at("TLOCREV")?,
        at("LOCRPROP")?,
        at("LOCRPAR")?,
    ];

    let number = |row: &[String], index: usize| -> f64 {
        row.get(index)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .unwrap_or(0.0)
    };

    let mut totals: std::collections::BTreeMap<String, [f64; 8]> =
        std::collections::BTreeMap::new();
    let mut systems: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();

    for row in rows.iter().skip(1) {
        let enrolled = number(row, enrolment);
        if enrolled <= 0.0 {
            continue;
        }
        let Some(code) = row.get(fips).map(|s| s.trim().to_string()) else {
            continue;
        };
        // A school level the survey does not use would mean the layout has moved under us.
        debug_assert!(matches!(
            row.get(school_level).map(String::as_str),
            Some("01" | "02" | "03" | "05" | "06" | "07")
        ));

        *systems.entry(code.clone()).or_default() += 1;
        let entry = totals.entry(code).or_insert([0.0; 8]);
        entry[0] += enrolled;
        for (slot, index) in columns.iter().enumerate() {
            entry[slot + 1] += number(row, *index);
        }
        entry[7] += number(row, spending);
    }

    Ok(totals
        .into_iter()
        .filter_map(|(fips, totals)| {
            // A FIPS code with no name is a territory. The survey carries Puerto Rico and the
            // outlying areas in some years and they are not states, so they are dropped rather
            // than ranked against them.
            let name = STATE_NAMES.iter().find(|(code, _)| *code == fips)?.1;
            let count = systems.get(&fips).copied().unwrap_or(0);
            let mut row = vec![fips, name.to_string(), count.to_string()];
            row.extend(totals.iter().map(|value| format_value(Some(*value), 0)));
            Some(row)
        })
        .collect())
}

/// Where the FY2024 state cross-section is written, relative to the repository root.
///
/// A second fixture rather than a year column on the first, because the FY2022 file is what the
/// corpus's published interstate figures were computed from and a test pins them. Two years side
/// by side is also the only way to see that the Bureau's own per-pupil figures are not
/// reconstructible from these files — see the WP015 catalog entry.
pub const F33_FY2024_FIXTURE: &str = "crates/dispersion/fixtures/census-f33-states-fy2024.csv";

/// Where the per-district F-33 extract is written, relative to the repository root.
pub const F33_DISTRICTS_FIXTURE: &str = "crates/dispersion/fixtures/f33-districts-fy2022.csv";

/// Columns of the per-district F-33 fixture.
pub const F33_DISTRICTS_HEADER: &[&str] = &[
    "leaid",
    "irn",
    "state",
    "comparable",
    "enrollment",
    "total_revenue",
    "federal_revenue",
    "state_revenue",
    "local_revenue",
    "property_tax",
    "current_spending",
];

/// Split one line of a delimited file, honouring double-quoted fields.
///
/// The CCD directory quotes any agency name containing a comma, and `LEA_NAME` sits at column 4
/// — ahead of the two columns this reads. Splitting on the delimiter alone gives the right answer
/// for the 2022-23 file, because no quoted comma happens to fall before column 8; that is luck
/// rather than a property of the format, and the kind that fails silently the year an agency is
/// renamed to "Dayton, City of".
fn delimited_fields(line: &str, delimiter: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            // An escaped quote inside a quoted field is written as two quotes.
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            c if c == delimiter && !quoted => out.push(core::mem::take(&mut field)),
            c => field.push(c),
        }
    }
    out.push(field);
    out
}

/// Find a column by name, so a layout change is an error rather than a silent misread.
///
/// The survey's columns are located by header rather than by position, unlike
/// [`build_f33_states`]. Both files are published once a year and neither promises a stable
/// layout — `census-f33`'s own note records that the column map is per-era — and a header lookup
/// turns next year's reshuffle into a named failure instead of a fixture full of the wrong
/// numbers.
fn column(header: &[String], name: &str, file: &str) -> Result<usize, String> {
    header
        .iter()
        .position(|h| h.trim() == name)
        .ok_or_else(|| format!("{file} has no {name} column; its layout has moved"))
}

/// Agency identifier and school year to Ohio IRN, from the rows [`build_ccd_directory`] wrote.
///
/// Keyed on the year as well as the agency, which is the whole reason the directory is held for
/// thirty years rather than one. Asking the 2022-23 file for a district that closed in 2015 gets
/// nothing back, and the panel then carried an empty identifier for 124 agencies in FY2012 and
/// described the count as a consolidation history. See `dispersion::lea_directory`.
fn directory_irn_map(rows: &[Vec<String>]) -> BTreeMap<(u16, String), String> {
    rows.iter()
        .filter_map(|row| {
            let opens = row.first()?.parse().ok()?;
            Some(((opens, row.get(1)?.clone()), row.get(2)?.clone()))
        })
        .collect()
}

/// `LEAID` to Ohio IRN, from the CCD LEA directory.
///
/// Shared by the national cross-section and the Ohio panel so the join cannot be written twice
/// and drift — the failure the F-33 district panel and the legislative crosswalk both had, where
/// a derivation existed only as prose in two places that could not check each other.
fn ccd_irn_map(directory: &str) -> Result<BTreeMap<String, String>, String> {
    const DIRECTORY: &str = "the CCD LEA directory";
    let mut rows = directory.lines();
    let head = delimited_fields(rows.next().unwrap_or_default(), ',');
    let (key, st_leaid) = (
        column(&head, "LEAID", DIRECTORY)?,
        column(&head, "ST_LEAID", DIRECTORY)?,
    );
    let mut irn_of = BTreeMap::new();
    for line in rows {
        if line.trim().is_empty() {
            continue;
        }
        let f = delimited_fields(line, ',');
        if let (Some(leaid), Some(irn)) = (f.get(key), f.get(st_leaid)) {
            irn_of.insert(leaid.trim().to_string(), irn.trim().to_string());
        }
    }
    Ok(irn_of)
}

/// The per-district F-33 panel: one row per agency the national comparison can use.
///
/// # Why two files
///
/// The Bureau keys the F-33 on `IDCENSUS` and Ohio keys everything on IRN, and the corpus
/// recorded that join as unavailable for as long as `census-f33` has been wired. It was not:
/// NCES publishes the same survey keyed on `LEAID`, and the CCD directory carries `ST_LEAID`,
/// which for Ohio is `OH-` followed by the IRN. `survey` is NCES's `sdf22_1a.txt` and
/// `directory` is the CCD LEA file; neither alone is enough.
///
/// # What is kept
///
/// **Comparable** is the survey's own distinction: `AGCHRT != 1` — no associated charter schools
/// — and `SCHLEV == 03`, a unified elementary-and-secondary agency. Every Ohio agency is kept
/// whether or not it is comparable, flagged in the `comparable` column, because the corpus needs
/// figures for all 968 of them and a national position for only the 611 that have one. Leaving
/// charters in the distribution put Ohio's 200 smallest agencies at an 8% local share, which is a
/// fact about charter finance and not about school districts — see
/// [`dispersion::national_peers`](../../dispersion/src/national_peers.rs).
///
/// A row also needs **enrolment and total revenue above zero**. The survey reports `-1` and `-2`
/// for missing and not-applicable, and 190 agencies carry those in every money column; admitting
/// them would put a district with no reported revenue at a 0% local share, at the bottom of a
/// distribution it is simply absent from. `T06` and `TCURELSC` are blanked rather than dropped
/// when negative, because a district that reports no property tax is usually fiscally dependent
/// rather than untaxed, which is a distinction the consumer draws.
///
/// Money is in thousands of dollars, as the survey publishes it and as
/// [`build_f33_states`] keeps it.
///
/// # Errors
///
/// Returns the missing column's name if either file's layout has moved.
pub fn build_f33_districts(survey: &str, directory: &str) -> Result<Vec<Vec<String>>, String> {
    const SURVEY: &str = "the F-33 district survey";
    let irn_of = ccd_irn_map(directory)?;

    let mut rows = survey.lines();
    let head = delimited_fields(rows.next().unwrap_or_default(), '\t');
    let at = |name: &str| column(&head, name, SURVEY);
    let (leaid, state, charter, level) =
        (at("LEAID")?, at("STABBR")?, at("AGCHRT")?, at("SCHLEV")?);
    let enrolment = at("V33")?;
    let revenue = [
        at("TOTALREV")?,
        at("TFEDREV")?,
        at("TSTREV")?,
        at("TLOCREV")?,
    ];
    let (property_tax, spending) = (at("T06")?, at("TCURELSC")?);

    let mut out = Vec::new();
    for line in rows {
        if line.trim().is_empty() {
            continue;
        }
        let f = delimited_fields(line, '\t');
        let field = |i: usize| f.get(i).map(|v| v.trim()).unwrap_or_default();
        let number = |i: usize| field(i).parse::<i64>().ok();

        let state = field(state).to_string();
        let comparable = field(charter) != "1" && field(level) == "03";
        if !comparable && state != "OH" {
            continue;
        }
        // Missing data is coded negative, not blank, so an unreported agency would otherwise
        // enter the distribution as a real zero.
        if number(enrolment).unwrap_or(-1) <= 0 || number(revenue[0]).unwrap_or(-1) <= 0 {
            continue;
        }

        let key = field(leaid).to_string();
        let irn = if state == "OH" {
            irn_of
                .get(&key)
                .map(|v| v.strip_prefix("OH-").unwrap_or(v).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        // Reported as-is; blanked only where the survey's negative codes mean "not reported".
        let plain = |i: usize| number(i).map(|v| v.to_string()).unwrap_or_default();
        let unreported_is_blank = |i: usize| match number(i) {
            Some(v) if v >= 0 => v.to_string(),
            _ => String::new(),
        };

        let mut row = vec![
            key,
            irn,
            state,
            if comparable { "1" } else { "0" }.to_string(),
            plain(enrolment),
        ];
        row.extend(revenue.iter().map(|i| plain(*i)));
        row.push(unreported_is_blank(property_tax));
        row.push(unreported_is_blank(spending));
        out.push(row);
    }
    Ok(out)
}

/// Where the multi-year Ohio F-33 panel is written, relative to the repository root.
pub const F33_OHIO_PANEL_FIXTURE: &str = "crates/dispersion/fixtures/f33-ohio-panel.csv";

/// Columns of the Ohio panel. `fiscal_year` leads because the file's whole purpose is the series.
pub const F33_OHIO_PANEL_HEADER: &[&str] = &[
    "fiscal_year",
    "leaid",
    "irn",
    "comparable",
    "enrollment",
    "total_revenue",
    "federal_revenue",
    "state_revenue",
    "local_revenue",
    "property_tax",
    "current_spending",
];

/// One year of the survey, paired with the fiscal year it reports.
#[derive(Debug, Clone, Copy)]
pub struct PanelYear<'a> {
    /// The fiscal year the file reports, which is not derivable from the file itself.
    pub fiscal_year: u16,
    /// The survey member's text.
    pub survey: &'a str,
}

/// Ohio across every year of the survey this repository holds.
///
/// # Why a second F-33 fixture rather than more rows in the first
///
/// The two answer different questions and need different populations. The national cross-section
/// exists to place one Ohio district among America's — 16,872 agencies, one year — and is the
/// only way to say whether Ohio is unusual. This panel exists to say how Ohio *changed*, which
/// needs many years and only Ohio. Putting ten years of the national file in one fixture would
/// commit roughly 7 MB to answer a question that needs 968 rows a year.
///
/// # The layout genuinely is per-era, and only the column names survive
///
/// The three eras this reads across carry **256, 260 and 354 columns**, the archive member is
/// `sdf121a.txt` in one year and `Sdf16_1a.txt` in another, and the file is tab-delimited
/// throughout. Every column this needs is present in all of them under the same name, which is
/// the entire reason every column is resolved by header: a positional map written against FY2022 would
/// read the wrong field in FY2012 and report it as a number rather than as an error.
///
/// # What is kept, and the one difference from the cross-section
///
/// The same comparability flag and the same non-negative enrolment and revenue test, so a figure
/// here and a figure there mean the same thing. The difference is that **every Ohio agency is
/// kept whatever its flag** — as in the cross-section — but nothing outside Ohio is, and the year
/// is carried on the row rather than in the filename.
///
/// # The join is to one directory, and that is a real limitation
///
/// `LEAID` to IRN comes from the FY2022-23 CCD directory, because that is the directory this
/// repository holds. An agency that existed in FY2012 and had closed or merged by FY2023 is in
/// the survey and not in the directory, so it keeps its `LEAID` and gets an empty `irn`. That is
/// the consolidation problem `nces-ccd` was approved to solve and has not, recorded on the rows
/// it affects rather than in prose: count the blank IRNs per year and the panel tells you how
/// much of Ohio's agency population it cannot name.
///
/// # Errors
///
/// Returns the missing column's name if any year's layout has moved.
pub fn build_f33_ohio_panel(
    years: &[PanelYear<'_>],
    directory: &[Vec<String>],
) -> Result<Vec<Vec<String>>, String> {
    let irn_of = directory_irn_map(directory);
    let mut out = Vec::new();

    for year in years {
        let label = format!("the F-33 district survey for FY{}", year.fiscal_year);
        let mut rows = year.survey.lines();
        let head = delimited_fields(rows.next().unwrap_or_default(), '\t');
        let at = |name: &str| column(&head, name, &label);
        let (leaid, state, charter, level) =
            (at("LEAID")?, at("STABBR")?, at("AGCHRT")?, at("SCHLEV")?);
        let enrolment = at("V33")?;
        let revenue = [
            at("TOTALREV")?,
            at("TFEDREV")?,
            at("TSTREV")?,
            at("TLOCREV")?,
        ];
        let (property_tax, spending) = (at("T06")?, at("TCURELSC")?);

        let mut kept = 0usize;
        for line in rows {
            if line.trim().is_empty() {
                continue;
            }
            let f = delimited_fields(line, '\t');
            let field = |i: usize| f.get(i).map(|v| v.trim()).unwrap_or_default();
            let number = |i: usize| field(i).parse::<i64>().ok();

            if field(state) != "OH" {
                continue;
            }
            if number(enrolment).unwrap_or(-1) <= 0 || number(revenue[0]).unwrap_or(-1) <= 0 {
                continue;
            }

            let key = field(leaid).to_string();
            // The survey's fiscal year and the directory's school year name the same year: FY2012
            // finance sits beside the 2011-12 directory. Resolved against that year where it is
            // held and against the nearest later one otherwise, so an agency is named by a file
            // written while it existed rather than by one written a decade after it closed.
            let irn = irn_of
                .get(&(year.fiscal_year.saturating_sub(1), key.clone()))
                .or_else(|| {
                    irn_of
                        .range((year.fiscal_year.saturating_sub(1), key.clone())..)
                        .find(|((_, leaid), _)| *leaid == key)
                        .map(|(_, irn)| irn)
                })
                .cloned()
                .unwrap_or_default();
            let plain = |i: usize| number(i).map(|v| v.to_string()).unwrap_or_default();
            let unreported_is_blank = |i: usize| match number(i) {
                Some(v) if v >= 0 => v.to_string(),
                _ => String::new(),
            };

            let mut row = vec![
                year.fiscal_year.to_string(),
                key,
                irn,
                if field(charter) != "1" && field(level) == "03" {
                    "1"
                } else {
                    "0"
                }
                .to_string(),
                plain(enrolment),
            ];
            row.extend(revenue.iter().map(|i| plain(*i)));
            row.push(unreported_is_blank(property_tax));
            row.push(unreported_is_blank(spending));
            out.push(row);
            kept += 1;
        }

        // A year that contributes nothing is a layout that parsed but matched no Ohio row, which
        // reads as a shorter series rather than as a failure. Ohio has never had fewer than 900
        // reporting agencies in any year the survey covers.
        if kept < 500 {
            return Err(format!(
                "FY{} yielded {kept} Ohio agencies; the survey has never had fewer than 900, so \
                 the state filter or the file is wrong",
                year.fiscal_year
            ));
        }
    }
    Ok(out)
}

// -------------------------------------------------------------------------------------------
// Ohio Revised Code
// -------------------------------------------------------------------------------------------

/// Where the statute extract is written, relative to the repository root.
pub const STATUTE_FIXTURE: &str = "crates/project/fixtures/revised-code.txt";

/// One record of a committed document extract: a statute section, or a court opinion.
///
/// # Why one type rather than two
///
/// A legal claim in this corpus rests on either a statute or an opinion, and both should read
/// alike and parse alike. The fields are therefore named for what every document has rather than
/// for what a statute has: this type began as a statute-only record and was reused for the
/// DeRolph opinions with `section` holding `derolph-i`, `legislation` holding a PDF URL, and
/// `effective` forced empty — three fields whose documented meaning the values contradicted, in
/// a file whose purpose is to make citations checkable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// What the record is cited as: `3317.013`, or `derolph-i`.
    pub id: String,
    /// The document's own heading, or the case as it is cited.
    pub title: String,
    /// The date the document speaks from: a statute's effective date, an opinion's decision date.
    pub date: String,
    /// Where it came from: the act that produced a section, or the URL a PDF was retrieved from.
    pub source: String,
    /// The operative text, one paragraph per line.
    pub body: String,
}

/// Pull one section out of a fetched page.
///
/// # Why the body is bounded by two landmarks rather than by a selector
///
/// The page is a site around a statute: navigation above, a version picker and footer below. The
/// operative text begins after the authenticated-PDF link and ends at the version list. Both are
/// stable strings the page prints for every section, and keying on them survives a restyling in a
/// way that keying on a class name would not.
///
/// Returns `None` when neither landmark is found, which is the signal that the page is no longer
/// the page this was written against — better than committing a fixture full of navigation.
#[must_use]
pub fn parse_statute(section: &str, page: &str) -> Option<Record> {
    let text = crate::html::to_text(page);
    let after = |label: &str| -> String {
        text.lines()
            .skip_while(|l| l.trim() != label)
            .nth(1)
            .unwrap_or_default()
            .trim()
            .to_string()
    };

    // "Section 3317.013 | Special education multiples." — the heading, not the <title>.
    let heading = format!("Section {section} |");
    let title = text
        .lines()
        .find(|l| l.starts_with(&heading))
        .map(|l| l[heading.len()..].trim().to_string())
        .unwrap_or_default();

    const BODY_STARTS: &str = "Download Authenticated PDF";
    let start = text.find(BODY_STARTS)? + BODY_STARTS.len();
    let rest = &text[start..];
    // The version picker follows the operative text on every section that has one.
    let end = ["Available Versions of this Section", "Last updated"]
        .iter()
        .filter_map(|marker| rest.find(marker))
        .min()
        .unwrap_or(rest.len());

    let body = rest[..end].trim().to_string();
    if body.is_empty() {
        return None;
    }
    Some(Record {
        id: section.to_string(),
        title,
        date: after("Effective:"),
        source: after("Latest Legislation:"),
        body,
    })
}

/// The marker that begins every record, at the start of a line.
///
/// This was `§ `, justified on the grounds that the Revised Code's own text never prints a
/// section mark at column 0. True of statutes, and false of the opinions the same format now
/// carries: DeRolph I quotes `Art. I, § 2` and an out-of-state `M.S.A. §`, and a repost of a PDF
/// or a different `pdftotext` line-breaking would only have to reflow one of them to column 0 to
/// truncate a record and add a phantom one. A marker no legal text has occasion to print is
/// cheaper than an argument about which line breaks are possible — and [`build_records`] checks
/// rather than assuming.
pub const RECORD_MARKER: &str = "=== ";

/// Render records as a committed fixture.
///
/// A record format rather than CSV, because legal text is full of commas and [`write_csv`] does
/// not quote. `caption` describes what the file holds; the rest of the header is fixed, because
/// the regeneration command and the record marker are properties of the format rather than of
/// the document. The caption is a parameter because reusing the statute builder for the opinions
/// stamped `# Ohio Revised Code, the sections this corpus cites.` over four Supreme Court of Ohio
/// opinions, and a file that misdescribes itself in its first line is a poor foundation for one
/// whose purpose is provenance.
///
/// # Panics
///
/// If any record's body contains a line beginning with [`RECORD_MARKER`], which would split that
/// record in two for every reader. Failing the rebuild is the point: the alternative is a
/// committed fixture that is silently wrong about how many documents it holds.
#[must_use]
pub fn build_records(caption: &str, records: &[Record]) -> String {
    let mut out = String::with_capacity(records.iter().map(|r| r.body.len() + 256).sum());
    out.push_str(&format!(
        "# {caption}\n\
         # Regenerate with `edfund-connect rebuild`. Records begin with `{RECORD_MARKER}` at the\n\
         # start of a line, which no record's own text does.\n"
    ));
    for r in records {
        assert!(
            !r.body.lines().any(|l| l.starts_with(RECORD_MARKER)),
            "{}: a body line begins with {RECORD_MARKER:?} and would split the record",
            r.id
        );
        out.push_str(&format!(
            "\n{RECORD_MARKER}{}\ntitle: {}\ndate: {}\nsource: {}\n--\n{}\n",
            r.id, r.title, r.date, r.source, r.body
        ));
    }
    out
}

/// Write a text fixture, creating its directory.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] if the file cannot be written.
pub fn write_text(path: &Path, contents: &str) -> io::Result<usize> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    Ok(contents.lines().count())
}

// -------------------------------------------------------------------------------------------
// Enacted budget acts
// -------------------------------------------------------------------------------------------

/// Where the enacted-act extract is written, relative to the repository root.
pub const ENACTED_FIXTURE: &str = "crates/project/fixtures/enacted-school-funding.txt";

/// The heading the school funding portion of an LSC final analysis begins at.
///
/// The department's own name, in the document's heading case. "Primary and secondary education"
/// was the first guess and appears nowhere — the rebuild reported the fixture skipped and said so,
/// which is the behaviour the skip exists for.
const FUNDING_HEADING: &str = "DEPARTMENT OF EDUCATION AND WORKFORCE";

/// Reduce a final analysis to the part that funds schools.
///
/// # Why an extract rather than the whole document
///
/// The H.B. 96 final analysis is 26,000 lines and legislates on everything from Medicaid to
/// liquor permits. Committing it whole would put a megabyte of unrelated law in the repository
/// and make the diff on the next budget unreadable. The school funding portion is what any node
/// here cites.
///
/// Returns `None` if the heading is absent, which means the document is not the document this was
/// written against — better than committing a slice of the wrong thing.
#[must_use]
pub fn extract_school_funding(text: &str) -> Option<String> {
    // The heading appears in the contents list first and as a body heading after; take the last
    // occurrence before the education content, which is the body one.
    let start = text.match_indices(FUNDING_HEADING).map(|(i, _)| i).nth(1)?;
    let rest = &text[start..];
    // Runs to the next top-level department heading.
    let end = ["DEPARTMENT OF HIGHER EDUCATION", "DEPARTMENT OF MEDICAID"]
        .iter()
        .filter_map(|m| rest.find(m))
        .filter(|i| *i > 10_000)
        .min()
        .unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

/// Where the department's redbook extract is written, relative to the repository root.
pub const REDBOOK_FIXTURE: &str = "crates/project/fixtures/dew-redbook.txt";

/// The same analysis as enacted. See [`REDBOOK_FIXTURE`], which is the introduced one.
pub const GREENBOOK_FIXTURE: &str = "crates/project/fixtures/dew-greenbook.txt";

/// Where the court opinion extract is written, relative to the repository root.
pub const OPINIONS_FIXTURE: &str = "crates/regime-diff/fixtures/derolph-opinions.txt";

/// Where the EdChoice challenge's appellate record is written, relative to the repository root.
///
/// A second file rather than a fifth record in [`OPINIONS_FIXTURE`], because that file's own first
/// line says *"DeRolph v. State, the four opinions this corpus cites"* and this is neither. The
/// rebuild now selects a connector's sources by the fixture each one declares it feeds, rather
/// than assuming every source of a connector feeds the same file — an assumption that was true
/// while `ohio-courts` held one case and would have quietly stopped being true here.
pub const EDCHOICE_FIXTURE: &str = "crates/regime-diff/fixtures/edchoice-appellate-record.txt";

/// Where the legislative-district crosswalk is written, relative to the repository root.
pub const CROSSWALK_FIXTURE: &str = "crates/project/fixtures/legislative-district-crosswalk.csv";

/// Columns of the legislative-district crosswalk.
pub const CROSSWALK_HEADER: &[&str] = &[
    "chamber",
    "irn",
    "district",
    "population",
    "population_under_18",
    "share",
];

/// `LOGRECNO` to block code, for the block records of a PL 94-171 geoheader.
///
/// The geoheader carries every summary level in one file — state, county, tract, block — and
/// only summary level `750` is a block. `LOGRECNO` is the join key the table files use; the
/// geoheader is the only place it can be turned back into a geography.
#[must_use]
pub fn pl_blocks(geo: &str) -> BTreeMap<String, String> {
    const SUMMARY_LEVEL: usize = 2;
    const LOGRECNO: usize = 7;
    const GEOCODE: usize = 9;
    const BLOCK: &str = "750";

    geo.lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            (f.get(SUMMARY_LEVEL) == Some(&BLOCK))
                .then(|| Some((f.get(LOGRECNO)?.to_string(), f.get(GEOCODE)?.to_string())))
                .flatten()
        })
        .collect()
}

/// `LOGRECNO` to the first count in a PL 94-171 table file.
///
/// The first data column of file 1 is `P0010001`, the total population; of file 2, `P0030001`,
/// the population 18 and over. Nothing here needs any of the other 140-odd columns, and reading
/// only the first keeps a 150 MB file from being parsed into anything larger than a count.
#[must_use]
pub fn pl_counts(table: &str) -> BTreeMap<String, i64> {
    const LOGRECNO: usize = 4;
    const FIRST_COUNT: usize = 5;

    table
        .lines()
        .filter_map(|line| {
            let f: Vec<&str> = line.split('|').collect();
            Some((
                f.get(LOGRECNO)?.to_string(),
                f.get(FIRST_COUNT)?.parse::<i64>().ok()?,
            ))
        })
        .collect()
}

/// Everything the crosswalk is assembled from.
#[derive(Debug, Clone, Copy)]
pub struct Crosswalk<'a> {
    /// Block code to its 2020 total and under-18 population.
    pub population: &'a BTreeMap<String, (i64, i64)>,
    /// The `BlockAssign` unified school district file: block to five-digit district code.
    pub school_districts: &'a str,
    /// The 2024 lower-chamber block equivalency file.
    pub house: &'a str,
    /// And the upper chamber's.
    pub senate: &'a str,
    /// The CCD LEA directory, which carries the district code to IRN join.
    pub directory: &'a str,
    /// The funding panel's IRNs. A district outside it is not carried — see
    /// [`build_legislative_crosswalk`].
    pub panel: &'a BTreeSet<String>,
}

/// Apportion each school district's pupils across the legislative seats that contain them.
///
/// # Why this has to be built from blocks
///
/// Ohio's funding system contains no legislative district, so the mapping does not exist to be
/// downloaded. The census block is the only geography both a school district and a House seat are
/// built out of, and 339 of the 609 districts straddle two or more seats — so a seat cannot be an
/// attribution the way a county can, and the crosswalk carries a share rather than an assignment.
///
/// # The weight is children, not people
///
/// A seat's share of a district is its share of the district's **under-18 population**, not of its
/// total. A district's pupils are what is being apportioned, and Ohio's under-18 share varies
/// enough between blocks that the choice moves the answer. Shares sum to one per district and
/// chamber, which is what lets
/// [`project::legislative_district`](../../project/src/legislative_district.rs) apportion a
/// statewide total across seats without losing or inventing a dollar.
///
/// # What is left out
///
/// A block with no population contributes nothing and is not carried; 53 such rows would
/// otherwise appear with a share of zero. Districts outside the funding panel are dropped
/// entirely — the two Lake Erie island districts, Middle Bass and North Bass, are in the CCD
/// directory and in the census, and are not in the model this crosswalk exists to apportion.
///
/// # Errors
///
/// Returns the missing column's name if the CCD directory's layout has moved.
pub fn build_legislative_crosswalk(sources: &Crosswalk<'_>) -> Result<Vec<Vec<String>>, String> {
    let mut rows = sources.directory.lines();
    let head = delimited_fields(rows.next().unwrap_or_default(), ',');
    let (key, st_leaid) = (
        column(&head, "LEAID", "the CCD LEA directory")?,
        column(&head, "ST_LEAID", "the CCD LEA directory")?,
    );
    // The census keys school districts by a five-digit local code; the CCD's LEAID is the state
    // FIPS followed by that code, and its ST_LEAID is `OH-` followed by the IRN.
    let mut irn_of: BTreeMap<String, String> = BTreeMap::new();
    for line in rows {
        let f = delimited_fields(line, ',');
        let (Some(leaid), Some(irn)) = (f.get(key), f.get(st_leaid)) else {
            continue;
        };
        if let Some(local) = leaid.trim().strip_prefix("39") {
            irn_of.insert(
                local.to_string(),
                irn.trim()
                    .strip_prefix("OH-")
                    .unwrap_or(irn.trim())
                    .to_string(),
            );
        }
    }

    // `BLOCKID|DISTRICT`, against the equivalency files' `GEOID,SLDLST`.
    let assignment = |text: &str, delimiter: char| -> BTreeMap<String, String> {
        text.lines()
            .skip(1)
            .filter_map(|line| {
                let (block, district) = line.trim_end().split_once(delimiter)?;
                Some((block.to_string(), district.to_string()))
            })
            .collect()
    };
    let in_district = assignment(sources.school_districts, '|');
    let chambers = [
        ("house", assignment(sources.house, ',')),
        ("senate", assignment(sources.senate, ',')),
    ];

    let mut totals: BTreeMap<(&str, String, String), (i64, i64)> = BTreeMap::new();
    for (block, (people, children)) in sources.population {
        if *people == 0 {
            continue;
        }
        let Some(irn) = in_district.get(block).and_then(|code| irn_of.get(code)) else {
            continue;
        };
        if !sources.panel.contains(irn) {
            continue;
        }
        for (chamber, blocks) in &chambers {
            let Some(seat) = blocks.get(block) else {
                continue;
            };
            let entry = totals
                .entry((chamber, irn.clone(), seat.clone()))
                .or_insert((0, 0));
            entry.0 += people;
            entry.1 += children;
        }
    }

    let mut pupils: BTreeMap<(&str, &String), i64> = BTreeMap::new();
    for ((chamber, irn, _), (_, children)) in &totals {
        *pupils.entry((chamber, irn)).or_insert(0) += children;
    }

    // `BTreeMap` orders by key, and the key is (chamber, IRN, seat) — but "house" sorts before
    // "senate" only by luck of the alphabet, so the chamber order is made explicit.
    let mut out: Vec<(usize, Vec<String>)> = totals
        .iter()
        .map(|((chamber, irn, seat), (people, children))| {
            let of_district = pupils.get(&(chamber, irn)).copied().unwrap_or(0);
            let share = if of_district > 0 {
                #[allow(clippy::cast_precision_loss)]
                let value = *children as f64 / of_district as f64;
                value
            } else {
                0.0
            };
            (
                usize::from(*chamber == "senate"),
                vec![
                    (*chamber).to_string(),
                    irn.clone(),
                    seat.clone(),
                    people.to_string(),
                    children.to_string(),
                    format_value(Some(share), 8),
                ],
            )
        })
        .collect();
    out.sort_by_key(|a| a.0);
    Ok(out.into_iter().map(|(_, row)| row).collect())
}

/// Every fixture [`crate::rebuild`] writes.
///
/// The list exists so that "a source declares a fixture" and "something regenerates it" can be
/// checked against each other without a populated cache, which CI does not have. It caught the
/// F-33 district panel: 754 KB committed, read by a calculator, declared by a `Source`, and
/// produced by nothing — the digest manifest could not see it, because a digest pins the input
/// and says nothing about whether the derivation from it still exists.
pub const REBUILT: &[&str] = &[
    FY27_FIXTURE,
    PROFILE_FIXTURE,
    GRADE_BANDS_FIXTURE,
    REPORT_CARD_FIXTURE,
    FUNCTIONS_FIXTURE,
    FINANCE_FIXTURE,
    SD1_FIXTURE,
    CASINO_FIXTURE,
    CPI_FIXTURE,
    F33_FIXTURE,
    F33_FY2024_FIXTURE,
    F33_DISTRICTS_FIXTURE,
    F33_OHIO_PANEL_FIXTURE,
    MR81_FIXTURE,
    CCD_DIRECTORY_FIXTURE,
    SESSION_LAW_FIXTURE,
    TRANSFER_FIXTURE,
    BUILDING_FIXTURE,
    IDENTIFIED_FIXTURE,
    CROSSWALK_FIXTURE,
    STATUTE_FIXTURE,
    ENACTED_FIXTURE,
    REDBOOK_FIXTURE,
    GREENBOOK_FIXTURE,
    OPINIONS_FIXTURE,
    EDCHOICE_FIXTURE,
    APPROPRIATION_FIXTURE,
    SCHOLARSHIP_FIXTURE,
    CATALOG_FIXTURE,
    CATALOG_BASIS_FIXTURE,
];

/// Fixtures a [`crate::registry::Source`] declares that [`crate::rebuild`] does not produce.
///
/// **An entry here is a debt with a name on it, not a category of fixture.** The pairing is the
/// path and what would have to be written to retire it.
///
/// The list is empty, and the two entries it held are the reason it exists: the F-33 district
/// panel and the legislative-district crosswalk were both committed, both read by a calculator,
/// both declared by a `Source`, and both produced by nothing. Neither was discoverable — a digest
/// pins the bytes that went in and cannot notice that the derivation from them was never written.
///
/// The test holds this list to both directions — an unlisted gap fails, and so does an entry that
/// has quietly been fixed — so it cannot grow silently or rot after the work is done.
pub const NOT_REGENERATED: &[(&str, &str)] = &[];

/// The date an opinion was decided, read off the document.
///
/// The supreme court prints a parenthetical on the first page — `(No. 95-2066--Submitted
/// September 10, 1996--Decided March 24, 1997.)` — with the dash rendering as `--` in the 1997 PDF
/// and as an em dash in the later three. Only the text after `Decided ` is wanted, so the dash
/// never has to be matched.
///
/// **The courts of appeals do not print that parenthetical.** They print `Rendered on March 29,
/// 2024` on its own line instead, and the first opinion of theirs this repository wired came out
/// with an empty date field. Both markers are tried, most specific first.
///
/// Returns an empty string if neither is present, which is the honest answer: a [`Record`] whose
/// date could not be read should say nothing rather than guess. It is worth reading at all because
/// the previous extract discarded the one date these documents actually print and wrote an empty
/// field in its place.
#[must_use]
pub fn decided_on(body: &str) -> String {
    const MARKERS: [&str; 2] = ["Decided ", "Rendered on "];
    let Some((start, marker)) = MARKERS
        .iter()
        .filter_map(|m| body.find(m).map(|at| (at, *m)))
        .min()
    else {
        return String::new();
    };
    let rest = &body[start + marker.len()..];
    // `March 24, 1997.)` — the date ends at the sentence stop that closes the parenthetical, or
    // at the end of the line, **whichever comes first**. Preferring `.)` and falling back to the
    // newline is what the supreme court's layout suggests and it is wrong: an opinion that prints
    // the date bare on its own line has no closing parenthesis, so the search runs on to the next
    // one anywhere in the document. The first appellate opinion wired here came out with
    // sixty-eight lines of holding in its date field.
    let end = [rest.find(".)"), rest.find('\n')]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(rest.len());
    rest[..end].trim().to_string()
}

// -------------------------------------------------------------------------------------------
// Territory transfers, as the Auditor of State recites them
// -------------------------------------------------------------------------------------------

/// Where the recited transfer orders are written, relative to the repository root.
pub const TRANSFER_FIXTURE: &str = "crates/dispersion/fixtures/territory-transfers.tsv";

/// Columns of the transfer extract.
///
/// Written as TSV rather than CSV, and for once the reason is the *quotation* rather than a name.
/// `recital` is a sentence from a state officer's report and it is carried verbatim; the writer
/// this repository uses does not quote, so a comma in a field is refused rather than escaped. A
/// quotation with its commas stripped is a damaged quotation, so the file takes the other
/// delimiter. The same choice was made for the Catalog's legal-basis extract, for the same reason.
pub const TRANSFER_HEADER: &[&str] = &[
    "report",
    "audited_entity",
    "role",
    "resolving_body",
    "resolution_date",
    "effective_date",
    "departing",
    "receiving",
    "section",
    "recital",
];

/// One audit report and what this reader expects to find in it.
///
/// The expectations are part of the extract rather than notes beside it. A recital is a sentence,
/// and a sentence extractor that finds *something* is not evidence it found the right thing — so
/// each report declares the departing agency, the receiving one and the resolution date it should
/// yield, and [`build_transfers`] fails if the document does not say so.
#[derive(Debug, Clone, Copy)]
pub struct AuditReport<'a> {
    /// The source key, which is how a row is traced back to a digest.
    pub report: &'a str,
    /// The entity whose audit this is.
    pub audited_entity: &'a str,
    /// `departing`, `receiving`, or `resolving` — where the audited entity stands to the transfer.
    pub role: &'a str,
    /// The body whose resolution the report recites.
    pub resolving_body: &'a str,
    /// The date that resolution was passed, as the report gives it.
    pub resolution_date: &'a str,
    /// When the transfer took effect.
    pub effective_date: &'a str,
    /// The agency that ceased.
    pub departing: &'a str,
    /// The agency that took its territory.
    pub receiving: &'a str,
    /// The Revised Code section, where the report names one. Empty otherwise, which is the
    /// ordinary case: only one of these five cites a section.
    pub section: &'a str,
    /// The words the recital opens with.
    ///
    /// Declared per report rather than inferred, because a sentence boundary is not one in a PDF
    /// financial statement: page numbers, running heads and all-capital note titles carry no full
    /// stop, so bounding on `". "` alone reaches back through `- iv -` and
    /// `NOTES TO THE BASIC FINANCIAL STATEMENTS` and swallows them into the quotation. Stating the
    /// opening is also the check — a report that no longer begins its recital this way has been
    /// revised, and that should stop the build rather than change a committed quotation.
    pub opens_with: &'a str,
    /// `pdftotext -layout` output for the whole report.
    pub text: &'a str,
}

/// The transfer orders, as recited in the Auditor of State's reports.
///
/// # Why a recital rather than the order
///
/// Three Ohio school districts stopped existing inside the window
/// [`dispersion::lea_directory`](../../dispersion/src/lea_directory.rs) holds, and the federal
/// agency directory files all three as closed *"with no effect on another agency's boundaries"*,
/// which is false. R.C. 3311.22 puts the order in an educational service center governing board's
/// minute book, and those are not published: one sits behind a vendor firewall, another begins its
/// public archive eight years after the resolution.
///
/// What is published is the Auditor of State's report on the **receiving** district, which recites
/// the resolution by date and issuing body. That is a state officer's account of a local body's
/// act, and it is the only route to the fact that this repository can commit.
///
/// **The proposing body's own audit does not mention it.** Geauga County ESC's final report spans
/// both Newbury resolutions and never says "Newbury"; its FY2015 report spans the Ledgemont
/// resolution and never says "Ledgemont". Anyone looking for a transfer where the order was made
/// finds nothing and concludes wrongly that nothing happened.
///
/// # What is checked
///
/// Each report declares what it should say before it is read. The extractor requires the departing
/// agency, the receiving agency and the resolution date all to appear, and it takes the recital
/// from the sentence carrying the date rather than from wherever the names first occur. A report
/// that has been reposted, or a sentence extractor that drifts onto the wrong paragraph, fails
/// here instead of writing a plausible row.
///
/// # Errors
///
/// Returns which expectation a report did not meet.
pub fn build_transfers(reports: &[AuditReport<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for report in reports {
        // Whitespace in these PDFs wraps mid-sentence, so everything is matched against a single
        // collapsed line rather than against the layout.
        let flat = collapse(report.text);
        for expected in [report.departing, report.receiving, report.resolution_date] {
            if !flat.contains(expected) {
                return Err(format!(
                    "{}: the report does not contain {expected:?}, so it is not the document this \
                     reader was written against",
                    report.report
                ));
            }
        }
        let recital = sentence_around(
            &flat,
            report.resolution_date,
            report.departing,
            report.opens_with,
        )
        .ok_or_else(|| {
            format!(
                "{}: no sentence opens {:?} and carries both {} and {}, so either the report has \
                 been revised or the date belongs to something else in it",
                report.report, report.opens_with, report.resolution_date, report.departing
            )
        })?;
        out.push(vec![
            report.report.to_string(),
            report.audited_entity.to_string(),
            report.role.to_string(),
            report.resolving_body.to_string(),
            report.resolution_date.to_string(),
            report.effective_date.to_string(),
            report.departing.to_string(),
            report.receiving.to_string(),
            report.section.to_string(),
            // Verbatim. Tabs are the delimiter and no report contains one; a recital that did
            // would be caught by the writer rather than silently split.
            recital,
        ]);
    }
    Ok(out)
}

/// The whole document as one line, with runs of whitespace collapsed.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// The sentence carrying both `needle` and `must_name`, bounded by full stops.
///
/// Bounded rather than windowed by character count: a fixed window cuts a recital in half and
/// leaves half of somebody else's, which reads as a quotation and is not one.
///
/// **Every** occurrence of the date is tried, not the first. These reports repeat a date in a
/// contents page, a transmittal letter and a note, and the first occurrence is routinely the one
/// that says nothing — which is what this function returned before it took a second condition.
fn sentence_around(flat: &str, needle: &str, must_name: &str, opens_with: &str) -> Option<String> {
    /// Long enough for the longest of these recitals and far short of a financial table, which is
    /// what an unbounded sentence turns into in a document with no full stops in its headings.
    const LONGEST: usize = 600;

    let mut from = 0usize;
    while let Some(offset) = flat[from..].find(needle) {
        let at = from + offset;
        // The declared opening, taken from before the date where the sentence leads up to it and
        // from the date itself where it opens on it.
        let start = flat[..at + needle.len()].rfind(opens_with);
        let end = flat[at..]
            .find(". ")
            .map_or(flat.len(), |stop| at + stop + 1);
        if let Some(start) = start {
            let sentence = flat[start..end].trim();
            if sentence.len() <= LONGEST && sentence.contains(must_name) {
                return Some(sentence.to_string());
            }
        }
        from = at + needle.len();
    }
    None
}

// -------------------------------------------------------------------------------------------
// Session laws: the appropriation tables in the acts themselves
// -------------------------------------------------------------------------------------------

/// Where the pre-FY2002 enacted appropriation lines are written, relative to the repository root.
pub const SESSION_LAW_FIXTURE: &str = "crates/project/fixtures/session-law-lines.csv";

/// Columns of the session-law extract.
pub const SESSION_LAW_HEADER: &[&str] = &[
    "general_assembly",
    "bill",
    "fiscal_year",
    "fund_group",
    "fund",
    "line_item",
    "title",
    "amount",
];

/// One enrolled act, with the two fiscal years its money columns stand for.
#[derive(Debug, Clone, Copy)]
pub struct ActText<'a> {
    /// The General Assembly that passed it.
    pub general_assembly: u16,
    /// The bill, as the registry keys it: `hb215`.
    pub bill: &'a str,
    /// The heading that opens the Department of Education appropriation table.
    pub heading: &'a str,
    /// The fiscal year the first money column stands for. The second is the year after.
    pub first_year: u16,
    /// `pdftotext -layout` output for the whole act.
    pub text: &'a str,
}

/// The fund-group totals an act prints, which are what the parsed rows must reproduce.
struct GroupTotal {
    label: String,
    first: i64,
    second: i64,
}

/// Every Department of Education appropriation line in the acts this repository holds.
///
/// # Why the acts and not an analysis of them
///
/// Everything else in the appropriation series comes from the Legislative Service Commission —
/// greenbooks, budget workbooks, the Catalog of Budget Line Items — and all three stop at FY2002
/// or later. The acts stop where the legislature's own archive stops, which is the 122nd General
/// Assembly, and that is four fiscal years earlier.
///
/// It is also a different kind of document. A greenbook is LSC describing what an act did; this is
/// the act. Where they can be compared they agree, and where they cannot the act is the thing that
/// was voted on.
///
/// # The reconciliation is the whole quality bar
///
/// Each fund group's table ends with a printed total, and the act closes with a grand total across
/// groups. Every parsed row is summed back against them, in both fiscal-year columns, and a
/// mismatch of one dollar fails the rebuild. That check is not decoration: it is the only thing
/// standing between this and a plausible table with a row missing, which is a failure this
/// repository has shipped before and could not see afterwards.
///
/// # What the FY1999 column means, and why no reader should average it
///
/// [`crate::registry`]'s note on `hb215-122-enrolled` has the detail. In short: fifty-one of
/// H.B. 215's fifty-three GRF education lines are appropriated **zero** for FY1999, and the whole
/// year sits in one line, `200-405`. That is not a defect in this reader and it is not a cut. Nine
/// months after the Supreme Court held the funding system unconstitutional, the General Assembly
/// passed a budget that declined to itemise the second year, and said so in the act:
///
/// > By January 15, 1998, the General Assembly shall develop a plan to provide itemized
/// > appropriations for the Department of Education for fiscal year 1999.
///
/// The rows are emitted as the act states them, zeros included, and the consumer is expected to
/// read `line_item` `200405` as the marker it is. See `project::session_laws`.
///
/// # Errors
///
/// Returns a description if an act's education heading is absent, if a fund group's rows do not
/// sum to its printed total, or if the group totals do not sum to the printed grand total.
pub fn build_session_laws(acts: &[ActText<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for act in acts {
        let label = format!(
            "{} of the {}th General Assembly",
            act.bill, act.general_assembly
        );
        let start = act
            .text
            .find(act.heading)
            .ok_or_else(|| format!("{label} has no {:?} heading", act.heading))?;
        // The table ends at its own grand total; everything after is the earmark prose.
        let rest = &act.text[start..];
        let end = rest
            .find("TOTAL ALL BUDGET FUND GROUPS")
            .ok_or_else(|| format!("{label} prints no grand total to check against"))?;
        let table = &rest[..end];
        let closing = &rest[end..];

        let (rows, totals) = session_law_rows(table, act)?;
        reconcile_act(&rows, &totals, closing, &label, act)?;
        out.extend(rows.into_iter().flat_map(|row| row.into_fields(act)));
    }
    Ok(out)
}

/// One appropriation line in one act, before it becomes two fixture rows.
struct ActLine {
    group: String,
    fund: String,
    item: String,
    title: String,
    first: i64,
    second: i64,
}

impl ActLine {
    /// One fixture row per fiscal year, because every consumer of the appropriation series reads
    /// a row as one year's claim about one line.
    fn into_fields(self, act: &ActText<'_>) -> [Vec<String>; 2] {
        let row = |year: u16, amount: i64| {
            vec![
                act.general_assembly.to_string(),
                act.bill.to_string(),
                year.to_string(),
                self.group.clone(),
                self.fund.clone(),
                self.item.clone(),
                self.title.clone(),
                amount.to_string(),
            ]
        };
        [
            row(act.first_year, self.first),
            row(act.first_year + 1, self.second),
        ]
    }
}

/// Read the rows and the printed fund-group totals out of one act's education table.
///
/// The layout is fixed and has been since at least 1997: a fund code, the hyphenated line item, a
/// title that may wrap onto the following lines, and two right-aligned dollar amounts. A wrapped
/// title carries no amounts, which is how the continuation is recognised without measuring
/// columns.
fn session_law_rows(
    table: &str,
    act: &ActText<'_>,
) -> Result<(Vec<ActLine>, Vec<GroupTotal>), String> {
    let mut rows: Vec<ActLine> = Vec::new();
    let mut totals: Vec<GroupTotal> = Vec::new();
    let mut group = String::new();
    let mut pending: Option<String> = None;
    // Where the last row or total put its two amounts, so a replacement printed beneath it can be
    // assigned to the year it replaces. `None` until the first row of the table.
    let mut last: Option<Columns> = None;
    // Whether the open thing is a row or a total, because an amending act corrects both.
    let mut last_was_total = false;

    for line in table.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        // An amendment binds to whatever it sits under, and it may sit two lines under: the
        // amended figure for `200-545 Vocational Education Enhancements` follows the wrapped
        // second half of its own title. So this is tried before the row and title cases and the
        // open row stays open across a wrap.
        if let Some(columns) = last {
            if let Some([first, second]) = amendment(line, columns) {
                if last_was_total {
                    if let Some(total) = totals.last_mut() {
                        total.first = first.unwrap_or(total.first);
                        total.second = second.unwrap_or(total.second);
                    }
                } else if let Some(row) = rows.last_mut() {
                    row.first = first.unwrap_or(row.first);
                    row.second = second.unwrap_or(row.second);
                }
                continue;
            }
        }
        // A total's label wraps as often as not — `TOTAL GSF General Services` on one line and
        // `Fund Group   $ 12,695,447 $ 13,033,594` on the next — so a `TOTAL` with no money on it
        // opens a label that the following line closes.
        if let Some(total) = printed_total(trimmed) {
            pending = None;
            last = columns_of(line);
            last_was_total = true;
            totals.push(total);
            continue;
        }
        if trimmed.len() > 6 && trimmed[..6].eq_ignore_ascii_case("total ") {
            pending = Some(trimmed.to_string());
            continue;
        }
        // The wrap can fall mid-word — `TOTAL LPE Lottery Pr` / `ofits Education` / `Fund Group
        // $ 699,892,200 ...` — so the label accumulates until a line carries the amounts rather
        // than for a fixed number of lines.
        if let Some(head) = pending.as_mut() {
            head.push(' ');
            head.push_str(trimmed);
            if let Some(total) = printed_total(head) {
                last = columns_of(line);
                last_was_total = true;
                totals.push(total);
                pending = None;
            }
            continue;
        }
        match appropriation_row(trimmed) {
            Some((fund, item, title, first, second)) => {
                last = columns_of(line);
                last_was_total = false;
                rows.push(ActLine {
                    group: group.clone(),
                    fund,
                    item,
                    title,
                    first,
                    second,
                });
            }
            None => {
                // A line with no amounts is either a fund-group heading or a wrapped title. The
                // difference is whether a row is open: the heading always precedes the first row
                // of its group.
                if trimmed.ends_with("Fund Group") || trimmed.ends_with("Fund") {
                    group = trimmed.to_string();
                } else if let Some(last) = rows.last_mut() {
                    if !trimmed.starts_with("H. B. No.")
                        && !trimmed.chars().all(|c| c.is_ascii_digit())
                    {
                        last.title.push(' ');
                        last.title.push_str(trimmed);
                    }
                }
            }
        }
    }

    if rows.is_empty() {
        return Err(format!(
            "{} of the {}th yielded no appropriation rows",
            act.bill, act.general_assembly
        ));
    }
    Ok((rows, totals))
}

/// Where a line's two money columns end, so an amendment beneath it can be assigned to one.
///
/// By position because that is the only thing that says which column an amending act is changing:
/// H.B. 770 prints the amended figure on its own line under the column it replaces, and a line
/// carrying one number is otherwise silent about which of the two years it belongs to.
#[derive(Debug, Clone, Copy)]
struct Columns {
    first: usize,
    second: usize,
}

/// `GRF 200-501 School Foundation Basic   $  2,202,851,688 $  0` — or `None` if the line is not one.
fn appropriation_row(line: &str) -> Option<(String, String, String, i64, i64)> {
    let mut parts = line.split_whitespace();
    let fund = parts.next()?.to_string();
    // Fund codes are three characters: `GRF`, `017`, `4D1`, `3L6`.
    if fund.len() != 3 || !fund.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let item = parts.next()?;
    let (head, tail) = item.split_once('-')?;
    if head.len() != 3 || tail.len() != 3 || !item.chars().all(|c| c.is_ascii_digit() || c == '-') {
        return None;
    }
    let rest: Vec<&str> = parts.collect();
    // Two dollar signs close the row and everything between the item and the first is the title.
    let mut money = rest.iter().enumerate().filter(|(_, w)| **w == "$");
    let (first_at, _) = money.next()?;
    let (second_at, _) = money.next()?;
    let amount =
        |at: usize| -> Option<i64> { rest.get(at + 1)?.replace(',', "").parse::<i64>().ok() };
    Some((
        fund,
        item.replace('-', ""),
        clean_name(&rest[..first_at].join(" ")),
        amount(first_at)?,
        amount(second_at)?,
    ))
}

/// `TOTAL GRF General Revenue Fund   $ 4,899,708,534 $ 5,134,145,592`, or `None`.
/// The end positions of the last two comma-grouped numbers on a line.
fn columns_of(line: &str) -> Option<Columns> {
    let ends: Vec<usize> = number_spans(line)
        .into_iter()
        .map(|(_, end, _)| end)
        .collect();
    match ends[..] {
        [.., first, second] => Some(Columns { first, second }),
        _ => None,
    }
}

/// Every run of digits and commas on a line, with where it starts and ends.
fn number_spans(line: &str) -> Vec<(usize, usize, i64)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut at = 0usize;
    while at < bytes.len() {
        if !bytes[at].is_ascii_digit() {
            at += 1;
            continue;
        }
        let start = at;
        while at < bytes.len() && (bytes[at].is_ascii_digit() || bytes[at] == b',') {
            at += 1;
        }
        // A trailing comma belongs to the prose, not to the number.
        let end = line[start..at].trim_end_matches(',').len() + start;
        if let Ok(value) = line[start..end].replace(',', "").parse::<i64>() {
            out.push((start, end, value));
        }
    }
    out
}

/// An amending act's replacement figures, aligned under the columns they replace.
///
/// `None` unless the line is nothing but numbers **and** at least one of them ends where one of
/// the row above's amounts ends. That second condition is what keeps page furniture out: the
/// enrolled acts print a bare page number on its own line, and `247` under a row whose columns
/// end at 59 and 74 is not an amendment to anything.
fn amendment(line: &str, columns: Columns) -> Option<[Option<i64>; 2]> {
    if line.trim().is_empty() || line.chars().any(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let mut out = [None, None];
    let mut aligned = false;
    for (_, end, value) in number_spans(line) {
        // Two characters of slack: the columns are right-aligned but the amending line is set
        // separately and the two do not always land on exactly the same byte.
        if end.abs_diff(columns.first) <= 2 {
            out[0] = Some(value);
            aligned = true;
        } else if end.abs_diff(columns.second) <= 2 {
            out[1] = Some(value);
            aligned = true;
        }
    }
    aligned.then_some(out)
}

/// The prefix is matched case-insensitively because one group's is not like the others: H.B. 215
/// closes the lottery funds with `Total 017 and 018 LPE Lottery Profits Education Fund Group`,
/// lowercase and named by fund number rather than by group. It carries $699,892,200, and the
/// reconciliation is what noticed it was missing.
fn printed_total(line: &str) -> Option<GroupTotal> {
    if line.len() <= 6 || !line[..6].eq_ignore_ascii_case("total ") {
        return None;
    }
    let rest = &line[6..];
    let words: Vec<&str> = rest.split_whitespace().collect();
    let mut money = words.iter().enumerate().filter(|(_, w)| **w == "$");
    let (first_at, _) = money.next()?;
    let (second_at, _) = money.next()?;
    let amount =
        |at: usize| -> Option<i64> { words.get(at + 1)?.replace(',', "").parse::<i64>().ok() };
    Some(GroupTotal {
        label: words[..first_at].join(" "),
        first: amount(first_at)?,
        second: amount(second_at)?,
    })
}

/// Discrepancies in an act's own arithmetic, each one named and its size fixed.
///
/// Not a tolerance. A tolerance would let any small error through and would grow silently; this
/// is a list of specific known defects, and a difference that is not on it — or is on it at a
/// different size — still fails the rebuild.
///
/// Each entry is the amount by which the act's own rows **exceed** what it foots them to, and
/// `at_grand_total` says which of the two additions is wrong: the rows against their fund-group
/// totals, or those totals against the grand total. An act is wrong in one place and right in the
/// other, so the allowance has to be aimed.
///
/// **H.B. 282, FY2001.** Its printed GRF total is one dollar *more* than its own fifty-two rows,
/// while its five group totals sum to its grand total exactly. The dollar sits between the rows
/// and the GRF footing and there is nothing to correct it against. Carried rather than absorbed
/// because $1 on $7.98 billion is exactly what a tolerance wide enough to hide a missing row
/// would swallow.
///
/// **H.B. 770, FY1999.** Its fund-group totals sum to $1,443,401 *more* than its printed grand
/// total, and this one is resolvable in the rows' favour: the missing amount is the Education
/// Improvement Fund's `006 200-689 Hazardous Waste Removal`, which the act's FY1998 grand total
/// includes and its FY1999 grand total does not. `appropriation-lines.csv` already carries that
/// line as a FY1999 **actual** of exactly $1,443,401, from the H.B. 94 greenbook — so the money
/// was appropriated and spent, and it is the act's footing that is wrong rather than its rows.
const ACT_FOOTING_DEFECTS: [(&str, u16, i64, bool); 2] =
    [("hb282", 2001, -1, false), ("hb770", 1999, 1_443_401, true)];

/// Sum the rows back against every total the act prints, in both columns.
fn reconcile_act(
    rows: &[ActLine],
    totals: &[GroupTotal],
    closing: &str,
    label: &str,
    act: &ActText<'_>,
) -> Result<(), String> {
    if totals.is_empty() {
        return Err(format!("{label} prints no fund-group totals"));
    }
    let summed = |k: usize| -> i64 {
        rows.iter()
            .map(|r| if k == 0 { r.first } else { r.second })
            .sum()
    };
    let printed = |k: usize| -> i64 {
        totals
            .iter()
            .map(|t| if k == 0 { t.first } else { t.second })
            .sum()
    };
    let allowed = |year: u16, at_grand_total: bool| -> i64 {
        ACT_FOOTING_DEFECTS
            .iter()
            .find(|(bill, defect_year, _, at)| {
                *bill == act.bill && *defect_year == year && *at == at_grand_total
            })
            .map_or(0, |(_, _, by, _)| *by)
    };
    for (k, year) in [(0usize, act.first_year), (1, act.first_year + 1)] {
        if summed(k) - printed(k) != allowed(year, false) {
            let names: Vec<&str> = totals.iter().map(|t| t.label.as_str()).collect();
            return Err(format!(
                "{label}: FY{year} rows sum to {} against the {} its fund-group totals print \
                 ({names:?}); a row is missing or double-counted",
                summed(k),
                printed(k)
            ));
        }
    }
    // And the groups against the act's own grand total, which is a separate assertion: the groups
    // can each be internally consistent and still not be all of them.
    // The grand total wraps like any other and is amended like any other: H.B. 770 prints its
    // own replacement beneath it, and reading only the first line gets the figure the act is
    // superseding rather than the one it enacts.
    let mut head = String::new();
    let mut grand = None;
    let mut at = 0usize;
    for (i, line) in closing.lines().take(4).enumerate() {
        head.push(' ');
        head.push_str(line.trim());
        if let Some(total) = printed_total(head.trim()) {
            grand = Some((total, columns_of(line)));
            at = i;
            break;
        }
    }
    let (mut grand, columns) =
        grand.ok_or_else(|| format!("{label}: the grand total does not parse"))?;
    if let (Some(columns), Some(next)) = (columns, closing.lines().nth(at + 1)) {
        if let Some([first, second]) = amendment(next, columns) {
            grand.first = first.unwrap_or(grand.first);
            grand.second = second.unwrap_or(grand.second);
        }
    }
    for (k, year, stated) in [
        (0usize, act.first_year, grand.first),
        (1, act.first_year + 1, grand.second),
    ] {
        if printed(k) - stated != allowed(year, true) {
            return Err(format!(
                "{label}: FY{year} fund-group totals sum to {} against the grand total's {stated}",
                printed(k)
            ));
        }
    }
    Ok(())
}

// -------------------------------------------------------------------------------------------
// CCD LEA directory, longitudinal
// -------------------------------------------------------------------------------------------

/// Where the Ohio slice of the CCD agency directory is written, relative to the repository root.
pub const CCD_DIRECTORY_FIXTURE: &str = "crates/dispersion/fixtures/ccd-lea-directory.csv";

/// Columns of the directory panel.
pub const CCD_DIRECTORY_HEADER: &[&str] = &[
    "school_year",
    "leaid",
    "irn",
    "name",
    "agency_type",
    "status",
];

/// One published year of the LEA universe directory.
#[derive(Debug, Clone, Copy)]
pub struct DirectoryYear<'a> {
    /// The school year it describes, as the calendar year it begins in: 2008 for 2008-09.
    pub opens: u16,
    /// The delimited file's text.
    pub text: &'a str,
}

/// Column names that mean the same thing in different years, most recent naming first.
///
/// Resolved by trying each in turn rather than by a per-year table. The year suffix is the reason:
/// 2008-09 calls it `STID08` and 2009-10 calls it `STID09`, so a table would carry one row per
/// year to say one thing. A list of aliases says the same thing once and fails loudly if a year
/// uses none of them.
const CCD_ALIASES: [(&str, &[&str]); 5] = [
    ("the agency identifier", &["LEAID"]),
    (
        "the state's own identifier",
        &["ST_LEAID", "STID", "STID09", "STID08", "STID07"],
    ),
    (
        "the agency name",
        &["LEA_NAME", "NAME", "NAME09", "NAME08", "NAME07"],
    ),
    (
        "the agency type",
        &["LEA_TYPE", "TYPE", "TYPE09", "TYPE08", "TYPE07"],
    ),
    (
        "the operational status",
        &["SY_STATUS", "BOUND", "BOUND09", "BOUND08", "BOUND07"],
    ),
];

/// Ohio's FIPS state code, which is how a row is selected.
const OHIO_FIPS: &str = "39";

/// One fixed-width year of the directory, by the byte positions its columns occupy.
///
/// Positions are one-based and inclusive, as the layout documents write them, and are converted
/// once at the point of use rather than stored two ways.
#[derive(Debug, Clone, Copy)]
struct FixedWidth {
    /// The school year, as the calendar year it opens in.
    opens: u16,
    /// The record length, every byte of it, excluding the terminator.
    record: usize,
    /// Where the agency name ends. It starts at 22 in every year.
    name_ends: usize,
    /// The single byte holding the agency type.
    agency_type: usize,
    /// The single byte holding the boundary-change status.
    status: usize,
}

/// Every year of the directory that is fixed-width, with where its columns are.
///
/// # Why a per-year table and not a per-era one
///
/// Because the record length moves in seven of the thirteen years and there is no era to speak
/// of. What the table also shows, which the issue that asked for this reader did not expect, is
/// that **almost nothing this reader wants moves with it**. The agency identifier is bytes 1-7 in
/// every year, the state identifier 8-21 in every year, and the name starts at 22 in every year.
/// Two columns move at all: the name's end and the agency type move exactly once, together, in
/// 1998-99; the status moves four times. A reader written per era would still be wrong, and a
/// reader written per year is thirteen lines.
///
/// # The record length is the check, not decoration
///
/// Each length is the `LRECL` its own layout document states, and [`build_ccd_directory`] holds
/// every row to it. That is what makes a wrong table loud: a layout that does not describe the
/// file fails on the first record rather than yielding plausible garbage from the wrong offsets.
///
/// # Where the table disagrees with the publisher
///
/// **2003-04.** Its layout document gives the status column as position 281 and, elsewhere in the
/// same document, as 384. 281 is the metropolitan-statistical-area indicator's position — stated
/// four lines above, for the same year — and 284 is the one byte the document leaves unaccounted
/// for between the locale code at 283 and the low grade span at 285. The neighbouring years put
/// the column at 284. So does this table, and it is not a guess: at 284 the codes are the
/// boundary-change vocabulary and every agency the file flags closed is absent from the following
/// year, which is the property [`build_ccd_directory`] checks across all thirty years. At 281 the
/// column reads `1` or `2` for reasons of geography and the check fails immediately.
const CCD_FIXED_WIDTH: [FixedWidth; 13] = [
    // 1994-95 through 1997-98: the wide record, the 30-character name.
    fixed_width(1994, 1030, 51, 121, 162),
    fixed_width(1995, 1030, 51, 121, 162),
    fixed_width(1996, 1030, 51, 121, 162),
    fixed_width(1997, 1030, 51, 121, 162),
    // 1998-99: the name doubles to 60 characters and everything after it shifts.
    fixed_width(1998, 722, 81, 234, 280),
    fixed_width(1999, 722, 81, 234, 280),
    fixed_width(2000, 723, 81, 234, 281),
    fixed_width(2001, 725, 81, 234, 281),
    fixed_width(2002, 729, 81, 234, 284),
    // 2003-04: 284 against a document that says 281. See the note above.
    fixed_width(2003, 729, 81, 234, 284),
    // 2004-05: the dropout and completer counts leave and the record almost halves.
    fixed_width(2004, 519, 81, 234, 284),
    fixed_width(2005, 519, 81, 234, 284),
    // 2006-07: coordinates and a congressional district arrive ahead of the status column.
    fixed_width(2006, 530, 81, 234, 309),
];

const fn fixed_width(
    opens: u16,
    record: usize,
    name_ends: usize,
    agency_type: usize,
    status: usize,
) -> FixedWidth {
    FixedWidth {
        opens,
        record,
        name_ends,
        agency_type,
        status,
    }
}

/// The layout for a year, if that year is one of the fixed-width ones.
fn fixed_width_of(opens: u16) -> Option<FixedWidth> {
    CCD_FIXED_WIDTH.into_iter().find(|w| w.opens == opens)
}

/// The Ohio rows of one fixed-width year: agency, state identifier, name, type, status.
///
/// Sliced by character rather than by byte. The archives are read Latin-1 byte-for-byte, so a
/// record holding one accented character is one byte longer in memory than it is on disk, and
/// byte offsets would walk off the columns from that record onward.
fn ccd_fixed_rows(layout: FixedWidth, text: &str, label: &str) -> Result<Vec<[String; 5]>, String> {
    let mut out = Vec::new();
    for (at, line) in text.lines().enumerate() {
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let chars: Vec<char> = line.chars().collect();
        if chars.len() != layout.record {
            return Err(format!(
                "{label} record {} is {} bytes and its layout says {}; the layout does not \
                 describe this file",
                at + 1,
                chars.len(),
                layout.record
            ));
        }
        if fixed(&chars, 0, 2) != OHIO_FIPS {
            continue;
        }
        out.push([
            fixed(&chars, 0, 7),
            fixed(&chars, 7, 21),
            fixed(&chars, 21, layout.name_ends),
            fixed(&chars, layout.agency_type - 1, layout.agency_type),
            fixed(&chars, layout.status - 1, layout.status),
        ]);
    }
    Ok(out)
}

/// Every Ohio agency in every published year of the directory this repository holds.
///
/// # Why more than one year of a directory is worth holding
///
/// One year answers "what is this agency's Ohio number". Thirty answer "when did this agency
/// exist", which is a different question and the one a panel spanning years actually asks.
/// [`crate::fixtures::F33_OHIO_PANEL_FIXTURE`] resolved every identifier through the 2022-23 file
/// alone, so an agency that closed before 2023 had no Ohio number at all — 124 of them in FY2012 —
/// and the module reading it described that count as the consolidation history. It is not. Of the
/// 689 agencies that leave this window, **616 are community schools**, 66 are service agencies,
/// and five are regular districts.
///
/// # Two eras of file, one set of rows
///
/// 2007-08 onward is delimited with a header and is read by the column names in `CCD_ALIASES`.
/// 1994-95 through 2006-07 is fixed-width with no header at all and is read by byte position
/// through a per-year byte-position table. The extension does not decide which: `ag031b.txt` is fixed-width
/// and `ag071b.txt` is delimited, and the year does.
///
/// # What the status column can and cannot say
///
/// The CCD vocabulary has eight operational-status codes and exactly one of them marks a
/// consolidation: code 5, *"significant change in geographic boundaries or instructional
/// responsibility"*. **Ohio has never used it** — zero occurrences in every Ohio agency-year this
/// reader covers, which is now every one the directory has published since 1994-95. What Ohio
/// files instead, for all 689 departures without exception, is code 2: *"closed with no effect on
/// another agency's boundaries"*.
///
/// That is not silence. It is the negation of the thing, filed about three districts whose
/// territory demonstrably went to a neighbour — and, thirty-nine more times in the 1990s, about
/// the county boards of education that became today's educational service centers. So the code is
/// carried verbatim and nothing here reads a reason out of it. See `dispersion::lea_directory`.
///
/// # Ohio is selected on `FIPST` and never on `LSTATE`
///
/// They disagree. LEAID 3901497, Urban Pathways of Youngstown, is filed under `FIPST=39` with
/// `LSTATE=PA` in 2012-13 and 2013-14 — a mailing address, not a jurisdiction. Earlier years have
/// the mirror-image defect, filing an Arizona agency as `LSTATE=OH`. The FIPS code is part of the
/// agency identifier's first two digits and cannot drift from it, and in the fixed-width years it
/// is the only way in: those files have no state column of their own.
///
/// # Errors
///
/// Returns a description if a year uses none of the names a field is known by, if a fixed-width
/// year holds a record of the wrong length, if a year's Ohio row count is outside the band the
/// survey has ever produced, or if an agency leaves the register without having been filed closed
/// — which is the check that settles the one year whose layout document is wrong.
pub fn build_ccd_directory(years: &[DirectoryYear<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for year in years {
        let label = format!(
            "the CCD LEA directory for {}-{:02}",
            year.opens,
            (year.opens + 1) % 100
        );
        let rows = match fixed_width_of(year.opens) {
            Some(layout) => ccd_fixed_rows(layout, year.text, &label)?,
            None => ccd_delimited_rows(year.text, &label)?,
        };

        let kept = rows.len();
        for row in rows {
            // The prefix appears in 2016-17 and the digits either side of it are the same six.
            // Stripped here so the panel joins to itself across the change, and so it joins to
            // every other fixture in this repository, which write the bare IRN.
            let irn = row[1].trim_start_matches("OH-");
            if irn.len() != 6 || !irn.chars().all(|c| c.is_ascii_digit()) {
                return Err(format!(
                    "{label} gives agency {} the state identifier {irn:?}, which is not a \
                     six-digit IRN",
                    row[0]
                ));
            }
            out.push(vec![
                year.opens.to_string(),
                row[0].clone(),
                irn.to_string(),
                clean_name(&row[2]),
                row[3].clone(),
                row[4].clone(),
            ]);
        }

        // Ohio has run between seven hundred and twelve hundred agencies in every year the
        // directory has published, the low end being 1996-97 before community schools existed and
        // the high end 2005-06 at the top of the charter opening wave. A count outside that band
        // is a state filter that matched the wrong column, which is the failure `LSTATE` would
        // have produced silently.
        if !(700..=1400).contains(&kept) {
            return Err(format!(
                "{label} yielded {kept} Ohio agencies, which is outside anything the directory \
                 has published"
            ));
        }
    }
    check_ccd_closures(&out)?;
    Ok(out)
}

/// The Ohio rows of one delimited year, by the names its header gives the columns.
fn ccd_delimited_rows(text: &str, label: &str) -> Result<Vec<[String; 5]>, String> {
    let mut lines = text.lines();
    let header_line = lines.next().unwrap_or_default();
    let delimiter = if header_line.contains('\t') {
        '\t'
    } else {
        ','
    };
    let head = delimited_fields(header_line, delimiter);

    let mut at = [0usize; CCD_ALIASES.len()];
    for (i, (what, names)) in CCD_ALIASES.iter().enumerate() {
        at[i] = names
            .iter()
            .find_map(|name| column(&head, name, label).ok())
            .ok_or_else(|| {
                format!("{label} names {what} none of {names:?}; its layout has moved")
            })?;
    }
    let fips = column(&head, "FIPST", label)?;

    let mut out = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let f = delimited_fields(line, delimiter);
        let field = |i: usize| f.get(i).map(|v| v.trim()).unwrap_or_default();
        if field(fips).trim_start_matches('0') != OHIO_FIPS {
            continue;
        }
        out.push([
            field(at[0]).to_string(),
            field(at[1]).to_string(),
            field(at[2]).to_string(),
            field(at[3]).to_string(),
            field(at[4]).to_string(),
        ]);
    }
    Ok(out)
}

/// Every agency that stops appearing was flagged closed in the last year that names it.
///
/// # Why this is the check that matters
///
/// The status column is one byte, its position moves five times, and reading it one byte off
/// yields a column of plausible small integers rather than an error. Nothing about a single
/// year's rows can catch that. What can is the column's relationship to the *next* year's
/// membership: code 2 means *"education agency has closed"*, so an agency carrying it should not
/// be in the following year's file, and one that vanishes without it would be an agency the
/// register lost track of rather than closed.
///
/// Across the twenty-nine transitions from 1994-95 to 2023-24 the containment holds exactly —
/// every departure is flagged, without a single exception in thirty years. The converse very
/// nearly holds too: two agencies are flagged closed and come back, both community schools, and
/// both are then filed under code 8, *"closed on previous year's file but has reopened"*. So the
/// vocabulary accounts for its own exceptions and this reader checks the direction that has none.
///
/// This is also what settles 2003-04's column against its own layout document. At the documented
/// position the check fails on that year's transition; at 284 it passes.
fn check_ccd_closures(rows: &[Vec<String>]) -> Result<(), String> {
    const CLOSED: &str = "2";
    let mut by_year: BTreeMap<u16, BTreeMap<&str, &str>> = BTreeMap::new();
    for row in rows {
        let Ok(opens) = row[0].parse::<u16>() else {
            continue;
        };
        by_year
            .entry(opens)
            .or_default()
            .insert(row[1].as_str(), row[5].as_str());
    }
    let years: Vec<u16> = by_year.keys().copied().collect();
    for pair in years.windows(2) {
        let (before, after) = (pair[0], pair[1]);
        // Only consecutive years can say anything about each other. A gap in what is held is not
        // a claim about the register.
        if after != before + 1 {
            continue;
        }
        let gone: Vec<&str> = by_year[&before]
            .keys()
            .filter(|leaid| !by_year[&after].contains_key(*leaid))
            .filter(|leaid| by_year[&before][*leaid] != CLOSED)
            .copied()
            .collect();
        if !gone.is_empty() {
            return Err(format!(
                "{} agencies are absent from {after}-{:02} without {before}-{:02} filing them \
                 closed, the first being {}; the status column is being read at the wrong \
                 position in one of those two years",
                gone.len(),
                (after + 1) % 100,
                (before + 1) % 100,
                gone[0]
            ));
        }
    }
    Ok(())
}

// -------------------------------------------------------------------------------------------
// MR-81 child nutrition
// -------------------------------------------------------------------------------------------

/// Where the MR-81 sponsor panel is written, relative to the repository root.
pub const MR81_FIXTURE: &str = "crates/dispersion/fixtures/mr81-sponsor-panel.csv";

/// Columns of the MR-81 sponsor panel.
///
/// `stream`, `identified` and `claimable` exist for the same reason `enrollment_basis` does: the
/// report stops being one report. From FY2012 it is published as three, and the three do not
/// count the same thing. See [`build_mr81`].
pub const MR81_HEADER: &[&str] = &[
    "fiscal_year",
    "sponsor_irn",
    "sponsor_name",
    "county",
    "sponsor_type",
    "stream",
    "sites",
    "enrollment",
    "enrollment_basis",
    "free_lunch",
    "reduced_lunch",
    "identified",
    "claimable",
];

/// Which of the report's publications a row came from.
///
/// Not cosmetic. Each stream reports a different quantity in the same columns, and a consumer
/// that adds them without reading this gets a number that looks like a poverty rate and is not
/// one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stream {
    /// FY1998 through FY2011, when the report was one file covering every sponsor.
    Single,
    /// From FY2012: sponsors still collecting meal applications. The header says outright that it
    /// *"Excludes Provision 2 and Community Eligibility Option (CEO) sponsors."*
    Traditional,
    /// From FY2012: sponsors serving under Provision 2, whose approvals are **frozen at a base
    /// year**. `ProvisionYear` names it, and the same sponsor reports the same free and reduced
    /// counts in FY2012, FY2013 and FY2014 while its enrolment moves underneath them.
    Provision2,
    /// From FY2012: community eligibility. **No applications are collected at all**, so free and
    /// reduced are structurally zero and the comparable quantity is `identified`.
    Community,
}

impl Stream {
    /// The value written to the fixture.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Traditional => "traditional",
            Self::Provision2 => "provision2",
            Self::Community => "community",
        }
    }
}

/// How a year's file is laid out, which decides which reader runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mr81Layout {
    /// FY1998 through FY2000: one row per **school**, 170 characters wide, with the district
    /// named and numbered on every row.
    ///
    /// Nominally comma-separated and actually fixed-width, which is the whole reason it is read
    /// by offset. Seven district names a year carry a comma — `Graham School, The`,
    /// `Holy Trinity, Swanton` — and splitting on it shifts those rows past the last column. The
    /// separators sit at the same thirteen offsets in all 4,246 rows of all three files, so
    /// slicing never meets the problem.
    SchoolCentric,
    /// FY2001 through FY2014: one row per school **site**, with its sponsor named on the row.
    Delimited,
    /// The printed report, for the one stream-year posted no other way.
    ///
    /// Sponsor blocks with a repeated column heading, a `SPONSOR TOTAL` after each and a `STATE
    /// TOTAL` at the end. Fixed-width as well, and for the same reason — five site names a year
    /// run flush into the IRN beside them with no space between.
    Printed,
}

/// One published MR-81 file: which October, which stream, which layout, and its text.
#[derive(Debug, Clone, Copy)]
pub struct Mr81Report<'a> {
    /// The October the report counts.
    pub year: u16,
    /// Which of the three publications this is.
    pub stream: Stream,
    /// Which reader this file needs.
    pub layout: Mr81Layout,
    /// The file's text.
    pub text: &'a str,
}

/// One sponsor's running totals while its school sites are being summed.
#[derive(Debug, Default)]
struct SponsorTotal {
    name: String,
    county: String,
    kind: String,
    sites: usize,
    enrollment: i64,
    free: i64,
    reduced: i64,
    identified: i64,
    claimable: i64,
}

/// One school site, whichever layout it was read from.
struct Site {
    sponsor_irn: String,
    sponsor_name: String,
    county: String,
    kind: String,
    enrollment: i64,
    free: i64,
    reduced: i64,
    identified: i64,
    /// The percentage the report itself prints in its free-and-reduced-of-enrolment column.
    reported_share: f64,
}

/// Sponsor number and type by name, for the printed report that carries neither.
#[derive(Debug, Default)]
struct Identity {
    /// Keyed on name and county together, which is how all but one sponsor resolves.
    by_place: BTreeMap<(String, String), (String, String)>,
    /// Keyed on name alone, `None` where the name is not unique.
    by_name: BTreeMap<String, Option<(String, String)>>,
}

impl Identity {
    /// Both sides of the lookup pass through here, because they do not arrive alike: names taken
    /// from a delimited file have had their commas substituted for the fixture's sake and names
    /// read off the printed report have not. `Virtual Schoolhouse, Inc.` is one of them.
    fn key(raw: &str) -> String {
        without_comma(raw).to_lowercase()
    }

    fn insert(&mut self, name: &str, county: &str, irn: &str, kind: &str) {
        let value = (irn.to_string(), kind.to_string());
        self.by_place
            .insert((Self::key(name), Self::key(county)), value.clone());
        match self.by_name.entry(Self::key(name)) {
            std::collections::btree_map::Entry::Vacant(slot) => {
                slot.insert(Some(value));
            }
            std::collections::btree_map::Entry::Occupied(mut slot) => {
                if slot.get().as_ref().is_some_and(|held| held.0 != value.0) {
                    slot.insert(None);
                }
            }
        }
    }

    /// The county is tried first and the name alone second.
    ///
    /// The fallback is not tidiness. `Believe to Achieve-Canton` is filed under Cuyahoga in the
    /// FY2012 file and under Stark in the FY2013 one — the publisher moved it, and on a
    /// name-and-county key it would be a sponsor this repository had never seen.
    fn get(&self, name: &str, county: &str) -> Option<&(String, String)> {
        self.by_place
            .get(&(Self::key(name), Self::key(county)))
            .or_else(|| self.by_name.get(&Self::key(name))?.as_ref())
    }
}

/// The sponsor types the report uses, which is also how a shifted row is put back together.
const SPONSOR_TYPES: [&str; 4] = [
    "Public",
    "Non-Public",
    "Residential Child Care Institution",
    "Camp",
];

/// Sponsors across every October of MR-81 this repository holds, aggregated from school sites.
///
/// # What this is, and what the catalog said it was
///
/// MR-81 is the Office for Child Nutrition's free and reduced-price lunch report, one row per
/// school site grouped by sponsor. It is not an enrollment archive; it carries an enrolment
/// column because a lunch claim needs a denominator. Its value here is the **free and
/// reduced-price counts**, which are the closest available long series of the measure Ohio's
/// disadvantaged pupil funding is paid on — R.C. 3317.03(B)(21) hands the definition of
/// "economically disadvantaged" to the department, and free-lunch eligibility has been the
/// department's operative test.
///
/// # Three breaks, each carried on the row rather than annotated
///
/// **The denominator is renamed in FY2010.** Through FY2009 the column is `AdmCount`. From
/// FY2010 it is `CECount`, and the report's own header defines CE as the *"highest daily number
/// of students with access to the program"* — not average daily membership and not the same
/// quantity. `enrollment_basis` says which.
///
/// **The report splits into three in FY2012**, and the three count differently. Traditional is
/// current-year applications. Provision 2 is applications **frozen at a base year** — the same
/// sponsor reports the same free and reduced counts three years running while its enrolment moves
/// underneath them. Community eligibility collects **no applications at all**: its free and
/// reduced columns are zero by construction, and what it publishes instead is
/// `CEOEligibleStudents`, the directly-certified count. `stream` says which, and a consumer that
/// sums the three without reading it produces a poverty rate that falls because the poorest
/// sponsors stopped filling in forms.
///
/// `identified` and `claimable` bound what the community stream can be read as. `identified` is
/// the published directly-certified count — a floor, because direct certification reaches SNAP,
/// TANF, foster and homeless children and nobody else. `claimable` is that count run through
/// USDA's 1.6 multiplier and capped at enrolment, which is the ceiling the programme itself uses
/// and which reproduces the report's own printed percentage in all 735 of the FY2014 rows. For
/// the two application streams both equal the approvals, so a consumer can carry either bound
/// across the whole panel without a special case.
///
/// **The grain changes in FY2001.** FY1998-FY2000 are one row per school with the district on it;
/// everything later is one row per site with the sponsor on it. Both aggregate to the same shape,
/// but a district is not a sponsor: the earlier files have no sponsor-type column at all, so
/// their rows are written `Unknown` rather than guessed at.
///
/// # Sponsors are not districts
///
/// `SponsorType` is carried rather than filtered on. "Public" includes county boards of
/// developmental disabilities and community schools alongside traditional districts, and the
/// report also covers non-public schools, residential child care institutions and camps.
/// Deciding which sponsors are districts needs a join this function does not have; emitting the
/// type lets the consumer draw the line and lets a test count what was drawn.
///
/// # Every row is checked against the percentage printed beside it
///
/// The FY2001 file is the only comma-delimited one, and nine of its rows carry a comma inside a
/// school name. Split positionally, those rows put a **site IRN into the enrolment column** —
/// `00026450` and `00093153` for two Cleveland City schools, adding 119,603 students to a
/// district of 73,562 and understating the statewide FY2001 poverty share by 1.8 points. The
/// figure was plausible, the row count was right, and the panel shipped that way.
///
/// So the columns are checked against each other: the report prints free-and-reduced as a share
/// of enrolment beside the counts, and a shifted row fails that arithmetic by orders of
/// magnitude. Rows reporting **no applications at all** are exempt, because that is the
/// community-eligibility signature rather than a parse failure — and the exemption is itself
/// informative, since those are precisely the rows whose printed percentage is a claiming rate.
///
/// # Errors
///
/// Returns the missing column's name if any year's layout has moved, or a description of the
/// arithmetic that failed.
pub fn build_mr81(reports: &[Mr81Report<'_>]) -> Result<Vec<Vec<String>>, String> {
    // The printed report names its sponsors and neither numbers nor types them. Built in full
    // before anything is read, so the lookup does not depend on the order reports arrive in.
    let mut identity = Identity::default();
    for report in reports {
        if report.layout != Mr81Layout::Delimited {
            continue;
        }
        let label = format!("the MR-81 report for October {}", report.year);
        for site in delimited_sites(report.text, &label)?.0 {
            identity.insert(
                &site.sponsor_name,
                &site.county,
                &site.sponsor_irn,
                &site.kind,
            );
        }
    }

    // Sponsor totals, keyed so the output is stable without a sort: year, stream, then IRN.
    let mut totals: BTreeMap<(u16, &'static str, String), SponsorTotal> = BTreeMap::new();
    let mut basis: BTreeMap<u16, &'static str> = BTreeMap::new();

    for report in reports {
        let label = format!(
            "the MR-81 {} report for October {}",
            report.stream.label(),
            report.year
        );
        let (sites, which) = match report.layout {
            Mr81Layout::SchoolCentric => (school_centric_sites(report.text, &label)?, "adm"),
            Mr81Layout::Delimited => delimited_sites(report.text, &label)?,
            // The one printed file is FY2013's, which is well inside the CE era. Stated rather
            // than sniffed because the printed heading spells CE out in prose and not in a
            // column name.
            Mr81Layout::Printed => (printed_sites(report.text, &identity, &label)?, "ce"),
        };
        // Every stream of one October shares a denominator, and they agree.
        if let Some(prior) = basis.insert(report.year, which) {
            if prior != which {
                return Err(format!(
                    "October {} is published on both {prior} and {which}",
                    report.year
                ));
            }
        }

        // The traditional and single-stream files have never carried fewer than three thousand
        // sites; the other two are small by construction and only have to be non-empty.
        let floor = match report.stream {
            Stream::Single | Stream::Traditional => 2000,
            Stream::Provision2 | Stream::Community => 1,
        };
        if sites.len() < floor {
            return Err(format!(
                "{label} yielded {} sites against a floor of {floor}, so the delimiter or the \
                 layout is wrong",
                sites.len()
            ));
        }

        for site in sites {
            check_reported_share(&site, &label)?;
            let entry = totals
                .entry((report.year, report.stream.label(), site.sponsor_irn))
                .or_insert_with(|| SponsorTotal {
                    name: site.sponsor_name,
                    county: site.county,
                    kind: site.kind,
                    ..SponsorTotal::default()
                });
            entry.sites += 1;
            entry.enrollment += site.enrollment;
            entry.free += site.free;
            entry.reduced += site.reduced;
            entry.identified += site.identified;
            // Capped at enrolment site by site, because that is where the programme caps it.
            // Rolled up first, the cap would let one site's slack forgive another's overshoot.
            entry.claimable += if report.stream == Stream::Community {
                (site.identified * 8 / 5).min(site.enrollment)
            } else {
                site.free + site.reduced
            };
        }
    }

    Ok(totals
        .into_iter()
        .map(|((year, stream, irn), t)| {
            vec![
                year.to_string(),
                irn,
                t.name,
                t.county,
                t.kind,
                stream.to_string(),
                t.sites.to_string(),
                t.enrollment.to_string(),
                (*basis.get(&year).unwrap_or(&"adm")).to_string(),
                t.free.to_string(),
                t.reduced.to_string(),
                t.identified.to_string(),
                t.claimable.to_string(),
            ]
        })
        .collect())
}

/// Zero-padded to six, because the years disagree: FY2004 writes `00043786` and FY2005 writes
/// `43786` for Cleveland. Joining the panel to itself across years — or to anything else in this
/// repository, which uses the padded form — silently matches nothing for whichever half is
/// written the other way.
fn sponsor_key(raw: &str) -> Option<String> {
    let trimmed = raw.trim().trim_start_matches('0');
    (!trimmed.is_empty()).then(|| format!("{trimmed:0>6}"))
}

/// `write_csv` does not quote and every consumer splits on the comma, so a sponsor called "Edge
/// Academy, The" would shift its own row's remaining columns. Ninety-nine rows were written that
/// way before the writer learned to refuse them. Substituted rather than dropped, because the
/// name is how a reader recognises the sponsor and the IRN beside it is the key anything joins on.
fn without_comma(raw: &str) -> String {
    raw.trim().replace(',', ";")
}

/// The characters of a fixed-width row from `start` up to `end`, by position and not by byte.
///
/// The reports are read Latin-1 byte for byte and re-encoded, so a row carrying a high byte is
/// longer in bytes than in the columns the printer laid out. One line of the FY2013 community
/// report is, and a byte-wise slice would take its site IRN from the wrong place.
fn fixed(line: &[char], start: usize, end: usize) -> String {
    line.get(start..end.min(line.len()))
        .unwrap_or_default()
        .iter()
        .collect::<String>()
        .trim()
        .to_string()
}

/// The printed percentage has to reproduce from the counts beside it, or the row is misread.
fn check_reported_share(site: &Site, label: &str) -> Result<(), String> {
    // No applications at all is the community-eligibility signature, not a shifted row: those
    // files publish a claiming rate in this column and nothing to reproduce it from.
    if site.enrollment <= 0 || (site.free == 0 && site.reduced == 0) {
        return Ok(());
    }
    let computed = 100.0 * (site.free + site.reduced) as f64 / site.enrollment as f64;
    if (computed - site.reported_share).abs() > 0.02 {
        return Err(format!(
            "in {label}, a site of {} in {} reports {} and {} approvals against {} enrolled, \
             which is {computed:.2}% and not the {:.2}% printed beside it — the columns have \
             shifted",
            site.sponsor_name,
            site.county,
            site.free,
            site.reduced,
            site.enrollment,
            site.reported_share,
        ));
    }
    Ok(())
}

/// FY2001-FY2014, one row per site, and which denominator the file is on.
fn delimited_sites(text: &str, label: &str) -> Result<(Vec<Site>, &'static str), String> {
    let mut lines = text.lines();
    let header_line = lines.next().unwrap_or_default();
    // FY2001 is comma-delimited and every later year is tab. Sniffed rather than tabulated,
    // because the delimiter is visible in the file and a table is another thing to maintain.
    let delimiter = if header_line.contains('\t') {
        '\t'
    } else {
        ','
    };
    // The FY2012 community file's header carries an empty column before `CEOEligibleStudents`
    // that its data rows do not, so the header is one wider than every row beneath it. Dropped
    // here, because a name resolved against it would point one past the end of the data.
    let head: Vec<String> = delimited_fields(header_line, delimiter)
        .into_iter()
        .filter(|h| !h.trim().is_empty())
        .collect();
    let at = |name: &str| column(&head, name, label);

    let (county, irn, name, kind) = (
        at("County")?,
        at("SponsorIRN")?,
        at("SponsorName")?,
        at("SponsorType")?,
    );
    let free = at("FreeLunchApps")?;
    let reduced = at("RedLunchApps")?;
    let (enrolment, which, share) = match at("AdmCount") {
        Ok(i) => (i, "adm", at("PctFreeRedAdm")?),
        Err(_) => (at("CECount")?, "ce", at("PctFreeRedCE")?),
    };
    // Only the community files carry it, so its absence is a fact about the file rather than an
    // error: a stream that still collects applications has no directly-certified column because
    // it does not run on one.
    let identified = at("CEOEligibleStudents").ok();

    let mut sites = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let f = rejoin(delimited_fields(line, delimiter), head.len());
        let field = |i: usize| f.get(i).map(|v| v.trim()).unwrap_or_default();
        let number = |i: usize| field(i).parse::<i64>().unwrap_or(0);
        let Some(key) = sponsor_key(field(irn)) else {
            continue;
        };
        sites.push(Site {
            sponsor_irn: key,
            sponsor_name: without_comma(field(name)),
            county: without_comma(field(county)),
            kind: without_comma(field(kind)),
            enrollment: number(enrolment),
            free: number(free),
            reduced: number(reduced),
            identified: identified.map_or(0, number),
            reported_share: field(share).parse::<f64>().unwrap_or(0.0),
        });
    }
    Ok((sites, which))
}

/// Put a row back together when an unquoted name has split across the delimiter.
///
/// Nine FY2001 rows do this and nothing else in the panel does. The repair anchors from the
/// right, where the shape is fixed — site IRN, kitchen type, base IRN and six figures close every
/// row — and finds the sponsor type in what is left, since it comes from a four-word vocabulary
/// no school name collides with. What precedes the type is the sponsor's name and what follows it
/// is the breakfast flag and the school's name, each rejoined with the comma that split it.
///
/// A row this cannot read is returned untouched, so it fails the arithmetic check rather than
/// being quietly reshaped into something plausible.
fn rejoin(fields: Vec<String>, expected: usize) -> Vec<String> {
    /// Site IRN, kitchen type, base IRN, and the six figures that close every row.
    const TAIL: usize = 9;
    if fields.len() <= expected || expected < TAIL + 2 || fields.len() <= TAIL + 3 {
        return fields;
    }
    let middle = &fields[2..fields.len() - TAIL];
    let mut found = middle
        .iter()
        .enumerate()
        .filter(|(_, v)| SPONSOR_TYPES.contains(&v.trim()));
    let Some((at, _)) = found.next() else {
        return fields;
    };
    // Two candidates means the vocabulary matched something that is not the type column, and
    // guessing between them is exactly what this is meant not to do.
    if found.next().is_some() || at + 2 > middle.len() {
        return fields;
    }
    let mut out = fields[..2].to_vec();
    out.push(middle[..at].join(","));
    out.push(middle[at].clone());
    // Between the type and the school name sits the one-character breakfast flag; everything
    // after it belongs to the name, however many commas it was written with.
    out.push(middle[at + 1].clone());
    out.push(middle[at + 2..].join(","));
    out.extend_from_slice(&fields[fields.len() - TAIL..]);
    if out.len() == expected {
        out
    } else {
        fields
    }
}

/// FY1998-FY2000, one fixed-width row per school with its district named on it.
///
/// The header names fourteen columns and the rows are 170 characters with the separators at the
/// same offsets throughout, so the fields are taken by offset. `% REDU` is misnamed in that
/// header: it holds free **and** reduced as a share of ADM, which every row's arithmetic confirms
/// and which is what the later files call `PctFreeRedAdm`.
///
/// The three files were regenerated together in 2004 from one roster — all three carry the same
/// 4,246 schools, with the ones not participating in a given October written as zeros. Sites are
/// therefore counted only where the school reported an enrolment, or a district's site count
/// would include schools that were not open.
fn school_centric_sites(text: &str, label: &str) -> Result<Vec<Site>, String> {
    /// Start and end of each field, from the separator offsets shared by every row.
    const CUTS: [(usize, usize); 14] = [
        (0, 1),
        (2, 32),
        (33, 39),
        (40, 50),
        (51, 57),
        (58, 67),
        (68, 77),
        (78, 87),
        (88, 97),
        (98, 107),
        (108, 117),
        (118, 152),
        (153, 163),
        (164, 170),
    ];
    const WIDTH: usize = 170;
    const ADM: usize = 5;
    const FREE: usize = 6;
    const REDUCED: usize = 7;
    const SHARE: usize = 10;
    const DISTRICT: usize = 11;
    const COUNTY: usize = 12;
    const DISTRICT_IRN: usize = 13;

    let mut lines = text.lines();
    let header = lines.next().unwrap_or_default();
    for name in ["SCHOOL IRN", "ADM", "FREE", "REDUCED", "DISTRICT IRN"] {
        if !header.contains(name) {
            return Err(format!(
                "{label} has no {name} column; its layout has moved"
            ));
        }
    }

    let mut sites = Vec::new();
    for line in lines {
        let row: Vec<char> = line.chars().collect();
        if row.len() != WIDTH {
            continue;
        }
        let at = |i: usize| fixed(&row, CUTS[i].0, CUTS[i].1);
        let number = |i: usize| at(i).parse::<i64>().unwrap_or(0);
        let Some(key) = sponsor_key(&at(DISTRICT_IRN)) else {
            continue;
        };
        if number(ADM) <= 0 {
            continue;
        }
        sites.push(Site {
            sponsor_irn: key,
            sponsor_name: without_comma(at(DISTRICT).trim_end_matches('.')),
            county: without_comma(&at(COUNTY)),
            // These files have no sponsor type. Resolved from FY2001 downstream, where the same
            // district carries the same number; left visible rather than guessed at here.
            kind: "Unknown".to_string(),
            enrollment: number(ADM),
            free: number(FREE),
            reduced: number(REDUCED),
            identified: 0,
            reported_share: at(SHARE).parse::<f64>().unwrap_or(0.0),
        });
    }
    if sites.is_empty() {
        return Err(format!("{label} yielded no rows {WIDTH} characters wide"));
    }
    Ok(sites)
}

/// The printed report, for the one stream-year published no other way.
///
/// FY2013's community file is posted rendered and not delimited, so this reads the rendering.
/// Both the per-sponsor and the statewide totals it prints are recomputed against what was
/// parsed, so a row this misses is an error rather than a quietly smaller number — which is how
/// the two sites whose IRN is printed seven digits wide instead of eight were found.
///
/// The sponsor is named and neither numbered nor typed, so [`Identity`] supplies both from the
/// delimited files of the years either side. A sponsor it cannot name is an error: an unnumbered
/// row cannot be joined to anything and would sit in the panel as a sponsor of its own.
fn printed_sites(text: &str, identity: &Identity, label: &str) -> Result<Vec<Site>, String> {
    /// The site IRN column, right-aligned within it.
    const IRN: (usize, usize) = (39, 47);

    let mut sites: Vec<Site> = Vec::new();
    let (mut name, mut county) = (String::new(), String::new());
    let mut running = [0i64; 3];
    let mut counted = 0usize;
    let mut state: Option<[i64; 3]> = None;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("SPONSOR: ") {
            name = rest.trim().to_string();
            running = [0; 3];
            counted = 0;
            continue;
        }
        if let Some(rest) = line.strip_prefix("COUNTY:") {
            county = rest.trim().to_string();
            continue;
        }
        if let Some(rest) = line.strip_prefix("SPONSOR TOTAL") {
            let stated = printed_totals(rest);
            if stated != running {
                return Err(format!(
                    "in {label}, {name} of {county} sums to {running:?} across {counted} sites \
                     against the {stated:?} it prints"
                ));
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("STATE TOTAL") {
            state = Some(printed_totals(rest));
            continue;
        }
        let row: Vec<char> = line.chars().collect();
        let site_irn = fixed(&row, IRN.0, IRN.1);
        if site_irn.is_empty() || !site_irn.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let figures: Vec<String> = line
            .chars()
            .skip(IRN.1)
            .collect::<String>()
            .split_whitespace()
            .map(str::to_string)
            .collect();
        // Enrolment, free, reduced, their total, two percentages, and the identified count.
        if figures.len() < 7 {
            continue;
        }
        let number = |i: usize| figures[i].parse::<i64>().unwrap_or(0);
        let Some((irn, kind)) = identity.get(&name, &county) else {
            return Err(format!(
                "in {label}, {name} of {county} is named by no delimited file, so it has no IRN"
            ));
        };
        running[0] += number(0);
        running[1] += number(1);
        running[2] += number(2);
        counted += 1;
        sites.push(Site {
            sponsor_irn: irn.clone(),
            sponsor_name: without_comma(&name),
            county: without_comma(&county),
            kind: kind.clone(),
            enrollment: number(0),
            free: number(1),
            reduced: number(2),
            identified: number(6),
            reported_share: figures[5].parse::<f64>().unwrap_or(0.0),
        });
    }

    let Some(stated) = state else {
        return Err(format!("{label} prints no state total to check against"));
    };
    let summed = [
        sites.iter().map(|s| s.enrollment).sum::<i64>(),
        sites.iter().map(|s| s.free).sum::<i64>(),
        sites.iter().map(|s| s.reduced).sum::<i64>(),
    ];
    if summed != stated {
        return Err(format!(
            "{label} sums to {summed:?} across {} sites against the {stated:?} it prints",
            sites.len()
        ));
    }
    Ok(sites)
}

/// Enrolment, free and reduced off a printed total line.
fn printed_totals(rest: &str) -> [i64; 3] {
    let figures: Vec<i64> = rest
        .split_whitespace()
        .map(|f| f.parse::<i64>().unwrap_or(0))
        .collect();
    [
        figures.first().copied().unwrap_or(0),
        figures.get(1).copied().unwrap_or(0),
        figures.get(2).copied().unwrap_or(0),
    ]
}

// -------------------------------------------------------------------------------------------
// Report card, building grain
// -------------------------------------------------------------------------------------------

/// Where the building-level report card extract is written, relative to the repository root.
pub const BUILDING_FIXTURE: &str = "crates/dispersion/fixtures/report-card-2425-buildings.csv";

/// Columns of the building fixture.
///
/// **There is no overall rating here, and its absence is the point.** R.C. 3302.10 triggers an
/// academic distress commission on three consecutive years of an *overall* grade or star rating,
/// and ESSA rank-orders the same overall rating to identify CSI schools. Neither published
/// building file carries it: `Achievement_Building` gives the **achievement component** star
/// rating, and `Building_Details` gives no rating at all. What is held is therefore the input to
/// the accountability trigger rather than the trigger itself, and a node must not present the
/// achievement star as though it were the overall one.
pub const BUILDING_HEADER: &[&str] = &[
    "building_irn",
    "building_name",
    "district_irn",
    "district_name",
    "county",
    "enrollment",
    "chronic_absenteeism",
    "achievement_star_rating",
    "performance_index_2425",
    "performance_index_2324",
    "performance_index_2223",
];

/// The building-level report card extract, from the achievement and details workbooks.
///
/// # Why the building grain is held at all
///
/// Every other fixture here is agency-level, because the funding formula pays agencies. The
/// accountability system does not: ESSA identifies **schools**, and R.C. 3302.12 attaches its
/// intervention to a **building**. Rolling either up to the district would record the consequence
/// and discard the unit that triggers it.
///
/// # Three years of Performance Index, on purpose
///
/// The achievement workbook publishes 2024-25 beside 2023-24 and 2022-23 in the same row. The
/// distress trigger and the CSI escalation both read three consecutive years, so a single-year
/// extract could not be checked against either. Carried as three columns rather than three rows
/// because that is how the source publishes them and reshaping would invent a panel the file does
/// not contain.
///
/// # Chronic absenteeism is a CSI criterion, not colour
///
/// For schools with fewer than three rated report card components, Ohio identifies the lowest 5%
/// **by chronic absenteeism**. It comes from the details workbook's `All Students` rows, which are
/// long-form — one row per building per student group — so everything but `All Students` is
/// dropped here rather than aggregated.
///
/// # Errors
///
/// Returns the missing column's name if either workbook's layout has moved.
pub fn build_buildings(
    achievement: &[Vec<String>],
    details: &[Vec<String>],
) -> Result<Vec<Vec<String>>, String> {
    const ACH: &str = "the building achievement workbook";
    const DET: &str = "the building details workbook";

    let head = |rows: &[Vec<String>]| rows.first().cloned().unwrap_or_default();
    let ach_head = head(achievement);
    let det_head = head(details);
    let at = |h: &[String], name: &str, file: &str| column(h, name, file);

    let (b_irn, b_name, d_irn, d_name, county) = (
        at(&ach_head, "Building IRN", ACH)?,
        at(&ach_head, "Building Name", ACH)?,
        at(&ach_head, "District IRN", ACH)?,
        at(&ach_head, "District Name", ACH)?,
        at(&ach_head, "County", ACH)?,
    );
    let star = at(&ach_head, "Achievement Component Star Rating", ACH)?;
    let pi = [
        at(&ach_head, "Performance Index Score 2024-2025", ACH)?,
        at(&ach_head, "Performance Index Score 2023-2024", ACH)?,
        at(&ach_head, "Performance Index Score 2022-2023", ACH)?,
    ];

    let (d_b_irn, d_group, d_enrol, d_absent) = (
        at(&det_head, "Building IRN", DET)?,
        at(&det_head, "Student Group", DET)?,
        at(&det_head, "Enrollment", DET)?,
        at(&det_head, "Chronic Absenteeism Rate", DET)?,
    );

    let mut detail: BTreeMap<String, (String, String)> = BTreeMap::new();
    for row in details.iter().skip(1) {
        let get = |i: usize| row.get(i).map(|v| v.trim()).unwrap_or_default();
        if get(d_group) != "All Students" {
            continue;
        }
        detail.insert(
            get(d_b_irn).to_string(),
            (get(d_enrol).to_string(), get(d_absent).to_string()),
        );
    }

    let mut out = Vec::new();
    for row in achievement.iter().skip(1) {
        let get = |i: usize| row.get(i).map(|v| v.trim()).unwrap_or_default();
        let irn = get(b_irn).to_string();
        if irn.is_empty() {
            continue;
        }
        let (enrolment, absent) = detail.get(&irn).cloned().unwrap_or_default();
        // "4  Stars" as published, with the doubled space. Kept as the number alone so a consumer
        // does not have to know that, and blank where the building is unrated.
        let stars = get(star)
            .split_whitespace()
            .next()
            .filter(|v| v.parse::<f64>().is_ok())
            .unwrap_or_default()
            .to_string();
        out.push(vec![
            irn,
            get(b_name).replace(',', ";"),
            get(d_irn).to_string(),
            get(d_name).replace(',', ";"),
            get(county).replace(',', ";"),
            enrolment,
            absent,
            stars,
            get(pi[0]).to_string(),
            get(pi[1]).to_string(),
            get(pi[2]).to_string(),
        ]);
    }
    out.sort();
    Ok(out)
}

/// Where the identified-schools extract is written, relative to the repository root.
pub const IDENTIFIED_FIXTURE: &str = "crates/dispersion/fixtures/identified-schools-2026.csv";

/// Columns of the identified-schools extract.
pub const IDENTIFIED_HEADER: &[&str] = &[
    "status",
    "building_irn",
    "building_name",
    "lea_irn",
    "lea_name",
    "school_type",
    "subgroups",
    "year_identified",
    "open_closed",
    "cycle",
    "school_year",
];

/// The subgroup columns TSI and ATSI mark. CSI has none — it is a whole-school identification.
const IDENTIFIED_SUBGROUPS: &[&str] = &[
    "Asian or Pacific Islander",
    "Black, Non-Hispanic",
    "Economic Disadvantage",
    "English Learner",
    "Hispanic",
    "Multiracial",
    "Students with Disabilities",
    "White, Non-Hispanic",
];

/// Who is actually in the accountability system, from the three published lists.
///
/// # Why one fixture for three files
///
/// CSI, TSI and ATSI are three tiers of one system and a reader almost always wants them
/// together — a building's status is the union of what it appears on, and the interesting rows
/// are those on more than one list. Keeping them apart would mean three joins to ask one
/// question.
///
/// The files do not share a layout. CSI carries `Year Identified`, `Previous Cycle Status` and an
/// open/closed flag and no subgroups, because it is a whole-school identification. TSI and ATSI
/// carry eight subgroup columns and no year, because they identify a school *through* a subgroup.
/// The union is taken with the missing fields blank rather than by inventing a common shape.
///
/// # `subgroups` is joined with a semicolon on purpose
///
/// [`write_csv`] refuses a field containing a comma, and "Black, Non-Hispanic" contains one. The
/// separator is a semicolon and the subgroup names keep their own commas replaced, so a consumer
/// splits on `;` and gets the department's labels back unaltered apart from that substitution.
///
/// # Errors
///
/// Returns the missing column's name if a list's layout has moved.
pub fn build_identified(lists: &[(&str, &[Vec<String>])]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for (status, rows) in lists {
        let label = format!("the {} identified-schools list", status.to_uppercase());
        let head = rows.first().cloned().unwrap_or_default();
        let at = |name: &str| column(&head, name, &label);
        let (lea_irn, lea_name, b_irn, b_name, kind) = (
            at("LEA IRN")?,
            at("LEA Name")?,
            at("Building IRN")?,
            at("Building Name")?,
            at("School Type")?,
        );
        // Present on CSI only; TSI and ATSI identify through a subgroup and carry neither.
        let year = at("Year Identified").ok();
        let open = at("Open/Closed Status").ok();
        let cycle = at("Identification Year Cycle").ok();
        let school_year = at("School Year").ok();
        let subgroups: Vec<(usize, &str)> = IDENTIFIED_SUBGROUPS
            .iter()
            .filter_map(|name| at(name).ok().map(|i| (i, *name)))
            .collect();

        for row in rows.iter().skip(1) {
            let get = |i: usize| row.get(i).map(|v| v.trim()).unwrap_or_default();
            // `Year Identified` is not always a year. It carries an escalation history —
            // "ATSI 2022 , CSI 2025" — for a building that moved between tiers, which is the
            // three-year ATSI-to-CSI path written into a cell. The comma is substituted for the
            // same reason the names are: this writer refuses one, and it caught this on the first
            // build.
            let opt =
                |i: Option<usize>| i.map(get).unwrap_or_default().replace(',', ";").to_string();
            if get(b_irn).is_empty() {
                continue;
            }
            let marked: Vec<String> = subgroups
                .iter()
                .filter(|(i, _)| !get(*i).is_empty())
                .map(|(_, name)| name.replace(',', ""))
                .collect();
            out.push(vec![
                (*status).to_string(),
                get(b_irn).to_string(),
                get(b_name).replace(',', ";"),
                get(lea_irn).to_string(),
                get(lea_name).replace(',', ";"),
                get(kind).replace(',', ";"),
                marked.join(";"),
                opt(year),
                opt(open),
                opt(cycle),
                opt(school_year),
            ]);
        }
    }
    out.sort();
    Ok(out)
}

/// The appropriation-line series: every Department of Education line item, by fiscal year.
pub const APPROPRIATION_FIXTURE: &str = "crates/project/fixtures/appropriation-lines.csv";

/// Columns of the appropriation series.
pub const APPROPRIATION_HEADER: &[&str] = &[
    "general_assembly",
    "bill",
    "fiscal_year",
    "kind",
    "source",
    "documents",
    "fund_group",
    "fund",
    "line_item",
    "title",
    "amount",
];

/// One budget workbook, with the biennium it appropriates for.
#[derive(Debug, Clone, Copy)]
pub struct AppropriationBook<'a> {
    /// The General Assembly that enacted it.
    pub general_assembly: u16,
    /// The bill, as the registry keys it — `hb96`.
    pub bill: &'a str,
    /// The registry key of the document, which is the provenance of every figure from it.
    pub source: &'a str,
    /// Which of the two documents this is: `enacted` or `actuals`.
    ///
    /// It decides what an unlabelled column means, and nothing else does. The `actuals` workbook
    /// heads its prior years and its closed biennium year alike with a bare `FY 2014`, and marks
    /// only the year still open as `Adj. Appr.`; the `enacted` one labels every biennium column
    /// with the stage it belongs to and leaves only genuine prior years bare.
    pub variant: &'a str,
    /// The first fiscal year of the biennium this act appropriates for.
    pub first_year: u16,
    /// The workbook's first sheet, as rows.
    pub rows: &'a [Vec<String>],
}

/// What a column of amounts is a claim about, or why it is not one.
///
/// # Why this is a whitelist and not a set of exclusions
///
/// The first attempt classified a column by looking for `Actual` or `Approp` in its label and
/// treating everything else as an amount for the year it named. That silently swept in `$ Change`
/// and `% Change` — so `200550 Foundation Funding` in FY2019 arrived three times, as
/// $6,970,372,221.42, $167,292,415.57 and $0.02, with nothing to say which was the appropriation.
///
/// The `as enacted` workbooks make the point sharper still: they carry the bill's entire path,
/// with an amount per fiscal year for *introduced*, each chamber's substitute, each committee
/// report, conference, and finally as enacted. Every one of those is a real dollar figure for a
/// real fiscal year, and all but the last are proposals that never became law. A classifier that
/// recognises what it wants and refuses the rest is the only shape that survives this source.
fn amount_kind(label: &str) -> Option<&'static str> {
    // Collapse the newlines LSC wraps its headers on, and the year, leaving the kind.
    let stripped: String = label
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let mut without_year = String::new();
    let chars: Vec<char> = stripped.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        let is_year = index + 4 <= chars.len()
            && chars[index..index + 4].iter().all(char::is_ascii_digit)
            && matches!(
                chars[index..index + 2].iter().collect::<String>().as_str(),
                "19" | "20"
            );
        if is_year {
            index += 4;
        } else {
            without_year.push(chars[index]);
            index += 1;
        }
    }
    let key = without_year
        .to_uppercase()
        .replace("FY", " ")
        .replace(['.', ',', '/'], " ");
    let key = key.split_whitespace().collect::<Vec<_>>().join(" ");

    // Order matters: "ADJUSTED APPROPRIATIONS AS OF 9 30 22" must not fall through to the plain
    // appropriation arm, and "AS ENACTED AFTER GOVERNOR'S VETOES" is still the enacted figure.
    if key.is_empty() {
        // A bare `FY 2024` column. Always a closed prior year in this series — but the caller
        // checks that against the biennium rather than trusting it here.
        Some("bare")
    } else if key.starts_with("ADJ") {
        Some("adjusted")
    } else if key.contains("ACTUAL") {
        Some("actual")
    } else if key.contains("ENACTED") || key == "APPROPRIATION" {
        Some("enacted")
    } else {
        // Introduced, OBM Estimate, House Substitute, Senate Reported, Conference Report, and
        // both change columns. Real figures, and none of them law.
        None
    }
}

/// The first four-digit fiscal year named in a header cell.
fn header_year(label: &str) -> Option<u16> {
    let chars: Vec<char> = label.chars().collect();
    for window in chars.windows(4) {
        if window.iter().all(char::is_ascii_digit) {
            let year: u16 = window.iter().collect::<String>().parse().ok()?;
            if (1990..=2100).contains(&year) {
                return Some(year);
            }
        }
    }
    None
}

/// Locate a column whose header matches any of `names`, ignoring case, spacing and punctuation.
fn column_named(header: &[String], names: &[&str]) -> Option<usize> {
    let normalize = |text: &str| -> String {
        text.chars()
            .filter(char::is_ascii_alphanumeric)
            .collect::<String>()
            .to_uppercase()
    };
    header
        .iter()
        .position(|cell| names.iter().any(|name| normalize(cell) == normalize(name)))
}

/// Build the appropriation-line series from LSC's budget workbooks.
///
/// # Errors
///
/// Returns an error naming the workbook and the column when a header cannot be mapped, when a
/// bare year column falls inside the biennium — where it could be either the enacted amount or a
/// revised one and the label does not say — or when two accepted columns in one document claim
/// the same fiscal year and kind. Each of those is a figure this series would otherwise carry
/// under a label it cannot support.
pub fn build_appropriations(books: &[AppropriationBook<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for book in books {
        let label = format!("{} ({})", book.bill, book.source);
        let header_index = book
            .rows
            .iter()
            .take(8)
            .position(|row| column_named(row, &["ALI"]).is_some())
            .ok_or_else(|| format!("{label}: no header row carrying an `ALI` column"))?;
        let header = &book.rows[header_index];

        let ali =
            column_named(header, &["ALI"]).ok_or_else(|| format!("{label}: no `ALI` column"))?;
        let title = column_named(header, &["ALI Title", "ALIName", "Title"])
            .ok_or_else(|| format!("{label}: no line-item title column"))?;
        let fund_group = column_named(header, &["Fund Group", "FundGroup"]);
        let fund = column_named(header, &["Fund", "Fund Number"]);

        let mut columns: Vec<(usize, u16, &'static str)> = Vec::new();
        let mut seen: Vec<(u16, &'static str)> = Vec::new();
        for (index, cell) in header.iter().enumerate() {
            let Some(year) = header_year(cell) else {
                continue;
            };
            let Some(kind) = amount_kind(cell) else {
                continue;
            };
            let kind = if kind == "bare" {
                // In the revised workbook an unlabelled column is spending: that document exists
                // to report what was spent, it marks the year still open as `Adj. Appr.`, and
                // where a biennium year is genuinely an appropriation — the current biennium, in
                // the 136th — it says `Appropriation` on the column. In the enacted workbook a
                // bare column is only ever a prior year, because every biennium column there
                // carries the legislative stage it belongs to.
                if book.variant == "actuals" || year < book.first_year {
                    "actual"
                } else {
                    return Err(format!(
                        "{label}: column {index} is headed `{}`, which names a biennium year with \
                         no word saying whether it is the enacted amount or a later revision",
                        cell.trim()
                    ));
                }
            } else {
                kind
            };
            if seen.contains(&(year, kind)) {
                return Err(format!(
                    "{label}: two columns claim FY{year} {kind}; the second is `{}`",
                    cell.trim()
                ));
            }
            seen.push((year, kind));
            columns.push((index, year, kind));
        }
        if columns.is_empty() {
            return Err(format!("{label}: no column header names a fiscal year"));
        }

        for row in book.rows.iter().skip(header_index + 1) {
            let Some(line_item) = row.get(ali).map(|cell| cell.trim()) else {
                continue;
            };
            // Line items are numbered by agency and 200 is the Department of Education's.
            // Filtering on the number rather than an agency column is deliberate: three of the
            // sixteen carry no agency column, and two that do spell the agency differently.
            if line_item.len() != 6
                || !line_item.starts_with("200")
                || !line_item.chars().all(|c| c.is_ascii_digit())
            {
                continue;
            }
            let name = row.get(title).map(|cell| cell.trim()).unwrap_or_default();
            let at = |column: Option<usize>| -> String {
                column
                    .and_then(|index| row.get(index))
                    .map(|cell| cell.trim())
                    .unwrap_or_default()
                    .to_string()
            };
            for (index, year, kind) in &columns {
                let Some(raw) = row.get(*index).map(|cell| cell.trim()) else {
                    continue;
                };
                let Some(amount) = crate::conventions::number(raw) else {
                    continue;
                };
                out.push(vec![
                    book.general_assembly.to_string(),
                    book.bill.to_string(),
                    year.to_string(),
                    (*kind).to_string(),
                    book.source.to_string(),
                    at(fund_group),
                    at(fund),
                    line_item.to_string(),
                    name.to_string(),
                    format!("{amount:.2}"),
                ]);
            }
        }
    }
    if out.is_empty() {
        return Err("no appropriation lines were extracted from any workbook".to_string());
    }
    Ok(out)
}

/// Collapse every document into one row per claim, refusing if any two disagree.
///
/// # Errors
///
/// Returns an error naming the claim when two documents give it different amounts.
///
/// # Why the fixture is deduplicated rather than left as it was read
///
/// The documents overlap heavily and that is the source of confidence here: a fiscal year is
/// reported as an enacted amount by the act that made it and as a prior-year actual by the next
/// act's workbook, across layouts that share no conventions. Agreement across those is worth more
/// than any check this crate could write.
///
/// It is also, left alone, a way to be confidently wrong. Summing the raw extract to get "what
/// Ohio appropriated for schools in FY2027" counts every figure twice, because two documents
/// report it — which is how `200550 Foundation Funding` first came out at $17.5 billion against a
/// true $8.7 billion, on a total budget of $15.3 billion. A number that is exactly double is the
/// kind that survives a sanity check.
///
/// So agreement is asserted here, at build time, and the fixture carries one row per claim with
/// `documents` recording how many said it. Provenance is kept: `source` names the document the
/// figure was taken from, preferring the act whose own biennium the year falls in.
pub fn reconcile(rows: Vec<Vec<String>>) -> Result<Vec<Vec<String>>, String> {
    use std::collections::BTreeMap;
    // (fiscal_year, kind, line_item) -> the rows claiming it.
    let mut claims: BTreeMap<(String, String, String), Vec<Vec<String>>> = BTreeMap::new();
    for row in rows {
        claims
            .entry((row[2].clone(), row[3].clone(), row[7].clone()))
            .or_default()
            .push(row);
    }

    let mut out = Vec::with_capacity(claims.len());
    for ((year, kind, item), reports) in claims {
        let first: f64 = reports[0][9].parse().map_err(|_| {
            format!(
                "FY{year} {kind} {item}: {:?} is not a number",
                reports[0][9]
            )
        })?;
        for report in &reports {
            let amount: f64 = report[9]
                .parse()
                .map_err(|_| format!("FY{year} {kind} {item}: {:?} is not a number", report[9]))?;
            if (amount - first).abs() >= 0.005 {
                return Err(format!(
                    "FY{year} {kind} for line item {item} is {} in {} and {} in {}; two \
                     documents disagree about the same figure",
                    report[9], report[4], reports[0][9], reports[0][4]
                ));
            }
        }
        // The act whose own biennium the year falls in is the one that decided it; anything else
        // is reporting it second-hand. `general_assembly` and the year are both on the row.
        let ga_of = |row: &Vec<String>| -> u16 { row[0].parse().unwrap_or(0) };
        let year_number: u16 = year.parse().unwrap_or(0);
        let own = |row: &Vec<String>| -> bool {
            // The Nth General Assembly appropriates for the biennium starting FY(2N + 1754):
            // the 136th for FY2026-27, the 129th for FY2012-13. Derived rather than tabled so it
            // cannot fall out of step with the table in `rebuild`.
            let first_year = 2 * ga_of(row) + 1754;
            year_number == first_year || year_number == first_year + 1
        };
        let chosen = reports
            .iter()
            .find(|row| own(row) && row[4].ends_with("-enacted"))
            .or_else(|| reports.iter().find(|row| own(row)))
            .unwrap_or(&reports[0]);

        let mut row = chosen.clone();
        row.insert(5, reports.len().to_string());
        out.push(row);
    }
    out.sort();
    Ok(out)
}

/// One greenbook, with the biennium it appropriates for.
#[derive(Debug, Clone, Copy)]
pub struct Greenbook<'a> {
    /// The General Assembly that enacted it.
    pub general_assembly: u16,
    /// The bill, as the registry keys it.
    pub bill: &'a str,
    /// The registry key of the document.
    pub source: &'a str,
    /// The first fiscal year of the biennium it appropriates for.
    pub first_year: u16,
    /// `pdftotext -layout` output.
    pub text: &'a str,
}

/// The heading above each page of the line-item table.
const DETAIL_MARKER: &str = "Line Item Detail by Agency";

/// How far apart two right edges may be and still be the same column.
///
/// Amounts are right-aligned, so within a column their ends land within a character or two of
/// each other, while the gap between columns is a dozen or more. Four is comfortably inside that
/// margin in both directions.
const COLUMN_GAP: usize = 4;

/// The fiscal years a page's header names, left to right.
fn header_years(above: &str, below: &str) -> Vec<u16> {
    let mut found: Vec<(usize, u16)> = Vec::new();
    for line in [above, below] {
        let bytes: Vec<char> = line.chars().collect();
        let mut index = 0;
        while index + 3 <= bytes.len() {
            // `FY 1999` or `FY1999`. The bare years in a `2001 to 2002:` span label are
            // deliberately not matched: they are the `% Change` column's caption, not a column.
            if bytes[index] == 'F' && bytes.get(index + 1) == Some(&'Y') {
                let mut cursor = index + 2;
                if bytes.get(cursor) == Some(&' ') {
                    cursor += 1;
                }
                if cursor + 4 <= bytes.len()
                    && bytes[cursor..cursor + 4].iter().all(char::is_ascii_digit)
                {
                    if let Ok(year) = bytes[cursor..cursor + 4].iter().collect::<String>().parse() {
                        if (1990..=2100).contains(&year) && !found.iter().any(|(_, y)| *y == year) {
                            found.push((index, year));
                        }
                    }
                }
            }
            index += 1;
        }
    }
    found.sort_unstable();
    found.into_iter().map(|(_, year)| year).collect()
}

/// The end position of every dollar amount on a line, with its value.
fn amounts(line: &str) -> Vec<(usize, f64)> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        if chars[index] != '$' {
            index += 1;
            continue;
        }
        let mut cursor = index + 1;
        while chars.get(cursor) == Some(&' ') {
            cursor += 1;
        }
        let start = cursor;
        while cursor < chars.len() && (chars[cursor].is_ascii_digit() || chars[cursor] == ',') {
            cursor += 1;
        }
        if cursor > start {
            let digits: String = chars[start..cursor].iter().filter(|c| **c != ',').collect();
            if let Ok(value) = digits.parse::<f64>() {
                out.push((cursor, value));
            }
        }
        index = cursor.max(index + 1);
    }
    out
}

/// Group right edges into columns.
fn columns(mut ends: Vec<usize>) -> Vec<usize> {
    ends.sort_unstable();
    let mut centres = Vec::new();
    let mut group: Vec<usize> = Vec::new();
    for end in ends {
        if group.last().is_some_and(|last| end - last > COLUMN_GAP) {
            centres.push(group[group.len() / 2]);
            group.clear();
        }
        group.push(end);
    }
    if !group.is_empty() {
        centres.push(group[group.len() / 2]);
    }
    centres
}

/// Whether a line is a line-item row, and its fund, item and title if so.
fn detail_row(line: &str) -> Option<(String, String, String)> {
    let trimmed = line.trim_start();
    let mut parts = trimmed.split_whitespace();
    let fund = parts.next()?.to_string();
    if fund.len() < 3 || fund.len() > 4 || !fund.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let raw = parts.next()?;
    let item: String = raw.chars().filter(char::is_ascii_digit).collect();
    if item.len() != 6
        || !item.starts_with("200")
        || raw.trim_matches(['-'; 1].as_slice()).len() < 6
    {
        return None;
    }
    // The title runs to the first run of two spaces, which is where the figures begin.
    let after = trimmed.split_once(raw)?.1;
    let title = after
        .split("  ")
        .find(|piece| !piece.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_string();
    Some((fund, item, title))
}

/// Build appropriation lines from the greenbook PDFs of the 124th to 128th General Assemblies.
///
/// # Why the columns are calibrated from the figures and not from the header
///
/// The obvious source of column positions is the table header, and the header is not where the
/// data is. Two things had to be measured before this worked. Each table repeats per page and
/// `pdftotext -layout` lays every page out independently, so the same five columns sit at
/// character 60/75/91/107/137 on one page of the 124th and 64/82/100/119/148 on another. And the
/// header *labels* are narrower than the columns of figures beneath them: in the 127th they land
/// at 67, 81 and 91 while the amounts spread far wider, so assigning an amount to its nearest
/// label misdates 16 rows there and 6 in the 124th.
///
/// So the header is used for **which years**, in order, and the figures for **where the columns
/// are**: amounts are right-aligned, so their end positions cluster, and the number of clusters is
/// the number of columns. Across the four documents this assigns 2,185 figures with no row ever
/// claiming one column twice.
///
/// # Errors
///
/// Returns an error naming the page when its cluster count and year count disagree, or when a row
/// puts two amounts in one column. Either means the layout is not the one measured, and a misdated
/// appropriation is a plausible figure in the wrong year rather than a parse failure — these years
/// have no later document to be reconciled against, so refusing is the only guard available.
pub fn build_greenbook_appropriations(books: &[Greenbook<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for book in books {
        let lines: Vec<&str> = book.text.lines().collect();
        let marks: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.contains(DETAIL_MARKER))
            .map(|(index, _)| index)
            .collect();
        if marks.is_empty() {
            return Err(format!(
                "{}: no `{DETAIL_MARKER}` page; this document keeps its appropriations only in \
                 per-category tables",
                book.source
            ));
        }

        for (position, start) in marks.iter().enumerate() {
            let end = marks.get(position + 1).copied().unwrap_or(lines.len());
            let years = header_years(
                start.checked_sub(1).map(|i| lines[i]).unwrap_or_default(),
                lines.get(start + 1).copied().unwrap_or_default(),
            );
            if years.is_empty() {
                continue;
            }
            let rows: Vec<&str> = lines[*start..end]
                .iter()
                .copied()
                .filter(|line| detail_row(line).is_some())
                .collect();
            if rows.is_empty() {
                continue;
            }
            let centres = columns(
                rows.iter()
                    .flat_map(|line| amounts(line))
                    .map(|(end, _)| end)
                    .collect(),
            );
            if centres.len() != years.len() {
                return Err(format!(
                    "{}: the page at line {start} has {} columns of figures and names {} years",
                    book.source,
                    centres.len(),
                    years.len()
                ));
            }

            for line in rows {
                let Some((fund, item, title)) = detail_row(line) else {
                    continue;
                };
                let mut claimed: Vec<usize> = Vec::new();
                for (position, value) in amounts(line) {
                    let column = centres
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, centre)| centre.abs_diff(position))
                        .map(|(index, _)| index)
                        .unwrap_or_default();
                    if claimed.contains(&column) {
                        return Err(format!(
                            "{}: line item {item} puts two figures in the FY{} column",
                            book.source, years[column]
                        ));
                    }
                    claimed.push(column);
                    // The last two columns are the biennium this act appropriates for; everything
                    // to their left is a closed year, reported as spent.
                    let year = years[column];
                    let kind = if year >= book.first_year {
                        "enacted"
                    } else {
                        "actual"
                    };
                    out.push(vec![
                        book.general_assembly.to_string(),
                        book.bill.to_string(),
                        year.to_string(),
                        kind.to_string(),
                        book.source.to_string(),
                        String::new(),
                        fund.clone(),
                        item.clone(),
                        title.clone(),
                        format!("{value:.2}"),
                    ]);
                }
            }
        }
    }
    if out.is_empty() {
        return Err("no appropriation lines were extracted from any greenbook".to_string());
    }
    Ok(out)
}

// -------------------------------------------------------------------------------------------
// Scholarship annual report
// -------------------------------------------------------------------------------------------

/// Where the scholarship programme summary is written, relative to the repository root.
pub const SCHOLARSHIP_FIXTURE: &str = "crates/project/fixtures/scholarship-programs.csv";

/// The five programmes, in the order the report presents them.
///
/// Matched on a phrase the report uses in running prose rather than on a heading, because the
/// headings appear twice — once in the table of contents with dot leaders, once in the body — and
/// the table of contents carries no figures. The prose sentence appears only in the body.
const PROGRAMMES: &[(&str, &str)] = &[
    (
        "traditional-edchoice",
        "Traditional EdChoice Scholarship Program",
    ),
    (
        "edchoice-expansion",
        "EdChoice Expansion Scholarship Program",
    ),
    ("cleveland", "Cleveland Scholarship Program"),
    ("autism", "Autism Scholarship Program"),
    (
        "jon-peterson",
        "Jon Peterson Special Needs Scholarship Program",
    ),
];

/// Collapse runs of whitespace, so a figure split across a line break still reads as one token.
fn flatten(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The number immediately before `marker`, written with thousands separators.
fn count_before(text: &str, marker: usize) -> Option<f64> {
    let head = &text[..marker];
    let digits: String = head
        .chars()
        .rev()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit() || *c == ',')
        .filter(|c| *c != ',')
        .collect();
    if digits.is_empty() {
        return None;
    }
    digits.chars().rev().collect::<String>().parse().ok()
}

/// The dollar amount following the first occurrence of `marker` in `text`.
fn dollars_after(text: &str, marker: &str) -> Option<f64> {
    let at = text.find(marker)? + marker.len();
    let rest = &text[at..];
    let sign = rest.find('$')?;
    let after = &rest[sign + 1..];
    let end = after
        .find(|c: char| !c.is_ascii_digit() && c != ',' && c != '.')
        .unwrap_or(after.len());
    // `end` stops at the first character that is neither digit, comma nor point — which for
    // "$283,149,735.14." is the space *after* the sentence stop, so the stop comes along. Parsing
    // "283149735.14." fails, and it fails silently into an empty column rather than loudly.
    after[..end]
        .trim_end_matches('.')
        .chars()
        .filter(|c| *c != ',')
        .collect::<String>()
        .parse()
        .ok()
}

/// One row per scholarship programme: participation, expenditure, and the report's own average.
///
/// # How a programme is located
///
/// By its participation sentence — "N students utilized the `<programme>`", or for Jon Peterson
/// "N students received services from" — and not by its heading. The headings appear twice, once
/// in the table of contents with dot leaders and once in the body, and the table of contents
/// carries no figures. The sentence appears only in the body.
///
/// Each programme's section then runs to the next programme's participation sentence, and the two
/// dollar figures are read inside that window. Anchoring on order alone would work on this
/// edition and break silently on an edition that reordered the programmes.
///
/// # Why the average is carried rather than computed
///
/// Because the report's does not equal expenditure over participation, and the gap is the finding.
/// Autism reconciles to the cent; the other three published averages sit 1.5-3.4% above the
/// implied figure. Recomputing the column would erase that, and declaring the report wrong would
/// claim more than is known — the likeliest reading is that the two divide by different student
/// counts, and the report does not say. Both numbers are committed and the disagreement is pinned
/// in `crates/project/tests/` rather than resolved.
///
/// Jon Peterson publishes no expenditure total and no average: its spending appears only as six
/// disability-category figures in a chart. Those two columns are left empty, which states the
/// absence rather than filling it.
///
/// # Errors
///
/// If fewer than five programmes carry a participation sentence, which means the report was
/// restructured and every figure downstream of it should be treated as unread.
pub fn scholarship_programmes(text: &str) -> Result<Vec<Vec<String>>, String> {
    const MARKERS: [&str; 2] = ["students utilized the ", "students received services from "];
    let flat = flatten(text);

    // Section starts, in document order, with the count that opens each one.
    let mut sections: Vec<(usize, f64)> = Vec::new();
    for marker in MARKERS {
        for (at, _) in flat.match_indices(marker) {
            if let Some(students) = count_before(&flat, at) {
                sections.push((at, students));
            }
        }
    }
    sections.sort_by_key(|(at, _)| *at);

    if sections.len() != PROGRAMMES.len() {
        return Err(format!(
            "the scholarship report names {} programmes, not {}; its structure changed",
            sections.len(),
            PROGRAMMES.len()
        ));
    }

    let mut out = Vec::new();
    for (i, (at, students)) in sections.iter().enumerate() {
        let end = sections.get(i + 1).map_or(flat.len(), |(next, _)| *next);
        let window = &flat[*at..end];
        // Which programme this is, read from the sentence itself rather than assumed from order.
        let (key, name) = PROGRAMMES
            .iter()
            .find(|(_, phrase)| window.contains(phrase))
            .ok_or_else(|| format!("the section at {at} names no known programme"))?;
        out.push(vec![
            (*key).to_string(),
            (*name).to_string(),
            format!("{students:.0}"),
            dollars_after(window, "totaled").map_or(String::new(), |v| format!("{v:.2}")),
            dollars_after(window, "per participant was")
                .map_or(String::new(), |v| format!("{v:.2}")),
        ]);
    }
    Ok(out)
}

// -------------------------------------------------------------------------------------------
// Catalog of Budget Line Items
// -------------------------------------------------------------------------------------------

/// Where the per-line-item money series is written, relative to the repository root.
pub const CATALOG_FIXTURE: &str = "crates/project/fixtures/catalog-line-items.csv";

/// Where the per-line-item legal basis is written.
pub const CATALOG_BASIS_FIXTURE: &str = "crates/project/fixtures/catalog-line-item-basis.tsv";

/// One appropriation line item in one edition, before its years are split out.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogItem {
    /// The fund the line is paid from: `GRF`, `3AF`, `5VS0`.
    pub fund: String,
    /// The line item number, hyphens removed: `200550`.
    pub ali: String,
    /// The line's name as printed.
    pub name: String,
    /// The act that authorises it this biennium, and the act that established it.
    pub legal_basis: String,
    /// `(fiscal year, kind, amount)`, where `amount` is `None` for a printed `---`.
    pub years: Vec<(u16, String, Option<f64>)>,
}

/// A line-item heading: fund, number, and name.
///
/// The number is printed `200-100` in the 2006 edition and `200100` from 2008, so the hyphen is
/// optional here and stripped from the value. Without that the same line item reads as two
/// different ones across the series, which is the join this whole extract exists to support.
fn catalog_heading(line: &str) -> Option<(String, String, String)> {
    let mut parts = line.split_whitespace();
    let fund = parts.next()?;
    // Funds are short alphanumeric codes. Anything longer is prose that happens to start a line.
    if fund.len() > 5 || !fund.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let raw = parts.next()?;
    let ali: String = raw.chars().filter(char::is_ascii_digit).collect();
    if ali.len() != 6 || raw.chars().any(|c| !c.is_ascii_digit() && c != '-') {
        return None;
    }
    let name = parts.collect::<Vec<_>>().join(" ");
    // The 2025 layout appends a cross-reference like `IId-3470` to the name.
    let name = name
        .rsplit_once(" IId-")
        .map_or(name.as_str(), |(head, _)| head)
        .trim()
        .to_string();
    if name.is_empty() {
        return None;
    }
    Some((fund.to_string(), ali, name))
}

/// The amounts on one line, in order, with `---` carried as a stated absence.
///
/// # Why `---` is a value rather than a gap
///
/// Because it is the difference between an extraction that aligns and one that does not. Measured
/// across five editions spanning the whole series, 566 of 566 line items have exactly as many
/// amount tokens as year headings **once `---` counts as a token**; without it, seven items in the
/// 2006 edition come up short and every year after the gap is misdated by one column.
///
/// That is the failure the greenbook attempt was reverted for, arriving from the other direction:
/// there an unfunded year left an empty column, here the publisher prints a mark for it. A mark
/// that is read is a column that stays aligned.
fn catalog_amounts(line: &str) -> Vec<Option<f64>> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(at) = rest.find(['$', '-']) {
        let tail = &rest[at..];
        if tail.starts_with("--") {
            out.push(None);
            let end = tail.find(|c: char| c != '-').unwrap_or(tail.len());
            rest = &tail[end..];
        } else {
            let after = &tail[1..];
            let end = after
                .find(|c: char| !c.is_ascii_digit() && c != ',')
                .unwrap_or(after.len());
            let digits: String = after[..end].chars().filter(|c| *c != ',').collect();
            if digits.is_empty() {
                rest = after;
                continue;
            }
            out.push(digits.parse().ok());
            rest = &after[end..];
        }
    }
    out
}

/// The column kinds on a labels row, in order.
///
/// # The distinction this exists to preserve
///
/// Editions come in two shapes, and the difference is not cosmetic. An edition published at the
/// start of a biennium carries four actuals and **two `Appropriation` columns** — the enacted
/// amounts for the biennium just begun. An edition published a year later carries five actuals and
/// **one `Adj. Approp.`**: the biennium's first year has closed into an actual, and the year that
/// remains is stated as *adjusted*, not as enacted.
///
/// That is the same trap that reverted the workbook attempt on the greenbook series, where the
/// with-actuals variant silently superseded the enacted column and the extraction produced zero
/// appropriation rows for a whole biennium without failing. Here the publisher labels it, so the
/// only way to reproduce that failure is to throw the label away. These are therefore three kinds
/// and never two, and `adjusted` is never written as `appropriation`.
fn catalog_labels(line: &str) -> Vec<&'static str> {
    // Longest first, so `Adj. Approp.` is not read as a bare `Approp.`.
    const KINDS: [(&str, &str); 6] = [
        ("Adjusted Approp.", "adjusted"),
        ("Adj. Approp.", "adjusted"),
        ("Appropriation", "appropriation"),
        ("Approp.", "appropriation"),
        ("Estimate", "estimate"),
        ("Actual", "actual"),
    ];
    let mut out = Vec::new();
    let mut at = 0;
    while at < line.len() {
        let rest = &line[at..];
        if let Some((token, kind)) = KINDS
            .iter()
            .find(|(token, _)| rest.starts_with(token))
            .copied()
        {
            out.push(kind);
            at += token.len();
        } else {
            at += rest.chars().next().map_or(1, char::len_utf8);
        }
    }
    out
}

/// Every line item in one edition of the Catalog's education volume.
///
/// # How a block is found
///
/// By the **labels** row — the one reading `Actual Actual Actual Actual Appropriation
/// Appropriation` — rather than by the year row or the heading. The year row is printed `2002` in
/// the 2006 and 2009 editions and `FY 2022` from later ones, and a bare four-digit number is not a
/// distinctive thing to search a budget document for. The labels row is, and it fixes the year row
/// as the line above it and the amounts as the next line carrying figures.
///
/// # Why the years are paired ordinally rather than by column position
///
/// The greenbook attempt failed reading columns by nearest header label, because the labels are
/// narrower than the figures beneath them. The lesson recorded there was to calibrate from the
/// amounts. Here that reduces to something simpler and stronger: when the count of years, the
/// count of labels and the count of amounts all agree, left-to-right order is the pairing, and no
/// position arithmetic is involved at all.
///
/// So the guard is the count, and a block whose three counts disagree is **refused** rather than
/// guessed at. Across the sampled editions nothing is refused, which is the point — the rule costs
/// nothing when the document is well formed and it is the only thing standing between a misread
/// page and a plausible figure in the wrong year.
///
/// # Errors
///
/// If no line item parses at all, which means the edition's layout is one this has not seen.
pub fn catalog_items(text: &str) -> Result<(Vec<CatalogItem>, Vec<String>), String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out: Vec<CatalogItem> = Vec::new();
    let mut refused: Vec<String> = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let labels = catalog_labels(line);
        if labels.len() < 4 || i == 0 {
            continue;
        }

        let years: Vec<u16> = {
            let mut found = Vec::new();
            let row = lines[i - 1];
            let bytes = row.as_bytes();
            let mut k = 0;
            while k < bytes.len() {
                if bytes[k].is_ascii_digit() {
                    let start = k;
                    while k < bytes.len() && bytes[k].is_ascii_digit() {
                        k += 1;
                    }
                    if k - start == 4 {
                        if let Ok(y) = row[start..k].parse::<u16>() {
                            if (1990..2100).contains(&y) {
                                found.push(y);
                            }
                        }
                    }
                } else {
                    k += 1;
                }
            }
            found
        };
        if years.is_empty() {
            continue;
        }

        let Some((amounts, _)) = lines[i + 1..lines.len().min(i + 4)]
            .iter()
            .enumerate()
            .map(|(offset, l)| (catalog_amounts(l), offset))
            .find(|(a, _)| !a.is_empty())
        else {
            continue;
        };

        // The heading is the nearest line above the year row that parses as one.
        let Some((fund, ali, name)) = (i.saturating_sub(6)..i - 1)
            .rev()
            .find_map(|k| catalog_heading(lines[k]))
        else {
            continue;
        };

        if years.len() != labels.len() || years.len() != amounts.len() {
            refused.push(format!(
                "{fund} {ali} {name}: {} years, {} labels, {} amounts",
                years.len(),
                labels.len(),
                amounts.len()
            ));
            continue;
        }

        // `Legal Basis:` runs to the next labelled field, wrapping across lines.
        let mut legal_basis = String::new();
        for l in lines[i..lines.len().min(i + 30)].iter() {
            if let Some(rest) = l.trim_start().strip_prefix("Legal Basis:") {
                legal_basis = rest.trim().to_string();
                continue;
            }
            if !legal_basis.is_empty() {
                let t = l.trim();
                if t.is_empty() || t.contains(':') {
                    break;
                }
                legal_basis.push(' ');
                legal_basis.push_str(t);
            }
        }

        out.push(CatalogItem {
            fund,
            ali,
            name,
            legal_basis: legal_basis.split_whitespace().collect::<Vec<_>>().join(" "),
            years: years
                .into_iter()
                .zip(labels)
                .zip(amounts)
                .map(|((y, kind), amount)| (y, kind.to_string(), amount))
                .collect(),
        });
    }

    if out.is_empty() {
        return Err("no line items parsed; the edition's layout is unrecognised".to_string());
    }
    Ok((out, refused))
}
