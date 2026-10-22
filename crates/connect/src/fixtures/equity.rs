//! The Facilities Construction Commission's K-12 equity list, one row per district per list year.
//!
//! R.C. 3318.011 has the department rank every city, local and exempted village district on its
//! three-year average adjusted valuation per pupil, lowest first, and certify the ranking to the
//! Commission by the first of September. The Commission publishes it as a PDF table: a rank, the
//! district's name, its county and the adjusted valuation per pupil the rank was cut on.
//!
//! The ranking is what the Classroom Facilities Assistance Program's local share is read from —
//! R.C. 3318.01(K) makes a district's share of the basic project cost its percentile, for the
//! fiscal year before the controlling board approves the project — so a list year is the year
//! *before* the approvals it governs.
//!
//! # Rows are keyed on name and county
//!
//! The list carries no IRN. Names are not unique (Lakota Local is in Sandusky and in Butler, and
//! Buckeye Local in three counties), and the pair is: the parser refuses an edition where it is
//! not, and refuses one whose ranks are not exactly `1..=n` with no gap and no repeat.
//!
//! # Except a row printed twice
//!
//! The FY2024 edition's text layer prints ranks 445 to 448 twice in a row, mid-page, cell for
//! cell. A row identical in all four cells to one already read is that artifact and is dropped;
//! a repeat that differs in any cell is a second district under one key and is refused.

/// Columns of the equity-rank fixture, one row per district per list year.
pub const EQUITY_RANK_HEADER: &[&str] = &[
    "list_year",
    "rank",
    "district",
    "county",
    "adjusted_valuation_per_pupil",
];

/// One table line split into its cells, or `None` when the line is not a ranked row.
///
/// The table is set in columns separated by runs of spaces, and a name has single spaces inside
/// it, so a cell boundary is two spaces or more.
fn cells(line: &str) -> Option<[&str; 4]> {
    let cells: Vec<&str> = line
        .trim()
        .split("  ")
        .map(str::trim)
        .filter(|cell| !cell.is_empty())
        .collect();
    let [rank, district, county, value] = cells.as_slice() else {
        return None;
    };
    let ranked = !rank.is_empty() && rank.len() <= 3 && rank.bytes().all(|b| b.is_ascii_digit());
    let dollars = value.strip_prefix('$').is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit() || b == b',')
    });
    (ranked && dollars).then_some([rank, district, county, value])
}

/// Every ranked row of one edition's text layer, as fixture rows.
///
/// # Errors
///
/// If the edition does not say it is the `list_year` list, if a name or county carries a comma
/// (the fixture is unquoted), if the ranks are not exactly `1..=n`, or if a district and county
/// pair appears twice.
pub fn equity_ranks(text: &str, list_year: u16) -> Result<Vec<Vec<String>>, String> {
    let heading = format!("Fiscal Year {list_year}");
    if !text.lines().any(|line| line.trim() == heading) {
        return Err(format!("the edition is not headed `{heading}`"));
    }
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for [rank, district, county, value] in text.lines().filter_map(cells) {
        if district.contains(',') || county.contains(',') {
            return Err(format!(
                "rank {rank}: a comma in `{district}` or `{county}`"
            ));
        }
        let value = value.trim_start_matches('$').replace(',', "");
        if !seen.insert((district.to_string(), county.to_string())) {
            let repeat = rows.iter().any(|row| {
                row[1] == rank && row[2] == district && row[3] == county && row[4] == value
            });
            if repeat {
                continue;
            }
            return Err(format!("`{district}` in {county} is listed twice"));
        }
        rows.push(vec![
            list_year.to_string(),
            rank.to_string(),
            district.to_string(),
            county.to_string(),
            value,
        ]);
    }
    for (expected, row) in (1..).zip(&rows) {
        let rank: usize = row[1].parse().map_err(|_| format!("rank `{}`", row[1]))?;
        if rank != expected {
            return Err(format!(
                "rank {rank} where {expected} was expected: the ranks are not 1..=n in order"
            ));
        }
    }
    if rows.is_empty() {
        return Err("no ranked rows".to_string());
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EDITION: &str = "\
                                   Fiscal Year 2025
                      K-12 School District Equity List (by rank)

   Ranking #     K-12 School District            County         Per Pupil
     1           Warren City SD                  Trumbull        $42,315
     2           Campbell City SD                Mahoning        $43,287
     3           Lakota Local SD                 Sandusky        $90,001
     4           Lakota Local SD                 Butler          $190,002
FY2025 K-12 School District Equity List                Page 1 of 16
";

    #[test]
    fn reads_every_ranked_row_and_strips_the_dollars() {
        let rows = equity_ranks(EDITION, 2025).unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!(
            rows[0],
            ["2025", "1", "Warren City SD", "Trumbull", "42315"]
        );
        assert_eq!(rows[3][4], "190002");
    }

    #[test]
    fn a_shared_name_in_two_counties_is_two_districts() {
        let rows = equity_ranks(EDITION, 2025).unwrap();
        let lakota: Vec<_> = rows.iter().filter(|r| r[2] == "Lakota Local SD").collect();
        assert_eq!(lakota.len(), 2);
    }

    #[test]
    fn refuses_the_wrong_year() {
        assert!(equity_ranks(EDITION, 2024).unwrap_err().contains("2024"));
    }

    #[test]
    fn refuses_a_gap_in_the_ranks() {
        let gapped = EDITION.replace("     3   ", "     5   ");
        assert!(equity_ranks(&gapped, 2025)
            .unwrap_err()
            .contains("not 1..=n"));
    }

    #[test]
    fn a_row_printed_twice_is_read_once() {
        let doubled = EDITION.replace(
            "     3           Lakota",
            "     2           Campbell City SD                Mahoning        $43,287\n     3           Lakota",
        );
        assert_eq!(equity_ranks(&doubled, 2025).unwrap().len(), 4);
    }

    #[test]
    fn refuses_a_repeated_district() {
        let repeated = EDITION.replace("Sandusky", "Butler  ");
        assert!(equity_ranks(&repeated, 2025).unwrap_err().contains("twice"));
    }
}
