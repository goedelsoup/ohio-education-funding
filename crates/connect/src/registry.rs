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
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.02",
        filename: "rc-3317-02.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.02. Definitions, including the economically disadvantaged index and its squaring — which the DPIA node recorded as not located in statute.",
    },
    Source {
        key: "rc-3317-011",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.011",
        filename: "rc-3317-011.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.011. Base cost and its components, the most-cited section in the corpus.",
    },
    Source {
        key: "rc-3317-013",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.013",
        filename: "rc-3317-013.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.013. Special education weights, and the clinical categories the corpus could not name.",
    },
    Source {
        key: "rc-3317-014",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.014",
        filename: "rc-3317-014.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.014. Career-technical weights, and the programme categories behind their ordering.",
    },
    Source {
        key: "rc-3317-016",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.016",
        filename: "rc-3317-016.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.016. English learner weights, and the rule the taper actually expresses.",
    },
    Source {
        key: "rc-3317-017",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.017",
        filename: "rc-3317-017.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.017. Local capacity and the state share percentage.",
    },
    Source {
        key: "rc-3317-019",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.019",
        filename: "rc-3317-019.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.019. Temporary transitional aid — the guarantee. Labelled \"gifted units \
               and their prices\" when this list was first written, from the corpus's own citation \
               rather than from the section; gifted units are R.C. 3317.051.",
    },
    Source {
        key: "rc-3317-022",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.022",
        filename: "rc-3317-022.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.022. Core foundation funding: the section that assembles every component, and where disadvantaged pupil impact aid actually lives.",
    },
    Source {
        key: "rc-3317-051",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.051",
        filename: "rc-3317-051.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.051. Gifted unit funding and the three salary prices the units are \
               bought at, which R.C. 3317.022 reaches by cross-reference.",
    },
    Source {
        key: "rc-3317-0212",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.0212",
        filename: "rc-3317-0212.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.0212. Transportation.",
    },
    Source {
        key: "rc-3317-0213",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.0213",
        filename: "rc-3317-0213.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.0213. Preschool special education.",
    },
    Source {
        key: "rc-3317-0217",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.0217",
        filename: "rc-3317-0217.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.0217. The temporary transitional aid guarantee.",
    },
    Source {
        key: "rc-3317-03",
        url: "https://codes.ohio.gov/ohio-revised-code/section-3317.03",
        filename: "rc-3317-03.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 3317.03. What each reported count means, including the economically disadvantaged certification the department is left to define.",
    },
    Source {
        key: "rc-319-301",
        url: "https://codes.ohio.gov/ohio-revised-code/section-319.301",
        filename: "rc-319-301.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 319.301. H.B. 920 tax reduction factors and the twenty-mill floor.",
    },
    Source {
        key: "rc-5705-391",
        url: "https://codes.ohio.gov/ohio-revised-code/section-5705.391",
        filename: "rc-5705-391.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixture: Some(crate::fixtures::STATUTE_FIXTURE),
        note: "R.C. 5705.391. The five-year forecast requirement.",
    },
];

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
        key: "dew-five-year-forecast",
        publisher: "Ohio Department of Education and Workforce",
        feeds: &["education-agency", "revenue-stream", "metric", "fiscal-period"],
        status: Status::Wired,
        note: "The only per-district record here of money that changed hands rather than money \
               a formula computed — and the only one carrying what a district holds.",
        sources: &[
            Source {
                key: "five-year-forecast-fy23",
                url: "https://public.education.ohio.gov/School%20District%20Five-year%20Forecasts/\
                      FY23_5YR_Forecast_Required_Spring%20Update%20Submissions.txt",
                filename: "five-year-forecast-fy23.txt",
                format: Format::Tsv,
                catalog: Some("dew-five-year-forecast"),
                fixture: Some(crate::fixtures::FINANCE_FIXTURE),
                note: "Actuals for FY2020, FY2021 and FY2022. FY2020 is the year the temporary \
                       transitional aid guarantee holds districts at, which this corpus has \
                       until now only been able to infer.",
            },
            Source {
                key: "five-year-forecast-fy26",
                url: "https://public.education.ohio.gov/School%20District%20Five-year%20Forecasts/\
                      FY26_Financial_Forecast_Required_Spring_Update_Submissions.txt",
                filename: "five-year-forecast-fy26.txt",
                format: Format::Tsv,
                catalog: Some("dew-five-year-forecast"),
                fixture: Some(crate::fixtures::FINANCE_FIXTURE),
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
        status: Status::Wired,
        note: "Without this the local half of Ohio school funding is invisible, and the local \
               half is where the disparities live.",
        sources: &[
            Source {
                key: "sd1-ty2021",
                url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/tax_analysis/\
                      tax_data_series/school_district_data/sd1/SD1CY21.xlsx",
                filename: "sd1-ty2021.xlsx",
                format: Format::Xlsx,
                catalog: Some("dot-sd1-school-district-taxes"),
                fixture: Some(crate::fixtures::SD1_FIXTURE),
                note: "The oldest of the four. Two tax years give a level and a change; four give \
                       each district a reappraisal and two quiet years to measure it against, \
                       which is what makes recognized valuation reconstructible. See \
                       `regime_diff::recognized_valuation`.",
            },
            Source {
                key: "sd1-ty2022",
                url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/tax_analysis/\
                      tax_data_series/school_district_data/sd1/SD1CY22.xlsx",
                filename: "sd1-ty2022.xlsx",
                format: Format::Xlsx,
                catalog: Some("dot-sd1-school-district-taxes"),
                fixture: Some(crate::fixtures::SD1_FIXTURE),
                note: "Ohio's counties reappraise or update on a staggered three-year cycle, so \
                       TY2022 through TY2024 contains exactly one valuation event for every one \
                       of the 88. This year completes that window.",
            },
            Source {
                key: "sd1-ty2023",
                url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/tax_analysis/\
                      tax_data_series/school_district_data/sd1/SD1CY23.xlsx",
                filename: "sd1-ty2023.xlsx",
                format: Format::Xlsx,
                catalog: Some("dot-sd1-school-district-taxes"),
                fixture: Some(crate::fixtures::SD1_FIXTURE),
                note: "Carried so that a change in taxes charged can be separated from the level. \
                       Its worksheets are named `ExJVS` and `SD1DATWK23`, against `ExJVS24` and \
                       `SD1DAT24` a year later — the layout drift this connector was blocked on, \
                       now handled by prefix. TY2021 and TY2022 use `SD1DATWK21`/`SD1DATWK22` and \
                       a bare `ExJVS`, which the same prefixes already reach.",
            },
            Source {
                key: "sd1-ty2024",
                url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/tax_analysis/\
                      tax_data_series/school_district_data/sd1/SD1CY24.xlsx",
                filename: "sd1-ty2024.xlsx",
                format: Format::Xlsx,
                catalog: Some("dot-sd1-school-district-taxes"),
                fixture: Some(crate::fixtures::SD1_FIXTURE),
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
            blocked_on: "the deduct-era School Foundation Payment Reports are posted per fiscal \
                         year with no index and no stable path, and the years before about 2015 \
                         are not on the current host at all",
        },
        note: "The one source that would carry the voucher and community-school deduction per \
               resident district, for the years it existed. The FY2027 calculator does not: its \
               transfer channel is a named service-centre charge plus a residual too small to \
               hide one, and under the Fair School Funding Plan those students are funded \
               directly rather than deducted. So the deduction is not missing from the current \
               model — it is absent from it by design, and what is missing is the era before.",
        sources: &[],
    },
    Connector {
        key: "lsc-budget",
        publisher: "Ohio Legislative Service Commission",
        feeds: &["legislation", "fiscal-period", "program", "parameter"],
        status: Status::Wired,
        note: "The only continuous appropriation-line series across the whole period, and the \
               primary source for the pre-2000 record. Wired for **one document**: the final \
               analysis of the current budget act, which is where every provision the Revised \
               Code does not contain actually lives. The redbooks, the Catalog of Budget Line \
               Items and the per-district simulations remain unretrieved, so the appropriation \
               series this connector exists for is still ahead.\n\
               \n\
               The recorded blocker said these are PDFs, which is true and was treated as the \
               end of the matter. A PDF is a container; `Format::Pdf` now has a reader.",
        sources: &[
        Source {
            key: "hb96-edu-redbook",
            url: "https://www.lsc.ohio.gov/assets/legislation/136/hb96/in/files/\
                  hb96-edu-redbook-as-introduced-136th-general-assembly.pdf",
            filename: "hb96-edu-redbook.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-dew-redbook"),
            fixture: Some(crate::fixtures::REDBOOK_FIXTURE),
            note: "The department's appropriation line items, with each ALI's earmarks and their \
                   amounts. This is what answers \"which budget line governs\" for a program that \
                   prorates. **As introduced** — LSC publishes redbooks only for the introduced \
                   bill — so the line item *numbers* are the enacted ones and the *amounts* are \
                   the executive proposal. The distinction is stated wherever a figure from here \
                   is quoted.",
        },
        Source {
            key: "hb96-final-analysis",
            url: "https://www.lsc.ohio.gov/assets/legislation/136/hb96/en0/files/\
                  hb96-bill-analysis-as-enacted-136th-general-assembly.pdf",
            filename: "hb96-final-analysis.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-hb96-analysis"),
            fixture: Some(crate::fixtures::ENACTED_FIXTURE),
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
        status: Status::Wired,
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
        status: Status::Wired,
        note: "Comparability in two directions: whether Ohio is unusual, and an independent \
               check on department figures computed on different definitions. The first is \
               wired and settles the DeRolph claim comparatively; the second needs the \
               per-district file, since the NCESID-to-IRN crosswalk it was waiting on is now \
               held by `nces-ccd`.",
        sources: &[Source {
            key: "f33-fy2022",
            url: "https://www2.census.gov/programs-surveys/school-finances/tables/2022/\
                  secondary-education-finance/elsec22t.xls",
            filename: "elsec22t.xls",
            format: Format::LegacyXls,
            catalog: Some("census-f33-school-system-finances"),
            fixture: Some("crates/dispersion/fixtures/census-f33-states.csv"),
            note: "One year per file and the layout is not stable across years, so the column \
                   map is per-era. The fixture is the state aggregate rather than the panel: \
                   14,106 school systems reduce to 51 rows. The per-district view is now held \
                   too, from NCES's own keying of the same survey — see `sdf22-districts` \
                   below, which reproduces this file's Ohio local share to a tenth of a point.",
        },
        Source {
            key: "sdf22-districts",
            url: "https://nces.ed.gov/ccd/data/zip/sdf22_1a.zip",
            filename: "sdf22_1a.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixture: Some("crates/dispersion/fixtures/f33-districts-fy2022.csv"),
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
        status: Status::Wired,
        note: "A corpus spanning 1851 to the present is a panel whose members change, and a \
               long series assembled without accounting for consolidation is silently wrong. \
               That series is still not built: agency files are per-year zips whose column sets \
               change, and the identifier-change history has to be derived rather than read. \
               But a single year of the directory was never blocked by any of that, and it \
               carries the one column two other connectors were waiting on.",
        sources: &[Source {
            key: "ccd-lea-directory-2223",
            url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_2223_w_1a_083023.zip",
            filename: "ccd-lea-directory-2223.zip",
            format: Format::Zip,
            catalog: Some("nces-ccd-lea-directory"),
            fixture: Some("crates/project/fixtures/legislative-district-crosswalk.csv"),
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
        status: Status::Wired,
        note: "Ohio's funding system has no legislative district in it, so the mapping from \
               school districts to House districts does not exist and has to be built from \
               census blocks. 339 of 609 districts straddle two or more seats, which is why it \
               cannot be an attribution the way county is. See \
               .yidam/decisions/census-geography-connector.yml.",
        sources: &[
            Source {
                key: "baf-2020-oh",
                url: "https://www2.census.gov/geo/docs/maps-data/data/baf2020/\
                      BlockAssign_ST39_OH.zip",
                filename: "BlockAssign_ST39_OH.zip",
                format: Format::Zip,
                catalog: Some("census-block-geography"),
                fixture: Some("crates/project/fixtures/legislative-district-crosswalk.csv"),
                note: "`SDUNI` gives the unified school district for each of Ohio's 276,428 \
                       census blocks. The archive also carries an `SLDL` file and it is the \
                       WRONG one to use: it is the 2020-cycle map, and 66.3% of Ohio's blocks \
                       have changed House district since. Take the House district from \
                       `sldl24-bef` instead.",
            },
            Source {
                key: "sldl24-bef",
                url: "https://www2.census.gov/programs-surveys/decennial/rdo/mapping-files/2025/\
                      2024-state-legislative-bef/sldl24.zip",
                filename: "sldl24.zip",
                format: Format::Zip,
                catalog: Some("census-block-geography"),
                fixture: Some("crates/project/fixtures/legislative-district-crosswalk.csv"),
                note: "The 2024 lower-chamber map, which is the one now in use. Ohio is one of \
                       eight states with changes to both chambers in that cycle. Pinned to a \
                       vintage: when Ohio redistricts again this file changes and the crosswalk \
                       must be regenerated, which no header assertion will catch because the \
                       header will not change.",
            },
            Source {
                key: "sldu24-bef",
                url: "https://www2.census.gov/programs-surveys/decennial/rdo/mapping-files/2025/\
                      2024-state-legislative-bef/sldu24.zip",
                filename: "sldu24.zip",
                format: Format::Zip,
                catalog: Some("census-block-geography"),
                fixture: Some("crates/project/fixtures/legislative-district-crosswalk.csv"),
                note: "The 2024 upper-chamber map. Ohio's constitution requires each Senate \
                       district to be exactly three whole House districts, and the block data \
                       confirms it with no exceptions — so this could have been derived from \
                       `sldl24-bef` by grouping. It is read instead, because the composition is \
                       not sequential (Senate 2 is House 44, 75 and 89) and because a rule worth \
                       relying on is worth checking against the file that would break it.",
            },
            Source {
                key: "pl94-171-2020-oh",
                url: "https://www2.census.gov/programs-surveys/decennial/2020/data/\
                      01-Redistricting_File--PL_94-171/Ohio/oh2020.pl.zip",
                filename: "oh2020.pl.zip",
                format: Format::Zip,
                catalog: Some("census-block-geography"),
                fixture: Some("crates/project/fixtures/legislative-district-crosswalk.csv"),
                note: "Block population, and population 18 and over. The difference is the \
                       apportionment weight: 2,591,886 Ohioans under 18, 22.0% of the state. \
                       Total population would weight a seat full of retirees like one full of \
                       families, against a quantity that is school funding.",
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
