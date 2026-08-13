//! Which Ohio education agencies existed in which year, and why that is all this can say.
//!
//! # What this is for
//!
//! Every other fixture here measures something. This one measures *membership*: the federal
//! directory's Ohio slice for sixteen school years, 2008-09 through 2023-24, one row per agency
//! per year. It exists because a panel spanning years is a panel whose members change, and
//! [`crate::ohio_panel`] had been resolving every agency's Ohio number through a **single**
//! directory year — so an agency that closed before 2023 had no number at all, and the module
//! said so in a docstring that then drew the wrong conclusion from it.
//!
//! # The wrong conclusion, and the right one
//!
//! That docstring read: *"the count going from 124 in FY2012 to 0 in FY2022 is the consolidation
//! history, measured."* All 124 are nameable from the contemporaneous 2011-12 file, and when you
//! name them, 121 are community schools. Across this whole window **341 agencies leave the
//! directory: 327 community schools, nine service centres, five regular districts.** Two of the
//! five are STEM schools, which Ohio funds as their own unit under R.C. 3326 and which the federal
//! directory types as regular districts.
//!
//! So the number was measuring charter churn and being read as district consolidation, at
//! something like a hundred to one. The three genuine district consolidations in sixteen years are
//! Bettsville Local, Ledgemont Local and Newbury Local, and [`departures`] names them.
//!
//! # Why no departure carries a reason
//!
//! The CCD has eight operational-status codes and exactly one marks a consolidation — code 5,
//! *"significant change in geographic boundaries or instructional responsibility"*. **Ohio has
//! never used it.** Zero occurrences in 17,618 Ohio agency-years.
//!
//! What Ohio files instead, for all 341 departures without a single exception, is code 2:
//! *"closed with no effect on another agency's boundaries."* That is not the source declining to
//! answer. It is the source answering wrongly, about districts whose territory demonstrably went
//! to a neighbour — Bettsville's to Old Fort, Ledgemont's to Berkshire, Newbury's split between
//! West Geauga and Chardon. And the receiving agencies' rows do not change at all: they are coded
//! `1 Open` in the year before, the year of, and the year after.
//!
//! Nothing here reads a reason out of that, and the alternatives were tested rather than assumed.
//! Enrolment absorption on the survivor is confounded roughly fifty to one by ordinary growth,
//! and it gets the sign wrong on Newbury — Chardon *lost* pupils the year it received them. Name
//! changes on the survivor arrive up to two years out of alignment with the closure and share a
//! vocabulary with cosmetic re-spellings. [`consolidations_marked`] holds the negative finding
//! itself, so the day Ohio starts filing code 5 this module fails rather than stays quiet.
//!
//! Settling the reason needs Ohio's own record — the territory-transfer orders under R.C. 3311 —
//! which is the same answer
//! [`state-foundation-aid`](../../../.yidam/corpus/revenue-stream/state-foundation-aid.yml)
//! reached about the disappearing appropriation lines, for the same reason.

use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = include_str!("../fixtures/ccd-lea-directory.csv");

const EXPECTED_HEADER: &str = "school_year,leaid,irn,name,agency_type,status";

/// The first school year the directory is held for, as the year it opens in.
pub const FIRST_YEAR: u16 = 2008;

/// The last, which is the latest NCES has published.
pub const LAST_YEAR: u16 = 2023;

/// The status code the CCD defines for a consolidation, and Ohio has never filed.
pub const CONSOLIDATION_CODE: &str = "5";

/// The status code Ohio files for every departure instead.
pub const CLOSED_CODE: &str = "2";

/// One Ohio agency as one school year's directory describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agency {
    /// The school year, as the calendar year it opens in: 2008 for 2008-09.
    pub opens: u16,
    /// The NCES agency identifier, which is stable across renames and is the only key that is.
    pub leaid: String,
    /// Ohio's own six-digit IRN, with the `OH-` prefix the later years add already stripped.
    pub irn: String,
    /// The agency's name that year.
    pub name: String,
    /// The CCD agency type. `1` regular district, `4` service agency, `5` state agency,
    /// `7` independent charter district — which is what Ohio's community schools are filed as.
    pub agency_type: String,
    /// The operational status code, verbatim. See the module note on what it does not mean.
    pub status: String,
}

impl Agency {
    /// Whether the federal directory files this as a regular school district.
    ///
    /// Not the same as Ohio's own idea of one: two of the five regular districts that leave this
    /// window are STEM schools, which Ohio funds as a separate unit under R.C. 3326.
    #[must_use]
    pub fn is_regular_district(&self) -> bool {
        self.agency_type == "1"
    }

    /// Whether it is one of Ohio's community schools.
    #[must_use]
    pub fn is_community_school(&self) -> bool {
        self.agency_type == "7"
    }
}

/// Every row of the directory panel.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against.
#[must_use]
pub fn panel() -> Vec<Agency> {
    let mut lines = FIXTURE.lines();
    assert_eq!(
        lines.next().unwrap_or_default().trim(),
        EXPECTED_HEADER,
        "the CCD directory fixture header changed; update dispersion::lea_directory"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').map(str::trim).collect();
            Some(Agency {
                opens: f.first()?.parse().ok()?,
                leaid: f.get(1)?.to_string(),
                irn: f.get(2)?.to_string(),
                name: f.get(3)?.to_string(),
                agency_type: f.get(4)?.to_string(),
                status: f.get(5)?.to_string(),
            })
        })
        .collect()
}

/// Agency identifier to Ohio IRN, as each school year states it.
///
/// The join [`crate::ohio_panel`] uses. Keyed on the year as well as the agency because that is
/// the whole point: asking the 2022-23 file about a district that closed in 2015 gets nothing,
/// and asking the 2014-15 file gets the answer.
#[must_use]
pub fn irn_by_year() -> BTreeMap<(u16, String), String> {
    panel()
        .into_iter()
        .map(|a| ((a.opens, a.leaid), a.irn))
        .collect()
}

/// The IRN an agency carried in the most recent year the directory names it.
///
/// For resolving a panel row when the contemporaneous directory year is not held — the F-33
/// reports a fiscal year and the directory a school year, and the two do not align at the edges.
#[must_use]
pub fn last_known_irn() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for agency in panel() {
        out.insert(agency.leaid, agency.irn);
    }
    out
}

/// An agency that stops appearing in the directory, with what the last year to name it said.
///
/// **There is no reason field and that is deliberate.** See the module note: the one code that
/// would carry a reason is unused by Ohio, and the code Ohio does file asserts the negation of
/// the only reading anyone would want.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Departure {
    /// The last school year the directory carries it.
    pub last_year: u16,
    /// The agency identifier.
    pub leaid: String,
    /// Its Ohio IRN in that year.
    pub irn: String,
    /// Its name in that year.
    pub name: String,
    /// Its type in that year.
    pub agency_type: String,
    /// The operational status it was filed under in that year.
    pub terminal_status: String,
}

/// Every agency that leaves the directory inside this window, oldest departure first.
///
/// An agency absent from [`LAST_YEAR`] has left. Departure is measured from the *last* year that
/// names it rather than from the first that does not, because agencies come back: a handful of
/// Ohio's have a single missing year and then reappear, and reading the gap as an exit would
/// invent a departure and a founding out of one file.
#[must_use]
pub fn departures() -> Vec<Departure> {
    let mut last: BTreeMap<String, Agency> = BTreeMap::new();
    for agency in panel() {
        last.insert(agency.leaid.clone(), agency);
    }
    let mut out: Vec<Departure> = last
        .into_values()
        .filter(|a| a.opens < LAST_YEAR)
        .map(|a| Departure {
            last_year: a.opens,
            leaid: a.leaid,
            irn: a.irn,
            name: a.name,
            agency_type: a.agency_type,
            terminal_status: a.status,
        })
        .collect();
    out.sort_by(|a, b| {
        (a.last_year, &a.leaid)
            .partial_cmp(&(b.last_year, &b.leaid))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

/// Departures by the agency type they were filed under in their last year.
///
/// The number this module exists to publish. A count of departures with no type on it reads as
/// districts disappearing; the composition says it is community schools opening and closing.
#[must_use]
pub fn departures_by_type() -> BTreeMap<String, usize> {
    let mut out: BTreeMap<String, usize> = BTreeMap::new();
    for departure in departures() {
        *out.entry(departure.agency_type).or_default() += 1;
    }
    out
}

/// Agency-years Ohio has filed under the CCD's consolidation code. There are none.
///
/// Held as a function rather than as a comment so that the day Ohio starts populating the field,
/// a test fails and this module's whole account of itself is revisited — rather than the corpus
/// going on saying the question is unanswerable after the answer arrives.
#[must_use]
pub fn consolidations_marked() -> Vec<Agency> {
    panel()
        .into_iter()
        .filter(|a| a.status == CONSOLIDATION_CODE)
        .collect()
}

/// The school years held, oldest first.
#[must_use]
pub fn years() -> Vec<u16> {
    panel()
        .into_iter()
        .map(|a| a.opens)
        .collect::<BTreeSet<u16>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_school_year_is_present_and_none_is_doubled() {
        assert_eq!(years(), (FIRST_YEAR..=LAST_YEAR).collect::<Vec<u16>>());
        for year in years() {
            let rows: Vec<Agency> = panel().into_iter().filter(|a| a.opens == year).collect();
            let distinct: BTreeSet<&str> = rows.iter().map(|a| a.leaid.as_str()).collect();
            assert_eq!(
                rows.len(),
                distinct.len(),
                "{year}-{:02} names an agency twice",
                (year + 1) % 100
            );
            assert!(
                (900..=1400).contains(&rows.len()),
                "{year}-{:02} has {} Ohio agencies",
                (year + 1) % 100,
                rows.len()
            );
        }
    }

    /// The join the whole panel rests on, held to the property that makes it a join.
    #[test]
    fn every_agency_carries_a_six_digit_irn_in_every_year() {
        for agency in panel() {
            assert_eq!(
                agency.irn.len(),
                6,
                "{} in {} has the IRN {:?}",
                agency.name,
                agency.opens,
                agency.irn
            );
            assert!(agency.irn.chars().all(|c| c.is_ascii_digit()));
        }
    }

    /// The finding. Ohio has never once filed the code that would answer the question.
    #[test]
    fn ohio_has_never_filed_a_consolidation() {
        let marked = consolidations_marked();
        assert!(
            marked.is_empty(),
            "Ohio has filed {} agency-years under the consolidation code, and this module's \
             account of what the source cannot say is now out of date: {:?}",
            marked.len(),
            marked.iter().map(|a| &a.name).collect::<Vec<_>>()
        );
        // And the panel is large enough for that to mean something rather than being a small
        // sample that happens to miss a rare code.
        assert!(panel().len() > 17_000, "{} agency-years", panel().len());
    }

    /// And the code it does file says the opposite of the only reading anyone would want.
    #[test]
    fn every_departure_is_filed_as_a_closure_with_no_effect_on_anyone() {
        let departures = departures();
        assert!(!departures.is_empty());
        for departure in &departures {
            assert_eq!(
                departure.terminal_status, CLOSED_CODE,
                "{} left in {} under status {}, and this module claims they all leave under 2",
                departure.name, departure.last_year, departure.terminal_status
            );
        }
    }

    /// What the count is actually a count of.
    #[test]
    fn the_departures_are_overwhelmingly_community_schools() {
        let by_type = departures_by_type();
        let total: usize = by_type.values().sum();
        let community = by_type.get("7").copied().unwrap_or(0);
        let districts = by_type.get("1").copied().unwrap_or(0);
        assert!(
            community * 10 > total * 9,
            "community schools are {community} of {total} departures, and the claim that this \
             count is charter churn rather than consolidation rests on that ratio"
        );
        assert!(
            districts < 10,
            "{districts} regular districts left the directory in sixteen years"
        );
    }

    /// The three that are genuinely district consolidations, named.
    ///
    /// Named rather than counted because three is small enough to name, and because naming them
    /// is the only thing that makes the refusal to classify honest: the corpus says the source
    /// cannot give a reason, and a reader can go and find one for each of these in a county
    /// record.
    #[test]
    fn the_district_departures_are_the_five_known_ones() {
        let districts: Vec<Departure> = departures()
            .into_iter()
            .filter(|d| d.agency_type == "1")
            .collect();
        let named: Vec<(&str, &str, u16)> = districts
            .iter()
            .map(|d| (d.irn.as_str(), d.name.as_str(), d.last_year))
            .collect();
        assert_eq!(
            named,
            vec![
                ("049692", "Bettsville Local", 2014),
                ("015328", "Collins Career Center STEM Academy", 2016),
                ("047209", "Ledgemont Local", 2016),
                ("014877", "Metro Institute of Technology", 2017),
                ("047217", "Newbury Local", 2020),
            ],
            "the regular-district departures have changed"
        );
    }

    /// The receiving districts are still open and their rows say nothing happened.
    #[test]
    fn the_agencies_that_took_the_pupils_are_unmarked() {
        // Old Fort took Bettsville, Berkshire took Ledgemont, and Newbury split between West
        // Geauga and Chardon. Every one of them is coded `1 Open` throughout, which is why no
        // rule over this source can find them.
        let survivors = ["3910021", "3904716", "3904722", "3904718"];
        let panel = panel();
        for leaid in survivors {
            let rows: Vec<&Agency> = panel.iter().filter(|a| a.leaid == leaid).collect();
            assert_eq!(
                rows.len(),
                (LAST_YEAR - FIRST_YEAR + 1) as usize,
                "{leaid} is not in every year"
            );
            assert!(
                rows.iter().all(|a| a.status == "1"),
                "{leaid} carries a status other than open in some year"
            );
        }
    }

    /// A gap is not a departure, and the panel has gaps.
    #[test]
    fn an_agency_that_comes_back_is_not_recorded_as_having_left() {
        let mut seen: BTreeMap<String, Vec<u16>> = BTreeMap::new();
        for agency in panel() {
            seen.entry(agency.leaid).or_default().push(agency.opens);
        }
        let broken: Vec<&String> = seen
            .iter()
            .filter(|(_, years)| {
                years.last().copied().unwrap_or(0) - years[0] + 1 != years.len() as u16
            })
            .map(|(leaid, _)| leaid)
            .collect();
        let departed: BTreeSet<String> = departures().into_iter().map(|d| d.leaid).collect();
        for leaid in &broken {
            let years = &seen[*leaid];
            if years.last().copied().unwrap_or(0) == LAST_YEAR {
                assert!(
                    !departed.contains(*leaid),
                    "{leaid} has a gap at {years:?} and is recorded as having departed"
                );
            }
        }
    }
}
