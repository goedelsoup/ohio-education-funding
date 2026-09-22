//! Whether the projection's growing positive bias should be corrected, and on what — answered
//! no, and on nothing, because the bias belongs to the years and not to the districts.
//!
//! [`the_horizons_the_backtest_stopped_short_of`] left one thing open: the point sits about 5.8%
//! high at the feed's ten-year horizon, [`the_bias_no_single_damping_can_remove`] says no damping
//! takes it out, and an interval centred on a biased point inherits the bias whatever its width.
//! #423 asked four questions about that. This file measures each and the answers are, in order:
//! the pre-closure drift is stable and is not the total's; neither the rate nor the level is a
//! lever; an asymmetric band is the level correction drawn differently; and the feed should
//! publish the bias rather than correct it. Everything below is the shipped `Shrunk` method at
//! its shipped constants, over the eight origins and thirteen horizons the panel supports.
//!
//! # The 5.8% is the mean district's, and the state's figure is half of it
//!
//! The feed publishes a statewide total and six hundred district figures from the same
//! projections, and the two carry different biases. Mean log error is the mean *district's*;
//! the total's is the log of summed points over summed actuals:
//!
//! | horizon | mean district, pre-closure | total, pre-closure | mean district, all | total, all |
//! |---|--:|--:|--:|--:|
//! | 1 | −0.0071 | −0.0052 | −0.0071 | −0.0052 |
//! | 3 | −0.0007 | −0.0050 | −0.0007 | −0.0050 |
//! | 5 | +0.0075 | **+0.0006** | +0.0171 | +0.0107 |
//! | 7 | +0.0179 | +0.0061 | +0.0294 | +0.0172 |
//! | 9 | +0.0272 | +0.0171 | +0.0434 | +0.0278 |
//! | 10 | — | — | **+0.0562** | **+0.0322** |
//! | 13 | — | — | +0.0734 | +0.0569 |
//!
//! Before the closure the total is within seven tenths of a point at every horizon to seven
//! years, and at five years its sign is not even stable across origins: +0.0068, +0.0019,
//! −0.0054 and −0.0010 for the four that land on FY2020 or earlier. The mean district drifts
//! at the same horizons. A correction that is right for one is wrong for the other.
//!
//! # The pre-closure drift is stable, and it is a year effect
//!
//! Per origin, the mean district's bias grows linearly in the horizon past the 1.43 years the
//! damping carries, at a rate that barely varies:
//!
//! | origin | clean horizons | bias per year past 1.43 |
//! |---|--:|--:|
//! | FY2011 | 8 | +0.0036 |
//! | FY2012 | 7 | +0.0039 |
//! | FY2013 | 6 | +0.0035 |
//! | FY2015 | 5 | +0.0038 |
//! | FY2016 | 4 | +0.0032 |
//!
//! That is the first thing #423 asked, and the answer is yes: about a third of a point a year,
//! within a fifth of itself, from five independent origins. But it is a **year** effect. Sorted
//! on anything the forecast could see at the origin, the quarters do not order:
//!
//! | five years, the population `the_trend_the_damping_discards` sorted | Q1 | Q2 | Q3 | Q4 |
//! |---|--:|--:|--:|--:|
//! | by the FY2009–FY2024 rate — includes the target years | +0.0519 | +0.0218 | +0.0025 | −0.0455 |
//! | by the rate to the origin — what a forecast can see | +0.0021 | +0.0214 | +0.0298 | +0.0151 |
//! | by the district's size at the origin | +0.0117 | +0.0200 | +0.0242 | +0.0125 |
//!
//! The first row is that file's finding and it is true: a district that shrank fastest over the
//! whole panel was, tautologically, shrinking during the target years, and a projection that
//! flattens is high for it. The second row is the same errors sorted on what was known when the
//! forecast was made, and the gradient is gone — the two middle quarters carry the most by either
//! key, which is a shape no correction keyed on either could use and this file does not explain.
//! *"A period artefact would show as a bias shared across districts"* — it is shared, and it is
//! one. See `the_bias_is_proportional_to_how_fast_a_district_is_shrinking`, whose docs now say
//! which sort it used.
//!
//! # The rate is not the lever, because the damping already is the persistence
//!
//! The second question was whether to correct on the rate, since the damping and the shrink push
//! that way. The natural rate-side correction is to decay the fitted rate toward some share
//! `kappa` of the district's long-run rate rather than toward zero, so a district that has
//! shrunk for a decade keeps shrinking. Measured on the pre-closure set:
//!
//! | `kappa` of the long-run rate carried | 5-year bias | 9-year bias | MAE to 5 years | MAE to 9 |
//! |---|--:|--:|--:|--:|
//! | 0 — what ships | +0.0075 | +0.0272 | 0.03737 | 0.04636 |
//! | 0.05 | ≈ 0 at 4 years | ≈ 0 at 9 | 0.03746 | **0.04622** |
//! | 0.25 | −0.0117 | −0.0254 | 0.03822 | 0.04731 |
//! | 1.00 | −0.0702 | −0.1869 | 0.05255 | 0.07780 |
//!
//! Carrying the whole long-run rate turns a +0.027 bias into −0.187 at nine years, because the
//! statewide decline of FY2009–FY2013, about 1.7% a year, became 0.3% a year by FY2015–FY2020.
//! The `kappa` that zeroes the bias rises with the horizon — 0.05 at four years, 0.10 at five,
//! 0.14 at eight — which is the shape [`the_bias_no_single_damping_can_remove`] found for the
//! damping, and the error-minimising value is **0.05**, worth **0.3%** on the nine-year set and
//! nothing on the five-year one. The surface is flat where the damping's was.
//!
//! Carrying the district's rate *relative to the state's* instead — so the deceleration is the
//! state's problem and only the district's deviation persists — does remove the ex-post gradient
//! in the first table above, and costs more: the `kappa` that zeroes the bias runs 0.29 at four
//! years to 0.73 at seven, and at that value mean absolute error is **14% higher** at five years
//! and **34% higher** at nine. Its error-minimising value is zero.
//!
//! The reason is one number. A district's long-run rate to the origin predicts its rate over the
//! next three, five, seven or nine years with a correlation of **0.30, 0.29, 0.29, 0.27** —
//! the same three tenths [`the-fitted-damping`] measured year on year (0.328 and 0.341) and set
//! the constant to. Relative to the state it is no more persistent (0.28 at five years). The
//! regression coefficient of the future rate on the past one falls from 0.33 at three years to
//! 0.15 at nine, and 1.43 year-equivalents spread over the horizon is 0.48 and 0.16: what ships
//! carries a little more than persists at short horizons and a little less at long ones, which
//! is exactly the sign pattern of the bias — negative to two years, positive after.
//!
//! # A level correction serves one of the two published figures and harms the other
//!
//! Fitted on the pre-closure mean district, the drift is +0.0031 per year past 1.43. Applied out
//! of sample to the forecasts that cross the closure it removes about half of what it finds —
//! +0.056 becomes +0.030 at ten years, and pooled coverage recovers from 60.3% to 65.6% — which
//! is what #423 predicted: the clean drift is one period effect and the closure is another,
//! twice its size, and the first says nothing about the second. Applied to the **total** it
//! over-corrects: the state's figure goes from within seven tenths of a point to about a point
//! low at every pre-closure horizon from three to seven. One constant cannot serve both.
//!
//! The third question — an asymmetric band — is the same correction drawn as a band. Widening
//! the lower end alone by the fitted drift holds 62.5% of the closure-spanning ten-year errors
//! where shifting the centre holds 65.6%; it is dominated, and it inherits the same defect.
//!
//! # So the bias is published, not corrected, and what to publish is now known
//!
//! Two numbers per horizon, not one: the mean district's and the total's, each before the
//! closure and across it. The sentence *"the point sits about 5.8% high at ten years"* is true
//! of the mean district across the closure; the state's total sits 3.2% high there, and was
//! within 0.7% at every horizon to seven years before it. Neither the damping, the shrink weight,
//! the exponent nor `sigma` moves; a feed field is a decision of its own and is tracked
//! separately.
//!
//! [`the_horizons_the_backtest_stopped_short_of`]: ./the_horizons_the_backtest_stopped_short_of.rs
//! [`the_bias_no_single_damping_can_remove`]: ./the_bias_no_single_damping_can_remove.rs
//! [`the-fitted-damping`]: ../../../.yidam/decisions/the-fitted-damping.yml

use dispersion::ohio_panel::{self, PanelRow};
use edfund_core::FiscalYear;
use project::series::{self, Method, Observation, Prior, ONE_SIGMA};
use project::{panel, report};
use std::collections::BTreeMap;

/// Every fiscal year the panel carries, oldest first. FY2014 is absent from the archive.
const PANEL_YEARS: [u16; 15] = [
    2009, 2010, 2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019, 2020, 2021, 2022, 2023, 2024,
];

/// Every origin the panel supports three observations for.
const EVERY_ORIGIN: [u16; 8] = [2011, 2012, 2013, 2015, 2016, 2017, 2018, 2019];

/// Targets no method is scored on: the year enrolment fell 54,777 in one step, and the rebound.
const PANDEMIC: [u16; 2] = [2021, 2022];

/// The last target year reachable without the forecast spanning the closure.
const BEFORE_THE_CLOSURE: u16 = 2020;

/// The damping and shrink weight these figures were measured at, pinned rather than imported.
///
/// The same reasoning as `SHIPPING_DAMPING` in `the_horizons_the_backtest_stopped_short_of`: a
/// file that followed the constants would turn a finding about the shipped method into a
/// tautology about whatever ships.
const SHIPPING_DAMPING: f64 = 0.30;
const SHIPPING_WEIGHT: f64 = 0.30;

/// The year-equivalents of a rate the damping carries in the limit: `1 / (1 - d)`.
///
/// 1.4286 at 0.30. The bias grows linearly past this point and is measured per year past it.
const YEARS_CARRIED: f64 = 1.0 / (1.0 - SHIPPING_DAMPING);

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

/// One forecast the backtest scores: everything the method saw at the origin, and the outcome.
#[derive(Clone, Copy)]
struct Forecast {
    origin: u16,
    horizon: u16,
    target: u16,
    /// The last observation, which every method carries forward from.
    base: f64,
    /// The three-point rate a production caller fits.
    own: f64,
    /// The district's long-run rate from the panel's start to the origin.
    toward: f64,
    /// The statewide long-run rate to the origin, for the relative variant.
    state: f64,
    /// The district's rate over the whole panel — which includes the target, and is the sort
    /// `the_trend_the_damping_discards` used.
    whole_panel: f64,
    actual: f64,
}

impl Forecast {
    fn crosses_the_closure(&self) -> bool {
        self.target > BEFORE_THE_CLOSURE
    }

    /// The shipped method's point: the blended rate, decayed toward zero.
    fn shipped(&self) -> f64 {
        self.carrying(0.0)
    }

    /// The shipped method with the blended rate decayed toward `floor` instead of toward zero.
    ///
    /// `floor` of zero reproduces [`series::advance`] exactly; the test below asserts so against
    /// the crate rather than trusting the reimplementation.
    fn carrying(&self, floor: f64) -> f64 {
        let blend = SHIPPING_WEIGHT * self.own + (1.0 - SHIPPING_WEIGHT) * self.toward;
        let mut value = self.base;
        let mut deviation = blend - floor;
        for _ in 0..self.horizon {
            value *= 1.0 + floor + deviation;
            deviation *= SHIPPING_DAMPING;
        }
        value
    }

    fn log_error(&self) -> f64 {
        (self.shipped() / self.actual).ln()
    }

    fn log_error_carrying(&self, floor: f64) -> f64 {
        (self.carrying(floor) / self.actual).ln()
    }

    /// The rate actually realised from the origin to the target.
    fn realised(&self) -> f64 {
        compound_rate(self.base, self.actual, f64::from(self.horizon))
    }
}

/// The statewide compound rate from the panel's start to `origin`.
fn state_rate_to(histories: &[History], origin: u16) -> f64 {
    let years: Vec<u16> = PANEL_YEARS
        .iter()
        .copied()
        .filter(|y| *y <= origin)
        .collect();
    let (first, last) = (years[0], *years.last().expect("an origin is in the panel"));
    let total = |y: u16| histories.iter().map(|h| h[&y]).sum::<f64>();
    compound_rate(total(first), total(last), f64::from(last - first))
}

/// Every forecast the panel can score, over every origin, horizon and district.
fn forecasts(histories: &[History]) -> Vec<Forecast> {
    let mut out = Vec::new();
    for history in histories {
        let whole_panel = compound_rate(history[&2009], history[&2024], 15.0);
        for origin in EVERY_ORIGIN {
            let state = state_rate_to(histories, origin);
            let years: Vec<u16> = PANEL_YEARS
                .iter()
                .copied()
                .filter(|y| *y <= origin)
                .collect();
            let last = years[years.len() - 1];
            let third_from_last = years[years.len() - 3];
            let own = compound_rate(
                history[&third_from_last],
                history[&last],
                f64::from(last - third_from_last),
            );
            let toward = compound_rate(
                history[&years[0]],
                history[&last],
                f64::from(last - years[0]),
            );
            for horizon in 1..=13u16 {
                let target = origin + horizon;
                if PANDEMIC.contains(&target) || !PANEL_YEARS.contains(&target) {
                    continue;
                }
                out.push(Forecast {
                    origin,
                    horizon,
                    target,
                    base: history[&last],
                    own,
                    toward,
                    state,
                    whole_panel,
                    actual: history[&target],
                });
            }
        }
    }
    out
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

/// Pearson correlation and the least-squares slope of `ys` on `xs`.
fn correlation_and_slope(xs: &[f64], ys: &[f64]) -> (f64, f64) {
    let (mx, my) = (mean(xs), mean(ys));
    let covariance = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (x - mx) * (y - my))
        .sum::<f64>();
    let vx = xs.iter().map(|x| (x - mx).powi(2)).sum::<f64>();
    let vy = ys.iter().map(|y| (y - my).powi(2)).sum::<f64>();
    (covariance / (vx * vy).sqrt(), covariance / vx)
}

/// The mean district's bias: the mean of the log errors.
fn mean_district_bias(set: &[&Forecast]) -> f64 {
    mean(&set.iter().map(|f| f.log_error()).collect::<Vec<_>>())
}

/// The total's bias: the log of summed points over summed actuals, which is what a statewide
/// figure carries.
fn total_bias(set: &[&Forecast]) -> f64 {
    let points: f64 = set.iter().map(|f| f.shipped()).sum();
    let actuals: f64 = set.iter().map(|f| f.actual).sum();
    (points / actuals).ln()
}

fn at_horizon(all: &[Forecast], horizon: u16, latest_target: u16) -> Vec<&Forecast> {
    all.iter()
        .filter(|f| f.horizon == horizon && f.target <= latest_target)
        .collect()
}

/// The mean log error in each quarter of `set`, sorted on `key`.
fn quartile_biases(set: &[&Forecast], key: impl Fn(&Forecast) -> f64) -> Vec<f64> {
    let mut keyed: Vec<(f64, f64)> = set.iter().map(|f| (key(f), f.log_error())).collect();
    keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("keys are finite"));
    let quarter = keyed.len() / 4;
    keyed
        .chunks(quarter)
        .take(4)
        .map(|chunk| mean(&chunk.iter().map(|k| k.1).collect::<Vec<_>>()))
        .collect()
}

fn is_monotone_decreasing(values: &[f64]) -> bool {
    values.windows(2).all(|pair| pair[1] < pair[0])
}

fn span(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::MIN, f64::max)
        - values.iter().copied().fold(f64::MAX, f64::min)
}

/// The production prior, taken from the path the feed takes.
fn prior() -> Prior {
    report::enrollment_growth_prior(&panel::panel(), ONE_SIGMA)
}

/// The share of errors a band of `width` holds around `centre`.
fn coverage(errors: &[f64], centre: f64, width: f64) -> f64 {
    errors
        .iter()
        .filter(|e| (*e - centre).abs() <= width)
        .count() as f64
        / errors.len() as f64
}

/// The drift fitted on the pre-closure mean district: bias per year past `YEARS_CARRIED`,
/// least squares through the origin over horizons one to nine.
fn fitted_drift(all: &[Forecast]) -> f64 {
    let (mut numerator, mut denominator) = (0.0, 0.0);
    for horizon in 1..=9u16 {
        let x = f64::from(horizon) - YEARS_CARRIED;
        let bias = mean_district_bias(&at_horizon(all, horizon, BEFORE_THE_CLOSURE));
        numerator += x * bias;
        denominator += x * x;
    }
    numerator / denominator
}

/// The reimplementation reproduces the crate to the bit at a floor of zero.
///
/// Everything else here varies the floor, and that variation has to be measured against a
/// method known to be the shipped one rather than a plausible copy of it.
#[test]
fn a_floor_of_zero_is_the_shipped_method_exactly() {
    let histories = complete_histories();
    let all = forecasts(&histories);
    let mut checked = 0;
    for forecast in all.iter().filter(|f| f.origin == 2016 && f.horizon == 7) {
        let history = histories
            .iter()
            .find(|h| {
                (h[&2016] - forecast.base).abs() < f64::EPSILON
                    && (h[&2023] - forecast.actual).abs() < f64::EPSILON
            })
            .expect("the forecast came from a history");
        let observed: Vec<Observation> = [2013u16, 2015, 2016]
            .iter()
            .map(|y| Observation {
                fiscal_year: FiscalYear(*y),
                value: history[y],
            })
            .collect();
        let point = series::project(
            &observed,
            FiscalYear(forecast.target),
            Method::Shrunk {
                rate: 0.0,
                damping: SHIPPING_DAMPING,
                weight: SHIPPING_WEIGHT,
                toward: forecast.toward,
            },
            Prior::none(),
        )
        .into_iter()
        .find(|p| p.fiscal_year == FiscalYear(forecast.target))
        .expect("the projection reaches the target year")
        .point;
        assert_eq!(
            forecast.shipped().to_bits(),
            point.to_bits(),
            "the reimplementation should reproduce `series::project` exactly: {} against {point}",
            forecast.shipped()
        );
        checked += 1;
    }
    assert_eq!(checked, 602, "every district is checked");
}

/// The mean district's 5.8% at ten years is the total's 3.2%, and before the closure the total
/// was very nearly unbiased with a sign that changed from origin to origin.
#[test]
fn the_total_carries_half_the_mean_districts_bias_and_none_of_it_before_the_closure() {
    let all = forecasts(&complete_histories());

    let ten = at_horizon(&all, 10, 2024);
    let (district, total) = (mean_district_bias(&ten), total_bias(&ten));
    assert!(
        (district - 0.0562).abs() < 0.004,
        "the mean district should sit about +0.056 high at ten years, sits {district:+.4}"
    );
    assert!(
        (total - 0.0322).abs() < 0.004,
        "and the total about +0.032, sits {total:+.4}"
    );
    assert!(
        total < district * 0.65,
        "the total's bias should be well under two thirds of the mean district's"
    );

    for horizon in 1..=7u16 {
        let bias = total_bias(&at_horizon(&all, horizon, BEFORE_THE_CLOSURE));
        assert!(
            bias.abs() < 0.007,
            "before the closure the total should be within seven tenths of a point at {horizon} \
             years, is {bias:+.4}"
        );
    }

    let mut signs = Vec::new();
    for origin in EVERY_ORIGIN {
        let set: Vec<&Forecast> = all
            .iter()
            .filter(|f| f.origin == origin && f.horizon == 5 && !f.crosses_the_closure())
            .collect();
        if !set.is_empty() {
            signs.push(total_bias(&set));
        }
    }
    assert_eq!(
        signs.len(),
        4,
        "four origins land five years out on FY2020 or earlier"
    );
    assert!(
        signs.iter().any(|s| *s > 0.0) && signs.iter().any(|s| *s < 0.0),
        "the total's five-year bias should change sign across origins: {signs:?}"
    );
}

/// The pre-closure drift is a third of a point a year at every origin that reaches four years.
///
/// The first thing #423 asked. Measured per origin rather than pooled, and it is stable: five
/// independent origins agree within a fifth of the value.
#[test]
fn the_pre_closure_drift_is_the_same_at_every_origin_that_reaches_four_years() {
    let all = forecasts(&complete_histories());
    let mut drifts = Vec::new();
    for origin in EVERY_ORIGIN {
        let deepest = (1..=13u16)
            .filter(|h| {
                origin + h <= BEFORE_THE_CLOSURE
                    && all.iter().any(|f| f.origin == origin && f.horizon == *h)
            })
            .max()
            .unwrap_or(0);
        if deepest < 4 {
            continue;
        }
        let set: Vec<&Forecast> = all
            .iter()
            .filter(|f| f.origin == origin && f.horizon == deepest)
            .collect();
        let per_year = mean_district_bias(&set) / (f64::from(deepest) - YEARS_CARRIED);
        drifts.push((origin, deepest, per_year));
    }
    assert_eq!(
        drifts.iter().map(|d| d.0).collect::<Vec<_>>(),
        [2011, 2012, 2013, 2015, 2016],
        "five origins reach four clean years"
    );
    for (origin, deepest, per_year) in &drifts {
        assert!(
            (0.0030..0.0040).contains(per_year),
            "origin {origin} at {deepest} years should drift +0.0030 to +0.0040 a year past the \
             carried span, drifts {per_year:+.5}"
        );
    }
    let values: Vec<f64> = drifts.iter().map(|d| d.2).collect();
    assert!(
        span(&values) < mean(&values) * 0.25,
        "and the five should agree within a quarter of their mean: {values:?}"
    );
}

/// Sorted on what the forecast could see, the quarters do not order; sorted on the whole panel
/// they do. The gradient was the sort.
///
/// Same population as `the_bias_is_proportional_to_how_fast_a_district_is_shrinking`: origins
/// FY2013 to FY2019, five years, every target the panel holds.
#[test]
fn sorted_on_what_the_forecast_could_see_the_bias_has_no_gradient() {
    let all = forecasts(&complete_histories());
    let set: Vec<&Forecast> = all
        .iter()
        .filter(|f| f.origin >= 2013 && f.horizon == 5)
        .collect();

    let ex_post = quartile_biases(&set, |f| f.whole_panel);
    assert!(
        is_monotone_decreasing(&ex_post) && span(&ex_post) > 0.09,
        "sorted on the FY2009-FY2024 rate the quarters should fall from about +0.05 to about \
         -0.05, which is that file's finding: {ex_post:?}"
    );

    let ex_ante = quartile_biases(&set, |f| f.toward);
    assert!(
        !is_monotone_decreasing(&ex_ante) && span(&ex_ante) < 0.03,
        "sorted on the rate to the origin the quarters should neither order nor spread: \
         {ex_ante:?}"
    );
    assert!(
        ex_ante.iter().all(|q| *q > 0.0),
        "and every quarter should be forecast high — the bias is shared, so it is a year \
         effect: {ex_ante:?}"
    );

    let by_size = quartile_biases(&set, |f| f.base);
    assert!(
        !is_monotone_decreasing(&by_size) && span(&by_size) < 0.02,
        "nor does size order it: {by_size:?}"
    );
}

/// A district's long-run rate predicts its next decade at three tenths, however long the decade.
///
/// The number the whole file turns on. The damping was set to the year-on-year persistence,
/// 0.328 and 0.341; the same statistic over three, five, seven and nine years is 0.30, 0.29,
/// 0.29 and 0.27, and relative to the state it is no higher. There is no longer-run persistence
/// the damping is failing to carry.
#[test]
fn a_districts_rate_persists_at_three_tenths_however_long_it_is_measured() {
    let all = forecasts(&complete_histories());
    let mut slopes = Vec::new();
    for horizon in [3u16, 5, 7, 9] {
        let set = at_horizon(&all, horizon, BEFORE_THE_CLOSURE);
        let past: Vec<f64> = set.iter().map(|f| f.toward).collect();
        let future: Vec<f64> = set.iter().map(|f| f.realised()).collect();
        let (r, slope) = correlation_and_slope(&past, &future);
        assert!(
            (0.25..0.32).contains(&r),
            "the long-run rate should predict the next {horizon} years at about three tenths, \
             predicts at {r:.3}"
        );
        slopes.push(slope);

        // Relative to the state: the district's deviation from the statewide rate, past and
        // future, where the future statewide rate is the origin's own realised one.
        let mut by_origin: BTreeMap<u16, (f64, f64)> = BTreeMap::new();
        for f in &set {
            let entry = by_origin.entry(f.origin).or_default();
            entry.0 += f.base;
            entry.1 += f.actual;
        }
        let past_relative: Vec<f64> = set.iter().map(|f| f.toward - f.state).collect();
        let future_relative: Vec<f64> = set
            .iter()
            .map(|f| {
                let (base, actual) = by_origin[&f.origin];
                f.realised() - compound_rate(base, actual, f64::from(f.horizon))
            })
            .collect();
        let (relative, _) = correlation_and_slope(&past_relative, &future_relative);
        assert!(
            relative < r + 0.01,
            "relative to the state the rate should be no more persistent: {relative:.3} \
             against {r:.3} at {horizon} years"
        );
    }

    // The regression coefficient falls with the horizon, and the shipped carry straddles it:
    // more than persists at three years, less at nine.
    assert!(
        (slopes[0] - 0.326).abs() < 0.03 && (slopes[3] - 0.147).abs() < 0.03,
        "the slope of the future rate on the past one should fall from about 0.33 at three \
         years to about 0.15 at nine: {slopes:?}"
    );
    let carried =
        |horizon: f64| (1.0 - SHIPPING_DAMPING.powf(horizon)) / (1.0 - SHIPPING_DAMPING) / horizon;
    assert!(
        carried(3.0) > slopes[0]
            && carried(9.0) > slopes[3] * 0.9
            && carried(9.0) < slopes[3] * 1.2,
        "what ships carries {:.3} of a rate per year at three years and {:.3} at nine, against \
         persistence of {:.3} and {:.3}",
        carried(3.0),
        carried(9.0),
        slopes[0],
        slopes[3]
    );
}

/// Decaying the rate toward the long-run rate rather than toward zero over-corrects at every
/// share, and the share that zeroes the bias rises with the horizon.
///
/// The second thing #423 asked, on the rate side. The statewide decline decelerated from 1.7% a
/// year to 0.3% between the panel's first years and its pre-closure ones, so a district's own
/// history over-states the trend ahead of it, and the damping's discarding of it was mostly
/// right.
#[test]
fn carrying_the_long_run_rate_over_corrects_and_no_share_of_it_is_unbiased_at_two_horizons() {
    let all = forecasts(&complete_histories());
    let early = state_rate_to(&complete_histories(), 2013);
    let late = compound_rate(
        complete_histories().iter().map(|h| h[&2015]).sum(),
        complete_histories().iter().map(|h| h[&2020]).sum(),
        5.0,
    );
    assert!(
        early < -0.015 && late > -0.005,
        "the state should have fallen about 1.7% a year to FY2013 and about 0.3% a year from \
         FY2015 to FY2020: {early:+.4} then {late:+.4}"
    );

    let bias_at = |horizon: u16, kappa: f64| {
        let set = at_horizon(&all, horizon, BEFORE_THE_CLOSURE);
        mean(
            &set.iter()
                .map(|f| f.log_error_carrying(kappa * f.toward))
                .collect::<Vec<_>>(),
        )
    };
    assert!(
        bias_at(9, 1.0) < -0.15,
        "carrying the whole long-run rate should turn +0.027 at nine years into about -0.19, \
         gives {:+.4}",
        bias_at(9, 1.0)
    );

    // The share that zeroes each horizon's bias, by bisection: the quantity is monotone in it.
    let crossing = |horizon: u16| -> f64 {
        let (mut low, mut high) = (-0.5f64, 1.5f64);
        assert!(bias_at(horizon, low) > 0.0 && bias_at(horizon, high) < 0.0);
        for _ in 0..40 {
            let mid = f64::midpoint(low, high);
            if bias_at(horizon, mid) > 0.0 {
                low = mid;
            } else {
                high = mid;
            }
        }
        f64::midpoint(low, high)
    };
    let (four, five, eight) = (crossing(4), crossing(5), crossing(8));
    assert!(
        four < five && five < eight,
        "the unbiased share should rise with the horizon: {four:.3}, {five:.3}, {eight:.3}"
    );
    assert!(
        (four - 0.053).abs() < 0.02 && (five - 0.098).abs() < 0.02 && (eight - 0.140).abs() < 0.02,
        "and be about 0.05, 0.10 and 0.14: {four:.3}, {five:.3}, {eight:.3}"
    );
}

/// The error-minimising share of the long-run rate is five hundredths, worth a third of a percent.
///
/// The strongest case for a rate-side correction, kept so the decision not to make one rests on
/// a measured number. Relative to the state, the optimum is zero.
#[test]
fn the_error_minimising_share_is_five_hundredths_and_buys_a_third_of_a_percent() {
    let all = forecasts(&complete_histories());
    let clean_to_nine: Vec<&Forecast> = all
        .iter()
        .filter(|f| f.horizon <= 9 && !f.crosses_the_closure())
        .collect();
    let clean_to_five: Vec<&Forecast> = clean_to_nine
        .iter()
        .copied()
        .filter(|f| f.horizon <= 5)
        .collect();
    let mae = |set: &[&Forecast], floor: &dyn Fn(&Forecast) -> f64| {
        mean(
            &set.iter()
                .map(|f| f.log_error_carrying(floor(f)).abs())
                .collect::<Vec<_>>(),
        )
    };
    let grid: Vec<f64> = (-2..=6).map(|i| f64::from(i) * 0.05).collect();

    let best = |set: &[&Forecast], relative: bool| -> (f64, f64) {
        grid.iter()
            .copied()
            .map(|kappa| {
                let floor = move |f: &Forecast| {
                    if relative {
                        kappa * (f.toward - f.state)
                    } else {
                        kappa * f.toward
                    }
                };
                (kappa, mae(set, &floor))
            })
            .fold((0.0, f64::MAX), |a, b| if b.1 < a.1 { b } else { a })
    };

    let (nine_kappa, nine_mae) = best(&clean_to_nine, false);
    assert!(
        (nine_kappa - 0.05).abs() < 1e-9,
        "over nine clean years the optimum should be 0.05, is {nine_kappa:.2}"
    );
    let shipped_nine = mae(&clean_to_nine, &|_| 0.0);
    let gain = shipped_nine / nine_mae - 1.0;
    assert!(
        gain > 0.0 && gain < 0.005,
        "and worth under half a percent against what ships, is {:.2}%",
        gain * 100.0
    );
    let (five_kappa, _) = best(&clean_to_five, false);
    assert!(
        five_kappa.abs() < 1e-9,
        "over five clean years the optimum should be what ships, is {five_kappa:.2}"
    );
    let (relative_kappa, _) = best(&clean_to_nine, true);
    assert!(
        relative_kappa <= 0.0,
        "relative to the state the optimum should be zero or below, is {relative_kappa:.2}"
    );
}

/// Carrying the district's rate relative to the state's removes the gradient and doubles the
/// dispersion.
///
/// The variant that could have worked: if the state's deceleration is the state's problem, only
/// the district's deviation from it need persist. It does not persist either — the previous test
/// puts the correlation at 0.28 — so the share that zeroes the bias adds far more variance than
/// bias it removes.
#[test]
fn carrying_the_relative_rate_zeroes_the_bias_at_a_cost_of_a_third_of_the_error() {
    let all = forecasts(&complete_histories());
    let bias_at = |horizon: u16, kappa: f64| {
        let set = at_horizon(&all, horizon, BEFORE_THE_CLOSURE);
        mean(
            &set.iter()
                .map(|f| f.log_error_carrying(kappa * (f.toward - f.state)))
                .collect::<Vec<_>>(),
        )
    };
    let mae_at = |horizon: u16, kappa: f64| {
        let set = at_horizon(&all, horizon, BEFORE_THE_CLOSURE);
        mean(
            &set.iter()
                .map(|f| f.log_error_carrying(kappa * (f.toward - f.state)).abs())
                .collect::<Vec<_>>(),
        )
    };
    let crossing = |horizon: u16| -> f64 {
        let (mut low, mut high) = (-0.5f64, 1.5f64);
        assert!(bias_at(horizon, low) > 0.0 && bias_at(horizon, high) < 0.0);
        for _ in 0..40 {
            let mid = f64::midpoint(low, high);
            if bias_at(horizon, mid) > 0.0 {
                low = mid;
            } else {
                high = mid;
            }
        }
        f64::midpoint(low, high)
    };

    let (four, five, seven, nine) = (crossing(4), crossing(5), crossing(7), crossing(9));
    assert!(
        (four - 0.29).abs() < 0.05 && (seven - 0.73).abs() < 0.05,
        "the unbiased share should run from about 0.29 at four years to about 0.73 at seven: \
         {four:.3}, {five:.3}, {seven:.3}, {nine:.3}"
    );
    let cost = |horizon: u16, kappa: f64| mae_at(horizon, kappa) / mae_at(horizon, 0.0) - 1.0;
    assert!(
        (cost(5, five) - 0.14).abs() < 0.03,
        "at the five-year unbiased share the error should be about 14% higher, is {:+.1}%",
        cost(5, five) * 100.0
    );
    assert!(
        (cost(9, nine) - 0.34).abs() < 0.05,
        "and at the nine-year one about 34% higher, is {:+.1}%",
        cost(9, nine) * 100.0
    );

    // The dispersion is what pays for it: at the whole relative rate, nine-year errors are
    // nearly twice as spread.
    let spread = |kappa: f64| {
        let set = at_horizon(&all, 9, BEFORE_THE_CLOSURE);
        stdev(
            &set.iter()
                .map(|f| f.log_error_carrying(kappa * (f.toward - f.state)))
                .collect::<Vec<_>>(),
        )
    };
    assert!(
        spread(1.0) > spread(0.0) * 1.8,
        "carrying the whole relative rate should nearly double the nine-year dispersion: \
         {:.4} against {:.4}",
        spread(1.0),
        spread(0.0)
    );
}

/// A drift fitted before the closure removes half of what crosses it, and puts the total low.
///
/// The level side of the second question, and the third question with it: an asymmetric band
/// is this correction drawn as a band, and is dominated by it.
#[test]
fn a_level_correction_fitted_before_the_closure_removes_half_of_what_crosses_it_and_puts_the_total_low(
) {
    let all = forecasts(&complete_histories());
    let drift = fitted_drift(&all);
    assert!(
        (drift - 0.0031).abs() < 0.0005,
        "the pre-closure drift should be about +0.0031 a year past the carried span, is \
         {drift:+.5}"
    );
    let correction = |horizon: u16| drift * (f64::from(horizon) - YEARS_CARRIED).max(0.0);

    // Out of sample: the forecasts that cross the closure, at the feed's horizon.
    let prior = prior();
    let ten: Vec<&Forecast> = all
        .iter()
        .filter(|f| f.horizon == 10 && f.crosses_the_closure())
        .collect();
    let errors: Vec<f64> = ten.iter().map(|f| f.log_error()).collect();
    let (before, after) = (mean(&errors), mean(&errors) - correction(10));
    assert!(
        (before - 0.0562).abs() < 0.004 && (after - 0.0295).abs() < 0.004,
        "the ten-year bias should fall from about +0.056 to about +0.030, falls {before:+.4} \
         to {after:+.4}"
    );
    assert!(
        after > before * 0.45,
        "which is about half of it remaining, not none: the closure is a second period effect"
    );

    let width = prior.spread(10);
    let centred = coverage(&errors, 0.0, width);
    let shifted = coverage(&errors, correction(10), width);
    let lower_only = errors
        .iter()
        .filter(|e| **e <= width && **e >= -width - correction(10))
        .count() as f64
        / errors.len() as f64;
    assert!(
        (centred - 0.603).abs() < 0.02 && (shifted - 0.656).abs() < 0.02,
        "pooled coverage at ten years should recover from about 60.3% to about 65.6%: \
         {centred:.3} to {shifted:.3}"
    );
    assert!(
        lower_only < shifted && (lower_only - 0.625).abs() < 0.02,
        "and widening the lower end alone should hold less, about 62.5%: {lower_only:.3}"
    );

    // Applied to the total, the same correction over-corrects before the closure.
    for horizon in 3..=7u16 {
        let set = at_horizon(&all, horizon, BEFORE_THE_CLOSURE);
        let corrected = total_bias(&set) - correction(horizon);
        assert!(
            corrected < -0.008 && corrected > -0.013,
            "the corrected total should sit about a point low at {horizon} years, sits \
             {corrected:+.4}"
        );
    }
}
