//! How to read a cell the Ohio Department of Education and Workforce wrote.
//!
//! [`spreadsheet`] knows the file format. This module knows the publisher: which strings mean
//! "no data", which mean "withheld", and which rows in a per-district sheet are not districts.
//! Keeping the two apart means a change in what the department writes is a one-file edit here,
//! and does not touch the reader.
//!
//! Each convention below cost something to learn. They are tests, not comments, for that
//! reason: every one of them produces a confidently wrong number rather than an error.

/// Strings the department writes where there is no value.
///
/// The spreadsheet error markers appear because the published workbooks contain live formulas
/// whose cached results are errors — a district with no reported denominator leaves `#DIV/0!`
/// in the cell, and that is data about the district, not a number.
pub const MISSING: &[&str] = &[
    "", "#N/A", "#DIV/0!", "#VALUE!", "#REF!", "#NULL!", "#NAME?",
];

/// A count small enough that publishing it would identify students.
///
/// This is **not zero**. Summing it as zero understates any aggregate over small districts, and
/// small districts are exactly the ones a school-funding question is usually about.
pub const SUPPRESSED: &str = "<10";

/// The aggregate row the department ships inside its per-district summary sheets.
///
/// Its IRN is numeric, so it survives any digit filter and must be excluded by name. Counting
/// it as a district double-counts every statewide total — which happened here once, and put
/// the guarantee at exactly twice its real size before anyone noticed the factor was suspicious.
pub const STATEWIDE_ROW: &str = "State of Ohio";

/// Parse a published cell into a number, or `None` where there is no usable value.
///
/// Thousands separators and currency symbols are stripped. Suppressed counts return `None`
/// rather than `0.0`: a caller that wants to treat them as zero has to say so, because the two
/// are different claims.
#[must_use]
pub fn number(cell: &str) -> Option<f64> {
    let trimmed = cell.trim();
    if trimmed == SUPPRESSED || MISSING.contains(&trimmed) {
        return None;
    }
    // Thousands separators and dollar signs only. A trailing `%` is deliberately *not*
    // stripped: the department stores shares as fractions, so a cell that reads `50.22%` is
    // not the same quantity as one that reads `0.5022`, and silently turning it into 50.22
    // would be a hundredfold error that nothing downstream could detect.
    let cleaned: String = trimmed.chars().filter(|c| *c != ',' && *c != '$').collect();
    cleaned.parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Whether a row's key column holds a district IRN.
///
/// The department's sheets carry title banners, blank spacers, and column-legend rows above the
/// data. Filtering on an all-digits key drops every one of them.
#[must_use]
pub fn is_district_key(cell: &str) -> bool {
    let trimmed = cell.trim();
    !trimmed.is_empty() && trimmed.bytes().all(|b| b.is_ascii_digit())
}

/// Rows whose key column holds a district IRN, paired with that key.
///
/// The statewide aggregate is **not** filtered out here — its key is numeric too. Excluding it
/// is the caller's job, and [`STATEWIDE_ROW`] says how.
pub fn rows_by_key(
    rows: &[Vec<String>],
    key_column: usize,
) -> impl Iterator<Item = (&str, &Vec<String>)> {
    rows.iter().filter_map(move |row| {
        let key = row.get(key_column).map_or("", |cell| cell.trim());
        is_district_key(key).then_some((key, row))
    })
}

/// A cell by index, or `""` if the row is shorter than that.
///
/// Rows are padded only to their widest populated cell, so a district with nothing in the last
/// column produces a shorter row than its neighbours. Indexing it directly would panic on
/// exactly the districts with missing data.
#[must_use]
pub fn cell(row: &[String], index: usize) -> &str {
    row.get(index).map_or("", String::as_str)
}

/// A numeric cell by index.
#[must_use]
pub fn cell_number(row: &[String], index: usize) -> Option<f64> {
    number(cell(row, index))
}

/// Whether a row is the statewide aggregate rather than a district.
#[must_use]
pub fn is_statewide_row(row: &[String], name_column: usize) -> bool {
    cell(row, name_column).contains(STATEWIDE_ROW)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(cells: &[&str]) -> Vec<String> {
        cells.iter().map(|c| (*c).to_string()).collect()
    }

    #[test]
    fn parses_plain_and_formatted_numbers() {
        assert_eq!(number("1234.5"), Some(1234.5));
        assert_eq!(number(" 1,234.50 "), Some(1234.5));
        assert_eq!(number("$8,241.61"), Some(8241.61));
        assert_eq!(number("-3.25"), Some(-3.25));
    }

    #[test]
    fn department_missing_markers_become_none() {
        for marker in ["", "#N/A", "#DIV/0!", "   ", "#REF!"] {
            assert_eq!(number(marker), None, "{marker}");
        }
    }

    #[test]
    fn suppressed_small_counts_are_none_not_zero() {
        // "<10" means "fewer than ten, withheld". Treating it as 0 understates any aggregate
        // over small districts.
        assert_eq!(number(SUPPRESSED), None);
        assert_ne!(number(SUPPRESSED), Some(0.0));
    }

    #[test]
    fn unparseable_text_is_none_rather_than_a_panic() {
        assert_eq!(number("not a number"), None);
        assert_eq!(number("--"), None);
    }

    #[test]
    fn infinities_do_not_pass_as_numbers() {
        // A cached formula result can be "inf"; it parses in Rust and would poison an average.
        assert_eq!(number("inf"), None);
        assert_eq!(number("NaN"), None);
    }

    #[test]
    fn only_all_digit_keys_are_districts() {
        assert!(is_district_key("043786"));
        assert!(is_district_key(" 049056 "));
        assert!(!is_district_key("District IRN"));
        assert!(!is_district_key(""));
        assert!(!is_district_key("04378A"));
    }

    #[test]
    fn banners_are_dropped_but_the_aggregate_survives_the_filter() {
        let rows = vec![
            row(&["Ohio Department of Education & Workforce"]),
            row(&["District IRN", "District Names"]),
            row(&["043786", "Cleveland Municipal"]),
            row(&["999999", STATEWIDE_ROW]),
        ];
        let keys: Vec<&str> = rows_by_key(&rows, 0).map(|(key, _)| key).collect();
        assert_eq!(keys, ["043786", "999999"]);
    }

    #[test]
    fn the_statewide_row_is_identified_by_name_because_its_key_is_numeric() {
        let aggregate = row(&["999999", "State of Ohio Total"]);
        let district = row(&["043786", "Cleveland Municipal"]);
        assert!(is_statewide_row(&aggregate, 1));
        assert!(!is_statewide_row(&district, 1));
    }

    #[test]
    fn a_short_row_reads_as_empty_rather_than_panicking() {
        let short = row(&["043786", "Cleveland"]);
        assert_eq!(cell(&short, 57), "");
        assert_eq!(cell_number(&short, 57), None);
    }
}
