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

use super::{Connector, Status};

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
