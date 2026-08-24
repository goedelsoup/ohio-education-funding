//! The department's FY2027 foundation funding calculator, and the profile report it joins.
//!
//! The largest extractor in the crate and the one the whole corpus rests on: [`build_fy27_model`]
//! turns eleven sheets of the department's published workbook into the 162-column fixture every
//! calculator crate reads.
//!
//! Twenty private `*_columns` modules stand between the arithmetic and the sheets. Each names the
//! positions of one block, so a workbook reshaped by the department fails at a named constant
//! rather than reading a neighbouring column silently — which is what happened when the transition
//! block was emitted before the preschool block and listed after it.
//!
//! The FY2024 District Profile Report is here rather than in a file of its own because the model
//! joins it: [`build_fy27_model`] takes assessed valuation from it, [`build_profile_extract`]
//! writes it out as its own fixture, and both read it through the shared `profile_columns` table, as does
//! [`super::grade_bands`].

use std::collections::HashMap;

use super::format::{clean_name, format_value};
use crate::conventions::{cell, cell_number, is_statewide_row, number, rows_by_key};

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
    /// `[H2] Funding Base` — the **FY2020** amount the guarantee compares against, and the
    /// origin R.C. 3317.022 interpolates the phase-in from. Not FY2021: that is `[L1]` below,
    /// which belongs to the formula transition supplement.
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
pub(super) mod profile_columns {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conventions::STATEWIDE_ROW;

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
}
