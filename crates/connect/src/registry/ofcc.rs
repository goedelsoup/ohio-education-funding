//! Ohio Facilities Construction Commission.
//!
//! The capital channel, and the one connector blocked by a policy choice rather than a data
//! problem: the project data exists and is not served in a form this repository may take.
//!
//! # The refusal is the estate's, not this agency's
//!
//! `ofcc.ohio.gov` answers **every** path with the same 5,248-byte 404 — root, `robots.txt`,
//! `sitemap.xml`, a nonexistent path — so no 404 from that host is evidence about any document.
//! `obm.ohio.gov` does the same at 5,278 bytes and `tax.ohio.gov` at 5,036. It is one filter
//! across `*.ohio.gov` rather than a decision this commission took, which matters because it
//! means no amount of asking OFCC would change it. `ohioauditor.gov`, on its own domain, answers
//! this project's agent with a 200.
//!
//! # The alternate publishers named in #11 have been surveyed
//!
//! See `decisions/the-capital-channel-has-no-second-publisher`. The short of it: the capital
//! appropriation acts are reachable through machinery this repository already has and
//! appropriate the whole school facilities programme as two lump lines. They size the channel
//! and cannot allocate it.
//!
//! # Except for the share rule
//!
//! The archive holds more than the portal shell. The Commission's equity lists and its
//! programme status editions are PDFs on `dam.assets.ohio.gov`, and the Wayback Machine captured
//! several of them. That does not allocate the channel — no archived document says what a project
//! cost — but it is the input R.C. 3318.01(K) reads a district's share from, so [`EQUITY_LISTS`]
//! makes the share rule checkable where [`PROJECTS`] cannot make the spending checkable. See
//! `decisions/ofcc-equity-lists-connector`.

use super::{Connector, Format, Source, Status};

/// A Wayback capture of a file on the Commission's asset host, by capture timestamp and path.
///
/// `id_` asks the archive for the bytes it captured rather than its replay page.
macro_rules! ofcc_capture {
    ($timestamp:literal, $path:literal) => {
        concat!(
            "https://web.archive.org/web/",
            $timestamp,
            "id_/https://dam.assets.ohio.gov/image/upload/ofcc.ohio.gov/Portals/0/Documents/",
            $path
        )
    };
}

pub(super) const PROJECTS: Connector = Connector {
    key: "ofcc-projects",
    publisher: "Ohio Facilities Construction Commission",
    feeds: &["program", "education-agency"],
    status: Status::Declared {
        blocked_on: "the site refuses a self-identifying agent — an identical 5,248-byte 404 \
                     to every path including robots.txt, 200 to a browser string — and its \
                     project data is rendered by interactive maps rather than served as files. \
                     The alternate publishers are surveyed and none allocates: the capital acts \
                     appropriate the programme as two lump lines, and the archive holds only \
                     the portal shell",
    },
    note: "The only source for the capital channel, which is invisible in every operating \
           per-pupil figure and was itself part of the DeRolph remedy.",
    sources: &[],
};

pub(super) const EQUITY_LISTS: Connector = Connector {
    key: "ofcc-equity-lists",
    publisher: "Ohio Facilities Construction Commission",
    feeds: &["parameter", "program"],
    status: Status::Wired {
        still_blocked: Some(
            "wired for the FY2024 and FY2025 K-12 lists by rank, recovered from the Internet \
             Archive. The FY2026 list has no capture at any path the CDX index knows; its ranks \
             survive only beside each district's status in the rolling CFAP edition, which is \
             committed as text and not parsed into rows. The Commission's 2024-2026 meeting \
             minutes, which state each approved project's state and local shares, are archived \
             nowhere, so no project share is held as data",
        ),
    },
    note: "The R.C. 3318.011 ranking the Classroom Facilities Assistance Program reads a \
           district's local share from. The Commission's host refuses this repository's agent; \
           the Wayback Machine is the publisher of record here. See \
           .yidam/decisions/ofcc-equity-lists-connector.yml.",
    sources: &[
        Source {
            key: "ofcc-equity-fy25",
            title: Some("Fiscal Year 2025 K-12 School District Equity List (by rank)"),
            url: ofcc_capture!(
                "20250628133113",
                "Resources/Treasurers/K-12-Equity-List-FY25-_by-rank.pdf"
            ),
            filename: "ofcc-equity-fy25.pdf",
            format: Format::Pdf,
            catalog: Some("ofcc-equity-list"),
            fixtures: &[crate::fixtures::OFCC_EQUITY_FIXTURE],
            note: "609 districts, ranks 1 to 609. The list that governs projects the controlling \
                   board approves in FY2026. One capture, 28 June 2025.",
        },
        Source {
            key: "ofcc-equity-fy24",
            title: Some("Fiscal Year 2024 K-12 District Equity List (by rank)"),
            url: ofcc_capture!(
                "20240701055948",
                "Resources/Ranking-Lists/2024/K-12%20District%20Equity%20List%20(by%20rank).pdf"
            ),
            filename: "ofcc-equity-fy24.pdf",
            format: Format::Pdf,
            catalog: Some("ofcc-equity-list"),
            fixtures: &[crate::fixtures::OFCC_EQUITY_FIXTURE],
            note: "609 districts. Captured twice, 1 July 2024 and 5 June 2025, under one digest.",
        },
        Source {
            key: "ofcc-cfap-status",
            title: Some("Classroom Facilities Assistance Program Status (as of December 31, 2025)"),
            url: ofcc_capture!(
                "20261017015835",
                "Our-Programs/K-12%20Schools/Programs/Classroom%20Facilities%20Assistance%20Program/\
                 Classroom-Facilities-Assistance-Program.pdf"
            ),
            filename: "ofcc-cfap-status.pdf",
            format: Format::Pdf,
            catalog: Some("ofcc-equity-list"),
            fixtures: &[crate::fixtures::CFAP_STATUS_FIXTURE],
            note: "A rolling URL the Commission overwrites each quarter; this capture holds the \
                   FY2026 Q2 edition and is the only one the archive has at this path. The \
                   dated editions before it were published under their own names.",
        },
    ],
};
