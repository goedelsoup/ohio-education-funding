//! The two statewide medians targeted assistance runs on, and the second pair beside them.
//!
//! `formula-component/fsfp-targeted-assistance` closed with a question in two halves:
//!
//! > Whether the statewide medians are recomputed each year, and how far they move. Both are
//! > single numbers that set the whole scale, and the sheet publishes a second pair **excluding
//! > North Bass and Middle Bass** which no formula references. The exclusion is displayed and
//! > unused, which suggests it was considered.
//!
//! Both halves answer off the two published models.
//!
//! **They are recomputed, and they move a lot.** Median weighted wealth rises 8.2% and median
//! weighted wealth per pupil 9.2% in one year — a bigger move than anything on the cost side of
//! the plan, and the quantity the whole component is equalised to.
//!
//! **The exclusion was computed once and made no difference.** In the FY2026 model the excluding
//! pair equals the main pair to the last published digit: two districts out of 611 cannot move a
//! median, which is what a median is for. In the FY2027 model the excluding cells still hold the
//! **FY2026** numbers — so they are not an alternative calculation being maintained beside the
//! live one, they are last year's values left in place.
//!
//! That is the third instance of the same habit in this workbook, after the `-FY21` suffix on the
//! career-technical and English learner counts and the `FY21` on the transportation guarantee's
//! base. A cell in these files can be stale in three ways: a heading that no longer describes its
//! column, a table of vintages two biennia old, and a value nothing recomputes.

use project::prior_model as prior;

/// One scalar in one year, or a panic naming it.
fn at(name: &str, year: u16) -> f64 {
    *prior::scalar(name)
        .get(&year)
        .unwrap_or_else(|| panic!("{name} is not in the scalar fixture for FY{year}"))
}

#[test]
fn the_medians_are_recomputed_and_they_move_further_than_anything_on_the_cost_side() {
    let wealth = (
        at("median_weighted_wealth", 2026),
        at("median_weighted_wealth", 2027),
    );
    let per_pupil = (
        at("median_weighted_wealth_per_pupil", 2026),
        at("median_weighted_wealth_per_pupil", 2027),
    );
    assert!((wealth.0 - 362_326_436.80).abs() < 0.01, "{}", wealth.0);
    assert!((wealth.1 - 392_151_306.63).abs() < 0.01, "{}", wealth.1);
    assert!((per_pupil.0 - 253_337.38).abs() < 0.01, "{}", per_pupil.0);
    assert!((per_pupil.1 - 276_708.97).abs() < 0.01, "{}", per_pupil.1);

    let wealth_move = wealth.1 / wealth.0 - 1.0;
    let per_pupil_move = per_pupil.1 / per_pupil.0 - 1.0;
    assert!(
        (wealth_move - 0.082_314_9).abs() < 0.000_01,
        "{wealth_move}"
    );
    assert!(
        (per_pupil_move - 0.092_254_8).abs() < 0.000_01,
        "{per_pupil_move}"
    );

    // For scale: the statewide average base cost per pupil is the same number in both models,
    // and every weight with it. The scale this component equalises to moves nine per cent while
    // the scale the rest of the plan prices against does not move at all.
    let (_, held) = prior::scalars_that_moved();
    assert!(held.contains(&"average_base_cost_per_pupil".to_string()));
    assert!(held.contains(&"career_technical_base_cost_per_pupil".to_string()));
}

#[test]
fn the_exclusion_made_no_difference_in_the_year_it_was_computed() {
    // Two districts of 611, against a median. The pair is identical to the last digit the
    // department publishes, which is the answer to whether excluding them would change anything.
    assert!(
        (at("median_weighted_wealth_excluding_islands", 2026) - at("median_weighted_wealth", 2026))
            .abs()
            < 1e-6
    );
    assert!(
        (at("median_weighted_wealth_per_pupil_excluding_islands", 2026)
            - at("median_weighted_wealth_per_pupil", 2026))
        .abs()
            < 1e-9
    );
}

#[test]
fn and_in_the_later_model_it_is_the_earlier_models_number() {
    // The excluding cells did not move. They hold FY2026's main pair, to the last digit, beside a
    // main pair that moved eight and nine per cent — so nothing is recomputing them.
    assert!(
        (at("median_weighted_wealth_excluding_islands", 2027) - at("median_weighted_wealth", 2026))
            .abs()
            < 1e-6,
        "{} against {}",
        at("median_weighted_wealth_excluding_islands", 2027),
        at("median_weighted_wealth", 2026)
    );
    assert!(
        (at("median_weighted_wealth_per_pupil_excluding_islands", 2027)
            - at("median_weighted_wealth_per_pupil", 2026))
        .abs()
            < 1e-9
    );

    // And the pair it sits beside is a long way from it, so this is not two calculations agreeing.
    let drift = at("median_weighted_wealth", 2027)
        / at("median_weighted_wealth_excluding_islands", 2027)
        - 1.0;
    assert!((drift - 0.082_314_9).abs() < 0.000_01, "{drift}");
}

#[test]
fn the_two_islands_are_the_pair_every_population_question_in_this_corpus_turns_on() {
    // North Bass and Middle Bass are the two districts the department computes no base cost for,
    // which is why the panel holds 609 of 611. The exclusion cells are the department asking the
    // same question this corpus keeps having to answer, once, and then not asking it again.
    let scalars = prior::scalars();
    for year in [2026, 2027] {
        let row = &scalars[&year];
        assert!(row.contains_key("median_weighted_wealth_excluding_islands"));
        assert!(row.contains_key("median_weighted_wealth_per_pupil_excluding_islands"));
    }
    assert_eq!(prior::frame().len(), 611);
    assert_eq!(project::panel::panel().len(), 609);
}
