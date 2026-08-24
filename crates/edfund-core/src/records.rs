//! One way to read a committed *document* fixture.
//!
//! [`crate::csv`] is the reader for tabular fixtures. This is its sibling for the other shape
//! `connect` commits: legal text, which cannot go in a CSV because statutes and opinions are
//! full of commas and `connect::fixtures::write::write_csv` does not quote. Those files are
//! written by `connect::fixtures::statute::build_records` as a sequence of records, each
//! beginning with [`MARKER`] at the start of a line and carrying `title:`, `date:` and
//! `source:` fields above a `--` separator.
//!
//! The format had a writer and no reader. Two files read it — the Revised Code extract and the
//! *DeRolph* opinions — and each had grown its own eighteen-line `find`/`strip_prefix` copy
//! inside a test, so the format was defined in one place and understood in three.
//!
//! # What this asserts and what it does not
//!
//! [`record`] panics on a record that is not there, because every caller names a document the
//! committed fixture is supposed to hold and a missing one means the extract and the reader
//! have come apart. It does **not** check the fields: a record with an empty `date:` is a fact
//! about the publisher's page, and the tests that care assert it themselves.

/// The marker that begins every record, at the start of a line.
///
/// The same constant `connect::fixtures::statute::RECORD_MARKER` writes. It lives here as well
/// because a reader that hard-codes `"=== "` is a reader that will not be found when the
/// writer's marker changes, and `connect` is a leaf that nothing in this workspace depends on.
pub const MARKER: &str = "=== ";

/// One record of a committed document fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record<'a> {
    /// What the record is of — a Revised Code section number, an opinion's short name.
    pub id: &'a str,
    /// The document's own title, as its publisher gives it.
    pub title: &'a str,
    /// The effective or decision date, verbatim. Empty where the publisher shows none.
    pub date: &'a str,
    /// The act or reporter the text came from. Empty where the publisher shows none.
    pub source: &'a str,
    /// The operative text, after the `--` separator and trimmed.
    pub body: &'a str,
}

/// Every record in a committed document fixture, in file order.
///
/// The leading comment block that names the file and its regeneration command is not a record
/// and does not appear here.
pub fn records(text: &str) -> impl Iterator<Item = Record<'_>> {
    // `split` on the marker at a line start, which is the guarantee the writer enforces: it
    // panics on a body line that begins with the marker, so no record can be split in two.
    text.match_indices(MARKER)
        .filter(|(at, _)| *at == 0 || text.as_bytes()[at - 1] == b'\n')
        .map(|(at, _)| at + MARKER.len())
        .map(|start| {
            let rest = &text[start..];
            let end = rest.find(&format!("\n{MARKER}")).unwrap_or(rest.len());
            parse(&rest[..end])
        })
}

/// The record with this identifier.
///
/// # Panics
///
/// If the fixture holds no record with that identifier. Every caller names a document the
/// committed extract is meant to carry, so an absence means the extractor's list and this
/// caller's have come apart — which is a build-time fact about the repository and not a
/// condition to be handled.
#[must_use]
pub fn record<'a>(text: &'a str, id: &str) -> Record<'a> {
    records(text)
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("{id} is not in the committed extract"))
}

/// One record's text, from just after its marker to just before the next.
fn parse(record: &str) -> Record<'_> {
    let field = |name: &str| {
        record
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .unwrap_or_default()
            .trim()
    };
    Record {
        id: record.lines().next().unwrap_or_default().trim(),
        title: field("title:"),
        date: field("date:"),
        source: field("source:"),
        body: record
            .find("\n--\n")
            .map_or("", |at| &record[at + 4..])
            .trim(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = "# A caption, and the note about regenerating it.\n\
         \n\
         === 3317.011\n\
         title: Aggregate base cost.\n\
         date: September 30, 2025\n\
         source: House Bill 96 - 136th General Assembly\n\
         --\n\
         The aggregate base cost is the sum of the following:\n\
         (A) A multiple of 0.2435.\n\
         \n\
         === 319.301\n\
         title: Reduction factors.\n\
         date: April 3, 2023\n\
         source: House Bill 45\n\
         --\n\
         The rate may not be reduced below twenty mills.\n";

    #[test]
    fn the_caption_is_not_a_record() {
        let ids: Vec<&str> = records(FIXTURE).map(|r| r.id).collect();
        assert_eq!(ids, ["3317.011", "319.301"]);
    }

    #[test]
    fn a_record_carries_its_fields_and_its_body() {
        let r = record(FIXTURE, "3317.011");
        assert_eq!(r.title, "Aggregate base cost.");
        assert_eq!(r.date, "September 30, 2025");
        assert_eq!(r.source, "House Bill 96 - 136th General Assembly");
        assert!(r.body.starts_with("The aggregate base cost"));
        assert!(r.body.ends_with("multiple of 0.2435."));
    }

    /// The body runs to the next record and not past it.
    #[test]
    fn one_records_body_does_not_reach_into_the_next() {
        assert!(!record(FIXTURE, "3317.011").body.contains("twenty mills"));
        assert!(!record(FIXTURE, "319.301")
            .body
            .contains("aggregate base cost"));
    }

    /// A marker that is not at the start of a line is text, not a record boundary. Legal
    /// prose quotes rules and tables, and `connect` only guarantees the marker never opens a
    /// line.
    #[test]
    fn a_marker_inside_a_line_is_not_a_boundary() {
        let text = "=== a\ntitle: t\ndate: d\nsource: s\n--\nthe rule reads x === y here.\n";
        let all: Vec<Record<'_>> = records(text).collect();
        assert_eq!(all.len(), 1);
        assert!(all[0].body.contains("x === y here."));
    }

    #[test]
    #[should_panic(expected = "3317.99 is not in the committed extract")]
    fn a_missing_record_fails_loudly() {
        let _ = record(FIXTURE, "3317.99");
    }
}
