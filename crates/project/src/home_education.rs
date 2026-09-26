//! The statewide home-education series, and what ties it to a figure the department published.
//!
//! Twenty school years, 2005-06 through 2024-25, from the Johns Hopkins Homeschool Hub. Ohio
//! publishes one home-education number in machine-reachable form and it is the 2023-24 one, in
//! the fact sheet [`crate::ledger::nonpublic_support`] reads; every other year in this series
//! rests on the aggregator alone and is `[inference]` for that reason.
//!
//! # Why the series is worth holding at all
//!
//! Because of one row. The Hub's 2023-24 cell and the department's own *Ohio's Education
//! Landscape* both say **53,051**, and the two files are pinned separately by digest under
//! different publishers. Agreement to the student between an aggregator and the source of record
//! is what identifies this series as the department's counts carried forward rather than an
//! estimate — and it is what licences reading the other nineteen years as the same quantity.
//! [`agrees_with_the_department`] is that check, and it is a test rather than a comment.
//!
//! # What the series shows, which is the reason home education is a funding question
//!
//! 25,937 in 2005-06, 33,328 in 2019-20, **51,502 in 2020-21** — a 55% jump in one year — then
//! 47,491, 47,468, 53,051, and 61,009 in 2024-25. The pandemic peak did not come back down; it
//! plateaued for two years and then resumed climbing past it. At 61,009 the population is larger
//! than every scholarship programme the department reports but one, and larger than the joint
//! vocational enrolment printed beside it on the same page.
//!
//! None of those children carry ADM. They leave the formula without appearing in any enrollment
//! file, which is what makes the per-district cut — `dispersion::home_education` — the part of
//! this that a funding question actually needs.

use edfund_core::csv;

const FIXTURE: &str = include_str!("../fixtures/home-education-statewide.csv");

const EXPECTED_HEADER: &str = "school_year,students";

/// The one year in the series that also exists in a department publication.
pub const VERIFIED_YEAR: &str = "2023-2024";

/// One school year's statewide count.
#[derive(Debug, Clone, PartialEq)]
pub struct Year {
    /// The school year, as the workbook writes it: `2023-2024`.
    pub school_year: String,
    /// Children reported in home education statewide.
    pub students: u32,
}

/// The series, earliest year first.
///
/// The years before 2005-06 are not here: the workbook leaves them blank, and a blank is not a
/// count of no home educators in Ohio.
///
/// # Panics
///
/// If the fixture's header has moved, or a count is not a number.
#[must_use]
pub fn series() -> Vec<Year> {
    csv::rows(FIXTURE, EXPECTED_HEADER)
        .map(|row| Year {
            school_year: row.str(0).to_string(),
            students: row
                .str(1)
                .parse()
                .unwrap_or_else(|_| panic!("{} has a count that is not a number", row.str(0))),
        })
        .collect()
}

/// One year of the series.
#[must_use]
pub fn students(school_year: &str) -> Option<u32> {
    series()
        .into_iter()
        .find(|year| year.school_year == school_year)
        .map(|year| year.students)
}

/// Whether the aggregator's [`VERIFIED_YEAR`] cell equals the department's own published count.
///
/// The department's figure comes from the Landscape fact sheet's School Options table, pinned
/// as its own source under a different publisher. This is the join between the one `[verified]`
/// home-education figure in the corpus and the nineteen `[inference]` ones beside it.
#[must_use]
pub fn agrees_with_the_department() -> bool {
    let published = crate::ledger::nonpublic_support::home_education_enrolment();
    students(VERIFIED_YEAR).is_some_and(|hub| f64::from(hub) == published)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_aggregator_and_the_department_agree_to_the_student() {
        assert_eq!(
            students(VERIFIED_YEAR),
            Some(53_051),
            "the Hub's 2023-24 cell"
        );
        assert!(
            agrees_with_the_department(),
            "two files pinned separately under different publishers, one number"
        );
    }

    #[test]
    fn the_series_is_twenty_years_and_starts_where_the_workbook_stops_being_blank() {
        let series = series();
        assert_eq!(series.len(), 20);
        assert_eq!(series[0].school_year, "2005-2006");
        assert_eq!(series[0].students, 25_937);
        assert_eq!(series[19].school_year, "2024-2025");
        assert_eq!(series[19].students, 61_009);
        assert!(
            series
                .windows(2)
                .all(|w| w[0].school_year < w[1].school_year),
            "the series is in year order"
        );
    }

    #[test]
    fn the_pandemic_jump_did_not_come_back_down() {
        // Worth a test rather than a sentence: the prose above reads the series this way, and a
        // revision that flattened it would leave the prose standing and wrong.
        let before = students("2019-2020").expect("2019-20 is in the series");
        let peak = students("2020-2021").expect("2020-21 is in the series");
        let latest = students("2024-2025").expect("2024-25 is in the series");
        assert!(peak > before * 3 / 2, "{peak} against {before}");
        assert!(
            latest > peak,
            "{latest} against the pandemic peak of {peak}"
        );
    }

    #[test]
    fn the_district_sheet_contradicts_the_year_it_is_labeled_with() {
        // The two fixtures from this workbook, read against each other. The district rows are
        // labeled SY21-22 and their reported cells alone come to more than this series gives
        // for 2021-22 — which is why `dispersion::home_education` carries no year of its own.
        let labeled = students("2021-2022").expect("2021-22 is in the series");
        let floor = dispersion::home_education::total().floor;
        assert!(
            floor > labeled,
            "the district floor {floor} no longer exceeds the {labeled} the same workbook gives \
             for the year it labels those rows with"
        );
    }
}
