//! Territory transfers under R.C. 3311, as the Auditor of State recites them.
//!
//! The answer to "why did a district stop appearing": a transfer order moves territory between
//! districts and sometimes dissolves one. The orders are not published as data, so this reads the
//! sentence that recites each one out of the audit report's prose.

use super::text::flatten;

/// Columns of the transfer extract.
///
/// Written as TSV rather than CSV, and for once the reason is the *quotation* rather than a name.
/// `recital` is a sentence from a state officer's report and it is carried verbatim; the writer
/// this repository uses does not quote, so a comma in a field is refused rather than escaped. A
/// quotation with its commas stripped is a damaged quotation, so the file takes the other
/// delimiter. The same choice was made for the Catalog's legal-basis extract, for the same reason.
pub const TRANSFER_HEADER: &[&str] = &[
    "report",
    "audited_entity",
    "role",
    "resolving_body",
    "resolution_date",
    "effective_date",
    "departing",
    "receiving",
    "section",
    "recital",
];

/// One audit report and what this reader expects to find in it.
///
/// The expectations are part of the extract rather than notes beside it. A recital is a sentence,
/// and a sentence extractor that finds *something* is not evidence it found the right thing — so
/// each report declares the departing agency, the receiving one and the resolution date it should
/// yield, and [`build_transfers`] fails if the document does not say so.
#[derive(Debug, Clone, Copy)]
pub struct AuditReport<'a> {
    /// The source key, which is how a row is traced back to a digest.
    pub report: &'a str,
    /// The entity whose audit this is.
    pub audited_entity: &'a str,
    /// `departing`, `receiving`, or `resolving` — where the audited entity stands to the transfer.
    pub role: &'a str,
    /// The body whose resolution the report recites.
    pub resolving_body: &'a str,
    /// The date that resolution was passed, as the report gives it.
    pub resolution_date: &'a str,
    /// When the transfer took effect.
    pub effective_date: &'a str,
    /// The agency that ceased.
    pub departing: &'a str,
    /// The agency that took its territory.
    pub receiving: &'a str,
    /// The Revised Code section, where the report names one. Empty otherwise, which is the
    /// ordinary case: only one of these five cites a section.
    pub section: &'a str,
    /// The words the recital opens with.
    ///
    /// Declared per report rather than inferred, because a sentence boundary is not one in a PDF
    /// financial statement: page numbers, running heads and all-capital note titles carry no full
    /// stop, so bounding on `". "` alone reaches back through `- iv -` and
    /// `NOTES TO THE BASIC FINANCIAL STATEMENTS` and swallows them into the quotation. Stating the
    /// opening is also the check — a report that no longer begins its recital this way has been
    /// revised, and that should stop the build rather than change a committed quotation.
    pub opens_with: &'a str,
    /// `pdftotext -layout` output for the whole report.
    pub text: &'a str,
}

/// The transfer orders, as recited in the Auditor of State's reports.
///
/// # Why a recital rather than the order
///
/// Three Ohio school districts stopped existing inside the window
/// [`dispersion::lea_directory`](../../dispersion/src/lea_directory.rs) holds, and the federal
/// agency directory files all three as closed *"with no effect on another agency's boundaries"*,
/// which is false. R.C. 3311.22 puts the order in an educational service center governing board's
/// minute book, and those are not published: one sits behind a vendor firewall, another begins its
/// public archive eight years after the resolution.
///
/// What is published is the Auditor of State's report on the **receiving** district, which recites
/// the resolution by date and issuing body. That is a state officer's account of a local body's
/// act, and it is the only route to the fact that this repository can commit.
///
/// **The proposing body's own audit does not mention it.** Geauga County ESC's final report spans
/// both Newbury resolutions and never says "Newbury"; its FY2015 report spans the Ledgemont
/// resolution and never says "Ledgemont". Anyone looking for a transfer where the order was made
/// finds nothing and concludes wrongly that nothing happened.
///
/// # What is checked
///
/// Each report declares what it should say before it is read. The extractor requires the departing
/// agency, the receiving agency and the resolution date all to appear, and it takes the recital
/// from the sentence carrying the date rather than from wherever the names first occur. A report
/// that has been reposted, or a sentence extractor that drifts onto the wrong paragraph, fails
/// here instead of writing a plausible row.
///
/// # Errors
///
/// Returns which expectation a report did not meet.
pub fn build_transfers(reports: &[AuditReport<'_>]) -> Result<Vec<Vec<String>>, String> {
    let mut out = Vec::new();
    for report in reports {
        // Whitespace in these PDFs wraps mid-sentence, so everything is matched against a single
        // collapsed line rather than against the layout.
        let flat = flatten(report.text);
        for expected in [report.departing, report.receiving, report.resolution_date] {
            if !flat.contains(expected) {
                return Err(format!(
                    "{}: the report does not contain {expected:?}, so it is not the document this \
                     reader was written against",
                    report.report
                ));
            }
        }
        let recital = sentence_around(
            &flat,
            report.resolution_date,
            report.departing,
            report.opens_with,
        )
        .ok_or_else(|| {
            format!(
                "{}: no sentence opens {:?} and carries both {} and {}, so either the report has \
                 been revised or the date belongs to something else in it",
                report.report, report.opens_with, report.resolution_date, report.departing
            )
        })?;
        out.push(vec![
            report.report.to_string(),
            report.audited_entity.to_string(),
            report.role.to_string(),
            report.resolving_body.to_string(),
            report.resolution_date.to_string(),
            report.effective_date.to_string(),
            report.departing.to_string(),
            report.receiving.to_string(),
            report.section.to_string(),
            // Verbatim. Tabs are the delimiter and no report contains one; a recital that did
            // would be caught by the writer rather than silently split.
            recital,
        ]);
    }
    Ok(out)
}

/// The sentence carrying both `needle` and `must_name`, bounded by full stops.
///
/// Bounded rather than windowed by character count: a fixed window cuts a recital in half and
/// leaves half of somebody else's, which reads as a quotation and is not one.
///
/// **Every** occurrence of the date is tried, not the first. These reports repeat a date in a
/// contents page, a transmittal letter and a note, and the first occurrence is routinely the one
/// that says nothing — which is what this function returned before it took a second condition.
fn sentence_around(flat: &str, needle: &str, must_name: &str, opens_with: &str) -> Option<String> {
    /// Long enough for the longest of these recitals and far short of a financial table, which is
    /// what an unbounded sentence turns into in a document with no full stops in its headings.
    const LONGEST: usize = 600;

    let mut from = 0usize;
    while let Some(offset) = flat[from..].find(needle) {
        let at = from + offset;
        // The declared opening, taken from before the date where the sentence leads up to it and
        // from the date itself where it opens on it.
        let start = flat[..at + needle.len()].rfind(opens_with);
        let end = flat[at..]
            .find(". ")
            .map_or(flat.len(), |stop| at + stop + 1);
        if let Some(start) = start {
            let sentence = flat[start..end].trim();
            if sentence.len() <= LONGEST && sentence.contains(must_name) {
                return Some(sentence.to_string());
            }
        }
        from = at + needle.len();
    }
    None
}
