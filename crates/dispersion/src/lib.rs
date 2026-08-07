//! Equity statistics over Ohio education agencies.
//!
//! This crate turns [`doctrine/equity`](../../.yidam/corpus/doctrine/equity.yml) from a
//! rhetorical position into a hypothesis about data. Equity claims in school finance are
//! claims about a *distribution*, and the standard measures disagree with each other by
//! design — which is the point.
//!
//! # Why several statistics and not one
//!
//! A reform that raises the bottom improves the McLoone index and may leave the coefficient of
//! variation unchanged. A reform that constrains the top does the reverse. Reporting a single
//! dispersion number lets an advocate pick the one that supports their case, so
//! [`Dispersion::of`] returns all of them together.
//!
//! - **Coefficient of variation** — overall spread relative to the mean.
//! - **McLoone index** — how far the bottom half falls below the median. 1.0 means the bottom
//!   half is at the median; lower is worse.
//! - **Verstegen index** — the mirror: how far the top half rises above the median. 1.0 means
//!   no district exceeds the median; higher means a longer top tail.
//! - **Federal range ratio** — the 95th percentile over the 5th, ignoring the tails entirely.
//!
//! # Wealth neutrality is the sharper test
//!
//! Dispersion alone does not distinguish a system where spending varies with local wealth from
//! one where it varies for any other reason. [`wealth_neutrality`] measures the association
//! directly. A funding system that compensates for local wealth as designed should show state
//! aid falling as valuation per pupil rises.
//!
//! Purity: no network, no filesystem, no clock. Callers supply slices; the fixture-driven
//! tests live in `tests/`.

#![forbid(unsafe_code)]

use edfund_core::Dollars;

/// The equity statistics of one distribution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dispersion {
    /// Number of observations.
    pub n: usize,
    /// Arithmetic mean.
    pub mean: f64,
    /// Median.
    pub median: f64,
    /// Population standard deviation.
    pub std_dev: f64,
    /// Standard deviation over the mean.
    pub coefficient_of_variation: f64,
    /// Mean of the bottom half over the median.
    pub mcloone_index: f64,
    /// Mean of the top half over the median.
    pub verstegen_index: f64,
    /// 95th percentile over the 5th percentile.
    pub federal_range_ratio: f64,
    /// 5th percentile.
    pub p05: f64,
    /// 95th percentile.
    pub p95: f64,
}

/// Why a distribution could not be summarised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispersionError {
    /// Fewer than two observations.
    TooFewObservations,
    /// The mean or median is zero, making a ratio-based index undefined.
    DegenerateDistribution,
    /// Two paired series had different lengths.
    LengthMismatch,
}

impl core::fmt::Display for DispersionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            Self::TooFewObservations => "need at least two observations",
            Self::DegenerateDistribution => "mean or median is zero",
            Self::LengthMismatch => "paired series must have equal length",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for DispersionError {}

/// Linear interpolated percentile of an already-sorted slice.
fn percentile_sorted(sorted: &[f64], q: f64) -> f64 {
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = q * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        sorted[lo] + (rank - lo as f64) * (sorted[hi] - sorted[lo])
    }
}

impl Dispersion {
    /// Summarise a distribution.
    ///
    /// Observations are **not** weighted by enrollment. A per-pupil-weighted variant answers a
    /// different question — how the average *student* is treated rather than how the average
    /// *district* is — and Ohio's small districts are numerous enough that the two diverge.
    /// See [`weighted_mean`].
    ///
    /// # Errors
    ///
    /// Returns [`DispersionError`] for fewer than two observations or a zero mean or median.
    pub fn of(values: &[Dollars]) -> Result<Self, DispersionError> {
        if values.len() < 2 {
            return Err(DispersionError::TooFewObservations);
        }
        let mut sorted: Vec<f64> = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in finance series"));

        let n = sorted.len();
        let mean = sorted.iter().sum::<f64>() / n as f64;
        let median = percentile_sorted(&sorted, 0.5);
        if mean == 0.0 || median == 0.0 {
            return Err(DispersionError::DegenerateDistribution);
        }

        let variance = sorted.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
        let std_dev = variance.sqrt();

        let bottom: Vec<f64> = sorted.iter().copied().filter(|v| *v <= median).collect();
        let top: Vec<f64> = sorted.iter().copied().filter(|v| *v >= median).collect();
        let bottom_mean = bottom.iter().sum::<f64>() / bottom.len() as f64;
        let top_mean = top.iter().sum::<f64>() / top.len() as f64;

        let p05 = percentile_sorted(&sorted, 0.05);
        let p95 = percentile_sorted(&sorted, 0.95);

        Ok(Self {
            n,
            mean,
            median,
            std_dev,
            coefficient_of_variation: std_dev / mean,
            mcloone_index: bottom_mean / median,
            verstegen_index: top_mean / median,
            federal_range_ratio: if p05 == 0.0 { f64::INFINITY } else { p95 / p05 },
            p05,
            p95,
        })
    }
}

/// Enrollment-weighted mean of a per-pupil series.
///
/// Answers "what does the average student's district spend" rather than "what does the average
/// district spend". The gap between this and [`Dispersion::mean`] is a measure of how much
/// Ohio's many small districts pull the unweighted figure around.
///
/// # Errors
///
/// Returns [`DispersionError::LengthMismatch`] if the slices differ in length, or
/// [`DispersionError::DegenerateDistribution`] if total weight is zero.
pub fn weighted_mean(values: &[f64], weights: &[f64]) -> Result<f64, DispersionError> {
    if values.len() != weights.len() {
        return Err(DispersionError::LengthMismatch);
    }
    let total: f64 = weights.iter().sum();
    if total == 0.0 {
        return Err(DispersionError::DegenerateDistribution);
    }
    Ok(values.iter().zip(weights).map(|(v, w)| v * w).sum::<f64>() / total)
}

/// The result of a wealth-neutrality test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WealthNeutrality {
    /// Number of paired observations.
    pub n: usize,
    /// Pearson correlation between the wealth measure and the resource measure.
    pub correlation: f64,
    /// Slope of the least-squares line: resource units per unit of wealth.
    pub slope: f64,
    /// Share of variance in the resource measure explained by wealth.
    pub r_squared: f64,
}

/// Measure how strongly a resource series tracks a wealth series.
///
/// For an equalising system, pass valuation per pupil as `wealth` and state aid per pupil as
/// `resource`; a strong negative correlation is the system compensating. Passing *total*
/// resources instead tests something different and more demanding: whether wealth predicts
/// what a district actually has.
///
/// # Errors
///
/// Returns [`DispersionError`] on mismatched lengths, fewer than two pairs, or a series with
/// no variation.
pub fn wealth_neutrality(
    wealth: &[f64],
    resource: &[f64],
) -> Result<WealthNeutrality, DispersionError> {
    if wealth.len() != resource.len() {
        return Err(DispersionError::LengthMismatch);
    }
    let n = wealth.len();
    if n < 2 {
        return Err(DispersionError::TooFewObservations);
    }
    let mean_w = wealth.iter().sum::<f64>() / n as f64;
    let mean_r = resource.iter().sum::<f64>() / n as f64;

    let cov: f64 = wealth
        .iter()
        .zip(resource)
        .map(|(w, r)| (w - mean_w) * (r - mean_r))
        .sum();
    let var_w: f64 = wealth.iter().map(|w| (w - mean_w).powi(2)).sum();
    let var_r: f64 = resource.iter().map(|r| (r - mean_r).powi(2)).sum();

    if var_w == 0.0 || var_r == 0.0 {
        return Err(DispersionError::DegenerateDistribution);
    }
    let correlation = cov / (var_w * var_r).sqrt();
    Ok(WealthNeutrality {
        n,
        correlation,
        slope: cov / var_w,
        r_squared: correlation * correlation,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_perfectly_equal_distribution_has_zero_dispersion() {
        let d = Dispersion::of(&[100.0; 10]).unwrap();
        assert!((d.coefficient_of_variation - 0.0).abs() < 1e-12);
        assert!((d.mcloone_index - 1.0).abs() < 1e-12);
        assert!((d.verstegen_index - 1.0).abs() < 1e-12);
        assert!((d.federal_range_ratio - 1.0).abs() < 1e-12);
    }

    /// The indices are mirrors: raising the bottom moves McLoone, raising the top moves
    /// Verstegen, and a single summary statistic would hide which one a reform did.
    #[test]
    fn mcloone_and_verstegen_respond_to_opposite_ends() {
        let base = [80.0, 90.0, 100.0, 110.0, 120.0];
        let bottom_raised = [95.0, 98.0, 100.0, 110.0, 120.0];
        let top_raised = [80.0, 90.0, 100.0, 140.0, 180.0];

        let b = Dispersion::of(&base).unwrap();
        let lifted = Dispersion::of(&bottom_raised).unwrap();
        let stretched = Dispersion::of(&top_raised).unwrap();

        assert!(lifted.mcloone_index > b.mcloone_index);
        assert!((lifted.verstegen_index - b.verstegen_index).abs() < 1e-12);

        assert!(stretched.verstegen_index > b.verstegen_index);
        assert!((stretched.mcloone_index - b.mcloone_index).abs() < 1e-12);
    }

    /// Levelling down satisfies equity and fails adequacy — the tension the corpus records
    /// between the two doctrines, as an assertion.
    #[test]
    fn levelling_down_improves_every_dispersion_measure() {
        let unequal = [80.0, 100.0, 140.0, 200.0];
        let levelled = [80.0, 80.0, 80.0, 80.0];
        let u = Dispersion::of(&unequal).unwrap();
        let l = Dispersion::of(&levelled).unwrap();
        assert!(l.coefficient_of_variation < u.coefficient_of_variation);
        assert!(l.federal_range_ratio < u.federal_range_ratio);
        assert!(l.mean < u.mean, "and it makes everyone worse off");
    }

    #[test]
    fn federal_range_ratio_ignores_the_extreme_tails() {
        let mut with_outlier = vec![100.0; 100];
        with_outlier[0] = 1.0;
        with_outlier[99] = 10_000.0;
        let d = Dispersion::of(&with_outlier).unwrap();
        assert!(
            (d.federal_range_ratio - 1.0).abs() < 0.01,
            "p05/p95 should be unmoved by single extreme values, got {}",
            d.federal_range_ratio
        );
        assert!(
            d.coefficient_of_variation > 0.5,
            "but CV should notice them"
        );
    }

    #[test]
    fn weighted_mean_differs_from_unweighted_when_small_units_are_numerous() {
        // Many small high-spending districts, one large low-spending one.
        let spending = [20_000.0, 20_000.0, 20_000.0, 10_000.0];
        let enrollment = [200.0, 200.0, 200.0, 30_000.0];
        let unweighted = Dispersion::of(&spending).unwrap().mean;
        let weighted = weighted_mean(&spending, &enrollment).unwrap();
        assert!(unweighted > 17_000.0);
        assert!(
            weighted < 11_000.0,
            "the average student is not the average district"
        );
    }

    #[test]
    fn wealth_neutrality_detects_perfect_compensation() {
        let wealth = [100.0, 200.0, 300.0, 400.0];
        let aid = [400.0, 300.0, 200.0, 100.0];
        let w = wealth_neutrality(&wealth, &aid).unwrap();
        assert!((w.correlation - -1.0).abs() < 1e-12);
        assert!((w.r_squared - 1.0).abs() < 1e-12);
        assert!(w.slope < 0.0);
    }

    #[test]
    fn wealth_neutrality_detects_no_relationship() {
        let wealth = [100.0, 200.0, 300.0, 400.0];
        let aid = [250.0, 250.5, 249.5, 250.0];
        let w = wealth_neutrality(&wealth, &aid).unwrap();
        assert!(w.correlation.abs() < 0.5);
    }

    #[test]
    fn rejects_degenerate_and_mismatched_inputs() {
        assert_eq!(
            Dispersion::of(&[1.0]),
            Err(DispersionError::TooFewObservations)
        );
        assert_eq!(
            Dispersion::of(&[0.0, 0.0]),
            Err(DispersionError::DegenerateDistribution)
        );
        assert_eq!(
            wealth_neutrality(&[1.0, 2.0], &[1.0]),
            Err(DispersionError::LengthMismatch)
        );
        assert_eq!(
            wealth_neutrality(&[1.0, 1.0], &[1.0, 2.0]),
            Err(DispersionError::DegenerateDistribution)
        );
    }

    #[test]
    fn percentiles_interpolate_between_observations() {
        let sorted = [0.0, 10.0];
        assert!((percentile_sorted(&sorted, 0.5) - 5.0).abs() < 1e-12);
        assert!((percentile_sorted(&sorted, 0.0) - 0.0).abs() < 1e-12);
        assert!((percentile_sorted(&sorted, 1.0) - 10.0).abs() < 1e-12);
    }
}
