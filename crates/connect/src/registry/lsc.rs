//! Ohio Legislative Service Commission.
//!
//! What the General Assembly appropriated, in the two forms LSC publishes it: the Catalog of
//! Budget Line Items, which reaches furthest back, and the budget workbooks and greenbooks, which
//! carry the line-item detail.

use super::{Connector, Format, Source, Status};

pub(super) const CATALOG: Connector = Connector {
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
};

pub(super) const BUDGET: Connector = Connector {
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
        fixtures: &[
            crate::fixtures::APPROPRIATION_FIXTURE,
            crate::fixtures::LSC_GREENBOOK_FIXTURE,
        ],
        note: "FY2002-FY2003 as enrolled, with the three preceding years as actuals. \
               Read by clustering the right edges of the figures, because the table's own \
               header labels sit narrower than the columns beneath them.",
    },
    Source {
        key: "hb66-greenbook",
        title: Some("H.B. 66 of the 126th General Assembly — LSC Greenbook, Department of Education"),
        url: "https://www.lsc.ohio.gov/assets/legislation/126/hb66/en/files/\
              hb66-edu-greenbook-as-enrolled-126th-general-assembly.pdf",
        filename: "hb66-greenbook.pdf",
        format: Format::Pdf,
        catalog: Some("lsc-dew-redbook"),
        fixtures: &[crate::fixtures::LSC_GREENBOOK_FIXTURE],
        note: "FY2006-07, the act that began the tangible personal property phase-out. Not \
               also a source for the appropriation series, which reaches these years through \
               the Catalog.",
    },
    Source {
        key: "hb153-greenbook",
        title: Some("H.B. 153 of the 129th General Assembly — LSC Greenbook, Department of Education"),
        url: "https://www.lsc.ohio.gov/assets/legislation/129/hb153/en/files/\
              hb153-edu-greenbook-as-enrolled-129th-general-assembly.pdf",
        filename: "hb153-greenbook.pdf",
        format: Format::Pdf,
        catalog: Some("lsc-dew-redbook"),
        fixtures: &[crate::fixtures::LSC_GREENBOOK_FIXTURE],
        note: "FY2012-13, the repeal of the Evidence-Based Model and the Bridge formula's \
               first biennium. The anchor chain that reaches FY2027 passes into this act and \
               how far below it goes has been open since the chain was written.",
    },
    Source {
        key: "hb110-greenbook",
        title: Some("H.B. 110 of the 134th General Assembly — LSC Greenbook, Department of Education"),
        url: "https://www.lsc.ohio.gov/assets/legislation/134/hb110/en/files/\
              hb110-edu-greenbook-as-enrolled-134th-general-assembly.pdf",
        filename: "hb110-greenbook.pdf",
        format: Format::Pdf,
        catalog: Some("lsc-dew-redbook"),
        fixtures: &[crate::fixtures::LSC_GREENBOOK_FIXTURE],
        note: "FY2022-23, the Fair School Funding Plan as enacted. The largest of the twelve \
               and the one a reader is most likely to want, because every current per-district \
               figure in this repository is computed under it.",
    },
    Source {
        key: "hb33-greenbook",
        title: Some("H.B. 33 of the 135th General Assembly — LSC Greenbook, Department of Education"),
        url: "https://www.lsc.ohio.gov/assets/legislation/135/hb33/en0/files/\
              hb33-edu-greenbook-as-enacted-135th-general-assembly.pdf",
        filename: "hb33-greenbook.pdf",
        format: Format::Pdf,
        catalog: Some("lsc-dew-redbook"),
        fixtures: &[crate::fixtures::LSC_GREENBOOK_FIXTURE],
        note: "FY2024-25. `as-enacted` rather than `as-enrolled` in the path, and `en0` rather \
               than `en` — LSC changed both conventions at the 135th, which is why guessing \
               one naming across the series returns a 404 that looks like a missing document.",
    },
    Source {
        key: "hb95-greenbook",
        title: None,
        url: "https://www.lsc.ohio.gov/assets/legislation/125/hb95/en/files/\
              hb95-edu-greenbook-as-enrolled-125th-general-assembly.pdf",
        filename: "hb95-greenbook.pdf",
        format: Format::Pdf,
        catalog: Some("lsc-dew-redbook"),
        fixtures: &[
            crate::fixtures::APPROPRIATION_FIXTURE,
            crate::fixtures::LSC_GREENBOOK_FIXTURE,
        ],
        note: "FY2004-FY2005 as enrolled, with the three preceding years as actuals. \
               Read by clustering the right edges of the figures, because the table's own \
               header labels sit narrower than the columns beneath them.",
    },
    Source {
        key: "hb59-greenbook",
        title: Some("H.B. 59 of the 130th General Assembly — LSC Greenbook, Department of Education"),
        url: "https://www.lsc.ohio.gov/assets/legislation/130/hb59/en/files/\
              hb59-edu-greenbook-as-enrolled-130th-general-assembly.pdf",
        filename: "hb59-greenbook.pdf",
        format: Format::Pdf,
        catalog: Some("lsc-dew-redbook"),
        fixtures: &[crate::fixtures::LSC_GREENBOOK_FIXTURE],
        note: "FY2014-FY2015, and the act that put a formula back on top of the Bridge \
               guarantee: the state share index, the Opportunity Grant, and seven categorical \
               aids. Its `Deductions and Transfers` chapter is the only committed account here \
               of the deduct-era mechanism `dew-payment-reports` is blocked on.",
    },
    Source {
        key: "hb64-greenbook",
        title: Some("H.B. 64 of the 131st General Assembly — LSC Greenbook, Department of Education"),
        url: "https://www.lsc.ohio.gov/assets/legislation/131/hb64/en/files/\
              hb64-edu-greenbook-as-enrolled-131st-general-assembly.pdf",
        filename: "hb64-greenbook.pdf",
        format: Format::Pdf,
        catalog: Some("lsc-dew-redbook"),
        fixtures: &[crate::fixtures::LSC_GREENBOOK_FIXTURE],
        note: "FY2016-FY2017. The second design over the same guarantee, which is the pair \
               `funding-regime/bridge-formula` names when it asks whether the overlays are \
               separate regimes or one continuous mechanism.",
    },
    Source {
        key: "hb49-greenbook",
        title: Some("H.B. 49 of the 132nd General Assembly — LSC Greenbook, Department of Education"),
        url: "https://www.lsc.ohio.gov/assets/legislation/132/hb49/en/files/\
              hb49-edu-greenbook-as-enrolled-132nd-general-assembly.pdf",
        filename: "hb49-greenbook.pdf",
        format: Format::Pdf,
        catalog: Some("lsc-dew-redbook"),
        fixtures: &[crate::fixtures::LSC_GREENBOOK_FIXTURE],
        note: "FY2018-FY2019.",
    },
    Source {
        key: "hb166-greenbook",
        title: Some("H.B. 166 of the 133rd General Assembly — LSC Greenbook, Department of Education"),
        url: "https://www.lsc.ohio.gov/assets/legislation/133/hb166/en/files/\
              hb166-edu-greenbook-as-enrolled-133rd-general-assembly.pdf",
        filename: "hb166-greenbook.pdf",
        format: Format::Pdf,
        catalog: Some("lsc-dew-redbook"),
        fixtures: &[crate::fixtures::LSC_GREENBOOK_FIXTURE],
        note: "FY2020-FY2021, the last biennium before the Fair School Funding Plan and the \
               one FY2020 the plan's temporary transitional aid guarantee holds districts at. \
               What that guarantee preserves is decided here.",
    },
    Source {
        key: "hb119-greenbook",
        title: None,
        url: "https://www.lsc.ohio.gov/assets/legislation/127/hb119/en/files/\
              hb119-edu-greenbook-as-enrolled-127th-general-assembly.pdf",
        filename: "hb119-greenbook.pdf",
        format: Format::Pdf,
        catalog: Some("lsc-dew-redbook"),
        fixtures: &[
            crate::fixtures::APPROPRIATION_FIXTURE,
            crate::fixtures::LSC_GREENBOOK_FIXTURE,
        ],
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
        fixtures: &[
            crate::fixtures::APPROPRIATION_FIXTURE,
            crate::fixtures::LSC_GREENBOOK_FIXTURE,
        ],
        note: "FY2010-FY2011 as enrolled, with the three preceding years as actuals. \
               Read by clustering the right edges of the figures, because the table's own \
               header labels sit narrower than the columns beneath them.",
    },
    ],
};
