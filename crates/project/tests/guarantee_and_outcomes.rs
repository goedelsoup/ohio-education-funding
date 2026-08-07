//! Does the guarantee buy outcomes?
//!
//! The question this corpus was in a position to ask for three phases and could not compute,
//! because the funding panel and the report-card panel were never joined. [`project::joined`]
//! is that join; these tests are what it was for.
//!
//! # The answer is no, and the way it is no is the point
//!
//! Guarantee status looks like it predicts achievement. Districts the guarantee protects score
//! **+0.187** against the Performance Index, and their median Index is 89.9 against 85.6 for
//! districts on the formula. Hold poverty constant and the association is **+0.035** — nothing.
//!
//! The guaranteed districts score higher because they are **less poor**: median economic
//! disadvantage **42.8% against 51.9%** on the profile report's headcount share. That is the
//! same wealth gradient the corpus already established on the funding side, arriving on the
//! outcome side as a spurious achievement effect. It is the sharpest instance yet of a rule
//! this corpus wrote down two phases ago: most district-level correlates of the Performance
//! Index are poverty in disguise.
//!
//! # And the two poverty measures do not agree about how large the gap is
//!
//! On the report card's top-coded share the same two medians are **44.1% against 65.4%** — a
//! 21.3-point gap where the profile report shows 9.1. Top-coding piles the poorest districts at
//! the ceiling, and those are disproportionately the formula districts, so the ceiling widens a
//! gap it is not measuring. Every partial correlation here uses the profile report's share for
//! that reason.
//!
//! # Every figure here is a correlation
//!
//! Nothing below identifies an effect. Districts are not assigned to the guarantee at random —
//! they are on it because their FY2020 funding exceeded what the formula now computes, which is
//! itself a function of wealth and enrollment history. These tests pin what the data shows and
//! bound how it may be read.

use dispersion::{partial_correlation, rank_correlation, wealth_neutrality};
use project::outcomes::{joined, Joined};

/// Pearson correlation, via the crate that owns it.
fn correlation(xs: &[f64], ys: &[f64]) -> f64 {
    wealth_neutrality(xs, ys)
        .expect("equal-length series")
        .correlation
}

/// Correlation of `xs` with `ys` holding `control` constant.
///
/// [`partial_correlation`] takes the three pairwise correlations rather than the series, which
/// keeps the partialling separable from how each correlation was computed. This wraps it for
/// the common case where all three are Pearson over the same rows.
fn controlling_for(xs: &[f64], ys: &[f64], control: &[f64]) -> f64 {
    partial_correlation(
        correlation(xs, ys),
        correlation(xs, control),
        correlation(ys, control),
    )
    .expect("a non-degenerate control")
}

/// Series of `(x, outcome, poverty)` for districts where all three are present.
fn series(
    records: &[Joined],
    x: impl Fn(&Joined) -> Option<f64>,
    outcome: impl Fn(&Joined) -> Option<f64>,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut poverty = Vec::new();
    for record in records {
        let (Some(a), Some(b), Some(p)) = (
            x(record),
            outcome(record),
            record.economically_disadvantaged,
        ) else {
            continue;
        };
        xs.push(a);
        ys.push(b);
        poverty.push(p);
    }
    (xs, ys, poverty)
}

fn on_guarantee(record: &Joined) -> Option<f64> {
    Some(if record.on_guarantee() { 1.0 } else { 0.0 })
}
fn guarantee_share(record: &Joined) -> Option<f64> {
    Some(record.guarantee_share())
}
fn performance_index(record: &Joined) -> Option<f64> {
    record.outcome.performance_index
}
fn progress(record: &Joined) -> Option<f64> {
    record.outcome.progress_effect_size
}

fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    values[values.len() / 2]
}

#[test]
fn the_join_covers_every_district_in_all_three_panels() {
    assert_eq!(joined().len(), 606);
}

#[test]
fn poverty_dominates_the_performance_index() {
    // The control everything below rests on, reproduced here so that a fixture refresh which
    // moved it would fail at the source rather than silently rescale every partial.
    let records = joined();
    let (poverty, index, _) = series(
        &records,
        |r| r.economically_disadvantaged,
        performance_index,
    );
    let r = correlation(&poverty, &index);
    assert!(
        (r - -0.846).abs() < 0.01,
        "poverty against the Performance Index is {r:.3}, not the established -0.846"
    );
}

#[test]
fn guarantee_status_predicts_achievement_until_poverty_is_held_constant() {
    // The finding. 0.187 is a real association and 0.035 is what is left of it.
    let records = joined();
    let (guarantee, index, poverty) = series(&records, on_guarantee, performance_index);

    let raw = correlation(&guarantee, &index);
    assert!((raw - 0.187).abs() < 0.01, "raw {raw:.3}");

    let controlled = controlling_for(&guarantee, &index, &poverty);
    assert!(
        controlled.abs() < 0.06,
        "holding poverty, guarantee status still explains {controlled:.3}"
    );
    assert!(
        raw.abs() > 4.0 * controlled.abs(),
        "the confound should absorb most of the association: {raw:.3} -> {controlled:.3}"
    );
}

#[test]
fn the_same_holds_for_growth_and_for_a_continuous_reading_of_dependence() {
    // Not an artifact of the binary split or of the attainment measure: dependence measured as
    // a share of aid, against both published outcome measures, does the same thing.
    let records = joined();
    for (label, x) in [
        (
            "on the guarantee",
            on_guarantee as fn(&Joined) -> Option<f64>,
        ),
        ("guarantee share of aid", guarantee_share),
    ] {
        for (outcome_label, y) in [
            (
                "Performance Index",
                performance_index as fn(&Joined) -> Option<f64>,
            ),
            ("Progress", progress),
        ] {
            let (xs, ys, poverty) = series(&records, x, y);
            let controlled = controlling_for(&xs, &ys, &poverty);
            assert!(
                controlled.abs() < 0.06,
                "{label} against {outcome_label} holding poverty is {controlled:.3}"
            );
        }
    }
}

#[test]
fn the_guaranteed_districts_are_less_poor_which_is_the_whole_mechanism() {
    let records = joined();
    let poverty_of = |on: bool| {
        median(
            records
                .iter()
                .filter(|r| r.on_guarantee() == on)
                .filter_map(|r| r.economically_disadvantaged)
                .collect(),
        )
    };
    let guaranteed = poverty_of(true);
    let formula = poverty_of(false);
    assert!(
        guaranteed < formula - 0.05,
        "guaranteed median poverty {guaranteed:.3} vs formula {formula:.3}"
    );
    assert!((guaranteed - 0.428).abs() < 0.01 && (formula - 0.519).abs() < 0.01);
}

#[test]
fn the_top_coded_measure_more_than_doubles_the_apparent_poverty_gap() {
    // Why the choice of poverty measure is not a detail. The report card tops out at 100% for
    // any district where community eligibility certifies every student, and those are
    // disproportionately the formula districts — so the ceiling widens a gap it is not
    // measuring, from 9 points to 21.
    let records = joined();
    let gap = |share: fn(&Joined) -> Option<f64>, scale: f64| {
        let of = |on: bool| {
            median(
                records
                    .iter()
                    .filter(|r| r.on_guarantee() == on)
                    .filter_map(share)
                    .collect(),
            )
        };
        (of(false) - of(true)) * scale
    };
    let profile_gap = gap(|r| r.economically_disadvantaged, 100.0);
    let report_card_gap = gap(|r| r.outcome.economically_disadvantaged, 1.0);
    assert!(
        report_card_gap > 2.0 * profile_gap,
        "profile {profile_gap:.1} points, report card {report_card_gap:.1} points"
    );
}

#[test]
fn state_aid_per_pupil_is_not_a_spending_measure() {
    // A caution the join makes checkable. State aid substitutes for local revenue rather than
    // adding to spending: it is 38% of the median district's operating budget, and once poverty
    // is held constant it is almost unrelated to what a district actually spends per pupil.
    //
    // So the corpus's +0.146 for spending against growth and the -0.247 below for aid against
    // growth are not in tension. They are different variables, and reading one as the other is
    // the mistake this test exists to make hard.
    let records = joined();
    let (aid, spending, poverty) = series(
        &records,
        |r| Some(r.realized_aid_per_pupil()),
        |r| r.outcome.per_enrolled_pupil(),
    );
    let controlled = controlling_for(&aid, &spending, &poverty);
    assert!(
        controlled.abs() < 0.10,
        "state aid and total spending track each other at {controlled:.3} once poverty is held"
    );

    let (aid, growth, poverty) = series(&records, |r| Some(r.realized_aid_per_pupil()), progress);
    let against_growth = controlling_for(&aid, &growth, &poverty);
    assert!(
        against_growth < -0.15,
        "aid against growth holding poverty is {against_growth:.3}"
    );
}

#[test]
fn total_spending_against_growth_reproduces_the_established_figure() {
    // An independent check that this join is the same join the dispersion tests make: computed
    // from a different crate, through different code, it lands on the corpus's +0.146.
    let records = joined();
    let (spending, growth, poverty) =
        series(&records, |r| r.outcome.per_enrolled_pupil(), progress);
    let controlled = controlling_for(&spending, &growth, &poverty);
    assert!(
        (controlled - 0.146).abs() < 0.01,
        "spending against growth holding poverty is {controlled:.3}, not the established +0.146"
    );
}

#[test]
fn the_result_survives_a_rank_based_reading() {
    // Guarantee status is binary and the Index is bounded, so a Pearson coefficient is worth
    // checking against a rank one before anything rests on it.
    let records = joined();
    let (guarantee, index, _) = series(&records, guarantee_share, performance_index);
    let spearman = rank_correlation(&guarantee, &index).expect("equal lengths");
    let pearson = correlation(&guarantee, &index);
    assert!(
        spearman.signum() == pearson.signum() && (spearman - pearson).abs() < 0.15,
        "rank {spearman:.3} vs pearson {pearson:.3}"
    );
}

#[test]
fn the_published_near_zero_correlation_turns_on_a_single_tiny_district() {
    // The complete panel is 607 minus Put-in-Bay, and that one district — 77 weighted pupils,
    // the largest quantisation error in the file — accounts for most of the magnitude the
    // weighted-denominator correlation has at all: -0.0155 over 607 becomes -0.0043 over 606.
    //
    // So the published figure is not merely denominator-driven, it is fragile. The corpus's own
    // enrolled-denominator figure moves by a twentieth over the same swap. A near-zero result
    // that one small district can move by seventy percent was never carrying much.
    let records = joined();
    let (weighted, index, _) = series(
        &records,
        |r| r.outcome.per_equivalent_pupil,
        performance_index,
    );
    let (enrolled, index_again, _) = series(
        &records,
        |r| r.outcome.per_enrolled_pupil(),
        performance_index,
    );

    let weighted_606 = correlation(&weighted, &index);
    let enrolled_606 = correlation(&enrolled, &index_again);

    // Published over 607: -0.0155 weighted, -0.3365 enrolled.
    assert!(
        (weighted_606 - -0.0043).abs() < 0.002,
        "weighted over the complete panel is {weighted_606:.4}"
    );
    assert!(
        (weighted_606 - -0.0155).abs() > 0.008,
        "one district should move the weighted figure appreciably"
    );
    assert!(
        (enrolled_606 - -0.3365).abs() < 0.03,
        "the enrolled figure should be near its published value: {enrolled_606:.4}"
    );
}

#[test]
fn nothing_here_is_computed_on_the_three_excluded_districts() {
    // The crosswalk's exceptions are outside every figure above. Stated as a test so that a
    // future join which quietly included them would have to say so.
    let joined_irns: Vec<String> = joined().into_iter().map(|r| r.funding.irn).collect();
    for exception in project::crosswalk::EXCEPTIONS {
        assert!(
            !joined_irns.contains(&exception.irn.to_string()),
            "{} is in the join",
            exception.name
        );
    }
}
