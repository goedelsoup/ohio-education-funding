//! Table SD-1 of the Department of Taxation's abstract: taxable value and rates, by district.
//!
//! Four tax years of assessed valuation, and the reason `project`'s doc comment about
//! unprojectable valuation had to be corrected — a series where there used to be a point.

use super::format::{clean_name, format_value};
use crate::conventions::{cell, cell_number, rows_by_key};

/// Header of the SD-1 panel: one row per district per **tax** year.
///
/// Tax year, not fiscal year, and the column is named so that nothing joins the two by accident.
/// A tax year's charge is collected in the *following* calendar year, half in the first part of
/// it and half in July — so TY2024 money arrives across FY2025 and FY2026. See
/// [`build_sd1_extract`].
pub const SD1_HEADER: &[&str] = &[
    "irn",
    "district",
    "county",
    "tax_year",
    "agricultural_value",
    "residential_value",
    "class1_value",
    "mineral_value",
    "industrial_value",
    "commercial_value",
    "railroad_value",
    "class2_value",
    "real_property_value",
    "public_utility_value",
    "total_value",
    "class1_taxes_charged",
    "class2_taxes_charged",
    "real_property_taxes_charged",
    "real_property_taxes_charged_with_jvsd",
    "public_utility_taxes_charged",
    "class1_rate",
    "class2_rate",
    "real_property_millage",
    "public_utility_millage",
    "value_per_pupil",
    "adm",
];

/// Column positions on SD-1's per-district worksheets.
///
/// Both worksheets in a workbook carry the same 28 columns in the same order, and so do both
/// tax years — it is the *sheet names* that drift, not the layout. The banner above the header
/// is a row shorter in the TY2023 workbook, which is why nothing here indexes rows by number.
mod sd1_columns {
    pub const COUNTY: usize = 0;
    pub const IRN: usize = 2;
    pub const NAME: usize = 4;
    pub const AGRICULTURAL: usize = 5;
    pub const RESIDENTIAL: usize = 6;
    pub const CLASS1_VALUE: usize = 7;
    pub const MINERAL: usize = 8;
    pub const INDUSTRIAL: usize = 9;
    pub const COMMERCIAL: usize = 10;
    pub const RAILROAD: usize = 11;
    pub const CLASS2_VALUE: usize = 12;
    pub const REAL_VALUE: usize = 13;
    pub const PUBLIC_UTILITY_VALUE: usize = 14;
    pub const TOTAL_VALUE: usize = 15;
    pub const CLASS1_TAXES: usize = 16;
    pub const CLASS2_TAXES: usize = 17;
    pub const REAL_TAXES: usize = 18;
    pub const PUBLIC_UTILITY_TAXES: usize = 19;
    pub const CLASS1_RATE: usize = 21;
    pub const CLASS2_RATE: usize = 22;
    pub const REAL_MILLS: usize = 23;
    pub const PUBLIC_UTILITY_MILLS: usize = 24;
    pub const VALUE_PER_PUPIL: usize = 25;
    pub const ADM: usize = 27;
}

/// One tax year of SD-1, as its two per-district worksheets.
///
/// The department publishes the same districts twice in one workbook, differing in one thing:
/// whether the joint vocational school district's operating levy is counted in the taxes
/// charged. 501 of 611 districts are in a JVSD, so the choice moves most of the state — $513
/// million in TY2024 — and carrying only one of the two would make the fixture silently take a
/// side on a question its callers should be able to ask.
#[derive(Debug, Clone, Copy)]
pub struct Sd1Year<'a> {
    /// The tax year the charge was levied for.
    pub tax_year: u16,
    /// The `ExJVS…` worksheet: JVSD operating levies removed.
    pub excluding_jvsd: &'a [Vec<String>],
    /// The `SD1DAT…` worksheet: the same districts with the JVSD levy left in.
    pub including_jvsd: &'a [Vec<String>],
}

/// Reduce SD-1 to one row per district per tax year.
///
/// # What "taxes charged" is, and is not
///
/// It is a **levy**, not a receipt. Three consequences, each of which has produced a wrong
/// number somewhere:
///
/// - **It is gross of the credits the state reimburses.** The department's own note says the
///   figures "include taxes that have been reduced under various property tax programs that are
///   reimbursed to local school districts by the state" — the non-business credit, the
///   owner-occupancy credit and the homestead exemption. About a tenth of what this column calls
///   property tax is state money. Treating it as the local share overstates local effort.
/// - **It is a tax year, not a fiscal year.** A TY2024 charge is collected across calendar 2025,
///   half early and half in July, so it straddles FY2025 and FY2026. Dividing it by a
///   single-fiscal-year denominator counts money the district had not yet received — which is
///   largest exactly where a new levy has just passed.
/// - **Charged is not collected.** Delinquency means receipts run below the charge, and by a
///   district-specific amount this table cannot see.
///
/// Emitted in IRN then tax-year order, so the fixture diffs cleanly.
#[must_use]
pub fn build_sd1_extract(years: &[Sd1Year<'_>]) -> Vec<Vec<String>> {
    use sd1_columns as c;

    let value_columns = [
        c::AGRICULTURAL,
        c::RESIDENTIAL,
        c::CLASS1_VALUE,
        c::MINERAL,
        c::INDUSTRIAL,
        c::COMMERCIAL,
        c::RAILROAD,
        c::CLASS2_VALUE,
        c::REAL_VALUE,
        c::PUBLIC_UTILITY_VALUE,
        c::TOTAL_VALUE,
        c::CLASS1_TAXES,
        c::CLASS2_TAXES,
        c::REAL_TAXES,
    ];
    let rate_columns = [
        c::PUBLIC_UTILITY_TAXES,
        c::CLASS1_RATE,
        c::CLASS2_RATE,
        c::REAL_MILLS,
        c::PUBLIC_UTILITY_MILLS,
        c::VALUE_PER_PUPIL,
        c::ADM,
    ];

    let mut out: Vec<Vec<String>> = Vec::new();
    for year in years {
        // The JVSD-inclusive total, by IRN. Looked up rather than zipped: the two worksheets
        // happen to agree on row order today, and relying on that would break silently.
        let with_jvsd: Vec<(&str, &Vec<String>)> =
            rows_by_key(year.including_jvsd, c::IRN).collect();

        for (key, row) in rows_by_key(year.excluding_jvsd, c::IRN) {
            let irn = pad_irn(key);
            let mut record = vec![
                irn,
                clean_name(cell(row, c::NAME)),
                clean_name(cell(row, c::COUNTY)),
                year.tax_year.to_string(),
            ];
            record.extend(
                value_columns
                    .iter()
                    .map(|i| format_value(cell_number(row, *i), 0)),
            );
            record.push(format_value(
                with_jvsd
                    .iter()
                    .find(|(other, _)| *other == key)
                    .and_then(|(_, other)| cell_number(other, c::REAL_TAXES)),
                0,
            ));
            record.extend(
                rate_columns
                    .iter()
                    .map(|i| format_value(cell_number(row, *i), 4)),
            );
            out.push(record);
        }
    }
    out.sort_by(|a, b| a[0].cmp(&b[0]).then(a[3].cmp(&b[3])));
    out
}

/// Left-pad an Information Retrieval Number to the six digits every other fixture here uses.
///
/// The Department of Taxation stores the IRN as a number, so Manchester Local's `000442` arrives
/// as `442` and Ohio Valley's `061903` as `61903`. Joining that against the department of
/// education's zero-padded keys matches on neither, and the failure mode is an empty join rather
/// than an error.
fn pad_irn(key: &str) -> String {
    if key.len() >= 6 {
        key.to_string()
    } else {
        format!("{key:0>6}")
    }
}
