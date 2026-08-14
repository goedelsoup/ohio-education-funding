//! Which Ohio education agencies existed in which year, and why that is all this can say.
//!
//! # What this is for
//!
//! Every other fixture here measures something. This one measures *membership*: the federal
//! directory's Ohio slice for thirty school years, 1994-95 through 2023-24, one row per agency
//! per year. It exists because a panel spanning years is a panel whose members change, and
//! [`crate::ohio_panel`] had been resolving every agency's Ohio number through a **single**
//! directory year — so an agency that closed before 2023 had no number at all, and the module
//! said so in a docstring that then drew the wrong conclusion from it.
//!
//! # The wrong conclusion, and the right one
//!
//! That docstring read: *"the count going from 124 in FY2012 to 0 in FY2022 is the consolidation
//! history, measured."* All 124 are nameable from the contemporaneous 2011-12 file, and when you
//! name them, 121 are community schools. Across thirty years **689 agencies leave the directory:
//! 616 community schools, 66 service agencies, five regular districts, one local district and one
//! state agency.** Two of the five regular districts are STEM schools, which Ohio funds as their
//! own unit under R.C. 3326 and which the federal directory types as districts.
//!
//! So the number was measuring charter churn and being read as district consolidation, at
//! something like a hundred to one. The three genuine district consolidations are Bettsville
//! Local, Ledgemont Local and Newbury Local, and [`departures`] names them.
//!
//! # Why no departure carries a reason
//!
//! The CCD has eight operational-status codes and exactly one marks a consolidation — code 5,
//! *"significant change in geographic boundaries or instructional responsibility"*. **Ohio has
//! never used it.** Zero occurrences in 30,655 Ohio agency-years, which is every agency-year the
//! directory has published since 1994-95.
//!
//! What Ohio files instead, for all 689 departures without a single exception, is code 2:
//! *"closed with no effect on another agency's boundaries."* That is not the source declining to
//! answer. It is the source answering wrongly, about districts whose territory went to a
//! neighbour — Bettsville's to Old Fort, Ledgemont's to Berkshire, Newbury's to West Geauga. And
//! the receiving agencies' rows do not change at all: they are coded `1 Open` in the year before,
//! the year of, and the year after.
//!
//! Nothing here reads a reason out of that, and the alternatives were tested rather than assumed.
//! Enrolment absorption on the survivor is confounded roughly fifty to one by ordinary growth.
//! Name changes on the survivor arrive up to two years out of alignment with the closure and share
//! a vocabulary with cosmetic re-spellings. [`consolidations_marked`] holds the negative finding
//! itself, so the day Ohio starts filing code 5 this module fails rather than stays quiet.
//!
//! # What the fourteen older years added, which was not what was expected of them
//!
//! They were scoped out once as *"a reader built ahead of its reader"*: the finance panel starts
//! at FY2009 and nothing consumed a directory year before 2008-09. What they actually carry is
//! the record of **the bodies that issue the transfer orders**.
//!
//! R.C. 3311.22 vests a territory transfer in an educational service center's governing board.
//! Ohio did not have educational service centers in 1994-95. It had **86 county boards of
//! education**, and this directory watches thirty-nine of them leave between 1995-96 and 2001-02,
//! in ones and fives, while multi-county centres appear beside them with the departed counties'
//! names in their own — Athens and Meigs leave, Athens-Meigs arrives; Ross and Pike leave,
//! Ross-Pike arrives; Guernsey, Monroe and Noble leave, Guernsey-Monroe-Noble arrives. Fourteen
//! such centres join, so the category holds 61 by 2001-02 rather than 47, and 60 of those are
//! recoded into the service-agency type in 2002-03 while the last, Carroll-Harrison, leaves.
//!
//! **Every one of the forty is filed under code 2.** So the defect this module was
//! written about is not particular to school districts and did not start in the 2010s. The
//! register says "closed with no effect on another agency's boundaries" about the consolidation
//! of the very bodies whose minute books hold the orders it cannot describe.
//!
//! The last of the sixty-six is the one this repository already holds an instrument for. Geauga
//! County ESC's final audit recites its own merger with Lake County ESC into the Educational
//! Service Center of the Western Reserve, on 7 November 2019. In the directory, Geauga County ESC
//! is filed closed with no effect on anyone; Lake County ESC keeps its agency identifier, keeps
//! IRN 047860, is coded `1 Open` in the year before, the year of and the year after, and simply
//! takes the new name in 2020-21. That is the district case exactly, one level up.
//!
//! The county-to-compound-name correspondence is legible and is **not committed as a fact**. It
//! is a reading of a name field, the timing is up to two years out of alignment, and the same
//! field flips a survivor between "Guernsey-Monroe-Noble" and "Ohio Valley" and back over four
//! years. It is stated because it is the strongest form the answer takes anywhere in this source,
//! and it is still an inference — which is the measure of how far the source is from carrying one.
//!
//! # And the arrival code carries the same denial as the departure code
//!
//! Ohio forms two school districts in thirty years. **Monroe Local** appears in 2000-01 and
//! **Manchester Local** in 2004-05, both filed under code 3 — *"a new education agency formed with
//! no effect on another agency's boundaries."* In the same year Monroe Local appears, Middletown
//! Monroe City is renamed **Middletown City**, same identifier, same IRN, coded open on both
//! sides. Adams County/Ohio Valley Local, in the year Manchester Local appears, does not change at
//! all. Both new districts carry six-digit IRNs from Ohio's original county-ordered block, which
//! is to say numbers Ohio had used before and the federal register has no way to say so.
//!
//! A third, **Peebles**, is filed in 2004-05 as *"scheduled to be operational within 2 years"* and
//! closed the following year without ever reaching open. A district announced and abandoned, and
//! the only reason it is not a district departure is that it never arrived.
//!
//! So the register denies the boundary effect at both ends of an agency's life, and again the only
//! trace of it anywhere is a rename on the survivor — present in one of the two cases and absent
//! in the other. The successor readings here are stated and committed nowhere, on the same footing
//! as everything else in this module.
//!
//! # The agency type is not comparable across three of these years
//!
//! Three times the CCD recodes Ohio without an agency joining or leaving. 532 agencies change
//! type on those three seams and not one of them moved, so a rule that reads a type change as an
//! event finds 532 events that did not happen:
//!
//! - **2000-01.** 47 agencies move from type 1 to type 7. They are Ohio's first community
//!   schools, filed as regular school districts for their first two years. This is the dangerous
//!   one: on the type alone Ohio appears to gain 47 school districts in 1999-2000 and lose them
//!   again the next year. Types 1 and 2 together read 661 either side and 708 in between.
//! - **2002-03.** 60 supervisory-union centres and 49 districts move into type 4 together. The 60
//!   are what remains of the county boards; the 49 are the joint vocational school districts.
//!   Type 3 empties and type 4 goes from nothing to 109.
//! - **2006-07.** 376 agencies move from type 2, *"local school district component of a
//!   supervisory union"*, into type 1. Type 2 empties.
//!
//! [`types_by_year`] is what makes those visible, and the test beside it pins all three.
//!
//! # Where the order is, which is not where this module first said
//!
//! This note used to end by saying the reason needed "the territory-transfer orders under
//! R.C. 3311", and to imply the State Board of Education held them. **It does not, and for these
//! districts it never did.** R.C. 3311.22 — now committed at
//! [`revised-code.txt`](../../project/fixtures/revised-code.txt) — vests the transfer in the
//! *educational service center governing board*, which "shall at its next regular meeting …
//! adopt a resolution making the transfer effective". The State Board appears twice in the
//! section and neither is an approval: once as an appeal body, reachable only if the receiving
//! district's board opposes an ESC-initiated transfer, and once as a place a boundary **map** is
//! filed after the fact.
//!
//! West Geauga's board accepted unanimously, so no appeal arose and no State Board instrument was
//! ever created. A search of the Board's own record confirms it and is worth stating as a bound:
//! fifty-eight meetings of adopted minutes across 2014-2017 and 2019-2020, plus 4,981 board-book
//! PDFs spanning 2014-2021, contain zero occurrences of "Bettsville", zero of "Ledgemont" and
//! zero of `3311.22`. The same corpus **does** carry State Board territory-transfer resolutions
//! for other district pairs under R.C. 3311.24 and 3311.06, which is the positive control: the
//! instrument detects the thing, and these three are not in it.
//!
//! So the corpus was reaching for the wrong custodian rather than meeting a closed door. The
//! orders are recited, by date and issuing body, in the Auditor of State's reports on the
//! *receiving* districts — a state officer's account, one hop from a node. Committing those is
//! its own phase; what this module now says is where to look and where not to.

use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = include_str!("../fixtures/ccd-lea-directory.csv");

const EXPECTED_HEADER: &str = "school_year,leaid,irn,name,agency_type,status";

/// The first school year the directory is held for, as the year it opens in.
///
/// 1994-95 because that is the oldest edition NCES still serves. The survey itself reaches back to
/// 1986-87 in the same fixed-width family; those nine years are retrievable and unheld.
pub const FIRST_YEAR: u16 = 1994;

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

/// How many agencies of each type each year holds.
///
/// Published because the type vocabulary is not stable and a reader comparing types across a year
/// boundary needs to be able to see that. Twice — in 2002-03 and again in 2006-07 — Ohio's
/// composition changes by hundreds of agencies with nobody joining or leaving. See the module
/// note; the arithmetic of both seams is pinned in the tests below.
#[must_use]
pub fn types_by_year() -> BTreeMap<u16, BTreeMap<String, usize>> {
    let mut out: BTreeMap<u16, BTreeMap<String, usize>> = BTreeMap::new();
    for agency in panel() {
        *out.entry(agency.opens)
            .or_default()
            .entry(agency.agency_type)
            .or_default() += 1;
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

/// The transfer orders, as the Auditor of State recites them.
const TRANSFERS: &str = include_str!("../fixtures/territory-transfers.tsv");

/// One territory transfer, from the audit report that recites it.
///
/// **This is a recital, not the instrument.** The order is a resolution in an educational service
/// center's minute book, and those are not published. What is published is the Auditor of State's
/// report on the district, which quotes the resolution by date and issuing body. A state officer's
/// account of a local body's act is one hop from a corpus node; the act itself is not here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    /// The source key of the report, which is how a row reaches a digest.
    pub report: String,
    /// The entity whose audit it is.
    pub audited_entity: String,
    /// Where that entity stands to the transfer: `departing`, `receiving` or `resolving`.
    pub role: String,
    /// The body that passed the resolution.
    pub resolving_body: String,
    /// When it passed.
    pub resolution_date: String,
    /// When the transfer took effect.
    pub effective_date: String,
    /// The agency that ceased.
    pub departing: String,
    /// The agency that took its territory.
    pub receiving: String,
    /// The Revised Code section, where the report names one. Only one of the five does.
    pub section: String,
    /// The sentence, verbatim.
    pub recital: String,
}

/// Every recited transfer.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against.
#[must_use]
pub fn transfers() -> Vec<Transfer> {
    let mut rows = TRANSFERS.lines();
    assert_eq!(
        rows.next().unwrap_or_default().trim(),
        concat!(
            "report\taudited_entity\trole\tresolving_body\tresolution_date\t",
            "effective_date\tdeparting\treceiving\tsection\trecital"
        ),
        "the transfer fixture header changed; update dispersion::lea_directory"
    );
    rows.filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let f: Vec<&str> = line.split('\t').collect();
            Some(Transfer {
                report: f.first()?.to_string(),
                audited_entity: f.get(1)?.to_string(),
                role: f.get(2)?.to_string(),
                resolving_body: f.get(3)?.to_string(),
                resolution_date: f.get(4)?.to_string(),
                effective_date: f.get(5)?.to_string(),
                departing: f.get(6)?.to_string(),
                receiving: f.get(7)?.to_string(),
                section: f.get(8)?.to_string(),
                recital: f.get(9)?.to_string(),
            })
        })
        .collect()
}

/// Departures this repository can now give a reason for, keyed on the departing agency's name.
///
/// Five of 341, and the ratio is the point: the reason for a departure is not in the directory and
/// is not derivable from it. It has to be fetched one document at a time.
#[must_use]
pub fn explained() -> BTreeMap<String, Transfer> {
    transfers()
        .into_iter()
        .map(|t| (t.departing.clone(), t))
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
            // The low end is 1996-97, before Ohio had a community school; the high end 2005-06,
            // at the top of the charter opening wave.
            assert!(
                (700..=1400).contains(&rows.len()),
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
        // sample that happens to miss a rare code. Thirty consecutive years, completely
        // enumerated: this is not a sample of Ohio's agency-years, it is all of them that the
        // directory has published since 1994-95.
        assert!(panel().len() > 30_000, "{} agency-years", panel().len());
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
            community * 10 > total * 8,
            "community schools are {community} of {total} departures, and the claim that this \
             count is charter churn rather than consolidation rests on that ratio"
        );
        assert!(
            districts < 10,
            "{districts} regular districts left the directory in thirty years"
        );
    }

    /// The service agencies, which is what the fourteen older years were actually for.
    ///
    /// Sixty-six of them leave, and they are not a footnote to the community schools: they are
    /// the second-largest population in the departure list and the only one whose members issue
    /// the instruments this repository could not find. Ohio's 86 county boards of education are
    /// 47 by 2002-03, and every disappearance in between is filed as a closure affecting nobody.
    #[test]
    fn the_service_agencies_consolidate_and_the_register_denies_it() {
        let leaving: Vec<Departure> = departures()
            .into_iter()
            .filter(|d| d.agency_type == "3" || d.agency_type == "4")
            .collect();
        assert_eq!(
            leaving.len(),
            66,
            "the service-agency departures have moved"
        );
        for departure in &leaving {
            assert_eq!(
                departure.terminal_status, CLOSED_CODE,
                "{} left in {} under status {}",
                departure.name, departure.last_year, departure.terminal_status
            );
        }

        // The county boards, counted at both ends. Type 3 is the CCD's supervisory-union
        // administrative centre, which is what Ohio's county boards were filed as.
        let types = types_by_year();
        assert_eq!(types[&1994].get("3"), Some(&86));
        assert_eq!(types[&2001].get("3"), Some(&61));
        assert_eq!(types[&2002].get("3"), None, "type 3 should be empty by now");

        // And the one this repository already holds an order for: Geauga County ESC, whose final
        // audit recites its own dissolution into the Educational Service Center of the Western
        // Reserve. It is in this list, filed like all the others as a closure that affected
        // nobody.
        let geauga = leaving
            .iter()
            .find(|d| d.name.contains("Geauga County"))
            .expect("the one service-agency departure with a recited order behind it is missing");
        assert_eq!((geauga.irn.as_str(), geauga.last_year), ("047159", 2020));

        // The body it merged into is Lake County ESC, which keeps its agency identifier and its
        // IRN and takes the new name — and is coded open in the year before, the year of, and
        // the year after. This is the district case exactly, one level up: the only trace of the
        // merger anywhere in the register is a rename on the survivor, which is the derivation
        // the module note explains why nothing here trusts.
        let survivor: Vec<Agency> = panel()
            .into_iter()
            .filter(|a| a.leaid == "3904786" && (2018..=2021).contains(&a.opens))
            .collect();
        assert!(survivor
            .iter()
            .all(|a| a.status == "1" && a.irn == "047860"));
        assert_eq!(
            survivor.iter().map(|a| a.name.as_str()).collect::<Vec<_>>(),
            vec![
                "Lake County ESC",
                "Lake County ESC",
                "Educational Service Center of the Western Reserve",
                "Educational Service Center of the Western Reserve",
            ]
        );
    }

    /// Three times the type changes under agencies that did not move, 532 of them in total.
    ///
    /// Measured by following each agency identifier across the seam rather than by differencing
    /// the counts, because the counts move for two reasons at once and only one of them is a
    /// recode. Pinned because a rule that reads a type change as an event — a district becoming a
    /// service agency, say — would find every one of these and every one would be an artefact of
    /// the federal vocabulary rather than anything Ohio did.
    #[test]
    fn the_agency_type_is_recoded_three_times_without_anyone_moving() {
        let mut by_leaid: BTreeMap<&str, BTreeMap<u16, &str>> = BTreeMap::new();
        let panel = panel();
        for agency in &panel {
            by_leaid
                .entry(agency.leaid.as_str())
                .or_default()
                .insert(agency.opens, agency.agency_type.as_str());
        }
        let moved = |before: u16, from: &str, to: &str| {
            by_leaid
                .values()
                .filter(|years| {
                    years.get(&before) == Some(&from) && years.get(&(before + 1)) == Some(&to)
                })
                .count()
        };

        // Ohio's first community schools, filed as regular districts for two years.
        assert_eq!(moved(1999, "1", "7"), 47);
        // The county boards and the joint vocational districts, into type 4 together.
        assert_eq!((moved(2001, "3", "4"), moved(2001, "1", "4")), (60, 49));
        // Type 2 into type 1.
        assert_eq!(moved(2005, "2", "1"), 376);

        // And the vocabulary really does empty on each seam, rather than these being a handful of
        // agencies inside a category that carries on.
        let types = types_by_year();
        let count = |year: u16, kind: &str| types[&year].get(kind).copied().unwrap_or(0);
        assert_eq!((count(2001, "3"), count(2002, "3")), (61, 0));
        assert_eq!((count(2001, "4"), count(2002, "4")), (0, 109));
        assert_eq!((count(2005, "2"), count(2006, "2")), (377, 0));
        assert_eq!(count(2006, "1"), 614);
    }

    /// The federal type is not a safe way to count Ohio's school districts, and this is the shape
    /// of the error.
    ///
    /// Types 1 and 2 together give 661 through 1997-98, 675 and 708 in the two years the first
    /// community schools were filed as districts, 662 once they move to type 7, and 613 to 622
    /// from 2002-03 once the joint vocational districts move to type 4. Three discontinuities,
    /// none of them an agency opening or closing, and the largest of them 47 apparent new school
    /// districts in a single year.
    ///
    /// Ohio's actual number sits under all of it and barely moves: 661 minus 47 joint vocational
    /// districts is 614, and thirty years later it is 619.
    #[test]
    fn counting_districts_by_federal_type_produces_three_discontinuities() {
        let mut regular: BTreeMap<u16, usize> = BTreeMap::new();
        for agency in panel() {
            if agency.agency_type == "1" || agency.agency_type == "2" {
                *regular.entry(agency.opens).or_default() += 1;
            }
        }
        // The three eras, each internally steady.
        assert!((1994..=1997).all(|y| regular[&y] == 661));
        assert_eq!((regular[&1998], regular[&1999]), (675, 708));
        assert!((2000..=2001).all(|y| regular[&y] == 662));
        assert!((2002..=2023).all(|y| (613..=622).contains(&regular[&y])));

        // And the step between the first era and the last is the vocabulary, not Ohio: 47 joint
        // vocational districts leave type 1 in 2002-03, and 661 - 47 is what the modern count
        // has been within a few agencies for thirty years.
        assert_eq!(regular[&1994] - 47, 614);
        assert_eq!(regular[&2023], 619);
    }

    /// Ohio forms two school districts in thirty years, and the register says neither of them
    /// took territory from anybody.
    ///
    /// The mirror image of this module's whole subject. Code 3 is *"a new education agency formed
    /// with no effect on another agency's boundaries"*, and it is what Ohio files for both — while
    /// Middletown Monroe City becomes **Middletown City** in the same year Monroe Local appears,
    /// and Adams County/Ohio Valley Local carries on unaltered in the year Manchester Local does.
    /// So the arrival code carries the same denial as the departure code, and again the only
    /// trace anywhere in the register is a rename on the survivor — this time in one of the two
    /// cases and not the other.
    ///
    /// The successor reading is stated and not committed, exactly as for the departures. What is
    /// committed is what the file says.
    #[test]
    fn the_two_districts_ohio_forms_are_filed_as_affecting_nobody() {
        let panel = panel();
        let mut first: BTreeMap<String, Agency> = BTreeMap::new();
        for agency in panel.iter().rev() {
            first.insert(agency.leaid.clone(), agency.clone());
        }
        // A district-shaped arrival is one whose first year is after the first year held and is
        // typed as a district. Named rather than counted, and named in full, because the list is
        // short enough to read and because the classification is the finding: everything here
        // with an IRN in the 01xxxx block is a STEM school under R.C. 3326, which Ohio funds as
        // its own unit and the federal directory types as a school district.
        let mut named: Vec<(&str, u16, &str, &str)> = first
            .values()
            .filter(|a| a.opens > FIRST_YEAR && (a.agency_type == "1" || a.agency_type == "2"))
            // 1998-99 and 1999-2000 are the misfiled community schools, not districts.
            .filter(|a| !(1998..=1999).contains(&a.opens))
            .map(|a| (a.irn.as_str(), a.opens, a.status.as_str(), a.name.as_str()))
            .collect();
        named.sort_unstable();
        assert_eq!(
            named,
            vec![
                // Peebles, which never reaches status 1 and is filed closed the next year: a
                // district announced and abandoned rather than one that opened.
                ("000441", 2004, "7", "PEEBLES"),
                // Manchester, scheduled in 2003-04, formed in 2004-05, open ever since.
                ("000442", 2003, "7", "MANCHESTER LOCAL SD"),
                ("011506", 2009, "3", "DAYTON REGIONAL STEM SCHOOL"),
                ("012391", 2012, "3", "METRO EARLY COLLEGE HIGH SCHOOL"),
                ("013930", 2013, "3", "GLOBAL IMPACT STEM ACADEMY"),
                ("014231", 2013, "3", "BIO-MED SCIENCE ACADEMY STEM SCHOOL"),
                ("014877", 2015, "3", "Metro Institute of Technology"),
                ("014943", 2014, "7", "Valley STEM+ME2 Academy"),
                ("015328", 2015, "3", "Collins Career Center STEM Academy"),
                (
                    "015329",
                    2015,
                    "3",
                    "iSTEM Geauga Early College High School"
                ),
                ("015344", 2015, "3", "Tri State Early College STEM School"),
                ("019602", 2023, "3", "Community STE(A)M Academy - Xenia"),
                // The one unambiguous new school district in thirty years.
                ("139303", 2000, "3", "MONROE LOCAL SD"),
            ],
            "the district-shaped arrivals have changed"
        );

        let row = |leaid: &str, year: u16| {
            panel
                .iter()
                .find(|a| a.leaid == leaid && a.opens == year)
                .map(|a| (a.name.as_str(), a.status.as_str()))
        };
        assert_eq!(
            row("3900537", 2004).map(|(_, s)| s),
            Some("3"),
            "Manchester Local should be filed as newly formed in 2004-05"
        );
        // Middletown drops "Monroe" from its name the year Monroe Local appears, and is coded
        // open on both sides of it.
        assert_eq!(
            row("3904440", 1999),
            Some(("MIDDLETOWN MONROE CITY SD", "1"))
        );
        assert_eq!(row("3904440", 2000), Some(("MIDDLETOWN CITY SD", "1")));
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
        // Old Fort took Bettsville, Berkshire took Ledgemont, West Geauga took Newbury. Every one
        // is coded `1 Open` throughout, which is why no rule over this source can find them.
        //
        // Chardon (3904718) is deliberately absent. An earlier version of this test named it as a
        // second receiver of Newbury's territory, on a judge's reading rather than a record. The
        // resolution transfers "all" of Newbury to West Geauga, and this repository's own panel
        // says the same: West Geauga gains 208 pupils in FY2021 and **Chardon loses 110** in the
        // same year. A survivor that shrinks did not receive anybody.
        let survivors = ["3910021", "3904716", "3904722"];
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

    /// The three district transfers, each recited by a state officer, with dates.
    #[test]
    fn every_district_departure_now_has_an_order_behind_it() {
        let explained = explained();
        for (departing, receiving, date) in [
            ("Bettsville", "Old Fort", "June 24, 2014"),
            ("Ledgemont", "Berkshire", "January 27, 2015"),
            ("Newbury", "West Geauga", "August 20, 2019"),
        ] {
            let transfer = explained
                .get(departing)
                .unwrap_or_else(|| panic!("{departing} has no recited order"));
            assert_eq!(transfer.receiving, receiving);
            assert_eq!(transfer.resolution_date, date);
            assert!(
                transfer.recital.contains(departing) && transfer.recital.contains(receiving),
                "{departing}'s recital names neither end of its own transfer"
            );
        }
    }

    /// One transfer is recited twice, by two audited entities two years apart.
    ///
    /// The corroboration that makes a recital usable. Bettsville's own final audit and Old Fort's
    /// the following year carry the same resolution, the same date and the same effect — two
    /// state reports on two entities describing one transaction. Nothing else here has that.
    #[test]
    fn the_bettsville_transfer_is_recited_by_both_ends() {
        let both: Vec<Transfer> = transfers()
            .into_iter()
            .filter(|t| t.departing == "Bettsville")
            .collect();
        assert_eq!(
            both.len(),
            2,
            "only one end of the Bettsville transfer is here"
        );
        let roles: BTreeSet<&str> = both.iter().map(|t| t.role.as_str()).collect();
        assert_eq!(roles, ["departing", "receiving"].into_iter().collect());
        for transfer in &both {
            assert_eq!(transfer.resolution_date, "June 24, 2014");
            assert_eq!(transfer.effective_date, "June 30, 2014");
            assert_eq!(
                transfer.resolving_body,
                "North Central Ohio Educational Service Center"
            );
        }
    }

    /// Only one report names the section, and that is a fact about the reports.
    ///
    /// The corpus can say the mechanism is R.C. 3311.22 for Newbury because West Geauga's audit
    /// says so. For the other two it is an inference from the statute's shape, and the empty
    /// column is what keeps the two apart.
    #[test]
    fn only_one_recital_cites_a_section() {
        let cited: Vec<Transfer> = transfers()
            .into_iter()
            .filter(|t| !t.section.is_empty())
            .collect();
        assert_eq!(cited.len(), 1);
        assert_eq!(cited[0].departing, "Newbury");
        assert_eq!(cited[0].section, "3311.22");
    }

    /// Every departure with an order is one the directory said had no effect on anybody.
    #[test]
    fn the_directory_denies_every_transfer_the_auditor_records() {
        let departures = departures();
        for transfer in transfers() {
            // The service centre's own dissolution is in the panel too, under its own name.
            let Some(row) = departures.iter().find(|d| {
                d.name.contains(&transfer.departing) || transfer.departing.contains(&d.name)
            }) else {
                continue;
            };
            assert_eq!(
                row.terminal_status, CLOSED_CODE,
                "{} left under status {}, so the contradiction this module rests on has changed",
                row.name, row.terminal_status
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
