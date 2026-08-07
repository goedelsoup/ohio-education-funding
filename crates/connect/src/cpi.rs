//! The `bls-cpi` connector: the Bureau of Labor Statistics price series.
//!
//! [`deflate`] carries a CPI-U June series as compiled-in constants, and twenty-one of its
//! twenty-three points are marked [`Confidence::Unverified`] — transcribed, never checked
//! against the agency. The crate's own documentation says what that costs: a corpus claim
//! resting on an unverified point must be tagged `[inference]` rather than `[verified]`.
//!
//! This module is what closes that. The Bureau publishes every CPI series as one tab-separated
//! flat file, which needs no API key and no JSON parser, and comparing it to the committed
//! constants is a few dozen lines. The result is a check that can be run, not a promise that
//! the numbers were typed correctly.
//!
//! # Fiscal year alignment
//!
//! Ohio's fiscal year ends 30 June, so the June index is the natural point observation for a
//! fiscal year and FY*N* aligns to June of calendar year *N*. That is a choice, and a
//! defensible alternative is a July-to-June average. The choice is recorded here rather than
//! implied, because applying a calendar-year index to a fiscal-year figure introduces a
//! systematic half-year error that compounds across a long series.

use deflate::{Confidence, CpiSeries};
use edfund_core::FiscalYear;

/// The all-items CPI-U for all urban consumers, not seasonally adjusted.
pub const ALL_ITEMS_NSA: &str = "CUUR0000SA0";

/// The Bureau's period code for June.
pub const JUNE: &str = "M06";

/// One observation from the flat file.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Observation {
    /// Calendar year of the observation.
    pub calendar_year: u16,
    /// The index level.
    pub index: f64,
}

/// Extract one series and period from the Bureau's flat file.
///
/// The file holds every CPI series — tens of millions of rows — so this filters while scanning
/// rather than parsing the whole thing into memory.
#[must_use]
pub fn parse_series(text: &str, series_id: &str, period: &str) -> Vec<Observation> {
    let mut out = Vec::new();
    for line in text.lines().skip(1) {
        let mut fields = line.split('\t').map(str::trim);
        let (Some(series), Some(year), Some(found), Some(value)) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if series != series_id || found != period {
            continue;
        }
        let (Ok(calendar_year), Ok(index)) = (year.parse::<u16>(), value.parse::<f64>()) else {
            continue;
        };
        out.push(Observation {
            calendar_year,
            index,
        });
    }
    out.sort_by_key(|observation| observation.calendar_year);
    out
}

/// How one committed index point compares to what the Bureau publishes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Check {
    /// The fiscal year checked.
    pub fiscal_year: FiscalYear,
    /// What [`deflate`] carries.
    pub committed: f64,
    /// What the Bureau publishes, if the series covers the year.
    pub published: Option<f64>,
    /// Whether the committed point was already marked verified before this check.
    pub was_verified: bool,
}

impl Check {
    /// Whether the committed value matches the published one.
    ///
    /// The comparison is exact to the Bureau's published precision — half of the last digit of
    /// three decimals — and deliberately not looser. A tolerance wide enough to absorb "close
    /// enough" is wide enough to absorb a transposed digit, which is the error a transcribed
    /// constant actually makes. `241.038` for `241.018` sat in this workspace's deflator until
    /// this comparison was tightened; a tenth-of-a-point tolerance had called it agreement.
    #[must_use]
    pub fn agrees(&self) -> bool {
        self.published
            .is_some_and(|published| (published - self.committed).abs() < 0.000_5)
    }
}

/// Compare every point of the committed CPI-U June series to the published file.
#[must_use]
pub fn check_committed_series(observations: &[Observation]) -> Vec<Check> {
    let committed = CpiSeries::cpi_u_june();
    (2000..=2022)
        .filter_map(|year| {
            let fiscal_year = FiscalYear(year);
            let point = committed.point(fiscal_year)?;
            Some(Check {
                fiscal_year,
                committed: point.index,
                published: observations
                    .iter()
                    .find(|observation| observation.calendar_year == year)
                    .map(|observation| observation.index),
                was_verified: point.confidence == Confidence::Verified,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The flat file's shape: a header row, tab separators, and values padded with spaces.
    const SAMPLE: &str = "series_id\tyear\tperiod\tvalue\tfootnote_codes\n\
         CUUR0000SA0     \t2000\tM05\t       171.300\t\n\
         CUUR0000SA0     \t2000\tM06\t       172.400\t\n\
         CUUR0000SA0     \t2022\tM06\t       296.311\t\n\
         CUUR0000SA0     \t2021\tM06\t       271.696\t\n\
         CUUS0000SA0     \t2000\tM06\t         1.000\t\n\
         CUUR0000SAF1    \t2000\tM06\t       167.900\t\n";

    #[test]
    fn selects_one_series_and_one_period() {
        let series = parse_series(SAMPLE, ALL_ITEMS_NSA, JUNE);
        assert_eq!(series.len(), 3);
        assert!(series.iter().all(|o| o.index > 100.0));
    }

    #[test]
    fn returns_observations_in_year_order() {
        let years: Vec<u16> = parse_series(SAMPLE, ALL_ITEMS_NSA, JUNE)
            .iter()
            .map(|o| o.calendar_year)
            .collect();
        assert_eq!(years, [2000, 2021, 2022]);
    }

    #[test]
    fn a_similarly_named_series_is_not_confused_for_the_wanted_one() {
        // CUUS is semi-annual, CUUR0000SAF1 is food. Both would silently corrupt a deflation.
        let series = parse_series(SAMPLE, ALL_ITEMS_NSA, JUNE);
        assert!(!series.iter().any(|o| (o.index - 1.0).abs() < 0.001));
        assert!(!series.iter().any(|o| (o.index - 167.9).abs() < 0.001));
    }

    #[test]
    fn malformed_lines_are_skipped_rather_than_fatal() {
        let text = format!("{SAMPLE}garbage\nCUUR0000SA0\t\tM06\tx\t\n");
        assert_eq!(parse_series(&text, ALL_ITEMS_NSA, JUNE).len(), 3);
    }

    #[test]
    fn the_committed_series_is_checkable_against_a_published_one() {
        let checks = check_committed_series(&parse_series(SAMPLE, ALL_ITEMS_NSA, JUNE));
        assert_eq!(checks.len(), 23, "FY2000 through FY2022");

        let fy2000 = checks.iter().find(|c| c.fiscal_year.0 == 2000).unwrap();
        assert!(fy2000.agrees());

        // FY2021 was transcribed and never checked until this connector existed. It is marked
        // verified now, which is a claim this comparison is what backs.
        let fy2021 = checks.iter().find(|c| c.fiscal_year.0 == 2021).unwrap();
        assert!(fy2021.agrees());
        assert!(
            checks.iter().all(|check| check.was_verified),
            "deflate marks the whole series verified; if that is relaxed, so must this be"
        );
    }

    #[test]
    fn a_year_the_published_file_does_not_cover_does_not_count_as_agreement() {
        let checks = check_committed_series(&parse_series(SAMPLE, ALL_ITEMS_NSA, JUNE));
        let fy2010 = checks.iter().find(|c| c.fiscal_year.0 == 2010).unwrap();
        assert_eq!(fy2010.published, None);
        assert!(
            !fy2010.agrees(),
            "an absent observation must not read as a match"
        );
    }

    #[test]
    fn a_transposed_digit_is_a_difference_not_a_rounding_artefact() {
        // The real case: the deflator carried 241.038 where the Bureau publishes 241.018. Any
        // tolerance loose enough to call that agreement is too loose to be worth running.
        let base = Check {
            fiscal_year: FiscalYear(2016),
            committed: 241.038,
            published: Some(241.018),
            was_verified: false,
        };
        assert!(!base.agrees());
        assert!(Check {
            published: Some(241.038),
            ..base
        }
        .agrees());
    }

    #[test]
    fn trailing_zeros_in_the_committed_form_still_agree() {
        // The early years are written as 178.000 against a published 178.000 — same number,
        // and the exact comparison must not trip over the way it is spelled.
        assert!(Check {
            fiscal_year: FiscalYear(2001),
            committed: 178.000,
            published: Some(178.0),
            was_verified: false,
        }
        .agrees());
    }
}
