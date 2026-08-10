//! Equity statistics over Ohio education agencies.
//!
//! This crate turns [`doctrine/equity`](../../../.yidam/corpus/doctrine/equity.yml) from a
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

pub mod national_peers;
pub mod ohio_panel;

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

/// Fractional ranks with ties averaged, as Spearman's coefficient requires.
fn ranks(values: &[f64]) -> Vec<f64> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|a, b| {
        values[*a]
            .partial_cmp(&values[*b])
            .expect("no NaN in a series")
    });

    let mut out = vec![0.0; values.len()];
    let mut i = 0;
    while i < order.len() {
        let mut j = i;
        while j + 1 < order.len() && values[order[j + 1]] == values[order[i]] {
            j += 1;
        }
        // Ties share the average of the ranks they span. Assigning them in encounter order
        // instead makes the coefficient depend on input order, which is how a rank statistic
        // silently stops being one.
        #[allow(clippy::cast_precision_loss)]
        let shared = (i + j) as f64 / 2.0 + 1.0;
        for slot in &order[i..=j] {
            out[*slot] = shared;
        }
        i = j + 1;
    }
    out
}

/// Spearman's rank correlation between two paired series.
///
/// Prefer this to Pearson wherever a single extreme district can carry the result — Ohio's
/// spending distribution has an island district four times above the 99th percentile, and the
/// two coefficients disagreeing is itself informative.
///
/// # Errors
///
/// Returns [`DispersionError`] on mismatched lengths, fewer than two pairs, or a series with no
/// variation.
pub fn rank_correlation(a: &[f64], b: &[f64]) -> Result<f64, DispersionError> {
    Ok(wealth_neutrality(&ranks(a), &ranks(b))?.correlation)
}

/// The correlation between two series with a third held constant.
///
/// Takes coefficients rather than series because the three pairings are usually already
/// computed, and because the caller has to have thought about which variable is the control.
///
/// # What this is for, and what it is not
///
/// In this domain the honest use is subtractive: showing how much of an association survives
/// once a known confounder is removed. Ohio's economically disadvantaged share correlates with
/// the Performance Index at −0.85, so almost any district-level correlate of the Index is
/// partly that variable in disguise, and reporting one without this check overstates it.
///
/// It does not establish a causal path. Holding one measured confounder constant is not the
/// same as adjustment, and this crate has no model.
///
/// # Errors
///
/// Returns [`DispersionError::DegenerateDistribution`] if either control correlation is ±1,
/// which leaves nothing to hold constant.
pub fn partial_correlation(
    a_with_b: f64,
    a_with_control: f64,
    b_with_control: f64,
) -> Result<f64, DispersionError> {
    let denominator = ((1.0 - a_with_control.powi(2)) * (1.0 - b_with_control.powi(2))).sqrt();
    if denominator == 0.0 || !denominator.is_finite() {
        return Err(DispersionError::DegenerateDistribution);
    }
    Ok((a_with_b - a_with_control * b_with_control) / denominator)
}

/// A fitted least-squares model.
#[derive(Debug, Clone, PartialEq)]
pub struct Regression {
    /// Number of observations.
    pub n: usize,
    /// Intercept, then one coefficient per predictor in the order given.
    pub coefficients: Vec<f64>,
    /// Standard error of each coefficient, aligned to `coefficients`.
    pub standard_errors: Vec<f64>,
    /// Coefficient over its standard error. Above about 2 in absolute value is conventionally
    /// "detectable"; this crate reports the statistic and takes no view.
    pub t_statistics: Vec<f64>,
    /// Slope coefficients rescaled to standard deviations of predictor per standard deviation
    /// of outcome — one per predictor, with no intercept term.
    ///
    /// These are what make predictors on different scales comparable. A raw coefficient on
    /// dollars-per-pupil and one on a percentage share cannot be read against each other.
    pub standardized: Vec<f64>,
    /// Share of outcome variance the model explains.
    pub r_squared: f64,
    /// The same, penalised for the number of predictors.
    pub adjusted_r_squared: f64,
}

/// Fit an ordinary least-squares model of `outcome` on `predictors`.
///
/// `predictors` is column-major: one inner slice per variable, each the length of `outcome`.
/// An intercept is added automatically.
///
/// # This is a description, not an identification
///
/// Adding controls to a cross-section removes the part of an association that the controls
/// explain. It does not turn the remainder into an effect. Every caution on
/// [`partial_correlation`] applies with more force here, because a model with six predictors
/// looks far more like an answer than a correlation does. Ohio districts differ in what they
/// spend money *on*, and no column in this workspace measures that.
///
/// # Errors
///
/// Returns [`DispersionError::LengthMismatch`] if any predictor differs in length from the
/// outcome, [`DispersionError::TooFewObservations`] if there are not more observations than
/// coefficients, and [`DispersionError::DegenerateDistribution`] if the predictors are
/// collinear or one has no variation.
pub fn least_squares(
    predictors: &[Vec<f64>],
    outcome: &[f64],
) -> Result<Regression, DispersionError> {
    let n = outcome.len();
    let k = predictors.len();
    let p = k + 1;
    if predictors.iter().any(|column| column.len() != n) {
        return Err(DispersionError::LengthMismatch);
    }
    if n <= p {
        return Err(DispersionError::TooFewObservations);
    }

    // Centring the predictors leaves every slope unchanged and greatly improves conditioning:
    // this workspace regresses dollars-per-pupil in the tens of thousands alongside shares
    // between 0 and 100, and the uncentred normal equations lose precision on that spread.
    let means: Vec<f64> = predictors
        .iter()
        .map(|c| c.iter().sum::<f64>() / n as f64)
        .collect();
    let row = |i: usize| -> Vec<f64> {
        let mut r = Vec::with_capacity(p);
        r.push(1.0);
        r.extend((0..k).map(|j| predictors[j][i] - means[j]));
        r
    };

    let mut normal = vec![vec![0.0; p * 2]; p];
    let mut rhs = vec![0.0; p];
    for (i, y) in outcome.iter().enumerate() {
        let x = row(i);
        for a in 0..p {
            rhs[a] += x[a] * y;
            for b in 0..p {
                normal[a][b] += x[a] * x[b];
            }
        }
    }
    for (a, r) in normal.iter_mut().enumerate() {
        r[p + a] = 1.0;
    }

    // Gauss-Jordan with partial pivoting, inverting in place alongside the identity.
    for column in 0..p {
        let pivot = (column..p)
            .max_by(|a, b| {
                normal[*a][column]
                    .abs()
                    .total_cmp(&normal[*b][column].abs())
            })
            .expect("at least one row");
        normal.swap(column, pivot);
        let divisor = normal[column][column];
        if divisor.abs() < 1e-10 {
            return Err(DispersionError::DegenerateDistribution);
        }
        for v in normal[column].iter_mut() {
            *v /= divisor;
        }
        for r in 0..p {
            if r != column && normal[r][column] != 0.0 {
                let factor = normal[r][column];
                for c in 0..(p * 2) {
                    normal[r][c] -= factor * normal[column][c];
                }
            }
        }
    }
    let inverse: Vec<Vec<f64>> = normal.iter().map(|r| r[p..].to_vec()).collect();

    let coefficients: Vec<f64> = (0..p)
        .map(|a| (0..p).map(|b| inverse[a][b] * rhs[b]).sum())
        .collect();

    let mean_outcome = outcome.iter().sum::<f64>() / n as f64;
    let mut residual_sum = 0.0;
    let mut total_sum = 0.0;
    for (i, y) in outcome.iter().enumerate() {
        let x = row(i);
        let fitted: f64 = (0..p).map(|a| coefficients[a] * x[a]).sum();
        residual_sum += (y - fitted).powi(2);
        total_sum += (y - mean_outcome).powi(2);
    }
    if total_sum == 0.0 {
        return Err(DispersionError::DegenerateDistribution);
    }

    let variance = residual_sum / (n - p) as f64;
    let standard_errors: Vec<f64> = (0..p).map(|a| (variance * inverse[a][a]).sqrt()).collect();
    let t_statistics: Vec<f64> = (0..p)
        .map(|a| coefficients[a] / standard_errors[a])
        .collect();

    let deviation = |v: &[f64]| {
        let m = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    };
    let outcome_sd = deviation(outcome);
    let standardized: Vec<f64> = (0..k)
        .map(|j| coefficients[j + 1] * deviation(&predictors[j]) / outcome_sd)
        .collect();

    let r_squared = 1.0 - residual_sum / total_sum;
    Ok(Regression {
        n,
        coefficients,
        standard_errors,
        t_statistics,
        standardized,
        r_squared,
        adjusted_r_squared: 1.0 - (1.0 - r_squared) * (n - 1) as f64 / (n - p) as f64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn least_squares_recovers_a_known_plane() {
        // y = 3 + 2a - 1b exactly; the fit must return it and explain everything.
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = vec![2.0, 1.0, 4.0, 3.0, 6.0, 5.0];
        let y: Vec<f64> = a.iter().zip(&b).map(|(x, z)| 3.0 + 2.0 * x - z).collect();
        let fit = least_squares(&[a, b], &y).unwrap();
        assert!((fit.coefficients[1] - 2.0).abs() < 1e-9);
        assert!((fit.coefficients[2] + 1.0).abs() < 1e-9);
        assert!(fit.r_squared > 1.0 - 1e-9);
    }

    /// With one predictor the standardised coefficient is the correlation, which is the
    /// cheapest available check that the scaling is right.
    #[test]
    fn a_single_predictor_standardises_to_its_correlation() {
        let x = vec![1.0, 3.0, 2.0, 7.0, 5.0, 4.0, 9.0];
        let y = vec![2.0, 2.5, 4.0, 6.0, 5.5, 3.0, 8.0];
        let fit = least_squares(&[x.clone()], &y).unwrap();
        let r = wealth_neutrality(&x, &y).unwrap().correlation;
        assert!((fit.standardized[0] - r).abs() < 1e-9);
        assert!((fit.r_squared - r * r).abs() < 1e-9);
    }

    #[test]
    fn least_squares_rejects_collinear_and_undersized_input() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let doubled: Vec<f64> = x.iter().map(|v| v * 2.0).collect();
        let y = vec![1.0, 3.0, 2.0, 5.0, 4.0];
        assert!(least_squares(&[x.clone(), doubled], &y).is_err());
        assert!(least_squares(&[vec![1.0, 2.0], vec![3.0, 4.0]], &[1.0, 2.0]).is_err());
        assert_eq!(
            least_squares(&[vec![1.0, 2.0, 3.0]], &[1.0, 2.0]),
            Err(DispersionError::LengthMismatch)
        );
    }

    #[test]
    fn rank_correlation_is_one_for_any_monotone_transform() {
        // The whole point of the rank coefficient: a series and its cube are the same ranking,
        // where Pearson would report a weaker association.
        let x: [f64; 6] = [1.0, 2.0, 3.0, 4.0, 5.0, 9.0];
        let cubed: Vec<f64> = x.iter().map(|v| v.powi(3)).collect();
        assert!((rank_correlation(&x, &cubed).unwrap() - 1.0).abs() < 1e-12);
        assert!(wealth_neutrality(&x, &cubed).unwrap().correlation < 0.98);
    }

    #[test]
    fn tied_values_share_an_averaged_rank() {
        // Three-way tie across ranks 2, 3 and 4 averages to 3 for all of them.
        assert_eq!(
            ranks(&[10.0, 20.0, 20.0, 20.0, 30.0]),
            vec![1.0, 3.0, 3.0, 3.0, 5.0]
        );
        // And the coefficient must not depend on the order the ties arrive in.
        let a = [5.0, 5.0, 1.0, 9.0];
        let b = [2.0, 7.0, 1.0, 8.0];
        let mut ra = a;
        ra.swap(0, 1);
        let mut rb = b;
        rb.swap(0, 1);
        let forward = rank_correlation(&a, &b).unwrap();
        let swapped = rank_correlation(&ra, &rb).unwrap();
        assert!((forward - swapped).abs() < 1e-12);
    }

    #[test]
    fn partial_correlation_removes_a_shared_driver() {
        // Two series correlated only through a control fall to zero when it is held constant.
        assert!(partial_correlation(0.25, 0.5, 0.5).unwrap().abs() < 1e-12);
        // An association independent of the control survives untouched.
        assert!((partial_correlation(-0.6, 0.0, 0.0).unwrap() + 0.6).abs() < 1e-12);
        // Nothing left to hold constant.
        assert!(partial_correlation(0.3, 1.0, 0.4).is_err());
    }

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
