//! Supreme Court of Ohio, and the appellate courts it publishes alongside.
//!
//! *DeRolph* and what followed it. The corpus reads opinions rather than summaries because the
//! holdings it needs — what "thorough and efficient" was held to require — are in the text.

use super::{Connector, Format, Source, Status};

pub(super) const OPINIONS: Connector = Connector {
    key: "ohio-courts",
    publisher: "Supreme Court of Ohio",
    feeds: &["litigation"],
    status: Status::Wired {
        still_blocked: Some(
            "the 2025 EdChoice merits ruling is a common pleas decision, which the Ohio \
             Reporter of Decisions does not publish and the Franklin County clerk publishes \
             under conditions that forbid redistribution and direct organizations to a \
             records request. And `citing_cases` stays unfilled: a free citation graph \
             exists, and it identifies ten of the twenty-five Ohio decisions that cite \
             DeRolph I by reporter citation in its own corpus",
        ),
    },
    note: "Wired for the four DeRolph opinions and the EdChoice case's one appellate \
           decision. The recorded blocker had two clauses and neither survived contact \
           intact: \"opinions are PDFs\" stopped being one the moment `Format::Pdf` had a \
           reader, and the other two were assumptions about a citator and about a trial \
           court's archive that are now measured. See `sources/ohio-courts.md`.",
    sources: &[
        Source {
            key: "derolph-i",
            title: Some("DeRolph I, 78 Ohio St.3d 193 (1997)"),
            url: "https://www.supremecourt.ohio.gov/rod/docs/pdf/0/1997/1997-ohio-84.pdf",
            filename: "derolph-i.pdf",
            format: Format::Pdf,
            catalog: Some("derolph-litigation-record"),
            fixtures: &[crate::fixtures::OPINIONS_FIXTURE],
            note: "The opinion the corpus's charge-off rate series is built from. \
                   Paragraph 97 recites the whole progression with its session-law \
                   citations, and names the base as total taxable value.",
        },
        Source {
            key: "derolph-ii",
            title: Some("DeRolph II, 89 Ohio St.3d 1 (2000)"),
            url: "https://www.supremecourt.ohio.gov/rod/docs/pdf/0/2000/2000-ohio-437.pdf",
            filename: "derolph-ii.pdf",
            format: Format::Pdf,
            catalog: Some("derolph-litigation-record"),
            fixtures: &[crate::fixtures::OPINIONS_FIXTURE],
            note: "The second of the four, holding the system still unconstitutional \
                   after the General Assembly's first response.",
        },
        Source {
            key: "derolph-iii",
            title: Some("DeRolph III, 93 Ohio St.3d 309 (2001)"),
            url: "https://www.supremecourt.ohio.gov/rod/docs/pdf/0/2001/2001-ohio-1343.pdf",
            filename: "derolph-iii.pdf",
            format: Format::Pdf,
            catalog: Some("derolph-litigation-record"),
            fixtures: &[crate::fixtures::OPINIONS_FIXTURE],
            note: "The WebCite is 2001-Ohio-1343; \
                   this entry first carried 2001-Ohio-114, which is a workers' compensation \
                   appeal. A citation guessed from a plausible number rather than read off \
                   the document — the same failure this connector was wired to make \
                   checkable, committed while wiring it, and caught by a test that reads what \
                   the file actually says.",
        },
        Source {
            key: "derolph-iv",
            title: Some("DeRolph IV, 97 Ohio St.3d 434 (2002)"),
            url: "https://www.supremecourt.ohio.gov/rod/docs/pdf/0/2002/2002-ohio-6750.pdf",
            filename: "derolph-iv.pdf",
            format: Format::Pdf,
            catalog: Some("derolph-litigation-record"),
            fixtures: &[crate::fixtures::OPINIONS_FIXTURE],
            note: "The last word, and the one that ended judicial supervision without \
                   a remedy.",
        },
        Source {
            key: "edchoice-10th-2024",
            title: Some("Columbus City School Dist. v. State, 2024-Ohio-1217 (10th Dist.)"),
            url: "https://www.supremecourt.ohio.gov/rod/docs/pdf/10/2024/2024-Ohio-1217.pdf",
            filename: "edchoice-10th-2024.pdf",
            format: Format::Pdf,
            catalog: Some("derolph-litigation-record"),
            fixtures: &[crate::fixtures::EDCHOICE_FIXTURE],
            note: "The EdChoice challenge's only appellate decision, and it is not about \
                   EdChoice. The Ohio Senate President, a non-party, appealed an order \
                   modifying a deposition subpoena served on him; the Tenth District \
                   dismissed for want of a final appealable order. What it is wired for is \
                   identity: it states the caption, the appellate number, and the trial \
                   court's case number, all of which `vouchers-hurt-ohio-2025` carried as \
                   `[open]` and none of which the merits ruling is published anywhere this \
                   repository may redistribute from.",
        },
    ],
};
