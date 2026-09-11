//! The department's FY2026 calculator, in the columns that answer what moved.
//!
//! # Why a second year, and why only sixteen columns of it
//!
//! [`super::fy27`] turns eleven sheets of the department's FY2027 workbook into the 162-column
//! fixture every calculator crate reads. This is the same workbook a year earlier, and the
//! catalog entry for it — `dew-fy26-funding-calculator` — recorded a decision not to take its
//! per-district tables at all:
//!
//! > **What is not taken.** The per-district tables. They are the FY2027 model's a year earlier
//! > and the counts are very nearly identical — bus miles are the same 825,828 in both, public
//! > riders the same 651,702 — so committing them would be four megabytes to restate a panel
//! > already held.
//!
//! Both facts in that sentence are true and the conclusion does not follow from them. Bus miles
//! and rider counts are a survey a year apart and they are stable. The columns
//! `formula-component/fsfp-local-capacity-measure` and
//! `formula-component/fsfp-disadvantaged-pupil-impact-aid` had recorded as unmeasurable "because
//! the corpus holds one year of the calculator" are not:
//!
//! - per-pupil local capacity moves **+9.34% at the median**, and 2 districts of 609 are unchanged;
//! - disadvantaged pupil impact aid moves **-8.57%**, and none is unchanged.
//!
//! So this takes sixteen columns rather than 162 — the two sides of the state share, the local
//! capacity build-up, and the disadvantaged pupil blend — which is about sixty kilobytes and
//! answers the questions. Everything else on those sheets is either already held for FY2027 or
//! genuinely does restate it.
//!
//! # The aggregate row is named here
//!
//! On these sheets the statewide total carries IRN `050765` and the name `State of Ohio`, so
//! [`edfund_core::conventions::is_statewide_row`] excludes it. It is not blank as it is on the
//! `Transportation` sheet, and a reader that guards against only one of the two shapes lets the
//! other through.

use std::collections::HashMap;

use super::format::{clean_name, format_value};
use crate::conventions::{cell, cell_number, is_statewide_row, rows_by_key};

/// Columns of the FY2026 comparison fixture.
pub const FY26_HEADER: &[&str] = &[
    "irn",
    "district",
    "enrolled_adm",
    "base_cost_enrolled_adm",
    "aggregate_base_cost",
    "base_cost_per_pupil",
    "capacity_per_pupil",
    "state_share_percentage",
    "valuation_per_pupil",
    "local_capacity_percentage",
    "benchmark_ratio",
    "dpia_econ_disadvantaged_adm",
    "dpia_directly_certified_adm",
    "dpia_weighted_adm",
    "dpia_percentage",
    "dpia_aid",
];

/// The sheets this reads, by the name the workbook gives them.
pub const DETAIL_SHEET: &str = "Detail_SFPR";
/// The base cost build-up, one row per district.
pub const BASE_COST_SHEET: &str = "Base_Cost";
/// The local capacity build-up, one row per district.
pub const LOCAL_CAPACITY_SHEET: &str = "Local_Capacity";
/// The disadvantaged pupil blend, one row per district.
pub const DPIA_SHEET: &str = "DPIA";

/// The four sheets, as the rebuild reads them.
pub struct Fy26Sheets<'a> {
    /// `Detail_SFPR`.
    pub detail: &'a [Vec<String>],
    /// `Base_Cost`.
    pub base_cost: &'a [Vec<String>],
    /// `Local_Capacity`.
    pub local_capacity: &'a [Vec<String>],
    /// `DPIA`.
    pub dpia: &'a [Vec<String>],
}

/// The first row whose first cell is the department's IRN header, and the rows after it.
///
/// Found rather than assumed, because the `Local_Capacity` sheet carries a spacer row above its
/// header in FY2026 and does not in FY2027. A positional reader works on one of them.
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

/// The index of the column whose header starts with `tag`, trimmed.
///
/// By label, for the reason [`super::transport_rates`] reads by label: a column that has moved is
/// followed and a column that has gone is an error, where a positional reader silently takes its
/// neighbour. The department's own bracketed tags — `[b4]`, `[C7]`, `d1a` — are stable across
/// both years and are what the line-by-line explanation cites.
fn column(header: &[String], tag: &str, sheet: &str) -> Result<usize, String> {
    header
        .iter()
        .position(|name| name.trim().starts_with(tag))
        .ok_or_else(|| format!("{sheet}: no column headed `{tag}`"))
}

/// One column to take: its sheet's rows, that sheet's header, the department's label for it, and
/// the sheet's name for the error message.
type Join<'a> = (&'a [Vec<String>], &'a [String], &'a str, &'a str);

/// Every district's value in one column, keyed by IRN, statewide row excluded.
fn by_irn<'a>(
    rows: &'a [Vec<String>],
    header: &[String],
    tag: &str,
    sheet: &str,
) -> Result<HashMap<&'a str, f64>, String> {
    let index = column(header, tag, sheet)?;
    Ok(rows_by_key(rows, 0)
        .filter(|(_, row)| !is_statewide_row(row, 1))
        .filter_map(|(irn, row)| Some((irn, cell_number(row, index)?)))
        .collect())
}

/// One fiscal year of the department's model, in the columns that let it be compared to another.
///
/// # Errors
///
/// If a sheet has no `District IRN` header row, or a column the department labels is absent.
pub fn build_fy26_model(sheets: &Fy26Sheets<'_>) -> Result<Vec<Vec<String>>, String> {
    let (detail_header, detail) = table(sheets.detail, DETAIL_SHEET)?;
    let (base_header, base) = table(sheets.base_cost, BASE_COST_SHEET)?;
    let (capacity_header, capacity) = table(sheets.local_capacity, LOCAL_CAPACITY_SHEET)?;
    let (dpia_header, dpia) = table(sheets.dpia, DPIA_SHEET)?;

    let name_at = column(detail_header, "District Names", DETAIL_SHEET)?;
    let joins: [Join<'_>; 13] = [
        (detail, detail_header, "[a] Enrolled ADM", DETAIL_SHEET),
        (
            base,
            base_header,
            "[b] Base Cost Enrolled ADM",
            BASE_COST_SHEET,
        ),
        (
            base,
            base_header,
            "[F] Aggregate Base Cost",
            BASE_COST_SHEET,
        ),
        (detail, detail_header, "[b2]", DETAIL_SHEET),
        (detail, detail_header, "[b1]", DETAIL_SHEET),
        (detail, detail_header, "[b4]", DETAIL_SHEET),
        (capacity, capacity_header, "[C1]", LOCAL_CAPACITY_SHEET),
        (capacity, capacity_header, "[C6]", LOCAL_CAPACITY_SHEET),
        (capacity, capacity_header, "[C5]", LOCAL_CAPACITY_SHEET),
        (dpia, dpia_header, "d1a", DPIA_SHEET),
        (dpia, dpia_header, "d1b", DPIA_SHEET),
        (dpia, dpia_header, "d1 ", DPIA_SHEET),
        (dpia, dpia_header, "d2 ", DPIA_SHEET),
    ];

    let mut columns: Vec<HashMap<&str, f64>> = Vec::with_capacity(joins.len() + 1);
    for (rows, header, tag, sheet) in joins {
        columns.push(by_irn(rows, header, tag, sheet)?);
    }
    columns.push(by_irn(
        dpia,
        dpia_header,
        "Disadvantaged Pupil",
        DPIA_SHEET,
    )?);

    // Places per column, matching how `fy27` writes the same quantity. Shares and ratios carry
    // more than dollars because a share rounded to two places is not a share any more.
    const PLACES: [usize; 14] = [4, 4, 2, 2, 2, 10, 2, 12, 10, 4, 4, 4, 12, 2];

    let mut out = Vec::new();
    for (irn, row) in rows_by_key(detail, 0).filter(|(_, row)| !is_statewide_row(row, 1)) {
        let mut written = vec![irn.to_string(), clean_name(cell(row, name_at))];
        for (column, places) in columns.iter().zip(PLACES) {
            written.push(format_value(column.get(irn).copied(), places));
        }
        out.push(written);
    }
    out.sort_by(|left, right| left[0].cmp(&right[0]));
    Ok(out)
}
