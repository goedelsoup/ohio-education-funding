//! Six years of joint vocational school district foundation payments, per district per year.
//!
//! The third population the department funds by formula, and the one it publishes least visibly.
//! There is no JVSD calculator: traditional districts get a simulator and community and STEM
//! schools get another, and for these forty-nine the department publishes **payment reports** —
//! what was paid, after the year closed. For FY2022 through FY2026 that is the better artefact
//! and for FY2027 it is the only one.
//!
//! It is also filed where nothing named it. The reports sit under `Career-Technical-Funding`,
//! and no path on the department's site spells "Joint Vocational" at all; a probe for one gets a
//! 404 that looks like an absence and is a misspelling. See the catalog entry.
//!
//! # What the series is for: item 7 of H.B. 96
//!
//! Item 7 rewrote how a JVSD's state share of base cost is computed, into the per-pupil form the
//! greenbook sets out:
//!
//! ```text
//! Per-pupil local capacity amount = (Local capacity valuation x 0.0005) / Base cost enrolled ADM
//! Local capacity valuation       = Lesser of three-year average value or most recent annual value
//! State share percentage         = (per-pupil base cost - per-pupil local capacity) / per-pupil base cost
//! ```
//!
//! …with current law's 10% minimum state share maintained. The span here is chosen so the
//! amendment has a before and an after inside one fixture: **FY2022-FY2025 under prior law,
//! FY2026-FY2027 under item 7.**
//!
//! # The rewrite does not change the percentage, and that is the finding
//!
//! Prior law charged a flat half-mill against the whole district: state share dollars were
//! *aggregate base cost less the charge-off*, a subtraction, and the percentage the report prints
//! was a quotient derived after the fact rather than an operand. Item 7 makes the charge
//! per-pupil and multiplies. But base cost per-pupil is aggregate base cost over the same base
//! cost enrolled ADM that divides the charge, so the ADM cancels:
//!
//! ```text
//! (valuation x 0.0005) / ADM          valuation x 0.0005
//! --------------------------   =   ----------------------
//!  aggregate base cost / ADM      aggregate base cost
//! ```
//!
//! [`state_share_percentage_identity`] is that identity, and it holds on **all 49 districts in
//! all six years, both sides of the amendment**, to a part in a million. A reader who priced item
//! 7 by recomputing the share would find it had done nothing.
//!
//! **What does not cancel is the ADM the percentage multiplies.** Prior law applied it to
//! aggregate base cost, which is built on base cost enrolled ADM — a three-year average. Item 7
//! applies it to `[a] Enrolled ADM`, the *current* year. So the amendment moves JVSD base cost
//! aid from a smoothed enrolment count onto a live one, and the districts it moves are the ones
//! whose enrolment is not flat. That is the whole of it, and neither the greenbook's formula nor
//! the act's own list says so.
//!
//! # Three traps, any of which yields a fixture that reads perfectly well and is wrong
//!
//! **The bracket tags are not stable across the series.** In FY2024 and FY2025 `[I]` is
//! *Total State Support*; in FY2026 and FY2027 `[I]` is the *Base Funding Supplement* and total
//! state support has moved to `[J]`. Those are a $559m column and a $1.4m one under one letter.
//! Worse, `[A1]` is *Aggregate Base Cost* in FY2024-25 and *Base Cost Per-Pupil* in FY2026-27 —
//! dollars and dollars-per-pupil, the same tag, a factor of a thousand apart. Every column here
//! is found by the label text after the tag, through `column_any`, which errors on a label it
//! cannot find and on one that matches twice.
//!
//! **Two live columns share a name inside one workbook.** `Detailed SFPR` carries
//! `[i] Base Cost Enrolled ADM` and the `Base Cost` sheet carries `[b] Base Cost Enrolled ADM`.
//! They agree on 44 of the 49 FY2026 districts and disagree on five, and the department's own
//! local capacity is computed from the **`Base Cost` sheet's**. Reading the nearer one reproduces
//! 44 of 49 and misses the rest by 1-5%, which is close enough to read as rounding and is not.
//! `BASE_COST_ADM` is taken off the `Base Cost` sheet for that reason.
//!
//! **Per-pupil base cost is published twice at two precisions, and the department pays on the
//! rounded one.** `Base Cost` `[F]` carries 9612.6462793433 where `Detailed SFPR` `[A1]` carries
//! 9612.65, and the state share dollars are the *rounded* value times enrolled ADM times the
//! share — using the full-precision copy misses Apollo's payment by $3.42. The two agree to a
//! cent, so a check written at cent tolerance passes while throwing away the only thing that
//! distinguishes them. [`build_jvsd_funding`] takes the published rounded column and asserts the
//! dollars reproduce from it exactly.
//!
//! # And the sheet names move twice
//!
//! `JVSD_BaseCost` and `JVSD_DetailedSFPR` through FY2023; `Base Cost` and `Detailed SFPR` from
//! FY2024. The column spellings move on a different schedule again — CamelCase in FY2022
//! (`StateSharePercent`), plain labels in FY2023 (`State Share Percentage`), tagged from FY2024
//! (`[A3] State Share Percentage`) — so no single spelling reads the series and every field below
//! carries the aliases its three eras use.
//!
//! # What FY2027 is, and is not
//!
//! FY2027's report is the September 2026 payment, the first of an open year rather than a final.
//! **Its `[a] Enrolled ADM` is FY2026's on 43 of the 49 districts** and equals base cost enrolled
//! ADM on 39, because FY2027 enrolment does not exist in September 2026. Under prior law that
//! would not have mattered: the state share multiplied a three-year average that was already
//! known. Under item 7 it multiplies the current year, so **FY2027's state share of base cost is
//! provisional in a way no earlier year's is** — a property of the amendment, not of the file.
//! [`carried_forward_enrolment`] counts it, `enrolment_is_provisional` records it per row, and
//! FY2026 is the only closed year in which item 7 can be priced.

use std::collections::BTreeMap;

use edfund_core::conventions::{cell, cell_number};

use super::format::{clean_name, format_value};

/// Columns of the joint vocational district extract.
///
/// Every level the two methods need on one row, so the amendment can be priced without opening a
/// second file: the valuation both laws charge against, the two ADMs item 7 moved between, the
/// aggregate and per-pupil base cost, and the lines as paid.
///
/// `local capacity` is not among them. It is `property_valuation_used` times 0.0005 exactly —
/// verified on all 49 districts in all six years, against both the dollar column prior law
/// publishes and the per-pupil column item 7 publishes — so carrying it would be a second copy of
/// a number the fixture already determines. The check lives in [`build_jvsd_funding`] instead,
/// where a changed millage becomes a named failure rather than a column nobody re-derives.
pub const JVSD_FUNDING_HEADER: &[&str] = &[
    "fiscal_year",
    "irn",
    "district",
    "county",
    "enrolled_adm",
    "base_cost_enrolled_adm",
    "enrolment_is_provisional",
    "base_cost_per_pupil",
    "aggregate_base_cost",
    "property_valuation_annual",
    "property_valuation_three_year_average",
    "property_valuation_used",
    "state_share_percentage",
    "state_share_of_base_cost",
    "core_foundation_funding",
    "guarantee",
    "formula_transition_supplement",
    "base_funding_supplement",
    "total_state_support",
];

/// The per-district sheet carrying the state share and its valuation inputs.
///
/// Two spellings because the department renamed its sheets for FY2024 and the series spans the
/// change. Trying both is not a fallback: a file matching neither is an error.
pub const JVSD_DETAIL_SHEETS: &[&str] = &["Detailed SFPR", "JVSD_DetailedSFPR"];

/// The per-district sheet carrying the base cost build-up.
///
/// The sheet the issue this fixture answers called unreconstructable: five sub-components against
/// staffing ratios, 76 columns of it. Only the four this extract needs are read, and the other
/// seventy-two are why the catalog entry describes the sheet rather than summarising it.
pub const JVSD_BASE_COST_SHEETS: &[&str] = &["Base Cost", "JVSD_BaseCost"];

/// The half-mill the plan charges a joint vocational district's valuation.
///
/// R.C. 3317.16 as item 7 rewrote it, and unchanged by the rewrite: prior law charged it against
/// the district and item 7 charges it per pupil, which — see the module note — is the same
/// percentage either way.
const LOCAL_CAPACITY_MILLAGE: f64 = 0.0005;

/// The minimum state share of base cost, maintained by item 7 from prior law.
///
/// Carried as a constant because it is the floor [`build_jvsd_funding`] checks the published
/// percentage against, not because anything reaches it: **no JVSD sits on this floor in any of
/// the six years**, and the closest any has come is FY2027's 0.10923. That is worth a named
/// failure if it ever changes, since a district arriving on the floor would make the published
/// percentage stop being a quotient.
const MINIMUM_STATE_SHARE: f64 = 0.10;

/// How many districts the department funds under this system.
///
/// An equality, not a floor, and it is checkable against this repository's own answer:
/// `dispersion::lea_directory::joint_vocational_districts()` finds the same forty-nine from the
/// federal directory by two routes that do not use this file. Joint vocational districts are
/// created by a vote of their member boards and none has been created or dissolved in the span.
const JVSD_COUNT: usize = 49;

/// A cent. The reports store cached formula results, so every equality below is exact to well
/// inside this.
const CENT: f64 = 0.01;

/// How far two independently rounded copies of one cent-precise value may sit apart.
///
/// Ten thousand times tighter than the cent they are rounded to, so it cannot mask a real
/// disagreement, and far looser than the last bits of a float, so it does not fail on one.
const ROUNDED_TOLERANCE: f64 = 1e-6;

/// How far a published percentage may sit from the one recomputed from its own inputs.
///
/// A part in a million. The reports publish the percentage to more places than they publish the
/// terms, so the residual is the terms' own rounding and not a disagreement about the formula.
const SHARE_TOLERANCE: f64 = 1e-6;

/// One year's two sheets, with the year they are for.
///
/// The fiscal year is passed in rather than read from the `Parameters` sheet, so that a file
/// filed under the wrong year is a mismatch the caller can see rather than a row that quietly
/// labels itself.
pub struct JvsdYear<'a> {
    /// The fiscal year these sheets are the payment report for.
    pub fiscal_year: u16,
    /// Rows of whichever of [`JVSD_DETAIL_SHEETS`] this year spells.
    pub detail: &'a [Vec<String>],
    /// Rows of whichever of [`JVSD_BASE_COST_SHEETS`] this year spells.
    pub base_cost: &'a [Vec<String>],
}

/// The identifying columns, which are the only ones spelled the same way in every era but FY2022.
const IRN: &[&str] = &["JVSD IRN", "FundIRN"];
const NAME: &[&str] = &["JVSD Name", "OrgName"];
const COUNTY: &[&str] = &["County Name", "CountyName"];

/// `[a]`: the current year's enrolled ADM. What item 7 made the state share multiply.
const ENROLLED_ADM: &[&str] = &["Enrolled ADM", "EnrolledADM_CurFY"];

/// `[b]` on the base cost sheet: the three-year average the base cost build-up divides by.
///
/// **Not** `Detailed SFPR`'s `[i]`, which is spelled identically and differs on five of the 49.
/// See the module note.
const BASE_COST_ADM: &[&str] = &["Base Cost Enrolled ADM", "BaseCostEnrolledADM"];

/// `[F]` on the base cost sheet: aggregate base cost over base cost enrolled ADM.
///
/// Absent in FY2022, whose base cost sheet is 72 columns rather than 76. Prior law never divided
/// by a pupil, so the column had nothing to do until item 7 gave it one.
const BASE_COST_PER_PUPIL_FULL: &[&str] = &["District Base Cost Per-Pupil"];

/// `[A1]` on the detail sheet from FY2026: the same quantity rounded to the cent.
///
/// The operand the department actually pays on, which is why this and not
/// [`BASE_COST_PER_PUPIL_FULL`] is what the fixture carries where both exist.
const BASE_COST_PER_PUPIL_PAID: &[&str] = &["Base Cost Per-Pupil"];

/// `[E]`: the aggregate the prior-law subtraction worked on.
const AGGREGATE_BASE_COST: &[&str] = &[
    "Aggregate Base Cost",
    "Aggregate Cost",
    "DistrictAggregateBaseCost",
];

/// `[f]`: the most recent annual valuation. Its label names the tax year and so moves every year.
const VALUATION_ANNUAL: &[&str] = &[
    "TY21 Property Valuation",
    "TY22 Property Valuation",
    "TY23 Property Valuation",
    "TY24 Property Valuation",
    "TY25 Property Valuation",
    "TotalValuationTY2020",
];

/// `[g]` and `[h]`: the three-year average, and whichever of the two the plan charges.
const VALUATION_THREE_YEAR: &[&str] =
    &["3-Years Average Property Valuation", "TotalValuation3yrAvg"];
const VALUATION_USED: &[&str] = &["District Property Valuation", "DistrictPropertyValuation"];

/// `[A]` and `[A3]`: the state share of base cost, in dollars and as a percentage.
const STATE_SHARE_DOLLARS: &[&str] = &[
    "District\u{2019}s State Share of the Base Cost",
    "District's State Share of the Base Cost",
    "DistShareBaseCostCalcBase",
];
const STATE_SHARE_PERCENTAGE: &[&str] = &["State Share Percentage", "StateSharePercent"];

/// `[A2]` under prior law: the charge-off in dollars, which prior law subtracted.
const LOCAL_SHARE_AMOUNT: &[&str] = &["Local Share Amount", "LocalShare"];

/// `[A2]` under item 7: the same charge expressed per pupil.
const LOCAL_CAPACITY_PER_PUPIL: &[&str] = &["Local Capacity Per-Pupil"];

/// The lines as paid, which are spelled the same in every era once the tag is off.
const CORE_FOUNDATION: &[&str] = &["Core Foundation Funding", "CoreFoundationStateFunding"];
const GUARANTEE: &[&str] = &[
    "Temporary Transitional Aid Guarantee",
    "TempTransitionalAidGuaranteeFunding",
];
const TRANSITION_SUPPLEMENT: &[&str] = &[
    "Formula Transition Supplement",
    "FormulaTransitionSupplementFunding",
];
const BASE_FUNDING_SUPPLEMENT: &[&str] = &["Base Funding Supplement"];
const TOTAL_STATE_SUPPORT: &[&str] = &["Total State Support", "TotalStateSupport"];

/// The index of the column whose label, after the department's tag, is one of `names`.
///
/// The tag is stripped before matching and the remainder is compared whole, because the tags are
/// not stable across this series and the labels are not unique on a prefix. Two matches is an
/// error rather than a first-wins, since an ambiguous match is what a renamed column looks like
/// from here.
fn column_any(header: &[String], names: &[&str], sheet: &str) -> Result<usize, String> {
    let matches: Vec<usize> = header
        .iter()
        .enumerate()
        .filter(|(_, label)| {
            let label = label.trim();
            let after = label.rfind(']').map_or(label, |at| label[at + 1..].trim());
            let after: String = after.split_whitespace().collect::<Vec<_>>().join(" ");
            names.iter().any(|name| after.eq_ignore_ascii_case(name))
        })
        .map(|(at, _)| at)
        .collect();
    match matches.as_slice() {
        [one] => Ok(*one),
        [] => Err(format!(
            "{sheet}: no column labelled any of {names:?}; the department has renamed it"
        )),
        many => Err(format!(
            "{sheet}: {names:?} matches {} columns at {many:?}; the label is not unique",
            many.len()
        )),
    }
}

/// The same lookup where the column is legitimately absent in some years.
///
/// `Ok(None)` means no column carries the label, which for the four columns that use this is a
/// fact about which side of the amendment a file sits on. **An ambiguous match is still an
/// error**, because "two columns claim this label" and "no column does" are different situations
/// and only the second one is expected here. Folding them together would turn the collision this
/// module exists to catch into a silently empty field.
fn optional_column(
    header: &[String],
    names: &[&str],
    sheet: &str,
) -> Result<Option<usize>, String> {
    match column_any(header, names, sheet) {
        Ok(at) => Ok(Some(at)),
        Err(why) if why.contains("no column labelled") => Ok(None),
        Err(why) => Err(why),
    }
}

/// The header row of a sheet, found by its first identifying column rather than counted.
fn header_row<'a>(
    rows: &'a [Vec<String>],
    sheet: &str,
) -> Result<(&'a [String], &'a [Vec<String>]), String> {
    let at = rows
        .iter()
        .position(|row| {
            IRN.iter()
                .any(|name| cell(row, 0).trim().eq_ignore_ascii_case(name))
        })
        .ok_or_else(|| format!("{sheet}: no header row; its first column is not an IRN"))?;
    Ok((&rows[at], &rows[at + 1..]))
}

/// Whether the published state share percentage is the one its own inputs imply.
///
/// This is the identity the module note is about, and it is written once because it is the same
/// expression on both sides of the amendment: prior law's charge against aggregate base cost, and
/// item 7's per-pupil charge against per-pupil base cost, differ by a factor of base cost enrolled
/// ADM in both numerator and denominator.
///
/// Returns the recomputed share, floored at `MINIMUM_STATE_SHARE`.
#[must_use]
pub fn state_share_percentage_identity(valuation_used: f64, aggregate_base_cost: f64) -> f64 {
    let charge = valuation_used * LOCAL_CAPACITY_MILLAGE;
    (1.0 - charge / aggregate_base_cost).max(MINIMUM_STATE_SHARE)
}

/// One district's row, before it is formatted.
struct Row {
    irn: String,
    district: String,
    county: String,
    enrolled_adm: f64,
    base_cost_adm: f64,
    base_cost_per_pupil: Option<f64>,
    aggregate_base_cost: f64,
    valuation_annual: f64,
    valuation_three_year: f64,
    valuation_used: f64,
    state_share_percentage: f64,
    state_share_dollars: f64,
    core_foundation: f64,
    guarantee: f64,
    transition_supplement: f64,
    base_funding_supplement: f64,
    total_state_support: f64,
}

/// Build the six-year extract from each year's detail and base cost sheets.
///
/// The years are read in the order given and each is checked against the arithmetic of the law in
/// force for it, which the file itself declares: a year publishing `Local Capacity Per-Pupil` is
/// under item 7 and a year publishing `Local Share Amount` is under prior law. That is a better
/// test than the fiscal year, because it fails if a file is ever filed under the wrong year.
///
/// # Errors
///
/// Returns the label that has moved or become ambiguous, a year carrying neither or both of the
/// two local capacity columns, a district count that is not `JVSD_COUNT`, a district on one
/// sheet and not the other, a charged valuation that is not the lesser of the two published, a
/// local capacity that is not the valuation times `LOCAL_CAPACITY_MILLAGE`, a published state
/// share percentage that its own inputs do not reproduce, a district sitting on
/// `MINIMUM_STATE_SHARE`, or state share dollars that do not reproduce under the method that
/// year's law prescribes.
///
/// Every one of those is a fixture that reads perfectly well and is wrong — and the last is the
/// one that matters, because it is the only check that distinguishes the two methods at all.
pub fn build_jvsd_funding(years: &[JvsdYear<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    let mut previous: Option<(u16, BTreeMap<String, f64>)> = None;

    for year in years {
        let fy = year.fiscal_year;
        let rows = read_year(year)?;

        let enrolment: BTreeMap<String, f64> = rows
            .iter()
            .map(|row| (row.irn.clone(), row.enrolled_adm))
            .collect();
        let carried = previous
            .as_ref()
            .map_or(0, |(_, prior)| carried_forward_enrolment(prior, &enrolment));
        // A year whose enrolment is mostly last year's has not been counted yet. FY2027's
        // September payment is that, on 43 of 49; every closed year here shares none at all.
        let provisional = carried * 2 > JVSD_COUNT;

        for row in &rows {
            out.push(format_row(fy, row, provisional));
        }
        previous = Some((fy, enrolment));
    }
    Ok(out)
}

/// How many districts carry exactly the enrolled ADM they carried the year before.
///
/// Exact equality, not closeness: ADM is published to six places and two independent counts do
/// not agree to the sixth. A district matching to the last place has not been recounted.
#[must_use]
pub fn carried_forward_enrolment(
    prior: &BTreeMap<String, f64>,
    current: &BTreeMap<String, f64>,
) -> usize {
    current
        .iter()
        .filter(|(irn, adm)| {
            prior
                .get(*irn)
                .is_some_and(|was| (*was - **adm).abs() < f64::EPSILON)
        })
        .count()
}

/// Read and check one year.
fn read_year(year: &JvsdYear<'_>) -> Result<Vec<Row>, String> {
    let fy = year.fiscal_year;
    let detail_sheet = format!("FY{fy} JVSD detail");
    let base_sheet = format!("FY{fy} JVSD base cost");

    let (detail_header, detail_rows) = header_row(year.detail, &detail_sheet)?;
    let (base_header, base_rows) = header_row(year.base_cost, &base_sheet)?;

    let at = |names: &[&str]| column_any(detail_header, names, &detail_sheet);
    let at_base = |names: &[&str]| column_any(base_header, names, &base_sheet);

    // Which law this file is under, taken from the file rather than from the year.
    let per_pupil_capacity =
        optional_column(detail_header, LOCAL_CAPACITY_PER_PUPIL, &detail_sheet)?;
    let capacity_amount = optional_column(detail_header, LOCAL_SHARE_AMOUNT, &detail_sheet)?;
    let under_item_7 = match (per_pupil_capacity, capacity_amount) {
        (Some(_), None) => true,
        (None, Some(_)) => false,
        (Some(_), Some(_)) => {
            return Err(format!(
                "{detail_sheet}: publishes both a local capacity per-pupil and a local share \
                 amount; which law it is under cannot be read off it"
            ))
        }
        (None, None) => {
            return Err(format!(
                "{detail_sheet}: publishes neither a local capacity per-pupil nor a local share \
                 amount; the state share cannot be checked"
            ))
        }
    };

    let (c_irn, c_name, c_county) = (at(IRN)?, at(NAME)?, at(COUNTY)?);
    let c_enrolled = at(ENROLLED_ADM)?;
    let c_annual = at(VALUATION_ANNUAL)?;
    let c_three = at(VALUATION_THREE_YEAR)?;
    let c_used = at(VALUATION_USED)?;
    let c_share_pct = at(STATE_SHARE_PERCENTAGE)?;
    let c_share_dollars = at(STATE_SHARE_DOLLARS)?;
    let c_core = at(CORE_FOUNDATION)?;
    let c_guarantee = at(GUARANTEE)?;
    let c_transition = at(TRANSITION_SUPPLEMENT)?;
    let c_total = at(TOTAL_STATE_SUPPORT)?;
    // Both of these exist only on one side of the amendment, which `under_item_7` already checked.
    let c_base_supplement = optional_column(detail_header, BASE_FUNDING_SUPPLEMENT, &detail_sheet)?;
    let c_paid_per_pupil = optional_column(detail_header, BASE_COST_PER_PUPIL_PAID, &detail_sheet)?;

    let b_irn = at_base(IRN)?;
    let b_adm = at_base(BASE_COST_ADM)?;
    let b_aggregate = at_base(AGGREGATE_BASE_COST)?;
    let b_per_pupil = optional_column(base_header, BASE_COST_PER_PUPIL_FULL, &base_sheet)?;

    let base: BTreeMap<&str, &Vec<String>> = base_rows
        .iter()
        .map(|row| (cell(row, b_irn).trim(), row))
        .filter(|(irn, _)| !irn.is_empty())
        .collect();

    let mut out = Vec::new();
    for row in detail_rows {
        let irn = cell(row, c_irn).trim().to_string();
        if irn.is_empty() {
            continue;
        }
        let base_row = base.get(irn.as_str()).ok_or_else(|| {
            format!("{base_sheet}: district {irn} is on the detail sheet and not here")
        })?;

        let number = |at: usize, what: &str| -> Result<f64, String> {
            cell_number(row, at)
                .ok_or_else(|| format!("{detail_sheet}: district {irn} has no {what}"))
        };
        let base_number = |at: usize, what: &str| -> Result<f64, String> {
            cell_number(base_row, at)
                .ok_or_else(|| format!("{base_sheet}: district {irn} has no {what}"))
        };

        let valuation_annual = number(c_annual, "annual valuation")?;
        let valuation_three_year = number(c_three, "three-year average valuation")?;
        let valuation_used = number(c_used, "charged valuation")?;
        let aggregate_base_cost = base_number(b_aggregate, "aggregate base cost")?;
        let base_cost_adm = base_number(b_adm, "base cost enrolled ADM")?;
        let enrolled_adm = number(c_enrolled, "enrolled ADM")?;
        let state_share_percentage = number(c_share_pct, "state share percentage")?;
        let state_share_dollars = number(c_share_dollars, "state share of base cost")?;

        // The lesser of the two, which is the whole of what `[h]` means.
        let lesser = valuation_annual.min(valuation_three_year);
        if (valuation_used - lesser).abs() > CENT {
            return Err(format!(
                "{detail_sheet}: district {irn} is charged on {valuation_used:.2}, which is \
                 neither its annual valuation {valuation_annual:.2} nor its three-year average \
                 {valuation_three_year:.2}"
            ));
        }

        // The half-mill, checked against whichever form this year publishes.
        let charge = valuation_used * LOCAL_CAPACITY_MILLAGE;
        if under_item_7 {
            let published = number(
                per_pupil_capacity.expect("checked above"),
                "local capacity per-pupil",
            )?;
            // Rounded to the cent because that is how the department publishes it, then
            // compared well inside a cent rather than at `f64::EPSILON`: both sides are already
            // the result of a rounding, and two roundings of the same value can differ in the
            // last bits without disagreeing about anything.
            let implied = (charge / base_cost_adm * 100.0).round() / 100.0;
            if (implied - published).abs() > ROUNDED_TOLERANCE {
                return Err(format!(
                    "{detail_sheet}: district {irn} publishes local capacity per-pupil \
                     {published:.2}, and its charged valuation over its base cost enrolled ADM is \
                     {implied:.2}"
                ));
            }
        } else {
            let published = number(
                capacity_amount.expect("checked above"),
                "local share amount",
            )?;
            if (charge - published).abs() > CENT {
                return Err(format!(
                    "{detail_sheet}: district {irn} publishes a local share of {published:.2} and \
                     is charged {charge:.2} on its valuation"
                ));
            }
        }

        // The identity the amendment did not change.
        let implied_share = state_share_percentage_identity(valuation_used, aggregate_base_cost);
        if (implied_share - state_share_percentage).abs() > SHARE_TOLERANCE {
            return Err(format!(
                "{detail_sheet}: district {irn} publishes a state share of \
                 {state_share_percentage:.6} and its valuation over its aggregate base cost gives \
                 {implied_share:.6}"
            ));
        }
        if (state_share_percentage - MINIMUM_STATE_SHARE).abs() < SHARE_TOLERANCE {
            return Err(format!(
                "{detail_sheet}: district {irn} sits on the {MINIMUM_STATE_SHARE} minimum state \
                 share. No JVSD has in the six years this fixture spans, and on the floor the \
                 published percentage stops being a quotient — the checks above no longer say \
                 what the department computed"
            ));
        }

        // The dollars, under the method that year's law prescribes. This is the only check that
        // tells the two methods apart, because the percentage does not.
        let base_cost_per_pupil = if under_item_7 {
            let paid = number(
                c_paid_per_pupil.ok_or_else(|| {
                    format!("{detail_sheet}: is under item 7 and publishes no per-pupil base cost")
                })?,
                "base cost per-pupil",
            )?;
            let expected = paid * enrolled_adm * state_share_percentage;
            if (expected - state_share_dollars).abs() > CENT {
                return Err(format!(
                    "{detail_sheet}: district {irn} is paid {state_share_dollars:.2} of base cost \
                     and its per-pupil base cost {paid:.2} times its enrolled ADM \
                     {enrolled_adm:.6} times its state share {state_share_percentage:.6} is \
                     {expected:.2}"
                ));
            }
            Some(paid)
        } else {
            // Prior law subtracts; it does not multiply. Writing this as aggregate times share
            // passes at a dollar and fails at a cent, which is the tell that the percentage is
            // derived rather than applied.
            let expected = aggregate_base_cost - charge;
            if (expected - state_share_dollars).abs() > CENT {
                return Err(format!(
                    "{detail_sheet}: district {irn} is paid {state_share_dollars:.2} of base cost \
                     and its aggregate base cost {aggregate_base_cost:.2} less its charge-off \
                     {charge:.2} is {expected:.2}"
                ));
            }
            b_per_pupil.and_then(|at| cell_number(base_row, at))
        };

        out.push(Row {
            irn: irn.clone(),
            district: cell(row, c_name).trim().to_string(),
            county: cell(row, c_county).trim().to_string(),
            enrolled_adm,
            base_cost_adm,
            base_cost_per_pupil,
            aggregate_base_cost,
            valuation_annual,
            valuation_three_year,
            valuation_used,
            state_share_percentage,
            state_share_dollars,
            core_foundation: number(c_core, "core foundation funding")?,
            guarantee: number(c_guarantee, "guarantee")?,
            transition_supplement: number(c_transition, "formula transition supplement")?,
            base_funding_supplement: c_base_supplement
                .and_then(|at| cell_number(row, at))
                .unwrap_or(0.0),
            total_state_support: number(c_total, "total state support")?,
        });
    }

    if out.len() != JVSD_COUNT {
        return Err(format!(
            "{detail_sheet}: carries {} districts and Ohio has {JVSD_COUNT} joint vocational \
             school districts",
            out.len()
        ));
    }
    Ok(out)
}

/// One district-year, in the order [`JVSD_FUNDING_HEADER`] names.
///
/// ADM to six places and money to two, which is what the reports publish. The state share
/// percentage is written to eight because the department computes on more places than it prints
/// the terms to, and rounding it to the terms' precision would stop
/// [`state_share_percentage_identity`] reproducing from the fixture the way it reproduces from
/// the file.
fn format_row(fiscal_year: u16, row: &Row, provisional: bool) -> Vec<String> {
    let money = |value: f64| format_value(Some(value), 2);
    let pupils = |value: f64| format_value(Some(value), 6);
    vec![
        fiscal_year.to_string(),
        row.irn.clone(),
        clean_name(&row.district),
        clean_name(&row.county),
        pupils(row.enrolled_adm),
        pupils(row.base_cost_adm),
        if provisional { "yes" } else { "no" }.to_string(),
        format_value(row.base_cost_per_pupil, 2),
        money(row.aggregate_base_cost),
        money(row.valuation_annual),
        money(row.valuation_three_year),
        money(row.valuation_used),
        format_value(Some(row.state_share_percentage), 8),
        money(row.state_share_dollars),
        money(row.core_foundation),
        money(row.guarantee),
        money(row.transition_supplement),
        money(row.base_funding_supplement),
        money(row.total_state_support),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The identity is the same expression on both sides of the amendment, so a per-pupil
    /// restatement of the same charge has to give the same share.
    #[test]
    fn the_per_pupil_rewrite_computes_the_same_share() {
        let (valuation, aggregate, adm) = (1_200_000_000.0, 10_000_000.0, 1_000.0);
        let whole_district = state_share_percentage_identity(valuation, aggregate);

        let per_pupil_capacity = valuation * LOCAL_CAPACITY_MILLAGE / adm;
        let per_pupil_base_cost = aggregate / adm;
        let restated = (per_pupil_base_cost - per_pupil_capacity) / per_pupil_base_cost;

        assert!(
            (whole_district - restated).abs() < SHARE_TOLERANCE,
            "item 7 restates the charge per pupil and the ADM cancels: {whole_district} vs {restated}"
        );
    }

    /// And the ADM it does not cancel out of is the one the share multiplies, which is why the
    /// amendment moves money at all.
    #[test]
    fn what_the_rewrite_moves_is_the_adm_the_share_multiplies() {
        let (valuation, aggregate) = (1_200_000_000.0, 10_000_000.0);
        let base_cost_adm = 1_000.0;
        let share = state_share_percentage_identity(valuation, aggregate);

        let prior_law = aggregate - valuation * LOCAL_CAPACITY_MILLAGE;
        // A district whose current enrolment is 10% above its three-year average.
        let item_7 = (aggregate / base_cost_adm) * (base_cost_adm * 1.10) * share;

        assert!(
            (item_7 - prior_law * 1.10).abs() < CENT,
            "the rewrite scales base cost aid by current enrolment over the smoothed count"
        );
    }

    /// The floor is a `max`, so a district below it is lifted rather than dropped.
    #[test]
    fn the_minimum_state_share_is_a_floor() {
        // A valuation large enough to exhaust the whole base cost twice over.
        let share = state_share_percentage_identity(60_000_000_000.0, 10_000_000.0);
        assert!(
            (share - MINIMUM_STATE_SHARE).abs() < f64::EPSILON,
            "a district charged more than its base cost keeps the 10% minimum, not a negative share"
        );
    }

    /// A tag that has moved must not resolve, because the tags move and the labels do not.
    #[test]
    fn a_column_is_found_by_its_label_and_not_its_tag() {
        let header: Vec<String> = ["[I] Base Funding Supplement", "[J] Total State Support"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        assert_eq!(column_any(&header, TOTAL_STATE_SUPPORT, "x"), Ok(1));
        assert_eq!(column_any(&header, BASE_FUNDING_SUPPLEMENT, "x"), Ok(0));
    }

    /// Two columns whose labels differ only in the tag are one column to a prefix match and two
    /// to this one.
    #[test]
    fn an_ambiguous_label_is_an_error_and_not_a_first_match() {
        let header: Vec<String> = ["[i] Base Cost Enrolled ADM", "[b] Base Cost Enrolled ADM"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        let found = column_any(&header, BASE_COST_ADM, "the sheet");
        assert!(
            found.is_err_and(|why| why.contains("not unique")),
            "the two Base Cost Enrolled ADM columns disagree on five districts; taking either \
             silently is the bug this guards"
        );
    }

    /// An absent optional column is absent; an ambiguous one is still an error.
    ///
    /// These are the two outcomes a single `Option` would fold together, and only one of them is
    /// expected: a year on the far side of the amendment simply lacks the column, while two
    /// columns claiming one label is the collision this module exists to catch.
    #[test]
    fn an_optional_column_distinguishes_absent_from_ambiguous() {
        let absent: Vec<String> = ["[A3] State Share Percentage"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert_eq!(
            optional_column(&absent, LOCAL_CAPACITY_PER_PUPIL, "a prior-law year"),
            Ok(None),
            "a prior-law report has no per-pupil local capacity, which is how its law is read"
        );

        let twice: Vec<String> = ["[A2] Local Capacity Per-Pupil", "Local Capacity Per-Pupil"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert!(
            optional_column(&twice, LOCAL_CAPACITY_PER_PUPIL, "a sheet")
                .is_err_and(|why| why.contains("not unique")),
            "two columns claiming the label is a collision, not an absence"
        );
    }

    /// Enrolment carried forward is equal to the last place, not merely close.
    #[test]
    fn carried_forward_enrolment_counts_exact_repeats() {
        let prior: BTreeMap<String, f64> = [
            ("a".to_string(), 1_089.766_767),
            ("b".to_string(), 629.224_732),
        ]
        .into_iter()
        .collect();
        let current: BTreeMap<String, f64> = [
            ("a".to_string(), 1_089.766_767),
            ("b".to_string(), 629.224_733),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            carried_forward_enrolment(&prior, &current),
            1,
            "a district recounted to the sixth place has been recounted"
        );
    }
}
