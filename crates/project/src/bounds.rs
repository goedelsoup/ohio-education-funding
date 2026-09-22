//! Every bound the modelled formula contains, and which of them are ever the operative term.
//!
//! Issue #410 asked, after #389 found one by accident, how many of the plan's floors, ceilings
//! and clamps share R.C. 3317.011(F)(6)(c)'s condition: a bound written into the section that is
//! never what decides a district's number. It asked for two predicates kept apart —
//!
//! - **cannot bind**: unreachable by construction, a property of the function provable without
//!   data and a drafting observation;
//! - **has never bound**: reachable in principle and reached by no district in a committed year,
//!   a property of Ohio and a policy fact.
//!
//! — and it asked that a null result be reported as one. This module is the census that answers
//! it. [`Bound`] names every place the modelled formula takes a greater-of, a lesser-of, a
//! ceiling or a clamp at zero; [`Bound::reach`] states the first predicate for each as a property
//! of the arithmetic; [`census`] measures the second over the department's FY2027 model.
//!
//! # What the census finds
//!
//! **Thirty-eight bounds, and (F)(6)(c) is the only one that cannot bind.** Every other bound in
//! the modelled formula is reachable, and every reachable bound is reached by at least one
//! district in FY2027. So the second list — bounds that could bind and never have — is **empty**.
//! There is no policy written into the plan that has never been applied, on this panel; there is
//! one sentence in one section with no force.
//!
//! The nearest things to a never-bound provision are two bounds reached by **exactly one**
//! district each, which is the extreme case of reachable-and-barely-reached:
//!
//! - the department's cap of the DPIA blended count at enrolled ADM, which only Edgerton Local
//!   reaches — 116 districts report more economically disadvantaged pupils than enrolled ones,
//!   and Edgerton is the one whose 65/35 blend still exceeds its enrolment;
//! - R.C. 3317.019(A)(1)'s clamp of the guarantee's floor at zero, reached only by Richmond
//!   Heights Local, whose FY2020 funding base is negative after the deductions R.C.
//!   3317.02(N)(1)(b) subtracts.
//!
//! A third is nearly so: the open-enrolment clawback's clamp at zero binds for 22 of the 43
//! districts charged, and for one of those — West Muskingum Local — it binds *partially*, taking
//! all $7,105.69 of the guarantee it had and leaving it at exactly zero.
//!
//! One bound has a population fixed by construction rather than by Ohio. R.C. 3317.017(A)(4)(d)(i)
//! caps the local capacity percentage at 0.025 for every district whose income ratio is at or
//! above the fortieth highest district's — so the cap binds for **40 of 609** whatever the
//! incomes are. It is a ceiling that cannot fail to bind and cannot bind for more, ties aside,
//! and the census reports it as such rather than as a fact about the distribution.
//!
//! # What "binds" means here, bound by bound
//!
//! Every bound is a comparison with two sides, and the census counts the districts on the side
//! the provision exists to produce. For a floor that is the districts funded at the floor; for a
//! ceiling, those held at it; for a clamp at zero, those clamped; for a hold-harmless
//! `max(base − computed, 0)`, those *held* — the guarantee's 294 and not the 315. For the three
//! lesser-of and greater-of rules that compare a single year against a three-year average, it is
//! the districts on the **single-year** branch, because the average is the default and the
//! single year is the departure the rule exists to make. [`Bound::label`] says which side each
//! count is for, so a row cannot be read the other way.
//!
//! A count of districts *on* a bound is not the count *moved* by it. 138 districts sit on the
//! minimum state share and 31 move when it is lowered, because the guarantee absorbs the rest —
//! `crates/scenario-delta/tests/the_minimum_share_the_guarantee_has_already_paid_past.rs`. The
//! same distinction is measured here for one bound the corpus had not priced: R.C.
//! 3317.019(C)(1)'s floor of twenty pupils under the clawback's decrease threshold governs the
//! threshold for 499 districts and [`spared_by_the_decrease_threshold_floor`] counts the ones it
//! actually kept off the charge.
//!
//! # What is not a bound, and is deliberately not in the table
//!
//! Steps. The plan has four two-branch tests on a boolean or a rank that pay everything on one
//! side and nothing on the other, and a `max` is not one of them: R.C. 3317.011(A)(11)'s
//! athletics eligibility (dormant — all 609 districts qualify), the enrolment growth
//! supplement's 3% gate (43 clear it), the performance supplement's rating gates, and the
//! repealed supplemental targeted assistance tier's two FY2019 gates (36 qualify and are paid
//! nothing, because the section is repealed). Each is measured elsewhere in this crate; none is a
//! bound in the sense #410 asked about, and listing them beside the floors would invite the cliff
//! reading `crate::staffing_minimums` exists to refute.
//!
//! Prorations are a `min(appropriation / entitlement, 1)` taken once statewide rather than once
//! per district, and the panel carries three: regular transportation at exactly 1.0 (the cap
//! binds), special education transportation at 0.917 (it does not), and preschool special
//! education's published 0.9685 against a year whose appropriation covers the programme. They
//! are in [`crate::transport`] and [`crate::prior_model`] and not here, because a row with a
//! population of one is not a census.

use foundation::minimums::{Ceiling, Minimum};
use foundation::ratios;

use crate::panel::{
    DistrictRecord, TargetedAssistance, DPIA_BLEND, GIFTED_COORDINATOR_DIVISOR,
    GIFTED_COORDINATOR_UNIT_BOUNDS, GIFTED_SPECIALIST_DIVISOR, GIFTED_SPECIALIST_UNIT_FLOOR,
    OPEN_ENROLLMENT_THRESHOLD_FLOOR, OPEN_ENROLLMENT_THRESHOLD_FRACTION, TA_CAPACITY_FULL_AT,
    TA_CAPACITY_MINIMUM_ADM, TA_WEALTH_INDEX_FLOOR, TRANSPORT_DENSITY_PIVOT,
    TRANSPORT_EFFICIENCY_BAND,
};
use crate::transport::MINIMUM_STATE_SHARE_FY2027 as TRANSPORTATION_FLOOR;

/// The arithmetic a bound is written as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operator {
    /// The greater of a computed quantity and a fixed number.
    Floor,
    /// The lesser of a computed quantity and a fixed number.
    Ceiling,
    /// A difference that may not go below zero.
    ClampAtZero,
    /// The greater of two computed quantities, neither of them fixed.
    GreaterOf,
    /// The lesser of two computed quantities, neither of them fixed.
    LesserOf,
}

/// Which of the plan's instruments a bound sits in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Family {
    /// R.C. 3317.011 and the definition it funds on.
    BaseCost,
    /// R.C. 3317.017 and the state share it produces.
    LocalCapacity,
    /// The categoricals of R.C. 3317.022, 3317.051 and 3317.0217.
    Categoricals,
    /// R.C. 3317.019 and the uncodified supplement beside it.
    Guarantee,
    /// R.C. 3317.0212 and the special education transportation line.
    Transportation,
}

impl Family {
    /// The five, in the order the plan computes them.
    pub const ALL: [Self; 5] = [
        Self::BaseCost,
        Self::LocalCapacity,
        Self::Categoricals,
        Self::Guarantee,
        Self::Transportation,
    ];

    /// What the family is called, in words — a heading rather than a citation.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BaseCost => "Base cost",
            Self::LocalCapacity => "Local capacity",
            Self::Categoricals => "Categoricals",
            Self::Guarantee => "Guarantee",
            Self::Transportation => "Transportation",
        }
    }

    /// The sections the family's bounds are stated in.
    ///
    /// Every member's own [`Bound::authority`] is inside one of these, which
    /// `the_bounds_that_cannot_bind_and_the_ones_that_never_have.rs` asserts rather than trusts:
    /// a bound filed under the wrong family would otherwise be a heading that is quietly wrong
    /// about which instrument a reader is looking at.
    #[must_use]
    pub const fn authority(self) -> &'static str {
        match self {
            Self::BaseCost => "R.C. 3317.011, 3317.02",
            Self::LocalCapacity => "R.C. 3317.017",
            Self::Categoricals => "R.C. 3317.022, 3317.051, 3317.0217",
            Self::Guarantee => "R.C. 3317.019, H.B. 110 Section 265.225",
            // R.C. 3317.019(A)(2) is here and not under the guarantee because the bound it
            // states is transportation's: the section that holds a district harmless holds
            // its transportation payment harmless in its own division, and the family a
            // bound belongs to is the instrument it bounds rather than the section it is
            // printed in.
            Self::Transportation => "R.C. 3317.0212, R.C. 3317.019(A)(2)",
        }
    }

    /// How many of the modelled formula's bounds sit in this family.
    #[must_use]
    pub fn size(self) -> usize {
        Bound::all()
            .into_iter()
            .filter(|b| b.family() == self)
            .count()
    }
}

/// Whether any input at all can put a bound in force — a property of the function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reach {
    /// Some district could be on it.
    Reachable,
    /// None can, as the section stands: the operator's own argument is already bounded past it.
    Unreachable,
    /// A fixed number of districts are on it whatever their inputs, because the bound is stated
    /// against a rank.
    Fixed(usize),
}

/// One bound in the modelled formula.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bound {
    /// One of R.C. 3317.011's eight staffing floors — see [`Minimum`].
    StaffingFloor(Minimum),
    /// One of its two staffing ceilings — see [`Ceiling`].
    StaffingCeiling(Ceiling),
    /// The size-banded salaries' small end: below 500 ADM the small figure applies unramped.
    SizeBandSmall,
    /// Their large end: above 4,000 ADM the large figure applies unramped.
    SizeBandLarge,
    /// Base cost enrolled ADM is the greater of the previous year and the three-year average.
    BaseCostEnrolledAdm,
    /// Capacity valuation is the lesser of the most recent tax year and the three-year average.
    ValuationLesserOf,
    /// Capacity income is the lesser of the most recent tax year and the three-year average.
    IncomeLesserOf,
    /// The local capacity percentage is capped at 0.025 from the fortieth-ranked district up.
    CapacityRateCeiling,
    /// The state pays at least 10% of base cost, however wealthy the district.
    MinimumStateShare,
    /// The department caps the DPIA blended count at enrolled ADM.
    DpiaCountCap,
    /// Gifted coordinator units floor at 0.5.
    GiftedCoordinatorFloor,
    /// And cap at 8.
    GiftedCoordinatorCeiling,
    /// K-8 gifted intervention specialist units floor at 0.3.
    GiftedSpecialistK8Floor,
    /// 9-12 gifted intervention specialist units floor at 0.3.
    GiftedSpecialist912Floor,
    /// The capacity tier pays nothing to a district at or above the median district's wealth.
    CapacityTierZero,
    /// And nothing to a district under 200 enrolled ADM, however poor.
    CapacityTierSizeCutoff,
    /// From 200 to 600 ADM it pays a fraction, 5% to a shelf and then a ramp.
    CapacityTierSizeRamp,
    /// The wealth tier pays nothing to a district whose wealth index is under 0.8.
    WealthTierZero,
    /// The guarantee's floor is the funding base less the clawback, and not below zero.
    FundingBaseClampAtZero,
    /// The guarantee itself: the greater of the formula and that floor.
    Guarantee,
    /// The clawback's decrease threshold is the greater of ten per cent of last year's
    /// open-enrolment count and twenty.
    DecreaseThresholdFloor,
    /// The clawback is taken off the guarantee and cannot take it below zero.
    ClawbackClampAtZero,
    /// The formula transition supplement: the FY2021 base less everything else, or nothing.
    TransitionSupplement,
    /// Transportation is paid at the greater of the district's share and 50%.
    TransportationFloor,
    /// The transportation base is the greater of the rider amount and the mile amount.
    MileBase,
    /// The efficiency adjustment is nothing below an index of 1.0.
    EfficiencyZero,
    /// And is capped at 15% of the base from an index of 1.5.
    EfficiencyCeiling,
    /// The density supplement is nothing above 28 riders per square mile.
    DensityZero,
    /// Transportation's own guarantee: the FY2020 amount less the computed one, or nothing.
    TransportationGuarantee,
    /// Special education transportation is reimbursed at the greater of the share and 50%.
    SpecialEducationTransportFloor,
}

impl Bound {
    /// Every bound the modelled formula contains, in statutory order.
    #[must_use]
    pub fn all() -> Vec<Self> {
        let mut all: Vec<Self> = Minimum::ALL
            .iter()
            .map(|m| Self::StaffingFloor(*m))
            .collect();
        all.extend(Ceiling::ALL.iter().map(|c| Self::StaffingCeiling(*c)));
        all.extend([
            Self::SizeBandSmall,
            Self::SizeBandLarge,
            Self::BaseCostEnrolledAdm,
            Self::ValuationLesserOf,
            Self::IncomeLesserOf,
            Self::CapacityRateCeiling,
            Self::MinimumStateShare,
            Self::DpiaCountCap,
            Self::GiftedCoordinatorFloor,
            Self::GiftedCoordinatorCeiling,
            Self::GiftedSpecialistK8Floor,
            Self::GiftedSpecialist912Floor,
            Self::CapacityTierZero,
            Self::CapacityTierSizeCutoff,
            Self::CapacityTierSizeRamp,
            Self::WealthTierZero,
            Self::FundingBaseClampAtZero,
            Self::Guarantee,
            Self::DecreaseThresholdFloor,
            Self::ClawbackClampAtZero,
            Self::TransitionSupplement,
            Self::TransportationFloor,
            Self::MileBase,
            Self::EfficiencyZero,
            Self::EfficiencyCeiling,
            Self::DensityZero,
            Self::TransportationGuarantee,
            Self::SpecialEducationTransportFloor,
        ]);
        all
    }

    /// The instrument the bound sits in.
    #[must_use]
    pub const fn family(self) -> Family {
        match self {
            Self::StaffingFloor(_)
            | Self::StaffingCeiling(_)
            | Self::SizeBandSmall
            | Self::SizeBandLarge
            | Self::BaseCostEnrolledAdm => Family::BaseCost,
            Self::ValuationLesserOf
            | Self::IncomeLesserOf
            | Self::CapacityRateCeiling
            | Self::MinimumStateShare => Family::LocalCapacity,
            Self::DpiaCountCap
            | Self::GiftedCoordinatorFloor
            | Self::GiftedCoordinatorCeiling
            | Self::GiftedSpecialistK8Floor
            | Self::GiftedSpecialist912Floor
            | Self::CapacityTierZero
            | Self::CapacityTierSizeCutoff
            | Self::CapacityTierSizeRamp
            | Self::WealthTierZero => Family::Categoricals,
            Self::FundingBaseClampAtZero
            | Self::Guarantee
            | Self::DecreaseThresholdFloor
            | Self::ClawbackClampAtZero
            | Self::TransitionSupplement => Family::Guarantee,
            Self::TransportationFloor
            | Self::MileBase
            | Self::EfficiencyZero
            | Self::EfficiencyCeiling
            | Self::DensityZero
            | Self::TransportationGuarantee
            | Self::SpecialEducationTransportFloor => Family::Transportation,
        }
    }

    /// The arithmetic the bound is written as.
    #[must_use]
    pub const fn operator(self) -> Operator {
        match self {
            Self::StaffingFloor(_)
            | Self::MinimumStateShare
            | Self::GiftedCoordinatorFloor
            | Self::GiftedSpecialistK8Floor
            | Self::GiftedSpecialist912Floor
            | Self::DecreaseThresholdFloor
            | Self::TransportationFloor
            | Self::SpecialEducationTransportFloor => Operator::Floor,
            Self::StaffingCeiling(_)
            | Self::SizeBandSmall
            | Self::SizeBandLarge
            | Self::CapacityRateCeiling
            | Self::DpiaCountCap
            | Self::GiftedCoordinatorCeiling
            | Self::CapacityTierSizeCutoff
            | Self::CapacityTierSizeRamp
            | Self::EfficiencyZero
            | Self::EfficiencyCeiling => Operator::Ceiling,
            Self::CapacityTierZero
            | Self::WealthTierZero
            | Self::FundingBaseClampAtZero
            | Self::ClawbackClampAtZero
            | Self::TransitionSupplement
            | Self::DensityZero
            | Self::TransportationGuarantee => Operator::ClampAtZero,
            Self::BaseCostEnrolledAdm | Self::Guarantee | Self::MileBase => Operator::GreaterOf,
            Self::ValuationLesserOf | Self::IncomeLesserOf => Operator::LesserOf,
        }
    }

    /// The provision that states it.
    #[must_use]
    pub const fn authority(self) -> &'static str {
        match self {
            Self::StaffingFloor(Minimum::SpecialTeachers) => "R.C. 3317.011(D)(2)(b)-(c)",
            Self::StaffingFloor(Minimum::Counselors) => "R.C. 3317.011(E)(1)(b)",
            Self::StaffingFloor(Minimum::Wellness) => "R.C. 3317.011(E)(3)(b)",
            Self::StaffingFloor(Minimum::OtherAdministrators) => "R.C. 3317.011(F)(3)(c)",
            Self::StaffingFloor(Minimum::FiscalSupport) => "R.C. 3317.011(F)(4)(b)(i)",
            Self::StaffingFloor(Minimum::Emis) => "R.C. 3317.011(F)(5)(b)",
            Self::StaffingFloor(Minimum::LeadershipSupport) => "R.C. 3317.011(F)(6)(c)",
            Self::StaffingFloor(Minimum::BuildingSupport) => "R.C. 3317.011(G)(2)(c)(i)",
            Self::StaffingCeiling(Ceiling::FiscalSupport) => "R.C. 3317.011(F)(4)(b)(ii)",
            Self::StaffingCeiling(Ceiling::BuildingSupport) => "R.C. 3317.011(G)(2)(c)(ii)",
            Self::SizeBandSmall | Self::SizeBandLarge => "R.C. 3317.011(A)(10)",
            Self::BaseCostEnrolledAdm => "R.C. 3317.02(C)",
            Self::ValuationLesserOf => "R.C. 3317.017(A)(1)(a)",
            Self::IncomeLesserOf => "R.C. 3317.017(A)(2)(a)",
            Self::CapacityRateCeiling => "R.C. 3317.017(A)(4)(d)(i)",
            Self::MinimumStateShare => "R.C. 3317.017(C)",
            Self::DpiaCountCap => "the department's DPIA sheet; R.C. 3317.022(A)(4) states no cap",
            Self::GiftedCoordinatorFloor | Self::GiftedCoordinatorCeiling => {
                "R.C. 3317.051(A)(1)(a)"
            }
            Self::GiftedSpecialistK8Floor => "R.C. 3317.051(A)(1)(b)",
            Self::GiftedSpecialist912Floor => "R.C. 3317.051(A)(1)(c)",
            Self::CapacityTierZero => "R.C. 3317.0217(B)(4)(a)(i)",
            Self::CapacityTierSizeCutoff => "R.C. 3317.0217(B)(4)(a)(ii)",
            Self::CapacityTierSizeRamp => "R.C. 3317.0217(B)(4)(b)(ii)-(iii)",
            Self::WealthTierZero => "R.C. 3317.0217(C)(4)(a)",
            Self::FundingBaseClampAtZero | Self::Guarantee => "R.C. 3317.019(A)(1)",
            Self::DecreaseThresholdFloor => "R.C. 3317.019(C)(1)",
            Self::ClawbackClampAtZero => "R.C. 3317.019(C)(2)",
            Self::TransitionSupplement => "H.B. 110 Section 265.225, extended by H.B. 96",
            Self::TransportationFloor => "R.C. 3317.0212(E)(1)(c)(ii)",
            Self::MileBase => "R.C. 3317.0212(E)(1)(c)",
            Self::EfficiencyZero => "R.C. 3317.0212(F)(3)(c)",
            Self::EfficiencyCeiling => "R.C. 3317.0212(F)(3)(a)",
            Self::DensityZero => "R.C. 3317.0212(H)(1)",
            Self::TransportationGuarantee => "R.C. 3317.019(A)(2)",
            Self::SpecialEducationTransportFloor => {
                "R.C. 3317.0212(E)(1)(c), as applied to ALI 200502"
            }
        }
    }

    /// What the bound is, and which side of it the census counts.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::StaffingFloor(Minimum::SpecialTeachers) => {
                "six special teachers, funded at the floor"
            }
            Self::StaffingFloor(Minimum::Counselors) => {
                "one guidance counselor, funded at the floor"
            }
            Self::StaffingFloor(Minimum::Wellness) => {
                "five student wellness and success staff, funded at the floor"
            }
            Self::StaffingFloor(Minimum::OtherAdministrators) => {
                "two other district administrators, funded at the floor"
            }
            Self::StaffingFloor(Minimum::FiscalSupport) => {
                "two fiscal support staff, funded at the floor"
            }
            Self::StaffingFloor(Minimum::Emis) => "one EMIS support employee, funded at the floor",
            Self::StaffingFloor(Minimum::LeadershipSupport) => {
                "one district leadership support staff member, funded at the floor"
            }
            Self::StaffingFloor(Minimum::BuildingSupport) => {
                "one building leadership support staff per open building, funded at the floor"
            }
            Self::StaffingCeiling(Ceiling::FiscalSupport) => {
                "thirty-five fiscal support staff, held at the ceiling"
            }
            Self::StaffingCeiling(Ceiling::BuildingSupport) => {
                "three building leadership support staff per building, held at the ceiling"
            }
            Self::SizeBandSmall => "size-banded salaries at the small end, below 500 ADM",
            Self::SizeBandLarge => "size-banded salaries at the large end, above 4,000 ADM",
            Self::BaseCostEnrolledAdm => {
                "base cost enrolled ADM, the single year governing over the three-year average"
            }
            Self::ValuationLesserOf => {
                "capacity valuation, the most recent year governing over the three-year average"
            }
            Self::IncomeLesserOf => {
                "capacity income, the most recent year governing over the three-year average"
            }
            Self::CapacityRateCeiling => {
                "local capacity percentage, capped at 0.025 from the fortieth-ranked district up"
            }
            Self::MinimumStateShare => "minimum state share of base cost, paid at 10%",
            Self::DpiaCountCap => "DPIA blended count, capped at enrolled ADM",
            Self::GiftedCoordinatorFloor => "gifted coordinator units, funded at the floor of 0.5",
            Self::GiftedCoordinatorCeiling => "gifted coordinator units, held at the ceiling of 8",
            Self::GiftedSpecialistK8Floor => {
                "K-8 gifted intervention specialist units, funded at the floor of 0.3"
            }
            Self::GiftedSpecialist912Floor => {
                "9-12 gifted intervention specialist units, funded at the floor of 0.3"
            }
            Self::CapacityTierZero => {
                "targeted assistance capacity tier, zero at or above the median district's wealth"
            }
            Self::CapacityTierSizeCutoff => "targeted assistance capacity tier, zero under 200 ADM",
            Self::CapacityTierSizeRamp => {
                "targeted assistance capacity tier, paid a fraction between 200 and 600 ADM"
            }
            Self::WealthTierZero => "targeted assistance wealth tier, zero below an index of 0.8",
            Self::FundingBaseClampAtZero => "the guarantee's floor, clamped at zero",
            Self::Guarantee => "the guarantee, holding the district above the formula",
            Self::DecreaseThresholdFloor => {
                "the clawback's decrease threshold, the floor of twenty governing over ten per cent"
            }
            Self::ClawbackClampAtZero => {
                "the clawback, clamped by a guarantee smaller than the charge"
            }
            Self::TransitionSupplement => "the formula transition supplement, holding the district",
            Self::TransportationFloor => "transportation state share, paid at the 50% floor",
            Self::MileBase => {
                "transportation base, the mile amount governing over the rider amount"
            }
            Self::EfficiencyZero => "efficiency adjustment, zero below an index of 1.0",
            Self::EfficiencyCeiling => "efficiency adjustment, held at 15% from an index of 1.5",
            Self::DensityZero => "density supplement, zero at 28 riders per square mile and above",
            Self::TransportationGuarantee => "transportation's own guarantee, holding the district",
            Self::SpecialEducationTransportFloor => {
                "special education transportation state share, paid at the 50% floor"
            }
        }
    }

    /// The same bound named short enough to sit in a chart's gutter.
    ///
    /// # Why there are two names and not one
    ///
    /// [`Self::label`] is written to be read in a sentence — it says which *side* of the
    /// comparison the census counts, because a row read the other way is a row read backwards.
    /// That makes it up to seventy characters, and thirty-eight of those down the left of one
    /// chart is a name gutter wider than the chart. This is the same bound named for a reader who
    /// already has the axis above it telling them what the count is of.
    ///
    /// Held to forty characters and to uniqueness by
    /// `the_bounds_that_cannot_bind_and_the_ones_that_never_have.rs`, because a short name that
    /// collides with another short name is two rows of a chart a reader cannot tell apart — and
    /// the two gifted unit floors, the two staffing support ceilings and the two 50% transport
    /// floors are all near enough to collide if nobody is checking.
    #[must_use]
    pub const fn short(self) -> &'static str {
        match self {
            Self::StaffingFloor(Minimum::SpecialTeachers) => "Six special teachers",
            Self::StaffingFloor(Minimum::Counselors) => "One guidance counselor",
            Self::StaffingFloor(Minimum::Wellness) => "Five wellness and success staff",
            Self::StaffingFloor(Minimum::OtherAdministrators) => {
                "Two other district administrators"
            }
            Self::StaffingFloor(Minimum::FiscalSupport) => "Two fiscal support staff",
            Self::StaffingFloor(Minimum::Emis) => "One EMIS support employee",
            Self::StaffingFloor(Minimum::LeadershipSupport) => "One leadership support staff",
            Self::StaffingFloor(Minimum::BuildingSupport) => {
                "One building support staff per building"
            }
            Self::StaffingCeiling(Ceiling::FiscalSupport) => "Thirty-five fiscal support staff",
            Self::StaffingCeiling(Ceiling::BuildingSupport) => "Three building support staff each",
            Self::SizeBandSmall => "Size-banded salaries, under 500 ADM",
            Self::SizeBandLarge => "Size-banded salaries, over 4,000 ADM",
            Self::BaseCostEnrolledAdm => "Enrolled ADM, the single year",
            Self::ValuationLesserOf => "Capacity valuation, the recent year",
            Self::IncomeLesserOf => "Capacity income, the recent year",
            Self::CapacityRateCeiling => "Capacity percentage capped at 0.025",
            Self::MinimumStateShare => "Minimum state share of base cost",
            Self::DpiaCountCap => "DPIA count capped at enrolled ADM",
            Self::GiftedCoordinatorFloor => "Gifted coordinator units, floor",
            Self::GiftedCoordinatorCeiling => "Gifted coordinator units, ceiling",
            Self::GiftedSpecialistK8Floor => "K-8 gifted specialist units, floor",
            Self::GiftedSpecialist912Floor => "9-12 gifted specialist units, floor",
            Self::CapacityTierZero => "Capacity tier zero at median wealth",
            Self::CapacityTierSizeCutoff => "Capacity tier zero under 200 ADM",
            Self::CapacityTierSizeRamp => "Capacity tier ramped to 600 ADM",
            Self::WealthTierZero => "Wealth tier zero under an index of 0.8",
            Self::FundingBaseClampAtZero => "Guarantee floor clamped at zero",
            Self::Guarantee => "The guarantee itself",
            Self::DecreaseThresholdFloor => "Decrease threshold, floor of twenty",
            Self::ClawbackClampAtZero => "Clawback clamped by the guarantee",
            Self::TransitionSupplement => "Formula transition supplement",
            Self::TransportationFloor => "Transportation share at the 50% floor",
            Self::MileBase => "Mile base over rider base",
            Self::EfficiencyZero => "Efficiency zero under an index of 1.0",
            Self::EfficiencyCeiling => "Efficiency held at 15%",
            Self::DensityZero => "Density zero at 28 riders a square mile",
            Self::TransportationGuarantee => "Transportation's own guarantee",
            Self::SpecialEducationTransportFloor => "Special education transport, 50% floor",
        }
    }

    /// Whether any input at all can put this bound in force — as the section stands.
    ///
    /// Asserted for one and stated for the rest. R.C. 3317.011(F)(6)(c) is unreachable because
    /// (F)(3)(c) floors its input; strike that floor out and it binds for every small district,
    /// which is why the claim is always relative to the section as written. See
    /// [`Minimum::can_bind`]. R.C. 3317.017(A)(4)(d)(i) is stated against the fortieth-ranked
    /// district, so the districts at or above that rank are on it by definition — forty, ties
    /// aside, whatever their incomes.
    #[must_use]
    pub const fn reach(self) -> Reach {
        match self {
            Self::StaffingFloor(m) => {
                if m.can_bind() {
                    Reach::Reachable
                } else {
                    Reach::Unreachable
                }
            }
            Self::CapacityRateCeiling => Reach::Fixed(local_capacity::BENCHMARK_RANK),
            _ => Reach::Reachable,
        }
    }

    /// Whether the bound is defined for this district at all.
    ///
    /// Most are defined for every district. The transportation share bounds are defined only
    /// where the department paid transportation through a share — a published share and a
    /// positive component sum — and the special education transportation floor only where a
    /// cost was reported. The capacity rate ceiling needs the three income figures. The clawback
    /// clamp is defined only for the districts charged a clawback, since a clamp on a subtraction
    /// nobody made is not a comparison.
    #[must_use]
    pub fn in_population(self, record: &DistrictRecord) -> bool {
        match self {
            Self::CapacityRateCeiling => {
                record.median_income.is_some()
                    && record.statewide_median_income.is_some()
                    && record.benchmark_ratio.is_some()
            }
            Self::ClawbackClampAtZero => record.transition.open_enrollment_adjustment > 0.0,
            Self::TransportationFloor
            | Self::MileBase
            | Self::EfficiencyZero
            | Self::EfficiencyCeiling
            | Self::DensityZero => {
                record.published_state_share.is_some_and(|s| s > 0.0)
                    && record.transportation.components() > 0.0
            }
            Self::SpecialEducationTransportFloor => record.transportation.reported_sped_cost > 0.0,
            _ => true,
        }
    }

    /// Whether the bound is the operative term for this district — the side of the comparison
    /// [`Self::label`] names.
    ///
    /// Meaningful only where [`Self::in_population`] holds; false otherwise.
    #[must_use]
    pub fn binds(self, record: &DistrictRecord) -> bool {
        if !self.in_population(record) {
            return false;
        }
        let adm = record.base_cost_adm();
        let three_year_mean = |values: [f64; 3]| values.iter().sum::<f64>() / 3.0;
        match self {
            Self::StaffingFloor(m) => m.binds(&record.enrollment),
            Self::StaffingCeiling(c) => c.binds(&record.enrollment),
            Self::SizeBandSmall => adm < ratios::SIZE_BAND_LOWER,
            Self::SizeBandLarge => adm > ratios::SIZE_BAND_UPPER,
            Self::BaseCostEnrolledAdm => {
                record.current_year_adm > three_year_mean(record.adm_history) + 1e-9
            }
            Self::ValuationLesserOf => {
                record.valuation_three_year[0] < three_year_mean(record.valuation_three_year)
            }
            Self::IncomeLesserOf => {
                record.agi_three_year[0] < three_year_mean(record.agi_three_year)
            }
            Self::CapacityRateCeiling => {
                let (Some(median), Some(statewide), Some(benchmark)) = (
                    record.median_income,
                    record.statewide_median_income,
                    record.benchmark_ratio,
                ) else {
                    return false;
                };
                statewide > 0.0 && median / statewide >= benchmark
            }
            Self::MinimumStateShare => record.at_minimum_state_share(),
            Self::DpiaCountCap => {
                record.dpia.count_at(DPIA_BLEND.1) > record.categorical_enrolled_adm + 1e-6
            }
            Self::GiftedCoordinatorFloor => {
                record.categorical_enrolled_adm / GIFTED_COORDINATOR_DIVISOR
                    < GIFTED_COORDINATOR_UNIT_BOUNDS.0
            }
            Self::GiftedCoordinatorCeiling => {
                record.categorical_enrolled_adm / GIFTED_COORDINATOR_DIVISOR
                    > GIFTED_COORDINATOR_UNIT_BOUNDS.1
            }
            Self::GiftedSpecialistK8Floor => {
                record.gifted.fte_k8 / GIFTED_SPECIALIST_DIVISOR < GIFTED_SPECIALIST_UNIT_FLOOR
            }
            Self::GiftedSpecialist912Floor => {
                record.gifted.fte_9_12 / GIFTED_SPECIALIST_DIVISOR < GIFTED_SPECIALIST_UNIT_FLOOR
            }
            Self::CapacityTierZero => record.targeted_assistance.capacity_index < 1.0,
            Self::CapacityTierSizeCutoff => record.current_year_adm < TA_CAPACITY_MINIMUM_ADM,
            Self::CapacityTierSizeRamp => {
                record.current_year_adm >= TA_CAPACITY_MINIMUM_ADM
                    && record.current_year_adm < TA_CAPACITY_FULL_AT
                    && TargetedAssistance::capacity_size_share(record.current_year_adm) < 1.0
            }
            Self::WealthTierZero => record.targeted_assistance.wealth_index < TA_WEALTH_INDEX_FLOOR,
            Self::FundingBaseClampAtZero => {
                record.transition.funding_base - record.transition.open_enrollment_adjustment < 0.0
            }
            Self::Guarantee => record.on_guarantee(),
            Self::DecreaseThresholdFloor => {
                record.transition.open_enrollment_prior * OPEN_ENROLLMENT_THRESHOLD_FRACTION
                    < OPEN_ENROLLMENT_THRESHOLD_FLOOR
            }
            Self::ClawbackClampAtZero => {
                record.guarantee_before_clawback() < record.transition.open_enrollment_adjustment
            }
            Self::TransitionSupplement => record.transition.transition_supplement > 0.0,
            Self::TransportationFloor => {
                record.published_state_share.unwrap_or(0.0) < TRANSPORTATION_FLOOR
            }
            Self::MileBase => record.transportation.paid_on_miles(),
            Self::EfficiencyZero => {
                record.transportation.efficiency_index < TRANSPORT_EFFICIENCY_BAND.0
            }
            Self::EfficiencyCeiling => {
                record.transportation.efficiency_index >= TRANSPORT_EFFICIENCY_BAND.1
            }
            Self::DensityZero => record.transportation.district_density >= TRANSPORT_DENSITY_PIVOT,
            Self::TransportationGuarantee => record.transportation.guarantee > 0.0,
            Self::SpecialEducationTransportFloor => {
                record.published_state_share.unwrap_or(0.0) < TRANSPORTATION_FLOOR
            }
        }
    }
}

/// One row of the census: a bound, the districts it is defined for, and the ones on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    /// The bound.
    pub bound: Bound,
    /// Districts the bound is defined for — see [`Bound::in_population`].
    pub population: usize,
    /// Districts for which it is the operative term.
    pub operative: usize,
}

impl Row {
    /// Reachable in principle, defined for somebody, and reached by nobody.
    #[must_use]
    pub const fn never_bound(&self) -> bool {
        matches!(self.bound.reach(), Reach::Reachable) && self.population > 0 && self.operative == 0
    }
}

/// Every bound, counted over a panel.
#[must_use]
pub fn census(panel: &[DistrictRecord]) -> Vec<Row> {
    Bound::all()
        .into_iter()
        .map(|bound| Row {
            bound,
            population: panel.iter().filter(|r| bound.in_population(r)).count(),
            operative: panel.iter().filter(|r| bound.binds(r)).count(),
        })
        .collect()
}

/// The bounds no input can put in force — the drafting facts.
#[must_use]
pub fn cannot_bind() -> Vec<Bound> {
    Bound::all()
        .into_iter()
        .filter(|b| b.reach() == Reach::Unreachable)
        .collect()
}

/// The bounds that could bind and are reached by no district in the panel — the policy facts.
#[must_use]
pub fn never_bound(panel: &[DistrictRecord]) -> Vec<Bound> {
    census(panel)
        .into_iter()
        .filter(Row::never_bound)
        .map(|row| row.bound)
        .collect()
}

/// The bounds reached by exactly one district, with the district's name.
#[must_use]
pub fn reached_by_one(panel: &[DistrictRecord]) -> Vec<(Bound, String)> {
    Bound::all()
        .into_iter()
        .filter_map(|bound| {
            let mut on: Vec<&DistrictRecord> = panel.iter().filter(|r| bound.binds(r)).collect();
            (on.len() == 1).then(|| (bound, on.remove(0).name.clone()))
        })
        .collect()
}

/// Districts the decrease threshold's floor of twenty kept off the clawback entirely.
///
/// A district whose open-enrolment loss exceeds ten per cent of last year's count but not
/// twenty pupils would be charged under the percentage alone and is charged nothing under the
/// floor. This is the *moved* count for [`Bound::DecreaseThresholdFloor`], against the 499 the
/// floor merely governs — the same distinction
/// `crates/scenario-delta/tests/the_minimum_share_the_guarantee_has_already_paid_past.rs` draws
/// for the minimum state share.
#[must_use]
pub fn spared_by_the_decrease_threshold_floor(panel: &[DistrictRecord]) -> usize {
    panel
        .iter()
        .filter(|r| {
            let lost = r.transition.open_enrollment_lost();
            lost > r.transition.open_enrollment_prior * OPEN_ENROLLMENT_THRESHOLD_FRACTION
                && lost <= OPEN_ENROLLMENT_THRESHOLD_FLOOR
        })
        .count()
}

/// Whether every district at or above the fortieth-ranked income ratio, and no other, is on
/// [`Bound::CapacityRateCeiling`] — the check that [`Reach::Fixed`] is a property of the rank and
/// not a coincidence of the year.
#[must_use]
pub fn capacity_ceiling_is_the_rank(panel: &[DistrictRecord]) -> bool {
    let mut ratios: Vec<f64> = panel
        .iter()
        .filter_map(|r| Some(r.median_income? / r.statewide_median_income?))
        .collect();
    ratios.sort_by(|a, b| b.total_cmp(a));
    let Some(benchmark) = ratios.get(local_capacity::BENCHMARK_RANK - 1) else {
        return false;
    };
    let on = panel
        .iter()
        .filter(|r| Bound::CapacityRateCeiling.binds(r))
        .count();
    on == ratios.iter().filter(|x| *x >= benchmark).count()
}

/// The three lesser-of and greater-of rules against a three-year average, as a triple: the
/// districts on the single-year branch of base cost enrolled ADM, of capacity valuation, and of
/// capacity income.
#[must_use]
pub fn single_year_branches(panel: &[DistrictRecord]) -> (usize, usize, usize) {
    let count = |bound: Bound| panel.iter().filter(|r| bound.binds(r)).count();
    (
        count(Bound::BaseCostEnrolledAdm),
        count(Bound::ValuationLesserOf),
        count(Bound::IncomeLesserOf),
    )
}

/// Whether the plan's one two-branch step inside R.C. 3317.011 — athletics eligibility at
/// (A)(11) — is dormant on this panel.
#[must_use]
pub fn athletics_step_is_dormant(panel: &[DistrictRecord]) -> bool {
    panel.iter().all(|r| r.enrollment.athletics_eligible)
}
