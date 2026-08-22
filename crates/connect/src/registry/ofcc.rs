//! Ohio Facilities Construction Commission.
//!
//! The capital channel, and the one connector blocked by a policy choice rather than a data
//! problem: the project data exists and is not served in a form this repository may take.

use super::{Connector, Status};

pub(super) const PROJECTS: Connector = Connector {
    key: "ofcc-projects",
    publisher: "Ohio Facilities Construction Commission",
    feeds: &["program", "education-agency"],
    status: Status::Declared {
        blocked_on: "the site refuses a self-identifying agent — 404 to this project's \
                     user-agent, 200 to a browser string — and its project data is rendered \
                     by interactive maps rather than served as files",
    },
    note: "The only source for the capital channel, which is invisible in every operating \
           per-pupil figure and was itself part of the DeRolph remedy.",
    sources: &[],
};
