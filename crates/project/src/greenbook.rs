//! The Legislative Service Commission's education analysis of every enacted budget act since 2001.
//!
//! # Why this reader did not exist
//!
//! Twelve greenbooks, 2.6 MB, one record per act from the 124th General Assembly through the
//! 135th — written by `connect` on every rebuild and read by nothing. One test in `web/` opens
//! the file to check that each act the budget Catalog cites has a corpus node; nothing had ever
//! read a sentence of it.
//!
//! That mattered because [`crate::statute`] cannot reach the era these cover. Ohio Laws' version
//! archive for R.C. 3317.022 begins on 1 July 2014, after the charge-off was already retired, and
//! `parameter/local-share-charge-off-millage` recorded the consequence: the operative text "must
//! come from the opinions or from session law". The greenbooks are a third route and a better one
//! for a *parameter* — LSC states each formula's rate and base in prose, biennium by biennium,
//! which is exactly the shape a series needs and neither an opinion nor an appropriation table
//! gives.
//!
//! # What a greenbook is and is not
//!
//! It is LSC's analysis of the act **as enrolled** — its account of what the budget does, written
//! for legislators. It is not the act, and where the two differ the act governs. It is also not
//! neutral about emphasis: a greenbook explains what changed, so a parameter that did not change
//! that biennium may go unmentioned in a document that states it plainly two bienniums later.
//! [`Greenbook::mentions`] is for finding where a subject is discussed; the reading is still the
//! caller's.
//!
//! # The General Assembly is derived, not typed
//!
//! LSC serves each analysis under `…/legislation/<general assembly>/<bill>/…`, so
//! [`Greenbook::general_assembly`] comes off the record's own `source:` line. A hand-written table
//! of twelve acts to twelve numbers would be twelve chances to be wrong about a fact the fixture
//! already carries.

use edfund_core::records;

/// The committed extract.
pub const FIXTURE: &str = include_str!("../fixtures/lsc-education-greenbooks.txt");

/// LSC's analysis of one enacted budget act.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Greenbook<'a> {
    /// The record's identifier, as `hb1-greenbook`.
    pub id: &'a str,
    /// The bill, as `hb1`. The act's own designation without its `Am. Sub.` prefix.
    pub bill: &'a str,
    /// The General Assembly that enacted it, read off the publisher's URL.
    pub general_assembly: u16,
    /// LSC's own title for the analysis, which for the earliest records is just the slug.
    pub title: &'a str,
    /// Where the analysis was retrieved from.
    pub source: &'a str,
    /// The analysis itself.
    pub body: &'a str,
}

impl<'a> Greenbook<'a> {
    /// The first fiscal year of the biennium the act appropriates for.
    ///
    /// Ohio's General Assembly convenes in the odd year and its main operating budget takes effect
    /// the following July, so the 124th's budget is FY2002-03 and each later Assembly's is two
    /// years on. Arithmetic rather than a table, and
    /// `tests::the_biennium_follows_from_the_general_assembly` pins it against all twelve.
    #[must_use]
    pub fn first_fiscal_year(&self) -> u16 {
        2002 + 2 * (self.general_assembly - 124)
    }

    /// Whether the analysis discusses a subject, matched case-insensitively.
    ///
    /// For asking *which* bienniums discuss a thing before reading any of them — which is how the
    /// charge-off's disappearance was found, since the answer was a biennium with no mentions.
    #[must_use]
    pub fn mentions(&self, subject: &str) -> bool {
        self.body.to_lowercase().contains(&subject.to_lowercase())
    }

    /// Every line of the analysis containing `subject`, trimmed, in document order.
    ///
    /// Line-oriented because these are extracted from PDFs: the text carries the publisher's own
    /// line breaks, and a sentence routinely spans three of them with a page footer in the middle.
    /// A caller quoting LSC needs the surrounding lines, which [`Greenbook::around`] gives.
    #[must_use]
    pub fn lines_matching(&self, subject: &str) -> Vec<&'a str> {
        let needle = subject.to_lowercase();
        self.body
            .lines()
            .filter(|line| line.to_lowercase().contains(&needle))
            .map(str::trim)
            .collect()
    }

    /// The analysis from the first line matching `subject`, for `lines` lines, whitespace
    /// collapsed — the form a quotation is checked against.
    ///
    /// Returns an empty string where the subject does not appear, because a caller asking for
    /// context around an absent phrase is asking a question whose answer is "it is not there".
    #[must_use]
    pub fn around(&self, subject: &str, lines: usize) -> String {
        let needle = subject.to_lowercase();
        let all: Vec<&str> = self.body.lines().collect();
        let Some(at) = all.iter().position(|l| l.to_lowercase().contains(&needle)) else {
            return String::new();
        };
        all[at..(at + lines).min(all.len())]
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Every greenbook in the extract, oldest General Assembly first.
#[must_use]
pub fn greenbooks() -> Vec<Greenbook<'static>> {
    records::records(FIXTURE).map(read).collect()
}

/// The greenbook for one bill, as `hb1`.
///
/// # Panics
///
/// If the extract holds no analysis of that bill. Every caller names an act the committed extract
/// is meant to carry, so an absence means the extractor's list and the caller's have come apart.
#[must_use]
pub fn greenbook(bill: &str) -> Greenbook<'static> {
    greenbooks()
        .into_iter()
        .find(|g| g.bill == bill)
        .unwrap_or_else(|| panic!("no LSC greenbook for {bill} is committed"))
}

/// One record, with the bill and General Assembly read off it.
fn read(record: records::Record<'static>) -> Greenbook<'static> {
    let (assembly, bill) = from_url(record.source);
    Greenbook {
        id: record.id,
        bill,
        general_assembly: assembly,
        title: record.title,
        source: record.source,
        body: record.body,
    }
}

/// The General Assembly and bill in an LSC asset URL: `…/legislation/128/hb1/en/files/…`.
///
/// # Panics
///
/// On a source line that is not one of those. The fixture's writer builds these URLs, so a shape
/// this cannot read means the writer changed and the reader did not.
fn from_url(source: &str) -> (u16, &str) {
    let rest = source
        .split_once("/legislation/")
        .unwrap_or_else(|| panic!("a greenbook's source is not an LSC legislation URL: {source}"))
        .1;
    let mut parts = rest.split('/');
    let assembly = parts
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("no General Assembly in {source}"));
    let bill = parts
        .next()
        .unwrap_or_else(|| panic!("no bill in {source}"));
    (assembly, bill)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_enacted_budget_since_two_thousand_one_is_here_exactly_once() {
        let all = greenbooks();
        assert_eq!(all.len(), 12);

        let assemblies: Vec<u16> = all.iter().map(|g| g.general_assembly).collect();
        assert_eq!(assemblies, (124..=135).collect::<Vec<u16>>());

        let bills: Vec<&str> = all.iter().map(|g| g.bill).collect();
        assert_eq!(
            bills,
            [
                "hb94", "hb95", "hb66", "hb119", "hb1", "hb153", "hb59", "hb64", "hb49", "hb166",
                "hb110", "hb33"
            ]
        );
    }

    /// The biennium is arithmetic on the General Assembly, and this is what says so.
    #[test]
    fn the_biennium_follows_from_the_general_assembly() {
        assert_eq!(greenbook("hb94").first_fiscal_year(), 2002);
        assert_eq!(greenbook("hb1").first_fiscal_year(), 2010);
        assert_eq!(greenbook("hb110").first_fiscal_year(), 2022);
        assert_eq!(greenbook("hb33").first_fiscal_year(), 2024);

        // No gap: twelve consecutive Assemblies, twelve consecutive bienniums.
        let years: Vec<u16> = greenbooks()
            .iter()
            .map(Greenbook::first_fiscal_year)
            .collect();
        assert_eq!(years, (0..12).map(|i| 2002 + 2 * i).collect::<Vec<u16>>());
    }

    #[test]
    fn a_bill_the_extract_does_not_carry_is_a_panic_and_not_an_empty_answer() {
        assert!(std::panic::catch_unwind(|| greenbook("hb999")).is_err());
    }

    #[test]
    fn context_around_an_absent_phrase_is_empty_rather_than_the_start_of_the_document() {
        let book = greenbook("hb33");
        assert!(book
            .around("a phrase no budget analysis contains", 3)
            .is_empty());
        assert!(!book.around("Department of Education", 3).is_empty());
    }
}
