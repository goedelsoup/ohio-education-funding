//! U.S. Bureau of Labor Statistics.
//!
//! One series, and every real-dollar figure in the corpus rests on it. Kept separate from the
//! Census connectors because a price index is not a survey.

use super::{Connector, Format, Source, Status};

pub(super) const CPI: Connector = Connector {
    key: "bls-cpi",
    publisher: "U.S. Bureau of Labor Statistics",
    feeds: &["metric", "fiscal-period"],
    status: Status::Wired {
        still_blocked: None,
    },
    note: "The smallest connector and the one without which nothing else here is honest: \
           H.B. 920 is only visible as a decline once a series is deflated.",
    sources: &[Source {
        key: "cpi-u-all-items",
        title: None,
        url: "https://download.bls.gov/pub/time.series/cu/cu.data.1.AllItems",
        filename: "cu.data.1.AllItems.tsv",
        format: Format::Tsv,
        catalog: Some("bls-cpi-u"),
        fixtures: &[crate::fixtures::CPI_FIXTURE],
        note: "Every CPI series in one flat file. CUUR0000SA0 period M06 is the June \
               all-items index this workspace deflates with.",
    }],
};
