//! The department's FY2027 community and STEM school simulator, per school.
//!
//! The sibling of [`super::fy27`], published one directory across under
//! `Community-School-Funding/`, and the only per-*school* formula source this repository holds.
//! It exists here for one line the district model cannot carry at all: the **community school
//! equity supplement**, which H.B. 96 codified into R.C. 3317.022 and which no school district
//! receives.
//!
//! # Who receives it, and why that is the fact worth extracting
//!
//! The supplement is a flat per-pupil payment, and the workbook states its rate in a header cell
//! rather than leaving it to be inferred: `Detail SFPR` row 2 reads *Equity Suplement per pupil*
//! — the department's spelling, matched exactly — against `400`. Every site-based school's
//! payment is that rate times its enrolled ADM, to the cent.
//!
//! **It is not paid to every school in the file, and the split is not by size or county.** Of the
//! 355 schools, 324 are site-based and receive it; **23 e-schools and 8 STEM schools receive
//! nothing**, and those 31 are exactly the schools whose payment is zero. Nothing about a row
//! says so except the `E-School Designation` column, so [`Designation`] is extracted beside the
//! money rather than left to be rediscovered from a zero.
//!
//! That split is worth a builder check rather than a note, because **the published analysis of
//! the introduced budget says the opposite**. The redbook records that the executive proposal
//! "expands eligibility to STEM schools", at $58.0 million a year. The enacted act did not keep
//! it: item 10 of the act's own list pays "community schools **that are not internet- or
//! computer-based**", the greenbook says "site-based community schools", and this workbook pays
//! the eight STEM schools zero. Three enacted sources and the department's own model agree
//! against one document about a bill that changed. So [`build_community_school_funding`] fails if
//! a STEM or e-school row ever carries a payment — if the department extends it, that is a change
//! in the law this corpus should have to notice, not absorb.
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
//! FY2026 model — its phase-in cell reads `0.8333` where this year's reads `1` — carrying 358
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
            "STEM" => Some(Self::Stem),
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

/// The department's own label on the cell holding the per-pupil rate.
///
/// Its misspelling is the department's and is matched as published. Correcting it here would
/// mean the reader stops finding the cell the day they correct it themselves, which is the
/// wrong way round: a changed label should be a named failure.
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
    detail: &[Vec<String>],
    summary: &[Vec<String>],
) -> Result<Vec<Vec<String>>, String> {
    const FILE: &str = "the FY2027 community and STEM school simulator";

    let detail_header = header_row(detail, CS_DETAIL_SHEET, FILE)?;
    let summary_header = header_row(summary, CS_SUMMARY_SHEET, FILE)?;
    let rate = published_rate(&detail[..detail_header], FILE)?;

    let detail_head = detail[detail_header].clone();
    let summary_head = summary[summary_header].clone();
    let at_detail = |name: &str| column(&detail_head, name, FILE);
    let at_summary = |name: &str| column(&summary_head, name, FILE);

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
    // columns and the ADM that comes off the other sheet.
    let money: Vec<usize> = [
        "A. Base Cost Calculated",
        "B. Special Education Calculated",
        "C. DPIA Calculated",
        "D. English Learner Calculated",
        "E. Career Technical Education Calculated",
        "F. Core Foundation Funding",
        "G. Transportation Funding",
        "H. Equity Supplement",
        "I. Formula Transition Supplement",
        "J. Base Funding Supplement",
        "K. Facilities",
        "L. Total State Supports",
    ]
    .iter()
    .map(|name| at_summary(name))
    .collect::<Result<_, _>>()?;
    let s_equity = at_summary("H. Equity Supplement")?;

    let adm = enrolled_adm(&detail[detail_header + 1..], &detail_columns, FILE)?;

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
                "{FILE} gives school {irn} the designation {:?}, which is not N, Y or STEM",
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
                "{FILE} has school {irn} on {CS_SUMMARY_SHEET} and not on {CS_DETAIL_SHEET}; the \
                 two sheets are not the same population"
            ));
        };
        if detail_designation != designation {
            return Err(format!(
                "{FILE} designates school {irn} {} on {CS_DETAIL_SHEET} and {} on \
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
                "{FILE} pays school {irn} an equity supplement of {equity:.2} on \
                 {CS_SUMMARY_SHEET} and {detail_equity:.2} on {CS_DETAIL_SHEET}"
            ));
        }
        check_payment(irn, designation, enrolled, equity, rate, FILE)?;
        paid += equity;

        let mut record = vec![
            irn.to_string(),
            clean_name(cell(row, s_name)),
            clean_name(cell(row, s_county)),
            designation.as_str().to_string(),
            format_value(Some(enrolled), 6),
        ];
        for index in &money {
            record.push(format_value(cell_number(row, *index), 2));
        }
        out.push(record);
    }

    if out.len() < AT_LEAST {
        return Err(format!(
            "{FILE} yielded {} schools, fewer than the {AT_LEAST} it should carry",
            out.len()
        ));
    }
    let Some(statewide) = statewide else {
        return Err(format!(
            "{FILE} has no {} row on {CS_SUMMARY_SHEET}, so the per-school rows are checked \
             against nothing",
            edfund_core::conventions::STATEWIDE_ROW
        ));
    };
    // The department's own total, against the rows this fixture keeps. A dropped school or a
    // double-counted one is invisible in a per-school file and obvious here — which is the
    // failure the district calculator's `State of Ohio` row produced once, at exactly twice size.
    if (statewide - paid).abs() > CENT {
        return Err(format!(
            "{FILE} publishes a statewide equity supplement of {statewide:.2} and its schools sum \
             to {paid:.2}; a row is missing or counted twice"
        ));
    }
    Ok(out)
}

/// The per-pupil rate, read from the label the department writes beside it.
///
/// Read rather than assumed. The rate is the one thing about this supplement that H.B. 96 changed
/// twice in two years — $650 to $500 to $400 — so a builder carrying it as a constant would
/// silently keep checking FY2027's arithmetic against FY2027's rate while extracting some later
/// year's payments.
fn published_rate(above_header: &[Vec<String>], file: &str) -> Result<f64, String> {
    let (row, at) = above_header
        .iter()
        .find_map(|row| {
            row.iter()
                .position(|cell| cell.trim() == RATE_LABEL)
                .map(|at| (row, at))
        })
        .ok_or_else(|| {
            format!("{file} has no {RATE_LABEL:?} cell on {CS_DETAIL_SHEET}; its layout has moved")
        })?;
    // The label and its value are not adjacent — the department leaves a merged gap — so the rate
    // is the first number to its right rather than the next cell.
    row.iter()
        .skip(at + 1)
        .find_map(|cell| number(cell))
        .filter(|rate| *rate > 0.0)
        .ok_or_else(|| format!("{file} states no per-pupil rate beside {RATE_LABEL:?}"))
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
                "{file} gives school {irn} the designation {:?}, which is not N, Y or STEM",
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

    /// The detail sheet's headings, in the department's order and spelling.
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

    /// The summary sheet's headings.
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

    /// Column indices this module's tests reach for, on each sheet.
    const D_DESIGNATION: usize = 3;
    const D_EQUITY: usize = 13;
    const D_ADM: usize = 17;
    const S_DESIGNATION: usize = 3;
    const S_EQUITY: usize = 11;

    /// The rate the workbook publishes, and the ADM each generated school carries.
    const RATE: f64 = 400.0;
    const ADM: f64 = 100.0;

    fn row(cells: &[&str], width: usize) -> Vec<String> {
        let mut out: Vec<String> = cells.iter().map(|c| (*c).to_string()).collect();
        out.resize(width, String::new());
        out
    }

    fn heading(head: &[&str]) -> Vec<String> {
        head.iter().map(|h| (*h).to_string()).collect()
    }

    /// A workbook of `count` schools, every one site-based and correctly paid.
    ///
    /// The detail sheet carries the two rate rows above its heading and the summary sheet carries
    /// none, which is the shape the reader actually sees: empty rows are stripped, so the two
    /// sheets' headings land at different indices.
    fn sheets(count: usize) -> (Vec<Vec<String>>, Vec<Vec<String>>) {
        let width = DETAIL_HEAD.len();
        let mut detail = vec![
            row(
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
            ),
            row(
                &["", "", "", "", "", "", "", "", "", RATE_LABEL, "", "400"],
                width,
            ),
            heading(DETAIL_HEAD),
        ];
        let mut summary = vec![heading(SUMMARY_HEAD)];
        let equity = format!("{:.6}", ADM * RATE);

        for i in 0..count {
            let irn = format!("{:06}", i + 1);
            let mut d = row(&[&irn, "Somewhere Academy", "Adams", "N"], width);
            d[D_ADM] = ADM.to_string();
            d[D_EQUITY].clone_from(&equity);
            detail.push(d);

            let mut s = row(
                &[&irn, "Somewhere Academy", "Adams", "N"],
                SUMMARY_HEAD.len(),
            );
            s[S_EQUITY].clone_from(&equity);
            summary.push(s);
        }

        let mut statewide = row(&["050765", "State of Ohio", "***", ""], SUMMARY_HEAD.len());
        #[allow(clippy::cast_precision_loss)]
        let total = count as f64 * ADM * RATE;
        statewide[S_EQUITY] = format!("{total:.6}");
        summary.push(statewide);
        (detail, summary)
    }

    /// Rewrite one school on both sheets, so a test changes a school rather than a cell.
    fn school(
        detail: &mut [Vec<String>],
        summary: &mut [Vec<String>],
        designation: &str,
        equity: Option<f64>,
    ) {
        // The first data row of each sheet: three rate-and-heading rows on one, one on the other.
        detail[3][D_DESIGNATION] = designation.to_string();
        summary[1][S_DESIGNATION] = designation.to_string();
        if let Some(equity) = equity {
            detail[3][D_EQUITY] = format!("{equity:.6}");
            summary[1][S_EQUITY] = format!("{equity:.6}");
        }
    }

    #[test]
    fn a_heading_that_moved_is_an_error_rather_than_a_misread() {
        let (detail, mut summary) = sheets(AT_LEAST);
        summary[0][11] = "Equity".to_string();
        let error = build_community_school_funding(&detail, &summary).unwrap_err();
        assert!(
            error.contains("H. Equity Supplement"),
            "the error names the column that moved: {error}"
        );
    }

    #[test]
    fn the_header_row_is_found_rather_than_counted() {
        // The two sheets' headings sit at different indices because empty rows are stripped, and
        // the builder reads both. A reader that counted would have to be right twice.
        let (detail, summary) = sheets(AT_LEAST);
        assert_eq!(header_row(&detail, CS_DETAIL_SHEET, "f").unwrap(), 2);
        assert_eq!(header_row(&summary, CS_SUMMARY_SHEET, "f").unwrap(), 0);
        assert!(build_community_school_funding(&detail, &summary).is_ok());
    }

    #[test]
    fn the_rate_is_read_from_the_workbook_rather_than_assumed() {
        // H.B. 96 moved this rate twice in two years. A builder holding it as a constant would
        // check a later year's payments against FY2027's and pass.
        let (mut detail, mut summary) = sheets(AT_LEAST);
        detail[1][11] = "500".to_string();
        let error = build_community_school_funding(&detail, &summary).unwrap_err();
        assert!(error.contains("50000.00"), "{error}");

        // And at the published rate the same file is accepted.
        let paid = format!("{:.6}", ADM * 500.0);
        for row in detail.iter_mut().skip(3) {
            row[D_EQUITY].clone_from(&paid);
        }
        let last = summary.len() - 1;
        for row in summary[1..last].iter_mut() {
            row[S_EQUITY].clone_from(&paid);
        }
        #[allow(clippy::cast_precision_loss)]
        let total = AT_LEAST as f64 * ADM * 500.0;
        summary[last][S_EQUITY] = format!("{total:.6}");
        assert!(build_community_school_funding(&detail, &summary).is_ok());
    }

    #[test]
    fn a_designation_outside_the_departments_three_is_refused() {
        let (mut detail, mut summary) = sheets(AT_LEAST);
        school(&mut detail, &mut summary, "DIGITAL", None);
        let error = build_community_school_funding(&detail, &summary).unwrap_err();
        assert!(error.contains("not N, Y or STEM"), "{error}");
    }

    #[test]
    fn a_site_based_school_paid_something_other_than_rate_times_adm_is_refused() {
        let (mut detail, mut summary) = sheets(AT_LEAST);
        school(&mut detail, &mut summary, "N", Some(ADM * RATE + 1.0));
        let error = build_community_school_funding(&detail, &summary).unwrap_err();
        assert!(error.contains("40001.00"), "{error}");
        assert!(error.contains("000001"), "and it names the school: {error}");
    }

    #[test]
    fn a_stem_school_paid_the_supplement_at_all_is_refused() {
        // The check this file exists for. The redbook says the *introduced* budget expanded
        // eligibility to STEM schools; the enacted act did not, and the department's model pays
        // the eight of them zero. If that ever changes it is a change in the law, and the corpus
        // should find out here rather than restate a rule it has recorded as settled.
        let (mut detail, mut summary) = sheets(AT_LEAST);
        school(&mut detail, &mut summary, "STEM", Some(ADM * RATE));
        let error = build_community_school_funding(&detail, &summary).unwrap_err();
        assert!(error.contains("pays stem school 000001"), "{error}");
        assert!(error.contains("R.C. 3317.022"), "{error}");
    }

    #[test]
    fn an_e_school_paid_the_supplement_is_refused_and_one_paid_nothing_is_kept() {
        let (mut detail, mut summary) = sheets(AT_LEAST);
        school(&mut detail, &mut summary, "Y", Some(1.0));
        assert!(build_community_school_funding(&detail, &summary)
            .unwrap_err()
            .contains("pays e-school school 000001"));

        // Paid nothing, it stays in the file — with its designation and its ADM, which is how a
        // reader tells a school that receives nothing from a school that is not there.
        let (mut detail, mut summary) = sheets(AT_LEAST);
        school(&mut detail, &mut summary, "Y", Some(0.0));
        let last = summary.len() - 1;
        #[allow(clippy::cast_precision_loss)]
        let total = (AT_LEAST - 1) as f64 * ADM * RATE;
        summary[last][S_EQUITY] = format!("{total:.6}");
        let built = build_community_school_funding(&detail, &summary).unwrap();
        assert_eq!(built[0][3], "e-school");
        assert_eq!(built[0][4], "100");
        assert_eq!(built[0][12], "0");
    }

    #[test]
    fn the_two_sheets_are_checked_against_each_other() {
        let (mut detail, summary) = sheets(AT_LEAST);
        detail[3][D_EQUITY] = format!("{:.6}", ADM * RATE + 5.0);
        let error = build_community_school_funding(&detail, &summary).unwrap_err();
        assert!(
            error.contains(CS_SUMMARY_SHEET) && error.contains(CS_DETAIL_SHEET),
            "the error names both sheets: {error}"
        );
    }

    #[test]
    fn a_school_on_one_sheet_and_not_the_other_is_refused() {
        let (mut detail, summary) = sheets(AT_LEAST);
        detail.remove(3);
        let error = build_community_school_funding(&detail, &summary).unwrap_err();
        assert!(error.contains("not the same population"), "{error}");
    }

    #[test]
    fn the_statewide_row_is_excluded_and_used_to_check_the_rest() {
        // One over the floor, so that removing a school below tests the statewide check rather
        // than tripping the floor first.
        let (detail, summary) = sheets(AT_LEAST + 1);
        let built = build_community_school_funding(&detail, &summary).unwrap();
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
        let error = build_community_school_funding(&detail, &short).unwrap_err();
        assert!(error.contains("counted twice"), "{error}");
    }

    #[test]
    fn a_file_with_no_statewide_row_is_refused() {
        let (detail, mut summary) = sheets(AT_LEAST);
        summary.pop();
        let error = build_community_school_funding(&detail, &summary).unwrap_err();
        assert!(error.contains("checked against nothing"), "{error}");
    }

    #[test]
    fn a_workbook_that_lost_its_schools_is_refused() {
        let (detail, summary) = sheets(12);
        let error = build_community_school_funding(&detail, &summary).unwrap_err();
        assert!(error.contains("fewer than"), "{error}");
    }
}
