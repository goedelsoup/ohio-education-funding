//! The statistics more than one crate takes, with the convention in the name.
//!
//! # Why the medians are two functions and neither is called `median`
//!
//! There were two conventions in this workspace and one name for both. `dispersion::median`
//! delegated to [`percentile_sorted`] — R's type 7, which **averages** the two middle
//! observations of an even sample — while `crates/project` carried six private `upper_middle`
//! helpers taking `values[len / 2]`, the **upper** of the two, and three more modules did the
//! same inline. The docstring on [`percentile_sorted`] had said since #158 that the ad-hoc
//! copies were exposed away so callers would share one definition. They were not; the count
//! reached six, and then a seventh arrived in `crates/figures`.
//!
//! The two disagree on every even-length series, and the corpus binds medians at full `f64`
//! precision. So two figures on one panel could disagree with each other and a reader
//! comparing a `dispersion` statistic against a `project` one had nothing on the page saying
//! which median either meant. The known cost of that: the casino noise floor, where the corpus
//! published the $236m [`median_interpolated`] gives and the test standing behind it computed
//! $252m from the upper-of-two, under an assertion wide enough to hold both.
//!
//! Both conventions are defensible and this module keeps both. What it does not keep is a bare
//! `median`, because the name is the whole of the defect: a caller reaching for "the median"
//! got whichever one the crate it happened to be in had defined, and nothing asked it to
//! choose. Choosing is now the only thing a caller can do.
//!
//! # Why they sort, and why they take an iterator
//!
//! [`median_interpolated`] and [`median_upper_middle`] sort what they are given.
//! `dispersion::median` required a sorted slice and checked nothing, so every one of its eight
//! call sites sorted first and any that forgot would have moved a figure quietly. `f64::total_cmp`
//! rather than `partial_cmp`: the call sites that reached for `partial_cmp` disagreed about what
//! to do with a NaN, some panicking and some silently treating it as equal.
//!
//! They take an `IntoIterator` rather than a slice because almost every caller is a
//! `rows.iter().map(pick)` that had to `.collect()` into a named local only so a slice could be
//! borrowed from it. A slice caller writes `.iter().copied()`, which is the one place the borrow
//! has to be said out loud.
//!
//! [`percentile_sorted`] keeps the sorted-slice contract, because `dispersion::Dispersion::of`
//! takes three percentiles off one sorted vector and sorting it three times would be the only
//! thing that changed.

/// The `q`th percentile of an already-sorted series, by linear interpolation on rank
/// `q * (n - 1)`.
///
/// R's type 7 and Excel's `PERCENTILE`, which is the convention every statistic in
/// `crates/dispersion` is computed on.
///
/// The slice must be sorted ascending; nothing here checks it.
///
/// # Panics
///
/// On an empty slice. A percentile of nothing is not a number, and the callers that can be
/// handed an empty series want [`median_interpolated`], which says so in its return type.
#[must_use]
pub fn percentile_sorted(sorted: &[f64], q: f64) -> f64 {
    assert!(
        !sorted.is_empty(),
        "no observations to take a percentile of"
    );
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

/// The median as [`percentile_sorted`] takes it: the **mean** of the two middle observations of
/// an even-length sample.
///
/// This is Python's `statistics.median`, R's `median` and Excel's `MEDIAN`, so it is the
/// convention a reader checking a figure by hand will reach for. It is also the one every
/// `dispersion` statistic is computed on, which is why `Dispersion::of` reports it alongside
/// P05 and P95 rather than beside a different median.
///
/// `None` rather than `0.0` on an empty series: zero is a plausible dollar figure, a plausible
/// millage and a plausible share, so returning it puts a value where an absence belongs.
#[must_use]
pub fn median_interpolated(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let sorted = ascending(values);
    (!sorted.is_empty()).then(|| percentile_sorted(&sorted, 0.5))
}

/// The median as the **upper** of the two middle observations of an even-length sample.
///
/// `values[len / 2]` after a sort, which is an *observation* rather than an average of two. That
/// is the reason to prefer it: on a sample of dollar figures the interpolated median can be a
/// value no district was paid, and a reader who looks up "the median district" finds one here.
/// It is also the convention every subgroup statistic in `crates/project` was written on, so a
/// caller moving off it moves a bound figure.
///
/// `None` on an empty series, for the reason [`median_interpolated`] gives.
#[must_use]
pub fn median_upper_middle(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let sorted = ascending(values);
    sorted.get(sorted.len() / 2).copied()
}

/// Everything the caller offered, ascending, NaN last.
///
/// `total_cmp` and not `partial_cmp`, so that a NaN sorts rather than deciding at runtime whether
/// this panics or silently treats it as equal to its neighbour. Both of those were live.
fn ascending(values: impl IntoIterator<Item = f64>) -> Vec<f64> {
    let mut sorted: Vec<f64> = values.into_iter().collect();
    sorted.sort_by(f64::total_cmp);
    sorted
}

/// How strongly two paired series move together, as a least-squares fit.
///
/// `dispersion::WealthNeutrality` is this, named for the question it was written to answer;
/// the arithmetic is here because four other modules had each written it again.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Association {
    /// Number of paired observations.
    pub n: usize,
    /// Pearson correlation between the two series.
    pub correlation: f64,
    /// Slope of the least-squares line: `ys` units per unit of `xs`.
    pub slope: f64,
    /// Share of variance in `ys` explained by `xs`.
    pub r_squared: f64,
}

/// The least-squares association between two equal-length series, or `None` where there is not
/// one to take.
///
/// `None` on mismatched lengths, on fewer than two pairs, and on a series with **no variation**
/// — three cases the private copies handled four different ways. Two returned `f64::NAN` and
/// propagated it into a figure, one returned `0.0` for a constant series, which reads as "no
/// association" where the truth is "not defined", and one asserted. A constant series is the
/// case that actually occurs: a panel column censored at a floor, or a subgroup small enough
/// that every member sits on the same value.
#[must_use]
pub fn association(xs: &[f64], ys: &[f64]) -> Option<Association> {
    if xs.len() != ys.len() || xs.len() < 2 {
        return None;
    }
    let n = xs.len();
    let (mean_x, mean_y) = (
        xs.iter().sum::<f64>() / n as f64,
        ys.iter().sum::<f64>() / n as f64,
    );
    let cov: f64 = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum();
    let var_x: f64 = xs.iter().map(|x| (x - mean_x).powi(2)).sum();
    let var_y: f64 = ys.iter().map(|y| (y - mean_y).powi(2)).sum();
    if var_x == 0.0 || var_y == 0.0 {
        return None;
    }
    let correlation = cov / (var_x * var_y).sqrt();
    Some(Association {
        n,
        correlation,
        slope: cov / var_x,
        r_squared: correlation * correlation,
    })
}

/// Pearson correlation of two equal-length series, or `None` where [`association`] has none.
#[must_use]
pub fn correlation(xs: &[f64], ys: &[f64]) -> Option<f64> {
    association(xs, ys).map(|a| a.correlation)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two conventions disagree on an even sample and agree on an odd one.
    ///
    /// The point of the whole module, asserted rather than asserted about: on `[1, 2, 3, 4]` one
    /// says 2.5 and the other says 3, and no caller reading "median" could have known which it
    /// was going to get.
    #[test]
    fn the_two_medians_disagree_on_an_even_sample_and_agree_on_an_odd_one() {
        let even = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(median_interpolated(even), Some(2.5));
        assert_eq!(median_upper_middle(even), Some(3.0));

        let odd = [1.0, 2.0, 3.0];
        assert_eq!(median_interpolated(odd), Some(2.0));
        assert_eq!(median_upper_middle(odd), Some(2.0));
    }

    /// Both sort what they are given, so an unsorted caller is not a silent wrong answer.
    #[test]
    fn both_medians_sort_their_input() {
        let scrambled = [4.0, 1.0, 3.0, 2.0];
        assert_eq!(median_interpolated(scrambled), Some(2.5));
        assert_eq!(median_upper_middle(scrambled), Some(3.0));
    }

    /// An absent median reads as absent, not as zero.
    #[test]
    fn an_empty_series_has_no_median() {
        assert_eq!(median_interpolated([]), None);
        assert_eq!(median_upper_middle([]), None);
        assert_eq!(median_interpolated([7.0]), Some(7.0));
        assert_eq!(median_upper_middle([7.0]), Some(7.0));
    }

    /// The percentile ladder, including both ends.
    #[test]
    fn percentiles_interpolate_between_ranks() {
        let sorted: Vec<f64> = (0..=10).map(f64::from).collect();
        assert!((percentile_sorted(&sorted, 0.5) - 5.0).abs() < 1e-12);
        assert!((percentile_sorted(&sorted, 0.0) - 0.0).abs() < 1e-12);
        assert!((percentile_sorted(&sorted, 1.0) - 10.0).abs() < 1e-12);
        assert!((percentile_sorted(&[1.0, 2.0], 0.5) - 1.5).abs() < 1e-12);
    }

    /// A perfect line correlates at one and carries its own slope.
    #[test]
    fn a_perfect_line_is_perfectly_associated() {
        let xs = [1.0, 2.0, 3.0, 4.0];
        let ys = [2.0, 4.0, 6.0, 8.0];
        let a = association(&xs, &ys).expect("four varying pairs");
        assert_eq!(a.n, 4);
        assert!((a.correlation - 1.0).abs() < 1e-12);
        assert!((a.slope - 2.0).abs() < 1e-12);
        assert!((a.r_squared - 1.0).abs() < 1e-12);
    }

    /// The three refusals, which are the reason this returns an `Option`.
    ///
    /// The constant series is the one worth reading: the copies this replaces answered `0.0`
    /// and `NaN` for it, and `0.0` is the answer a reader takes for "measured, and no
    /// relationship" rather than "not defined".
    #[test]
    fn a_constant_series_a_short_one_and_a_mismatch_have_no_association() {
        assert_eq!(correlation(&[1.0, 1.0, 1.0], &[1.0, 2.0, 3.0]), None);
        assert_eq!(correlation(&[1.0, 2.0, 3.0], &[5.0, 5.0, 5.0]), None);
        assert_eq!(correlation(&[1.0], &[2.0]), None);
        assert_eq!(correlation(&[1.0, 2.0], &[2.0]), None);
    }
}
