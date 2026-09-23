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

/// One scored forecast, kept as its two levels rather than as the log error alone.
///
/// The two biases the feed publishes come from the same forecasts and differ only in where the
/// summing happens: the mean district's is the mean of the logs of the ratios, and the total's is
/// the log of the ratio of the sums. A forecast already collapsed to its log error can produce
/// the first and not the second, which is why this carries both levels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scored {
    /// What the shipped method projected for the target year.
    pub point: f64,
    /// What the panel recorded there.
    pub actual: f64,
}

impl Scored {
    /// The log forecast error, positive where the projection ran high.
    #[must_use]
    pub fn log_error(self) -> f64 {
        (self.point / self.actual).ln()
    }
}

/// Every scored forecast at one horizon, grouped by the origin that produced it.
///
/// Grouped rather than pooled because the grouping is the finding: a pooled standard deviation
/// cannot tell a band that is the wrong width from a set of origins that disagree about the
/// level, and at these horizons it is always the second.
///
/// An origin contributing a single forecast is dropped, so every group below carries a spread.
#[must_use]
pub fn scored_at(
    histories: &[History],
    origins: &[u16],
    horizon: u16,
    latest_target: u16,
) -> Vec<(u16, Vec<Scored>)> {
    let mut out = Vec::new();
    for origin in origins {
        let Some(target) = scored(*origin, horizon, latest_target) else {
            continue;
        };
        let forecasts: Vec<Scored> = histories
            .iter()
            .filter_map(|history| {
                let actual = *history.get(&target)?;
                let point = series::project(
                    &observed_to(history, *origin),
                    FiscalYear(target),
                    method(history, *origin),
                    Prior::none(),
                )
                .into_iter()
                .find(|p| p.fiscal_year == FiscalYear(target))?
                .point;
                Some(Scored { point, actual })
            })
            .collect();
        if forecasts.len() > 1 {
            out.push((*origin, forecasts));
        }
    }
    out
}

/// Log forecast errors at one horizon, grouped by the origin that produced them.
///
/// [`scored_at`] with the levels discarded. One projection path serves both, so a band scored
/// against one set of forecasts and a bias measured against another cannot happen.
#[must_use]
pub fn errors_by_origin(
    histories: &[History],
    origins: &[u16],
    horizon: u16,
    latest_target: u16,
) -> Vec<(u16, Vec<f64>)> {
    scored_at(histories, origins, horizon, latest_target)
        .into_iter()
        .map(|(origin, forecasts)| {
            (
                origin,
                forecasts.into_iter().map(Scored::log_error).collect(),
            )
        })
        .collect()
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

/// The bias carried by each of the two quantities the feed publishes from one set of forecasts.
///
/// The feed publishes a statewide total and six hundred district figures from the same
/// projections, and a reader who has only one of these numbers cannot tell which of the two the
/// correction they are contemplating would serve. They are not two estimates of one thing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bias {
    /// The mean district's: the mean log error, which weights every district alike.
    pub mean_district: f64,
    /// The total's: the log of summed points over summed actuals, which weights by size.
    pub total: f64,
}

/// What one horizon's forecasts were biased by, before the closure and across it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drift {
    /// Years ahead, one-based.
    pub horizon: u16,
    /// Restricted to targets of [`BEFORE_THE_CLOSURE`] or earlier.
    ///
    /// `None` where no origin reaches this horizon without crossing the shutdown, which is what
    /// the deeper horizons are: the panel cannot say what this method does at ten years in a
    /// decade that did not contain a school closure, and an absence is the honest answer rather
    /// than the figure that crosses it wearing the other label.
    pub before_the_closure: Option<Bias>,
    /// Every scored forecast at this horizon, the closure included.
    pub across_it: Bias,
}

/// The mean district's bias over a set of forecasts.
///
/// # Panics
///
/// Never; an empty slice gives `NaN`, and [`bias_profile`] does not produce one.
#[must_use]
pub fn mean_district_bias(forecasts: &[Scored]) -> f64 {
    mean(&forecasts.iter().map(|f| f.log_error()).collect::<Vec<_>>())
}

/// The total's bias over a set of forecasts — the log of summed points over summed actuals.
#[must_use]
pub fn total_bias(forecasts: &[Scored]) -> f64 {
    let points: f64 = forecasts.iter().map(|f| f.point).sum();
    let actuals: f64 = forecasts.iter().map(|f| f.actual).sum();
    (points / actuals).ln()
}

/// Both biases at every horizon from one to `deepest`, over every origin and district.
///
/// The profile the feed publishes beside the projection and `/method` draws. The two columns are
/// the same forecasts summed differently; the two rows are two populations of forecasts, since
/// restricting targets to [`BEFORE_THE_CLOSURE`] drops origins rather than reweighting them.
///
/// Pass [`DEEPEST_HORIZON`] and the panel's last year for the published profile.
///
/// **Empty where the Census panel is absent**, rather than a panic. [`profile`] may assert its
/// way out of that because nothing builds without it; this one is written into the feed, which a
/// contributor can build from a checkout that has no F-33 fixture in it, and an empty list is how
/// every other block of the feed says the same thing.
///
/// # Panics
///
/// If the panel is present and a horizon in the range still scores no forecast against
/// `latest_target`, which would mean `deepest` reaches past what the origins support —
/// [`deepest_horizon`] answers that.
#[must_use]
pub fn bias_profile(deepest: u16, latest_target: u16) -> Vec<Drift> {
    let histories = complete_histories();
    if histories.is_empty() {
        return Vec::new();
    }
    let at = |horizon: u16, until: u16| -> Option<Bias> {
        let forecasts: Vec<Scored> = scored_at(&histories, &ORIGINS, horizon, until)
            .into_iter()
            .flat_map(|(_, group)| group)
            .collect();
        (!forecasts.is_empty()).then(|| Bias {
            mean_district: mean_district_bias(&forecasts),
            total: total_bias(&forecasts),
        })
    };
    (1..=deepest)
        .map(|horizon| Drift {
            horizon,
            before_the_closure: at(horizon, BEFORE_THE_CLOSURE),
            across_it: at(horizon, latest_target).unwrap_or_else(|| {
                panic!(
                    "horizon {horizon} scores no forecast against a target no later than \
                     FY{latest_target}"
                )
            }),
        })
        .collect()
}

/// The horizon whose bias is furthest from zero on a drawn line, and by how much.
///
/// The curve's third figure, and the same shape as [`worst_gap`]: a line read only at its ends is
/// read least well where its finding lives. Ties go to the shallower horizon.
///
/// Returns the **signed** bias at that horizon rather than its magnitude, because the sign is the
/// whole of what separates these four lines — see `the-bias-published-beside-the-point`.
///
/// # Panics
///
/// If no row in `rows` carries the line `of` selects.
#[must_use]
pub fn worst_drift(rows: &[Drift], of: impl Fn(&Drift) -> Option<f64>) -> (u16, f64) {
    rows.iter()
        .filter_map(|row| of(row).map(|value| (row.horizon, value)))
        .max_by(|a, b| match a.1.abs().total_cmp(&b.1.abs()) {
            Ordering::Equal => b.0.cmp(&a.0),
            other => other,
        })
        .expect("a profile with no rows carrying this line")
}
