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
    /// Rows are padded only to their widest populated cell, so a record with nothing in its
    /// last column is shorter than its neighbours. Indexing directly would panic on exactly
    /// the records with missing data.
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

/// The data rows of a comma-delimited fixture, after asserting its header.
///
/// Blank lines are skipped. The header is compared verbatim after trimming, which is what
/// makes a column insertion upstream a loud failure rather than a silent shift.
///
/// # Panics
///
/// Panics if the first line is not `expected`. That is a build-time fact rather than a runtime
/// condition — these fixtures are compiled in — and reading shifted columns silently is worse
/// than not reading them at all.
pub fn rows<'a>(text: &'a str, expected: &str) -> impl Iterator<Item = Row<'a>> {
    let mut lines = text.lines();
    let header = lines.next().unwrap_or_default().trim();
    assert_eq!(
        header, expected,
        "a committed fixture's header changed; the columns this reader indexes have moved"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| Row {
            fields: line.split(',').collect(),
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
    fn a_short_row_reads_as_empty_rather_than_panicking() {
        let text = "irn,name,amount\n000442,Manchester Local\n";
        let rows: Vec<Row<'_>> = rows(text, HEADER).collect();
        assert_eq!(rows[0].str(2), "");
        assert_eq!(rows[0].num(2), None);
        assert_eq!(rows[0].required(2), 0.0);
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

    /// This reader splits on commas and does not honour quoting, deliberately.
    ///
    /// It can be that simple because the fixtures are guaranteed not to need more:
    /// `connect::fixtures::write_csv` asserts on the way out that no cell contains a comma —
    /// a guard added after two sponsor names ("Holy Trinity, Swanton Ele Sch") shifted a
    /// column and put site IRNs in an enrolment field. This test states the dependency, so
    /// that if the guard upstream is ever relaxed the reader's limitation is already written
    /// down rather than discovered.
    #[test]
    fn a_quoted_comma_is_not_supported_because_no_fixture_may_contain_one() {
        let text = "irn,name,amount\n000442,\"A, B\",1\n";
        let rows: Vec<Row<'_>> = rows(text, HEADER).collect();
        assert_eq!(
            rows[0].len(),
            4,
            "the quoted comma splits, as a naive reader must"
        );
    }

    #[test]
    #[should_panic(expected = "header changed")]
    fn a_changed_header_fails_loudly() {
        let text = "irn,name,total\n000442,A,1\n";
        let _ = rows(text, HEADER).count();
    }
}
