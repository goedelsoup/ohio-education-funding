//! The join `revenue-stream/esser` asks for, run.
//!
//! The node records the question and the route: *"Whether districts spent ESSER on recurring or
//! one-time costs, and which ones are now cutting, needs the FY2025 and FY2026 five-year
//! forecasts read against the ESSER allocations — both of which exist, and neither of which is
//! joined here."* Both are here — the forecasts as [`project::finances`], the relief money as the
//! federal column of [`dispersion::ohio_panel`] — and this joins them.
//!
//! # The control comes first
//!
//! ESSER was never general-fund money, and the arithmetic says so plainly: across the peak,
//! federal revenue in these districts rose 139% while their general-fund revenue fell. That is
//! not a limitation of the instrument, it is what makes the instrument work. A district that
//! spent relief on one-time costs leaves no general-fund trace; one that spent it on recurring
//! costs has to carry them somewhere once the grant ends, and the general fund is the somewhere.
//!
//! # And the prediction fails
//!
//! Split on relief per pupil, the four quartiles' general-fund spending grows 20.2%, 19.5%,
//! 24.6% and 19.3% from least-relieved to most. Not ordered, and the most exposed quartile is
//! the slowest-growing of the four. The cash drawdown is the same tenth of a year of spending in
//! all four. The correlation is 0.027.
//!
//! What that answers is the systematic question and not the district-by-district one. Ohio's
//! districts did not, as a population, move relief-funded costs onto the general fund in
//! proportion to what they received. Which particular districts did needs the special revenue
//! funds, and the five-year forecast does not report them.

use project::esser;

/// The relief bulge is unmistakable in the federal column and absent from the general fund.
#[test]
fn federal_revenue_more_than_doubled_and_the_general_fund_did_not_move() {
    let series = esser::federal_against_the_general_fund();

    let baseline = series[&esser::BASELINE_YEAR]
        .federal_revenue
        .expect("FY2019 federal");
    let peak = series[&2022].federal_revenue.expect("FY2022 federal");
    assert!(
        (baseline / 1e9 - 1.579).abs() < 0.001 && (peak / 1e9 - 3.776).abs() < 0.001,
        "federal revenue {baseline} to {peak}"
    );
    assert!(
        peak / baseline > 2.3,
        "the relief bulge is {:.2}x",
        peak / baseline
    );

    // The general fund over the same span. FY2019 is before the forecast filings reach, so the
    // comparison starts at FY2020 — which is still a pre-relief year for the general fund.
    let before = series[&2020]
        .general_fund_revenue
        .expect("FY2020 general fund");
    let at_the_peak = series[&2022]
        .general_fund_revenue
        .expect("FY2022 general fund");
    assert!(
        at_the_peak < before,
        "general-fund revenue went {before} to {at_the_peak}"
    );
    assert!(
        (at_the_peak / before - 1.0).abs() < 0.005,
        "and it moved by {:.4}",
        at_the_peak / before - 1.0
    );
}

/// Spending growth after the cliff does not order by how much relief a district had.
#[test]
fn general_fund_spending_growth_does_not_order_by_relief_exposure() {
    let quartiles = esser::by_exposure();
    assert_eq!(
        quartiles.map(|q| q.districts),
        [151, 151, 151, 154],
        "districts per exposure quartile"
    );

    // The split is real: the most-relieved quartile received nearly three times the least.
    let (least, most) = (quartiles[0].surge_per_pupil, quartiles[3].surge_per_pupil);
    assert!(
        (least - 808.0).abs() < 1.0 && (most - 2_322.0).abs() < 1.0,
        "median relief per pupil {least} against {most}"
    );
    assert!(most / least > 2.8, "the exposure ratio is {}", most / least);

    let growth = quartiles.map(|q| q.spending_growth);
    for (i, expected) in [0.2015, 0.1950, 0.2458, 0.1935].iter().enumerate() {
        assert!(
            (growth[i] - expected).abs() < 5e-4,
            "quartile {i} grew {}",
            growth[i]
        );
    }
    assert!(
        growth[3] < growth[0],
        "the most exposed quartile grew {} against the least exposed {}",
        growth[3],
        growth[0]
    );
}

/// Neither does the cash drawdown.
///
/// The other shape a recurring cost would take: a district carrying one without new revenue eats
/// its balance. Every quartile eats about the same tenth of a year of spending.
#[test]
fn neither_does_the_cash_drawdown() {
    let drawdown = esser::by_exposure().map(|q| q.cash_years_change);
    assert!(
        drawdown.iter().all(|d| *d < 0.0),
        "every quartile draws down: {drawdown:?}"
    );
    let (deepest, shallowest) = (
        drawdown.iter().copied().fold(f64::MAX, f64::min),
        drawdown.iter().copied().fold(f64::MIN, f64::max),
    );
    assert!(
        (deepest - -0.1136).abs() < 5e-4 && (shallowest - -0.0944).abs() < 5e-4,
        "drawdowns run {deepest} to {shallowest}"
    );
    assert!(
        drawdown[3] > drawdown[0],
        "the most exposed quartile draws down less, not more: {:?}",
        drawdown
    );
}

/// The correlation is within noise of zero.
#[test]
fn the_association_between_relief_and_later_spending_is_nothing() {
    let corr = esser::exposure_against_spending_growth();
    assert!(
        (corr - 0.0274).abs() < 5e-4,
        "correlation between relief per pupil and later spending growth is {corr}"
    );
    assert!(corr.abs() < 0.05, "and it is {corr}");
}

/// Twenty-seven districts are spending less than they were, and they are not the relieved ones.
///
/// This is the "which ones are now cutting" half of the node's question, answered as far as the
/// general fund can answer it: hardly any district is cutting in nominal terms, and the ones
/// that are are spread across every exposure quartile.
#[test]
fn the_districts_spending_less_are_spread_across_every_exposure_quartile() {
    let all = esser::districts();
    assert_eq!(all.len(), 607, "districts both sources reach");

    let size = all.len() / 4;
    let mut by_quartile = [0_usize; 4];
    for (rank, district) in all.iter().enumerate() {
        if district.spending_growth < 0.0 {
            by_quartile[(rank / size).min(3)] += 1;
        }
    }
    assert_eq!(
        by_quartile.iter().sum::<usize>(),
        27,
        "districts spending less in FY2025 than FY2023"
    );
    assert!(
        by_quartile.iter().all(|n| *n >= 2),
        "every quartile holds some: {by_quartile:?}"
    );
    assert!(
        by_quartile[3] < 12,
        "and the most exposed quartile holds {} of the 27",
        by_quartile[3]
    );
}
