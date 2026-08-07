//! The `dew-five-year-forecast` connector: what districts actually received, spent, and held.
//!
//! Every Ohio school district files a five-year forecast under R.C. 5705.391, and the department
//! republishes all of them as one tab-separated file per submission window. Each file carries
//! **three years of audited actuals** alongside the five forecast years, which is the part this
//! module is for: the actuals are the only per-district record in this corpus of money that
//! changed hands, as against money a formula computed.
//!
//! # Actual and forecast are different claims and are not mixed
//!
//! A row holds eight values: three prior actuals, the submission year's own forecast, and four
//! future ones. Only the actuals are extracted into the committed fixture. The forecast columns
//! are a district treasurer's projection, made under incentives — a board arguing for a levy and
//! a board defending a balance want different numbers on the same line — and this corpus already
//! has one projection engine whose assumptions it can state. Treating a submitted forecast as a
//! measurement is the specific error this module refuses to make; [`Line::current`] is parsed and
//! named so a caller who wants it has to ask for it.
//!
//! # Fiscal year alignment
//!
//! `SCHOOL_YEAR` is the fiscal year the submission was filed *for*, so its three actual columns
//! are the three years before it: a 2026 filing carries FY2023, FY2024, and FY2025. See
//! [`actual_years`].

use std::collections::HashMap;

/// General property tax (real estate) — the local levy yield, after H.B. 920 reduction.
pub const PROPERTY_TAX: &str = "1.010";
/// School district income tax, where one is levied.
pub const INCOME_TAX: &str = "1.030";
/// Unrestricted grants-in-aid: state foundation funding as the district books it.
pub const UNRESTRICTED_AID: &str = "1.035";
/// Restricted grants-in-aid: state money the district may not spend freely.
pub const RESTRICTED_AID: &str = "1.040";
/// Property tax allocation — the state reimbursing rollback and homestead exemptions.
///
/// State money that arrives on the local line in most published summaries, which is why the
/// state share of Ohio school funding is quoted at different numbers by different people.
pub const PROPERTY_TAX_ALLOCATION: &str = "1.050";
/// Total revenue, general fund. Excludes transfers, advances, and note proceeds.
pub const TOTAL_REVENUE: &str = "1.070";
/// Total revenue **and other financing sources** — transfers and advances included.
///
/// The one that closes the cash identity. `1.070` is the district's own revenue and is the
/// right numerator for a state-share question; only this one accounts for every dollar that
/// entered the fund, so only this one satisfies
/// `beginning + received - spent = ending`.
pub const TOTAL_REVENUE_AND_SOURCES: &str = "2.080";
/// Total expenditures and other financing uses.
pub const TOTAL_EXPENDITURE: &str = "5.050";
/// Beginning cash balance, 1 July. The carry-over.
pub const BEGINNING_CASH: &str = "7.010";
/// Ending cash balance, 30 June, excluding proposed levies.
pub const ENDING_CASH: &str = "7.020";

/// Every line code the fixture carries, in fixture column order.
pub const EXTRACTED: &[&str] = &[
    UNRESTRICTED_AID,
    RESTRICTED_AID,
    PROPERTY_TAX,
    INCOME_TAX,
    PROPERTY_TAX_ALLOCATION,
    TOTAL_REVENUE,
    TOTAL_REVENUE_AND_SOURCES,
    TOTAL_EXPENDITURE,
    BEGINNING_CASH,
    ENDING_CASH,
];

/// A file that would not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ForecastError {
    /// The file has no header row.
    Empty,
    /// A column this module needs is not in the header.
    ///
    /// Named rather than positional because the department has published this file under three
    /// different layouts; a header-driven parser fails loudly on the fourth instead of reading
    /// the wrong column.
    MissingColumn(String),
}

impl core::fmt::Display for ForecastError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => write!(f, "the forecast file is empty"),
            Self::MissingColumn(name) => {
                write!(f, "the forecast file has no {name:?} column")
            }
        }
    }
}

impl std::error::Error for ForecastError {}

/// One line of one district's filing.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name as filed.
    pub name: String,
    /// County.
    pub county: String,
    /// The forecast line code, such as `7.020`.
    pub code: String,
    /// The department's description of the line.
    pub description: String,
    /// The fiscal year this filing was made for.
    pub school_year: u16,
    /// Three prior fiscal years, oldest first. Audited actuals.
    pub actual: [f64; 3],
    /// The filing year's own forecast. **Not an actual** — see the module note.
    pub current: f64,
}

impl Line {
    /// The three fiscal years [`Line::actual`] covers, oldest first.
    #[must_use]
    pub fn actual_years(&self) -> [u16; 3] {
        actual_years(self.school_year)
    }
}

/// The three fiscal years a filing for `school_year` reports actuals for.
///
/// A 2026 filing is made partway through FY2026, so FY2025 is the most recent closed year.
#[must_use]
pub const fn actual_years(school_year: u16) -> [u16; 3] {
    [school_year - 3, school_year - 2, school_year - 1]
}

/// A number as the department writes it: blank, or with separators it does not usually use.
fn number(raw: &str) -> f64 {
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
        .collect();
    cleaned.parse().unwrap_or(0.0)
}

/// Parse a headered forecast file.
///
/// Only the lines in [`EXTRACTED`] are returned; the file carries about sixty per district and
/// the rest are sub-components of them.
///
/// # Errors
///
/// Returns [`ForecastError`] if the file is empty or its header lacks a column this needs.
pub fn parse(text: &str) -> Result<Vec<Line>, ForecastError> {
    let mut lines = text.lines();
    let header = lines.next().ok_or(ForecastError::Empty)?;
    // A UTF-8 BOM on the first column name would make every lookup miss.
    let header = header.trim_start_matches('\u{feff}');

    let index: HashMap<&str, usize> = header
        .split('\t')
        .enumerate()
        .map(|(at, name)| (name.trim(), at))
        .collect();
    let column = |name: &str| {
        index
            .get(name)
            .copied()
            .ok_or_else(|| ForecastError::MissingColumn(name.to_string()))
    };

    let year_at = column("SCHOOL_YEAR")?;
    let irn_at = column("LEA_IRN")?;
    let name_at = column("LEA_NAME")?;
    let county_at = column("LEA_COUNTY_NAME")?;
    let code_at = column("FRCST_CAT_LINE_CODE")?;
    let descr_at = column("FRCST_CAT_LINE_DESCR")?;
    let three_at = column("THREE_YEAR_PRIOR_ACTUAL_AMT")?;
    let two_at = column("TWO_YEAR_PRIOR_ACTUAL_AMT")?;
    let one_at = column("ONE_YEAR_PRIOR_ACTUAL_AMT")?;
    let current_at = column("CUR_FY_FRCST_AMT")?;

    let mut out = Vec::new();
    for row in lines {
        if row.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = row.split('\t').map(str::trim).collect();
        let get = |at: usize| fields.get(at).copied().unwrap_or_default();
        let code = get(code_at);
        if !EXTRACTED.contains(&code) {
            continue;
        }
        let Ok(school_year) = get(year_at).parse::<u16>() else {
            continue;
        };
        out.push(Line {
            irn: get(irn_at).to_string(),
            name: get(name_at).to_string(),
            county: get(county_at).to_string(),
            code: code.to_string(),
            description: get(descr_at).to_string(),
            school_year,
            actual: [
                number(get(three_at)),
                number(get(two_at)),
                number(get(one_at)),
            ],
            current: number(get(current_at)),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "SCHOOL_YEAR\tSUBMISSION\tLEA_IRN\tLEA_NAME\tLEA_COUNTY_NAME\t\
                          FRCST_CAT_LINE_CODE\tFRCST_CAT_LINE_DESCR\t\
                          THREE_YEAR_PRIOR_ACTUAL_AMT\tTWO_YEAR_PRIOR_ACTUAL_AMT\t\
                          ONE_YEAR_PRIOR_ACTUAL_AMT\tCUR_FY_FRCST_AMT\t\
                          ONE_YEAR_FUTURE_FRCST_AMT";

    fn file(rows: &[&str]) -> String {
        let mut text = String::from(HEADER);
        for row in rows {
            text.push('\n');
            text.push_str(row);
        }
        text
    }

    #[test]
    fn a_filing_reports_the_three_years_before_the_one_it_is_filed_for() {
        // The alignment error this prevents is off by one in the direction that looks right: a
        // 2026 filing is made during FY2026, so FY2026 is forecast and FY2025 is the actual.
        assert_eq!(actual_years(2026), [2023, 2024, 2025]);
        assert_eq!(actual_years(2023), [2020, 2021, 2022]);
    }

    #[test]
    fn the_two_pinned_filings_cover_six_consecutive_years_without_overlapping() {
        let early = actual_years(2023);
        let late = actual_years(2026);
        assert_eq!(early[2] + 1, late[0], "{early:?} then {late:?}");
    }

    #[test]
    fn only_the_lines_the_fixture_carries_are_returned() {
        let text = file(&[
            "2026\tSpring\t046763\tOlentangy Local\tDelaware\t7.020\tEnding Cash\t1\t2\t3\t4\t5",
            "2026\tSpring\t046763\tOlentangy Local\tDelaware\t3.030\tPurchased Services\t9\t9\t9\t9\t9",
        ]);
        let lines = parse(&text).expect("parses");
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].code, ENDING_CASH);
    }

    #[test]
    fn the_three_actuals_are_read_oldest_first() {
        let text = file(&[
            "2026\tSpring\t046763\tOlentangy\tDelaware\t7.020\tEnding Cash\t100\t200\t300\t400\t500",
        ]);
        let line = &parse(&text).expect("parses")[0];
        assert_eq!(line.actual, [100.0, 200.0, 300.0]);
        assert_eq!(line.actual_years(), [2023, 2024, 2025]);
        // The filing year's own number is a forecast and is kept apart from the actuals.
        assert!((line.current - 400.0).abs() < f64::EPSILON);
    }

    #[test]
    fn columns_are_found_by_name_rather_than_position() {
        // The department has published this file under three layouts. A positional parser reads
        // the wrong column on the fourth; this one refuses.
        let reordered = "LEA_IRN\tSCHOOL_YEAR\tLEA_NAME\tLEA_COUNTY_NAME\tFRCST_CAT_LINE_CODE\t\
                         FRCST_CAT_LINE_DESCR\tTHREE_YEAR_PRIOR_ACTUAL_AMT\t\
                         TWO_YEAR_PRIOR_ACTUAL_AMT\tONE_YEAR_PRIOR_ACTUAL_AMT\tCUR_FY_FRCST_AMT\n\
                         046763\t2026\tOlentangy\tDelaware\t7.010\tBeginning Cash\t7\t8\t9\t10";
        let line = &parse(reordered).expect("parses")[0];
        assert_eq!(line.irn, "046763");
        assert_eq!(line.actual, [7.0, 8.0, 9.0]);
    }

    #[test]
    fn a_headerless_file_is_refused_rather_than_read_positionally() {
        // The pre-2022 files have no header at all. Reading one as though it did would silently
        // treat a district name as a line code and return nothing, which looks like a clean
        // parse of an empty file.
        let headerless = "Teays Valley Local\tPickaway\t049098\t1.010\t7457265\t7754060";
        assert!(matches!(
            parse(headerless),
            Err(ForecastError::MissingColumn(_))
        ));
    }

    #[test]
    fn an_empty_file_says_so() {
        assert_eq!(parse(""), Err(ForecastError::Empty));
    }

    #[test]
    fn a_byte_order_mark_does_not_hide_the_first_column() {
        let text = format!(
            "\u{feff}{}",
            file(
                &["2026\tSpring\t046763\tOlentangy\tDelaware\t7.020\tEnding Cash\t1\t2\t3\t4\t5",]
            )
        );
        assert_eq!(parse(&text).expect("parses").len(), 1);
    }

    #[test]
    fn a_blank_amount_reads_as_zero_and_a_negative_one_keeps_its_sign() {
        let text = file(&[
            "2026\tSpring\t046763\tOlentangy\tDelaware\t7.020\tEnding Cash\t\t-500\t1,200\t0\t0",
        ]);
        let line = &parse(&text).expect("parses")[0];
        assert_eq!(line.actual, [0.0, -500.0, 1200.0]);
    }

    #[test]
    fn rows_that_are_not_numbered_years_are_skipped_rather_than_defaulted() {
        let text =
            file(&["\tSpring\t046763\tOlentangy\tDelaware\t7.020\tEnding Cash\t1\t2\t3\t4\t5"]);
        assert!(parse(&text).expect("parses").is_empty());
    }
}
