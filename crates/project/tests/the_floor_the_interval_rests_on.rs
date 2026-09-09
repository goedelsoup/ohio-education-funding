//! Whether the projection band is the floor it says it is.
//!
//! [`project::series`] states plainly where its interval comes from: *"we do not know how
//! variable this district is, but we know how much districts differ from one another, and that
//! is a defensible floor on the uncertainty."* [`report::enrollment_growth_prior`] builds it —
//! the cross-sectional standard deviation of district annual enrolled-ADM growth over the three
//! years the department publishes, widened by the square root of the horizon.
//!
//! Two claims live in that sentence and neither had been checked. That the band is a **floor**:
//! true forecast error is at least this wide. And that `sqrt(horizon)` is how it should widen.
//! The F-33 panel can now check both, because the same fourteen years of district enrolment that
//! fitted the damping (see [`the-fitted-damping`]) also hold what actually happened after every
//! forecast a backtest can make.
//!
//! # The floor holds, at every horizon, under both methods
//!
//! Realized dispersion of out-of-sample log forecast error, against the band the production
//! prior claims at the same horizon. σ = 0.0242, z = 1.
//!
//! | horizon | claimed ±1σ | realized, as shipped | ratio | coverage |
//! |---|--:|--:|--:|--:|
//! | 1 year | 0.0242 | 0.0285 | 1.18× | 67.3% |
//! | 2 | 0.0342 | 0.0407 | 1.19× | 66.1% |
//! | 3 | 0.0419 | 0.0522 | 1.25× | 63.8% |
//! | 4 | 0.0484 | 0.0645 | 1.33× | 60.6% |
//! | 5 | 0.0541 | 0.0756 | 1.40× | 55.8% |
//!
//! **The band never over-states.** That is the claim, and it survives: at no horizon, under
//! either the shipped `Shrunk` method or the plain `Damped` one it replaced, is realized error
//! narrower than the interval drawn around it.
//!
//! # But it is a floor that sinks, and the square root is why
//!
//! A ±1σ band on a normal should cover 68.3%. At one year the prior covers **67.3%** — very
//! nearly calibrated, which is the more surprising half of this result: a spread taken across
//! districts turns out to be about the right size for a single district's one-year error. By
//! five years it covers **55.8%**.
//!
//! The decay is not in σ, it is in the widening rule. Realized dispersion grows as
//! **`horizon^0.60`**, not the `horizon^0.50` [`Prior::spread`] applies. Forecast errors are
//! positively autocorrelated — a district whose trend is misjudged stays misjudged, and the
//! random-walk assumption that each year's error is drawn fresh is what a school district does
//! not do.
//!
//! This is stable rather than an artefact of one period. At five years, by origin:
//!
//! | origin → target | ratio |
//! |---|--:|
//! | FY2013 → FY2018 | 1.34× |
//! | FY2015 → FY2020 | 1.44× |
//! | FY2018 → FY2023 | 1.36× |
//! | FY2019 → FY2024 | 1.36× |
//!
//! # The published horizons are past where this can see
//!
//! Five years is as far as six origins across fifteen surveyed years will reach. The FY2032 leg
//! in [`scenario/guarantee-phase-out`] is **six**, and the feed's horizon is FY2036, which is
//! **ten**. 1.40× is therefore a lower bound on how much the band under-states there, and under
//! an exponent that is still climbing at the edge of the measurement it is a weak one.
//!
//! # A bias the mean absolute error could not show
//!
//! Mean log error runs −0.005 at one year to **+0.023** at five: the method increasingly
//! over-forecasts, predicting more children than arrive. [`the-fitted-damping`] and
//! [`the-shrunk-rate`] both scored on *absolute* error, which is blind to sign, so neither would
//! have found this. It is small against the dispersion — a fifth of it at five years — but it is
//! one-directional, and Ohio enrolment has been falling for the whole panel.
//!
//! The shrink helps here too, on a metric it was not chosen for: against the plain damped method
//! it cuts the five-year bias from +0.030 to +0.023 and the dispersion at every horizon. That is
//! independent evidence for a decision taken on other grounds.
//!
//! # What this does not do
//!
//! It does not recalibrate anything. σ, z and the square root are published constants that the
//! web layer re-derives and must reproduce, and moving them is a decision with a record, not a
//! consequence of a test file. What is settled here is that the sentence describing them is
//! true, that it is true by a measured margin, and that the margin is a function of horizon
//! nobody had looked at.
//!
//! [`the-fitted-damping`]: ../../../.yidam/decisions/the-fitted-damping.yml
//! [`the-shrunk-rate`]: ../../../.yidam/decisions/the-shrunk-rate.yml
//! [`scenario/guarantee-phase-out`]: ../../../.yidam/corpus/scenario/guarantee-phase-out.yml

use dispersion::ohio_panel::{self, PanelRow};
use edfund_core::FiscalYear;
use project::series::{self, Method, Observation, Prior, ONE_SIGMA};
use project::{panel, report};
use std::collections::BTreeMap;

/// Every fiscal year the panel carries, oldest first. FY2014 is absent from the archive.
const PANEL_YEARS: [u16; 15] = [
    2009, 2010, 2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023, 2024,
];

/// The years a forecast is made from. FY2020 onward is excluded so no fit spans the closure.
const ORIGINS: [u16; 6] = [2013, 2015, 2016, 2017, 2018, 2019];

/// Targets no method is scored on: the year enrolment fell 54,777 in one step, and the rebound.
const PANDEMIC: [u16; 2] = [2021, 2022];

/// The damping and shrink weight these figures were measured at, pinned rather than imported.
///
/// The same reasoning as `THE_CONVENTION` in [`the_damping_nobody_fitted`]: a file that followed
/// the constants would turn a finding about the shipped method into a tautology about whatever
/// ships. The *prior* is deliberately not pinned — it is the thing under test, so a recalibration
/// should fail this file rather than slip past it.
const SHIPPING_DAMPING: f64 = 0.30;
const SHIPPING_WEIGHT: f64 = 0.30;

/// The fraction of a normal distribution inside ±1σ, which is what the band claims to draw.
const NORMAL_ONE_SIGMA_COVERAGE: f64 = 0.683;

/// One district's enrolment history, keyed by fiscal year.
type History = BTreeMap<u16, f64>;

/// Districts with an observation in every year of the panel, keyed on `LEAID`.
fn complete_histories() -> Vec<History> {
    let mut by_district: BTreeMap<String, History> = BTreeMap::new();
    for row in ohio_panel::panel()
        .into_iter()
        .filter(|r: &PanelRow| r.comparable)
    {
        if row.enrollment > 0.0 {
            by_district
                .entry(row.leaid.clone())
                .or_default()
                .insert(row.fiscal_year, row.enrollment);
        }
    }
    by_district
        .into_values()
        .filter(|h| PANEL_YEARS.iter().all(|y| h.contains_key(y)))
        .collect()
}

fn compound_rate(from: f64, to: f64, years: f64) -> f64 {
    if from <= 0.0 || to <= 0.0 || years <= 0.0 {
        return 0.0;
    }
    (to / from).powf(1.0 / years) - 1.0
}

/// The long-run rate as of `origin` — computed from panel start to the origin only.
///
/// Production takes this rate over the whole panel, which ends at the same place its ADM history
/// does. A backtest cannot: a rate that has seen the target year is not a forecast. So the window
/// here is shorter than the one that ships, which makes the shrink a little weaker than it is in
/// the feed and the figures below a little conservative.
fn long_run_to(history: &History, origin: u16) -> f64 {
    let years: Vec<u16> = PANEL_YEARS
        .iter()
        .copied()
        .filter(|y| *y <= origin)
        .collect();
    let (first, last) = (years[0], *years.last().expect("an origin is in the panel"));
    compound_rate(history[&first], history[&last], f64::from(last - first))
}

/// The three observations a production caller fits on, as of `origin`.
fn observed_to(history: &History, origin: u16) -> Vec<Observation> {
    let mut observed: Vec<Observation> = PANEL_YEARS
        .iter()
        .filter(|y| **y <= origin)
        .filter_map(|y| {
            history.get(y).map(|value| Observation {
                fiscal_year: FiscalYear(*y),
                value: *value,
            })
        })
        .collect();
    let drop = observed.len().saturating_sub(3);
    observed.drain(..drop);
    observed
}

/// Which projection method a run uses. `rate` is refitted by `series::fit`, so the zero passed
/// below is ignored and only the parameters under test survive.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// What shipped before [`the-shrunk-rate`], kept as the comparison.
    Damped,
    /// What the feed uses now.
    Shrunk,
}

fn method(shape: Shape, history: &History, origin: u16) -> Method {
    match shape {
        Shape::Damped => Method::Damped {
            rate: 0.0,
            damping: SHIPPING_DAMPING,
        },
        Shape::Shrunk => Method::Shrunk {
            rate: 0.0,
            damping: SHIPPING_DAMPING,
            weight: SHIPPING_WEIGHT,
            toward: long_run_to(history, origin),
        },
    }
}

/// Log forecast errors at one horizon, over every origin and district the panel allows.
fn log_errors(histories: &[History], shape: Shape, horizon: u16) -> Vec<f64> {
    let mut errors = Vec::new();
    for history in histories {
        for origin in ORIGINS {
            let target = origin + horizon;
            if PANDEMIC.contains(&target) {
                continue;
            }
            let Some(actual) = history.get(&target) else {
                continue;
            };
            let point = series::project(
                &observed_to(history, origin),
                FiscalYear(target),
                method(shape, history, origin),
                Prior::none(),
            )
            .into_iter()
            .find(|p| p.fiscal_year == FiscalYear(target))
            .expect("the projection reaches the target year")
            .point;
            errors.push((point / actual).ln());
        }
    }
    errors
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

/// Sample standard deviation.
fn stdev(values: &[f64]) -> f64 {
    let m = mean(values);
    let n = values.len() as f64;
    (values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
}

/// The production prior, taken from the path the feed takes.
fn prior() -> Prior {
    report::enrollment_growth_prior(&panel::panel(), ONE_SIGMA)
}

/// The band is a floor: realized error is wider than the interval at every horizon, both methods.
///
/// This is the claim `series`'s module doc makes, and the one direction that matters — a band
/// that over-stated would be reporting confidence it had not earned.
#[test]
fn realized_forecast_error_is_wider_than_the_band_at_every_horizon() {
    let histories = complete_histories();
    let prior = prior();
    for shape in [Shape::Damped, Shape::Shrunk] {
        for horizon in 1..=5u16 {
            let realized = stdev(&log_errors(&histories, shape, horizon));
            let claimed = prior.spread(horizon);
            assert!(
                realized > claimed,
                "at horizon {horizon} the band claims {claimed:.5} and error is {realized:.5} — \
                 the interval would be over-stating, not flooring"
            );
        }
    }
}

/// It is a floor by 1.18x at one year and 1.40x at five, as shipped.
///
/// The margin is the finding, not just its sign: a floor 18% below the truth is a usable
/// interval and one 40% below it is not the same object.
#[test]
fn the_margin_widens_from_one_fifth_to_two_fifths_across_five_years() {
    let histories = complete_histories();
    let prior = prior();
    let ratio =
        |horizon| stdev(&log_errors(&histories, Shape::Shrunk, horizon)) / prior.spread(horizon);
    let (one, five) = (ratio(1), ratio(5));
    assert!(
        (one - 1.18).abs() < 0.02,
        "one-year ratio should be about 1.18x, is {one:.3}x"
    );
    assert!(
        (five - 1.40).abs() < 0.02,
        "five-year ratio should be about 1.40x, is {five:.3}x"
    );
    assert!(
        five > one,
        "the floor should sink with horizon: {one:.3}x at one year, {five:.3}x at five"
    );
}

/// Coverage is very nearly right at one year and has lost a ninth of itself by five.
///
/// A cross-sectional spread being the correct size for one district's one-year error is not
/// something the sentence claimed — it claimed only a floor — so this is the prior doing better
/// than advertised at the horizon nobody publishes, and worse at the ones that are published.
#[test]
fn the_one_year_band_is_nearly_calibrated_and_the_five_year_band_is_not() {
    let histories = complete_histories();
    let prior = prior();
    let coverage = |horizon| {
        let errors = log_errors(&histories, Shape::Shrunk, horizon);
        let claimed = prior.spread(horizon);
        errors.iter().filter(|e| e.abs() <= claimed).count() as f64 / errors.len() as f64
    };
    let (one, five) = (coverage(1), coverage(5));
    assert!(
        (one - NORMAL_ONE_SIGMA_COVERAGE).abs() < 0.02,
        "one-year coverage {one:.3} should sit within a couple of points of {NORMAL_ONE_SIGMA_COVERAGE}"
    );
    assert!(
        (0.50..0.60).contains(&five),
        "five-year coverage should be near 0.558, is {five:.3}"
    );
}

/// Dispersion grows as `horizon^0.60`, and `Prior::spread` widens it as `horizon^0.50`.
///
/// The exponent is fitted by least squares on the logs of the five measured dispersions. That
/// errors compound faster than a random walk is the mechanism behind every row of the coverage
/// table, and it is a property of the widening rule rather than of sigma — no rescaling of the
/// prior fixes a band that is the wrong shape in the horizon.
#[test]
fn the_dispersion_grows_faster_than_the_square_root_the_band_assumes() {
    let histories = complete_histories();
    let points: Vec<(f64, f64)> = (1..=5u16)
        .map(|h| {
            (
                f64::from(h).ln(),
                stdev(&log_errors(&histories, Shape::Shrunk, h)).ln(),
            )
        })
        .collect();
    let (xs, ys): (Vec<f64>, Vec<f64>) = points.iter().copied().unzip();
    let (mx, my) = (mean(&xs), mean(&ys));
    let exponent = xs
        .iter()
        .zip(&ys)
        .map(|(x, y)| (x - mx) * (y - my))
        .sum::<f64>()
        / xs.iter().map(|x| (x - mx).powi(2)).sum::<f64>();
    assert!(
        (exponent - 0.604).abs() < 0.02,
        "the fitted exponent should be about 0.60, is {exponent:.3}"
    );
    assert!(
        exponent > 0.5,
        "a square-root band assumes 0.5 and the data says {exponent:.3}; if this ever drops to \
         0.5 the widening rule has become right and this whole file is stale"
    );
}

/// The five-year shortfall is not one bad period: four origins give 1.34x to 1.44x.
#[test]
fn the_five_year_shortfall_holds_across_every_origin_that_reaches_that_far() {
    let histories = complete_histories();
    let prior = prior();
    let mut measured = Vec::new();
    for origin in ORIGINS {
        let target = origin + 5;
        if PANDEMIC.contains(&target) {
            continue;
        }
        let errors: Vec<f64> = histories
            .iter()
            .filter_map(|history| {
                let actual = history.get(&target)?;
                let point = series::project(
                    &observed_to(history, origin),
                    FiscalYear(target),
                    method(Shape::Shrunk, history, origin),
                    Prior::none(),
                )
                .into_iter()
                .find(|p| p.fiscal_year == FiscalYear(target))?
                .point;
                Some((point / actual).ln())
            })
            .collect();
        if errors.len() < 2 {
            continue;
        }
        measured.push((origin, stdev(&errors) / prior.spread(5)));
    }
    assert_eq!(
        measured.len(),
        4,
        "four origins reach five years without landing on a pandemic target"
    );
    for (origin, ratio) in &measured {
        assert!(
            (1.30..1.50).contains(ratio),
            "origin {origin} gives {ratio:.2}x, outside the 1.34-1.44 band the others sit in"
        );
    }
}

/// The method over-forecasts, and by more the further out it goes.
///
/// Both fitting decisions scored on absolute error, which cannot see a sign. This is what they
/// were blind to: at five years the mean log error is +0.023, so the central projection is
/// systematically above what arrives.
#[test]
fn a_positive_bias_grows_with_horizon_and_neither_fitting_decision_could_see_it() {
    let histories = complete_histories();
    let one = mean(&log_errors(&histories, Shape::Shrunk, 1));
    let five = mean(&log_errors(&histories, Shape::Shrunk, 5));
    assert!(
        one < 0.0,
        "the one-year mean error is slightly negative, is {one:+.5}"
    );
    assert!(
        (five - 0.0232).abs() < 0.003,
        "the five-year mean error should be about +0.023, is {five:+.5}"
    );
    assert!(
        five.abs() < stdev(&log_errors(&histories, Shape::Shrunk, 5)) / 3.0,
        "the bias should stay small against the dispersion — it is a tilt, not the main error"
    );
}

/// The shrink narrowed the band's shortfall as well as the error it was chosen for.
///
/// `the-shrunk-rate` was decided on mean absolute log error. It also improves dispersion at every
/// horizon and cuts the five-year bias, neither of which was part of the case for it.
#[test]
fn the_shrunk_rate_improves_the_calibration_it_was_not_chosen_for() {
    let histories = complete_histories();
    for horizon in 1..=5u16 {
        let damped = stdev(&log_errors(&histories, Shape::Damped, horizon));
        let shrunk = stdev(&log_errors(&histories, Shape::Shrunk, horizon));
        assert!(
            shrunk < damped,
            "at horizon {horizon} the shrink should narrow dispersion: {shrunk:.5} vs {damped:.5}"
        );
    }
    let damped_bias = mean(&log_errors(&histories, Shape::Damped, 5));
    let shrunk_bias = mean(&log_errors(&histories, Shape::Shrunk, 5));
    assert!(
        shrunk_bias < damped_bias,
        "the shrink should cut the five-year bias: {shrunk_bias:+.5} vs {damped_bias:+.5}"
    );
}
