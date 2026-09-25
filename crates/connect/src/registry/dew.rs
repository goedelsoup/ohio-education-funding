//! Ohio Department of Education and Workforce, including two of its offices.
//!
//! The publisher of the formula itself and of most of what the corpus measures: the foundation
//! funding calculator, the report card, the five-year forecasts, and — from the Office for Child
//! Nutrition and the Office for School and District Improvement — two series that answer
//! questions the funding files cannot.

use super::{Connector, Format, Source, Status};

pub(super) const FOUNDATION: Connector = Connector {
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
            key: "fy25-cs-calculator",
            title: Some("FY25 Community/STEM School State Foundation Funding Simulator"),
            // The FY2025 sibling of `fy27-cs-calculator`, from the Internet Archive because the
            // department replaced it in place. Unlike the FY2025 *traditional* calculator, whose
            // captures are truncated at a megabyte, this one was archived whole and reproduces
            // the department's own arithmetic; see `decisions/an-archived-source-is-still-a-source`.
            url: "https://web.archive.org/web/20250222101322id_/\
                  https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Community-School-Funding/\
                  FY25-CS-State-Foundation-Funding-Calculator-3-7-2024-1.xlsx.aspx?lang=en-US",
            filename: "fy25-cs-calculator.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-fy25-community-school-calculator"),
            fixtures: &[crate::fixtures::FY25_COMMUNITY_SCHOOL_FUNDING_FIXTURE],
            note: "The year H.B. 33 set the equity supplement at $650, read from the \
                   department's own model rather than from the act alone. Same two data sheets \
                   as the FY2027 sibling and the same identity to check against — rate times \
                   enrolled ADM for site-based schools, zero for e-schools and STEM schools — \
                   over a different column layout and a designation column that spells STEM \
                   `S` rather than `STEM`. It carries no base funding supplement: that line \
                   did not exist in FY2025.",
        },
        Source {
            key: "fy27-cs-calculator",
            title: Some("FY27 Community/STEM School State Foundation Funding Simulator"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Community-School-Funding/\
                  FY27-CS-State-Foundation-Funding-Calculator-12-24-2025.xlsx.aspx\
                  ?lang=en-US",
            filename: "fy27-cs-calculator.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-fy27-community-school-calculator"),
            fixtures: &[crate::fixtures::COMMUNITY_SCHOOL_FUNDING_FIXTURE],
            note: "The sibling of `fy27-calculator`, one directory across, and the only \
                   per-*school* formula source in this registry. Same office, same release \
                   calendar, same workbook shape; a different population, which is why it is a \
                   source here and not a wider version of the district model. Read for the \
                   community school equity supplement — R.C. 3317.022 as H.B. 96 codified it — \
                   which no district panel can carry because no district receives it. Its two \
                   data sheets are `Detail SFPR` and `Summary_SFPR`, and the space-versus- \
                   underscore rule that tells display sheets from data sheets in the district \
                   workbook does **not** hold here: see the catalog entry, and \
                   `fixtures::community_schools` for the hidden sheet whose \
                   `H. Equity Supplement` column holds total state support instead.",
        },
        Source {
            key: "cupp-fy24",
            title: None,
            // The department's **revised** FY2024 edition, adopted by #439 a year after the
            // original was connected. `...-Final-12-12-2024.xlsx` still resolves and still
            // serves its pinned bytes; it is not kept because the revision corrects a real
            // defect rather than reissuing the file. See the catalog entry for what moved.
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/District-Profile-Reports/\
                  FY2024-District-Profile-Report/\
                  FY24-District-Profile-Report-Revised-12-18-2025.xlsx.aspx?lang=en-US",
            filename: "cupp-fy24.xlsx",
            format: Format::Xlsx,
            catalog: Some("cupp-district-profile-report"),
            fixtures: &[
                crate::fixtures::FY27_FIXTURE,
                crate::fixtures::PROFILE_FIXTURE,
                crate::fixtures::GRADE_BANDS_FIXTURE,
            ],
            note: "60 variables per district. Fiscal and tax years are mixed within a row. \
                   The connected edition is the department's 12-18-2025 revision, adopted \
                   by #439 over the original 12-12-2024 file.",
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
        Source {
            key: "fy25-payment-report",
            title: Some("FY2025 Final Traditional District Foundation Payment Report"),
            // Openly served, and the only complete FY2025 artefact there is. The FY2025
            // calculator was replaced in place like FY2026's, but its archive captures are
            // truncated at exactly 1 MiB — 1,033,192 bytes stored against FY2026's 4,187,092 —
            // so the workbook cannot be read from the Wayback Machine at all.
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Traditional-School-Districts/Districts-Payment-Reports-in-Excel/\
                  2025-11-07_TRAD_Foundation_Payment-1.xlsx.aspx?lang=en-US",
            filename: "fy25-payment-report.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-sfpr-line-by-line"),
            fixtures: &[crate::fixtures::FY25_FIXTURE],
            note: "`Final #2`, published 7 November 2025 — what districts were actually paid \
                   for FY2025, after the year closed, rather than what a calculator projected \
                   before it. That makes it the right baseline for a biennium comparison and \
                   not merely the available one. It also carries `[J] Supplemental Targeted \
                   Assistance` per district: the tier H.B. 96 repealed, which \
                   `fsfp-targeted-assistance` records as a live zero whose payment was never \
                   computed from its rate. This is the last year it was paid. Its bracketed \
                   column letters are **not** FY2026's — the scheme shifted when the act \
                   removed one line and added two — so the extractor matches label text.",
        },
        Source {
            key: "fy19-payment-report",
            title: Some("FY2019 Final Traditional District Foundation Payment Report"),
            // Openly served, anonymously, and never behind the reports portal that gates the
            // legacy series. `dew-payment-reports` records 1999-2021 as OH|ID-only; that is true
            // of the portal's *Foundation Legacy Payment Reports* collection and not of the
            // per-year files still attached to this page, which reach back to FY2016.
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Traditional-School-Districts/Districts-Payment-Reports-in-Excel/\
                  FY19_SFPR_FIN_2.xlsx.aspx?lang=en-US",
            filename: "fy19-payment-report.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-fy19-payment-report"),
            fixtures: &[crate::fixtures::FY19_PAYMENT_REPORT_FIXTURE],
            note: "`Final #2`, the last FY2019 settlement, under the formula the Fair School \
                   Funding Plan replaced. Retrieved for `TRANSITIONAL GUARANTEE` per district: \
                   the one column that splits `[L1]` into the guarantee it contains and the \
                   funding it would contain without one, which is what the third reading of \
                   H.B. 110 Section 265.225 needs and no FY2021 file carries. One sheet, 133 \
                   columns, 612 districts — one more than every later file, because Newbury \
                   Local SD had not yet dissolved into West Geauga.",
        },
        Source {
            key: "fy21-funding-bases",
            title: Some("Foundation Funding Bases"),
            // Unlinked from the current page and still served at its own address. The department
            // published it to explain the bases the two transitional provisions compare against
            // and has since replaced the page around it; `docProps` dates the bytes to a
            // 16 June 2022 revision of a 12 January 2022 file.
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Traditional-School-Districts/\
                  Foundation-Funding-Bases-6-16-2022.xlsx.aspx?lang=en-US",
            filename: "foundation-funding-bases-2022.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-foundation-funding-bases"),
            fixtures: &[crate::fixtures::FY21_FUNDING_BASE_FIXTURE],
            note: "Six sheets, of which the sixth is the one the corpus could not do without: \
                   `FY21 Funding Base` decomposes `[L1]` into seven terms whose first is the \
                   FY2019 total foundation funding, district for district. That identity is the \
                   licence for treating the guarantee as a term *inside* the formula transition \
                   supplement’s base rather than a sibling of it, and the extractor refuses a \
                   file that no longer reproduces it.",
        },
        Source {
            key: "fy26-calculator",
            title: Some("FY26 TRAD State Foundation Funding Calculator"),
            // The only address this file still has. The department replaced it in place when the
            // FY2027 model went up; see `decisions/an-archived-source-is-still-a-source`.
            url: "https://web.archive.org/web/20251008001131id_/\
                  https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Traditional-School-Districts/\
                  FY26-TRAD-State-Foundation-Funding-Calculator-9-22-2025.xlsx.aspx?lang=en-US",
            filename: "fy26-calculator.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-fy26-funding-calculator"),
            fixtures: &[
                crate::fixtures::TRANSPORT_RATES_FIXTURE,
                crate::fixtures::FY26_FIXTURE,
                crate::fixtures::CALCULATOR_SCALARS_FIXTURE,
                crate::fixtures::CALCULATOR_COUNTS_FIXTURE,
                crate::fixtures::CALCULATOR_VINTAGES_FIXTURE,
                crate::fixtures::CALCULATOR_ADM_FIXTURE,
            ],
            note: "The FY2026 model, from the Internet Archive because the department serves \
                   no copy. Five fixtures: the statewide transportation factors, sixteen \
                   per-district columns that let the FY2027 model be read against it, the \
                   statewide scalars both years state, every count the plan multiplies, the \
                   department's own table of where each input came from, and enrolled ADM \
                   across the two overlapping three-year windows. The note \
                   here said those tables restate FY2027's and are not committed; per-pupil \
                   local capacity moves 9.3% between the two years and DPIA aid moves -8.6%, \
                   so they do not.",
        },
        Source {
            key: "sfpr-line-by-line-fy21",
            title: Some("FY 2021 School Finance Payment Report Line by Line Explanation"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Traditional-School-Districts/\
                  FY21-SFPR-Funding-Form-Line-by-Line-Explanation.pdf.aspx?lang=en-US",
            filename: "fy21-sfpr-funding-form-line-by-line-explanation.pdf",
            format: Format::Pdf,
            catalog: Some("dew-sfpr-line-by-line"),
            fixtures: &[crate::fixtures::SFPR_FIXTURE],
            note: "The first edition, published under the Fair School Funding Plan's opening year.",
        },
        Source {
            key: "sfpr-line-by-line-fy22",
            title: Some("FY 2022 School Finance Payment Report Line by Line Explanation"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Traditional-School-Districts/\
                  FY22-SFPR-Funding-Line-by-Line-Explanation-7-18-2022.pdf.aspx?lang=en-US",
            filename: "fy22-sfpr-funding-line-by-line-explanation-7-18-2022.pdf",
            format: Format::Pdf,
            catalog: Some("dew-sfpr-line-by-line"),
            fixtures: &[crate::fixtures::SFPR_FIXTURE],
            note: "The edition most of this corpus's verified base-cost figures came from.",
        },
        Source {
            key: "sfpr-line-by-line-fy23",
            title: Some("FY 2023 School Finance Payment Report Line by Line Explanation"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Traditional-School-Districts/\
                  FY23-SFPR-Funding-Line-by-Line-Explanation-2-23-2023.pdf.aspx?lang=en-US",
            filename: "fy23-sfpr-funding-line-by-line-explanation-2-23-2023.pdf",
            format: Format::Pdf,
            catalog: Some("dew-sfpr-line-by-line"),
            fixtures: &[crate::fixtures::SFPR_FIXTURE],
            note: "The last edition published as the Ohio Department of Education.",
        },
        Source {
            key: "sfpr-line-by-line-fy24",
            title: Some("FY 2024 School Finance Payment Report Line by Line Explanation"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Traditional-School-Districts/\
                  FY24-SFPR-Funding-Line-by-Line-Explanation-11-2-2023.pdf.aspx?lang=en-US",
            filename: "fy24-sfpr-funding-line-by-line-explanation-11-2-2023.pdf",
            format: Format::Pdf,
            catalog: Some("dew-sfpr-line-by-line"),
            fixtures: &[crate::fixtures::SFPR_FIXTURE],
            note: "The first published as the Department of Education and Workforce.",
        },
        Source {
            key: "sfpr-line-by-line-fy25",
            title: Some("FY 2025 School Finance Payment Report Line by Line Explanation"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Traditional-School-Districts/\
                  FY25-SFPR-Funding-Line-by-Line-Explanation-10-25-2024.pdf.aspx?lang=en-US",
            filename: "fy25-sfpr-funding-line-by-line-explanation-10-25-2024.pdf",
            format: Format::Pdf,
            catalog: Some("dew-sfpr-line-by-line"),
            fixtures: &[crate::fixtures::SFPR_FIXTURE],
            note: "The year whose reported costs set the FY2026 transportation rates.",
        },
        Source {
            key: "sfpr-line-by-line-fy26",
            title: Some("FY 2026 School Finance Payment Report Line by Line Explanation"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Traditional-School-Districts/\
                  FY26-SFPR-Funding-Line-by-Line-Explanation-9-29-2025.pdf.aspx?lang=en-US",
            filename: "fy26-sfpr-funding-line-by-line-explanation-9-29-2025.pdf",
            format: Format::Pdf,
            catalog: Some("dew-sfpr-line-by-line"),
            fixtures: &[crate::fixtures::SFPR_FIXTURE],
            note: "States the 180 days of service that annualise reported daily miles, and that the density supplement is prorated to fit the biennial appropriation.",
        },
        Source {
            key: "jvsd-payment-fy22",
            title: Some("FY22 JVSD Foundation Payment Report (final #3)"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Career-Tech-Planning-and-Funding/Career-Tech-Payment-Reports/\
                  Career-Tech-Payment-Reports-in-Excel-Format/\
                  FY22_final-3_JVSD_Foundation_Payment.xlsx.aspx?lang=en-US",
            filename: "jvsd-fy22.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-jvsd-foundation-payments"),
            fixtures: &[crate::fixtures::JVSD_FUNDING_FIXTURE],
            note: "Paid February 15, 2023. The plan's opening year, and the only one whose `Base Cost` sheet omits per-pupil base \
                   cost: 72 columns where every later year has 76. Prior law never divided by a \
                   pupil, so the column had nothing to do until H.B. 96 gave it a job.",
        },
        Source {
            key: "jvsd-payment-fy23",
            title: Some("FY23 JVSD Foundation Payment Report (final #2)"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Career-Tech-Planning-and-Funding/Career-Tech-Payment-Reports/\
                  Career-Tech-Payment-Reports-in-Excel-Format/\
                  FY23_Final2_JVSD_Foundation_Payment.xlsx.aspx?lang=en-US",
            filename: "jvsd-fy23.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-jvsd-foundation-payments"),
            fixtures: &[crate::fixtures::JVSD_FUNDING_FIXTURE],
            note: "Paid November 15, 2023. The one year written in plain labels — `State Share Percentage`, no bracket tag and no \
                   CamelCase. Both neighbours tag their columns and FY2022 runs them together, so \
                   no single spelling reads the series.",
        },
        Source {
            key: "jvsd-payment-fy24",
            title: Some("FY24 JVSD Foundation Payment Report (final #2)"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Career-Tech-Planning-and-Funding/Career-Tech-Payment-Reports/\
                  Career-Tech-Payment-Reports-in-Excel-Format/\
                  FY24_Final2_JVSD_Foundation_Payment-1.xlsx.aspx?lang=en-US",
            filename: "jvsd-fy24.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-jvsd-foundation-payments"),
            fixtures: &[crate::fixtures::JVSD_FUNDING_FIXTURE],
            note: "Paid November 15, 2024. The first year tagged `[A1]`…`[I]`, and the reason the tags cannot be trusted across \
                   the series: here `[I]` is total state support and in FY2026 `[I]` is the base \
                   funding supplement, a $559m column and a $1.4m one under one letter.",
        },
        Source {
            key: "jvsd-payment-fy25",
            title: Some("FY25 JVSD Foundation Payment Report (final #2)"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Career-Tech-Planning-and-Funding/Career-Tech-Payment-Reports/\
                  Career-Tech-Payment-Reports-in-Excel-Format/\
                  FY25_Final-2_JVSD_Foundation_Payment.xlsx.aspx?lang=en-US",
            filename: "jvsd-fy25.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-jvsd-foundation-payments"),
            fixtures: &[crate::fixtures::JVSD_FUNDING_FIXTURE],
            note: "Paid November 14, 2025. The last year under prior law, which is what makes it the baseline item 7 moved from. \
                   Its `Detailed SFPR` has no per-pupil base cost, no local capacity per-pupil and \
                   no base cost enrolled ADM: those three columns appear in FY2026 and are the \
                   schema's record of the amendment.",
        },
        Source {
            key: "jvsd-payment-fy26",
            title: Some("FY26 JVSD Foundation Payment Report (final #1)"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Career-Tech-Planning-and-Funding/Career-Tech-Payment-Reports/\
                  Career-Tech-Payment-Reports-in-Excel-Format/\
                  FY26_Final-1_JVSD_FL1_Foundation_Payment.xlsx.aspx?lang=en-US",
            filename: "jvsd-fy26.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-jvsd-foundation-payments"),
            fixtures: &[crate::fixtures::JVSD_FUNDING_FIXTURE],
            note: "Paid August 14, 2026. The first year under H.B. 96 and the only closed one, which makes it the single year \
                   the new method can be priced against its own counterfactual. Its enrolled ADM \
                   is this year's, unlike FY2027's.",
        },
        Source {
            key: "jvsd-payment-fy27",
            title: Some("FY27 JVSD Foundation Payment Report (September)"),
            url: "https://education.ohio.gov/getattachment/Topics/Finance-and-Funding/\
                  School-Payment-Reports/State-Funding-For-Schools/\
                  Career-Tech-Planning-and-Funding/Career-Tech-Payment-Reports/\
                  Career-Tech-Payment-Reports-in-Excel-Format/\
                  FY27_JVSD_SEP_Foundation_Payment.xlsx.aspx?lang=en-US",
            filename: "jvsd-fy27.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-jvsd-foundation-payments"),
            fixtures: &[crate::fixtures::JVSD_FUNDING_FIXTURE],
            note: "Paid September 15, 2026. The first payment of an open year, not a final. **Its `[a] Enrolled ADM` is FY2026's \
                   on 43 of the 49 districts** and equals base cost enrolled ADM on 39, because \
                   FY2027 enrolment does not exist in September 2026. Every level here reads, and \
                   any quantity the current-year ADM multiplies is provisional — which under item \
                   7, and not under prior law, is the state share of base cost itself.",
        },
    ],
};

pub(super) const FACTS_AND_FIGURES: Connector = Connector {
    key: "dew-facts-and-figures",
    publisher: "Ohio Department of Education and Workforce",
    feeds: &["metric", "program"],
    status: Status::Wired {
        still_blocked: Some(
            "Only the current edition. The department replaces the sheet in place and the \
             prior-year URLs return an identical 1,245-byte 404, so this is a series with one \
             retrievable member.",
        ),
    },
    note: "The annual fact sheet, and the only machine-reachable place the department \
           publishes a home-education count.",
    sources: &[Source {
        key: "education-landscape-2024",
        title: None,
        url: "https://education.ohio.gov/getattachment/Topics/Data/\
              Frequently-Requested-Data/Facts-and-Figures/\
              Ohios-Education-Landscape-2023-2024.pdf.aspx?lang=en-US",
        filename: "education-landscape-2024.pdf",
        format: Format::Pdf,
        catalog: Some("dew-education-landscape"),
        fixtures: &[crate::fixtures::LANDSCAPE_FIXTURE],
        note: "Two pages of statewide counts for 2023-2024. Its `School Options` table is \
               the one that matters here: thirteen channels counted the same way in the same \
               year, including the home-education figure that exists nowhere else.",
    }],
};

pub(super) const REPORT_CARD: Connector = Connector {
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
        // --- Career-technical planning district membership. The only source here that is an
        // API rather than a file, and the only place a CTPD's members are published at all: the
        // report card's static catalogue carries twenty-four CTPD files back to 2013 and every
        // one of them is ratings. See `fixtures::ctpd`.
        Source {
            key: "report-card-org-index",
            title: Some("Ohio School Report Cards organisation index"),
            url: "https://reportcardstorage.education.ohio.gov/2026appsettings/\
                  live-search.json?sv=2020-08-04&ss=b&srt=sco&sp=rlx\
                  &se=2031-07-28T05:10:18Z&st=2021-07-27T21:10:18Z&spr=https\
                  &sig=nPOvW%2Br2caitHi%2F8WhYwU7xqalHo0dFrudeJq%2B%2Bmyuo%3D",
            filename: "report-card-org-index.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Every organisation the report card rates \u{2014} 3,230 schools, 607 \
                   districts, 90 planning districts, 90 dorps and one STEM school \u{2014} with \
                   its IRN, its kind and its name. Retrieved for the last of those: a roster \
                   names its members and gives no IRN, so this is what resolves them, and it is \
                   the same publisher rather than a match against this repository's own panel.",
        },
        Source {
            key: "ctpd-roster-021357",
            title: Some("Bedford City Comprehensive CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/021357/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-021357.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200001",
            title: Some("Apollo JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200001/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200001.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 11 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200002",
            title: Some("Lima City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200002/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200002.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 2 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200003",
            title: Some("Heartland Technical Education Center CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200003/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200003.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 5 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200004",
            title: Some("Ashtabula County Technical and Career Center CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200004/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200004.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 7 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200005",
            title: Some("Tri-County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200005/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200005.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 8 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200006",
            title: Some("Belmont-Harrison Area JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200006/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200006.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 8 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200007",
            title: Some("Southern Hills JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200007/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200007.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 6 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200008",
            title: Some("Butler County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200008/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200008.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 11 districts and 5 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200011",
            title: Some("Springfield-Clark County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200011/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200011.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 7 districts and 5 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200012",
            title: Some("U S Grant JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200012/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200012.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 4 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200015",
            title: Some("Columbiana County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200015/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200015.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 9 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200016",
            title: Some("East Liverpool City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200016/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200016.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200019",
            title: Some("Cleveland Municipal CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200019/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200019.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 55 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200020",
            title: Some("Tri-Heights Career Prep Consortium CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200020/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200020.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 4 districts and 4 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200021",
            title: Some("East Cleveland City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200021/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200021.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200023",
            title: Some("Lakewood City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200023/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200023.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 4 districts and 2 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200025",
            title: Some("Mayfield Excel T.E.C.C. CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200025/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200025.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 10 districts and 2 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200026",
            title: Some("Parma City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200026/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200026.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 4 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200027",
            title: Some("Cuyahoga Valley JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200027/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200027.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 8 districts and 5 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200028",
            title: Some("Polaris JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200028/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200028.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 6 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200030",
            title: Some("Four County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200030/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200030.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 22 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200031",
            title: Some("Delaware County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200031/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200031.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 5 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200032",
            title: Some("EHOVE JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200032/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200032.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 13 districts and 2 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200033",
            title: Some("Sandusky City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200033/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200033.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200035",
            title: Some("Columbus City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200035/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200035.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 4 districts and 77 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200036",
            title: Some("Eastland JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200036/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200036.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 16 districts and 2 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200037",
            title: Some("Tolles Career & Technical Center CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200037/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200037.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 7 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200038",
            title: Some("South-Western City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200038/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200038.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200039",
            title: Some("Penta County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200039/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200039.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 16 districts and 3 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200040",
            title: Some("Gallia-Jackson-Vinton JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200040/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200040.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 6 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200042",
            title: Some("Greene County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200042/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200042.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 7 districts and 3 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200043",
            title: Some("Cincinnati City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200043/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200043.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 19 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200044",
            title: Some("Great Oaks Career Campuses CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200044/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200044.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 36 districts and 3 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200050",
            title: Some("Jefferson County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200050/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200050.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 5 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200051",
            title: Some("Knox County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200051/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200051.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 6 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200052",
            title: Some("Auburn JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200052/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200052.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 10 districts and 3 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200053",
            title: Some("Lake Shore Compact CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200053/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200053.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 3 districts and 3 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200054",
            title: Some("Lawrence County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200054/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200054.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 7 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200055",
            title: Some("Licking County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200055/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200055.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 10 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200056",
            title: Some("Ohio Hi-Point JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200056/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200056.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 14 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200057",
            title: Some("Lorain City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200057/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200057.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 9 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200058",
            title: Some("Lorain County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200058/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200058.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 13 districts and 3 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200059",
            title: Some("Oregon City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200059/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200059.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200060",
            title: Some("Sylvania City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200060/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200060.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 2 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200061",
            title: Some("Toledo City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200061/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200061.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 24 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200062",
            title: Some("Washington Local CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200062/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200062.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200063",
            title: Some("Mahoning County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200063/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200063.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 13 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200064",
            title: Some("Youngstown City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200064/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200064.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 11 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200066",
            title: Some("Medina County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200066/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200066.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 6 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200067",
            title: Some("Meigs Local CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200067/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200067.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 3 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200069",
            title: Some("Upper Valley JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200069/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200069.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 13 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200070",
            title: Some("Switzerland of Ohio Local CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200070/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200070.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200071",
            title: Some("Dayton City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200071/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200071.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 23 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200072",
            title: Some("Mad River Local CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200072/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200072.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200073",
            title: Some("Miami Valley Career Technology JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200073/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200073.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 27 districts and 2 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200074",
            title: Some("Morgan Local CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200074/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200074.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200075",
            title: Some("Mid-East Ohio JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200075/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200075.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 13 districts and 3 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200077",
            title: Some("Pike County Career Technology Center CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200077/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200077.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 4 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200078",
            title: Some("Maplewood Area JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200078/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200078.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 10 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200081",
            title: Some("Madison Local CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200081/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200081.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200082",
            title: Some("Mansfield City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200082/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200082.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 6 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200083",
            title: Some("Pioneer CTC JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200083/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200083.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 14 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200084",
            title: Some("Pickaway-Ross County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200084/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200084.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 10 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200085",
            title: Some("Vanguard-Sentinel JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200085/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200085.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 13 districts and 3 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200086",
            title: Some("Scioto County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200086/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200086.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 10 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200089",
            title: Some("Alliance City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200089/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200089.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 3 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200090",
            title: Some("Canton City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200090/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200090.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 7 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200092",
            title: Some("Massillon City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200092/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200092.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200093",
            title: Some("Stark County Career Compact CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200093/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200093.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 4 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200094",
            title: Some("Stark County Area JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200094/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200094.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 6 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200095",
            title: Some("Akron City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200095/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200095.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district and 17 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200096",
            title: Some("Four City Compact CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200096/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200096.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 4 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200097",
            title: Some("Six District Voc Ed Compact CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200097/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200097.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 6 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200098",
            title: Some("Portage Lakes JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200098/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200098.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 4 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200099",
            title: Some("Trumbull County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200099/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200099.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 20 districts and 6 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200101",
            title: Some("Buckeye JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200101/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200101.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 11 districts and 2 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200102",
            title: Some("Vantage JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200102/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200102.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 12 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200103",
            title: Some("Warren County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200103/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200103.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 6 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200104",
            title: Some("Washington County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200104/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200104.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 6 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200105",
            title: Some("Wayne County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200105/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200105.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 10 districts and 3 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200107",
            title: Some("Ohio Valley Local CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200107/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200107.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 2 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200108",
            title: Some("Coshocton County JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200108/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200108.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 3 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200111",
            title: Some("Tri-Rivers JVSD CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200111/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200111.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 9 districts and 4 community or STEM schools.",
        },
        Source {
            key: "ctpd-roster-200115",
            title: Some("Canton Local CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200115/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200115.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 4 districts, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200116",
            title: Some("Millstream Career Cooperative CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200116/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200116.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 14 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200117",
            title: Some("Greenville City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200117/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200117.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200119",
            title: Some("Lancaster City CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200119/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200119.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 1 district, and no community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200120",
            title: Some("Centerville-Kettering-Oakwood CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200120/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200120.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 3 districts and 1 community or STEM school.",
        },
        Source {
            key: "ctpd-roster-200121",
            title: Some("Tri Star CTPD"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/200121/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-roster-200121.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "FY2025: 9 districts, and no community or STEM school.",
        },
        // --- The forty-seven districts whose normalised name another district shares. A roster
        // names its members and gives no IRN, and nineteen names are shared by two or three
        // districts; each of these carries its own `ctpdIrn`, which places it. ---
        Source {
            key: "ctpd-place-000442",
            title: Some("Manchester Local - Adams County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  000442/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-000442.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-045781",
            title: Some("Perry Local - Allen County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  045781/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-045781.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-045856",
            title: Some("Buckeye Local - Ashtabula County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  045856/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-045856.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-046037",
            title: Some("Eastern Local School District - Brown County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  046037/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-046037.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-046110",
            title: Some("Lakota Local - Butler County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  046110/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-046110.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-046128",
            title: Some("Madison Local - Butler County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  046128/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-046128.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-046250",
            title: Some("Northeastern Local - Clark County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  046250/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-046250.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-046268",
            title: Some("Northwestern Local - Clark County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  046268/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-046268.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-046276",
            title: Some("Southeastern Local - Clark County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  046276/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-046276.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-046433",
            title: Some("Crestview Local - Columbiana County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  046433/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-046433.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-046441",
            title: Some("Southern Local - Columbiana County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  046441/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-046441.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-046722",
            title: Some("Northeastern Local - Defiance County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  046722/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-046722.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-047365",
            title: Some("Northwest Local - Hamilton County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  047365/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-047365.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-047746",
            title: Some("Western Reserve Local - Huron County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  047746/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-047746.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-047787",
            title: Some("Buckeye Local - Jefferson County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  047787/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-047787.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-047886",
            title: Some("Madison Local - Lake County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  047886/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-047886.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-047894",
            title: Some("Riverside Local - Lake County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  047894/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-047894.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-047902",
            title: Some("Perry Local - Lake County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  047902/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-047902.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048033",
            title: Some("Northridge Local - Licking County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048033/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048033.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048090",
            title: Some("Riverside Local - Logan County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048090/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048090.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048223",
            title: Some("Springfield Local - Lucas County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048223/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048223.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048371",
            title: Some("Springfield Local - Mahoning County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048371/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048371.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048397",
            title: Some("Western Reserve Local - Mahoning County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048397/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048397.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048470",
            title: Some("Buckeye Local - Medina County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048470/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048470.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048496",
            title: Some("Highland Local - Medina County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048496/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048496.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048512",
            title: Some("Eastern Local - Meigs County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048512/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048512.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048538",
            title: Some("Southern Local - Meigs County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048538/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048538.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048736",
            title: Some("Northridge Local - Montgomery County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048736/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048736.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-048801",
            title: Some("Highland Local - Morrow County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  048801/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-048801.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049064",
            title: Some("Southern Local - Perry County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049064/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049064.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049122",
            title: Some("Eastern Local School District - Pike County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049122/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049122.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049429",
            title: Some("Crestview Local - Richland County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049429/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049429.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049452",
            title: Some("Madison Local - Richland County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049452/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049452.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049528",
            title: Some("Southeastern Local - Ross County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049528/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049528.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049569",
            title: Some("Lakota Local - Sandusky County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049569/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049569.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049619",
            title: Some("Green Local - Scioto County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049619/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049619.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049635",
            title: Some("Northwest Local - Scioto County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049635/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049635.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049866",
            title: Some("Lake Local - Stark County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049866/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049866.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049908",
            title: Some("Northwest Local - Stark County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049908/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049908.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-049924",
            title: Some("Perry Local - Stark County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  049924/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-049924.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-050005",
            title: Some("Manchester Local - Summit County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  050005/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-050005.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-050013",
            title: Some("Green Local - Summit County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  050013/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-050013.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-050062",
            title: Some("Springfield Local - Summit County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  050062/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-050062.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-050351",
            title: Some("Crestview Local - Van Wert County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  050351/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-050351.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-050559",
            title: Some("Green Local - Wayne County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  050559/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-050559.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-050575",
            title: Some("Northwestern Local - Wayne County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  050575/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-050575.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
        Source {
            key: "ctpd-place-050690",
            title: Some("Lake Local - Wood County"),
            url: "https://reportcarddataapi.education.ohio.gov/api/GetByIrn/\
                  050690/2025?code={REPORT_CARD_KEY}",
            filename: "ctpd-place-050690.json",
            format: Format::Json,
            catalog: Some("dew-ctpd-membership"),
            fixtures: &[crate::fixtures::CTPD_MEMBERSHIP_FIXTURE],
            note: "Its name is one of the nineteen two or more districts share, so a roster \
                   naming it does not say which. Its own record carries `ctpdIrn` and places it.",
        },
    ],
};

pub(super) const FIVE_YEAR_FORECAST: Connector = Connector {
    key: "dew-five-year-forecast",
    publisher: "Ohio Department of Education and Workforce",
    feeds: &[
        "education-agency",
        "revenue-stream",
        "metric",
        "fiscal-period",
    ],
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
};

pub(super) const TYPOLOGY: Connector = Connector {
    key: "dew-typology",
    publisher: "Ohio Department of Education and Workforce",
    feeds: &["education-agency"],
    status: Status::Wired {
        still_blocked: None,
    },
    note: "The department's own similar-district grouping, which this corpus declared as a \
           property, characterised in prose on five agency nodes, and recorded as unfilled on \
           every one of them.",
    sources: &[Source {
        key: "district-typology-2013",
        title: Some("Typology of Ohio School Districts"),
        url: "https://education.ohio.gov/getattachment/Topics/Data/Frequently-Requested-Data/\
              Typology-of-Ohio-School-Districts/2013-School-District-Typology.xlsx.aspx",
        filename: "district-typology-2013.xlsx",
        format: Format::Xlsx,
        catalog: Some("dew-district-typology"),
        fixtures: &[crate::fixtures::TYPOLOGY_FIXTURE],
        note: "Eight codes ordered by urbanicity, plus a zero for the districts the department \
               removes from the analysis. The assignment is 2013 and has not been revised since, \
               so the four measures it was made on are carried beside it under their own vintage.",
    }],
};

pub(super) const PAYMENT_REPORTS: Connector = Connector {
    key: "dew-payment-reports",
    publisher: "Ohio Department of Education and Workforce",
    feeds: &["program", "education-agency", "revenue-stream"],
    status: Status::Declared {
        blocked_on: "the deduct-era reports (1999-2021) are behind OH|ID authentication on \
                     the department's reports portal; the current-era ones are open and \
                     indexed but post-date the deduction entirely. The portal is an obstacle \
                     to an unattended agent and not to a person who holds an OH|ID account, so \
                     wiring this means a human fetching the files into the cache and a refresh \
                     path that says so — `edfund-connect fetch` will not re-run it",
    },
    note: "The one source that would carry the voucher and community-school deduction per \
           resident district, for the years it existed. Kept separate from \
           `dew-scholarship-reports`, which is the department's *public* account of the same \
           channel: one is a per-district file behind a login, the other a statewide \
           aggregate anyone can fetch, and collapsing them would make this blocker look \
           half-lifted when nothing about it has moved.",
    sources: &[],
};

pub(super) const SCHOLARSHIP_REPORTS: Connector = Connector {
    key: "dew-scholarship-reports",
    publisher: "Ohio Department of Education and Workforce",
    feeds: &["program", "education-agency", "school"],
    status: Status::Wired {
        still_blocked: Some(
            "wired for everything the department publishes openly about this channel: the \
             current statewide aggregates, the archived participation series FY1997-FY2013, and \
             per-building eligibility for three editions of the designated list. Two holes are \
             left, and they are different in kind. Per-district *participation* is still a file \
             nobody has fetched — the annual report cites two routes for it on the department's \
             reports portal, which answers every path with the same application shell and \
             serves its reports as Power BI embeds behind an entitlement its anonymous token \
             does not carry, so the breakdown is referenced by a current departmental document, \
             has not been found, and was never shown to be taken down; see \
             `dew-payment-reports` for the deduct-era half of that gap. And the archived \
             series stops at \
             FY2013 while the consolidated annual report starts at 2024-25, so FY2014 through \
             FY2022 has no participation series for the channel as a whole — LSC's greenbooks \
             quote rounded counts inside it, which bounds that hole rather than filling it. The \
             hole is one year shorter than it was and only for one programme: what precedes \
             2024-25 is a per-programme lineage, and Jon Peterson's FY2023 and FY2024 editions \
             are now held, which gives that programme FY2023 through FY2025 unbroken and no \
             other programme anything. Three earlier JPSN editions — FY2016, FY2017 and FY2020 \
             — and an Ohio ACE report are named on archived captures of the Annual Reports page \
             and not held; the department serves neither, so they are an archive fetch rather \
             than a live one. \
             The designated list is current-edition-only at the department: the EdChoice \
             Resources page carries one edition at a time and deletes the last, so 2023-2024, \
             2024-2025 and 2025-2026 all return this host's genuine 404. Two of those three are \
             held from the Internet Archive; 2023-2024 is linked from captures of the page and \
             was never itself crawled, so a fourth edition is named and not retrievable",
        ),
    },
    note: "The department's own public account of the scholarship channel. It answers how \
           large each programme is, how far it reaches, how many families applied against how \
           many were ever paid, which buildings' students may claim traditional EdChoice, and \
           — for Jon Peterson alone, which had its own report before the consolidated one — \
           three consecutive years of participation and spending; it does not answer which \
           district a scholarship was charged against, and no public source does.",
    sources: &[
        Source {
            key: "scholarship-annual-2025",
            title: Some("2025 Scholarship Annual Report"),
            url: "https://education.ohio.gov/getattachment/About/Annual-Reports/\
                  2025-Scholarship-Annual-Report.pdf.aspx?lang=en-US",
            filename: "scholarship-annual-2025.pdf",
            format: Format::Pdf,
            catalog: Some("dew-scholarship-annual-report"),
            fixtures: &[
                crate::fixtures::SCHOLARSHIP_FIXTURE,
                crate::fixtures::SCHOLARSHIP_JPSN_FIXTURE,
                crate::fixtures::SCHOLARSHIP_JPSN_CATEGORY_FIXTURE,
            ],
            note: "Participation and award totals for all five scholarship programmes, \
                   2024-25. The only committed source here that sizes the channel from the \
                   department rather than from statute.",
        },
        Source {
            key: "jpsn-annual-2024",
            title: Some("FY24 Jon Peterson Special Needs Scholarship Annual Report"),
            url: "https://education.ohio.gov/getattachment/About/Annual-Reports/\
                  FY24_JPSN_Annual_Report.pdf.aspx?lang=en-US",
            filename: "jpsn-annual-2024.pdf",
            format: Format::Pdf,
            catalog: Some("dew-jpsn-annual-report"),
            fixtures: &[
                crate::fixtures::SCHOLARSHIP_JPSN_FIXTURE,
                crate::fixtures::SCHOLARSHIP_JPSN_CATEGORY_FIXTURE,
            ],
            note: "One programme for FY2024, six pages, and the year the consolidated report \
                   does not cover. Its derived expenditure total is the one figure in this \
                   lineage a second publisher checks: LSC's FY2026-27 redbook quotes about \
                   $95.4 million for the same fiscal year, against $95,362,957.53 summed from \
                   this edition's own chart. Its student count is not checked so well — the \
                   same redbook says about 7,800 where this says 8,551.",
        },
        Source {
            key: "jpsn-annual-2023",
            title: Some("FY23 Jon Peterson Special Needs Scholarship Annual Report"),
            url: "https://education.ohio.gov/getattachment/About/Annual-Reports/\
                  FY23_JPSN_Annual_Report.pdf.aspx?lang=en-US",
            filename: "jpsn-annual-2023.pdf",
            format: Format::Pdf,
            catalog: Some("dew-jpsn-annual-report"),
            fixtures: &[
                crate::fixtures::SCHOLARSHIP_JPSN_FIXTURE,
                crate::fixtures::SCHOLARSHIP_JPSN_CATEGORY_FIXTURE,
            ],
            note: "The earliest edition of this report the department still serves, and the \
                   only one in the lineage that states an expenditure total in words — \
                   $81,773,133.70, which equals the sum of its own six category figures to the \
                   cent and so licenses that derivation for every edition. It attributes the \
                   total to the 2021-2022 school year while reporting FY2023 throughout, which \
                   the fixture carries in a column of its own rather than correcting.",
        },
        Source {
            key: "scholarship-historical",
            title: Some("Historical Scholarship Data"),
            url: "https://education.ohio.gov/getattachment/Topics/Other-Resources/Scholarships/\
                  Historical-Information/Historical-Scholarship-Data.xlsx.aspx?lang=en-US",
            filename: "scholarship-historical.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-scholarship-historical-data"),
            fixtures: &[crate::fixtures::SCHOLARSHIP_HISTORY_FIXTURE],
            note: "The deduct era, statewide: Cleveland from FY1997, traditional EdChoice from \
                   FY2007, Autism from FY2004, Jon Peterson from FY2013. Applications and \
                   scholarships actually paid are separate columns, which is the distinction \
                   the annual report's unreconciled averages need and no other held source \
                   carries. Written once in October 2018 out of the system that preceded the \
                   Enterprise Application System, so it is an archival snapshot rather than a \
                   series anyone maintains.",
        },
        Source {
            key: "edchoice-designated-2627",
            title: Some("EdChoice Designated List 2026-2027, With Criteria"),
            url: "https://education.ohio.gov/getattachment/Topics/Other-Resources/Scholarships/\
                  EdChoice-Scholarship/EdChoice-Resources/\
                  Designated-List-2026-2027-With-Criteria.xlsx.aspx?lang=en-US",
            filename: "edchoice-designated-2627.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-edchoice-designated-list"),
            fixtures: &[
                crate::fixtures::EDCHOICE_DESIGNATED_FIXTURE,
                crate::fixtures::PI_RANKING_FIXTURE,
                crate::fixtures::TITLE1_FIXTURE,
            ],
            note: "Which buildings' students may claim traditional EdChoice for 2026-2027, and \
                   under which of the two criteria, per district and per building IRN. The \
                   public substitute for the `nonpublic-data-historical-ed-choice-designated-list` \
                   portal route this project has not reached — eligibility rather than \
                   participation, so it answers a different question and answers it per \
                   district. The current edition, and the only one the department serves: \
                   the EdChoice Resources page lists one list at a time and its predecessors \
                   are gone from the host under their real names, which the two sources below \
                   carry from the archive instead.",
        },
        Source {
            key: "edchoice-designated-2526",
            title: Some("EdChoice Designated List 2025-2026, with Criteria"),
            // The department's own file, from the Internet Archive because the EdChoice
            // Resources page lists one edition at a time and deletes the last. Its live name
            // spells `with` in lower case — the 2026-2027 edition is the one that capitalizes
            // it — and returns the host's genuine 1,245-byte 404 under either spelling; see
            // `decisions/an-archived-source-is-still-a-source`.
            url: "https://web.archive.org/web/20250309082659id_/\
                  https://education.ohio.gov/getattachment/Topics/Other-Resources/Scholarships/\
                  EdChoice-Scholarship/EdChoice-Resources/\
                  Designated-List-2025-2026-with-Criteria.xlsx.aspx?lang=en-US",
            filename: "edchoice-designated-2526.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-edchoice-designated-list"),
            fixtures: &[crate::fixtures::EDCHOICE_DESIGNATED_2526_FIXTURE],
            note: "The edition before the held one, and the first pair from which eligibility \
                   can be read as a *change* rather than a snapshot. Same twenty columns one \
                   year back, over 2,906 buildings and 494 designations. It proves it is the \
                   department's the way the archived calculators do: the designation rebuilds \
                   from its own criteria columns in all 2,906 rows, the unstated open-building \
                   condition included.",
        },
        Source {
            key: "edchoice-designated-2425",
            title: Some("EdChoice Designated List 2024-2025, with Criteria"),
            // Archived a year earlier, and the oldest edition the archive holds as a file. The
            // 2023-2024 edition is linked from captures of the page and was never itself
            // crawled, so a fourth edition is named and not retrievable.
            url: "https://web.archive.org/web/20240928123347id_/\
                  https://education.ohio.gov/getattachment/Topics/Other-Resources/Scholarships/\
                  EdChoice-Scholarship/EdChoice-Resources/\
                  Designated-List-2024-2025-with-Criteria.xlsx.aspx?lang=en-US",
            filename: "edchoice-designated-2425.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-edchoice-designated-list"),
            fixtures: &[crate::fixtures::EDCHOICE_DESIGNATED_2425_FIXTURE],
            note: "Nineteen columns, not twenty, and the difference is the finding: this \
                   edition ranks **two** Performance Index years and its own heading says so. \
                   R.C. 3310.03(A)(1)(a) asks for the bottom twenty per cent in two of the \
                   three most recent rankings, and in 2024-2025 the department had two \
                   rankings to apply it to. 2,937 buildings and 460 designations, rebuilding \
                   from their own criteria in every row.",
        },
    ],
};

pub(super) const CHILD_NUTRITION: Connector = Connector {
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
        // --- The workbook era. One file an October again, with the stream on every row. ---
        Source {
            key: "mr81-2015",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Student-Supports/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  Data-for-Free-and-Reduced-Price-Meal-Eligibility/\
                  October_2015_-Data_for_Free-and-Reduced-Price-Meal.xlsx.aspx?lang=en-US",
            filename: "mr81-2015.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2015, and the October the report came back together. One workbook \
                   per year with an `NSLP Provision` column, so the three streams are rows \
                   rather than three files again. Its Notes sheet states the CEP rule this \
                   repository had measured off the FY2014 file: the printed percentage is the \
                   directly-certified count times 1.6 over enrolment.",
        },
        Source {
            key: "mr81-2016",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Student-Supports/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  Data-for-Free-and-Reduced-Price-Meal-Eligibility/\
                  October_2016_-Data_for_Free-and-Reduced-Price-Meal.xlsx.aspx?lang=en-US",
            filename: "mr81-2016.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2016.",
        },
        Source {
            key: "mr81-2017",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Student-Supports/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  Data-for-Free-and-Reduced-Price-Meal-Eligibility/\
                  October_2017_-Data_for_Free-and-Reduced-Price-Meal.xlsx.aspx?lang=en-US",
            filename: "mr81-2017.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2017.",
        },
        Source {
            key: "mr81-2018",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Other-Resources/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  MR81-Data-for-Free-and-Reduced-Price-Meal-Eligibil/\
                  October_2018_Data-for-Free-and-Reduced-Price-Meal.xlsx.aspx?lang=en-US",
            filename: "mr81-2018.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2018. Filed under `Topics/Other-Resources` rather than \
                   `Topics/Student-Supports`, alone among the eleven — the department moved \
                   the page and left this one attachment behind.",
        },
        Source {
            key: "mr81-2019",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Student-Supports/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  MR81-Data-for-Free-and-Reduced-Price-Meal-Eligibil/\
                  October-2019-Data-for-Free-and-Reduced-Price-Meal-Eligibility.xlsx.aspx?lang=en-US",
            filename: "mr81-2019.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2019.",
        },
        Source {
            key: "mr81-2020",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Student-Supports/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  MR81-Data-for-Free-and-Reduced-Price-Meal-Eligibil/\
                  October-2020-Data-for-Free-and-Reduced-Price-Meal-Eligibility.xlsx.aspx?lang=en-US",
            filename: "mr81-2020.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2020, the first pandemic October. USDA's nationwide free-meal \
                   waivers ran through this school year, so the applications a Traditional \
                   sponsor collected are not comparable with the years around it.",
        },
        Source {
            key: "mr81-2021",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Student-Supports/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  Data-for-Free-and-Reduced-Price-Meal-Eligibility/\
                  October-2021-Data-for-Free-and-Reduced-Price-Meal-Eligibility-2.xlsx.aspx?lang=en-US",
            filename: "mr81-2021.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2021, the second waiver year. The published filename carries a \
                   `-2` suffix, which is a revision marker rather than a second file.",
        },
        Source {
            key: "mr81-2022",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Student-Supports/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  Data-for-Free-and-Reduced-Price-Meal-Eligibility/\
                  October-2022-Data-for-Free-and-Reduced-Price-Meal-Eligibilty.xlsx.aspx?lang=en-US",
            filename: "mr81-2022.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2022.",
        },
        Source {
            key: "mr81-2023",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Student-Supports/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  Data-for-Free-and-Reduced-Price-Meal-Eligibility/\
                  October-2023-Data-for-Free-and-Reduced-Price-Meal-Eligibilty.xlsx.aspx?lang=en-US",
            filename: "mr81-2023.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2023.",
        },
        Source {
            key: "mr81-2024",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Student-Supports/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  Data-for-Free-and-Reduced-Price-Meal-Eligibility/\
                  October-2024-FY2025-Data-for-Free-and-Reduced-Price-Meal-Eligibilty.xlsx.aspx?lang=en-US",
            filename: "mr81-2024.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2024.",
        },
        Source {
            key: "mr81-2025",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Student-Supports/Food-and-Nutrition/\
                  Resources-and-Tools-for-Food-and-Nutrition/\
                  Data-for-Free-and-Reduced-Price-Meal-Eligibility/\
                  October-2025-FY2026-Data-for-Free-and-Reduced-Price-Meal-Eligibilty.xlsx.aspx?lang=en-US",
            filename: "mr81-2025.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-mr81-enrollment-archive"),
            fixtures: &[crate::fixtures::MR81_FIXTURE],
            note: "October 2025, posted 13 February 2026 and the most recent published. The \
                   department is `Education & Workforce` and the office is `Office of \
                   Nutrition` on this one; the series is otherwise unchanged.",
        },
    ],
};

pub(super) const SCHOOL_IMPROVEMENT: Connector = Connector {
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
};

pub(super) const NONPUBLIC_ENROLLMENT: Connector = Connector {
    key: "dew-nonpublic-enrollment",
    publisher: "Ohio Department of Education and Workforce, Office of Data Quality",
    feeds: &["school", "metric", "program"],
    status: Status::Wired {
        still_blocked: Some(
            "wired for every October the department publishes, 1977 through 2025, at the \
             building and at the sector. What it cannot reach is the district each building \
             sits in. Auxiliary services under R.C. 3317.06 flows to a nonpublic school \
             through the district its building is located in, so a per-district question \
             about the nonpublic sector needs that column; no file in any of the three eras \
             carries it, the Chartered Nonpublic School Information page publishes no \
             directory, and `edchoice-designated` is public buildings only. County and \
             school type are published for 1977-78 through 2006-07 and then stop, so even \
             the coarser geography is gone for the modern years",
        ),
    },
    note: "Forty-nine Octobers of chartered nonpublic enrolment by building — the denominator \
           of R.C. 3317.024(E)(2)(d) and the only long series of the sector Ohio has. The \
           repository had held one year of it, from a fact sheet. Every count below ten is \
           masked, so every quantity built from these files is a floor, a ceiling and a \
           censored-cell count rather than a point.",
    sources: &[
        Source {
            key: "nonpublic-1977-1978",
            title: Some("Nonpublic enrollment, 1977-78 and 1978-79"),
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/1978-1979-Nonpublic.xls.aspx",
            filename: "1978-1979-Nonpublic.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "The start of the series, and the two years where the grade block and the \
                   race block disagree most. Each compilation is named for the school years \
                   its sheets *end* in, so this one begins at 1977-78. Both sheets carry a \
                   `State Total` row that is an Excel sum over cells the masking had already \
                   overwritten: it reads 243,489 for a sector of at least 260,867.",
        },
        Source {
            key: "nonpublic-1979-1988",
            title: Some("Nonpublic enrollment, 1979-80 through 1988-89"),
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/1980-1989-Nonpublic.xls.aspx",
            filename: "1980-1989-Nonpublic.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "Ten sheets, one a school year, and the decade the two blocks converge \
                   across: schools reporting race and leaving every grade cell zero stop \
                   after 1982-83, and from 1983-84 the blocks agree on every building with \
                   no masked cell in either.",
        },
        Source {
            key: "nonpublic-1989-1998",
            title: Some("Nonpublic enrollment, 1989-90 through 1998-99"),
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/1990-1999-Nonpublic.xls.aspx",
            filename: "1990-1999-Nonpublic.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "The sector's modern peak sits in here: 1996-97 floors at 218,542 and the \
                   building count rises every year of the decade. These are the years the \
                   scholarship participation series (FY1997 onward) can finally be read \
                   against a denominator rather than against itself.",
        },
        Source {
            key: "nonpublic-1999-2007",
            title: Some("Nonpublic enrollment, 1999-00 through 2007-08"),
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/2000-2008-NonPublic.xls.aspx",
            filename: "2000-2008-NonPublic.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "Both layout breaks are inside this one file. 2001-02 inserts a `Total` \
                   column ahead of the race block and shifts it one to the right; the five \
                   sheets from 2002-03 are named `… ethnic data unavailable` and carry no \
                   race block at all, after which the grade block is the only measure there \
                   is. The last sheet, 2007-08, is already the modern one-row-per-grade \
                   layout — the break happens mid-compilation.",
        },
        Source {
            key: "nonpublic-2008",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/\
                  oct_08_fy09_adm_web_nonpub.xls.aspx",
            filename: "oct_08_fy09_adm_web_nonpub.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2008, the first annual file, and 798 buildings — the largest the \
                   sector is in this series. Its building sheet is named `Oct09_adm_nonpub` \
                   for an October the file does not hold, which is why the year is declared \
                   in the file table and never read off a sheet name.",
        },
        Source {
            key: "nonpublic-2009",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_FY2010_totals.xls.aspx",
            filename: "nonpub_FY2010_totals.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "The one October published as building totals and nothing else: IRN, name, \
                   one number, 787 schools. No grade detail and no sex split, so this year \
                   contributes a bound per building and no grade-level anything.",
        },
        Source {
            key: "nonpublic-2010",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/web_nonpub_2011.xls.aspx",
            filename: "web_nonpub_2011.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2010. One sheet, named `2010_11` — the only two years whose \
                   sheet name states the school year the way the compilations do.",
        },
        Source {
            key: "nonpublic-2011",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/web_nonpub_2012.xls.aspx",
            filename: "web_nonpub_2012.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2011, on the sheet `2011_12`. The last file with no notes sheet \
                   and no state total of any kind.",
        },
        Source {
            key: "nonpublic-2012",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/\
                  Nonpub_FY2013-Data.xlsx.aspx?lang=en-US",
            filename: "Nonpub_FY2013-Data.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2012, and the one file in the series published in the post-2007 \
                   format. Its data is on `Sheet1` beside two empty sheets the publisher \
                   never deleted.",
        },
        Source {
            key: "nonpublic-2013",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub.xls.aspx",
            filename: "nonpub.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2013, published under a filename with no year in it at all. The \
                   2014-2019 compilation states 174,108 in-state for this October against a \
                   building floor well below it, and the two are reconciled nowhere.",
        },
        Source {
            key: "nonpublic-2014",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub2015.xls.aspx",
            filename: "nonpub2015.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2014, and the last annual file with no state-totals sheet. Grade \
                   codes change inside this year: `Kindergarten` and `1st Grade` through \
                   here, `KG` and `01` from the next file on.",
        },
        Source {
            key: "nonpublic-2015",
            title: Some("Revised FY2016 nonpublic enrollment"),
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/Revised-FY16-nonpub.xls.aspx",
            filename: "Revised-FY16-nonpub.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2015, the first file to carry a state-totals sheet — and the one \
                   whose state-totals sheet is wrong. It reads 144,512, below its own \
                   building rows' floor of 147,309, so it cannot be a total of them; the \
                   2014-2019 compilation's 171,395 is the figure for this October. The \
                   building sheet itself is sound, and is the sheet the next year's file \
                   reposts.",
        },
        Source {
            key: "nonpublic-2016",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_fy17.xls.aspx",
            filename: "nonpub_fy17.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2016's state totals over October 2015's buildings. The building \
                   sheet is `Revised-FY16-nonpub.xls`'s: 5,883 keyed cells are in both and \
                   not one of them differs, and against the compilation's per-school totals \
                   it agrees exactly with FY2016 on 86 unmasked schools and with no other \
                   year on more than six. Only the state-totals sheet is read from this \
                   file; the building rows are `RepostedFrom` and dropped.",
        },
        Source {
            key: "nonpublic-2017",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_fy18.xls.aspx?lang=en-US",
            filename: "nonpub_fy18.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2017, and the one year where the annual file and the 2014-2019 \
                   compilation reconcile exactly: 167,558 in-state plus 1,533 out-of-state \
                   is the compilation's 169,091 total.",
        },
        Source {
            key: "nonpublic-2018",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_fy19.xls.aspx?lang=en-US",
            filename: "nonpub_fy19.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2018, and the last October two publications describe. They \
                   disagree: 165,346 on this file's own totals sheet against 165,511 in the \
                   compilation, which is neither that figure nor the compilation's total. \
                   Both rows are kept, distinguished by basis.",
        },
        Source {
            key: "nonpublic-2019",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_fy20.xls.aspx?lang=en-US",
            filename: "nonpub_fy20.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2019, the last October before the pandemic and the last with a \
                   compilation to check it against. The totals sheet is named \
                   `FY20 Nonpub Counts`; from the next file the name becomes \
                   `fyNN_nonpub_state`.",
        },
        Source {
            key: "nonpublic-2020",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_fy21.xls.aspx?lang=en-US",
            filename: "nonpub_fy21.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2020 — the pandemic October, and the one whose reading needs most \
                   care, since the sector's count moved for reasons the file does not record.",
        },
        Source {
            key: "nonpublic-2021",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_fy22.xls.aspx?lang=en-US",
            filename: "nonpub_fy22.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2021, the October before universal EdChoice eligibility was \
                   enacted and so the sector's last pre-expansion count.",
        },
        Source {
            key: "nonpublic-2022",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_fy23.xls.aspx?lang=en-US",
            filename: "nonpub_fy23.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2022.",
        },
        Source {
            key: "nonpublic-2023",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_fy24.xls.aspx?lang=en-US",
            filename: "nonpub_fy24.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2023, and the cross-publisher pin. Its state-totals sheet gives \
                   86,232 male and 85,598 female in state and 742 and 584 out of it — \
                   173,156 in all — and its building sheet carries 711 distinct IRNs. Both \
                   are exactly what the 2023-24 Education Landscape fact sheet states, from \
                   a different office and a different publication. The totals sheet is named \
                   `fy23_nonpub_state` inside a file named for FY2024.",
        },
        Source {
            key: "nonpublic-2024",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_fy25.xls.aspx?lang=en-US",
            filename: "nonpub_fy25.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2024, which is the October R.C. 3317.024(E)(2)(d) divides the \
                   FY2025 auxiliary services appropriation by: 179,693 in state across 722 \
                   buildings. $164,077,754 over that is $913.10, and LSC publishes $913. \
                   Out-of-state pupils included it would be $906, so the statutory \
                   denominator is the in-state count.",
        },
        Source {
            key: "nonpublic-2025",
            title: None,
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/nonpub_fy26.xls.aspx?lang=en-US",
            filename: "nonpub_fy26.xls",
            format: Format::LegacyXls,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "October 2025, the current file: 184,069 in state across 749 buildings. \
                   The sector has grown in each of the three Octobers since universal \
                   eligibility, having fallen in almost every October before them.",
        },
        Source {
            key: "nonpublic-2013-2018",
            title: Some("Nonpublic enrollment, FY2014 through FY2019"),
            url: "https://education.ohio.gov/getattachment/Topics/Data/\
                  Frequently-Requested-Data/Enrollment-Data/\
                  2014-2019-NonPublic.xlsx.aspx?lang=en-US",
            filename: "2014-2019-NonPublic.xlsx",
            format: Format::Xlsx,
            catalog: Some("dew-nonpublic-enrollment"),
            fixtures: &[
                crate::fixtures::NONPUBLIC_BUILDING_FIXTURE,
                crate::fixtures::NONPUBLIC_SECTOR_FIXTURE,
            ],
            note: "A second publication of six Octobers the annual files already cover, and \
                   the only reason the disagreement between them is visible at all. Its \
                   `By School` sheet masks 30 rows of 4,286 against thousands in the annual \
                   files, because it states one number a school rather than fifty-two; its \
                   `State Totals` sheet gives both a PK-12 and a K-12 row a year, which is \
                   the span the annual files never state. It is also the only file that \
                   settles what `nonpub_fy17.xls` holds.",
        },
    ],
};
