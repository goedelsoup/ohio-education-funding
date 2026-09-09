//! What damping costs the point estimate, which no fitting decision here has ever scored.
//!
//! [`the-fitted-damping`] chose `DEFAULT_DAMPING` on mean absolute log error and
//! [`the-shrunk-rate`] chose the blend weight the same way. Absolute error cannot see a sign, so
//! neither could see what the sibling file
//! [`the_floor_the_interval_rests_on`] noticed in passing: the projections run **positive** and
//! more so the further out they go. This measures where that comes from and what it is worth at
//! the horizon the feed publishes.
//!
//! # Damping causes it, and the relationship is monotone
//!
//! Mean log forecast error at five years, over the same 602-district backtest:
//!
//! | damping | h=1 | h=3 | h=5 | dispersion at h=5 |
//! |---|--:|--:|--:|--:|
//! | 0.00 — flat immediately | −0.00018 | +0.01091 | **+0.03347** | 0.07578 |
//! | 0.30 — what ships | −0.00018 | +0.00785 | **+0.03023** | 0.07574 |
//! | 0.60 | −0.00018 | +0.00333 | +0.02347 | 0.07860 |
//! | 0.85 — the old convention | −0.00018 | −0.00160 | +0.01256 | 0.09026 |
//! | 1.00 — undamped | −0.00018 | −0.00510 | **+0.00231** | 0.10657 |
//!
//! The undamped trend is very nearly unbiased and 41% more dispersed. **This is a bias-variance
//! trade, and it was priced on a metric that only sees one side of it.** Choosing 0.30 was right
//! for absolute error and is not free; the bias is what it cost.
//!
//! The h=1 column is identical everywhere because [`series::advance`] decays the rate only from
//! the second step onward — a useful check that the harness is measuring damping and not
//! something else.
//!
//! # The mechanism is arithmetic, not a property of Ohio
//!
//! Damping applies the fitted rate for `(1 - d^h)/(1 - d)` year-equivalents however long the
//! horizon is — **1.43 years at d = 0.30, for every horizon past about six.** So a projection
//! misses `h - 1.43` years of whatever trend the district is on, and in a shrinking population
//! that miss is upward. Sorting the 602 districts by their own fifteen-year rate:
//!
//! | quartile | median rate | miss × rate predicts | observed bias at h=5 |
//! |---|--:|--:|--:|
//! | Q1 fastest-declining | −2.22%/yr | +0.0795 | **+0.0804** |
//! | Q2 | −1.48%/yr | +0.0528 | +0.0416 |
//! | Q3 | −0.90%/yr | +0.0320 | +0.0212 |
//! | Q4 roughly flat | +0.01%/yr | −0.0003 | −0.0195 |
//!
//! The sign flips exactly where the growth rate does. The prediction is near-exact for Q1 and
//! over-states for the flatter quartiles, because a rate fitted on three points is not the
//! fifteen-year rate this table sorts on — the gap between them is largest where the long rate
//! is smallest.
//!
//! **The fastest-declining quarter of Ohio districts are forecast 8% too high at five years.**
//! Those are disproportionately the districts the guarantee protects — 57.2% of that quarter are
//! on the guarantee at observed enrolment against 34.2% of the flattest quarter — so the
//! over-forecast lands hardest where the guarantee is what pays, and the error runs in the
//! direction that under-states guarantee reliance. The gradient is not clean: the two middle
//! quarters sit at 48.7% and 53.3%, so this is a difference between the flattest quarter and the
//! rest rather than a rate-by-rate relationship.
//!
//! # At the published horizon the trend is simply gone
//!
//! Statewide ADM, projected from the production panel by the shipped method:
//!
//! | | shipped, d = 0.30 | undamped | shipped year-on-year |
//! |---|--:|--:|--:|
//! | FY2026 observed | 1,401,939 | 1,401,939 | — |
//! | FY2027 | 1,389,440 | 1,389,440 | −12,500 |
//! | FY2028 | 1,385,785 | 1,377,258 | −3,654 |
//! | FY2030 | 1,384,372 | 1,353,830 | −1,413 |
//! | FY2032 | 1,384,245 | 1,331,615 | −127 |
//! | FY2036 | 1,384,232 | 1,290,680 | −1 |
//!
//! **The feed's horizon is FY2036 and the projection stops moving around FY2030.** Between the
//! two years this repository actually publishes, FY2032 and FY2036, statewide enrolment falls by
//! **twelve pupils** — across four years and 609 districts, in a state whose F-33 panel records
//! a fall of 208,918 over fifteen.
//!
//! That is not a defect in [`series::advance`], which does what damping means. It is that a
//! ten-year panel drawn this way carries about four years of demographic content and six years of
//! the same number relabelled, and nothing said so.
//!
//! # What this does not do
//!
//! It does not move the constant. 0.30 minimises the error the decisions were made on, the
//! backtest reaches five years and the feed publishes ten, and an undamped projection would trade
//! a known bias for 41% more dispersion. Whether the point estimate should be de-biased is a
//! decision with a record. What is settled here is its size and its cause.
//!
//! [`the-fitted-damping`]: ../../../.yidam/decisions/the-fitted-damping.yml
//! [`the-shrunk-rate`]: ../../../.yidam/decisions/the-shrunk-rate.yml

use dispersion::ohio_panel::{self, PanelRow};
use edfund_core::FiscalYear;
use project::policy::Policy;
use project::series::{
    self, Method, Observation, Prior, DEFAULT_DAMPING, DEFAULT_SHRINK_WEIGHT, ONE_SIGMA,
};
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

fn forecast(history: &History, origin: u16, target: u16, damping: f64) -> f64 {
    series::project(
        &observed_to(history, origin),
        FiscalYear(target),
        Method::Damped { rate: 0.0, damping },
        Prior::none(),
    )
    .into_iter()
    .find(|p| p.fiscal_year == FiscalYear(target))
    .expect("the projection reaches the target year")
    .point
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn stdev(values: &[f64]) -> f64 {
    let m = mean(values);
    let n = values.len() as f64;
    (values.iter().map(|v| (v - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
}

/// Log forecast errors at one horizon and damping, over every origin and district.
fn log_errors(histories: &[History], damping: f64, horizon: u16) -> Vec<f64> {
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
            errors.push((forecast(history, origin, target, damping) / actual).ln());
        }
    }
    errors
}

/// Statewide projected ADM at `year`, by the production path.
fn statewide(year: u16, damping: f64) -> f64 {
    panel::panel()
        .iter()
        .map(|record| {
            let observed: Vec<Observation> = [2024u16, 2025, 2026]
                .iter()
                .zip(record.adm_history)
                .map(|(y, value)| Observation {
                    fiscal_year: FiscalYear(*y),
                    value,
                })
                .collect();
            let method = record.projection_method(Method::Shrunk {
                rate: 0.0,
                damping,
                weight: DEFAULT_SHRINK_WEIGHT,
                toward: 0.0,
            });
            series::project(&observed, FiscalYear(year), method, Prior::none())
                .into_iter()
                .find(|p| p.fiscal_year == FiscalYear(year))
                .map_or(0.0, |p| p.point)
        })
        .sum()
}

/// The bias is monotone in the damping, and the undamped trend is very nearly unbiased.
///
/// This is the trade in one assertion: the parameter that minimises absolute error is near the
/// end of the range that maximises bias.
#[test]
fn the_five_year_bias_falls_monotonically_as_the_damping_rises() {
    let histories = complete_histories();
    let biases: Vec<f64> = [0.0, 0.30, 0.60, 0.85, 1.00]
        .iter()
        .map(|d| mean(&log_errors(&histories, *d, 5)))
        .collect();
    for pair in biases.windows(2) {
        assert!(
            pair[1] < pair[0],
            "bias should fall as damping rises: {:+.5} then {:+.5}",
            pair[0],
            pair[1]
        );
    }
    assert!(
        (biases[1] - 0.0302).abs() < 0.003,
        "at the shipped 0.30 the five-year bias should be about +0.030, is {:+.5}",
        biases[1]
    );
    assert!(
        biases[4].abs() < 0.005,
        "undamped should be very nearly unbiased, is {:+.5}",
        biases[4]
    );
}

/// And it buys that bias with variance: undamped is 41% more dispersed.
///
/// Without this the previous test would read as an argument for undamping, which it is not.
#[test]
fn the_undamped_trend_pays_for_its_lack_of_bias_in_dispersion() {
    let histories = complete_histories();
    let shipped = stdev(&log_errors(&histories, DEFAULT_DAMPING, 5));
    let undamped = stdev(&log_errors(&histories, 1.00, 5));
    assert!(
        undamped > shipped * 1.35,
        "undamped should be much more dispersed: {undamped:.5} against {shipped:.5}"
    );
}

/// The bias tracks the district's own long-run rate, and changes sign where the rate does.
///
/// A period artefact would show as a bias shared across districts. This is not that: it is
/// proportional to how fast a district is shrinking, which is what a mechanical explanation
/// predicts and a period explanation does not.
#[test]
fn the_bias_is_proportional_to_how_fast_a_district_is_shrinking() {
    let histories = complete_histories();
    let mut rated: Vec<(f64, &History)> = histories
        .iter()
        .map(|h| (compound_rate(h[&2009], h[&2024], 15.0), h))
        .collect();
    rated.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("rates are finite"));

    let quartile = rated.len() / 4;
    let biases: Vec<f64> = rated
        .chunks(quartile)
        .take(4)
        .map(|chunk| {
            let mut errors = Vec::new();
            for (_, history) in chunk {
                for origin in ORIGINS {
                    let target = origin + 5;
                    if PANDEMIC.contains(&target) {
                        continue;
                    }
                    let Some(actual) = history.get(&target) else {
                        continue;
                    };
                    errors.push((forecast(history, origin, target, DEFAULT_DAMPING) / actual).ln());
                }
            }
            mean(&errors)
        })
        .collect();

    for pair in biases.windows(2) {
        assert!(
            pair[1] < pair[0],
            "bias should fall from the fastest-declining quartile to the flattest: \
             {:+.5} then {:+.5}",
            pair[0],
            pair[1]
        );
    }
    assert!(
        (biases[0] - 0.0804).abs() < 0.005,
        "the fastest-declining quartile should be forecast about 8% high, is {:+.4}",
        biases[0]
    );
    assert!(
        biases[3] < 0.0,
        "the flattest quartile should be forecast low, is {:+.5}",
        biases[3]
    );
}

/// The mechanism: damping spends the rate over 1.43 years however long the horizon is.
///
/// `(1 - d^h)/(1 - d)` at d = 0.30 is 1.4251 at five years and 1.4286 in the limit — the whole
/// difference between a five-year and a ten-year projection is four thousandths of one year's
/// growth. Checked against the fastest-declining quartile, where the three-point fitted rate is
/// closest to the long-run rate this predicts from.
#[test]
fn damping_spends_the_rate_over_a_year_and_a_half_whatever_the_horizon() {
    let year_equivalents = |d: f64, h: f64| (1.0 - d.powf(h)) / (1.0 - d);
    let five = year_equivalents(DEFAULT_DAMPING, 5.0);
    let ten = year_equivalents(DEFAULT_DAMPING, 10.0);
    assert!(
        (five - 1.4251).abs() < 0.001 && (ten - 1.4286).abs() < 0.001,
        "five years buys {five:.4} year-equivalents and ten buys {ten:.4}"
    );

    let histories = complete_histories();
    let mut rated: Vec<(f64, &History)> = histories
        .iter()
        .map(|h| (compound_rate(h[&2009], h[&2024], 15.0), h))
        .collect();
    rated.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("rates are finite"));
    let quartile = &rated[..rated.len() / 4];
    let median_rate = quartile[quartile.len() / 2].0;
    let predicted = -(5.0 - five) * median_rate;

    let mut errors = Vec::new();
    for (_, history) in quartile {
        for origin in ORIGINS {
            let target = origin + 5;
            if PANDEMIC.contains(&target) {
                continue;
            }
            let Some(actual) = history.get(&target) else {
                continue;
            };
            errors.push((forecast(history, origin, target, DEFAULT_DAMPING) / actual).ln());
        }
    }
    let observed = mean(&errors);
    assert!(
        (predicted - observed).abs() < 0.005,
        "arithmetic predicts {predicted:+.4} for the fastest-declining quartile, observed \
         {observed:+.4}"
    );
}

/// Between the two years this repository publishes, the projection loses twelve pupils.
///
/// The point of the assertion is the smallness. Anything that made the tail carry real
/// demographic content would break it, and should.
#[test]
fn the_projection_stops_moving_four_years_before_the_horizon_it_publishes() {
    let fy2032 = statewide(2032, DEFAULT_DAMPING);
    let fy2036 = statewide(2036, DEFAULT_DAMPING);
    assert!(
        (fy2032 - 1_384_245.0).abs() < 50.0,
        "FY2032 statewide ADM should be about 1,384,245, is {fy2032:.0}"
    );
    assert!(
        (fy2032 - fy2036).abs() < 100.0,
        "FY2032 to FY2036 should be a rounding error, is {:.0} pupils",
        fy2032 - fy2036
    );
    let undamped_2036 = statewide(2036, 1.00);
    assert!(
        fy2036 - undamped_2036 > 90_000.0,
        "the damping should hold about 94,000 pupils above the undamped trend, holds {:.0}",
        fy2036 - undamped_2036
    );
}

/// What moving the damping does **under the method the feed actually runs**.
///
/// `the_damping_nobody_fitted` measures the same move under `Method::Damped`, which is what
/// shipped when [`the-fitted-damping`] was decided and what that decision's evidence must keep
/// reproducing. [`the-shrunk-rate`] then changed the method, and every figure moved:
///
/// | | under `Damped` | under `Shrunk`, as shipped |
/// |---|--:|--:|
/// | statewide ADM | +4.66%, 61,297 pupils | **+3.38%, 45,294 pupils** |
/// | realized state aid | +1.05%, \$75.4m | **+1.42%, \$101.0m** |
/// | districts changing guarantee status | 47 — 367 to 320 | **44 — 356 to 312** |
///
/// The qualitative claim moved too. Under `Damped` the enrollment move was four times the aid
/// move and the corpus says so; under `Shrunk` it is under two and a half times, because a
/// shrunk rate moves fewer districts across the guarantee threshold and each crossing carries
/// more aid. This test exists so that the pair cannot drift apart again unnoticed.
#[test]
fn the_feed_moves_less_in_enrollment_and_more_in_aid_than_the_damped_figures_say() {
    let districts = panel::panel();
    let prior = report::enrollment_growth_prior(&districts, ONE_SIGMA);
    let at = |damping: f64| {
        report::forecast(
            &districts,
            &Policy::current_law(),
            FiscalYear(2036),
            Method::Shrunk {
                rate: 0.0,
                damping,
                weight: DEFAULT_SHRINK_WEIGHT,
                toward: 0.0,
            },
            prior,
        )
    };
    let convention = at(0.85);
    let shipped = at(DEFAULT_DAMPING);

    let adm_move = shipped.adm / convention.adm - 1.0;
    let aid_move = shipped.realized_aid / convention.realized_aid - 1.0;
    assert!(
        (0.030..0.038).contains(&adm_move),
        "ADM should move about +3.38%; it moves {:+.2}%",
        adm_move * 100.0
    );
    assert!(
        (0.012..0.017).contains(&aid_move),
        "aid should move about +1.42%; it moves {:+.2}%",
        aid_move * 100.0
    );
    assert!(
        adm_move < aid_move * 3.0,
        "under the shrunk rate the guarantee absorbs less: ADM {:+.2}% against aid {:+.2}%, \
         which is no longer the four-to-one the corpus was written against",
        adm_move * 100.0,
        aid_move * 100.0
    );
    assert_eq!(convention.on_guarantee, 356);
    assert_eq!(shipped.on_guarantee, 312);
}

/// The over-forecast lands on the districts the guarantee pays.
///
/// Measured at *observed* enrolment, so it carries no forecast error of its own. The comparison
/// that holds is the flattest quarter against the rest; the three declining quarters do not
/// order among themselves, and the assertion below says only what the data supports.
#[test]
fn the_districts_forecast_high_are_the_ones_the_guarantee_pays() {
    let districts = panel::panel();
    let effect = report::simulate(&districts, &Policy::current_law());
    let on: BTreeMap<&str, bool> = effect
        .outcomes
        .iter()
        .map(|o| (o.irn.as_str(), o.on_guarantee))
        .collect();

    let mut rated: Vec<(f64, bool)> = districts
        .iter()
        .filter_map(|d| Some((d.long_run_enrollment_rate?, *on.get(d.irn.as_str())?)))
        .collect();
    rated.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("rates are finite"));
    assert_eq!(
        rated.len(),
        609,
        "every district carries a long-run rate and an outcome"
    );

    let share = |chunk: &[(f64, bool)]| {
        chunk.iter().filter(|(_, on)| *on).count() as f64 / chunk.len() as f64
    };
    let quartile = rated.len() / 4;
    let shares: Vec<f64> = rated.chunks(quartile).take(4).map(share).collect();

    assert!(
        (shares[0] - 0.572).abs() < 0.02,
        "the fastest-declining quarter should be about 57.2% guaranteed, is {:.1}%",
        shares[0] * 100.0
    );
    assert!(
        (shares[3] - 0.342).abs() < 0.02,
        "the flattest quarter should be about 34.2% guaranteed, is {:.1}%",
        shares[3] * 100.0
    );
    for (i, declining) in shares[..3].iter().enumerate() {
        assert!(
            *declining > shares[3] + 0.10,
            "declining quarter {} at {:.1}% should sit well above the flattest at {:.1}%",
            i + 1,
            declining * 100.0,
            shares[3] * 100.0
        );
    }
}
