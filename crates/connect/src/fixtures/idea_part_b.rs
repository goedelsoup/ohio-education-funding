//! The department's IDEA Part B final allocations, one row per local education agency per year.
//!
//! Each edition is a PDF table of every agency that received a subgrant: its IRN, its name, the
//! students with disabilities it counted in public and in nonpublic schools, its school-age
//! allocation, and the **proportionate share** — the part of that allocation 34 CFR 300.133
//! requires the agency to spend on equitable services for parentally placed private school
//! children with disabilities. It is the only held source that prices federal special-education
//! money reaching nonpublic pupils, and it prices it per district.
//!
//! # The name is not kept
//!
//! A long name wraps onto the line above or below the IRN, and sometimes onto both, so the line
//! that carries the numbers carries a fragment of the name or none of it. The IRN is the key —
//! names are not unique in Ohio anyway — and the fixture drops the name rather than reassemble it
//! from neighboring lines.
//!
//! # Rows are read from the right
//!
//! The four editions differ in column spacing, in whether a count is written `1,352` or `1352`,
//! and in how a zero share is printed (`$0.00` from FY2022, a bare `0` in FY2021). What they share
//! is the order of the trailing cells, so each IRN line is read right to left: share, allocation,
//! then the two counts if both are there. Three shapes of absence are kept as they are printed
//! rather than filled:
//!
//! - **Blank counts** with a `$0.00` allocation: agencies listed and paid nothing, such as the
//!   Ohio Schools for the Deaf and the Blind in FY2024. Both count cells stay blank.
//! - **A blank share**: FY2021 prints Brookville Local's allocation and no share at all. Blank is
//!   not zero, so the cell stays blank.
//! - **Suppressed counts**: `<10` (FY2021) and `-` (FY2022) are written verbatim.

/// Columns of the allocation panel, one row per agency per fiscal year.
pub const IDEA_PART_B_HEADER: &[&str] = &[
    "fiscal_year",
    "irn",
    "public_swd",
    "nonpublic_swd",
    "allocation",
    "proportionate_share",
];

/// Whether `token` is a dollar amount to the cent, as every edition prints an allocation.
fn is_money(token: &str) -> bool {
    let Some(digits) = token.strip_prefix('$') else {
        return false;
    };
    let Some((whole, cents)) = digits.split_once('.') else {
        return false;
    };
    cents.len() == 2
        && cents.bytes().all(|b| b.is_ascii_digit())
        && !whole.is_empty()
        && whole.bytes().all(|b| b.is_ascii_digit() || b == b',')
}

/// Whether `token` is a count cell: digits with or without thousands separators, or one of the
/// two suppression marks.
fn is_count(token: &str) -> bool {
    token == "<10"
        || token == "-"
        || (!token.is_empty()
            && token.bytes().next().is_some_and(|b| b.is_ascii_digit())
            && token.bytes().all(|b| b.is_ascii_digit() || b == b','))
}

/// A money or count cell with its `$` and separators removed. Suppression marks pass through.
fn plain(token: &str) -> String {
    token.trim_start_matches('$').replace(',', "")
}

/// One IRN line, as the fixture's cells after `fiscal_year`, or `None` when the line is not a row.
fn row(line: &str) -> Option<Result<Vec<String>, String>> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    let (&irn, rest) = tokens.split_first()?;
    if irn.len() != 6 || !irn.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut rest = rest.to_vec();
    let share = match rest.as_slice() {
        [.., a, s] if is_money(a) && is_money(s) => Some(plain(s)),
        [.., a, "0"] if is_money(a) => Some("0".to_string()),
        [.., a] if is_money(a) => None,
        _ => {
            return Some(Err(format!(
                "IRN {irn}: no allocation at the end of `{line}`"
            )))
        }
    };
    rest.truncate(rest.len() - usize::from(share.is_some()));
    let allocation = plain(rest.pop().expect("matched above"));
    // A name is at least one token, so two counts need three tokens before the allocation.
    let (public, nonpublic) = match rest.as_slice() {
        [_, .., p, n] if is_count(p) && is_count(n) => (plain(p), plain(n)),
        _ => (String::new(), String::new()),
    };
    Some(Ok(vec![
        irn.to_string(),
        public,
        nonpublic,
        allocation,
        share.unwrap_or_default(),
    ]))
}

/// Read one edition's text into rows in [`IDEA_PART_B_HEADER`]'s columns.
///
/// `fiscal_year` is the year the edition is registered for, and the text must name it as the
/// table heading does (`FY24`), so a source pointed at the wrong file fails here. An IRN printed
/// twice is refused: the panel is keyed on it.
pub fn idea_part_b_allocations(text: &str, fiscal_year: u16) -> Result<Vec<Vec<String>>, String> {
    let label = format!("FY{:02} ", fiscal_year % 100);
    if !text.contains(&label) {
        return Err(format!("the edition does not name {}", label.trim_end()));
    }
    let mut rows = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in text.lines() {
        let Some(cells) = row(line) else { continue };
        let mut cells = cells?;
        if !seen.insert(cells[0].clone()) {
            return Err(format!("IRN {} is printed twice", cells[0]));
        }
        cells.insert(0, fiscal_year.to_string());
        rows.push(cells);
    }
    if rows.is_empty() {
        return Err("no agency rows".to_string());
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(line: &str) -> Vec<String> {
        row(line).expect("a row").expect("parses")
    }

    #[test]
    fn a_full_row_reads_counts_allocation_and_share() {
        assert_eq!(
            one("043752      Cincinnati Public Schools                        6,654           939      $11,389,587.25 $1,408,510.79"),
            ["043752", "6654", "939", "11389587.25", "1408510.79"]
        );
    }

    #[test]
    fn a_name_ending_in_a_digit_is_not_a_count() {
        assert_eq!(
            one("000736      Wings Academy 1                                          12            0        $43,553.71           $0.00"),
            ["000736", "12", "0", "43553.71", "0.00"]
        );
    }

    #[test]
    fn blank_counts_stay_blank() {
        assert_eq!(
            one("071530      Ohio School For The Deaf                                                    $0.00             $0.00"),
            ["071530", "", "", "0.00", "0.00"]
        );
    }

    #[test]
    fn a_bare_zero_share_is_a_zero_and_a_missing_share_is_blank() {
        assert_eq!(
            one("045104   Willoughby-Eastlake City                 1,228     0   $1,810,176.36             0"),
            ["045104", "1228", "0", "1810176.36", "0"]
        );
        assert_eq!(
            one("048678   Brookville Local                     171     0    $307,019.06"),
            ["048678", "171", "0", "307019.06", ""]
        );
    }

    #[test]
    fn suppression_marks_pass_through() {
        assert_eq!(
            one("065821      Allen County Board of DD                   -             0         $16,417.27         $0.00"),
            ["065821", "-", "0", "16417.27", "0.00"]
        );
        assert_eq!(
            one("000123   Some Academy                <10   <10   $1,000.00   $10.00"),
            ["000123", "<10", "<10", "1000.00", "10.00"]
        );
    }

    #[test]
    fn a_name_only_line_and_a_heading_are_not_rows() {
        assert!(row("            Alliance Academy of").is_none());
        assert!(row("LEA IRN     Organization Name             Disabilities").is_none());
    }

    #[test]
    fn an_irn_line_without_an_allocation_is_an_error() {
        assert!(row("000131      Glass City Academy")
            .expect("an IRN line")
            .is_err());
    }

    #[test]
    fn the_edition_must_name_its_year_and_an_irn_once() {
        let text = "FY24 IDEA Part B\n000131  Glass City Academy  92  0  $124,034.42  $0.00\n";
        let rows = idea_part_b_allocations(text, 2024).expect("reads");
        assert_eq!(rows, [["2024", "000131", "92", "0", "124034.42", "0.00"]]);
        assert!(idea_part_b_allocations(text, 2023).is_err());
        let twice = format!("{text}000131  Glass City Academy  92  0  $124,034.42  $0.00\n");
        assert!(idea_part_b_allocations(&twice, 2024)
            .unwrap_err()
            .contains("twice"));
    }
}
