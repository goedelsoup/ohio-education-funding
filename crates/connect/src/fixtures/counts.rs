//! The counts the plan multiplies, both published years, and the department's own table of
//! where each one came from.
//!
//! # The question this was built to settle
//!
//! Four corpus nodes recorded that the career-technical and English learner counts are **frozen
//! at FY2021**, on the strength of a column header: the `CTE` sheet writes
//! `Category 1 Career Tech FTE-FY21` and the `EL` sheet writes `Category 1 EL ADM-FY21`. The
//! reading is a natural one and the consequence drawn from it was serious — "a district whose
//! career-technical programme has grown since 2021 is funded for the programme it had".
//!
//! It is wrong, and two things in the same workbook say so.
//!
//! The first is the [`super::fy26`] comparison generalised: every count the plan multiplies,
//! read out of both years' `ADM Data` sheet. English learner Category 1 falls **22%** between
//! the two workbooks and 439 of 611 districts move. A count taken in FY2021 cannot do that.
//!
//! The second is [`build_calculator_vintages`]: the `Directions` sheet of each workbook carries
//! the department's own table of *what data each variable is drawn from*, and it says
//! `Categorical FTEs (b-g)` are FY26 — the August #1 collection for the FY2026 model and the
//! November #2 collection for the FY2027 one. The `Notes` sheet carries the same table for the
//! FY2024 and FY2025 models and says FY23. There is no year in which the department says it
//! used FY2021 counts.
//!
//! # Two labels, and only one of them is maintained
//!
//! The `-FY21` suffix appears on exactly two sheets and nowhere else in either workbook. The
//! `ADM Data` sheet those two sheets are driven from carries the same numbers under headers
//! with no year at all, for all 611 districts in both years. So the suffix is a label on the
//! FY2022 report it was written for, carried forward with the sheet.
//!
//! It is not the only one. The `Notes` sheet is byte-identical in both workbooks and describes
//! the FY2024 and FY2025 models — two biennia stale. The FY2026 workbook's `ADM Data` heads its
//! directly certified column `[d1b] FY26 Directly Certified ADM` while its own `Directions`
//! sheet says that column is *"FY25 Directly Certified from December 2024"*.
//!
//! **The `Directions` "Data from" column is the maintained one**: it is rewritten for each
//! year's workbook, and where it can be checked against something independent it holds — its
//! base cost enrolled ADM line moves from "Greater of FY25 or average of (FY25, 24, 23)" to
//! "Greater of FY26 (Nov #2) or average of (FY26, 25, 24)", which is R.C. 3317.02(C) advancing
//! one year exactly as written. A per-sheet column header is not evidence of a vintage; this
//! table is.
//!
//! # The control the workbook supplies itself
//!
//! `[a] School Building Count - FY25` names a year in its header, and the `Directions` table
//! names the same year, and it is **identical for all 611 districts in both workbooks**. That
//! is what a genuinely held-back column looks like. Career-technical Category 1 is identical
//! for 589 of 611 — stable, which is a different thing and has a different cause: a
//! career-technical FTE is fixed by a course schedule and a gifted identification is
//! cumulative, so neither moves much between two collections of one year, while a special
//! education ADM is re-evaluated continuously and moves in 607 districts of 611.

use super::format::{clean_name, format_value};
use crate::conventions::{cell, is_statewide_row};

/// Columns of the counts fixture, one row per district per published year.
pub const CALCULATOR_COUNTS_HEADER: &[&str] = &[
    "fiscal_year",
    "irn",
    "district",
    "enrolled_adm",
    "special_education_1",
    "special_education_2",
    "special_education_3",
    "special_education_4",
    "special_education_5",
    "special_education_6",
    "english_learner_1",
    "english_learner_2",
    "english_learner_3",
    "gifted_k_8",
    "gifted_9_12",
    "career_technical_1",
    "career_technical_2",
    "career_technical_3",
    "career_technical_4",
    "career_technical_5",
    "school_buildings",
];

/// Columns of the vintage fixture: the department's own account of its own inputs.
pub const CALCULATOR_VINTAGES_HEADER: &[&str] =
    &["workbook", "model", "calculation", "variable", "data_from"];

/// The sheet every count in the plan is edited on, and the one both report sheets are driven
/// from.
pub const ADM_SHEET: &str = "ADM Data";
/// The report sheet whose five count columns carry the `-FY21` suffix.
pub const CTE_SHEET: &str = "CTE";
/// The report sheet whose three count columns carry it.
pub const EL_SHEET: &str = "EL";
/// The maintained vintage table, rewritten for each year's workbook.
pub const DIRECTIONS_SHEET: &str = "Directions";
/// The same table for the FY2024 and FY2025 models, carried forward unchanged.
pub const NOTES_SHEET: &str = "Notes";

/// The department's label for each count, in fixture-column order after `district`.
///
/// Matched on the trimmed label and nothing shorter. Two of them begin `[a]` — enrolled ADM and
/// the building count — so a prefix match would fill one column from the other, and the building
/// count is the control this fixture exists to compare against.
const COUNTS: &[(&str, usize)] = &[
    ("[a] Enrolled ADM", 6),
    ("[c1] Sped Ed Cat1 ADM", 6),
    ("[c2] Sped Ed Cat2 ADM", 6),
    ("[c3] Sped Ed Cat3 ADM", 6),
    ("[c4] Sped Ed Cat4 ADM", 6),
    ("[c5] Sped Ed Cat5 ADM", 6),
    ("[c6] Sped Ed Cat6 ADM", 6),
    ("[e1] English learner Cat1 ADM", 6),
    ("[e2] English learner Cat2 ADM", 6),
    ("[e3] English learner Cat3 ADM", 6),
    ("[f1] Gifted Grade K-8", 6),
    ("[f2] Gifted Grade 9-12", 6),
    ("[g1] CTE Cat1 FTE", 6),
    ("[g2] CTE Cat2 FTE", 6),
    ("[g3] CTE Cat3 FTE", 6),
    ("[g4] CTE Cat4 FTE", 6),
    ("[g5] CTE Cat5 FTE", 6),
    ("[a] School Building Count - FY25", 0),
];

/// One year's workbook, in the three sheets this reads.
pub struct CountYear<'a> {
    /// The fiscal year the workbook models, which no sheet states.
    pub fiscal_year: u16,
    /// [`ADM_SHEET`], the editable sheet every count is taken from.
    pub adm: &'a [Vec<String>],
    /// [`CTE_SHEET`], read only to check its labelled columns against the taken ones.
    pub cte: &'a [Vec<String>],
    /// [`EL_SHEET`], likewise.
    pub el: &'a [Vec<String>],
}

/// The `-FY21` columns, against the `ADM Data` column each one restates.
///
/// The whole reason this fixture exists is that four corpus nodes read a vintage off the left
/// half of this table. Checking it at build time is what turns "the suffix is stale" from a
/// reading into an invariant: if a workbook ever makes the two differ, the rebuild stops and
/// says so rather than quietly committing the wrong one.
const SUFFIXED: &[(&str, &str, &str)] = &[
    (
        CTE_SHEET,
        "Category 1 Career Tech FTE-FY21",
        "[g1] CTE Cat1 FTE",
    ),
    (
        CTE_SHEET,
        "Category 2 Career Tech FTE-FY21",
        "[g2] CTE Cat2 FTE",
    ),
    (
        CTE_SHEET,
        "Category 3 Career Tech FTE-FY21",
        "[g3] CTE Cat3 FTE",
    ),
    (
        CTE_SHEET,
        "Category 4 Career Tech FTE-FY21",
        "[g4] CTE Cat4 FTE",
    ),
    (
        CTE_SHEET,
        "Category 5 Career Tech FTE-FY21",
        "[g5] CTE Cat5 FTE",
    ),
    (
        EL_SHEET,
        "Category 1 EL ADM-FY21",
        "[e1] English learner Cat1 ADM",
    ),
    (
        EL_SHEET,
        "Category 2 EL ADM-FY21",
        "[e2] English learner Cat2 ADM",
    ),
    (
        EL_SHEET,
        "Category 3 EL ADM-FY21",
        "[e3] English learner Cat3 ADM",
    ),
];

/// One sheet's column, by IRN.
fn by_irn(
    rows: &[Vec<String>],
    sheet: &str,
    label: &str,
) -> Result<std::collections::BTreeMap<String, f64>, String> {
    let head = header_row(rows, sheet)?;
    let at = column(&rows[head], label, sheet)?;
    let mut out = std::collections::BTreeMap::new();
    for row in &rows[head + 1..] {
        let irn = cell(row, 0).trim().to_string();
        if irn.is_empty() || is_statewide_row(row, 1) {
            continue;
        }
        if let Some(value) = row.get(at).and_then(|raw| raw.trim().parse::<f64>().ok()) {
            out.insert(irn, value);
        }
    }
    Ok(out)
}

/// Check that every `-FY21` column restates an unlabelled one exactly.
///
/// # Errors
///
/// If a suffixed column and the `ADM Data` column it restates disagree for any district, or if
/// either is missing. A disagreement would mean the suffix names a real second vintage, which is
/// the reading this fixture was built to test.
fn suffixed_columns_restate_the_data_sheet(year: &CountYear<'_>) -> Result<usize, String> {
    let mut checked = 0;
    for (sheet, label, adm_label) in SUFFIXED {
        let rows = match *sheet {
            CTE_SHEET => year.cte,
            _ => year.el,
        };
        let labelled = by_irn(rows, sheet, label)?;
        let plain = by_irn(year.adm, ADM_SHEET, adm_label)?;
        for (irn, value) in &labelled {
            let Some(other) = plain.get(irn) else {
                return Err(format!(
                    "FY{}: `{sheet}` has {irn} and `{ADM_SHEET}` does not",
                    year.fiscal_year
                ));
            };
            if (value - other).abs() > 1e-9 {
                return Err(format!(
                    "FY{}: {irn} reads {value} under `{label}` and {other} under `{adm_label}`; \
                     the suffix names a second vintage after all",
                    year.fiscal_year
                ));
            }
            checked += 1;
        }
    }
    Ok(checked)
}

/// One year's workbook, in the two tables that say where its numbers came from.
pub struct VintageYear<'a> {
    /// The fiscal year the workbook models.
    pub workbook: u16,
    /// [`DIRECTIONS_SHEET`].
    pub directions: &'a [Vec<String>],
    /// [`NOTES_SHEET`].
    pub notes: &'a [Vec<String>],
}

/// The header row of a per-district table, found by its first cell rather than by position.
fn header_row(rows: &[Vec<String>], sheet: &str) -> Result<usize, String> {
    rows.iter()
        .position(|row| cell(row, 0).trim() == "District IRN")
        .ok_or_else(|| format!("`{sheet}` has no `District IRN` header row"))
}

/// The column carrying a label, matched whole so that `[a] Enrolled ADM` and
/// `[a] School Building Count - FY25` cannot be confused for one another.
fn column(header: &[String], label: &str, sheet: &str) -> Result<usize, String> {
    header
        .iter()
        .position(|found| found.trim() == label)
        .ok_or_else(|| format!("`{sheet}` states no `{label}`"))
}

/// One row per district per year, oldest year first and by IRN within a year.
///
/// # Errors
///
/// If a year's `ADM Data` sheet is missing its header row or any labelled column. Both mean the
/// department reshaped the sheet, and a reader that guessed at a position would fill a column
/// with a neighbour's numbers.
pub fn build_calculator_counts(years: &[CountYear<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for year in years {
        suffixed_columns_restate_the_data_sheet(year)?;
        let head = header_row(year.adm, ADM_SHEET)?;
        let columns: Vec<(usize, usize)> = COUNTS
            .iter()
            .map(|(label, places)| {
                column(&year.adm[head], label, ADM_SHEET).map(|at| (at, *places))
            })
            .collect::<Result<_, _>>()?;
        for row in &year.adm[head + 1..] {
            let irn = cell(row, 0).trim().to_string();
            if irn.is_empty() || is_statewide_row(row, 1) {
                continue;
            }
            let mut out_row = vec![year.fiscal_year.to_string(), irn, clean_name(cell(row, 1))];
            for (at, places) in &columns {
                let value = row.get(*at).and_then(|raw| raw.trim().parse::<f64>().ok());
                out_row.push(format_value(value, *places));
            }
            out.push(out_row);
        }
    }
    out.sort_by(|left, right| (&left[0], &left[1]).cmp(&(&right[0], &right[1])));
    Ok(out)
}

/// Where a vintage table starts, and which fiscal year each of its value columns describes.
///
/// The `Directions` sheet heads its one value column `Data from` and describes the workbook's
/// own year. The `Notes` sheet heads two, `Data used for FY24` and `Data used for FY25`, and the
/// years are read out of those headings rather than assumed — if the department ever refreshes
/// that sheet the fixture follows it instead of mislabelling it.
fn vintage_table(rows: &[Vec<String>], workbook: u16) -> Option<(usize, Vec<(usize, u16)>)> {
    let head = rows.iter().position(|row| {
        cell(row, 0).trim() == "Calculation" && cell(row, 1).trim() == "Variable"
    })?;
    let mut models = Vec::new();
    for (at, heading) in rows[head].iter().enumerate().skip(2) {
        let heading = heading.trim();
        if heading == "Data from" {
            models.push((at, workbook));
        } else if let Some(rest) = heading.strip_prefix("Data used for FY") {
            let year = rest.trim().parse::<u16>().ok()?;
            models.push((at, 2000 + year));
        }
    }
    (!models.is_empty()).then_some((head, models))
}

/// Read one vintage table into rows, carrying the calculation group down the blank cells.
fn vintage_rows(rows: &[Vec<String>], workbook: u16, out: &mut Vec<Vec<String>>) {
    let Some((head, models)) = vintage_table(rows, workbook) else {
        return;
    };
    let mut group = String::new();
    for row in &rows[head + 1..] {
        // A blank row ends the table. The `Notes` sheet puts a second, unrelated table under
        // this one — the FY2018-FY2022 salary and cost-per-pupil series — whose `Parameter` and
        // dollar columns sit exactly where this table's `Data used for FY24` and `FY25` do. Read
        // past the separator and `s5a` becomes a variable whose vintage is "Superintendent".
        if row.iter().all(|found| found.trim().is_empty()) {
            break;
        }
        let calculation = cell(row, 0);
        let calculation = calculation.trim();
        if !calculation.is_empty() {
            group = calculation.to_string();
        }
        let variable = cell(row, 1);
        let variable = variable.trim();
        if variable.is_empty() {
            continue;
        }
        for (at, model) in &models {
            let from = cell(row, *at);
            let from = from.trim();
            if from.is_empty() {
                continue;
            }
            out.push(vec![
                workbook.to_string(),
                model.to_string(),
                group.clone(),
                variable.to_string(),
                from.to_string(),
            ]);
        }
    }
}

/// The department's own vintage tables, both workbooks, `Directions` then `Notes`.
///
/// Tab-separated on the way out, because a `data_from` reads "Greater of FY26 (Nov #2) or
/// average of (FY26, 25, 24)" and the commas are part of the department's sentence.
///
/// # Errors
///
/// If a workbook's `Directions` sheet carries no vintage table at all, or a cell in one holds a
/// tab or a newline. The `Notes` sheet is optional: it is a stale carry-forward and a workbook
/// that finally dropped it should rebuild rather than fail.
pub fn build_calculator_vintages(years: &[VintageYear<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for year in years {
        let before = out.len();
        vintage_rows(year.directions, year.workbook, &mut out);
        if out.len() == before {
            return Err(format!(
                "FY{}: `{DIRECTIONS_SHEET}` carries no `Calculation`/`Variable` table",
                year.workbook
            ));
        }
        vintage_rows(year.notes, year.workbook, &mut out);
    }
    if let Some(bad) = out
        .iter()
        .find(|row| row.iter().any(|field| field.contains(['\t', '\n', '\r'])))
    {
        return Err(format!(
            "a vintage field carries a tab or a newline: {bad:?}"
        ));
    }
    Ok(out)
}
