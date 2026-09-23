//! How far the panel actually reaches, and what the widening rule does out there.
//!
//! [`the_floor_the_interval_rests_on`] fitted [`series::HORIZON_EXPONENT`] over horizons one to
//! five and said that five years *"is as far as six origins across fifteen surveyed years will
//! reach"*. That sentence is wrong, and it is wrong about the file's own constants: that file's
//! `ORIGINS` carries FY2013, and FY2013 plus eleven is FY2024. The five-year ceiling was a
//! **choice**, not the panel's limit, and it was the choice that left the feed's ten-year horizon
//! extrapolated.
//!
//! # The panel reaches thirteen years, and two more origins were sitting in it
//!
//! `FITTED_ORIGINS` starts at FY2013 because a production caller fits on three observations, and
//! the panel starts at FY2009 — so FY2011 and FY2012 are origins too. Eight origins rather than six,
//! and **thirteen years** rather than five:
//!
//! | | origins | deepest horizon |
//! |---|--:|--:|
//! | `FITTED_ORIGINS`, as the fit used them | 6 | **11** |
//! | [`backtest::ORIGINS`], with FY2011 and FY2012 added | **8** | **13** |
//!
//! # The exponent holds the whole way, across districts
//!
//! Coverage at `horizon^0.65`, over every origin and district the panel admits. The two columns
//! are the same errors; the second removes each origin's own mean first.
//!
//! | horizon | pooled | within origin |
//! |---|--:|--:|
//! | 1 | 64.5% | 66.1% |
//! | 2 | 69.1% | 70.1% |
//! | 3 | 71.3% | **71.3%** |
//! | 4 | 68.7% | 69.6% |
//! | 5 | 67.1% | 69.7% |
//! | 6 | 66.8% | 70.2% |
//! | 7 | 64.7% | 69.5% |
//! | 8 | 63.3% | 67.9% |
//! | 9 | 61.6% | 67.7% |
//! | 10 | 60.3% | 69.6% |
//! | 11 | 59.4% | 69.2% |
//! | 12 | 60.2% | 70.0% |
//! | 13 | **60.1%** | 69.4% |
//! | worst gap from 68.3% | **8.9 pts** | **3.0 pts** |
//!
//! The right-hand column is flat. **Nothing happens to the band's width between five years and
//! thirteen** — the shortfall the left-hand column reports is one number per origin, not a shape
//! in the horizon, and 0.65 is as right at thirteen years as it was at five.
//!
//! # What the left-hand column is measuring is the closure
//!
//! The per-origin means separate on the target year and on nothing else. At five years:
//!
//! | origin | target | mean log error |
//! |---|--:|--:|
//! | FY2011 | FY2016 | +0.0024 |
//! | FY2012 | FY2017 | +0.0073 |
//! | FY2013 | FY2018 | +0.0068 |
//! | FY2015 | FY2020 | +0.0135 |
//! | FY2018 | FY2023 | **+0.0346** |
//! | FY2019 | FY2024 | **+0.0380** |
//!
//! Three origins at the same horizon, forecasting into FY2016 through FY2018, are very nearly
//! unbiased, and the fourth, landing on FY2020, is inside a point and a half. The two landing
//! after the shutdown are more than twice the worst of those four. Restrict every
//! target to FY2020 or earlier and the long horizons come back: coverage runs within **3.8
//! points** of nominal at every horizon from one to **nine**, and the bias at nine years is
//! **+0.027** where the closure-spanning set reaches **+0.073** at thirteen.
//!
//! # Which is a statement about what `sigma` is
//!
//! [`series::Prior::sigma`] is the **cross-sectional** spread of district growth. It was only
//! ever going to price cross-district error, and the table above says it prices it correctly at
//! every horizon this panel can see. A statewide level break is not in it at any exponent, and no
//! value of the exponent puts it there: the one is a spread across districts in a year and the
//! other is a year.
//!
//! So the feed's ten-year horizon does not need cutting, which was the third thing
//! [#391](https://github.com/goedelsoup/ohio-education-funding/issues/391) offered to do. The
//! **width** is measured there now. What was not measured there was the **centre**: mean log
//! error at ten years is +0.056, so the point sits about 5.8% high, and `the_bias_no_single_damping_can_remove`
//! already says no damping takes it out. An interval centred on a biased point inherits the bias
//! whatever its width, which is what [`the-widening-rule`]'s own open note said it did not know.
//! `the_bias_that_belongs_to_the_years` takes it from here: the 5.8% is the mean district's and
//! the total's is 3.2%, the bias is a year effect with no gradient on anything the forecast could
//! see, and it is published rather than corrected.
//!
//! # And the dispersion exponent was never a second opinion
//!
//! `horizon^0.60` for dispersion against `horizon^0.65` for coverage was read as a fact about the
//! error distribution's shape. Part of it is the window. Refit as the range extends:
//!
//! | fitted over | dispersion exponent |
//! |---|--:|
//! | 1-5, pre-closure | 0.573 |
//! | 1-7, pre-closure | 0.600 |
//! | 1-9, pre-closure | 0.623 |
//! | 1-13, every target | 0.650 |
//!
//! It climbs toward the coverage exponent as the range grows. The two targets do differ — the
//! errors are not normal and that has not changed — but **the size of the gap is a property of
//! the five-year window**, so the second question
//! [#391](https://github.com/goedelsoup/ohio-education-funding/issues/391) asked answers
//! negatively: `0.60` implies nothing about the shape beyond five years, because `0.60` is not a
//! measurement of anything beyond five years.
//!
//! # What did not move
//!
//! **The exponent.** Fitted to coverage over the pre-closure range one to nine, the answer is
//! **0.65** on both tie-breaks, which is what ships. The longer range confirms it rather than
//! revising it.
//!
//! **Sigma.** One-year coverage is 64.5% here against the 67.3% the six-origin fit reported,
//! because FY2011 and FY2012 are more volatile origins than the ones they join. One year is the
//! horizon at which the exponent does nothing — `1^k` is 1 for every `k` — so that is a statement
//! about the prior and not about the widening rule. #391 said not to re-open sigma while looking
//! at this, and this file does not.
//!
//! # The thin end
//!
//! At thirteen years there is one origin and 602 districts, which is 602 observations and **one**
//! draw of the year. The within-origin column is the honest one at that depth: it says the spread
//! across districts is right, and it cannot say anything about the year, because it has removed
//! it. Nine years is the deepest horizon with two origins on the pre-closure set and eight is the
//! deepest with two that do not share a target.
//!
//! [`backtest::ORIGINS`]: project::backtest::ORIGINS
//! [`the_floor_the_interval_rests_on`]: ./the_floor_the_interval_rests_on.rs
//! [`the-widening-rule`]: ../../../.yidam/decisions/the-widening-rule.yml

use project::backtest::{
    complete_histories, coverage, deepest_horizon, dispersion_exponent, errors_by_origin, mean,
    observed_to, pooled, prior, profile, within_origin, Held, BEFORE_THE_CLOSURE, DEEPEST_HORIZON,
    NOMINAL_ONE_SIGMA_COVERAGE, ORIGINS, SCORED_DAMPING, SCORED_WEIGHT,
};
use project::series;

/// The origins the exponent was fitted over, pinned rather than imported.
///
/// `the_floor_the_interval_rests_on` owns this list and this file's whole subject is that the
/// list reaches further than that file looked. Importing it would make the reach claim follow
/// whatever that file later decides, which is the one thing it must not do.
const FITTED_ORIGINS: [u16; 6] = [2013, 2015, 2016, 2017, 2018, 2019];

/// The last target year the panel carries, which is the cap every unrestricted claim here uses.
const LATEST_TARGET: u16 = 2024;

/// The five-year ceiling was the file's own choice: `ORIGINS` reaches eleven years, not five.
///
/// This is the first of the three things #391 offered to establish, and it needs no new data at
/// all — FY2013 plus eleven is FY2024, and FY2024 is in the panel. The two extra origins are the
/// only new thing here, and they take the reach to thirteen.
#[test]
fn the_fitted_origins_already_reached_eleven_years_and_two_more_reach_thirteen() {
    assert_eq!(
        deepest_horizon(&FITTED_ORIGINS, LATEST_TARGET),
        11,
        "the six origins the exponent was fitted over reach eleven years, not five"
    );
    assert_eq!(
        deepest_horizon(&ORIGINS, LATEST_TARGET),
        13,
        "adding FY2011 and FY2012 — both of which have three observations behind them — reaches \
         thirteen"
    );
    for origin in [2011u16, 2012] {
        let histories = complete_histories();
        assert_eq!(
            observed_to(&histories[0], origin).len(),
            3,
            "origin {origin} has the three observations a production caller fits on"
        );
    }
}

/// Across districts, the fitted exponent holds within three points at every horizon to thirteen.
///
/// The acceptance test for the whole question. `series::HORIZON_EXPONENT` is imported rather than
/// pinned here, because the claim is about the rule that ships: if it is recalibrated, this file
/// should be re-run against the new value rather than quietly keep grading the old one.
#[test]
fn the_shipped_exponent_holds_the_cross_district_spread_out_to_thirteen_years() {
    let histories = complete_histories();
    let prior = prior();
    let held: Vec<f64> = (1..=13u16)
        .map(|h| {
            let groups = errors_by_origin(&histories, &ORIGINS, h, LATEST_TARGET);
            coverage(&within_origin(&groups), prior.spread(h))
        })
        .collect();
    assert_eq!(held.len(), 13, "every horizon to thirteen is scored");

    let worst = held
        .iter()
        .map(|c| (c - NOMINAL_ONE_SIGMA_COVERAGE).abs())
        .fold(0.0f64, f64::max);
    assert!(
        worst < 0.032,
        "the cross-district coverage should stay within three points of \
         {NOMINAL_ONE_SIGMA_COVERAGE} at every horizon; worst is {:.1} points, from {held:?}",
        worst * 100.0
    );
    assert!(
        (held[12] - 0.694).abs() < 0.02,
        "thirteen years should hold about 69.4%, holds {:.3}",
        held[12]
    );
    assert!(
        (held[4] - held[12]).abs() < 0.02,
        "and it should be flat between five years and thirteen: {:.3} then {:.3}",
        held[4],
        held[12]
    );
}

/// Pooled, it falls to 60.1% — and the whole of the fall is one number per origin.
///
/// The contrast that makes the previous test mean something. A pooled coverage that sinks with
/// the horizon looks exactly like a band widening too slowly, which is what the square root was
/// actually doing before `the-widening-rule`. It is not what is happening here: removing each
/// origin's own mean restores the coverage completely, so the width is right and the level is
/// not.
#[test]
fn pooled_coverage_sinks_to_sixty_percent_and_de_meaning_puts_all_of_it_back() {
    let histories = complete_histories();
    let prior = prior();
    let at = |h: u16| {
        let groups = errors_by_origin(&histories, &ORIGINS, h, LATEST_TARGET);
        (
            coverage(&pooled(&groups), prior.spread(h)),
            coverage(&within_origin(&groups), prior.spread(h)),
        )
    };
    let (five_pooled, _) = at(5);
    let (thirteen_pooled, thirteen_within) = at(13);
    assert!(
        (0.58..0.62).contains(&thirteen_pooled),
        "pooled coverage at thirteen years should be near 60.1%, is {thirteen_pooled:.3}"
    );
    assert!(
        thirteen_pooled < five_pooled - 0.05,
        "and it should be well below the five-year figure: {five_pooled:.3} then \
         {thirteen_pooled:.3}"
    );
    assert!(
        thirteen_within - thirteen_pooled > 0.08,
        "de-meaning by origin should recover the whole shortfall: {thirteen_pooled:.3} pooled \
         against {thirteen_within:.3} within"
    );
}

/// The origins that disagree are the ones whose target lands after the shutdown.
///
/// At a fixed horizon every origin runs the same method over the same districts for the same
/// number of years, so an origin effect is a year effect. Ordered on the target year, the effect
/// is monotone and the step is at FY2023.
#[test]
fn the_origin_effect_is_the_closure_and_nothing_else() {
    let histories = complete_histories();
    let groups = errors_by_origin(&histories, &ORIGINS, 5, LATEST_TARGET);
    let means: Vec<(u16, f64)> = groups
        .iter()
        .map(|(origin, errors)| (origin + 5, mean(errors)))
        .collect();
    assert_eq!(means.len(), 6, "six origins reach five years");

    let before: Vec<f64> = means
        .iter()
        .filter(|(target, _)| *target <= BEFORE_THE_CLOSURE)
        .map(|(_, m)| *m)
        .collect();
    let after: Vec<f64> = means
        .iter()
        .filter(|(target, _)| *target > BEFORE_THE_CLOSURE)
        .map(|(_, m)| *m)
        .collect();
    assert_eq!(before.len(), 4, "four of the six land on FY2020 or earlier");
    let worst_before = before.iter().copied().fold(f64::MIN, f64::max);
    let best_after = after.iter().copied().fold(f64::MAX, f64::min);
    assert!(
        best_after > worst_before * 2.0,
        "the post-closure targets should be several times more biased: before {before:?}, \
         after {after:?}"
    );
    assert!(
        worst_before < 0.015,
        "and the pre-closure ones very nearly unbiased at five years, worst {worst_before:+.4}"
    );
}

/// Restrict every target to FY2020 and the long horizons come back, pooled and all.
///
/// The same errors the previous test decomposes, cut a different way: instead of removing each
/// origin's mean, drop every forecast that crosses the shutdown. Coverage is within 3.8 points at
/// every horizon from one to nine with nothing de-meaned at all, which is the same answer.
#[test]
fn before_the_closure_the_pooled_band_holds_to_nine_years() {
    let histories = complete_histories();
    let prior = prior();
    assert_eq!(
        deepest_horizon(&ORIGINS, BEFORE_THE_CLOSURE),
        9,
        "FY2011 to FY2020 is the deepest pre-closure span the panel carries"
    );
    let held: Vec<f64> = (1..=9u16)
        .map(|h| {
            let groups = errors_by_origin(&histories, &ORIGINS, h, BEFORE_THE_CLOSURE);
            coverage(&pooled(&groups), prior.spread(h))
        })
        .collect();
    let worst = held
        .iter()
        .map(|c| (c - NOMINAL_ONE_SIGMA_COVERAGE).abs())
        .fold(0.0f64, f64::max);
    assert!(
        worst < 0.04,
        "pre-closure coverage should stay within four points to nine years; worst is {:.1} \
         points, from {held:?}",
        worst * 100.0
    );
}

/// The bias is what grows with the horizon, and the closure roughly doubles its rate.
///
/// `a_positive_bias_grows_with_horizon_and_neither_fitting_decision_could_see_it` found +0.023 at
/// five years and stopped there. It keeps going, and it is the whole of what a longer horizon
/// costs: about three tenths of a point a year on forecasts that stay inside the panel's ordinary
/// years, and about seven on ones that cross FY2021.
#[test]
fn the_bias_keeps_growing_and_the_closure_doubles_its_rate() {
    let histories = complete_histories();
    let bias = |h: u16, cap: u16| mean(&pooled(&errors_by_origin(&histories, &ORIGINS, h, cap)));

    let clean_nine = bias(9, BEFORE_THE_CLOSURE);
    assert!(
        (clean_nine - 0.0272).abs() < 0.004,
        "pre-closure the nine-year bias should be about +0.027, is {clean_nine:+.4}"
    );
    let spanning_ten = bias(10, LATEST_TARGET);
    assert!(
        (spanning_ten - 0.0562).abs() < 0.006,
        "at the feed's ten-year horizon it is about +0.056 — the point sits some 5.8% high — \
         is {spanning_ten:+.4}"
    );
    let spanning_thirteen = bias(13, LATEST_TARGET);
    assert!(
        spanning_thirteen > spanning_ten,
        "and it is still growing at thirteen: {spanning_ten:+.4} then {spanning_thirteen:+.4}"
    );
    assert!(
        spanning_thirteen / 13.0 > clean_nine / 9.0 * 1.8,
        "the closure-spanning rate should be near twice the clean one: {:+.5}/yr against \
         {:+.5}/yr",
        spanning_thirteen / 13.0,
        clean_nine / 9.0
    );
}

/// The dispersion exponent is a property of the range it is fitted over, so 0.60 said nothing.
///
/// The second question #391 asked, answered negatively. `horizon^0.60` against the coverage fit's
/// `0.65` was read as a fact about a non-normal error distribution. Part of it is the window: the
/// dispersion exponent climbs toward the coverage exponent as the range extends, and a number
/// measured only inside five years implies nothing about the shape outside them.
#[test]
fn the_dispersion_exponent_climbs_with_the_range_it_is_measured_over() {
    let histories = complete_histories();
    let clean: Vec<f64> = [5u16, 7, 9]
        .iter()
        .map(|d| dispersion_exponent(&histories, &ORIGINS, *d, BEFORE_THE_CLOSURE))
        .collect();
    for pair in clean.windows(2) {
        assert!(
            pair[1] > pair[0],
            "it should rise with every extension of the range: {:.3} then {:.3}",
            pair[0],
            pair[1]
        );
    }
    assert!(
        (clean[0] - 0.573).abs() < 0.02 && (clean[2] - 0.623).abs() < 0.02,
        "over one to five and one to nine it should be about 0.573 and 0.623, is {clean:?}"
    );
    let everything = dispersion_exponent(&histories, &ORIGINS, 13, LATEST_TARGET);
    assert!(
        (everything - 0.650).abs() < 0.02,
        "and over the whole reach about 0.650, is {everything:.3}"
    );
    assert!(
        everything > clean[2],
        "the longest range should give the largest exponent: {:.3} then {everything:.3}",
        clean[2]
    );
}

/// Fitted to coverage over the longer clean range, the answer is still 0.65.
///
/// The exponent does not move. `HORIZON_EXPONENT` is imported deliberately: this is the one claim
/// in the file that is *about the shipped value*, and it should fail if that value changes.
#[test]
fn the_coverage_fit_over_nine_clean_years_returns_the_exponent_that_ships() {
    let histories = complete_histories();
    let prior = prior();
    let errors: Vec<Vec<f64>> = (1..=9u16)
        .map(|h| {
            pooled(&errors_by_origin(
                &histories,
                &ORIGINS,
                h,
                BEFORE_THE_CLOSURE,
            ))
        })
        .collect();

    let gaps = |exponent: f64| -> Vec<f64> {
        errors
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let horizon = u16::try_from(i + 1).expect("nine horizons fit in a u16");
                let width = prior.z * prior.sigma * f64::from(horizon).powf(exponent);
                (coverage(e, width) - NOMINAL_ONE_SIGMA_COVERAGE).abs()
            })
            .collect()
    };

    // A hundredth is the grid the surface was read on when the exponent was chosen, and the third
    // digit was not identified then either.
    let mut best = (f64::MAX, 0.0f64);
    for step in 40..=100u16 {
        let exponent = f64::from(step) / 100.0;
        let average = mean(&gaps(exponent));
        if average < best.0 {
            best = (average, exponent);
        }
    }
    assert!(
        (best.1 - series::HORIZON_EXPONENT).abs() < 1e-9,
        "the longer clean range should return the exponent that ships, {}; it returns {:.2}",
        series::HORIZON_EXPONENT,
        best.1
    );
    assert!(
        best.0 < 0.02,
        "at an average gap under two points, which is {:.1}",
        best.0 * 100.0
    );
}

/// One year loses three points to the two new origins, which is a statement about sigma.
///
/// `1^k` is 1 for every `k`, so the one-year figure cannot be moved by the exponent and is not
/// evidence about it. It moves because FY2011 and FY2012 are more volatile origins than the six
/// they join. #391 said not to re-open sigma while looking at the widening rule, and this records
/// the one place the longer range touches it without touching it.
#[test]
fn one_year_coverage_falls_with_the_older_origins_and_the_exponent_cannot_be_why() {
    let histories = complete_histories();
    let prior = prior();
    let held = |origins: &[u16]| {
        coverage(
            &pooled(&errors_by_origin(&histories, origins, 1, LATEST_TARGET)),
            prior.spread(1),
        )
    };
    let fitted = held(&FITTED_ORIGINS);
    let every = held(&ORIGINS);
    assert!(
        (fitted - 0.673).abs() < 0.01,
        "the six origins hold 67.3% at one year, hold {fitted:.3}"
    );
    assert!(
        (every - 0.645).abs() < 0.01,
        "the eight hold 64.5%, hold {every:.3}"
    );
    assert!(
        (prior.spread(1) - prior.z * prior.sigma).abs() < 1e-12,
        "and the exponent is not in either figure: the band at one year is z times sigma"
    );
}

/// The method the profile was scored at is still the method that ships.
///
/// [`SCORED_DAMPING`] and [`SCORED_WEIGHT`] are written out in [`project::backtest`] rather than
/// read from [`series`], so that every number in this file stays a finding about a method rather
/// than a tautology about whatever the constants currently say. That is only safe while someone
/// checks; this is the check. When it fails the profile has to be re-run and this file's tables
/// re-read, not the pins edited to match.
#[test]
fn the_constants_the_profile_was_scored_at_are_the_ones_that_ship() {
    assert!(
        (SCORED_DAMPING - series::DEFAULT_DAMPING).abs() < f64::EPSILON,
        "the backtest scored damping {SCORED_DAMPING}, the feed ships {}",
        series::DEFAULT_DAMPING
    );
    assert!(
        (SCORED_WEIGHT - series::DEFAULT_SHRINK_WEIGHT).abs() < f64::EPSILON,
        "the backtest scored a shrink weight of {SCORED_WEIGHT}, the feed ships {}",
        series::DEFAULT_SHRINK_WEIGHT
    );
}

/// The profile `/method` draws is this file's table, row for row.
///
/// The two columns above were a table in a doc comment that nothing recomputed, quoted in three
/// places that nothing recomputed either. [`profile`] is the same arithmetic assembled once so a
/// figure can pin its ends; this asserts the assembly did not change any of it, which is the
/// whole of what hoisting owed.
#[test]
fn the_drawn_profile_is_the_table_this_file_scores() {
    let drawn = profile(DEEPEST_HORIZON, LATEST_TARGET);
    assert_eq!(
        drawn.len(),
        usize::from(DEEPEST_HORIZON),
        "one row per horizon to thirteen"
    );

    let histories = complete_histories();
    let prior = prior();
    for Held {
        horizon,
        pooled: drawn_pooled,
        within_origin: drawn_within,
    } in drawn
    {
        let groups = errors_by_origin(&histories, &ORIGINS, horizon, LATEST_TARGET);
        let width = prior.spread(horizon);
        assert!(
            (drawn_pooled - coverage(&pooled(&groups), width)).abs() < f64::EPSILON,
            "horizon {horizon} pooled"
        );
        assert!(
            (drawn_within - coverage(&within_origin(&groups), width)).abs() < f64::EPSILON,
            "horizon {horizon} within origin"
        );
    }
}
