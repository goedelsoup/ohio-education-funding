//! The damping factor the projections ship with, measured against the enrollment history the
//! corpus said it did not have.
//!
//! **The constant moved to 0.30 in `the-fitted-damping`; this file is what moved it, and is
//! written in the tense it was measured in.** `THE_CONVENTION` below holds the 0.85 so that every
//! claim about it stays an assertion.
//!
//! `series::DEFAULT_DAMPING` was 0.85 and said of itself: *"A convention, not an estimate. Three
//! observations per district cannot identify a damping parameter, and there is no Ohio-specific
//! study here to borrow one from; 0.85 is the value damped-trend forecasting commonly defaults
//! to. It is named as a constant so that a future phase with a real enrollment history can
//! replace it with a fitted number and see what moves."* `metric/enrolled-adm` carries the same
//! thing as its one open question.
//!
//! The history arrived when the F-33 panel was extended past FY2022: `V33` fall membership for
//! **602 districts across fourteen years**, which is four to five times the three points that
//! sentence was written against. This is the measurement it asked for.
//!
//! # The convention is on the wrong side of the optimum, in every metric
//!
//! 13,244 out-of-sample forecasts — six fitting origins from FY2013 to FY2019, horizons of one
//! to five years, the two pandemic years dropped as targets because no method predicts a school
//! closure. Each forecast is produced by `series::project` itself rather than by a
//! reimplementation, so what is measured is what ships.
//!
//! | damping | MAE of log error | against the best |
//! |---|---|---|
//! | 0.00 | 0.04042 | +0.3% |
//! | **0.20** | **0.04031** | best |
//! | 0.30 | 0.04033 | +0.1% |
//! | 0.50 | 0.04069 | +0.9% |
//! | **0.85 — the convention** | **0.04441** | **+10.2%** |
//! | 1.00 — undamped | 0.04874 | +20.9% |
//!
//! The optimum sits at **0.2 to 0.3** and is flat across that range, and 0.85 costs about a
//! tenth of the error in all four metrics tried — mean and root-mean-square of log error, mean
//! absolute percentage error, and the enrollment-weighted mean. Ohio district enrollment is
//! close enough to a random walk that a fitted growth rate is worth carrying for about one year
//! and should be nearly gone by the second; 0.85 carries three-quarters of it into the third.
//!
//! # And at 0.85 the method was beaten by doing nothing
//!
//! Carrying the last observation forward unchanged scores 0.04340. That was **better than the
//! convention** on the mean of log error, on root-mean-square, and on mean absolute percentage error;
//! the enrollment-weighted mean is the one metric where 0.85 wins, by 1.3%, against the 9.9% by
//! which damping at 0.30 wins it.
//!
//! This is not an argument for flat. Damped at its optimum the method beats flat by 7.1% and is
//! the right shape — the finding is specifically that **0.85 damps too weakly to collect that
//! gain**, and gives back more than it collects.
//!
//! Two details worth keeping. At a one-year horizon flat beats every damping, because `damping`
//! only decays the rate from the second step onward and the first application of the fitted
//! rate is itself noise. And a least-squares linear trend is far worse than either — 0.05110 on
//! three points, 0.05990 on the full history — which is the one part of the current design this
//! confirms outright.
//!
//! # And what moves, which is the half the constant's own comment asked for
//!
//! "Replace it with a fitted number **and see what moves**". At the feed's FY2036 horizon,
//! moving the damping from 0.85 to 0.30 moves:
//!
//! | | 0.85 — the convention | 0.30 — fitted | difference |
//! |---|--:|--:|--:|
//! | statewide ADM | 1,315,656 | 1,376,953 | **+4.66%** — 61,297 pupils |
//! | realized state aid | \$7,154.7m | \$7,230.1m | **+1.05%** — \$75.4m |
//! | districts on the guarantee | 367 | 320 | **−47** |
//!
//! **The enrollment error is four times the aid error, and the reason is already in the corpus.**
//! `the_forecast_interval_is_narrower_than_the_enrollment_interval_that_drives_it` establishes
//! that the guarantee makes state cost far less sensitive to enrollment than the enrollment
//! forecast is uncertain, because it pays a fixed amount to about half the state. That absorbs
//! most of this too. A projection error of 61,297 pupils reaches the appropriation as \$75.4m.
//!
//! So the constant is wrong and the consequence is bounded, and both halves belong in the same
//! sentence. The number that moves most is not a dollar figure at all: **47 districts** change
//! guarantee status at the horizon, and that is a count the site publishes.
//!
//! # What this did, in the end
//!
//! It did not change the constant. Moving `DEFAULT_DAMPING` moves every projection in the
//! published feed, which is a decision rather than a measurement, and the measurement was what
//! was missing. The numbers were left here for the decision to be made against.
//!
//! It was made: [`the-fitted-damping`](../../../.yidam/decisions/the-fitted-damping.yml) moved it
//! to **0.30**, choosing between the two indistinguishable optima on the enrollment-weighted
//! metric. One test's name stopped being true in the process, and says so.

use dispersion::ohio_panel::{self, PanelRow};
use edfund_core::FiscalYear;
use project::policy::Policy;
use project::series::{self, Method, Observation, Prior, DEFAULT_DAMPING, ONE_SIGMA};
use project::{panel, report};
use std::collections::BTreeMap;

/// Every fiscal year the panel carries, oldest first. FY2014 is absent from the archive.
const PANEL_YEARS: [u16; 15] = [
    2009, 2010, 2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023, 2024,
];

/// The value this shipped with before it was fitted, kept so that the claims about it remain
/// assertions rather than prose in a decision record.
///
/// Every test below that once read `DEFAULT_DAMPING` and meant "the convention" now reads this.
/// The two were the same number until `the-fitted-damping` moved the constant to 0.30, and a file
/// that quietly followed the constant would have turned findings about 0.85 into tautologies
/// about whatever ships.
const THE_CONVENTION: f64 = 0.85;

/// The years a forecast is made from. FY2020 onward is excluded as an origin so that no fit is
/// taken across the closure.
const ORIGINS: [u16; 6] = [2013, 2015, 2016, 2017, 2018, 2019];

/// Targets no method should be scored on.
///
/// FY2021 is the year Ohio's buses stopped and its enrolment fell 54,777 in one step; FY2022 is
/// the rebound. Scoring a forecast on either measures whether it predicted a pandemic. Dropping
/// them is the conservative choice here rather than a flattering one — including them *favours*
/// heavier damping, because heavy damping predicts less movement, so it would move the answer
/// toward the value this file reports as too high.
const PANDEMIC: [u16; 2] = [2021, 2022];

/// One district's enrolment history, keyed by fiscal year.
type History = BTreeMap<u16, f64>;

/// Districts with an observation in every year of the panel.
///
/// Keyed on `LEAID`, which is the only identifier present in every era. A district missing a
/// year is dropped rather than interpolated: the point is to measure a forecast method, and an
/// invented observation would be scored as though it were data.
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

/// The forecast `series::project` makes for `target`, fitted on the observations up to `origin`.
///
/// `window` caps how many of them are used, so that the three-point fit the production callers
/// make can be measured beside a fit on everything.
fn forecast(
    history: &History,
    origin: u16,
    target: u16,
    window: Option<usize>,
    damping: f64,
) -> f64 {
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
    if let Some(window) = window {
        let drop = observed.len().saturating_sub(window);
        observed.drain(..drop);
    }
    // `rate` is refitted from the observations by `series::fit`, so what is passed here is
    // ignored. Only `damping` survives, which is exactly the parameter under test.
    let method = Method::Damped { rate: 0.0, damping };
    series::project(&observed, FiscalYear(target), method, Prior::none())
        .into_iter()
        .find(|p| p.fiscal_year == FiscalYear(target))
        .expect("the projection reaches the target year")
        .point
}

/// The same, carrying the last observation forward unchanged.
fn flat(history: &History, origin: u16) -> f64 {
    *PANEL_YEARS
        .iter()
        .filter(|y| **y <= origin)
        .filter_map(|y| history.get(y))
        .next_back()
        .expect("an origin year is in the panel")
}

/// Mean absolute log error of every out-of-sample forecast, at one damping.
///
/// Log error rather than percentage error so that an over- and under-forecast of the same ratio
/// weigh the same; a district that grows 10% and one that shrinks to 1/1.1 are the same size of
/// mistake and a percentage metric says otherwise.
fn score(histories: &[History], window: Option<usize>, damping: Option<f64>) -> (f64, usize) {
    let mut total = 0.0;
    let mut n = 0usize;
    for history in histories {
        for origin in ORIGINS {
            for horizon in 1..=5u16 {
                let target = origin + horizon;
                if PANDEMIC.contains(&target) {
                    continue;
                }
                let Some(actual) = history.get(&target) else {
                    continue;
                };
                let predicted = match damping {
                    Some(d) => forecast(history, origin, target, window, d),
                    None => flat(history, origin),
                };
                total += (predicted / actual).ln().abs();
                n += 1;
            }
        }
    }
    (total / n as f64, n)
}

/// The panel supplies the history the constant said it lacked.
#[test]
fn the_enrollment_history_is_now_long_enough_to_fit_a_damping_parameter() {
    let histories = complete_histories();
    assert_eq!(
        histories.len(),
        602,
        "602 districts have an observation in all fourteen surveyed years"
    );
    let (_, n) = score(&histories, Some(3), Some(THE_CONVENTION));
    assert_eq!(n, 13_244, "the backtest should hold this many forecasts");
}

/// The fitted optimum is 0.2 to 0.3, the convention was 0.85, and what ships is now the optimum.
///
/// Stated as an interval containing the minimum rather than as a single value, because the
/// objective is flat there — 0.00, 0.20 and 0.30 are within 0.3% of each other — and a test
/// asserting one of them would fail on a rounding change while saying nothing more. That
/// flatness is also why `DEFAULT_DAMPING` is only asserted to be *inside* the interval: 0.20 and
/// 0.30 are both defensible and `the-fitted-damping` chose between them on the
/// enrollment-weighted metric, not on this one.
#[test]
fn the_error_minimising_damping_is_far_below_the_convention_and_is_what_now_ships() {
    let histories = complete_histories();
    let grid: Vec<f64> = (0..=20).map(|i| f64::from(i) * 0.05).collect();
    let mut best = (f64::INFINITY, 0.0);
    for damping in grid {
        let (error, _) = score(&histories, Some(3), Some(damping));
        if error < best.0 {
            best = (error, damping);
        }
    }
    assert!(
        (0.15..=0.35).contains(&best.1),
        "the minimum should sit near 0.2-0.3; it is at {:.2}",
        best.1
    );
    assert!(
        best.1 < THE_CONVENTION,
        "the fitted value {:.2} should be below the convention's {THE_CONVENTION}",
        best.1
    );
    assert!(
        (0.15..=0.35).contains(&DEFAULT_DAMPING),
        "what ships ({DEFAULT_DAMPING}) should now sit inside the fitted interval"
    );

    let (convention, _) = score(&histories, Some(3), Some(THE_CONVENTION));
    let cost = convention / best.0 - 1.0;
    assert!(
        (0.08..0.13).contains(&cost),
        "0.85 should cost about a tenth of the error against the optimum; it costs {:+.1}%",
        cost * 100.0
    );

    let (shipping, _) = score(&histories, Some(3), Some(DEFAULT_DAMPING));
    assert!(
        shipping / best.0 - 1.0 < 0.005,
        "what ships should now be within half a percent of the optimum; it is {:+.2}%",
        (shipping / best.0 - 1.0) * 100.0
    );
}

/// At 0.85 the method was beaten by carrying the last observation forward. At what ships now it
/// is not.
///
/// **This test's name used to be `what_ships_is_worse_than_making_no_forecast_at_all`, and it was
/// true.** `the-fitted-damping` made it false by moving the constant, which is the outcome the
/// finding was for. Both halves are asserted rather than the second replacing the first: that the
/// convention lost to a baseline that does no fitting at all is why the constant moved, and a
/// file that dropped the claim would leave the decision record citing a measurement nothing
/// checks.
#[test]
fn what_used_to_ship_was_worse_than_making_no_forecast_at_all() {
    let histories = complete_histories();
    let (convention, _) = score(&histories, Some(3), Some(THE_CONVENTION));
    let (flat_error, _) = score(&histories, Some(3), None);
    let (shipping, _) = score(&histories, Some(3), Some(DEFAULT_DAMPING));

    assert!(
        convention > flat_error,
        "0.85 scores {convention:.5} against flat's {flat_error:.5}"
    );
    assert!(
        shipping < flat_error,
        "what ships scores {shipping:.5} and should now beat flat's {flat_error:.5}"
    );
    assert!(
        flat_error / shipping - 1.0 > 0.05,
        "the gain damping now collects should exceed 5%; it is {:+.1}%",
        (flat_error / shipping - 1.0) * 100.0
    );
}

/// More history helps more than better damping does, and both help.
///
/// Fitting the rate on everything available rather than on three points lowers the error at
/// every damping tried. It is the larger of the two available improvements, and it is free —
/// the observations are already in the panel.
#[test]
fn a_longer_fitting_window_beats_a_better_damping_factor() {
    let histories = complete_histories();
    let (short_best, _) = score(&histories, Some(3), Some(0.20));
    let (long_best, _) = score(&histories, None, Some(0.30));
    let (short_shipped, _) = score(&histories, Some(3), Some(THE_CONVENTION));

    assert!(
        long_best < short_best,
        "the full history at its optimum ({long_best:.5}) should beat three points at theirs \
         ({short_best:.5})"
    );
    assert!(
        long_best < short_shipped,
        "and should beat what ships ({short_shipped:.5})"
    );
}

/// A least-squares linear trend is worse than any damping, which the current design already
/// assumes and nothing had checked.
///
/// `Method::LinearTrend` exists beside `Damped` and is not what the feed uses. On this panel it
/// is the worst of everything tried, and it gets worse rather than better with more history: an
/// unbounded straight line through a fourteen-year enrolment decline projects a district through
/// zero, which is the failure mode damping was introduced to prevent.
#[test]
fn the_straight_line_is_worse_than_every_damping_and_worse_the_longer_it_is_fitted() {
    let histories = complete_histories();
    let linear = |window: Option<usize>| {
        let mut total = 0.0;
        let mut n = 0usize;
        for history in histories.iter() {
            for origin in ORIGINS {
                for horizon in 1..=5u16 {
                    let target = origin + horizon;
                    if PANDEMIC.contains(&target) {
                        continue;
                    }
                    let Some(actual) = history.get(&target) else {
                        continue;
                    };
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
                    if let Some(window) = window {
                        let drop = observed.len().saturating_sub(window);
                        observed.drain(..drop);
                    }
                    let point = series::project(
                        &observed,
                        FiscalYear(target),
                        Method::LinearTrend {
                            slope: 0.0,
                            intercept: 0.0,
                        },
                        Prior::none(),
                    )
                    .into_iter()
                    .find(|p| p.fiscal_year == FiscalYear(target))
                    .expect("the projection reaches the target year")
                    .point;
                    total += (point.max(1.0) / actual).ln().abs();
                    n += 1;
                }
            }
        }
        total / n as f64
    };

    let (best_damped, _) = score(&histories, Some(3), Some(0.20));
    assert!(
        linear(Some(3)) > best_damped,
        "the straight line should be worse than the best damping"
    );
    assert!(
        linear(None) > linear(Some(3)),
        "and worse still on the full history"
    );
}

/// What the move actually moved — measured before it was made, and still asserted after.
///
/// Through `report::forecast` at the feed's own FY2036 horizon under current law. These were the
/// figures the page printed when the damping was the only thing being changed; the feed has since
/// moved to `Method::Shrunk` and prints different ones. Both damping values are still measured
/// here under `Damped`, which is what makes this a comparison of the damping alone rather than of
/// two changes at once.
///
/// The enrollment difference is large and the aid difference is not, which is the guarantee doing
/// what the corpus already says it does.
///
/// Kept as a live comparison rather than retired once the constant moved: it is the evidence
/// `the-fitted-damping` rests on, and the feed's own regression test against a change to
/// `report::forecast` that would alter the size of the move without anyone noticing.
#[test]
fn moving_the_damping_moves_the_enrollment_four_times_as_much_as_the_aid() {
    let districts = panel::panel();
    let prior = report::enrollment_growth_prior(&districts, ONE_SIGMA);
    let at = |damping: f64| {
        report::forecast(
            &districts,
            &Policy::current_law(),
            FiscalYear(2036),
            Method::Damped { rate: 0.0, damping },
            prior,
        )
    };
    let shipped = at(THE_CONVENTION);
    let fitted = at(DEFAULT_DAMPING);

    let adm_move = fitted.adm / shipped.adm - 1.0;
    let aid_move = fitted.realized_aid / shipped.realized_aid - 1.0;
    assert!(
        (0.04..0.055).contains(&adm_move),
        "ADM should move about 4.7%; it moves {:+.2}%",
        adm_move * 100.0
    );
    assert!(
        (0.008..0.014).contains(&aid_move),
        "aid should move about 1.05%; it moves {:+.2}%",
        aid_move * 100.0
    );
    assert!(
        adm_move > aid_move * 3.0,
        "the guarantee should absorb most of it: ADM {:+.2}% against aid {:+.2}%",
        adm_move * 100.0,
        aid_move * 100.0
    );

    // The count that moves most, and the one the site prints.
    assert_eq!(shipped.on_guarantee, 367);
    assert_eq!(fitted.on_guarantee, 320);
}
