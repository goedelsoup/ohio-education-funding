//! The shape of Ohio's real per-pupil spending series, and what an endpoint comparison hides.
//!
//! The nominal series rises monotonically from FY2000 to FY2022 and is almost always presented
//! that way — as a single "+117% nominal, +26% real" pair, or as a smooth upward chart. Deflated
//! year by year it is not monotone. It has a trough, and the trough is large enough that a
//! speaker describing it is describing something real.
//!
//! This matters because "has Ohio been defunding schools" is routinely tested endpoint to
//! endpoint. OCG White Paper 015 answers "No — up ~25–26% real" under its per-pupil-reduction
//! standard, correctly for FY2000→FY2022. Over FY2010→FY2014 the same series falls about 7%.
//! Both are true. The disagreement in the public argument is not only definitional, which that
//! paper establishes well; it is also **periodic**, which no choice of definition surfaces.
//!
//! Cited by `corpus/metric/per-pupil-operating-expenditure.yml` and
//! `catalog/ocg-white-paper-015.md`.

use deflator::ohio_epp::{nominal_series, quotable, real_at, real_series, BASE_YEAR};
use deflator::CpiSeries;
use edfund_core::FiscalYear;

/// The series restated in constant FY2022 dollars, as (fiscal year, real dollars).
fn series() -> Vec<(u16, f64)> {
    real_series(BASE_YEAR)
        .into_iter()
        .map(|(point, real)| (point.fiscal_year.0, real.value))
        .collect()
}

fn at(year: u16) -> f64 {
    real_at(FiscalYear(year))
}

#[test]
fn the_endpoints_reproduce_the_auditor_and_the_corpus() {
    // Both endpoints are exact on the nominal side and verified on the index side, so these are
    // the two rows that carry no approximation at all.
    assert!((at(2000) - 12_143.0).abs() < 1.0);
    assert!((at(2022) - 15_314.0).abs() < 1.0);
    let growth = at(2022) / at(2000) - 1.0;
    assert!(
        (growth - 0.261).abs() < 0.002,
        "real growth over the full window is {growth:.3}, not the published +26.1%"
    );
}

/// **The trough.** Real per-pupil spending fell for four years and did not recover for eight.
#[test]
fn real_per_pupil_spending_fell_from_fy2010_to_fy2014() {
    let peak = at(2010);
    let bottom = at(2014);
    assert!(bottom < peak, "{bottom:.0} against {peak:.0}");

    let decline = bottom / peak - 1.0;
    assert!(
        (decline - -0.069).abs() < 0.005,
        "the FY2010-FY2014 real decline is {decline:.3}"
    );

    // FY2014 lands back on FY2006. Eight years, no real gain.
    assert!(
        (bottom - at(2006)).abs() < 100.0,
        "FY2014 {bottom:.0} against FY2006 {:.0}",
        at(2006)
    );

    // And the recovery does not clear FY2010 until FY2018.
    assert!(at(2016) < peak);
    assert!(at(2018) < peak);
    assert!(at(2020) > peak);
}

/// The decline is an order of magnitude larger than the uncertainty in its inputs.
///
/// Every interior row rests on a chart label read to about $100. Perturbing both endpoints of
/// the trough in the direction that would erase it leaves it clearly intact, which is why the
/// corpus states it despite the inputs being `[inference]`.
#[test]
fn the_trough_survives_the_chart_label_uncertainty() {
    let cpi = CpiSeries::cpi_u_june();
    let base = FiscalYear(2022);
    // Push FY2010 down by $100 and FY2014 up by $100 — the most favourable case for "no decline".
    let low_peak = cpi
        .convert(11_200.0 - 100.0, FiscalYear(2010), base)
        .unwrap()
        .value;
    let high_bottom = cpi
        .convert(11_400.0 + 100.0, FiscalYear(2014), base)
        .unwrap()
        .value;
    assert!(
        high_bottom < low_peak,
        "even at the favourable extreme the trough holds: {high_bottom:.0} against {low_peak:.0}"
    );
    assert!(low_peak - high_bottom > 700.0);
}

/// **The real peak is FY2020, not FY2022.** The nominal high and the real high are two years
/// apart, because relief arrived while prices were low and was then eroded by FY2021-22
/// inflation.
#[test]
fn the_real_peak_precedes_the_nominal_peak_by_two_years() {
    let series = series();
    let (peak_year, peak_value) =
        series.iter().copied().fold(
            (0u16, f64::MIN),
            |acc, (y, v)| if v > acc.1 { (y, v) } else { acc },
        );
    assert_eq!(peak_year, 2020, "the real series peaks in FY{peak_year}");

    // The nominal series peaks at the end, which is why the distinction is invisible undeflated.
    let nominal_peak = nominal_series()
        .iter()
        .fold((0u16, f64::MIN), |acc, p| {
            if p.dollars > acc.1 {
                (p.fiscal_year.0, p.dollars)
            } else {
                acc
            }
        })
        .0;
    assert_eq!(nominal_peak, 2022);

    let below_peak = at(2022) - peak_value;
    assert!(
        (below_peak - -433.0).abs() < 40.0,
        "FY2022 sits {below_peak:.0} from the real peak"
    );
    assert!(below_peak / peak_value < -0.02);
}

/// An endpoint-to-endpoint real change can be the opposite sign of an interior window, and
/// nothing in the endpoint pair says so. Stated as a test because it is the general rule the
/// series happens to demonstrate.
#[test]
fn an_endpoint_comparison_cannot_see_an_interior_reversal() {
    let series = series();
    let full = series.last().unwrap().1 / series.first().unwrap().1 - 1.0;
    assert!(full > 0.0, "the full window rises");

    let reversals: Vec<(u16, u16, f64)> = series
        .windows(2)
        .filter(|w| w[1].1 < w[0].1)
        .map(|w| (w[0].0, w[1].0, w[1].1 / w[0].1 - 1.0))
        .collect();
    assert!(
        reversals.len() >= 3,
        "a monotone reading of this series is wrong in {} places",
        reversals.len()
    );
    // The worst single interval is a fall of more than 4%.
    let worst = reversals
        .iter()
        .map(|(_, _, r)| *r)
        .fold(f64::MAX, f64::min);
    assert!(worst < -0.04, "worst two-year interval is {worst:.3}");
}

/// Confidence propagates from the weaker endpoint, so the trough rows are `inference` and the
/// corpus tags them that way. Asserted so a future series refresh cannot quietly promote them.
#[test]
fn interior_rows_are_inference_and_the_endpoints_are_not() {
    for (point, real) in real_series(BASE_YEAR) {
        let year = point.fiscal_year.0;
        let is_quotable = quotable(&point, &real);
        assert_eq!(
            is_quotable,
            year == 2000 || year == 2022,
            "FY{year} quotability is {is_quotable}"
        );
    }
}
