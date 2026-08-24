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
