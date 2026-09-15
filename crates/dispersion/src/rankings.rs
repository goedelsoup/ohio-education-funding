//! Every building the department ranks by Performance Index, three years running.
//!
//! The three ranking sheets of the designated-list workbook, 9,080 rows: 2,937 buildings ranked
//! for 2023, 2,906 for 2024 and 3,237 for 2025. [`super::designated`] holds the *flags* these
//! produce; this holds the scores and the order they were cut from.
//!
//! # What this makes possible that the flags alone did not
//!
//! The designated list publishes a `Bottom 20% PI Ranking <year>` column and does not publish
//! what was ranked, so the determination could be checked for internal consistency and no
//! further. With the rankings beside it the cut itself is reproducible, and
//! [`bottom_twenty_percent`] reproduces it — exactly, in all three years and all 2,877 buildings.
//! See `crates/dispersion/tests/the_designation_rebuilt_from_its_inputs.rs`.
//!
//! # The rule is one line, and two of its three parts are invisible for two years in three
//!
//! Rank every **`Public School`** building operated by a district **other than the pilot-project
//! district** that carries an index that year; the lowest `ceil(0.20 × N)` are the year's flags.
//!
//! In 2023 and 2024 the filter does nothing: those sheets hold no community school, no STEM
//! school and no Cleveland building, so N is simply the sheet. In 2025 it removes 359 of 3,237
//! rows and is the whole difference between reproducing the flags and missing 163 of them.
//!
//! # The 2025 sheet ranks buildings the statute says shall not be ranked
//!
//! R.C. 3310.03(A)(1)(a) confines the ranking to "all buildings operated by city, local, and
//! exempted village school districts", and the division closes by directing that "the department
//! shall not include buildings operated by" the pilot-project district. The 2025 sheet holds 258
//! community schools, 8 STEM schools and Cleveland's other 93 buildings — 256 LEAs that appear in
//! neither of the other two years — and a community school holds rank 4.
//!
//! **The flag column is nonetheless right.** `ceil(0.20 × 3,230)` is 646 and the department
//! flagged 576, which is `ceil(0.20 × 2,877)` — the eligible population, not the sheet's. So the
//! department applied the statutory denominator and published a ranking that does not show it,
//! which is the same shape of defect [`super::designated`] records in the Option B column: the
//! rule applied is narrower than the file states.
//!
//! # A rank is not a key and an unranked building is not a low one
//!
//! Fourteen rows across the three years carry no rank and no index. They are excluded from the
//! population the percentile is taken of rather than sorted to the bottom of it — and the
//! distinction is load-bearing, because sorting them to the bottom would flag fourteen buildings
//! the department does not flag. Ranks also tie, so [`Ranking::rank`] identifies no row.

use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = include_str!("../fixtures/edchoice-pi-rankings.csv");

const EXPECTED_HEADER: &str = "ranking_year,rank,lea_irn,lea_name,building_irn,building_name,\
building_org_type,performance_index";

/// The three rankings the 2026-2027 designation is cut from, oldest first.
///
/// R.C. 3310.03(A)(1)(a)(iv) governs a scholarship sought for 2025-2026 "or any school year
/// thereafter" and asks for "the three most recent consecutive rankings", which is these. The
/// three earlier clauses name *pairs* of named years instead, so a list for 2024-2025 or before
/// could not be rebuilt from three rankings at all.
pub const RANKING_YEARS: [u16; 3] = [2023, 2024, 2025];

/// The share of buildings R.C. 3310.03(A)(1)(a) puts in the cut.
pub const BOTTOM_SHARE: f64 = 0.20;

/// The organisation type the statute's "city, local, and exempted village school districts"
/// leaves in the ranking.
///
/// The other two the 2025 sheet carries — `Community School` and `STEM` — are established under
/// Chapter 3314. and Chapter 3326. respectively and are operated by nobody's school district.
pub const DISTRICT_OPERATED: &str = "Public School";

/// One building's place in one year's ranking.
#[derive(Debug, Clone, PartialEq)]
pub struct Ranking {
    /// The year the ranking is of: 2023, 2024 or 2025.
    pub year: u16,
    /// The department's own rank, ascending from the lowest index. Ties, and absent for a
    /// building carrying no index.
    pub rank: Option<u32>,
    /// The operating LEA's IRN.
    pub lea_irn: String,
    /// The operating LEA's name. Not unique — key on the IRN.
    pub lea_name: String,
    /// The building's IRN, unique within a year.
    pub building_irn: String,
    /// The building's name.
    pub building_name: String,
    /// `Public School`, `Community School` or `STEM`.
    pub org_type: String,
    /// The index the ranking is on. Absent for fourteen rows across the three years.
    pub performance_index: Option<f64>,
}

impl Ranking {
    /// Whether the statute permits this building in the ranking it appears in.
    ///
    /// Both halves of R.C. 3310.03(A)(1)(a)'s population: operated by a city, local or exempted
    /// village district, and not operated by the pilot-project district. False for 359 of the
    /// 2025 sheet's 3,237 rows and for none of the other two years'.
    #[must_use]
    pub fn statutorily_ranked(&self) -> bool {
        self.org_type == DISTRICT_OPERATED
            && self.lea_irn != super::designated::PILOT_PROJECT_DISTRICT_IRN
    }
}

/// Every row of the three rankings.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or if a year or rank is not
/// a number — both by way of the extractor's own guarantees.
#[must_use]
pub fn rankings() -> Vec<Ranking> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .map(|row| Ranking {
            year: row.str(0).parse().expect("a ranking year"),
            rank: row.num(1).map(|rank| rank as u32),
            lea_irn: row.str(2).to_string(),
            lea_name: row.str(3).to_string(),
            building_irn: row.str(4).to_string(),
            building_name: row.str(5).to_string(),
            org_type: row.str(6).to_string(),
            performance_index: row.num(7),
        })
        .collect()
}

/// The buildings one year's ranking puts in the lowest twenty per cent, cut as the department
/// cuts it.
///
/// The population is the rows the statute permits and that carry an index; the cut is
/// `ceil(0.20 × N)` of it, taken on the index ascending. Reproduces the designated list's own
/// flag column exactly, in each of [`RANKING_YEARS`].
///
/// # Why the ceiling
///
/// The floor misses one building in 2023 and one in 2024 and is therefore not what the
/// department does. Nothing in R.C. 3310.03 says which way to round, and the workbook settles it:
/// 2,933 ranked buildings in 2023 give 586.6, and the 587th is flagged.
#[must_use]
pub fn bottom_twenty_percent(year: u16) -> BTreeSet<String> {
    let mut ranked: Vec<(f64, String)> = rankings()
        .into_iter()
        .filter(|r| r.year == year && r.statutorily_ranked())
        .filter_map(|r| r.performance_index.map(|pi| (pi, r.building_irn)))
        .collect();
    ranked.sort_by(|left, right| left.0.total_cmp(&right.0).then(left.1.cmp(&right.1)));
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    let cut = (ranked.len() as f64 * BOTTOM_SHARE).ceil() as usize;
    ranked.into_iter().take(cut).map(|(_, irn)| irn).collect()
}

/// How many of [`RANKING_YEARS`] put each building in the bottom twenty per cent.
///
/// The count R.C. 3310.03(A)(1)(a)(iv) tests against two. Buildings absent from a year's
/// ranking contribute nothing for it, which is the right reading: a building that did not exist
/// in 2023 was not ranked in the lowest twenty per cent in 2023.
#[must_use]
pub fn years_in_the_bottom() -> BTreeMap<String, usize> {
    let mut out: BTreeMap<String, usize> = BTreeMap::new();
    for year in RANKING_YEARS {
        for irn in bottom_twenty_percent(year) {
            *out.entry(irn).or_default() += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_sheets_are_the_sizes_the_catalog_records() {
        let all = rankings();
        assert_eq!(all.len(), 9_080);
        for (year, rows) in [(2023, 2_937), (2024, 2_906), (2025, 3_237)] {
            assert_eq!(
                all.iter().filter(|r| r.year == year).count(),
                rows,
                "{year}"
            );
        }
    }

    #[test]
    fn a_building_appears_at_most_once_in_a_year() {
        for year in RANKING_YEARS {
            let irns: Vec<String> = rankings()
                .into_iter()
                .filter(|r| r.year == year)
                .map(|r| r.building_irn)
                .collect();
            let distinct: BTreeSet<&String> = irns.iter().collect();
            assert_eq!(irns.len(), distinct.len(), "{year}");
        }
    }

    /// An unranked row is unranked in both columns, which is what lets the population be taken
    /// on the index alone.
    #[test]
    fn a_row_without_a_rank_is_a_row_without_an_index() {
        let unranked: Vec<Ranking> = rankings()
            .into_iter()
            .filter(|r| r.rank.is_none())
            .collect();
        assert_eq!(unranked.len(), 14);
        assert!(unranked.iter().all(|r| r.performance_index.is_none()));
    }

    #[test]
    fn only_the_2025_sheet_ranks_buildings_the_statute_excludes() {
        for (year, excluded) in [(2023, 0), (2024, 0), (2025, 359)] {
            let count = rankings()
                .iter()
                .filter(|r| r.year == year && !r.statutorily_ranked())
                .count();
            assert_eq!(count, excluded, "{year}");
        }
    }

    #[test]
    fn the_cut_is_a_fifth_of_the_population_the_statute_permits() {
        for (year, population, cut) in [(2023, 2_933, 587), (2024, 2_903, 581), (2025, 2_877, 576)]
        {
            let ranked = rankings()
                .iter()
                .filter(|r| {
                    r.year == year && r.statutorily_ranked() && r.performance_index.is_some()
                })
                .count();
            assert_eq!(ranked, population, "{year} population");
            assert_eq!(bottom_twenty_percent(year).len(), cut, "{year} cut");
        }
    }
}
