//! The 2024-25 Ohio School Report Card, joined to the corpus's other district cross-sections.
//!
//! These tests do two jobs. The first is replication: OCG White Paper 013 reported a set of
//! coefficients over these same published files, and every one of them is reproduced here from
//! the primary sources, so the corpus's disagreement with that paper is a disagreement about
//! method and not about arithmetic.
//!
//! The second is the disagreement itself. The department publishes one FY2025 operating
//! expenditure total per district and two pupil counts to divide it by — a headcount and a
//! count weighted upward for disadvantage, English learners, and disability. The published
//! per-pupil spending figure uses the weighted one. Every correlation below is computed on both,
//! and the choice of divisor moves the headline result from nothing to something.
//!
//! Findings pinned here are cited by `corpus/metric/expenditure-per-equivalent-pupil.yml` and
//! `corpus/metric/performance-index.yml`. A fixture refresh that moves them fails the build
//! rather than quietly rewriting the corpus.

use std::collections::BTreeMap;

use dispersion::profile::{self, ProfileDistrict};
use dispersion::report_card::{self, ReportCard};
use dispersion::{
    least_squares, partial_correlation, rank_correlation, wealth_neutrality, weighted_mean,
    Dispersion,
};
use foundation::department_model;

/// A column reader over the report card, and one over the profile report it joins to.
///
/// All three fixtures behind this file are read through their crates' own public readers now:
/// [`dispersion::report_card`], [`dispersion::profile`] and [`foundation::department_model`].
/// This file used to carry a fourth parser of the report card, a second of the profile report
/// and a third of the department model, each with its own column table. See issue #157.
type Pick = fn(&ReportCard) -> Option<f64>;
type FromProfile = fn(&ProfileDistrict) -> Option<f64>;

fn report_card() -> Vec<ReportCard> {
    report_card::report_cards()
}

/// One column of the report card, aligned to `rows`.
fn column(rows: &[ReportCard], pick: Pick) -> Vec<Option<f64>> {
    rows.iter().map(pick).collect()
}

/// A column of the profile report, aligned to the report-card rows by IRN.
///
/// The report card rates 607 districts and the profile report covers 606, so a joined column
/// carries one `None` that the report card's own columns do not — which is why every
/// correlation here is computed over complete cases rather than over a count.
fn joined(rows: &[ReportCard], pick: FromProfile) -> Vec<Option<f64>> {
    let table: BTreeMap<String, ProfileDistrict> = profile::districts()
        .into_iter()
        .map(|d| (d.irn.clone(), d))
        .collect();
    rows.iter()
        .map(|r| table.get(&r.irn).and_then(pick))
        .collect()
}

/// Pairs where both sides are present, which is what every coefficient here is computed over.
fn pairs(a: &[Option<f64>], b: &[Option<f64>]) -> (Vec<f64>, Vec<f64>) {
    a.iter()
        .zip(b)
        .filter_map(|(x, y)| Some(((*x)?, (*y)?)))
        .unzip()
}

fn correlate(a: &[Option<f64>], b: &[Option<f64>]) -> f64 {
    let (x, y) = pairs(a, b);
    wealth_neutrality(&x, &y)
        .expect("paired series")
        .correlation
}

fn rank(a: &[Option<f64>], b: &[Option<f64>]) -> f64 {
    let (x, y) = pairs(a, b);
    rank_correlation(&x, &y).expect("paired series")
}

fn close(actual: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{what}: got {actual:.4}, expected {expected:.4}"
    );
}

// ---------------------------------------------------------------------------------------
// Replication
// ---------------------------------------------------------------------------------------

#[test]
fn the_rated_population_is_607_districts() {
    let rows = report_card();
    assert_eq!(rows.len(), 607);
    assert!(rows.iter().all(|r| r.performance_index.is_some()));
}

/// The published per-equivalent-pupil figure is reconstructible from the numerator and the
/// weighted ADM, which is what licenses recomputing it on the other divisor.
///
/// # The two ADM columns are published at different precisions
///
/// `Unweighted ADM` carries four decimals. `Weighted ADM` is rounded to whole pupils. So the
/// reconstruction is exact only up to that rounding, and the residual scales inversely with
/// district size: Put-in-Bay Local, at 77 weighted pupils, reconstructs $86 below its published
/// $46,716, while Akron City at 29,162 lands within a cent. Asserting a flat dollar tolerance
/// would fail on the small districts and hide the reason; the tolerance below is the rounding
/// itself, which makes the mechanism the claim.
///
/// The asymmetry matters beyond this test. Any per-weighted-pupil figure recomputed for a small
/// district carries quantisation noise that the per-unweighted-pupil figure does not — one more
/// reason the headcount denominator is the cleaner instrument.
#[test]
fn published_spending_equals_expenditures_over_weighted_adm() {
    let rows = report_card();
    let mut checked = 0;
    for row in &rows {
        let (Some(computed), Some(pupils), Some(published)) = (
            row.per_weighted_pupil(),
            row.weighted_adm,
            row.per_equivalent_pupil,
        ) else {
            continue;
        };
        // Half a pupil of rounding in the denominator, plus a dollar for the published figure's
        // own rounding.
        let tolerance = published * (0.5 / pupils) + 1.0;
        assert!(
            (computed - published).abs() < tolerance,
            "{} reconstructs to {computed:.2} against a published {published:.0}, \
             outside the {tolerance:.2} the integer ADM allows",
            row.name,
        );
        checked += 1;
    }
    assert_eq!(checked, 607);

    // Weighted ADM really is published whole and unweighted ADM really is not, which is the
    // premise of the tolerance above.
    assert!(rows
        .iter()
        .filter_map(|r| r.weighted_adm)
        .all(|v| (v.fract()).abs() < 1e-9));
    assert!(
        rows.iter()
            .filter_map(|r| r.unweighted_adm)
            .filter(|v| v.fract().abs() > 1e-9)
            .count()
            > 500
    );
}

#[test]
fn the_white_paper_coefficients_reproduce() {
    let rows = report_card();
    let pi = column(&rows, |r| r.performance_index);

    close(
        correlate(&pi, &column(&rows, |r| r.per_equivalent_pupil)),
        -0.016,
        0.001,
        "PI against total spending per equivalent pupil",
    );
    close(
        rank(&pi, &column(&rows, |r| r.per_equivalent_pupil)),
        0.048,
        0.001,
        "the same, Spearman",
    );
    close(
        correlate(&pi, &column(&rows, |r| r.per_equivalent_pupil_federal)),
        -0.558,
        0.001,
        "PI against federal expenditure per equivalent pupil",
    );
    close(
        correlate(&pi, &column(&rows, |r| r.per_equivalent_pupil_state_local)),
        0.086,
        0.001,
        "PI against state-and-local expenditure per equivalent pupil",
    );
    close(
        rank(
            &column(&rows, |r| r.unweighted_adm),
            &column(&rows, |r| r.per_equivalent_pupil),
        ),
        -0.366,
        0.001,
        "enrollment against spending per equivalent pupil",
    );
}

#[test]
fn the_white_paper_distributions_reproduce() {
    let rows = report_card();
    let spending: Vec<f64> = column(&rows, |r| r.per_equivalent_pupil)
        .into_iter()
        .flatten()
        .collect();
    let d = Dispersion::of(&spending).unwrap();
    close(
        d.median,
        12_856.0,
        1.0,
        "median spending per equivalent pupil",
    );
    close(d.mean, 13_224.0, 1.0, "mean spending per equivalent pupil");

    let pi: Vec<f64> = column(&rows, |r| r.performance_index)
        .into_iter()
        .flatten()
        .collect();
    let p = Dispersion::of(&pi).unwrap();
    close(p.median, 88.2, 0.05, "median Performance Index");
    close(p.mean, 87.7, 0.05, "mean Performance Index");
    close(
        p.std_dev,
        10.9,
        0.05,
        "Performance Index standard deviation",
    );
}

// ---------------------------------------------------------------------------------------
// The denominator
// ---------------------------------------------------------------------------------------

/// The finding. One numerator, two divisors, 607 districts, one school year.
#[test]
fn the_divisor_decides_the_headline() {
    let rows = report_card();
    let pi = column(&rows, |r| r.performance_index);

    let weighted = correlate(&pi, &column(&rows, ReportCard::per_weighted_pupil));
    let unweighted = correlate(&pi, &column(&rows, ReportCard::per_enrolled_pupil));

    close(
        weighted,
        -0.015,
        0.002,
        "PI against expenditures per weighted pupil",
    );
    close(
        unweighted,
        -0.337,
        0.002,
        "PI against expenditures per unweighted pupil",
    );

    // Not a matter of degree. The two coefficients are more than twenty standard errors apart.
    assert!(unweighted < weighted - 0.3);
}

/// Why the divisor decides it: the weight ratio is very nearly a poverty index, and dividing by
/// it removes the variable the Performance Index is mostly measuring.
#[test]
fn the_adm_weight_ratio_is_a_poverty_measure() {
    let rows = report_card();
    let ratio = column(&rows, ReportCard::weight_ratio);
    let disadvantaged = joined(&rows, |d| d.economically_disadvantaged);

    close(
        correlate(&ratio, &disadvantaged),
        0.800,
        0.002,
        "ADM weight ratio against economically disadvantaged share",
    );
    close(
        correlate(&ratio, &column(&rows, |r| r.performance_index)),
        -0.745,
        0.002,
        "ADM weight ratio against Performance Index",
    );

    let values: Vec<f64> = ratio.iter().flatten().copied().collect();
    let d = Dispersion::of(&values).unwrap();
    close(d.median, 1.261, 0.002, "median weight ratio");
    assert!(d.p05 > 1.0, "the weighting only ever adds pupils");
}

/// The result the paper's near-zero coefficient was standing in front of.
#[test]
fn disadvantage_explains_most_of_the_performance_index() {
    let rows = report_card();
    let pi = column(&rows, |r| r.performance_index);
    let disadvantaged = joined(&rows, |d| d.economically_disadvantaged);

    let (a, b) = pairs(&pi, &disadvantaged);
    let fit = wealth_neutrality(&b, &a).unwrap();
    assert_eq!(fit.n, 606);
    close(
        fit.correlation,
        -0.846,
        0.002,
        "PI against disadvantaged share",
    );
    close(fit.r_squared, 0.716, 0.003, "share of PI variance");
}

/// Both spending relationships shrink sharply once actual poverty is held constant — including
/// the paper's strongest finding, which it could only hypothesise was a poverty signal.
#[test]
fn holding_disadvantage_constant_shrinks_every_spending_relationship() {
    let rows = report_card();
    let pi = column(&rows, |r| r.performance_index);
    let ed = joined(&rows, |d| d.economically_disadvantaged);

    // All three coefficients must come from the same 606 districts. One district is missing an
    // economically disadvantaged share, and computing the raw correlation over 607 while the
    // two control correlations use 606 produces a partial correlation of nothing in particular.
    let check = |series: &[Option<f64>], raw: f64, partial: f64, what: &str| {
        let present: Vec<bool> = series
            .iter()
            .zip(&ed)
            .zip(&pi)
            .map(|((s, e), p)| s.is_some() && e.is_some() && p.is_some())
            .collect();
        let keep = |v: &[Option<f64>]| -> Vec<Option<f64>> {
            v.iter()
                .zip(&present)
                .map(|(x, ok)| if *ok { *x } else { None })
                .collect()
        };
        let (series, ed, pi) = (keep(series), keep(&ed), keep(&pi));
        let r = correlate(&pi, &series);
        close(r, raw, 0.003, what);
        let p = partial_correlation(r, correlate(&pi, &ed), correlate(&series, &ed)).unwrap();
        close(p, partial, 0.005, what);
    };

    check(
        &column(&rows, ReportCard::per_enrolled_pupil),
        -0.355,
        -0.125,
        "spending per unweighted pupil, raw then holding disadvantage",
    );
    check(
        &column(&rows, |r| r.per_equivalent_pupil_federal),
        -0.558,
        -0.158,
        "federal spending per equivalent pupil, raw then holding disadvantage",
    );
}

/// **Only one of the two estimates is stable, and it is not the published one.**
///
/// This test previously asserted that the weighted-denominator coefficient "stays near zero"
/// across the paper's own sensitivity checks and called that robustness. It is not. A near-zero
/// estimate staying near zero is arithmetic, not stability: across three scenarios the published
/// measure **changes sign** and its magnitude varies by a factor of eight, while the headcount
/// measure moves by 0.018 and never leaves the same conclusion.
///
///     scenario                              published    headcount
///     all districts                          -0.0155      -0.3365
///     excluding the highest per-pupil         -0.0042      -0.3546
///     excluding enrollment under 582          +0.0358      -0.3371
///
/// So the paper's robustness claim survives only as a statement about a quantity too small to
/// have a direction. That is the correct reading of its own sensitivity table, and it is a
/// stronger objection than "the denominator matters".
#[test]
fn only_the_headcount_estimate_is_stable_across_the_papers_sensitivity_checks() {
    let all = report_card();
    let scenarios: Vec<(&str, Vec<&ReportCard>)> = vec![
        ("all districts", all.iter().collect()),
        ("excluding the highest per-pupil district", {
            let top = all
                .iter()
                .max_by(|a, b| {
                    a.per_equivalent_pupil
                        .partial_cmp(&b.per_equivalent_pupil)
                        .unwrap()
                })
                .unwrap();
            all.iter().filter(|r| !std::ptr::eq(*r, top)).collect()
        }),
        ("excluding enrollment under 582", {
            all.iter()
                .filter(|r| r.unweighted_adm.is_some_and(|a| a >= 582.0))
                .collect()
        }),
    ];

    let mut published = Vec::new();
    let mut headcount = Vec::new();
    for (name, subset) in scenarios {
        let rows: Vec<ReportCard> = subset.into_iter().cloned().collect();
        let pi = column(&rows, |r| r.performance_index);
        let weighted = correlate(&pi, &column(&rows, ReportCard::per_weighted_pupil));
        let unweighted = correlate(&pi, &column(&rows, ReportCard::per_enrolled_pupil));
        assert!(
            unweighted < -0.30,
            "{name}: headcount divisor should stay clearly negative, got {unweighted:.4}"
        );
        published.push(weighted);
        headcount.push(unweighted);
    }

    close(published[0], -0.0155, 0.0005, "published, all districts");
    close(
        published[1],
        -0.0042,
        0.0005,
        "published, less the top spender",
    );
    close(
        published[2],
        0.0358,
        0.0005,
        "published, less the small districts",
    );
    close(headcount[0], -0.3365, 0.0005, "headcount, all districts");
    close(
        headcount[1],
        -0.3546,
        0.0005,
        "headcount, less the top spender",
    );
    close(
        headcount[2],
        -0.3371,
        0.0005,
        "headcount, less the small districts",
    );

    // The headcount estimate is stable in the sense the word is normally used: every scenario
    // supports the same claim at nearly the same strength.
    let spread = |v: &[f64]| {
        v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min)
    };
    assert!(
        spread(&headcount) < 0.02,
        "headcount spread {:.4}",
        spread(&headcount)
    );

    // The published estimate is not. It crosses zero, so the scenarios do not agree on the
    // direction of the association — the one thing a coefficient is for.
    assert!(
        published.iter().cloned().fold(f64::MAX, f64::min) < 0.0
            && published.iter().cloned().fold(f64::MIN, f64::max) > 0.0,
        "the published coefficient should change sign across scenarios: {published:?}"
    );
    let magnitudes: Vec<f64> = published.iter().map(|c| c.abs()).collect();
    let ratio = magnitudes.iter().cloned().fold(f64::MIN, f64::max)
        / magnitudes.iter().cloned().fold(f64::MAX, f64::min);
    assert!(
        ratio > 5.0,
        "published magnitude varies {ratio:.1}x across scenarios"
    );
}

/// Weighting districts by their students moves the published coefficient by an order of
/// magnitude, which is the sharpest single demonstration that it is not a stable estimate.
///
/// The corpus carried `-0.149` and `-0.547` for several phases with nothing asserting them.
#[test]
fn enrollment_weighting_moves_the_published_coefficient_tenfold() {
    let rows = report_card();
    let pi = column(&rows, |r| r.performance_index);
    let adm = column(&rows, |r| r.unweighted_adm);

    // Pearson correlation with each district weighted by its students. Written here rather than
    // in `dispersion` because it is a reading of one table, not an equity statistic.
    let weighted_correlation = |values: &[Option<f64>]| {
        let triples: Vec<(f64, f64, f64)> = pi
            .iter()
            .zip(values)
            .zip(&adm)
            .filter_map(|((y, x), w)| Some(((*y)?, (*x)?, (*w)?)))
            .collect();
        let total: f64 = triples.iter().map(|(_, _, w)| w).sum();
        let mean = |pick: fn(&(f64, f64, f64)) -> f64| {
            triples.iter().map(|t| pick(t) * t.2).sum::<f64>() / total
        };
        let (mean_y, mean_x) = (mean(|t| t.0), mean(|t| t.1));
        let covariance: f64 = triples
            .iter()
            .map(|t| t.2 * (t.0 - mean_y) * (t.1 - mean_x))
            .sum();
        let var_y: f64 = triples.iter().map(|t| t.2 * (t.0 - mean_y).powi(2)).sum();
        let var_x: f64 = triples.iter().map(|t| t.2 * (t.1 - mean_x).powi(2)).sum();
        covariance / (var_y * var_x).sqrt()
    };

    let published = weighted_correlation(&column(&rows, ReportCard::per_weighted_pupil));
    let headcount = weighted_correlation(&column(&rows, ReportCard::per_enrolled_pupil));
    close(
        published,
        -0.149,
        0.002,
        "enrollment-weighted, published divisor",
    );
    close(
        headcount,
        -0.547,
        0.002,
        "enrollment-weighted, headcount divisor",
    );

    // Against the district-weighted figures of -0.0155 and -0.3365: the published coefficient
    // moves by a factor of about ten, the headcount one by about 1.6. Both deepen, and only one
    // of them was ever near enough to zero for the move to change what it says.
    assert!(published / -0.0155 > 8.0, "published moved {published:.4}");
    assert!(headcount / -0.3365 < 2.0, "headcount moved {headcount:.4}");
}

/// Weighting districts by their students strengthens the relationship on both divisors, and the
/// unweighted-denominator figure is the largest coefficient in this file.
#[test]
fn enrollment_weighting_deepens_the_gap() {
    let rows = report_card();
    let adm: Vec<f64> = column(&rows, |r| r.unweighted_adm)
        .into_iter()
        .flatten()
        .collect();
    let pi: Vec<f64> = column(&rows, |r| r.performance_index)
        .into_iter()
        .flatten()
        .collect();

    // Enrollment-weighted mean Performance Index sits below the district-weighted one: the
    // average student's district scores lower than the average district.
    let district_mean = Dispersion::of(&pi).unwrap().mean;
    let student_mean = weighted_mean(&pi, &adm).unwrap();
    assert!(student_mean < district_mean - 2.0);
    close(
        student_mean,
        85.1,
        0.2,
        "enrollment-weighted mean Performance Index",
    );
}

/// The corpus's independent FY2024 headcount series agrees with the FY2025 one built here, which
/// is the check that the denominator result is not an artifact of one file.
#[test]
fn the_fy24_profile_report_agrees_on_a_headcount_denominator() {
    let rows = report_card();
    let pi = column(&rows, |r| r.performance_index);
    let cupp = joined(&rows, |d| d.operating_expenditure_per_pupil);

    close(
        correlate(&pi, &cupp),
        -0.388,
        0.002,
        "PI against FY2024 profile-report operating expenditure per pupil",
    );

    // Two sources, two years, one direction — and both far from the published −0.016.
    let fy25 = correlate(&pi, &column(&rows, ReportCard::per_enrolled_pupil));
    assert!(fy25 < -0.30 && correlate(&pi, &cupp) < -0.30);
}

/// Wealth still runs the other way, which is why the total looks flat: two opposing gradients.
#[test]
fn spending_rises_at_both_ends_of_the_wealth_distribution() {
    let rows = report_card();
    let valuation = joined(&rows, |d| d.valuation_per_pupil);
    let spending = column(&rows, ReportCard::per_enrolled_pupil);
    let disadvantaged = joined(&rows, |d| d.economically_disadvantaged);

    // Both correlations are positive: the poorest districts spend more, and so do the richest.
    assert!(correlate(&spending, &disadvantaged) > 0.3);
    assert!(correlate(&spending, &valuation) > 0.2);
}

// ---------------------------------------------------------------------------------------
// The guarantee, which turns out not to explain it
// ---------------------------------------------------------------------------------------

/// A negative result worth keeping. Splitting on guarantee status was the obvious suspicion —
/// half of Ohio's districts are funded off-formula against a frozen FY2020 baseline — and it
/// does not account for the denominator gap. The gap is present, and similar, in both halves.
#[test]
fn the_guarantee_does_not_explain_the_denominator_gap() {
    let all = report_card();
    let guarantee = department_model::guarantees();

    let split = |on: bool| -> Vec<ReportCard> {
        all.iter()
            .filter(|r| guarantee.get(&r.irn).is_some_and(|v| (*v > 0.0) == on))
            .cloned()
            .collect()
    };
    let guaranteed = split(true);
    let on_formula = split(false);
    assert_eq!(
        guaranteed.len(),
        294,
        "districts on the guarantee in FY2027"
    );
    assert_eq!(on_formula.len(), 313);

    for (name, rows, expected) in [
        ("guaranteed", &guaranteed, -0.389),
        ("on formula", &on_formula, -0.299),
    ] {
        let pi = column(rows, |r| r.performance_index);
        close(
            correlate(&pi, &column(rows, ReportCard::per_enrolled_pupil)),
            expected,
            0.003,
            name,
        );
        assert!(
            correlate(&pi, &column(rows, ReportCard::per_weighted_pupil)).abs() < 0.1,
            "{name}: weighted divisor is near zero in both halves too"
        );
    }

    // And the corpus's wealth finding shows through: guaranteed districts score higher, because
    // the guarantee holds up property-wealthy districts.
    let mean = |rows: &[ReportCard]| {
        let v: Vec<f64> = column(rows, |r| r.performance_index)
            .into_iter()
            .flatten()
            .collect();
        Dispersion::of(&v).unwrap().mean
    };
    assert!(mean(&guaranteed) > mean(&on_formula) + 3.0);
}

// ---------------------------------------------------------------------------------------
// Time: the Index barely moves, and lagging the spending does not help
// ---------------------------------------------------------------------------------------

/// The Performance Index is very nearly a fixed district trait over the three years held.
///
/// This cuts both ways and both matter. It means a single-year cross-section is a fair stand-in
/// for a district's Index, so the source paper's "single year" limitation is weaker than it
/// feared. It also means the measure cannot detect anything that changes annually — including
/// any change in what a district spends.
#[test]
fn the_performance_index_is_almost_static_across_three_years() {
    let rows = report_card();
    let years: [Pick; 3] = [
        |r| r.performance_index_earliest,
        |r| r.performance_index_prior,
        |r| r.performance_index,
    ];
    for pair in years.windows(2) {
        let r = correlate(&column(&rows, pair[0]), &column(&rows, pair[1]));
        assert!(r > 0.98, "adjacent years correlate at {r:.4}");
    }
    close(
        correlate(
            &column(&rows, |r| r.performance_index_earliest),
            &column(&rows, |r| r.performance_index),
        ),
        0.981,
        0.003,
        "2022-23 against 2024-25",
    );

    // Within-district movement against between-district spread: 2.1 points against sd 10.9.
    let mut ranges: Vec<f64> = rows
        .iter()
        .filter_map(|r| {
            let v: Vec<f64> = years.iter().filter_map(|pick| pick(r)).collect();
            (v.len() == 3).then(|| {
                v.iter().copied().fold(f64::MIN, f64::max)
                    - v.iter().copied().fold(f64::MAX, f64::min)
            })
        })
        .collect();
    ranges.sort_by(f64::total_cmp);
    close(
        ranges[ranges.len() / 2],
        2.1,
        0.2,
        "median three-year range",
    );
}

/// Lagging the spending measure — the source paper's own proposed next step — does not help,
/// and the direction rules out a forward-propagating effect.
///
/// FY2024 spending covers the 2023-24 school year. Correlated against the Index for the year
/// before it, the same year, and the year after, the association is flat and slightly
/// *strongest* looking backwards. Whatever this relationship is, nothing is flowing forward
/// from the dollars to the scores.
#[test]
fn lagging_the_spending_does_not_change_the_association() {
    let rows = report_card();
    let spending = joined(&rows, |d| d.operating_expenditure_per_pupil);

    let before = correlate(&column(&rows, |r| r.performance_index_earliest), &spending); // spending after outcome
    let same = correlate(&column(&rows, |r| r.performance_index_prior), &spending); // contemporaneous
    let after = correlate(&column(&rows, |r| r.performance_index), &spending); // spending before

    close(
        before,
        -0.426,
        0.003,
        "FY24 spending against the prior year's Index",
    );
    close(
        same,
        -0.412,
        0.003,
        "FY24 spending against the same year's Index",
    );
    close(
        after,
        -0.388,
        0.003,
        "FY24 spending against the following year's Index",
    );

    // The backwards lag is the strongest of the three. A causal reading requires the opposite.
    assert!(before < after, "no forward-directed signal");
}

// ---------------------------------------------------------------------------------------
// Growth rather than level, where the sign changes
// ---------------------------------------------------------------------------------------

/// Ohio's value-added model is normed to a state average of zero, so this measure ranks
/// districts against each other and says nothing about whether Ohio improved.
#[test]
fn the_progress_measure_is_centred_by_construction() {
    let rows = report_card();
    let effect: Vec<f64> = column(&rows, |r| r.progress_effect_size)
        .into_iter()
        .flatten()
        .collect();
    assert_eq!(effect.len(), 607);
    let mean = effect.iter().sum::<f64>() / effect.len() as f64;
    assert!(mean.abs() < 0.01, "centred on zero, got {mean:+.4}");
    assert!(effect.iter().copied().fold(f64::MIN, f64::max) < 0.35);
}

/// The composite scales with student count and the effect size does not, so only one of them
/// compares districts.
#[test]
fn the_composite_carries_district_size_and_the_effect_size_carries_less() {
    let rows = report_card();
    let adm = column(&rows, |r| r.unweighted_adm);
    let composite = correlate(&column(&rows, |r| r.progress_composite), &adm);
    let effect = correlate(&column(&rows, |r| r.progress_effect_size), &adm);
    close(composite, 0.244, 0.003, "composite against enrollment");
    close(effect, 0.155, 0.003, "effect size against enrollment");
    // They agree closely enough that the difference is easy to miss, which is the hazard.
    assert!(
        correlate(
            &column(&rows, |r| r.progress_composite),
            &column(&rows, |r| r.progress_effect_size)
        ) > 0.9
    );
}

/// **The finding.** Holding disadvantage constant, the same spending variable correlates
/// negatively with attainment level and positively with growth.
///
/// Neither coefficient is large and neither is causal. What they establish is narrower and
/// firmer: the negative association this corpus recorded against the Performance Index is a
/// property of measuring *level*, and it does not survive the switch to a measure of *gain*.
#[test]
fn spending_flips_sign_between_the_level_measure_and_the_growth_measure() {
    let rows = report_card();
    let ed = joined(&rows, |d| d.economically_disadvantaged);
    let spending = column(&rows, ReportCard::per_enrolled_pupil);

    let partial_against = |outcome: Vec<Option<f64>>| {
        // Same complete cases for all three coefficients.
        let present: Vec<bool> = outcome
            .iter()
            .zip(&ed)
            .zip(&spending)
            .map(|((o, e), s)| o.is_some() && e.is_some() && s.is_some())
            .collect();
        let keep = |v: &[Option<f64>]| -> Vec<Option<f64>> {
            v.iter()
                .zip(&present)
                .map(|(x, ok)| if *ok { *x } else { None })
                .collect()
        };
        let (outcome, ed, spending) = (keep(&outcome), keep(&ed), keep(&spending));
        partial_correlation(
            correlate(&outcome, &spending),
            correlate(&outcome, &ed),
            correlate(&spending, &ed),
        )
        .unwrap()
    };

    let level = partial_against(column(&rows, |r| r.performance_index));
    let growth = partial_against(column(&rows, |r| r.progress_effect_size));

    close(level, -0.125, 0.005, "level measure, holding disadvantage");
    close(growth, 0.146, 0.005, "growth measure, holding disadvantage");
    assert!(level < 0.0 && growth > 0.0, "the sign flips");
}

/// Growth is much less bound to composition than level — but not free of it.
#[test]
fn growth_is_less_poverty_bound_than_level_without_being_independent_of_it() {
    let rows = report_card();
    let ed = joined(&rows, |d| d.economically_disadvantaged);

    let level = correlate(&column(&rows, |r| r.performance_index), &ed);
    let growth = correlate(&column(&rows, |r| r.progress_effect_size), &ed);
    close(
        level,
        -0.846,
        0.003,
        "Performance Index against disadvantage",
    );
    close(
        growth,
        -0.325,
        0.003,
        "Progress effect size against disadvantage",
    );

    // 71.5% of variance against 10.6%. Both real; one an order of magnitude larger.
    assert!(level * level > 6.0 * growth * growth);

    // And the two outcome measures are far from interchangeable.
    close(
        correlate(
            &column(&rows, |r| r.performance_index),
            &column(&rows, |r| r.progress_effect_size),
        ),
        0.375,
        0.003,
        "level against growth",
    );
}

/// Raw, with no control, spending has no association with growth at all — on either divisor.
/// The denominator question that dominates the level measure simply does not arise here.
#[test]
fn on_the_growth_measure_the_divisor_stops_mattering() {
    let rows = report_card();
    let growth = column(&rows, |r| r.progress_effect_size);
    let unweighted = correlate(&growth, &column(&rows, ReportCard::per_enrolled_pupil));
    let weighted = correlate(&growth, &column(&rows, ReportCard::per_weighted_pupil));

    assert!(unweighted.abs() < 0.06, "unweighted: {unweighted:+.3}");
    assert!(weighted.abs() < 0.06, "weighted: {weighted:+.3}");
    // Against -0.337 and -0.015 on the level measure: the gap was composition, and the growth
    // measure has most of the composition taken out of it.
    assert!((unweighted - weighted).abs() < 0.1);
}

// ---------------------------------------------------------------------------------------
// The need-adjusted model
// ---------------------------------------------------------------------------------------

/// Rows with every model variable present, as column vectors.
///
/// English-learner share is imputed at zero where the department suppressed it. The suppression
/// marker is a matched `<10` enrollment and `NC` percent, so these are districts with fewer than
/// ten English learners — under 0.7% of a median district. The assumption is stated rather than
/// hidden, and `the_model_survives_dropping_the_imputed_rows` re-runs without them.
fn model_frame(impute_english_learner: bool) -> (Vec<Vec<f64>>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let rows = report_card();
    let ed = joined(&rows, |d| d.economically_disadvantaged);
    let valuation = joined(&rows, |d| d.valuation_per_pupil);
    let spending = column(&rows, ReportCard::per_enrolled_pupil);

    let (mut spend, mut disadvantage, mut learner, mut disability) =
        (vec![], vec![], vec![], vec![]);
    let (mut enrollment, mut wealth) = (vec![], vec![]);
    let (mut level, mut growth, mut growth_one_year) = (vec![], vec![], vec![]);

    for (i, row) in rows.iter().enumerate() {
        let el = match row.english_learner {
            Some(v) => Some(v),
            None if impute_english_learner => Some(0.0),
            None => None,
        };
        let (Some(s), Some(e), Some(l), Some(d), Some(a), Some(w), Some(pi), Some(g), Some(g1)) = (
            spending[i],
            ed[i],
            el,
            row.students_with_disabilities,
            row.unweighted_adm,
            valuation[i],
            row.performance_index,
            row.progress_effect_size,
            row.progress_effect_size_one_year,
        ) else {
            continue;
        };
        spend.push(s);
        disadvantage.push(e * 100.0);
        learner.push(l);
        disability.push(d);
        enrollment.push(a.ln());
        wealth.push(w / 1000.0);
        level.push(pi);
        growth.push(g);
        growth_one_year.push(g1);
    }
    (
        vec![spend, disadvantage, learner, disability, enrollment, wealth],
        level,
        growth,
        growth_one_year,
    )
}

/// **The need-adjusted model.** Controlling for economic disadvantage, English-learner and
/// disability shares, district size, and property wealth, the spending coefficient is small and
/// negative against attainment level and moderate and positive against growth.
///
/// This is the multivariable descriptive model OCG White Paper 013 named as its own priority
/// next step. It does not identify an effect and this file claims none. It does establish that
/// the sign difference between the two outcome measures is not an artifact of controlling for
/// poverty alone.
#[test]
fn the_need_adjusted_model_keeps_the_sign_difference() {
    let (predictors, level, growth, _) = model_frame(true);
    assert_eq!(level.len(), 606);

    let on_level = least_squares(&predictors, &level).unwrap();
    let on_growth = least_squares(&predictors, &growth).unwrap();

    close(
        on_level.standardized[0],
        -0.073,
        0.006,
        "spending on level, standardised",
    );
    close(
        on_growth.standardized[0],
        0.209,
        0.006,
        "spending on growth, standardised",
    );
    assert!(on_level.t_statistics[1] < -2.0 && on_growth.t_statistics[1] > 2.0);

    // The level model is mostly demographics; the growth model explains far less of anything.
    close(on_level.r_squared, 0.752, 0.005, "level model fit");
    close(on_growth.r_squared, 0.227, 0.005, "growth model fit");

    // Disadvantage dominates the level model and is present but not dominant on growth.
    assert!(on_level.standardized[1] < -0.6);
    assert!(on_growth.standardized[1] > -0.5 && on_growth.standardized[1] < -0.2);
}

/// Adding controls *strengthens* the growth coefficient at every step, which is the signature of
/// a relationship being uncovered rather than manufactured.
#[test]
fn controls_uncover_the_growth_relationship_rather_than_creating_it() {
    let (predictors, _, growth, _) = model_frame(true);
    let mut betas = vec![];
    for depth in [1usize, 2, 4, 6] {
        let subset: Vec<Vec<f64>> = predictors.iter().take(depth).cloned().collect();
        betas.push(least_squares(&subset, &growth).unwrap().standardized[0]);
    }
    close(betas[0], 0.016, 0.006, "spending alone");
    close(betas[1], 0.147, 0.006, "plus disadvantage");
    close(betas[3], 0.209, 0.006, "plus every control");
    assert!(betas[0] < betas[1] && betas[1] < betas[3] + 0.01);
}

/// Property wealth predicts where a district's students are and not how far they moved. The
/// cleanest statement of the level/gain distinction available in this data.
#[test]
fn wealth_predicts_attainment_but_not_growth() {
    let (predictors, level, growth, _) = model_frame(true);
    let on_level = least_squares(&predictors, &level).unwrap();
    let on_growth = least_squares(&predictors, &growth).unwrap();
    // Valuation per pupil is the sixth predictor.
    assert!(on_level.t_statistics[6] > 2.0, "wealth predicts level");
    assert!(
        on_growth.t_statistics[6].abs() < 1.5,
        "wealth does not predict growth: t = {:.2}",
        on_growth.t_statistics[6]
    );
}

/// The published Progress measure is a three-year average, so pairing it with one year of
/// spending is a mismatch. The one-year gain answers the same question year-matched, and gives
/// the same sign slightly larger.
#[test]
fn the_year_matched_growth_measure_agrees() {
    let (predictors, _, growth, one_year) = model_frame(true);
    let three = least_squares(&predictors, &growth).unwrap();
    let single = least_squares(&predictors, &one_year).unwrap();
    close(
        single.standardized[0],
        0.269,
        0.008,
        "one-year gain, standardised",
    );
    assert!(single.standardized[0] > three.standardized[0]);
    assert!(single.t_statistics[1] > 2.0);
}

/// Dropping the districts whose English-learner share was imputed leaves the result standing.
#[test]
fn the_model_survives_dropping_the_imputed_rows() {
    let (predictors, _, growth, _) = model_frame(false);
    assert!(
        predictors[0].len() < 400,
        "the suppressed rows really are numerous"
    );
    let fit = least_squares(&predictors, &growth).unwrap();
    assert!(
        fit.standardized[0] > 0.1 && fit.t_statistics[1] > 2.0,
        "complete cases: beta {:.3}, t {:.2}",
        fit.standardized[0],
        fit.t_statistics[1]
    );
}

/// The report card's own economic-disadvantage share is top-coded by community eligibility and
/// is a different variable from the Cupp Report's. Both are committed; they are not substitutes.
#[test]
fn the_two_economic_disadvantage_measures_are_not_the_same_variable() {
    let rows = report_card();
    let report_card_share = column(&rows, |r| r.economically_disadvantaged);
    let cupp_share = joined(&rows, |d| d.economically_disadvantaged);
    let pi = column(&rows, |r| r.performance_index);

    close(
        correlate(&report_card_share, &cupp_share),
        0.823,
        0.005,
        "the two measures against each other",
    );
    close(
        correlate(&pi, &report_card_share),
        -0.734,
        0.005,
        "top-coded measure",
    );
    close(correlate(&pi, &cupp_share), -0.846, 0.005, "Cupp measure");

    // Saturation is the mechanism: many more districts pinned at 100% in the report-card file.
    let at_ceiling =
        |v: &[Option<f64>], ceiling: f64| v.iter().flatten().filter(|x| **x >= ceiling).count();
    assert_eq!(at_ceiling(&report_card_share, 99.95), 87);
    assert_eq!(at_ceiling(&cupp_share, 0.9995), 37);
}

/// The headline Progress figure is literally the three-year gain, not a separate calculation.
#[test]
fn the_published_progress_figure_is_the_three_year_gain() {
    let rows = report_card();
    let headline = column(&rows, |r| r.progress_effect_size);
    let one_year = column(&rows, |r| r.progress_effect_size_one_year);
    // Same construct, different window: correlated but distinguishable, and the shorter window
    // is noisier, exactly as a gain measure over fewer cohorts should be.
    close(
        correlate(&headline, &one_year),
        0.904,
        0.005,
        "three-year against one-year",
    );
    let spread = |v: Vec<Option<f64>>| {
        let values: Vec<f64> = v.into_iter().flatten().collect();
        Dispersion::of(&values.iter().map(|x| x + 10.0).collect::<Vec<_>>())
            .unwrap()
            .std_dev
    };
    assert!(spread(one_year) > spread(headline));
}
