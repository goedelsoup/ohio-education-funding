//! Home education by district: who has left the public system by the one route no enrollment
//! file records.
//!
//! A home-educated child carries no ADM. They are not in the foundation formula, not in the
//! community-school deduct, not in a scholarship count, and not in the district's enrollment —
//! they are simply gone, and the only thing that records the going is a notice under
//! R.C. 3321.042 held by the district of residence. The department collects those counts and
//! publishes one number from them: the statewide total.
//!
//! This is the per-district breakdown, from the Johns Hopkins Homeschool Hub workbook — an
//! aggregator that asked the department for what it does not publish. See
//! `connect::fixtures::home_education` for how 611 unkeyed district names were resolved to IRNs
//! and [`jhu-homeschool-hub`](../../../.yidam/catalog/jhu-homeschool-hub.md) for what the file
//! can and cannot be trusted for.
//!
//! # The year is the publisher's label and the publisher is wrong
//!
//! Every row reads [`PUBLISHED_YEAR`], and the same workbook's statewide sheet gives that year as
//! 47,491 against a district floor of [`PUBLISHED_FLOOR`] — a file cannot exceed its own state
//! total by 3,858. The label is refuted by the workbook against itself, which is a fact about the
//! file; what year the rows actually are is an inference, it is argued in the catalog entry
//! (2023-24, on a size-matched reconstruction landing 49 students from the published 53,051), and
//! it is deliberately not in the data. A reader who needs the year needs the entry.
//!
//! So this is a **cross-section without a date**, and the honest uses of it are ones that do not
//! need one: the shape of the distribution, which districts are heavy and which are empty, and
//! a participation rate read against an enrollment denominator whose own year is stated.
//!
//! # Most of what is not a number here is not a zero
//!
//! Three censoring markers, none of them defined in the workbook's codebook, and the difference
//! between them matters more than 52 rows out of 611 suggests — one of them is a 4,150-pupil
//! district. [`Basis`] keeps them apart and [`Count`] bounds each one as far as the file bounds
//! it and no further. The twenty districts that report an explicit **zero** are a fourth thing
//! again, and [`Basis::Reported`] covers them, because a published zero is a measurement.

use std::collections::BTreeMap;

use edfund_core::csv;

const FIXTURE: &str = include_str!("../fixtures/home-education-districts.csv");

const EXPECTED_HEADER: &str =
    "irn,district,county,published_year,published,students_floor,students_ceiling";

/// The school year the workbook labels every Ohio row with.
///
/// Carried as a constant so the prose above and the fixture cannot drift apart: the module says
/// the label is refuted, and a test reads this off the committed rows.
pub const PUBLISHED_YEAR: &str = "SY21-22";

/// What the reported rows sum to, which is the least the sheet can come to.
///
/// The censored rows are each at least zero, so this is a floor on the whole sheet. It is above
/// the same workbook's 47,491 for [`PUBLISHED_YEAR`], which is the contradiction that dates the
/// file to something other than its label.
pub const PUBLISHED_FLOOR: u32 = 51_349;

/// The columns of [`EXPECTED_HEADER`].
mod column {
    pub const IRN: usize = 0;
    pub const DISTRICT: usize = 1;
    pub const COUNTY: usize = 2;
    pub const PUBLISHED_YEAR: usize = 3;
    pub const PUBLISHED: usize = 4;
    pub const FLOOR: usize = 5;
    pub const CEILING: usize = 6;
}

/// What a cell of the published sheet is.
///
/// The codebook defines none of the three markers. What separates them is where they appear:
/// `.` is used by nine other states' blocks and reads as the compiler's own blank, while `<10`
/// and `NC` appear on Ohio's rows alone and read as the department's markers on the file it
/// released. That is evidence, not documentation, which is why this enum keeps all three rather
/// than folding two of them into one "missing".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Basis {
    /// A number, which may be zero. Twenty districts report an explicit zero and this is what
    /// they are: a published measurement, not an absence.
    Reported,
    /// `<10`. The department's mask for a small count, the same convention the chartered
    /// nonpublic files use, and the only one of the three that bounds the cell above.
    UnderTen,
    /// `NC`. Ohio's rows alone, five of them, undefined anywhere in the workbook, and one of
    /// them is a district of 4,150 pupils — so it is not a small-count mask under another name.
    NotCollected,
    /// `.`. Nine other states use it too, which is what makes it read as the aggregator's blank
    /// rather than something the department wrote.
    Blank,
}

impl Basis {
    /// Read a published cell. Anything that is not one of the three markers is a count.
    #[must_use]
    pub fn of(published: &str) -> Self {
        match published.trim() {
            "<10" => Self::UnderTen,
            "NC" => Self::NotCollected,
            "." => Self::Blank,
            _ => Self::Reported,
        }
    }

    /// Whether the count behind this cell is unknown.
    #[must_use]
    pub const fn is_censored(self) -> bool {
        !matches!(self, Self::Reported)
    }
}

/// A count known only as far as the file states it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Count {
    /// The least it can be. Zero for every censored cell; the reported number otherwise.
    pub floor: u32,
    /// The most it can be, where anything in the file says. `None` for `.` and `NC`, which
    /// nothing bounds above — writing a ceiling for those would be inventing one.
    pub ceiling: Option<u32>,
}

impl Count {
    /// The count, where it is a point rather than a range.
    #[must_use]
    pub const fn exact(self) -> Option<u32> {
        match self.ceiling {
            Some(ceiling) if ceiling == self.floor => Some(self.floor),
            _ => None,
        }
    }
}

/// One district's home-education count, as the released file has it.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// The district's IRN, resolved from its name — the published sheet carries no key.
    pub irn: String,
    /// The department's spelling of the name, from the district list the IRN was resolved
    /// against. Names repeat, so 20 of them carry a parenthesised county.
    pub county: String,
    /// The department's spelling of the district name.
    pub name: String,
    /// The school year the workbook labels the row with. See [`PUBLISHED_YEAR`].
    pub published_year: String,
    /// The cell exactly as published: a count, or `<10`, `NC` or `.`.
    pub published: String,
    /// Which of those it is.
    pub basis: Basis,
    /// What the file bounds the count to.
    pub count: Count,
}

/// Every district in the released sheet.
///
/// # Panics
///
/// If the fixture's header has moved, or a row's floor is not a number — either means the
/// extract is not the file this reader was written against.
#[must_use]
pub fn districts() -> Vec<District> {
    csv::rows(FIXTURE, EXPECTED_HEADER)
        .map(|row| {
            let published = row.str(column::PUBLISHED).to_string();
            let floor = row
                .str(column::FLOOR)
                .parse()
                .unwrap_or_else(|_| panic!("{} has a floor that is not a number", row.str(0)));
            District {
                irn: row.str(column::IRN).to_string(),
                name: row.str(column::DISTRICT).to_string(),
                county: row.str(column::COUNTY).to_string(),
                published_year: row.str(column::PUBLISHED_YEAR).to_string(),
                basis: Basis::of(&published),
                published,
                count: Count {
                    floor,
                    ceiling: row.str(column::CEILING).parse().ok(),
                },
            }
        })
        .collect()
}

/// The sheet by IRN. District names are not unique; IRNs are.
#[must_use]
pub fn by_irn() -> BTreeMap<String, District> {
    districts()
        .into_iter()
        .map(|district| (district.irn.clone(), district))
        .collect()
}

/// What the sheet comes to, as a range.
///
/// The ceiling is `None` whenever any row is `.` or `NC`, which is every reading of the real
/// file: 25 of the 611 rows are unbounded above, so the sheet has a floor and no ceiling at all.
/// That is the honest answer, and it is why the statewide series is worth holding beside this.
#[must_use]
pub fn total() -> Count {
    let districts = districts();
    Count {
        floor: districts.iter().map(|d| d.count.floor).sum(),
        ceiling: districts
            .iter()
            .map(|d| d.count.ceiling)
            .try_fold(0u32, |sum, ceiling| Some(sum + ceiling?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_district_in_the_sheet_resolved_to_a_distinct_irn() {
        let districts = districts();
        assert_eq!(districts.len(), 611, "the released sheet has 611 Ohio rows");
        let distinct: std::collections::BTreeSet<&String> =
            districts.iter().map(|d| &d.irn).collect();
        assert_eq!(
            distinct.len(),
            districts.len(),
            "the whole point of the join is that no two rows are the same district"
        );
    }

    #[test]
    fn every_irn_is_a_district_the_department_classifies() {
        // The IRNs were resolved against the department's own district list, so this is a check
        // that the two committed fixtures still agree — a typology rebuild that dropped a
        // district would leave an IRN here pointing at nothing.
        let typology = crate::typology::by_irn();
        let sheet = districts();
        let orphans: Vec<&str> = sheet
            .iter()
            .filter(|d| !typology.contains_key(&d.irn))
            .map(|d| d.irn.as_str())
            .collect();
        assert!(orphans.is_empty(), "IRNs in no district list: {orphans:?}");
    }

    #[test]
    fn the_markers_are_three_and_the_codebook_documents_none() {
        let districts = districts();
        let count = |basis: Basis| districts.iter().filter(|d| d.basis == basis).count();
        // The catalog entry recorded two markers and got 564 numeric rows by subtracting 47 from
        // 611. There is a third, and the numeric rows are 559.
        assert_eq!(count(Basis::UnderTen), 27);
        assert_eq!(count(Basis::Blank), 20);
        assert_eq!(count(Basis::NotCollected), 5, "`NC` is the third marker");
        assert_eq!(count(Basis::Reported), 559);
    }

    #[test]
    fn only_the_under_ten_mask_bounds_a_cell_above() {
        for district in districts() {
            match district.basis {
                Basis::Reported => assert_eq!(
                    district.count.exact(),
                    Some(district.count.floor),
                    "{} is a published number and should be a point",
                    district.name
                ),
                Basis::UnderTen => assert_eq!(
                    (district.count.floor, district.count.ceiling),
                    (0, Some(9)),
                    "{} is masked under ten",
                    district.name
                ),
                Basis::NotCollected | Basis::Blank => assert_eq!(
                    (district.count.floor, district.count.ceiling),
                    (0, None),
                    "{} has nothing in the file bounding it above",
                    district.name
                ),
            }
        }
    }

    #[test]
    fn twenty_districts_report_an_explicit_zero_and_one_of_them_is_cincinnati() {
        let zeros: Vec<District> = districts()
            .into_iter()
            .filter(|d| d.basis == Basis::Reported && d.count.floor == 0)
            .collect();
        assert_eq!(
            zeros.len(),
            20,
            "a defect the catalog entry does not record"
        );
        // 34,860 pupils and a published zero. Whether that is a true zero or a fourth unmarked
        // absence is open; what it is not is a censored cell, and nothing downstream should
        // treat it as one.
        let names: Vec<&str> = zeros.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"Cincinnati City"), "{names:?}");
    }

    #[test]
    fn the_sheet_has_a_floor_and_no_ceiling() {
        let total = total();
        assert_eq!(total.floor, PUBLISHED_FLOOR);
        assert_eq!(
            total.ceiling, None,
            "25 rows are unbounded above, so the sheet is too"
        );
    }

    #[test]
    fn the_floor_refutes_the_year_every_row_is_labeled_with() {
        // The workbook's own statewide sheet gives 2021-22 as 47,491. A district file cannot
        // exceed its own state total, and this one does by 3,858 before a single censored cell
        // is counted. That is the whole licence for committing these rows undated.
        const STATEWIDE_2021_22: u32 = 47_491;
        assert!(
            districts()
                .iter()
                .all(|d| d.published_year == PUBLISHED_YEAR),
            "every row carries the publisher's label and nothing else"
        );
        assert!(
            total().floor > STATEWIDE_2021_22,
            "{} against {STATEWIDE_2021_22}",
            total().floor
        );
    }
}
