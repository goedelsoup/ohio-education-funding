//! A fitted partition, for measuring what one would find — not for reasoning from.
//!
//! Issue #396 asked whether a multivariate partition of all 609 districts — clusters of similarly
//! profiled districts on numerous independent variables — would find a blind spot in the formula
//! that every partition built from the guarantee cannot see. The identity in
//! `project::guarantee_origin` was the reason not to build one for the 187 the guarantee pays
//! above the minimum share: the two terms multiply back to the district's own multiple, so
//! nothing is fitted. But that argument does not say what a fitted partition would find across
//! the whole state, and *that* can only be settled by running one.
//!
//! This module is the estimator that was run. It exists so the negative result in
//! `project::lost_pupils` is reproducible, and **no corpus claim reasons from its output** — the
//! claims it supports are of the form "a k-means over these variables does not separate the
//! cluster at any k", which is a statement about what a fit cannot see rather than about what one
//! sees.
//!
//! # What is deliberately plain
//!
//! - **Lloyd's algorithm**, squared Euclidean distance, run to a fixed point. No crates.io
//!   dependency; the workspace has none and this is not the reason to add one.
//! - **No random seed.** [`farthest_first`] seeds are deterministic — the point farthest from the
//!   centroid, then the point farthest from every seed so far — so two runs are the same run. A
//!   caller who wants a different starting point supplies its own centres, and
//!   [`centres_of`] turns any labelling into one, which is how a run can be started from a
//!   baseline such as the department's typology.
//! - **Known-answer tests first.** Three separated blobs are recovered exactly, a six-point line
//!   yields the centres and inertia arithmetic gives, and the helpers each have a case a reader
//!   can check by hand. All in `mod tests` below, none of them Ohio.
//!
//! # What a cluster is not
//!
//! A cluster is not a mechanism. [`majority_ceiling`] is the most any partition can score against
//! a labelling — every cell assigned to whichever label is most common inside it — and it is an
//! upper bound on description, not evidence of cause. [`nearest_neighbours`] is the same
//! question asked without fitting anything: whether a district's nearest neighbour on the profile
//! shares its label more often than chance.

use crate::DispersionError;

/// One run of Lloyd's algorithm at its fixed point.
#[derive(Debug, Clone, PartialEq)]
pub struct Partition {
    /// The cell each row landed in, `0..k`.
    pub assignment: Vec<usize>,
    /// The centre of each cell, in the same coordinates as the rows.
    pub centres: Vec<Vec<f64>>,
    /// Within-cell sum of squared distances, the quantity Lloyd's algorithm descends.
    pub inertia: f64,
    /// How many reassignment passes were run before nothing moved.
    pub iterations: usize,
}

impl Partition {
    /// How many rows each cell holds.
    #[must_use]
    pub fn sizes(&self) -> Vec<usize> {
        let mut sizes = vec![0; self.centres.len()];
        for &cell in &self.assignment {
            sizes[cell] += 1;
        }
        sizes
    }

    /// How many cells hold exactly one row.
    ///
    /// Reported because [`farthest_first`] seeds at the extremes, and an outlier seeded as a
    /// cell tends to stay one: a singleton is a cell the partition has spent on one district.
    #[must_use]
    pub fn singletons(&self) -> usize {
        self.sizes().iter().filter(|&&n| n == 1).count()
    }
}

/// Squared Euclidean distance.
fn squared(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(p, q)| (p - q) * (p - q)).sum()
}

/// Columns z-scored and turned into rows.
///
/// Each column is centred on its mean and divided by its population standard deviation, so every
/// variable weighs the same in a distance regardless of its unit — which is the one choice a
/// distance over dollars, shares and log counts cannot avoid, and is made once here.
///
/// # Errors
///
/// [`DispersionError::LengthMismatch`] if the columns differ in length,
/// [`DispersionError::TooFewObservations`] with fewer than two rows or no columns, and
/// [`DispersionError::DegenerateDistribution`] if a column has no variation.
pub fn standardize(columns: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, DispersionError> {
    let Some(first) = columns.first() else {
        return Err(DispersionError::TooFewObservations);
    };
    let n = first.len();
    if n < 2 {
        return Err(DispersionError::TooFewObservations);
    }
    if columns.iter().any(|c| c.len() != n) {
        return Err(DispersionError::LengthMismatch);
    }
    let mut rows = vec![Vec::with_capacity(columns.len()); n];
    for column in columns {
        let mean = column.iter().sum::<f64>() / n as f64;
        let variance = column.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
        if variance <= 0.0 {
            return Err(DispersionError::DegenerateDistribution);
        }
        let sd = variance.sqrt();
        for (row, value) in rows.iter_mut().zip(column) {
            row.push((value - mean) / sd);
        }
    }
    Ok(rows)
}

/// `k` seed centres with no random number in them.
///
/// The first is the row farthest from the centroid of all rows; each next one is the row farthest
/// from its nearest seed so far. Ties go to the lower index. This is the standard deterministic
/// seeding and it has the standard weakness — it starts at the extremes — which is why
/// [`Partition::singletons`] is reported beside every result.
///
/// # Errors
///
/// [`DispersionError::TooFewObservations`] if `k` is zero or exceeds the number of rows.
pub fn farthest_first(rows: &[Vec<f64>], k: usize) -> Result<Vec<Vec<f64>>, DispersionError> {
    if k == 0 || k > rows.len() {
        return Err(DispersionError::TooFewObservations);
    }
    let width = rows[0].len();
    let centroid: Vec<f64> = (0..width)
        .map(|j| rows.iter().map(|r| r[j]).sum::<f64>() / rows.len() as f64)
        .collect();
    let argmax = |score: &dyn Fn(usize) -> f64| {
        let mut best = 0;
        let mut best_score = f64::MIN;
        for i in 0..rows.len() {
            let s = score(i);
            if s > best_score {
                best = i;
                best_score = s;
            }
        }
        best
    };
    let mut seeds: Vec<Vec<f64>> = vec![rows[argmax(&|i| squared(&rows[i], &centroid))].clone()];
    while seeds.len() < k {
        let next = argmax(&|i| {
            seeds
                .iter()
                .map(|s| squared(&rows[i], s))
                .fold(f64::MAX, f64::min)
        });
        seeds.push(rows[next].clone());
    }
    Ok(seeds)
}

/// The centre of each cell of a labelling — a starting point for [`k_means`] built from any
/// partition a reader already trusts.
///
/// A cell with no members gets the centroid of all rows, so that a labelling with an unused label
/// still yields `k` finite centres.
///
/// # Errors
///
/// [`DispersionError::LengthMismatch`] if the labelling and the rows differ in length, and
/// [`DispersionError::TooFewObservations`] if there are no rows or `k` is zero.
pub fn centres_of(
    rows: &[Vec<f64>],
    assignment: &[usize],
    k: usize,
) -> Result<Vec<Vec<f64>>, DispersionError> {
    if rows.is_empty() || k == 0 {
        return Err(DispersionError::TooFewObservations);
    }
    if rows.len() != assignment.len() {
        return Err(DispersionError::LengthMismatch);
    }
    let width = rows[0].len();
    let mut sums = vec![vec![0.0; width]; k];
    let mut counts = vec![0usize; k];
    for (row, &cell) in rows.iter().zip(assignment) {
        counts[cell] += 1;
        for (s, v) in sums[cell].iter_mut().zip(row) {
            *s += v;
        }
    }
    let centroid: Vec<f64> = (0..width)
        .map(|j| rows.iter().map(|r| r[j]).sum::<f64>() / rows.len() as f64)
        .collect();
    Ok(sums
        .into_iter()
        .zip(counts)
        .map(|(sum, count)| {
            if count == 0 {
                centroid.clone()
            } else {
                sum.into_iter().map(|s| s / count as f64).collect()
            }
        })
        .collect())
}

/// Lloyd's algorithm from the given centres, to a fixed point.
///
/// Each pass assigns every row to its nearest centre (ties to the lower index) and moves every
/// centre to the mean of its rows. A centre that loses all its rows stays where it is rather
/// than being re-seeded, so that the run is a descent on one objective from one start. Stops
/// when a pass moves nothing, or after a thousand passes — which no input in this workspace has
/// come near.
///
/// # Errors
///
/// [`DispersionError::TooFewObservations`] if there are no rows or no centres, and
/// [`DispersionError::LengthMismatch`] if any centre's width differs from the rows'.
pub fn k_means(rows: &[Vec<f64>], centres: &[Vec<f64>]) -> Result<Partition, DispersionError> {
    if rows.is_empty() || centres.is_empty() {
        return Err(DispersionError::TooFewObservations);
    }
    let width = rows[0].len();
    if centres.iter().any(|c| c.len() != width) || rows.iter().any(|r| r.len() != width) {
        return Err(DispersionError::LengthMismatch);
    }
    let k = centres.len();
    let mut centres = centres.to_vec();
    let mut assignment = vec![usize::MAX; rows.len()];
    let nearest = |row: &[f64], centres: &[Vec<f64>]| {
        let mut best = 0;
        let mut best_distance = f64::MAX;
        for (cell, centre) in centres.iter().enumerate() {
            let d = squared(row, centre);
            if d < best_distance {
                best = cell;
                best_distance = d;
            }
        }
        best
    };
    let mut iterations = 0;
    while iterations < 1000 {
        iterations += 1;
        let next: Vec<usize> = rows.iter().map(|r| nearest(r, &centres)).collect();
        if next == assignment {
            break;
        }
        assignment = next;
        let mut sums = vec![vec![0.0; width]; k];
        let mut counts = vec![0usize; k];
        for (row, &cell) in rows.iter().zip(&assignment) {
            counts[cell] += 1;
            for (s, v) in sums[cell].iter_mut().zip(row) {
                *s += v;
            }
        }
        for (cell, count) in counts.iter().enumerate() {
            if *count > 0 {
                centres[cell] = sums[cell].iter().map(|s| s / *count as f64).collect();
            }
        }
    }
    let inertia = rows
        .iter()
        .zip(&assignment)
        .map(|(r, &cell)| squared(r, &centres[cell]))
        .sum();
    Ok(Partition {
        assignment,
        centres,
        inertia,
        iterations,
    })
}

/// The most rows any partition into these cells can label correctly.
///
/// Every cell is assigned to whichever label is most common inside it, and the matches are
/// summed. It is an upper bound on how well the partition *describes* the labelling — a partition
/// of one cell scores the size of the largest label, which is the number to read every other
/// score against — and it says nothing about why.
///
/// # Panics
///
/// If the two slices differ in length; a ceiling over mismatched rows is not a number.
#[must_use]
pub fn majority_ceiling(assignment: &[usize], labels: &[usize]) -> usize {
    assert_eq!(assignment.len(), labels.len(), "one label per row");
    let cells = assignment.iter().copied().max().map_or(0, |c| c + 1);
    let width = labels.iter().copied().max().map_or(0, |l| l + 1);
    let mut table = vec![vec![0usize; width]; cells];
    for (&cell, &label) in assignment.iter().zip(labels) {
        table[cell][label] += 1;
    }
    table
        .iter()
        .map(|row| row.iter().copied().max().unwrap_or(0))
        .sum()
}

/// Each row's nearest other row, by squared distance, ties to the lower index.
///
/// The question a fitted partition asks, with nothing fitted: if a row's neighbours share its
/// label more often than the label's share of the population, the label is a region of the
/// space; if not, no partition of the space will find it.
///
/// # Errors
///
/// [`DispersionError::TooFewObservations`] with fewer than two rows.
pub fn nearest_neighbours(rows: &[Vec<f64>]) -> Result<Vec<usize>, DispersionError> {
    if rows.len() < 2 {
        return Err(DispersionError::TooFewObservations);
    }
    Ok((0..rows.len())
        .map(|i| {
            let mut best = usize::MAX;
            let mut best_distance = f64::MAX;
            for (j, other) in rows.iter().enumerate() {
                if j == i {
                    continue;
                }
                let d = squared(&rows[i], other);
                if d < best_distance {
                    best = j;
                    best_distance = d;
                }
            }
            best
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A linear congruential generator, so the blobs are the same blobs every run and nothing
    /// random enters a known-answer test.
    fn jitter(state: &mut u64) -> f64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        // Top 53 bits as a fraction in [0, 1), then centred on zero at width one.
        ((*state >> 11) as f64 / (1u64 << 53) as f64) - 0.5
    }

    #[test]
    fn three_separated_blobs_are_recovered_exactly() {
        let mut state = 7;
        let offsets = [(0.0, 0.0), (10.0, 0.0), (0.0, 10.0)];
        let mut rows = Vec::new();
        let mut truth = Vec::new();
        for (label, (x, y)) in offsets.iter().enumerate() {
            for _ in 0..20 {
                rows.push(vec![x + jitter(&mut state), y + jitter(&mut state)]);
                truth.push(label);
            }
        }
        let seeds = farthest_first(&rows, 3).unwrap();
        let fit = k_means(&rows, &seeds).unwrap();
        assert_eq!(fit.sizes(), vec![20, 20, 20]);
        assert_eq!(majority_ceiling(&fit.assignment, &truth), 60);
        assert_eq!(fit.singletons(), 0);
        // Every point is within half a unit of its centre on each axis, so the inertia is at most
        // sixty times a half-unit square in two dimensions.
        assert!(fit.inertia < 60.0 * 0.5, "inertia {}", fit.inertia);
        // Started instead from the truth's own centres, the same fixed point.
        let from_truth = centres_of(&rows, &truth, 3).unwrap();
        let again = k_means(&rows, &from_truth).unwrap();
        assert_eq!(majority_ceiling(&again.assignment, &truth), 60);
        assert!((again.inertia - fit.inertia).abs() < 1e-9);
    }

    #[test]
    fn a_six_point_line_gives_the_centres_and_inertia_arithmetic_gives() {
        let rows: Vec<Vec<f64>> = [0.0, 1.0, 2.0, 10.0, 11.0, 12.0]
            .iter()
            .map(|v| vec![*v])
            .collect();
        let seeds = farthest_first(&rows, 2).unwrap();
        // Centroid 6; the farthest point is 0 (distance 36, beating 12's 36 on the tie); the
        // farthest from 0 is 12.
        assert_eq!(seeds, vec![vec![0.0], vec![12.0]]);
        let fit = k_means(&rows, &seeds).unwrap();
        assert_eq!(fit.assignment, vec![0, 0, 0, 1, 1, 1]);
        assert_eq!(fit.centres, vec![vec![1.0], vec![11.0]]);
        // (1 + 0 + 1) + (1 + 0 + 1).
        assert!((fit.inertia - 4.0).abs() < 1e-12);
        assert_eq!(fit.sizes(), vec![3, 3]);
    }

    #[test]
    fn standardizing_centres_and_scales_each_column() {
        let rows = standardize(&[vec![1.0, 2.0, 3.0, 4.0], vec![10.0, 10.0, 10.0, 30.0]]).unwrap();
        assert_eq!(rows.len(), 4);
        for j in 0..2 {
            let mean = rows.iter().map(|r| r[j]).sum::<f64>() / 4.0;
            let var = rows.iter().map(|r| r[j] * r[j]).sum::<f64>() / 4.0;
            assert!(mean.abs() < 1e-12);
            assert!((var - 1.0).abs() < 1e-12);
        }
        // 1 is 1.5 below a mean of 2.5, over a population sd of sqrt(1.25).
        assert!((rows[0][0] + 1.5 / 1.25f64.sqrt()).abs() < 1e-12);
        assert_eq!(
            standardize(&[vec![1.0, 1.0, 1.0]]),
            Err(DispersionError::DegenerateDistribution)
        );
        assert_eq!(
            standardize(&[vec![1.0, 2.0], vec![1.0]]),
            Err(DispersionError::LengthMismatch)
        );
        assert_eq!(standardize(&[]), Err(DispersionError::TooFewObservations));
    }

    #[test]
    fn the_ceiling_assigns_each_cell_to_its_majority() {
        // Cell 0 holds labels {0, 0, 1}: majority 2. Cell 1 holds {1, 1}: 2. Four of five.
        assert_eq!(majority_ceiling(&[0, 0, 0, 1, 1], &[0, 0, 1, 1, 1]), 4);
        // One cell scores the largest label.
        assert_eq!(majority_ceiling(&[0, 0, 0, 0], &[0, 1, 1, 2]), 2);
        // A perfect partition scores everything.
        assert_eq!(majority_ceiling(&[2, 0, 1], &[5, 3, 4]), 3);
    }

    #[test]
    fn nearest_neighbours_pair_the_close_points() {
        let rows: Vec<Vec<f64>> = [0.0, 0.1, 5.0, 5.1].iter().map(|v| vec![*v]).collect();
        assert_eq!(nearest_neighbours(&rows).unwrap(), vec![1, 0, 3, 2]);
        assert_eq!(
            nearest_neighbours(&[vec![1.0]]),
            Err(DispersionError::TooFewObservations)
        );
    }

    #[test]
    fn the_seeding_and_the_run_refuse_what_they_cannot_do() {
        let rows = vec![vec![0.0], vec![1.0]];
        assert_eq!(
            farthest_first(&rows, 3),
            Err(DispersionError::TooFewObservations)
        );
        assert_eq!(
            farthest_first(&rows, 0),
            Err(DispersionError::TooFewObservations)
        );
        assert_eq!(
            k_means(&rows, &[vec![0.0, 0.0]]),
            Err(DispersionError::LengthMismatch)
        );
        assert_eq!(
            k_means(&rows, &[]),
            Err(DispersionError::TooFewObservations)
        );
        assert_eq!(
            centres_of(&rows, &[0], 1),
            Err(DispersionError::LengthMismatch)
        );
        // An unused label gets the centroid rather than a NaN.
        let centres = centres_of(&rows, &[0, 0], 2).unwrap();
        assert_eq!(centres, vec![vec![0.5], vec![0.5]]);
    }
}
