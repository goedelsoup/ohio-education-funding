//! Cleveland's renewal column changes meaning in FY2010, and nothing in the workbook says so.
//!
//! The department's *Historical Scholarship Data* archive publishes two denominators for every
//! programme-year it covers — applications made, and scholarships with at least one payment made
//! against them. That pair is the reason to hold the file: the 2025 annual report's three
//! unreconciled averages were recorded in the catalog as "a denominator the report does not name",
//! and this is the first held evidence that the department maintains two.
//!
//! What the pair also does is make one of the columns checkable against the other, and it does not
//! survive the check. For twelve consecutive years Cleveland's renewal *applications* are exactly
//! the prior year's *payments* — not close, equal — which is a roll-forward rather than a count of
//! anybody applying. It stops at FY2010, the same year the low-income renewal column starts
//! carrying values at all.
//!
//! These tests pin the break rather than repairing it. The fixture carries what the department
//! published; what is asserted here is where the definition changes, so that a reader taking a
//! "renewal rate" out of this series knows which half of it they are standing on.

use project::scholarship::history::{self, Measure};

use edfund_core::FiscalYear;

/// Cleveland's twelve years of exact agreement, and the four that follow them.
const ROLL_FORWARD_LAST_YEAR: u16 = 2009;

#[test]
fn the_archive_covers_the_deduct_era_and_stops_before_the_plan() {
    let years: Vec<u16> = history::observations().iter().map(|o| o.year.0).collect();
    let first = years.iter().copied().min().expect("the archive has rows");
    let last = years.iter().copied().max().expect("the archive has rows");

    // FY1997 is eleven years before this repository's next-earliest participation figure and
    // twenty-eight before the annual report's. If either end moves, the department has replaced
    // the workbook at its URL with something else.
    assert_eq!(first, 1997);
    assert_eq!(last, 2013);

    // The hole this source does not close, stated as an assertion so it cannot quietly go away.
    // The annual report is 2024-25, and no committed series covers between — only the quoted
    // points `project::scholarship::bounds` holds, which
    // `the_points_that_bound_the_participation_hole` asserts are joinable to neither end.
    assert!(
        !years.contains(&2014) && !years.contains(&2020),
        "the archive has grown a year it did not have; the FY2014-FY2023 gap may have moved"
    );
}

/// Applications are not participants, and the distance between them is not small.
#[test]
fn the_two_denominators_are_far_apart_in_every_year_that_has_both() {
    let applied = history::series("cleveland", Measure::Applications);
    let paid = history::series("cleveland", Measure::Used);
    assert_eq!(applied.len(), 17);
    assert_eq!(paid.len(), 17);

    let mut ratios = Vec::new();
    for (year, counts) in &applied {
        let requests = counts
            .total
            .expect("every year publishes an application total");
        let payments = paid[year]
            .total
            .expect("every year publishes a payment total");
        assert!(
            payments < requests,
            "FY{} paid {payments} against {requests} applications",
            year.0
        );
        ratios.push(requests / payments);
    }
    let widest = ratios.iter().copied().fold(f64::MIN, f64::max);
    let narrowest = ratios.iter().copied().fold(f64::MAX, f64::min);

    // FY1997, the programme's first year, when four applications in five went unpaid; and FY2013,
    // its last in this file. A "participants" figure taken off the wrong column overstates the
    // early years threefold.
    assert!((3.12..3.13).contains(&widest), "{widest}");
    assert!((1.10..1.11).contains(&narrowest), "{narrowest}");
}

/// The finding: twelve exact matches, which is not how families behave.
#[test]
fn cleveland_renewals_are_the_prior_years_payments_exactly_until_fy2010() {
    let applied = history::series("cleveland", Measure::Applications);
    let paid = history::series("cleveland", Measure::Used);

    let mut matched = Vec::new();
    for (year, counts) in &applied {
        let Some(previous) = paid.get(&FiscalYear(year.0 - 1)) else {
            continue;
        };
        let renewals = counts
            .renewal
            .expect("every year publishes a renewal count");
        let before = previous
            .total
            .expect("every year publishes a payment total");
        if (renewals - before).abs() < f64::EPSILON {
            matched.push(year.0);
        }
    }

    // FY1998 through FY2009. An identity holding twelve times running is a rule, and the rule is
    // that everyone paid last year was carried into this year's renewal count administratively.
    assert_eq!(
        matched,
        (1998..=ROLL_FORWARD_LAST_YEAR).collect::<Vec<u16>>()
    );
}

/// And what the roll-forward was hiding: attrition, every year, once anyone counted it.
#[test]
fn the_years_after_the_break_lose_a_tenth_of_the_prior_years_recipients() {
    let applied = history::series("cleveland", Measure::Applications);
    let paid = history::series("cleveland", Measure::Used);

    for year in (ROLL_FORWARD_LAST_YEAR + 1)..=2013 {
        let renewals = applied[&FiscalYear(year)]
            .renewal
            .expect("every year publishes a renewal count");
        let before = paid[&FiscalYear(year - 1)]
            .total
            .expect("every year publishes a payment total");
        let change = renewals / before - 1.0;
        assert!(
            (-0.15..-0.09).contains(&change),
            "FY{year} renewals are {change:+.3} against the prior year's payments"
        );
    }
}

/// The break is a definitional one, and the low-income columns date it independently.
///
/// `Renewal and Qualified as Low Income` is empty for every Cleveland year through FY2009 and
/// populated from FY2010 — the same boundary, from a different column. Two columns changing in the
/// same year is what makes this a change in how the sheet was compiled rather than a coincidence
/// in the counts.
#[test]
fn the_low_income_split_begins_in_the_same_year_the_identity_ends() {
    let applied = history::series("cleveland", Measure::Applications);
    for (year, counts) in &applied {
        let has_split = counts.renewal_low_income.is_some();
        assert_eq!(
            has_split,
            year.0 > ROLL_FORWARD_LAST_YEAR,
            "FY{} disagrees with the boundary the counts mark",
            year.0
        );
    }
}

/// Whatever the convention was, it was Cleveland's alone.
#[test]
fn traditional_edchoice_renewals_never_match_the_prior_years_payments() {
    let applied = history::series("traditional-edchoice", Measure::Applications);
    let paid = history::series("traditional-edchoice", Measure::Used);
    assert_eq!(applied.len(), 7, "FY2007 through FY2013");

    for (year, counts) in &applied {
        let Some(previous) = paid.get(&FiscalYear(year.0 - 1)) else {
            continue;
        };
        let renewals = counts
            .renewal
            .expect("every year publishes a renewal count");
        let before = previous
            .total
            .expect("every year publishes a payment total");
        assert!(
            (renewals - before).abs() > 1.0,
            "FY{} renewals equal the prior year's payments, as Cleveland's did",
            year.0
        );
    }
}

/// Cleveland's tutoring grants stop being reported, and stopping is not the same as zero.
#[test]
fn the_tutoring_column_goes_absent_rather_than_to_nothing() {
    let tutoring = history::series("cleveland", Measure::Tutoring);

    // Thirteen years, FY1997 through FY2009. The workbook reads `NA` for the four after that,
    // which is the department declining to report rather than reporting none — so there is no row
    // at all, and a consumer summing the column cannot silently add four zeroes.
    assert_eq!(tutoring.len(), 13);
    assert_eq!(tutoring.keys().map(|y| y.0).max(), Some(2009));
    assert!(tutoring.values().all(|c| c.total.is_some_and(|n| n > 0.0)));

    // And no other programme has one.
    let elsewhere = history::observations()
        .into_iter()
        .filter(|o| o.measure == Measure::Tutoring && o.program != "cleveland")
        .count();
    assert_eq!(elsewhere, 0);
}

/// Autism publishes payments and no applications, and Jon Peterson has exactly one year.
///
/// Both are the department's sheets rather than gaps in the reader, and both are asserted so that
/// a later edition filling either one is noticed rather than absorbed.
#[test]
fn two_programmes_are_thinner_than_the_other_two() {
    let autism = history::observations()
        .into_iter()
        .filter(|o| o.program == "autism")
        .collect::<Vec<_>>();
    assert_eq!(autism.len(), 10, "FY2004 through FY2013");
    assert!(autism.iter().all(|o| o.measure == Measure::Used));
    assert!(
        autism.iter().all(|o| o.counts.new.is_none()),
        "the sheet's note says its system could not distinguish new from renewal"
    );

    let jon_peterson = history::series("jon-peterson", Measure::Used);
    assert_eq!(jon_peterson.len(), 1);
    assert!(
        jon_peterson.contains_key(&FiscalYear(2013)),
        "the programme's first year"
    );
}
