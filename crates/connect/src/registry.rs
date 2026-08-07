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

/// How far a connector actually got.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Retrieved, parsed, and feeding a fixture that a test in this workspace reads.
    Wired,
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
            Self::Wired => "wired",
            Self::Parsed => "parsed",
            Self::Retrievable => "retrievable",
            Self::Declared { .. } => "declared",
        }
    }

    /// Whether the connector can produce structured records rather than only bytes.
    #[must_use]
    pub const fn has_parser(self) -> bool {
        matches!(self, Self::Wired | Self::Parsed)
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
    /// A web page. No parser here.
    Html,
    /// A PDF. No parser here.
    Pdf,
}

/// One retrievable publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Source {
    /// Stable key, used on the command line and in the digest manifest.
    pub key: &'static str,
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
    /// Repository-relative path of the committed fixture built from this source, if any.
    ///
    /// Making the source-to-fixture mapping explicit is what lets a test insist that anything
    /// a calculator reads can be traced back to a catalogued publication.
    pub fixture: Option<&'static str>,
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

/// Every approved connector.
pub const CONNECTORS: &[Connector] = &[
    Connector {
        key: "dew-foundation",
        publisher: "Ohio Department of Education and Workforce",
        feeds: &["education-agency", "revenue-stream", "metric", "program"],
        status: Status::Wired,
        note: "The spine of the numeric corpus: nearly every per-agency state aid figure \
               originates here.",
        sources: &[
            Source {
                key: "fy27-calculator",
                url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                      School-Payment-Reports/State-Funding-For-Schools/Traditional-School-Districts/\
                      FY27-TRAD-State-Foundation-Funding-Calculator_12-16-2025_lock-1.xlsx.aspx\
                      ?lang=en-US",
                filename: "fy27-calculator.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-fy27-funding-calculator"),
                fixture: Some(crate::fixtures::FY27_FIXTURE),
                note: "The department's own FY2027 model. A projection, not an actual. Read \
                       through its cached formula results.",
            },
            Source {
                key: "cupp-fy24",
                url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                      School-Payment-Reports/District-Profile-Reports/FY2024-District-Profile-Report/\
                      FY24-District-Profile-Report-Final-12-12-2024.xlsx.aspx?lang=en-US",
                filename: "cupp-fy24.xlsx",
                format: Format::Xlsx,
                catalog: Some("cupp-district-profile-report"),
                fixture: Some(crate::fixtures::PROFILE_FIXTURE),
                note: "60 variables per district. Fiscal and tax years are mixed within a row.",
            },
            Source {
                key: "enrollment-fy24",
                url: "https://education.ohio.gov/getattachment/Topics/Data/\
                      Frequently-Requested-Data/Enrollment-Data/oct_hdcnt_fy24.xls.aspx?lang=en-US",
                filename: "oct_hdcnt_fy24.xls",
                format: Format::LegacyXls,
                catalog: Some("dew-october-enrollment"),
                fixture: Some("crates/foundation/fixtures/fy24-district-grade-bands.csv"),
                note: "October headcount by individual grade. Still published in the pre-2007 \
                       format, which `spreadsheet` now reads natively; district data is on the \
                       third of seven sheets.",
            },
        ],
    },
    Connector {
        key: "dew-report-card",
        publisher: "Ohio Department of Education and Workforce",
        feeds: &["metric", "education-agency"],
        status: Status::Wired,
        note: "The only publisher of an Ohio district outcome measure, and — in the Expanded \
               List — of one expenditure numerator against two different pupil denominators.",
        sources: &[
            Source {
                key: "achievement-district-2425",
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      24-25_Achievement_District.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "achievement-district-2425.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-report-card-achievement"),
                fixture: Some(crate::fixtures::REPORT_CARD_FIXTURE),
                note: "Performance Index for 607 rated traditional districts, with the two \
                       prior years in the same row.",
            },
            Source {
                key: "spend-per-pupil-2425",
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      2425_DISTRICT_SPEND_PER_PUPIL.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "district-spend-per-pupil-2425.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-report-card-spending"),
                fixture: Some(crate::fixtures::REPORT_CARD_FIXTURE),
                note: "Expenditure per EQUIVALENT pupil. The denominator is weighted for \
                       disadvantage, so this is not a headcount average and is about 21% \
                       lower than one.",
            },
            Source {
                key: "va-district-details-2425",
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      2425_VA_DIST_DETAILS.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "va-district-details-2425.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-report-card-value-added"),
                fixture: Some(crate::fixtures::REPORT_CARD_FIXTURE),
                note: "The Progress component — growth rather than attainment. Use the effect \
                       size, not the composite index: the composite scales with student count.",
            },
            Source {
                key: "district-details-2425",
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      2025_District_Details.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "district-details-2425.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-report-card-district-details"),
                fixture: Some(crate::fixtures::REPORT_CARD_FIXTURE),
                note: "Subgroup enrollment shares in long form, one row per district per \
                       student group. The need covariates, same year as the outcomes. Its \
                       economic-disadvantage share is top-coded by community eligibility and \
                       is NOT the Cupp Report's measure.",
            },
            Source {
                key: "expanded-list-fy25",
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      FY25%20Expanded%20List.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "fy25-expanded-list.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-expenditure-expanded-list"),
                fixture: Some(crate::fixtures::REPORT_CARD_FIXTURE),
                note: "One operating-expenditure numerator on two sheets, divided by weighted \
                       ADM on one and unweighted ADM on the other. Covers community schools, \
                       JVSDs, STEM and eschools too; filter on org type.",
            },
        ],
    },
    Connector {
        key: "bls-cpi",
        publisher: "U.S. Bureau of Labor Statistics",
        feeds: &["metric", "fiscal-period"],
        status: Status::Wired,
        note: "The smallest connector and the one without which nothing else here is honest: \
               H.B. 920 is only visible as a decline once a series is deflated.",
        sources: &[Source {
            key: "cpi-u-all-items",
            url: "https://download.bls.gov/pub/time.series/cu/cu.data.1.AllItems",
            filename: "cu.data.1.AllItems.tsv",
            format: Format::Tsv,
            catalog: Some("bls-cpi-u"),
            fixture: Some(crate::fixtures::CPI_FIXTURE),
            note: "Every CPI series in one flat file. CUUR0000SA0 period M06 is the June \
                   all-items index this workspace deflates with.",
        }],
    },
    Connector {
        key: "tax-abstract",
        publisher: "Ohio Department of Taxation",
        feeds: &["revenue-stream", "parameter", "metric"],
        status: Status::Declared {
            blocked_on: "abstracts are published per tax year at unstable URLs, one workbook \
                         per table, with the district table's layout changing across years",
        },
        note: "Without this the local half of Ohio school funding is invisible, and the local \
               half is where the disparities live.",
        sources: &[],
    },
    Connector {
        key: "lsc-budget",
        publisher: "Ohio Legislative Service Commission",
        feeds: &["legislation", "fiscal-period", "program", "parameter"],
        status: Status::Declared {
            blocked_on: "redbooks and the Catalog of Budget Line Items are PDFs; the \
                         per-district simulations are workbooks posted per bill with no index",
        },
        note: "The only continuous appropriation-line series across the whole period, and the \
               primary source for the pre-2000 record.",
        sources: &[],
    },
    Connector {
        key: "ohio-laws",
        publisher: "Ohio General Assembly",
        feeds: &["legislation", "parameter", "formula-component"],
        status: Status::Declared {
            blocked_on: "codes.ohio.gov serves HTML with no bulk export; section history is \
                         rendered rather than published as data",
        },
        note: "Most `statutory_basis` fields in the corpus are `[open]` and waiting on exactly \
               this.",
        sources: &[],
    },
    Connector {
        key: "ohio-courts",
        publisher: "Supreme Court of Ohio",
        feeds: &["litigation"],
        status: Status::Declared {
            blocked_on: "opinions are PDFs, and trial-level rulings such as the 2025 EdChoice \
                         decision are not in the supreme court archive at all",
        },
        note: "`citing_cases` is what would make the precedent chain traversable rather than \
               hand-maintained.",
        sources: &[],
    },
    Connector {
        key: "ofcc-projects",
        publisher: "Ohio Facilities Construction Commission",
        feeds: &["program", "education-agency"],
        status: Status::Declared {
            blocked_on: "project records are behind a search form with no bulk export",
        },
        note: "The only source for the capital channel, which is invisible in every operating \
               per-pupil figure and was itself part of the DeRolph remedy.",
        sources: &[],
    },
    Connector {
        key: "census-f33",
        publisher: "U.S. Census Bureau",
        feeds: &["metric", "education-agency"],
        status: Status::Retrievable,
        note: "Comparability in two directions: whether Ohio is unusual, and an independent \
               check on department figures computed on different definitions.",
        sources: &[Source {
            key: "f33-fy2022",
            url: "https://www2.census.gov/programs-surveys/school-finances/tables/2022/\
                  secondary-education-finance/elsec22t.xls",
            filename: "elsec22t.xls",
            format: Format::LegacyXls,
            catalog: None,
            fixture: None,
            note: "One year per file, and the layout is not stable across years — a parser \
                   here has to be per-era, which is why this is not yet wired.",
        }],
    },
    Connector {
        key: "nces-ccd",
        publisher: "National Center for Education Statistics",
        feeds: &["education-agency"],
        status: Status::Declared {
            blocked_on: "agency files are per-year zips whose column sets change; the \
                         identifier-change series that justifies the connector is not published \
                         directly and must be derived",
        },
        note: "A corpus spanning 1851 to the present is a panel whose members change, and a \
               long series assembled without accounting for consolidation is silently wrong.",
        sources: &[],
    },
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
        // decisions/report-card-connector.yml. A connector dropping out of this list is a
        // decision, not an oversight, and should fail here first.
        let expected = [
            "dew-foundation",
            "tax-abstract",
            "lsc-budget",
            "ohio-laws",
            "ohio-courts",
            "ofcc-projects",
            "census-f33",
            "nces-ccd",
            "bls-cpi",
            "dew-report-card",
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
    fn a_declared_connector_says_what_blocks_it() {
        for connector in CONNECTORS {
            if let Status::Declared { blocked_on } = connector.status {
                assert!(
                    blocked_on.len() > 20,
                    "{} is declared without a reason",
                    connector.key
                );
            }
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
            let Some(fixture) = artifact.fixture else {
                continue;
            };
            assert!(
                root.join(fixture).exists(),
                "{} names a fixture that is not committed: {fixture}",
                artifact.key
            );
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
            if connector.status == Status::Wired {
                assert!(
                    connector.sources.iter().any(|s| s.fixture.is_some()),
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
