//! The FY2025 baseline, from what districts were actually paid.
//!
//! # Why this year is a payment report and every other is a calculator
//!
//! [`super::fy27`] and [`super::fy26`] read `State Foundation Funding Calculator` workbooks.
//! FY2025's cannot be read: the department replaced it in place, and the Internet Archive's
//! captures of it are truncated at exactly 1 MiB — 1,033,192 and 1,032,259 bytes stored, against
//! FY2026's complete 4,187,092. The bytes begin `PK` and end mid-file.
//!
//! The department's **final** payment report is open, complete, and answers a better question.
//! A calculator states what a model projected before the year; this states what was paid after it
//! closed. For a baseline the second is the one that belongs.
//!
//! # The bracketed letters shift between years, and that is the trap
//!
//! `Summary SFPR` labels its columns with the department's own tags, and H.B. 96 moved them:
//! it removed supplemental targeted assistance from the run-up to the total and added the base
//! funding and enrollment growth supplements, so everything after `[I]` slid.
//!
//! | tag | FY2025 | FY2026 |
//! |---|---|---|
//! | `[J]` | Supplemental Targeted Assistance | Transportation |
//! | `[K]` | Transportation | Formula Transition Supplement |
//! | `[L]` | Formula Transition Supplement | Base Funding Supplement |
//! | `[M]` | **Total Formula Funding** | Enrollment Growth Supplement |
//! | `[N]` | Preschool Special Education | **Total Formula Funding** |
//!
//! [`super::fy26`]'s module comment says the department's bracketed tags "are stable across both
//! years". That is true of the detail sheets it reads — `[b4]`, `[C7]`, `d1a` — and false here.
//! So this reads by the **text** of the label rather than by its letter, and a tag it cannot find
//! is an error rather than a neighbouring column silently taken.
//!
//! # The foundation quadruple, and why a total alone cannot answer the question
//!
//! `[Ha]` base, `[Hb]` calculated, `[Hc]` phase-in and `[Hd]` funding are taken together because
//! the guarantee is legible only in their sum. A district fully on the guarantee satisfies
//! `[Hd] + [I] = [Ha]` exactly, and for such a district the foundation formula contributes
//! **nothing** to a change between years however far `[Hb]` moves — Delphos City's `[Hb]` falls
//! 40% from here to FY2027 and its foundation aid does not move a cent. Reading only
//! `[Hd] Foundation State Funding` shows the offset and hides what it offsets, which is the
//! question a comparison is asking.
//!
//! `[Hb]` is also the closest published figure to the formula's own output: after the state
//! share, before the phase-in interpolation, before the guarantee. FY2027's workbook has no
//! quadruple at all — at 100% phase-in the department collapsed it to the calculated column
//! alone — so this year and FY2026 are the only two where the interpolation can be seen.
//!
//! # What it takes, and the one column that exists in no other year
//!
//! The twelve columns a biennium comparison needs, plus `[J] Supplemental Targeted Assistance` —
//! the tier H.B. 96 repealed. `formula-component/fsfp-targeted-assistance` records that repeal as
//! "a live zero rather than an absence" and notes the eligibility test still runs and still
//! qualifies districts, while the payment was never computed from its rate. This is the last year
//! it was paid, per district, which is the only place that amount survives.

use std::collections::HashMap;

use super::format::{clean_name, format_value};
use edfund_core::conventions::{cell, cell_number, is_statewide_row, rows_by_key};

/// Columns of the FY2025 baseline fixture.
pub const FY25_HEADER: &[&str] = &[
    "irn",
    "district",
    "county",
    "foundation_base",
    "foundation_calculated",
    "foundation_phase_in",
    "foundation_state_funding",
    "temporary_transitional_aid_guarantee",
    "supplemental_targeted_assistance",
    "transportation",
    "formula_transition_supplement",
    "total_formula_funding",
    "preschool_special_education",
    "special_education_transportation",
    "total_state_support",
    "net_state_funding",
    "open_enrollment_adjustment",
];

/// The sheet this reads, by the name the workbook gives it.
///
/// Spaced, not underscored. The payment report and the calculator disagree about this too: the
/// FY2026 calculator carries both `Summary SFPR` and `Summary_SFPR` as separate sheets.
pub const SUMMARY_SHEET: &str = "Summary SFPR";

/// The per-district detail, for the one column that is not on the summary.
///
/// `[I1] Open Enrollment Adjustment` is the reason 16 districts are paid less than their funding
/// base, and the summary sheet does not carry it. Without this the shortfall is visible and
/// unexplained, which is how it was first read here.
pub const DETAIL_SHEET: &str = "Detailed SFPR";

/// The department's label for each column taken, in fixture order after the three key columns.
///
/// Matched on the text after the tag rather than on the tag, because the tags move between years
/// and the text does not. See the table in this module's comment.
const COLUMNS: &[&str] = &[
    "Foundation Base State Funding",
    "Foundation Calculated State Funding",
    "Foundation Phase-In Funding",
    "Foundation State Funding",
    "Temporary Transitional Aid Guarantee",
    "Supplemental Targeted Assistance",
    "Transportation",
    "Formula Transition Supplement",
    "Total Formula Funding",
    "Preschool Special Education",
    "Special Education Transportation",
    "Total State Support",
    "Net State Funding",
];

/// The header row, found rather than assumed, and the rows after it.
fn table<'a>(
    rows: &'a [Vec<String>],
    sheet: &str,
) -> Result<(&'a [String], &'a [Vec<String>]), String> {
    let at = rows
        .iter()
        .position(|row| cell(row, 0).trim() == "District IRN")
        .ok_or_else(|| format!("{sheet}: no `District IRN` header row"))?;
    Ok((&rows[at], &rows[at + 1..]))
}

/// The index of the column whose label ends with `text`, ignoring the department's tag.
///
/// `[Hd] Foundation State Funding` and `[Ha] Foundation Base State Funding` differ only in the
/// tag and in one word, so the match is on the whole text after the tag and is exact. A
/// `starts_with` on the tag would be shorter and is what makes a shifted scheme silent.
fn column_on(header: &[String], text: &str, sheet: &str) -> Result<usize, String> {
    let matches: Vec<usize> = header
        .iter()
        .enumerate()
        .filter(|(_, name)| {
            let name = name.trim();
            let after = name.rfind(']').map_or(name, |at| name[at + 1..].trim());
            after.eq_ignore_ascii_case(text)
        })
        .map(|(at, _)| at)
        .collect();
    match matches.as_slice() {
        [one] => Ok(*one),
        [] => Err(format!("{sheet}: no column labelled `{text}`")),
        many => Err(format!(
            "{sheet}: `{text}` matches {} columns at {many:?}; the label is not unique",
            many.len()
        )),
    }
}

/// Every district's value in one column, keyed by IRN, statewide row excluded.
fn by_irn<'a>(
    rows: &'a [Vec<String>],
    header: &[String],
    text: &str,
    sheet: &str,
) -> Result<HashMap<&'a str, f64>, String> {
    let index = column_on(header, text, sheet)?;
    Ok(rows_by_key(rows, 0)
        .filter(|(_, row)| !is_statewide_row(row, 1))
        .filter_map(|(irn, row)| Some((irn, cell_number(row, index)?)))
        .collect())
}

/// The FY2025 baseline, one row per district.
///
/// # Errors
///
/// If either header row is absent, or if any of the fourteen column labels this takes is missing
/// from its sheet or matches more than one column. An ambiguous match is what a shifted letter
/// scheme looks like from here, which is why it is an error rather than a fallback to a neighbour.
pub fn build_fy25_baseline(
    summary: &[Vec<String>],
    detail: &[Vec<String>],
) -> Result<Vec<Vec<String>>, String> {
    let (header, rows) = table(summary, SUMMARY_SHEET)?;
    let (detail_header, detail_rows) = table(detail, DETAIL_SHEET)?;
    let name_at = column_on(header, "District Name", SUMMARY_SHEET)?;
    let county_at = column_on(header, "County Name", SUMMARY_SHEET)?;

    let mut taken: Vec<HashMap<&str, f64>> = COLUMNS
        .iter()
        .map(|text| by_irn(rows, header, text, SUMMARY_SHEET))
        .collect::<Result<_, _>>()?;
    taken.push(by_irn(
        detail_rows,
        detail_header,
        "Open Enrollment Adjustment",
        DETAIL_SHEET,
    )?);

    let mut out = Vec::new();
    for (irn, row) in rows_by_key(rows, 0).filter(|(_, row)| !is_statewide_row(row, 1)) {
        let mut record = vec![
            irn.to_string(),
            clean_name(cell(row, name_at)),
            clean_name(cell(row, county_at)),
        ];
        // Every column here is dollars, so two places throughout — unlike `fy26`, which carries
        // a per-column table because its shares and ratios need more.
        for column in &taken {
            record.push(format_value(column.get(irn).copied(), 2));
        }
        out.push(record);
    }
    out.sort_by(|a, b| a[0].cmp(&b[0]));
    Ok(out)
}
