//! The tangible personal property replacement payments, year by year, across four line items.
//!
//! H.B. 66 replaced a local tax with a state payment and scheduled the payment to phase down.
//! Every later budget changed the schedule, and two of them changed the line it was paid from.
//! `revenue-stream/tpp-replacement-payments` states the schedule each act enacted; this is the
//! money it moved, so that a sentence about the phase-down is checked against what was paid.

use project::ledger::appropriations::{
    latest_actual, lines, tpp_replacement_lines, tpp_replacement_paid,
};

/// Business and utility run side by side through FY2015, one merged line carries them after, and
/// the succession has no gap from FY2006 to FY2027.
#[test]
fn every_year_has_its_lines() {
    for year in 2006..=2015 {
        assert_eq!(
            tpp_replacement_lines(year),
            vec!["200909", "200900"],
            "FY{year}"
        );
    }
    for year in 2016..=2025 {
        assert_eq!(tpp_replacement_lines(year), vec!["200902"], "FY{year}");
    }
    for year in 2026..=2027 {
        assert_eq!(tpp_replacement_lines(year), vec!["200417"], "FY{year}");
    }
    assert!(tpp_replacement_lines(2005).is_empty());
    assert!(tpp_replacement_lines(2028).is_empty());
}

/// Each seam is a real handover: the outgoing lines are at zero in the year the next one opens.
#[test]
fn the_outgoing_line_is_empty_where_the_next_begins() {
    assert_eq!(latest_actual("200909", 2016), Some(0.0));
    assert_eq!(latest_actual("200900", 2016), Some(0.0));
    let hb96_200902_fy2026 = lines()
        .into_iter()
        .find(|l| l.line_item == "200902" && l.fiscal_year == 2026 && l.kind == "enacted")
        .expect("H.B. 96 carries 200902 for FY2026");
    assert_eq!(hb96_200902_fy2026.amount, 0.0);
}

/// An actual exists for every year from the first payment to the last one reported.
#[test]
fn every_year_through_fy2025_has_an_actual() {
    for year in 2006..=2025 {
        let paid = tpp_replacement_paid(year).unwrap_or_else(|| panic!("no actual for FY{year}"));
        assert!(paid > 0.0, "FY{year} paid {paid}");
    }
    assert_eq!(tpp_replacement_paid(2026), None, "FY2026 has no actual yet");
}

/// The Catalog's spending is preferred to the FY2008-09 greenbook's FY2007 estimate.
#[test]
fn fy2007_is_what_was_spent_not_what_was_estimated() {
    assert_eq!(latest_actual("200909", 2007), Some(418_161_375.0));
    let greenbook_row = lines()
        .into_iter()
        .find(|l| l.line_item == "200909" && l.fiscal_year == 2007 && l.kind == "actual")
        .expect("the FY2008-09 greenbook carries an FY2007 row");
    assert_eq!(greenbook_row.amount, 420_000_000.0);
}

/// The two halves sum to what LSC reports appropriated for FY2015, which is the check that the
/// merged FY2016 line is compared against the right predecessor.
///
/// H.B. 64's greenbook: "In FY 2015, reimbursement payments to school districts are appropriated
/// at $510.0 million." Business and utility together spent within half a per cent of it and not
/// above it; the business line alone falls $28 million short.
#[test]
fn fy2015_business_and_utility_together_are_lscs_510_million() {
    let both = tpp_replacement_paid(2015).unwrap();
    assert!(both <= 510_050_000.0, "{both}");
    assert!(both / 510_000_000.0 > 0.995, "{both}");
    let business = latest_actual("200909", 2015).unwrap();
    assert!(510_000_000.0 - business > 25_000_000.0, "{business}");
}

/// The peak is FY2011, and from FY2012 every year paid less than the year before except two.
///
/// H.B. 153 froze the payments at their FY2013 level, so FY2014 and FY2015 sit within a fraction
/// of a per cent of FY2013 and are the only years from FY2012 that do not fall by more than one.
#[test]
fn the_payments_peak_in_fy2011_and_then_only_fall() {
    let paid: Vec<(u16, f64)> = (2006..=2025)
        .map(|y| (y, tpp_replacement_paid(y).unwrap()))
        .collect();
    let peak = paid.iter().max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
    assert_eq!(peak.0, 2011);
    for pair in paid.windows(2).filter(|w| w[1].0 >= 2012) {
        let change = pair[1].1 / pair[0].1 - 1.0;
        if matches!(pair[1].0, 2014 | 2015) {
            assert!(change.abs() < 0.001, "FY{} moved {change}", pair[1].0);
        } else {
            assert!(change < -0.01, "FY{} moved {change}", pair[1].0);
        }
    }
}

/// The supplement H.B. 64 created was paid in three years, though appropriated in one.
#[test]
fn the_supplement_was_paid_in_three_years() {
    let paid: Vec<f64> = (2016..=2019)
        .map(|y| latest_actual("200697", y).unwrap())
        .collect();
    assert_eq!(paid, vec![39_290_230.0, 44_208_587.0, 9_467_328.0, 0.0]);
    let enacted_fy2017 = lines()
        .into_iter()
        .find(|l| l.line_item == "200697" && l.fiscal_year == 2017 && l.kind == "enacted")
        .expect("H.B. 64 carries an FY2017 row for the supplement");
    assert_eq!(
        enacted_fy2017.amount, 0.0,
        "the FY2017 appropriation was vetoed"
    );
}

/// The seams in the money are seams in the statute.
///
/// 5751.21 holds H.B. 66's schedule and stops paying after June 30, 2015, the year before 200909
/// empties; 5751.20 defines H.B. 153's thresholds as two per cent for FY2012 and four from FY2013;
/// 5709.92 is H.B. 64's replacement, with S.B. 208's yearly reduction of one-sixteenth of one per
/// cent of taxable value and an end to the operating fixed-sum levy payments after tax year 2021.
#[test]
fn the_schedule_is_in_the_vendored_statute() {
    let old = project::statute::section("5751.21").body;
    assert!(old.contains(
        "No determinations, computations, certifications, or payments shall be made under this \
         section after June 30, 2015."
    ));
    let definitions = project::statute::section("5751.20").body;
    assert!(definitions.contains(
        "two per cent for fiscal year 2012 and four per cent for fiscal years 2013 and thereafter"
    ));
    let new = project::statute::section("5709.92").body;
    assert!(new.contains(
        "One-sixteenth of one per cent of the average of the total taxable value of the district \
         for tax years 2014, 2015, and 2016."
    ));
    assert!(new.contains(
        "No payment shall be made under division (D)(1) of this section after tax year 2021."
    ));
    assert!(
        new.contains("two per cent for fiscal year 2016 and four per cent for fiscal year 2017")
    );
}
