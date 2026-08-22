//! One way to read a committed fixture.
//!
//! Every crate here reads delimited fixtures, and before this module there were twenty-one
//! loaders doing it six different ways. Four of them checked nothing at all: they skipped the
//! header line and indexed columns by position, so a column inserted upstream would move every
//! field one to the left and the file would still parse.
//!
//! The `project::panel` reader states the rule the others should have followed — a fixture whose columns
//! shifted is a build-time mistake, and reading it silently "would put wrong numbers into a
//! scenario". This module makes following that rule the shortest path: [`rows`] cannot be
//! called without naming the header it expects.
//!
//! Numbers go through [`crate::conventions::number`], so a suppressed count reads as absent
//! rather than as zero — the distinction `connect` established and four parsers downstream
//! were losing.

use crate::conventions;

/// One row of a fixture, addressed by column position.
pub struct Row<'a> {
    fields: Vec<&'a str>,
}

impl<'a> Row<'a> {
    /// The cell at `index`, or `""` where the row is shorter than that.
    ///
    /// [`rows`] asserts every row against the header's width, so a short row never reaches
    /// here. The tolerance remains for the other way to leave the row — a reader indexing a
    /// column the header does not have — which is a mistake in the reader rather than in the
    /// file, and not one worth a panic in the middle of a fixture load.
    #[must_use]
    pub fn str(&self, index: usize) -> &'a str {
        self.fields.get(index).copied().unwrap_or("").trim()
    }

    /// The cell at `index` as a number, or `None` where there is no usable value.
    ///
    /// Reads the publisher's conventions: a suppressed count, a spreadsheet error marker and
    /// an empty cell all give `None`, and thousands separators are stripped.
    #[must_use]
    pub fn num(&self, index: usize) -> Option<f64> {
        conventions::number(self.str(index))
    }

    /// The cell at `index` as a number, treating an absent one as zero.
    ///
    /// Only for columns where the fixture never omits a value and zero is the right reading of
    /// one that is. Where a missing figure is genuinely unknown, use [`Row::num`] and keep the
    /// absence — summing a suppressed count as zero understates every aggregate over small
    /// districts.
    #[must_use]
    pub fn required(&self, index: usize) -> f64 {
        self.num(index).unwrap_or(0.0)
    }

    /// How many cells the row holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Whether the row holds no cells at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

/// The data rows of a comma-delimited fixture, after asserting its header and their width.
///
/// Blank lines are skipped. The header is compared verbatim after trimming, which is what
/// makes a column insertion upstream a loud failure rather than a silent shift. Every row is
/// then checked against the header's width, which catches the other way columns move: a cell
/// that contains a comma splits into two and shifts everything after it.
///
/// Every committed fixture in this workspace is uniform-width — that was measured, not
/// assumed, before this assertion went in — so the check costs nothing and no reader needs to
/// tolerate a ragged row.
///
/// # Panics
///
/// Panics if the first line is not `expected`, or if any row's cell count differs from the
/// header's. Both are build-time facts rather than runtime conditions — these fixtures are
/// compiled in — and reading shifted columns silently is worse than not reading them at all.
pub fn rows<'a>(text: &'a str, expected: &str) -> impl Iterator<Item = Row<'a>> {
    let mut lines = text.lines();
    let header = lines.next().unwrap_or_default().trim();
    assert_eq!(
        header, expected,
        "a committed fixture's header changed; the columns this reader indexes have moved"
    );
    let width = expected.split(',').count();
    lines
        .filter(|line| !line.trim().is_empty())
        .map(move |line| {
            let fields: Vec<&str> = line.split(',').collect();
            assert_eq!(
                fields.len(),
                width,
                "a fixture row holds {} cells where the header names {width}; a cell containing \
                 a comma shifts every column after it, and reads cleanly: {line}",
                fields.len()
            );
            Row { fields }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADER: &str = "irn,name,amount";

    #[test]
    fn a_row_reads_its_cells_by_position() {
        let text = "irn,name,amount\n000442,Manchester Local,1234.5\n";
        let rows: Vec<Row<'_>> = rows(text, HEADER).collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].str(0), "000442");
        assert_eq!(rows[0].str(1), "Manchester Local");
        assert_eq!(rows[0].num(2), Some(1234.5));
    }

    #[test]
    #[should_panic(expected = "holds 2 cells where the header names 3")]
    fn a_row_narrower_than_its_header_fails_loudly() {
        let text = "irn,name,amount\n000442,Manchester Local\n";
        let _ = rows(text, HEADER).count();
    }

    /// Reading past the header's width is a mistake in the reader, not in the file, and gives
    /// an empty cell rather than a panic part-way through a load.
    #[test]
    fn a_column_the_header_does_not_have_reads_as_empty() {
        let text = "irn,name,amount\n000442,Manchester Local,1234.5\n";
        let rows: Vec<Row<'_>> = rows(text, HEADER).collect();
        assert_eq!(rows[0].str(9), "");
        assert_eq!(rows[0].num(9), None);
        assert_eq!(rows[0].required(9), 0.0);
    }

    #[test]
    fn blank_lines_are_not_rows() {
        let text = "irn,name,amount\n\n000442,A,1\n\n000443,B,2\n\n";
        assert_eq!(rows(text, HEADER).count(), 2);
    }

    /// A suppressed count is an absence, not a zero.
    #[test]
    fn the_publishers_conventions_are_read_rather_than_reimplemented() {
        let text = "irn,name,amount\n000442,A,<10\n000443,B,#DIV/0!\n000444,C,\n";
        let rows: Vec<Row<'_>> = rows(text, HEADER).collect();
        assert_eq!(
            rows[0].num(2),
            None,
            "a suppressed count is absent, not zero"
        );
        assert_eq!(rows[1].num(2), None, "so is a spreadsheet error marker");
        assert_eq!(rows[2].num(2), None, "and so is an empty cell");
    }

    /// This reader splits on commas and does not honour quoting, deliberately — and now says
    /// so out loud rather than reading the shifted columns.
    ///
    /// It can be that simple because the fixtures are guaranteed not to need more:
    /// `connect::fixtures::write_csv` asserts on the way out that no cell contains a comma —
    /// a guard added after two sponsor names ("Holy Trinity, Swanton Ele Sch") shifted a
    /// column and put site IRNs in an enrolment field. But that guard only runs on a rebuild.
    /// The width assertion is the same invariant checked on every read, so a fixture that ever
    /// slips past it fails in `cargo test` rather than in a published figure.
    #[test]
    #[should_panic(expected = "holds 4 cells where the header names 3")]
    fn a_cell_containing_a_comma_fails_rather_than_shifting_the_columns() {
        let text = "irn,name,amount\n000442,\"A, B\",1\n";
        let _ = rows(text, HEADER).count();
    }

    #[test]
    #[should_panic(expected = "header changed")]
    fn a_changed_header_fails_loudly() {
        let text = "irn,name,total\n000442,A,1\n";
        let _ = rows(text, HEADER).count();
    }
}
