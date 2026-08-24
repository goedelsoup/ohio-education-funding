//! Ohio's statewide operating expenditure per pupil, nominal and real.
//!
//! # Why this crate owns the series and not `dispersion`
//!
//! Every other per-pupil series in this workspace is a *distribution* over districts.  This one
//! is a single statewide number per year, and the only thing that makes it interesting is what
//! happens when it is deflated: nominal it rises monotonically from FY2000 to FY2022, and in
//! constant dollars it does not. It exists to be converted, so it lives beside the converter.
//!
//! # The rows are not all the same kind of claim
//!
//! Two of them — FY2000 and FY2022 — are figures the Ohio Auditor of State publishes as
//! numbers. The rest are read off a chart to about $100, which is why every row carries
//! [`NominalPoint::exact`] and why [`real_series`] returns the [`Confidence`] the conversion
//! ends up with rather than assuming one. A caller that quotes an interior row without saying
//! it is an inference is misreporting the source, and the corpus tags them accordingly.
//!
//! The uncertainty is small against what the series shows. `tests/ohio_epp_real_series.rs`
//! perturbs both ends of the FY2010-FY2014 trough by the full chart-label error in the
//! direction that would erase it, and the trough survives with room to spare.

use edfund_core::FiscalYear;

use crate::{Confidence, CpiSeries, Deflated};

/// The committed series.
const FIXTURE: &str = include_str!("../fixtures/ohio-epp-nominal.csv");

/// The header this reader was written against.
const EXPECTED_HEADER: &str = "fiscal_year,operating_expenditure_per_pupil,exact";

/// The base year every real figure here is stated in, unless a caller names another.
///
/// FY2022 because it is the series' last year and one of the two exact ones, so the endpoint
/// that anchors the restatement carries no chart-reading error of its own.
pub const BASE_YEAR: FiscalYear = FiscalYear(2022);

/// One year of the statewide series, as published.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NominalPoint {
    /// The fiscal year.
    pub fiscal_year: FiscalYear,
    /// Operating expenditure per pupil, in that year's dollars.
    pub dollars: f64,
    /// Whether the publisher states this figure as a number rather than plotting it.
    ///
    /// False means it was read off a chart, to roughly $100. It is carried per row because the
    /// series mixes both, and a reader that loses the distinction turns an inference into a
    /// verified figure at no cost to itself.
    pub exact: bool,
}

/// The statewide series as published, in fiscal-year order.
///
/// # Panics
///
/// If the fixture's header is not the one this reader was written against, or a row's width
/// differs from the header's — both by way of [`edfund_core::csv::rows`].
#[must_use]
pub fn nominal_series() -> Vec<NominalPoint> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            Some(NominalPoint {
                fiscal_year: FiscalYear(row.str(0).parse().ok()?),
                dollars: row.num(1)?,
                exact: row.str(2) == "true",
            })
        })
        .collect()
}

/// The series restated in `base` dollars on CPI-U June, with the confidence each row earns.
///
/// The confidence is the weaker of the two index points involved, and a row whose nominal
/// figure was read off a chart is never quotable however good the index is —
/// [`NominalPoint::exact`] is the other half of that judgement and is returned alongside.
///
/// # Panics
///
/// If a year in the series is absent from [`CpiSeries::cpi_u_june`]. The two are committed
/// together and a gap means one of them was refreshed without the other.
#[must_use]
pub fn real_series(base: FiscalYear) -> Vec<(NominalPoint, Deflated)> {
    let cpi = CpiSeries::cpi_u_june();
    nominal_series()
        .into_iter()
        .map(|point| {
            let deflated = cpi
                .convert(point.dollars, point.fiscal_year, base)
                .unwrap_or_else(|e| {
                    panic!("FY{} is not in the CPI series: {e}", point.fiscal_year.0)
                });
            (point, deflated)
        })
        .collect()
}

/// The real figure for one year, in [`BASE_YEAR`] dollars.
///
/// # Panics
///
/// If the year is not in the series, or is absent from the index.
#[must_use]
pub fn real_at(year: FiscalYear) -> f64 {
    real_series(BASE_YEAR)
        .into_iter()
        .find(|(point, _)| point.fiscal_year == year)
        .unwrap_or_else(|| panic!("FY{} is not in the committed series", year.0))
        .1
        .value
}

/// Whether a year's real figure can be quoted as verified.
///
/// True only where the nominal figure is stated rather than plotted *and* both index points are
/// [`Confidence::Verified`] — which is FY2000 and FY2022 and nothing else.
#[must_use]
pub fn quotable(point: &NominalPoint, deflated: &Deflated) -> bool {
    point.exact && deflated.confidence == Confidence::Verified
}
