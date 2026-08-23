//! Nine fiscal years of the one education channel that enters no appropriation table.
//!
//! A 33% tax on gross casino revenue, 34% of which goes to the gross casino revenue county
//! student fund and is apportioned among the eighty-eight counties by resident public school
//! student population, then within each county among its districts. [R.C. 5753.03(D)(2),
//! R.C. 5753.11] It reaches every district in Ohio, is constitutionally restricted to primary and
//! secondary education, and is absent from the department's budget, from the School Finance
//! Payment Report, and from every fixture built from either — because it never passes through an
//! appropriation to the department at all. See
//! [`casino-tax-distribution`](../../../.yidam/corpus/revenue-stream/casino-tax-distribution.yml).
//!
//! # Four things that will produce a wrong reading
//!
//! **A district is paid once per county it has resident students in.** [`Distribution::counties`]
//! is that count, not a duplicate and not the district's home county. It is 1 for 294 of the 1,001
//! districts in the last distribution and **88** for three of them — Ohio Connections Academy,
//! Ohio Virtual Academy and Ohio Distance Education, e-schools whose students live everywhere.
//! Keying a join on IRN alone against the published sheets, rather than on (county, IRN), silently
//! collapses eighty-eight rows into one.
//!
//! **The grain is the payment, not the year.** Money moves twice a year and the halves are not
//! interchangeable: January-June 2020 came to $24.6 million against a $45-58 million norm, because
//! the casinos were closed by order from mid-March. An annual figure averages that away.
//!
//! **A fiscal year here is the year the money was *paid*, not the half-year it was earned on.**
//! [`Distribution::fiscal_year`] is the state fiscal year containing the payment month, which is
//! how the money lands in a district's books; [`Distribution::revenue_period`] is the half-year
//! the tax was collected in, which is a different pair of dates and one fiscal year earlier for
//! every August payment. Mixing the two shifts the closure from FY2021 to FY2020.
//!
//! **"Public school district" here includes community schools, STEM schools and JVSDs.**
//! [R.C. 5753.11(A)(1)] The 1,001 districts in the last distribution are not the 609 traditional
//! ones, and a per-pupil figure computed against a traditional-district denominator is wrong by
//! that difference.
//!
//! # What the series can and cannot say
//!
//! It settles the magnitude question the corpus node carried open for four phases, and it settles
//! it negatively: at **$114.2 million** in its largest fiscal year, the whole channel is under half
//! the $236 million median annual movement in constant-dollar foundation aid. See
//! `crates/project/tests/what_the_series_cannot_settle.rs`, where the two are compared.
//!
//! It stops at the January 2024 distribution because the department's casino page stops there.

use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = include_str!("../fixtures/casino-district-distributions.csv");

const EXPECTED_HEADER: &str = "irn,district,distribution,counties,amount";

/// What one district was paid in one distribution.
#[derive(Debug, Clone, PartialEq)]
pub struct Distribution {
    /// Six-digit IRN, zero-padded.
    pub irn: String,
    /// The district's name as the Department of Taxation writes it, which is not always how the
    /// Department of Education and Workforce writes it.
    pub district: String,
    /// The month the money was paid, `YYYY-MM`. Always `01` or `08`.
    pub month: String,
    /// How many county funds the district was paid out of.
    ///
    /// `None` for August 2015 through January 2017 read from the statewide sheets, where the
    /// department published district totals and no county breakdown.
    pub counties: Option<usize>,
    /// Dollars.
    pub amount: f64,
}

impl Distribution {
    /// The state fiscal year the payment falls in.
    ///
    /// July starts Ohio's fiscal year, so an August payment belongs to the fiscal year named for
    /// the calendar year *after* the half-year it was earned on. This is the basis on which the
    /// closure shows up in FY2021.
    #[must_use]
    pub fn fiscal_year(&self) -> u16 {
        let (year, month) = self.month.split_once('-').unwrap_or((&self.month, "01"));
        let year: u16 = year.parse().unwrap_or_default();
        if month == "08" {
            year + 1
        } else {
            year
        }
    }

    /// The half-year the tax was collected in, as `(start, end)` ISO dates.
    ///
    /// R.C. 5753.03(D)(2) pays in January for the half-year that ended in December and in August
    /// for the one that ended in June. Fifteen of the eighteen published sheets print this in
    /// their title banner and `connect::fixtures::build_casino_extract` checks every one of them
    /// against the rule before the fixture is written, so it is derived here rather than carried.
    #[must_use]
    pub fn revenue_period(&self) -> (String, String) {
        let (year, month) = self.month.split_once('-').unwrap_or((&self.month, "01"));
        let year: i32 = year.parse().unwrap_or_default();
        if month == "08" {
            (format!("{year}-01-01"), format!("{year}-06-30"))
        } else {
            (format!("{}-07-01", year - 1), format!("{}-12-31", year - 1))
        }
    }
}

/// Every row of the panel.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or if a row's width differs
/// from the header's — both by way of [`edfund_core::csv::rows`], which is what holds the
/// uniform-width invariant these fixtures are written under. A hand-rolled `split(',')` here
/// checked the header and then indexed by position, so a cell that grew a comma shifted every
/// field after it and the row still parsed.
#[must_use]
pub fn panel() -> Vec<Distribution> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            Some(Distribution {
                irn: row.str(0).to_string(),
                district: row.str(1).to_string(),
                month: row.str(2).to_string(),
                counties: row.str(3).parse().ok(),
                amount: row.num(4)?,
            })
        })
        .collect()
}

/// What each distribution came to, oldest first.
#[must_use]
pub fn by_month() -> BTreeMap<String, f64> {
    let mut out: BTreeMap<String, f64> = BTreeMap::new();
    for row in panel() {
        *out.entry(row.month).or_default() += row.amount;
    }
    out
}

/// What each **complete** fiscal year came to.
///
/// A fiscal year is complete when both of its payments are in the panel. Half a year is excluded
/// rather than reported low, which is the failure the series would otherwise invite at whichever
/// end the department next moves: a fiscal year missing its August payment reads as a 55% collapse
/// and looks exactly like the closure.
#[must_use]
pub fn by_fiscal_year() -> BTreeMap<u16, f64> {
    let mut months: BTreeMap<u16, BTreeSet<String>> = BTreeMap::new();
    let mut totals: BTreeMap<u16, f64> = BTreeMap::new();
    for row in panel() {
        let year = row.fiscal_year();
        months.entry(year).or_default().insert(row.month.clone());
        *totals.entry(year).or_default() += row.amount;
    }
    totals
        .into_iter()
        .filter(|(year, _)| months.get(year).is_some_and(|set| set.len() == 2))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_is_eighteen_half_years_without_a_gap() {
        let months: Vec<String> = by_month().into_keys().collect();
        assert_eq!(months.len(), 18);
        assert_eq!(months.first().map(String::as_str), Some("2015-08"));
        assert_eq!(months.last().map(String::as_str), Some("2024-01"));
        // Alternating January and August with nothing missing. A hole would not fail any total,
        // and it would take a fiscal year down by half.
        for pair in months.windows(2) {
            let (before, after) = (&pair[0], &pair[1]);
            let expected = if before.ends_with("-01") {
                format!("{}-08", &before[..4])
            } else {
                format!("{}-01", before[..4].parse::<u16>().unwrap() + 1)
            };
            assert_eq!(after, &expected, "gap after {before}");
        }
    }

    #[test]
    fn every_fiscal_year_in_the_panel_is_complete() {
        // The series happens to begin on an August and end on a January, so all nine fiscal years
        // have both halves. This is what makes `by_fiscal_year` return everything today; if the
        // department publishes again it will stop being true, and the filter is what keeps a
        // half-year from being reported as a year.
        assert_eq!(by_fiscal_year().len(), 9);
        assert_eq!(
            by_fiscal_year().keys().copied().collect::<Vec<_>>(),
            (2016..=2024).collect::<Vec<u16>>()
        );
    }

    #[test]
    fn the_closure_is_one_half_year_and_it_is_in_fiscal_2021() {
        let months = by_month();
        let closed = months["2020-08"];
        let after = months["2021-01"];
        // Not "below average" — outside the series. The other seventeen half-years run from
        // $42.9m to $58.7m, and this one is $18.3m below the lowest of them.
        let others: Vec<f64> = months
            .iter()
            .filter(|(month, _)| *month != "2020-08")
            .map(|(_, amount)| *amount)
            .collect();
        let floor = others.iter().copied().fold(f64::MAX, f64::min);
        assert!(
            closed < floor - 18_000_000.0,
            "{closed} against a floor of {floor}"
        );
        assert!(after > floor, "recovery within one half-year");
        // Paid in August 2020, which is FY2021 — not FY2020, which is the half-year it was earned
        // in. The two bases disagree about which year the pandemic hit by exactly one year.
        let years = by_fiscal_year();
        assert!(years[&2021] < years[&2020]);
        assert!(years[&2021] < years[&2022]);
    }

    #[test]
    fn a_district_is_paid_once_per_county_it_has_students_in() {
        let last: Vec<Distribution> = panel()
            .into_iter()
            .filter(|row| row.month == "2024-01")
            .collect();
        let spans: Vec<usize> = last.iter().filter_map(|row| row.counties).collect();
        assert_eq!(spans.len(), last.len(), "every 2024-01 row states a span");
        assert_eq!(spans.iter().copied().max(), Some(88));
        assert_eq!(spans.iter().filter(|span| **span == 1).count(), 294);
    }

    #[test]
    fn the_statewide_sheets_are_the_only_rows_without_a_county_span() {
        for row in panel() {
            let statewide = matches!(row.month.as_str(), "2016-01" | "2016-08" | "2017-01");
            assert_eq!(
                row.counties.is_none(),
                statewide,
                "{} {}",
                row.month,
                row.irn
            );
        }
    }

    #[test]
    fn a_fiscal_year_is_the_year_of_the_payment_and_the_revenue_period_is_not() {
        let august = Distribution {
            irn: "043802".into(),
            district: "COLUMBUS PUBLIC SCHOOL".into(),
            month: "2020-08".into(),
            counties: Some(1),
            amount: 1.0,
        };
        assert_eq!(august.fiscal_year(), 2021);
        assert_eq!(
            august.revenue_period(),
            ("2020-01-01".to_string(), "2020-06-30".to_string())
        );
        let january = Distribution {
            month: "2024-01".into(),
            ..august
        };
        assert_eq!(january.fiscal_year(), 2024);
        assert_eq!(
            january.revenue_period(),
            ("2023-07-01".to_string(), "2023-12-31".to_string())
        );
    }
}
