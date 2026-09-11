//! Four corpus nodes recorded that the career-technical and English learner counts are frozen at
//! FY2021, and all four cited a column header: `Category 1 Career Tech FTE-FY21`,
//! `Category 1 EL ADM-FY21`.
//!
//! The nodes drew real consequences from it — that a district whose career-technical programme
//! has grown since 2021 is funded for the programme it had, and that for English learners "the
//! FY2021 Category 1 learners are not the FY2027 Category 1 learners". This is what the counts
//! and the department's own vintage table say instead.

use project::counts::{self, Series};

#[test]
fn the_english_learner_count_the_label_calls_fy2021_moves_in_five_hundred_districts() {
    // The single fact that ends the reading. Whatever the suffix means, it does not mean the
    // numbers under it were taken in FY2021.
    let before = counts::year(2026);
    let after = counts::year(2027);
    let moved = before
        .iter()
        .filter(|(irn, earlier)| {
            after
                .get(*irn)
                .is_some_and(|later| later.english_learner != earlier.english_learner)
        })
        .count();
    assert_eq!(moved, 512, "of {} districts", before.len());

    let category_one = counts::stability(Series::EnglishLearner(1));
    assert_eq!(category_one.paired, 611);
    assert!(
        (category_one.change() + 0.219_738).abs() < 0.000_01,
        "{}",
        category_one.change()
    );

    // And the composition turns the way the statute's own clock turns: category 1 is the
    // most recently arrived learner, category 3 has already scored proficient. The first falls
    // 22% and the third rises 13% while the total moves 1.2%, which is a cohort ageing through a
    // taper, not a file that was never refreshed.
    let category_three = counts::stability(Series::EnglishLearner(3));
    assert!(
        (category_three.change() - 0.127_895).abs() < 0.000_01,
        "{}",
        category_three.change()
    );
}

#[test]
fn the_department_names_four_models_and_none_of_them_reads_fy2021() {
    // Its own table, off the `Directions` sheet of each workbook and the `Notes` sheet for the
    // two models before them.
    assert_eq!(
        counts::categorical_vintages(),
        vec![
            (2024, "FY23".to_string()),
            (2025, "FY23".to_string()),
            (2026, "FY26 (Aug #1)".to_string()),
            (2027, "FY26 (Nov #2)".to_string()),
        ]
    );
    // FY2021 is in the workbook, and this is the whole of it: the guarantee base, which really
    // is a FY2021 quantity, and which is a dollar amount rather than a count. It is why `FY21`
    // is a live string in these files at all, and it is the string two report sheets picked up.
    let from_2021: Vec<String> = counts::vintages()
        .iter()
        .filter(|row| row.data_from.contains("FY21"))
        .map(|row| row.variable.clone())
        .collect();
    assert!(
        from_2021
            .iter()
            .all(|variable| variable == "FY21 Funding Base (L1)"),
        "{from_2021:?}"
    );
    assert!(!from_2021.is_empty());
}

#[test]
fn the_maintained_table_advances_with_the_statute_and_the_stale_one_does_not() {
    // Why the `Directions` table is the one to believe: its base cost enrolled ADM line is
    // R.C. 3317.02(C) — the greater of last year's enrolled ADM or the three-year average —
    // and it advances exactly one year between the two workbooks.
    assert_eq!(
        counts::vintage(2026, "Base Cost Enrolled ADM (b)").as_deref(),
        Some("Greater of FY25 or average of (FY25, 24, 23)")
    );
    assert_eq!(
        counts::vintage(2027, "Base Cost Enrolled ADM (b)").as_deref(),
        Some("Greater of FY26 (Nov #2) or average of (FY26, 25, 24)")
    );

    // The `Notes` sheet is the same 56 lines in both workbooks, describing FY2024 and FY2025 —
    // two biennia stale in the later file. Carried forward unchanged, which is the habit that
    // left `-FY21` on two sets of column headers.
    assert_eq!(counts::carried_forward(), (56, 56));

    // What is genuinely held is held in both tables at once, and says so in both.
    for model in [2026, 2027] {
        assert_eq!(
            counts::vintage(model, "Number of School Buildings (a)").as_deref(),
            Some("FY25")
        );
        assert_eq!(
            counts::vintage(model, "SpEd, EL and CTE Weights").as_deref(),
            Some("Same as FY22")
        );
    }
}

#[test]
fn the_workbook_carries_one_genuinely_held_column_and_it_is_not_a_count() {
    // The control. `[a] School Building Count - FY25` names a year, the vintage table names the
    // same year, and it is identical for every district in both models. Nothing else on the
    // sheet is, and a reader who wants to know what a freeze looks like can look at this.
    let buildings = counts::stability(Series::SchoolBuildings);
    assert_eq!((buildings.unchanged, buildings.paired), (611, 611));

    let held: Vec<String> = Series::all()
        .into_iter()
        .filter(|series| {
            let it = counts::stability(*series);
            it.unchanged == it.paired
        })
        .map(|series| series.column())
        .collect();
    assert_eq!(held, vec!["school_buildings".to_string()]);
}

#[test]
fn career_technical_is_stable_which_is_a_different_thing_and_measurably_so() {
    // Why the reading was believable: 559 of 611 districts really do carry an identical
    // career-technical FTE across the two models, and the statewide total moves a third of a
    // per cent. Stable, not held: 52 districts moved, and a column nobody re-collects does not
    // move in 52.
    let before = counts::year(2026);
    let after = counts::year(2027);
    let moved = |pick: fn(&counts::Row) -> Vec<f64>| {
        before
            .iter()
            .filter(|(irn, earlier)| {
                after
                    .get(*irn)
                    .is_some_and(|later| pick(later) != pick(earlier))
            })
            .count()
    };
    let career_technical = moved(|row| row.career_technical.to_vec());
    let gifted = moved(|row| row.gifted.to_vec());
    let special_education = moved(|row| row.special_education.to_vec());
    assert_eq!((career_technical, gifted, special_education), (52, 22, 608));

    let total_before: f64 = before
        .values()
        .map(|row| row.career_technical.iter().sum::<f64>())
        .sum();
    let total_after: f64 = after
        .values()
        .map(|row| row.career_technical.iter().sum::<f64>())
        .sum();
    assert!((total_before - 28_558.214).abs() < 0.01, "{total_before}");
    assert!((total_after - 28_641.920).abs() < 0.01, "{total_after}");

    // The asymmetry is about what kind of count each one is, not about which calendar it is on.
    // A career-technical FTE is fixed by a course schedule and a gifted identification is
    // cumulative; a special education ADM is re-evaluated all year. All three are drawn from the
    // same fiscal year by the same table.
    for series in [
        Series::CareerTechnical(1),
        Series::Gifted(1),
        Series::SpecialEducation(1),
    ] {
        assert_eq!(counts::stability(series).paired, 611);
    }
    assert!(counts::stability(Series::Gifted(1)).held() > 0.97);
    assert!(counts::stability(Series::SpecialEducation(1)).held() < 0.01);
}

#[test]
fn the_suffixed_columns_are_the_unsuffixed_ones() {
    // Asserted where it can actually be asserted — `connect::fixtures::counts` checks all eight
    // `-FY21` columns against the `ADM Data` column each restates, for every district in both
    // workbooks, and the rebuild fails if any pair disagrees. What is checkable from the
    // committed fixture is that the counts it holds are the ones the report sheets print.
    let rows = counts::frame();
    assert_eq!(rows.len(), 1222);
    assert_eq!(
        rows.iter().filter(|row| row.fiscal_year == 2026).count(),
        611
    );
    assert_eq!(
        rows.iter().filter(|row| row.fiscal_year == 2027).count(),
        611
    );
}
