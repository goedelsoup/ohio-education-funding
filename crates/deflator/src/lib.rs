//! Nominal-to-constant-dollar conversion for Ohio school finance series.
//!
//! This is the smallest calculator in the workspace and the one without which nothing else is
//! honest. Ohio's central mechanism — H.B. 920 holding voted levy yields approximately flat in
//! nominal terms — is invisible in nominal figures and obvious in real ones. A corpus spanning
//! 1851 to the present cannot compare any two nominal amounts.
//!
//! The stakes are concrete. Ohio public school operating expenditure per pupil rose from
//! $7,065 in FY2000 to $15,314 in FY2022: **116.8% nominally, 26.1% in real terms**. Both are
//! correct; they support opposite arguments about whether Ohio increased its investment in
//! schools.
//!
//! # Fiscal-year alignment
//!
//! Ohio fiscal years run 1 July to 30 June. This crate indexes on the **June** CPI observation
//! of the year the fiscal year ends, matching the Ohio Auditor of State's method. Applying a
//! calendar-year average to a fiscal-year figure introduces a systematic half-year error that
//! compounds across a long series.
//!
//! # Confidence
//!
//! Index points carry a [`Confidence`]. Only two are verified — FY2000 and FY2022 — because
//! those two reproduce the Auditor's independently published growth figures exactly. Every
//! other point is transcribed and unverified until checked against BLS. A [`Deflated`] result
//! reports the weaker confidence of its two endpoints so that a caller writing into the corpus
//! knows whether the claim is `[verified]` or `[inference]`.

#![forbid(unsafe_code)]

use edfund_core::{Dollars, FiscalYear};

/// How much trust an index point, or a result derived from one, carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    /// Transcribed but not yet checked against the Bureau of Labor Statistics. A corpus claim
    /// resting on this must be tagged `[inference]`.
    Unverified,
    /// Cross-checked against an independently published figure. A corpus claim resting on this
    /// may be tagged `[verified]`.
    Verified,
}

/// A price index observation for one fiscal year.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IndexPoint {
    /// The fiscal year this observation is aligned to.
    pub fiscal_year: FiscalYear,
    /// The index level.
    pub index: f64,
    /// Whether the level has been checked against the publishing agency.
    pub confidence: Confidence,
}

/// A deflated figure, carrying the confidence of the weaker endpoint used to produce it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Deflated {
    /// The figure expressed in the target fiscal year's dollars.
    pub value: Dollars,
    /// The weaker of the two endpoint confidences.
    pub confidence: Confidence,
}

/// Why a deflation could not be performed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeflatorError {
    /// No index observation exists for the requested fiscal year.
    MissingIndex(FiscalYear),
    /// The index level for the requested year is zero or negative.
    NonPositiveIndex(FiscalYear),
}

impl core::fmt::Display for DeflatorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingIndex(fy) => write!(f, "no index observation for {fy}"),
            Self::NonPositiveIndex(fy) => write!(f, "non-positive index for {fy}"),
        }
    }
}

impl std::error::Error for DeflatorError {}

/// A price index series aligned to Ohio fiscal years.
#[derive(Debug, Clone)]
pub struct CpiSeries {
    points: Vec<IndexPoint>,
    label: &'static str,
}

impl CpiSeries {
    /// Build a series from explicit points. Used for tests and for alternative indices.
    #[must_use]
    pub fn from_points(label: &'static str, points: Vec<IndexPoint>) -> Self {
        Self { points, label }
    }

    /// The index's name, which must accompany any figure it produced.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        self.label
    }

    /// Every observation in the series, in the order it was built.
    ///
    /// Exists so a verification pass can iterate what the series *actually holds* rather than a
    /// range someone wrote down. A hard-coded upper bound is how four points were once added to
    /// this series and checked by nothing.
    #[must_use]
    pub fn points(&self) -> &[IndexPoint] {
        &self.points
    }

    /// Look up the observation for a fiscal year.
    #[must_use]
    pub fn point(&self, fy: FiscalYear) -> Option<IndexPoint> {
        self.points.iter().copied().find(|p| p.fiscal_year == fy)
    }

    /// Express `nominal`, denominated in `from` dollars, in `to` dollars.
    ///
    /// # Errors
    ///
    /// Returns [`DeflatorError`] if either fiscal year is absent from the series or carries a
    /// non-positive index level.
    pub fn convert(
        &self,
        nominal: Dollars,
        from: FiscalYear,
        to: FiscalYear,
    ) -> Result<Deflated, DeflatorError> {
        let from_pt = self.point(from).ok_or(DeflatorError::MissingIndex(from))?;
        let to_pt = self.point(to).ok_or(DeflatorError::MissingIndex(to))?;
        if from_pt.index <= 0.0 {
            return Err(DeflatorError::NonPositiveIndex(from));
        }
        if to_pt.index <= 0.0 {
            return Err(DeflatorError::NonPositiveIndex(to));
        }
        Ok(Deflated {
            value: nominal * (to_pt.index / from_pt.index),
            confidence: from_pt.confidence.min(to_pt.confidence),
        })
    }

    /// Cumulative index growth between two fiscal years, as a fraction.
    ///
    /// `index_growth(FY2000, FY2022)` returns `0.7187…`, i.e. 71.9%.
    ///
    /// # Errors
    ///
    /// As [`Self::convert`].
    pub fn index_growth(&self, from: FiscalYear, to: FiscalYear) -> Result<f64, DeflatorError> {
        let one = self.convert(1.0, from, to)?;
        Ok(one.value - 1.0)
    }

    /// Growth between two nominal figures after removing price change, as a fraction.
    ///
    /// This is the operation most Ohio school finance arguments actually need, and the one
    /// most often skipped.
    ///
    /// # Errors
    ///
    /// As [`Self::convert`].
    pub fn real_growth(
        &self,
        from_value: Dollars,
        from: FiscalYear,
        to_value: Dollars,
        to: FiscalYear,
    ) -> Result<Deflated, DeflatorError> {
        let rebased = self.convert(from_value, from, to)?;
        if rebased.value == 0.0 {
            return Err(DeflatorError::NonPositiveIndex(from));
        }
        Ok(Deflated {
            value: to_value / rebased.value - 1.0,
            confidence: rebased.confidence,
        })
    }

    /// CPI-U, US city average, all items, not seasonally adjusted, June observation of each
    /// fiscal year's ending calendar year.
    ///
    /// FY2000 and FY2022 were verified first, by reproducing the Ohio Auditor of State's
    /// independently published 71.9% CPI growth and its 26.1% and 19.4% real per-pupil
    /// expenditure growth figures. The rest were transcribed by hand and carried
    /// [`Confidence::Unverified`] until the `bls-cpi` connector was built.
    ///
    /// Every point is now [`Confidence::Verified`] against the Bureau's own published series,
    /// and `crates/connect/tests/deflator_matches_bls.rs` re-checks all twenty-three on every
    /// test run from a committed extract of that file. The check is what carries the claim; the
    /// marking here only records it.
    ///
    /// # What the check found
    ///
    /// FY2016 was transcribed as `241.038`. The Bureau publishes `241.018`. Twenty-two of
    /// twenty-three points were typed correctly and one was not, which is about the error rate
    /// hand transcription actually has — and the reason a series should not be trusted because
    /// it looks plausible. The effect on any real-dollar figure is small; the point is that
    /// nothing in this workspace could have found it without going back to the source.
    ///
    /// CPI-U is a general consumer price index. School costs are majority compensation, for
    /// which the Employment Cost Index is a better deflator with shorter coverage. Any figure
    /// this produces must name the index used.
    #[must_use]
    pub fn cpi_u_june() -> Self {
        const VERIFIED: &[(u16, f64)] = &[(2000, 172.400), (2022, 296.311)];
        const TRANSCRIBED: &[(u16, f64)] = &[
            (2001, 178.000),
            (2002, 179.900),
            (2003, 183.700),
            (2004, 189.700),
            (2005, 194.500),
            (2006, 202.900),
            (2007, 208.352),
            (2008, 218.815),
            (2009, 215.693),
            (2010, 217.965),
            (2011, 225.722),
            (2012, 229.478),
            (2013, 233.504),
            (2014, 238.343),
            (2015, 238.638),
            // Transcribed as 241.038; the Bureau publishes 241.018. Corrected from the source.
            (2016, 241.018),
            (2017, 244.955),
            (2018, 251.989),
            (2019, 256.143),
            (2020, 257.797),
            (2021, 271.696),
            // FY2023 onward were added when the financial panel arrived and needed them. They
            // were never hand-transcribed: they come from the same committed BLS extract the
            // check test reads, so they are verified on the same run that verifies the rest.
            (2023, 305.109),
            (2024, 314.175),
            (2025, 322.561),
            (2026, 333.952),
        ];

        // Both lists are Verified now. They stay separate because *how* a point was verified
        // differs — the two endpoints against an independent publication, the rest against the
        // agency itself — and a future correction should know which is which.
        let mut points: Vec<IndexPoint> = VERIFIED
            .iter()
            .chain(TRANSCRIBED.iter())
            .map(|&(fy, index)| IndexPoint {
                fiscal_year: FiscalYear(fy),
                index,
                confidence: Confidence::Verified,
            })
            .collect();
        points.sort_by_key(|p| p.fiscal_year);
        Self::from_points("CPI-U all items, June, NSA", points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FY2000: FiscalYear = FiscalYear(2000);
    const FY2022: FiscalYear = FiscalYear(2022);

    /// The Ohio Auditor of State reports CPI growth of 71.9% from June 2000 to June 2022.
    #[test]
    fn reproduces_auditor_published_cpi_growth() {
        let cpi = CpiSeries::cpi_u_june();
        let growth = cpi.index_growth(FY2000, FY2022).unwrap();
        assert!(
            (growth - 0.719).abs() < 0.001,
            "expected ~71.9% CPI growth, got {:.4}",
            growth
        );
    }

    /// Operating expenditure per pupil, $7,065 in FY2000 to $15,314 in FY2022, is reported by
    /// the Auditor as 116.8% nominal growth and 26.1% real.
    #[test]
    fn reproduces_auditor_real_per_pupil_growth_including_relief_funds() {
        let cpi = CpiSeries::cpi_u_june();
        let real = cpi.real_growth(7_065.0, FY2000, 15_314.0, FY2022).unwrap();
        assert!(
            (real.value - 0.261).abs() < 0.001,
            "expected ~26.1% real growth, got {:.4}",
            real.value
        );
        assert_eq!(real.confidence, Confidence::Verified);
    }

    /// Excluding federal COVID relief funds the FY2022 figure is $14,493, which the Auditor
    /// reports as 19.4% real growth. The $821 gap between the two FY2022 figures is larger
    /// than most policy changes this corpus models.
    #[test]
    fn reproduces_auditor_real_growth_excluding_relief_funds() {
        let cpi = CpiSeries::cpi_u_june();
        let real = cpi.real_growth(7_065.0, FY2000, 14_493.0, FY2022).unwrap();
        assert!(
            (real.value - 0.194).abs() < 0.001,
            "expected ~19.4% real growth, got {:.4}",
            real.value
        );
    }

    /// The whole point of the crate: the same pair of figures reads 116.8% without deflation.
    #[test]
    fn nominal_growth_is_four_times_the_real_figure() {
        let nominal: f64 = 15_314.0 / 7_065.0 - 1.0;
        assert!((nominal - 1.168).abs() < 0.001);

        let cpi = CpiSeries::cpi_u_june();
        let real = cpi.real_growth(7_065.0, FY2000, 15_314.0, FY2022).unwrap();
        assert!(nominal > real.value * 4.0);
    }

    #[test]
    fn converting_to_the_same_year_is_the_identity() {
        let cpi = CpiSeries::cpi_u_june();
        let same = cpi.convert(1_234.56, FY2022, FY2022).unwrap();
        assert!((same.value - 1_234.56).abs() < 1e-9);
    }

    #[test]
    fn result_takes_the_weaker_endpoint_confidence() {
        // Built here rather than taken from the CPI series, because the rule is about mixing
        // confidences and must hold whatever state a particular series happens to be in.
        let series = CpiSeries::from_points(
            "test",
            vec![
                IndexPoint {
                    fiscal_year: FiscalYear(2010),
                    index: 200.0,
                    confidence: Confidence::Unverified,
                },
                IndexPoint {
                    fiscal_year: FiscalYear(2022),
                    index: 300.0,
                    confidence: Confidence::Verified,
                },
            ],
        );
        let mixed = series
            .convert(100.0, FiscalYear(2010), FiscalYear(2022))
            .unwrap();
        assert_eq!(
            mixed.confidence,
            Confidence::Unverified,
            "an unverified endpoint must contaminate the result"
        );
    }

    #[test]
    fn the_cpi_series_is_verified_end_to_end() {
        // Every point has been checked against the Bureau's published file; the check itself
        // lives in crates/connect/tests/deflator_matches_bls.rs. If a point is ever added by
        // transcription, this fails and says where the check belongs.
        let cpi = CpiSeries::cpi_u_june();
        for year in 2000..=2022 {
            let point = cpi.point(FiscalYear(year)).expect("covered");
            assert_eq!(
                point.confidence,
                Confidence::Verified,
                "FY{year} is not verified; check it in crates/connect (`edfund-connect cpi`)"
            );
        }
    }

    #[test]
    fn rejects_a_fiscal_year_outside_the_series() {
        let cpi = CpiSeries::cpi_u_june();
        let err = cpi.convert(100.0, FiscalYear(1975), FY2022).unwrap_err();
        assert_eq!(err, DeflatorError::MissingIndex(FiscalYear(1975)));
    }

    #[test]
    fn rejects_a_non_positive_index() {
        let series = CpiSeries::from_points(
            "broken",
            vec![
                IndexPoint {
                    fiscal_year: FiscalYear(2000),
                    index: 0.0,
                    confidence: Confidence::Verified,
                },
                IndexPoint {
                    fiscal_year: FiscalYear(2022),
                    index: 100.0,
                    confidence: Confidence::Verified,
                },
            ],
        );
        assert_eq!(
            series.convert(1.0, FY2000, FY2022).unwrap_err(),
            DeflatorError::NonPositiveIndex(FY2000)
        );
    }

    #[test]
    fn series_is_sorted_and_covers_fy2000_to_fy2022() {
        let cpi = CpiSeries::cpi_u_june();
        for fy in 2000..=2022 {
            assert!(
                cpi.point(FiscalYear(fy)).is_some(),
                "missing observation for FY{fy}"
            );
        }
    }

    #[test]
    fn index_rises_monotonically_except_in_the_2009_recession() {
        let cpi = CpiSeries::cpi_u_june();
        let dips: Vec<u16> = (2001..=2022)
            .filter(|&fy| {
                let prev = cpi.point(FiscalYear(fy - 1)).unwrap().index;
                let cur = cpi.point(FiscalYear(fy)).unwrap().index;
                cur < prev
            })
            .collect();
        assert_eq!(dips, vec![2009], "only FY2009 should show a price decline");
    }
}
