//! District five-year forecasts, reduced to the fields the finance panel carries.
//!
//! The one extractor whose input is not a workbook or a printed report but the parsed output of
//! [`crate::forecast`], which reads the department's filing format.

use super::format::format_value;

/// Header of the financial panel: one row per district per fiscal year.
pub const FINANCE_HEADER: &[&str] = &[
    "irn",
    "name",
    "county",
    "fiscal_year",
    "unrestricted_aid",
    "restricted_aid",
    "property_tax",
    "income_tax",
    "property_tax_allocation",
    "total_revenue",
    "total_revenue_and_sources",
    "total_expenditure",
    "beginning_cash",
    "ending_cash",
];

/// Reduce a set of five-year-forecast filings to one row per district per **actual** fiscal year.
///
/// # Only actuals, and only once each
///
/// A filing carries three prior actuals and five forecast years; only the actuals are kept, for
/// the reason in [`crate::forecast`] — a submitted forecast is a treasurer's projection made
/// under incentives, not a measurement.
///
/// The pinned filings are three years apart so their actual windows tile without overlapping,
/// and if that ever stops being true the later filing wins: a restatement is the department's
/// most recent word on a closed year. The seam between two filings is checkable — one's last
/// ending cash balance is the next's first beginning balance, to the dollar — and
/// `crates/project` tests exactly that.
///
/// Districts are emitted in IRN order, then fiscal year, so the fixture diffs cleanly.
#[must_use]
pub fn build_finance_extract(filings: &[Vec<crate::forecast::Line>]) -> Vec<Vec<String>> {
    use std::collections::BTreeMap;

    // (irn, fiscal year) -> (name, county, line code -> amount). BTreeMap so the output order is
    // the key order rather than a hash order that changes between builds.
    type Cell = (String, String, BTreeMap<String, f64>);
    let mut panel: BTreeMap<(String, u16), Cell> = BTreeMap::new();

    for filing in filings {
        for line in filing {
            for (index, fiscal_year) in line.actual_years().into_iter().enumerate() {
                let entry = panel
                    .entry((line.irn.clone(), fiscal_year))
                    .or_insert_with(|| (line.name.clone(), line.county.clone(), BTreeMap::new()));
                entry.0 = line.name.clone();
                entry.1 = line.county.clone();
                entry.2.insert(line.code.clone(), line.actual[index]);
            }
        }
    }

    panel
        .into_iter()
        .map(|((irn, fiscal_year), (name, county, amounts))| {
            let mut row = vec![irn, name, county, fiscal_year.to_string()];
            for code in crate::forecast::EXTRACTED {
                // Empty, not zero, where the filing has no such line. A district that did not
                // report line 5.050 did not spend nothing; writing `0` there makes an absence
                // indistinguishable from a measurement, and every reader downstream then sums
                // it. Toronto City's FY2023 filing omits 5.050, 7.010 and 7.020, and shipped
                // $0 expenditure against $9.86M of revenue for three years because of it.
                row.push(format_value(amounts.get(*code).copied(), 2));
            }
            row
        })
        .collect()
}
