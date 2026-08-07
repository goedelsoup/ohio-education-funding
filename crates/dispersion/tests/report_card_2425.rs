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

use dispersion::{
    partial_correlation, rank_correlation, wealth_neutrality, weighted_mean, Dispersion,
};

const REPORT_CARD: &str = include_str!("../fixtures/report-card-2425-district-data.csv");
const PROFILE: &str = include_str!("../fixtures/cupp-fy24-district-data.csv");
const FY27_MODEL: &str = include_str!("../../foundation/fixtures/fy27-department-model.csv");

/// Column indices in the report-card fixture.
mod col {
    pub const IRN: usize = 0;
    pub const PERFORMANCE_INDEX: usize = 2;
    pub const UNWEIGHTED_ADM: usize = 5;
    pub const WEIGHTED_ADM: usize = 6;
    pub const OPERATING_EXPENDITURES: usize = 7;
    pub const EQUIVALENT_PUPIL: usize = 8;
    pub const FEDERAL: usize = 9;
    pub const STATE_AND_LOCAL: usize = 10;
}

/// Column indices in the profile-report fixture.
mod profile {
    pub const IRN: usize = 0;
    pub const ECON_DISADVANTAGED: usize = 3;
    pub const VALUATION_PER_PUPIL: usize = 4;
    pub const OPERATING_EPP: usize = 7;
}

/// Column indices in the FY2027 department-model fixture.
mod model {
    pub const IRN: usize = 0;
    pub const GUARANTEE: usize = 15;
}

struct Row(Vec<String>);

impl Row {
    fn key(&self) -> &str {
        self.0.get(col::IRN).map_or("", String::as_str)
    }
    fn get(&self, i: usize) -> Option<f64> {
        self.0.get(i).and_then(|c| c.trim().parse::<f64>().ok())
    }
}

fn parse(csv: &str) -> Vec<Row> {
    csv.lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| Row(l.split(',').map(str::to_string).collect()))
        .collect()
}

fn report_card() -> Vec<Row> {
    parse(REPORT_CARD)
}

/// Look up a column of another fixture by IRN, aligned to the report-card rows.
fn joined(rows: &[Row], other: &str, key: usize, value: usize) -> Vec<Option<f64>> {
    let table = parse(other);
    rows.iter()
        .map(|r| {
            table
                .iter()
                .find(|o| o.0.get(key).map(String::as_str) == Some(r.key()))
                .and_then(|o| o.get(value))
        })
        .collect()
}

/// Pairs where both sides are present, which is what every coefficient here is computed over.
fn pairs(a: &[Option<f64>], b: &[Option<f64>]) -> (Vec<f64>, Vec<f64>) {
    a.iter()
        .zip(b)
        .filter_map(|(x, y)| Some(((*x)?, (*y)?)))
        .unzip()
}

fn column(rows: &[Row], i: usize) -> Vec<Option<f64>> {
    rows.iter().map(|r| r.get(i)).collect()
}

/// Operating expenditures over a chosen ADM column — the whole subject of this file.
fn per_pupil(rows: &[Row], adm: usize) -> Vec<Option<f64>> {
    rows.iter()
        .map(|r| match (r.get(col::OPERATING_EXPENDITURES), r.get(adm)) {
            (Some(dollars), Some(pupils)) if pupils > 0.0 => Some(dollars / pupils),
            _ => None,
        })
        .collect()
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
    assert!(rows.iter().all(|r| r.get(col::PERFORMANCE_INDEX).is_some()));
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
        let (Some(dollars), Some(pupils), Some(published)) = (
            row.get(col::OPERATING_EXPENDITURES),
            row.get(col::WEIGHTED_ADM),
            row.get(col::EQUIVALENT_PUPIL),
        ) else {
            continue;
        };
        let computed = dollars / pupils;
        // Half a pupil of rounding in the denominator, plus a dollar for the published figure's
        // own rounding.
        let tolerance = published * (0.5 / pupils) + 1.0;
        assert!(
            (computed - published).abs() < tolerance,
            "{} reconstructs to {computed:.2} against a published {published:.0}, \
             outside the {tolerance:.2} the integer ADM allows",
            row.0.get(1).map_or("?", String::as_str),
        );
        checked += 1;
    }
    assert_eq!(checked, 607);

    // Weighted ADM really is published whole and unweighted ADM really is not, which is the
    // premise of the tolerance above.
    assert!(rows
        .iter()
        .filter_map(|r| r.get(col::WEIGHTED_ADM))
        .all(|v| (v.fract()).abs() < 1e-9));
    assert!(
        rows.iter()
            .filter_map(|r| r.get(col::UNWEIGHTED_ADM))
            .filter(|v| v.fract().abs() > 1e-9)
            .count()
            > 500
    );
}

#[test]
fn the_white_paper_coefficients_reproduce() {
    let rows = report_card();
    let pi = column(&rows, col::PERFORMANCE_INDEX);

    close(
        correlate(&pi, &column(&rows, col::EQUIVALENT_PUPIL)),
        -0.016,
        0.001,
        "PI against total spending per equivalent pupil",
    );
    close(
        rank(&pi, &column(&rows, col::EQUIVALENT_PUPIL)),
        0.048,
        0.001,
        "the same, Spearman",
    );
    close(
        correlate(&pi, &column(&rows, col::FEDERAL)),
        -0.558,
        0.001,
        "PI against federal expenditure per equivalent pupil",
    );
    close(
        correlate(&pi, &column(&rows, col::STATE_AND_LOCAL)),
        0.086,
        0.001,
        "PI against state-and-local expenditure per equivalent pupil",
    );
    close(
        rank(
            &column(&rows, col::UNWEIGHTED_ADM),
            &column(&rows, col::EQUIVALENT_PUPIL),
        ),
        -0.366,
        0.001,
        "enrollment against spending per equivalent pupil",
    );
}

#[test]
fn the_white_paper_distributions_reproduce() {
    let rows = report_card();
    let spending: Vec<f64> = column(&rows, col::EQUIVALENT_PUPIL)
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

    let pi: Vec<f64> = column(&rows, col::PERFORMANCE_INDEX)
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
    let pi = column(&rows, col::PERFORMANCE_INDEX);

    let weighted = correlate(&pi, &per_pupil(&rows, col::WEIGHTED_ADM));
    let unweighted = correlate(&pi, &per_pupil(&rows, col::UNWEIGHTED_ADM));

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
    let ratio: Vec<Option<f64>> = rows
        .iter()
        .map(
            |r| match (r.get(col::WEIGHTED_ADM), r.get(col::UNWEIGHTED_ADM)) {
                (Some(w), Some(u)) if u > 0.0 => Some(w / u),
                _ => None,
            },
        )
        .collect();
    let disadvantaged = joined(&rows, PROFILE, profile::IRN, profile::ECON_DISADVANTAGED);

    close(
        correlate(&ratio, &disadvantaged),
        0.800,
        0.002,
        "ADM weight ratio against economically disadvantaged share",
    );
    close(
        correlate(&ratio, &column(&rows, col::PERFORMANCE_INDEX)),
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
    let pi = column(&rows, col::PERFORMANCE_INDEX);
    let disadvantaged = joined(&rows, PROFILE, profile::IRN, profile::ECON_DISADVANTAGED);

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
    let pi = column(&rows, col::PERFORMANCE_INDEX);
    let ed = joined(&rows, PROFILE, profile::IRN, profile::ECON_DISADVANTAGED);

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
        &per_pupil(&rows, col::UNWEIGHTED_ADM),
        -0.355,
        -0.125,
        "spending per unweighted pupil, raw then holding disadvantage",
    );
    check(
        &column(&rows, col::FEDERAL),
        -0.558,
        -0.158,
        "federal spending per equivalent pupil, raw then holding disadvantage",
    );
}

/// Every sensitivity scenario the paper ran, on both divisors. Its own conclusion — that the
/// near-zero result is not driven by outliers or small districts — holds; so does the opposite
/// result on the other denominator.
#[test]
fn the_divisor_gap_survives_the_papers_own_sensitivity_checks() {
    let all = report_card();
    let scenarios: Vec<(&str, Vec<&Row>)> = vec![
        ("all districts", all.iter().collect()),
        ("excluding the highest per-pupil district", {
            let top = all
                .iter()
                .max_by(|a, b| {
                    a.get(col::EQUIVALENT_PUPIL)
                        .partial_cmp(&b.get(col::EQUIVALENT_PUPIL))
                        .unwrap()
                })
                .unwrap();
            all.iter().filter(|r| !std::ptr::eq(*r, top)).collect()
        }),
        ("excluding enrollment under 582", {
            all.iter()
                .filter(|r| r.get(col::UNWEIGHTED_ADM).is_some_and(|a| a >= 582.0))
                .collect()
        }),
    ];

    for (name, subset) in scenarios {
        let rows: Vec<Row> = subset.into_iter().map(|r| Row(r.0.clone())).collect();
        let pi = column(&rows, col::PERFORMANCE_INDEX);
        let weighted = correlate(&pi, &per_pupil(&rows, col::WEIGHTED_ADM));
        let unweighted = correlate(&pi, &per_pupil(&rows, col::UNWEIGHTED_ADM));
        assert!(
            weighted.abs() < 0.08,
            "{name}: weighted divisor should stay near zero, got {weighted:.3}"
        );
        assert!(
            unweighted < -0.30,
            "{name}: unweighted divisor should stay clearly negative, got {unweighted:.3}"
        );
    }
}

/// Weighting districts by their students strengthens the relationship on both divisors, and the
/// unweighted-denominator figure is the largest coefficient in this file.
#[test]
fn enrollment_weighting_deepens_the_gap() {
    let rows = report_card();
    let adm: Vec<f64> = column(&rows, col::UNWEIGHTED_ADM)
        .into_iter()
        .flatten()
        .collect();
    let pi: Vec<f64> = column(&rows, col::PERFORMANCE_INDEX)
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
    let pi = column(&rows, col::PERFORMANCE_INDEX);
    let cupp = joined(&rows, PROFILE, profile::IRN, profile::OPERATING_EPP);

    close(
        correlate(&pi, &cupp),
        -0.388,
        0.002,
        "PI against FY2024 profile-report operating expenditure per pupil",
    );

    // Two sources, two years, one direction — and both far from the published −0.016.
    let fy25 = correlate(&pi, &per_pupil(&rows, col::UNWEIGHTED_ADM));
    assert!(fy25 < -0.30 && correlate(&pi, &cupp) < -0.30);
}

/// Wealth still runs the other way, which is why the total looks flat: two opposing gradients.
#[test]
fn spending_rises_at_both_ends_of_the_wealth_distribution() {
    let rows = report_card();
    let valuation = joined(&rows, PROFILE, profile::IRN, profile::VALUATION_PER_PUPIL);
    let spending = per_pupil(&rows, col::UNWEIGHTED_ADM);
    let disadvantaged = joined(&rows, PROFILE, profile::IRN, profile::ECON_DISADVANTAGED);

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
    let guarantee = joined(&all, FY27_MODEL, model::IRN, model::GUARANTEE);

    let split = |on: bool| -> Vec<Row> {
        all.iter()
            .zip(&guarantee)
            .filter(|(_, g)| g.is_some_and(|v| (v > 0.0) == on))
            .map(|(r, _)| Row(r.0.clone()))
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
        let pi = column(rows, col::PERFORMANCE_INDEX);
        close(
            correlate(&pi, &per_pupil(rows, col::UNWEIGHTED_ADM)),
            expected,
            0.003,
            name,
        );
        assert!(
            correlate(&pi, &per_pupil(rows, col::WEIGHTED_ADM)).abs() < 0.1,
            "{name}: weighted divisor is near zero in both halves too"
        );
    }

    // And the corpus's wealth finding shows through: guaranteed districts score higher, because
    // the guarantee holds up property-wealthy districts.
    let mean = |rows: &[Row]| {
        let v: Vec<f64> = column(rows, col::PERFORMANCE_INDEX)
            .into_iter()
            .flatten()
            .collect();
        Dispersion::of(&v).unwrap().mean
    };
    assert!(mean(&guaranteed) > mean(&on_formula) + 3.0);
}
