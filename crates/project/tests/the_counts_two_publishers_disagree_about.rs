//! The department's archive against the Legislative Service Commission's greenbooks.
//!
//! Both publishers state scholarship participation for years the other covers, and until now
//! nothing had put them beside each other. The greenbooks are LSC's analysis of each enacted
//! budget act and quote the programmes' recent counts in passing; the archive is the department's
//! own extract of the system that administered them.
//!
//! **Four overlaps, and one of them reconciles.** Where LSC gives Cleveland's FY2005 scholarship
//! count it agrees with the department to the student. The other three do not, and they fail in
//! three different ways — a tutoring count 32 apart, a Cleveland count LSC itself marks
//! "approximately", and an Autism count that is wrong by a factor of seven.
//!
//! That last one is the reason this file exists. A greenbook figure is the number a legislator
//! reads, and this repository cites greenbooks for parameters it cannot reach any other way. One
//! of them is off by 2,053 students, and there was no way to know before a second source was held.

use project::greenbook::greenbook;
use project::scholarship::history::{self, Measure};

use edfund_core::FiscalYear;

/// One count, from the department, for a programme and year.
fn archived(program: &str, year: u16, measure: Measure) -> f64 {
    history::series(program, measure)[&FiscalYear(year)]
        .total
        .unwrap_or_else(|| panic!("{program} FY{year} publishes no total on {measure:?}"))
}

/// LSC's FY2006-07 analysis quotes Cleveland's FY2005 scholarships, and it is exactly right.
///
/// The one clean reconciliation between the two publishers, and worth having: it establishes that
/// LSC and the department are counting the same thing when they both say "received scholarships",
/// which is what makes the disagreements below disagreements rather than definitional noise.
#[test]
fn the_cleveland_scholarship_count_reconciles_to_the_student() {
    let analysis = greenbook("hb66");
    let passage = analysis.around("In FY 2005, 5,710 students received scholarships", 2);
    assert!(
        passage.contains("5,710 students received scholarships"),
        "LSC's FY2005 Cleveland sentence has moved: {passage}"
    );
    assert_eq!(archived("cleveland", 2005, Measure::Used), 5_710.0);
}

/// The tutoring grants in the same sentence do not reconcile, and neither publisher hedges.
#[test]
fn the_tutoring_count_in_the_same_sentence_is_thirty_two_apart() {
    let analysis = greenbook("hb66");
    let passage = analysis.around("In FY 2005, 5,710 students received scholarships", 2);
    assert!(passage.contains("2,982 students"), "{passage}");

    let held = archived("cleveland", 2005, Measure::Tutoring);
    assert_eq!(held, 3_014.0);

    // 1.1% apart, in one sentence, where the scholarship half of the same sentence is exact. So
    // this is not a vintage difference — LSC and the department count tutoring differently, and
    // neither says how. The corpus records the gap and does not pick a side.
    let gap = held - 2_982.0;
    assert_eq!(gap, 32.0);
}

/// LSC's FY2012 Cleveland figure is marked "approximately", and is 50 students out.
///
/// The hedge is doing real work, so this is the disagreement that is not a defect in anything —
/// it is recorded because a reader comparing the two files should not mistake it for one.
#[test]
fn an_approximate_count_is_approximate_by_about_a_per_cent() {
    let analysis = greenbook("hb59");
    let passage = analysis.around("In FY 2012, approximately 5,128 students participated", 2);
    assert!(
        passage.contains("approximately 5,128 students"),
        "{passage}"
    );

    let held = archived("cleveland", 2012, Measure::Used);
    assert_eq!(held, 5_078.0);
    assert!((5_128.0 / held - 1.0).abs() < 0.01);
}

/// LSC's FY2012 EdChoice figure sits below the department's payment count and above nothing.
///
/// "15,219 scholarships were awarded" is neither of the two denominators the archive keeps:
/// applications were 17,160 and payments 15,574. An award is a third quantity, and one that ought
/// to be at least as large as the payments made against it — awards that go unused are the gap
/// between the two, in that direction. This one is 355 *smaller*, which it cannot be if the three
/// words mean what they appear to.
#[test]
fn an_award_count_falls_between_nothing_and_is_smaller_than_the_payments() {
    let analysis = greenbook("hb59");
    assert!(
        analysis
            .flat()
            .contains("In FY 2012, 15,219 scholarships were awarded"),
        "LSC's FY2012 EdChoice sentence has moved"
    );

    let applications = archived("traditional-edchoice", 2012, Measure::Applications);
    let payments = archived("traditional-edchoice", 2012, Measure::Used);
    assert_eq!(applications, 17_160.0);
    assert_eq!(payments, 15_574.0);

    assert!(
        15_219.0 < payments,
        "the ordering the sentence implies holds after all"
    );
    assert_eq!(payments - 15_219.0, 355.0);
}

/// And the one that matters: LSC's FY2012 Autism count is wrong by a factor of seven.
///
/// The greenbook says 360 students. The department's archive says 2,413, and the *next*
/// greenbook says 2,623 for FY2014 — continuous with the department's series and not with its own
/// predecessor. So the outlier is identified from two directions, and it is LSC's.
///
/// Asserted rather than corrected: the fixture holds what each publisher wrote. What this pins is
/// that the two cannot both be read, which is the thing a reader quoting the greenbook needs to
/// know.
#[test]
fn an_autism_count_in_one_greenbook_is_off_by_a_factor_of_seven() {
    let earlier = greenbook("hb59");
    assert!(
        earlier
            .flat()
            .contains("In FY 2012, 360 students received scholarships"),
        "LSC's FY2012 Autism sentence has moved"
    );

    let held = archived("autism", 2012, Measure::Used);
    assert_eq!(held, 2_413.0);
    assert!(held / 360.0 > 6.0, "{held}");

    // The next greenbook, on the department's own trajectory. The archive's last year is FY2013 at
    // 2,622; LSC's FY2014 figure is 2,623, one student later. A series that moves by one in a year
    // did not move by two thousand in the two before it.
    let later = greenbook("hb64");
    assert!(
        later.flat().contains("In FY 2014, 2,623 students received"),
        "LSC's FY2014 Autism sentence has moved"
    );
    assert_eq!(archived("autism", 2013, Measure::Used), 2_622.0);
}

/// The eleven-year hole is not empty: LSC quotes counts inside it, and they are not the same
/// measurement.
///
/// The archive stops at FY2013 and the annual report starts at 2024-25, which was recorded as a
/// stretch with no participation source. It is not quite that. The 131st's analysis gives FY2014
/// for all four programmes and the 133rd's gives FY2018 and FY2019 — but three of the four are
/// prefixed "about" and rounded to the nearest five hundred, and none states which denominator it
/// is counting on, which is the distinction the archive exists to keep.
///
/// So the gap has *bounds* rather than a series. Recorded here so that a later phase looking for
/// FY2014-FY2023 participation starts from what is already held rather than from nothing.
#[test]
fn the_greenbooks_bound_the_gap_without_filling_it() {
    let last_archived = history::observations()
        .iter()
        .map(|o| o.year.0)
        .max()
        .expect("the archive has rows");
    assert_eq!(last_archived, 2013);

    // One year past the archive, stated exactly, for all four programmes.
    let after = greenbook("hb64").flat();
    for quoted in [
        "In FY 2014, 16,987 scholarships were awarded",
        "In FY 2014, 2,623 students received scholarships",
        "In FY 2014, 2,103 students received",
        "In FY 2014, 6,337 students participated in the program",
    ] {
        assert!(
            after.contains(quoted),
            "{quoted} is no longer in the 131st's analysis"
        );
    }

    // Five years further in, and rounded. These are the only participation figures this
    // repository holds between FY2015 and FY2023.
    let inside = greenbook("hb166").flat();
    for quoted in [
        "In FY 2019, about 23,000 students received scholarships",
        "about 3,500 students received scholarships",
        "about 6,000 students received scholarships",
        "In FY 2018, 8,362 students participated in the program",
    ] {
        assert!(
            inside.contains(quoted),
            "{quoted} is no longer in the 133rd's analysis"
        );
    }

    // The hedge is the point. Three of the four are "about", and the archive's own figures never
    // are — so these bound the gap and cannot be spliced onto the series as if they were more of
    // it.
    assert!(inside.matches("about").count() > 3);
}
