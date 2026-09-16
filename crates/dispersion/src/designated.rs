//! Which buildings' students may claim a traditional EdChoice scholarship, and why.
//!
//! The department's designated list for 2026-2027: 2,877 buildings across 606 districts, 513 of
//! them designated, each carrying every input to the determination as well as the determination
//! itself.
//!
//! # Eligibility, not participation
//!
//! A designated building is one whose students *may* apply. Nothing here says a scholarship was
//! awarded or used, and the per-district participation route the department once published is
//! withdrawn — see [`crate::deduct`] and the `deduction` skill. So this closes the per-district
//! question in one direction only, and a reader looking for uptake will not find it.
//!
//! # The criteria are the statute's, and they check out
//!
//! R.C. 3310.03(A)(1) makes a student eligible if their building was in the lowest twenty per
//! cent by performance index "for at least two of the three most recent consecutive rankings"
//! *and* is operated by a district where "an average of twenty per cent or more" of its students
//! were in the Title I formula over three consecutive years. Both flags are in the file, and
//! [`Building::meets_stated_option_b`] recomputes the conjunction from them.

use std::collections::BTreeMap;

const FIXTURE: &str = include_str!("../fixtures/edchoice-designated-2627.csv");

const EXPECTED_HEADER: &str = "county,district_irn,district_name,building_irn,building_name,\
grade_levels,open_closed,designated,option_a_academic_distress,option_b_pi_and_title1,\
academic_distress_school,bottom_20_pi_two_of_three,bottom_20_pi_2223,bottom_20_pi_2324,\
bottom_20_pi_2425,title1_at_least_20_three_years,title1_share_2324,title1_share_2425,\
title1_share_2526,title1_average";

/// The share of a district's students in the Title I formula that R.C. 3310.03(A)(1)(b) requires,
/// as a three-year average.
pub const TITLE1_THRESHOLD: f64 = 0.20;

/// One building on the designated list, designated or not.
#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    /// The county the district sits in.
    pub county: String,
    /// The operating district's IRN.
    pub district_irn: String,
    /// The operating district's name. Not unique across districts — key on the IRN.
    pub district_name: String,
    /// The building's IRN.
    pub building_irn: String,
    /// The building's name.
    pub building_name: String,
    /// Grade levels served, semicolon-separated because the source comma-separates them.
    pub grade_levels: String,
    /// `Open`, `Closed` or `Inactive`.
    pub open_closed: String,
    /// Whether students enrolled here may claim a scholarship for 2026-2027.
    pub designated: bool,
    /// Option A: the district is under an academic distress commission.
    pub option_a: bool,
    /// Option B: the performance-index and Title I route.
    pub option_b: bool,
    /// Whether the district is an academic distress district. Identical to [`Self::option_a`] in
    /// every row of this edition.
    pub academic_distress: bool,
    /// Bottom twenty per cent by performance index in two of the three rankings below.
    pub bottom_20_two_of_three: bool,
    /// The three rankings themselves, oldest first: 2022-23, 2023-24, 2024-25.
    pub bottom_20_by_year: [bool; 3],
    /// Whether the district's three-year Title I average is at or above [`TITLE1_THRESHOLD`].
    pub title1_at_least_20: bool,
    /// The three Title I formula shares, oldest first: 2023-24, 2024-25, 2025-26. A district
    /// figure repeated onto each of its buildings.
    pub title1_shares: [f64; 3],
    /// The department's own average of the three. The mean, to the last place, in every row.
    pub title1_average: f64,
}

impl Building {
    /// Whether the building is open, which the workbook requires and the statute's criteria do
    /// not list.
    ///
    /// It follows from the premise R.C. 3310.03(A)(1) puts in front of its two criteria — the
    /// student "is enrolled in a school building operated by the student's resident district" —
    /// which a closed building has nobody to satisfy. Stated here because it is what separates
    /// [`Self::meets_stated_option_b`] from [`Self::option_b`].
    #[must_use]
    pub fn open(&self) -> bool {
        self.open_closed == "Open"
    }

    /// Option B read off its two published criteria alone, ignoring the building's status.
    ///
    /// Disagrees with [`Self::option_b`] on exactly four buildings, all of them closed or
    /// inactive.
    #[must_use]
    pub fn meets_stated_option_b(&self) -> bool {
        self.bottom_20_two_of_three && self.title1_at_least_20
    }

    /// How many of the three rankings put this building in the bottom twenty per cent.
    #[must_use]
    pub fn bottom_20_years(&self) -> usize {
        self.bottom_20_by_year.iter().filter(|flag| **flag).count()
    }
}

/// Every building on the list.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or if a flag cell is neither
/// `yes` nor `no` — both by way of the extractor's own guarantees, which a hand-edited fixture
/// would break.
#[must_use]
pub fn buildings() -> Vec<Building> {
    let flag = |cell: &str| match cell {
        "yes" => true,
        "no" => false,
        other => panic!("the designated list holds {other:?} where a yes/no flag belongs"),
    };
    let share = |cell: Option<f64>| cell.expect("every row carries all three Title I shares");
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .map(|row| Building {
            county: row.str(0).to_string(),
            district_irn: row.str(1).to_string(),
            district_name: row.str(2).to_string(),
            building_irn: row.str(3).to_string(),
            building_name: row.str(4).to_string(),
            grade_levels: row.str(5).to_string(),
            open_closed: row.str(6).to_string(),
            designated: flag(row.str(7)),
            option_a: flag(row.str(8)),
            option_b: flag(row.str(9)),
            academic_distress: flag(row.str(10)),
            bottom_20_two_of_three: flag(row.str(11)),
            bottom_20_by_year: [flag(row.str(12)), flag(row.str(13)), flag(row.str(14))],
            title1_at_least_20: flag(row.str(15)),
            title1_shares: [share(row.num(16)), share(row.num(17)), share(row.num(18))],
            title1_average: share(row.num(19)),
        })
        .collect()
}

/// How many buildings each district has on the list, and how many of them are designated.
#[must_use]
pub fn by_district() -> BTreeMap<String, (usize, usize)> {
    let mut out: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for building in buildings() {
        let entry = out.entry(building.district_irn).or_default();
        entry.0 += 1;
        entry.1 += usize::from(building.designated);
    }
    out
}

/// The date the designated list workbook was authored, from its own `docProps/core.xml`.
///
/// `dcterms:created` and `dcterms:modified` both fall on it, thirty-four minutes apart, so the
/// file was made in one sitting and has not been revised since. Every other column in it is an
/// arithmetic on the sheets beside it and carries no date of its own; the academic-distress column
/// is a fact about the world on *this* date, which is why it can go stale while the rest cannot.
/// See [`ACADEMIC_DISTRESS_DISTRICT_IRNS`].
pub const AUTHORED: &str = "2025-11-13";

/// The two districts this edition flags as subject to R.C. 3302.10.
///
/// East Cleveland City and Youngstown City. On [`AUTHORED`] both were. By the time the 2026-2027
/// school year this list governs began, only Youngstown was: East Cleveland's commission ceased to
/// exist in late December 2025, when the director released the district and R.C. 3302.10(N)(1)
/// ended the commission with the transition period.
pub const ACADEMIC_DISTRESS_DISTRICT_IRNS: [&str; 2] = [EAST_CLEVELAND_IRN, YOUNGSTOWN_IRN];

/// East Cleveland City's IRN. Flagged, and released from its commission six weeks after
/// [`AUTHORED`].
pub const EAST_CLEVELAND_IRN: &str = "043901";

/// Youngstown City's IRN. The one commission still in existence, and the only district with
/// buildings resting on the academic-distress route alone.
pub const YOUNGSTOWN_IRN: &str = "045161";

/// Lorain City's IRN, which is `044263` — `047076` is Pettisville Local, a different district in a
/// different county.
///
/// The third district ever to hold a commission, and the control on whether this column is
/// maintained rather than carried forward: H.B. 33 dissolved Lorain's commission on 4 July 2023,
/// and the flag is `no` in all fifteen of Lorain's rows.
pub const LORAIN_IRN: &str = "044263";

/// The IRN of the one district the designated list does not cover.
///
/// Cleveland Municipal, and by statute rather than by omission: R.C. 3310.03 makes a student
/// ineligible whose "resident district is not a school district in which the pilot project
/// scholarship program is operating", and directs that the building ranking "shall not include
/// buildings operated by" such a district. Cleveland is the pilot project district, with the
/// scholarship programme the corpus holds as `cleveland-scholarship`.
pub const PILOT_PROJECT_DISTRICT_IRN: &str = "043786";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_is_the_size_the_catalog_records() {
        let all = buildings();
        assert_eq!(all.len(), 2_877);
        assert_eq!(all.iter().filter(|b| b.designated).count(), 513);
        assert_eq!(by_district().len(), 606);
    }

    #[test]
    fn a_grade_list_keeps_its_boundaries() {
        let split = buildings()
            .into_iter()
            .filter(|b| b.grade_levels.contains(';'))
            .count();
        assert_eq!(split, 602, "the comma substitution lost a boundary");
    }

    #[test]
    fn the_pilot_project_district_is_the_one_that_is_absent() {
        assert!(!by_district().contains_key(PILOT_PROJECT_DISTRICT_IRN));
    }
}
