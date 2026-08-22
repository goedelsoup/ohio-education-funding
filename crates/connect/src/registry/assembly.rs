//! Ohio General Assembly.
//!
//! The primary sources: the Revised Code sections the formula is written in, the session laws
//! that appropriated against it before the workbook series begins, and the bills themselves.
//!
//! [`OHIO_LAWS_SECTIONS`] is the largest source list in the registry — every statutory section
//! this corpus cites, each with the catalog node saying what it can be trusted for.

use super::{Connector, Format, Source, Status};

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

pub(super) const LAWS: Connector = Connector {
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
};

pub(super) const SESSION_LAWS: Connector = Connector {
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
           before stopping against a wall the publisher put there. It also carries one act \
           read for its provisions rather than its table — see `hb583-134-enrolled`, which \
           establishes that the two purposes are separable and that the second has no floor: \
           the version index serves every act back to the 122nd, whether or not the act \
           prints a money column anyone here can read.",
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
        Source {
            key: "hb583-134-enrolled",
            title: Some("Sub. H.B. 583 of the 134th General Assembly, as enrolled"),
            url: "https://search-prod.lis.state.oh.us/api/v2/general_assembly_134/\
                  legislation/hb583/08_EN/pdf/",
            filename: "hb583-134-enrolled.pdf",
            format: Format::Pdf,
            catalog: Some("ohio-session-laws"),
            fixtures: &[crate::fixtures::CORRECTIONS_FIXTURE],
            note: "Not an appropriation table, and the reason this connector's note now has \
                   two halves. The corpus recorded for four phases that H.B. 583's contents \
                   were unreachable — \"a separate PDF that no connector fetches\" — while \
                   every other act here came through this same API. Its amending title names \
                   twelve sections of R.C. 3317 and four uncodified sections of H.B. 110, \
                   which is the answer to \"which provisions moved\" printed on the act's own \
                   first page. Version code `08_EN`, which is the same index H.B. 282 uses \
                   and not a coincidence worth trusting: the six below it are all 404.",
        },
    ],
};

pub(super) const BILLS: Connector = Connector {
    key: "ohio-bills",
    publisher: "Ohio General Assembly",
    feeds: &["draft-legislation"],
    // Deliberately not `Wired`, and not for want of effort. Turning a bill into provisions is
    // reading rather than extraction — deciding that a section amending R.C. 3310.032 changes
    // eligibility rather than an award, and that no lever here expresses it, is a judgement
    // about the funding system. A parser over section headings would emit something that
    // looked authoritative and was not. What the retrieval is for is the text, pinned, so a
    // draft node can be checked against the document it claims to describe.
    status: Status::Retrievable,
    note: "A bill before it is law, which is a third artefact and not a variant of the two \
           already here: `ohio-laws` serves the Revised Code as it stands and \
           `ohio-session-laws` serves acts as they were passed. The as-introduced version is \
           always `00_IN`, so unlike an enrolled act it needs no index lookup — introduction \
           is always a bill's first version. Every other stage is positional and does. The \
           listing endpoint reports a bill's *first* version rather than its current one, so \
           it cannot be used to tell a pending bill from an enacted one: H.B. 186 of the \
           136th appears there as `As Introduced` and was enrolled effective 20 March 2026.",
    sources: &[Source {
        key: "hb643-136-introduced",
        title: Some("H.B. 643 of the 136th General Assembly, as introduced"),
        url: "https://search-prod.lis.state.oh.us/api/v2/general_assembly_136/\
                  legislation/hb643/00_IN/html/",
        filename: "hb643-136-introduced.html",
        format: Format::Html,
        catalog: Some("ohio-bills"),
        fixtures: &[],
        note: "One section, amending R.C. 3310.032 to cap EdChoice expansion eligibility \
                   at $500,000 of federal adjusted gross income from the 2026-2027 school \
                   year, indexed to CPI. Chosen as the first pending bill here because every \
                   provision it has falls in the scholarship channel, which this workspace \
                   does not model at all — so it is the case that proves a draft can be real, \
                   current, and entirely unpriceable, and it is the one that found \
                   `project::drafts` reporting an unpriceable bill as costing zero.",
    }],
};
