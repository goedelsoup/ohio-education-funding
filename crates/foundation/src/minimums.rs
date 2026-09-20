//! The floors R.C. 3317.011 puts under a district's funded staffing, and what they are worth.
//!
//! Seven divisions of the base cost build-up compute a funded position count as `ADM / ratio`
//! and then take **the greater of that quotient and a fixed number**. A district small enough
//! that the quotient falls below the number is funded as though it were larger. This module
//! names each one, says where it binds, and prices the difference — the [`uplift`] — against the
//! bare ratio the same division would otherwise pay.
//!
//! # Every one of them is a `max`, and a `max` is continuous
//!
//! This is the whole of why the word *cliff* does not belong here, and it is visible in the
//! statute rather than inferred from the data. The funded count is `max(ADM / ratio, n)`, which
//! agrees with the bare ratio at the threshold and everywhere above it. Nothing steps. What
//! changes at `ADM = ratio × n` is the **slope**: below the threshold the funded count is flat
//! and the per-pupil uplift is a hyperbola `n × cost / ADM − cost / ratio`, falling to exactly
//! zero at the threshold; above it the uplift is zero and stays zero.
//!
//! So a district does not fall off anything by growing past a threshold. It stops gaining
//! protection it was already down to nothing of.
//!
//! The statute writes the same operation three ways and this module treats them as one.
//! Divisions (E)(1)(b), (E)(3)(b), (F)(3)(c), (F)(5)(b) and (F)(6)(c) say "the greater of the
//! quotient and *n*"; (D)(2)(b) and (c) write two branches on `> 6` and `<= 6` that agree at 6;
//! (F)(4)(b) writes "the lesser of (the maximum of the quotient and 2) and 35", which is a
//! [`Ceiling`] on top of a [`Minimum`]. The grammars differ and the arithmetic does not.
//!
//! # One of them cannot bind
//!
//! [`Minimum::LeadershipSupport`] is `max((max(ADM / 750, 2) + 1) / 3, 1)`. The inner `max` is
//! never below 2, so the quotient is never below 1, so the outer `max` is never the operative
//! term — **for any district, at any enrolment, including zero**. R.C. 3317.011(F)(6)(c) states
//! a floor that the division above it has already made unreachable. See
//! [`Minimum::can_bind`], which is asserted rather than described.
//!
//! The leadership support cost of a small district is nonetheless raised, by
//! [`Minimum::OtherAdministrators`] flowing through the same division's own input. That uplift
//! is attributed to the administrator minimum, which is what causes it — see
//! [`Uplift::induced`].
//!
//! # What has no floor at all
//!
//! Five staffed elements are funded at the bare ratio with nothing under them: librarians and
//! media staff (E)(2), building leaders (G)(1), and the three classroom-teacher bands. The two
//! that a very small district can least plausibly staff in fractions are in that list. Ohio's
//! smallest district is funded for **six** special teachers and for **one one-hundredth** of a
//! principal, out of the same section.
//!
//! This module does not price that, because there is no counterfactual to price it against: an
//! element with no floor has no second value to subtract. It is named here because the census
//! below is otherwise read as the set of places the statute worried about small districts, and
//! the omissions are as much a design as the inclusions.

use edfund_core::{round_dp, Adm, Dollars};

use crate::{ratios, DistrictEnrollment, StatewideFactors};

/// A floor on a funded position count, per R.C. 3317.011.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Minimum {
    /// Six special teachers, (D)(2).
    SpecialTeachers,
    /// One guidance counselor, (E)(1).
    Counselors,
    /// Five student wellness and success staff, (E)(3).
    Wellness,
    /// Two district administrators other than the superintendent, (F)(3).
    OtherAdministrators,
    /// Two fiscal support staff, (F)(4).
    FiscalSupport,
    /// One EMIS support employee, (F)(5).
    Emis,
    /// One district leadership support staff member, (F)(6) — the one that cannot bind.
    LeadershipSupport,
    /// One clerical staff member per open building, (G)(2)(c)(i).
    ///
    /// The odd one out: its floor is a count of the district's own buildings rather than a fixed
    /// number, so it has no ADM threshold and it can bind at any size.
    BuildingSupport,
}

/// A ceiling on a funded position count. Two of them, and they take money away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Ceiling {
    /// Thirty-five fiscal support staff however large the district, (F)(4)(b)(ii).
    FiscalSupport,
    /// Three building leadership support staff per open building, (G)(2)(c)(ii).
    BuildingSupport,
}

impl Minimum {
    /// Every floor in the section, in statutory order.
    pub const ALL: [Self; 8] = [
        Self::SpecialTeachers,
        Self::Counselors,
        Self::Wellness,
        Self::OtherAdministrators,
        Self::FiscalSupport,
        Self::Emis,
        Self::LeadershipSupport,
        Self::BuildingSupport,
    ];

    /// The element the floor is under.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SpecialTeachers => "special teachers",
            Self::Counselors => "guidance counselors",
            Self::Wellness => "student wellness and success staff",
            Self::OtherAdministrators => "other district administrators",
            Self::FiscalSupport => "fiscal support staff",
            Self::Emis => "EMIS support employees",
            Self::LeadershipSupport => "district leadership support staff",
            Self::BuildingSupport => "building leadership support staff",
        }
    }

    /// The division of R.C. 3317.011 that states it.
    #[must_use]
    pub const fn division(self) -> &'static str {
        match self {
            Self::SpecialTeachers => "(D)(2)",
            Self::Counselors => "(E)(1)",
            Self::Wellness => "(E)(3)",
            Self::OtherAdministrators => "(F)(3)",
            Self::FiscalSupport => "(F)(4)",
            Self::Emis => "(F)(5)",
            Self::LeadershipSupport => "(F)(6)",
            Self::BuildingSupport => "(G)(2)",
        }
    }

    /// The positions the floor funds, where it is a fixed number.
    ///
    /// `None` for [`Self::BuildingSupport`], whose floor is the district's own building count.
    #[must_use]
    pub const fn positions(self) -> Option<f64> {
        match self {
            Self::SpecialTeachers => Some(ratios::SPECIAL_TEACHER_MINIMUM),
            Self::Counselors => Some(ratios::COUNSELOR_MINIMUM),
            Self::Wellness => Some(ratios::WELLNESS_MINIMUM),
            Self::OtherAdministrators => Some(ratios::OTHER_ADMINISTRATOR_MINIMUM),
            Self::FiscalSupport => Some(ratios::FISCAL_SUPPORT_MINIMUM),
            Self::Emis => Some(ratios::EMIS_MINIMUM),
            Self::LeadershipSupport => Some(ratios::LEADERSHIP_SUPPORT_MINIMUM),
            Self::BuildingSupport => None,
        }
    }

    /// The base cost enrolled ADM below which the floor binds.
    ///
    /// `None` where the count the ratio divides is not base cost enrolled ADM:
    /// [`Self::Counselors`] divides grades 9-12, [`Self::BuildingSupport`] compares against a
    /// building count, and [`Self::LeadershipSupport`] divides a funded position count that is
    /// itself floored — which is why it has no threshold rather than a very low one.
    #[must_use]
    pub fn threshold(self) -> Option<Adm> {
        match self {
            Self::SpecialTeachers => {
                Some(ratios::SPECIAL_TEACHER * ratios::SPECIAL_TEACHER_MINIMUM)
            }
            Self::Wellness => Some(ratios::WELLNESS * ratios::WELLNESS_MINIMUM),
            Self::OtherAdministrators => {
                Some(ratios::OTHER_ADMINISTRATOR * ratios::OTHER_ADMINISTRATOR_MINIMUM)
            }
            Self::FiscalSupport => Some(ratios::FISCAL_SUPPORT * ratios::FISCAL_SUPPORT_MINIMUM),
            Self::Emis => Some(ratios::EMIS * ratios::EMIS_MINIMUM),
            Self::Counselors | Self::LeadershipSupport | Self::BuildingSupport => None,
        }
    }

    /// Whether any enrolment at all puts this floor in force.
    ///
    /// False for exactly one: see the module note on R.C. 3317.011(F)(6)(c).
    #[must_use]
    pub const fn can_bind(self) -> bool {
        !matches!(self, Self::LeadershipSupport)
    }

    /// The funded position count the division computes before its floor.
    #[must_use]
    pub fn unfloored(self, enrollment: &DistrictEnrollment) -> f64 {
        let adm = enrollment.base_cost_enrolled_adm;
        match self {
            Self::SpecialTeachers => adm / ratios::SPECIAL_TEACHER,
            Self::Counselors => enrollment.grades_9_12_total / ratios::COUNSELOR,
            Self::Wellness => adm / ratios::WELLNESS,
            Self::OtherAdministrators => adm / ratios::OTHER_ADMINISTRATOR,
            Self::FiscalSupport => adm / ratios::FISCAL_SUPPORT,
            Self::Emis => adm / ratios::EMIS,
            // Its input is the *floored* administrator count, because that is what (F)(6)(a)
            // reads. Removing this floor alone leaves the administrator floor in place.
            Self::LeadershipSupport => {
                (Self::OtherAdministrators.funded(enrollment) + 1.0) / ratios::LEADERSHIP_SUPPORT
            }
            Self::BuildingSupport => adm / ratios::BUILDING_LEADERSHIP_SUPPORT,
        }
    }

    /// The funded position count the division computes after its floor, before any ceiling.
    #[must_use]
    pub fn funded(self, enrollment: &DistrictEnrollment) -> f64 {
        self.counts(enrollment).0
    }

    /// Whether the floor is what sets this district's funded count.
    ///
    /// Tested the way the division states the comparison, which is not the same test in every
    /// one. Six of them compare the **unrounded** quotient against a fixed number, so a district
    /// whose quotient is 0.999 binds even though both sides round to 1.00 and the uplift is
    /// therefore zero. (G)(2) compares the **rounded** quotient against the building count,
    /// because that is where `aggregate_base_cost` applies the rounding.
    #[must_use]
    pub fn binds(self, enrollment: &DistrictEnrollment) -> bool {
        match self {
            Self::BuildingSupport => {
                round_dp(self.unfloored(enrollment), 2) < enrollment.open_buildings
            }
            _ => {
                self.unfloored(enrollment)
                    < self
                        .positions()
                        .expect("only building support has no fixed floor")
            }
        }
    }

    /// The two counts whose difference this floor is worth: what the division funds, and what it
    /// would fund with the floor struck out.
    ///
    /// Both carry the department's two-decimal rounding **exactly where
    /// [`crate::aggregate_base_cost`] applies it**, which is not uniform. Six divisions round
    /// the funded count after taking the greater. (G)(2) rounds its quotient and then compares,
    /// and when the building count governs it multiplies that count through unrounded — so a
    /// district with a fractional building count is funded on the fraction. Rounding it here
    /// instead put $142.51 of residual into a district at 901 ADM, which is how this comment
    /// came to exist.
    fn counts(self, enrollment: &DistrictEnrollment) -> (f64, f64) {
        match self {
            Self::BuildingSupport => {
                let by_enrollment = round_dp(self.unfloored(enrollment), 2);
                if by_enrollment < enrollment.open_buildings {
                    (enrollment.open_buildings, by_enrollment)
                } else {
                    (by_enrollment, by_enrollment)
                }
            }
            _ => {
                let floor = self
                    .positions()
                    .expect("only building support has no fixed floor");
                let raw = self.unfloored(enrollment);
                (round_dp(raw.max(floor), 2), round_dp(raw, 2))
            }
        }
    }

    /// The cost of one funded position of this element, at the stated factors.
    ///
    /// [`Self::SpecialTeachers`] carries three terms rather than one: a special teacher enters
    /// the substitute and professional development costs at (D)(3)(b) and (D)(4) as well as its
    /// own at (D)(2), so a floored special teacher is worth more than an average teacher cost.
    #[must_use]
    pub fn position_cost(self, factors: &StatewideFactors) -> Dollars {
        match self {
            Self::SpecialTeachers => {
                factors.position_cost(factors.teacher_salary)
                    + factors.substitute_daily_rate
                        * factors.benefit_multiplier
                        * ratios::SUBSTITUTE_DAYS
                    + (factors.teacher_salary * factors.benefit_multiplier) / ratios::CONTRACT_DAYS
                        * ratios::PROFESSIONAL_DEVELOPMENT_DAYS
            }
            // (E)(1)(b) and (E)(3)(b) both price against the counselor salary.
            Self::Counselors | Self::Wellness => factors.position_cost(factors.counselor_salary),
            Self::OtherAdministrators => factors.position_cost(factors.other_administrator_salary),
            Self::FiscalSupport => factors.position_cost(factors.bookkeeping_salary),
            Self::Emis => factors.position_cost(factors.emis_salary),
            Self::LeadershipSupport => {
                factors.position_cost(factors.administrative_assistant_salary)
            }
            Self::BuildingSupport => factors.position_cost(factors.clerical_salary),
        }
    }

    /// The unit cost this district's funded count is actually multiplied by.
    ///
    /// Differs from [`Self::position_cost`] for [`Self::OtherAdministrators`] alone, which
    /// (F)(3)(c) prices as a *ratio* against the district's own banded superintendent cost
    /// rather than from the administrator salary directly. A small district's superintendent is
    /// priced at the bottom of the band, so its floored administrators are cheaper than the
    /// statewide average administrator.
    #[must_use]
    pub fn unit_cost(self, enrollment: &DistrictEnrollment, factors: &StatewideFactors) -> Dollars {
        match self {
            Self::OtherAdministrators => {
                let superintendent = factors.banded_position_cost(
                    enrollment.base_cost_enrolled_adm,
                    factors.superintendent_salary_small,
                    factors.superintendent_salary_large,
                );
                (superintendent - factors.insurance)
                    * (factors.other_administrator_salary / factors.superintendent_salary)
                    + factors.insurance
            }
            other => other.position_cost(factors),
        }
    }

    /// What this floor adds to the district's aggregate base cost.
    ///
    /// The funded count is rounded to two decimals on both sides before multiplying, because
    /// that is what the department does and what [`crate::aggregate_base_cost`] reproduces.
    #[must_use]
    pub fn uplift(self, enrollment: &DistrictEnrollment, factors: &StatewideFactors) -> Dollars {
        let (with, without) = self.counts(enrollment);
        (with - without) * self.unit_cost(enrollment, factors)
    }
}

impl Ceiling {
    /// Both ceilings, in statutory order.
    pub const ALL: [Self; 2] = [Self::FiscalSupport, Self::BuildingSupport];

    /// The element the ceiling is over.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::FiscalSupport => "fiscal support staff",
            Self::BuildingSupport => "building leadership support staff",
        }
    }

    /// Whether the ceiling is what sets this district's funded count.
    #[must_use]
    pub fn binds(self, enrollment: &DistrictEnrollment) -> bool {
        match self {
            Self::FiscalSupport => {
                enrollment.base_cost_enrolled_adm / ratios::FISCAL_SUPPORT
                    > ratios::FISCAL_SUPPORT_MAXIMUM
            }
            Self::BuildingSupport => {
                let by_enrollment =
                    enrollment.base_cost_enrolled_adm / ratios::BUILDING_LEADERSHIP_SUPPORT;
                by_enrollment >= enrollment.open_buildings
                    && by_enrollment
                        > enrollment.open_buildings * ratios::BUILDING_SUPPORT_PER_BUILDING
            }
        }
    }

    /// What this ceiling takes off the district's aggregate base cost, as a positive number.
    #[must_use]
    pub fn reduction(self, enrollment: &DistrictEnrollment, factors: &StatewideFactors) -> Dollars {
        if !self.binds(enrollment) {
            return 0.0;
        }
        match self {
            Self::FiscalSupport => {
                let raw = round_dp(
                    enrollment.base_cost_enrolled_adm / ratios::FISCAL_SUPPORT,
                    2,
                );
                (raw - ratios::FISCAL_SUPPORT_MAXIMUM)
                    * Minimum::FiscalSupport.position_cost(factors)
            }
            Self::BuildingSupport => {
                let raw = round_dp(
                    enrollment.base_cost_enrolled_adm / ratios::BUILDING_LEADERSHIP_SUPPORT,
                    2,
                );
                (raw - enrollment.open_buildings * ratios::BUILDING_SUPPORT_PER_BUILDING)
                    * Minimum::BuildingSupport.position_cost(factors)
            }
        }
    }
}

/// One district's uplift, split into the floors that cause it directly and the one knock-on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Uplift {
    /// What the seven floors that can bind add, summed.
    pub direct: Dollars,
    /// What [`Minimum::OtherAdministrators`] adds a second time by raising the input
    /// R.C. 3317.011(F)(6)(a) reads.
    ///
    /// Kept separate because it is the administrator floor's doing and not the leadership
    /// support floor's — that one cannot bind. Adding it to what
    /// [`Minimum::uplift`] returns for [`Minimum::LeadershipSupport`] would attribute it to a
    /// provision with no force.
    pub induced: Dollars,
    /// What the two ceilings take away, as a negative number.
    pub ceilings: Dollars,
}

impl Uplift {
    /// The whole of what R.C. 3317.011's floors and ceilings do to this district's base cost.
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.direct + self.induced + self.ceilings
    }
}

/// Price every floor and ceiling for one district.
#[must_use]
pub fn uplift(enrollment: &DistrictEnrollment, factors: &StatewideFactors) -> Uplift {
    let direct = Minimum::ALL
        .iter()
        .map(|m| m.uplift(enrollment, factors))
        .sum();

    // The administrator floor's second effect. (F)(6)(a) reads the *floored* administrator
    // count, so removing that floor lowers leadership support too; the difference between the
    // two leadership support counts is what the administrator floor is worth there.
    //
    // No `max` on either side: this term is the administrator floor's movement alone, and
    // `Minimum::LeadershipSupport::uplift` carries whatever the leadership floor itself does —
    // which is nothing. The two partition the division's total movement exactly.
    let with = round_dp(
        (Minimum::OtherAdministrators.funded(enrollment) + 1.0) / ratios::LEADERSHIP_SUPPORT,
        2,
    );
    let without = round_dp(
        (Minimum::OtherAdministrators.unfloored(enrollment) + 1.0) / ratios::LEADERSHIP_SUPPORT,
        2,
    );
    let induced = (with - without) * Minimum::LeadershipSupport.position_cost(factors);

    Uplift {
        direct,
        induced,
        ceilings: Ceiling::ALL
            .iter()
            .map(|c| -c.reduction(enrollment, factors))
            .sum(),
    }
}

impl DistrictEnrollment {
    /// The same district at a different enrolment, grade composition held.
    ///
    /// Every grade band and the high-school total scale by the same factor, so the classroom
    /// teacher ratios see the mix they saw before. The building count does **not** scale: a
    /// district that loses a fifth of its pupils does not close a fifth of a building in the
    /// year it loses them, and holding it is what makes the building support floor visible as
    /// enrolment falls.
    ///
    /// Returns the district unchanged if its enrolment is not positive, since there is no
    /// composition to hold.
    #[must_use]
    pub fn scaled_to(&self, adm: Adm) -> Self {
        if self.base_cost_enrolled_adm <= 0.0 {
            return *self;
        }
        let k = adm / self.base_cost_enrolled_adm;
        Self {
            kindergarten: self.kindergarten * k,
            grades_1_3: self.grades_1_3 * k,
            grades_4_8: self.grades_4_8 * k,
            grades_9_12: self.grades_9_12 * k,
            career_technical: self.career_technical * k,
            grades_9_12_total: self.grades_9_12_total * k,
            base_cost_enrolled_adm: adm,
            open_buildings: self.open_buildings,
            athletics_eligible: self.athletics_eligible,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn district(adm: Adm) -> DistrictEnrollment {
        DistrictEnrollment {
            kindergarten: adm * 0.07,
            grades_1_3: adm * 0.22,
            grades_4_8: adm * 0.38,
            grades_9_12: adm * 0.30,
            career_technical: adm * 0.03,
            grades_9_12_total: adm * 0.33,
            base_cost_enrolled_adm: adm,
            open_buildings: (adm / 400.0).max(1.0),
            athletics_eligible: true,
        }
    }

    /// **The dead letter.** R.C. 3317.011(F)(6)(c) states a floor its own input cannot reach.
    #[test]
    fn the_leadership_support_minimum_cannot_bind_at_any_enrolment() {
        for adm in [0.0, 1.0, 5.2, 100.0, 1_499.0, 1_500.0, 1_501.0, 43_628.6] {
            let e = district(adm);
            assert!(
                !Minimum::LeadershipSupport.binds(&e),
                "the leadership support floor bound at {adm} ADM"
            );
            assert!(
                Minimum::LeadershipSupport.uplift(&e, &StatewideFactors::fy2027()) == 0.0,
                "and it was worth something at {adm} ADM"
            );
        }
        assert!(!Minimum::LeadershipSupport.can_bind());
        assert!(Minimum::ALL.iter().filter(|m| !m.can_bind()).count() == 1);
    }

    /// The uplift accounts for the whole difference the floors make, term by term.
    ///
    /// Not a restatement of [`uplift`]: this rebuilds the district's aggregate base cost from
    /// the published build-up with every floor removed, and asserts the difference is what the
    /// module says it is. If a floor is ever added to a division this module does not know
    /// about, the residual stops being zero.
    #[test]
    fn the_uplift_is_the_whole_difference_the_floors_make() {
        let factors = StatewideFactors::fy2027();
        for adm in [
            120.0, 400.0, 899.0, 901.0, 1_249.0, 1_700.0, 4_000.0, 30_000.0,
        ] {
            let e = district(adm);
            let with = crate::aggregate_base_cost(&e, &factors).aggregate;
            let without = unfloored_aggregate(&e, &factors);
            let priced = uplift(&e, &factors).total();
            assert!(
                (with - without - priced).abs() < 0.01,
                "at {adm} ADM the floors move ${:.2} and are priced at ${priced:.2}",
                with - without
            );
        }
    }

    /// The build-up with every floor and ceiling taken out, for the test above.
    fn unfloored_aggregate(e: &DistrictEnrollment, f: &StatewideFactors) -> Dollars {
        let adm = e.base_cost_enrolled_adm;
        let teachers = round_dp(
            e.kindergarten / ratios::KINDERGARTEN
                + e.grades_1_3 / ratios::GRADES_1_3
                + e.grades_4_8 / ratios::GRADES_4_8
                + e.grades_9_12 / ratios::GRADES_9_12
                + e.career_technical / ratios::CAREER_TECHNICAL,
            2,
        );
        let special = round_dp(adm / ratios::SPECIAL_TEACHER, 2);
        let positions = teachers + special;
        let teacher = (teachers + special) * f.position_cost(f.teacher_salary)
            + positions * f.substitute_daily_rate * f.benefit_multiplier * ratios::SUBSTITUTE_DAYS
            + positions * (f.teacher_salary * f.benefit_multiplier) / ratios::CONTRACT_DAYS
                * ratios::PROFESSIONAL_DEVELOPMENT_DAYS;

        let counselor_cost = f.position_cost(f.counselor_salary);
        let support = round_dp(e.grades_9_12_total / ratios::COUNSELOR, 2) * counselor_cost
            + round_dp(adm / ratios::LIBRARIAN, 2) * f.position_cost(f.librarian_salary)
            + round_dp(adm / ratios::WELLNESS, 2) * counselor_cost
            + adm
                * (f.academic_cocurricular_per_pupil
                    + f.safety_per_pupil
                    + f.supplies_per_pupil
                    + f.technology_per_pupil);

        let superintendent = f.banded_position_cost(
            adm,
            f.superintendent_salary_small,
            f.superintendent_salary_large,
        );
        let admins = round_dp(adm / ratios::OTHER_ADMINISTRATOR, 2);
        let leadership = superintendent
            + f.banded_position_cost(adm, f.treasurer_salary_small, f.treasurer_salary_large)
            + admins * Minimum::OtherAdministrators.unit_cost(e, f)
            + round_dp(adm / ratios::FISCAL_SUPPORT, 2) * f.position_cost(f.bookkeeping_salary)
            + round_dp(adm / ratios::EMIS, 2) * f.position_cost(f.emis_salary)
            + round_dp((admins + 1.0) / ratios::LEADERSHIP_SUPPORT, 2)
                * f.position_cost(f.administrative_assistant_salary)
            + adm * f.itc_per_pupil;

        let leader_unit = (superintendent - f.insurance)
            * (f.principal_salary / f.superintendent_salary)
            + f.insurance;
        let building = round_dp(adm / ratios::BUILDING_LEADER, 2) * leader_unit
            + round_dp(adm / ratios::BUILDING_LEADERSHIP_SUPPORT, 2)
                * f.position_cost(f.clerical_salary)
            + adm * (f.building_per_pupil - f.safety_per_pupil);

        let athletics = if e.athletics_eligible {
            adm * f.athletic_cocurricular_per_pupil
        } else {
            0.0
        };
        teacher + support + leadership + building + athletics
    }

    /// **The continuity claim, as a property of the function.**
    ///
    /// At every threshold the floored count equals the bare quotient, so there is nothing to
    /// step. Asserted on the counts rather than on dollars so that the two-decimal rounding of
    /// the funded count cannot be mistaken for the floor.
    #[test]
    fn every_floor_meets_the_bare_ratio_exactly_at_its_threshold() {
        for minimum in Minimum::ALL {
            let Some(threshold) = minimum.threshold() else {
                continue;
            };
            let e = district(threshold);
            assert!(
                (minimum.funded(&e) - minimum.unfloored(&e)).abs() < 1e-9,
                "{} steps at {threshold} ADM",
                minimum.label()
            );
            assert!(
                !minimum.binds(&e),
                "{} binds at its own threshold",
                minimum.label()
            );
            assert!(
                minimum.binds(&district(threshold * 0.999)),
                "{} does not bind just below its threshold",
                minimum.label()
            );
        }
    }

    /// Scaling holds the grade mix and does not close buildings.
    #[test]
    fn scaling_holds_composition_and_the_building_count() {
        let e = district(2_000.0);
        let half = e.scaled_to(1_000.0);
        assert!((half.base_cost_enrolled_adm - 1_000.0).abs() < 1e-9);
        assert!(
            (half.grades_4_8 / half.base_cost_enrolled_adm
                - e.grades_4_8 / e.base_cost_enrolled_adm)
                .abs()
                < 1e-12
        );
        assert!((half.open_buildings - e.open_buildings).abs() < 1e-12);
        assert_eq!(
            DistrictEnrollment {
                base_cost_enrolled_adm: 0.0,
                ..e
            }
            .scaled_to(50.0)
            .base_cost_enrolled_adm,
            0.0
        );
    }
}
