//! `[L1]`, split into the guarantee it contains and the funding it would contain without one.
//!
//! # The question, and why it needed two files
//!
//! `[K] Formula Transition Supplement` — H.B. 110 Section 265.225, extended to FY2026–27 by
//! H.B. 96 — holds a district at `[L1]`, a **FY2021** funding base, and is paid as
//! `max([L1] - (core foundation funding + [I] + transportation), 0)`. `[I]` is the temporary
//! transitional aid guarantee. So retiring the guarantee has three coherent readings, not two:
//! `[K]` stands as enacted, `[K]` is repealed alongside, or `[K]` is recomputed against a base
//! that never contained a guarantee either. The third could not be priced, because `[L1]` reaches
//! this corpus as a single published column and nothing said how much of it was guarantee.
//!
//! Two of the department's own files say. Neither needed credentials, and neither is the FY2021
//! payment report: **the FY2021 report carries no guarantee line at all**, so the era the issue
//! asked for could never have answered it. The answer is one year earlier.
//!
//! - `Foundation-Funding-Bases-6-16-2022.xlsx`, sheet `FY21 Funding Base`, decomposes `[L1]` into
//!   seven terms, and its first is `FY21 FOUNDATION FUNDING (pre-reduction)` — the workbook's own
//!   `Introduction` sheet names it `FY21 Final #2 Foundation Formula Funding Payment (line A)
//!   (plus the restoration of the executive budget reductions)`. The seven sum to `[L1]` to the
//!   cent on all 611 rows.
//! - `FY19_SFPR_FIN_2.xlsx` — the FY2019 final payment report, under the formula H.B. 110
//!   replaced — carries `TRANSITIONAL GUARANTEE` per district as one of the thirteen components
//!   of `TOTAL FOUNDATION FUNDING`.
//!
//! And that first term of `[L1]` **is** the FY2019 report's `TOTAL FOUNDATION FUNDING`, district
//! for district. It is a FY2021 figure that happens to equal a FY2019 one, and the department says
//! why: *"References to FY19 funding are used because FY20 state foundation funding for
//! traditional districts was flat to FY19 funding for the categories described below."* That note
//! is written about the FY2020 base; the identity checked here carries the freeze one year further
//! than the department's sentence does, which is why it is checked rather than assumed.
//!
//! That is the licence for the split. The guarantee is not a sibling of `[L1]` to be netted against
//! it, it is a term inside it, and the same `Introduction` sheet says so in as many words: *"any
//! eliminated funding elements from the previous formula (such as K-3 literacy) or elements not
//! specifically mentioned (such as **the guarantee** or EdChoice scholarship deductions) default to
//! the funding base used for line A Base Cost."* The base carries the guarantee and does not
//! itemise it. The FY2019 report says how large.
//!
//! # Why the two are read together
//!
//! The match is the whole claim, so it is checked here rather than asserted in prose. The base
//! sheet's own first term is verified against the other workbook, and a file that no longer
//! reproduces it is refused rather than extracted. The check found two things prose would not
//! have:
//!
//! - **The FY2019 report carries one district the base sheet does not.** Newbury Local SD
//!   (`047217`) dissolved into West Geauga Local SD, and the department carried its funding
//!   forward: West Geauga's `[L1]` first term exceeds its own FY2019 total by exactly Newbury's,
//!   $1,048,132.43. See `DISSOLVED`.
//! - **After that fold, the only districts left disagreeing are the two the department's own
//!   footnote names.** The FY2019 *Line by Line* explanation records exactly two exceptions to
//!   Section 265.220, and they are Hamilton City SD and Deer Park Community SD. See
//!   `FOOTNOTED`.
//!
//! # Transportation, and why `[L1]` is larger than `[H2]`
//!
//! `[L1]` includes transportation where `[H2]` — the FY2020 base the guarantee itself compares
//! against — does not, and the difference is a subtraction one definition makes and the other does
//! not. `[H2]` is built by taking a payment total and then going *"Less FY19 Final #2 Capped
//! Transportation Funding (line G) (Note: transportation funding is not subject to the phase-in
//! and is therefore removed from the base state funding calculation.)"*. `[L1]`'s seven terms
//! carry no such subtraction, so the transportation that was inside the FY2019 total stays inside
//! `[L1]`. The gap is not an accident of which components were capped; it is the phase-in's own
//! scope.
//!
//! # The header blocks straddle differently, so the rule is about banners
//!
//! Both sheets stack their labels over several rows, and no rule keyed on the `IRN` row reads
//! both: FY2019's block ends at its `IRN` row while the base sheet's runs one row past it —
//! `(pre-reduction)` and `(less preschool)` sit *below* the row carrying `IRN`. What separates a
//! label row from a banner is not position but **where a lone cell sits**, so `labels` takes
//! every row above the first district row except one whose only populated cell is in the key
//! column. That is where a title goes: it drops FY2019's `daria.shams:fy19_sfpr_fin_2.xlsx` and
//! the base sheet's `Used in FY22 and FY23`, both in column one.
//!
//! A width rule would read both sheets too, and would be wrong. The base sheet's community and
//! STEM deduction carries its `FY21` **alone, two rows above the rest of its label**, in no other
//! column — a single-cell row that is a label fragment and not a banner. Dropping it would leave
//! that one column of seven with no year in its name while its six siblings kept theirs, and the
//! extractor would stop asserting the vintage on the only term of `[L1]` whose year it could
//! otherwise confuse with the FY2020 base's.

use std::collections::{BTreeMap, BTreeSet};

use super::format::{clean_name, format_value};
use edfund_core::conventions::{cell, cell_number, is_district_key};

/// Columns of the FY2019 payment report fixture.
///
/// The thirteen components in the order the report prints them, then the total they sum to, then
/// the guarantee's own machinery. `funding_within_guarantee` is not a repeat of
/// `funding_before_guarantee`: the two differ by `career_technical` on every one of the 612 rows,
/// because the FY2019 guarantee does not protect career-technical funding.
pub const FY19_PAYMENT_REPORT_HEADER: &[&str] = &[
    "irn",
    "district",
    "county",
    "opportunity_grant",
    "targeted_assistance",
    "k3_literacy",
    "economic_disadvantaged",
    "limited_english_proficiency",
    "gifted",
    "transportation",
    "special_education_additional",
    "capacity_aid",
    "graduation_bonus",
    "third_grade_reading_bonus",
    "transitional_guarantee",
    "career_technical",
    "total_foundation_funding",
    "funding_before_guarantee",
    "funding_within_guarantee",
    "guarantee_base",
    "cap_ratio",
];

/// Columns of the FY2021 funding base fixture — the seven terms of `[L1]`, and `[L1]`.
pub const FY21_FUNDING_BASE_HEADER: &[&str] = &[
    "irn",
    "district",
    "county",
    "foundation_funding_pre_reduction",
    "net_open_enrollment",
    "net_excess_cost",
    "scholarship_deduction",
    "community_stem_deduction",
    "student_wellness_success",
    "enrollment_growth_supplement",
    "funding_base",
];

/// The only sheet in the FY2019 payment report workbook.
pub const FY19_SHEET: &str = "FY19_SFPR_FIN_2";

/// The sixth of the six sheets in the funding bases workbook, and the only one this reads.
///
/// The other five decompose FY2022 and FY2023 components, which the FY2026 and FY2027 calculators
/// already carry per district.
pub const FY21_BASE_SHEET: &str = "FY21 Funding Base";

/// The thirteen components that sum to `TOTAL FOUNDATION FUNDING`, in the order it prints them.
///
/// Matched on the whole joined label, exactly, and resolved **left of the total**. Both halves of
/// that matter. Nine of the thirteen are published twice, `CALCULATED` beside `CAPPED`, and the
/// pair differs in one word — a prefix or substring match takes whichever comes first and reports
/// a plausible number; the *capped* copy is the one that sums. And `GRADUATION BONUS` is published
/// twice under labels that are **identical**: once here as paid, and again ninety columns to the
/// right among the measures it was computed from. Nothing in the label tells them apart, so the
/// region does: a component is a column the total is the sum of, and the report prints those
/// before it.
const FY19_COMPONENTS: &[&str] = &[
    "CAPPED OPPORTUNITY GRANT",
    "CAPPED TARGETED ASSISTANCE",
    "CAPPED K-3 LITERACY FUNDING",
    "CAPPED ECONOMIC DISADVANTAGED FUNDING",
    "CAPPED LIMITED ENGLISH PROFICIENCY FUNDING",
    "CAPPED GIFTED EDUCATION FUNDING",
    "CAPPED TRANSPORTATION FUNDING",
    "CAPPED SPECIAL EDUCATION ADDITIONAL FUNDING",
    "CAPPED CAPACITY AID",
    "GRADUATION BONUS",
    "THIRD GRADE READING BONUS",
    "TRANSITIONAL GUARANTEE",
    "CAREER TECH EDUCATION FUNDING",
];

/// The guarantee and cap machinery, resolved right of the total the components sum to.
///
/// `FUNDING BEFORE GUARANTEE` and `FUNDING WITHIN GUARANTEE` are not a repeat of each other: they
/// differ by `CAREER TECH EDUCATION FUNDING` on every one of the 612 rows, because the FY2019
/// guarantee does not protect career-technical funding. `CAP RATIO` is what says the guarantee and
/// the cap never touch the same district.
const FY19_GUARANTEE_COLUMNS: &[&str] = &[
    "FUNDING BEFORE GUARANTEE",
    "FUNDING WITHIN GUARANTEE",
    "GUARANTEE BASE",
    "CAP RATIO",
];

/// The total the components sum to, and the boundary between the two lists above.
const FY19_TOTAL: &str = "TOTAL FOUNDATION FUNDING";

/// The FY2021 base labels taken, in fixture order after the three key columns.
///
/// `FY22 NET EXCESS COST` is the department's own spelling and is not a typo here: one of the
/// seven terms of a FY2021 base is a FY2022 quantity. Two others are marked `(pre-reduction)`,
/// which is what makes them terms of a base rather than of a payment.
const FY21_BASE_COLUMNS: &[&str] = &[
    "FY21 FOUNDATION FUNDING (pre-reduction)",
    "FY21 NET OPEN ENROLLMENT (less preschool)",
    "FY22 NET EXCESS COST",
    "FY21 TOTAL SCHOLARSHIP DEDUCTION",
    "FY21 COMMUNITY/STEM SCHOOL DEDUCTION (pre-reduction)",
    "FY21 STUDENT WELLNESS AND SUCCESS",
    "FY21 ENROLLMENT GROWTH SUPPLEMENT",
    "FY21 FUNDING BASE",
];

/// How many of [`FY21_BASE_COLUMNS`] sum to `FY21 FUNDING BASE`.
const FY21_BASE_TERMS: usize = 7;

/// A district the FY2019 report carries and the FY2021 base sheet does not, and its successor.
///
/// Newbury Local SD dissolved into West Geauga Local SD, and the department carried its FY2019
/// funding forward into West Geauga's `[L1]`. This is declared rather than derived, and both
/// directions are checked: the set of IRNs the FY2019 report has and the base sheet lacks must be
/// exactly this list, and the fold must be what resolves the successor's difference.
///
/// It matters for the census and not for the arithmetic. West Geauga's recomputed `[K]` is zero
/// either way, so the third reading of Section 265.225 does not move a cent on the fold — but the
/// statewide FY2019 guarantee does, by Newbury's own $469,077.69 over 334 districts against 333.
const DISSOLVED: &[(&str, &str)] = &[("047217", "047225")];

/// The two districts whose `[L1]` first term is not their FY2019 total, and the provision each is.
///
/// Both are named in the FY2019 *School Finance Payment Report Line by Line Explanation*, which
/// records exactly two exceptions to Section 265.220. Finding these two and only these two is the
/// check that the match is the department's rule and not a coincidence of scale.
const FOOTNOTED: &[(&str, &str)] = &[
    (
        "043851",
        "Deer Park Community SD, Sec. 18 of Am. Sub. S.B. 51 of the 132nd General Assembly: \
         the Taxation Department adjusted its TY16 certified values",
    ),
    (
        "044107",
        "Hamilton City SD, Section 265.227: career-technical funding shifted to Butler Tech \
         Joint Vocational School District on joining it",
    ),
];

/// Districts on the FY2021 base sheet, the department's own count.
const FY21_BASE_DISTRICTS: usize = 611;

/// Districts on the FY2019 report — the 611 plus Newbury.
const FY19_DISTRICTS: usize = 612;

/// What the base sheet carries where a district's name did not resolve.
///
/// One row has it — Middle Bass Local (`048959`), whose name and county are both `#N/A` and whose
/// seven terms are all zero. The lookup that failed was the department's; the district is real and
/// the FY2019 report names it, so [`build_fy21_funding_base`] takes the name from there rather
/// than committing a spreadsheet error as a district name.
const UNNAMED: &str = "#N/A";

/// A cent, as the tolerance for an identity over published dollars.
const CENT: f64 = 0.005;

/// The label block above a table, joined per column.
///
/// Every row above the data is part of the block except a banner, and a banner is a row whose only
/// populated cell sits in the key column — the place a title or a filename goes. That rule is what
/// reads both of these sheets, whose blocks sit differently relative to the `IRN` row. See this
/// module's comment.
fn labels(rows: &[Vec<String>], first_data_row: usize) -> Vec<String> {
    let block: Vec<&[String]> = rows[..first_data_row]
        .iter()
        .filter(|row| {
            let mut populated = row
                .iter()
                .enumerate()
                .filter(|(_, value)| !value.trim().is_empty());
            !matches!((populated.next(), populated.next()), (Some((0, _)), None))
        })
        .map(Vec::as_slice)
        .collect();
    let width = block.iter().map(|row| row.len()).max().unwrap_or(0);
    (0..width)
        .map(|at| {
            block
                .iter()
                .map(|row| cell(row, at).trim())
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

/// The first row whose key column holds a district IRN.
fn first_district(rows: &[Vec<String>], sheet: &str) -> Result<usize, String> {
    rows.iter()
        .position(|row| is_district_key(cell(row, 0)))
        .ok_or_else(|| format!("{sheet}: no row whose first column is a district IRN"))
}

/// The index of the column whose joined label is exactly `text`.
fn column_on(header: &[String], text: &str, sheet: &str) -> Result<usize, String> {
    column_in(header, text, sheet, 0..header.len())
}

/// The same, searching one region of the header rather than all of it.
///
/// A region is how a label published twice under the same spelling is resolved, and the FY2019
/// report has one: see [`FY19_COMPONENTS`]. It narrows the search and nothing else — a label that
/// is still ambiguous inside its region is still an error.
fn column_in(
    header: &[String],
    text: &str,
    sheet: &str,
    within: std::ops::Range<usize>,
) -> Result<usize, String> {
    let matches: Vec<usize> = header
        .iter()
        .enumerate()
        .filter(|(at, _)| within.contains(at))
        .filter(|(_, name)| name.trim().eq_ignore_ascii_case(text))
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

/// One publisher's table: the joined labels, and the district rows under them.
struct Table<'a> {
    header: Vec<String>,
    rows: Vec<&'a [String]>,
}

impl Table<'_> {
    /// Every district's value in one labelled column, keyed by IRN.
    ///
    /// Absent cells become `0.0` rather than `None`. Both of these sheets are dense grids of
    /// cached formula results in which a blank is a nil term, and a term that is not paid has to
    /// participate in a sum for the sum to be checkable.
    fn column(&self, text: &str, sheet: &str) -> Result<BTreeMap<String, f64>, String> {
        self.column_in(text, sheet, 0..self.header.len())
    }

    /// The same, resolving the label inside one region of the header. See [`column_in`].
    fn column_in(
        &self,
        text: &str,
        sheet: &str,
        within: std::ops::Range<usize>,
    ) -> Result<BTreeMap<String, f64>, String> {
        let at = column_in(&self.header, text, sheet, within)?;
        Ok(self
            .rows
            .iter()
            .map(|row| {
                (
                    cell(row, 0).trim().to_string(),
                    cell_number(row, at).unwrap_or(0.0),
                )
            })
            .collect())
    }
}

/// Read one of these two sheets: find the data, join the labels above it, keep the district rows.
fn table<'a>(rows: &'a [Vec<String>], sheet: &str, expected: usize) -> Result<Table<'a>, String> {
    let at = first_district(rows, sheet)?;
    let header = labels(rows, at);
    let district_rows: Vec<&[String]> = rows[at..]
        .iter()
        .map(Vec::as_slice)
        .filter(|row| is_district_key(cell(row, 0)))
        .collect();
    if district_rows.len() != expected {
        return Err(format!(
            "{sheet}: {} district rows, expected {expected}",
            district_rows.len()
        ));
    }
    Ok(Table {
        header,
        rows: district_rows,
    })
}

/// Take a column set in one pass, so a moved label fails before any arithmetic runs.
fn columns(
    source: &Table<'_>,
    names: &[&str],
    sheet: &str,
    within: std::ops::Range<usize>,
) -> Result<Vec<BTreeMap<String, f64>>, String> {
    names
        .iter()
        .map(|text| source.column_in(text, sheet, within.clone()))
        .collect()
}

/// Every district's name and county in one table, keyed by IRN.
fn names(source: &Table<'_>, sheet: &str) -> Result<BTreeMap<String, (String, String)>, String> {
    let name_at = column_on(&source.header, "DISTRICT", sheet)?;
    let county_at = column_on(&source.header, "COUNTY", sheet)?;
    Ok(source
        .rows
        .iter()
        .map(|row| {
            (
                cell(row, 0).trim().to_string(),
                (
                    clean_name(cell(row, name_at)),
                    clean_name(cell(row, county_at)),
                ),
            )
        })
        .collect())
}

/// Whether a set of terms sums to a total on every district, and the first district it does not.
fn sums_to(
    terms: &[BTreeMap<String, f64>],
    total: &BTreeMap<String, f64>,
) -> Option<(String, f64)> {
    total.iter().find_map(|(irn, expected)| {
        let sum: f64 = terms
            .iter()
            .map(|term| term.get(irn).copied().unwrap_or(0.0))
            .sum();
        ((sum - expected).abs() > CENT).then(|| (irn.clone(), sum - expected))
    })
}

/// The FY2019 final payment report, one row per district.
///
/// # Errors
///
/// If any of the labels this takes is missing or ambiguous within its region, if the sheet does
/// not carry `FY19_DISTRICTS` districts, if the thirteen components do not sum to
/// `TOTAL FOUNDATION FUNDING`, if `TOTAL FUNDING` is not that same total, if
/// `FUNDING BEFORE GUARANTEE` less `FUNDING WITHIN GUARANTEE` is not `CAREER TECH EDUCATION
/// FUNDING`, or if any district is both guaranteed and capped.
///
/// The last of those is the one that is not bookkeeping. The guarantee and the cap are the two
/// halves of the same provision and no district can be on both, so a file in which one is would
/// be a file whose two halves were computed against different populations.
pub fn build_fy19_payment_report(rows: &[Vec<String>]) -> Result<Vec<Vec<String>>, String> {
    let source = table(rows, FY19_SHEET, FY19_DISTRICTS)?;
    let name_at = column_on(&source.header, "DISTRICT", FY19_SHEET)?;
    let county_at = column_on(&source.header, "COUNTY", FY19_SHEET)?;

    // The total is the boundary between the two regions, so it is resolved first and the two
    // label sets are read on either side of it. See [`FY19_COMPONENTS`].
    let total_at = column_on(&source.header, FY19_TOTAL, FY19_SHEET)?;
    let components = columns(&source, FY19_COMPONENTS, FY19_SHEET, 0..total_at)?;
    let total = source.column_in(FY19_TOTAL, FY19_SHEET, total_at..total_at + 1)?;
    let guarantee = columns(
        &source,
        FY19_GUARANTEE_COLUMNS,
        FY19_SHEET,
        total_at + 1..source.header.len(),
    )?;

    if let Some((irn, by)) = sums_to(&components, &total) {
        return Err(format!(
            "{FY19_SHEET}: {irn}'s {} components miss its TOTAL FOUNDATION FUNDING by {by:.2}",
            FY19_COMPONENTS.len()
        ));
    }

    // `TOTAL FUNDING` sits 101 columns to the right of `TOTAL FOUNDATION FUNDING`, at the end of
    // the guarantee and cap machinery, and is identical to it on all 612 rows. Checking it is how
    // this knows the machinery it reads belongs to the total it reads.
    let paid = source.column("TOTAL FUNDING", FY19_SHEET)?;
    if let Some((irn, by)) = sums_to(std::slice::from_ref(&total), &paid) {
        return Err(format!(
            "{FY19_SHEET}: {irn}'s TOTAL FUNDING and TOTAL FOUNDATION FUNDING differ by {by:.2}"
        ));
    }

    let career_technical = &components[FY19_COMPONENTS.len() - 1];
    let held = &components[FY19_COMPONENTS.len() - 2];
    let [before, within, _, ratio] = guarantee.as_slice() else {
        return Err(format!(
            "{FY19_SHEET}: {} guarantee columns, expected four",
            guarantee.len()
        ));
    };

    for (irn, before) in before {
        let within = within.get(irn).copied().unwrap_or(0.0);
        let career = career_technical.get(irn).copied().unwrap_or(0.0);
        if (before - within - career).abs() > CENT {
            return Err(format!(
                "{FY19_SHEET}: {irn}'s funding before and within the guarantee differ by \
                 {:.2}, not by its career-technical funding of {career:.2}",
                before - within
            ));
        }
    }
    for (irn, held) in held {
        let ratio = ratio.get(irn).copied().unwrap_or(0.0);
        if *held > 0.0 && ratio < 1.0 {
            return Err(format!(
                "{FY19_SHEET}: {irn} is on the guarantee at {held:.2} and under the cap at \
                 {ratio}; no district can be on both"
            ));
        }
    }

    let taken: Vec<&BTreeMap<String, f64>> = components
        .iter()
        .chain(std::iter::once(&total))
        .chain(guarantee.iter())
        .collect();
    let mut out = Vec::new();
    for row in &source.rows {
        let irn = cell(row, 0).trim();
        let mut record = vec![
            irn.to_string(),
            clean_name(cell(row, name_at)),
            clean_name(cell(row, county_at)),
        ];
        for (at, column) in taken.iter().enumerate() {
            // Every column is dollars but the cap ratio, which is a multiplier and carries the
            // places the department publishes it to.
            let places = if at + 1 == taken.len() { 10 } else { 2 };
            record.push(format_value(column.get(irn).copied(), places));
        }
        out.push(record);
    }
    out.sort_by(|a, b| a[0].cmp(&b[0]));
    Ok(out)
}

/// The seven terms of `[L1]`, one row per district, checked against the FY2019 report.
///
/// # Errors
///
/// If either sheet's labels have moved, if the base sheet does not carry
/// `FY21_BASE_DISTRICTS` districts, if the seven terms do not sum to `FY21 FUNDING BASE`, if the
/// districts the FY2019 report carries and this sheet does not are not exactly `DISSOLVED`, if a
/// dissolved district's funding is not what its successor's first term carries over its own, or if
/// the districts whose first term is not their FY2019 total are not exactly `FOOTNOTED`.
///
/// The last three are the licence for splitting `[L1]`, and they are checked here because the
/// claim is about two files at once. A base sheet that still sums to itself while no longer
/// agreeing with the FY2019 report would read perfectly well and would have stopped meaning what
/// the third reading of Section 265.225 needs it to mean.
pub fn build_fy21_funding_base(
    base: &[Vec<String>],
    fy19: &[Vec<String>],
) -> Result<Vec<Vec<String>>, String> {
    let source = table(base, FY21_BASE_SHEET, FY21_BASE_DISTRICTS)?;
    let name_at = column_on(&source.header, "DISTRICT", FY21_BASE_SHEET)?;
    let county_at = column_on(&source.header, "COUNTY", FY21_BASE_SHEET)?;
    let taken = columns(
        &source,
        FY21_BASE_COLUMNS,
        FY21_BASE_SHEET,
        0..source.header.len(),
    )?;
    let total = &taken[FY21_BASE_TERMS];

    if let Some((irn, by)) = sums_to(&taken[..FY21_BASE_TERMS], total) {
        return Err(format!(
            "{FY21_BASE_SHEET}: {irn}'s {FY21_BASE_TERMS} terms miss its FY21 FUNDING BASE \
             by {by:.2}"
        ));
    }

    let report = table(fy19, FY19_SHEET, FY19_DISTRICTS)?;
    let fy19_total = report.column("TOTAL FOUNDATION FUNDING", FY19_SHEET)?;
    let first_term = &taken[0];
    reconcile_first_term(first_term, &fy19_total)?;
    let named = names(&report, FY19_SHEET)?;

    let mut out = Vec::new();
    for row in &source.rows {
        let irn = cell(row, 0).trim();
        let mut record = match (clean_name(cell(row, name_at)), named.get(irn)) {
            (name, Some((from_report, county))) if name == UNNAMED => {
                vec![irn.to_string(), from_report.clone(), county.clone()]
            }
            (name, _) => vec![irn.to_string(), name, clean_name(cell(row, county_at))],
        };
        for column in &taken {
            record.push(format_value(column.get(irn).copied(), 2));
        }
        out.push(record);
    }
    out.sort_by(|a, b| a[0].cmp(&b[0]));
    Ok(out)
}

/// Whether `[L1]`'s first term is still the FY2019 report's total, district for district.
///
/// Three claims, each failing with the district that broke it: the only IRNs the report has and
/// the base sheet lacks are [`DISSOLVED`]; each one's funding is exactly what its successor's
/// first term carries over its own; and after that fold the residual mismatches are exactly
/// [`FOOTNOTED`].
fn reconcile_first_term(
    first_term: &BTreeMap<String, f64>,
    fy19_total: &BTreeMap<String, f64>,
) -> Result<(), String> {
    let missing: BTreeSet<&str> = fy19_total
        .keys()
        .map(String::as_str)
        .filter(|irn| !first_term.contains_key(*irn))
        .collect();
    let declared: BTreeSet<&str> = DISSOLVED.iter().map(|(gone, _)| *gone).collect();
    if missing != declared {
        return Err(format!(
            "{FY19_SHEET}: carries {missing:?} that {FY21_BASE_SHEET} does not; \
             the declared dissolutions are {declared:?}"
        ));
    }

    let mut folded: BTreeMap<&str, f64> = fy19_total
        .iter()
        .map(|(irn, total)| (irn.as_str(), *total))
        .collect();
    for (gone, successor) in DISSOLVED {
        let carried = folded.remove(gone).unwrap_or(0.0);
        let into = folded
            .get_mut(successor)
            .ok_or_else(|| format!("{FY19_SHEET}: {gone}'s successor {successor} is not on it"))?;
        let expected = first_term.get(*successor).copied().unwrap_or(0.0);
        if (*into + carried - expected).abs() > CENT {
            return Err(format!(
                "{FY21_BASE_SHEET}: {successor}'s first term is {expected:.2}, which is not its \
                 own FY2019 total of {into:.2} plus dissolved {gone}'s {carried:.2}"
            ));
        }
        *into += carried;
    }

    let disagreeing: BTreeSet<&str> = first_term
        .iter()
        .filter(|(irn, term)| {
            (*term - folded.get(irn.as_str()).copied().unwrap_or(0.0)).abs() > CENT
        })
        .map(|(irn, _)| irn.as_str())
        .collect();
    let footnoted: BTreeSet<&str> = FOOTNOTED.iter().map(|(irn, _)| *irn).collect();
    if disagreeing != footnoted {
        return Err(format!(
            "{FY21_BASE_SHEET}: {disagreeing:?} do not carry their FY2019 total as the first \
             term of `[L1]`; the department's footnote names {footnoted:?}"
        ));
    }
    Ok(())
}
