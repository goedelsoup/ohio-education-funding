//! The EdChoice designated list can be checked against its own criteria, and it does not pass.
//!
//! The department publishes the designation *and* every input to it, which is unusual and is what
//! makes this file worth more than a list of names: the determination is reproducible. Reproducing
//! it finds a fourth condition the workbook applies and states nowhere, and a column whose name
//! describes a rule other than the one it implements.
//!
//! Neither is a defect in the department's arithmetic. Both are places where a reader who trusts
//! the labels gets a different answer from the one the file gives, which is exactly the class of
//! thing this corpus exists to pin.

use dispersion::designated::{self, Building, PILOT_PROJECT_DISTRICT_IRN, TITLE1_THRESHOLD};

/// The designation is the disjunction of its two routes, with nothing left over.
#[test]
fn every_designated_building_arrives_by_one_of_the_two_options() {
    let all = designated::buildings();
    for building in &all {
        assert_eq!(
            building.designated,
            building.option_a || building.option_b,
            "{} is designated={} on A={} B={}",
            building.building_name,
            building.designated,
            building.option_a,
            building.option_b
        );
    }

    let count = |f: fn(&Building) -> bool| all.iter().filter(|b| f(b)).count();
    assert_eq!(count(|b| b.option_a && !b.option_b), 2);
    assert_eq!(count(|b| b.option_b && !b.option_a), 495);
    assert_eq!(count(|b| b.option_a && b.option_b), 16);
    assert_eq!(count(|b| b.designated), 513);
}

/// Option A is the academic distress route, and it is that column renamed.
///
/// Two districts, eighteen buildings. Worth pinning because the corpus holds
/// `dew-academic-distress-commission` as a catalog record and this is the first place an ADC
/// designation has a per-building consequence attached to it.
#[test]
fn option_a_is_the_academic_distress_column_under_another_name() {
    let all = designated::buildings();
    assert!(all.iter().all(|b| b.option_a == b.academic_distress));

    let districts: std::collections::BTreeSet<&str> = all
        .iter()
        .filter(|b| b.option_a)
        .map(|b| b.district_name.as_str())
        .collect();
    assert_eq!(
        districts.into_iter().collect::<Vec<&str>>(),
        ["East Cleveland City School District", "Youngstown City"]
    );
    assert_eq!(all.iter().filter(|b| b.option_a).count(), 18);
}

/// The finding: Option B is not its two stated criteria. It is those two and being open.
#[test]
fn option_b_needs_a_third_condition_the_workbook_never_names() {
    let all = designated::buildings();

    let stated: Vec<&Building> = all
        .iter()
        .filter(|b| b.meets_stated_option_b() != b.option_b)
        .collect();

    // Four buildings meet both published criteria and are marked `No` anyway.
    assert_eq!(stated.len(), 4);
    assert!(stated
        .iter()
        .all(|b| b.meets_stated_option_b() && !b.option_b));
    let mut names: Vec<&str> = stated.iter().map(|b| b.building_name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "Beallsville High School",
            "Highview 6th Grade Center",
            "Ripley Union Lewis Huntington Junior High School",
            "Willyard Elementary School",
        ]
    );

    // And every one of them is shut. Add that and the identity is exact across all 2,877 rows.
    assert!(stated.iter().all(|b| !b.open()));
    for building in &all {
        assert_eq!(
            building.option_b,
            building.meets_stated_option_b() && building.open(),
            "{} breaks the identity",
            building.building_name
        );
    }
}

/// No closed or inactive building is designated by either route.
#[test]
fn a_building_that_is_not_open_is_never_designated() {
    let all = designated::buildings();
    let shut: Vec<&Building> = all.iter().filter(|b| !b.open()).collect();
    assert_eq!(shut.len(), 15);
    assert!(shut.iter().all(|b| !b.designated));

    // Three statuses, and `Inactive` is its own — a building that is neither serving pupils nor
    // formally closed. One building carries it, and reading the column as a boolean would have
    // made it open.
    let statuses: std::collections::BTreeSet<&str> =
        all.iter().map(|b| b.open_closed.as_str()).collect();
    assert_eq!(
        statuses.into_iter().collect::<Vec<&str>>(),
        ["Closed", "Inactive", "Open"]
    );
}

/// The Title I column tests the three-year average, which is not what its name says.
///
/// R.C. 3310.03(A)(1)(b) requires "an average of twenty per cent or more" over three consecutive
/// years. The workbook's heading — *Title 1 Percent 20% or Above Across 3 Years* — reads like a
/// condition on each of the three. It is not, and the difference is 164 buildings.
#[test]
fn the_title_one_flag_is_an_average_and_not_a_floor_in_every_year() {
    let all = designated::buildings();

    for building in &all {
        assert_eq!(
            building.title1_at_least_20,
            building.title1_average >= TITLE1_THRESHOLD,
            "{} has an average of {}",
            building.building_name,
            building.title1_average
        );
    }

    let each_year = all
        .iter()
        .filter(|b| b.title1_at_least_20 != b.title1_shares.iter().all(|s| *s >= TITLE1_THRESHOLD))
        .count();
    assert_eq!(each_year, 164, "the misreading's cost has changed");
}

/// The average is the mean of the three shares beside it, and it is a district figure.
#[test]
fn the_published_average_is_the_mean_and_is_constant_within_a_district() {
    let all = designated::buildings();
    for building in &all {
        let mean = building.title1_shares.iter().sum::<f64>() / 3.0;
        assert!(
            (mean - building.title1_average).abs() < 1e-12,
            "{}: {mean} against {}",
            building.building_name,
            building.title1_average
        );
    }

    let mut per_district: std::collections::BTreeMap<&str, Vec<f64>> =
        std::collections::BTreeMap::new();
    for building in &all {
        per_district
            .entry(building.district_irn.as_str())
            .or_default()
            .push(building.title1_average);
    }
    for (irn, averages) in &per_district {
        let first = averages[0];
        assert!(
            averages.iter().all(|a| (a - first).abs() < 1e-12),
            "district {irn} carries more than one Title I average"
        );
    }
}

/// So two buildings in one district can differ only through their performance index.
///
/// Which is what makes the designation a building-level determination at all: the Title I half of
/// Option B is the same for every building the district operates.
#[test]
fn within_a_district_only_the_performance_index_separates_buildings() {
    let all = designated::buildings();
    let akron: Vec<&Building> = all.iter().filter(|b| b.district_irn == "043489").collect();
    assert_eq!(akron.len(), 42);
    assert_eq!(akron.iter().filter(|b| b.designated).count(), 30);

    // Every Akron building clears the district's Title I bar, and twelve are not designated.
    assert!(akron.iter().all(|b| b.title1_at_least_20));
    assert!(akron
        .iter()
        .filter(|b| !b.designated)
        .all(|b| !b.bottom_20_two_of_three || !b.open()));
}

/// Cleveland Municipal is not in the file, and the statute says why.
///
/// 606 districts here against the 607 the report card carries. R.C. 3310.03 excludes a district
/// where the pilot project scholarship programme operates — both from eligibility and from the
/// building ranking itself — and Cleveland is that district. So the one absence in a
/// six-hundred-row roster is a statutory exclusion made visible as a row count.
#[test]
fn the_pilot_project_district_is_excluded_by_statute_and_by_absence() {
    let designated_districts = designated::by_district();
    assert_eq!(designated_districts.len(), 606);
    assert!(!designated_districts.contains_key(PILOT_PROJECT_DISTRICT_IRN));

    // The other side of the comparison, from a file with no connection to this one.
    let report_card = dispersion::report_card::report_cards();
    assert_eq!(report_card.len(), 607);
    assert!(report_card
        .iter()
        .any(|d| d.irn == PILOT_PROJECT_DISTRICT_IRN));

    // And it is the only one missing, which is what makes the exclusion legible rather than
    // merely consistent with one.
    let missing: Vec<&str> = report_card
        .iter()
        .map(|d| d.irn.as_str())
        .filter(|irn| !designated_districts.contains_key(*irn))
        .collect();
    assert_eq!(missing, [PILOT_PROJECT_DISTRICT_IRN]);
}

/// Eligibility is concentrated: one district in eight has a designated building at all.
#[test]
fn seventy_eight_districts_hold_every_designated_building_in_the_state() {
    let per_district = designated::by_district();
    let with_any = per_district.values().filter(|(_, yes)| *yes > 0).count();
    assert_eq!(with_any, 78);
    assert!(with_any * 8 > per_district.len() && with_any * 7 < per_district.len());

    let designated: usize = per_district.values().map(|(_, yes)| yes).sum();
    assert_eq!(designated, 513);
}
