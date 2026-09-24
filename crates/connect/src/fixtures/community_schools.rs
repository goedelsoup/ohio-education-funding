//! The department's community and STEM school simulator, per school, in the two years the corpus
//! holds one.
//!
//! The sibling of [`super::fy27`], published one directory across under
//! `Community-School-Funding/`, and the only per-*school* formula source this repository holds.
//! It exists here for one line the district model cannot carry at all: the **community school
//! equity supplement**, which H.B. 96 codified into R.C. 3317.022 and which no school district
//! receives.
//!
//! **Two vintages are read by the same code.** FY2027 from the file the department serves, and
//! FY2025 from the Internet Archive, the department having replaced it in place.
//! [`CommunitySchoolVintage`] is the whole of the difference between them, and it is small: four
//! column spellings, a letter in the designation column, and a supplement that did not exist yet.
//! Reading both matters because the rate is the thing that moved — $650, then $500, then $400 —
//! and a schedule with one year's model behind it is a schedule mostly taken on the act's word.
//!
//! # Who receives it, and why that is the fact worth extracting
//!
//! The supplement is a flat per-pupil payment, and the workbook writes the formula out on its
//! display sheet rather than leaving it to be inferred: `Detailed SFPR ` reads
//! *Equity Supplement [a*$400]* in FY2027 and *[a*$650]* in FY2025, where `a` is enrolled ADM.
//! Every site-based school's payment is that rate times its enrolled ADM, to the cent.
//!
//! **It is not paid to every school in the file, and the split is not by size or county.** Of the
//! 355 schools FY2027 models, 324 are site-based and receive it; **23 e-schools and 8 STEM schools
//! receive nothing**, and those 31 are exactly the schools whose payment is zero. FY2025 splits
//! 317, 18 and 8 the same way. Nothing about a row says so except the `E-School Designation`
//! column, so [`Designation`] is extracted beside the money rather than left to be rediscovered
//! from a zero.
//!
//! That split is worth a builder check rather than a note, because **the published analysis of
//! the introduced budget says the opposite**. The redbook records that the executive proposal
//! "expands eligibility to STEM schools", at $58.0 million a year. The enacted act did not keep
//! it: item 10 of the act's own list pays "community schools **that are not internet- or
//! computer-based**", the greenbook says "site-based community schools", and both workbooks pay
//! the STEM schools zero. Three enacted sources and two of the department's own models agree
//! against one document about a bill that changed. So [`build_community_school_funding`] fails if
//! a STEM or e-school row ever carries a payment — if the department extends it, that is a change
//! in the law this corpus should have to notice, not absorb.
//!
//! # Where the rate is read from, which is not the same cell in both years
//!
//! FY2027 states it twice: once as the bracketed formula above, and once as a value cell on
//! `Detail SFPR` labelled *Equity Suplement per pupil* — the department's misspelling — against
//! `400`. **FY2025 states it only once**, in the bracket. So the bracket is what is read, in both
//! years, and the value cell is a cross-check applied where it exists rather than a requirement.
//!
//! The alternative was to carry FY2025's rate as a constant because its value cell is missing.
//! That would have made the one number this fixture exists to verify the one number nothing
//! checks: the builder would have asserted $650 against payments computed from $650. Reading the
//! bracket keeps every year's rate a thing the department said, and the years that say it twice
//! are checked twice.
//!
//! # Which sheets, and the trap in the ones next to them
//!
//! The district workbook's convention is that an underscore names the per-district data sheet and
//! a space names a single-district display view — `Summary_SFPR` against `Summary SFPR`. **That
//! rule does not hold in this file.** Its per-school detail sheet is `Detail SFPR`, *with a
//! space*, and the display view beside it is `Detailed SFPR ` with a trailing one. Picking sheets
//! by the district rule gets an 88-row report laid out for a printer.
//!
//! And one sheet further on is worse than useless. `Summary_SFPR (2)` is a hidden leftover of the
//! FY2026 model — its phase-in cell reads `0.8333` where FY2027's reads `1` — carrying 358
//! rows against the live sheet's 356. Its column headed **`H. Equity Supplement` does not hold
//! the equity supplement**: on 352 of the 353 schools the two sheets share, that column holds
//! *total state support*, and on **none** of them does it hold the supplement. A reader that
//! found the column by its heading would report $1.5 billion as $34.7 million's quantity and
//! every row would look plausible. The sheet names below are constants for that reason, and the
//! arithmetic check is what would catch a reader pointed at the wrong one anyway.
//!
//! # What is not extracted
//!
//! The local share, because there is none: community schools have no taxing authority, so their
//! state share is 100% and the valuation and capacity columns the district model turns on do not
//! exist here. Nothing in this fixture joins the 609-district panel, and
//! `decisions/a-supplement-paid-to-a-population-the-panel-does-not-hold` records why that makes
//! it a reported figure rather than a ninth `Policy` lever.

use edfund_core::conventions::{cell, cell_number, is_statewide_row, number};

use super::delimited::column;
use super::format::{clean_name, format_value};

/// The per-school sheet carrying enrolled ADM and the supplement's inputs.
///
/// A space, not an underscore. See the module note: this file does not follow the district
/// workbook's naming rule, and `Detailed SFPR ` — with a trailing space — is the display view.
pub const CS_DETAIL_SHEET: &str = "Detail SFPR";

/// The per-school sheet carrying each funding line as paid.
///
/// The underscore is load-bearing here even though it is not on [`CS_DETAIL_SHEET`]: `Summary
/// SFPR` is a 37-row display view. So is `Summary SFPR (2)`, and `Summary_SFPR (2)` is the stale
/// FY2026 sheet whose supplement column holds something else.
pub const CS_SUMMARY_SHEET: &str = "Summary_SFPR";

/// The single-school printout, which is where the rate is stated in a form both years share.
///
/// A trailing space, and not the same sheet as [`CS_DETAIL_SHEET`]. It carries no per-school data
/// this fixture wants; it is read for one cell, the bracketed
/// `Equity Supplement [a*$N]` that writes the formula out.
pub const CS_DISPLAY_SHEET: &str = "Detailed SFPR ";

/// Which year's model is being read.
///
/// The department publishes one of these a year and replaces the previous one in place. Two are
/// held: FY2027 as served, FY2025 from the Internet Archive. They compute the same formula and
/// spell four of its columns differently, which is a difference small enough that a single
/// builder reads both and large enough that a builder assuming either year's spellings would
/// take the other year's file and fail at the header rather than the arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunitySchoolVintage {
    /// The year H.B. 33 set the rate at $650, before H.B. 96 cut it. Archived, not served.
    Fy2025,
    /// The year the corpus's headline figures come from. $400.
    Fy2027,
}

impl CommunitySchoolVintage {
    /// The fiscal year it models.
    #[must_use]
    pub const fn fiscal_year(self) -> u16 {
        match self {
            Self::Fy2025 => 2025,
            Self::Fy2027 => 2027,
        }
    }

    /// How an error names the file, so a failed rebuild says which year moved.
    #[must_use]
    pub const fn file(self) -> &'static str {
        match self {
            Self::Fy2025 => "the FY2025 community and STEM school simulator",
            Self::Fy2027 => "the FY2027 community and STEM school simulator",
        }
    }

    /// The summary sheet's money columns, in the order [`COMMUNITY_SCHOOL_FUNDING_HEADER`] names
    /// them.
    ///
    /// `None` is a line the year did not have, not a line this builder declines to read. The one
    /// instance is the base funding supplement, which H.B. 96 created: FY2025 has no such column
    /// and its fixture cell is left empty rather than written zero, because a school paid nothing
    /// and a payment that did not exist are different facts and only one of them is true here.
    #[must_use]
    pub const fn summary_money(self) -> [Option<&'static str>; 12] {
        match self {
            Self::Fy2025 => [
                Some("Ab. Base Cost (new)"),
                Some("Bb. Special Education"),
                Some("Cb. DPIA"),
                Some("Db. English Learner"),
                Some("Eb. Career Technical Education"),
                Some("F. Core Foundation Funding"),
                Some("G. Transportation Funding"),
                Some("H. Equity Supplement"),
                Some("I. Formula Transition Supplement"),
                None,
                Some("J. Facilities"),
                Some("K. Total State Supports"),
            ],
            Self::Fy2027 => [
                Some("A. Base Cost Calculated"),
                Some("B. Special Education Calculated"),
                Some("C. DPIA Calculated"),
                Some("D. English Learner Calculated"),
                Some("E. Career Technical Education Calculated"),
                Some("F. Core Foundation Funding"),
                Some("G. Transportation Funding"),
                Some("H. Equity Supplement"),
                Some("I. Formula Transition Supplement"),
                Some("J. Base Funding Supplement"),
                Some("K. Facilities"),
                Some("L. Total State Supports"),
            ],
        }
    }
}

/// Columns of the community and STEM school extract.
///
/// `enrolled_adm` comes from [`CS_DETAIL_SHEET`] and the money from [`CS_SUMMARY_SHEET`]; the
/// rest of the detail sheet's columns are the intermediate terms of lines this one carries as
/// paid, and restating them would be a second copy of the same arithmetic.
pub const COMMUNITY_SCHOOL_FUNDING_HEADER: &[&str] = &[
    "irn",
    "school",
    "county",
    "designation",
    "enrolled_adm",
    "base_cost",
    "special_education",
    "dpia",
    "english_learners",
    "career_technical",
    "core_foundation_funding",
    "transportation",
    "equity_supplement",
    "formula_transition_supplement",
    "base_funding_supplement",
    "facilities",
    "total_state_support",
];

/// What kind of school a row is, which decides whether it is paid the equity supplement.
///
/// The department writes this as an `E-School Designation` of `N`, `Y` or `STEM`, which is a
/// two-question column answered in one: `Y` and `STEM` are both "not a site-based community
/// school" and they are not the same thing — a STEM school is a separate creature of R.C. 3326
/// that this formula happens to fund alongside community schools.
///
/// FY2025 writes the third value `S` rather than `STEM`. Both are matched exactly, as two
/// spellings of one value and not as a prefix or a fallback: a column with four legal strings in
/// it should reject the fifth, and a `_ =>` arm that swallowed an unknown letter would be how a
/// designation the department invents later becomes a site-based school by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Designation {
    /// A site-based community school. The 324 that receive the supplement.
    SiteBased,
    /// An internet- or computer-based community school. Paid no supplement.
    ESchool,
    /// A STEM school under R.C. 3326. Paid no supplement, against what the redbook said the
    /// introduced budget would do.
    Stem,
}

impl Designation {
    /// Read the department's designation cell.
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "N" => Some(Self::SiteBased),
            "Y" => Some(Self::ESchool),
            // Two spellings of one value: FY2027 writes `STEM` and FY2025 writes `S`.
            "STEM" | "S" => Some(Self::Stem),
            _ => None,
        }
    }

    /// How the fixture spells it.
    const fn as_str(self) -> &'static str {
        match self {
            Self::SiteBased => "site-based",
            Self::ESchool => "e-school",
            Self::Stem => "stem",
        }
    }

    /// Whether R.C. 3317.022 pays this kind of school the equity supplement.
    #[must_use]
    pub const fn receives_equity_supplement(self) -> bool {
        matches!(self, Self::SiteBased)
    }
}

/// The department's bracketed statement of the formula, on [`CS_DISPLAY_SHEET`].
///
/// `Equity Supplement [a*$400]` in FY2027 and `Equity Supplement [a*$650]` in FY2025, where `a`
/// is enrolled ADM. This is the only statement of the rate the two vintages share, so it is the
/// one the rate is read from.
const RATE_BRACKET: &str = "Equity Supplement [a*$";

/// The department's own label on the value cell holding the per-pupil rate, in the years it has
/// one.
///
/// Its misspelling is the department's and is matched as published. Correcting it here would
/// mean the reader stops finding the cell the day they correct it themselves, which is the
/// wrong way round: a changed label should be a named failure.
///
/// FY2027 carries it on [`CS_DETAIL_SHEET`]; FY2025 carries no such cell. So it is a cross-check
/// on [`RATE_BRACKET`] where it exists rather than the source of the rate.
const RATE_LABEL: &str = "Equity Suplement per pupil";

/// The first heading of both data sheets, which is how their header row is found.
///
/// Found rather than counted. The reader strips a sheet's empty rows, so the header sits at index
/// 2 of `Detail SFPR` — under two rows of rate labels — and at index 0 of `Summary_SFPR`, whose
/// three leading blanks disappear. Neither number is a fact about the workbook, and hard-coding
/// either would break on a file the department spaced differently while looking like a layout
/// change here.
const FIRST_HEADING: &str = "CS IRN";

/// How many schools the file should carry. A floor, not an equality: schools close.
const AT_LEAST: usize = 300;

/// A cent. Every equality below is checked to it — the workbook stores cached formula results, so
/// the terms are exact to well inside this, and a rate or population change is not close.
const CENT: f64 = 0.01;

/// Build the per-school extract from the detail and summary sheets.
///
/// # Errors
///
/// Returns the heading or label that has moved, a designation outside the department's three, a
/// site-based school whose supplement is not its enrolled ADM times the published rate, an
/// e-school or STEM school that was paid one at all, a school on one sheet and not the other, a
/// statewide row that does not equal the sum of its schools, or a file that has lost its schools.
/// Every one of those would otherwise produce a fixture that reads perfectly well and is wrong.
pub fn build_community_school_funding(
    vintage: CommunitySchoolVintage,
    display: &[Vec<String>],
    detail: &[Vec<String>],
    summary: &[Vec<String>],
) -> Result<Vec<Vec<String>>, String> {
    let file = vintage.file();

    let detail_header = header_row(detail, CS_DETAIL_SHEET, file)?;
    let summary_header = header_row(summary, CS_SUMMARY_SHEET, file)?;
    let rate = published_rate(display, &detail[..detail_header], file)?;

    let detail_head = detail[detail_header].clone();
    let summary_head = summary[summary_header].clone();
    let at_detail = |name: &str| column(&detail_head, name, file);
    let at_summary = |name: &str| column(&summary_head, name, file);

    let detail_columns = DetailColumns {
        irn: at_detail("CS IRN")?,
        name: at_detail("Community School Names")?,
        designation: at_detail("E-School Designation")?,
        adm: at_detail("a. Enrolled ADM")?,
        equity: at_detail("H. Equity Supplement")?,
    };
    let (s_irn, s_name, s_county, s_designation) = (
        at_summary("CS IRN")?,
        at_summary("Community School Names")?,
        at_summary("County Names")?,
        at_summary("E-School Designation")?,
    );
    // In the order `COMMUNITY_SCHOOL_FUNDING_HEADER` names them, after the four identifying
    // columns and the ADM that comes off the other sheet. A `None` is a line the year did not
    // have; its cell is written empty, not zero.
    let money: Vec<Option<usize>> = vintage
        .summary_money()
        .iter()
        .map(|name| name.map(&at_summary).transpose())
        .collect::<Result<_, _>>()?;
    let s_equity = at_summary("H. Equity Supplement")?;

    let adm = enrolled_adm(&detail[detail_header + 1..], &detail_columns, file)?;

    let mut out = Vec::new();
    let mut statewide: Option<f64> = None;
    let mut paid = 0.0_f64;

    for row in summary.iter().skip(summary_header + 1) {
        let irn = cell(row, s_irn).trim();
        if irn.is_empty() {
            continue;
        }
        let equity = cell_number(row, s_equity).unwrap_or(0.0);
        if is_statewide_row(row, s_name) {
            statewide = Some(equity);
            continue;
        }

        let Some(designation) = Designation::parse(cell(row, s_designation)) else {
            return Err(format!(
                "{file} gives school {irn} the designation {:?}, which is not N, Y, S or STEM",
                cell(row, s_designation)
            ));
        };
        let Some(&Detailed {
            designation: detail_designation,
            enrolled,
            equity: detail_equity,
        }) = adm.get(irn)
        else {
            return Err(format!(
                "{file} has school {irn} on {CS_SUMMARY_SHEET} and not on {CS_DETAIL_SHEET}; the \
                 two sheets are not the same population"
            ));
        };
        if detail_designation != designation {
            return Err(format!(
                "{file} designates school {irn} {} on {CS_DETAIL_SHEET} and {} on \
                 {CS_SUMMARY_SHEET}",
                detail_designation.as_str(),
                designation.as_str()
            ));
        }

        // The two sheets compute the supplement separately. Neither is taken on trust: a
        // disagreement between them is the department's file being inconsistent with itself,
        // which is a different fault from the arithmetic check below and is not implied by it.
        if (equity - detail_equity).abs() > CENT {
            return Err(format!(
                "{file} pays school {irn} an equity supplement of {equity:.2} on \
                 {CS_SUMMARY_SHEET} and {detail_equity:.2} on {CS_DETAIL_SHEET}"
            ));
        }
        check_payment(irn, designation, enrolled, equity, rate, file)?;
        paid += equity;

        let mut record = vec![
            irn.to_string(),
            clean_name(cell(row, s_name)),
            clean_name(cell(row, s_county)),
            designation.as_str().to_string(),
            format_value(Some(enrolled), 6),
        ];
        for index in &money {
            record.push(match index {
                Some(index) => format_value(cell_number(row, *index), 2),
                None => String::new(),
            });
        }
        out.push(record);
    }

    if out.len() < AT_LEAST {
        return Err(format!(
            "{file} yielded {} schools, fewer than the {AT_LEAST} it should carry",
            out.len()
        ));
    }
    let Some(statewide) = statewide else {
        return Err(format!(
            "{file} has no {} row on {CS_SUMMARY_SHEET}, so the per-school rows are checked \
             against nothing",
            edfund_core::conventions::STATEWIDE_ROW
        ));
    };
    // The department's own total, against the rows this fixture keeps. A dropped school or a
    // double-counted one is invisible in a per-school file and obvious here — which is the
    // failure the district calculator's `State of Ohio` row produced once, at exactly twice size.
    if (statewide - paid).abs() > CENT {
        return Err(format!(
            "{file} publishes a statewide equity supplement of {statewide:.2} and its schools sum \
             to {paid:.2}; a row is missing or counted twice"
        ));
    }
    Ok(out)
}

/// The per-pupil rate, read from the formula the department writes out.
///
/// Read rather than assumed. The rate is the one thing about this supplement that H.B. 96 changed
/// twice in two years — $650 to $500 to $400 — so a builder carrying it as a constant would
/// check one year's payments against another year's rate and pass.
///
/// `RATE_BRACKET` is the source because it is the statement both vintages make. `RATE_LABEL` is
/// a value cell FY2027 also carries and FY2025 does not, so where it exists the two must agree
/// and where it does not the bracket stands alone. The department disagreeing with itself about
/// its own rate is a fault worth a name, and a year that states the rate once is not a year that
/// states it wrongly.
fn published_rate(
    display: &[Vec<String>],
    above_header: &[Vec<String>],
    file: &str,
) -> Result<f64, String> {
    let bracketed = bracketed_rate(display, file)?;
    if let Some(stated) = stated_rate(above_header) {
        if (stated - bracketed).abs() > CENT {
            return Err(format!(
                "{file} states a per-pupil rate of {stated} beside {RATE_LABEL:?} on \
                 {CS_DETAIL_SHEET} and {bracketed} in its own formula on {CS_DISPLAY_SHEET}"
            ));
        }
    }
    Ok(bracketed)
}

/// The rate out of `Equity Supplement [a*$400]`, which is how both vintages write the formula.
fn bracketed_rate(display: &[Vec<String>], file: &str) -> Result<f64, String> {
    display
        .iter()
        .flatten()
        .find_map(|cell| {
            let cell = cell.trim();
            cell.strip_prefix(RATE_BRACKET)
                .and_then(|rest| rest.strip_suffix(']'))
                .and_then(number)
        })
        .filter(|rate| *rate > 0.0)
        .ok_or_else(|| {
            format!(
                "{file} has no {RATE_BRACKET:?}…] cell on {CS_DISPLAY_SHEET}; the rate is stated \
                 nowhere this reader can check the payments against"
            )
        })
}

/// The rate out of the value cell FY2027 carries and FY2025 does not.
fn stated_rate(above_header: &[Vec<String>]) -> Option<f64> {
    let (row, at) = above_header.iter().find_map(|row| {
        row.iter()
            .position(|cell| cell.trim() == RATE_LABEL)
            .map(|at| (row, at))
    })?;
    // The label and its value are not adjacent — the department leaves a merged gap — so the rate
    // is the first number to its right rather than the next cell.
    row.iter()
        .skip(at + 1)
        .find_map(|cell| number(cell))
        .filter(|rate| *rate > 0.0)
}

/// The index of a sheet's header row, found by its first heading.
fn header_row(rows: &[Vec<String>], sheet: &str, file: &str) -> Result<usize, String> {
    rows.iter()
        .position(|row| cell(row, 0).trim() == FIRST_HEADING)
        .ok_or_else(|| {
            format!("{file} has no {FIRST_HEADING:?} heading on {sheet}; its layout has moved")
        })
}

/// The columns [`enrolled_adm`] reads off [`CS_DETAIL_SHEET`].
struct DetailColumns {
    irn: usize,
    name: usize,
    designation: usize,
    adm: usize,
    equity: usize,
}

/// One school as the detail sheet states it.
#[derive(Debug, Clone, Copy)]
struct Detailed {
    designation: Designation,
    enrolled: f64,
    equity: f64,
}

/// Enrolled ADM, designation and the detail sheet's own copy of the supplement, per school.
///
/// The supplement is carried out of here rather than recomputed, because the two sheets are
/// separate computations in the published file and the caller checks them against each other. A
/// reader that took the summary sheet alone would have one witness to the number this whole
/// fixture exists to carry.
fn enrolled_adm(
    detail: &[Vec<String>],
    columns: &DetailColumns,
    file: &str,
) -> Result<std::collections::BTreeMap<String, Detailed>, String> {
    let mut out = std::collections::BTreeMap::new();
    for row in detail.iter() {
        let irn = cell(row, columns.irn).trim();
        if irn.is_empty() || is_statewide_row(row, columns.name) {
            continue;
        }
        let Some(designation) = Designation::parse(cell(row, columns.designation)) else {
            return Err(format!(
                "{file} gives school {irn} the designation {:?}, which is not N, Y, S or STEM",
                cell(row, columns.designation)
            ));
        };
        let enrolled = cell_number(row, columns.adm).ok_or_else(|| {
            format!("{file} states no enrolled ADM for school {irn} on {CS_DETAIL_SHEET}")
        })?;
        let entry = Detailed {
            designation,
            enrolled,
            equity: cell_number(row, columns.equity).unwrap_or(0.0),
        };
        if let Some(previous) = out.insert(irn.to_string(), entry) {
            return Err(format!(
                "{file} lists school {irn} twice on {CS_DETAIL_SHEET}, at {} and {enrolled} \
                 enrolled ADM",
                previous.enrolled
            ));
        }
    }
    if out.is_empty() {
        return Err(format!("{file} yielded no schools from {CS_DETAIL_SHEET}"));
    }
    Ok(out)
}

/// Whether a school's supplement is what its designation and enrolled ADM say it should be.
///
/// Both directions are errors, and the second is the one that matters. A site-based school paid
/// something other than rate times ADM means the formula moved; an e-school or STEM school paid
/// anything at all means eligibility moved, which is the change the redbook describes and the
/// enacted act did not make.
fn check_payment(
    irn: &str,
    designation: Designation,
    enrolled: f64,
    equity: f64,
    rate: f64,
    file: &str,
) -> Result<(), String> {
    if designation.receives_equity_supplement() {
        let expected = enrolled * rate;
        if (equity - expected).abs() > CENT {
            return Err(format!(
                "{file} pays site-based school {irn} an equity supplement of {equity:.2} against \
                 {enrolled} enrolled ADM at the {rate} it publishes, which is {expected:.2}"
            ));
        }
    } else if equity.abs() > CENT {
        return Err(format!(
            "{file} pays {} school {irn} an equity supplement of {equity:.2}. R.C. 3317.022 as \
             H.B. 96 enacted it pays site-based community schools only — if that has changed, the \
             corpus records the old rule and this is where it finds out",
            designation.as_str()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The FY2027 detail sheet's headings, in the department's order and spelling.
    const DETAIL_HEAD: &[&str] = &[
        "CS IRN",
        "Community School Names",
        "County Names",
        "E-School Designation",
        "A. Aggregate Base Cost ",
        "B. Special Education Funding",
        "C. Disadvantaged Pupil Impact Aid ",
        "D. English Learners ",
        "E. Career-Tech Education ",
        "F1. Core Foundation Funding ",
        "F2. FY20 Funding Base",
        "F. Core Foundation Funding ",
        "G. Transportation Funding",
        "H. Equity Supplement",
        "I1a. FY21 Funding Base",
        "I1b FY21 Student FTE",
        "I1. FY21 Per Pupil",
        "a. Enrolled ADM",
        "I2. FY25 Per Pupil",
        "I. Formula Transition Supplement ",
        "J. Base Funding Supplement [a * $40]",
        "K. Facilities",
        "L. Total State Supports",
    ];

    /// The FY2027 summary sheet's headings.
    const SUMMARY_HEAD: &[&str] = &[
        "CS IRN",
        "Community School Names",
        "County Names",
        "E-School Designation",
        "A. Base Cost Calculated",
        "B. Special Education Calculated",
        "C. DPIA Calculated",
        "D. English Learner Calculated",
        "E. Career Technical Education Calculated",
        "F. Core Foundation Funding",
        "G. Transportation Funding",
        "H. Equity Supplement",
        "I. Formula Transition Supplement ",
        "J. Base Funding Supplement",
        "K. Facilities",
        "L. Total State Supports",
        "M. Quality Community and STEM School Support",
        "K. Educational Service Center\t",
        "L. Other Adjustments",
        "M. Total Transfers [K + L]",
        "N. Net State Funding [J + M]",
        "Base Cost SWS",
    ];

    /// The FY2025 detail sheet's headings. Four columns fewer and the equity supplement at the
    /// far end rather than in the middle, which is why the builder finds columns by name.
    const FY25_DETAIL_HEAD: &[&str] = &[
        "CS IRN",
        "Community School Names",
        "County Names",
        "E-School Designation",
        "A. Aggregate Base Cost ",
        "B. Special Education Funding",
        "C. Disadvantaged Pupil Impact Aid ",
        "D. English Learners ",
        "E. Career-Tech Education ",
        "F1. Core Foundation Funding ",
        "F2. FY20 Funding Base",
        "F. Core Foundation Funding ",
        "G. Transportation Funding",
        "H1a. FY21 Funding Base",
        "H1b FY21 Student FTE",
        "H1. FY21 Per Pupil",
        "a. Enrolled ADM",
        "H2. FY24 Per Pupil",
        "I. Formula Transition Supplement ",
        "J. Facilities",
        "J. Total State Supports",
        "",
        "H. Equity Supplement",
        "K Total State Support [F+G+H+I+J]",
    ];

    /// The FY2025 summary sheet's headings, as far as the builder reads.
    const FY25_SUMMARY_HEAD: &[&str] = &[
        "CS IRN",
        "Community School Names",
        "County Names",
        "E-School Designation",
        "Aa. Base Cost",
        "Ba. Special Education",
        "Ca. DPIA",
        "Da. English Learner",
        "Ea. Career Technical Education",
        "Ab. Base Cost (new)",
        "Bb. Special Education",
        "Cb. DPIA",
        "Db. English Learner",
        "Eb. Career Technical Education",
        "C. Phase in Funding \"Base Cost\"",
        "C. Phase in Funding \"Special ED\"",
        "C. Phase in Funding \"DPIA\"",
        "C. Phase in Funding \"English Learner\"",
        "C. Phase in Funding \"CTE\"",
        "Fa. Total Base State Funding",
        "Fb. Total New Funding",
        "Fc. Total Phase in ",
        "F. Core Foundation Funding",
        "G. Transportation Funding",
        "H. Equity Supplement",
        "I. Formula Transition Supplement ",
        "J. Facilities",
        "K. Total State Supports",
    ];

    /// Column indices this module's tests reach for, on each sheet.
    const D_DESIGNATION: usize = 3;
    const D_EQUITY: usize = 13;
    const D_ADM: usize = 17;
    const S_DESIGNATION: usize = 3;
    const S_EQUITY: usize = 11;

    /// And the same three on the FY2025 sheets, which put them elsewhere.
    const FY25_D_EQUITY: usize = 22;
    const FY25_D_ADM: usize = 16;
    const FY25_S_EQUITY: usize = 24;

    /// The rate each workbook publishes, and the ADM every generated school carries.
    const RATE: f64 = 400.0;
    const FY25_RATE: f64 = 650.0;
    const ADM: f64 = 100.0;

    fn row(cells: &[&str], width: usize) -> Vec<String> {
        let mut out: Vec<String> = cells.iter().map(|c| (*c).to_string()).collect();
        out.resize(width, String::new());
        out
    }

    fn heading(head: &[&str]) -> Vec<String> {
        head.iter().map(|h| (*h).to_string()).collect()
    }

    /// The display sheet, which in both years is read for one cell.
    fn display(rate: f64) -> Vec<Vec<String>> {
        vec![row(
            &["H", &format!("Equity Supplement [a*${rate:.0}]")],
            12,
        )]
    }

    /// Where each vintage's generated sheets put the columns the tests edit.
    struct Layout {
        vintage: CommunitySchoolVintage,
        rate: f64,
        detail_head: &'static [&'static str],
        summary_head: &'static [&'static str],
        d_equity: usize,
        d_adm: usize,
        s_equity: usize,
        /// How many rows of rate labels sit above the detail sheet's heading.
        above: usize,
    }

    const FY2027: Layout = Layout {
        vintage: CommunitySchoolVintage::Fy2027,
        rate: RATE,
        detail_head: DETAIL_HEAD,
        summary_head: SUMMARY_HEAD,
        d_equity: D_EQUITY,
        d_adm: D_ADM,
        s_equity: S_EQUITY,
        above: 2,
    };

    const FY2025: Layout = Layout {
        vintage: CommunitySchoolVintage::Fy2025,
        rate: FY25_RATE,
        detail_head: FY25_DETAIL_HEAD,
        summary_head: FY25_SUMMARY_HEAD,
        d_equity: FY25_D_EQUITY,
        d_adm: FY25_D_ADM,
        s_equity: FY25_S_EQUITY,
        // FY2025 states the rate only in the bracket, so there is one row above its heading and
        // it carries no rate cell at all.
        above: 1,
    };

    /// A workbook of `count` schools, every one site-based and correctly paid.
    ///
    /// The detail sheet carries its rate rows above its heading and the summary sheet carries
    /// none, which is the shape the reader actually sees: empty rows are stripped, so the two
    /// sheets' headings land at different indices.
    fn sheets(layout: &Layout, count: usize) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
        let width = layout.detail_head.len();
        let mut detail = vec![row(
            &[
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "General Phase in",
                "1",
            ],
            width,
        )];
        if layout.above > 1 {
            detail.push(row(
                &["", "", "", "", "", "", "", "", "", RATE_LABEL, "", "400"],
                width,
            ));
        }
        detail.push(heading(layout.detail_head));

        let mut summary = vec![heading(layout.summary_head)];
        let equity = format!("{:.6}", ADM * layout.rate);

        for i in 0..count {
            let irn = format!("{:06}", i + 1);
            let mut d = row(&[&irn, "Somewhere Academy", "Adams", "N"], width);
            d[layout.d_adm] = ADM.to_string();
            d[layout.d_equity].clone_from(&equity);
            detail.push(d);

            let mut s = row(
                &[&irn, "Somewhere Academy", "Adams", "N"],
                layout.summary_head.len(),
            );
            s[layout.s_equity].clone_from(&equity);
            summary.push(s);
        }

        let mut statewide = row(
            &["050765", "State of Ohio", "***", ""],
            layout.summary_head.len(),
        );
        #[allow(clippy::cast_precision_loss)]
        let total = count as f64 * ADM * layout.rate;
        statewide[layout.s_equity] = format!("{total:.6}");
        summary.push(statewide);
        (detail, summary)
    }

    /// Build a generated workbook the way the rebuild does.
    fn build(
        layout: &Layout,
        detail: &[Vec<String>],
        summary: &[Vec<String>],
    ) -> Result<Vec<Vec<String>>, String> {
        build_community_school_funding(layout.vintage, &display(layout.rate), detail, summary)
    }

    /// Rewrite one school on both sheets, so a test changes a school rather than a cell.
    fn school(
        layout: &Layout,
        detail: &mut [Vec<String>],
        summary: &mut [Vec<String>],
        designation: &str,
        equity: Option<f64>,
    ) {
        let first = layout.above + 1;
        detail[first][D_DESIGNATION] = designation.to_string();
        summary[1][S_DESIGNATION] = designation.to_string();
        if let Some(equity) = equity {
            detail[first][layout.d_equity] = format!("{equity:.6}");
            summary[1][layout.s_equity] = format!("{equity:.6}");
        }
    }

    #[test]
    fn a_heading_that_moved_is_an_error_rather_than_a_misread() {
        let (detail, mut summary) = sheets(&FY2027, AT_LEAST);
        summary[0][S_EQUITY] = "Equity".to_string();
        let error = build(&FY2027, &detail, &summary).unwrap_err();
        assert!(
            error.contains("H. Equity Supplement"),
            "the error names the column that moved: {error}"
        );
    }

    #[test]
    fn the_header_row_is_found_rather_than_counted() {
        // The two sheets' headings sit at different indices because empty rows are stripped, and
        // the builder reads both. A reader that counted would have to be right twice — and four
        // times across the two vintages, which put their headings in different places again.
        let (detail, summary) = sheets(&FY2027, AT_LEAST);
        assert_eq!(header_row(&detail, CS_DETAIL_SHEET, "f").unwrap(), 2);
        assert_eq!(header_row(&summary, CS_SUMMARY_SHEET, "f").unwrap(), 0);
        assert!(build(&FY2027, &detail, &summary).is_ok());
    }

    #[test]
    fn the_rate_is_read_from_the_workbook_rather_than_assumed() {
        // H.B. 96 moved this rate twice in two years. A builder holding it as a constant would
        // check a later year's payments against FY2027's and pass.
        let (mut detail, summary) = sheets(&FY2027, AT_LEAST);
        detail[1][11] = "500".to_string();
        let error = build_community_school_funding(
            CommunitySchoolVintage::Fy2027,
            &display(500.0),
            &detail,
            &summary,
        )
        .unwrap_err();
        assert!(error.contains("50000.00"), "{error}");
    }

    #[test]
    fn the_rate_comes_from_the_bracket_and_the_value_cell_has_to_agree_with_it() {
        // FY2027 states the rate twice and FY2025 once. The bracket is the statement both make,
        // so it is the one read; the value cell is a cross-check where it exists. A file that
        // disagreed with itself would otherwise be extracted against whichever cell was picked.
        let (detail, _) = sheets(&FY2027, AT_LEAST);
        assert_eq!(
            published_rate(&display(RATE), &detail[..2], "f").unwrap(),
            RATE
        );

        let mut disagreeing = detail.clone();
        disagreeing[1][11] = "650".to_string();
        let error = published_rate(&display(RATE), &disagreeing[..2], "f").unwrap_err();
        assert!(error.contains("650") && error.contains("400"), "{error}");

        // And a year with no value cell at all is read from the bracket alone rather than
        // refused — which is the whole of why FY2025 is extractable.
        let (fy25_detail, _) = sheets(&FY2025, AT_LEAST);
        assert_eq!(
            published_rate(&display(FY25_RATE), &fy25_detail[..1], "f").unwrap(),
            FY25_RATE
        );
    }

    #[test]
    fn a_workbook_that_states_its_rate_nowhere_is_refused() {
        let (detail, summary) = sheets(&FY2027, AT_LEAST);
        let error =
            build_community_school_funding(CommunitySchoolVintage::Fy2027, &[], &detail, &summary)
                .unwrap_err();
        assert!(error.contains("stated"), "{error}");
    }

    #[test]
    fn a_designation_outside_the_departments_four_spellings_is_refused() {
        let (mut detail, mut summary) = sheets(&FY2027, AT_LEAST);
        school(&FY2027, &mut detail, &mut summary, "DIGITAL", None);
        let error = build(&FY2027, &detail, &summary).unwrap_err();
        assert!(error.contains("not N, Y, S or STEM"), "{error}");
    }

    #[test]
    fn the_two_spellings_of_stem_are_matched_exactly_and_nothing_else_is() {
        // FY2025 writes `S` where FY2027 writes `STEM`. Both are the same value and neither is a
        // prefix rule: `STEM` is not `S` plus anything, and a designation the department invents
        // later must not arrive as a site-based school.
        assert_eq!(Designation::parse("STEM"), Some(Designation::Stem));
        assert_eq!(Designation::parse("S"), Some(Designation::Stem));
        assert_eq!(Designation::parse(" S "), Some(Designation::Stem));
        assert_eq!(Designation::parse("SBASED"), None);
        assert_eq!(Designation::parse("s"), None, "the column is upper case");
        assert_eq!(Designation::parse(""), None);
    }

    #[test]
    fn a_site_based_school_paid_something_other_than_rate_times_adm_is_refused() {
        let (mut detail, mut summary) = sheets(&FY2027, AT_LEAST);
        school(
            &FY2027,
            &mut detail,
            &mut summary,
            "N",
            Some(ADM * RATE + 1.0),
        );
        let error = build(&FY2027, &detail, &summary).unwrap_err();
        assert!(error.contains("40001.00"), "{error}");
        assert!(error.contains("000001"), "and it names the school: {error}");
    }

    #[test]
    fn a_stem_school_paid_the_supplement_at_all_is_refused_in_either_spelling() {
        // The check this file exists for. The redbook says the *introduced* budget expanded
        // eligibility to STEM schools; the enacted act did not, and the department's models pay
        // them zero in both years. If that ever changes it is a change in the law, and the corpus
        // should find out here rather than restate a rule it has recorded as settled.
        let (mut detail, mut summary) = sheets(&FY2027, AT_LEAST);
        school(&FY2027, &mut detail, &mut summary, "STEM", Some(ADM * RATE));
        let error = build(&FY2027, &detail, &summary).unwrap_err();
        assert!(error.contains("pays stem school 000001"), "{error}");
        assert!(error.contains("R.C. 3317.022"), "{error}");

        let (mut detail, mut summary) = sheets(&FY2025, AT_LEAST);
        school(
            &FY2025,
            &mut detail,
            &mut summary,
            "S",
            Some(ADM * FY25_RATE),
        );
        let error = build(&FY2025, &detail, &summary).unwrap_err();
        assert!(error.contains("pays stem school 000001"), "{error}");
    }

    #[test]
    fn an_e_school_paid_the_supplement_is_refused_and_one_paid_nothing_is_kept() {
        let (mut detail, mut summary) = sheets(&FY2027, AT_LEAST);
        school(&FY2027, &mut detail, &mut summary, "Y", Some(1.0));
        assert!(build(&FY2027, &detail, &summary)
            .unwrap_err()
            .contains("pays e-school school 000001"));

        // Paid nothing, it stays in the file — with its designation and its ADM, which is how a
        // reader tells a school that receives nothing from a school that is not there.
        let (mut detail, mut summary) = sheets(&FY2027, AT_LEAST);
        school(&FY2027, &mut detail, &mut summary, "Y", Some(0.0));
        let last = summary.len() - 1;
        #[allow(clippy::cast_precision_loss)]
        let total = (AT_LEAST - 1) as f64 * ADM * RATE;
        summary[last][S_EQUITY] = format!("{total:.6}");
        let built = build(&FY2027, &detail, &summary).unwrap();
        assert_eq!(built[0][3], "e-school");
        assert_eq!(built[0][4], "100");
        assert_eq!(built[0][12], "0");
    }

    #[test]
    fn the_two_sheets_are_checked_against_each_other() {
        let (mut detail, summary) = sheets(&FY2027, AT_LEAST);
        detail[3][D_EQUITY] = format!("{:.6}", ADM * RATE + 5.0);
        let error = build(&FY2027, &detail, &summary).unwrap_err();
        assert!(
            error.contains(CS_SUMMARY_SHEET) && error.contains(CS_DETAIL_SHEET),
            "the error names both sheets: {error}"
        );
    }

    #[test]
    fn a_school_on_one_sheet_and_not_the_other_is_refused() {
        let (mut detail, summary) = sheets(&FY2027, AT_LEAST);
        detail.remove(3);
        let error = build(&FY2027, &detail, &summary).unwrap_err();
        assert!(error.contains("not the same population"), "{error}");
    }

    #[test]
    fn the_statewide_row_is_excluded_and_used_to_check_the_rest() {
        // One over the floor, so that removing a school below tests the statewide check rather
        // than tripping the floor first.
        let (detail, summary) = sheets(&FY2027, AT_LEAST + 1);
        let built = build(&FY2027, &detail, &summary).unwrap();
        assert_eq!(
            built.len(),
            AT_LEAST + 1,
            "the aggregate row is not a school"
        );
        assert!(!built.iter().any(|r| r[0] == "050765"));

        // And it is not decoration: a dropped school is invisible in a per-school file and
        // obvious against the department's own total.
        let mut short = summary.clone();
        short.remove(1);
        let error = build(&FY2027, &detail, &short).unwrap_err();
        assert!(error.contains("counted twice"), "{error}");
    }

    #[test]
    fn a_file_with_no_statewide_row_is_refused() {
        let (detail, mut summary) = sheets(&FY2027, AT_LEAST);
        summary.pop();
        let error = build(&FY2027, &detail, &summary).unwrap_err();
        assert!(error.contains("checked against nothing"), "{error}");
    }

    #[test]
    fn a_workbook_that_lost_its_schools_is_refused() {
        let (detail, summary) = sheets(&FY2027, 12);
        let error = build(&FY2027, &detail, &summary).unwrap_err();
        assert!(error.contains("fewer than"), "{error}");
    }

    #[test]
    fn the_fy2025_layout_is_read_by_the_same_builder_and_yields_the_same_width() {
        // Four column spellings apart and one line short, and every row still comes out
        // seventeen wide — which is what lets one reader hold both years.
        let (detail, summary) = sheets(&FY2025, AT_LEAST);
        let built = build(&FY2025, &detail, &summary).unwrap();
        assert_eq!(built.len(), AT_LEAST);
        for record in &built {
            assert_eq!(record.len(), COMMUNITY_SCHOOL_FUNDING_HEADER.len());
        }
        assert_eq!(built[0][12], format!("{}", ADM * FY25_RATE));
    }

    #[test]
    fn a_line_the_year_did_not_have_is_left_empty_rather_than_written_zero() {
        // The base funding supplement is H.B. 96's. FY2025 has no such column, and a zero there
        // would say the department computed the line and paid nothing — which is a different
        // claim from the true one, and the one a reader summing the column would believe.
        let base_funding_supplement = COMMUNITY_SCHOOL_FUNDING_HEADER
            .iter()
            .position(|name| *name == "base_funding_supplement")
            .expect("the header names it");

        let (detail, summary) = sheets(&FY2025, AT_LEAST);
        let built = build(&FY2025, &detail, &summary).unwrap();
        assert!(built
            .iter()
            .all(|record| record[base_funding_supplement].is_empty()));

        // And the absence is the year's, not the builder's: FY2027 has the column and is refused
        // without it, so a future file that quietly dropped the line could not pass as FY2025.
        let (detail, mut summary) = sheets(&FY2027, AT_LEAST);
        summary[0][13] = "J. Base Funding".to_string();
        let error = build(&FY2027, &detail, &summary).unwrap_err();
        assert!(error.contains("J. Base Funding Supplement"), "{error}");
    }

    #[test]
    fn each_vintage_refuses_the_others_spellings() {
        // The reason the vintage is an argument rather than something sniffed from the file. A
        // builder that fell back through both years' column names would read FY2025's `Aa. Base
        // Cost` — the FY2020 base — as FY2027's calculated base cost on a file that had lost a
        // column, and every row would look like money.
        let (detail, summary) = sheets(&FY2025, AT_LEAST);
        let error = build_community_school_funding(
            CommunitySchoolVintage::Fy2027,
            &display(FY25_RATE),
            &detail,
            &summary,
        )
        .unwrap_err();
        assert!(error.contains("A. Base Cost Calculated"), "{error}");

        let (detail, summary) = sheets(&FY2027, AT_LEAST);
        let error = build_community_school_funding(
            CommunitySchoolVintage::Fy2025,
            &display(RATE),
            &detail,
            &summary,
        )
        .unwrap_err();
        assert!(error.contains("Ab. Base Cost (new)"), "{error}");
    }
}
