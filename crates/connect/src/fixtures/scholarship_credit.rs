//! The scholarship donation credit of R.C. 5747.73, measured twice by the Department of Taxation.
//!
//! The credit has no appropriation line, so no budget table in this repository can see it. What
//! it costs is revenue the general revenue fund never collects, and Taxation publishes that in two
//! documents that do not agree:
//!
//! - **Table Y-1**, the summary of individual income tax returns, carries a column for the credit
//!   in every tax year it has existed — the dollars filers claimed and the returns claiming them,
//!   by federal adjusted gross income class. [`sgo_credit_claims`] reads it.
//! - **The Tax Expenditure Report**, published with each executive budget, estimates the revenue
//!   forgone four fiscal years ahead. [`sgo_credit_estimates`] reads its entry for the credit.
//!
//! # Two layouts of one column
//!
//! Through TY2023 the table puts the line's name in one header cell over a two-cell `Value` /
//! `Number of Returns` subheader. From TY2024 the workbook was split into one sheet per schedule
//! and each column carries the whole name, `Line 15: Scholarship Donation Credit: Value`. Both are
//! found by the label rather than by a column letter — the column was `EW` in TY2021 and TY2022,
//! `FG` in TY2023 and `X` in TY2024.
//!
//! # The estimate is read from the entry, not the summary table
//!
//! Each report states every expenditure twice: a summary table at the front, and an entry with the
//! statute, a description and the estimates. The entry is anchored by its own citation, which is
//! what stays fixed — the credit is 2.25 in both editions held, and the summary's row label wraps
//! differently in each.

use super::format::format_value;

/// Columns of the claims panel: one row per tax year per income class, and one `Total` row each.
pub const SGO_CLAIMS_HEADER: &[&str] = &["tax_year", "income_class", "claimed", "returns"];

/// Columns of the estimates: one row per edition per fiscal year.
///
/// `estimate` is in millions of dollars, as the report prints it, to the report's own tenth.
pub const SGO_ESTIMATES_HEADER: &[&str] = &["edition", "fiscal_year", "estimate", "data_sources"];

/// The line's name, as every edition of Table Y-1 prints it.
const LINE: &str = "Line 15: Scholarship Donation Credit";

/// The citation the report's entry for the credit opens with.
const CITATION: &str = "R.C. 5747.73;";

/// An income class with the publisher's dollar signs and thousands separators removed, which the
/// fixture format cannot carry: `$5,001-$10,000` becomes `5001-10000`.
fn income_class(label: &str) -> String {
    label.replace(['$', ','], "").trim().to_string()
}

/// A cell the table suppressed for confidentiality, which it marks `*`.
fn suppressed(cell: &str) -> bool {
    cell.trim() == "*"
}

/// The cell at `row`, `column`, or empty.
fn cell(rows: &[Vec<String>], row: usize, column: usize) -> &str {
    rows.get(row)
        .and_then(|cells| cells.get(column))
        .map_or("", |text| text.trim())
}

/// A value or count cell as the fixture writes it: the number to the cent, or `*` kept verbatim.
fn amount(text: &str, tax_year: u16, what: &str) -> Result<String, String> {
    if suppressed(text) {
        return Ok("*".to_string());
    }
    text.parse::<f64>()
        .map(|value| format_value(Some(value), 2))
        .map_err(|_| format!("TY{tax_year}: {what} `{text}` is not a number"))
}

/// The scholarship donation credit's column of one tax year's Table Y-1.
///
/// `rows` is the sheet the column is on. Every income class is kept, suppressed cells as `*`, and
/// the table's own `Total` row last — the total includes the suppressed cells, so it is not
/// recomputable from the classes and is carried rather than derived.
///
/// # Errors
///
/// A layout error if the line is not on the sheet, if its two cells are not value and count, or
/// if there is not exactly one `Total` row.
pub fn sgo_credit_claims(rows: &[Vec<String>], tax_year: u16) -> Result<Vec<Vec<String>>, String> {
    let (header_row, value_column) = rows
        .iter()
        .enumerate()
        .find_map(|(r, cells)| {
            cells
                .iter()
                .position(|text| {
                    let text = text.trim();
                    text == LINE || text == format!("{LINE}: Value")
                })
                .map(|c| (r, c))
        })
        .ok_or_else(|| format!("TY{tax_year}: no column is labelled `{LINE}`"))?;
    let count_column = value_column + 1;

    // Which layout, and the row the income classes start below.
    let split = cell(rows, header_row, value_column) != LINE;
    let first_class = if split {
        let count_label = cell(rows, header_row, count_column);
        if count_label != format!("{LINE}: Number of Returns") {
            return Err(format!(
                "TY{tax_year}: the cell beside the credit's value reads `{count_label}`, not its \
                 return count"
            ));
        }
        header_row + 1
    } else {
        let (value, count) = (
            cell(rows, header_row + 1, value_column),
            cell(rows, header_row + 1, count_column),
        );
        if value != "Value" || count != "Number of Returns" {
            return Err(format!(
                "TY{tax_year}: the credit's subheader reads `{value}` / `{count}`, not `Value` / \
                 `Number of Returns`"
            ));
        }
        header_row + 2
    };

    let mut out = Vec::new();
    let mut totals = 0;
    for r in first_class..rows.len() {
        let label = cell(rows, r, 0);
        if label.is_empty() || label == "FAGI Income Class" {
            continue;
        }
        let total = label == "Total";
        if !total && !label.starts_with(['$', 'U', 'O']) {
            continue;
        }
        out.push(vec![
            tax_year.to_string(),
            if total {
                "Total".to_string()
            } else {
                income_class(label)
            },
            amount(cell(rows, r, value_column), tax_year, "a value")?,
            amount(cell(rows, r, count_column), tax_year, "a count")?,
        ]);
        if total {
            totals += 1;
            break;
        }
    }
    if totals != 1 {
        return Err(format!(
            "TY{tax_year}: the credit's column has no `Total` row"
        ));
    }
    Ok(out)
}

/// The tenths of a million `$ 50.5` is written as, or `None` for anything else.
fn millions(token: &str) -> Option<String> {
    let digits = token.trim_start_matches('$');
    let (whole, tenth) = digits.split_once('.')?;
    (!whole.is_empty()
        && whole.bytes().all(|b| b.is_ascii_digit())
        && tenth.len() == 1
        && tenth.bytes().all(|b| b.is_ascii_digit()))
    .then(|| digits.to_string())
}

/// The credit's entry in one edition of the Tax Expenditure Report, read from its text.
///
/// The entry opens with `R.C. 5747.73;` and is followed by an `Estimate:` line naming four fiscal
/// years, a line of four figures in millions, and a `Data Source Code:` line. All three are read
/// from the first of each after the citation; anything else in between is description.
///
/// # Errors
///
/// A layout error if the citation is absent, appears twice, or the lines after it are not four
/// fiscal years over four figures.
pub fn sgo_credit_estimates(text: &str, edition: &str) -> Result<Vec<Vec<String>>, String> {
    let lines: Vec<&str> = text.lines().collect();
    let entries: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_start().starts_with(CITATION))
        .map(|(i, _)| i)
        .collect();
    let [entry] = entries[..] else {
        return Err(format!(
            "{edition}: `{CITATION}` opens {} entries, not one",
            entries.len()
        ));
    };
    let after = |prefix: &str| {
        lines[entry..]
            .iter()
            .position(|line| line.trim_start().starts_with(prefix))
            .map(|i| entry + i)
            .ok_or_else(|| format!("{edition}: no `{prefix}` line follows the credit's entry"))
    };
    let years_line = after("Estimate:")?;
    let sources_line = after("Data Source Code:")?;

    let years: Vec<&str> = lines[years_line]
        .split_whitespace()
        .skip(1)
        .collect::<Vec<_>>()
        .chunks(2)
        .filter_map(|pair| match pair {
            ["FY", year] => Some(*year),
            _ => None,
        })
        .collect();
    let figures: Vec<String> = lines[years_line + 1]
        .replace('$', " ")
        .split_whitespace()
        .filter_map(millions)
        .collect();
    if years.len() != 4 || figures.len() != 4 || sources_line != years_line + 2 {
        return Err(format!(
            "{edition}: the credit's estimate reads {years:?} over {figures:?}, not four fiscal \
             years over four figures"
        ));
    }
    let sources = lines[sources_line]
        .trim()
        .trim_start_matches("Data Source Code:")
        .split(',')
        .map(str::trim)
        .collect::<Vec<_>>()
        .join("+");

    Ok(years
        .iter()
        .zip(figures)
        .map(|(year, figure)| {
            vec![
                edition.to_string(),
                (*year).to_string(),
                figure,
                sources.clone(),
            ]
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet(rows: &[&[&str]]) -> Vec<Vec<String>> {
        rows.iter()
            .map(|row| row.iter().map(|cell| (*cell).to_string()).collect())
            .collect()
    }

    #[test]
    fn the_merged_header_layout_reads_value_then_count() {
        let rows = sheet(&[
            &["Table Y-1", "", "", ""],
            &["", "Line 1", "Number of Returns", LINE],
            &["FAGI Income Class", "", "", "Value", "Number of Returns"],
            &["$5,001-$10,000", "1", "2", "*", "*"],
            &["Over $10,000,000", "1", "2", "41100", "33"],
            &["", "", "", "", ""],
            &["Total", "1", "2", "268616", "523"],
        ]);
        let read = sgo_credit_claims(&rows, 2021).expect("reads");
        assert_eq!(
            read,
            [
                ["2021", "5001-10000", "*", "*"],
                ["2021", "Over 10000000", "41100", "33"],
                ["2021", "Total", "268616", "523"],
            ]
        );
    }

    #[test]
    fn the_split_header_layout_reads_the_same_columns() {
        let rows = sheet(&[
            &[
                "FAGI Income Class",
                "Line 2: Retirement Income Credit: Value",
                "Line 15: Scholarship Donation Credit: Value",
                "Line 15: Scholarship Donation Credit: Number of Returns",
            ],
            &["Under $0", "1", "28", "2"],
            &["Total", "9", "35509908.41", "30584"],
        ]);
        let read = sgo_credit_claims(&rows, 2024).expect("reads");
        assert_eq!(read[0], ["2024", "Under 0", "28", "2"]);
        assert_eq!(read[1], ["2024", "Total", "35509908.41", "30584"]);
    }

    #[test]
    fn a_column_beside_the_value_that_is_not_its_count_is_a_layout_error() {
        let rows = sheet(&[
            &[
                "FAGI Income Class",
                "Line 15: Scholarship Donation Credit: Value",
                "Line 16",
            ],
            &["Total", "1", "2"],
        ]);
        assert!(sgo_credit_claims(&rows, 2024).is_err());
    }

    #[test]
    fn a_column_with_no_total_is_a_layout_error() {
        let rows = sheet(&[
            &["", LINE, ""],
            &["", "Value", "Number of Returns"],
            &["Under $0", "1", "2"],
        ]);
        assert!(sgo_credit_claims(&rows, 2022).is_err());
    }

    const ENTRY: &str = "\
2.25 Credit for donations to scholarship organizations
    R.C. 5747.73; originally enacted 2021, revised 2023.

    A maximum $750 non-refundable credit for cash donations.

        (Dollars in millions)
Estimate:              FY 2024        FY 2025          FY 2026          FY 2027
                       $   21.0       $   23.1         $   24.3         $   25.5
Data Source Code: B, C
";

    #[test]
    fn the_entry_reads_four_years_four_figures_and_the_sources() {
        let read = sgo_credit_estimates(ENTRY, "2026-27").expect("reads");
        assert_eq!(
            read,
            [
                ["2026-27", "2024", "21.0", "B+C"],
                ["2026-27", "2025", "23.1", "B+C"],
                ["2026-27", "2026", "24.3", "B+C"],
                ["2026-27", "2027", "25.5", "B+C"],
            ]
        );
    }

    #[test]
    fn an_entry_whose_figures_are_not_in_tenths_is_a_layout_error() {
        let doctored = ENTRY.replace("$   24.3", "Minimal");
        assert!(sgo_credit_estimates(&doctored, "2026-27").is_err());
    }

    #[test]
    fn a_citation_that_appears_twice_is_a_layout_error() {
        let doubled = format!("{ENTRY}{ENTRY}");
        assert!(sgo_credit_estimates(&doubled, "2026-27").is_err());
    }
}
