//! Table SD-2-Q4 of the Department of Taxation: the school district income tax, by district.
//!
//! The one list of which districts levy a tax under R.C. Chapter 5748, at what rate and on which
//! base, published once a year as the fourth-quarter distribution table. A district appears on it
//! because money was distributed to it in the fiscal year, so the list is a little wider than the
//! set of districts with a tax in force: a tax that expired on 31 December still has a quarter of
//! withholding to pay out, and two districts that merged a decade ago still receive late
//! payments. [`build_sd2_extract`] keeps those rows and says what each one is.

use super::format::{clean_name, format_value};
use crate::conventions::{cell, cell_number, rows_by_key};

/// Header of the SD-2 panel: one row per district per **fiscal** year of distribution.
///
/// `status` is `in-force`, `expired` or `merged`. `base` is `earned` or `traditional` for a tax
/// in force and empty otherwise, because the table marks an expired or merged row with that flag
/// in place of the earned-income one.
pub const SD2_HEADER: &[&str] = &[
    "irn",
    "district",
    "county",
    "fiscal_year",
    "sd_number",
    "year_began",
    "rate",
    "base",
    "status",
    "terms",
    "distributed",
];

/// Column positions on the `SD2Q_FY…` worksheet. FY2024 and FY2025 agree on all of them.
mod sd2_columns {
    pub const COUNTY: usize = 0;
    pub const SD_NUMBER: usize = 1;
    pub const IRN: usize = 3;
    pub const NAME: usize = 4;
    pub const YEAR_BEGAN: usize = 5;
    pub const RATE: usize = 6;
    pub const FLAG: usize = 8;
    pub const TOTAL: usize = 14;
}

/// The label the table prints on its statewide row, in the name column.
const STATEWIDE: &str = "Statewide Total";

/// One fiscal year of SD-2-Q4.
#[derive(Debug, Clone, Copy)]
pub struct Sd2Year<'a> {
    /// The state fiscal year the four distributions were paid in.
    pub fiscal_year: u16,
    /// The worksheet's rows.
    pub rows: &'a [Vec<String>],
}

/// What a footnote flag in column I says about a row.
fn classify(flag: &str) -> Result<(&'static str, &'static str), String> {
    Ok(match flag.trim() {
        "" => ("traditional", "in-force"),
        "(c)" => ("earned", "in-force"),
        "(e)" => ("", "expired"),
        // Ledgemont into Berkshire in 2015 and Bettsville into Old Fort in 2014. The successor is
        // a row of its own, so the predecessor is a stream of late payments and not a levy.
        "(d)" | "(f)" => ("", "merged"),
        other => return Err(format!("SD-2: unrecognized footnote flag {other:?}")),
    })
}

/// Split `Bluffton EVSD (expires 2025)` into the name and the parenthetical terms.
fn split_terms(raw: &str) -> (String, String) {
    let cleaned = clean_name(raw);
    match cleaned.split_once(" (") {
        Some((name, rest)) => (
            name.to_string(),
            rest.trim_end_matches(')').replace(';', " /"),
        ),
        None => (cleaned, String::new()),
    }
}

/// Reduce SD-2-Q4 to one row per district per fiscal year.
///
/// Three checks, each of which turns a malformed year into an error rather than a fixture:
///
/// - **The districts sum to the printed statewide total**, to within a dollar. The table states
///   the aggregate beside its parts, so a dropped or doubled row is detectable from the sheet
///   alone.
/// - **Every rate is a multiple of a quarter of one per cent.** R.C. 5748.02(B)(1) and
///   5748.021 round the Tax Commissioner's estimate to the nearest one-fourth of one per cent, so
///   a rate that is not is a misread column.
/// - **No IRN appears twice in a year**, and every footnote flag is one the table defines.
///
/// # Errors
///
/// A message naming the year and the check that failed.
pub fn build_sd2_extract(years: &[Sd2Year<'_>]) -> Result<Vec<Vec<String>>, String> {
    use sd2_columns as c;

    let mut out: Vec<Vec<String>> = Vec::new();
    for year in years {
        let fy = year.fiscal_year;
        let printed = year
            .rows
            .iter()
            .find(|row| cell(row, c::NAME).trim() == STATEWIDE)
            .and_then(|row| cell_number(row, c::TOTAL))
            .ok_or_else(|| format!("SD-2 FY{fy}: no `{STATEWIDE}` row"))?;

        let mut seen = std::collections::BTreeSet::new();
        let mut sum = 0.0;
        for (key, row) in rows_by_key(year.rows, c::IRN) {
            let irn = format!("{key:0>6}");
            if !seen.insert(irn.clone()) {
                return Err(format!("SD-2 FY{fy}: IRN {irn} appears twice"));
            }
            let (base, status) = classify(cell(row, c::FLAG)).map_err(|e| format!("{e} FY{fy}"))?;
            let rate = cell_number(row, c::RATE)
                .ok_or_else(|| format!("SD-2 FY{fy}: IRN {irn} has no rate"))?;
            if ((rate * 4.0) - (rate * 4.0).round()).abs() > 1e-9 {
                return Err(format!(
                    "SD-2 FY{fy}: IRN {irn} rate {rate} is not a multiple of 0.25%"
                ));
            }
            let total = cell_number(row, c::TOTAL).unwrap_or(0.0);
            sum += total;
            let (name, terms) = split_terms(cell(row, c::NAME));
            out.push(vec![
                irn,
                name,
                clean_name(cell(row, c::COUNTY)),
                fy.to_string(),
                cell(row, c::SD_NUMBER).trim().to_string(),
                cell(row, c::YEAR_BEGAN).trim().to_string(),
                format_value(Some(rate), 2),
                base.to_string(),
                status.to_string(),
                terms,
                format_value(Some(total), 2),
            ]);
        }
        if (sum - printed).abs() > 1.0 {
            return Err(format!(
                "SD-2 FY{fy}: districts sum to {sum:.2}, the sheet prints {printed:.2}"
            ));
        }
    }
    out.sort_by(|a, b| a[0].cmp(&b[0]).then(a[3].cmp(&b[3])));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|c| (*c).to_string()).collect()
    }

    fn sheet() -> Vec<Vec<String>> {
        let blank = "";
        vec![
            row(&[
                "County",
                "District ID",
                "Unit No.",
                "IRN",
                "School District",
            ]),
            row(&[
                blank, blank, blank, blank, STATEWIDE, blank, blank, blank, blank, blank, blank,
                blank, blank, blank, "300.5",
            ]),
            row(&[
                "Allen",
                "203",
                "20540",
                "45211",
                "Bluffton EVSD (expires 2025)",
                "2008",
                "0.5",
                "%",
                blank,
                blank,
                blank,
                blank,
                blank,
                blank,
                "100.25",
            ]),
            row(&[
                "Ashland",
                "302",
                "22330",
                "45823",
                "Hillsdale LSD",
                "2014",
                "1.25",
                "%",
                "(c)",
                blank,
                blank,
                blank,
                blank,
                blank,
                "200",
            ]),
            row(&[
                "Athens",
                "501",
                "20040",
                "45906",
                "Alexander LSD (expired 2024)",
                "2020",
                "1",
                "%",
                "(e)",
                blank,
                blank,
                blank,
                blank,
                blank,
                "0.25",
            ]),
        ]
    }

    #[test]
    fn a_row_carries_its_base_status_and_terms() {
        let rows = sheet();
        let out = build_sd2_extract(&[Sd2Year {
            fiscal_year: 2025,
            rows: &rows,
        }])
        .unwrap();
        assert_eq!(out.len(), 3);
        assert_eq!(
            out[0],
            row(&[
                "045211",
                "Bluffton EVSD",
                "Allen",
                "2025",
                "203",
                "2008",
                "0.5",
                "traditional",
                "in-force",
                "expires 2025",
                "100.25"
            ])
        );
        assert_eq!(out[1][7..9], ["earned".to_string(), "in-force".to_string()]);
        assert_eq!(
            out[2][7..10],
            [String::new(), "expired".into(), "expired 2024".into()]
        );
    }

    #[test]
    fn a_dropped_row_fails_against_the_printed_total() {
        let mut rows = sheet();
        rows.pop();
        rows.pop();
        let err = build_sd2_extract(&[Sd2Year {
            fiscal_year: 2025,
            rows: &rows,
        }])
        .unwrap_err();
        assert!(err.contains("sum to"), "{err}");
    }

    #[test]
    fn a_rate_off_the_quarter_point_grid_is_a_misread_column() {
        let mut rows = sheet();
        rows[2][6] = "0.6".into();
        let err = build_sd2_extract(&[Sd2Year {
            fiscal_year: 2025,
            rows: &rows,
        }])
        .unwrap_err();
        assert!(err.contains("0.25%"), "{err}");
    }

    #[test]
    fn an_undefined_flag_is_refused() {
        let mut rows = sheet();
        rows[3][8] = "(g)".into();
        assert!(build_sd2_extract(&[Sd2Year {
            fiscal_year: 2025,
            rows: &rows
        }])
        .is_err());
    }
}
