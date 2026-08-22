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

pub(super) const PAYMENT_REPORTS: Connector = Connector {
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
};

pub(super) const SCHOLARSHIP_REPORTS: Connector = Connector {
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
