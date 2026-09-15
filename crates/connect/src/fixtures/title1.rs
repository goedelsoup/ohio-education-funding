//! The Title I formula counts the EdChoice designation's district criterion is cut from.
//!
//! Three sheets of the designated-list workbook — `Title 1 2024`, `2025` and `2026` — give each
//! of 611 districts the count of children in the Title I formula, decomposed into the five
//! categories the formula adds, against the district's 5-to-17 population. The first held source
//! in this repository for Ohio's Title I formula counts at all.
//!
//! # The input to a criterion the corpus holds the output of
//!
//! [`super::designated`]'s `Overview` carries three years of *shares* — `Title 1 Percent Formula
//! 2023-2024` and its two successors — and the average R.C. 3310.03(A)(1)(b) tests. These sheets
//! carry the numerator and the denominator those shares are formed from. So a share that was a
//! published number becomes a computed one, and the criterion becomes checkable rather than
//! merely recorded.
//!
//! # Every data column carries a year, and the year is the point
//!
//! `CENSUS 2021 POVERTY` on one sheet, `CENSUS 2022 POVERTY` on the next; `OCT. 22 NEG.` against
//! `OCT. 23 NEG.`. An exact-name reader reads one sheet of three. But the varying part is not
//! noise to be normalised away — it is **the department's own statement of each input's
//! vintage**, and it says something worth carrying: the 2023-2024 formula count is built from a
//! 2021 census poverty estimate and an October 2022 collection. So the year is parsed out of the
//! heading and written into the fixture as a column, and the four October columns are required
//! to name the *same* year, which is a check the sheet supplies about itself.
//!
//! # Blank is not zero
//!
//! `OCT. 22 DEL.` is blank in every row of `Title 1 2024` and `Title 1 2025`, and reads `0` in
//! every row of `Title 1 2026`. The convention changed between editions, and
//! [`edfund_core::conventions`] is the standing rule: a blank stays blank here. What a reader
//! may conclude is that the department omits where it later prints a zero — which is exactly
//! what [`build_title1_counts`]'s sum check establishes, since the five components reach the
//! published total in all three sheets with the blanks contributing nothing.
//!
//! That is a narrow conclusion and it is the only one available. It does **not** license
//! writing zeros into the fixture: a caller totalling delinquent children across Ohio in
//! 2023-2024 would then report zero where the honest answer is that the department did not
//! publish the column.

use super::delimited::column;
use super::format::{clean_name, format_value};
use edfund_core::conventions::{cell, is_statewide_row, number};

/// Columns of the Title I formula panel, one row per district per published school year.
pub const TITLE1_HEADER: &[&str] = &[
    "school_year",
    "census_year",
    "october_year",
    "district_irn",
    "district_name",
    "census_poverty",
    "neglected",
    "delinquent",
    "foster",
    "tanf",
    "total_formula_count",
    "population_5_to_17",
    "title1_share",
];

/// The three Title I sheets, oldest first.
pub const TITLE1_SHEETS: &[&str] = &["Title 1 2024", "Title 1 2025", "Title 1 2026"];

/// The headings that carry no year, matched exactly.
const FIXED: &[&str] = &["DIST_IRN", "DISTRICT", "TOTAL FORMULA COUNT", "5-17 POP."];

/// The four October-collection components, by the tail of the heading that names each.
///
/// In the order [`TITLE1_HEADER`] writes them, and the order the workbook writes them in.
const OCTOBER: &[&str] = &["NEG.", "DEL.", "FOSTER", "TANF"];

/// How the share column's heading begins. The rest of it is the school year.
const SHARE_PREFIX: &str = "Title 1 Percent Formula ";

/// Places kept for a published share.
///
/// Fifteen, the same as [`super::designated`]'s, because one of the two files' shares is checked
/// against the other's and a rounded copy would turn an identity into an approximation.
const SHARE_PLACES: usize = 15;

/// A heading's year, given the text around it.
fn year_in(heading: &str, prefix: &str, suffix: &str) -> Option<u16> {
    heading
        .trim()
        .strip_prefix(prefix)?
        .strip_suffix(suffix)?
        .trim()
        .parse()
        .ok()
}

/// The one column whose heading is `prefix … suffix`, and the year between them.
fn dated_column(
    head: &[String],
    sheet: &str,
    prefix: &str,
    suffix: &str,
) -> Result<(usize, u16), String> {
    let matches: Vec<(usize, u16)> = head
        .iter()
        .enumerate()
        .filter_map(|(at, heading)| Some((at, year_in(heading, prefix, suffix)?)))
        .collect();
    match matches[..] {
        [found] => Ok(found),
        _ => Err(format!(
            "`{sheet}` has {} columns reading `{prefix}<year>{suffix}`, not one",
            matches.len()
        )),
    }
}

/// The four October components, and the single collection year all four must name.
fn october_columns(head: &[String], sheet: &str) -> Result<(Vec<usize>, u16), String> {
    let found: Vec<(usize, u16)> = OCTOBER
        .iter()
        .map(|tail| dated_column(head, sheet, "OCT. ", &format!(" {tail}")))
        .collect::<Result<_, _>>()?;
    let year = found[0].1;
    if let Some((_, other)) = found.iter().find(|(_, y)| *y != year) {
        return Err(format!(
            "`{sheet}` dates its October components to both 20{year} and 20{other}"
        ));
    }
    Ok((found.iter().map(|(at, _)| *at).collect(), 2000 + year))
}

/// One row per district per school year, oldest year first and by district IRN within a year.
///
/// # The sum is checked, not assumed
///
/// The five components reach `TOTAL FORMULA COUNT` exactly in all 1,833 rows, so a departure is
/// a change in what the formula adds and is worth a failed rebuild rather than a fixture whose
/// columns no longer decompose the thing they sit beside. A missing component counts as nothing
/// toward the sum — which is the only reading the blank delinquent column of the first two
/// editions admits — while still being written blank.
///
/// # Errors
///
/// If a sheet is missing any of the four undated columns, if any dated column cannot be
/// identified unambiguously, if the four October components disagree about their year, or if a
/// row's components do not sum to its published total.
pub fn build_title1_counts(sheets: &[(&str, &[Vec<String>])]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for (sheet, rows) in sheets {
        let head = rows.first().cloned().unwrap_or_default();
        let fixed: Vec<usize> = FIXED
            .iter()
            .map(|name| column(&head, name, sheet))
            .collect::<Result<_, _>>()?;
        let [irn_at, name_at, total_at, population_at] = fixed[..] else {
            unreachable!("FIXED names four columns");
        };
        let (poverty_at, census_year) = dated_column(&head, sheet, "CENSUS ", " POVERTY")?;
        let (october_at, october_year) = october_columns(&head, sheet)?;
        let (share_at, school_year) = head
            .iter()
            .enumerate()
            .find_map(|(at, heading)| {
                Some((at, heading.trim().strip_prefix(SHARE_PREFIX)?.to_string()))
            })
            .ok_or_else(|| format!("`{sheet}` has no `{SHARE_PREFIX}<school year>` column"))?;

        for row in rows.iter().skip(1) {
            let irn = cell(row, irn_at).trim();
            if irn.is_empty() || is_statewide_row(row, name_at) {
                continue;
            }
            let components: Vec<Option<f64>> = std::iter::once(poverty_at)
                .chain(october_at.iter().copied())
                .map(|at| number(cell(row, at)))
                .collect();
            let total = number(cell(row, total_at));
            let summed: f64 = components.iter().flatten().sum();
            if total != Some(summed) {
                return Err(format!(
                    "`{sheet}` gives district {irn} a formula count of {total:?}, and its five \
                     components sum to {summed}"
                ));
            }
            let mut record = vec![
                school_year.clone(),
                census_year.to_string(),
                october_year.to_string(),
                irn.to_string(),
                clean_name(cell(row, name_at)),
            ];
            record.extend(components.into_iter().map(|value| format_value(value, 0)));
            record.push(format_value(total, 0));
            record.push(format_value(number(cell(row, population_at)), 0));
            record.push(format_value(number(cell(row, share_at)), SHARE_PLACES));
            out.push(record);
        }
    }
    out.sort_by(|left, right| (&left[0], &left[3]).cmp(&(&right[0], &right[3])));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn head(census: u16, october: u16, school_year: &str) -> Vec<String> {
        let mut head = vec![
            "DIST_IRN".to_string(),
            "DISTRICT".into(),
            format!("CENSUS {census} POVERTY"),
        ];
        head.extend(OCTOBER.iter().map(|tail| format!("OCT. {october} {tail}")));
        head.push("TOTAL FORMULA COUNT".into());
        head.push("5-17 POP.".into());
        head.push(format!("{SHARE_PREFIX}{school_year}"));
        head
    }

    /// Ada Exempted Village as `Title 1 2024` writes it, delinquent column and all.
    fn ada(delinquent: &str) -> Vec<String> {
        [
            "045187",
            "Ada Exempted Village School District",
            "161",
            "0",
            delinquent,
            "3",
            "1",
            "165",
            "957",
            "0.17241379310344829",
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    fn sheet(delinquent: &str) -> Vec<Vec<String>> {
        vec![head(2021, 22, "2023-2024"), ada(delinquent)]
    }

    fn built(delinquent: &str) -> Vec<Vec<String>> {
        build_title1_counts(&[("Title 1 2024", &sheet(delinquent))]).expect("the sheet reads")
    }

    #[test]
    fn the_vintages_come_out_of_the_headings() {
        let rows = built("");
        assert_eq!(rows[0][0], "2023-2024");
        assert_eq!(rows[0][1], "2021");
        assert_eq!(rows[0][2], "2022");
    }

    /// The trap the whole module is shaped around: three sheets, three sets of column names.
    #[test]
    fn a_later_edition_reads_under_its_own_column_names() {
        let mut rows = vec![head(2023, 24, "2025-2026")];
        rows.push(ada("0"));
        let built = build_title1_counts(&[("Title 1 2026", &rows)]).expect("the sheet reads");
        assert_eq!(built[0][0], "2025-2026");
        assert_eq!(built[0][1], "2023");
        assert_eq!(built[0][2], "2024");
    }

    /// The convention that changed between editions, and the one that did not.
    #[test]
    fn a_blank_component_stays_blank_and_a_printed_zero_stays_zero() {
        assert_eq!(built("")[0][7], "");
        assert_eq!(built("0")[0][7], "0");
    }

    /// The blank contributes nothing to the sum, which is what licenses reading it as an
    /// omission rather than as a number the reader lost.
    #[test]
    fn a_blank_component_still_reaches_the_published_total() {
        assert_eq!(built("")[0][10], "165");
        assert_eq!(built("0")[0][10], "165");
    }

    #[test]
    fn components_that_miss_the_total_are_an_error() {
        let mut rows = sheet("");
        rows[1][2] = "162".to_string();
        let error = build_title1_counts(&[("Title 1 2024", &rows)]).expect_err("the sum moved");
        assert!(error.contains("045187"), "{error}");
        assert!(error.contains("components sum to 166"), "{error}");
    }

    #[test]
    fn october_columns_dated_to_different_years_are_an_error() {
        let mut rows = sheet("");
        rows[0][4] = "OCT. 23 DEL.".to_string();
        let error = build_title1_counts(&[("Title 1 2024", &rows)]).expect_err("years disagree");
        assert!(error.contains("both 2022 and 2023"), "{error}");
    }

    #[test]
    fn a_renamed_undated_column_is_an_error() {
        let mut rows = sheet("");
        rows[0][7] = "FORMULA COUNT".to_string();
        let error = build_title1_counts(&[("Title 1 2024", &rows)]).expect_err("renamed");
        assert!(error.contains("layout has moved"), "{error}");
    }

    #[test]
    fn the_share_keeps_enough_places_to_match_the_overview() {
        assert_eq!(built("")[0][12], "0.172413793103448");
    }

    #[test]
    fn every_row_is_the_header_width() {
        assert!(built("").iter().all(|r| r.len() == TITLE1_HEADER.len()));
    }
}
