//! Auditor of State of Ohio.
//!
//! Audit reports, which are where the R.C. 3311 territory-transfer orders are recited. They are
//! not published as data anywhere, and this is the only route to them the corpus found.

use super::{Connector, Format, Source, Status};

pub(super) const REPORTS: Connector = Connector {
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
};
