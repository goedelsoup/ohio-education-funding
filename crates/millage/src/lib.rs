//! Effective operating millage under H.B. 920 reduction factors, and floor status.
//!
//! H.B. 920 (1976) created the tax reduction factors at R.C. 319.301: as reappraisal raises the
//! assessed value of **existing** real property, the effective millage of previously voted
//! levies is reduced so the levy yields approximately the dollars it yielded when passed.
//! Revenue from an existing levy therefore does not grow with inflation or with property
//! values. Growth comes from new construction, from new levies, or from hitting the floor.
//!
//! # The floor is a regime switch, not a threshold
//!
//! Below the floor, reduction factors stop operating entirely and revenue rises directly with
//! valuation. Above it, valuation growth is neutralised. Two districts on opposite sides of the
//! floor respond to the same reappraisal in opposite directions — which is why any statewide
//! claim about a property tax change that has not partitioned districts by floor status is
//! unreliable, and frequently wrong in sign for a large share of them.
//!
//! The floor is 20 mills for city, local, and exempted village districts and **2 mills** for
//! joint vocational school districts.

#![forbid(unsafe_code)]

use edfund_core::{AgencyType, Dollars, Mills};

/// Statutory reduction-factor floor for school districts, in mills.
pub const SCHOOL_DISTRICT_FLOOR: Mills = 20.0;
/// Statutory reduction-factor floor for joint vocational school districts, in mills.
pub const JVSD_FLOOR: Mills = 2.0;

/// The floor that applies to an agency type.
///
/// Community and STEM schools do not levy property tax and have no floor.
#[must_use]
pub const fn floor_for(agency: AgencyType) -> Option<Mills> {
    match agency {
        AgencyType::City | AgencyType::Local | AgencyType::ExemptedVillage => {
            Some(SCHOOL_DISTRICT_FLOOR)
        }
        AgencyType::JointVocational => Some(JVSD_FLOOR),
        AgencyType::Community | AgencyType::Stem => None,
    }
}

/// Which side of the reduction-factor floor a district sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorStatus {
    /// Reduction factors are operating: valuation growth on existing property does not reach
    /// revenue.
    AboveFloor,
    /// Reduction factors have stopped: valuation growth passes through to revenue directly,
    /// and reappraisal becomes a revenue event.
    AtFloor,
    /// The agency does not levy property tax, so the concept does not apply.
    NotApplicable,
}

impl FloorStatus {
    /// Whether valuation growth on existing property reaches this district's revenue.
    #[must_use]
    pub const fn valuation_growth_reaches_revenue(self) -> bool {
        matches!(self, Self::AtFloor)
    }
}

/// The result of applying reduction factors to a district's voted millage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MillageResult {
    /// Voted operating millage, before reduction factors.
    pub voted: Mills,
    /// Effective operating millage after reduction factors and the floor.
    pub effective: Mills,
    /// The reduction factor applied, as a fraction of voted millage removed. Zero when the
    /// floor binds.
    pub reduction_factor: f64,
    /// Which side of the floor the district ends on.
    pub status: FloorStatus,
}

/// Why an effective millage could not be computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MillageError {
    /// Current valuation is zero or negative.
    NonPositiveCurrentValuation,
    /// Base-year valuation is zero or negative.
    NonPositiveBaseValuation,
    /// Voted millage is negative.
    NegativeVotedMillage,
}

impl core::fmt::Display for MillageError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let msg = match self {
            Self::NonPositiveCurrentValuation => "current valuation must be positive",
            Self::NonPositiveBaseValuation => "base-year valuation must be positive",
            Self::NegativeVotedMillage => "voted millage must not be negative",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for MillageError {}

/// Compute effective operating millage under H.B. 920, applying the statutory floor.
///
/// `base_valuation` is the assessed valuation of the carryover property when the levies were
/// voted; `current_valuation` is that same property's valuation now. Growth from new
/// construction is excluded from both by construction — reduction factors do not apply to it,
/// which is why new construction is one of the three ways an Ohio district's revenue grows.
///
/// # Errors
///
/// Returns [`MillageError`] if either valuation is non-positive or voted millage is negative.
pub fn effective_millage(
    voted: Mills,
    base_valuation: Dollars,
    current_valuation: Dollars,
    agency: AgencyType,
) -> Result<MillageResult, MillageError> {
    if voted < 0.0 {
        return Err(MillageError::NegativeVotedMillage);
    }
    if current_valuation <= 0.0 {
        return Err(MillageError::NonPositiveCurrentValuation);
    }
    if base_valuation <= 0.0 {
        return Err(MillageError::NonPositiveBaseValuation);
    }

    // Hold the dollar yield constant: rate falls in proportion to valuation growth.
    let unfloored = voted * (base_valuation / current_valuation);
    let unfloored = unfloored.min(voted); // reduction factors never raise a rate

    match floor_for(agency) {
        None => Ok(MillageResult {
            voted,
            effective: 0.0,
            reduction_factor: 0.0,
            status: FloorStatus::NotApplicable,
        }),
        Some(floor) => {
            let binds = unfloored < floor && voted >= floor;
            let effective = if binds { floor } else { unfloored };
            let status = if effective <= floor {
                FloorStatus::AtFloor
            } else {
                FloorStatus::AboveFloor
            };
            Ok(MillageResult {
                voted,
                effective,
                reduction_factor: if voted > 0.0 {
                    1.0 - effective / voted
                } else {
                    0.0
                },
                status,
            })
        }
    }
}

/// The dollar yield of an effective millage against a valuation.
///
/// One mill is one dollar per thousand dollars of assessed valuation.
#[must_use]
pub fn yield_of(effective: Mills, valuation: Dollars) -> Dollars {
    effective * valuation / 1_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jvsd_floor_is_two_mills_not_twenty() {
        assert_eq!(floor_for(AgencyType::JointVocational), Some(JVSD_FLOOR));
        assert_eq!(floor_for(AgencyType::City), Some(SCHOOL_DISTRICT_FLOOR));
        assert_eq!(floor_for(AgencyType::Local), Some(SCHOOL_DISTRICT_FLOOR));
        assert_eq!(
            floor_for(AgencyType::ExemptedVillage),
            Some(SCHOOL_DISTRICT_FLOOR)
        );
    }

    #[test]
    fn community_and_stem_schools_have_no_floor() {
        assert_eq!(floor_for(AgencyType::Community), None);
        assert_eq!(floor_for(AgencyType::Stem), None);
    }

    /// The core H.B. 920 behaviour: doubling valuation halves the effective rate, leaving the
    /// dollar yield unchanged.
    #[test]
    fn doubling_valuation_halves_effective_millage_and_holds_yield_flat() {
        let before = effective_millage(40.0, 1_000_000.0, 1_000_000.0, AgencyType::City).unwrap();
        let after = effective_millage(40.0, 1_000_000.0, 2_000_000.0, AgencyType::City).unwrap();

        assert!((before.effective - 40.0).abs() < 1e-9);
        assert!((after.effective - 20.0).abs() < 1e-9);

        let yield_before = yield_of(before.effective, 1_000_000.0);
        let yield_after = yield_of(after.effective, 2_000_000.0);
        assert!(
            (yield_before - yield_after).abs() < 1e-6,
            "H.B. 920 holds the dollar yield constant: {yield_before} vs {yield_after}"
        );
    }

    /// The mechanism that produces real decline without anyone deciding anything: nominal
    /// revenue is flat while costs rise.
    #[test]
    fn revenue_is_nominally_flat_across_repeated_reappraisals() {
        let yields: Vec<f64> = [1.0_f64, 1.15, 1.35, 1.60]
            .iter()
            .map(|growth| {
                let valuation = 1_000_000.0 * growth;
                let m = effective_millage(40.0, 1_000_000.0, valuation, AgencyType::City).unwrap();
                yield_of(m.effective, valuation)
            })
            .collect();
        for y in &yields {
            assert!((y - yields[0]).abs() < 1e-6, "yields drifted: {yields:?}");
        }
    }

    #[test]
    fn floor_binds_once_reduction_would_push_below_twenty_mills() {
        // Voted 30 mills, valuation tripled: unfloored rate would be 10 mills.
        let m = effective_millage(30.0, 1_000_000.0, 3_000_000.0, AgencyType::City).unwrap();
        assert!((m.effective - SCHOOL_DISTRICT_FLOOR).abs() < 1e-9);
        assert_eq!(m.status, FloorStatus::AtFloor);
    }

    /// The consequence that matters for policy: at the floor, valuation growth becomes revenue.
    #[test]
    fn at_the_floor_valuation_growth_reaches_revenue() {
        let modest = effective_millage(30.0, 1_000_000.0, 3_000_000.0, AgencyType::City).unwrap();
        let larger = effective_millage(30.0, 1_000_000.0, 4_000_000.0, AgencyType::City).unwrap();

        assert!(modest.status.valuation_growth_reaches_revenue());
        assert!(larger.status.valuation_growth_reaches_revenue());

        let yield_modest = yield_of(modest.effective, 3_000_000.0);
        let yield_larger = yield_of(larger.effective, 4_000_000.0);
        assert!(
            yield_larger > yield_modest,
            "at the floor, reappraisal is a revenue event"
        );
    }

    /// The same reappraisal moves two districts in opposite directions.
    #[test]
    fn reappraisal_moves_floored_and_unfloored_districts_oppositely() {
        // District A voted 60 mills and is far above the floor.
        let a_before = effective_millage(60.0, 1_000_000.0, 1_000_000.0, AgencyType::City).unwrap();
        let a_after = effective_millage(60.0, 1_000_000.0, 1_200_000.0, AgencyType::City).unwrap();
        let a_yield_before = yield_of(a_before.effective, 1_000_000.0);
        let a_yield_after = yield_of(a_after.effective, 1_200_000.0);

        // District B voted 22 mills and is already at the floor.
        let b_before = effective_millage(22.0, 1_000_000.0, 1_500_000.0, AgencyType::City).unwrap();
        let b_after = effective_millage(22.0, 1_000_000.0, 1_800_000.0, AgencyType::City).unwrap();
        let b_yield_before = yield_of(b_before.effective, 1_500_000.0);
        let b_yield_after = yield_of(b_after.effective, 1_800_000.0);

        assert!(
            (a_yield_after - a_yield_before).abs() < 1e-6,
            "A stays flat"
        );
        assert!(b_yield_after > b_yield_before, "B gains");
    }

    #[test]
    fn a_jvsd_reaches_its_floor_far_later_than_a_district_would() {
        // 6 voted mills, valuation up sixfold: unfloored rate is 1 mill.
        let as_jvsd =
            effective_millage(6.0, 1_000_000.0, 6_000_000.0, AgencyType::JointVocational).unwrap();
        assert!((as_jvsd.effective - JVSD_FLOOR).abs() < 1e-9);
        assert_eq!(as_jvsd.status, FloorStatus::AtFloor);

        // The same numbers for a city district: voted millage is already below 20, so the
        // floor cannot bind — a district may not levy up to a floor it never voted.
        let as_city = effective_millage(6.0, 1_000_000.0, 6_000_000.0, AgencyType::City).unwrap();
        assert!(as_city.effective < SCHOOL_DISTRICT_FLOOR);
    }

    #[test]
    fn floor_never_raises_a_rate_above_what_voters_approved() {
        let m = effective_millage(12.0, 1_000_000.0, 1_000_000.0, AgencyType::City).unwrap();
        assert!(
            m.effective <= 12.0,
            "a district that voted 12 mills cannot be pushed to 20"
        );
    }

    #[test]
    fn reduction_factors_never_increase_a_rate_when_valuation_falls() {
        let m = effective_millage(30.0, 2_000_000.0, 1_000_000.0, AgencyType::City).unwrap();
        assert!(
            (m.effective - 30.0).abs() < 1e-9,
            "falling valuation must not push the rate above voted millage"
        );
    }

    #[test]
    fn community_school_millage_is_not_applicable() {
        let m = effective_millage(20.0, 1_000_000.0, 1_000_000.0, AgencyType::Community).unwrap();
        assert_eq!(m.status, FloorStatus::NotApplicable);
        assert!(!m.status.valuation_growth_reaches_revenue());
    }

    #[test]
    fn rejects_non_positive_valuations_and_negative_millage() {
        assert_eq!(
            effective_millage(20.0, 1.0, 0.0, AgencyType::City),
            Err(MillageError::NonPositiveCurrentValuation)
        );
        assert_eq!(
            effective_millage(20.0, 0.0, 1.0, AgencyType::City),
            Err(MillageError::NonPositiveBaseValuation)
        );
        assert_eq!(
            effective_millage(-1.0, 1.0, 1.0, AgencyType::City),
            Err(MillageError::NegativeVotedMillage)
        );
    }

    #[test]
    fn one_mill_yields_one_dollar_per_thousand_of_valuation() {
        assert!((yield_of(1.0, 1_000.0) - 1.0).abs() < 1e-12);
        assert!((yield_of(20.0, 1_000_000.0) - 20_000.0).abs() < 1e-9);
    }
}
