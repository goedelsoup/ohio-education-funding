//! U.S. Energy Information Administration.
//!
//! One price series, for one reason: R.C. 3317.0212(C) and (D) set the state's two
//! transportation rates as trimmed means of what districts reported spending, so the formula's
//! transportation input is a cost the formula does not control, and diesel is the part of that
//! cost that moves fastest. Kept apart from [`super::bls`] because a fuel price is not a price
//! index — the deflator restates dollars, this explains them.

use super::{Connector, Format, Source, Status};

pub(super) const DIESEL: Connector = Connector {
    key: "eia-diesel",
    publisher: "U.S. Energy Information Administration",
    feeds: &["metric", "fiscal-period"],
    status: Status::Parsed,
    note: "The cost side of the transportation rates. Ohio's school buses run on diesel and \
           the rate that reimburses them is a trimmed mean of last year's spending, so the \
           price and the payment are a year apart by construction.",
    sources: &[Source {
        key: "midwest-diesel-monthly",
        title: None,
        url: "https://www.eia.gov/dnav/pet/hist_xls/EMD_EPD2D_PTE_R20_DPGm.xls",
        filename: "midwest-diesel-monthly.xls",
        format: Format::LegacyXls,
        catalog: Some("eia-diesel-prices"),
        fixtures: &[crate::fixtures::DIESEL_FIXTURE],
        note: "No. 2 diesel retail prices, Midwest (PADD 2), monthly, dollars per gallon. \
               Ohio has no state series; PADD 2 is the smallest region that contains it.",
    }],
};
