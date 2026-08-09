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

use std::collections::HashMap;
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
];

/// Column positions in the department's `Base_Cost` sheet, whose header is on the fourth row.
/// Named here rather than inline so a layout change is one edit.
mod base_cost_columns {
    pub const NAME: usize = 1;
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
mod adm_columns {
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

/// Join the department's FY2027 base cost and summary sheets with profile-report valuation.
#[must_use]
pub fn build_fy27_model(
    base_cost_rows: &[Vec<String>],
    summary_rows: &[Vec<String>],
    adm_rows: &[Vec<String>],
    profile_rows: &[Vec<String>],
    detail_rows: &[Vec<String>],
    capacity_rows: &[Vec<String>],
) -> Vec<Vec<String>> {
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
    let capacity: HashMap<&str, &Vec<String>> = capacity_rows
        .iter()
        .skip(2)
        .filter(|row| !cell(row, capacity_columns::IRN).trim().is_empty())
        .map(|row| (cell(row, capacity_columns::IRN).trim(), row))
        .collect();

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
                6,
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
        build_fy27_model(
            &base_cost_rows(),
            &summary_rows(),
            &adm_rows(),
            &profile_rows(),
            &[],
            &[],
        )
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
        let rows = build_fy27_model(
            &base_cost_rows(),
            &summary_rows()[..1],
            &adm_rows(),
            &profile_rows(),
            &[],
            &[],
        );
        assert!(rows.is_empty());
    }

    #[test]
    fn skips_a_district_with_no_usable_adm() {
        let mut base = base_cost_rows();
        base[5][7] = "0".into();
        let rows = build_fy27_model(
            &base,
            &summary_rows(),
            &adm_rows(),
            &profile_rows(),
            &[],
            &[],
        );
        assert_eq!(
            rows.iter().map(|r| r[0].as_str()).collect::<Vec<_>>(),
            ["043786"]
        );
    }

    #[test]
    fn commas_are_stripped_from_district_names() {
        let mut base = base_cost_rows();
        base[5][1] = "Northern Local, Perry".into();
        let rows = build_fy27_model(
            &base,
            &summary_rows(),
            &adm_rows(),
            &profile_rows(),
            &[],
            &[],
        );
        assert!(!rows[1][1].contains(','));
        assert_eq!(rows[1][1], "Northern Local  Perry");
    }

    #[test]
    fn trailing_whitespace_is_stripped_from_district_names() {
        let mut base = base_cost_rows();
        base[5][1] = "Bellefontaine City ".into();
        let rows = build_fy27_model(
            &base,
            &summary_rows(),
            &adm_rows(),
            &profile_rows(),
            &[],
            &[],
        );
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

/// Column positions in the Census `elsec22t` sheet.
///
/// # `STATE` is not a FIPS code
///
/// Column 0 is the Census Bureau's own state ordering and column 4 is FIPS. They agree for the
/// early states and diverge after: filtering on `STATE == "39"` returns Pennsylvania, whose FIPS
/// is 42, while Ohio's FIPS 39 is Census 36. Both columns are two-digit, zero-padded, and
/// numeric, so the mistake produces a full and entirely wrong answer.
mod f33 {
    pub const FIPS: usize = 4;
    pub const SCHOOL_LEVEL: usize = 9;
    pub const ENROLLMENT: usize = 12;
    pub const TOTAL_REVENUE: usize = 13;
    pub const FEDERAL_REVENUE: usize = 14;
    pub const STATE_REVENUE: usize = 19;
    pub const LOCAL_REVENUE: usize = 24;
    /// Local revenue from property tax specifically.
    ///
    /// Zero for nine states, and not because they levy none — see [`super::build_f33_states`].
    pub const PROPERTY_TAX: usize = 26;
    /// Appropriations from a parent city or county, where the district is not fiscally
    /// independent. The counterpart to [`PROPERTY_TAX`] and the reason it cannot be ranked alone.
    pub const PARENT_GOVERNMENT: usize = 27;
    pub const CURRENT_SPENDING: usize = 33;
}

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
#[must_use]
pub fn build_f33_states(rows: &[Vec<String>]) -> Vec<Vec<String>> {
    let number = |row: &[String], index: usize| -> f64 {
        row.get(index)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .unwrap_or(0.0)
    };

    let mut totals: std::collections::BTreeMap<String, [f64; 8]> =
        std::collections::BTreeMap::new();
    let mut systems: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();

    for row in rows.iter().skip(1) {
        let enrolled = number(row, f33::ENROLLMENT);
        if enrolled <= 0.0 {
            continue;
        }
        let Some(fips) = row.get(f33::FIPS).map(|s| s.trim().to_string()) else {
            continue;
        };
        // A school level the survey does not use would mean the layout has moved under us.
        debug_assert!(matches!(
            row.get(f33::SCHOOL_LEVEL).map(String::as_str),
            Some("01" | "02" | "03" | "05" | "06" | "07")
        ));

        *systems.entry(fips.clone()).or_default() += 1;
        let entry = totals.entry(fips).or_insert([0.0; 8]);
        entry[0] += enrolled;
        entry[1] += number(row, f33::TOTAL_REVENUE);
        entry[2] += number(row, f33::FEDERAL_REVENUE);
        entry[3] += number(row, f33::STATE_REVENUE);
        entry[4] += number(row, f33::LOCAL_REVENUE);
        entry[5] += number(row, f33::PROPERTY_TAX);
        entry[6] += number(row, f33::PARENT_GOVERNMENT);
        entry[7] += number(row, f33::CURRENT_SPENDING);
    }

    totals
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
        .collect()
}
