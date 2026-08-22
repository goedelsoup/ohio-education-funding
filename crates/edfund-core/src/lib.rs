//! Shared domain types for the Ohio education funding computer.
//!
//! Deliberately dependency-free. Every calculator in this workspace is pure and
//! deterministic — no network, no filesystem, no clock — so that a
//! [`scenario`](../../../.yidam/corpus/scenario/) node can commit its output as a
//! reproducible knowledge claim.
//!
//! # On money and floating point
//!
//! Ohio's funding formulas are ratio-heavy: staffing counts are enrollment divided by a
//! grade-band ratio, local capacity is a weighted blend scaled by a percentage. Fixed-point
//! cents would force conversions at every step. This workspace therefore uses [`f64`] with
//! **explicit rounding at the points the Department of Education and Workforce rounds**, and
//! proves correctness by reproducing published figures to the cent in tests rather than by
//! relying on the type system.
//!
//! Where the department's own worked examples show a rounded intermediate — funded teacher
//! counts at two decimals, average teacher base cost at two decimals — the calculator rounds
//! there too. Getting this wrong changes the answer: the special teacher base cost differs by
//! roughly $365 on a mid-sized district depending on whether the funded count is rounded
//! before multiplication.

#![forbid(unsafe_code)]

pub mod conventions;
pub mod csv;
pub mod decimal;

/// A dollar amount. See the module note on floating point.
pub type Dollars = f64;

/// A count of students, in average daily membership terms. Fractional by construction —
/// ADM is a time-weighted average, so a district can and does have 20,342.13 students.
pub type Adm = f64;

/// A property tax rate in mills — one mill is one dollar per thousand dollars of assessed
/// valuation.
pub type Mills = f64;

/// The minimum state share of the base cost, FY2022.
///
/// A statutory parameter, set by each biennial budget rather than fixed in permanent law.
/// It lives here rather than in a calculator because more than one calculator reads it and
/// nothing about it is derived: [`local_capacity`](../local_capacity/) applies it, and
/// `project`'s FY2027 panel names it as the floor its state-share column is censored at.
pub const MINIMUM_STATE_SHARE_FY2022: Dollars = 0.05;

/// The minimum state share for FY2026 and FY2027, per the department's own model.
///
/// The percentage has **doubled** against [`MINIMUM_STATE_SHARE_FY2022`]: the department's
/// FY2027 calculator states `0.1` for both FY2026 and FY2027, on its `Notes` sheet, in as
/// many words. In the FY2027 model 138 of 609 districts — 22.7% — sit exactly on it. That is
/// a large policy fact, and it is the reason these are two named constants rather than one
/// hard-coded figure applied to every year.
pub const MINIMUM_STATE_SHARE_FY2027: Dollars = 0.10;

/// An Ohio state fiscal year, named for the calendar year it ends in.
///
/// FY2022 runs 1 July 2021 to 30 June 2022. The offset is not cosmetic: applying a
/// calendar-year price index to a fiscal-year figure introduces a systematic half-year error
/// that compounds across a long series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FiscalYear(pub u16);

impl FiscalYear {
    /// The calendar year in which this fiscal year begins.
    #[must_use]
    pub const fn start_calendar_year(self) -> u16 {
        self.0 - 1
    }

    /// The calendar year in which this fiscal year ends, which is also its name.
    #[must_use]
    pub const fn end_calendar_year(self) -> u16 {
        self.0
    }

    /// The biennium label this fiscal year belongs to, given the biennium's first year.
    ///
    /// Ohio biennia begin in odd-numbered fiscal years: FY2022-23, FY2024-25, FY2026-27.
    #[must_use]
    pub const fn biennium_start(self) -> FiscalYear {
        if self.0.is_multiple_of(2) {
            self
        } else {
            FiscalYear(self.0 - 1)
        }
    }
}

impl core::fmt::Display for FiscalYear {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FY{}", self.0)
    }
}

/// Round to whole cents.
#[must_use]
pub fn round_cents(value: f64) -> f64 {
    round_dp(value, 2)
}

/// Round to `places` decimal places.
///
/// Used at the specific points the department's published calculations round. Placing these
/// differently changes published totals, so the call sites are load-bearing and documented
/// individually.
///
/// # Decimal ties are not reliable, and cannot be
///
/// This rounds a binary float, so a value that *looks* like an exact decimal tie usually is
/// not one. `1.005` is stored as `1.00499999999999989…` and rounds **down** to `1.00`, not up
/// to `1.01`. No amount of care inside this function changes that; the tie never reaches it.
///
/// This is acceptable here because Ohio's published figures are not ties — funded teacher
/// counts like `851.52019` and salary products like `86993.0988` sit well away from the
/// midpoint. The guarantee this workspace actually relies on is not a rounding rule but the
/// regression tests that reproduce the department's published totals to the cent. If a future
/// input does land on a genuine decimal tie, that figure needs decimal arithmetic, not a
/// different `f64` rounding mode.
#[must_use]
pub fn round_dp(value: f64, places: u32) -> f64 {
    let factor = 10_f64.powi(places as i32);
    (value * factor).round() / factor
}

/// The kind of education agency, which determines several statutory thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgencyType {
    /// A city school district.
    City,
    /// A local school district.
    Local,
    /// An exempted village school district.
    ExemptedVillage,
    /// A joint vocational school district. Subject to a 2-mill reduction-factor floor rather
    /// than the 20-mill floor that applies to the other district types.
    JointVocational,
    /// A community (charter) school.
    Community,
    /// A STEM school.
    Stem,
}

impl AgencyType {
    /// Whether this agency levies property tax in its own right.
    ///
    /// Community and STEM schools do not; they are funded directly by the state under the
    /// Fair School Funding Plan and by resident-district deduction before it.
    #[must_use]
    pub const fn levies_property_tax(self) -> bool {
        matches!(
            self,
            Self::City | Self::Local | Self::ExemptedVillage | Self::JointVocational
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fiscal_year_spans_two_calendar_years() {
        let fy = FiscalYear(2022);
        assert_eq!(fy.start_calendar_year(), 2021);
        assert_eq!(fy.end_calendar_year(), 2022);
    }

    #[test]
    fn fiscal_year_displays_with_prefix() {
        assert_eq!(FiscalYear(2026).to_string(), "FY2026");
    }

    #[test]
    fn biennium_starts_on_even_fiscal_years() {
        assert_eq!(FiscalYear(2022).biennium_start(), FiscalYear(2022));
        assert_eq!(FiscalYear(2023).biennium_start(), FiscalYear(2022));
        assert_eq!(FiscalYear(2026).biennium_start(), FiscalYear(2026));
        assert_eq!(FiscalYear(2027).biennium_start(), FiscalYear(2026));
    }

    #[test]
    fn rounds_to_two_places_away_from_the_midpoint() {
        assert!((round_cents(1.006) - 1.01).abs() < 1e-9);
        assert!((round_cents(2.344_9) - 2.34).abs() < 1e-9);
        assert!((round_cents(-1.006) - -1.01).abs() < 1e-9);
        assert!((round_cents(-2.344_9) - -2.34).abs() < 1e-9);
    }

    /// Documents a limitation rather than asserting a guarantee. `1.005` is not representable
    /// in binary and is stored just below the midpoint, so it rounds down. The workspace's
    /// correctness rests on reproducing published totals, not on decimal tie behaviour.
    #[test]
    fn apparent_decimal_ties_round_by_their_binary_value() {
        assert!(
            (round_cents(1.005) - 1.00).abs() < 1e-9,
            "1.005 is stored below the midpoint and rounds down"
        );
        // Nudging the value above the true midpoint restores the expected direction, which
        // confirms the cause is representation rather than the rounding rule.
        let above_midpoint = 1.005_f64 + f64::EPSILON;
        assert!((round_cents(above_midpoint) - 1.01).abs() < 1e-9);
    }

    #[test]
    fn rounds_funded_teacher_counts_to_two_places() {
        // The department publishes 851.52 funded classroom teachers for the worked example.
        assert!((round_dp(851.520_19, 2) - 851.52).abs() < 1e-9);
    }

    #[test]
    fn jvsd_levies_but_community_school_does_not() {
        assert!(AgencyType::JointVocational.levies_property_tax());
        assert!(AgencyType::City.levies_property_tax());
        assert!(!AgencyType::Community.levies_property_tax());
        assert!(!AgencyType::Stem.levies_property_tax());
    }
}
