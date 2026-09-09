//! What the Performance Index is made of, and what its star rating is measured against.
//!
//! `corpus/metric/performance-index.yml` recorded that "the exact level weights and the
//! untested-student treatment are set by DEW and are not transcribed here." They are not
//! published in anything this corpus holds, and they are still not transcribed. They are
//! **recovered**: the department publishes each district's index beside the seven achievement-
//! level shares it was computed from, 607 times over, which is an overdetermined system in
//! seven unknowns.
//!
//! The answer is one hundred for a district whose tested students are all proficient, and
//! tenths from there. A student who was not tested counts as a zero — below the lowest scored
//! level, not level with it.
//!
//! The second half of this file is about the *denominator*. R.C. 3302.03(D)(1)(c) awards the
//! achievement component not on the index but on the index over "the average of the highest
//! two per cent of performance index scores achieved by a district for the school year for
//! which a report card is issued." That is refitted every year, so the achievement rating is a
//! share of the top of the state rather than a level, and a district can improve on the index
//! and fall on the rating. 121 of them did between 2022-23 and 2024-25.
//!
//! # These are recoveries, not transcriptions
//!
//! Nothing here reads DEW's technical documentation. What is established is that a particular
//! weight vector reproduces all 607 published scores inside the publication's own rounding, and
//! that least squares over the same 607 lands on it. A vector differing by a few thousandths
//! per level would fit as well; the round numbers are the reading, not the measurement.

use dispersion::least_squares;
use dispersion::report_card::{self, ReportCard};

/// The seven levels, in the department's own order.
///
/// `PROFICIENT` is the reference level below and is deliberately not in the predictor set: the
/// shares sum to one hundred, so all seven together are collinear with the intercept and the
/// normal equations are singular. Omitting one makes the intercept the all-proficient score and
/// each slope that level's distance from proficient — which is both well conditioned and the
/// form the answer wants to be read in.
const NOT_TESTED: usize = 0;
const LIMITED: usize = 1;
const BASIC: usize = 2;
const PROFICIENT: usize = 3;
const ACCOMPLISHED: usize = 4;
const ADVANCED: usize = 5;
const ADVANCED_PLUS: usize = 6;

/// Every level except the reference, and the tenth each sits from proficient.
const AGAINST_PROFICIENT: [(usize, &str, f64); 6] = [
    (NOT_TESTED, "not tested", -1.0),
    (LIMITED, "limited", -0.7),
    (BASIC, "basic", -0.4),
    (ACCOMPLISHED, "accomplished", 0.1),
    (ADVANCED, "advanced", 0.2),
    (ADVANCED_PLUS, "advanced plus", 0.3),
];

/// The recovered weights on their own scale: an all-proficient district scores 100.
const WEIGHTS: [f64; 7] = [0.0, 0.3, 0.6, 1.0, 1.1, 1.2, 1.3];

/// How far a reconstruction may sit from the published score without indicting the weights.
///
/// Each of the seven shares is published to a tenth and rounded on its own — they sum to
/// between 99.8 and 100.2 rather than to 100 — and the score is published to a tenth as well.
/// So the worst honest reconstruction error is half a tenth times the sum of the weights, plus
/// half a tenth for the score itself.
fn rounding_bound() -> f64 {
    0.05 * WEIGHTS.iter().sum::<f64>() + 0.05
}

fn rated() -> Vec<ReportCard> {
    report_card::report_cards()
}

/// Districts carrying all seven shares and a score, which here is all 607 of them.
fn complete(rows: &[ReportCard]) -> Vec<([f64; 7], f64)> {
    rows.iter()
        .filter_map(|r| {
            let mut shares = [0.0; 7];
            for (slot, value) in shares.iter_mut().zip(r.levels) {
                *slot = value?;
            }
            Some((shares, r.performance_index?))
        })
        .collect()
}

/// Least squares of the index on six of the seven shares, with the intercept un-centred.
///
/// [`dispersion::least_squares`] centres its predictors, which leaves every slope alone and
/// makes `coefficients[0]` the mean of the outcome rather than its value at zero. The
/// all-proficient score is the value at zero, so it has to be put back.
fn recover(sample: &[([f64; 7], f64)]) -> (f64, [f64; 6]) {
    let predictors: Vec<Vec<f64>> = AGAINST_PROFICIENT
        .iter()
        .map(|(level, _, _)| sample.iter().map(|(s, _)| s[*level]).collect())
        .collect();
    let outcome: Vec<f64> = sample.iter().map(|(_, y)| *y).collect();
    let fit = least_squares(&predictors, &outcome).expect("607 districts, six predictors");

    let mut slopes = [0.0; 6];
    slopes.copy_from_slice(&fit.coefficients[1..]);
    let intercept = fit.coefficients[0]
        - slopes
            .iter()
            .zip(&predictors)
            .map(|(slope, column)| slope * column.iter().sum::<f64>() / column.len() as f64)
            .sum::<f64>();
    (intercept, slopes)
}

// ---------------------------------------------------------------------------------------
// The weights
// ---------------------------------------------------------------------------------------

/// The index is one hundred for a district whose tested students are all proficient, and every
/// other level is a tenth or a multiple of one from there.
///
/// This is the whole recovery. Six slopes, each within a hundredth of an exact tenth, over 607
/// districts. Nothing was fitted to a target: the levels went in as published shares and the
/// index came out as the outcome.
#[test]
fn the_index_is_one_hundred_when_every_tested_student_is_proficient() {
    let rows = rated();
    let sample = complete(&rows);
    assert_eq!(
        sample.len(),
        607,
        "every rated district carries all seven shares"
    );

    let (intercept, slopes) = recover(&sample);
    assert!(
        (intercept - 100.0).abs() < 0.2,
        "the all-proficient score came out at {intercept:.4}, not 100"
    );
    for ((_, name, expected), got) in AGAINST_PROFICIENT.iter().zip(slopes) {
        assert!(
            (got - expected).abs() < 0.01,
            "{name} recovered at {got:+.5} from proficient, not {expected:+.1}"
        );
    }
}

/// A student who was not tested counts as a zero, which is below the lowest scored level.
///
/// This is the part of the open claim that was about policy rather than arithmetic. Limited —
/// the worst result a tested student can produce — carries 0.3. Not tested carries 0.0. So a
/// district's index falls further for failing to test a student than for testing that student
/// and having them score at the bottom, and the gap is 0.3 index points per percentage point of
/// enrollment.
#[test]
fn an_untested_student_counts_below_the_lowest_scored_level() {
    let (_, slopes) = recover(&complete(&rated()));
    let absolute = |i: usize| 1.0 + slopes[i];

    assert!(
        (WEIGHTS[PROFICIENT] - 1.0).abs() < f64::EPSILON,
        "proficient is the reference level and carries 1.0 by construction"
    );
    assert!(
        absolute(0).abs() < 0.01,
        "not tested recovered at {:.5}, which is not zero",
        absolute(0)
    );
    assert!(
        absolute(1) - absolute(0) > 0.25,
        "limited should sit clear of untested; got {:.4} against {:.4}",
        absolute(1),
        absolute(0)
    );
}

/// The recovered weights reproduce every published score inside the publication's own rounding.
///
/// Least squares finding a vector is one thing; that vector accounting for all 607 districts is
/// the check that matters, and it is the one that would fail if the department weighted
/// anything else — a subgroup, an enrollment term, a prior year.
#[test]
fn the_recovered_weights_reproduce_every_district() {
    let rows = rated();
    let bound = rounding_bound();
    let mut worst: (f64, &str) = (0.0, "");
    for row in &rows {
        let Some(published) = row.performance_index else {
            continue;
        };
        let reconstructed: f64 = row
            .levels
            .iter()
            .zip(WEIGHTS)
            .map(|(share, weight)| share.expect("all seven shares") * weight)
            .sum();
        let gap = (reconstructed - published).abs();
        if gap > worst.0 {
            worst = (gap, &row.name);
        }
        assert!(
            gap <= bound,
            "{}: published {published}, reconstructed {reconstructed:.4}, \
             outside the {bound:.4} rounding bound",
            row.name
        );
    }
    // Comfortably inside it, which is what says the residual is rounding and not a missing term.
    assert!(
        worst.0 < 0.75 * bound,
        "worst reconstruction is {} at {:.4}, close enough to the {bound:.4} bound to be worth \
         rechecking the weights",
        worst.1,
        worst.0
    );
}

// ---------------------------------------------------------------------------------------
// The denominator
// ---------------------------------------------------------------------------------------

/// The denominator is one number for the whole state, and it is the mean of the top thirteen.
///
/// R.C. 3302.03(D)(1)(c) says "the highest two per cent." Two per cent of 607 is 12.14, and the
/// statute does not say which way to round it. The published 109.8 does: the mean of the top
/// thirteen is 109.81, the top twelve give 109.94 and the top fourteen 109.69. Only the ceiling
/// reproduces the file.
#[test]
fn the_achievement_denominator_is_the_mean_of_the_highest_thirteen_districts() {
    let rows = rated();
    let published: Vec<f64> = rows
        .iter()
        .filter_map(|r| r.performance_index_maximum)
        .collect();
    assert_eq!(published.len(), 607);
    assert!(
        published.iter().all(|v| (v - 109.8).abs() < f64::EPSILON),
        "the maximum is supposed to be one statewide number and is not"
    );

    let mut scores: Vec<f64> = rows.iter().filter_map(|r| r.performance_index).collect();
    scores.sort_by(|a, b| b.total_cmp(a));
    let top = |k: usize| scores[..k].iter().sum::<f64>() / k as f64;

    let two_percent = scores.len() * 2;
    let ceiling = two_percent.div_ceil(100);
    assert_eq!(ceiling, 13, "ceil(2% of {}) should be 13", scores.len());

    assert!(
        (top(13) - 109.8).abs() < 0.05,
        "the top thirteen average {:.4}, which does not round to the published 109.8",
        top(13)
    );
    for k in [12, 14] {
        assert!(
            (top(k) - 109.8).abs() > 0.05,
            "the top {k} average {:.4}, which also rounds to 109.8 — the ceiling is no longer \
             what picks the count",
            top(k)
        );
    }
}

/// The published percentage is the index over that denominator, for all 607.
///
/// It does not look like it on a third of them: dividing the published index by 109.8 disagrees
/// with the published percentage by a tenth on 131 districts. That is double rounding, not
/// disagreement. The department divides the unrounded index and rounds once; the corpus sees
/// the index already rounded to a tenth and rounds again. The test is therefore not whether the
/// two agree but whether some single unrounded index is consistent with both, and one is —
/// every time.
#[test]
fn the_published_percentage_is_the_index_over_that_denominator() {
    let rows = rated();
    let mut checked = 0;
    let mut naive_disagreements = 0;
    for row in &rows {
        let (Some(index), Some(percent), Some(maximum)) = (
            row.performance_index,
            row.performance_index_percent,
            row.performance_index_maximum,
        ) else {
            continue;
        };
        checked += 1;
        if ((index / maximum * 100.0 * 10.0).round() / 10.0 - percent).abs() > 1e-9 {
            naive_disagreements += 1;
        }
        // The interval of unrounded indices that round to the published index, against the
        // interval that produces the published percentage. They have to overlap.
        let (low, high) = (index - 0.05, index + 0.05);
        let (from_percent_low, from_percent_high) = (
            (percent - 0.05) * maximum / 100.0,
            (percent + 0.05) * maximum / 100.0,
        );
        assert!(
            low.max(from_percent_low) < high.min(from_percent_high),
            "{}: no index rounds to both {index} and {percent}%",
            row.name
        );
    }
    assert_eq!(checked, 607);
    assert_eq!(
        naive_disagreements, 131,
        "the number of districts where re-rounding the published index disagrees with the \
         published percentage by a tenth"
    );
}

/// A district can rise on the index and fall on the measure the rating is awarded on.
///
/// The denominator rose from 108.80 in 2022-23 to 109.80 in 2024-25 — a point, against a median
/// district's whole three-year range of 2.1 points, so the annual refitting is about half the
/// size of a typical district's own movement. 121 of the 607 districts improved on the raw
/// index over that window and declined as a share of the maximum. That is one district in five.
///
/// The 2022-23 denominator is computed here the way the 2024-25 one was verified above, because
/// the department publishes the maximum only for the current year. The 2024-25 side uses the
/// *published* 109.8 rather than the recomputed 109.8077 — a seventh of a per mille apart, which
/// sounds like nothing and is not, because the nearest district sits five millionths from the
/// boundary. `crates/figures` makes the same choice for the same reason.
///
/// # The count is exact and its edge is soft
///
/// 121 is arithmetic and reproduces every time, but it is not a partition of the state into two
/// kinds of district. 28 of the 425 risers change share by less than a thousandth in either
/// direction, and the nearest changes by 0.000005. The finding that survives is the direction and
/// not the cut: the denominator rose 0.919%, so *every* district's share fell by that much
/// relative to its own index, and 121 is the subset whose gain did not cover it.
#[test]
fn a_fifth_of_districts_rose_on_the_index_and_fell_on_the_share() {
    let rows = rated();
    let mut earliest: Vec<f64> = rows
        .iter()
        .filter_map(|r| r.performance_index_earliest)
        .collect();
    earliest.sort_by(|a, b| b.total_cmp(a));
    let then = earliest[..13].iter().sum::<f64>() / 13.0;
    let now = 109.8;
    assert!(
        (then - 108.8).abs() < 0.05,
        "the 2022-23 denominator came out at {then:.4}"
    );

    let (mut both, mut rose_on_the_index, mut fell_on_the_share) = (0, 0, 0);
    for row in &rows {
        let (Some(before), Some(after)) = (row.performance_index_earliest, row.performance_index)
        else {
            continue;
        };
        both += 1;
        if after > before {
            rose_on_the_index += 1;
            if after / now <= before / then {
                fell_on_the_share += 1;
            }
        }
    }
    assert_eq!(both, 607, "districts carrying both years");
    assert_eq!(rose_on_the_index, 425);
    assert_eq!(
        fell_on_the_share, 121,
        "districts that improved absolutely and declined relatively"
    );

    // How much of the state sits close enough to the cut that the cut is not the finding.
    let mut margins: Vec<f64> = rows
        .iter()
        .filter_map(|row| {
            let (before, after) = (row.performance_index_earliest?, row.performance_index?);
            (after > before).then_some(after / now - before / then)
        })
        .collect();
    margins.sort_by(|a, b| a.abs().total_cmp(&b.abs()));
    assert!(
        margins[0].abs() < 1e-4,
        "the nearest riser is {:.6} from the boundary, which is far enough that this file's \
         warning about a soft edge has stopped being true",
        margins[0]
    );
    assert_eq!(
        margins.iter().filter(|m| m.abs() < 1e-3).count(),
        28,
        "risers within a thousandth of the boundary"
    );

    // And the part that does not depend on where the cut falls.
    let drift = now / then - 1.0;
    assert!(
        (drift - 0.00919).abs() < 0.00005,
        "the denominator rose {drift:.5}, not 0.00919"
    );
}

/// The statute constrains the distribution of ratings rather than the cutpoints, and the
/// achievement component obeys it.
///
/// R.C. 3302.03(D)(4) puts the star thresholds in administrative rule, not in the section — so
/// "how the star thresholds are set" has no answer in statute and is not supposed to. What the
/// section does set is a shape: division (D)(4)(b) requires that, outside one named exception,
/// "more than half of all districts or buildings do not earn the same performance rating in any
/// component or overall." The modal achievement rating is four stars, at 227 of 607.
///
/// It is a weak constraint and the file shows why. Three districts in the state hold one star
/// and 91 hold five; nothing in the section stops that, and nothing in it requires the scale to
/// be centred.
#[test]
fn no_achievement_rating_is_held_by_half_the_state() {
    let rows = rated();
    let mut counts = [0usize; 6];
    for row in &rows {
        let stars = row.achievement_stars.expect("every rated district");
        assert!(
            (stars - stars.round()).abs() < f64::EPSILON,
            "component ratings are whole stars; got {stars}"
        );
        counts[stars as usize] += 1;
    }
    assert_eq!(counts, [0, 3, 76, 210, 227, 91]);

    let rated_count: usize = counts.iter().sum();
    assert_eq!(rated_count, 607);
    let modal = *counts.iter().max().expect("six buckets");
    assert!(
        modal * 2 <= rated_count,
        "{modal} of {rated_count} districts share a rating, which division (D)(4)(b) forbids"
    );
}
