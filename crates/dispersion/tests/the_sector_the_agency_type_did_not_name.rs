//! Ohio's community schools, once the agency type stops naming two different things.
//!
//! `lea_directory` says CCD type 7 is what Ohio's community schools are filed as. It is, from
//! 2000-01. Before that the same code names **regional data-processing co-operatives** — Ohio had
//! no community school law until 1997 and no community school until 1998-99 — and a series built
//! on the bare type opens with a 1994 cohort of twenty-four that has nothing to do with charter
//! schools.
//!
//! This is `project::appropriations`' ALI 200604 in a different source: an identifier reused for
//! an unrelated programme, where the caution is only useful if the succession is actually
//! resolved rather than used to avoid the question. Here it resolves cleanly, and these tests hold
//! why — four independent discriminators agree on the same partition, and no agency straddles it.
//!
//! # What falls out once it is resolved
//!
//! Two findings that are invisible in the row count, and both change a number a reader would
//! otherwise take from the directory directly:
//!
//! - **267 of 934 community schools never opened.** They are announced and then absent.
//! - **The operating sector peaks in 2013-14, not 2005-06.** Eight years apart, and the earlier
//!   date is what a count of rows gives.

use std::collections::BTreeSet;

use dispersion::community_schools::{self, FIRST_POSSIBLE, OPERATING};
use dispersion::lea_directory::{self, LAST_YEAR};

/// The directory's last edition, restated so a change to the fixture fails here first.
const LAST_EDITION: u16 = 2023;

/// **The licence for the whole module.** The two populations are disjoint on every axis.
///
/// A year rule is the weakest kind, and it is safe here only because the cut falls in a gap. If a
/// single agency were typed 7 on both sides of it, this partition would be a judgement call and
/// the module would need a different rule — so the straddle count is asserted rather than assumed.
#[test]
fn the_two_populations_do_not_overlap_on_any_axis() {
    let typed_seven: Vec<_> = lea_directory::panel()
        .into_iter()
        .filter(|agency| agency.agency_type == "7")
        .collect();

    let early: BTreeSet<&str> = typed_seven
        .iter()
        .filter(|agency| agency.opens < FIRST_POSSIBLE)
        .map(|agency| agency.irn.as_str())
        .collect();
    let late: BTreeSet<&str> = typed_seven
        .iter()
        .filter(|agency| agency.opens > 2000)
        .map(|agency| agency.irn.as_str())
        .collect();

    assert_eq!(early.len(), 24, "the co-operatives");
    assert_eq!(
        early.intersection(&late).count(),
        0,
        "an agency typed 7 on both sides of the cut would make this partition a judgement call"
    );

    // The IRN block agrees, and it is a fact about Ohio's numbering rather than about time.
    for irn in &early {
        let block = &irn[..2];
        assert!(
            block == "08" || block == "09",
            "co-operative {irn} is outside the regional block"
        );
    }
    for irn in &late {
        let block = &irn[..2];
        assert!(
            block != "08" && block != "09",
            "community school {irn} is inside the regional block"
        );
    }
}

/// And neither co-operative ever ran anything a school report would notice.
///
/// The third and fourth discriminators. A data-processing co-operative serves no meals and files
/// no school finance return, and none of the twenty-four appears in either fixture — against 490
/// and 539 of the community schools that do.
#[test]
fn no_co_operative_appears_in_a_fixture_that_only_schools_reach() {
    let co_operatives: BTreeSet<String> = community_schools::predecessors()
        .into_iter()
        .map(|agency| agency.irn)
        .collect();
    assert_eq!(co_operatives.len(), 24);

    let meals: BTreeSet<String> = dispersion::mr81::panel()
        .into_iter()
        .map(|sponsor| sponsor.irn)
        .collect();
    let finance: BTreeSet<String> = dispersion::ohio_panel::panel()
        .into_iter()
        .map(|row| row.irn)
        .collect();

    assert_eq!(
        co_operatives.intersection(&meals).count(),
        0,
        "a co-operative serving meals would break the discriminator"
    );
    assert_eq!(co_operatives.intersection(&finance).count(), 0);

    // The positive control: the discriminator detects the thing it is meant to detect.
    let schools: BTreeSet<String> = community_schools::schools()
        .into_iter()
        .map(|school| school.irn)
        .collect();
    assert_eq!(schools.len(), 934);
    assert_eq!(schools.intersection(&meals).count(), 490);
    assert_eq!(schools.intersection(&finance).count(), 539);
}

/// The schools the directory filed as regular districts keep their real opening year.
///
/// `lea_directory` records the 2000-01 recode, in which 47 agencies move from type 1 to type 7.
/// The consequence here is that an opening year read off the first *type-7* edition is late for
/// every one of them, so [`School::listed_from`](dispersion::community_schools::School) reads the
/// first edition of any type.
///
/// **They are not one cohort, and the directory's note reads as though they were.** 14 open in
/// 1998-99 and 33 in 1999-2000; the recode gathers two intakes into one edition. So Ohio's first
/// community school cohort is fourteen schools, a year earlier than the type alone can show.
#[test]
fn the_schools_filed_as_districts_are_dated_from_when_they_were_filed() {
    let directory = lea_directory::panel();
    let first_typed = |irn: &str| {
        directory
            .iter()
            .filter(|agency| agency.irn == irn && agency.agency_type == "7")
            .map(|agency| agency.opens)
            .min()
    };

    let recoded: Vec<_> = community_schools::schools()
        .into_iter()
        .filter(|school| first_typed(&school.irn).is_some_and(|typed| typed > school.listed_from))
        .collect();
    assert_eq!(recoded.len(), 47, "the cohort lea_directory names");

    // Two intakes, not one, and every one of them is filed as a district and new in its first
    // edition — so the earlier date is the directory's own claim rather than an inference here.
    let mut intake = [0usize; 2];
    for school in &recoded {
        let first = directory
            .iter()
            .find(|agency| agency.irn == school.irn && agency.opens == school.listed_from)
            .expect("listed in its own first edition");
        assert_eq!(
            first.agency_type, "1",
            "{} is not typed as a district",
            school.irn
        );
        assert_eq!(first.status, "3", "{} is not filed new", school.irn);
        assert_eq!(
            school.opened,
            Some(school.listed_from),
            "{} is dated late",
            school.irn
        );
        assert_eq!(
            first_typed(&school.irn),
            Some(2000),
            "{} recoded elsewhere",
            school.irn
        );

        match school.listed_from {
            1998 => intake[0] += 1,
            1999 => intake[1] += 1,
            other => panic!("{} opens in {other}, outside both intakes", school.irn),
        }
    }
    assert_eq!(
        intake,
        [14, 33],
        "the two intakes the recode gathers into one"
    );
}

/// **The first finding.** Nearly three in ten were announced and never opened.
///
/// Status 7 is *scheduled to be operational within 2 years*. 267 agencies carry it and never carry
/// an operating status afterwards — they are in the directory, they are typed as community
/// schools, and they never taught anybody. Any count of Ohio's community schools taken off the row
/// count includes them.
#[test]
fn two_hundred_and_sixty_seven_community_schools_never_opened() {
    let schools = community_schools::schools();
    assert_eq!(schools.len(), 934, "community schools the directory names");

    let opened = schools.iter().filter(|s| s.opened.is_some()).count();
    let never = community_schools::never_opened();
    assert_eq!(opened, 667);
    assert_eq!(never.len(), 267);
    assert_eq!(opened + never.len(), schools.len());

    // And they are announcements rather than a coding artifact: every one of them is filed under a
    // scheduled or inactive status in its last edition, never under a closure.
    let directory = lea_directory::panel();
    for school in never.iter().take(50) {
        let last = directory
            .iter()
            .filter(|agency| agency.irn == school.irn)
            .max_by_key(|agency| agency.opens)
            .expect("listed at all");
        assert!(
            !OPERATING.contains(&last.status.as_str()),
            "{} ends open and should not be in this list",
            school.irn
        );
    }
}

/// **The second finding.** The sector peaked eight years after the row count says.
///
/// 2005-06 lists 504 agencies and 318 of them are open; the other 186 are announcements and
/// closures piled into one edition. The operating sector goes on growing for another eight years
/// and peaks at 391 in 2013-14.
#[test]
fn the_operating_sector_peaks_eight_years_after_the_row_count_does() {
    let sector = community_schools::sector_by_year();

    let busiest_list = sector
        .values()
        .max_by_key(|year| year.listed)
        .expect("years");
    let busiest_open = sector.values().max_by_key(|year| year.open).expect("years");

    assert_eq!((busiest_list.year, busiest_list.listed), (2005, 504));
    assert_eq!((busiest_open.year, busiest_open.open), (2013, 391));
    assert_eq!(busiest_open.year - busiest_list.year, 8);

    // In the year the row count peaks, three rows in eight are not a school.
    assert_eq!(sector[&2005].open, 318);
    assert_eq!(sector[&2005].listed - sector[&2005].open, 186);

    // The sector's own history, for the record: 14 schools in 1998-99, and it has not been below
    // 300 since 2004-05.
    assert_eq!(sector[&1998].open, 14);
    for (year, row) in &sector {
        if *year >= 2005 {
            assert!(row.open >= 300, "{year} has {} open", row.open);
        }
    }
}

/// Survival, on the years the directory can actually observe.
///
/// Roughly one community school in five is gone within five years and two in five within ten. The
/// twenty-year figure is over the 274 schools old enough to have one, which is why it is not a
/// statement about the sector as it stands.
#[test]
fn one_in_five_is_gone_within_five_years_and_two_in_five_within_ten() {
    let at = |years: usize| {
        let survival = community_schools::survival(years);
        (survival.at_risk, survival.rate().expect("schools at risk"))
    };

    let (risk3, rate3) = at(3);
    let (risk5, rate5) = at(5);
    let (risk10, rate10) = at(10);
    let (risk20, rate20) = at(20);

    assert_eq!((risk3, risk5, risk10, risk20), (644, 622, 578, 274));
    assert!((rate3 - 0.887).abs() < 0.002, "three-year survival {rate3}");
    assert!((rate5 - 0.794).abs() < 0.002, "five-year survival {rate5}");
    assert!((rate10 - 0.606).abs() < 0.002, "ten-year survival {rate10}");
    assert!(
        (rate20 - 0.434).abs() < 0.002,
        "twenty-year survival {rate20}"
    );

    // Monotone, which a survival curve must be and this one is not guaranteed to be: the at-risk
    // population shrinks as the horizon lengthens, so each point is over a different, older cohort.
    let mut previous = 1.1;
    for years in 1..=20 {
        let rate = community_schools::survival(years).rate().expect("at risk");
        assert!(rate <= previous, "survival rose at {years} years");
        previous = rate;
    }
}

/// Every school that is still open is open in the last edition, and none is open past it.
///
/// The censoring check. A span is a floor for a school the directory has not seen close, and this
/// is what keeps `still_open` meaning that rather than drifting into meaning "closed recently".
#[test]
fn a_school_open_in_the_last_edition_is_censored_rather_than_closed() {
    assert_eq!(LAST_YEAR, LAST_EDITION, "the directory's span changed");

    let schools = community_schools::schools();
    let open_now = schools.iter().filter(|school| school.still_open).count();
    assert_eq!(open_now, 336);
    assert_eq!(community_schools::sector_by_year()[&LAST_EDITION].open, 336);

    for school in &schools {
        assert!(
            school.last_open.is_none_or(|last| last <= LAST_EDITION),
            "{} is open past the directory",
            school.irn
        );
        assert_eq!(
            school.still_open,
            school.last_open == Some(LAST_EDITION),
            "{} disagrees with its own last open year",
            school.irn
        );
    }
}
