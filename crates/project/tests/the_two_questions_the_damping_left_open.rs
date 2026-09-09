//! The two questions [`the-fitted-damping`] recorded as still open, answered.
//!
//! It moved `DEFAULT_DAMPING` from 0.85 to 0.30 and named two things it had not done: fit the
//! damping per district rather than once for the feed, and lengthen the fitting window from the
//! three points the department's model carries to the fourteen years the F-33 panel holds. One
//! turns out to be noise. The other is worth about 4.6%, and rests on a bridge this file can
//! state but not test.
//!
//! # A per-district damping is overfitting, and the in-sample gain is how you would miss it
//!
//! Fitting a damping to each district on its own history looks like a large improvement and is
//! not one:
//!
//! | | mean absolute log error |
//! |---|--:|
//! | in sample — fitted and scored on the same origins | **−15.3%** against the global 0.30 |
//! | out of sample — fitted early, scored later | **+3.6%** *worse* than the global 0.30 |
//!
//! The distribution says why. Of 602 districts, **316 fit 0.00 and 101 fit 1.00** — more than two
//! thirds land on a boundary of the grid. Fifteen forecasts per district cannot identify a decay
//! parameter, so the optimum is wherever the noise pushed it, and carrying that into the next
//! period is worse than carrying nothing. This is the same finding the constant's original
//! comment made about three observations, one level down.
//!
//! So the feed keeps one damping, and it keeps it for a reason rather than for want of trying.
//!
//! # The fitting window is worth more than the damping was, and blending beats replacing
//!
//! A rate fitted over three points is an endpoint ratio across two years, which is the noisiest
//! estimator available. Weighting it against the same district's long-history rate:
//!
//! | weight on the three-point rate | mean absolute log error |
//! |---|--:|
//! | 1.0 — what the projections do | 0.04033 |
//! | 0.5 | 0.03874 |
//! | **0.3** | **0.03855** |
//! | 0.2 | 0.03857 |
//! | 0.0 — long history alone | 0.03881 |
//!
//! **The blend beats both ends.** Discarding the recent rate entirely is worse than keeping about
//! a third of it, so the three-point window carries real information about where a district is
//! now — just not enough to stand alone. At 0.3 the improvement over what ships is **4.6%**,
//! which is larger than the 2.4% that moving the damping from 0.85 to 0.30 bought.
//!
//! Re-tuning the damping on top of the blend moves the optimum to 0.40 and buys a further
//! **0.18%**. That is not worth a second change to a published constant, and it is the expected
//! direction: a less noisy rate can be carried a little further before it stops helping.
//!
//! # What stood between this and the feed, and how it was crossed
//!
//! The projection runs on **enrolled ADM**, which the department publishes for three years. The
//! long rate here is **`V33` fall membership**, which the Census publishes for fifteen. They are
//! not the same count, and the fixture join is only sound because they turn out to be nearly the
//! same population: at FY2024, across 608 districts, their levels correlate at **0.9997**.
//!
//! **Their growth rates do not.** Over the same two years and the same districts, the ADM rate and
//! the F-33 rate correlate at **0.287**. Two counts of nearly the same children disagree about
//! which way most districts moved.
//!
//! That is the strongest evidence here that the three-point window is noise-dominated — if a
//! two-year rate were mostly signal, two measures of the same children would agree on it. But it
//! is also exactly why the blend could not be dropped into the feed as a mechanical change: it
//! means shrinking an ADM rate toward an F-33 rate, and the assumption that the two series share
//! a long-run trend while differing in short-run noise is one this repository cannot test,
//! because only three years of ADM exist.
//!
//! It was made a decision rather than an improvement, and taken:
//! [`the-shrunk-rate`](../../../.yidam/decisions/the-shrunk-rate.yml) owns the assumption and
//! `Method::Shrunk` is what the feed now uses, at this weight. The numbers below are the evidence
//! it rests on, so they stay measured here rather than moving into the record.
//!
//! [`the-fitted-damping`]: ../../../.yidam/decisions/the-fitted-damping.yml

mod common;

use dispersion::ohio_panel::{self, PanelRow};
use project::panel;
use std::collections::BTreeMap;

/// Every fiscal year the panel carries, oldest first. FY2014 is absent from the archive.
const PANEL_YEARS: [u16; 15] = [
    2009, 2010, 2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023, 2024,
];

/// Origins a district's own damping is fitted on.
const FIT_ORIGINS: [u16; 3] = [2013, 2015, 2016];

/// Origins it is then scored on, which it has never seen.
const EVAL_ORIGINS: [u16; 3] = [2017, 2018, 2019];

/// Every origin, for the rate-window comparison.
const ALL_ORIGINS: [u16; 6] = [2013, 2015, 2016, 2017, 2018, 2019];

/// Targets no method is scored on. See the sibling file for why.
const PANDEMIC: [u16; 2] = [2021, 2022];

/// What the feed's damping is, restated rather than imported so that this file's numbers do not
/// silently follow a change to it — every figure above was measured at 0.30.
const SHIPPING_DAMPING: f64 = 0.30;

/// One district's enrolment history.
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

/// The endpoint compound rate `series::fit` computes, reproduced here so that a window other than
/// the shipped one can be asked for.
fn compound_rate(from: f64, to: f64, years: f64) -> f64 {
    if from <= 0.0 || to <= 0.0 || years <= 0.0 {
        return 0.0;
    }
    (to / from).powf(1.0 / years) - 1.0
}

/// `series::advance` for a damped method.
fn damped(base: f64, rate: f64, damping: f64, horizon: u16) -> f64 {
    let mut value = base;
    let mut step = rate;
    for _ in 0..horizon {
        value *= 1.0 + step;
        step *= damping;
    }
    value
}

/// Mean absolute log error over `origins`, with `short_weight` of the three-point rate and the
/// rest from the district's whole history.
fn score(
    histories: &[History],
    origins: &[u16],
    short_weight: f64,
    damping: f64,
    per_district: bool,
) -> f64 {
    let mut total = 0.0;
    let mut n = 0usize;
    for history in histories {
        let damping = if per_district {
            fitted_damping(history)
        } else {
            damping
        };
        for origin in origins {
            let observed: Vec<u16> = PANEL_YEARS
                .iter()
                .copied()
                .filter(|y| y <= origin)
                .collect();
            let short = &observed[observed.len() - 3..];
            let base = history[observed.last().expect("an origin is in the panel")];
            let short_rate = compound_rate(
                history[&short[0]],
                history[short.last().expect("three points")],
                f64::from(short.last().expect("three points") - short[0]),
            );
            let long_rate = compound_rate(
                history[&observed[0]],
                base,
                f64::from(observed.last().expect("non-empty") - observed[0]),
            );
            let rate = short_weight * short_rate + (1.0 - short_weight) * long_rate;
            for horizon in 1..=5u16 {
                let target = origin + horizon;
                if PANDEMIC.contains(&target) {
                    continue;
                }
                let Some(actual) = history.get(&target) else {
                    continue;
                };
                total += (damped(base, rate, damping, horizon) / actual).ln().abs();
                n += 1;
            }
        }
    }
    total / n as f64
}

/// The damping that minimises one district's error on [`FIT_ORIGINS`] alone.
fn fitted_damping(history: &History) -> f64 {
    let one = std::slice::from_ref(history);
    let mut best = (f64::INFINITY, 0.0);
    for step in 0..=20 {
        let damping = f64::from(step) * 0.05;
        let error = score(&one[..1], &FIT_ORIGINS, 1.0, damping, false);
        if error < best.0 {
            best = (error, damping);
        }
    }
    best.1
}

/// A per-district damping looks like a large gain in sample and is a loss out of it.
///
/// The comparison the question needed and could not have been answered without: fitted on
/// FY2013-FY2016 origins and scored on FY2017-FY2019, which the fit has never seen.
#[test]
fn a_damping_fitted_per_district_is_worse_out_of_sample_than_the_one_the_feed_ships() {
    let histories = complete_histories();
    let in_sample_global = score(&histories, &FIT_ORIGINS, 1.0, SHIPPING_DAMPING, false);
    let in_sample_per = score(&histories, &FIT_ORIGINS, 1.0, 0.0, true);
    let out_global = score(&histories, &EVAL_ORIGINS, 1.0, SHIPPING_DAMPING, false);
    let out_per = score(&histories, &EVAL_ORIGINS, 1.0, 0.0, true);

    assert!(
        in_sample_per < in_sample_global * 0.90,
        "in sample the per-district fit should look much better: {in_sample_per:.5} against \
         {in_sample_global:.5}"
    );
    assert!(
        out_per > out_global,
        "out of sample it should be worse: {out_per:.5} against {out_global:.5}"
    );
    assert!(
        (0.02..0.06).contains(&(out_per / out_global - 1.0)),
        "and worse by a few percent; it is {:+.1}%",
        (out_per / out_global - 1.0) * 100.0
    );
}

/// Why it overfits, in one number: the fitted damping piles up on the boundaries.
///
/// More than two thirds of districts land on 0.00 or 1.00 — the ends of the grid — which is what
/// an unidentified parameter looks like when it is fitted anyway. Fifteen forecasts cannot locate
/// a decay rate, so the optimum goes wherever the noise pushed it.
#[test]
fn two_thirds_of_the_per_district_fits_land_on_a_boundary_of_the_grid() {
    let histories = complete_histories();
    let mut boundary = 0usize;
    let mut zero = 0usize;
    for history in &histories {
        let damping = fitted_damping(history);
        if damping <= f64::EPSILON || damping >= 1.0 - f64::EPSILON {
            boundary += 1;
        }
        if damping <= f64::EPSILON {
            zero += 1;
        }
    }
    assert!(
        boundary * 3 > histories.len() * 2,
        "{boundary} of {} districts fit a boundary value",
        histories.len()
    );
    assert!(
        zero > histories.len() / 2,
        "{zero} of {} fit exactly zero",
        histories.len()
    );
}

/// Blending the three-point rate toward the district's long-history rate beats both ends.
///
/// The finding of the second open question, and the shape of it matters: **discarding the recent
/// rate is worse than keeping about a third of it**. Three points carry real information about
/// where a district is now, and not enough of it to stand alone.
#[test]
fn a_blended_rate_beats_both_the_three_point_window_and_the_long_one() {
    let histories = complete_histories();
    let shipping = score(&histories, &ALL_ORIGINS, 1.0, SHIPPING_DAMPING, false);
    let blended = score(&histories, &ALL_ORIGINS, 0.3, SHIPPING_DAMPING, false);
    let long_only = score(&histories, &ALL_ORIGINS, 0.0, SHIPPING_DAMPING, false);

    assert!(
        blended < shipping && blended < long_only,
        "the blend ({blended:.5}) should beat three points ({shipping:.5}) and the long history \
         alone ({long_only:.5})"
    );
    let gain = shipping / blended - 1.0;
    assert!(
        (0.04..0.055).contains(&gain),
        "the blend should be worth about 4.6%; it is {:+.1}%",
        gain * 100.0
    );
    // Larger than the change that has already been made to the same projection.
    assert!(
        gain > 0.02,
        "and larger than the 2.4% moving the damping bought"
    );
}

/// Re-tuning the damping on top of the blend is not worth a second change to a published constant.
///
/// The optimum moves from 0.30 to about 0.40, which is the expected direction — a less noisy rate
/// survives a little further — and it buys under a fifth of a percent.
#[test]
fn the_blend_does_not_justify_moving_the_damping_a_second_time() {
    let histories = complete_histories();
    let at_shipping = score(&histories, &ALL_ORIGINS, 0.3, SHIPPING_DAMPING, false);
    let at_retuned = score(&histories, &ALL_ORIGINS, 0.3, 0.40, false);
    assert!(
        at_retuned < at_shipping,
        "0.40 should be the better of the two"
    );
    assert!(
        at_shipping / at_retuned - 1.0 < 0.005,
        "and by under half a percent; it is {:+.2}%",
        (at_shipping / at_retuned - 1.0) * 100.0
    );
}

/// The bridge the blend would have to cross, measured on both sides.
///
/// The projection runs on enrolled ADM and the long rate is `V33` fall membership. The join is
/// sound — the two count nearly the same children — and the growth rates still disagree, which is
/// the whole problem and also the best evidence that a two-year rate is mostly noise.
///
/// Both sides are read through the readers that already exist: `project::panel` for the
/// department's model and `dispersion::ohio_panel` for the survey. An earlier draft of this test
/// parsed the model's CSV itself, which would have been a third reader of a fixture two crates
/// already read, and passed the file's own header line to `csv::rows` as the expected one — an
/// assertion that cannot fail.
#[test]
fn the_two_enrollment_counts_agree_on_levels_and_not_on_rates() {
    // Two passes, though one would work today. `PanelRow::irn` is documented as carrying Ohio's
    // identifier "where the FY2022-23 directory still carries the agency", which reads as though
    // only those years hold one; in fact the directory's IRN is backfilled onto every year of a
    // surviving agency, so the earliest row already names it. Resolving the map first does not
    // depend on that, and the assertion below is what would notice if it stopped being true.
    let survey = ohio_panel::panel();
    let mut irn_of: BTreeMap<&str, &str> = BTreeMap::new();
    for row in survey.iter().filter(|r: &&PanelRow| r.comparable) {
        if !row.irn.is_empty() {
            irn_of.insert(row.leaid.as_str(), row.irn.as_str());
        }
    }
    let mut by_irn: BTreeMap<&str, History> = BTreeMap::new();
    for row in survey.iter().filter(|r: &&PanelRow| r.comparable) {
        if let Some(irn) = irn_of.get(row.leaid.as_str()) {
            if row.enrollment > 0.0 {
                by_irn
                    .entry(irn)
                    .or_default()
                    .insert(row.fiscal_year, row.enrollment);
            }
        }
    }
    let reaching_2009 = by_irn.values().filter(|h| h.contains_key(&2009)).count();
    assert!(
        reaching_2009 * 100 > by_irn.len() * 99,
        "the join should reach FY2009 for all but a handful of agencies; it reaches {reaching_2009} \
         of {}",
        by_irn.len()
    );

    let mut levels = Vec::new();
    let mut rates = Vec::new();
    for record in panel::panel() {
        let adm = record.adm_observations();
        let (Some(first), Some(last)) = (adm.first(), adm.last()) else {
            continue;
        };
        let Some(history) = by_irn.get(record.irn.as_str()) else {
            continue;
        };
        let (Some(v24), Some(v22)) = (history.get(&2024), history.get(&2022)) else {
            continue;
        };
        if first.value <= 0.0 || last.value <= 0.0 {
            continue;
        }
        let span = f64::from(last.fiscal_year.0 - first.fiscal_year.0);
        levels.push((first.value, *v24));
        rates.push((
            compound_rate(first.value, last.value, span),
            compound_rate(*v22, *v24, 2.0),
        ));
    }

    assert!(levels.len() > 590, "{} districts joined", levels.len());
    let on_levels = common::correlation(&levels);
    assert!(
        on_levels > 0.999,
        "the two counts should agree on levels; they correlate at {on_levels:.4}"
    );
    let on_rates = common::correlation(&rates);
    assert!(
        (0.2..0.4).contains(&on_rates),
        "and disagree on two-year rates; they correlate at {on_rates:.3}"
    );
}
