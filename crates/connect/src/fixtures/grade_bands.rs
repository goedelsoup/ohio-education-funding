//! FY2024 headcount by grade band, from the profile report's own headcount columns.
//!
//! Reads the same publisher file as [`super::fy27`]'s profile extract, through the same
//! `profile_columns` table, and takes a different set of columns out of it: the
//! per-grade headcounts that `foundation` needs to build a grade-band ADM.

use super::format::{clean_name, format_value};
use super::fy27::profile_columns;
use crate::conventions::{cell, cell_number, rows_by_key};

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
