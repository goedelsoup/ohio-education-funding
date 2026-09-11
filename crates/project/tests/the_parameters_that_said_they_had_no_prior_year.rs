//! Several `parameter` nodes carry a one-year `series:` and say so — "no prior-year values are
//! held", "the calculator publishes one year and no history". Two years of the calculator have
//! been in the cache the whole time, and both print these numbers on labelled cells above their
//! district tables.
//!
//! `crates/project/fixtures/calculator-parameters.csv` is two rows of them, and this is what
//! holds it to the sources on both ends: to the department's own labels through
//! `connect::fixtures::calculator_scalars`, and to the FY2027 constants a *different* extractor
//! already committed.

use project::panel::categoricals;
use project::prior_model as prior;

/// The FY2027 row of the scalar fixture against the constants `connect::fixtures::fy27` produced.
///
/// Two extractors, two paths through the same workbook, one answer. This is the check that
/// matters most here and it is the one that caught the bug worth recording: the scalar reader
/// took "the first number to the right of the label", which on the `EL` sheet is
/// `Average Base Cost Per-Pupil` sitting to the right of `Weight Cat 1` — so all three English
/// learner weights came out as $8,241.61 and looked plausible in a column of dollars.
#[test]
fn the_fy2027_row_agrees_with_what_the_other_extractor_committed() {
    let at = |name: &str| {
        *prior::scalar(name)
            .get(&2027)
            .unwrap_or_else(|| panic!("{name} is not in the scalar fixture for FY2027"))
    };

    for (index, weight) in categoricals::SPECIAL_EDUCATION_WEIGHTS.iter().enumerate() {
        let found = at(&format!("special_education_weight_{}", index + 1));
        assert!(
            (found - weight).abs() < 1e-9,
            "category {}: {found}",
            index + 1
        );
    }
    for (index, weight) in categoricals::CTE_WEIGHTS.iter().enumerate() {
        let found = at(&format!("career_technical_weight_{}", index + 1));
        assert!(
            (found - weight).abs() < 1e-9,
            "category {}: {found}",
            index + 1
        );
    }
    for (index, weight) in categoricals::ENGLISH_LEARNER_WEIGHTS.iter().enumerate() {
        let found = at(&format!("english_learner_weight_{}", index + 1));
        assert!(
            (found - weight).abs() < 1e-9,
            "category {}: {found}",
            index + 1
        );
    }
    assert!(
        (at("career_technical_associated_weight") - categoricals::CTE_ASSOCIATED_WEIGHT).abs()
            < 1e-9
    );
    assert!(
        (at("average_base_cost_per_pupil") - categoricals::AVERAGE_BASE_COST_PER_PUPIL).abs()
            < 0.01
    );
    assert!(
        (at("career_technical_base_cost_per_pupil") - categoricals::CTE_BASE_COST_PER_PUPIL).abs()
            < 0.01
    );
    assert!((at("dpia_per_pupil") - categoricals::DPIA_PER_PUPIL).abs() < 1e-9);
    assert!(
        (at("dpia_weight_economically_disadvantaged") - categoricals::DPIA_BLEND.0).abs() < 1e-12
    );
    assert!((at("dpia_weight_directly_certified") - categoricals::DPIA_BLEND.1).abs() < 1e-12);
    assert!(
        (at("dpia_statewide_percentage") - categoricals::DPIA_STATEWIDE_PERCENTAGE).abs() < 1e-12
    );
}

#[test]
fn every_weight_and_every_base_cost_holds_across_the_two_years() {
    // H.B. 96 retaining FY2022 cost inputs and the FY2024 statewide averages, visible a year
    // before the year the act names.
    let (moved, held) = prior::scalars_that_moved();
    assert_eq!(moved.len(), 10, "{moved:?}");
    assert_eq!(held.len(), 23, "{held:?}");

    for name in held.iter().filter(|name| name.contains("weight")) {
        assert!(!moved.contains(name), "{name}");
    }
    for name in [
        "average_base_cost_per_pupil",
        "career_technical_base_cost_per_pupil",
        "dpia_per_pupil",
        "performance_supplement_per_pupil",
        "preschool_base_amount",
    ] {
        assert!(held.contains(&name.to_string()), "{name} moved: {moved:?}");
    }
}

#[test]
fn the_performance_supplement_rate_did_not_move() {
    // `parameter/performance-supplement-rate` closed with "No prior-year values are held."
    let series = prior::scalar("performance_supplement_per_pupil");
    assert_eq!(series.len(), 2);
    assert!(
        series.values().all(|rate| (rate - 13.0).abs() < 1e-9),
        "{series:?}"
    );
}

#[test]
fn the_two_enrolment_supplements_are_the_only_per_pupil_amounts_that_rose() {
    // `parameter/enrolment-supplement-amounts` closed the same way, and these are the answer:
    // the only per-pupil dollar figures in the plan that are larger in FY2027 than in FY2026.
    let base = prior::scalar("base_funding_supplement_per_pupil");
    let growth = prior::scalar("enrolment_growth_supplement_per_pupil");
    assert_eq!(base.get(&2026).copied(), Some(27.0));
    assert_eq!(base.get(&2027).copied(), Some(40.0));
    assert_eq!(growth.get(&2026).copied(), Some(225.0));
    assert_eq!(growth.get(&2027).copied(), Some(250.0));

    // Every per-pupil figure the plan *pays*, which is not every column ending `_per_pupil`:
    // `median_weighted_wealth_per_pupil` also rose and is a measured wealth statistic rather
    // than a rate. Naming the rates rather than pattern-matching the suffix is the difference
    // between "the only rates that rose" and a claim that is false.
    const RATES: &[&str] = &[
        "average_base_cost_per_pupil",
        "career_technical_base_cost_per_pupil",
        "dpia_per_pupil",
        "performance_supplement_per_pupil",
        "base_funding_supplement_per_pupil",
        "enrolment_growth_supplement_per_pupil",
        "preschool_base_amount",
    ];
    let (moved, _) = prior::scalars_that_moved();
    let rose: Vec<&&str> = RATES
        .iter()
        .filter(|name| moved.contains(&(**name).to_string()))
        .collect();
    assert_eq!(
        rose,
        vec![
            &"base_funding_supplement_per_pupil",
            &"enrolment_growth_supplement_per_pupil"
        ],
        "{rose:?}"
    );

    // The threshold *did* move, and the other way: the act sets 5% of FY2022-FY2025 growth for
    // FY2026 and 3% of FY2023-FY2026 growth for FY2027. So the growth supplement rose in rate
    // and loosened in eligibility at once — and the calculator's `EGS pct` cell, 1 in both
    // years, is not that threshold whatever else it is.
    let cell = prior::scalar("enrolment_growth_supplement_pct_cell");
    assert_eq!(cell.len(), 2, "{cell:?}");
    assert!(
        cell.values().all(|value| (value - 1.0).abs() < 1e-9),
        "{cell:?}"
    );
}

#[test]
fn the_preschool_proration_is_this_years_appropriation_over_this_years_demand() {
    // `parameter/appropriation-proration-factor` left open whether the factor is measured or
    // delegated: "a factor being computable does not establish that it was computed rather than
    // set". In FY2026 it was computed, to eight decimal places.
    let (stated, computed) = prior::preschool_proration(2026).expect("FY2026 is in the fixture");
    assert!((stated - 0.968_538_11).abs() < 1e-9, "{stated}");
    assert!(
        (stated - computed).abs() < 5e-8,
        "stated {stated}, appropriation over demand {computed}"
    );

    // And the program lands on its appropriation, to $3.57 of $153,976,832.
    let (paid, appropriated) = prior::preschool_headroom(2026).expect("FY2026 is in the fixture");
    assert_eq!(appropriated, prior::PRESCHOOL_APPROPRIATION);
    assert!(
        (paid - appropriated).abs() < 10.0,
        "{paid} against {appropriated}"
    );
}

#[test]
fn and_in_the_year_the_calculator_models_it_is_not() {
    // FY2027's demand is below its appropriation, so its own arithmetic gives no proration at
    // all — and the workbook prorates anyway, by a factor within six millionths of FY2026's.
    let (stated, computed) = prior::preschool_proration(2027).expect("FY2027 is in the fixture");
    assert!((stated - 0.968_544_48).abs() < 1e-9, "{stated}");
    assert!(
        computed > 1.0,
        "FY2027 needs no proration but computes {computed}"
    );

    let (paid, appropriated) = prior::preschool_headroom(2027).expect("FY2027 is in the fixture");
    assert!(
        (appropriated - paid - 5_568_648.27).abs() < 1.0,
        "{paid} against {appropriated}"
    );

    let earlier = prior::preschool_proration(2026)
        .expect("FY2026 is in the fixture")
        .0;
    assert!(
        (stated - earlier).abs() < 1e-5 && (stated - earlier).abs() > 0.0,
        "FY2027 {stated} against FY2026 {earlier}"
    );
}

#[test]
fn the_transportation_proration_history_says_fy2027_is_the_first() {
    // Already committed, in `transportation-rates.csv`, and read by nothing when the node said
    // earlier years were open.
    let rates = include_str!("../fixtures/transportation-rates.csv");
    let column = |year: &str| -> f64 {
        rates
            .lines()
            .find(|line| line.starts_with(year))
            .and_then(|line| line.split(',').nth(3))
            .and_then(|cell| cell.parse::<f64>().ok())
            .unwrap_or_else(|| panic!("no FY{year} row"))
    };
    assert!((column("2026") - 1.0).abs() < 1e-12, "{}", column("2026"));
    assert!((column("2027") - 0.917_459_740_976_215).abs() < 1e-12);
}
