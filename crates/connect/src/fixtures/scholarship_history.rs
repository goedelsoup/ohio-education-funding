//! The department's archived scholarship participation series, FY1997 through FY2013.
//!
//! One workbook, four programme sheets, and two denominators for every programme-year:
//! applications made and scholarships actually paid. The second is the department's own phrase —
//! *Scholarships Used (At least 1 payment made)* — and the pair is the distinction no other
//! source in this repository carries.
//!
//! # Why the sheets are pinned rather than enumerated
//!
//! The workbook was written once, in October 2018, out of the system that preceded the
//! Enterprise Application System. Nothing maintains it, so a sheet appearing or disappearing is
//! not a new programme year — it is the department having replaced the file with a different
//! document at the same URL, which is the failure this reader should announce rather than absorb.
//!
//! # The programme keys are the annual report's
//!
//! `cleveland`, `traditional-edchoice`, `autism` and `jon-peterson` are the keys
//! [`super::scholarship`] writes for the 2024-25 report, and the sheet names here are none of
//! those — *EdChoice (Traditional)*, *JPSN*. Mapping onto the existing keys is what lets the two
//! fixtures be read as one series with an eleven-year hole in the middle, which is what they are.

use super::delimited::column;
use super::format::format_value;

/// Columns of the archived participation series.
pub const SCHOLARSHIP_HISTORY_HEADER: &[&str] = &[
    "program",
    "fiscal_year",
    "measure",
    "total",
    "new",
    "renewal",
    "new_low_income",
    "renewal_low_income",
];

/// The sub-header a programme's five-column block carries, in order.
const SPLIT_COLUMNS: &[&str] = &[
    "Total",
    "New",
    "Renewal",
    "New and Qualified as Low Income",
    "Renewal and Qualified as Low Income",
];

/// The sub-header Jon Peterson's two-column block carries. Its first and only year distinguishes
/// new from renewal and nothing else, because the programme began that year.
const PAIR_COLUMNS: &[&str] = &["Total", "New"];

/// How one sheet lays its counts out.
#[derive(Clone, Copy)]
enum Layout {
    /// Applications and payments, each split five ways, with a sub-header row. `tutoring` marks
    /// the twelfth column Cleveland carries and no other programme has.
    Split { tutoring: bool },
    /// Applications and payments, total and new only, with a sub-header row.
    Pair,
    /// Payments only, in one column, with no sub-header row at all — the sheet's second row is
    /// already a fiscal year. Autism's note says why: the system that wrote this file "did not
    /// distinguish between new and renewal applicants" for it.
    PaidOnly,
}

/// One denominator's block inside a sheet.
struct Block {
    /// The `measure` value its rows are written under.
    measure: &'static str,
    /// The column its first sub-header sits in.
    at: usize,
    /// The sub-headers it must carry, in order.
    columns: &'static [&'static str],
}

/// Where one sheet's blocks sit, and where its data begins.
struct Shape {
    blocks: Vec<Block>,
    /// Cleveland's twelfth column, which is labelled only in the group row.
    tutoring: Option<usize>,
    /// The first row that is a fiscal year rather than a heading.
    first: usize,
}

/// A sheet, and the programme key its rows are written under.
struct Series {
    program: &'static str,
    sheet: &'static str,
    layout: Layout,
}

/// The four programme sheets, in the order the workbook presents them.
const SERIES: &[Series] = &[
    Series {
        program: "cleveland",
        sheet: "Cleveland",
        layout: Layout::Split { tutoring: true },
    },
    Series {
        program: "traditional-edchoice",
        sheet: "EdChoice (Traditional)",
        layout: Layout::Split { tutoring: false },
    },
    Series {
        program: "autism",
        sheet: "Autism",
        layout: Layout::PaidOnly,
    },
    Series {
        program: "jon-peterson",
        sheet: "JPSN",
        layout: Layout::Pair,
    },
];

/// Collapse a header cell to compare it against an expected label.
///
/// Cleveland writes its payments heading with a newline inside it — `"Scholarships Used
/// \n(At least 1 payment made)"` — and EdChoice writes the same heading on one line. They are the
/// same heading and a byte comparison says they are not.
fn squash(cell: &str) -> String {
    cell.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The fiscal year a row label names, if it is one.
///
/// `"FY 1997"`. Anything else ends the sheet: the workbook pads three of its four sheets with
/// blank rows, and one of them carries a short row with fewer cells than the header.
fn fiscal_year(cell: &str) -> Option<u16> {
    let rest = cell.trim().strip_prefix("FY")?;
    rest.trim()
        .parse()
        .ok()
        .filter(|y| (1990..2030).contains(y))
}

/// One count, or nothing.
///
/// `NA` is the publisher's own mark and becomes blank, alongside the empty cell. The two are
/// different statements — *we stopped collecting this* against *nothing was written here* — and
/// this fixture can carry neither, so both arrive as absent and the catalog record says which
/// cells are which. What matters is that neither becomes a zero: Cleveland's tutoring programme
/// reads `NA` from FY2010, and a zero there would assert the programme ran and served nobody.
fn count(cell: &str) -> Option<f64> {
    let text = cell.trim();
    if text.is_empty() || text.eq_ignore_ascii_case("NA") {
        return None;
    }
    text.replace(',', "").parse().ok()
}

/// Check that a block's sub-headers are the ones expected, starting at `at`.
fn expect(
    head: &[String],
    at: usize,
    names: &[&str],
    sheet: &str,
    block: &str,
) -> Result<(), String> {
    for (offset, name) in names.iter().enumerate() {
        let found = head.get(at + offset).map_or("", String::as_str);
        if squash(found) != *name {
            return Err(format!(
                "the {sheet} sheet's {block} block reads {found:?} where {name:?} belongs; its \
                 layout has moved"
            ));
        }
    }
    Ok(())
}

/// One row per programme, fiscal year and denominator.
///
/// # What the two denominators are for
///
/// The 2025 annual report publishes an average award that is not expenditure over participation
/// for three of its four programmes, and the catalog record guesses the gap is a denominator the
/// report does not name. This workbook publishes two denominators for the same programme-year,
/// which is the shape of evidence that guess needs — and shows they differ by a great deal:
/// Cleveland's applications exceed its payments in every one of its seventeen years, threefold in
/// FY1997 and by a tenth in FY2013.
///
/// # A row is written only where a count exists
///
/// A programme-year with nothing in a block produces no row for that block rather than a row of
/// blanks. Autism therefore has payment rows and no application rows at all, which is the
/// department's sheet rather than a gap in this reader, and Cleveland's tutoring rows stop at
/// FY2009 where the sheet starts reading `NA`.
///
/// # Errors
///
/// If a sheet is missing, if a block's sub-headers are not the ones this reader was written
/// against, or if a sheet yields no years at all.
pub fn build_scholarship_history(
    sheets: &[(&str, &[Vec<String>])],
) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for series in SERIES {
        let rows = sheets
            .iter()
            .find(|(name, _)| *name == series.sheet)
            .map(|(_, rows)| *rows)
            .ok_or_else(|| {
                format!(
                    "the historical scholarship workbook has no {} sheet; its structure changed",
                    series.sheet
                )
            })?;

        let shape = match series.layout {
            Layout::Split { tutoring } => Shape {
                blocks: vec![
                    Block {
                        measure: "applications",
                        at: 1,
                        columns: SPLIT_COLUMNS,
                    },
                    Block {
                        measure: "used",
                        at: 1 + SPLIT_COLUMNS.len(),
                        columns: SPLIT_COLUMNS,
                    },
                ],
                tutoring: tutoring.then_some(1 + 2 * SPLIT_COLUMNS.len()),
                first: 2,
            },
            Layout::Pair => Shape {
                blocks: vec![
                    Block {
                        measure: "applications",
                        at: 1,
                        columns: PAIR_COLUMNS,
                    },
                    Block {
                        measure: "used",
                        at: 1 + PAIR_COLUMNS.len(),
                        columns: PAIR_COLUMNS,
                    },
                ],
                tutoring: None,
                first: 2,
            },
            // No sub-header row, so the block is one column and the data starts immediately.
            Layout::PaidOnly => Shape {
                blocks: vec![Block {
                    measure: "used",
                    at: 1,
                    columns: &["Total"],
                }],
                tutoring: None,
                first: 1,
            },
        };

        // The sub-header row, where there is one. `PaidOnly` has none and its single column is
        // checked against the group heading instead.
        if matches!(series.layout, Layout::PaidOnly) {
            let group = rows
                .first()
                .and_then(|r| r.get(1))
                .map_or("", String::as_str);
            if !squash(group).starts_with("Scholarships Used") {
                return Err(format!(
                    "the {} sheet's only count column reads {group:?} rather than a payments \
                     heading; its layout has moved",
                    series.sheet
                ));
            }
        } else {
            let head = rows
                .get(1)
                .ok_or_else(|| format!("the {} sheet has no sub-header row", series.sheet))?;
            for block in &shape.blocks {
                expect(head, block.at, block.columns, series.sheet, block.measure)?;
            }
            if let Some(at) = shape.tutoring {
                // Not in the sub-header row — the tutoring column's only label is the group
                // heading above it, which is why it is located by the group row.
                let group = rows
                    .first()
                    .and_then(|r| r.get(at))
                    .map_or("", String::as_str);
                column(&[group.to_string()], "Tutoring Program", series.sheet)?;
            }
        }

        let mut years = 0;
        for row in rows.iter().skip(shape.first) {
            let Some(year) = row.first().and_then(|cell| fiscal_year(cell)) else {
                break;
            };
            years += 1;
            let cell = |i: usize| row.get(i).map_or("", String::as_str);
            for block in &shape.blocks {
                let values: Vec<Option<f64>> = (0..block.columns.len())
                    .map(|offset| count(cell(block.at + offset)))
                    .collect();
                if values.iter().all(Option::is_none) {
                    continue;
                }
                let mut record = vec![
                    series.program.to_string(),
                    year.to_string(),
                    block.measure.to_string(),
                ];
                // Padded to the five-column shape whatever the sheet's own width, so every row
                // of the fixture reads the same way.
                for slot in 0..SPLIT_COLUMNS.len() {
                    record.push(format_value(values.get(slot).copied().flatten(), 0));
                }
                out.push(record);
            }
            if let Some(at) = shape.tutoring {
                if let Some(value) = count(cell(at)) {
                    let mut record = vec![
                        series.program.to_string(),
                        year.to_string(),
                        "tutoring".to_string(),
                    ];
                    record.push(format_value(Some(value), 0));
                    record.extend(std::iter::repeat_n(String::new(), SPLIT_COLUMNS.len() - 1));
                    out.push(record);
                }
            }
        }
        if years == 0 {
            return Err(format!(
                "the {} sheet names no fiscal year; its structure changed",
                series.sheet
            ));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|c| (*c).to_string()).collect()
    }

    fn cleveland() -> Vec<Vec<String>> {
        let mut head = vec![
            "Cleveland Scholarship ".to_string(),
            "Application Count".into(),
        ];
        head.resize(6, String::new());
        head.push("Scholarships Used \n(At least 1 payment made)".into());
        head.resize(11, String::new());
        head.push("Tutoring Program".into());
        let mut sub = vec![String::new()];
        sub.extend(SPLIT_COLUMNS.iter().map(|c| (*c).to_string()));
        sub.extend(SPLIT_COLUMNS.iter().map(|c| (*c).to_string()));
        sub.push(String::new());
        vec![
            head,
            sub,
            row(&[
                "FY 1997", "6233", "6233", "0", "3657", "", "1994", "1994", "0", "", "0", "124",
            ]),
            row(&[
                "FY 2010", "7376", "2351", "5025", "1142", "3545", "5525", "941", "4584", "941",
                "3303", "NA",
            ]),
            row(&["", "", "", "", "", ""]),
        ]
    }

    fn autism() -> Vec<Vec<String>> {
        vec![
            row(&[
                "Autism Scholarship ",
                "Scholarships Used \n(At least 1 payment made)",
            ]),
            row(&[
                "FY 2004",
                "70",
                "*Previous application system did not distinguish between new and renewal \
                 applicants",
            ]),
            row(&["FY 2005", "300"]),
        ]
    }

    fn built(sheets: &[(&str, Vec<Vec<String>>)]) -> Result<Vec<Vec<String>>, String> {
        let borrowed: Vec<(&str, &[Vec<String>])> =
            sheets.iter().map(|(n, r)| (*n, r.as_slice())).collect();
        build_scholarship_history(&borrowed)
    }

    /// The sheets this reader needs, with only Cleveland and Autism carrying data.
    fn workbook() -> Vec<(&'static str, Vec<Vec<String>>)> {
        let mut sub = vec![String::new()];
        sub.extend(SPLIT_COLUMNS.iter().map(|c| (*c).to_string()));
        sub.extend(SPLIT_COLUMNS.iter().map(|c| (*c).to_string()));
        let mut pair = vec![String::new()];
        pair.extend(PAIR_COLUMNS.iter().map(|c| (*c).to_string()));
        pair.extend(PAIR_COLUMNS.iter().map(|c| (*c).to_string()));
        vec![
            ("Cleveland", cleveland()),
            (
                "EdChoice (Traditional)",
                vec![
                    row(&["EdChoice - School Based Scholarship ", "Application Count"]),
                    sub,
                    row(&[
                        "FY 2007", "3655", "3655", "0", "1516", "0", "3071", "3071", "0", "713",
                        "0",
                    ]),
                ],
            ),
            ("Autism", autism()),
            (
                "JPSN",
                vec![
                    row(&["JPSN Scholarship ", "Application Count"]),
                    pair,
                    row(&["FY 2013", "2129", "2129", "1779", "1779"]),
                ],
            ),
        ]
    }

    #[test]
    fn a_split_sheet_yields_a_row_per_denominator() {
        let rows = built(&workbook()).expect("the workbook reads");
        let cleveland: Vec<&Vec<String>> = rows
            .iter()
            .filter(|r| r[0] == "cleveland" && r[1] == "1997")
            .collect();
        assert_eq!(cleveland.len(), 3, "applications, payments and tutoring");
        assert_eq!(
            cleveland[0][2..],
            ["applications", "6233", "6233", "0", "3657", ""]
        );
        assert_eq!(cleveland[1][2..], ["used", "1994", "1994", "0", "", "0"]);
    }

    /// The point of the tutoring column is that it stops without saying so in the row count.
    #[test]
    fn na_is_absent_rather_than_zero() {
        let rows = built(&workbook()).expect("the workbook reads");
        let tutoring: Vec<&Vec<String>> = rows.iter().filter(|r| r[2] == "tutoring").collect();
        assert_eq!(tutoring.len(), 1, "FY2010 reads NA and writes no row");
        assert_eq!(tutoring[0][1], "1997");
    }

    /// Autism has no sub-header row and one count column, and its footnote sits in a third cell
    /// on the first data row where a naive reader would take it for a number.
    #[test]
    fn a_paid_only_sheet_yields_payment_rows_alone() {
        let rows = built(&workbook()).expect("the workbook reads");
        let autism: Vec<&Vec<String>> = rows.iter().filter(|r| r[0] == "autism").collect();
        assert_eq!(autism.len(), 2);
        assert!(autism.iter().all(|r| r[2] == "used"));
        assert_eq!(autism[0][3], "70");
        assert_eq!(
            autism[0][4], "",
            "the footnote is not a count of new awards"
        );
    }

    /// Every row is the header's width, whatever its sheet's own is.
    #[test]
    fn every_row_is_the_header_width() {
        let rows = built(&workbook()).expect("the workbook reads");
        assert!(rows
            .iter()
            .all(|r| r.len() == SCHOLARSHIP_HISTORY_HEADER.len()));
    }

    #[test]
    fn a_moved_sub_header_is_an_error() {
        let mut sheets = workbook();
        sheets[0].1[1][4] = "New and Low Income".to_string();
        let error = built(&sheets).expect_err("the layout moved");
        assert!(error.contains("Cleveland"), "{error}");
        assert!(error.contains("layout has moved"), "{error}");
    }

    #[test]
    fn a_missing_sheet_is_an_error() {
        let sheets: Vec<(&str, Vec<Vec<String>>)> = workbook()
            .into_iter()
            .filter(|(n, _)| *n != "JPSN")
            .collect();
        let error = built(&sheets).expect_err("a sheet is gone");
        assert!(error.contains("JPSN"), "{error}");
    }

    #[test]
    fn a_sheet_with_no_years_is_an_error() {
        let mut sheets = workbook();
        sheets[3].1.truncate(2);
        let error = built(&sheets).expect_err("no years");
        assert!(error.contains("names no fiscal year"), "{error}");
    }

    #[test]
    fn padding_rows_end_a_sheet_rather_than_failing_it() {
        let rows = built(&workbook()).expect("the workbook reads");
        assert!(
            rows.iter().all(|r| r[1].parse::<u16>().is_ok()),
            "the short blank row after Cleveland's data is not a year"
        );
    }
}
