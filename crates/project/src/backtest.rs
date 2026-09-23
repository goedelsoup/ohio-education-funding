//! Scoring the projection against the years it has already lived through.
//!
//! # What this measures
//!
//! [`series::project`] publishes a point and a band. The band is `z * sigma * horizon^k`, where
//! `sigma` is the **cross-sectional** spread of district growth rates and `k` is
//! [`series::HORIZON_EXPONENT`]. A band that claims one standard deviation claims to hold
//! [`NOMINAL_ONE_SIGMA_COVERAGE`] of what actually happens, and whether it does is not a question
//! about the rule — it is a question about the panel, answered by running the shipped method from
//! every origin the panel supports and counting.
//!
//! # Why it is library code rather than a test
//!
//! It was a test, and the numbers it produced were quoted in three places — the corpus node for
//! the guarantee scenario, the widening-rule decision, and `/method`'s own prose — with nothing
//! recomputing them. A figure quoted from a test is a figure that moves when the method moves and
//! says nothing about it. The profile is computed here so that `crates/figures` can pin its ends
//! and `crates/series.json` can carry the whole of it, and
//! `tests/the_horizons_the_backtest_stopped_short_of.rs` keeps every claim it made about the
//! shape, now asserted against this module rather than against a private copy of it.
//!
//! # What is pinned here and why
//!
//! [`SCORED_DAMPING`] and [`SCORED_WEIGHT`] are written out rather than imported from
//! [`series`]. A backtest that followed the shipped constants would turn a finding about *the
//! method as it was scored* into a tautology about whatever ships, which is the mistake
//! `the_damping_nobody_fitted` records under `THE_CONVENTION`. The test asserts they still agree
//! with the shipped values; when they stop agreeing the profile has to be re-run rather than
//! re-labelled.
//!
//! The prior is **not** pinned: [`profile`] takes it from the same path the feed takes, because
//! the width being scored is the width the feed draws.

use core::cmp::Ordering;
use std::collections::BTreeMap;

use dispersion::ohio_panel::{self, PanelRow};
use edfund_core::FiscalYear;

use crate::series::{self, Method, Observation, Prior, ONE_SIGMA};
use crate::{panel, report};

/// Every fiscal year the F-33 panel carries, oldest first.
///
/// FY2014 is absent from the Census archive, which is why this is a list rather than a range: a
/// horizon is a difference between two years that both exist, and treating the gap as a year
/// would score a forecast against nothing.
pub const PANEL_YEARS: [u16; 15] = [
    2009, 2010, 2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023, 2024,
];

/// Every origin the panel supports, oldest first.
///
/// A production caller fits on three observations, and the panel starts at FY2009, so FY2011 is
/// the first year an origin can be assembled for. The exponent was fitted over six of these —
/// FY2013 onward — and that was a choice rather than the panel's limit: the two older origins
/// take the deepest scored horizon from eleven years to [`DEEPEST_HORIZON`].
pub const ORIGINS: [u16; 8] = [2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019];

/// Targets no method is scored on: the year enrolment fell 54,777 in one step, and the rebound.
pub const PANDEMIC: [u16; 2] = [2021, 2022];

/// The last target year a forecast can reach without spanning the school closures.
///
/// FY2020's count is taken in October 2019. [`PANDEMIC`] keeps a forecast from *landing* on the
/// shutdown; this keeps one from crossing it, which is a different exclusion and the one the long
/// horizons turn on.
pub const BEFORE_THE_CLOSURE: u16 = 2020;

/// The deepest horizon [`ORIGINS`] scores against the whole panel — FY2011 to FY2024.
pub const DEEPEST_HORIZON: u16 = 13;

/// The damping the profile was scored at. See the module docs for why it is written and not read.
pub const SCORED_DAMPING: f64 = 0.30;

/// The shrink weight it was scored at, on the same rule.
pub const SCORED_WEIGHT: f64 = 0.30;

/// The fraction of a normal distribution inside ±1σ, which is what the band claims to hold.
///
/// A definition rather than a measurement: there is no crate that computes it and nothing for a
/// figure to pin. It is the line every coverage number on this page is read against.
pub const NOMINAL_ONE_SIGMA_COVERAGE: f64 = 0.683;

/// One district's enrolment history, keyed by fiscal year.
pub type History = BTreeMap<u16, f64>;

/// Districts with an observation in every year of [`PANEL_YEARS`], keyed on `LEAID`.
///
/// Complete histories only, and the completeness is what makes a horizon comparable to its
/// neighbours: a population that thins as the horizon deepens would report a coverage that moved
/// because the districts moved.
#[must_use]
pub fn complete_histories() -> Vec<History> {
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

/// The long-run rate as of `origin` — computed from the panel's start to the origin only.
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
///
/// Public because "a production caller fits on three observations" is the sentence the origin
/// list rests on, and `the_horizons_the_backtest_stopped_short_of` asserts it.
#[must_use]
pub fn observed_to(history: &History, origin: u16) -> Vec<Observation> {
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

/// The shipped method, fitted as of `origin` so that no rate has seen its own target.
fn method(history: &History, origin: u16) -> Method {
    Method::Shrunk {
        rate: 0.0,
        damping: SCORED_DAMPING,
        weight: SCORED_WEIGHT,
        toward: long_run_to(history, origin),
    }
}

/// Whether a forecast from `origin` at `horizon` is scored at all, and against which year.
#[must_use]
pub fn scored(origin: u16, horizon: u16, latest_target: u16) -> Option<u16> {
    let target = origin + horizon;
    (target <= latest_target && PANEL_YEARS.contains(&target) && !PANDEMIC.contains(&target))
        .then_some(target)
}

/// The deepest horizon a set of origins scores at least one forecast at.
///
/// # Panics
///
/// If no horizon in the panel's range is scored at all, which would mean the origins are outside
/// it.
#[must_use]
pub fn deepest_horizon(origins: &[u16], latest_target: u16) -> u16 {
    (1..=20u16)
        .filter(|h| {
            origins
                .iter()
                .any(|o| scored(*o, *h, latest_target).is_some())
        })
        .max()
        .expect("at least one horizon is scored")
}

/// Log forecast errors at one horizon, grouped by the origin that produced them.
///
/// Grouped rather than pooled because the grouping is the finding: a pooled standard deviation
/// cannot tell a band that is the wrong width from a set of origins that disagree about the
/// level, and at these horizons it is always the second.
#[must_use]
pub fn errors_by_origin(
    histories: &[History],
    origins: &[u16],
    horizon: u16,
    latest_target: u16,
) -> Vec<(u16, Vec<f64>)> {
    let mut out = Vec::new();
    for origin in origins {
        let Some(target) = scored(*origin, horizon, latest_target) else {
            continue;
        };
        let errors: Vec<f64> = histories
            .iter()
            .filter_map(|history| {
                let actual = history.get(&target)?;
                let point = series::project(
                    &observed_to(history, *origin),
                    FiscalYear(target),
                    method(history, *origin),
                    Prior::none(),
                )
                .into_iter()
                .find(|p| p.fiscal_year == FiscalYear(target))?
                .point;
                Some((point / actual).ln())
            })
            .collect();
        if errors.len() > 1 {
            out.push((*origin, errors));
        }
    }
    out
}

/// Every error at one horizon, origins pooled.
#[must_use]
pub fn pooled(groups: &[(u16, Vec<f64>)]) -> Vec<f64> {
    groups.iter().flat_map(|(_, e)| e.iter().copied()).collect()
}

/// The same errors with each origin's own mean removed — the cross-district component alone.
#[must_use]
pub fn within_origin(groups: &[(u16, Vec<f64>)]) -> Vec<f64> {
    groups
        .iter()
        .flat_map(|(_, errors)| {
            let m = mean(errors);
            errors.iter().map(move |e| e - m)
        })
        .collect()
}

/// The arithmetic mean.
///
/// # Panics
///
/// Never; an empty slice gives `NaN`, which every caller here has already excluded by requiring
/// more than one error per origin.
#[must_use]
pub fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

/// Sample standard deviation.
#[must_use]
pub fn stdev(values: &[f64]) -> f64 {
    let m = mean(values);
    let n = values.len() as f64;
    (values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
}

/// The share of errors a band of `width` holds.
#[must_use]
pub fn coverage(errors: &[f64], width: f64) -> f64 {
    errors.iter().filter(|e| e.abs() <= width).count() as f64 / errors.len() as f64
}

/// The production prior, taken from the path the feed takes.
#[must_use]
pub fn prior() -> Prior {
    report::enrollment_growth_prior(&panel::panel(), ONE_SIGMA)
}

/// The exponent that best fits the dispersion over `1..=deepest`, by least squares on the logs.
#[must_use]
pub fn dispersion_exponent(
    histories: &[History],
    origins: &[u16],
    deepest: u16,
    latest_target: u16,
) -> f64 {
    let points: Vec<(f64, f64)> = (1..=deepest)
        .filter_map(|h| {
            let errors = pooled(&errors_by_origin(histories, origins, h, latest_target));
            (errors.len() > 1).then(|| (f64::from(h).ln(), stdev(&errors).ln()))
        })
        .collect();
    let (xs, ys): (Vec<f64>, Vec<f64>) = points.iter().copied().unzip();
    let (mx, my) = (mean(&xs), mean(&ys));
    xs.iter()
        .zip(&ys)
        .map(|(x, y)| (x - mx) * (y - my))
        .sum::<f64>()
        / xs.iter().map(|x| (x - mx).powi(2)).sum::<f64>()
}

/// What the band held at one horizon, both ways of asking.
///
/// The two shares are the **same errors**. [`Held::within_origin`] removes each origin's own mean
/// first, so it is the cross-district spread alone and [`Held::pooled`] is that plus whatever the
/// years did. A reader who has only the first cannot tell a band of the wrong width from a set of
/// origins that disagree about the level; a reader who has both can, because the difference
/// between them is exactly the year effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Held {
    /// Years ahead, one-based.
    pub horizon: u16,
    /// The share of every scored error the band holds, origins pooled.
    pub pooled: f64,
    /// The same, with each origin's mean removed.
    pub within_origin: f64,
}

/// Coverage at every horizon the panel reaches, both ways, over every origin and district.
///
/// The profile `/method` draws and `crates/series.json` carries. One row per horizon from one to
/// `deepest`, scored against `latest_target`; pass [`DEEPEST_HORIZON`] and the panel's last year
/// for the published one.
///
/// # Panics
///
/// If a horizon in the range scores no errors at all, which would mean `deepest` reaches past
/// what the origins support — [`deepest_horizon`] is the function that answers that.
#[must_use]
pub fn profile(deepest: u16, latest_target: u16) -> Vec<Held> {
    let histories = complete_histories();
    let prior = prior();
    (1..=deepest)
        .map(|horizon| {
            let groups = errors_by_origin(&histories, &ORIGINS, horizon, latest_target);
            assert!(
                !groups.is_empty(),
                "horizon {horizon} scores no forecast against a target no later than \
                 FY{latest_target}"
            );
            let width = prior.spread(horizon);
            Held {
                horizon,
                pooled: coverage(&pooled(&groups), width),
                within_origin: coverage(&within_origin(&groups), width),
            }
        })
        .collect()
}

/// The horizon whose coverage sits furthest from [`NOMINAL_ONE_SIGMA_COVERAGE`], and by how much.
///
/// The number a reader takes off the flat line. "Within three points the whole way" is the claim
/// the within-origin row makes, and the claim is a *maximum* over the horizons rather than a
/// value at one of them — so it is computed rather than read off, and pinned as a figure of its
/// own beside the two ends.
///
/// Ties go to the shallower horizon, which is the one a reader would quote.
///
/// # Panics
///
/// If the profile is empty.
#[must_use]
pub fn worst_gap(rows: &[Held], of: fn(&Held) -> f64) -> (u16, f64) {
    rows.iter()
        .map(|row| (row.horizon, (of(row) - NOMINAL_ONE_SIGMA_COVERAGE).abs()))
        .max_by(|a, b| match a.1.total_cmp(&b.1) {
            Ordering::Equal => b.0.cmp(&a.0),
            other => other,
        })
        .expect("a profile with no horizons in it")
}
