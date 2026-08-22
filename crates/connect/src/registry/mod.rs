//! The connector registry: what this repository retrieves, from where, and how far each one
//! got.
//!
//! Nine connectors were approved in
//! [`decisions/proposals.yml`](../../../.yidam/decisions/proposals.yml). For a long time all
//! nine were directories holding a README describing an interface that did not exist. This
//! module replaces that with a registry that runs: each connector declares its sources and its
//! [`Status`], and the status is checkable rather than aspirational — a `Wired` connector has a
//! parser and a committed fixture, and a `Declared` one says what it is blocked on.
//!
//! The prose describing what each connector is *for* did not deserve to be thrown away with
//! the stubs; it lives in [`crates/connect/sources/`](../sources/) and is linked from here.
//!
//! # Status is a claim, and the CLI checks it
//!
//! `edfund-connect list` prints this table. `edfund-connect verify` recomputes the digest of
//! every cached source and compares it to the committed manifest. A connector claiming `Wired`
//! whose fixture cannot be rebuilt is a failing test, not a stale README.
//!
//! # One module per publisher
//!
//! The declarations are 2,570 lines of struct literals — 207 [`Source`]s across 21 [`Connector`]s
//! — and they stay Rust rather than becoming a data file, for three reasons worth recording so
//! the question is not reopened as a tidiness matter.
//!
//! `fixtures: &[crate::fixtures::FY27_FIXTURE]` is checked by the compiler against a real
//! constant, and the test below that every rebuilt fixture is claimed by a source leans on that
//! identity; a TSV would demote it to a runtime string match. With no dependencies, parsing 207
//! records whose `note` fields carry commas, quotes and Markdown means hand-writing a parser —
//! a new failure class in the one module whose job is to be trustworthy. And
//! `Status::Wired { still_blocked: Some(..) }` is a sum type, which a flat file is not.
//!
//! So they are grouped by publisher instead, one module each, and composed back into
//! [`CONNECTORS`] below. Const items compose in an array literal at no runtime cost and with no
//! macro, and the order of that array is now visible in twenty-one lines rather than inferred
//! from 2,570.

mod assembly;
mod auditor;
mod bls;
mod census;
mod courts;
mod dew;
mod lsc;
mod nces;
mod ofcc;
mod tax;

pub use assembly::OHIO_LAWS_SECTIONS;

/// How far a connector actually got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Retrieved, parsed, and feeding a fixture that a test in this workspace reads.
    Wired {
        /// What the connector still cannot reach, when it is wired for only part of what its
        /// `feeds` claim.
        ///
        /// `None` is the ordinary case. `Some` exists because a connector can be genuinely wired
        /// and genuinely incomplete at once — `ohio-courts` retrieves the four DeRolph opinions
        /// and cannot retrieve a common pleas ruling, and a bare `Wired` would leave that second
        /// half recorded only in prose that no test reads.
        still_blocked: Option<&'static str>,
    },
    /// Retrieved and parsed, but nothing downstream consumes it yet.
    Parsed,
    /// A URL is known and the bytes can be fetched; no parser exists.
    Retrievable,
    /// Approved in the ontology, with no machine-readable endpoint identified.
    Declared {
        /// What stands between this and a parser.
        blocked_on: &'static str,
    },
}

impl Status {
    /// A short label for the CLI table.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Wired { .. } => "wired",
            Self::Parsed => "parsed",
            Self::Retrievable => "retrievable",
            Self::Declared { .. } => "declared",
        }
    }

    /// Whether the connector can produce structured records rather than only bytes.
    #[must_use]
    pub const fn has_parser(self) -> bool {
        matches!(self, Self::Wired { .. } | Self::Parsed)
    }

    /// Whether the connector feeds a fixture, whatever else it still cannot reach.
    #[must_use]
    pub const fn is_wired(self) -> bool {
        matches!(self, Self::Wired { .. })
    }
}

/// What a retrieved file is, which decides what can read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// An Office Open XML workbook. [`spreadsheet`] reads these.
    Xlsx,
    /// A pre-2007 OLE2 workbook. A different format entirely, read natively by
    /// [`spreadsheet::ole2`] and [`spreadsheet::biff`].
    LegacyXls,
    /// A tab-separated flat file, as the Bureau of Labor Statistics publishes.
    Tsv,
    /// Flat text that is not delimited: a fixed-width extract, or a report laid out for a printer
    /// and posted as the file. Read by offset, so its column positions are part of the parser.
    Text,
    /// A web page. No parser here.
    Html,
    /// A PDF. No parser here.
    Pdf,
    /// A zip of delimited text. The Census and NCES geography files come this way, and each
    /// archive holds several files of which this repository reads one or two.
    Zip,
}

/// One retrievable publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Source {
    /// Stable key, used on the command line and in the digest manifest.
    pub key: &'static str,
    /// What to call the source in a fixture built from it, when a fixture names it at all.
    ///
    /// Only sources whose records carry a title need one; `None` everywhere else. It exists
    /// because the DeRolph records took their title from the first sentence of [`Self::note`],
    /// which made a prose comment load-bearing input to committed data — rewording the note
    /// silently rewrote the fixture, and splitting the sentence on `.` cut
    /// `93 Ohio St.3d 309` down to `93 Ohio St`. A title a fixture prints is data, so it is
    /// declared rather than parsed out of a comment.
    pub title: Option<&'static str>,
    /// Where to fetch it.
    pub url: &'static str,
    /// What to call it in the cache.
    pub filename: &'static str,
    /// What kind of file it is.
    pub format: Format,
    /// Slug of the [`.yidam/catalog/`](../../../.yidam/catalog/) node describing what this
    /// source is and what it can be trusted for.
    ///
    /// Provenance attaches to the artifact, not to the connector: `dew-foundation` retrieves
    /// three publications with three different sets of caveats.
    pub catalog: Option<&'static str>,
    /// Repository-relative paths of the committed fixtures built from this source.
    ///
    /// Making the source-to-fixture mapping explicit is what lets a test insist that anything a
    /// calculator reads can be traced back to a catalogued publication.
    ///
    /// A list rather than an `Option`, because a publication can feed more than one fixture and
    /// three of them do: `cupp-fy24`'s district data goes into the FY2027 model, its own extract
    /// and the grade bands; `expanded-list-fy25` has a sheet each for the report card and the
    /// expenditure functions; `ccd-lea-directory-2223` supplies the IRN join to both the F-33
    /// district panel and the legislative crosswalk. While this held one path each of those
    /// declared whichever fixture was written first, and the rest of what they feed was not
    /// recorded anywhere — `expenditure-functions-fy25.csv` had no declared source at all.
    pub fixtures: &'static [&'static str],
    /// What it is and what to watch out for.
    pub note: &'static str,
}

/// One approved connector: a feed with a publisher, a catalog anchor, and zero or more sources.
#[derive(Debug, Clone, Copy)]
pub struct Connector {
    /// Stable key, matching the directory these once lived in.
    pub key: &'static str,
    /// Who publishes it.
    pub publisher: &'static str,
    /// Corpus classes this feeds.
    pub feeds: &'static [&'static str],
    /// How far it got.
    pub status: Status,
    /// Its retrievable artifacts.
    pub sources: &'static [Source],
    /// Why it matters, in one line. The long form is in `crates/connect/sources/<key>.md`.
    pub note: &'static str,
}

/// Every approved connector, in the order `edfund-connect list` prints them.
///
/// The order is the corpus's reading order rather than any publisher's, which is why the
/// modules below are not simply concatenated: the department's own calculator first, and the
/// connectors that answer "why is this number what it is" after the ones that supply it.
pub const CONNECTORS: &[Connector] = &[
    dew::FOUNDATION,
    dew::REPORT_CARD,
    bls::CPI,
    dew::FIVE_YEAR_FORECAST,
    tax::ABSTRACT,
    tax::CASINO,
    dew::PAYMENT_REPORTS,
    dew::SCHOLARSHIP_REPORTS,
    lsc::CATALOG,
    lsc::BUDGET,
    assembly::LAWS,
    assembly::SESSION_LAWS,
    assembly::BILLS,
    auditor::REPORTS,
    courts::OPINIONS,
    ofcc::PROJECTS,
    census::F33,
    nces::CCD,
    census::GEOGRAPHY,
    dew::CHILD_NUTRITION,
    dew::SCHOOL_IMPROVEMENT,
];

/// Look up a connector by key.
#[must_use]
pub fn connector(key: &str) -> Option<&'static Connector> {
    CONNECTORS.iter().find(|c| c.key == key)
}

/// Look up a source by key, across every connector.
#[must_use]
pub fn source(key: &str) -> Option<(&'static Connector, &'static Source)> {
    CONNECTORS
        .iter()
        .find_map(|c| c.sources.iter().find(|s| s.key == key).map(|s| (c, s)))
}

/// Every source in the registry, in connector order.
pub fn sources() -> impl Iterator<Item = (&'static Connector, &'static Source)> {
    CONNECTORS
        .iter()
        .flat_map(|c| c.sources.iter().map(move |s| (c, s)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_connector_approved_in_the_ontology_is_present() {
        // The nine from decisions/proposals.yml, plus dew-report-card from
        // decisions/report-card-connector.yml, dew-five-year-forecast from
        // decisions/five-year-forecast-connector.yml, and dew-payment-reports from
        // decisions/payment-reports-connector.yml, and census-geography from
        // decisions/census-geography-connector.yml. A connector dropping out of this list is a
        // decision, not an oversight, and should fail here first — as should one appearing in the
        // registry without a decision record behind it, which is what caught the twelfth.
        let expected = [
            "dew-foundation",
            "tax-abstract",
            "lsc-budget",
            "ohio-laws",
            "ohio-session-laws",
            "ohio-auditor",
            "ohio-courts",
            "ofcc-projects",
            "census-f33",
            "nces-ccd",
            "bls-cpi",
            "dew-report-card",
            "dew-five-year-forecast",
            "dew-payment-reports",
            "census-geography",
            "dew-child-nutrition",
            "dew-school-improvement",
            "dew-scholarship-reports",
            "lsc-catalog",
            // ohio-bills from decisions/drafts-are-not-legislation.yml, which added the
            // `draft-legislation` class and needed a third artefact from a publisher already here:
            // `ohio-laws` serves the Revised Code as it stands and `ohio-session-laws` serves acts
            // as passed, and a bill that has not been enacted is neither.
            "ohio-bills",
            // tax-casino from decisions/the-channel-with-no-line.yml. A second connector on a
            // publisher already here for the same reason as `ohio-bills`: `tax-abstract` retrieves
            // what a district may levy, and this retrieves what the state hands it outside any
            // formula. They share a publisher and nothing else — different division, different
            // directory, different blocker.
            "tax-casino",
        ];
        for key in expected {
            assert!(
                connector(key).is_some(),
                "{key} is missing from the registry"
            );
        }
        assert_eq!(CONNECTORS.len(), expected.len());
    }

    #[test]
    fn keys_are_unique_across_connectors_and_sources() {
        let mut seen = Vec::new();
        for connector in CONNECTORS {
            assert!(
                !seen.contains(&connector.key),
                "duplicate {}",
                connector.key
            );
            seen.push(connector.key);
        }
        let mut source_keys = Vec::new();
        for (_, artifact) in sources() {
            assert!(
                !source_keys.contains(&artifact.key),
                "duplicate source {}",
                artifact.key
            );
            source_keys.push(artifact.key);
        }
    }

    #[test]
    fn a_connector_claiming_a_parser_has_something_to_parse() {
        for connector in CONNECTORS {
            if connector.status.has_parser() {
                assert!(
                    !connector.sources.is_empty(),
                    "{} claims {} with no sources",
                    connector.key,
                    connector.status.label()
                );
            }
        }
    }

    #[test]
    fn a_connector_that_is_blocked_says_what_blocks_it() {
        // Both halves of the claim, not just the declared one. A connector wired for part of what
        // it feeds records the rest in `still_blocked`, and that string is held to the same
        // standard as a declared connector's — the whole point of the field is that a surviving
        // blocker stays machine-readable instead of decaying into prose.
        for connector in CONNECTORS {
            let (kind, reason) = match connector.status {
                Status::Declared { blocked_on } => ("declared", Some(blocked_on)),
                Status::Wired { still_blocked } => ("wired", still_blocked),
                _ => continue,
            };
            let Some(reason) = reason else {
                continue;
            };
            assert!(
                reason.len() > 20,
                "{} is {kind} without a reason",
                connector.key
            );
        }
    }

    #[test]
    fn every_fixture_a_source_declares_is_regenerated_or_says_why_not() {
        // The other half of provenance. The catalog test below asks whether a fixture can be
        // traced back to a catalogued publication; this asks whether it can be *rebuilt* from
        // one. Those are different questions, and the F-33 district panel answered the first and
        // not the second for as long as it existed: 754 KB committed, read by
        // `dispersion::national_peers`, declared here, and produced by nothing. A digest pins the
        // bytes that went in and cannot notice that the derivation from them was never written.
        //
        // Static rather than a rebuild, because CI has no `.cache/` and running the real thing
        // would write into the working tree.
        use crate::fixtures::{NOT_REGENERATED, REBUILT};

        for (_, source) in sources() {
            for fixture in source.fixtures {
                if REBUILT.contains(fixture) {
                    continue;
                }
                let excused = NOT_REGENERATED.iter().find(|(path, _)| path == fixture);
                let (_, why) = excused.unwrap_or_else(|| {
                    panic!(
                        "{} declares {fixture}, which nothing in `rebuild` produces. Write the \
                         extractor, or record the gap in `fixtures::NOT_REGENERATED` with what it \
                         would take to close it.",
                        source.key
                    )
                });
                assert!(
                    why.len() > 30,
                    "{fixture} is excused from rebuilding without saying what is missing"
                );
            }
        }

        // And the other direction, which is where `expenditure-functions-fy25.csv` hid. It was
        // rebuilt on every run from a sheet of `expanded-list-fy25`, and no source said so,
        // because `fixture` held one path and that source's other sheet had already claimed it.
        // A fixture nobody declares has no catalog anchor and no digest behind it — the provenance
        // rule simply does not reach it, and the test that enforces the rule walks sources, so it
        // does not reach it either.
        for fixture in REBUILT {
            assert!(
                sources().any(|(_, s)| s.fixtures.contains(fixture)),
                "`rebuild` writes {fixture} but no source declares it, so nothing ties it to a \
                 catalogued publication. Add it to the `fixtures` of whichever source it is \
                 built from."
            );
        }

        // And the excuse list cannot outlive the work. An entry that has since been given an
        // extractor has to leave, or the next reader is told a gap exists that does not.
        for (path, _) in NOT_REGENERATED {
            assert!(
                !REBUILT.contains(path),
                "{path} is in NOT_REGENERATED but `rebuild` produces it — delete the entry"
            );
            assert!(
                sources().any(|(_, s)| s.fixtures.contains(path)),
                "{path} is in NOT_REGENERATED but no source declares it"
            );
        }
    }

    #[test]
    fn urls_are_absolute_and_carry_no_stray_whitespace() {
        // Multi-line string literals in this file continue with a backslash; a missed one
        // would leave the URL with embedded spaces and a confusing fetch failure.
        for (_, source) in sources() {
            assert!(source.url.starts_with("https://"), "{}", source.key);
            assert!(
                !source.url.contains(char::is_whitespace),
                "{} has whitespace in its URL",
                source.key
            );
        }
    }

    #[test]
    fn a_source_a_fixture_is_built_from_has_a_catalog_anchor_that_exists() {
        // The provenance rule, made checkable: anything a calculator reads must be traceable
        // to a catalogued publication. A fixture whose origin is only in a commit message is
        // the failure mode this prevents.
        let root = crate::cache::repository_root();
        for (connector, artifact) in sources() {
            for fixture in artifact.fixtures {
                assert!(
                    root.join(fixture).exists(),
                    "{} names a fixture that is not committed: {fixture}",
                    artifact.key
                );
            }
            let Some(fixture) = artifact.fixtures.first() else {
                continue;
            };
            let catalog = artifact.catalog.unwrap_or_else(|| {
                panic!(
                    "{} ({}) feeds {fixture} with no catalog anchor",
                    artifact.key, connector.key
                )
            });
            let node = root.join(".yidam/catalog").join(format!("{catalog}.md"));
            assert!(
                node.exists(),
                "{} points at a catalog node that does not exist: {}",
                artifact.key,
                node.display()
            );
        }
    }

    #[test]
    fn a_wired_connector_feeds_at_least_one_fixture() {
        for connector in CONNECTORS {
            if connector.status.is_wired() {
                assert!(
                    connector.sources.iter().any(|s| !s.fixtures.is_empty()),
                    "{} claims wired but feeds nothing",
                    connector.key
                );
            }
        }
    }

    #[test]
    fn sources_are_findable_by_key() {
        let (owner, artifact) = source("fy27-calculator").expect("registered");
        assert_eq!(owner.key, "dew-foundation");
        assert_eq!(artifact.format, Format::Xlsx);
        assert!(source("no-such-source").is_none());
    }
}
