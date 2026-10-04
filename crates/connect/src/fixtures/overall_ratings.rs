//! The overall district rating, one column per report card edition.
//!
//! R.C. 3302.10(A)(1) triggers an academic distress commission on three consecutive years of
//! the **overall** rating, and R.C. 3302.10(N)(1) begins a transition out on one year of it at
//! three stars. No other download this repository holds carries that rating: the achievement,
//! value-added and details files each carry one component. The department's `DISTRICT_HIGH_LEVEL`
//! file is the one that does, so it is read here, one edition per column.
//!
//! The columns are located by header name. The two editions held already differ in layout — the
//! 2025-26 file inserts a `Performance Index` column the 2024-25 file does not have — so a
//! positional map would read the wrong rating from one of them.

use std::collections::BTreeMap;

use super::delimited::column;
use super::format::{clean_name, format_value};
use crate::conventions::{cell, is_district_key};

/// Columns of the overall-ratings fixture: one rating column per edition, in
/// [`OVERALL_EDITIONS`] order.
pub const OVERALL_RATINGS_HEADER: &[&str] = &[
    "irn",
    "district",
    "overall_star_rating_2425",
    "overall_star_rating_2526",
];

/// The editions, oldest first, by the school year the report card rates.
pub const OVERALL_EDITIONS: &[&str] = &["2024-25", "2025-26"];

/// The sheet both editions put the overall rating on.
pub const OVERALL_SHEET: &str = "DISTRICT_OVERVIEW";

/// The star count out of an overall-rating cell — `2.5 Stars`, `1 Star`.
///
/// The leading token, as a float: the overall rating carries half-stars under R.C.
/// 3302.03(D)(3)(g), which no component rating does.
fn stars(text: &str) -> Option<f64> {
    text.split_whitespace().next()?.parse().ok()
}

/// Join the `DISTRICT_OVERVIEW` sheets of each edition on IRN.
///
/// `editions` is one sheet's rows per entry of [`OVERALL_EDITIONS`], in that order. A district
/// missing from an edition, or rated with no star, gets a blank cell for that year rather than
/// being dropped, so a row's absence from one year stays visible.
///
/// # Errors
///
/// Returns the missing column's name if an edition's layout has moved.
pub fn build_overall_ratings(editions: &[&[Vec<String>]]) -> Result<Vec<Vec<String>>, String> {
    let mut districts: BTreeMap<String, (String, Vec<Option<f64>>)> = BTreeMap::new();
    for (year, rows) in editions.iter().enumerate() {
        let label = format!(
            "the {} DISTRICT_HIGH_LEVEL download",
            OVERALL_EDITIONS.get(year).copied().unwrap_or("unnamed")
        );
        let head = rows.first().cloned().unwrap_or_default();
        let irn = column(&head, "District IRN", &label)?;
        let name = column(&head, "District Name", &label)?;
        let rating = column(&head, "Overall Star Rating", &label)?;
        for row in rows.iter().skip(1) {
            let key = cell(row, irn).trim();
            if !is_district_key(key) {
                continue;
            }
            let entry = districts
                .entry(key.to_string())
                .or_insert_with(|| (clean_name(cell(row, name)), vec![None; editions.len()]));
            entry.1[year] = stars(cell(row, rating));
        }
    }
    Ok(districts
        .into_iter()
        .map(|(irn, (name, ratings))| {
            [irn, name]
                .into_iter()
                .chain(ratings.into_iter().map(|r| format_value(r, 1)))
                .collect()
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(lines: &[&[&str]]) -> Vec<Vec<String>> {
        lines
            .iter()
            .map(|l| l.iter().map(|c| (*c).to_string()).collect())
            .collect()
    }

    #[test]
    fn each_edition_is_read_by_header_not_position() {
        // The 2025-26 layout puts a column between the name and the rating that 2024-25 lacks.
        let older = rows(&[
            &["District IRN", "District Name", "Overall Star Rating"],
            &["045161", "Youngstown City", "2.5 Stars"],
            &["044263", "Lorain City", "2 Stars"],
        ]);
        let newer = rows(&[
            &[
                "District IRN",
                "District Name",
                "Region",
                "Overall Star Rating",
            ],
            &["045161", "Youngstown City", "Region 5", "3 Stars"],
            &["044263", "Lorain City", "Region 2", "2.5 Stars"],
        ]);
        let out = build_overall_ratings(&[&older, &newer]).unwrap();
        assert_eq!(
            out,
            vec![
                vec!["044263", "Lorain City", "2", "2.5"],
                vec!["045161", "Youngstown City", "2.5", "3"],
            ]
        );
    }

    #[test]
    fn a_district_missing_from_one_edition_keeps_a_blank() {
        let older = rows(&[
            &["District IRN", "District Name", "Overall Star Rating"],
            &["045161", "Youngstown City", "2.5 Stars"],
        ]);
        let newer = rows(&[
            &["District IRN", "District Name", "Overall Star Rating"],
            &["045161", "Youngstown City", "3 Stars"],
            &["043901", "East Cleveland City School District", "1 Star"],
        ]);
        let out = build_overall_ratings(&[&older, &newer]).unwrap();
        assert_eq!(out[0], vec!["043901", "East Cleveland City School District", "", "1"]);
    }

    #[test]
    fn a_moved_rating_column_is_named() {
        let moved = rows(&[&["District IRN", "District Name", "Overall Rating"]]);
        let err = build_overall_ratings(&[&moved]).unwrap_err();
        assert!(err.contains("2024-25") && err.contains("Overall Star Rating"), "{err}");
    }
}
