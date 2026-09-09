//! Whether the point estimate can be de-biased, which was left open and is answerable no.
//!
//! [`the_trend_the_damping_discards`] found the projections running positive and more so the
//! further out they go, and `metric/enrolled-adm` recorded the remedy as an open question: whether
//! the point estimate should be de-biased, given that 0.30 minimises the error both fitting
//! decisions were made on. This answers it, and the answer is that **there is no value of
//! `DEFAULT_DAMPING` that removes the bias**, because the bias is structured in the horizon and
//! the parameter is not.
//!
//! # No scalar damping is unbiased at more than one horizon
//!
//! The damping that zeroes the mean log error, solved for at each horizon separately:
//!
//! | horizon | unbiased at damping |
//! |---|--:|
//! | 1 year | **unreachable** — see below |
//! | 2 | ≈ 0.00 |
//! | 3 | 0.32 |
//! | 4 | 0.64 |
//! | 5 | 0.79 |
//!
//! It rises monotonically, so any single value is over-damped for the near years and under-damped
//! for the far ones. What ships, 0.30, is very nearly the unbiased choice at three years and
//! leaves +0.023 at five. Choosing 0.79 to zero the five-year bias would put −0.010 on the second.
//! There is no setting that does both, and this is a property of applying one decay rate to every
//! horizon rather than anything about Ohio.
//!
//! # And at one year the parameter is not connected to the outcome at all
//!
//! [`series::advance`] decays the rate from the *second* step onward, so the first projected year
//! is `value * (1 + rate)` whatever the damping is. The one-year bias is therefore **−0.00492104
//! at every damping in the range, to the last digit this measures** — a floor the constant cannot
//! reach, and one that is negative where the long-horizon bias is positive. It comes from the
//! fitted rate, not the decay.
//!
//! # Pricing the bias explicitly barely moves the optimum
//!
//! [`the-fitted-damping`] and [`the-shrunk-rate`] both scored on mean *absolute* log error, which
//! cannot see a sign. That is how the bias went unnoticed, and it is a fair question whether it
//! also made them choose badly. It did not:
//!
//! | criterion | optimum |
//! |---|--:|
//! | mean absolute log error — what was used | 0.40 |
//! | root mean squared log error — prices bias and variance together | 0.45 |
//!
//! The decomposition says why. At 0.30 the squared bias is `0.0000190` against a variance of
//! `0.0029624` — **six-tenths of one percent of the mean squared error.** A criterion blind to
//! sign was ignoring almost nothing.
//!
//! # The one place the choice does move is the horizon the feed publishes
//!
//! Scored on four- and five-year forecasts alone, the optimum is **0.55**, and 0.30 costs 1.4%:
//!
//! | damping | RMSE at h=4,5 | bias |
//! |---|--:|--:|
//! | 0.30 — what ships | 0.07258 | +0.01731 |
//! | 0.55 — the long-horizon optimum | 0.07153 | ≈ +0.009 |
//! | 1.00 | 0.08730 | −0.02229 |
//!
//! That is the strongest available case for moving the constant and it is not a strong one: the
//! feed publishes every year from FY2027 to FY2036, not only the far ones, and pooled across
//! horizons 0.30 sits within 0.4% of every optimum measured here. The surface is flat from about
//! 0.30 to 0.55.
//!
//! # So the question closes as a no, and names what a yes would require
//!
//! De-biasing the point estimate is not an operation this parameter performs. It would take a
//! **horizon-dependent** damping — a change to the shape of [`Method::Damped`] and to the
//! TypeScript that mirrors it, not to a number — and the case for that rests on a five-year
//! backtest being extrapolated to a ten-year horizon. `the_floor_the_interval_rests_on` says what
//! is already known about extrapolating past five.
//!
//! [`the-fitted-damping`]: ../../../.yidam/decisions/the-fitted-damping.yml
//! [`the-shrunk-rate`]: ../../../.yidam/decisions/the-shrunk-rate.yml

use dispersion::ohio_panel::{self, PanelRow};
use edfund_core::FiscalYear;
use project::series::{self, Method, Observation, Prior, DEFAULT_DAMPING, DEFAULT_SHRINK_WEIGHT};
use std::collections::BTreeMap;

/// Every fiscal year the panel carries, oldest first. FY2014 is absent from the archive.
const PANEL_YEARS: [u16; 15] = [
    2009, 2010, 2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023, 2024,
];

/// The years a forecast is made from. FY2020 onward is excluded so no fit spans the closure.
const ORIGINS: [u16; 6] = [2013, 2015, 2016, 2017, 2018, 2019];

/// Targets no method is scored on: the year enrolment fell 54,777 in one step, and the rebound.
const PANDEMIC: [u16; 2] = [2021, 2022];

/// One district's enrolment history, keyed by fiscal year.
type History = BTreeMap<u16, f64>;

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

/// The long-run rate as of `origin` only — a rate that has seen the target is not a forecast.
fn long_run_to(history: &History, origin: u16) -> f64 {
    let years: Vec<u16> = PANEL_YEARS
        .iter()
        .copied()
        .filter(|y| *y <= origin)
        .collect();
    let last = *years.last().expect("an origin is in the panel");
    compound_rate(
        history[&years[0]],
        history[&last],
        f64::from(last - years[0]),
    )
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

/// Out-of-sample log errors at one damping, over the horizons asked for, by the shipped method.
fn log_errors(histories: &[History], damping: f64, horizons: &[u16]) -> Vec<f64> {
    let mut errors = Vec::new();
    for history in histories {
        for origin in ORIGINS {
            for horizon in horizons.iter().copied() {
                let target = origin + horizon;
                if PANDEMIC.contains(&target) {
                    continue;
                }
                let Some(actual) = history.get(&target) else {
                    continue;
                };
                let method = Method::Shrunk {
                    rate: 0.0,
                    damping,
                    weight: DEFAULT_SHRINK_WEIGHT,
                    toward: long_run_to(history, origin),
                };
                let point = series::project(
                    &observed_to(history, origin),
                    FiscalYear(target),
                    method,
                    Prior::none(),
                )
                .into_iter()
                .find(|p| p.fiscal_year == FiscalYear(target))
                .expect("the projection reaches the target year")
                .point;
                errors.push((point / actual).ln());
            }
        }
    }
    errors
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn bias(histories: &[History], damping: f64, horizon: u16) -> f64 {
    mean(&log_errors(histories, damping, &[horizon]))
}

/// The damping at which one horizon's bias is zero, or `None` if it never crosses in `0.0..=1.0`.
///
/// Bisection rather than a grid: the quantity is monotone in the damping at every horizon — the
/// test below asserts that — so the crossing is unique where it exists.
fn unbiased_damping(histories: &[History], horizon: u16) -> Option<f64> {
    let (at_zero, at_one) = (bias(histories, 0.0, horizon), bias(histories, 1.0, horizon));
    if at_zero.signum() == at_one.signum() {
        return None;
    }
    let (mut low, mut high) = (0.0f64, 1.0f64);
    for _ in 0..40 {
        let mid = f64::midpoint(low, high);
        if bias(histories, mid, horizon) > 0.0 {
            low = mid;
        } else {
            high = mid;
        }
    }
    Some(f64::midpoint(low, high))
}

/// At one year the damping is not connected to the outcome, so it cannot correct anything there.
///
/// `advance` multiplies by `1 + rate` and only then decays the rate, so the first projected year
/// is identical across the whole range. The equality is exact, not approximate, which is why this
/// asserts on the bits rather than within a tolerance.
#[test]
fn the_one_year_bias_is_the_same_at_every_damping_and_is_negative() {
    let histories = complete_histories();
    let at_zero = bias(&histories, 0.0, 1);
    for damping in [0.25, DEFAULT_DAMPING, 0.5, 0.75, 1.0] {
        assert_eq!(
            bias(&histories, damping, 1).to_bits(),
            at_zero.to_bits(),
            "the one-year bias moved at damping {damping}, so `advance` no longer decays from \
             the second step and this file's argument needs rechecking"
        );
    }
    assert!(
        (at_zero + 0.00492).abs() < 0.0001,
        "the one-year bias should be about -0.00492, is {at_zero:+.6}"
    );
}

/// The damping that removes the bias rises with the horizon, so no one value removes it.
///
/// This is the whole answer to the open question. The crossings are 0.32, 0.64 and 0.79 at three,
/// four and five years, and there is none at one or two — the bias is already negative across the
/// entire range there.
#[test]
fn the_unbiased_damping_rises_with_the_horizon_so_no_scalar_choice_removes_the_bias() {
    let histories = complete_histories();

    assert_eq!(
        unbiased_damping(&histories, 1),
        None,
        "one year never crosses zero"
    );
    assert_eq!(
        unbiased_damping(&histories, 2),
        None,
        "two years is negative across the whole range"
    );

    let crossings: Vec<f64> = (3..=5u16)
        .map(|h| unbiased_damping(&histories, h).expect("three years onward cross zero"))
        .collect();
    for pair in crossings.windows(2) {
        assert!(
            pair[1] > pair[0],
            "the unbiased damping should rise with the horizon: {:.3} then {:.3}",
            pair[0],
            pair[1]
        );
    }
    let expected = [0.321, 0.635, 0.785];
    for (got, want) in crossings.iter().zip(expected) {
        assert!(
            (got - want).abs() < 0.02,
            "expected a crossing near {want:.3}, found {got:.3}"
        );
    }

    // The trade, stated as an assertion rather than as prose: the value that zeroes five years
    // puts a negative bias on two, and what ships puts a positive one on five.
    assert!(
        bias(&histories, crossings[2], 2) < -0.005,
        "zeroing the five-year bias should cost the two-year one"
    );
    assert!(
        bias(&histories, DEFAULT_DAMPING, 5) > 0.02,
        "and what ships leaves the five-year bias positive"
    );
}

/// Pricing the bias moves the optimum by one grid step, because it is 0.6% of the error.
///
/// The fair worry about a criterion blind to sign is that it chose badly. Root mean squared log
/// error prices bias and variance together and lands within 0.05 of where absolute error did.
#[test]
fn scoring_on_squared_error_instead_of_absolute_barely_moves_the_optimum() {
    let histories = complete_histories();
    let horizons = [1u16, 2, 3, 4, 5];
    let grid: Vec<f64> = (0..=20).map(|i| f64::from(i) / 20.0).collect();

    let best = |score: &dyn Fn(&[f64]) -> f64| -> f64 {
        grid.iter()
            .copied()
            .map(|d| (d, score(&log_errors(&histories, d, &horizons))))
            .fold((0.0, f64::MAX), |a, b| if b.1 < a.1 { b } else { a })
            .0
    };
    let absolute = best(&|e: &[f64]| mean(&e.iter().map(|x| x.abs()).collect::<Vec<_>>()));
    let squared = best(&|e: &[f64]| mean(&e.iter().map(|x| x * x).collect::<Vec<_>>()).sqrt());

    assert!(
        (absolute - 0.40).abs() < 0.03 && (squared - 0.45).abs() < 0.03,
        "expected optima near 0.40 and 0.45, found {absolute:.2} and {squared:.2}"
    );
    assert!(
        (squared - absolute).abs() <= 0.10,
        "the two criteria should agree to within a couple of grid steps: {absolute:.2} vs \
         {squared:.2}"
    );

    // Why they agree: at what ships, squared bias is a rounding error against the variance.
    let errors = log_errors(&histories, DEFAULT_DAMPING, &horizons);
    let squared_bias = mean(&errors).powi(2);
    let mse = mean(&errors.iter().map(|x| x * x).collect::<Vec<_>>());
    assert!(
        squared_bias / mse < 0.01,
        "the bias should be under a hundredth of the mean squared error, is {:.4}",
        squared_bias / mse
    );
}

/// On the long horizons alone the optimum is 0.55, and what ships costs 1.4%.
///
/// The strongest case available for moving the constant, kept here so that the decision not to
/// move it rests on a measured number rather than on the absence of one.
#[test]
fn the_long_horizon_optimum_is_higher_than_what_ships_and_the_gap_is_small() {
    let histories = complete_histories();
    let long = [4u16, 5];
    let rmse = |d: f64| {
        mean(
            &log_errors(&histories, d, &long)
                .iter()
                .map(|x| x * x)
                .collect::<Vec<_>>(),
        )
        .sqrt()
    };
    let optimum = (0..=20)
        .map(|i| f64::from(i) / 20.0)
        .map(|d| (d, rmse(d)))
        .fold((0.0, f64::MAX), |a, b| if b.1 < a.1 { b } else { a });

    assert!(
        (optimum.0 - 0.55).abs() < 0.03,
        "the long-horizon optimum should be near 0.55, is {:.2}",
        optimum.0
    );
    let cost = rmse(DEFAULT_DAMPING) / optimum.1 - 1.0;
    assert!(
        (0.005..0.03).contains(&cost),
        "what ships should cost about 1.4% against it, costs {:+.2}%",
        cost * 100.0
    );
}
