//! U.S. Census Bureau.
//!
//! Two unrelated products: the F-33 school system finance survey, which is how Ohio is compared
//! against other states on the same definitions, and the PL 94-171 redistricting file, which is
//! the only route to a school-district-to-legislative-district crosswalk.

use super::{Connector, Format, Source, Status};

pub(super) const F33: Connector = Connector {
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
    sources: &[
        Source {
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
        },
    ],
};

pub(super) const GEOGRAPHY: Connector = Connector {
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
};
