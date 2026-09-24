//! Three editions of the designated list, and the change between them.
//!
//! The department publishes one designated list at a time and deletes the last, so eligibility
//! has been a snapshot in this repository: [`crate::designated`] reads 2026-2027 and nothing it
//! can be compared against. The two editions before it survive only in the Internet Archive, and
//! with them the first per-district *change* in scholarship eligibility this corpus can measure.
//!
//! # Why this module is narrower than [`crate::designated`]
//!
//! It carries the columns all three editions share and no more. The editions do not share a
//! layout: each names its Title I vintages in its own headings, and 2024-2025 ranks **two**
//! Performance Index years where the other two rank three, so there is no common set of
//! per-year columns to put in one record. What every edition does carry is the determination
//! and its two routes, which is what a question about change is asking about.
//!
//! # What a difference between two editions is, and is not
//!
//! A building can leave the designated set two ways: it can stop meeting the criteria, or it can
//! stop being on the list at all. Those are different events — the second is usually a closure —
//! and telling them apart needs both editions' *whole* lists, designated buildings and not. That
//! is why the extractor keeps all 2,877 rows rather than the 513 designations, and why
//! [`Change`] distinguishes a building that was delisted from one that was merely undesignated.

use std::collections::{BTreeMap, BTreeSet};

/// One published edition of the designated list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Edition {
    /// 2024-2025, from the Internet Archive. The edition whose ranking window is two years.
    Y2425,
    /// 2025-2026, from the Internet Archive.
    Y2526,
    /// 2026-2027, the edition the department currently serves.
    Y2627,
}

/// The editions in publication order, oldest first.
pub const EDITIONS: [Edition; 3] = [Edition::Y2425, Edition::Y2526, Edition::Y2627];

const FIXTURE_2425: &str = include_str!("../fixtures/edchoice-designated-2425.csv");
const FIXTURE_2526: &str = include_str!("../fixtures/edchoice-designated-2526.csv");
const FIXTURE_2627: &str = include_str!("../fixtures/edchoice-designated-2627.csv");

const HEADER_2425: &str = "county,district_irn,district_name,building_irn,building_name,\
grade_levels,open_closed,designated,option_a_academic_distress,option_b_pi_and_title1,\
academic_distress_school,bottom_20_pi_two_of_two,bottom_20_pi_2122,bottom_20_pi_2223,\
title1_at_least_20_three_years,title1_share_2122,title1_share_2223,title1_share_2324,\
title1_average";

const HEADER_2526: &str = "county,district_irn,district_name,building_irn,building_name,\
grade_levels,open_closed,designated,option_a_academic_distress,option_b_pi_and_title1,\
academic_distress_school,bottom_20_pi_two_of_three,bottom_20_pi_2122,bottom_20_pi_2223,\
bottom_20_pi_2324,title1_at_least_20_three_years,title1_share_2223,title1_share_2324,\
title1_share_2425,title1_average";

const HEADER_2627: &str = "county,district_irn,district_name,building_irn,building_name,\
grade_levels,open_closed,designated,option_a_academic_distress,option_b_pi_and_title1,\
academic_distress_school,bottom_20_pi_two_of_three,bottom_20_pi_2223,bottom_20_pi_2324,\
bottom_20_pi_2425,title1_at_least_20_three_years,title1_share_2324,title1_share_2425,\
title1_share_2526,title1_average";

impl Edition {
    /// The school year the edition governs.
    #[must_use]
    pub fn school_year(self) -> &'static str {
        match self {
            Edition::Y2425 => "2024-2025",
            Edition::Y2526 => "2025-2026",
            Edition::Y2627 => "2026-2027",
        }
    }

    /// How many Performance Index rankings the edition's bottom-20% window is cut from.
    ///
    /// R.C. 3310.03(A)(1)(a) asks for "at least two of the three most recent consecutive
    /// rankings". 2024-2025 has two rankings, and its own column heading names them —
    /// `Bottom 20% PI Ranking Across 2022 and 2023 School Years` — so in that edition the test
    /// is two of two, which is a materially easier one.
    #[must_use]
    pub fn ranking_years(self) -> usize {
        match self {
            Edition::Y2425 => 2,
            Edition::Y2526 | Edition::Y2627 => 3,
        }
    }

    fn fixture(self) -> (&'static str, &'static str) {
        match self {
            Edition::Y2425 => (FIXTURE_2425, HEADER_2425),
            Edition::Y2526 => (FIXTURE_2526, HEADER_2526),
            Edition::Y2627 => (FIXTURE_2627, HEADER_2627),
        }
    }

    /// Index of the last two columns, which sit one place earlier in the narrower edition.
    fn tail(self) -> (usize, usize) {
        match self {
            Edition::Y2425 => (14, 18),
            Edition::Y2526 | Edition::Y2627 => (15, 19),
        }
    }
}

/// One building's determination in one edition, in the columns every edition carries.
#[derive(Debug, Clone, PartialEq)]
pub struct Determination {
    /// The edition this row came from.
    pub edition: Edition,
    /// The operating district's IRN. Districts share names, so this is the key.
    pub district_irn: String,
    /// The operating district's name as that edition spells it.
    pub district_name: String,
    /// The building's IRN.
    pub building_irn: String,
    /// The building's name as that edition spells it.
    pub building_name: String,
    /// `Open`, `Closed` or `Inactive`.
    pub open_closed: String,
    /// Whether students enrolled here may claim a scholarship that year.
    pub designated: bool,
    /// Option A: the district is under an academic distress commission.
    pub option_a: bool,
    /// Option B: the performance-index and Title I route.
    pub option_b: bool,
    /// Bottom twenty per cent by performance index in two of the edition's ranking years.
    pub bottom_20_two_of_window: bool,
    /// Each of the edition's ranking years, oldest first. `None` where the building was not
    /// ranked that year at all, which the archived editions write as a blank and the current
    /// one never does — see [`Edition::ranking_years`].
    pub bottom_20_by_year: Vec<Option<bool>>,
    /// Whether the district's three-year Title I average is at or above twenty per cent.
    pub title1_at_least_20: bool,
    /// The edition's three Title I formula shares, oldest first. A district figure repeated
    /// onto each of its buildings.
    pub title1_shares: Vec<f64>,
    /// The department's own average of the three shares beside it.
    pub title1_average: f64,
}

impl Determination {
    /// Whether the building is open, which the workbook requires and the statute does not list.
    #[must_use]
    pub fn open(&self) -> bool {
        self.open_closed == "Open"
    }

    /// Option B read off its two published criteria alone, ignoring the building's status.
    #[must_use]
    pub fn meets_stated_option_b(&self) -> bool {
        self.bottom_20_two_of_window && self.title1_at_least_20
    }

    /// How many of the edition's ranking years put this building in the bottom twenty per cent.
    ///
    /// A year the building was not ranked in counts as a year it was not in the bottom fifth,
    /// which is how the department's own window column reads it in every row of every edition.
    #[must_use]
    pub fn bottom_20_years(&self) -> usize {
        self.bottom_20_by_year
            .iter()
            .filter(|year| **year == Some(true))
            .count()
    }

    /// The years the building was not ranked at all.
    #[must_use]
    pub fn unranked_years(&self) -> usize {
        self.bottom_20_by_year
            .iter()
            .filter(|y| y.is_none())
            .count()
    }
}

/// A building, keyed the way the list keys it.
pub type BuildingKey = (String, String);

/// Every building in one edition.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or if a flag cell is neither
/// `yes` nor `no`.
#[must_use]
pub fn edition(which: Edition) -> Vec<Determination> {
    let flag = |cell: &str| match cell {
        "yes" => true,
        "no" => false,
        other => panic!("the designated list holds {other:?} where a yes/no flag belongs"),
    };
    let ranked = |cell: &str| match cell {
        "" => None,
        other => Some(flag(other)),
    };
    let (text, header) = which.fixture();
    let (title1_flag, average) = which.tail();
    edfund_core::csv::rows(text, header)
        .map(|row| Determination {
            edition: which,
            district_irn: row.str(1).to_string(),
            district_name: row.str(2).to_string(),
            building_irn: row.str(3).to_string(),
            building_name: row.str(4).to_string(),
            open_closed: row.str(6).to_string(),
            designated: flag(row.str(7)),
            option_a: flag(row.str(8)),
            option_b: flag(row.str(9)),
            bottom_20_two_of_window: flag(row.str(11)),
            bottom_20_by_year: (12..12 + which.ranking_years())
                .map(|i| ranked(row.str(i)))
                .collect(),
            title1_at_least_20: flag(row.str(title1_flag)),
            title1_shares: (title1_flag + 1..average)
                .map(|i| {
                    row.num(i)
                        .expect("every row carries all three Title I shares")
                })
                .collect(),
            title1_average: row
                .num(average)
                .expect("every row carries a Title I average"),
        })
        .collect()
}

/// One edition keyed on district and building IRN.
#[must_use]
pub fn keyed(which: Edition) -> BTreeMap<BuildingKey, Determination> {
    edition(which)
        .into_iter()
        .map(|d| ((d.district_irn.clone(), d.building_irn.clone()), d))
        .collect()
}

/// What happened to one building between two editions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Change {
    /// Not designated before, designated after.
    Entered,
    /// Designated before, not designated after, and still on the list.
    Undesignated,
    /// Designated before, and absent from the later edition altogether.
    Delisted,
}

/// Every building whose designation changed between two editions, and how.
///
/// Keyed on district and building IRN, because 607 districts share 580 names and a building name
/// is not unique either.
#[must_use]
pub fn changes(before: Edition, after: Edition) -> BTreeMap<BuildingKey, Change> {
    let (old, new) = (keyed(before), keyed(after));
    let mut out = BTreeMap::new();
    for (key, was) in &old {
        match new.get(key) {
            Some(now) if was.designated && !now.designated => {
                out.insert(key.clone(), Change::Undesignated);
            }
            None if was.designated => {
                out.insert(key.clone(), Change::Delisted);
            }
            _ => {}
        }
    }
    for (key, now) in &new {
        if now.designated && !old.get(key).is_some_and(|was| was.designated) {
            out.insert(key.clone(), Change::Entered);
        }
    }
    out
}

/// How many buildings each change covers, between two editions.
#[must_use]
pub fn tally(before: Edition, after: Edition) -> BTreeMap<Change, usize> {
    let mut out = BTreeMap::new();
    for change in changes(before, after).into_values() {
        *out.entry(change).or_default() += 1;
    }
    out
}

/// The districts holding at least one designated building, in one edition.
#[must_use]
pub fn districts_with_a_designation(which: Edition) -> BTreeSet<String> {
    edition(which)
        .into_iter()
        .filter(|d| d.designated)
        .map(|d| d.district_irn)
        .collect()
}
