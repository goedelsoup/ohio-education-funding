//! The Ohio Revised Code sections this corpus cites, as committed text.
//!
//! # Why the law is a fixture and not a citation
//!
//! Every weight in this workspace was transcribed from the department's FY2027 calculator and
//! confirmed by reproducing the department's own per-district amounts. That is a strong check and
//! it verifies the wrong thing: it establishes that the department is **self-consistent**. A
//! spreadsheet can be internally perfect and still not be the statute.
//!
//! `codes.ohio.gov` serves HTML with no bulk export, which was true and was a fact about the
//! absence of a convenience rather than about the availability of the text. The text is
//! server-rendered, `connect`'s `ohio-laws` connector reads it, and the extract this module reads
//! is what makes the independent check possible. All fourteen funding multiples match the
//! Revised Code to the last digit; `tests/the_statute_behind_the_weights.rs` is that check.
//!
//! # And the text answers questions the numbers could not
//!
//! Four items the corpus recorded as `[open]` were open because the calculator publishes
//! quantities and not reasons. The statute publishes reasons: it names the clinical categories
//! behind the special education weights, states what the English learner taper tracks, lists the
//! career-technical programme areas, and contains the squaring the DPIA node had recorded as "not
//! located in statute here". One citation was simply wrong — the DPIA node cited R.C. 3317.029,
//! and there is no such section.
//!
//! # The extract is a snapshot with a date on every record
//!
//! [`Section::effective`] and [`Section::legislation`] are the publisher's own, and they matter:
//! R.C. 319.301 was amended by the 136th General Assembly, and a corpus pinned to an older
//! reading of the twenty-mill floor would not know. Any claim made from a body here should carry
//! the record's date.

use edfund_core::records;
use edfund_core::FiscalYear;

/// The committed extract.
pub const FIXTURE: &str = include_str!("../fixtures/revised-code.txt");

/// One section of the Revised Code, as the extract holds it.
///
/// A thin renaming of [`edfund_core::records::Record`]: the extract's field labels are generic —
/// it holds Supreme Court opinions in the same format — and this reads them back into the names
/// a statute actually has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Section<'a> {
    /// The section number, as `3317.011`.
    pub number: &'a str,
    /// The section's own title, as `codes.ohio.gov` gives it.
    pub title: &'a str,
    /// The effective date of the version held, verbatim.
    pub effective: &'a str,
    /// The act that last amended it.
    pub legislation: &'a str,
    /// The operative text.
    pub body: &'a str,
}

impl<'a> From<records::Record<'a>> for Section<'a> {
    fn from(record: records::Record<'a>) -> Self {
        Self {
            number: record.id,
            title: record.title,
            effective: record.date,
            legislation: record.source,
            body: record.body,
        }
    }
}

/// The clause three sections of the plan open with, verbatim.
const EXPIRY: &str = "This section shall apply only for fiscal years 2026 and 2027.";

/// The last fiscal year the Fair School Funding Plan's own sections apply to.
///
/// # Why this is a constant and not a horizon
///
/// This repository projects state aid to FY2032 and FY2036, and labels those runs "current law".
/// At FY2032 **current law is not a law**: R.C. 3317.011 (base cost), 3317.017 (local capacity and
/// the minimum state share inside it) and 3317.0217 (targeted assistance) each open with
/// *"This section shall apply only for fiscal years 2026 and 2027."*, and five further sections
/// hand values back clause by clause — forty divisions
/// reading *"For fiscal year 2028 and each fiscal year thereafter, an amount calculated in a
/// manner determined by the general assembly"*, seventeen of them in R.C. 3317.022 alone.
///
/// The projection is not wrong and shortening its horizon would not improve it: a five-year
/// question about a two-year statute is still worth asking, and "if the plan continues unchanged"
/// is the only tractable answer. What was missing is that the page never said so. See
/// `.yidam/decisions/the-plan-expires-and-the-projection-does-not.yml`.
///
/// `sections_expiring_in_2027` checks this against the committed extract rather than trusting the
/// digits here, so an amendment that moves the date fails in this crate rather than surfacing as a
/// caveat that has quietly become false.
pub const LAST_STATUTORY_YEAR: FiscalYear = FiscalYear(2027);

/// How many sections of the extract open *"This section shall apply only for fiscal years 2026
/// and 2027."*
///
/// Three, and naming the count is what makes [`LAST_STATUTORY_YEAR`] a reading of the source
/// rather than a number somebody typed.
#[must_use]
pub fn sections_expiring() -> usize {
    sections()
        .iter()
        .filter(|s| s.body.contains(EXPIRY))
        .count()
}

/// Every section the extract holds, in file order.
#[must_use]
pub fn sections() -> Vec<Section<'static>> {
    records::records(FIXTURE).map(Section::from).collect()
}

/// One section by number.
///
/// # Panics
///
/// If the extract holds no such section — which means `connect`'s section list and the caller's
/// have come apart, and is a build-time fact about this repository rather than a runtime
/// condition.
#[must_use]
pub fn section(number: &str) -> Section<'static> {
    records::record(FIXTURE, number).into()
}

/// Every `multiple of X` a section states, in the order it states them.
///
/// This is how the Revised Code writes a funding weight: `a multiple of 2.0961` for special
/// education category 6, `a multiple of 0.2104` for the first English learner category. The
/// corpus carries them as bare numbers, so reading them back out of the prose is what makes the
/// two comparable at all.
///
/// The ordering carries meaning and is preserved: special education ascends with severity,
/// English learners descend as the pupil progresses, and a list that came back sorted would
/// destroy both findings.
#[must_use]
pub fn multiples(body: &str) -> Vec<f64> {
    const MARKER: &str = "multiple of ";
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(at) = rest.find(MARKER) {
        rest = &rest[at + MARKER.len()..];
        let digits: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(value) = digits.trim_end_matches('.').parse::<f64>() {
            out.push(value);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every record carries a number, a title and a date. The extract's whole purpose is
    /// provenance, and a record without one is not evidence of anything.
    #[test]
    fn every_section_is_numbered_titled_and_dated() {
        let all = sections();
        assert!(all.len() > 30, "the extract holds {} sections", all.len());
        for s in all {
            assert!(!s.number.is_empty());
            assert!(!s.title.is_empty(), "{} has no title", s.number);
            assert!(!s.effective.is_empty(), "{} has no date", s.number);
            assert!(
                s.body.len() > 400,
                "{} body is {} chars",
                s.number,
                s.body.len()
            );
        }
    }

    /// The multiples come back in the statute's order, which is the finding rather than a
    /// convenience: R.C. 3317.013 lists special education ascending.
    #[test]
    fn the_multiples_keep_the_statutes_own_ordering() {
        let weights = multiples(section("3317.013").body);
        assert_eq!(weights.len(), 6);
        assert!(weights.windows(2).all(|w| w[1] > w[0]), "{weights:?}");
    }

    /// A section with no weights in it yields none, rather than a stray parse off the prose.
    #[test]
    fn a_section_that_states_no_multiple_yields_none() {
        assert!(multiples(section("319.301").body).is_empty());
    }
}

#[cfg(test)]
mod expiry {
    use super::{sections_expiring, LAST_STATUTORY_YEAR};

    /// The constant is a reading of the extract, and this is the reading.
    ///
    /// Three sections carry the clause: R.C. 3317.011, 3317.017 and 3317.0217. If an amendment
    /// moves the date or adds a fourth, this fails here — in the crate that owns the text —
    /// rather than as a caveat on a page that has quietly stopped being true.
    #[test]
    fn three_sections_expire_with_the_biennium() {
        assert_eq!(
            sections_expiring(),
            3,
            "sections carrying the expiry clause"
        );
        assert_eq!(LAST_STATUTORY_YEAR.0, 2027);
    }
}
