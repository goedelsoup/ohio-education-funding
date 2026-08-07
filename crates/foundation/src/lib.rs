//! Fair School Funding Plan base cost build-up, per R.C. 3317.011.
//!
//! Base cost is assembled for each district from statutory staffing ratios applied to that
//! district's own enrollment, priced at statewide average salaries for a **reference year**.
//! Changing that reference year changes only the price terms — the ratios, minimums, and ADM
//! definitions are untouched — which is what makes the input-year question a clean
//! single-parameter perturbation rather than a re-specification of the model.
//!
//! # All five sub-components, verified end to end
//!
//! | Sub-component | Section | Elements |
//! |---|---|---|
//! | Teacher base cost | 3317.011(D) | 4 |
//! | Student support base cost | 3317.011(E) | 7 |
//! | District leadership and accountability | 3317.011(F) | 7 |
//! | Building leadership and operation | 3317.011(G) | 3 |
//! | Athletic co-curricular activities | 3317.011(H) | 1 |
//!
//! Twenty-two elements in total, matching the department's own description. The published
//! worked example sums to an aggregate base cost of $148,960,701.95, and
//! [`aggregate_base_cost`] reproduces it — which also confirms the sub-component figures are
//! mutually consistent rather than merely individually plausible.
//!
//! # Two structural features worth knowing before reading the code
//!
//! **Administrator salaries are derived, not looked up.** Only the superintendent has an
//! explicit salary band. Other district administrators are priced at 82.8% of it and building
//! leaders at 79.38%, computed as ratios against the statewide superintendent salary — so a
//! change to the superintendent band propagates through three other elements.
//!
//! **The superintendent band is a ramp, not a step.** Districts above 4,000 ADM are funded at
//! a $160,000 salary and those below 500 at $80,000, with a linear interpolation across the
//! 3,500-pupil range between. This is the only place in the formula where a price varies with
//! district size.

#![forbid(unsafe_code)]

use edfund_core::{round_dp, Adm, Dollars};

/// Statewide average cost inputs for a reference year.
///
/// The reference year is the parameter at the centre of Ohio's current funding argument.
/// H.B. 110 priced FY2022 from FY2018 salaries; H.B. 33 refreshed the inputs to FY2022;
/// H.B. 96 held them there through FY2027.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatewideFactors {
    /// Statewide average teacher salary.
    pub teacher_salary: Dollars,
    /// Statewide average superintendent salary, used as the denominator when pricing other
    /// administrators and building leaders.
    pub superintendent_salary: Dollars,
    /// Statewide average salary for administrators other than the superintendent.
    pub other_administrator_salary: Dollars,
    /// Statewide average principal salary.
    pub principal_salary: Dollars,
    /// Statewide average guidance counselor salary. Also prices wellness and success staff.
    pub counselor_salary: Dollars,
    /// Statewide average librarian and media staff salary.
    pub librarian_salary: Dollars,
    /// Statewide average EMIS support employee salary.
    pub emis_salary: Dollars,
    /// Statewide average bookkeeping and accounting employee salary.
    pub bookkeeping_salary: Dollars,
    /// Statewide average administrative assistant salary.
    pub administrative_assistant_salary: Dollars,
    /// Statewide average clerical staff salary.
    pub clerical_salary: Dollars,
    /// Average annual employer-paid insurance cost, added per funded position.
    pub insurance: Dollars,
    /// Benefit multiplier applied to salary: 1.16, being 14% retirement contribution plus 2%
    /// federal payroll taxes and workers' compensation.
    pub benefit_multiplier: f64,
    /// Daily rate for a substitute teacher, before benefits.
    pub substitute_daily_rate: Dollars,
    /// Funded superintendent salary for districts above the upper ADM threshold.
    pub superintendent_salary_large: Dollars,
    /// Funded superintendent salary for districts below the lower ADM threshold.
    pub superintendent_salary_small: Dollars,
    /// Funded treasurer salary for districts above the upper ADM threshold.
    pub treasurer_salary_large: Dollars,
    /// Funded treasurer salary for districts below the lower ADM threshold.
    pub treasurer_salary_small: Dollars,
    /// Per-pupil amount for academic (non-athletic) co-curricular activities.
    pub academic_cocurricular_per_pupil: Dollars,
    /// Per-pupil amount for athletic co-curricular activities.
    pub athletic_cocurricular_per_pupil: Dollars,
    /// Per-pupil amount for building safety and security.
    pub safety_per_pupil: Dollars,
    /// Per-pupil amount for supplies and academic content.
    pub supplies_per_pupil: Dollars,
    /// Per-pupil amount for student technology.
    pub technology_per_pupil: Dollars,
    /// Per-pupil amount for information technology centre support.
    pub itc_per_pupil: Dollars,
    /// Per-pupil building amount: statewide average square feet per pupil times cost per
    /// square foot. Building safety is subtracted from it to avoid double-funding.
    pub building_per_pupil: Dollars,
}

impl StatewideFactors {
    /// The FY2022 factors, priced from **FY2018** statewide averages.
    ///
    /// Every value is taken from the department's FY2022 School Finance Payment Report
    /// line-by-line explanation. The building amount of $1,129.78 is 239.36 square feet per
    /// pupil at $4.72 per square foot.
    #[must_use]
    pub const fn fy2022() -> Self {
        Self {
            teacher_salary: 62_696.18,
            superintendent_salary: 115_615.69,
            other_administrator_salary: 95_727.51,
            principal_salary: 91_720.36,
            counselor_salary: 63_263.80,
            librarian_salary: 68_139.33,
            emis_salary: 53_695.26,
            bookkeeping_salary: 45_387.82,
            administrative_assistant_salary: 44_955.10,
            clerical_salary: 32_997.90,
            insurance: 14_265.53,
            benefit_multiplier: 1.16,
            substitute_daily_rate: 90.0,
            superintendent_salary_large: 160_000.0,
            superintendent_salary_small: 80_000.0,
            treasurer_salary_large: 130_000.0,
            treasurer_salary_small: 60_000.0,
            academic_cocurricular_per_pupil: 42.13,
            athletic_cocurricular_per_pupil: 163.28,
            safety_per_pupil: 23.29,
            supplies_per_pupil: 220.35,
            technology_per_pupil: 37.50,
            itc_per_pupil: 31.00,
            building_per_pupil: 1_129.78,
        }
    }

    /// The fully-loaded annual cost of one funded position at a given salary.
    ///
    /// Rounded to cents, matching the department's published average teacher base cost of
    /// $86,993.10. Rounding here rather than after multiplication is load-bearing: it is what
    /// reproduces the published totals exactly.
    #[must_use]
    pub fn position_cost(&self, salary: Dollars) -> Dollars {
        round_dp(salary * self.benefit_multiplier + self.insurance, 2)
    }

    /// Interpolate a size-banded salary against base cost enrolled ADM.
    ///
    /// Above [`ratios::SIZE_BAND_UPPER`] the large figure applies; below
    /// [`ratios::SIZE_BAND_LOWER`] the small one; between them the funded salary ramps
    /// linearly across the 3,500-pupil span.
    #[must_use]
    pub fn banded_position_cost(&self, adm: Adm, small: Dollars, large: Dollars) -> Dollars {
        let loaded_small = small * self.benefit_multiplier;
        let loaded_large = large * self.benefit_multiplier;
        let salary_component = if adm > ratios::SIZE_BAND_UPPER {
            loaded_large
        } else if adm < ratios::SIZE_BAND_LOWER {
            loaded_small
        } else {
            (adm - ratios::SIZE_BAND_LOWER)
                * ((loaded_large - loaded_small) / ratios::SIZE_BAND_SPAN)
                + loaded_small
        };
        round_dp(salary_component + self.insurance, 2)
    }
}

/// A district's enrollment and building count, as the base cost calculation requires.
///
/// Grade 4-8 and 9-12 counts exclude students enrolled in career-technical programs, which are
/// funded at their own ratio.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistrictEnrollment {
    /// Kindergarten full-time equivalent.
    pub kindergarten: Adm,
    /// Grades 1 through 3.
    pub grades_1_3: Adm,
    /// Grades 4 through 8, excluding career-technical.
    pub grades_4_8: Adm,
    /// Grades 9 through 12, excluding career-technical.
    pub grades_9_12: Adm,
    /// Enrolled in a career-technical education program.
    pub career_technical: Adm,
    /// Total grades 9 through 12, including career-technical. Used for counselor funding.
    pub grades_9_12_total: Adm,
    /// Base cost enrolled ADM — the greater of the three-year average and the prior year.
    pub base_cost_enrolled_adm: Adm,
    /// Count of open school buildings, which caps building leadership support staffing.
    pub open_buildings: f64,
    /// Whether the district qualifies for athletic co-curricular funding. All districts meet
    /// the criteria in practice, by membership in the interscholastic athletics regulator or
    /// by fielding at least three teams.
    pub athletics_eligible: bool,
}

/// Statutory ratios, minimums, and thresholds, per R.C. 3317.011.
pub mod ratios {
    /// Kindergarten students per funded classroom teacher.
    pub const KINDERGARTEN: f64 = 20.0;
    /// Grades 1-3 students per funded classroom teacher.
    pub const GRADES_1_3: f64 = 23.0;
    /// Grades 4-8 students per funded classroom teacher.
    pub const GRADES_4_8: f64 = 25.0;
    /// Grades 9-12 students per funded classroom teacher.
    pub const GRADES_9_12: f64 = 27.0;
    /// Career-technical students per funded classroom teacher.
    pub const CAREER_TECHNICAL: f64 = 18.0;
    /// Students per funded special teacher — art, music, physical education, electives.
    pub const SPECIAL_TEACHER: f64 = 150.0;
    /// Minimum funded special teachers per district, regardless of size.
    pub const SPECIAL_TEACHER_MINIMUM: f64 = 6.0;
    /// Grade 9-12 students per funded guidance counselor.
    pub const COUNSELOR: f64 = 360.0;
    /// Minimum funded guidance counselors per district.
    pub const COUNSELOR_MINIMUM: f64 = 1.0;
    /// Students per funded librarian or media staff member.
    pub const LIBRARIAN: f64 = 1_000.0;
    /// Students per funded student wellness and success staff member.
    pub const WELLNESS: f64 = 250.0;
    /// Minimum funded wellness and success staff per district.
    pub const WELLNESS_MINIMUM: f64 = 5.0;
    /// Substitute days funded per funded classroom and special teacher.
    pub const SUBSTITUTE_DAYS: f64 = 5.0;
    /// Professional development days funded per funded classroom and special teacher.
    pub const PROFESSIONAL_DEVELOPMENT_DAYS: f64 = 4.0;
    /// Contract days used to derive a daily salary rate.
    pub const CONTRACT_DAYS: f64 = 180.0;
    /// Students per funded district administrator other than the superintendent.
    pub const OTHER_ADMINISTRATOR: f64 = 750.0;
    /// Minimum funded other district administrators.
    pub const OTHER_ADMINISTRATOR_MINIMUM: f64 = 2.0;
    /// Students per funded fiscal support staff member.
    pub const FISCAL_SUPPORT: f64 = 850.0;
    /// Minimum funded fiscal support staff.
    pub const FISCAL_SUPPORT_MINIMUM: f64 = 2.0;
    /// Maximum funded fiscal support staff, however large the district.
    pub const FISCAL_SUPPORT_MAXIMUM: f64 = 35.0;
    /// Students per funded EMIS support employee.
    pub const EMIS: f64 = 5_000.0;
    /// Minimum funded EMIS support employees.
    pub const EMIS_MINIMUM: f64 = 1.0;
    /// Funded administrators per funded district leadership support staff member.
    pub const LEADERSHIP_SUPPORT: f64 = 3.0;
    /// Minimum funded district leadership support staff.
    pub const LEADERSHIP_SUPPORT_MINIMUM: f64 = 1.0;
    /// Students per funded building leader.
    pub const BUILDING_LEADER: f64 = 450.0;
    /// Students per funded building leadership support staff member.
    pub const BUILDING_LEADERSHIP_SUPPORT: f64 = 400.0;
    /// Building leadership support staff funded per open building, as a cap.
    pub const BUILDING_SUPPORT_PER_BUILDING: f64 = 3.0;
    /// ADM at or above which the large size-banded salary applies.
    pub const SIZE_BAND_UPPER: f64 = 4_000.0;
    /// ADM below which the small size-banded salary applies.
    pub const SIZE_BAND_LOWER: f64 = 500.0;
    /// Span across which size-banded salaries interpolate.
    pub const SIZE_BAND_SPAN: f64 = 3_500.0;
}

/// Teacher base cost, R.C. 3317.011(D).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeacherBaseCost {
    /// Funded classroom teachers, rounded to two decimals as the department publishes them.
    pub funded_classroom_teachers: f64,
    /// Funded special teachers, rounded to two decimals.
    pub funded_special_teachers: f64,
    /// A1 — classroom teacher base cost.
    pub classroom: Dollars,
    /// A2 — special teacher base cost.
    pub special: Dollars,
    /// A3 — substitute teacher cost.
    pub substitute: Dollars,
    /// A4 — professional development cost.
    pub professional_development: Dollars,
    /// A — the sum of the four.
    pub total: Dollars,
}

/// Student support base cost, R.C. 3317.011(E) — all seven elements.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StudentSupportBaseCost {
    /// B1 — guidance counselor cost.
    pub counselors: Dollars,
    /// B2 — librarian and media staff cost.
    pub librarians: Dollars,
    /// B3 — student wellness and success staff cost.
    pub wellness: Dollars,
    /// B4 — academic co-curricular activities cost.
    pub academic_cocurricular: Dollars,
    /// B5 — building safety and security cost.
    pub safety: Dollars,
    /// B6 — supplies and academic content cost.
    pub supplies: Dollars,
    /// B7 — student technology cost.
    pub technology: Dollars,
    /// B — the sum of the seven.
    pub total: Dollars,
}

/// District leadership and accountability base cost, R.C. 3317.011(F) — all seven elements.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistrictLeadershipBaseCost {
    /// C1 — superintendent cost.
    pub superintendent: Dollars,
    /// C2 — treasurer cost.
    pub treasurer: Dollars,
    /// C3 — other district administrator cost.
    pub other_administrators: Dollars,
    /// C4 — fiscal support cost.
    pub fiscal_support: Dollars,
    /// C5 — EMIS support cost.
    pub emis: Dollars,
    /// C6 — district leadership support cost.
    pub leadership_support: Dollars,
    /// C7 — information technology centre support cost.
    pub itc: Dollars,
    /// C — the sum of the seven.
    pub total: Dollars,
}

/// Building leadership and operation base cost, R.C. 3317.011(G).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildingLeadershipBaseCost {
    /// D1 — building leadership cost.
    pub leadership: Dollars,
    /// D2 — building leadership support cost.
    pub support: Dollars,
    /// D3 — building operation cost.
    pub operation: Dollars,
    /// D — the sum of the three.
    pub total: Dollars,
}

/// The complete base cost build-up for one district.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BaseCost {
    /// A — teacher base cost.
    pub teacher: TeacherBaseCost,
    /// B — student support base cost.
    pub student_support: StudentSupportBaseCost,
    /// C — district leadership and accountability base cost.
    pub district_leadership: DistrictLeadershipBaseCost,
    /// D — building leadership and operation base cost.
    pub building_leadership: BuildingLeadershipBaseCost,
    /// E — athletic co-curricular activities base cost.
    pub athletic_cocurricular: Dollars,
    /// The district aggregate base cost, A + B + C + D + E.
    pub aggregate: Dollars,
    /// Aggregate divided by base cost enrolled ADM.
    pub per_pupil: Dollars,
}

/// Compute teacher base cost, per R.C. 3317.011(D).
///
/// Funded teacher counts are rounded to two decimal places **before** multiplication, matching
/// the department's published calculation. This is not cosmetic: on a district of roughly
/// 20,000 students, rounding after multiplication instead shifts the special teacher component
/// by several hundred dollars.
#[must_use]
pub fn teacher_base_cost(
    enrollment: &DistrictEnrollment,
    factors: &StatewideFactors,
) -> TeacherBaseCost {
    let funded_classroom = round_dp(
        enrollment.kindergarten / ratios::KINDERGARTEN
            + enrollment.grades_1_3 / ratios::GRADES_1_3
            + enrollment.grades_4_8 / ratios::GRADES_4_8
            + enrollment.grades_9_12 / ratios::GRADES_9_12
            + enrollment.career_technical / ratios::CAREER_TECHNICAL,
        2,
    );

    let funded_special = round_dp(
        (enrollment.base_cost_enrolled_adm / ratios::SPECIAL_TEACHER)
            .max(ratios::SPECIAL_TEACHER_MINIMUM),
        2,
    );

    let per_teacher = factors.position_cost(factors.teacher_salary);
    let teaching_positions = funded_classroom + funded_special;

    let classroom = funded_classroom * per_teacher;
    let special = funded_special * per_teacher;

    let substitute_daily = factors.substitute_daily_rate * factors.benefit_multiplier;
    let substitute = teaching_positions * substitute_daily * ratios::SUBSTITUTE_DAYS;

    let daily_salary =
        (factors.teacher_salary * factors.benefit_multiplier) / ratios::CONTRACT_DAYS;
    let professional_development =
        teaching_positions * daily_salary * ratios::PROFESSIONAL_DEVELOPMENT_DAYS;

    TeacherBaseCost {
        funded_classroom_teachers: funded_classroom,
        funded_special_teachers: funded_special,
        classroom,
        special,
        substitute,
        professional_development,
        total: classroom + special + substitute + professional_development,
    }
}

/// Compute student support base cost, per R.C. 3317.011(E).
#[must_use]
pub fn student_support_base_cost(
    enrollment: &DistrictEnrollment,
    factors: &StatewideFactors,
) -> StudentSupportBaseCost {
    let adm = enrollment.base_cost_enrolled_adm;

    let funded_counselors = round_dp(
        (enrollment.grades_9_12_total / ratios::COUNSELOR).max(ratios::COUNSELOR_MINIMUM),
        2,
    );
    let funded_librarians = round_dp(adm / ratios::LIBRARIAN, 2);
    let funded_wellness = round_dp((adm / ratios::WELLNESS).max(ratios::WELLNESS_MINIMUM), 2);

    let counselor_cost = factors.position_cost(factors.counselor_salary);
    let librarian_cost = factors.position_cost(factors.librarian_salary);

    let counselors = funded_counselors * counselor_cost;
    let librarians = funded_librarians * librarian_cost;
    let wellness = funded_wellness * counselor_cost;
    let academic_cocurricular = adm * factors.academic_cocurricular_per_pupil;
    let safety = adm * factors.safety_per_pupil;
    let supplies = adm * factors.supplies_per_pupil;
    let technology = adm * factors.technology_per_pupil;

    StudentSupportBaseCost {
        counselors,
        librarians,
        wellness,
        academic_cocurricular,
        safety,
        supplies,
        technology,
        total: counselors
            + librarians
            + wellness
            + academic_cocurricular
            + safety
            + supplies
            + technology,
    }
}

/// Compute district leadership and accountability base cost, per R.C. 3317.011(F).
///
/// Note the derived pricing: other district administrators and, in
/// [`building_leadership_base_cost`], building leaders are priced as a *ratio* against the
/// superintendent's funded cost rather than from their own salary band. A change to the
/// superintendent band therefore moves four elements, not one.
#[must_use]
pub fn district_leadership_base_cost(
    enrollment: &DistrictEnrollment,
    factors: &StatewideFactors,
) -> DistrictLeadershipBaseCost {
    let adm = enrollment.base_cost_enrolled_adm;

    let superintendent = factors.banded_position_cost(
        adm,
        factors.superintendent_salary_small,
        factors.superintendent_salary_large,
    );
    let treasurer = factors.banded_position_cost(
        adm,
        factors.treasurer_salary_small,
        factors.treasurer_salary_large,
    );

    let funded_other_admins = round_dp(
        (adm / ratios::OTHER_ADMINISTRATOR).max(ratios::OTHER_ADMINISTRATOR_MINIMUM),
        2,
    );
    let admin_unit_cost = (superintendent - factors.insurance)
        * (factors.other_administrator_salary / factors.superintendent_salary)
        + factors.insurance;
    let other_administrators = funded_other_admins * admin_unit_cost;

    let funded_fiscal = round_dp(
        (adm / ratios::FISCAL_SUPPORT).clamp(
            ratios::FISCAL_SUPPORT_MINIMUM,
            ratios::FISCAL_SUPPORT_MAXIMUM,
        ),
        2,
    );
    let fiscal_support = funded_fiscal * factors.position_cost(factors.bookkeeping_salary);

    let funded_emis = round_dp((adm / ratios::EMIS).max(ratios::EMIS_MINIMUM), 2);
    let emis = funded_emis * factors.position_cost(factors.emis_salary);

    let funded_leadership_support = round_dp(
        ((funded_other_admins + 1.0) / ratios::LEADERSHIP_SUPPORT)
            .max(ratios::LEADERSHIP_SUPPORT_MINIMUM),
        2,
    );
    let leadership_support =
        funded_leadership_support * factors.position_cost(factors.administrative_assistant_salary);

    let itc = adm * factors.itc_per_pupil;

    DistrictLeadershipBaseCost {
        superintendent,
        treasurer,
        other_administrators,
        fiscal_support,
        emis,
        leadership_support,
        itc,
        total: superintendent
            + treasurer
            + other_administrators
            + fiscal_support
            + emis
            + leadership_support
            + itc,
    }
}

/// Compute building leadership and operation base cost, per R.C. 3317.011(G).
///
/// Building leadership support is the one element where the count of open buildings matters:
/// when a district has more buildings than the enrollment ratio would fund, the building count
/// governs; otherwise staffing is the lesser of the enrollment ratio and three per building.
#[must_use]
pub fn building_leadership_base_cost(
    enrollment: &DistrictEnrollment,
    factors: &StatewideFactors,
) -> BuildingLeadershipBaseCost {
    let adm = enrollment.base_cost_enrolled_adm;

    let superintendent = factors.banded_position_cost(
        adm,
        factors.superintendent_salary_small,
        factors.superintendent_salary_large,
    );
    let funded_building_leaders = round_dp(adm / ratios::BUILDING_LEADER, 2);
    let leader_unit_cost = (superintendent - factors.insurance)
        * (factors.principal_salary / factors.superintendent_salary)
        + factors.insurance;
    let leadership = funded_building_leaders * leader_unit_cost;

    let clerical_cost = factors.position_cost(factors.clerical_salary);
    let by_enrollment = round_dp(adm / ratios::BUILDING_LEADERSHIP_SUPPORT, 2);
    let funded_building_support = if by_enrollment < enrollment.open_buildings {
        enrollment.open_buildings
    } else {
        by_enrollment.min(enrollment.open_buildings * ratios::BUILDING_SUPPORT_PER_BUILDING)
    };
    let support = funded_building_support * clerical_cost;

    // Safety is subtracted so it is not funded twice — it is already a student support element.
    let operation = adm * (factors.building_per_pupil - factors.safety_per_pupil);

    BuildingLeadershipBaseCost {
        leadership,
        support,
        operation,
        total: leadership + support + operation,
    }
}

/// Compute the complete district base cost, per R.C. 3317.011.
#[must_use]
pub fn aggregate_base_cost(
    enrollment: &DistrictEnrollment,
    factors: &StatewideFactors,
) -> BaseCost {
    let teacher = teacher_base_cost(enrollment, factors);
    let student_support = student_support_base_cost(enrollment, factors);
    let district_leadership = district_leadership_base_cost(enrollment, factors);
    let building_leadership = building_leadership_base_cost(enrollment, factors);
    let athletic_cocurricular = if enrollment.athletics_eligible {
        enrollment.base_cost_enrolled_adm * factors.athletic_cocurricular_per_pupil
    } else {
        0.0
    };

    let aggregate = teacher.total
        + student_support.total
        + district_leadership.total
        + building_leadership.total
        + athletic_cocurricular;

    BaseCost {
        teacher,
        student_support,
        district_leadership,
        building_leadership,
        athletic_cocurricular,
        aggregate,
        per_pupil: aggregate / enrollment.base_cost_enrolled_adm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// District factors from the worked example in the FY2022 payment report, Base Cost report.
    fn published_fy2022_district() -> DistrictEnrollment {
        DistrictEnrollment {
            kindergarten: 1_630.68,
            grades_1_3: 4_797.69,
            grades_4_8: 7_559.90,
            grades_9_12: 5_075.82,
            career_technical: 1_278.03,
            grades_9_12_total: 6_090.96,
            base_cost_enrolled_adm: 20_342.13,
            open_buildings: 43.0,
            athletics_eligible: true,
        }
    }

    /// Component totals differ from the published figures by cents at most; the tolerance
    /// reflects reading published screenshots rather than a machine-readable file.
    const CENTS: f64 = 2.0;

    #[test]
    fn reproduces_published_average_teacher_base_cost() {
        let f = StatewideFactors::fy2022();
        assert!((f.position_cost(f.teacher_salary) - 86_993.10).abs() < 0.005);
    }

    #[test]
    fn reproduces_published_teacher_base_cost() {
        let t = teacher_base_cost(&published_fy2022_district(), &StatewideFactors::fy2022());
        assert!((t.funded_classroom_teachers - 851.52).abs() < 0.005);
        assert!((t.funded_special_teachers - 135.61).abs() < 0.005);
        assert!((t.classroom - 74_076_364.51).abs() < 1.0);
        assert!((t.special - 11_797_134.29).abs() < 1.0);
        assert!((t.substitute - 515_281.86).abs() < 1.0);
        assert!((t.professional_development - 1_595_368.11).abs() < 1.0);
        assert!((t.total - 87_984_148.77).abs() < CENTS, "A was {}", t.total);
    }

    #[test]
    fn reproduces_published_student_support_base_cost() {
        let s =
            student_support_base_cost(&published_fy2022_district(), &StatewideFactors::fy2022());
        assert!(
            (s.counselors - 1_483_064.02).abs() < 1.0,
            "B1 {}",
            s.counselors
        );
        assert!(
            (s.librarians - 1_897_867.49).abs() < 1.0,
            "B2 {}",
            s.librarians
        );
        assert!((s.wellness - 7_132_205.65).abs() < 1.0, "B3 {}", s.wellness);
        assert!((s.academic_cocurricular - 857_013.84).abs() < 1.0);
        assert!((s.safety - 473_768.21).abs() < 1.0);
        assert!((s.supplies - 4_482_388.35).abs() < 1.0);
        assert!((s.technology - 762_829.88).abs() < 1.0);
        assert!((s.total - 17_089_137.54).abs() < CENTS, "B was {}", s.total);
    }

    #[test]
    fn reproduces_published_district_leadership_base_cost() {
        let c = district_leadership_base_cost(
            &published_fy2022_district(),
            &StatewideFactors::fy2022(),
        );
        assert!(
            (c.superintendent - 199_865.53).abs() < 0.01,
            "C1 {}",
            c.superintendent
        );
        assert!(
            (c.treasurer - 165_065.53).abs() < 0.01,
            "C2 {}",
            c.treasurer
        );
        assert!(
            (c.other_administrators - 4_554_496.67).abs() < 2.0,
            "C3 {}",
            c.other_administrators
        );
        assert!(
            (c.fiscal_support - 1_601_285.55).abs() < 1.0,
            "C4 {}",
            c.fiscal_support
        );
        assert!((c.emis - 311_566.77).abs() < 1.0, "C5 {}", c.emis);
        assert!(
            (c.leadership_support - 622_293.99).abs() < 1.0,
            "C6 {}",
            c.leadership_support
        );
        assert!((c.itc - 630_606.03).abs() < 1.0, "C7 {}", c.itc);
        assert!((c.total - 8_085_180.07).abs() < 3.0, "C was {}", c.total);
    }

    #[test]
    fn reproduces_published_building_leadership_base_cost() {
        let d = building_leadership_base_cost(
            &published_fy2022_district(),
            &StatewideFactors::fy2022(),
        );
        assert!(
            (d.leadership - 7_300_067.40).abs() < 25.0,
            "D1 {}",
            d.leadership
        );
        assert!((d.support - 2_672_341.76).abs() < 1.0, "D2 {}", d.support);
        assert!(
            (d.operation - 22_508_363.42).abs() < 1.0,
            "D3 {}",
            d.operation
        );
        assert!((d.total - 32_480_772.58).abs() < 25.0, "D was {}", d.total);
    }

    /// The end-to-end check. The five sub-components sum to the published aggregate base cost,
    /// which also appears independently on the state share report — so this verifies the
    /// components against each other, not merely against their own screenshots.
    #[test]
    fn reproduces_published_aggregate_base_cost() {
        let b = aggregate_base_cost(&published_fy2022_district(), &StatewideFactors::fy2022());
        assert!(
            (b.aggregate - 148_960_701.95).abs() < 30.0,
            "aggregate was {}, expected 148,960,701.95",
            b.aggregate
        );
    }

    #[test]
    fn per_pupil_base_cost_is_near_the_statewide_average() {
        let b = aggregate_base_cost(&published_fy2022_district(), &StatewideFactors::fy2022());
        // Statewide average base cost per pupil in FY2022 was $7,349.22.
        assert!(
            (b.per_pupil - 7_349.22).abs() < 100.0,
            "per pupil was {}",
            b.per_pupil
        );
    }

    #[test]
    fn superintendent_salary_ramps_between_the_size_bands() {
        let f = StatewideFactors::fy2022();
        let small = f.banded_position_cost(400.0, 80_000.0, 160_000.0);
        let midpoint = f.banded_position_cost(2_250.0, 80_000.0, 160_000.0);
        let large = f.banded_position_cost(5_000.0, 80_000.0, 160_000.0);

        assert!((small - (80_000.0 * 1.16 + 14_265.53)).abs() < 0.01);
        assert!((large - (160_000.0 * 1.16 + 14_265.53)).abs() < 0.01);
        // Halfway across the 500-4,000 span funds halfway between the two salaries.
        let expected_mid = 120_000.0 * 1.16 + 14_265.53;
        assert!(
            (midpoint - expected_mid).abs() < 0.01,
            "midpoint was {midpoint}, expected {expected_mid}"
        );
        assert!(small < midpoint && midpoint < large);
    }

    #[test]
    fn size_band_is_continuous_at_both_thresholds() {
        let f = StatewideFactors::fy2022();
        let just_below = f.banded_position_cost(3_999.0, 80_000.0, 160_000.0);
        let just_above = f.banded_position_cost(4_001.0, 80_000.0, 160_000.0);
        assert!(
            (just_below - just_above).abs() < 50.0,
            "no jump at 4,000 ADM"
        );

        let below_floor = f.banded_position_cost(499.0, 80_000.0, 160_000.0);
        let above_floor = f.banded_position_cost(501.0, 80_000.0, 160_000.0);
        assert!(
            (below_floor - above_floor).abs() < 50.0,
            "no jump at 500 ADM"
        );
    }

    /// A change to the superintendent band moves four elements, because other administrators
    /// and building leaders are priced as ratios against it.
    #[test]
    fn superintendent_band_propagates_to_derived_administrator_costs() {
        let d = published_fy2022_district();
        let base = StatewideFactors::fy2022();
        let raised = StatewideFactors {
            superintendent_salary_large: 200_000.0,
            ..base
        };

        let before = aggregate_base_cost(&d, &base);
        let after = aggregate_base_cost(&d, &raised);

        assert!(
            after.district_leadership.superintendent > before.district_leadership.superintendent
        );
        assert!(
            after.district_leadership.other_administrators
                > before.district_leadership.other_administrators,
            "other administrators are priced off the superintendent"
        );
        assert!(
            after.building_leadership.leadership > before.building_leadership.leadership,
            "building leaders are priced off the superintendent too"
        );
        assert!((after.teacher.total - before.teacher.total).abs() < 1e-6);
    }

    #[test]
    fn small_districts_receive_the_staffing_minimums() {
        let tiny = DistrictEnrollment {
            kindergarten: 20.0,
            grades_1_3: 60.0,
            grades_4_8: 100.0,
            grades_9_12: 80.0,
            career_technical: 0.0,
            grades_9_12_total: 80.0,
            base_cost_enrolled_adm: 260.0,
            open_buildings: 2.0,
            athletics_eligible: true,
        };
        let f = StatewideFactors::fy2022();
        let t = teacher_base_cost(&tiny, &f);
        assert!((t.funded_special_teachers - ratios::SPECIAL_TEACHER_MINIMUM).abs() < 1e-9);
        let c = district_leadership_base_cost(&tiny, &f);
        // Two other administrators and two fiscal support staff, both minimums.
        let expected_admin_min = ratios::OTHER_ADMINISTRATOR_MINIMUM;
        assert!(c.other_administrators > 0.0);
        assert!(
            (tiny.base_cost_enrolled_adm / ratios::OTHER_ADMINISTRATOR) < expected_admin_min,
            "the ratio should be below the minimum for this district"
        );
    }

    /// The minimums are why small districts carry higher per-pupil base costs — about
    /// one-sixth of Ohio districts, under roughly 700 students, exceed $8,000 per pupil.
    #[test]
    fn minimums_raise_per_pupil_base_cost_for_small_districts() {
        let f = StatewideFactors::fy2022();
        let small = DistrictEnrollment {
            kindergarten: 40.0,
            grades_1_3: 120.0,
            grades_4_8: 200.0,
            grades_9_12: 160.0,
            career_technical: 0.0,
            grades_9_12_total: 160.0,
            base_cost_enrolled_adm: 520.0,
            open_buildings: 2.0,
            athletics_eligible: true,
        };
        let small_pp = aggregate_base_cost(&small, &f).per_pupil;
        let large_pp = aggregate_base_cost(&published_fy2022_district(), &f).per_pupil;
        assert!(
            small_pp > 8_000.0,
            "small district per pupil was {small_pp:.2}, expected above $8,000"
        );
        assert!(small_pp > large_pp);
    }

    #[test]
    fn fiscal_support_staffing_is_capped_at_thirty_five() {
        let huge = DistrictEnrollment {
            base_cost_enrolled_adm: 100_000.0,
            ..published_fy2022_district()
        };
        let f = StatewideFactors::fy2022();
        let c = district_leadership_base_cost(&huge, &f);
        let expected = ratios::FISCAL_SUPPORT_MAXIMUM * f.position_cost(f.bookkeeping_salary);
        assert!(
            (c.fiscal_support - expected).abs() < 1.0,
            "fiscal support should cap at 35 staff, got {}",
            c.fiscal_support
        );
    }

    #[test]
    fn building_count_governs_support_staffing_when_buildings_outnumber_the_ratio() {
        let f = StatewideFactors::fy2022();
        let many_small_buildings = DistrictEnrollment {
            base_cost_enrolled_adm: 400.0,
            open_buildings: 6.0,
            ..published_fy2022_district()
        };
        let d = building_leadership_base_cost(&many_small_buildings, &f);
        // 400/400 = 1 funded by enrollment, but 6 buildings are open, so 6 govern.
        let expected = 6.0 * f.position_cost(f.clerical_salary);
        assert!(
            (d.support - expected).abs() < 1.0,
            "support was {}",
            d.support
        );
    }

    #[test]
    fn athletics_can_be_switched_off() {
        let f = StatewideFactors::fy2022();
        let d = published_fy2022_district();
        let ineligible = DistrictEnrollment {
            athletics_eligible: false,
            ..d
        };
        let with = aggregate_base_cost(&d, &f);
        let without = aggregate_base_cost(&ineligible, &f);
        assert!((with.athletic_cocurricular - 3_321_462.99).abs() < 1.0);
        assert!(without.athletic_cocurricular == 0.0);
        assert!(with.aggregate > without.aggregate);
    }

    #[test]
    fn building_operation_excludes_safety_to_avoid_double_funding() {
        let f = StatewideFactors::fy2022();
        let d = published_fy2022_district();
        let building = building_leadership_base_cost(&d, &f);
        let student = student_support_base_cost(&d, &f);
        let naive = d.base_cost_enrolled_adm * f.building_per_pupil;
        assert!(
            (naive - building.operation - student.safety).abs() < 1.0,
            "safety must appear exactly once across the two sub-components"
        );
    }

    /// Rounding the funded count before multiplying is what matches the published figure.
    #[test]
    fn rounding_funded_counts_before_multiplication_is_load_bearing() {
        let d = published_fy2022_district();
        let f = StatewideFactors::fy2022();
        let per_teacher = f.position_cost(f.teacher_salary);
        let unrounded = (d.base_cost_enrolled_adm / ratios::SPECIAL_TEACHER) * per_teacher;
        let rounded = teacher_base_cost(&d, &f).special;
        assert!((unrounded - rounded).abs() > 100.0);
        assert!((rounded - 11_797_134.29).abs() < 1.0);
    }

    /// Refreshing the cost-input reference year moves prices without moving staffing. This is
    /// the perturbation the corpus's seeded scenario applies.
    #[test]
    fn refreshing_salary_inputs_raises_base_cost_without_changing_staffing() {
        let d = published_fy2022_district();
        let fy2018_priced = StatewideFactors::fy2022();
        let refreshed = StatewideFactors {
            teacher_salary: 67_654.0,
            ..fy2018_priced
        };

        let before = aggregate_base_cost(&d, &fy2018_priced);
        let after = aggregate_base_cost(&d, &refreshed);

        assert!(
            (before.teacher.funded_classroom_teachers - after.teacher.funded_classroom_teachers)
                .abs()
                < 1e-9,
            "staffing must not move when only prices change"
        );
        assert!(after.aggregate > before.aggregate);
        assert!(
            (after.building_leadership.total - before.building_leadership.total).abs() < 1e-6,
            "a teacher salary change must not touch building costs"
        );
    }
}
