//! The building-level Performance Index rankings the EdChoice designation is cut from.
//!
//! Three sheets of the designated-list workbook — `PI Ranking 2023`, `2024` and `2025` — rank
//! every building the department ranks, statewide, with its index score and its place in the
//! order. Nine thousand rows across the three.
//!
//! # Why this is not the report card's index
//!
//! [`super::report_card`] carries a Performance Index too, and at building level, but for one
//! year and at one decimal place. These sheets carry three years and full published precision:
//! `115.95699999999999` is a workbook cell, and the ranking is decided by digits the report card
//! rounds away. A three-year building panel is also what the designation's own criterion needs —
//! "at least two of the three most recent consecutive rankings" cannot be evaluated from one
//! year at all.
//!
//! # The PI column is named differently on every sheet
//!
//! `Performance Index for Ranking` in 2023, `2024 PI for Ranking` in 2024, `2025 PI for Ranking`
//! in 2025. A reader matching one exact name silently loses two sheets of three, so the six
//! stable headings are matched exactly and the index column is matched by the two shapes the
//! department has used — with **exactly one** column required to match, because a sheet where
//! two do is a sheet that has grown a second index and this reader would pick one arbitrarily.
//!
//! Where the heading does carry a year, it is checked against the sheet's own name rather than
//! trusted or ignored. That is free, and it is the only thing in the file that says which
//! ranking a sheet is.
//!
//! # Rank is not a key
//!
//! Ranks tie — 2,933 ranked rows share 2,837 distinct ranks in 2023 — and a handful of rows
//! carry no rank at all (four, three and seven). Both are carried as they are written: a blank
//! rank stays blank, per [`edfund_core::conventions`], because a building the department
//! declined to rank and a building ranked first are not the same claim.

use super::delimited::column;
use super::format::{clean_name, format_value};
use edfund_core::conventions::{cell, number};

/// Columns of the Performance Index ranking panel, one row per building per ranking year.
pub const PI_RANKING_HEADER: &[&str] = &[
    "ranking_year",
    "rank",
    "lea_irn",
    "lea_name",
    "building_irn",
    "building_name",
    "building_org_type",
    "performance_index",
];

/// The three ranking sheets, oldest first.
///
/// Exported so the rebuild names them once. The year is not written here — it is read off the
/// sheet name, which is the only place the 2023 sheet states which ranking it is.
pub const PI_RANKING_SHEETS: &[&str] = &["PI Ranking 2023", "PI Ranking 2024", "PI Ranking 2025"];

/// The headings that do not move between editions.
const FIXED: &[&str] = &[
    "PI Rank for Designated List",
    "LEA IRN",
    "LEA Name",
    "Building IRN",
    "Building Name",
    "Building Org Type",
];

/// The index column's unchanging tail, the one part of its heading three editions agree on.
const INDEX_SUFFIX: &str = "PI for Ranking";

/// Places kept for a published index score.
///
/// The underlying values carry three decimals; six is enough to hold them and to drop the float
/// noise the workbook stores them with, and rounding further would manufacture ties at a cut
/// this fixture exists to let somebody take.
const INDEX_PLACES: usize = 6;

/// The ranking year a sheet is, read off its own name.
fn ranking_year(sheet: &str) -> Result<u16, String> {
    sheet
        .rsplit(' ')
        .next()
        .and_then(|tail| tail.parse::<u16>().ok())
        .ok_or_else(|| format!("`{sheet}` does not end in the year it ranks"))
}

/// The one column holding the index score, and a check on any year its heading states.
///
/// # Errors
///
/// If no column matches, if more than one does, or if the heading names a year other than the
/// sheet's own.
fn index_column(head: &[String], sheet: &str, year: u16) -> Result<usize, String> {
    let matches: Vec<usize> = head
        .iter()
        .enumerate()
        .filter(|(_, heading)| {
            let heading = heading.trim();
            heading == "Performance Index for Ranking" || heading.ends_with(INDEX_SUFFIX)
        })
        .map(|(at, _)| at)
        .collect();
    let [at] = matches[..] else {
        return Err(format!(
            "`{sheet}` has {} columns ending in `{INDEX_SUFFIX}`, not one",
            matches.len()
        ));
    };
    let heading = head[at].trim();
    if let Some(stated) = heading
        .split_whitespace()
        .next()
        .and_then(|first| first.parse::<u16>().ok())
    {
        if stated != year {
            return Err(format!(
                "`{sheet}` heads its index column `{heading}`, which is not the {year} ranking"
            ));
        }
    }
    Ok(at)
}

/// One row per building per ranking year, oldest year first and by building IRN within a year.
///
/// Every building each sheet ranks is carried, not only the ones near the bottom. The cut this
/// panel exists to let somebody take is a percentile of the whole distribution, so the whole
/// distribution is the fixture — a file holding the bottom fifth would be a file that has
/// already assumed the answer to the question it is for.
///
/// # Errors
///
/// If a sheet's name does not end in a year, if any of the six stable columns is missing, or if
/// the index column cannot be identified unambiguously. All three mean the department reshaped
/// the workbook, and a reader that guessed would fill a column from its neighbour.
pub fn build_pi_rankings(sheets: &[(&str, &[Vec<String>])]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for (sheet, rows) in sheets {
        let year = ranking_year(sheet)?;
        let head = rows.first().cloned().unwrap_or_default();
        let fixed: Vec<usize> = FIXED
            .iter()
            .map(|name| column(&head, name, sheet))
            .collect::<Result<_, _>>()?;
        let [rank, lea_irn, lea_name, building_irn, building_name, org_type] = fixed[..] else {
            unreachable!("FIXED names six columns");
        };
        let index = index_column(&head, sheet, year)?;
        for row in rows.iter().skip(1) {
            let irn = cell(row, building_irn).trim();
            if irn.is_empty() {
                continue;
            }
            out.push(vec![
                year.to_string(),
                format_value(number(cell(row, rank)), 0),
                cell(row, lea_irn).trim().to_string(),
                clean_name(cell(row, lea_name)),
                irn.to_string(),
                clean_name(cell(row, building_name)),
                clean_name(cell(row, org_type)),
                format_value(number(cell(row, index)), INDEX_PLACES),
            ]);
        }
    }
    out.sort_by(|left, right| (&left[0], &left[4]).cmp(&(&right[0], &right[4])));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn head(index: &str) -> Vec<String> {
        let mut head: Vec<String> = FIXED.iter().map(|c| (*c).to_string()).collect();
        head.push(index.to_string());
        head
    }

    fn row(rank: &str, irn: &str, org: &str, pi: &str) -> Vec<String> {
        [
            rank,
            "046607",
            "Solon City",
            irn,
            "Roxbury, Grace L",
            org,
            pi,
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    fn sheet(index: &str) -> Vec<Vec<String>> {
        vec![
            head(index),
            row("1", "032854", "Public School", "115.95699999999999"),
            row("", "113878", "Community School", "80.5"),
        ]
    }

    fn built(name: &str, index: &str) -> Vec<Vec<String>> {
        build_pi_rankings(&[(name, &sheet(index))]).expect("the sheet reads")
    }

    #[test]
    fn each_edition_states_its_index_column_differently_and_all_three_read() {
        for (name, index) in [
            ("PI Ranking 2023", "Performance Index for Ranking"),
            ("PI Ranking 2024", "2024 PI for Ranking"),
            ("PI Ranking 2025", "2025 PI for Ranking"),
        ] {
            let rows = built(name, index);
            assert_eq!(rows.len(), 2, "{name}");
            assert_eq!(rows[0][7], "115.957", "{name}");
        }
    }

    #[test]
    fn the_year_comes_from_the_sheet_name_not_the_heading() {
        assert_eq!(
            built("PI Ranking 2023", "Performance Index for Ranking")[0][0],
            "2023"
        );
        assert_eq!(
            built("PI Ranking 2025", "2025 PI for Ranking")[0][0],
            "2025"
        );
    }

    #[test]
    fn a_heading_naming_another_year_is_an_error() {
        let rows = sheet("2024 PI for Ranking");
        let error = build_pi_rankings(&[("PI Ranking 2025", &rows)]).expect_err("years disagree");
        assert!(error.contains("not the 2025 ranking"), "{error}");
    }

    #[test]
    fn two_index_columns_are_an_error_rather_than_a_guess() {
        let mut rows = sheet("2025 PI for Ranking");
        rows[0].push("2024 PI for Ranking".to_string());
        let error = build_pi_rankings(&[("PI Ranking 2025", &rows)]).expect_err("ambiguous");
        assert!(error.contains("has 2 columns"), "{error}");
    }

    /// A building the department declined to rank is not a building ranked nowhere in
    /// particular, so the blank survives into the fixture.
    #[test]
    fn an_unranked_building_keeps_its_blank_rank() {
        let rows = built("PI Ranking 2025", "2025 PI for Ranking");
        assert_eq!(rows[0][1], "1");
        assert_eq!(rows[1][1], "");
    }

    #[test]
    fn a_comma_in_a_building_name_is_substituted_not_quoted() {
        let rows = built("PI Ranking 2025", "2025 PI for Ranking");
        assert!(rows
            .iter()
            .all(|r| r.iter().all(|cell| !cell.contains(','))));
        assert_eq!(rows[0][5], "Roxbury  Grace L");
    }

    #[test]
    fn a_community_school_is_carried_rather_than_filtered() {
        let rows = built("PI Ranking 2025", "2025 PI for Ranking");
        assert_eq!(rows[1][6], "Community School");
    }

    #[test]
    fn every_row_is_the_header_width() {
        let rows = built("PI Ranking 2024", "2024 PI for Ranking");
        assert!(rows.iter().all(|r| r.len() == PI_RANKING_HEADER.len()));
    }

    #[test]
    fn a_renamed_fixed_column_is_an_error() {
        let mut rows = sheet("2024 PI for Ranking");
        rows[0][5] = "Org Type".to_string();
        let error = build_pi_rankings(&[("PI Ranking 2024", &rows)]).expect_err("renamed");
        assert!(error.contains("layout has moved"), "{error}");
    }
}
