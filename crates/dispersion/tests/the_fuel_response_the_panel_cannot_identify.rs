//! Whether Ohio's transportation spending tracks the price of diesel, asked of the only panel
//! that could answer it.
//!
//! `f33_student_transportation.rs` pins the shape of the series and says why nothing is fitted
//! to it there: the FY2021 collapse is a school closure, not a price, and it is nearly collinear
//! with the fuel crash of calendar 2020. Two more years of the column were acquired precisely
//! because FY2023 is a large real move that no closure explains — the identifying variation the
//! earlier panel was missing. This file spends that variation and reports what it bought.
//!
//! It bought a negative result, and the negative result is the finding.
//!
//! # Diesel does not identify a fuel response here, under any alignment
//!
//! Eight specifications — two ways of collapsing a monthly price to a fiscal year, crossed with
//! the thirteen NCES years and all fifteen, crossed with and without a linear trend. **Not one
//! of them reaches |t| = 1.5**, and the slope is not even stable in sign: it runs from −$4.01 to
//! +$19.62 of real per-pupil spending per dollar-per-gallon. The extra variation did not rescue
//! the estimate; it moved it.
//!
//! What is significant in every trend specification is the trend itself, at t = 2.1 to 3.6. Real
//! per-pupil transportation rises about $4 to $5 a year whatever diesel does, and once that is
//! allowed for there is nothing left for fuel to explain. A pass-through estimated from this
//! panel would be reporting the drift.
//!
//! This is not a claim that districts are indifferent to fuel. It is a claim about what fifteen
//! annual observations of a survey can support, and it is the reason the corpus carries the
//! pass-through as unquantified rather than as a number with a wide interval.
//!
//! # The alignment is load-bearing, and the counterfactual is what is new here
//!
//! `connect::fixtures::diesel` commits the series monthly and says why in prose: fuel is bought
//! all year, so a July-to-June *mean* is the natural fiscal-year alignment where the deflator's
//! June *point* is right for an index. That was an argument for a convention.
//!
//! The convention is **already in use and already pinned**:
//! `the_refundable_tax_widens_the_rise_rather_than_narrowing_it`, in that module, computes the
//! July-to-June means and holds FY2025 to FY2026 at +17.5% retail, which is the figure
//! `parameter/transportation-cost-rates` publishes. None of that is restated as new.
//!
//! What was never measured is the **cost of the other choice**. The June point over those same
//! two years gives **+39.6%** — the same claim, 2.3 times larger, and it would have made the
//! per-rider rate's +4.88% look like a twelfth of fuel rather than a quarter of it.
//!
//! # One thing this understates, by the amount that module established
//!
//! Every slope below is fitted to *retail* diesel. Ohio refunds school districts the 47-cent
//! motor fuel tax under R.C. 5735.05(E)(2), and subtracting a fixed wedge from both ends of a
//! rise widens it, so a district's fuel bill moves further than this series does. A pass-through
//! taken from retail prices is therefore biased *toward zero*. That cuts the wrong way for the
//! finding here — the true coefficient is larger than any of these — and it is still nowhere
//! near significant, so the negative result survives the correction rather than depending on it.
//!
//! **The mechanism is one month.** June 2022, at $5.662, is the second-highest observation in
//! the whole 389-month series. The June rule assigns that spike entirely to FY2022; the mean
//! rule spreads it and leaves FY2023 as the high-diesel year. Real spending peaks in FY2023. So
//! the June point puts the price maximum a year *before* the spending maximum and the mean puts
//! them together — which is how the same panel yields −$2.49 under one rule and +$9.66 under the
//! other.

use deflator::CpiSeries;
use dispersion::ohio_panel::{self, PanelRow};
use edfund_core::{csv, FiscalYear};

/// The base year every real figure here is stated in, as in the sibling file.
const BASE_YEAR: u16 = 2022;

/// The monthly price series, which lives in the crate that retrieves it.
///
/// Included by path rather than through a dependency, as `regime-diff` includes this crate's
/// abstract of valuations: a committed fixture is data, and a reader of it does not need the
/// retrieval machinery that produced it.
const DIESEL: &str = include_str!("../../connect/fixtures/midwest-diesel-monthly.csv");

/// The header this reader indexes.
const DIESEL_HEADER: &str = "calendar_year,month,dollars_per_gallon";

/// One month's Midwest No. 2 diesel retail price.
#[derive(Debug, Clone, Copy)]
struct Observation {
    calendar_year: u16,
    month: u32,
    dollars_per_gallon: f64,
}

/// The series, oldest first.
fn diesel() -> Vec<Observation> {
    csv::rows(DIESEL, DIESEL_HEADER)
        .filter_map(|row| {
            Some(Observation {
                calendar_year: row.num(0)? as u16,
                month: row.num(1)? as u32,
                dollars_per_gallon: row.num(2)?,
            })
        })
        .collect()
}

/// The mean of the twelve months of Ohio's fiscal year: July of `fy - 1` through June of `fy`.
///
/// A twin of the helper in `connect::fixtures::diesel`'s test module, which cannot be shared: it
/// is `#[cfg(test)]` and so exists only inside that crate's own test binary. The difference is
/// deliberate — this one returns `None` unless all twelve months are present, so a fiscal year
/// the series only half covers drops out of a regression rather than being silently averaged
/// over six.
fn fiscal_year_mean(series: &[Observation], fy: u16) -> Option<f64> {
    let months: Vec<f64> = series
        .iter()
        .filter(|o| {
            (o.calendar_year == fy - 1 && o.month >= 7) || (o.calendar_year == fy && o.month <= 6)
        })
        .map(|o| o.dollars_per_gallon)
        .collect();
    (months.len() == 12).then(|| months.iter().sum::<f64>() / 12.0)
}

/// The price in the single month the fiscal year ends in — the deflator's alignment, applied to
/// a flow it does not fit.
fn june_point(series: &[Observation], fy: u16) -> Option<f64> {
    series
        .iter()
        .find(|o| o.calendar_year == fy && o.month == 6)
        .map(|o| o.dollars_per_gallon)
}

/// Comparable districts only, as everywhere this panel is used.
fn comparable() -> Vec<PanelRow> {
    ohio_panel::panel()
        .into_iter()
        .filter(|r| r.comparable)
        .collect()
}

/// Every fiscal year the panel carries, oldest first.
fn years(rows: &[PanelRow]) -> Vec<u16> {
    let mut all: Vec<u16> = rows.iter().map(|r| r.fiscal_year).collect();
    all.sort_unstable();
    all.dedup();
    all
}

/// Enrolment-weighted transportation spending per pupil, in [`BASE_YEAR`] dollars.
fn real_per_pupil(rows: &[PanelRow], year: u16) -> f64 {
    let (spend, pupils) = rows
        .iter()
        .filter(|r| r.fiscal_year == year)
        .filter_map(|r| Some((r.student_transportation?, r.enrollment)))
        .fold((0.0, 0.0), |(s, p), (spend, pupils)| {
            (s + spend, p + pupils)
        });
    CpiSeries::cpi_u_june()
        .convert(spend / pupils, FiscalYear(year), FiscalYear(BASE_YEAR))
        .expect("the CPI series covers every year of the panel")
        .value
}

/// A slope and its t-statistic.
#[derive(Debug, Clone, Copy)]
struct Fit {
    slope: f64,
    t: f64,
}

/// Ordinary least squares of `ys` on `ys`' intercept and `xs`, with `extra` further parameters
/// already partialled out of both.
///
/// `extra` is the count of regressors removed beforehand, and it exists only to get the degrees
/// of freedom right: by Frisch–Waugh the slope and the residual sum of squares of a multiple
/// regression are exactly those of the bivariate regression on residualized inputs, but the
/// variance divides by `n - 2 - extra` rather than `n - 2`. Passing the wrong `extra` would
/// understate the standard error of a trend specification, which is the direction that would
/// manufacture the significance this file reports the absence of.
///
/// `project::series` fits a least-squares trend and is not what this needs: it regresses on the
/// fiscal year alone, returns no standard error, and is private to a forecasting module in a
/// crate this one does not depend on. A slope without a standard error cannot report a null.
fn fit(xs: &[f64], ys: &[f64], extra: usize) -> Fit {
    let n = xs.len();
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let (mx, my) = (mean(xs), mean(ys));
    let sxx: f64 = xs.iter().map(|x| (x - mx).powi(2)).sum();
    let sxy: f64 = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (x - mx) * (y - my))
        .sum::<f64>();
    let slope = sxy / sxx;
    let intercept = my - slope * mx;
    let sse: f64 = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (y - (intercept + slope * x)).powi(2))
        .sum();
    let se = (sse / (n - 2 - extra) as f64 / sxx).sqrt();
    Fit {
        slope,
        t: slope / se,
    }
}

/// `values` with the least-squares fit on a linear trend removed.
fn detrend(values: &[f64], trend: &[f64]) -> Vec<f64> {
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let (mt, mv) = (mean(trend), mean(values));
    let stt: f64 = trend.iter().map(|t| (t - mt).powi(2)).sum();
    let stv: f64 = trend
        .iter()
        .zip(values)
        .map(|(t, v)| (t - mt) * (v - mv))
        .sum::<f64>();
    let slope = stv / stt;
    trend
        .iter()
        .zip(values)
        .map(|(t, v)| (v - mv) - slope * (t - mt))
        .collect()
}

/// Every specification this file reports, so that the sweep is written once.
///
/// Returns the label, the fit on diesel, and the fit on the trend where one is included.
fn specifications() -> Vec<(String, Fit, Option<Fit>)> {
    let rows = comparable();
    let all = years(&rows);
    let series = diesel();
    let mut out = Vec::new();
    for (panel_label, panel) in [
        (
            "13 NCES years",
            all.iter()
                .copied()
                .filter(|y| *y <= 2022)
                .collect::<Vec<_>>(),
        ),
        ("15 years", all.clone()),
    ] {
        for (align_label, align) in [
            (
                "June point",
                june_point as fn(&[Observation], u16) -> Option<f64>,
            ),
            ("July-June mean", fiscal_year_mean),
        ] {
            let xs: Vec<f64> = panel
                .iter()
                .map(|y| align(&series, *y).expect("the diesel series covers every year"))
                .collect();
            let ys: Vec<f64> = panel.iter().map(|y| real_per_pupil(&rows, *y)).collect();
            let trend: Vec<f64> = panel.iter().map(|y| f64::from(*y) - 2009.0).collect();
            out.push((
                format!("{panel_label}, {align_label}, no trend"),
                fit(&xs, &ys, 0),
                None,
            ));
            out.push((
                format!("{panel_label}, {align_label}, with trend"),
                fit(&detrend(&xs, &trend), &detrend(&ys, &trend), 1),
                Some(fit(&trend, &ys, 0)),
            ));
        }
    }
    out
}

/// No alignment of diesel, over either panel, with or without a trend, finds a fuel response.
///
/// The bound is |t| < 1.5, which at any of these sample sizes is far from any conventional
/// threshold — the largest is 1.43 and the next is 0.90. Stated as a bound rather than as eight
/// values so that the test says the thing it exists to say: a year that made one of these
/// significant should fail here and be looked at, not be absorbed.
#[test]
fn no_alignment_of_diesel_identifies_a_fuel_response_in_this_panel() {
    let specs = specifications();
    assert_eq!(
        specs.len(),
        8,
        "two panels, two alignments, two trend choices"
    );
    for (label, diesel, _) in &specs {
        assert!(
            diesel.t.abs() < 1.5,
            "{label}: diesel reached t = {:+.2} (slope {:+.2}), which this panel should not \
             support",
            diesel.t,
            diesel.slope
        );
    }
}

/// And the slope is not stable in sign, which is the sharper way to say it is not identified.
///
/// An insignificant coefficient of consistent sign is a weak estimate. One that changes sign
/// when the alignment changes is not an estimate at all, and both happen here: over all fifteen
/// years with a trend, the June point gives −$2.49 per dollar-per-gallon and the July-to-June
/// mean gives +$9.66.
#[test]
fn the_sign_of_the_estimate_depends_on_the_alignment_rather_than_the_data() {
    let specs = specifications();
    let slope = |needle: &str| {
        specs
            .iter()
            .find(|(label, _, _)| label == needle)
            .map(|(_, d, _)| d.slope)
            .expect("the sweep names this specification")
    };
    let june = slope("15 years, June point, with trend");
    let mean = slope("15 years, July-June mean, with trend");
    assert!(june < 0.0, "the June point gives {june:+.2}");
    assert!(mean > 0.0, "the July-June mean gives {mean:+.2}");
    assert!((-3.0..-2.0).contains(&june), "{june:+.4}");
    assert!((9.0..10.0).contains(&mean), "{mean:+.4}");
}

/// What the series has instead of a fuel response is a trend, and the trend is significant.
///
/// Every specification that includes one puts it above t = 2, and the two fifteen-year ones
/// above t = 3. Real per-pupil transportation drifts upward at about $4 to $5 a year. That drift
/// is the variation a naive pass-through regression would be spending.
#[test]
fn what_the_panel_supports_is_a_trend_and_not_a_price() {
    for (label, diesel, trend) in specifications() {
        let Some(trend) = trend else { continue };
        assert!(
            trend.t > 2.0,
            "{label}: the trend reached only t = {:+.2}",
            trend.t
        );
        assert!(
            trend.t > diesel.t.abs(),
            "{label}: diesel {:+.2} against a trend of {:+.2}",
            diesel.t,
            trend.t
        );
        assert!(
            (3.0..6.0).contains(&trend.slope),
            "{label}: ${:.2} a year",
            trend.slope
        );
    }
}

/// What the other alignment would have cost: the same two years, 2.3 times the rise.
///
/// The mean side is not this test's finding. `connect::fixtures::diesel`'s
/// `the_refundable_tax_widens_the_rise_rather_than_narrowing_it` already holds FY2025 to FY2026
/// at +17.5%, and `parameter/transportation-cost-rates` publishes it in support of the claim
/// that the per-rider rate "rose about a quarter as fast" as fuel. It is recomputed here only as
/// the anchor for the comparison, and a drift in it should fail there first.
///
/// The finding is the other number. Under the June point the same two years rise **39.6%**, and
/// the rate would read as an eighth of fuel rather than a quarter. A convention that moves a
/// published figure by that much is a result, not a house style.
#[test]
fn the_alignment_the_corpus_did_not_take_would_have_more_than_doubled_the_rise() {
    let series = diesel();
    let growth = |f: fn(&[Observation], u16) -> Option<f64>| {
        let from = f(&series, 2025).expect("FY2025 is covered");
        let to = f(&series, 2026).expect("FY2026 is covered");
        (to / from - 1.0) * 100.0
    };
    let mean = growth(fiscal_year_mean);
    let june = growth(june_point);
    assert!(
        (17.5..17.6).contains(&mean),
        "the corpus states 17.5%; the mean gives {mean:.4}%"
    );
    assert!(
        (39.6..39.7).contains(&june),
        "the June point gives {june:.4}%"
    );
    assert!(
        june / mean > 2.2,
        "the two alignments differ by {:.2}x",
        june / mean
    );
}

/// Why the alignment moves the estimate: it moves the peak a whole year.
///
/// June 2022 is the second-highest month in the series. Under the June rule FY2022 is the
/// high-diesel year; under the mean rule FY2023 is. Real spending peaks in FY2023. One rule puts
/// the price maximum on the spending maximum and the other puts it a year early, which is a sign
/// flip in a fifteen-point regression.
#[test]
fn the_june_point_puts_the_price_peak_a_year_before_the_spending_peak() {
    let rows = comparable();
    let panel = years(&rows);
    let series = diesel();

    let peak = |f: fn(&[Observation], u16) -> Option<f64>| {
        panel
            .iter()
            .copied()
            .max_by(|a, b| {
                f(&series, *a)
                    .unwrap_or(0.0)
                    .total_cmp(&f(&series, *b).unwrap_or(0.0))
            })
            .expect("the panel is not empty")
    };
    let spending_peak = panel
        .iter()
        .copied()
        .max_by(|a, b| real_per_pupil(&rows, *a).total_cmp(&real_per_pupil(&rows, *b)))
        .expect("the panel is not empty");

    assert_eq!(
        spending_peak, 2023,
        "real per-pupil transportation peaks here"
    );
    assert_eq!(
        peak(fiscal_year_mean),
        2023,
        "the mean agrees with spending"
    );
    assert_eq!(peak(june_point), 2022, "the June point is a year early");

    let june_2022 = june_point(&series, 2022).expect("June 2022 is in the series");
    let higher = series
        .iter()
        .filter(|o| o.dollars_per_gallon > june_2022)
        .count();
    assert_eq!(
        higher, 1,
        "June 2022 at ${june_2022:.3} should be the second-highest month of the series"
    );
}
