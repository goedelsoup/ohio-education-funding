//! The district each chartered nonpublic school sits in, and how much of an October that reaches.
//!
//! R.C. 3317.024(E)(1) pays auxiliary services through the district a chartered nonpublic school
//! is located in. No October file in any era says which district that is, and a join on name was
//! rejected because names repeat. [`project::ledger::nonpublic_location`] is the column, built on
//! keys instead: the department's directory gives each open school's address against its building
//! IRN, the Census Bureau's geocoder places the address in a 2020 block, and the block file says
//! which district the block lay in.
//!
//! What a column like that owes is a statement of its reach, and these tests are that statement.
//! The directory is a snapshot and the geocoder misses, so neither the rate nor the set is a
//! property of the method — both are what this fetch found, pinned so that a refresh which changes
//! them is read rather than absorbed.

use std::collections::BTreeSet;

use project::ledger::nonpublic_enrolment as enrolment;
use project::ledger::nonpublic_location as location;

/// October 2025, the latest the department has published and the one nearest the directory.
const OCTOBER: u16 = 2026;

#[test]
fn ninety_five_percent_of_october_2025_is_placed_in_a_district() {
    /*
     * The match rate, against the October nearest the directory's own day.
     *
     * 749 buildings reported in October 2025. 743 of them are in the directory, and the geocoder
     * placed 714 — 95.3% of the October. That is the reach to quote, and it is a rate against the
     * October rather than against the directory, because the October is what R.C. 3317.024 counts:
     * a directory school with no October row is not a school the appropriation is divided over.
     *
     * The directory side is 787 schools, 755 placed. The two counts differ in both directions: the
     * directory lists 44 schools the October does not, and the October lists six the directory
     * does not.
     */
    let buildings = enrolment::buildings(OCTOBER);
    let reported: BTreeSet<&str> = buildings.iter().map(|b| b.irn.as_str()).collect();
    assert_eq!(reported.len(), 749, "October 2025's buildings");

    let listed: Vec<_> = reported
        .iter()
        .filter_map(|irn| location::location(irn))
        .collect();
    assert_eq!(listed.len(), 743, "in the directory");

    let placed = listed.iter().filter(|row| row.is_placed()).count();
    assert_eq!(placed, 714, "placed in a district");
    assert_eq!(
        format!("{:.1}", 100.0 * placed as f64 / reported.len() as f64),
        "95.3"
    );

    // The directory's own count, and how the placed fall: 278 districts hold at least one.
    let all = location::locations();
    assert_eq!(all.len(), 787);
    assert_eq!(all.iter().filter(|row| row.is_placed()).count(), 755);
    let districts: BTreeSet<&str> = all
        .iter()
        .filter_map(|row| row.district_irn.as_deref())
        .collect();
    assert_eq!(districts.len(), 278);

    // The rate falls with distance from the directory's day, which is the snapshot showing: the
    // further back the October, the more of its buildings have closed and left the directory.
    let in_directory = |fiscal_year| {
        let irns: BTreeSet<String> = enrolment::buildings(fiscal_year)
            .into_iter()
            .map(|b| b.irn)
            .collect();
        let found = irns.iter().filter(|irn| location::location(irn).is_some());
        (irns.len(), found.count())
    };
    assert_eq!(in_directory(2025), (722, 710), "October 2024");
    assert_eq!(in_directory(2024), (711, 688), "October 2023");
}

#[test]
fn the_thirty_five_october_2025_buildings_without_a_district() {
    /*
     * The unmatched set, by name of cause rather than as a remainder.
     *
     * Six are not in the directory at all. The extract lists open organizations only, so these are
     * buildings that reported in October 2025 and have closed or been re-designated since; the
     * directory's search API reaches closed organizations and is not read.
     *
     * Twenty-nine are in the directory and the geocoder did not place: 28 addresses it could not
     * match and one it matched to two places equally (`058727`, a Shelby address on a state
     * route). Run against the 2020 benchmark as well, eight of the directory's 32 misses matched —
     * not taken,
     * because one benchmark is one set of address ranges and mixing them would make the block a
     * school is placed in depend on which pass found it.
     *
     * Pinned as sets, so a refresh that places one of these, or loses another, fails here with
     * its IRN rather than moving a percentage.
     */
    let reported: BTreeSet<String> = enrolment::buildings(OCTOBER)
        .into_iter()
        .map(|b| b.irn)
        .collect();

    let absent: Vec<&str> = reported
        .iter()
        .map(String::as_str)
        .filter(|irn| location::location(irn).is_none())
        .collect();
    assert_eq!(
        absent,
        ["000204", "019353", "020549", "022254", "060335", "132340"]
    );

    let unplaced: Vec<(&str, &str)> = reported
        .iter()
        .filter_map(|irn| location::location(irn))
        .filter(|row| !row.is_placed())
        .map(|row| (row.irn.as_str(), row.geocoder_match.as_str()))
        .collect();
    let ties: Vec<&str> = unplaced
        .iter()
        .filter(|(_, outcome)| *outcome == "Tie")
        .map(|(irn, _)| *irn)
        .collect();
    assert_eq!(ties, ["058727"]);
    let missed: Vec<&str> = unplaced
        .iter()
        .filter(|(_, outcome)| *outcome == "No_Match")
        .map(|(irn, _)| *irn)
        .collect();
    assert_eq!(
        missed,
        [
            "008246", "009484", "011374", "013257", "020400", "021698", "052613", "052639",
            "052837", "052860", "052977", "053199", "053488", "054957", "056127", "056358",
            "057208", "058339", "058495", "060095", "060848", "070151", "090464", "122457",
            "122879", "132662", "134312", "134478",
        ]
    );
    assert_eq!(absent.len() + unplaced.len(), 35);
}

#[test]
fn a_school_in_newbury_sits_in_a_district_that_ceased_after_the_census() {
    /*
     * The one place the 2020 blocks and the present disagree.
     *
     * Newbury Local's territory went to West Geauga on July 1, 2020, under R.C. 3311.22 — after
     * the census the block file describes. So St Helen, at 12060 Kinsman Road in Newbury, geocodes
     * to Newbury's code, and the CCD directory last lists that code in 2020. The fixture writes
     * what it found and the year beside it; resolving it is a join against the transfer record,
     * which is what this does.
     *
     * Every other placed school's district is listed in the 2023 directory, the latest held, so
     * this is the whole of the problem today. A future consolidation would add a second row here
     * and fail the count.
     */
    let ceased: Vec<_> = location::locations()
        .iter()
        .filter(|row| row.is_placed() && row.directory_year != Some(2023))
        .collect();
    assert_eq!(ceased.len(), 1, "{ceased:?}");
    let st_helen = ceased[0];
    assert_eq!(st_helen.irn, "057448");
    assert_eq!(st_helen.district_irn.as_deref(), Some("047217"));
    assert_eq!(st_helen.district.as_deref(), Some("Newbury Local"));
    assert_eq!(st_helen.directory_year, Some(2020));

    let transfer = dispersion::lea_directory::transfers()
        .into_iter()
        .find(|t| t.departing == "Newbury")
        .expect("the Newbury transfer is recited");
    assert_eq!(transfer.receiving, "West Geauga");
    assert_eq!(transfer.effective_date, "July 1, 2020");
    assert_eq!(transfer.section, "3311.22");
}
