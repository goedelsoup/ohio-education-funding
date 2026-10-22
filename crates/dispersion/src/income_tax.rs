//! Which districts levy a school district income tax, at what rate and on which base.
//!
//! Table SD-2-Q4 of the Department of Taxation for FY2024 and FY2025: every district Taxation
//! distributed income tax to under R.C. Chapter 5748 in the fiscal year, the rate, the year the tax
//! began, whether it is levied on the traditional base (Ohio modified adjusted gross income less
//! the R.C. 5747.025 exemptions) or the earned-income base (wages and self-employment earnings),
//! and the four distributions' total. See
//! [`school-district-income-tax`](../../../.yidam/corpus/revenue-stream/school-district-income-tax.yml).
//!
//! # Two things that will produce a wrong reading
//!
//! **A row is a payment, not a levy.** A tax that expired on 31 December still has withholding
//! to pay out, and two districts merged a decade ago — Ledgemont into Berkshire, Bettsville into
//! Old Fort — still receive late payments under their own IRNs. Counting rows counts 223
//! districts in FY2025; [`in_force`] counts 210.
//!
//! **The rate is the one in force on 1 January of the fiscal year**, so FY2025 is tax year 2025's
//! withholding for half the year and tax year 2024's annual returns for the rest. The table is
//! labelled by fiscal year and that is the label kept here.

use std::collections::BTreeMap;
use std::sync::OnceLock;

const FIXTURE: &str = include_str!("../fixtures/school-district-income-tax.csv");

const EXPECTED_HEADER: &str =
    "irn,district,county,fiscal_year,sd_number,year_began,rate,base,status,terms,distributed";

/// The base a tax is levied on, R.C. 5748.01(E).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Base {
    /// R.C. 5748.01(E)(1): modified adjusted gross income less the R.C. 5747.025 exemptions.
    Traditional,
    /// R.C. 5748.01(E)(2): wages, salaries, tips and self-employment earnings alone.
    Earned,
}

/// What a row of the table is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// A tax in force on the base given.
    InForce(Base),
    /// A tax that expired and is paying out its last withholding.
    Expired,
    /// A predecessor district merged into another, receiving late payments.
    Merged,
}

/// One district in one fiscal year of SD-2-Q4.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Six-digit IRN.
    pub irn: String,
    /// The district's name as Taxation writes it.
    pub district: String,
    /// The fiscal year the distributions were paid in.
    pub fiscal_year: u16,
    /// The calendar year the tax began.
    pub year_began: Option<u16>,
    /// Per cent of the base: `1.25` is 1.25%.
    pub rate: f64,
    /// In force on which base, expired, or merged.
    pub status: Status,
    /// The table's parenthetical, such as `expires 2028` or `continuing`.
    pub terms: String,
    /// Dollars distributed in the fiscal year.
    pub distributed: f64,
}

/// Every row of the panel.
///
/// # Panics
///
/// If the header is not the one this was written against, a row's width differs from it, or a
/// `status`/`base` pair is one the extract does not write.
#[must_use]
pub fn panel() -> &'static [Row] {
    static PANEL: OnceLock<Vec<Row>> = OnceLock::new();
    PANEL.get_or_init(parse)
}

fn parse() -> Vec<Row> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .map(|row| {
            let status = match (row.str(8), row.str(7)) {
                ("in-force", "traditional") => Status::InForce(Base::Traditional),
                ("in-force", "earned") => Status::InForce(Base::Earned),
                ("expired", "") => Status::Expired,
                ("merged", "") => Status::Merged,
                other => panic!("the SD-2 fixture has a status and base of {other:?}"),
            };
            Row {
                irn: row.str(0).to_string(),
                district: row.str(1).to_string(),
                fiscal_year: row.str(3).parse().expect("a fiscal year"),
                year_began: row.str(5).parse().ok(),
                rate: row.num(6).expect("a rate"),
                status,
                terms: row.str(9).to_string(),
                distributed: row.num(10).unwrap_or(0.0),
            }
        })
        .collect()
}

/// The districts with a tax in force in a fiscal year, by IRN, with the base.
#[must_use]
pub fn in_force(fiscal_year: u16) -> BTreeMap<String, Base> {
    panel()
        .iter()
        .filter(|row| row.fiscal_year == fiscal_year)
        .filter_map(|row| match row.status {
            Status::InForce(base) => Some((row.irn.clone(), base)),
            _ => None,
        })
        .collect()
}

/// How many districts levy on each base in a fiscal year.
#[must_use]
pub fn count_by_base(fiscal_year: u16) -> BTreeMap<Base, usize> {
    let mut out = BTreeMap::new();
    for base in in_force(fiscal_year).into_values() {
        *out.entry(base).or_default() += 1;
    }
    out
}

/// Everything distributed in a fiscal year, every row included.
#[must_use]
pub fn distributed(fiscal_year: u16) -> f64 {
    panel()
        .iter()
        .filter(|row| row.fiscal_year == fiscal_year)
        .map(|row| row.distributed)
        .sum()
}

/// Every district with a valuation per pupil in a tax year, and whether it levies the tax in a
/// fiscal year.
fn valuations(tax_year: u16, fiscal_year: u16) -> Vec<(String, f64, bool)> {
    let levying = in_force(fiscal_year);
    crate::sd1::rows()
        .into_iter()
        .filter(|row| row.tax_year == tax_year)
        .filter_map(|row| {
            let value = row.value_per_pupil?;
            let levies = levying.contains_key(&row.irn);
            Some((row.irn, value, levies))
        })
        .collect()
}

/// The median taxable value per pupil of the levying and the non-levying districts, in that
/// order, optionally restricted to the districts `keep` admits.
#[must_use]
pub fn median_valuation(
    tax_year: u16,
    fiscal_year: u16,
    keep: impl Fn(&str) -> bool,
) -> (Option<f64>, Option<f64>) {
    let rows = valuations(tax_year, fiscal_year);
    let pick = |levies: bool| {
        edfund_core::stats::median_interpolated(
            rows.iter()
                .filter(|(irn, _, l)| *l == levies && keep(irn))
                .map(|(_, value, _)| *value),
        )
    };
    (pick(true), pick(false))
}

/// The statewide valuation-per-pupil quartiles, lowest first: how many districts each holds, and
/// how many of those levy the tax.
///
/// The cuts are the order statistics at a quarter, a half and three quarters of the sorted
/// list, and a district at a cut belongs to the quarter above it.
#[must_use]
pub fn levying_by_valuation_quartile(tax_year: u16, fiscal_year: u16) -> [(usize, usize); 4] {
    let rows = valuations(tax_year, fiscal_year);
    let mut sorted: Vec<f64> = rows.iter().map(|(_, value, _)| *value).collect();
    sorted.sort_by(f64::total_cmp);
    let cuts: Vec<f64> = (1..4).map(|k| sorted[sorted.len() * k / 4]).collect();
    let mut out = [(0, 0); 4];
    for (_, value, levies) in &rows {
        let quarter = cuts.iter().filter(|cut| value >= cut).count();
        out[quarter].0 += 1;
        if *levies {
            out[quarter].1 += 1;
        }
    }
    out
}

/// Whether a district is rural or small-town in the department's typology.
#[must_use]
pub fn is_rural_or_small_town(irn: &str) -> bool {
    static TYPOLOGY: OnceLock<BTreeMap<String, crate::typology::District>> = OnceLock::new();
    TYPOLOGY
        .get_or_init(crate::typology::by_irn)
        .get(irn)
        .and_then(|district| district.typology.locale())
        .is_some_and(|locale| {
            matches!(
                locale,
                crate::typology::Locale::Rural | crate::typology::Locale::SmallTown
            )
        })
}

/// How many of the districts levying in a fiscal year are rural or small-town.
#[must_use]
pub fn rural_or_small_town_levying(fiscal_year: u16) -> usize {
    in_force(fiscal_year)
        .keys()
        .filter(|irn| is_rural_or_small_town(irn))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_row_is_a_payment_and_in_force_is_narrower() {
        let rows = panel().iter().filter(|r| r.fiscal_year == 2025).count();
        assert_eq!(rows, 223);
        assert_eq!(in_force(2025).len(), 210);
        assert_eq!(in_force(2024).len(), 212);
        // The two merged predecessors appear in both years and levy nothing.
        let merged: Vec<&str> = panel()
            .iter()
            .filter(|r| r.status == Status::Merged && r.fiscal_year == 2025)
            .map(|r| r.district.as_str())
            .collect();
        assert_eq!(merged, ["Ledgemont LSD", "Bettsville LSD"]);
    }

    #[test]
    fn the_bases_partition_the_districts_in_force() {
        let counts = count_by_base(2025);
        assert_eq!(counts[&Base::Earned], 64);
        assert_eq!(counts[&Base::Traditional], 146);
    }

    #[test]
    fn every_rate_sits_on_the_quarter_point_grid_the_statute_rounds_to() {
        // R.C. 5748.02(B)(1): the Tax Commissioner's estimate rounded to one-fourth of one per
        // cent. The extract refuses a rate off it, so this guards the reader, not the source.
        for row in panel() {
            assert!(
                (row.rate * 4.0 - (row.rate * 4.0).round()).abs() < 1e-9,
                "{row:?}"
            );
            assert!(row.rate > 0.0 && row.rate <= 2.0, "{row:?}");
        }
    }

    #[test]
    fn the_tax_is_rural_and_small_town_but_not_concentrated_at_the_bottom_of_valuation() {
        assert_eq!(rural_or_small_town_levying(2025), 193);
        let quartiles = levying_by_valuation_quartile(2024, 2025);
        // The lowest-valuation quarter levies less often than the second.
        assert_eq!(quartiles[0], (152, 35));
        assert_eq!(quartiles[1], (153, 85));
        let (levying, not) = median_valuation(2024, 2025, |_| true);
        assert!(levying.unwrap() < not.unwrap());
        let (levying, not) = median_valuation(2024, 2025, is_rural_or_small_town);
        // Within rural and small-town districts the gap is under 2%.
        assert!((not.unwrap() - levying.unwrap()) / not.unwrap() < 0.02);
    }

    #[test]
    fn an_expired_row_is_not_in_force() {
        // `in_force` must exclude a row by its status, not by anything else on it.
        let expired = panel()
            .iter()
            .find(|r| r.status == Status::Expired && r.fiscal_year == 2025)
            .expect("an expired row");
        assert!(!in_force(2025).contains_key(&expired.irn));
    }

    #[test]
    fn the_distributions_sum_to_the_printed_statewide_totals() {
        assert!((distributed(2025) - 737_718_964.22).abs() < 0.01);
        assert!((distributed(2024) - 680_506_527.29).abs() < 0.01);
    }
}
