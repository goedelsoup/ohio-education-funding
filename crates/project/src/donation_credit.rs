//! The scholarship donation credit: R.C. 5747.73's income tax credit for gifts to a certified
//! scholarship granting organization, read from three fixtures that each answer a different
//! question.
//!
//! - **What taxpayers claimed.** Taxation's Y-1 tables, tax years 2021-2024, by Ohio adjusted
//!   gross income class with a `Total` row. Y-1 reports each credit "as claimed on each
//!   individual return before application to income or tax liability", so a total is an upper
//!   bound on revenue forgone, never the cost.
//! - **What Taxation estimated.** Two editions of the Tax Expenditure Report. Every figure in
//!   them is an estimate, in millions, and the two editions disagree by more than half.
//! - **Who could receive a gift.** The Attorney General's certification list, hand-extracted on
//!   [`RETRIEVED`], because the page carries a per-request token and cannot be pinned by digest.
//!
//! Nothing here reports a scholarship. The statute asks no organization to report what it awarded
//! or to whom, and no source held does.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use edfund_core::csv;

const CLAIMS: &str = include_str!("../fixtures/sgo-credit-claims.csv");
const ESTIMATES: &str = include_str!("../fixtures/sgo-credit-estimates.csv");
const CERTIFICATIONS: &str = include_str!("../fixtures/sgo-certifications.tsv");

const CLAIMS_HEADER: &str = "tax_year,income_class,claimed,returns";
const ESTIMATES_HEADER: &str = "edition,fiscal_year,estimate,data_sources";
const CERTIFICATIONS_HEADER: &str = "table_year\tein\tcertified\tvalid_through\tname\tdba\taddress";

/// The day the Attorney General's list was read, which is the only day the certification count
/// is true of.
pub const RETRIEVED: &str = "2026-10-05";

/// One income class's claims in one tax year, or the year's `Total` row.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    /// The tax year the returns were for.
    pub tax_year: u16,
    /// The Ohio AGI class as Y-1 prints it, without `$` or separators, or `Total`.
    pub income_class: String,
    /// Dollars claimed; `None` where Y-1 suppresses the cell with `*`.
    pub claimed: Option<f64>,
    /// Returns claiming the credit; `None` where suppressed.
    pub returns: Option<u32>,
}

/// One year of one Tax Expenditure Report edition.
#[derive(Debug, Clone, PartialEq)]
pub struct Estimate {
    /// The edition, named by the biennium it was written for: `2024-25` or `2026-27`.
    pub edition: String,
    /// The state fiscal year estimated.
    pub fiscal_year: u16,
    /// Estimated revenue forgone, in millions of dollars.
    pub millions: f64,
    /// The report's data source codes, joined with `+`.
    pub data_sources: String,
}

/// One row of the Attorney General's list. An organization recertified within a year can hold
/// two rows in one table.
#[derive(Debug, Clone, PartialEq)]
pub struct Certification {
    /// The year of the table the row sits in on the Attorney General's page.
    pub table_year: u16,
    /// The organization's employer identification number, which is the key.
    pub ein: String,
    /// The certification date, ISO.
    pub certified: String,
    /// The last day the certification holds, ISO.
    pub valid_through: String,
    /// The organization's legal name.
    pub name: String,
}

/// Every claims row, tax year then income class in Y-1's order.
///
/// # Panics
///
/// If the fixture's header has moved, or a cell that is not `*` is not a number.
#[must_use]
pub fn claims() -> &'static [Claim] {
    static CLAIMS_READ: OnceLock<Vec<Claim>> = OnceLock::new();
    CLAIMS_READ.get_or_init(|| read_claims(CLAIMS))
}

fn read_claims(text: &str) -> Vec<Claim> {
    csv::rows(text, CLAIMS_HEADER)
        .map(|row| {
            let cell = |i: usize| match row.str(i) {
                "*" => None,
                text => Some(
                    text.parse::<f64>()
                        .unwrap_or_else(|_| panic!("a claims cell is not a number: {text}")),
                ),
            };
            Claim {
                tax_year: row.str(0).parse().expect("tax year"),
                income_class: row.str(1).to_string(),
                claimed: cell(2),
                // Counts are whole; a fractional one would be a shifted column.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                returns: cell(3).map(|n| {
                    assert!(n.fract() == 0.0, "a return count is fractional: {n}");
                    n as u32
                }),
            }
        })
        .collect()
}

/// The `Total` row of a tax year, which Y-1 prints and this does not compute: suppressed classes
/// would make a computed sum short.
#[must_use]
pub fn total(tax_year: u16) -> Option<&'static Claim> {
    claims()
        .iter()
        .find(|claim| claim.tax_year == tax_year && claim.income_class == "Total")
}

/// The tax years held, earliest first.
#[must_use]
pub fn tax_years() -> Vec<u16> {
    claims()
        .iter()
        .map(|claim| claim.tax_year)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// Average claim on a return in a tax year, from the `Total` row.
#[must_use]
pub fn average_claim(tax_year: u16) -> Option<f64> {
    let total = total(tax_year)?;
    Some(total.claimed? / f64::from(total.returns?))
}

/// Every estimate, by edition then fiscal year.
///
/// # Panics
///
/// If the fixture's header has moved, or an estimate is not a number.
#[must_use]
pub fn estimates() -> &'static [Estimate] {
    static READ: OnceLock<Vec<Estimate>> = OnceLock::new();
    READ.get_or_init(|| {
        csv::rows(ESTIMATES, ESTIMATES_HEADER)
            .map(|row| Estimate {
                edition: row.str(0).to_string(),
                fiscal_year: row.str(1).parse().expect("fiscal year"),
                millions: row.num(2).expect("an estimate is a number"),
                data_sources: row.str(3).to_string(),
            })
            .collect()
    })
}

/// One edition's estimate for one fiscal year, in millions.
#[must_use]
pub fn estimate(edition: &str, fiscal_year: u16) -> Option<f64> {
    estimates()
        .iter()
        .find(|e| e.edition == edition && e.fiscal_year == fiscal_year)
        .map(|e| e.millions)
}

/// Every row of the Attorney General's list as retrieved on [`RETRIEVED`].
///
/// # Panics
///
/// If the fixture's header has moved, or a row's validity ends before it began.
#[must_use]
pub fn certifications() -> &'static [Certification] {
    static READ: OnceLock<Vec<Certification>> = OnceLock::new();
    READ.get_or_init(|| read_certifications(CERTIFICATIONS))
}

fn read_certifications(text: &str) -> Vec<Certification> {
    csv::delimited(text, CERTIFICATIONS_HEADER, '\t')
        .map(|row| {
            let certification = Certification {
                table_year: row.str(0).parse().expect("table year"),
                ein: row.str(1).to_string(),
                certified: row.str(2).to_string(),
                valid_through: row.str(3).to_string(),
                name: row.str(4).to_string(),
            };
            assert!(
                is_iso_date(&certification.certified) && is_iso_date(&certification.valid_through),
                "a certification date is not ISO, so the string comparisons below would misorder \
                 it: {certification:?}"
            );
            assert!(
                certification.valid_through >= certification.certified,
                "a certification ends before it begins: {certification:?}"
            );
            certification
        })
        .collect()
}

fn is_iso_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
}

/// Distinct organizations, by EIN, whose certification held on `date` (ISO): certified on or
/// before it and valid through it.
#[must_use]
pub fn certified_on(date: &str) -> usize {
    certifications()
        .iter()
        .filter(|c| c.certified.as_str() <= date && date <= c.valid_through.as_str())
        .map(|c| c.ein.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

/// Distinct organizations ever certified on the list.
#[must_use]
pub fn ever_certified() -> usize {
    certifications()
        .iter()
        .map(|c| c.ein.as_str())
        .collect::<BTreeSet<_>>()
        .len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_tax_years_each_with_one_total() {
        assert_eq!(tax_years(), vec![2021, 2022, 2023, 2024]);
        for year in tax_years() {
            let totals = claims()
                .iter()
                .filter(|c| c.tax_year == year && c.income_class == "Total")
                .count();
            assert_eq!(totals, 1, "tax year {year}");
        }
    }

    #[test]
    fn the_printed_total_is_at_least_the_unsuppressed_classes() {
        // Suppressed classes make the sum of the printed cells short of the total, never over
        // it. A sum over it means a class row was read twice or a column shifted.
        for year in tax_years() {
            let classes: f64 = claims()
                .iter()
                .filter(|c| c.tax_year == year && c.income_class != "Total")
                .filter_map(|c| c.claimed)
                .sum();
            let printed = total(year).and_then(|t| t.claimed).expect("a total");
            assert!(
                classes <= printed + 0.01,
                "{year}: classes {classes} over total {printed}"
            );
        }
    }

    #[test]
    fn the_total_check_catches_a_doubled_class() {
        // The guard above, against a doctored copy of real rows: TY2024's top class read twice.
        let mut text = String::from(CLAIMS_HEADER);
        text.push('\n');
        let rows: Vec<&str> = CLAIMS.lines().filter(|l| l.starts_with("2024,")).collect();
        let top = rows
            .iter()
            .filter(|l| !l.contains("Total") && !l.contains('*'))
            .max_by(|a, b| {
                let v = |l: &&&str| l.split(',').nth(2).unwrap().parse::<f64>().unwrap();
                v(a).total_cmp(&v(b))
            })
            .copied()
            .expect("a class row");
        for row in &rows {
            text.push_str(row);
            text.push('\n');
        }
        text.push_str(top);
        text.push('\n');
        let doctored = read_claims(&text);
        let classes: f64 = doctored
            .iter()
            .filter(|c| c.income_class != "Total")
            .filter_map(|c| c.claimed)
            .sum();
        let printed = doctored
            .iter()
            .find(|c| c.income_class == "Total")
            .and_then(|c| c.claimed)
            .expect("a total");
        assert!(
            classes > printed,
            "the doubled row did not push the sum over"
        );
    }

    #[test]
    fn a_suppressed_cell_is_an_absence() {
        let suppressed = claims()
            .iter()
            .find(|c| c.tax_year == 2021 && c.income_class == "40001-45000")
            .expect("the class");
        assert_eq!(suppressed.claimed, None);
        assert_eq!(suppressed.returns, None);
    }

    #[test]
    fn the_editions_disagree_by_more_than_half() {
        let early = estimate("2024-25", 2025).expect("2024-25 edition, FY2025");
        let late = estimate("2026-27", 2025).expect("2026-27 edition, FY2025");
        assert!((early - 50.5).abs() < 1e-9);
        assert!((late - 23.1).abs() < 1e-9);
        assert!(late < early / 2.0);
    }

    #[test]
    fn a_certification_that_ends_before_it_begins_fails_the_read() {
        let real = CERTIFICATIONS.lines().nth(1).expect("a row");
        let mut cells: Vec<&str> = real.split('\t').collect();
        cells.swap(2, 3);
        let text = format!("{CERTIFICATIONS_HEADER}\n{}\n", cells.join("\t"));
        let read = std::panic::catch_unwind(|| read_certifications(&text));
        assert!(
            read.is_err(),
            "a doctored row with its dates swapped was accepted"
        );
    }

    #[test]
    fn no_organization_was_certified_before_november_2021() {
        assert!(certifications()
            .iter()
            .all(|c| c.certified.as_str() >= "2021-11-01"));
        assert_eq!(certified_on("2021-10-31"), 0);
    }
}
