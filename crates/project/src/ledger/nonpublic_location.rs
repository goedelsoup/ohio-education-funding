//! The school district each chartered nonpublic school sits in, as of the department's directory.
//!
//! R.C. 3317.024(E)(1) pays auxiliary services through the district a chartered nonpublic school
//! is located in, and [`super::nonpublic_enrolment`] cannot say which district that is: no October
//! file in any era carries the column. This is the column, built by
//! `connect::fixtures::nonpublic_location` from three publications joined on keys — the department's
//! directory gives
//! each open school's address against its building IRN, the Census Bureau's geocoder places the
//! address in a 2020 block, and the 2020 block assignment file says which district the block lay
//! in. The decision record `nonpublic-school-location` says why that is not the geographic match
//! `nonpublic-enrollment-connector` rejected.
//!
//! # A snapshot
//!
//! Every row is a school open when the directory was generated, at the address it had then.
//! Joining these to an earlier October places the buildings that still exist where they are now,
//! which for a school that moved is not where it was. [`locations`] holds no year because the
//! fixture describes one day, not a series.
//!
//! # What a blank district means
//!
//! The geocoder did not place the address: [`Location::geocoder_match`] says `No_Match` or `Tie`.
//! It is not a school outside every district. Those rows are kept so that a count of the placed
//! is always read beside the set that was not.
//!
//! # The district is 2020's
//!
//! The block file describes the 2020 census, so [`Location::district_irn`] is the district the
//! block lay in then. One school, St Helen in Newbury, sits in a district that ceased on July 1,
//! 2020; [`Location::directory_year`] is the last CCD year that listed the district, and a reader
//! wanting the present district resolves it against the territory-transfer record.

use std::sync::OnceLock;

use edfund_core::csv;

/// The committed fixture: one row per open nonpublic school.
const LOCATIONS: &str = include_str!("../../fixtures/nonpublic-school-district.csv");

/// The header [`locations`] indexes against.
const HEADER: &str = "irn,school,county,geocoder_match,block,district_irn,district,directory_year";

/// The columns of [`HEADER`], named where they are read.
mod column {
    pub const IRN: usize = 0;
    pub const SCHOOL: usize = 1;
    pub const COUNTY: usize = 2;
    pub const GEOCODER_MATCH: usize = 3;
    pub const BLOCK: usize = 4;
    pub const DISTRICT_IRN: usize = 5;
    pub const DISTRICT: usize = 6;
    pub const DIRECTORY_YEAR: usize = 7;
}

/// One open nonpublic school and the district its address lies in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// Six-digit building IRN, the key [`super::nonpublic_enrolment::Building::irn`] shares.
    pub irn: String,
    /// The directory's name for it. Not a key: names repeat.
    pub school: String,
    /// The directory's designated county.
    pub county: String,
    /// What the geocoder said: `Exact`, `Non_Exact`, `No_Match` or `Tie`.
    pub geocoder_match: String,
    /// The fifteen-digit 2020 census block, when placed.
    pub block: Option<String>,
    /// The IRN of the district the block lay in at the 2020 census, when placed.
    pub district_irn: Option<String>,
    /// That district's name in the last CCD year listing it.
    pub district: Option<String>,
    /// The last CCD directory year that listed the district. Earlier than the directory's latest
    /// year only for a district that has since ceased.
    pub directory_year: Option<u16>,
}

impl Location {
    /// Whether the geocoder placed the address in a block.
    #[must_use]
    pub fn is_placed(&self) -> bool {
        self.district_irn.is_some()
    }
}

/// Every school in the directory, in IRN order.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against.
#[must_use]
pub fn locations() -> &'static [Location] {
    static ROWS: OnceLock<Vec<Location>> = OnceLock::new();
    ROWS.get_or_init(|| {
        let optional = |text: &str| (!text.is_empty()).then(|| text.to_string());
        csv::rows(LOCATIONS, HEADER)
            .map(|row| Location {
                irn: row.str(column::IRN).to_string(),
                school: row.str(column::SCHOOL).to_string(),
                county: row.str(column::COUNTY).to_string(),
                geocoder_match: row.str(column::GEOCODER_MATCH).to_string(),
                block: optional(row.str(column::BLOCK)),
                district_irn: optional(row.str(column::DISTRICT_IRN)),
                district: optional(row.str(column::DISTRICT)),
                directory_year: row.str(column::DIRECTORY_YEAR).parse().ok(),
            })
            .collect()
    })
}

/// The school with building IRN `irn`, if the directory lists it.
#[must_use]
pub fn location(irn: &str) -> Option<&'static Location> {
    locations()
        .binary_search_by(|row| row.irn.as_str().cmp(irn))
        .ok()
        .map(|at| &locations()[at])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixture_is_in_irn_order_with_one_row_a_school() {
        // `location` binary-searches, which is only sound on a sorted, unique key.
        assert!(locations().windows(2).all(|pair| pair[0].irn < pair[1].irn));
    }

    #[test]
    fn a_placed_school_has_a_block_a_district_and_a_year() {
        for row in locations() {
            assert_eq!(row.block.is_some(), row.is_placed(), "{row:?}");
            assert_eq!(row.directory_year.is_some(), row.is_placed(), "{row:?}");
            assert_eq!(
                row.is_placed(),
                matches!(row.geocoder_match.as_str(), "Exact" | "Non_Exact"),
                "{row:?}"
            );
        }
    }
}
