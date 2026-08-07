//! Fair School Funding Plan base cost build-up, per R.C. 3317.011.
//!
//! Base cost is assembled for each district from statutory staffing ratios applied to that
//! district's own enrollment, priced at statewide average salaries for a **reference year**.
//! Changing that reference year changes only the price terms — the ratios, minimums, and ADM
//! definitions are untouched — which is what makes the input-year question a clean
//! single-parameter perturbation rather than a re-specification of the model.
//!
//! # Scope: this crate is deliberately incomplete
//!
//! R.C. 3317.011 defines base cost in five sub-components:
//!
//! | Sub-component | Section | Implemented |
//! |---|---|---|
//! | Teacher base cost | 3317.011(D) | **yes**, verified against published figures |
//! | Student support base cost | 3317.011(E) | partial — five of seven elements |
//! | District leadership and accountability | 3317.011(F) | no |
//! | Building leadership and operation | 3317.011(G) | no |
//! | Athletic co-curricular activities | 3317.011(H) | no |
//!
//! There is therefore **no `aggregate_base_cost` function**, and that omission is intentional.
//! A function returning a total from three of five sub-components would produce a number that
//! looks like base cost, is wrong by roughly a third, and would propagate into every state
//! share and scenario result downstream. Callers get the pieces that are verified and nothing
//! that pretends to be more.

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
    /// Statewide average guidance counselor salary. Also priced for wellness and success staff.
    pub counselor_salary: Dollars,
    /// Statewide average librarian and media staff salary.
    pub librarian_salary: Dollars,
    /// Average annual employer-paid insurance cost, added per funded position.
    pub insurance: Dollars,
    /// Benefit multiplier applied to salary: 1.16, being 14% retirement contribution plus 2%
    /// federal payroll taxes and workers' compensation.
    pub benefit_multiplier: f64,
    /// Daily rate for a substitute teacher, before benefits.
    pub substitute_daily_rate: Dollars,
    /// Per-pupil amount for academic (non-athletic) co-curricular activities.
    pub academic_cocurricular_per_pupil: Dollars,
    /// Per-pupil amount for building safety and security.
    pub safety_per_pupil: Dollars,
}

impl StatewideFactors {
    /// The FY2022 factors, priced from **FY2018** statewide averages.
    ///
    /// Every value is taken from the department's FY2022 School Finance Payment Report
    /// line-by-line explanation.
    #[must_use]
    pub const fn fy2022() -> Self {
        Self {
            teacher_salary: 62_696.18,
            counselor_salary: 63_263.80,
            librarian_salary: 68_139.33,
            insurance: 14_265.53,
            benefit_multiplier: 1.16,
            substitute_daily_rate: 90.0,
            academic_cocurricular_per_pupil: 42.13,
            safety_per_pupil: 23.29,
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
}

/// A district's enrollment, broken out as the base cost calculation requires.
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
}

/// Statutory student-to-teacher ratios, per R.C. 3317.011(D)(1).
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
}

/// The teacher base cost and its four published components.
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

/// The student support elements this crate implements — five of the seven in R.C. 3317.011(E).
///
/// Missing: supplies and academic content, technology, and ITC support, whose per-pupil
/// amounts are published but whose statutory treatment has not been confirmed here.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StudentSupportPartial {
    /// Funded guidance counselors.
    pub funded_counselors: f64,
    /// Funded librarians and media staff.
    pub funded_librarians: f64,
    /// Funded student wellness and success staff.
    pub funded_wellness_staff: f64,
    /// Guidance counselor cost.
    pub counselors: Dollars,
    /// Librarian and media staff cost.
    pub librarians: Dollars,
    /// Student wellness and success staff cost.
    pub wellness: Dollars,
    /// Academic co-curricular activities cost.
    pub academic_cocurricular: Dollars,
    /// Building safety and security cost.
    pub safety: Dollars,
    /// Sum of the five implemented elements. **Not** the full student support base cost.
    pub partial_total: Dollars,
}

/// Compute the five implemented student support elements, per R.C. 3317.011(E).
///
/// The returned total is explicitly partial — see the crate-level note on scope.
#[must_use]
pub fn student_support_partial(
    enrollment: &DistrictEnrollment,
    factors: &StatewideFactors,
) -> StudentSupportPartial {
    let adm = enrollment.base_cost_enrolled_adm;

    let funded_counselors =
        (enrollment.grades_9_12_total / ratios::COUNSELOR).max(ratios::COUNSELOR_MINIMUM);
    let funded_librarians = adm / ratios::LIBRARIAN;
    let funded_wellness = (adm / ratios::WELLNESS).max(ratios::WELLNESS_MINIMUM);

    let counselor_cost = factors.position_cost(factors.counselor_salary);
    let librarian_cost = factors.position_cost(factors.librarian_salary);

    let counselors = funded_counselors * counselor_cost;
    let librarians = funded_librarians * librarian_cost;
    let wellness = funded_wellness * counselor_cost;
    let academic_cocurricular = adm * factors.academic_cocurricular_per_pupil;
    let safety = adm * factors.safety_per_pupil;

    StudentSupportPartial {
        funded_counselors,
        funded_librarians,
        funded_wellness_staff: funded_wellness,
        counselors,
        librarians,
        wellness,
        academic_cocurricular,
        safety,
        partial_total: counselors + librarians + wellness + academic_cocurricular + safety,
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
        }
    }

    #[test]
    fn reproduces_published_average_teacher_base_cost() {
        let f = StatewideFactors::fy2022();
        // ($62,696.18 × 1.16) + $14,265.53 = $86,993.10
        assert!((f.position_cost(f.teacher_salary) - 86_993.10).abs() < 0.005);
    }

    #[test]
    fn reproduces_published_funded_teacher_counts() {
        let t = teacher_base_cost(&published_fy2022_district(), &StatewideFactors::fy2022());
        assert!(
            (t.funded_classroom_teachers - 851.52).abs() < 0.005,
            "classroom teachers were {}",
            t.funded_classroom_teachers
        );
        assert!(
            (t.funded_special_teachers - 135.61).abs() < 0.005,
            "special teachers were {}",
            t.funded_special_teachers
        );
    }

    #[test]
    fn reproduces_published_classroom_teacher_base_cost() {
        let t = teacher_base_cost(&published_fy2022_district(), &StatewideFactors::fy2022());
        assert!(
            (t.classroom - 74_076_364.51).abs() < 1.0,
            "A1 was {}",
            t.classroom
        );
    }

    #[test]
    fn reproduces_published_special_teacher_base_cost() {
        let t = teacher_base_cost(&published_fy2022_district(), &StatewideFactors::fy2022());
        assert!(
            (t.special - 11_797_134.29).abs() < 1.0,
            "A2 was {}",
            t.special
        );
    }

    #[test]
    fn reproduces_published_substitute_cost() {
        let t = teacher_base_cost(&published_fy2022_district(), &StatewideFactors::fy2022());
        // (851.52 + 135.61) × ($90 × 1.16) × 5 = $515,281.86
        assert!(
            (t.substitute - 515_281.86).abs() < 1.0,
            "A3 was {}",
            t.substitute
        );
    }

    #[test]
    fn reproduces_published_professional_development_cost() {
        let t = teacher_base_cost(&published_fy2022_district(), &StatewideFactors::fy2022());
        assert!(
            (t.professional_development - 1_595_368.11).abs() < 1.0,
            "A4 was {}",
            t.professional_development
        );
    }

    /// The published total teacher base cost for the worked example.
    #[test]
    fn reproduces_published_total_teacher_base_cost() {
        let t = teacher_base_cost(&published_fy2022_district(), &StatewideFactors::fy2022());
        assert!(
            (t.total - 87_984_148.77).abs() < 2.0,
            "A was {}, expected 87,984,148.77",
            t.total
        );
    }

    /// Rounding the funded count before multiplying is what matches the published figure.
    /// Keeping full precision instead moves the special teacher component by hundreds.
    #[test]
    fn rounding_funded_counts_before_multiplication_is_load_bearing() {
        let d = published_fy2022_district();
        let f = StatewideFactors::fy2022();
        let per_teacher = f.position_cost(f.teacher_salary);

        let unrounded = (d.base_cost_enrolled_adm / ratios::SPECIAL_TEACHER) * per_teacher;
        let rounded = teacher_base_cost(&d, &f).special;

        assert!(
            (unrounded - rounded).abs() > 100.0,
            "the two approaches should differ materially; got {unrounded} vs {rounded}"
        );
        assert!(
            (rounded - 11_797_134.29).abs() < 1.0,
            "only the rounded form matches the department"
        );
    }

    #[test]
    fn small_districts_receive_the_special_teacher_minimum() {
        let tiny = DistrictEnrollment {
            kindergarten: 20.0,
            grades_1_3: 60.0,
            grades_4_8: 100.0,
            grades_9_12: 80.0,
            career_technical: 0.0,
            grades_9_12_total: 80.0,
            base_cost_enrolled_adm: 260.0,
        };
        let t = teacher_base_cost(&tiny, &StatewideFactors::fy2022());
        // 260 / 150 = 1.73, floored up to the statutory minimum of 6.
        assert!((t.funded_special_teachers - ratios::SPECIAL_TEACHER_MINIMUM).abs() < 1e-9);
    }

    /// The minimums are why small districts carry higher per-pupil base costs — about
    /// one-sixth of Ohio districts, under roughly 700 students, exceed $8,000 per pupil.
    #[test]
    fn minimums_raise_per_pupil_cost_for_small_districts() {
        let f = StatewideFactors::fy2022();
        let small = DistrictEnrollment {
            kindergarten: 40.0,
            grades_1_3: 120.0,
            grades_4_8: 200.0,
            grades_9_12: 160.0,
            career_technical: 0.0,
            grades_9_12_total: 160.0,
            base_cost_enrolled_adm: 520.0,
        };
        let large = published_fy2022_district();

        let small_pp = teacher_base_cost(&small, &f).total / small.base_cost_enrolled_adm;
        let large_pp = teacher_base_cost(&large, &f).total / large.base_cost_enrolled_adm;
        assert!(
            small_pp > large_pp,
            "small district {small_pp:.2} should exceed large district {large_pp:.2}"
        );
    }

    #[test]
    fn career_technical_students_are_funded_at_the_richest_ratio() {
        let f = StatewideFactors::fy2022();
        let base = DistrictEnrollment {
            kindergarten: 0.0,
            grades_1_3: 0.0,
            grades_4_8: 0.0,
            grades_9_12: 1_800.0,
            career_technical: 0.0,
            grades_9_12_total: 1_800.0,
            base_cost_enrolled_adm: 1_800.0,
        };
        let mut shifted = base;
        shifted.grades_9_12 = 0.0;
        shifted.career_technical = 1_800.0;

        let general = teacher_base_cost(&base, &f).funded_classroom_teachers;
        let cte = teacher_base_cost(&shifted, &f).funded_classroom_teachers;
        assert!(cte > general, "1:18 must fund more teachers than 1:27");
    }

    #[test]
    fn student_support_applies_its_minimums() {
        let tiny = DistrictEnrollment {
            kindergarten: 10.0,
            grades_1_3: 30.0,
            grades_4_8: 50.0,
            grades_9_12: 40.0,
            career_technical: 0.0,
            grades_9_12_total: 40.0,
            base_cost_enrolled_adm: 130.0,
        };
        let s = student_support_partial(&tiny, &StatewideFactors::fy2022());
        assert!((s.funded_counselors - ratios::COUNSELOR_MINIMUM).abs() < 1e-9);
        assert!((s.funded_wellness_staff - ratios::WELLNESS_MINIMUM).abs() < 1e-9);
    }

    #[test]
    fn student_support_scales_per_pupil_amounts_with_enrollment() {
        let d = published_fy2022_district();
        let s = student_support_partial(&d, &StatewideFactors::fy2022());
        assert!((s.academic_cocurricular - d.base_cost_enrolled_adm * 42.13).abs() < 0.01);
        assert!((s.safety - d.base_cost_enrolled_adm * 23.29).abs() < 0.01);
    }

    /// Refreshing the cost-input reference year moves only the price terms. This is the
    /// perturbation the corpus's seeded scenario applies.
    #[test]
    fn refreshing_salary_inputs_raises_base_cost_without_changing_staffing() {
        let d = published_fy2022_district();
        let fy2018_priced = StatewideFactors::fy2022();
        let refreshed = StatewideFactors {
            teacher_salary: 67_654.0,
            ..fy2018_priced
        };

        let before = teacher_base_cost(&d, &fy2018_priced);
        let after = teacher_base_cost(&d, &refreshed);

        assert!(
            (before.funded_classroom_teachers - after.funded_classroom_teachers).abs() < 1e-9,
            "staffing must not move when only prices change"
        );
        assert!(after.total > before.total);

        // An 8.5% salary rise produces a broadly proportional rise in teacher base cost.
        let growth = after.total / before.total - 1.0;
        assert!(
            (0.06..0.09).contains(&growth),
            "expected roughly proportional growth, got {growth:.4}"
        );
    }
}
