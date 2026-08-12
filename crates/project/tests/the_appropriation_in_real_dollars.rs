//! What the appropriation series says once the dollars are made comparable.
//!
//! Ohio's school funding argument is conducted in nominal totals, and nearly every biennium in
//! this period can truthfully be called a record investment. These tests pin what the same series
//! says in constant dollars, because the two statements point opposite ways and both are correct.

use edfund_core::FiscalYear;
use project::appropriations::{enacted_history, growth, line_history, lines, TAX_REIMBURSEMENT};

/// The base year every figure here is denominated in: the last year the price index covers.
const BASE: FiscalYear = FiscalYear(2026);

/// The years that can be deflated — FY2027 is appropriated and the index does not reach it.
fn deflated(history: Vec<project::appropriations::Year>) -> Vec<project::appropriations::Year> {
    history.into_iter().filter(|y| y.real.is_some()).collect()
}

/// A named window of the series.
///
/// Every growth figure here states the years it is over, rather than taking the first and last of
/// whatever the series happens to hold. That was how these tests were written when the series
/// began at FY2014, and extending it back to FY2002 silently changed what they measured: the
/// formula line grows 3.9% in real terms over FY2014-FY2026 and falls 3.8% over FY2008-FY2026,
/// and both are true.
fn window(
    history: Vec<project::appropriations::Year>,
    from: u16,
    to: u16,
) -> Vec<project::appropriations::Year> {
    deflated(history)
        .into_iter()
        .filter(|y| y.fiscal_year >= from && y.fiscal_year <= to)
        .collect()
}

#[test]
fn the_department_grew_by_a_third_nominally_and_shrank_in_real_terms() {
    // The finding. Between FY2014 and FY2026 the enacted appropriation rose 29% in cash and fell
    // 8% against CPI-U. Neither number is wrong and neither is the whole story; what this test
    // guards is that the repository cannot report one without the other being available.
    let history = window(enacted_history(BASE), 2014, 2026);
    let (nominal, real, _) = growth(&history).expect("both endpoints deflate");
    assert!(
        nominal > 0.25 && nominal < 0.33,
        "nominal growth is {nominal}, no longer the +29% this series showed"
    );
    assert!(
        real < 0.0,
        "the real-terms series no longer falls; it is {real}, and the corpus says it does"
    );
    assert!(
        real > -0.12 && real < -0.04,
        "real growth is {real}, outside the band this finding has held"
    );
}

#[test]
fn the_formula_line_moved_the_other_way_from_the_department_around_it() {
    // Foundation Funding is the formula itself, and it is the one large line that gained ground:
    // +3.9% real while the department it sits in lost 7.9%. Whatever else the Fair School Funding
    // Plan did, it did not come out of a growing budget — it came out of a shrinking one.
    let foundation = window(line_history("200550", BASE), 2014, 2026);
    let (nominal, real, _) = growth(&foundation).expect("both endpoints deflate");
    assert!(
        nominal > 0.40,
        "the formula line's nominal growth is {nominal}"
    );
    assert!(
        real > 0.0,
        "Foundation Funding no longer grows in real terms; it is {real}"
    );

    let department = window(enacted_history(BASE), 2014, 2026);
    let (_, department_real, _) = growth(&department).expect("both endpoints deflate");
    assert!(
        real > department_real,
        "the formula line {real} no longer outpaces the department around it {department_real}"
    );
}

#[test]
fn the_last_appropriated_year_has_no_real_figure_rather_than_a_nominal_one() {
    // FY2027 is enacted and the index stops at FY2026. Reporting its cash amount in a column
    // headed "constant dollars" is the failure `deflate` exists to prevent, so the year is
    // carried with `real: None` and every caller here filters on it.
    let history = enacted_history(BASE);
    let last = history.last().expect("the series is not empty");
    assert_eq!(last.fiscal_year, 2027);
    assert!(last.real.is_none(), "FY2027 acquired a real figure");
    assert!(last.nominal > 0.0, "FY2027 lost its nominal figure");
}

#[test]
fn the_tax_reimbursement_lines_are_excluded_and_are_not_small() {
    // They are numbered as the department's and are not its budget, on the publisher's own
    // statement. This asserts both halves: that they are out of the total, and that leaving them
    // in would have mattered.
    let total: f64 = lines()
        .iter()
        .filter(|line| line.kind == "enacted" && line.fiscal_year == 2026)
        .map(|line| line.amount)
        .sum();
    let department = enacted_history(BASE)
        .into_iter()
        .find(|year| year.fiscal_year == 2026)
        .expect("FY2026 is in the series");
    let excluded = total - department.nominal;
    assert!(
        excluded > 1_000_000_000.0,
        "the excluded reimbursement lines are only {excluded}; either they have shrunk or the \
         exclusion has stopped working"
    );
    assert_eq!(TAX_REIMBURSEMENT.len(), 2);
}

#[test]
fn the_long_view_says_something_different_from_the_short_one() {
    // What the extension back to FY2002 bought, and the reason every figure here names its years.
    // Over FY2014-FY2026 the formula's line gains 3.9% in real terms; over FY2008-FY2026 — the
    // whole span the same line item exists for — it loses ground. The Fair School Funding Plan
    // recovered something that had been given up earlier, which is a different claim from growth.
    let short = window(line_history("200550", BASE), 2014, 2026);
    let long = window(line_history("200550", BASE), 2008, 2026);
    let (_, short_real, _) = growth(&short).expect("both endpoints deflate");
    let (_, long_real, _) = growth(&long).expect("both endpoints deflate");
    assert!(
        short_real > 0.0,
        "the short window no longer gains: {short_real}"
    );
    assert!(
        long_real < 0.0,
        "the long window no longer loses: {long_real}"
    );
    assert!(
        long[0].fiscal_year == 2008,
        "200550 now reaches before FY2008 and this window has changed meaning"
    );
}
