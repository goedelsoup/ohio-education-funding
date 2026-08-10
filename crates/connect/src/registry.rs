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
    /// A web page. No parser here.
    Html,
    /// A PDF. No parser here.
    Pdf,
    /// A zip of delimited text. The Census and NCES geography files come this way, and each
    /// archive holds several files of which this repository reads one or two.
    Zip,
}

/// The Revised Code sections the corpus cites, one source each.
///
/// Named rather than crawled. A crawler over Chapter 3317 would pull three hundred sections the
/// corpus has no use for and would make the digest manifest churn on every unrelated amendment;
/// this list is exactly what some node's `statutory_basis` points at, and it grows when a node
/// starts pointing somewhere new.
pub const OHIO_LAWS_SECTIONS: &[Source] = &[
    Source {
        key: "rc-3317-02",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.02",
        filename: "rc-3317-02.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.02. Definitions, including the economically disadvantaged index and its \
               squaring — which the DPIA node recorded as not located in statute.",
    },
    Source {
        key: "rc-3317-011",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.011",
        filename: "rc-3317-011.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.011. Base cost and its components, the most-cited section in the corpus.",
    },
    Source {
        key: "rc-3317-013",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.013",
        filename: "rc-3317-013.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.013. Special education weights, and the clinical categories the corpus \
               could not name.",
    },
    Source {
        key: "rc-3317-014",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.014",
        filename: "rc-3317-014.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.014. Career-technical weights, and the programme categories behind their \
               ordering.",
    },
    Source {
        key: "rc-3317-016",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.016",
        filename: "rc-3317-016.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.016. English learner weights, and the rule the taper actually expresses.",
    },
    Source {
        key: "rc-3317-017",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.017",
        filename: "rc-3317-017.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.017. Local capacity and the state share percentage.",
    },
    Source {
        key: "rc-3317-019",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.019",
        filename: "rc-3317-019.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.019. Temporary transitional aid — the guarantee. Labelled \"gifted units \
               and their prices\" when this list was first written, from the corpus's own citation \
               rather than from the section; gifted units are R.C. 3317.051.",
    },
    Source {
        key: "rc-3317-022",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.022",
        filename: "rc-3317-022.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note:
            "R.C. 3317.022. Core foundation funding: the section that assembles every component, \
               and where disadvantaged pupil impact aid actually lives.",
    },
    Source {
        key: "rc-3317-051",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.051",
        filename: "rc-3317-051.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.051. Gifted unit funding and the three salary prices the units are \
               bought at, which R.C. 3317.022 reaches by cross-reference.",
    },
    Source {
        key: "rc-3317-0212",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.0212",
        filename: "rc-3317-0212.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.0212. Transportation.",
    },
    Source {
        key: "rc-3317-0213",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.0213",
        filename: "rc-3317-0213.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.0213. Preschool special education.",
    },
    Source {
        key: "rc-3317-0217",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.0217",
        filename: "rc-3317-0217.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.0217. The temporary transitional aid guarantee.",
    },
    Source {
        key: "rc-3317-03",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.03",
        filename: "rc-3317-03.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3317.03. What each reported count means, including the economically \
               disadvantaged certification the department is left to define.",
    },
    Source {
        key: "rc-319-301",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-319.301",
        filename: "rc-319-301.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 319.301. H.B. 920 tax reduction factors and the twenty-mill floor.",
    },
    Source {
        key: "rc-5705-391",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-5705.391",
        filename: "rc-5705-391.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 5705.391. The five-year forecast requirement.",
    },
    Source {
        key: "rc-3770-06",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3770.06",
        filename: "rc-3770-06.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3770.06. The state lottery funds, including the lottery profits education \
               fund the 1987 constitutional amendment earmarks. The fund the corpus needed to \
               name in order to say that lottery profits are combined with the GRF to pay \
               foundation aid rather than added to it.",
    },
    Source {
        key: "rc-5753-02",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-5753.02",
        filename: "rc-5753-02.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 5753.02. The thirty-three per cent tax on gross casino revenue, which is the \
               thing the school distribution is a share of.",
    },
    Source {
        key: "rc-5753-03",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-5753.03",
        filename: "rc-5753-03.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 5753.03. The eleven funds the casino tax is split into, and the quarterly \
               transfer of thirty-four per cent to the county student fund. This is where it \
               becomes visible that the school share never passes through the department's \
               budget.",
    },
    Source {
        key: "rc-5753-11",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-5753.11",
        filename: "rc-5753-11.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 5753.11. The student population count the distribution is apportioned on — a \
               fifth pupil denominator, on two October and May count dates, including community \
               and STEM schools and double-counting JVSD dual enrolment on purpose.",
    },
];

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

/// Every approved connector.
pub const CONNECTORS: &[Connector] = &[
    Connector {
        key: "dew-foundation",
        publisher: "Ohio Department of Education and Workforce",
        feeds: &["education-agency", "revenue-stream", "metric", "program"],
        status: Status::Wired {
            still_blocked: None,
        },
        note: "The spine of the numeric corpus: nearly every per-agency state aid figure \
               originates here.",
        sources: &[
            Source {
                key: "fy27-calculator",
                title: None,
                url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                      School-Payment-Reports/State-Funding-For-Schools/\
                      Traditional-School-Districts/\
                      FY27-TRAD-State-Foundation-Funding-Calculator_12-16-2025_lock-1.xlsx.aspx\
                      ?lang=en-US",
                filename: "fy27-calculator.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-fy27-funding-calculator"),
                fixtures: &[crate::fixtures::FY27_FIXTURE],
                note: "The department's own FY2027 model. A projection, not an actual. Read \
                       through its cached formula results.",
            },
            Source {
                key: "cupp-fy24",
                title: None,
                url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                      School-Payment-Reports/District-Profile-Reports/\
                      FY2024-District-Profile-Report/\
                      FY24-District-Profile-Report-Final-12-12-2024.xlsx.aspx?lang=en-US",
                filename: "cupp-fy24.xlsx",
                format: Format::Xlsx,
                catalog: Some("cupp-district-profile-report"),
                fixtures: &[
                    crate::fixtures::FY27_FIXTURE,
                    crate::fixtures::PROFILE_FIXTURE,
                    crate::fixtures::GRADE_BANDS_FIXTURE,
                ],
                note: "60 variables per district. Fiscal and tax years are mixed within a row.",
            },
            Source {
                key: "enrollment-fy24",
                title: None,
                url: "https://education.ohio.gov/getattachment/Topics/Data/\
                      Frequently-Requested-Data/Enrollment-Data/oct_hdcnt_fy24.xls.aspx?lang=en-US",
                filename: "oct_hdcnt_fy24.xls",
                format: Format::LegacyXls,
                catalog: Some("dew-october-enrollment"),
                fixtures: &[crate::fixtures::GRADE_BANDS_FIXTURE],
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
        status: Status::Wired {
            still_blocked: None,
        },
        note: "The only publisher of an Ohio district outcome measure, and — in the Expanded \
               List — of one expenditure numerator against two different pupil denominators.",
        sources: &[
            Source {
                key: "achievement-district-2425",
                title: None,
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      24-25_Achievement_District.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "achievement-district-2425.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-report-card-achievement"),
                fixtures: &[crate::fixtures::REPORT_CARD_FIXTURE],
                note: "Performance Index for 607 rated traditional districts, with the two \
                       prior years in the same row.",
            },
            Source {
                key: "spend-per-pupil-2425",
                title: None,
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      2425_DISTRICT_SPEND_PER_PUPIL.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "district-spend-per-pupil-2425.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-report-card-spending"),
                fixtures: &[crate::fixtures::REPORT_CARD_FIXTURE],
                note: "Expenditure per EQUIVALENT pupil. The denominator is weighted for \
                       disadvantage, so this is not a headcount average and is about 21% \
                       lower than one.",
            },
            Source {
                key: "va-district-details-2425",
                title: None,
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      2425_VA_DIST_DETAILS.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "va-district-details-2425.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-report-card-value-added"),
                fixtures: &[crate::fixtures::REPORT_CARD_FIXTURE],
                note: "The Progress component — growth rather than attainment. Use the effect \
                       size, not the composite index: the composite scales with student count.",
            },
            Source {
                key: "district-details-2425",
                title: None,
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      2025_District_Details.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "district-details-2425.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-report-card-district-details"),
                fixtures: &[crate::fixtures::REPORT_CARD_FIXTURE],
                note: "Subgroup enrollment shares in long form, one row per district per \
                       student group. The need covariates, same year as the outcomes. Its \
                       economic-disadvantage share is top-coded by community eligibility and \
                       is NOT the Cupp Report's measure.",
            },
            Source {
                key: "expanded-list-fy25",
                title: None,
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      FY25%20Expanded%20List.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "fy25-expanded-list.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-expenditure-expanded-list"),
                fixtures: &[
                    crate::fixtures::REPORT_CARD_FIXTURE,
                    crate::fixtures::FUNCTIONS_FIXTURE,
                ],
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
    },
    Connector {
        key: "dew-five-year-forecast",
        publisher: "Ohio Department of Education and Workforce",
        feeds: &["education-agency", "revenue-stream", "metric", "fiscal-period"],
        status: Status::Wired {
            still_blocked: None,
        },
        note: "The only per-district record here of money that changed hands rather than money \
               a formula computed — and the only one carrying what a district holds.",
        sources: &[
            Source {
                key: "five-year-forecast-fy23",
                title: None,
                url: "https://public.education.ohio.gov/School%20District%20Five-year%20Forecasts/\
                      FY23_5YR_Forecast_Required_Spring%20Update%20Submissions.txt",
                filename: "five-year-forecast-fy23.txt",
                format: Format::Tsv,
                catalog: Some("dew-five-year-forecast"),
                fixtures: &[crate::fixtures::FINANCE_FIXTURE],
                note: "Actuals for FY2020, FY2021 and FY2022. FY2020 is the year the temporary \
                       transitional aid guarantee holds districts at, which this corpus has \
                       until now only been able to infer.",
            },
            Source {
                key: "five-year-forecast-fy26",
                title: None,
                url: "https://public.education.ohio.gov/School%20District%20Five-year%20Forecasts/\
                      FY26_Financial_Forecast_Required_Spring_Update_Submissions.txt",
                filename: "five-year-forecast-fy26.txt",
                format: Format::Tsv,
                catalog: Some("dew-five-year-forecast"),
                fixtures: &[crate::fixtures::FINANCE_FIXTURE],
                note: "Actuals for FY2023, FY2024 and FY2025. Picks up exactly where the FY2023 \
                       filing's actuals stop, and the two must agree about the cash balance at \
                       the instant they meet.",
            },
        ],
    },
    Connector {
        key: "tax-abstract",
        publisher: "Ohio Department of Taxation",
        feeds: &["revenue-stream", "parameter", "metric"],
        status: Status::Wired {
            still_blocked: None,
        },
        note: "Without this the local half of Ohio school funding is invisible, and the local \
               half is where the disparities live.",
        sources: &[
            Source {
                key: "sd1-ty2021",
                title: None,
                url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/tax_analysis/\
                      tax_data_series/school_district_data/sd1/SD1CY21.xlsx",
                filename: "sd1-ty2021.xlsx",
                format: Format::Xlsx,
                catalog: Some("dot-sd1-school-district-taxes"),
                fixtures: &[crate::fixtures::SD1_FIXTURE],
                note: "The oldest of the four. Two tax years give a level and a change; four give \
                       each district a reappraisal and two quiet years to measure it against, \
                       which is what makes recognized valuation reconstructible. See \
                       `regime_diff::recognized_valuation`.",
            },
            Source {
                key: "sd1-ty2022",
                title: None,
                url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/tax_analysis/\
                      tax_data_series/school_district_data/sd1/SD1CY22.xlsx",
                filename: "sd1-ty2022.xlsx",
                format: Format::Xlsx,
                catalog: Some("dot-sd1-school-district-taxes"),
                fixtures: &[crate::fixtures::SD1_FIXTURE],
                note: "Ohio's counties reappraise or update on a staggered three-year cycle, so \
                       TY2022 through TY2024 contains exactly one valuation event for every one \
                       of the 88. This year completes that window.",
            },
            Source {
                key: "sd1-ty2023",
                title: None,
                url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/tax_analysis/\
                      tax_data_series/school_district_data/sd1/SD1CY23.xlsx",
                filename: "sd1-ty2023.xlsx",
                format: Format::Xlsx,
                catalog: Some("dot-sd1-school-district-taxes"),
                fixtures: &[crate::fixtures::SD1_FIXTURE],
                note: "Carried so that a change in taxes charged can be separated from the level. \
                       Its worksheets are named `ExJVS` and `SD1DATWK23`, against `ExJVS24` and \
                       `SD1DAT24` a year later — the layout drift this connector was blocked on, \
                       now handled by prefix. TY2021 and TY2022 use `SD1DATWK21`/`SD1DATWK22` and \
                       a bare `ExJVS`, which the same prefixes already reach.",
            },
            Source {
                key: "sd1-ty2024",
                title: None,
                url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/tax_analysis/\
                      tax_data_series/school_district_data/sd1/SD1CY24.xlsx",
                filename: "sd1-ty2024.xlsx",
                format: Format::Xlsx,
                catalog: Some("dot-sd1-school-district-taxes"),
                fixtures: &[crate::fixtures::SD1_FIXTURE],
                note: "Taxable value by class and real property taxes charged for current \
                       expenses, per district, from the DTE-13/DTE-14 abstracts. Taxes charged \
                       is a levy, not a receipt, and it is gross of the rollback, owner-\
                       occupancy and homestead credits the state reimburses.",
            },
        ],
    },
    Connector {
        key: "dew-payment-reports",
        publisher: "Ohio Department of Education and Workforce",
        feeds: &["program", "education-agency", "revenue-stream"],
        status: Status::Declared {
            blocked_on: "the deduct-era reports (1999-2021) are behind OH|ID authentication on \
                         the department's reports portal; the current-era ones are open and \
                         indexed but post-date the deduction entirely",
        },
        note: "The one source that would carry the voucher and community-school deduction per \
               resident district, for the years it existed.",
        sources: &[],
    },
    Connector {
        key: "lsc-budget",
        publisher: "Ohio Legislative Service Commission",
        feeds: &["legislation", "fiscal-period", "program", "parameter"],
        status: Status::Wired {
            still_blocked: Some(
                "wired for one document, the final analysis of the current budget act; the \
                 redbooks, the Catalog of Budget Line Items and the per-district simulations are \
                 unretrieved, so the continuous appropriation-line series and the pre-2000 record \
                 are not built",
            ),
        },
        note: "The only continuous appropriation-line series across the whole period, and the \
               primary source for the pre-2000 record.",
        sources: &[
        Source {
            key: "hb96-edu-redbook",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/136/hb96/in/files/\
                  hb96-edu-redbook-as-introduced-136th-general-assembly.pdf",
            filename: "hb96-edu-redbook.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-dew-redbook"),
            fixtures: &[crate::fixtures::REDBOOK_FIXTURE],
            note: "The department's appropriation line items, with each ALI's earmarks and their \
                   amounts. This is what answers \"which budget line governs\" for a program that \
                   prorates. **As introduced** — LSC publishes redbooks only for the introduced \
                   bill — so the line item *numbers* are the enacted ones and the *amounts* are \
                   the executive proposal. The distinction is stated wherever a figure from here \
                   is quoted.",
        },
        Source {
            key: "hb96-final-analysis",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/136/hb96/en0/files/\
                  hb96-bill-analysis-as-enacted-136th-general-assembly.pdf",
            filename: "hb96-final-analysis.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-hb96-analysis"),
            fixtures: &[crate::fixtures::ENACTED_FIXTURE],
            note: "**As enacted**, and the distinction is not pedantic. The as-passed-by-the-House \
                   analysis sits at a sibling URL and gives the base funding supplement as $20 and \
                   $30 and the enrolment growth supplement as a tiered schedule — neither of which \
                   became law. Reading the convenient version would have contradicted the \
                   department's own payments and looked like a finding.",
        },
        ],
    },
    Connector {
        key: "ohio-laws",
        publisher: "Ohio General Assembly",
        feeds: &["legislation", "parameter", "formula-component"],
        status: Status::Wired {
            still_blocked: None,
        },
        note: "Most `statutory_basis` fields in the corpus were `[open]` and waiting on exactly \
               this. The recorded blocker — \"serves HTML with no bulk export\" — was a statement \
               about the absence of a convenience, and was read as one about the absence of the \
               data. Every section below is server-rendered: the text is in the response body. \
               What remains genuinely unavailable as data is *section history*, which the site \
               renders per version; the corpus takes the current text and its effective date and \
               does not attempt a series.",
        sources: OHIO_LAWS_SECTIONS,
    },
    Connector {
        key: "ohio-courts",
        publisher: "Supreme Court of Ohio",
        feeds: &["litigation"],
        status: Status::Wired {
            still_blocked: Some(
                "trial-level rulings such as the 2025 EdChoice decision are not in the supreme \
                 court archive at all, and `citing_cases` needs a citator rather than a document",
            ),
        },
        note: "Wired for the four DeRolph opinions, which is what the corpus actually cites. The \
               recorded blocker had two clauses and they are not equally true: \"opinions are \
               PDFs\" stopped being one the moment `Format::Pdf` had a reader, and the other half \
               is unfixable from here and is now carried in `still_blocked` rather than in this \
               sentence.",
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
        ],
    },
    Connector {
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
    },
    Connector {
        key: "census-f33",
        publisher: "U.S. Census Bureau",
        feeds: &["metric", "education-agency"],
        status: Status::Wired {
            still_blocked: None,
        },
        note: "Comparability in two directions: whether Ohio is unusual, and an independent \
               check on department figures computed on different definitions. Both are now \
               held — the state aggregate and, through NCES's keying of the same survey, the \
               per-district panel.",
        sources: &[Source {
            key: "f33-fy2022",
            title: None,
            url: "https://www2.census.gov/programs-surveys/school-finances/tables/2022/\
                  secondary-education-finance/elsec22t.xls",
            filename: "elsec22t.xls",
            format: Format::LegacyXls,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_FIXTURE],
            note: "One year per file and the layout is not stable across years, so the column \
                   map is per-era. The fixture is the state aggregate rather than the panel: \
                   14,106 school systems reduce to 51 rows. The per-district view is now held \
                   too, from NCES's own keying of the same survey — see `sdf22-districts` \
                   below, which reproduces this file's Ohio local share to a tenth of a point.",
        },
        Source {
            key: "sdf12-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf121a.zip",
            filename: "sdf121a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2012. One of the ten years behind the Ohio \
                   panel; FY2014 is absent from the archive under every naming the others use.",
        },
        Source {
            key: "sdf13-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf13_1a.zip",
            filename: "sdf13_1a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2013. One of the ten years behind the Ohio \
                   panel; FY2014 is absent from the archive under every naming the others use.",
        },
        Source {
            key: "sdf15-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf15_1a.zip",
            filename: "sdf15_1a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2015. One of the ten years behind the Ohio \
                   panel; FY2014 is absent from the archive under every naming the others use.",
        },
        Source {
            key: "sdf16-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf16_1a.zip",
            filename: "sdf16_1a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2016. One of the ten years behind the Ohio \
                   panel; FY2014 is absent from the archive under every naming the others use.",
        },
        Source {
            key: "sdf17-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf17_1a.zip",
            filename: "sdf17_1a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2017. One of the ten years behind the Ohio \
                   panel; FY2014 is absent from the archive under every naming the others use.",
        },
        Source {
            key: "sdf18-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf18_1a.zip",
            filename: "sdf18_1a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2018. One of the ten years behind the Ohio \
                   panel; FY2014 is absent from the archive under every naming the others use.",
        },
        Source {
            key: "sdf19-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf19_1a.zip",
            filename: "sdf19_1a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2019. One of the ten years behind the Ohio \
                   panel; FY2014 is absent from the archive under every naming the others use.",
        },
        Source {
            key: "sdf20-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf20_1a.zip",
            filename: "sdf20_1a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2020. One of the ten years behind the Ohio \
                   panel; FY2014 is absent from the archive under every naming the others use.",
        },
        Source {
            key: "sdf21-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf21_1a.zip",
            filename: "sdf21_1a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2021. One of the ten years behind the Ohio \
                   panel; FY2014 is absent from the archive under every naming the others use.",
        },
        Source {
            key: "sdf22-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf22_1a.zip",
            filename: "sdf22_1a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[
                crate::fixtures::F33_DISTRICTS_FIXTURE,
                crate::fixtures::F33_OHIO_PANEL_FIXTURE,
            ],
            note: "The same survey NCES publishes keyed on `LEAID` rather than the Bureau's \
                   `IDCENSUS`, which is what makes the per-district join possible at all. \
                   Tab-delimited, 354 columns, 19,572 agencies. The fixture keeps the 10,382 \
                   that are comparable — `AGCHRT != 1` and `SCHLEV == 03`, so no charter \
                   agencies and no non-unified districts — plus every Ohio agency. Leaving \
                   charters in put Ohio's 200 smallest agencies at an 8% local share, which is \
                   a fact about charter finance and not about school districts.",
        }],
    },
    Connector {
        key: "nces-ccd",
        publisher: "National Center for Education Statistics",
        feeds: &["education-agency"],
        status: Status::Wired {
            still_blocked: Some(
                "wired for a single year of the LEA directory; the consolidation-aware long \
                 series is not built, because agency files are per-year zips whose column sets \
                 change and the identifier-change history has to be derived rather than read",
            ),
        },
        note: "A corpus spanning 1851 to the present is a panel whose members change, and a \
               long series assembled without accounting for consolidation is silently wrong. \
               A single year of the directory was never blocked by that, and it carries the one \
               column two other connectors were waiting on.",
        sources: &[Source {
            key: "ccd-lea-directory-2223",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_2223_w_1a_083023.zip",
            filename: "ccd-lea-directory-2223.zip",
            format: Format::Zip,
            catalog: Some("nces-ccd-lea-directory"),
            fixtures: &[
                crate::fixtures::F33_DISTRICTS_FIXTURE,
                crate::fixtures::CROSSWALK_FIXTURE,
                crate::fixtures::F33_OHIO_PANEL_FIXTURE,
            ],
            note: "`ST_LEAID` is the Ohio IRN behind an `OH-` prefix, and `LEAID` is the NCES \
                   agency identifier whose last five digits are the Census school district code. \
                   All 609 districts in the funding panel join through it. This is the \
                   NCESID-to-IRN crosswalk `census-f33` records as missing and \
                   `census-geography` needs; it feeds no fixture of its own because it is \
                   consumed while building one belonging to `census-geography`.",
        }],
    },
    Connector {
        key: "census-geography",
        publisher: "U.S. Census Bureau",
        feeds: &["education-agency", "actor"],
        status: Status::Wired {
            still_blocked: None,
        },
        note: "Ohio's funding system has no legislative district in it, so the mapping from \
               school districts to House districts does not exist and has to be built from \
               census blocks. 339 of 609 districts straddle two or more seats, which is why it \
               cannot be an attribution the way county is. See \
               .yidam/decisions/census-geography-connector.yml.",
        sources: &[
            Source {
                key: "baf-2020-oh",
                title: None,
                url: "https://www2.census.gov/geo/docs/maps-data/data/baf2020/\
                      BlockAssign_ST39_OH.zip",
                filename: "BlockAssign_ST39_OH.zip",
                format: Format::Zip,
                catalog: Some("census-block-geography"),
                fixtures: &[crate::fixtures::CROSSWALK_FIXTURE],
                note: "`SDUNI` gives the unified school district for each of Ohio's 276,428 \
                       census blocks. The archive also carries an `SLDL` file and it is the \
                       WRONG one to use: it is the 2020-cycle map, and 66.3% of Ohio's blocks \
                       have changed House district since. Take the House district from \
                       `sldl24-bef` instead.",
            },
            Source {
                key: "sldl24-bef",
                title: None,
                url: "https://www2.census.gov/programs-surveys/decennial/rdo/mapping-files/2025/\
                      2024-state-legislative-bef/sldl24.zip",
                filename: "sldl24.zip",
                format: Format::Zip,
                catalog: Some("census-block-geography"),
                fixtures: &[crate::fixtures::CROSSWALK_FIXTURE],
                note: "The 2024 lower-chamber map, which is the one now in use. Ohio is one of \
                       eight states with changes to both chambers in that cycle. Pinned to a \
                       vintage: when Ohio redistricts again this file changes and the crosswalk \
                       must be regenerated, which no header assertion will catch because the \
                       header will not change.",
            },
            Source {
                key: "sldu24-bef",
                title: None,
                url: "https://www2.census.gov/programs-surveys/decennial/rdo/mapping-files/2025/\
                      2024-state-legislative-bef/sldu24.zip",
                filename: "sldu24.zip",
                format: Format::Zip,
                catalog: Some("census-block-geography"),
                fixtures: &[crate::fixtures::CROSSWALK_FIXTURE],
                note: "The 2024 upper-chamber map. Ohio's constitution requires each Senate \
                       district to be exactly three whole House districts, and the block data \
                       confirms it with no exceptions — so this could have been derived from \
                       `sldl24-bef` by grouping. It is read instead, because the composition is \
                       not sequential (Senate 2 is House 44, 75 and 89) and because a rule worth \
                       relying on is worth checking against the file that would break it.",
            },
            Source {
                key: "pl94-171-2020-oh",
                title: None,
                url: "https://www2.census.gov/programs-surveys/decennial/2020/data/\
                      01-Redistricting_File--PL_94-171/Ohio/oh2020.pl.zip",
                filename: "oh2020.pl.zip",
                format: Format::Zip,
                catalog: Some("census-block-geography"),
                fixtures: &[crate::fixtures::CROSSWALK_FIXTURE],
                note: "Block population, and population 18 and over. The difference is the \
                       apportionment weight: 2,591,886 Ohioans under 18, 22.0% of the state. \
                       Total population would weight a seat full of retirees like one full of \
                       families, against a quantity that is school funding.",
            },
        ],
    },
    Connector {
        key: "dew-child-nutrition",
        publisher: "Ohio Department of Education and Workforce, Office for Child Nutrition",
        feeds: &["education-agency", "metric", "formula-component"],
        status: Status::Wired {
            still_blocked: Some(
                "wired for the eleven sponsor-centric years, 2001 through 2011. The 1998-2000 \
                 files are a different, school-centric column set whose unquoted district names \
                 shift three rows past the last column; from 2012 the report splits into \
                 Traditional, Provision 2 and Community Eligibility streams, and the Traditional \
                 file excludes the highest-poverty sponsors by design",
            ),
        },
        note: "The only long series of the count Ohio's disadvantaged pupil funding is actually \
               paid on. Not an enrollment archive, which is what the catalog said it was for \
               fifteen phases.",
        sources: &[
            Source {
                key: "mr81-2001",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2001/MR81-Oct2001-Delimited.txt",
                filename: "mr81-2001.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2001. Published filename is `MR81-Oct2001-Delimited.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
            Source {
                key: "mr81-2002",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2002/MR81-Oct2002-TabDelimited.txt",
                filename: "mr81-2002.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2002. Published filename is `MR81-Oct2002-TabDelimited.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
            Source {
                key: "mr81-2003",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2003/MR81__Oct_2003_Delimited.txt",
                filename: "mr81-2003.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2003. Published filename is `MR81__Oct_2003_Delimited.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
            Source {
                key: "mr81-2004",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2004/MR81_Oct_2004_delimited.txt",
                filename: "mr81-2004.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2004. Published filename is `MR81_Oct_2004_delimited.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
            Source {
                key: "mr81-2005",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2005/MR81_Oct_2005_Delimited.txt",
                filename: "mr81-2005.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2005. Published filename is `MR81_Oct_2005_Delimited.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
            Source {
                key: "mr81-2006",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2006/MR81_October2006_delimited.txt",
                filename: "mr81-2006.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2006. Published filename is `MR81_October2006_delimited.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
            Source {
                key: "mr81-2007",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2007/MR81_Oct_2007_Delimited.txt",
                filename: "mr81-2007.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2007. Published filename is `MR81_Oct_2007_Delimited.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
            Source {
                key: "mr81-2008",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2008/MR81_October_2008_Delimited-rev2.txt",
                filename: "mr81-2008.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2008. Published filename is `MR81_October_2008_Delimited-rev2.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
            Source {
                key: "mr81-2009",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2009/MR81_October_2009_Delimited-rev.txt",
                filename: "mr81-2009.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2009. Published filename is `MR81_October_2009_Delimited-rev.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
            Source {
                key: "mr81-2010",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2010/MR81_Oct_2010_delimited%20Revised%200911.txt",
                filename: "mr81-2010.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2010. Published filename is `MR81_Oct_2010_delimited Revised 0911.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
            Source {
                key: "mr81-2011",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2011/MR8_Oct_2011_Delimited.txt",
                filename: "mr81-2011.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2011. Published filename is `MR8_Oct_2011_Delimited.txt`; the naming is \
                       inconsistent to the point of needing a per-year table, and 2011's is \
                       misspelled.",
            },
        ],
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
