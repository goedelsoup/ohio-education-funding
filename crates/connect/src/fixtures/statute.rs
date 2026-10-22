//! Sections of the Ohio Revised Code, as retrieved records.
//!
//! The output is a text fixture rather than a CSV: the statute is prose, and a test that asserts
//! the statute agrees with a weight has to quote it. [`RECORD_MARKER`] is what lets one file hold
//! many sections and still be split back apart.

/// One record of a committed document extract: a statute section, or a court opinion.
///
/// # Why one type rather than two
///
/// A legal claim in this corpus rests on either a statute or an opinion, and both should read
/// alike and parse alike. The fields are therefore named for what every document has rather than
/// for what a statute has: this type began as a statute-only record and was reused for the
/// DeRolph opinions with `section` holding `derolph-i`, `legislation` holding a PDF URL, and
/// `effective` forced empty — three fields whose documented meaning the values contradicted, in
/// a file whose purpose is to make citations checkable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// What the record is cited as: `3317.013`, or `derolph-i`.
    pub id: String,
    /// The document's own heading, or the case as it is cited.
    pub title: String,
    /// The date the document speaks from: a statute's effective date, an opinion's decision date.
    pub date: String,
    /// Where it came from: the act that produced a section, or the URL a PDF was retrieved from.
    pub source: String,
    /// The operative text, one paragraph per line.
    pub body: String,
}

/// Pull one section out of a fetched page.
///
/// # Why the body is bounded by two landmarks rather than by a selector
///
/// The page is a site around a statute: navigation above, a version picker and footer below. The
/// operative text begins after the authenticated-PDF link and ends at the version list. Both are
/// stable strings the page prints for every section, and keying on them survives a restyling in a
/// way that keying on a class name would not.
///
/// Returns `None` when neither landmark is found, which is the signal that the page is no longer
/// the page this was written against — better than committing a fixture full of navigation.
#[must_use]
pub fn parse_statute(section: &str, page: &str) -> Option<Record> {
    let text = crate::html::to_text(page);
    let after = |label: &str| -> String {
        text.lines()
            .skip_while(|l| l.trim() != label)
            .nth(1)
            .unwrap_or_default()
            .trim()
            .to_string()
    };

    // "Section 3317.013 | Special education multiples." — the heading, not the <title>.
    let heading = format!("Section {section} |");
    let title = text
        .lines()
        .find(|l| l.starts_with(&heading))
        .map(|l| l[heading.len()..].trim().to_string())
        .unwrap_or_default();

    const BODY_STARTS: &str = "Download Authenticated PDF";
    let start = text.find(BODY_STARTS)? + BODY_STARTS.len();
    let rest = &text[start..];
    // The version picker follows the operative text on every section that has one.
    let end = ["Available Versions of this Section", "Last updated"]
        .iter()
        .filter_map(|marker| rest.find(marker))
        .min()
        .unwrap_or(rest.len());

    let body = rest[..end].trim().to_string();
    if body.is_empty() {
        return None;
    }
    Some(Record {
        id: section.to_string(),
        title,
        date: after("Effective:"),
        source: after("Latest Legislation:"),
        body,
    })
}

/// The section and version a dated `codes.ohio.gov` URL names.
///
/// `https://codes.ohio.gov/ohio-revised-code/section-5705.21/4-9-2025` is the April 9, 2025
/// version of R.C. 5705.21, and this returns `("5705.21", "4-9-2025")`. The version is the URL's
/// own spelling — month, day, year — kept verbatim because it is what the publisher answers to.
/// `None` for a URL with no version segment, which is the current text and belongs in the
/// statute extract instead.
#[must_use]
pub fn version_of(url: &str) -> Option<(&str, &str)> {
    let rest = url.split("/section-").nth(1)?;
    let (section, version) = rest.split_once('/')?;
    let version = version.trim_end_matches('/');
    (!section.is_empty() && !version.is_empty() && !version.contains('/'))
        .then_some((section, version))
}

/// Pull one dated, superseded version of a section out of its page.
///
/// The body, title and effective date are read as [`parse_statute`] reads them. Two things
/// differ. The record is cited as `5705.21@4-9-2025`, so that it can never be mistaken for the
/// current text of the section. And a superseded version's page prints no "Latest Legislation"
/// line, so the act is read from the page's own version list instead: the entry beginning with
/// the version's effective date, as `April 9, 2025 – Amended by House Bill 496 - 135th General
/// Assembly`, less its "Amended by".
///
/// Returns `None` if the page does not parse as a section or its version list has no entry for
/// its own effective date — either says the page is not the one this was written against.
#[must_use]
pub fn parse_statute_version(section: &str, version: &str, page: &str) -> Option<Record> {
    let mut record = parse_statute(section, page)?;
    if record.date.is_empty() {
        return None;
    }
    let text = crate::html::to_text(page);
    let entry = format!("{} \u{2013} ", record.date);
    let act = text
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .find_map(|l| l.strip_prefix(&entry).map(str::to_string))?;
    record.source = act.strip_prefix("Amended by ").unwrap_or(&act).to_string();
    record.id = format!("{section}@{version}");
    Some(record)
}

/// The marker that begins every record, at the start of a line.
///
/// This was `§ `, justified on the grounds that the Revised Code's own text never prints a
/// section mark at column 0. True of statutes, and false of the opinions the same format now
/// carries: DeRolph I quotes `Art. I, § 2` and an out-of-state `M.S.A. §`, and a repost of a PDF
/// or a different `pdftotext` line-breaking would only have to reflow one of them to column 0 to
/// truncate a record and add a phantom one. A marker no legal text has occasion to print is
/// cheaper than an argument about which line breaks are possible — and [`build_records`] checks
/// rather than assuming.
///
/// Defined by [`edfund_core::records`], which is the reader half of this format. It was declared
/// here and understood nowhere: the two files that read these fixtures each spelled the marker
/// out again inside a test, so the writer had no way to tell them it had changed.
pub const RECORD_MARKER: &str = edfund_core::records::MARKER;

/// Render records as a committed fixture.
///
/// A record format rather than CSV, because legal text is full of commas and [`super::write::write_csv`] does
/// not quote. `caption` describes what the file holds; the rest of the header is fixed, because
/// the regeneration command and the record marker are properties of the format rather than of
/// the document. The caption is a parameter because reusing the statute builder for the opinions
/// stamped `# Ohio Revised Code, the sections this corpus cites.` over four Supreme Court of Ohio
/// opinions, and a file that misdescribes itself in its first line is a poor foundation for one
/// whose purpose is provenance.
///
/// # Panics
///
/// If any record's body contains a line beginning with [`RECORD_MARKER`], which would split that
/// record in two for every reader. Failing the rebuild is the point: the alternative is a
/// committed fixture that is silently wrong about how many documents it holds.
#[must_use]
pub fn build_records(caption: &str, records: &[Record]) -> String {
    let mut out = String::with_capacity(records.iter().map(|r| r.body.len() + 256).sum());
    out.push_str(&format!(
        "# {caption}\n\
         # Regenerate with `edfund-connect rebuild`. Records begin with `{RECORD_MARKER}` at the\n\
         # start of a line, which no record's own text does.\n"
    ));
    for r in records {
        assert!(
            !r.body.lines().any(|l| l.starts_with(RECORD_MARKER)),
            "{}: a body line begins with {RECORD_MARKER:?} and would split the record",
            r.id
        );
        out.push_str(&format!(
            "\n{RECORD_MARKER}{}\ntitle: {}\ndate: {}\nsource: {}\n--\n{}\n",
            r.id, r.title, r.date, r.source, r.body
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape of a superseded version's page, cut to its landmarks.
    const PRIOR: &str = "<h1>Section 5705.21 | Election on additional tax levy.</h1>\
        <div>View Current Version</div><div>Effective:</div><div>April 9, 2025</div>\
        <a>Download Authenticated PDF</a>\
        <p>A resolution may propose to increase or decrease a single levy.</p>\
        <h2>Available Versions of this Section</h2>\
        <div>September 29, 2015 \u{2013} House Bill 64 - 131st General Assembly</div>\
        <div>April 9, 2025 \u{2013} Amended  by House Bill 496 - 135th General Assembly</div>\
        <div>January 1, 2026 \u{2013} Amended  by House Bill 96 - 136th General Assembly</div>";

    #[test]
    fn a_dated_url_names_its_section_and_version() {
        assert_eq!(
            version_of("https://codes.ohio.gov/ohio-revised-code/section-5705.21/4-9-2025"),
            Some(("5705.21", "4-9-2025"))
        );
        assert_eq!(
            version_of("https://codes.ohio.gov/ohio-revised-code/section-5705.21"),
            None,
            "the current text has no version segment"
        );
    }

    #[test]
    fn a_prior_version_is_cited_by_its_version_and_dated_by_its_own_entry() {
        let record = parse_statute_version("5705.21", "4-9-2025", PRIOR).expect("parses");
        assert_eq!(record.id, "5705.21@4-9-2025");
        assert_eq!(record.date, "April 9, 2025");
        // Its own entry: not the newest one, and not the first.
        assert_eq!(record.source, "House Bill 496 - 135th General Assembly");
        assert!(record.body.contains("increase or decrease"));
        assert!(!record.body.contains("Available Versions"));
    }

    #[test]
    fn a_version_list_without_the_pages_own_date_is_refused() {
        let page = PRIOR.replace("April 9, 2025 \u{2013}", "April 10, 2025 \u{2013}");
        assert_eq!(parse_statute_version("5705.21", "4-9-2025", &page), None);
    }
}
