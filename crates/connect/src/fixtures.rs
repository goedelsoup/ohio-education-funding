//! Building the committed CSV fixtures the calculator crates read.
//!
//! Each builder is a pure function from parsed rows to rows of strings, so it tests without a
//! network or a workbook. Only [`write_csv`] touches disk.
//!
//! # Why a CSV sits in the middle
//!
//! Retrieval is side-effecting and can fail; calculation must be deterministic and auditable.
//! The fixture is the seam. It is also legible in a diff, which a cached workbook is not — when
//! the department reposts a corrected file, the change shows up as changed numbers in a review
//! rather than as a silently different answer.

use std::fs;
use std::io;
use std::path::Path;

use crate::conventions::{cell, cell_number, is_statewide_row, number, rows_by_key};

/// Format a value the way the fixtures are written: fixed decimals, trailing zeros trimmed,
/// blank for absent.
///
/// Blank rather than `0` for absent is load-bearing. A district with no reported valuation and
/// a district whose valuation is nil are different claims, and the calculators read the
/// difference as `Option<f64>`.
///
/// # A trailing zero is not a trailing zero
///
/// The trim only applies past a decimal point. Trimming the string form of an integer turns 10
/// into 1 and 30 into 3 — which is what the predecessor of this function did to the school
/// building count, silently, for every district that happened to have a multiple of ten.
#[must_use]
pub fn format_value(value: Option<f64>, places: usize) -> String {
    let Some(value) = value else {
        return String::new();
    };
    let rendered = format!("{value:.places$}");
    if rendered.contains('.') {
        rendered
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        rendered
    }
}

/// District names go into a comma-separated file, so commas are replaced rather than quoted.
#[must_use]
pub fn clean_name(raw: &str) -> String {
    raw.replace(',', " ").trim().to_string()
}

/// Columns of the FY2027 department-model fixture.
pub const FY27_HEADER: &[&str] = &[
    "irn",
    "district",
    "base_cost_enrolled_adm",
    "school_buildings",
    "adm_kindergarten",
    "adm_grades_1_3",
    "adm_grades_4_8_non_cte",
    "adm_grades_9_12_non_cte",
    "adm_cte",
    "adm_grades_9_12_total",
    "funded_classroom_teachers",
    "funded_special_teachers",
    "teacher_base_cost",
    "aggregate_base_cost",
    "base_cost_per_pupil",
    "temp_transitional_aid_guarantee",
    "enrolled_adm_fy22",
    "enrolled_adm_fy24",
    "assessed_valuation_per_pupil_fy23",
    "core_foundation_funding",
];

/// Column positions in the department's `Base_Cost` sheet, whose header is on the fourth row.
/// Named here rather than inline so a layout change is one edit.
mod base_cost_columns {
    pub const NAME: usize = 1;
    pub const BUILDINGS: usize = 3;
    pub const ADM_FY22: usize = 4;
    pub const ADM_FY24: usize = 6;
    pub const ADM: usize = 7;
    pub const KINDERGARTEN: usize = 8;
    pub const GRADES_1_3: usize = 9;
    pub const GRADES_4_8: usize = 10;
    pub const GRADES_9_12: usize = 11;
    pub const CAREER_TECHNICAL: usize = 12;
    pub const GRADES_9_12_TOTAL: usize = 13;
    pub const FUNDED_CLASSROOM: usize = 14;
    pub const FUNDED_SPECIAL: usize = 17;
    pub const TEACHER_COST: usize = 22;
    pub const AGGREGATE: usize = 57;
    pub const PER_PUPIL: usize = 58;
}

/// Column positions in `Summary_SFPR`.
mod summary_columns {
    pub const NAME: usize = 1;
    pub const CORE: usize = 10;
    pub const GUARANTEE: usize = 11;
}

/// Column positions in the District Profile Report's `District Data` sheet.
mod profile_columns {
    pub const NAME: usize = 0;
    pub const IRN: usize = 1;
    pub const ENROLLED_ADM: usize = 4;
    pub const ECON_DISADVANTAGED: usize = 11;
    pub const VALUATION_PER_PUPIL: usize = 21;
    pub const CURRENT_OPERATING_MILLAGE: usize = 33;
    pub const EFFECTIVE_CLASS1_MILLAGE: usize = 34;
    pub const OPERATING_EXPENDITURE: usize = 46;
    pub const STATE_REVENUE: usize = 47;
    pub const LOCAL_REVENUE: usize = 49;
}

/// Join the department's FY2027 base cost and summary sheets with profile-report valuation.
#[must_use]
pub fn build_fy27_model(
    base_cost_rows: &[Vec<String>],
    summary_rows: &[Vec<String>],
    profile_rows: &[Vec<String>],
) -> Vec<Vec<String>> {
    use base_cost_columns as bc;

    let summary: Vec<(&str, &Vec<String>)> = rows_by_key(summary_rows, 0)
        .filter(|(_, row)| !is_statewide_row(row, summary_columns::NAME))
        .collect();

    let valuation: Vec<(&str, &Vec<String>)> = profile_rows
        .iter()
        .skip(1)
        .map(|row| (cell(row, profile_columns::IRN).trim(), row))
        .collect();

    let mut districts: Vec<(&str, &Vec<String>)> = rows_by_key(base_cost_rows, 0).collect();
    districts.sort_by(|a, b| a.0.cmp(b.0));

    let mut out = Vec::with_capacity(districts.len());
    for (irn, row) in districts {
        let Some((_, summary_row)) = summary.iter().find(|(key, _)| *key == irn) else {
            continue;
        };
        // A district present in the base cost sheet with no ADM is a placeholder row — a
        // district that closed, or one that exists only to carry a footnote.
        let Some(adm) = cell_number(row, bc::ADM).filter(|adm| *adm > 0.0) else {
            continue;
        };
        let valuation_per_pupil = valuation
            .iter()
            .find(|(key, _)| *key == irn)
            .and_then(|(_, profile)| cell_number(profile, profile_columns::VALUATION_PER_PUPIL));

        out.push(vec![
            irn.to_string(),
            clean_name(cell(row, bc::NAME)),
            format_value(Some(adm), 4),
            format_value(cell_number(row, bc::BUILDINGS), 0),
            format_value(cell_number(row, bc::KINDERGARTEN), 4),
            format_value(cell_number(row, bc::GRADES_1_3), 4),
            format_value(cell_number(row, bc::GRADES_4_8), 4),
            format_value(cell_number(row, bc::GRADES_9_12), 4),
            format_value(cell_number(row, bc::CAREER_TECHNICAL), 4),
            format_value(cell_number(row, bc::GRADES_9_12_TOTAL), 4),
            format_value(cell_number(row, bc::FUNDED_CLASSROOM), 2),
            format_value(cell_number(row, bc::FUNDED_SPECIAL), 2),
            format_value(cell_number(row, bc::TEACHER_COST), 2),
            format_value(cell_number(row, bc::AGGREGATE), 2),
            format_value(cell_number(row, bc::PER_PUPIL), 2),
            // A district off the guarantee records 0, not blank: it is known to have none.
            format_value(
                Some(cell_number(summary_row, summary_columns::GUARANTEE).unwrap_or(0.0)),
                2,
            ),
            format_value(cell_number(row, bc::ADM_FY22), 4),
            format_value(cell_number(row, bc::ADM_FY24), 4),
            format_value(valuation_per_pupil, 2),
            format_value(cell_number(summary_row, summary_columns::CORE), 2),
        ]);
    }
    out
}

/// Columns of the district-profile fixture.
pub const PROFILE_HEADER: &[&str] = &[
    "irn",
    "district",
    "enrolled_adm_fy24",
    "econ_disadvantaged_pct_fy24",
    "assessed_valuation_per_pupil_fy23",
    "current_operating_millage_ty23",
    "effective_class1_millage_ty23",
    "operating_expenditure_per_pupil_fy24",
    "state_revenue_per_pupil_fy24",
    "local_revenue_per_pupil_fy24",
];

/// Reduce the District Profile Report's sixty columns to the ten the corpus uses.
#[must_use]
pub fn build_profile_extract(profile_rows: &[Vec<String>]) -> Vec<Vec<String>> {
    use profile_columns as p;
    let numeric = [
        p::ENROLLED_ADM,
        p::ECON_DISADVANTAGED,
        p::VALUATION_PER_PUPIL,
        p::CURRENT_OPERATING_MILLAGE,
        p::EFFECTIVE_CLASS1_MILLAGE,
        p::OPERATING_EXPENDITURE,
        p::STATE_REVENUE,
        p::LOCAL_REVENUE,
    ];
    profile_rows
        .iter()
        .skip(1)
        .filter(|row| !cell(row, p::IRN).trim().is_empty())
        .map(|row| {
            let mut record = Vec::with_capacity(PROFILE_HEADER.len());
            record.push(cell(row, p::IRN).trim().to_string());
            record.push(clean_name(cell(row, p::NAME)));
            record.extend(
                numeric
                    .iter()
                    .map(|index| format_value(number(cell(row, *index)), 4)),
            );
            record
        })
        .collect()
}

/// Write a fixture with LF endings, so git sees no spurious churn on a rebuild.
///
/// # Errors
///
/// Returns the underlying [`io::Error`] if the directory cannot be created or the file written.
pub fn write_csv(path: &Path, header: &[&str], rows: &[Vec<String>]) -> io::Result<usize> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut out = String::with_capacity(rows.len() * 160 + 256);
    out.push_str(&header.join(","));
    out.push('\n');
    for row in rows {
        out.push_str(&row.join(","));
        out.push('\n');
    }
    fs::write(path, out)?;
    Ok(rows.len())
}

/// Where each fixture is written, relative to the repository root.
pub const FY27_FIXTURE: &str = "crates/foundation/fixtures/fy27-department-model.csv";
/// Where the district-profile fixture is written, relative to the repository root.
pub const PROFILE_FIXTURE: &str = "crates/dispersion/fixtures/cupp-fy24-district-data.csv";
/// Where the deflator's check fixture is written, relative to the repository root.
pub const CPI_FIXTURE: &str = "crates/connect/fixtures/cpi-u-june.tsv";

/// Reduce the Bureau's 2.7 MB all-series flat file to the one series and period the deflator
/// uses.
///
/// Committing the extract rather than the whole file is what makes the deflator's verification
/// hermetic: [`deflate`](../deflate/) can claim its index points are checked against the
/// agency, and a test proves it without a network. Lines are kept in their published form so
/// [`crate::cpi::parse_series`] reads the extract and the original identically.
#[must_use]
pub fn build_cpi_extract(text: &str, series_id: &str, period: &str) -> String {
    let mut out = String::with_capacity(8 * 1024);
    let mut lines = text.lines();
    if let Some(header) = lines.next() {
        out.push_str(header);
        out.push('\n');
    }
    for line in lines {
        let mut fields = line.split('\t').map(str::trim);
        if fields.next() == Some(series_id) && fields.nth(1) == Some(period) {
            out.push_str(line.trim_end());
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conventions::STATEWIDE_ROW;

    fn row(width: usize, values: &[(usize, &str)]) -> Vec<String> {
        let mut row = vec![String::new(); width];
        for (index, value) in values {
            row[*index] = (*value).to_string();
        }
        row
    }

    /// Three banner rows, a header, then two districts — the department's layout.
    fn base_cost_rows() -> Vec<Vec<String>> {
        let district = |irn: &str, name: &str, buildings: &str| {
            row(
                59,
                &[
                    (0, irn),
                    (1, name),
                    (2, "Somewhere"),
                    (3, buildings),
                    (4, "2200"),
                    (6, "2100"),
                    (7, "2150.5"),
                    (8, "150"),
                    (9, "450"),
                    (10, "760"),
                    (11, "700"),
                    (12, "90"),
                    (13, "790"),
                    (14, "84.21"),
                    (17, "14.34"),
                    (22, "9500000.55"),
                    (57, "17400000.25"),
                    (58, "8091.16"),
                ],
            )
        };
        vec![
            row(1, &[(0, "Ohio Department of Education & Workforce")]),
            row(1, &[(0, "")]),
            row(1, &[(0, "Base Cost")]),
            row(4, &[(0, "District IRN"), (1, "District Names")]),
            district("043786", "Cleveland", "10"),
            district("049056", "Northern", "3"),
        ]
    }

    fn summary_rows() -> Vec<Vec<String>> {
        vec![
            row(4, &[(0, "District IRN"), (1, "District Names")]),
            row(
                12,
                &[(0, "043786"), (1, "Cleveland"), (10, "10000000"), (11, "0")],
            ),
            row(
                12,
                &[
                    (0, "049056"),
                    (1, "Northern"),
                    (10, "7000000"),
                    (11, "2500000"),
                ],
            ),
            // The aggregate the department ships in this sheet. Numeric IRN; must be excluded.
            row(
                12,
                &[
                    (0, "999999"),
                    (1, STATEWIDE_ROW),
                    (10, "999999999"),
                    (11, "888888888"),
                ],
            ),
        ]
    }

    fn profile_rows() -> Vec<Vec<String>> {
        let district = |irn: &str, name: &str, valuation: &str| {
            row(
                50,
                &[
                    (0, name),
                    (1, irn),
                    (4, "2100"),
                    (11, "0.42"),
                    (21, valuation),
                    (33, "31.35"),
                    (34, "20"),
                    (46, "11986.62"),
                    (47, "6423.28"),
                    (49, "5277.43"),
                ],
            )
        };
        vec![
            row(2, &[(0, "District"), (1, "IRN")]),
            district("043786", "Cleveland Municipal", "184903.95"),
            district("049056", "Northern Local", "279983.24"),
        ]
    }

    fn model() -> Vec<Vec<String>> {
        build_fy27_model(&base_cost_rows(), &summary_rows(), &profile_rows())
    }

    fn field<'a>(rows: &'a [Vec<String>], irn: &str, column: &str) -> &'a str {
        let index = FY27_HEADER.iter().position(|c| *c == column).unwrap();
        &rows.iter().find(|r| r[0] == irn).unwrap()[index]
    }

    #[test]
    fn emits_one_row_per_district_in_irn_order() {
        let rows = model();
        assert_eq!(
            rows.iter().map(|r| r[0].as_str()).collect::<Vec<_>>(),
            ["043786", "049056"]
        );
    }

    #[test]
    fn excludes_the_statewide_aggregate_row() {
        // Its IRN is numeric, so the digit filter keeps it; only the name check drops it.
        assert!(!model().iter().any(|r| r[0] == "999999"));
    }

    #[test]
    fn row_width_matches_the_declared_header() {
        for row in model() {
            assert_eq!(row.len(), FY27_HEADER.len());
        }
    }

    #[test]
    fn a_building_count_that_is_a_multiple_of_ten_keeps_its_zero() {
        // Trimming trailing zeros off an integer turned 10 buildings into 1. Nine districts
        // carried a wrong count in the committed fixture before this was caught.
        assert_eq!(field(&model(), "043786", "school_buildings"), "10");
        assert_eq!(field(&model(), "049056", "school_buildings"), "3");
    }

    #[test]
    fn joins_valuation_from_the_profile_report() {
        assert_eq!(
            field(&model(), "049056", "assessed_valuation_per_pupil_fy23"),
            "279983.24"
        );
    }

    #[test]
    fn carries_the_guarantee_and_core_funding() {
        let rows = model();
        assert_eq!(
            field(&rows, "049056", "temp_transitional_aid_guarantee"),
            "2500000"
        );
        assert_eq!(field(&rows, "049056", "core_foundation_funding"), "7000000");
    }

    #[test]
    fn a_district_with_no_guarantee_records_zero_not_blank() {
        // Blank would read as "unknown"; this district is known to have none.
        assert_eq!(
            field(&model(), "043786", "temp_transitional_aid_guarantee"),
            "0"
        );
    }

    #[test]
    fn skips_a_district_missing_from_the_summary_sheet() {
        let rows = build_fy27_model(&base_cost_rows(), &summary_rows()[..1], &profile_rows());
        assert!(rows.is_empty());
    }

    #[test]
    fn skips_a_district_with_no_usable_adm() {
        let mut base = base_cost_rows();
        base[5][7] = "0".into();
        let rows = build_fy27_model(&base, &summary_rows(), &profile_rows());
        assert_eq!(
            rows.iter().map(|r| r[0].as_str()).collect::<Vec<_>>(),
            ["043786"]
        );
    }

    #[test]
    fn commas_are_stripped_from_district_names() {
        let mut base = base_cost_rows();
        base[5][1] = "Northern Local, Perry".into();
        let rows = build_fy27_model(&base, &summary_rows(), &profile_rows());
        assert!(!rows[1][1].contains(','));
        assert_eq!(rows[1][1], "Northern Local  Perry");
    }

    #[test]
    fn trailing_whitespace_is_stripped_from_district_names() {
        let mut base = base_cost_rows();
        base[5][1] = "Bellefontaine City ".into();
        let rows = build_fy27_model(&base, &summary_rows(), &profile_rows());
        assert_eq!(rows[1][1], "Bellefontaine City");
    }

    #[test]
    fn the_profile_extract_reduces_to_the_declared_columns() {
        let rows = build_profile_extract(&profile_rows());
        assert_eq!(rows.len(), 2);
        for row in &rows {
            assert_eq!(row.len(), PROFILE_HEADER.len());
        }
        assert_eq!(rows[0][0], "043786");
        assert_eq!(rows[0][1], "Cleveland Municipal");
    }

    #[test]
    fn missing_profile_values_are_blank_not_zero() {
        let mut rows = profile_rows();
        rows[1][21] = "#N/A".into();
        let out = build_profile_extract(&rows);
        let index = PROFILE_HEADER
            .iter()
            .position(|c| *c == "assessed_valuation_per_pupil_fy23")
            .unwrap();
        assert_eq!(out[0][index], "");
    }

    #[test]
    fn formats_trim_only_past_a_decimal_point() {
        assert_eq!(format_value(Some(847.823), 4), "847.823");
        assert_eq!(format_value(Some(20.0), 4), "20");
        assert_eq!(format_value(Some(10.0), 0), "10");
        assert_eq!(format_value(Some(100.0), 0), "100");
        assert_eq!(format_value(None, 2), "");
    }

    #[test]
    fn the_cpi_extract_keeps_the_header_and_one_series() {
        let text = "series_id\tyear\tperiod\tvalue\n\
             CUUR0000SA0     \t2000\tM06\t       172.400\t\n\
             CUUR0000SA0     \t2000\tM05\t       171.300\t\n\
             CUUR0000SAF1    \t2000\tM06\t       167.900\t\n";
        let extract = build_cpi_extract(text, "CUUR0000SA0", "M06");
        let lines: Vec<&str> = extract.lines().collect();
        assert_eq!(lines.len(), 2, "header plus one matching observation");
        assert!(lines[0].starts_with("series_id"));
        assert!(lines[1].contains("172.400"));
    }

    #[test]
    fn writes_lf_endings_and_returns_a_row_count() {
        let dir = std::env::temp_dir().join(format!("edfund-fixture-{}", std::process::id()));
        let path = dir.join("nested/out.csv");
        let written = write_csv(
            &path,
            &["a", "b"],
            &[vec!["1".into(), "2".into()], vec!["3".into(), "4".into()]],
        )
        .unwrap();
        assert_eq!(written, 2);
        let raw = fs::read(&path).unwrap();
        assert!(
            !raw.windows(2).any(|w| w == b"\r\n"),
            "CRLF would churn the diff on every rebuild"
        );
        assert!(String::from_utf8_lossy(&raw).starts_with("a,b\n1,2\n"));
        let _ = fs::remove_dir_all(&dir);
    }
}
