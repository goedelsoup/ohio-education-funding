//! Ohio Department of Taxation.
//!
//! The local side of school funding. The abstract carries assessed valuation and rates by
//! district; the casino distribution is the one revenue stream paid to districts directly rather
//! than through the formula.

use super::{Connector, Format, Source, Status};

pub(super) const ABSTRACT: Connector = Connector {
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
};

pub(super) const CASINO: Connector = Connector {
    key: "tax-casino",
    publisher: "Ohio Department of Taxation",
    feeds: &["revenue-stream", "education-agency", "metric"],
    status: Status::Wired {
        still_blocked: Some(
            "wired for every per-district distribution the department publishes as a \
             workbook — eighteen of them, the August 2015 distribution through the January \
             2024 one. It stops there because the department's own casino page stops there, \
             and the distributions before it have no machine-readable twin: January 2015 and \
             earlier are `Final SD Distribution` PDFs",
        ),
    },
    note: "The size of the one education channel that reaches every district and enters no \
           appropriation table — which is what makes `casino-tax-distribution`'s null result \
           a measurement rather than an absence of evidence.",
    sources: &[
        Source {
            key: "casino-fy2016-fy2017",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/government/newdocs/\
                  Casino%20Docs/FY%202016-FY2017%20SD%20Distributions.xlsx",
            filename: "casino-fy2016-fy2017.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
            note: "Four distributions in one workbook, a sheet each: August 2015, January \
                   2016, August 2016 and January 2017. Statewide by district, with no county \
                   breakdown — the only layout published for the three middle ones, which is \
                   why this file is here rather than three that do not exist. Its August 2015 \
                   sheet overlaps `casino-2015-08` and is the cross-check in \
                   `build_casino_extract`: two layouts, two files, 1,044 districts and one \
                   total, agreeing to the cent.",
        },
        Source {
            key: "casino-2015-08",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/government/newdocs/\
                  Casino%20Docs/2015%2008%20Casino%20Student%20Dist.%20by%20County%20by%20\
                  SD%20Web.xlsx",
            filename: "casino-2015-08.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "January-June 2015, paid August 2015. The oldest distribution published as \
                   a workbook and the only one published in both layouts.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2017-08",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/government/newdocs/\
                  Casino%20Docs/2017%2008%20County%20Student%20Distribution.xlsx",
            filename: "casino-2017-08.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "January-June 2017. The county-by-district layout resumes here and runs to \
                   the end of the series.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2018-01",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/government/newdocs/\
                  Casino%20Docs/2018%2001%20County%20Student%20Distribution_2.xlsx",
            filename: "casino-2018-01.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "July-December 2017. One of three sources spelling the amount column \
                   `Distribution Amount`; twelve of the sixteen spell it `Distrubution \
                   Amount`, and the true-up year spells it `Total Distribution`.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2018-08",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/government/newdocs/\
                  Casino%20Docs/2018%2008%20County%20Student%20Distribution.xlsx",
            filename: "casino-2018-08.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "January-June 2018, and the one true-up in the series: three amount \
                   columns, a January 2018 recalculation of $7,475.78 beside the August 2018 \
                   calculation of $48,045,656.79, summing to `Total Distribution`. Reading \
                   the first column here — the position every other year's amount sits in — \
                   reports the half-year at seven thousand dollars.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2019-01",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/government/newdocs/\
                  Casino%20Docs/2019%2001%20County%20Student%20Distribution%20Report.xlsx",
            filename: "casino-2019-01.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "July-December 2018. The second of the three `Distribution Amount` \
                   spellings.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2019-08",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/government/newdocs/\
                  Casino%20Docs/2019%2008%20County%20Student%20Distribution%20Report.xlsx",
            filename: "casino-2019-08.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "January-June 2019. The last half-year before the casinos closed.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2020-01",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/government/newdocs/\
                  Casino%20Docs/2020%2001%20County%20Student%20Distribution%20Report.xlsx",
            filename: "casino-2020-01.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "July-December 2019, the last full normal half-year in the series.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2020-08",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/revenue_accounting/\
                  casino/2020%2008%20County%20Student%20Distribution%20Report.xlsx",
            filename: "casino-2020-08.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "January-June 2020: $24.59 million against a floor of $42.86 million \
                   across the other seventeen half-years, the casinos having been closed by \
                   order from mid-March. The publisher's directory moves here at this file, \
                   from `government/newdocs/Casino Docs/` to `revenue_accounting/casino/`.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2021-01",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/revenue_accounting/\
                  casino/2021%2001%20County%20Student%20Distribution%20Report.xlsx",
            filename: "casino-2021-01.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "July-December 2020. Back to $49.28 million within one half-year of the \
                   trough, which is what makes the trough a closure rather than a decline.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2021-08",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/revenue_accounting/\
                  casino/2021%2008%20County%20Student%20Distribution%20Report.xlsx",
            filename: "casino-2021-08.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "January-June 2021, and the only file carrying the department's own \
                   reconciliation: a `Sheet1` comparing each county's district sum against \
                   its allocated total. Seventy-five of the 88 balance and thirteen are out \
                   by exactly a cent, which is what apportioning one pot across a county's \
                   districts to the penny costs.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2022-01",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/revenue_accounting/\
                  casino/2022%2001%20County%20Student%20Distribution%20Report.xlsx",
            filename: "casino-2022-01.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "July-December 2021. The first half-year above $55 million.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2022-08",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/revenue_accounting/\
                  casino/2022%2008%20County%20Student%20Distribution%20Report.xlsx",
            filename: "casino-2022-08.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "January-June 2022.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2023-01",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/revenue_accounting/\
                  casino/2023%2001%20County%20Student%20Distribution%20Report%20web.xlsx",
            filename: "casino-2023-01.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "July-December 2022.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2023-08",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/revenue_accounting/\
                  casino/2023%2008%20County%20Student%20Distribution%20Report%20web.xlsx",
            filename: "casino-2023-08.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "January-June 2023, the largest half-year in the series at $58.67 million.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
        Source {
            key: "casino-2024-01",
            title: None,
            url: "https://dam.assets.ohio.gov/raw/upload/tax.ohio.gov/revenue_accounting/\
                  casino/2024%2001%20County%20Student%20Distribution%20Report%20Web.xlsx",
            filename: "casino-2024-01.xlsx",
            format: Format::Xlsx,
            catalog: Some("dot-casino-student-distribution"),
            note: "July-December 2023, the last distribution published. The `Web` in the path \
                   is load-bearing: the same directory serves a `Report.xlsx` without it, \
                   which is the analyst's copy and carries an extra `RP_MAIN_PG1 (2)` sheet \
                   reconciling the districts against the county allocations — where they \
                   differ by four cents.",
            fixtures: &[crate::fixtures::CASINO_FIXTURE],
        },
    ],
};
