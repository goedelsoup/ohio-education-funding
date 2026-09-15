//! The 2026-2027 EdChoice designation, rebuilt from the raw counts, with no flag of the
//! department's used anywhere.
//!
//! `the_condition_the_designated_list_does_not_state` checks the `Overview` sheet against itself:
//! it takes the bottom-20% flags and the Title I average as given and asks whether `DESIGNATED`
//! follows from them. It does, once a third condition nobody wrote down is added. What that check
//! cannot do is ask whether the given flags are right, because the sheet does not say what was
//! ranked.
//!
//! The six other sheets of the same workbook do. This file rebuilds every flag from them —
//! 9,080 building-level Performance Index scores and 1,833 district Title I formula counts — and
//! reaches the department's answer for **all 2,877 buildings with no disagreement anywhere**:
//! the three yearly bottom-20% columns, the two-of-three column, the Title I threshold column,
//! Option B, and `DESIGNATED` itself.
//!
//! # The rule, entire
//!
//! For each of the three rankings, take the buildings whose organisation type is `Public School`
//! and whose district is not the pilot-project district and that carry an index; sort ascending;
//! the lowest `ceil(0.20 × N)` are that year's flags. A building flagged in at least two of the
//! three, in a district whose three-year mean Title I formula share is at least twenty per cent,
//! and which is open, is designated.
//!
//! # What had to be got right, and what each error looks like
//!
//! Three choices are each individually fatal and none of them is visible in the file:
//!
//! - **`ceil`, not `floor`.** The floor misses exactly one flagged building in 2023 and one in
//!   2024. Nothing in R.C. 3310.03 says which way to round and the workbook settles it.
//! - **The statutory population, not the sheet.** The 2025 sheet ranks 258 community schools,
//!   8 STEM schools and 93 Cleveland buildings, and R.C. 3310.03(A)(1)(a) admits none of them.
//!   Cutting 20% of the sheet as published flags 646 buildings and misses 163 of the 576.
//! - **An unranked building is out of the population, not at the bottom of it.** Fourteen rows
//!   carry no index. Sorting them below the lowest score flags fourteen buildings the department
//!   does not.
//!
//! # And one thing that is not an error
//!
//! The 2023 cut is 587 buildings and only 553 of them are flagged. The other 34 are not
//! disagreements: they are buildings that were in the lowest twenty per cent in 2023 and are not
//! on the 2026-2027 list at all, having closed since. The same holds for 15 buildings in 2024.
//! A reproduction that counted them as misses would be measuring the list's turnover, not the
//! rule.

use dispersion::designated;
use dispersion::rankings::{self, RANKING_YEARS};
use dispersion::title1;
use std::collections::BTreeSet;

/// R.C. 3310.03(A)(1)(a)(iv)'s "at least two of the three most recent consecutive rankings".
const RANKINGS_REQUIRED: usize = 2;

/// R.C. 3310.03(A)(1)(b)'s "an average of twenty per cent or more".
const TITLE1_THRESHOLD: f64 = designated::TITLE1_THRESHOLD;

/// The cut, year by year, against the flag column the department publishes beside it.
///
/// Each of the three is exact on the list's own buildings. The surplus is the buildings the cut
/// reaches that the list does not carry, and it falls to zero in 2025 — the year whose ranking
/// population and whose list population are the same 2,877.
#[test]
fn every_yearly_bottom_twenty_per_cent_flag_is_the_cut_it_claims_to_be() {
    let buildings = designated::buildings();
    for (index, year) in RANKING_YEARS.into_iter().enumerate() {
        let cut = rankings::bottom_twenty_percent(year);
        let flagged: BTreeSet<String> = buildings
            .iter()
            .filter(|b| b.bottom_20_by_year[index])
            .map(|b| b.building_irn.clone())
            .collect();
        let on_the_list: BTreeSet<String> =
            buildings.iter().map(|b| b.building_irn.clone()).collect();

        assert!(
            flagged.difference(&cut).next().is_none(),
            "{year}: {} flagged buildings are outside the cut",
            flagged.difference(&cut).count()
        );
        let surplus: Vec<&String> = cut.difference(&flagged).collect();
        assert!(
            surplus.iter().all(|irn| !on_the_list.contains(*irn)),
            "{year}: a building on the list is in the cut and not flagged"
        );
        let (flags, extra) = match year {
            2023 => (553, 34),
            2024 => (566, 15),
            _ => (576, 0),
        };
        assert_eq!(flagged.len(), flags, "{year} flags");
        assert_eq!(surplus.len(), extra, "{year} buildings since closed");
    }
}

/// The three-year count R.C. 3310.03(A)(1)(a)(iv) tests, rebuilt.
#[test]
fn the_two_of_three_column_is_the_three_cuts_counted() {
    let years = rankings::years_in_the_bottom();
    for building in designated::buildings() {
        let rebuilt = years.get(&building.building_irn).copied().unwrap_or(0);
        assert_eq!(
            rebuilt,
            building.bottom_20_years(),
            "{} is in {rebuilt} cuts and flagged for {}",
            building.building_name,
            building.bottom_20_years()
        );
        assert_eq!(
            rebuilt >= RANKINGS_REQUIRED,
            building.bottom_20_two_of_three,
            "{}",
            building.building_name
        );
    }
}

/// The district criterion, rebuilt from the formula counts rather than read off the shares.
#[test]
fn the_title_one_column_is_the_formula_counts_averaged() {
    let averages = title1::three_year_average();
    for building in designated::buildings() {
        let mean = averages
            .get(&building.district_irn)
            .copied()
            .unwrap_or_else(|| panic!("{} has no formula count", building.district_name));
        assert!(
            (mean - building.title1_average).abs() <= 1e-15,
            "{}: rebuilt {mean} against published {}",
            building.district_name,
            building.title1_average
        );
        assert_eq!(
            mean >= TITLE1_THRESHOLD,
            building.title1_at_least_20,
            "{}",
            building.district_name
        );
    }
}

/// The whole determination, from the six sheets that are not the `Overview`.
///
/// The claim this file exists to make. Option B is rebuilt from the rankings and the formula
/// counts and the open/closed status — the fourth condition
/// `the_condition_the_designated_list_does_not_state` recovered — and `DESIGNATED` from Option B
/// and the academic-distress route. Both are exact in all 2,877 rows.
///
/// Option A is read rather than rebuilt, and that is the one remaining gap: the academic distress
/// commission is a designation of the state board's that this workbook records and does not
/// derive. It decides two buildings on its own.
#[test]
fn the_designation_follows_from_the_raw_counts_in_every_row() {
    let years = rankings::years_in_the_bottom();
    let averages = title1::three_year_average();
    let mut designated_count = 0;
    let buildings = designated::buildings();
    for building in &buildings {
        let two_of_three =
            years.get(&building.building_irn).copied().unwrap_or(0) >= RANKINGS_REQUIRED;
        let title1 = averages[&building.district_irn] >= TITLE1_THRESHOLD;
        let option_b = two_of_three && title1 && building.open();
        assert_eq!(
            option_b, building.option_b,
            "Option B: {}",
            building.building_name
        );

        let rebuilt = option_b || building.option_a;
        assert_eq!(rebuilt, building.designated, "{}", building.building_name);
        designated_count += usize::from(rebuilt);
    }
    assert_eq!(buildings.len(), 2_877);
    assert_eq!(designated_count, 513);
}

/// The three choices above, each shown to be load-bearing by making it wrongly.
///
/// A reproduction that matches proves nothing on its own — a rule with slack in it matches too.
/// These are the three places the slack would be.
#[test]
fn each_of_the_three_readings_the_file_does_not_state_is_decisive() {
    let all = rankings::rankings();
    let buildings = designated::buildings();
    let flagged = |index: usize| -> BTreeSet<String> {
        buildings
            .iter()
            .filter(|b| b.bottom_20_by_year[index])
            .map(|b| b.building_irn.clone())
            .collect()
    };
    let cut_of = |rows: Vec<(f64, String)>, round: fn(f64) -> f64| -> BTreeSet<String> {
        let mut rows = rows;
        rows.sort_by(|left, right| left.0.total_cmp(&right.0).then(left.1.cmp(&right.1)));
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation
        )]
        let take = round(rows.len() as f64 * 0.20) as usize;
        rows.into_iter().take(take).map(|(_, irn)| irn).collect()
    };
    let statutory = |year: u16| -> Vec<(f64, String)> {
        all.iter()
            .filter(|r| r.year == year && r.statutorily_ranked())
            .filter_map(|r| r.performance_index.map(|pi| (pi, r.building_irn.clone())))
            .collect()
    };

    // Rounding down misses one flagged building in each of the two earlier years.
    for (index, year) in [(0, 2023), (1, 2024)] {
        let floored = cut_of(statutory(year), f64::floor);
        assert_eq!(flagged(index).difference(&floored).count(), 1, "{year}");
    }

    // Cutting the 2025 sheet as published, rather than the population the statute names.
    let published: Vec<(f64, String)> = all
        .iter()
        .filter(|r| r.year == 2025)
        .filter_map(|r| r.performance_index.map(|pi| (pi, r.building_irn.clone())))
        .collect();
    let naive = cut_of(published, f64::ceil);
    assert_eq!(naive.len(), 646);
    assert_eq!(flagged(2).difference(&naive).count(), 163);

    // Sorting the unranked below the lowest score instead of dropping them.
    let mut with_unranked = statutory(2025);
    let unranked = all
        .iter()
        .filter(|r| r.year == 2025 && r.statutorily_ranked() && r.performance_index.is_none())
        .map(|r| (f64::NEG_INFINITY, r.building_irn.clone()));
    with_unranked.extend(unranked);
    let sunk = cut_of(with_unranked, f64::ceil);
    assert_eq!(sunk.difference(&flagged(2)).count(), 1);
}
