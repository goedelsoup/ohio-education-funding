//! The Title I half of the EdChoice criterion is a stale federal poverty estimate, and the
//! Ohio collections beside it decide 31 designations.
//!
//! R.C. 3310.03(A)(1)(b) designates a building whose district had "an average of twenty per cent
//! or more of the students entitled to attend school in the district … qualified to be included
//! in the formula to distribute funds under Title I". The designated list publishes the resulting
//! share and nothing behind it, so what the criterion *counts* was not a thing this corpus could
//! say. The workbook's `Title 1` sheets say it, and the answer is not what the criterion's name
//! suggests.
//!
//! **96.7% of the three-year formula count is census poverty** — the Census Bureau's Small Area
//! Income and Poverty Estimate, which each sheet dates in its own column heading to two years
//! before the school year it decides. The four categories Ohio collects each October come to
//! 3.3%, one of them contributes nothing in any year, and another goes to zero statewide between
//! the first edition and the second.
//!
//! That 3.3% is nonetheless decisive at the margin, because the four can only add: they move 16
//! districts over the twenty per cent bar and none under it, and six of those districts have
//! designated buildings.
//!
//! # And the district count is 611 for a reason that is really two reasons
//!
//! The designated list carries 606 districts and the report card 607. These sheets carry 611,
//! because a Title I formula count is a count of resident children and needs no building. One of
//! the five extra is excluded by statute from a list it is otherwise ranked for; the other four
//! are in no ranking and on no report card at all.

use dispersion::designated;
use dispersion::rankings;
use dispersion::report_card;
use dispersion::title1::{self, FormulaCount, ABSENT_FROM_THE_LIST, SCHOOL_YEARS};
use std::collections::BTreeSet;

/// R.C. 3310.03(A)(1)(b)'s threshold.
const THRESHOLD: f64 = designated::TITLE1_THRESHOLD;

fn totals(field: fn(&FormulaCount) -> Option<f64>) -> f64 {
    title1::formula_counts()
        .iter()
        .filter_map(field)
        .sum::<f64>()
}

/// What the criterion counts, by category, across the three years it averages.
///
/// The decomposition is the point. A reader told only that twenty per cent of a district's
/// children are "in the Title I formula" would reasonably picture an Ohio count of Ohio
/// children; it is a federal model-based estimate with a 3.3% Ohio garnish.
#[test]
fn the_formula_count_is_almost_entirely_census_poverty() {
    let total = totals(|c| c.total);
    let poverty = totals(|c| c.census_poverty);
    assert_eq!(total, 974_265.0);
    assert_eq!(poverty, 942_232.0);
    assert!(
        (poverty / total - 0.9671).abs() < 0.0001,
        "census poverty is {:.4} of the count",
        poverty / total
    );

    // The four Ohio collections, and what each of them is worth against that.
    assert_eq!(totals(|c| c.neglected), 2_740.0);
    assert_eq!(totals(|c| c.foster), 28_360.0);
    assert_eq!(totals(|c| c.tanf), 933.0);
    // Never once a positive number in three editions, whether written blank or written zero.
    assert_eq!(totals(|c| c.delinquent), 0.0);
}

/// One of the four columns empties out mid-panel, and the file gives no notice of it.
///
/// 930 children in 2023-2024, none in 2024-2025, three in 2025-2026. A component that goes to
/// zero statewide in one year is a change in a collection, not a change in Ohio's children, and
/// a reader averaging the three shares is averaging across it.
#[test]
fn the_tanf_component_goes_to_zero_statewide_between_editions() {
    for (year, children, districts) in [
        ("2023-2024", 930.0, 266),
        ("2024-2025", 0.0, 0),
        ("2025-2026", 3.0, 2),
    ] {
        let rows: Vec<FormulaCount> = title1::formula_counts()
            .into_iter()
            .filter(|c| c.school_year == year)
            .collect();
        assert_eq!(
            rows.iter().filter_map(|c| c.tanf).sum::<f64>(),
            children,
            "{year}"
        );
        assert_eq!(
            rows.iter()
                .filter(|c| c.tanf.is_some_and(|t| t > 0.0))
                .count(),
            districts,
            "{year}"
        );
    }
}

/// The 3.3% is not noise: it only ever adds, and it adds enough to matter 16 times.
///
/// The four Ohio components are non-negative, so a district's share on census poverty alone is a
/// lower bound on its formula share. The criterion is therefore strictly more generous than a
/// poverty rate would be, and the question is by how much.
#[test]
fn the_ohio_collections_only_ever_widen_eligibility_and_they_widen_it_for_sixteen_districts() {
    let mut poverty_only: std::collections::BTreeMap<String, Vec<f64>> =
        std::collections::BTreeMap::new();
    for count in title1::formula_counts() {
        let share = match (count.census_poverty, count.population_5_to_17) {
            (Some(poverty), Some(population)) if population > 0.0 => poverty / population,
            _ => 0.0,
        };
        poverty_only
            .entry(count.district_irn)
            .or_default()
            .push(share);
    }

    let full = title1::three_year_average();
    let mut moved = BTreeSet::new();
    for (irn, shares) in &poverty_only {
        #[allow(clippy::cast_precision_loss)]
        let mean = shares.iter().sum::<f64>() / shares.len() as f64;
        assert!(
            mean <= full[irn] + 1e-15,
            "{irn}: poverty alone exceeds the formula count"
        );
        if (mean >= THRESHOLD) != (full[irn] >= THRESHOLD) {
            moved.insert(irn.clone());
        }
    }
    assert_eq!(moved.len(), 16);
    assert_eq!(
        full.values().filter(|mean| **mean >= THRESHOLD).count(),
        134
    );

    let riding_on_the_collections: Vec<String> = designated::buildings()
        .into_iter()
        .filter(|b| moved.contains(&b.district_irn) && b.designated)
        .map(|b| b.district_irn)
        .collect();
    assert_eq!(riding_on_the_collections.len(), 31);
    assert_eq!(
        riding_on_the_collections
            .iter()
            .collect::<BTreeSet<&String>>()
            .len(),
        6
    );
}

/// Every input's vintage, as the department states it in its own column headings.
///
/// The census estimate is two years behind the school year and the October collection one. So the
/// 2023-2024 share — one of the three the 2026-2027 designation averages — rests on a poverty
/// estimate for 2021, five years before the scholarship it helps decide.
#[test]
fn the_criterion_rests_on_an_estimate_five_years_before_the_scholarship() {
    let oldest = title1::formula_counts()
        .iter()
        .map(|c| c.census_year)
        .min()
        .expect("a census year");
    assert_eq!(oldest, 2021);
    assert_eq!(SCHOOL_YEARS[0], "2023-2024");

    for count in title1::formula_counts() {
        let opens: u16 = count.school_year[..4].parse().expect("a school year");
        assert_eq!(count.census_year + 2, opens);
        assert_eq!(count.october_year + 1, opens);
    }
}

/// The five districts with a formula count and no place on the list, and why they are two groups.
///
/// Cleveland Municipal is ranked in 2025 and report-carded like any district, and is off the list
/// because R.C. 3310.03 puts it off. The other four are in no ranking sheet and on no report card:
/// they are districts that operate no building the department rates. Reporting "five districts
/// are missing" as one fact would merge a statutory exclusion with an absence of buildings.
#[test]
fn the_five_districts_the_list_does_not_carry_are_absent_for_two_different_reasons() {
    let with_counts: BTreeSet<String> = title1::formula_counts()
        .into_iter()
        .map(|c| c.district_irn)
        .collect();
    let on_the_list: BTreeSet<String> = designated::buildings()
        .into_iter()
        .map(|b| b.district_irn)
        .collect();
    assert_eq!(with_counts.len(), 611);
    assert_eq!(on_the_list.len(), 606);

    let absent: Vec<String> = with_counts.difference(&on_the_list).cloned().collect();
    let expected: BTreeSet<&str> = ABSENT_FROM_THE_LIST.into_iter().collect();
    assert_eq!(
        absent
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<&str>>(),
        expected
    );

    let ranked: BTreeSet<String> = rankings::rankings()
        .into_iter()
        .map(|r| r.lea_irn)
        .collect();
    let pilot = designated::PILOT_PROJECT_DISTRICT_IRN;

    // The pilot-project district is ranked and report-carded, and excluded all the same.
    assert!(ranked.contains(pilot));
    assert!(report_card::district(pilot).is_some());

    // The other four are nowhere the department rates a building.
    for irn in absent.iter().filter(|irn| *irn != pilot) {
        assert!(!ranked.contains(irn), "{irn} is ranked");
        assert!(
            report_card::district(irn).is_none(),
            "{irn} is report-carded"
        );
    }
    assert_eq!(report_card::report_cards().len(), 607);
}

/// Cleveland is sixth of 611 on the criterion it is excluded from.
///
/// The exclusion is not a reflection of the measure. R.C. 3310.03 puts the pilot-project district
/// outside the programme because it has a scholarship programme of its own, not because its
/// children fail the test — on the test they place sixth, at 40.0% against a 20% bar.
#[test]
fn the_excluded_district_is_sixth_on_the_criterion_that_excludes_nobody_else() {
    let averages = title1::three_year_average();
    let mut ordered: Vec<(&String, &f64)> = averages.iter().collect();
    ordered.sort_by(|left, right| right.1.total_cmp(left.1));
    let place = ordered
        .iter()
        .position(|(irn, _)| *irn == designated::PILOT_PROJECT_DISTRICT_IRN)
        .expect("the pilot-project district has a formula count");
    assert_eq!(place + 1, 6);
    let share = averages[designated::PILOT_PROJECT_DISTRICT_IRN];
    assert!((share - 0.4003).abs() < 0.0001, "{share}");
    assert!(share >= THRESHOLD);
}
