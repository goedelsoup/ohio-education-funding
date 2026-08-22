//! The casino tax distribution to school districts, quarterly, from the Department of Taxation.
//!
//! Two published layouts of the same money — one row per district and one row per district *per
//! county* — and this reconciles them district by district rather than trusting either.
//!
//! The date handling is the subtle part. A distribution is named for the month it is paid, and
//! the statutory period it covers is the half year *before* that. The printed banner states the
//! period and the payment month implies it, so both are read and a disagreement is a failure
//! rather than a note.

use std::collections::BTreeMap;

use super::format::{clean_name, format_value};
use crate::conventions::{cell, cell_number};

/// Header of the casino panel: one row per district per **distribution**.
///
/// The grain is the payment, not the year. Money moves twice a year under R.C. 5753.03(D)(2) and
/// the halves are not interchangeable: the closure that took January-June 2020 down to a third of
/// a normal half-year is invisible in any annual figure that averages it with the half beside it.
///
/// `counties` is how many county funds a district was paid out of, and it is empty for the three
/// distributions the department published only as a statewide list. It is not a district's county
/// — a district appears once per county it has resident students in, which for the statewide
/// e-schools is all 88.
pub const CASINO_HEADER: &[&str] = &["irn", "district", "distribution", "counties", "amount"];

/// One published sheet of one distribution.
#[derive(Debug, Clone, Copy)]
pub struct CasinoSheet<'a> {
    /// The month the money was paid, `YYYY-MM`. January or August, and it is the identity of a
    /// distribution: the revenue half-year follows from it by statute.
    pub distribution: &'a str,
    /// What to call this sheet when a check on it fails.
    pub label: &'a str,
    /// Its rows.
    pub rows: &'a [Vec<String>],
}

/// Which of the two layouts the department used for a sheet.
///
/// Detected from the header row rather than declared per file, because the layout is a fact about
/// the sheet and the department changed it twice inside one series. Column *positions* are read
/// off the header too: the amount sits in `E` in most years and in `G` in the statewide layout and
/// in the one true-up year, and a fixed index would read the wrong column in three of eighteen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CasinoLayout {
    /// County code, county name, IRN, district name, amount. One row per district per county.
    ByCounty {
        county: usize,
        irn: usize,
        name: usize,
        amount: usize,
    },
    /// IRN, district name, amount. One row per district, statewide.
    ByDistrict {
        irn: usize,
        name: usize,
        amount: usize,
    },
}

/// The row the department puts at the foot of the data block, carrying its own total.
///
/// It is the reason a digit filter is not optional. The row's amount sits in the amount column
/// like every district's, so a parser that sums the column without checking what is in the key
/// column reports exactly twice the distribution — and $90 million for a half-year is not
/// obviously wrong to anyone who has not seen the series.
const CASINO_TOTAL_ROW: &str = "Total Distribution Amount";

/// Find the header row and what layout it describes.
fn casino_layout(rows: &[Vec<String>]) -> Option<(usize, CasinoLayout)> {
    for (index, row) in rows.iter().enumerate() {
        let Some(irn) = row.iter().position(|cell| cell.trim() == "IRN") else {
            continue;
        };
        // The amount column is the *last* header cell naming a distribution. In the true-up year
        // there are three, and the first two are the halves of the third.
        let Some(amount) = row
            .iter()
            .rposition(|cell| cell.trim().contains("istr") && cell.trim().contains("ution"))
        else {
            continue;
        };
        let county = row.iter().position(|cell| cell.trim() == "County Code");
        let layout = match county {
            Some(county) => CasinoLayout::ByCounty {
                county,
                irn,
                name: irn + 1,
                amount,
            },
            None => CasinoLayout::ByDistrict {
                irn,
                name: irn + 1,
                amount,
            },
        };
        return Some((index, layout));
    }
    None
}

/// The half-year a distribution paid for, from the month it was paid in.
///
/// R.C. 5753.03(D)(2) transfers the tax quarterly and pays districts twice a year, in January for
/// the half-year that ended in December and in August for the one that ended in June. Fifteen of
/// the eighteen sheets print the period in their title banner; three do not, and rather than leave
/// those three unlabelled the rule is applied to all of them and checked against every banner that
/// exists. A rule that is checked eighteen times a rebuild is not an assumption.
#[must_use]
pub fn casino_statutory_period(distribution: &str) -> Option<(String, String)> {
    let (year, month) = distribution.split_once('-')?;
    let year: i32 = year.parse().ok()?;
    match month {
        "01" => Some((format!("{}-07-01", year - 1), format!("{}-12-31", year - 1))),
        "08" => Some((format!("{year}-01-01"), format!("{year}-06-30"))),
        _ => None,
    }
}

/// The half-year a sheet says it is for, read out of its title banner.
///
/// `None` where the banner names only the month the money moved, which is what the four sheets of
/// the combined FY2016-FY2017 workbook do.
#[must_use]
pub fn casino_printed_period(rows: &[Vec<String>]) -> Option<(String, String)> {
    for row in rows.iter().take(8) {
        for cell in row {
            if let Some((from, rest)) = cell.split_once(" - ") {
                if let (Some(start), Some(end)) = (casino_date(from), casino_date(rest)) {
                    return Some((start, end));
                }
            }
        }
    }
    None
}

/// `Jun 30, 2015` and anything following it, as `2015-06-30`.
fn casino_date(text: &str) -> Option<String> {
    let text = text.trim();
    let (month, rest) = text.split_once(' ')?;
    let month = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ]
    .iter()
    .position(|name| name.eq_ignore_ascii_case(month))?
        + 1;
    let (day, rest) = rest.split_once(", ")?;
    let day: u32 = day.trim().parse().ok()?;
    let year: i32 = rest
        .split_whitespace()
        .next()?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()?;
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// One district's share of one distribution, before the sheets are merged.
struct CasinoRow {
    name: String,
    counties: usize,
    amount: f64,
}

/// Reduce the published distribution sheets to one row per district per distribution.
///
/// # What the sheets are made to prove about themselves
///
/// Three checks, each of which has a way of failing that produces a plausible number rather than
/// an error, which is why each is a check and not a comment:
///
/// - **The foot total.** Every sheet prints its own `Total Distribution Amount`, and it must equal
///   the district rows summed. This is the property worth looking for in any new source: the file
///   states an aggregate beside the parts, so a column read from the wrong offset, a dropped
///   continuation page or a doubled row is caught by the source rather than by a reviewer's sense
///   of scale.
/// - **The banner.** Where a sheet names its half-year, it must be the half-year
///   [`casino_statutory_period`] derives from the payment month.
/// - **The overlap.** August 2015 is published twice, once by county and once statewide. The two
///   files are different layouts written by different reports, and they must agree on every
///   district and on the total.
///
/// # Two rules about which rows exist
///
/// Rows whose amount is zero are dropped. The statewide layout lists districts certified at no
/// students — 22 of the 1,066 in August 2015, closed community schools for the most part — and the
/// county layout omits them. Keeping them would make a district's presence in the panel depend on
/// which layout the department happened to use that year, which is a fact about the report and not
/// about the district.
///
/// A district's county rows are summed, and how many there were is kept. That count is the
/// apportionment R.C. 5753.11 describes rather than a duplicate: a district is paid out of every
/// county fund it has resident students in, which for a statewide e-school is all 88 of them.
///
/// # Errors
///
/// Returns the first check that fails, naming the sheet.
pub fn build_casino_extract(sheets: &[CasinoSheet<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut panel: BTreeMap<(String, String), CasinoRow> = BTreeMap::new();
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();

    for sheet in sheets {
        let (header, layout) = casino_layout(sheet.rows)
            .ok_or_else(|| format!("{}: no IRN header row", sheet.label))?;
        let (irn_column, name_column, amount_column, county_column) = match layout {
            CasinoLayout::ByCounty {
                county,
                irn,
                name,
                amount,
            } => (irn, name, amount, Some(county)),
            CasinoLayout::ByDistrict { irn, name, amount } => (irn, name, amount, None),
        };

        if let Some((start, end)) = casino_printed_period(sheet.rows) {
            let derived = casino_statutory_period(sheet.distribution).ok_or_else(|| {
                format!(
                    "{}: {} is not a payment month",
                    sheet.label, sheet.distribution
                )
            })?;
            if (start.clone(), end.clone()) != derived {
                return Err(format!(
                    "{}: banner says {start}..{end}, the payment month says {}..{}",
                    sheet.label, derived.0, derived.1
                ));
            }
        }

        let mut published: Option<f64> = None;
        let mut summed = 0.0;
        let mut fresh: BTreeMap<String, CasinoRow> = BTreeMap::new();

        for row in &sheet.rows[header + 1..] {
            let key = cell(row, irn_column).trim();
            if key.starts_with(CASINO_TOTAL_ROW)
                || cell(row, 0).trim().starts_with(CASINO_TOTAL_ROW)
            {
                published = cell_number(row, amount_column);
                continue;
            }
            if key.len() != 6 || !key.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            let Some(amount) = cell_number(row, amount_column) else {
                continue;
            };
            summed += amount;
            if amount == 0.0 {
                continue;
            }
            let entry = fresh.entry(key.to_string()).or_insert_with(|| CasinoRow {
                name: clean_name(cell(row, name_column)),
                counties: 0,
                amount: 0.0,
            });
            entry.amount += amount;
            if county_column.is_some() {
                entry.counties += 1;
            }
        }

        let published =
            published.ok_or_else(|| format!("{}: no {CASINO_TOTAL_ROW} row", sheet.label))?;
        if (published - summed).abs() >= 0.005 {
            return Err(format!(
                "{}: rows sum to {summed:.2}, the sheet says {published:.2}",
                sheet.label
            ));
        }

        match seen.insert(sheet.distribution.to_string(), sheet.label) {
            None => {
                for (irn, row) in fresh {
                    panel.insert((sheet.distribution.to_string(), irn), row);
                }
            }
            Some(first) => {
                for (irn, row) in fresh {
                    let key = (sheet.distribution.to_string(), irn.clone());
                    let held = panel.get(&key).ok_or_else(|| {
                        format!(
                            "{}: {irn} is paid in {} and absent from {first}",
                            sheet.label, sheet.distribution
                        )
                    })?;
                    if (held.amount - row.amount).abs() >= 0.005 {
                        return Err(format!(
                            "{} and {first} disagree about {irn} in {}: {:.2} against {:.2}",
                            sheet.label, sheet.distribution, row.amount, held.amount
                        ));
                    }
                }
            }
        }
    }

    Ok(panel
        .into_iter()
        .map(|((distribution, irn), row)| {
            vec![
                irn,
                row.name,
                distribution,
                if row.counties == 0 {
                    String::new()
                } else {
                    row.counties.to_string()
                },
                format_value(Some(row.amount), 2),
            ]
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Rows of a casino sheet in the county layout, banner and foot total included.
    fn casino_county_sheet(
        banner: &str,
        districts: &[(&str, &str, &str, &str)],
    ) -> Vec<Vec<String>> {
        let mut rows = vec![
            vec![
                String::new(),
                String::new(),
                "Ohio Department of Taxation".into(),
            ],
            vec![String::new(), String::new(), banner.into()],
            vec![
                "County Code".into(),
                "County Name".into(),
                "IRN".into(),
                "SD Name".into(),
                "Distrubution Amount".into(),
            ],
        ];
        let mut total = 0.0;
        for (county, name, irn, amount) in districts {
            total += amount.parse::<f64>().unwrap_or_default();
            rows.push(vec![
                (*county).to_string(),
                "SOMEWHERE".into(),
                (*irn).to_string(),
                (*name).to_string(),
                (*amount).to_string(),
            ]);
        }
        rows.push(vec![
            "Total Distribution Amount".into(),
            String::new(),
            String::new(),
            String::new(),
            format!("{total:.2}"),
        ]);
        rows
    }

    #[test]
    fn a_district_paid_from_three_counties_is_one_row_that_says_three() {
        let sheet = casino_county_sheet(
            "Jul 1, 2023 - Dec 31, 2023 County Student Distribution",
            &[
                ("01", "OHIO CONNECTIONS ACADEMY", "000236", "449.12"),
                ("02", "OHIO CONNECTIONS ACADEMY", "000236", "550.88"),
                ("03", "OHIO CONNECTIONS ACADEMY", "000236", "100.00"),
                ("01", "MANCHESTER LOCAL", "000442", "21076.75"),
            ],
        );
        let panel = build_casino_extract(&[CasinoSheet {
            distribution: "2024-01",
            label: "test",
            rows: &sheet,
        }])
        .expect("checks pass");
        assert_eq!(
            panel,
            vec![
                vec!["000236", "OHIO CONNECTIONS ACADEMY", "2024-01", "3", "1100"],
                vec!["000442", "MANCHESTER LOCAL", "2024-01", "1", "21076.75"],
            ]
        );
    }

    #[test]
    fn the_foot_total_is_what_catches_a_column_read_from_the_wrong_offset() {
        // The failure this stands against does not look like a failure. Sum the amount column
        // without excluding the row the department puts its own total on, and a half-year comes
        // out at exactly twice its size — which for this channel is $90m, a number with nothing
        // obviously wrong about it.
        let whole = casino_county_sheet(
            "Jan 1, 2023 - Jun 30, 2023 County Student Distribution",
            &[
                ("01", "MANCHESTER LOCAL", "000442", "21076.75"),
                ("02", "LOST TO THE PAGE BREAK", "000443", "500.00"),
            ],
        );
        // A page break drops a district. Nothing about the remaining rows is malformed and the
        // foot total does not move, so only the comparison between them notices.
        let torn: Vec<Vec<String>> = whole
            .iter()
            .filter(|row| crate::conventions::cell(row, 2) != "000443")
            .cloned()
            .collect();
        let failure = build_casino_extract(&[CasinoSheet {
            distribution: "2023-08",
            label: "the sheet that lost a row",
            rows: &torn,
        }])
        .expect_err("the sheet's own total disagrees");
        assert!(failure.contains("the sheet that lost a row"), "{failure}");
        assert!(failure.contains("21576.75"), "{failure}");
    }

    #[test]
    fn a_banner_that_disagrees_with_the_payment_month_is_a_failure_not_a_note() {
        // R.C. 5753.03(D)(2) pays in August for January-June. A sheet filed under the wrong month
        // parses perfectly and puts a half-year in the wrong fiscal year.
        let sheet = casino_county_sheet(
            "Jan 1, 2023 - Jun 30, 2023 County Student Distribution",
            &[("01", "MANCHESTER LOCAL", "000442", "21076.75")],
        );
        let failure = build_casino_extract(&[CasinoSheet {
            distribution: "2024-01",
            label: "misfiled",
            rows: &sheet,
        }])
        .expect_err("the banner and the month disagree");
        assert!(failure.contains("2023-01-01..2023-06-30"), "{failure}");
        assert!(failure.contains("2023-07-01"), "{failure}");
    }

    #[test]
    fn two_layouts_of_one_distribution_have_to_agree_district_by_district() {
        // August 2015 is published twice, by county and statewide. The check is worth having
        // because the two are generated by different reports: they agree today to the cent across
        // 1,044 districts, and a disagreement would mean one of the two was never what it said.
        let by_county = casino_county_sheet(
            "Jan 1, 2015 - Jun 30, 2015 County Student Distribution",
            &[
                ("01", "MANCHESTER LOCAL", "000442", "2000.00"),
                ("02", "MANCHESTER LOCAL", "000442", "1000.00"),
            ],
        );
        let statewide = vec![
            vec![
                String::new(),
                String::new(),
                String::new(),
                "Ohio Department of Taxation".into(),
            ],
            vec![
                String::new(),
                String::new(),
                String::new(),
                "August 2015".into(),
            ],
            vec![
                "IRN".into(),
                "School District Name".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "Distribution Amount".into(),
            ],
            vec![
                "000442".into(),
                "MANCHESTER LOCAL".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "3500.00".into(),
            ],
            vec![
                "Total Distribution Amount:".into(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                "3500.00".into(),
            ],
        ];
        let failure = build_casino_extract(&[
            CasinoSheet {
                distribution: "2015-08",
                label: "by county",
                rows: &by_county,
            },
            CasinoSheet {
                distribution: "2015-08",
                label: "statewide",
                rows: &statewide,
            },
        ])
        .expect_err("3500 against 3000");
        assert!(failure.contains("000442"), "{failure}");
    }

    #[test]
    fn the_true_up_years_amount_is_the_last_column_and_not_the_first() {
        // August 2018 carries three amount columns: a January 2018 recalculation of $7,475.78, the
        // August 2018 calculation of $48.0m, and their total. The first sits in column E, which is
        // where every other year's amount is — so a fixed offset reports the half-year at seven
        // thousand dollars and nothing else notices.
        let rows = vec![
            vec![
                String::new(),
                String::new(),
                "Ohio Department of Taxation".into(),
            ],
            vec![
                String::new(),
                String::new(),
                "Jan 1, 2018 - Jun 30, 2018 County Student Distribution".into(),
            ],
            vec![
                "County Code".into(),
                "County Name".into(),
                "IRN".into(),
                "SD Name".into(),
                "January 2018 Calculated Distribution (A)".into(),
                "August 2018 Calculated Distribution (B)".into(),
                "Total Distribution".into(),
            ],
            vec![
                "01".into(),
                "ADAMS".into(),
                "000442".into(),
                "MANCHESTER LOCAL".into(),
                "3.21".into(),
                "20000.00".into(),
                "20003.21".into(),
            ],
            vec![
                "Total Distribution Amount".into(),
                String::new(),
                String::new(),
                String::new(),
                "3.21".into(),
                "20000.00".into(),
                "20003.21".into(),
            ],
        ];
        let panel = build_casino_extract(&[CasinoSheet {
            distribution: "2018-08",
            label: "the true-up",
            rows: &rows,
        }])
        .expect("checks pass");
        assert_eq!(panel[0][4], "20003.21");
    }

    #[test]
    fn a_zero_row_is_dropped_because_only_one_of_the_two_layouts_publishes_them() {
        // The statewide sheets list districts certified at no students — 22 of the 1,066 in August
        // 2015. The county sheets omit them. Keeping them would make a district's presence in the
        // panel a fact about which report the department ran that year.
        let sheet = casino_county_sheet(
            "Jul 1, 2023 - Dec 31, 2023 County Student Distribution",
            &[
                ("01", "MANCHESTER LOCAL", "000442", "21076.75"),
                ("01", "CLOSED ACADEMY", "000909", "0.00"),
            ],
        );
        let panel = build_casino_extract(&[CasinoSheet {
            distribution: "2024-01",
            label: "test",
            rows: &sheet,
        }])
        .expect("checks pass");
        assert_eq!(panel.len(), 1);
        assert_eq!(panel[0][0], "000442");
    }

    #[test]
    fn the_statutory_period_is_the_half_year_before_the_payment() {
        assert_eq!(
            casino_statutory_period("2024-01"),
            Some(("2023-07-01".into(), "2023-12-31".into()))
        );
        assert_eq!(
            casino_statutory_period("2020-08"),
            Some(("2020-01-01".into(), "2020-06-30".into()))
        );
        // Money moves in January and August and no other month; anything else is a filing error.
        assert_eq!(casino_statutory_period("2024-04"), None);
    }

    #[test]
    fn a_banner_is_read_and_a_month_only_headline_is_not_mistaken_for_one() {
        let dated = vec![vec![
            "Jan 1, 2015 - Jun 30, 2015 County Student Distribution".to_string(),
        ]];
        assert_eq!(
            casino_printed_period(&dated),
            Some(("2015-01-01".into(), "2015-06-30".into()))
        );
        // The four sheets of the combined workbook name the month the money moved and not the
        // half-year it was earned in. Guessing one from the other is what the statutory rule is
        // for, and it is not this function's job.
        let undated = vec![
            vec!["School District Distribution".to_string()],
            vec!["August 2015".to_string()],
        ];
        assert_eq!(casino_printed_period(&undated), None);
    }
}
