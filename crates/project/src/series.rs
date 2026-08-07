//! Projecting a short series forward, with an interval that says where it came from.
//!
//! # Three observations is not a time series
//!
//! Each district has three enrolled-ADM observations: FY2024, FY2025, FY2026. That is enough to
//! fit a trend and not remotely enough to estimate how wrong the trend might be — with three
//! points, a residual variance is one degree of freedom and means nothing.
//!
//! So the interval does not come from the district's own history. It comes from the
//! **cross-sectional spread of district growth rates**: we do not know how variable this
//! district is, but we know how much districts differ from one another, and that is a defensible
//! floor on the uncertainty. Every [`Projection`] records the [`Prior`] that produced its
//! interval, because an interval whose provenance is unstated is decoration.

use edfund_core::FiscalYear;

/// One observed value in a fiscal year.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Observation {
    /// The fiscal year observed.
    pub fiscal_year: FiscalYear,
    /// The value.
    pub value: f64,
}

/// Whether a figure rests on observed inputs or projected ones.
///
/// This is the distinction the [`project` skill](../../../.yidam/skills/project.md) insists on:
/// re-running a formula with a changed parameter is deterministic, and forecasting enrollment
/// five years out is not. A result that mixes them without saying so reports forecast error as
/// though it were policy effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    /// Every input is observed or published. Re-running gives the same answer.
    Observed,
    /// At least one input is forecast.
    Projected,
}

/// How a value was carried forward.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Method {
    /// Hold the last observation. The honest default when a series has one point.
    LastObserved,
    /// Compound annual growth between the first and last observation, applied unchanged.
    Cagr {
        /// The fitted annual rate.
        rate: f64,
    },
    /// Compound growth damped geometrically toward zero across the horizon.
    ///
    /// Enrollment trends do not persist: a district losing 2% a year does not lose 2% a year
    /// forever, because the decline is driven by cohorts moving through and by housing stock
    /// that eventually turns over. Undamped extrapolation is the standard way to produce a
    /// confident and absurd number at a ten-year horizon.
    Damped {
        /// The fitted first-year rate.
        rate: f64,
        /// Per-year decay applied to the rate. 1.0 is undamped.
        damping: f64,
    },
    /// Ordinary least squares on fiscal year, extrapolated linearly.
    LinearTrend {
        /// Fitted change per year, in the series' own units.
        slope: f64,
        /// Fitted value at the last observed year.
        intercept: f64,
    },
    /// A rate supplied by the caller rather than fitted from the data.
    ///
    /// Used where the corpus has one observation and no history — assessed valuation, most
    /// obviously. The assumption is then the caller's, and is recorded as theirs.
    Assumed {
        /// The assumed annual rate.
        rate: f64,
    },
}

impl Method {
    /// A short label for a table.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::LastObserved => "last-observed",
            Self::Cagr { .. } => "cagr",
            Self::Damped { .. } => "damped",
            Self::LinearTrend { .. } => "linear",
            Self::Assumed { .. } => "assumed",
        }
    }
}

/// The default damping factor.
///
/// A convention, not an estimate. Three observations per district cannot identify a damping
/// parameter, and there is no Ohio-specific study here to borrow one from; 0.85 is the value
/// damped-trend forecasting commonly defaults to. It is named as a constant so that a future
/// phase with a real enrollment history can replace it with a fitted number and see what moves.
pub const DEFAULT_DAMPING: f64 = 0.85;

/// One standard deviation. Covers about 68% of a normal distribution.
pub const ONE_SIGMA: f64 = 1.0;

/// The multiplier for an 80% interval under normality.
pub const EIGHTY_PERCENT: f64 = 1.281_551_6;

/// The dispersion used to widen a projection, and where it came from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Prior {
    /// Standard deviation of the annual growth rate, as a fraction.
    pub sigma: f64,
    /// How many standard deviations the interval spans on each side.
    pub z: f64,
    /// What produced `sigma`. Printed with every interval.
    pub source: &'static str,
}

impl Prior {
    /// A prior with no dispersion, for a point estimate with no claim of uncertainty.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            sigma: 0.0,
            z: ONE_SIGMA,
            source: "none — point estimate only",
        }
    }

    /// The multiplicative half-width after `horizon` years.
    ///
    /// Growth errors compound, so the band widens with the square root of the horizon rather
    /// than staying fixed — the standard random-walk result, and the reason a five-year
    /// projection is not five times as uncertain as a one-year one.
    #[must_use]
    pub fn spread(&self, horizon: u16) -> f64 {
        self.z * self.sigma * f64::from(horizon).sqrt()
    }
}

/// A projected value with the interval and method that produced it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projection {
    /// The fiscal year projected to.
    pub fiscal_year: FiscalYear,
    /// The central estimate.
    pub point: f64,
    /// Lower bound of the interval.
    pub low: f64,
    /// Upper bound of the interval.
    pub high: f64,
    /// How the point was produced.
    pub method: Method,
    /// Observed or projected. An in-sample year is [`Basis::Observed`] and carries no interval.
    pub basis: Basis,
    /// Where the interval came from.
    pub prior: Prior,
}

impl Projection {
    /// An observed value, restated as a projection so a series can hold both.
    #[must_use]
    pub fn observed(observation: Observation) -> Self {
        Self {
            fiscal_year: observation.fiscal_year,
            point: observation.value,
            low: observation.value,
            high: observation.value,
            method: Method::LastObserved,
            basis: Basis::Observed,
            prior: Prior::none(),
        }
    }

    /// Half the interval width as a fraction of the point.
    #[must_use]
    pub fn relative_width(&self) -> f64 {
        if self.point.abs() < f64::EPSILON {
            return 0.0;
        }
        (self.high - self.low) / (2.0 * self.point)
    }
}

/// Fit a method to a series and carry it forward to `through`.
///
/// Observations are returned as-is at [`Basis::Observed`]; years past the last observation are
/// [`Basis::Projected`] and carry the interval. A caller that wants only the forecast can filter
/// on `basis`, and a caller that forgets to has at least been told.
///
/// An empty series projects nothing. A one-point series can only be carried flat, whatever
/// method is asked for — stated in the returned [`Method`] rather than silently.
#[must_use]
pub fn project(
    observations: &[Observation],
    through: FiscalYear,
    method: Method,
    prior: Prior,
) -> Vec<Projection> {
    let mut out: Vec<Projection> = observations
        .iter()
        .copied()
        .map(Projection::observed)
        .collect();
    let Some(last) = observations.last().copied() else {
        return out;
    };

    let fitted = fit(observations, method);
    let mut year = last.fiscal_year.0 + 1;
    while year <= through.0 {
        let horizon = year - last.fiscal_year.0;
        let point = advance(last.value, fitted, horizon);
        let spread = prior.spread(horizon);
        out.push(Projection {
            fiscal_year: FiscalYear(year),
            point,
            // A multiplicative band: growth uncertainty scales the level, so a large district
            // and a small one get the same *proportional* interval, which is the claim being
            // made. An additive band would imply the opposite.
            low: point * (-spread).exp(),
            high: point * spread.exp(),
            method: fitted,
            basis: Basis::Projected,
            prior,
        });
        year += 1;
    }
    out
}

/// Resolve a requested method against what the data can support.
fn fit(observations: &[Observation], requested: Method) -> Method {
    if observations.len() < 2 {
        // One point cannot support a trend, whatever was asked for. An assumed rate is the
        // exception: it was never claimed to come from the data.
        return match requested {
            Method::Assumed { rate } => Method::Assumed { rate },
            _ => Method::LastObserved,
        };
    }
    let first = observations[0];
    let last = observations[observations.len() - 1];
    let years = f64::from(last.fiscal_year.0 - first.fiscal_year.0);

    match requested {
        Method::LastObserved | Method::Assumed { .. } => requested,
        Method::Cagr { .. } => Method::Cagr {
            rate: compound_rate(first.value, last.value, years),
        },
        Method::Damped { damping, .. } => Method::Damped {
            rate: compound_rate(first.value, last.value, years),
            damping,
        },
        Method::LinearTrend { .. } => {
            let (slope, intercept) = least_squares(observations);
            Method::LinearTrend { slope, intercept }
        }
    }
}

fn compound_rate(from: f64, to: f64, years: f64) -> f64 {
    if from <= 0.0 || to <= 0.0 || years <= 0.0 {
        return 0.0;
    }
    (to / from).powf(1.0 / years) - 1.0
}

/// Slope per year and fitted value at the last observed year.
fn least_squares(observations: &[Observation]) -> (f64, f64) {
    let n = observations.len() as f64;
    let xs: Vec<f64> = observations
        .iter()
        .map(|o| f64::from(o.fiscal_year.0))
        .collect();
    let mean_x = xs.iter().sum::<f64>() / n;
    let mean_y = observations.iter().map(|o| o.value).sum::<f64>() / n;
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    for (x, observation) in xs.iter().zip(observations) {
        numerator += (x - mean_x) * (observation.value - mean_y);
        denominator += (x - mean_x).powi(2);
    }
    let slope = if denominator.abs() < f64::EPSILON {
        0.0
    } else {
        numerator / denominator
    };
    let last_x = xs[xs.len() - 1];
    (slope, mean_y + slope * (last_x - mean_x))
}

/// Carry `base` forward `horizon` years under `method`.
fn advance(base: f64, method: Method, horizon: u16) -> f64 {
    match method {
        Method::LastObserved => base,
        Method::Cagr { rate } | Method::Assumed { rate } => {
            base * (1.0 + rate).powi(i32::from(horizon))
        }
        Method::Damped { rate, damping } => {
            let mut value = base;
            let mut step = rate;
            for _ in 0..horizon {
                value *= 1.0 + step;
                step *= damping;
            }
            value
        }
        Method::LinearTrend { slope, intercept } => intercept + slope * f64::from(horizon),
    }
}

/// Standard deviation of a slice, over `n - 1`.
#[must_use]
pub fn standard_deviation(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn series(start: u16, values: &[f64]) -> Vec<Observation> {
        values
            .iter()
            .enumerate()
            .map(|(index, value)| Observation {
                fiscal_year: FiscalYear(start + index as u16),
                value: *value,
            })
            .collect()
    }

    fn prior() -> Prior {
        Prior {
            sigma: 0.02,
            z: ONE_SIGMA,
            source: "test",
        }
    }

    #[test]
    fn observed_years_come_back_unchanged_and_without_an_interval() {
        let out = project(
            &series(2024, &[100.0, 98.0, 96.0]),
            FiscalYear(2026),
            Method::LastObserved,
            prior(),
        );
        assert_eq!(out.len(), 3);
        for projection in &out {
            assert_eq!(projection.basis, Basis::Observed);
            assert_eq!(projection.low, projection.high);
        }
    }

    #[test]
    fn projected_years_are_marked_and_carry_an_interval() {
        let out = project(
            &series(2024, &[100.0, 98.0, 96.0]),
            FiscalYear(2029),
            Method::Cagr { rate: 0.0 },
            prior(),
        );
        let forecast: Vec<&Projection> =
            out.iter().filter(|p| p.basis == Basis::Projected).collect();
        assert_eq!(forecast.len(), 3, "FY2027, FY2028, FY2029");
        for projection in forecast {
            assert!(projection.low < projection.point && projection.point < projection.high);
        }
    }

    #[test]
    fn a_compound_rate_is_fitted_from_the_endpoints() {
        let out = project(
            &series(2024, &[100.0, 98.0, 96.04]),
            FiscalYear(2027),
            Method::Cagr { rate: 0.0 },
            Prior::none(),
        );
        let Method::Cagr { rate } = out.last().unwrap().method else {
            panic!("expected a fitted CAGR")
        };
        assert!((rate - -0.02).abs() < 1e-9, "{rate}");
        // One more year of -2%.
        assert!((out.last().unwrap().point - 94.1192).abs() < 1e-6);
    }

    #[test]
    fn damping_pulls_a_long_horizon_back_toward_the_last_observation() {
        let observations = series(2024, &[100.0, 98.0, 96.04]);
        let undamped = project(
            &observations,
            FiscalYear(2036),
            Method::Cagr { rate: 0.0 },
            Prior::none(),
        );
        let damped = project(
            &observations,
            FiscalYear(2036),
            Method::Damped {
                rate: 0.0,
                damping: DEFAULT_DAMPING,
            },
            Prior::none(),
        );
        let far_undamped = undamped.last().unwrap().point;
        let far_damped = damped.last().unwrap().point;
        assert!(
            far_damped > far_undamped,
            "damped {far_damped} should stay above undamped {far_undamped} for a declining series"
        );
        assert!(far_damped < 96.04, "it is still a decline");
    }

    #[test]
    fn the_interval_widens_with_the_square_root_of_the_horizon() {
        let out = project(
            &series(2024, &[100.0, 100.0, 100.0]),
            FiscalYear(2030),
            Method::LastObserved,
            prior(),
        );
        let forecast: Vec<&Projection> =
            out.iter().filter(|p| p.basis == Basis::Projected).collect();
        let first = forecast[0].relative_width();
        let fourth = forecast[3].relative_width();
        // Four years out is twice as wide as one year out, not four times.
        assert!((fourth / first - 2.0).abs() < 0.05, "{first} {fourth}");
    }

    #[test]
    fn a_single_observation_can_only_be_carried_flat() {
        let out = project(
            &series(2026, &[500.0]),
            FiscalYear(2030),
            Method::Cagr { rate: 0.0 },
            prior(),
        );
        let forecast = out.last().unwrap();
        assert_eq!(forecast.method, Method::LastObserved);
        assert!((forecast.point - 500.0).abs() < 1e-9);
    }

    #[test]
    fn an_assumed_rate_survives_a_single_observation_because_it_never_came_from_the_data() {
        // Assessed valuation has exactly one observation in this corpus. Refusing to project it
        // is right; pretending a fitted trend exists would not be.
        let out = project(
            &series(2023, &[200_000.0]),
            FiscalYear(2025),
            Method::Assumed { rate: 0.03 },
            Prior::none(),
        );
        let forecast = out.last().unwrap();
        assert_eq!(forecast.method, Method::Assumed { rate: 0.03 });
        assert!((forecast.point - 200_000.0 * 1.03 * 1.03).abs() < 1e-6);
    }

    #[test]
    fn a_linear_trend_fits_a_straight_line_exactly() {
        let out = project(
            &series(2024, &[100.0, 90.0, 80.0]),
            FiscalYear(2028),
            Method::LinearTrend {
                slope: 0.0,
                intercept: 0.0,
            },
            Prior::none(),
        );
        let Method::LinearTrend { slope, .. } = out.last().unwrap().method else {
            panic!("expected a fitted trend")
        };
        assert!((slope - -10.0).abs() < 1e-9);
        assert!(
            (out.last().unwrap().point - 60.0).abs() < 1e-9,
            "FY2028 is two years past FY2026"
        );
    }

    #[test]
    fn an_empty_series_projects_nothing_rather_than_zero() {
        assert!(project(&[], FiscalYear(2030), Method::LastObserved, prior()).is_empty());
    }

    #[test]
    fn a_prior_with_no_dispersion_produces_a_bare_point() {
        let out = project(
            &series(2024, &[100.0, 100.0]),
            FiscalYear(2028),
            Method::LastObserved,
            Prior::none(),
        );
        let forecast = out.last().unwrap();
        assert_eq!(forecast.low, forecast.point);
        assert_eq!(forecast.high, forecast.point);
    }

    #[test]
    fn the_interval_is_proportional_so_it_does_not_depend_on_district_size() {
        let small = project(
            &series(2024, &[500.0, 500.0]),
            FiscalYear(2030),
            Method::LastObserved,
            prior(),
        );
        let large = project(
            &series(2024, &[50_000.0, 50_000.0]),
            FiscalYear(2030),
            Method::LastObserved,
            prior(),
        );
        let width = |v: &[Projection]| v.last().unwrap().relative_width();
        assert!((width(&small) - width(&large)).abs() < 1e-12);
    }

    #[test]
    fn standard_deviation_needs_two_points() {
        assert_eq!(standard_deviation(&[]), 0.0);
        assert_eq!(standard_deviation(&[1.0]), 0.0);
        assert!(
            (standard_deviation(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]) - 2.138_089).abs()
                < 1e-5
        );
    }
}
