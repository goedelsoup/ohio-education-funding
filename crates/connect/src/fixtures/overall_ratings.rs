//! The overall district rating and the progress rating, one column per report card edition.
//!
//! R.C. 3302.10(A)(1) triggers an academic distress commission on three consecutive years of
//! the **overall** rating, and R.C. 3302.10(N)(1) begins a transition out on one year of it at
//! three stars. No other download this repository holds carries that rating: the achievement,
//! value-added and details files each carry one component. The department's `DISTRICT_HIGH_LEVEL`
//! file is the one that does, so it is read here, one edition per column.
//!
//! The **progress** component rating is read beside it because the performance supplement reads
//! the two together. A district qualifies on its overall rating, on a progress rating of three or
//! more, or on a progress rating higher than the year before. The calculators' own headers date
//! those progress columns a year too early, so the editions here are what dates them.
//!
//! The columns are located by header name. The editions held already differ in layout — the
//! 2025-26 file inserts a `Performance Index` column the 2024-25 file does not have — so a
//! positional map would read the wrong rating from one of them.

use std::collections::BTreeMap;

use super::delimited::column;
use super::format::{clean_name, format_value};
use crate::conventions::{cell, is_district_key};

/// Columns of the overall-ratings fixture: the overall rating for each edition, then the
/// progress rating for each, both in [`OVERALL_EDITIONS`] order.
pub const OVERALL_RATINGS_HEADER: &[&str] = &[
    "irn",
    "district",
    "overall_star_rating_2324",
    "overall_star_rating_2425",
    "overall_star_rating_2526",
    "progress_star_rating_2324",
    "progress_star_rating_2425",
    "progress_star_rating_2526",
];

/// The editions, oldest first, by the school year the report card rates.
pub const OVERALL_EDITIONS: &[&str] = &["2023-24", "2024-25", "2025-26"];

/// The rating columns read from each edition, in the order the fixture writes them.
const RATINGS: &[&str] = &["Overall Star Rating", "Progress Component Star Rating"];

/// The sheet both editions put the overall rating on.
pub const OVERALL_SHEET: &str = "DISTRICT_OVERVIEW";

/// The star count out of a rating cell — `2.5 Stars`, `1 Star`.
///
/// The leading token, as a float: the overall rating carries half-stars under R.C.
/// 3302.03(D)(3)(g), which no component rating does. `NR`, not rated, reads as no rating.
fn stars(text: &str) -> Option<f64> {
    text.split_whitespace().next()?.parse().ok()
}

/// Join the `DISTRICT_OVERVIEW` sheets of each edition on IRN.
///
/// `editions` is one sheet's rows per entry of [`OVERALL_EDITIONS`], in that order. A district
/// missing from an edition, or rated with no star, gets a blank cell for that year rather than
/// being dropped, so a row's absence from one year stays visible. The overall ratings come first,
/// one per edition, then the progress ratings.
///
/// # Errors
///
/// Returns the missing column's name if an edition's layout has moved.
pub fn build_overall_ratings(editions: &[&[Vec<String>]]) -> Result<Vec<Vec<String>>, String> {
    let width = editions.len();
    let mut districts: BTreeMap<String, (String, Vec<Option<f64>>)> = BTreeMap::new();
    for (year, rows) in editions.iter().enumerate() {
        let label = format!(
            "the {} DISTRICT_HIGH_LEVEL download",
            OVERALL_EDITIONS.get(year).copied().unwrap_or("unnamed")
        );
        let head = rows.first().cloned().unwrap_or_default();
        let irn = column(&head, "District IRN", &label)?;
        let name = column(&head, "District Name", &label)?;
        let ratings = RATINGS
            .iter()
            .map(|header| column(&head, header, &label))
            .collect::<Result<Vec<_>, _>>()?;
        for row in rows.iter().skip(1) {
            let key = cell(row, irn).trim();
            if !is_district_key(key) {
                continue;
            }
            let entry = districts.entry(key.to_string()).or_insert_with(|| {
                (
                    clean_name(cell(row, name)),
                    vec![None; RATINGS.len() * width],
                )
            });
            for (measure, &at) in ratings.iter().enumerate() {
                entry.1[measure * width + year] = stars(cell(row, at));
            }
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
            &[
                "District IRN",
                "District Name",
                "Overall Star Rating",
                "Progress Component Star Rating",
            ],
            &["045161", "Youngstown City", "2.5 Stars", "3 Stars"],
            &["044263", "Lorain City", "2 Stars", "1 Star"],
        ]);
        let newer = rows(&[
            &[
                "District IRN",
                "District Name",
                "Region",
                "Progress Component Star Rating",
                "Overall Star Rating",
            ],
            &[
                "045161",
                "Youngstown City",
                "Region 5",
                "4 Stars",
                "3 Stars",
            ],
            &["044263", "Lorain City", "Region 2", "2 Stars", "2.5 Stars"],
        ]);
        let out = build_overall_ratings(&[&older, &newer]).unwrap();
        assert_eq!(
            out,
            vec![
                vec!["044263", "Lorain City", "2", "2.5", "1", "2"],
                vec!["045161", "Youngstown City", "2.5", "3", "3", "4"],
            ]
        );
    }

    #[test]
    fn a_district_missing_from_one_edition_keeps_a_blank() {
        let older = rows(&[
            &[
                "District IRN",
                "District Name",
                "Overall Star Rating",
                "Progress Component Star Rating",
            ],
            &["045161", "Youngstown City", "2.5 Stars", "3 Stars"],
        ]);
        let newer = rows(&[
            &[
                "District IRN",
                "District Name",
                "Overall Star Rating",
                "Progress Component Star Rating",
            ],
            &["045161", "Youngstown City", "3 Stars", "4 Stars"],
            &[
                "043901",
                "East Cleveland City School District",
                "1 Star",
                "2 Stars",
            ],
        ]);
        let out = build_overall_ratings(&[&older, &newer]).unwrap();
        assert_eq!(
            out[0],
            vec![
                "043901",
                "East Cleveland City School District",
                "",
                "1",
                "",
                "2"
            ]
        );
    }

    #[test]
    fn a_rating_of_nr_is_blank_not_zero() {
        // Put-in-Bay's 2023-24 progress rating is `NR`; the calculators write it as 0.
        let card = rows(&[
            &[
                "District IRN",
                "District Name",
                "Overall Star Rating",
                "Progress Component Star Rating",
            ],
            &["048975", "Put-In-Bay Local", "3 Stars", "NR"],
        ]);
        let out = build_overall_ratings(&[&card]).unwrap();
        assert_eq!(out[0], vec!["048975", "Put-In-Bay Local", "3", ""]);
    }

    #[test]
    fn a_moved_rating_column_is_named() {
        let moved = rows(&[&["District IRN", "District Name", "Overall Rating"]]);
        let err = build_overall_ratings(&[&moved]).unwrap_err();
        assert!(
            err.contains("2023-24") && err.contains("Overall Star Rating"),
            "{err}"
        );
    }
}
