//! Three editions of the designated list, and what changed between them.
//!
//! The department serves one edition of the EdChoice designated list at a time and deletes the
//! last, so eligibility has been a snapshot here: 2026-2027, and nothing to compare it against.
//! The 2024-2025 and 2025-2026 editions survive in the Internet Archive, and with them the first
//! *change* in scholarship eligibility this repository can measure — 460 designations, then 494,
//! then 513.
//!
//! What the pair of transitions finds is narrower than the growth and more useful. Every one of
//! the 80 buildings that entered the designated set across the two years entered through Option
//! B, the performance-index and Title I route; not one entered through Option A, the
//! academic-distress route. The academic-distress commissions move almost nobody. The route that
//! decides who may claim a traditional EdChoice scholarship is the derived one, and it has been
//! the only moving part for two consecutive years.
//!
//! The three editions also settle a reading that one edition could only suggest. The 2026-2027
//! file applies a condition it does not state — the building must be open — and with one file
//! that is a rule inferred from four rows. All three editions apply it, on 1, 7 and 4 rows
//! respectively, and in none of the three is a closed or inactive building ever designated.

use dispersion::designated_editions::{
    changes, districts_with_a_designation, edition, keyed, Change, Edition, EDITIONS,
};

/// Designated buildings in each edition, oldest first.
const DESIGNATED: [usize; 3] = [460, 494, 513];

/// Every building the edition lists, designated or not.
const LISTED: [usize; 3] = [2937, 2906, 2877];

#[test]
fn each_edition_carries_every_building_not_only_the_designated_ones() {
    for (which, listed) in EDITIONS.into_iter().zip(LISTED) {
        let rows = edition(which);
        assert_eq!(
            rows.len(),
            listed,
            "{} listed buildings",
            which.school_year()
        );
        let districts: std::collections::BTreeSet<_> =
            rows.iter().map(|d| d.district_irn.clone()).collect();
        assert_eq!(
            districts.len(),
            606,
            "{}: the pilot-project district is excluded in every edition, and only it",
            which.school_year()
        );
    }
}

#[test]
fn the_designated_set_grew_in_both_transitions() {
    for (which, count) in EDITIONS.into_iter().zip(DESIGNATED) {
        let designated = edition(which).into_iter().filter(|d| d.designated).count();
        assert_eq!(designated, count, "{} designations", which.school_year());
    }
    assert_eq!(
        [62, 68, 78],
        EDITIONS.map(|which| districts_with_a_designation(which).len()),
        "the districts holding at least one designated building"
    );
}

#[test]
fn every_building_that_entered_the_designated_set_entered_through_option_b() {
    for (before, after, entries) in [
        (Edition::Y2425, Edition::Y2526, 42),
        (Edition::Y2526, Edition::Y2627, 38),
    ] {
        let now = keyed(after);
        let entered: Vec<_> = changes(before, after)
            .into_iter()
            .filter(|(_, change)| *change == Change::Entered)
            .map(|(key, _)| now[&key].clone())
            .collect();
        assert_eq!(
            entered.len(),
            entries,
            "entries into {}",
            after.school_year()
        );
        assert!(
            entered.iter().all(|d| d.option_b),
            "{}: every entrant meets Option B",
            after.school_year()
        );
        assert_eq!(
            entered.iter().filter(|d| d.option_a).count(),
            0,
            "{}: no building entered through the academic-distress route",
            after.school_year()
        );
    }
}

#[test]
fn a_building_that_stops_being_designated_and_one_that_stops_being_listed_are_counted_apart() {
    for (before, after, undesignated, delisted) in [
        (Edition::Y2425, Edition::Y2526, 7, 1),
        (Edition::Y2526, Edition::Y2627, 18, 1),
    ] {
        let tally = dispersion::designated_editions::tally(before, after);
        assert_eq!(
            tally
                .get(&Change::Undesignated)
                .copied()
                .unwrap_or_default(),
            undesignated,
            "{}: designated before, on the list after, not designated",
            after.school_year()
        );
        assert_eq!(
            tally.get(&Change::Delisted).copied().unwrap_or_default(),
            delisted,
            "{}: designated before, gone from the list after",
            after.school_year()
        );
    }
}

#[test]
fn only_one_departure_in_two_years_ran_through_the_academic_distress_route() {
    let was = keyed(Edition::Y2526);
    let held_option_a = changes(Edition::Y2526, Edition::Y2627)
        .into_iter()
        .filter(|(_, change)| *change != Change::Entered)
        .filter(|(key, _)| was[key].option_a)
        .count();
    assert_eq!(held_option_a, 1);
    let was = keyed(Edition::Y2425);
    let held_option_a = changes(Edition::Y2425, Edition::Y2526)
        .into_iter()
        .filter(|(_, change)| *change != Change::Entered)
        .filter(|(key, _)| was[key].option_a)
        .count();
    assert_eq!(held_option_a, 0);
}

#[test]
fn the_condition_the_list_does_not_state_holds_in_all_three_editions() {
    // Read literally, R.C. 3310.03(A)(1) makes Option B the conjunction of its two criteria. The
    // workbook adds a third — the building is open — and says so nowhere. One edition made that
    // a reading off four rows; three editions make it the department's practice.
    for (which, disagreements) in EDITIONS.into_iter().zip([1, 7, 4]) {
        let rows = edition(which);
        let literal: Vec<_> = rows
            .iter()
            .filter(|d| d.meets_stated_option_b() != d.option_b)
            .collect();
        assert_eq!(
            literal.len(),
            disagreements,
            "{}: rows where the stated criteria disagree with the published Option B",
            which.school_year()
        );
        assert!(
            literal.iter().all(|d| !d.open()),
            "{}: every disagreement is a building that is not open",
            which.school_year()
        );
        assert_eq!(
            rows.iter().filter(|d| !d.open() && d.designated).count(),
            0,
            "{}: no closed or inactive building is designated",
            which.school_year()
        );
    }
}

#[test]
fn the_first_archived_edition_applies_the_two_of_three_test_to_two_rankings() {
    // R.C. 3310.03(A)(1)(a) asks for the bottom twenty per cent "for at least two of the three
    // most recent consecutive rankings". In 2024-2025 the department had two rankings to apply
    // it to, and its own column heading says so: `Bottom 20% PI Ranking Across 2022 and 2023
    // School Years`, against `Across Two of Prior Three Years` in both later editions. Two of
    // two is a materially easier test than two of three, and the fixture keeps the distinction
    // in the column name rather than flattening it.
    assert_eq!(EDITIONS.map(Edition::ranking_years), [2, 3, 3]);
    for which in EDITIONS {
        assert!(edition(which)
            .iter()
            .all(|d| d.bottom_20_by_year.len() == which.ranking_years()));
    }
}

#[test]
fn a_year_a_building_was_not_ranked_is_blank_rather_than_no() {
    // 25 cells in 2024-2025 and 42 in 2025-2026 are empty where a yes/no flag belongs: buildings
    // the department did not rank that year at all — online academies, early learning centres,
    // buildings that had not opened. The current edition writes `No` in all three of its year
    // columns and leaves nothing blank, so the distinction is one the department stopped making
    // rather than one it never made.
    assert_eq!(
        EDITIONS.map(|which| edition(which)
            .iter()
            .map(|d| d.unranked_years())
            .sum::<usize>()),
        [25, 42, 0]
    );
}

#[test]
fn the_window_column_reads_an_unranked_year_as_a_year_outside_the_bottom_fifth() {
    for which in EDITIONS {
        for d in edition(which) {
            assert_eq!(
                d.bottom_20_two_of_window,
                d.bottom_20_years() >= 2,
                "{} building {}",
                which.school_year(),
                d.building_irn
            );
        }
    }
}

#[test]
fn every_edition_reproduces_the_department_own_arithmetic_in_every_row() {
    for which in EDITIONS {
        for d in edition(which) {
            let where_ = format!("{} building {}", which.school_year(), d.building_irn);
            assert_eq!(
                d.designated,
                d.option_a || d.option_b,
                "{where_}: designation"
            );
            assert_eq!(
                d.option_b,
                d.meets_stated_option_b() && d.open(),
                "{where_}: Option B"
            );
            let mean = d.title1_shares.iter().sum::<f64>() / d.title1_shares.len() as f64;
            assert!(
                (mean - d.title1_average).abs() < 1e-12,
                "{where_}: the published average is the mean of the three shares"
            );
            assert_eq!(
                d.title1_at_least_20,
                d.title1_average >= 0.20,
                "{where_}: the Title I column tests the average, not each year"
            );
        }
    }
}
