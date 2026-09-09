//! What the projection band held, which was not what it claimed, and what it holds now.
//!
//! [`project::series`] states plainly where its interval comes from: *"we do not know how
//! variable this district is, but we know how much districts differ from one another, and that
//! is a defensible floor on the uncertainty."* [`report::enrollment_growth_prior`] builds it —
//! the cross-sectional standard deviation of district annual enrolled-ADM growth over the three
//! years the department publishes, widened by the horizon.
//!
//! Two claims lived in that sentence and neither had been checked. That the band is a **floor**:
//! true forecast error is at least this wide. And that a **square root** is how it should widen.
//! The F-33 panel that fitted the damping can check both. The first held. The second did not, and
//! [`series::HORIZON_EXPONENT`] is what came of it.
//!
//! # The floor holds, at every horizon, under both methods and both rules
//!
//! Realized out-of-sample error is wider than the interval drawn around it at every horizon from
//! one year to five, under the shipped `Shrunk` method and the `Damped` one it replaced, before
//! the exponent moved and after. **The band never over-states.** That is the one claim in the
//! sentence that survived, and it survives the fix as well — widening the band did not overshoot
//! into claiming more precision than the errors show.
//!
//! # But under a square root it lost coverage with every year
//!
//! A one-sigma band should hold about **68.3%**. What it held:
//!
//! | horizon | `horizon^0.50` — what shipped | `horizon^0.65` — fitted |
//! |---|--:|--:|
//! | 1 year | 67.3% | 67.3% |
//! | 2 | 66.1% | 70.4% |
//! | 3 | 63.8% | 71.2% |
//! | 4 | 60.6% | 68.4% |
//! | 5 | **55.8%** | **66.5%** |
//! | worst gap from 68.3% | **12.5 pts** | **2.9 pts** |
//!
//! At one year the two are identical and always will be — `1^k` is 1 for every `k` — which is
//! also the more surprising half of the original result: a spread taken *across districts* turns
//! out to be very nearly the right size for *one* district's one-year error. Everything the
//! exponent was doing wrong, it was doing to the years after the first.
//!
//! # Fitted to coverage, which is not the same as fitted to dispersion
//!
//! Realized dispersion grows as **`horizon^0.60`**, and matching it would have been the obvious
//! move. It is the wrong target: the error distribution is not normal, so a band matching its
//! standard deviation does not hold the share of it a normal band would. Fitted against coverage
//! directly the answer is **0.65**, and the surface is flat from about 0.63 to 0.66 — 0.64
//! minimises the worst horizon and 0.65 the average one.
//!
//! The mechanism behind both numbers is the same. Forecast errors are positively autocorrelated:
//! a district whose trend is misjudged stays misjudged, and the random-walk assumption that each
//! year's error is drawn fresh is what a school district does not do.
//!
//! This was stable rather than an artefact of one period. Under the square root at five years, by
//! origin: FY2013 1.34x, FY2015 1.44x, FY2018 1.36x, FY2019 1.36x.
//!
//! # The published horizons are still past where this can see
//!
//! Five years is as far as six origins across fifteen surveyed years will reach. The FY2032 leg
//! in [`scenario/guarantee-phase-out`] is **six** and the feed's horizon is FY2036, which is
//! **ten**. The fit is extrapolated there either way — but from an exponent the data supports
//! rather than one it contradicts inside the measured range.
//!
//! # A bias the mean absolute error could not show
//!
//! Mean log error runs −0.005 at one year to **+0.023** at five: the method increasingly
//! over-forecasts. [`the-fitted-damping`] and [`the-shrunk-rate`] both scored on *absolute*
//! error, which is blind to sign, so neither would have found this.
//! `the_bias_no_single_damping_can_remove` takes it from here, and the answer is that no value of
//! the damping removes it.
//!
//! The shrink helps here too, on a metric it was not chosen for: against the plain damped method
//! it cuts the five-year bias from +0.030 to +0.023 and the dispersion at every horizon.
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

/// The exponent the band widened by before it was fitted, pinned rather than imported.
///
/// The same reasoning as `THE_CONVENTION` in [`the_damping_nobody_fitted`]: this file's findings
/// are about a rule the repository no longer uses, and a file that followed
/// [`series::HORIZON_EXPONENT`] would turn them into tautologies about whatever ships. The
/// *prior* is deliberately still imported — it is what the exponent multiplies, its provenance
/// did not change with the fit, and a recalibration of it should fail this file rather than slip
/// past.
const THE_RANDOM_WALK: f64 = 0.5;

/// The band a given exponent draws at `horizon`, from the production prior.
fn band(prior: Prior, horizon: u16, exponent: f64) -> f64 {
    prior.z * prior.sigma * f64::from(horizon).powf(exponent)
}

/// The share of out-of-sample errors that band holds.
fn coverage(errors: &[f64], width: f64) -> f64 {
    errors.iter().filter(|e| e.abs() <= width).count() as f64 / errors.len() as f64
}

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

/// Under the square root the floor sank from 1.18x at one year to 1.40x at five.
///
/// The margin was the finding, not just its sign: a floor 18% below the truth is a usable
/// interval and one 40% below it is not the same object. Measured against `THE_RANDOM_WALK`
/// rather than the shipped exponent, because this is a statement about the rule that was
/// replaced.
#[test]
fn the_square_root_rule_sank_from_one_fifth_to_two_fifths_across_five_years() {
    let histories = complete_histories();
    let prior = prior();
    let ratio = |horizon| {
        stdev(&log_errors(&histories, Shape::Shrunk, horizon))
            / band(prior, horizon, THE_RANDOM_WALK)
    };
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
        "the floor should sink with horizon under the old rule: {one:.3}x then {five:.3}x"
    );
}

/// It lost coverage with every year, from 67.3% to 55.8%.
///
/// A cross-sectional spread being the correct size for one district's one-year error is not
/// something the sentence claimed — it claimed only a floor — so the prior was doing better than
/// advertised at the horizon nobody publishes, and worse at the ones that are.
#[test]
fn the_square_root_rule_lost_coverage_with_every_year() {
    let histories = complete_histories();
    let prior = prior();
    let held = |horizon| {
        coverage(
            &log_errors(&histories, Shape::Shrunk, horizon),
            band(prior, horizon, THE_RANDOM_WALK),
        )
    };
    let (one, five) = (held(1), held(5));
    assert!(
        (one - NORMAL_ONE_SIGMA_COVERAGE).abs() < 0.02,
        "one-year coverage {one:.3} should sit within a couple of points of \
         {NORMAL_ONE_SIGMA_COVERAGE}"
    );
    assert!(
        (0.50..0.60).contains(&five),
        "five-year coverage should be near 0.558, is {five:.3}"
    );
    for pair in (1..=5u16).map(held).collect::<Vec<_>>().windows(2) {
        assert!(
            pair[1] < pair[0],
            "coverage should fall at every step under the old rule: {:.3} then {:.3}",
            pair[0],
            pair[1]
        );
    }
}

/// What ships holds it flat instead — never more than three points from where it should be.
///
/// The fix and its acceptance test. `1^k` is 1 for every `k`, so the one-year figure is untouched
/// by construction and the whole of the improvement is in the years after the first, which is
/// where the whole of the defect was.
#[test]
fn the_fitted_exponent_holds_coverage_within_three_points_at_every_horizon() {
    let histories = complete_histories();
    let prior = prior();
    let held: Vec<f64> = (1..=5u16)
        .map(|h| coverage(&log_errors(&histories, Shape::Shrunk, h), prior.spread(h)))
        .collect();

    let worst = held
        .iter()
        .map(|c| (c - NORMAL_ONE_SIGMA_COVERAGE).abs())
        .fold(0.0f64, f64::max);
    assert!(
        worst < 0.03,
        "the fitted rule should hold coverage within three points; worst is {:.1} points, from \
         {held:?}",
        worst * 100.0
    );

    let was_worst = (1..=5u16)
        .map(|h| {
            coverage(
                &log_errors(&histories, Shape::Shrunk, h),
                band(prior, h, THE_RANDOM_WALK),
            )
        })
        .map(|c| (c - NORMAL_ONE_SIGMA_COVERAGE).abs())
        .fold(0.0f64, f64::max);
    assert!(
        was_worst > worst * 3.0,
        "and it should be several times better than the square root's {:.1} points",
        was_worst * 100.0
    );

    assert!(
        (held[0]
            - coverage(
                &log_errors(&histories, Shape::Shrunk, 1),
                band(prior, 1, THE_RANDOM_WALK)
            ))
        .abs()
            < 1e-12,
        "one year is identical under both rules, because 1 to any power is 1"
    );
}

/// Dispersion grows as `horizon^0.60`, where the square root assumed 0.50.
///
/// The exponent is fitted by least squares on the logs of the five measured dispersions. That
/// errors compound faster than a random walk is the mechanism behind every row of the coverage
/// table, and it is a property of the widening rule rather than of sigma — no rescaling of the
/// prior fixes a band that is the wrong shape in the horizon.
///
/// **This is not the number that shipped**, and the difference is the point: 0.60 is what matches
/// the *dispersion* of the error, and `HORIZON_EXPONENT` is 0.65 because it is fitted to
/// *coverage*. The two would agree if the errors were normal. They are not, so the band that
/// matches the standard deviation is not the band that holds 68% of the mass.
#[test]
fn the_dispersion_grows_faster_than_the_square_root_the_band_used_to_assume() {
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
        exponent > THE_RANDOM_WALK,
        "a square-root band assumed 0.5 and the data says {exponent:.3}"
    );
}

/// The square root's five-year shortfall was not one bad period: four origins gave 1.34x to 1.44x.
///
/// Against `THE_RANDOM_WALK`, like the two above it — a statement about the rule that was
/// replaced, and the reason the replacement is a fit rather than a patch over one odd window.
#[test]
fn the_square_root_shortfall_held_across_every_origin_that_reaches_that_far() {
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
        measured.push((origin, stdev(&errors) / band(prior, 5, THE_RANDOM_WALK)));
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
