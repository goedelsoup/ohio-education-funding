//! The 2024-25 Ohio School Report Card, at district grain, and its expenditure-by-function file.
//!
//! Two fixtures from one publisher's release. The report card carries the corpus's only outcome
//! measures — the Performance Index and the value-added effect size — and the function file
//! carries what a district spent, broken out by what it spent it on.
//!
//! The subgroup constants exist because the report card publishes one row per district *per
//! subgroup* and the district row is identified by a literal string.

use std::collections::HashMap;

use super::format::{clean_name, format_value};
use crate::conventions::{cell, is_statewide_row, number, rows_by_key};

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
/// [`crates/dispersion/tests/expenditure_functions_fy25.rs`](../../../dispersion/tests/expenditure_functions_fy25.rs)
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
