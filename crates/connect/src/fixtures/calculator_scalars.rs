//! The statewide scalars a year's funding calculator states once and multiplies everywhere.
//!
//! # Why a whole fixture for thirty numbers
//!
//! Half a dozen `parameter` nodes carry a `series:` with one year in it, and several say so in
//! the same words — "no prior-year values are held", "the calculator publishes one year and no
//! history". None of that was ever true of the sources: the department serves FY2027 and the
//! Internet Archive has FY2026, and both print these numbers on the same labelled cells above
//! their district tables. What was missing was two rows of CSV.
//!
//! This is the file a `parameter` node's series should be read off, and the next year's
//! calculator adds a row to it rather than a claim to a node.
//!
//! # Read by label, and the labels are the department's
//!
//! Every value here is the first number to the right of a label the department wrote — `EGS PP`,
//! `Proration Factor`, `Statewide DPIA percentage`. A positional reader would work on both of
//! these workbooks and break silently on the third; a label that has moved is followed and a
//! label that has gone is an error. Same rule as `super::transport_rates`, and the same reason.
//!
//! # What it already shows
//!
//! Every weight and every base cost is **identical** across the two years — the six special
//! education weights, the five career-technical weights, the three English learner weights, the
//! $8,241.61 average base cost and the $9,855.62 career-technical one. That is H.B. 96 holding
//! the FY2024 statewide averages and the FY2022 cost inputs, visible a year before the year the
//! act names.
//!
//! What moved is everything downstream of a count or a valuation: the disadvantaged pupil blend
//! and the statewide share it indexes against, the median weighted wealth targeted assistance
//! measures from, and the two enrolment supplements — which are the only per-pupil amounts in
//! the plan that rose.

use super::format::format_value;

/// Columns of the scalar fixture, one row per fiscal year.
pub const CALCULATOR_SCALARS_HEADER: &[&str] = &[
    "fiscal_year",
    "average_base_cost_per_pupil",
    "career_technical_base_cost_per_pupil",
    "special_education_weight_1",
    "special_education_weight_2",
    "special_education_weight_3",
    "special_education_weight_4",
    "special_education_weight_5",
    "special_education_weight_6",
    "career_technical_weight_1",
    "career_technical_weight_2",
    "career_technical_weight_3",
    "career_technical_weight_4",
    "career_technical_weight_5",
    "career_technical_associated_weight",
    "english_learner_weight_1",
    "english_learner_weight_2",
    "english_learner_weight_3",
    "dpia_per_pupil",
    "dpia_statewide_percentage",
    "dpia_weight_economically_disadvantaged",
    "dpia_weight_directly_certified",
    "performance_supplement_per_pupil",
    "base_funding_supplement_per_pupil",
    "enrolment_growth_supplement_per_pupil",
    "enrolment_growth_supplement_pct_cell",
    "preschool_base_amount",
    "preschool_proration_factor",
    "preschool_appropriation_cell",
    "benchmark_ratio",
    "median_weighted_wealth",
    "median_weighted_wealth_per_pupil",
    "fy2019_maximum_targeted_assistance",
    "median_weighted_wealth_excluding_islands",
    "median_weighted_wealth_per_pupil_excluding_islands",
    "preschool_total_funds",
];

/// Where each scalar is: the sheet, and the department's label whose neighbour is the value.
///
/// In fixture-column order after `fiscal_year`, so the builder is a fold over this table and a
/// column cannot be filled from the wrong label by getting an index wrong.
const SCALARS: &[(&str, &str)] = &[
    ("Special Edu", "Average Base Cost Per-Pupil"),
    ("CTE", "Average CTE Base Cost Per-Pupil"),
    ("Special Edu", "Weight Cat1"),
    ("Special Edu", "Weight Cat2"),
    ("Special Edu", "Weight Cat3"),
    ("Special Edu", "Weight Cat4"),
    ("Special Edu", "Weight Cat5"),
    ("Special Edu", "Weight Cat6"),
    ("CTE", "Weight Cat1"),
    ("CTE", "Weight Cat2"),
    ("CTE", "Weight Cat3"),
    ("CTE", "Weight Cat4"),
    ("CTE", "Weight Cat5"),
    ("CTE", "Assoc Weight"),
    ("EL", "Weight Cat 1"),
    ("EL", "Weight Cat 2"),
    ("EL", "Weight Cat 3"),
    ("DPIA", "DPIA Per- Pupil Amount"),
    ("DPIA", "Statewide DPIA percentage"),
    ("DPIA", "d1a pct"),
    ("DPIA", "d1b pct"),
    ("Performance Supplement", "Performance Supplement PP"),
    ("Base_Enrollment Growth", "Base supp per-pupil"),
    ("Base_Enrollment Growth", "EGS PP"),
    // `EGS pct`, which is **not** the eligibility threshold: the act's own analysis puts that at
    // 5% of FY2022-FY2025 growth for FY2026 and 3% of FY2023-FY2026 growth for FY2027, and this
    // cell reads 1 in both years. Taken because it is a labelled scalar that did not move;
    // what it governs is not established here.
    ("Base_Enrollment Growth", "EGS pct"),
    ("PSS_pec_Ed", "Base Amount"),
    ("PSS_pec_Ed", "Proration Factor"),
    ("PSS_pec_Ed", "Appropriation Limit"),
    ("Local_Capacity", "[C5]"),
    ("Targeted_Assistance", "[s1] Median Weighted Wealth"),
    ("Targeted_Assistance", "[s2] Median Weighted Wealth Per Pu"),
    ("Targeted_Assistance", "[s3] FY19 Maximum Targeted Assista"),
    // The department's own words, split across two rows of one label: the cell reading
    // "Excluding North" sits above "and Middle Bass", and the number each wants is to its right.
    // A second pair of statewide medians, computed without the two Lake Erie island districts,
    // displayed beside the pair the formula uses and referenced by nothing.
    ("Targeted_Assistance", "Excluding North"),
    ("Targeted_Assistance", "and Middle Bass"),
];

/// The sheets a year's scalars are spread across, in the order the scalar table names them.
pub const SHEETS: &[&str] = &[
    "Special Edu",
    "CTE",
    "EL",
    "DPIA",
    "Performance Supplement",
    "Base_Enrollment Growth",
    "PSS_pec_Ed",
    "Local_Capacity",
    "Targeted_Assistance",
];

/// One year's workbook, as the rebuild reads it: each sheet this needs, by name.
pub struct ScalarYear<'a> {
    /// The fiscal year the model computes, which no sheet states.
    pub fiscal_year: u16,
    /// `(sheet name, its rows)`, covering at least [`SHEETS`].
    pub sheets: &'a [(&'a str, Vec<Vec<String>>)],
}

/// The first number to the right of a label, in the block above the district table.
///
/// Only the first six rows: below them is the district table, where `Weight Cat1` would find a
/// district's own weighted ADM and read as a statewide weight. Six because `Targeted_Assistance`
/// puts its scalar block at rows three to five and `PSS_pec_Ed` puts its at rows zero and one.
fn value_beside(rows: &[Vec<String>], label: &str) -> Option<f64> {
    for (index, row) in rows.iter().enumerate().take(6) {
        let Some(at) = row.iter().position(|cell| cell.trim().starts_with(label)) else {
            continue;
        };
        // **Below first, and this order is load-bearing.** Some sheets head a column with the
        // label and put the number under it; others put the number to the right on the same row.
        // Scanning rightwards first reads the `EL` sheet's three weights as $8,241.61, because
        // that sheet puts `Average Base Cost Per-Pupil` to the *right* of `Weight Cat 1` and the
        // first number on the row is therefore the wrong one. The cell directly under a label is
        // never a different quantity's value; the next number along the row can be.
        if let Some(found) = rows.get(index + 1).and_then(|below| {
            below
                .get(at)
                .and_then(|cell| cell.trim().parse::<f64>().ok())
        }) {
            return Some(found);
        }
        if let Some(found) = row
            .iter()
            .skip(at + 1)
            .find_map(|cell| cell.trim().parse::<f64>().ok())
        {
            return Some(found);
        }
    }
    None
}

/// The preschool sheet's own statewide total, off its aggregate row.
///
/// Not a labelled cell like everything else here, which is why it is read separately: the
/// `PSS_pec_Ed` aggregate carries a **blank IRN and a blank district name**, the third shape a
/// calculator aggregate row takes in this workbook — `Detail_SFPR` names it `State of Ohio` and
/// `Transportation` leaves the name blank but keeps an IRN. It is taken rather than summed
/// because it is the department's own figure, and what it is wanted for is checking the
/// department's own proration against it.
fn preschool_total(rows: &[Vec<String>]) -> Option<f64> {
    let head = rows.iter().position(|row| {
        row.first()
            .is_some_and(|cell| cell.trim() == "District IRN")
    })?;
    let column = rows[head]
        .iter()
        .position(|cell| cell.trim() == "Total Special Education Funds")?;
    rows[head + 1..]
        .iter()
        .filter(|row| row.first().is_none_or(|cell| cell.trim().is_empty()))
        .find_map(|row| row.get(column)?.trim().parse::<f64>().ok())
}

/// One row per fiscal year, oldest first.
///
/// # Errors
///
/// If a year's workbook is missing a sheet, or a sheet is missing a label. Both mean the
/// department reshaped the model and a caller would otherwise read a neighbouring number.
pub fn build_calculator_scalars(years: &[ScalarYear<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for year in years {
        let mut row = vec![year.fiscal_year.to_string()];
        for (sheet, label) in SCALARS {
            let rows = year
                .sheets
                .iter()
                .find(|(name, _)| name == sheet)
                .map(|(_, rows)| rows)
                .ok_or_else(|| format!("FY{}: no `{sheet}` sheet", year.fiscal_year))?;
            let value = value_beside(rows, label)
                .ok_or_else(|| format!("FY{}: `{sheet}` states no `{label}`", year.fiscal_year))?;
            row.push(format_value(Some(value), 12));
        }
        let preschool = year
            .sheets
            .iter()
            .find(|(name, _)| *name == "PSS_pec_Ed")
            .and_then(|(_, rows)| preschool_total(rows))
            .ok_or_else(|| {
                format!(
                    "FY{}: `PSS_pec_Ed` has no aggregate row carrying a statewide total",
                    year.fiscal_year
                )
            })?;
        row.push(format_value(Some(preschool), 2));
        out.push(row);
    }
    out.sort_by(|left, right| left[0].cmp(&right[0]));
    Ok(out)
}
