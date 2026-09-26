//! Johns Hopkins University, Institute for Education Policy.
//!
//! The only aggregator in this registry, and the only connector here whose publisher is not the
//! source of record for anything it publishes. It is registered anyway because of what it holds:
//! the per-district home-education counts the department collects, releases on request, and
//! publishes nowhere.
//!
//! # Why an aggregator is registered at all
//!
//! The prelude's rule is that where the source of record is unreachable and an aggregator
//! carries the same figures, those figures support `[inference]` and never `[verified]`. That is
//! a rule about what may be claimed, not about what may be committed. Pinning the file by digest
//! and building a fixture from it is what makes the `[inference]` checkable: the claim becomes
//! "this is what the Hub published, and here are its bytes", which is verifiable, rather than
//! "Ohio had this many home-educated children per district", which is not.
//!
//! The identification runs the other way too. The Hub's 2023-24 statewide cell is 53,051, and
//! the department's own fact sheet — [`super::dew::FACTS_AND_FIGURES`], pinned separately —
//! prints 53,051. Two independently pinned files agreeing to the student is the evidence that
//! this series is the department's counts carried forward and not an estimate, and neither file
//! alone carries it.
//!
//! # A Google Sheets export is a stable content address, as it turns out
//!
//! The catalog entry and the `dew-facts-and-figures` decision record both rejected this file on
//! the ground that a Sheets export has no stable content address and so nothing could rebuild
//! byte-identically from it. That was a supposition, and it is wrong: four downloads across two
//! days returned the same 645,110 bytes under the same SHA-256. The export is deterministic —
//! the document is not recalculated per request — and the digest manifest holds it to that. If
//! the Hub revises the workbook the digest moves and the rebuild says so, which is the same
//! guarantee every other source here gets.
//!
//! What remains true is that a Sheets export has no *version* in its URL: there is one edition,
//! and a revision replaces it rather than appearing beside it. So the digest is the only record
//! that the file read today is the file read before, and it is load-bearing here in a way it is
//! not for a publisher who leaves last year's PDF up.

use super::{Connector, Format, Source, Status};

pub(super) const HOMESCHOOL_HUB: Connector = Connector {
    key: "jhu-homeschool-hub",
    publisher: "Johns Hopkins University Institute for Education Policy",
    feeds: &["metric", "education-agency"],
    status: Status::Wired {
        still_blocked: Some(
            "One school year per district, and the workbook labels it wrongly. The department \
             holds the R.C. 3321.042 notice counts per district for every year and releases \
             them on request; nothing below the state total is published, so the other years \
             need a records request rather than a connector. What the count measures — notices \
             received or children named on them — is undocumented and the two differ by \
             household size.",
        ),
    },
    note: "An aggregator that asked the department for what it does not publish. Its Ohio \
           district sheet is the only per-district home-education breakdown known to exist, and \
           its statewide cell for 2023-24 matches the department's own fact sheet to the student.",
    sources: &[Source {
        key: "homeschool-hub-workbook",
        title: None,
        url: "https://docs.google.com/spreadsheets/d/1xFaTmfL1W-rkgrT_M39aApH2SVTjpnte/export?format=xlsx",
        filename: "homeschool-hub.xlsx",
        format: Format::Xlsx,
        catalog: Some("jhu-homeschool-hub"),
        fixtures: &[
            crate::fixtures::HOME_EDUCATION_FIXTURE,
            crate::fixtures::HOME_EDUCATION_STATEWIDE_FIXTURE,
        ],
        note: "47 sheets: an all-states summary, 39 state sheets, a codebook, a data-links \
               sheet, and five disaggregations. Ohio is in exactly two of them — a statewide \
               annual series on `OH`, and 611 district rows on the county/district sheet that \
               carry a name and a count and no key. The district rows are labeled SY21-22 and \
               sum to more than the same workbook's 2021-22 state total, so the label is \
               refuted by the file against itself; the fixture carries it verbatim and decides \
               nothing. Three censoring markers, of which the codebook documents none.",
    }],
};
