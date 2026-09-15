//! What each Ohio district's Title I formula count is made of, three years running.
//!
//! 611 districts × three school years, each row decomposing the count into the five categories
//! 20 U.S.C. 6333 adds — census poverty, neglected, delinquent, foster and TANF children — against
//! the district's 5-to-17 population. The first held source in this repository for Ohio's Title I
//! formula counts at all.
//!
//! # The input to a criterion the corpus held only the output of
//!
//! R.C. 3310.03(A)(1)(b) designates a building whose district had "an average of twenty per cent
//! or more" of its students in the Title I formula over three consecutive years.
//! [`super::designated`] carries the three shares and the average; this carries the numerator and
//! the denominator they are formed from, and [`FormulaCount::share`] recomputes them — to the last published
//! place, in all 1,818 district-years the two files share.
//!
//! # The criterion is a census poverty rate wearing a Title I name
//!
//! **96.7% of the three-year formula count is census poverty.** The four categories Ohio itself
//! collects each October come to 3.3% between them, and one of the four contributes nothing in
//! any of the three years: the delinquent column is blank in 2023-2024 and 2024-2025 and reads
//! zero for all 611 districts in 2025-2026. TANF collapses too — 930 children in 2023-2024, none
//! at all in 2024-2025, three in 2025-2026.
//!
//! That is not a rounding observation. The census estimate is the Census Bureau's Small Area
//! Income and Poverty Estimate, and each edition names its vintage in its own column heading:
//! the 2023-2024 formula count is built on a **2021** poverty estimate, two years stale, and the
//! October collection beside it is one year stale. So a criterion that decides which Ohio
//! children may take public money to a private school turns almost entirely on a federal
//! estimate of a year that ended before the school year began.
//!
//! The remaining 3.3% is not decorative. The four Ohio categories can only *add* to a count, so
//! they can only move a district over the twenty per cent bar and never under it — and they move
//! **16 districts** of 611 over it, six of which have designated buildings. **31 designations
//! rest on the 3.3%.**
//!
//! # Blank and zero are different, and the file changes its mind about which it writes
//!
//! The delinquent column is blank in the first two editions and an explicit `0` in the third.
//! The fixture keeps the difference rather than normalising it, on [`edfund_core::conventions`]'s
//! standing rule. What licenses reading the blanks as omissions is arithmetic rather than
//! assumption: the five components reach the published total in all 1,833 rows with the blanks
//! contributing nothing, which the extractor checks on every rebuild.
//!
//! # 611 districts against the designated list's 606, for two different reasons
//!
//! [`ABSENT_FROM_THE_LIST`] names the five. One of them is Cleveland Municipal, which is ranked
//! and report-carded like any district and excluded from the designated list by statute. The
//! other four are in no ranking sheet and on no report card: three Lake Erie island districts and
//! College Corner Local, whose children attend school in Indiana. A Title I formula count is a
//! count of *resident children*, so a district with no rated building still has one — North Bass
//! Local has a 5-to-17 population of zero and a row all the same.

use std::collections::BTreeMap;

const FIXTURE: &str = include_str!("../fixtures/title1-formula-counts.csv");

const EXPECTED_HEADER: &str = "school_year,census_year,october_year,district_irn,district_name,\
census_poverty,neglected,delinquent,foster,tanf,total_formula_count,population_5_to_17,\
title1_share";

/// The three school years the 2026-2027 designation averages, oldest first.
pub const SCHOOL_YEARS: [&str; 3] = ["2023-2024", "2024-2025", "2025-2026"];

/// The five districts with a Title I formula count and no row on the designated list.
///
/// Cleveland Municipal first, because its absence is the one with a statutory reason —
/// [`super::designated::PILOT_PROJECT_DISTRICT_IRN`]. The other four are absent from the ranking
/// sheets and the report card as well, and are not an exclusion at all: they operate no building
/// the department rates.
pub const ABSENT_FROM_THE_LIST: [&str; 5] = [
    "043786", // Cleveland Municipal — the pilot-project district, excluded by R.C. 3310.03
    "046797", // Kelleys Island Local
    "048959", // Middle Bass Local
    "048967", // North Bass Local
    "064964", // College Corner Local
];

/// One district's Title I formula count for one school year.
#[derive(Debug, Clone, PartialEq)]
pub struct FormulaCount {
    /// The school year the count is used for, as the workbook's share column names it.
    pub school_year: String,
    /// The year of the census poverty estimate the count is built on. Two behind
    /// [`Self::school_year`]'s first half in every edition.
    pub census_year: u16,
    /// The year of the October collection the other four components come from. One behind.
    pub october_year: u16,
    /// The district's IRN.
    pub district_irn: String,
    /// The district's name. Not unique across districts — key on the IRN.
    pub district_name: String,
    /// Children in poverty, as the Census Bureau estimates them for [`Self::census_year`].
    pub census_poverty: Option<f64>,
    /// Neglected children, from the October collection.
    pub neglected: Option<f64>,
    /// Delinquent children. Blank in the first two editions, zero in all 611 rows of the third.
    pub delinquent: Option<f64>,
    /// Children in foster care.
    pub foster: Option<f64>,
    /// Children in families receiving Temporary Assistance for Needy Families.
    pub tanf: Option<f64>,
    /// The department's own total. The five components above sum to it in every row.
    pub total: Option<f64>,
    /// The district's population aged five to seventeen. Zero for North Bass Local.
    pub population_5_to_17: Option<f64>,
    /// The published share, [`Self::total`] over [`Self::population_5_to_17`]. Written as zero
    /// rather than as an error where both are zero.
    pub published_share: Option<f64>,
}

impl FormulaCount {
    /// The share R.C. 3310.03(A)(1)(b) averages, recomputed rather than read.
    ///
    /// `None` where the district has no 5-to-17 population, which the workbook writes as a share
    /// of zero. Zero is a defensible thing for the department to print — a district with no
    /// resident children has no children in the formula either — but it is not a rate, and a
    /// mean taken over it would be one district's worth of made-up denominator.
    #[must_use]
    pub fn share(&self) -> Option<f64> {
        match (self.total, self.population_5_to_17) {
            (Some(total), Some(population)) if population > 0.0 => Some(total / population),
            _ => None,
        }
    }

    /// The four components Ohio collects each October, summed, treating an omitted one as none.
    ///
    /// Reading a blank as zero is licensed here and only here: these four reach the published
    /// total with the blanks contributing nothing, in every row, which the extractor enforces.
    #[must_use]
    pub fn ohio_collected(&self) -> f64 {
        [self.neglected, self.delinquent, self.foster, self.tanf]
            .into_iter()
            .flatten()
            .sum()
    }
}

/// Every district-year in the three sheets.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or if a vintage year is not
/// a number.
#[must_use]
pub fn formula_counts() -> Vec<FormulaCount> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .map(|row| FormulaCount {
            school_year: row.str(0).to_string(),
            census_year: row.str(1).parse().expect("a census year"),
            october_year: row.str(2).parse().expect("an October collection year"),
            district_irn: row.str(3).to_string(),
            district_name: row.str(4).to_string(),
            census_poverty: row.num(5),
            neglected: row.num(6),
            delinquent: row.num(7),
            foster: row.num(8),
            tanf: row.num(9),
            total: row.num(10),
            population_5_to_17: row.num(11),
            published_share: row.num(12),
        })
        .collect()
}

/// Each district's three-year mean share, the quantity R.C. 3310.03(A)(1)(b) tests.
///
/// Computed from the counts rather than from the published shares, and over districts holding
/// all three years — which is all 611 of them.
#[must_use]
pub fn three_year_average() -> BTreeMap<String, f64> {
    let mut shares: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for count in formula_counts() {
        shares
            .entry(count.district_irn.clone())
            .or_default()
            .push(count.share().unwrap_or(0.0));
    }
    shares
        .into_iter()
        .filter(|(_, years)| years.len() == SCHOOL_YEARS.len())
        .map(|(irn, years)| {
            #[allow(clippy::cast_precision_loss)]
            let mean = years.iter().sum::<f64>() / years.len() as f64;
            (irn, mean)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_is_three_years_of_every_district() {
        let all = formula_counts();
        assert_eq!(all.len(), 1_833);
        for year in SCHOOL_YEARS {
            assert_eq!(all.iter().filter(|c| c.school_year == year).count(), 611);
        }
        assert_eq!(three_year_average().len(), 611);
    }

    /// The department's own statement of how stale each half of the count is, and it does not
    /// move between editions.
    #[test]
    fn the_census_estimate_is_two_years_behind_and_the_collection_one() {
        for count in formula_counts() {
            let opens: u16 = count.school_year[..4].parse().expect("a school year");
            assert_eq!(count.census_year + 2, opens, "{}", count.district_irn);
            assert_eq!(count.october_year + 1, opens, "{}", count.district_irn);
        }
    }

    #[test]
    fn the_five_components_reach_the_published_total() {
        for count in formula_counts() {
            let summed = count.census_poverty.unwrap_or(0.0) + count.ohio_collected();
            assert_eq!(count.total, Some(summed), "{}", count.district_irn);
        }
    }

    /// The identity the two sheets of one workbook can be held to, and the reason the share is
    /// carried at fifteen places.
    ///
    /// The tolerance is the fixture's own last place and nothing looser. `write_csv` rounds a
    /// share to fifteen decimals, so a recomputed 0.3007114854072891 is written 0.300711485407289
    /// and reads back a half-ulp away — the difference between the two files, not a difference
    /// between the two quantities.
    #[test]
    fn the_recomputed_share_is_the_published_one() {
        const LAST_PLACE: f64 = 1e-15;
        for count in formula_counts() {
            match count.share() {
                Some(share) => {
                    let published = count.published_share.expect("a published share");
                    assert!(
                        (published - share).abs() <= LAST_PLACE,
                        "{} {}: {published} against {share}",
                        count.district_irn,
                        count.school_year
                    );
                }
                // North Bass Local, whose 5-to-17 population is zero in all three years. The
                // workbook writes a share of zero where the division is undefined.
                None => {
                    assert_eq!(count.population_5_to_17, Some(0.0));
                    assert_eq!(count.published_share, Some(0.0));
                }
            }
        }
    }

    #[test]
    fn the_delinquent_column_changes_from_blank_to_zero_between_editions() {
        for (year, blank, zero) in [
            ("2023-2024", 611, 0),
            ("2024-2025", 611, 0),
            ("2025-2026", 0, 611),
        ] {
            let rows: Vec<FormulaCount> = formula_counts()
                .into_iter()
                .filter(|c| c.school_year == year)
                .collect();
            assert_eq!(
                rows.iter().filter(|c| c.delinquent.is_none()).count(),
                blank,
                "{year}"
            );
            assert_eq!(
                rows.iter().filter(|c| c.delinquent == Some(0.0)).count(),
                zero,
                "{year}"
            );
        }
    }
}
