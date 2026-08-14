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
        key: "rc-3311-22",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3311.22",
        filename: "rc-3311-22.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3311.22. How a school district stops existing: one district inside a county \
               educational service center transfers all its territory to an adjacent district \
               served by the same center. Three Ohio districts left the federal agency directory \
               this way inside the window `dispersion::lea_directory` holds, and that directory \
               files all three as closed \"with no effect on another agency's boundaries\". This \
               is the section that says otherwise.",
    },
    Source {
        key: "rc-3311-06",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3311.06",
        filename: "rc-3311-06.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3311.06. The general territory-transfer section, which is the other route a \
               district's land can move and the one that governs transfers between districts in \
               different service centers.",
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
               and where disadvantaged pupil impact aid actually lives. Also the section that \
               settles the scholarship mechanism: it names six funding units, four of them \
               scholarship programs, each paid directly rather than deducted.",
    },
    // Chapter 3310 and the pilot project sections. Wired together because the question they
    // answer is one question — what a scholarship is, who qualifies, and what it pays — and
    // answering it from four programs' statutes separately is how the corpus ended up with three
    // program nodes missing and a fourth asserting a mechanism the current law does not use.
    Source {
        key: "rc-3310-01",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3310.01",
        filename: "rc-3310-01.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3310.01. The educational choice definitions, including what a chartered \
               nonpublic school is.",
    },
    Source {
        key: "rc-3310-03",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3310.03",
        filename: "rc-3310-03.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3310.03. Eligibility for the original educational choice scholarship — the \
               performance-designated route, as distinct from the income-scaled expansion in \
               3310.032.",
    },
    Source {
        key: "rc-3310-032",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3310.032",
        filename: "rc-3310-032.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3310.032. The expansion: eligibility for students whose resident district is \
               not a pilot project district. This is the universal-eligibility route.",
    },
    Source {
        key: "rc-3310-08",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3310.08",
        filename: "rc-3310-08.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3310.08. The educational choice award: the constant multiplier and the base \
               amount the income scale is applied to.",
    },
    Source {
        key: "rc-3310-41",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3310.41",
        filename: "rc-3310-41.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3310.41. The autism scholarship, whole: establishment, eligibility, and the \
               alternative public provider definition the Jon Peterson statute reuses.",
    },
    Source {
        key: "rc-3310-51",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3310.51",
        filename: "rc-3310-51.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3310.51. Jon Peterson definitions: alternative public provider and registered \
               private provider, which the amount in 3317.022(A)(13) is bounded by.",
    },
    Source {
        key: "rc-3310-52",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3310.52",
        filename: "rc-3310-52.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3310.52. The Jon Peterson establishment, its 2012-2013 start, and the \
               five per cent cap on participation.",
    },
    Source {
        key: "rc-3313-975",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3313.975",
        filename: "rc-3313-975.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3313.975. The pilot project scholarship programme — the Cleveland programme, \
               defined by federal court supervision rather than by name.",
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
        key: "rc-3302-01",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3302.01",
        filename: "rc-3302-01.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3302.01. Definitions for the accountability chapter, including \"school building\" \
               and the performance ratings the rest of Chapter 3302 keys on.",
    },
    Source {
        key: "rc-3302-03",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3302.03",
        filename: "rc-3302-03.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3302.03. Report cards. The source of the overall rating that pays the FSFP performance \
               supplement and triggers academic distress, which is the same number doing \
               two jobs.",
    },
    Source {
        key: "rc-3302-10",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3302.10",
        filename: "rc-3302-10.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3302.10. Academic distress commissions. The trigger — three consecutive years at an \
               overall F or under two stars — and the chief executive officer's powers, which \
               include creating the district's budget. State seizure of fiscal control, on \
               academic grounds.",
    },
    Source {
        key: "rc-3302-12",
        title: None,
        url: "https://codes.ohio.gov/ohio-revised-code/section-3302.12",
        filename: "rc-3302-12.html",
        format: Format::Html,
        catalog: Some("ohio-revised-code"),
        fixtures: &[crate::fixtures::STATUTE_FIXTURE],
        note: "R.C. 3302.12. Building-level intervention for a school ranked by performance. The section \
               that makes the school rather than the district the unit, which is why the \
               corpus needs a building class.",
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
                key: "achievement-building-2425",
                title: None,
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      24-25_Achievement_Building.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "achievement-building-2425.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-report-card-achievement"),
                fixtures: &[crate::fixtures::BUILDING_FIXTURE],
                note: "The same achievement file at building grain. ESSA identifies schools, not \
                       districts, and R.C. 3302.12 attaches its intervention to a building — so \
                       this is the file the accountability half of the corpus needs and the \
                       district one cannot substitute for.",
            },
            Source {
                key: "building-details-2425",
                title: None,
                url: "https://reportcardstorage.education.ohio.gov/data-download-2025/\
                      2025_Building_Details.xlsx?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                      &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                      &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
                filename: "building-details-2425.xlsx",
                format: Format::Xlsx,
                catalog: Some("dew-report-card-district-details"),
                fixtures: &[crate::fixtures::BUILDING_FIXTURE],
                note: "Building enrolment, subgroup shares and the district each building belongs \
                       to — the join that makes a school node reachable from its agency.",
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
               resident district, for the years it existed. Kept separate from \
               `dew-scholarship-reports`, which is the department's *public* account of the same \
               channel: one is a per-district file behind a login, the other a statewide \
               aggregate anyone can fetch, and collapsing them would make this blocker look \
               half-lifted when nothing about it has moved.",
        sources: &[],
    },
    Connector {
        key: "dew-scholarship-reports",
        publisher: "Ohio Department of Education and Workforce",
        feeds: &["program"],
        status: Status::Wired {
            still_blocked: Some(
                "wired for the statewide and program-level aggregates, which is what the \
                 department publishes openly. Per-district participation is a different file and \
                 is not here: the annual report cites two routes for it and both 404, so the \
                 breakdown was published, is still referenced by a current departmental document, \
                 and has been withdrawn. See `dew-payment-reports` for the deduct-era half of the \
                 same gap",
            ),
        },
        note: "The department's own public account of the scholarship channel. It answers how \
               large each programme is and how far it reaches; it does not answer which district \
               a scholarship was charged against, and no public source does.",
        sources: &[Source {
            key: "scholarship-annual-2025",
            title: Some("2025 Scholarship Annual Report"),
            url: "https://education.ohio.gov/getattachment/About/Annual-Reports/\
                  2025-Scholarship-Annual-Report.pdf.aspx?lang=en-US",
            filename: "scholarship-annual-2025.pdf",
            format: Format::Pdf,
            catalog: Some("dew-scholarship-annual-report"),
            fixtures: &[crate::fixtures::SCHOLARSHIP_FIXTURE],
            note: "Participation and award totals for all five scholarship programmes, 2024-25. \
                   The only committed source here that sizes the channel from the department \
                   rather than from statute.",
        }],
    },
    Connector {
        key: "lsc-catalog",
        publisher: "Ohio Legislative Service Commission",
        feeds: &["fiscal-period", "program", "legislation", "parameter"],
        status: Status::Wired {
            still_blocked: Some(
                "wired for the education volume of every edition that has one — 2006 and 2008 \
                 through 2025, eighteen distinct documents — the 2012 URL serves the 2011 file byte for \
                 byte, and 2007 has no edition at all. The \
                 Catalog is a standing reference restated each edition rather than a record of \
                 what one act appropriated, so it does not replace `lsc-budget`'s greenbooks and \
                 is not a substitute for the session laws before FY2002",
            ),
        },
        note: "One entry per appropriation line item: four years of actuals, two of \
               appropriation, the act that established it, and what it pays for. The only source \
               here that carries the enacted appropriation and the actual expenditure in adjacent \
               labelled columns — the defect that reverted the workbook attempt.",
        sources: &[
        Source {
            key: "cbli-edu-2006",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2006-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2006.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2006 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2008",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2008-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2008.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2008 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2009",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2009-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2009.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2009 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2010",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2010-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2010.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2010 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2011",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2011-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2011.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2011 edition. There is no 2012 edition: that URL resolves and returns a PDF, \
                   and it is this file byte for byte — same SHA-256, same length. It is left out \
                   rather than wired, because a second copy under a later name would put an FY2012 \
                   enacted appropriation in the fixture twice under two vintages, and vintage is \
                   the column this extract exists to keep. Verified by digest, not by status \
                   code, which is how it was missed the first time.",
        },
        Source {
            key: "cbli-edu-2013",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2013-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2013.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2013 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2014",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2014-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2014.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2014 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2015",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2015-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2015.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2015 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2016",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2016-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2016.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2016 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2017",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2017-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2017.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2017 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2018",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2018-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2018.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2018 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2019",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2019-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2019.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2019 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2020",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2020-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2020.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2020 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2021",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2021-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2021.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2021 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2022",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2022-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2022.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2022 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2023",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2023-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2023.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2023 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2024",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2024-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2024.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2024 edition of the education volume.",
        },
        Source {
            key: "cbli-edu-2025",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/organizations/legislative-service-commission/\
                  files/2025-catalog-of-budget-line-items-edu.pdf",
            filename: "cbli-edu-2025.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-catalog-of-budget-line-items"),
            fixtures: &[
                crate::fixtures::CATALOG_FIXTURE,
                crate::fixtures::CATALOG_BASIS_FIXTURE,
            ],
            note: "The 2025 edition of the education volume.",
        },
        ],
    },
    Connector {
        key: "lsc-budget",
        publisher: "Ohio Legislative Service Commission",
        feeds: &["legislation", "fiscal-period", "program", "parameter"],
        status: Status::Wired {
            still_blocked: Some(
                "wired for the appropriation-line series FY1999 through FY2027, from the \
                 124th General Assembly's greenbook to the 136th's workbooks. The two bienniums \
                 the greenbook route cannot reach — FY2006-07, whose greenbook has no line-item \
                 table, and FY2012-13, whose workbook variants LSC serves as one file — are both \
                 carried by the Catalog of Budget Line Items, which was probed and is not \
                 blocked, and is now extracted: eighteen distinct editions, 2006 and 2008 \
                 through 2025 less the 2012 URL, which serves the 2011 document byte for byte. \
                 See the `lsc-catalog` connector and `the-catalog-of-budget-line-items`. Before FY1999 there is still nothing — the Foundation Program era, \
                 DeRolph I and the equal yield formula need the session laws, and the Catalog's \
                 earliest edition reaches FY2002. The per-district simulations named in earlier \
                 versions of this string are not LSC's and have been removed from it",
            ),
        },
        note: "The only continuous appropriation-line series across the whole period, and the \
               primary source for the pre-2000 record. What is wired is the current biennium; \
               see the decision record `the-greenbook-series` for what the rest of it costs.",
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
                   is quoted, and `hb96-edu-greenbook` is now beside it with the enacted ones.",
        },
        Source {
            key: "hb96-edu-greenbook",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/136/hb96/en0/files/\
                  hb96-edu-greenbook-as-enacted-136th-general-assembly.pdf",
            filename: "hb96-edu-greenbook.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-dew-redbook"),
            fixtures: &[crate::fixtures::GREENBOOK_FIXTURE],
            note: "The same analysis **as enacted**, which the redbook beside it is not. Same \
                   structure, same earmark tables, and the columns are headed `Appropriation` \
                   where the redbook's are headed `Introduced` — so this is what settles whether \
                   a proposed amount survived the legislature. `the-greenbook-series` recorded \
                   that this document existed at a sibling URL to the redbook and it went \
                   unwired for four phases while the corpus quoted the executive proposal.",
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
        Source {
            key: "hb153-enacted",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/129/hb153/en/files/\
                  hb153-budget-in-detail-as-enrolled-129th-general-assembly-10075143.xlsx",
            filename: "hb153-enacted.xls",
            format: Format::LegacyXls,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2012-FY2013. The bill's whole path — introduced, each chamber's substitute, conference, and as enacted. Only the enacted columns are extracted; the rest are the legislative history and are left in the source.",
        },
        Source {
            key: "hb59-enacted",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/130/hb59/en/files/\
                  hb59-budget-in-detail-as-enrolled-130th-general-assembly-10075125.xlsx",
            filename: "hb59-enacted.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2014-FY2015. The bill's whole path — introduced, each chamber's substitute, conference, and as enacted. Only the enacted columns are extracted; the rest are the legislative history and are left in the source.",
        },
        Source {
            key: "hb64-enacted",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/131/hb64/en/files/\
                  hb64-budget-in-detail-as-enrolled-131st-general-assembly-10075109.xlsx",
            filename: "hb64-enacted.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2016-FY2017. The bill's whole path — introduced, each chamber's substitute, conference, and as enacted. Only the enacted columns are extracted; the rest are the legislative history and are left in the source.",
        },
        Source {
            key: "hb49-enacted",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/132/hb49/en/files/\
                  hb49-budget-in-detail-as-enrolled-132nd-general-assembly-10075086.xlsx",
            filename: "hb49-enacted.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2018-FY2019. The bill's whole path — introduced, each chamber's substitute, conference, and as enacted. Only the enacted columns are extracted; the rest are the legislative history and are left in the source.",
        },
        Source {
            key: "hb166-enacted",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/133/hb166/en/files/\
                  hb166-appropriation-spreadsheet-as-enrolled-133rd-general-assembly-10067299.xlsx",
            filename: "hb166-enacted.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2020-FY2021. The bill's whole path — introduced, each chamber's substitute, conference, and as enacted. Only the enacted columns are extracted; the rest are the legislative history and are left in the source.",
        },
        Source {
            key: "hb110-enacted",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/134/hb110/en/files/\
                  hb110-appropriation-spreadsheet-as-enrolled-134th-general-assembly-10067504.xlsx",
            filename: "hb110-enacted.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2022-FY2023. The bill's whole path — introduced, each chamber's substitute, conference, and as enacted. Only the enacted columns are extracted; the rest are the legislative history and are left in the source.",
        },
        Source {
            key: "hb33-enacted",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/135/hb33/en0/files/\
                  hb33-appropriation-spreadsheet-as-enacted-135th-general-assembly-10077370.xlsx",
            filename: "hb33-enacted.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2024-FY2025. The bill's whole path — introduced, each chamber's substitute, conference, and as enacted. Only the enacted columns are extracted; the rest are the legislative history and are left in the source.",
        },
        Source {
            key: "hb96-enacted",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/136/hb96/en0/files/\
                  hb96-appropriation-spreadsheet-as-enacted-136th-general-assembly-10080152.xlsx",
            filename: "hb96-enacted.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2026-FY2027. The bill's whole path — introduced, each chamber's substitute, conference, and as enacted. Only the enacted columns are extracted; the rest are the legislative history and are left in the source.",
        },
        Source {
            key: "hb153-actuals",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/129/hb153/en/files/\
                  hb153-budget-in-detail-with-actual-expenditures-and-adjusted-appropriations-as-enrolled-129th-general-assembly-10075145.xlsx",
            filename: "hb153-actuals.xls",
            format: Format::LegacyXls,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2012-FY2013. Revised after the biennium closed, so it carries actual expenditures and adjusted appropriations and no longer states what was enacted. That is why both variants are held.",
        },
        Source {
            key: "hb59-actuals",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/130/hb59/en/files/\
                  hb59-budget-in-detail-with-actual-expenditures-and-adjusted-appropriations-as-enrolled-130th-general-assembly-10075127.xlsx",
            filename: "hb59-actuals.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2014-FY2015. Revised after the biennium closed, so it carries actual expenditures and adjusted appropriations and no longer states what was enacted. That is why both variants are held.",
        },
        Source {
            key: "hb64-actuals",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/131/hb64/en/files/\
                  hb64-budget-in-detail-with-actual-expenditures-and-adjusted-appropriations-as-enrolled-131st-general-assembly-10075111.xlsx",
            filename: "hb64-actuals.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2016-FY2017. Revised after the biennium closed, so it carries actual expenditures and adjusted appropriations and no longer states what was enacted. That is why both variants are held.",
        },
        Source {
            key: "hb49-actuals",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/132/hb49/en/files/\
                  hb49-budget-in-detail-with-actual-expenditures-and-adjusted-appropriations-as-enrolled-132nd-general-assembly-10075088.xlsx",
            filename: "hb49-actuals.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2018-FY2019. Revised after the biennium closed, so it carries actual expenditures and adjusted appropriations and no longer states what was enacted. That is why both variants are held.",
        },
        Source {
            key: "hb166-actuals",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/133/hb166/en/files/\
                  hb166-appropriation-spreadsheet-with-actual-expenditures-and-adjusted-appropriations-as-enrolled-133rd-general-assembly-10067303.xlsx",
            filename: "hb166-actuals.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2020-FY2021. Revised after the biennium closed, so it carries actual expenditures and adjusted appropriations and no longer states what was enacted. That is why both variants are held.",
        },
        Source {
            key: "hb110-actuals",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/134/hb110/en/files/\
                  hb110-appropriation-spreadsheet-with-actual-expenditures-and-adjusted-appropriations-as-enrolled-134th-general-assembly-10067509.xlsx",
            filename: "hb110-actuals.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2022-FY2023. Revised after the biennium closed, so it carries actual expenditures and adjusted appropriations and no longer states what was enacted. That is why both variants are held.",
        },
        Source {
            key: "hb33-actuals",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/135/hb33/en0/files/\
                  hb33-appropriation-spreadsheet-with-actual-expenditures-and-adjusted-appropriations-as-enacted-135th-general-assembly-10077872.xlsx",
            filename: "hb33-actuals.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2024-FY2025. Revised after the biennium closed, so it carries actual expenditures and adjusted appropriations and no longer states what was enacted. That is why both variants are held.",
        },
        Source {
            key: "hb96-actuals",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/136/hb96/en0/files/\
                  hb96-appropriation-spreadsheet-with-actual-expenditures-as-enacted-136th-general-assembly-10080216.xlsx",
            filename: "hb96-actuals.xlsx",
            format: Format::Xlsx,
            catalog: Some("lsc-appropriation-spreadsheet"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2026-FY2027. Revised after the biennium closed, so it carries actual expenditures and adjusted appropriations and no longer states what was enacted. That is why both variants are held.",
        },
        Source {
            key: "hb94-greenbook",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/124/hb94/en/files/\
                  hb94-edu-greenbook-as-enrolled-124th-general-assembly.pdf",
            filename: "hb94-greenbook.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-dew-redbook"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2002-FY2003 as enrolled, with the three preceding years as actuals. \
                   Read by clustering the right edges of the figures, because the table's own \
                   header labels sit narrower than the columns beneath them.",
        },
        Source {
            key: "hb95-greenbook",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/125/hb95/en/files/\
                  hb95-edu-greenbook-as-enrolled-125th-general-assembly.pdf",
            filename: "hb95-greenbook.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-dew-redbook"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2004-FY2005 as enrolled, with the three preceding years as actuals. \
                   Read by clustering the right edges of the figures, because the table's own \
                   header labels sit narrower than the columns beneath them.",
        },
        Source {
            key: "hb119-greenbook",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/127/hb119/en/files/\
                  hb119-edu-greenbook-as-enrolled-127th-general-assembly.pdf",
            filename: "hb119-greenbook.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-dew-redbook"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2008-FY2009 as enrolled, with the three preceding years as actuals. \
                   Read by clustering the right edges of the figures, because the table's own \
                   header labels sit narrower than the columns beneath them.",
        },
        Source {
            key: "hb1-greenbook",
            title: None,
            url: "https://www.lsc.ohio.gov/assets/legislation/128/hb1/en/files/\
                  hb1-edu-greenbook-as-enrolled-128th-general-assembly.pdf",
            filename: "hb1-greenbook.pdf",
            format: Format::Pdf,
            catalog: Some("lsc-dew-redbook"),
            fixtures: &[crate::fixtures::APPROPRIATION_FIXTURE],
            note: "FY2010-FY2011 as enrolled, with the three preceding years as actuals. \
                   Read by clustering the right edges of the figures, because the table's own \
                   header labels sit narrower than the columns beneath them.",
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
        key: "ohio-session-laws",
        publisher: "Ohio General Assembly",
        feeds: &["legislation", "fiscal-period"],
        status: Status::Wired {
            still_blocked: Some(
                "wired for the two acts whose education tables are printed once and reconcile \
                 exactly — H.B. 215 of the 122nd and H.B. 282 of the 123rd — which gives enacted \
                 line items for FY1998, FY2000 and FY2001. FY1999 is enacted as a single \
                 undifferentiated line and was itemised later by H.B. 650 and corrected by \
                 H.B. 770, both of which print every amended row twice, struck and inserted, and \
                 need a reader that tells the two apart. And the floor is the publisher's: the \
                 legislature's own version index stops at the 122nd General Assembly, so no act \
                 before 1997 is served in any form",
            ),
        },
        note: "The acts themselves, rather than LSC's analyses of them. This is the only route to \
               an enacted appropriation before FY2002, and it reaches exactly four fiscal years \
               before stopping against a wall the publisher put there.",
        sources: &[
            Source {
                key: "hb215-122-enrolled",
                title: Some("Am. Sub. H.B. 215 of the 122nd General Assembly, as enrolled"),
                url: "https://search-prod.lis.state.oh.us/api/v2/general_assembly_122/\
                      legislation/hb215/06_EN/pdf/",
                filename: "hb215-122-enrolled.pdf",
                format: Format::Pdf,
                catalog: Some("ohio-session-laws"),
                fixtures: &[crate::fixtures::SESSION_LAW_FIXTURE],
                note: "The FY1998-99 main operating budget, and the first one enacted after \
                       DeRolph I. Its education table itemises FY1998 across fifty-three GRF \
                       lines and itemises FY1999 across none: the whole year sits in 200-405, \
                       Primary and Secondary Education Funding, against prose promising an \
                       itemisation by 15 January 1998. The version code is `06_EN` and this is \
                       the only act here for which the obvious guess is right.",
            },
            Source {
                key: "hb770-122-enrolled",
                title: Some("Am. Sub. H.B. 770 of the 122nd General Assembly, as enrolled"),
                url: "https://search-prod.lis.state.oh.us/api/v2/general_assembly_122/\
                      legislation/hb770/05_EN/pdf/",
                filename: "hb770-122-enrolled.pdf",
                format: Format::Pdf,
                catalog: Some("ohio-session-laws"),
                fixtures: &[crate::fixtures::SESSION_LAW_FIXTURE],
                note: "The operative FY1998-99 text. It reprints Section 50 as already amended by \
                       H.B. 650 — so its columns carry the itemisation H.B. 215 deferred, with \
                       200-405 struck back to zero — and then amends that again, each replacement \
                       printed on its own line under the column it replaces. Version code \
                       `05_EN`, and the heading is `\" Sec. 50.` rather than `SECTION 50.` \
                       because the whole section is quoted inside an amending act.",
            },
            Source {
                key: "hb282-123-enrolled",
                title: Some("Am. Sub. H.B. 282 of the 123rd General Assembly, as enrolled"),
                url: "https://search-prod.lis.state.oh.us/api/v2/general_assembly_123/\
                      legislation/hb282/08_EN/pdf/",
                filename: "hb282-123-enrolled.pdf",
                format: Format::Pdf,
                catalog: Some("ohio-session-laws"),
                fixtures: &[crate::fixtures::SESSION_LAW_FIXTURE],
                note: "FY2000-01, and not the operating budget. The 123rd appropriated education \
                       in its own act, enacted a day before the budget: H.B. 283 is 977 pages \
                       and contains no Department of Education section at all. Both fiscal years \
                       are fully itemised here — there is no successor to 200-405. Version code \
                       `08_EN`, because two interim postings sit in its version sequence.",
            },
        ],
    },
    Connector {
        key: "ohio-auditor",
        publisher: "Auditor of State of Ohio",
        feeds: &["education-agency", "legislation"],
        status: Status::Wired {
            still_blocked: Some(
                "wired for the five reports that recite a territory transfer, which is what \
                 the corpus needed and not what the Auditor mostly publishes. The executed \
                 resolutions are not here at all -- those sit in educational service center \
                 minute books behind a vendor firewall, and reaching them is a records request \
                 rather than a fetch",
            ),
        },
        note: "A state officer reciting a local body's act. Every other primary source here is \
               the acting body's own document; this is the one place the orders that dissolved \
               three school districts can be read, because the body that issued them records \
               them nowhere in its own audited filings.",
        sources: &[
            Source {
                key: "audit-bettsville-final-fy2014",
                title: None,
                url: "https://ohioauditor.gov/auditsearch/Reports/2015/Bettsville_Local_School_District_14_Seneca.pdf",
                filename: "audit-bettsville-final-fy2014.pdf",
                format: Format::Pdf,
                catalog: Some("auditor-district-audits"),
                fixtures: &[crate::fixtures::TRANSFER_FIXTURE],
                note: "Bettsville Local School District, final audit. The departing district's own last report, and \
                       the only one of these five in which the district that ceased is the audited entity.",
            },
            Source {
                key: "audit-old-fort-fy2015",
                title: None,
                url: "https://ohioauditor.gov/auditsearch/Reports/2016/Old_Fort_Local_School_District_15_Seneca.pdf",
                filename: "audit-old-fort-fy2015.pdf",
                format: Format::Pdf,
                catalog: Some("auditor-district-audits"),
                fixtures: &[crate::fixtures::TRANSFER_FIXTURE],
                note: "Old Fort Local School District. Recites the same resolution from the receiving side, which is \
                       the corroboration: two audited entities, two years apart, one transaction.",
            },
            Source {
                key: "audit-berkshire-fy2016",
                title: None,
                url: "https://ohioauditor.gov/auditsearch/Reports/2016/Berkshire_Local_School_District_16-Geauga.pdf",
                filename: "audit-berkshire-fy2016.pdf",
                format: Format::Pdf,
                catalog: Some("auditor-district-audits"),
                fixtures: &[crate::fixtures::TRANSFER_FIXTURE],
                note: "Berkshire Local School District. Carries both the resolution and the reason Ledgemont ceased \
                       -- 'due to financial difficulties' -- which is a fact about why that no other source here \
                       states.",
            },
            Source {
                key: "audit-west-geauga-fy2020",
                title: None,
                url: "https://ohioauditor.gov/auditsearch/Reports/2020/West_Geauga_LSD_20-Geauga_FINAL.pdf",
                filename: "audit-west-geauga-fy2020.pdf",
                format: Format::Pdf,
                catalog: Some("auditor-district-audits"),
                fixtures: &[crate::fixtures::TRANSFER_FIXTURE],
                note: "West Geauga Local School District. The only one of the three that names the section: the \
                       transfer was enacted 'under O.R.C. 3311.22'. It also states what moved -- all of Newbury's \
                       $180,214,110 valuation -- which is what withdrew this repository's claim that the territory \
                       was split with Chardon.",
            },
            Source {
                key: "audit-geauga-esc-final-fy2019",
                title: None,
                url: "https://ohioauditor.gov/auditsearch/Reports/2021/Geauga_County_Educational_Service_Center_FINAL_19-Geauga_FINAL.pdf",
                filename: "audit-geauga-esc-final-fy2019.pdf",
                format: Format::Pdf,
                catalog: Some("auditor-district-audits"),
                fixtures: &[crate::fixtures::TRANSFER_FIXTURE],
                note: "Geauga County Educational Service Center, final audit. The body that ordered two of the three \
                       district transfers, recording its own dissolution: a joint resolution with the Lake County ESC \
                       on 7 November 2019, merging into the Educational Service Center of the Western Reserve. One of \
                       the 66 service agencies `dispersion::lea_directory` counts out of the federal register, and the \
                       only one of them with an instrument behind it. The register files it closed with no effect on \
                       anyone; Lake County ESC keeps its identifier and takes the new name.",
            },
        ],
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
            key: "f33-fy2024",
            title: None,
            url: "https://www2.census.gov/programs-surveys/school-finances/tables/2024/\
                  secondary-education-finance/elsec24t.xlsx",
            filename: "elsec24t.xlsx",
            format: Format::Xlsx,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_FY2024_FIXTURE],
            note: "The FY2024 survey, and the year OCG White Paper 015 quotes for its \
                   cross-state table. Published as .xlsx where FY2022 is .xls, and one column \
                   shorter — IDCENSUS is gone, which shifts every later index by one.",
        },
        Source {
            key: "sdf09-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf091a_txt.zip",
            filename: "sdf09-districts.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2009. Published under a `_txt` suffix the \
                   FY2012-FY2022 files do not use, which is why the panel began at FY2012 and \
                   opened one year inside the FY2010-FY2014 real trough rather than before it.",
        },
        Source {
            key: "sdf10-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf101a_txt.zip",
            filename: "sdf10-districts.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2010. Published under a `_txt` suffix the \
                   FY2012-FY2022 files do not use, which is why the panel began at FY2012 and \
                   opened one year inside the FY2010-FY2014 real trough rather than before it.",
        },
        Source {
            key: "sdf11-districts",
            title: None,
            url: "https://nces.ed.gov/ccd/data/zip/sdf11_1a_txt.zip",
            filename: "sdf11-districts.zip",
            format: Format::Zip,
            catalog: Some("census-f33-school-system-finances"),
            fixtures: &[crate::fixtures::F33_OHIO_PANEL_FIXTURE],
            note: "The school finance survey for FY2011. Published under a `_txt` suffix the \
                   FY2012-FY2022 files do not use, which is why the panel began at FY2012 and \
                   opened one year inside the FY2010-FY2014 real trough rather than before it.",
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
                "wired for every school year the directory publishes with an Ohio slice this \
                 reader can take, 1994-95 through 2023-24. The nine years before 1994-95 exist \
                 and are not held: 1986-87 through 1993-94 are the same fixed-width family, and \
                 nothing consumes them. The identifier-change history is still not derivable \
                 from this source, and thirty years of it now say so rather than sixteen: Ohio \
                 has never once filed the status code that marks a consolidation",
            ),
        },
        note: "A corpus spanning 1851 to the present is a panel whose members change, and a \
               long series assembled without accounting for consolidation is silently wrong. \
               What the directory settles is which agencies existed when, and what it refuses to \
               settle is why any of them stopped.",
        sources: &[
            Source {
                key: "ccd-lea-directory-9495",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/pau94datr.zip",
                filename: "ccd-lea-directory-9495.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 1994-95, and the oldest year this repository holds. Fixed-width, 1,030 bytes a record, no header. The revision is the only edition served: `pau94data.zip` is a 404 and `pau94datr.zip` is what the publisher links. 790 Ohio agencies, every one of them filed `1 No significant boundary change` — the first year held is the one year with no departure in it.",
            },
            Source {
                key: "ccd-lea-directory-9596",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/pau95data.zip",
                filename: "ccd-lea-directory-9596.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 1995-96. The file named `pau95` is the year *after* the file named `pau94`, which is the trap in this whole era: the archive is keyed on the year the survey closed and the directory describes the year it opened. Its own layout document settles it, saying `1995-96` in the heading and again in the sentence beneath. Nine agencies here are flagged closed, and all nine are county boards of education.",
            },
            Source {
                key: "ccd-lea-directory-9697",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/pau96data.zip",
                filename: "ccd-lea-directory-9697.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 1996-97. The layout document changes shape without the file changing: 1994-95 and 1995-96 print name, type, position, size, and this one prints name, size, position, type. The record is byte-for-byte the same 1,030.",
            },
            Source {
                key: "ccd-lea-directory-9798",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/pau97data.zip",
                filename: "ccd-lea-directory-9798.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 1997-98, and the last year before Ohio has a community school. Type 7 here is 23 agencies and every one of them is a county board's data-processing centre. Four years later type 7 is 249 and the composition is the opposite.",
            },
            Source {
                key: "ccd-lea-directory-9899",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag981c_dat.zip",
                filename: "ccd-lea-directory-9899.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 1998-99. The record shrinks to 722 bytes and every column this reader takes moves at once: the name field widens from 30 to 60, the agency type goes from byte 121 to 234, and the status from 162 to 280. Nothing before this year can be read with the offsets of anything after it.",
            },
            Source {
                key: "ccd-lea-directory-9900",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag991b_dat.zip",
                filename: "ccd-lea-directory-9900.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 1999-2000, second revision. Its layout document's own header says `FILE NAME = ag991a.dat` while the record count beneath it is this file's, which is why the layout is read for offsets and never for identity. `ag991a_dat.zip` is still served; its Ohio rows are identical in all five columns taken here.",
            },
            Source {
                key: "ccd-lea-directory-0001",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag001a_dat.zip",
                filename: "ccd-lea-directory-0001.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2000-01. The record grows a byte, to 723, and the status column moves with it to 281. A one-byte drift is the failure this era is most likely to produce quietly, because the byte to the left is the metropolitan indicator and its values are also small integers.",
            },
            Source {
                key: "ccd-lea-directory-0102",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag011a_dat.zip",
                filename: "ccd-lea-directory-0102.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2001-02. 725 bytes, and the member inside the archive is `AG011A.DAT` in capitals where its neighbours are lower case. The year Ohio's sixteen special-education regional resource centres and its twenty-four data-processing centres all leave the directory at once.",
            },
            Source {
                key: "ccd-lea-directory-0203",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag021a_dat.zip",
                filename: "ccd-lea-directory-0203.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2002-03. 729 bytes, status at 284. Its layout document runs `BOUND02`'s entire entry onto the end of the line above it, so a parser that anchors variable names to the start of a line does not find the column at all — and the two neighbouring years put it somewhere else, so guessing is not available either.",
            },
            Source {
                key: "ccd-lea-directory-0304",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag031b_dat.zip",
                filename: "ccd-lea-directory-0304.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2003-04. Named `_dat` and holding `ag031b.txt`, which is fixed-width regardless of what it is called. **Its layout document states the status column's position wrongly** — 281, which is the metropolitan indicator's — and states it a second time as 384. The true position is 284, and this repository takes it from the file rather than the document: see `fixtures::CCD_FIXED_WIDTH`. `ag031a_dat.zip` is still served and its Ohio rows agree in all five columns.",
            },
            Source {
                key: "ccd-lea-directory-0405",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag041c_dat.zip",
                filename: "ccd-lea-directory-0405.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2004-05, third revision. The record almost halves, to 519 bytes, because the dropout and completer counts leave the agency file. Nothing this reader takes moves. `ag041a_dat.zip` is still served and agrees on Ohio.",
            },
            Source {
                key: "ccd-lea-directory-0506",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag051a_dat.zip",
                filename: "ccd-lea-directory-0506.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2005-06. Its layout document is the previous year's with the heading changed: the sentence beneath still reads *\"data for the school year 2004-05\"* and the observation count is still 18,085, where this file has 18,213 records. The record length it gives, 519, is right — which is the worst arrangement, because the parts that are wrong are the parts a reader would use to check the parts that are not.",
            },
            Source {
                key: "ccd-lea-directory-0607",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag061c_dat.zip",
                filename: "ccd-lea-directory-0607.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2006-07, and the last fixed-width year. 530 bytes; latitude, longitude and a congressional district code arrive ahead of the status column and push it to 309. The year the agency type stops distinguishing a local district from a component of a supervisory union: Ohio's 238 type 1 and 377 type 2 become 614 type 1, with no agency joining or leaving.",
            },
            Source {
                key: "ccd-lea-directory-0708",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag071b_txt.zip",
                filename: "ccd-lea-directory-0708.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2007-08, and not fixed-width at all — tab-delimited with a header, exactly like 2008-09. It reads through the alias table with nothing added: `STID07`, `NAME07`, `TYPE07` and `BOUND07` were listed there before any wired year used them, because the aliases are per naming convention rather than per year.",
            },
            Source {
                key: "ccd-lea-directory-0809",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag081a_txt.zip",
                filename: "ccd-lea-directory-0809.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2008-09. Tab-delimited with a header, and the columns carry a year suffix: `STID08`, `BOUND08`, `TYPE08`. This is also the one year with no charter column at all, which is why agency type rather than a charter flag is what this reader keys on.",
            },
            Source {
                key: "ccd-lea-directory-0910",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag092a_txt.zip",
                filename: "ccd-lea-directory-0910.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2009-10. The second revision. `ag091a_txt.zip` is the first and differs from it in 42 rows nationally, none of them Ohio's.",
            },
            Source {
                key: "ccd-lea-directory-1011",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag102a_txt.zip",
                filename: "ccd-lea-directory-1011.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2010-11. The second revision again, and the year the column names lose their suffix: `STID` and `BOUND` from here to 2013-14.",
            },
            Source {
                key: "ccd-lea-directory-1112",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag111a_txt.zip",
                filename: "ccd-lea-directory-1112.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2011-12. The archive is named `ag111a_txt` and the file inside it is `ag111a_supp.txt`, which is the naming the next two years use in the URL as well.",
            },
            Source {
                key: "ccd-lea-directory-1213",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag121a_supp_txt.zip",
                filename: "ccd-lea-directory-1213.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2012-13. The year `LSTATE` stops agreeing with `FIPST`: LEAID 3901497, Urban Pathways of Youngstown, is filed with `FIPST=39` and `LSTATE=PA`. Ohio is selected on `FIPST` throughout for that reason.",
            },
            Source {
                key: "ccd-lea-directory-1314",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ag131a_supp_txt.zip",
                filename: "ccd-lea-directory-1314.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2013-14. The last year of the tab-delimited era.",
            },
            Source {
                key: "ccd-lea-directory-1415",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_1415_w_0216161ar_txt.zip",
                filename: "ccd-lea-directory-1415.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2014-15. The revision, not the original. `..._0216161a_txt.zip` is superseded and NCES's own file index no longer lists it; the two differ in exactly two columns, which spell state names out rather than abbreviating them. Still tab-delimited, and the state identifier column is renamed `ST_LEAID` here while its contents stay bare digits.",
            },
            Source {
                key: "ccd-lea-directory-1516",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_1516_w_1a_011717_csv.zip",
                filename: "ccd-lea-directory-1516.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2015-16. The first comma-delimited year, and the last one whose `ST_LEAID` is a bare IRN.",
            },
            Source {
                key: "ccd-lea-directory-1617",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_1617_w_1a_11212017_csv.zip",
                filename: "ccd-lea-directory-1617.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2016-17. The year the `OH-` prefix appears on `ST_LEAID`. The combined `..._11212017.zip` holds the same CSV byte for byte alongside a SAS file; this is the smaller archive of the two and NCES's index lists it.",
            },
            Source {
                key: "ccd-lea-directory-1718",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_1718_w_1a_083118.zip",
                filename: "ccd-lea-directory-1718.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2017-18. Carries the CSV beside a `.sas7bdat` of the same directory, which is why the member is chosen by suffix rather than by name.",
            },
            Source {
                key: "ccd-lea-directory-1819",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_1819_l_1a_091019.zip",
                filename: "ccd-lea-directory-1819.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2018-19. The final release. The preliminary `..._0a_04082019_csv.zip` is a different file and NCES's index no longer lists it; for Ohio they carry the same 1,074 agencies with the same statuses. Note the URL says `_l_` and the CSV inside says `_w_`.",
            },
            Source {
                key: "ccd-lea-directory-1920",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_1920_w_1a_082120.zip",
                filename: "ccd-lea-directory-1920.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2019-20. From here the header is 58 columns in a fixed order and does not move again.",
            },
            Source {
                key: "ccd-lea-directory-2021",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_2021_w_1a_080621.zip",
                filename: "ccd-lea-directory-2021.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2020-21. Not UTF-8: a cp1252 en dash sits in an Arkansas agency's name. Every file here is read Latin-1 byte for byte, because a lossy read would put a replacement character into committed data.",
            },
            Source {
                key: "ccd-lea-directory-2122",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_2122_w_1a_071722.zip",
                filename: "ccd-lea-directory-2122.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2021-22. One Ohio agency closes this year, which is the fewest in the window.",
            },
            Source {
                key: "ccd-lea-directory-2324",
                title: None,
                url: "https://nces.ed.gov/ccd/data/zip/ccd_lea_029_2324_w_1a_073124.zip",
                filename: "ccd-lea-directory-2324.zip",
                format: Format::Zip,
                catalog: Some("nces-ccd-lea-directory"),
                fixtures: &[crate::fixtures::CCD_DIRECTORY_FIXTURE],
                note: "School year 2023-24. The latest published, and the year the departure count returns to twelve.",
            },
            Source {
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
                crate::fixtures::CCD_DIRECTORY_FIXTURE,
            ],
            note: "`ST_LEAID` is the Ohio IRN behind an `OH-` prefix, and `LEAID` is the NCES \
                   agency identifier whose last five digits are the Census school district code. \
                   All 609 districts in the funding panel join through it. This is the \
                   NCESID-to-IRN crosswalk `census-f33` records as missing and \
                   `census-geography` needs. School year 2022-23, and the year every other \
                   fixture here resolves an IRN through — which is what made it worth asking \
                   what the other fifteen years say.",
            },
        ],
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
                "wired for every October the archive holds, 1998 through 2014, across all three \
                 of the streams the report splits into from 2012. Three things it still cannot \
                 reach: October 2014 is where the directory stops, nine years short of the \
                 corpus's FY2024 observations; the three split Octobers have a band and not a \
                 poverty share, because community-eligibility sponsors collect no applications \
                 at all; and the 1998-2000 files state no sponsor type, so some thirty-five \
                 sponsors a year predate the FY2001 file they borrow one from and stay untyped",
            ),
        },
        note: "The only long series of the count Ohio's disadvantaged pupil funding is actually \
               paid on. Not an enrollment archive, which is what the catalog said it was for \
               fifteen phases.",
        sources: &[
            Source {
                key: "mr81-1998",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_1998/MR81_OCT_1998.txt",
                filename: "mr81-1998.txt",
                format: Format::Text,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 1998, and the first of three school-centric files. Nominally \
                       comma-separated and actually 170 characters wide with the separators at \
                       fixed offsets, which is the only reason the seven district names a year \
                       carrying a comma do not shift their own rows.",
            },
            Source {
                key: "mr81-1999",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_1999/\
                      MR81_OCT_1999_Delimited.TXT",
                filename: "mr81-1999.txt",
                format: Format::Text,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 1999. The directory also lists `MR81.1099`, the contemporaneous \
                       posting, which 404s — a listed file that is not there. This one and its \
                       two neighbours were all written in 2004 from one roster.",
            },
            Source {
                key: "mr81-2000",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2000/\
                      MR81_OCT_2000_Delimited.TXT",
                filename: "mr81-2000.txt",
                format: Format::Text,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2000. The one year whose contemporaneous printing survives beside \
                       the 2004 rewrite, so it is the only check on whether the rewrite was \
                       faithful: 4,233 of 4,236 schools carry the same three figures in both.",
            },
            Source {
                key: "mr81-2001",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2001/MR81-Oct2001-Delimited.txt",
                filename: "mr81-2001.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2001, and the only comma-delimited year. Nine of its rows carry a \
                       comma inside a school name, and split positionally they put a site IRN \
                       into the enrolment column — which is how the panel published an FY2001 \
                       poverty share 1.8 points low for four phases.",
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
                note: "October 2011, the last year the report is one file. Published filename is \
                       `MR8_Oct_2011_Delimited.txt` — the naming is inconsistent to the point of \
                       needing a per-year table, and this one is misspelled.",
            },
            Source {
                key: "mr81-2012-traditional",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2012/MR81%20Traditional/\
                      MR81%20Traditional%20Delimited%20File.txt",
                filename: "mr81-2012-traditional.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2012, the first year the report is three. This is the stream that \
                       still collects applications, and its own readme says it excludes the other \
                       two — so it is the file a naive extension would append, and the one that \
                       would bend the series downward for reasons that are not poverty.",
            },
            Source {
                key: "mr81-2012-provision2",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2012/\
                      MR81%20Provision%202/MR81%20Provision%202%20Delimited%20File.txt",
                filename: "mr81-2012-provision2.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2012, Provision 2. Twenty-odd sponsors whose approvals are frozen \
                       at the base year the file names, so the same counts appear in 2012, 2013 \
                       and 2014 while the enrolment beneath them moves.",
            },
            Source {
                key: "mr81-2012-community",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2012/MR81%20CEO/\
                      MR81%20CEO%20Delimited%20File.txt",
                filename: "mr81-2012-community.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2012, community eligibility. Every free and reduced figure in this \
                       file is zero, because these sponsors collect no applications at all. Its \
                       header also carries an empty column its rows do not, so a name resolved \
                       against it points one past the end of the data.",
            },
            Source {
                key: "mr81-2013-traditional",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2013/\
                      2013%20MR81Delimited%20tradional.txt",
                filename: "mr81-2013-traditional.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2013, traditional. The published filename misspells its own \
                       stream.",
            },
            Source {
                key: "mr81-2013-provision2",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2013/\
                      2013%20MR81Delimited%20prov%202.txt",
                filename: "mr81-2013-provision2.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2013, Provision 2. The directory posts this stream twice, once \
                       delimited and once printed, and the two agree to the unit across all \
                       twenty-four sites — which is what says the printed reader beside it can be \
                       trusted on the stream that has no delimited file.",
            },
            Source {
                key: "mr81-2013-community",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2013/\
                      2nd%20CEP%202013%20MR81.txt",
                filename: "mr81-2013-community.txt",
                format: Format::Text,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2013, community eligibility — the one stream-year with no \
                       delimited file. Posted as the printed report, which names its sponsors \
                       without numbering or typing them, so the identifiers come from the \
                       delimited files either side.",
            },
            Source {
                key: "mr81-2014-traditional",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2014/\
                      October_2014_MR81_Traditional_delimited.txt",
                filename: "mr81-2014-traditional.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2014, traditional, and the last October the archive holds.",
            },
            Source {
                key: "mr81-2014-provision2",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2014/\
                      October_2014_MR81_Provision_2_delimited.txt",
                filename: "mr81-2014-provision2.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2014, Provision 2. One of its fifteen sponsors reports no \
                       applications and a hundred per cent, which is a community-eligibility row \
                       filed under the wrong stream.",
            },
            Source {
                key: "mr81-2014-community",
                title: None,
                url: "https://public.education.ohio.gov/MR81/MR81_October_2014/\
                      October_2014_MR81_CEP_delimited_REVISED.txt",
                filename: "mr81-2014-community.txt",
                format: Format::Tsv,
                catalog: Some("dew-mr81-enrollment-archive"),
                fixtures: &[crate::fixtures::MR81_FIXTURE],
                note: "October 2014, community eligibility. By this October a sixth of Ohio's \
                       public meal-program enrolment is in this file, against a fourteenth two \
                       years earlier. Its printed percentage is the directly-certified count \
                       times 1.6 capped at enrolment, in all 735 rows.",
            },
        ],
    },
    Connector {
        key: "dew-school-improvement",
        publisher: "Ohio Department of Education and Workforce, School and District Improvement",
        feeds: &["school", "education-agency", "accountability-regime"],
        status: Status::Wired {
            still_blocked: Some(
                "wired for the current identification lists only. The department republishes each \
                 list in place under a dated filename rather than archiving prior cycles, so \
                 there is no history here: a school that exited before this file was written is \
                 indistinguishable from one never identified",
            ),
        },
        note: "Who is actually in the accountability system. The corpus modelled the regime, the \
               trigger and the intervention before it could name a single identified school.",
        sources: &[
            Source {
                key: "csi-identified-2026",
                title: None,
                url: "https://education.ohio.gov/getattachment/Topics/School-and-District-Improvement/\
                      Identification-and-Requirements/\
                      CSI-Identified-Schools-updated-5-1-26.xlsx.aspx?lang=en-US",
                filename: "csi-identified-2026.xlsx",
                format: Format::Xlsx,
                catalog: Some("ohio-essa-state-plan"),
                fixtures: &[crate::fixtures::IDENTIFIED_FIXTURE],
                note: "Comprehensive Support and Improvement. The lowest-performing Title I served \
                       schools, high schools under a 67% federal graduation rate, and buildings \
                       that have not exited ATSI in three years. Updated 1 May 2026.",
            },
            Source {
                key: "tsi-identified-2026",
                title: None,
                url: "https://education.ohio.gov/getattachment/Topics/School-and-District-Improvement/\
                      Identification-and-Requirements/\
                      TSI-Identified-Schools-updated-1-7-26.xlsx.aspx?lang=en-US",
                filename: "tsi-identified-2026.xlsx",
                format: Format::Xlsx,
                catalog: Some("ohio-essa-state-plan"),
                fixtures: &[crate::fixtures::IDENTIFIED_FIXTURE],
                note: "Targeted Support and Improvement — a subgroup consistently in the bottom 2% \
                       of its own rank order. Updated 7 January 2026.",
            },
            Source {
                key: "atsi-identified-2026",
                title: None,
                url: "https://education.ohio.gov/getattachment/Topics/School-and-District-Improvement/\
                      Identification-and-Requirements/\
                      ATSI-Identified-Schools-updated-1-7-26.xlsx.aspx?lang=en-US",
                filename: "atsi-identified-2026.xlsx",
                format: Format::Xlsx,
                catalog: Some("ohio-essa-state-plan"),
                fixtures: &[crate::fixtures::IDENTIFIED_FIXTURE],
                note: "Additional Targeted Support and Improvement — a subgroup at or below the CSI \
                       identification threshold score. Updated 7 January 2026.",
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
