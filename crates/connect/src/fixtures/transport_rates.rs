//! The statewide transportation factors, out of a year's funding calculator.
//!
//! Two rows and six columns, from two workbooks, and the file is small for a reason worth
//! stating: the department publishes one calculator and **replaces it in place**. FY2027 is what
//! it serves; FY2026 survives only in the Internet Archive; FY2022 to FY2025 do not survive at
//! all. So the series of the formula's one endogenous parameter is two observations long, and
//! that is not a shortcut taken here.
//!
//! # Why this reads by label and not by cell
//!
//! Both workbooks put the per-rider amount in `F1` and the per-mile amount in `F2`, so a
//! positional reader would work today on both. It is written to find the department's own labels
//! and take the first number to their right instead, because the two things this must never do
//! are read a *different* factor as if it were this one, and read a moved layout as a plausible
//! number. A label that has gone is an error; a label that has moved is followed.
//!
//! # The self-check, and what it is for
//!
//! `decisions/an-archived-source-is-still-a-source` admits a `web.archive.org` URL on the
//! condition that the file prove it is the publisher's by reproducing the publisher's own
//! arithmetic. That check lives here: every district's published per-rider base must equal its
//! published weighted ridership times the rate read from the header. It is not a rounding
//! tolerance exercise — the two are exact multiples in both years — so any disagreement means
//! the rate, the column, or the file is wrong.
//!
//! The trap it exists for is real. Every archived copy of the **FY2025** calculator is exactly
//! 1,048,576 bytes, truncated mid-stream, and answers 200. That one fails at the zip and never
//! reaches here; a subtler substitution would not, and this is what catches it.

/// Columns of the rate fixture.
pub const TRANSPORT_RATES_HEADER: &[&str] = &[
    "fiscal_year",
    "per_rider",
    "per_mile",
    "sped_proration",
    "minimum_state_share",
    "inflation_factor",
];

/// The sheet both calculators carry the factors on.
pub const TRANSPORT_SHEET: &str = "Transportation";

/// The label whose neighbour is the value, for each factor this extracts.
const FACTORS: &[(&str, &str)] = &[
    ("per_rider", "Average Per-Rider Amount"),
    ("per_mile", "Average Per-Mile Amount"),
    ("sped_proration", "SpEd Transportation Proration Factor"),
    ("minimum_state_share", "State min share"),
    ("inflation_factor", "Inflation factor"),
];

/// One fiscal year's `Transportation` sheet.
pub struct CalculatorYear<'a> {
    /// The fiscal year the model computes, which the sheet does not state.
    pub fiscal_year: u16,
    /// Its rows.
    pub rows: &'a [Vec<String>],
}

/// The first number to the right of a label, searching only the factor block above the table.
fn value_beside(rows: &[Vec<String>], label: &str) -> Option<f64> {
    for row in rows.iter().take(4) {
        let Some(at) = row.iter().position(|cell| cell.trim().contains(label)) else {
            continue;
        };
        for cell in row.iter().skip(at + 1) {
            if let Ok(value) = cell.trim().parse::<f64>() {
                return Some(value);
            }
        }
    }
    None
}

/// Resolve a column of the district table by the department's own header text.
fn column(header: &[String], label: &str) -> Option<usize> {
    header.iter().position(|cell| cell.trim().contains(label))
}

/// Check the rate against the publisher's own arithmetic, on every district it can.
///
/// Returns the number of districts checked, or the first disagreement.
fn reproduces_published_payments(rows: &[Vec<String>], per_rider: f64) -> Result<usize, String> {
    let head = rows
        .iter()
        .position(|row| row.iter().any(|cell| cell.trim() == "District IRN"))
        .ok_or("the transportation sheet has no district table")?;
    let header = &rows[head];
    let weighted = column(header, "[b] Weighted School Bus Ridership")
        .ok_or("no weighted ridership column")?;
    let based = column(header, "[A1] Per Rider Based").ok_or("no per-rider base column")?;

    let mut checked = 0usize;
    for row in rows.iter().skip(head + 1) {
        let (Some(riders), Some(published)) = (
            row.get(weighted).and_then(|c| c.trim().parse::<f64>().ok()),
            row.get(based).and_then(|c| c.trim().parse::<f64>().ok()),
        ) else {
            continue;
        };
        // The aggregate row carries the column's own sum and is not a district. It is left in
        // rather than filtered because it satisfies the identity too — a statewide weighted
        // ridership times the rate is the statewide base — so it costs nothing and excluding it
        // would mean naming it, which on this sheet is impossible: its district name is blank.
        let expected = riders * per_rider;
        if (expected - published).abs() > 0.5 {
            return Err(format!(
                "a district publishes {published:.2} where {riders} weighted riders at \
                 {per_rider} gives {expected:.2}; the rate, the column or the file is wrong"
            ));
        }
        checked += 1;
    }
    if checked < 500 {
        return Err(format!(
            "only {checked} districts carried a checkable per-rider base; Ohio has over 600, \
             so this file is not the whole calculator"
        ));
    }
    Ok(checked)
}

/// One row per fiscal year, oldest first.
///
/// # Errors
///
/// If a factor's label is absent, or if the workbook does not reproduce its own published
/// per-rider bases from the rate this read.
pub fn build_transportation_rates(
    years: &[CalculatorYear<'_>],
) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for year in years {
        let mut values = Vec::new();
        for (name, label) in FACTORS {
            let value = value_beside(year.rows, label).ok_or_else(|| {
                format!(
                    "FY{}: no value beside {label:?}, which is where the {name} is",
                    year.fiscal_year
                )
            })?;
            values.push(value);
        }

        let per_rider = values[0];
        reproduces_published_payments(year.rows, per_rider)
            .map_err(|cause| format!("FY{}: {cause}", year.fiscal_year))?;

        let mut row = vec![year.fiscal_year.to_string()];
        // Three places on the rates, which is what the department states them to, and the full
        // stored precision on the prorations, which are ratios rather than published figures.
        row.push(format!("{:.3}", values[0]));
        row.push(format!("{:.3}", values[1]));
        row.push(format!("{}", values[2]));
        row.push(format!("{}", values[3]));
        row.push(format!("{}", values[4]));
        out.push(row);
    }
    out.sort_by(|a, b| a[0].cmp(&b[0]));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet(per_rider: &str, riders: &str, based: &str) -> Vec<Vec<String>> {
        let mut rows = vec![
            vec![
                String::new(),
                "Inflation factor".into(),
                "0.05".into(),
                "[s1] Average Per-Rider Amount".into(),
                String::new(),
                per_rider.into(),
            ],
            vec![
                String::new(),
                "SpEd Transportation Proration Factor".into(),
                "1".into(),
                "[s2] Average Per-Mile Amount".into(),
                String::new(),
                "7.06".into(),
            ],
            vec![String::new(), "State min share".into(), "0.4583".into()],
            vec![],
            vec![
                "District IRN".into(),
                "[b] Weighted School Bus Ridership".into(),
                "[A1] Per Rider Based".into(),
            ],
        ];
        for _ in 0..600 {
            rows.push(vec!["000442".into(), riders.into(), based.into()]);
        }
        rows
    }

    #[test]
    fn the_factors_are_found_beside_their_labels() {
        let rows = sheet("1275", "386", "492150");
        let years = [CalculatorYear {
            fiscal_year: 2026,
            rows: &rows,
        }];
        let built = build_transportation_rates(&years).expect("the sheet is well formed");
        assert_eq!(
            built[0],
            vec!["2026", "1275.000", "7.060", "1", "0.4583", "0.05"]
        );
    }

    #[test]
    fn a_rate_that_does_not_reproduce_the_published_base_is_rejected() {
        // The right shape, the wrong rate: 386 riders at 1275 is 492,150, not 500,000.
        let rows = sheet("1275", "386", "500000");
        let years = [CalculatorYear {
            fiscal_year: 2026,
            rows: &rows,
        }];
        let error = build_transportation_rates(&years).expect_err("the arithmetic disagrees");
        assert!(
            error.contains("the rate, the column or the file is wrong"),
            "{error}"
        );
    }

    #[test]
    fn a_file_with_too_few_districts_is_not_the_calculator() {
        let mut rows = sheet("1275", "386", "492150");
        rows.truncate(20);
        let years = [CalculatorYear {
            fiscal_year: 2026,
            rows: &rows,
        }];
        let error = build_transportation_rates(&years).expect_err("a truncated file");
        assert!(error.contains("not the whole calculator"), "{error}");
    }
}
