//! Verify the base cost implementation against the department's own FY2027 model, and answer
//! the guarantee question the scenario left open.
//!
//! The fixture is extracted from `FY27 TRAD State Foundation Funding Calculator`, the
//! Department of Education and Workforce's working spreadsheet for the terminal year of the
//! Fair School Funding Plan phase-in. It carries, per district: base cost enrolled ADM broken
//! into the exact grade bands the formula uses, funded teacher counts, teacher base cost,
//! aggregate base cost, base cost per pupil, and the temporary transitional aid guarantee.
//!
//! This is a far stronger check than the single published worked example: 609 districts,
//! spanning three orders of magnitude of enrollment, against a factor set (`fy2027`) two
//! reference years newer than the one the implementation was originally written against.

use foundation::{ratios, teacher_base_cost, DistrictEnrollment, StatewideFactors};

const FIXTURE: &str = include_str!("../fixtures/fy27-department-model.csv");

/// ADM-weighted statewide average classroom teacher salary, FY2024, from the District Profile
/// Report. The refreshed input a reference-year update would adopt. [verified]
const FY2024_TEACHER_SALARY: f64 = 73_777.08;
const BENEFIT_MULTIPLIER: f64 = 1.16;

struct Row {
    name: String,
    adm: f64,
    buildings: f64,
    kindergarten: f64,
    grades_1_3: f64,
    grades_4_8: f64,
    grades_9_12: f64,
    cte: f64,
    grades_9_12_total: f64,
    funded_classroom: f64,
    funded_special: f64,
    teacher_base_cost: f64,
    guarantee: f64,
}

impl Row {
    fn enrollment(&self) -> DistrictEnrollment {
        DistrictEnrollment {
            kindergarten: self.kindergarten,
            grades_1_3: self.grades_1_3,
            grades_4_8: self.grades_4_8,
            grades_9_12: self.grades_9_12,
            career_technical: self.cte,
            grades_9_12_total: self.grades_9_12_total,
            base_cost_enrolled_adm: self.adm,
            open_buildings: self.buildings,
            athletics_eligible: true,
        }
    }

    /// Base cost increase from refreshing the teacher salary input, using the department's own
    /// funded position counts rather than any reconstruction of them.
    fn refresh_delta(&self, from_salary: f64) -> f64 {
        (self.funded_classroom + self.funded_special)
            * (FY2024_TEACHER_SALARY - from_salary)
            * BENEFIT_MULTIPLIER
            * (1.0 + ratios::PROFESSIONAL_DEVELOPMENT_DAYS / ratios::CONTRACT_DAYS)
    }

    fn on_guarantee(&self) -> bool {
        self.guarantee > 0.0
    }
}

fn rows() -> Vec<Row> {
    FIXTURE
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let p: Vec<&str> = line.split(',').collect();
            let f = |i: usize| p[i].trim().parse::<f64>().unwrap_or(0.0);
            Row {
                name: p[1].to_string(),
                adm: f(2),
                buildings: f(3),
                kindergarten: f(4),
                grades_1_3: f(5),
                grades_4_8: f(6),
                grades_9_12: f(7),
                cte: f(8),
                grades_9_12_total: f(9),
                funded_classroom: f(10),
                funded_special: f(11),
                teacher_base_cost: f(12),
                guarantee: f(15),
            }
        })
        .collect()
}

#[test]
fn fixture_covers_the_department_model() {
    assert_eq!(rows().len(), 609);
}

/// Funded classroom teachers, reconstructed from grade-band ADM, match the department's own
/// count for every district.
#[test]
fn reproduces_departments_funded_teacher_counts_for_every_district() {
    let f = StatewideFactors::fy2027();
    let mut worst = 0.0_f64;
    for r in rows() {
        let mine = teacher_base_cost(&r.enrollment(), &f);
        let dc = (mine.funded_classroom_teachers - r.funded_classroom).abs();
        let ds = (mine.funded_special_teachers - r.funded_special).abs();
        worst = worst.max(dc).max(ds);
        assert!(
            dc < 0.02,
            "{}: classroom {} vs department {}",
            r.name,
            mine.funded_classroom_teachers,
            r.funded_classroom
        );
        assert!(ds < 0.02, "{}: special mismatch", r.name);
    }
    assert!(worst < 0.02, "worst funded-count deviation {worst}");
}

/// Teacher base cost matches the department to within a cent per district across all 609.
/// This validates the whole pricing chain — benefit multiplier, insurance, substitute rate,
/// professional development days — against a factor set the implementation never saw.
#[test]
fn reproduces_departments_teacher_base_cost_for_every_district() {
    let f = StatewideFactors::fy2027();
    let mut worst = 0.0_f64;
    let mut worst_name = String::new();
    for r in rows() {
        let mine = teacher_base_cost(&r.enrollment(), &f).total;
        let diff = (mine - r.teacher_base_cost).abs();
        if diff > worst {
            worst = diff;
            worst_name = r.name.clone();
        }
    }
    assert!(
        worst < 0.02,
        "largest deviation ${worst:.4} at {worst_name} — expected agreement to the cent"
    );
}

/// The FY2027 factor set is the FY2022 reference year H.B. 96 carries forward. Comparing it to
/// the FY2022 factor set — which was priced from FY2018 — shows what one refresh was worth.
#[test]
fn the_one_reference_year_refresh_that_happened_raised_every_price() {
    let old = StatewideFactors::fy2022();
    let new = StatewideFactors::fy2027();
    assert!(new.teacher_salary > old.teacher_salary);
    assert!(new.insurance > old.insurance);
    assert!(new.principal_salary > old.principal_salary);
    assert!(new.building_per_pupil > old.building_per_pupil);
    // Teacher salary rose 8.5% across that update; the insurance term rose far faster, 20%.
    let salary_growth = new.teacher_salary / old.teacher_salary - 1.0;
    let insurance_growth = new.insurance / old.insurance - 1.0;
    assert!(
        (0.08..0.09).contains(&salary_growth),
        "salary {salary_growth:.4}"
    );
    assert!(
        insurance_growth > salary_growth * 2.0,
        "insurance {insurance_growth:.4}"
    );
}

/// **Nearly half of Ohio's districts are off-formula in the terminal phase-in year.**
#[test]
fn almost_half_of_districts_are_on_the_guarantee_in_fy2027() {
    let rs = rows();
    let on: Vec<&Row> = rs.iter().filter(|r| r.on_guarantee()).collect();
    assert_eq!(on.len(), 294);
    assert!((on.len() as f64 / rs.len() as f64 - 0.483).abs() < 0.005);

    let guaranteed_adm: f64 = on.iter().map(|r| r.adm).sum();
    let all_adm: f64 = rs.iter().map(|r| r.adm).sum();
    assert!(
        (guaranteed_adm / all_adm - 0.541).abs() < 0.01,
        "guaranteed ADM share was {:.3}",
        guaranteed_adm / all_adm
    );
}

/// The result that qualifies the scenario: the guarantee absorbs almost half of any base cost
/// increase, because a district below its guarantee floor sees the top-up shrink by exactly
/// what the formula gains.
#[test]
fn the_guarantee_absorbs_about_half_of_a_refresh() {
    let f = StatewideFactors::fy2027();
    let rs = rows();

    let computed: f64 = rs.iter().map(|r| r.refresh_delta(f.teacher_salary)).sum();
    let absorbed: f64 = rs
        .iter()
        .filter(|r| r.on_guarantee())
        .map(|r| r.refresh_delta(f.teacher_salary).min(r.guarantee))
        .sum();
    let delivered = computed - absorbed;

    assert!(
        (computed / 1e6 - 465.0).abs() < 3.0,
        "computed increase ${:.1}M",
        computed / 1e6
    );
    assert!(
        (delivered / 1e6 - 242.1).abs() < 3.0,
        "delivered increase ${:.1}M",
        delivered / 1e6
    );
    assert!(
        (delivered / computed - 0.521).abs() < 0.02,
        "delivered share {:.3}",
        delivered / computed
    );
}

/// Two in five Ohio districts would receive nothing at all from a reference-year refresh,
/// because their guarantee exceeds the increase several times over.
#[test]
fn two_in_five_districts_would_gain_nothing_from_a_refresh() {
    let f = StatewideFactors::fy2027();
    let rs = rows();
    let stuck: Vec<&Row> = rs
        .iter()
        .filter(|r| r.on_guarantee() && r.refresh_delta(f.teacher_salary) <= r.guarantee)
        .collect();
    let lifted_off = rs
        .iter()
        .filter(|r| r.on_guarantee() && r.refresh_delta(f.teacher_salary) > r.guarantee)
        .count();

    assert_eq!(stuck.len(), 242);
    assert_eq!(lifted_off, 52);
    assert!(
        (stuck.len() as f64 / rs.len() as f64 - 0.397).abs() < 0.01,
        "stuck share {:.3}",
        stuck.len() as f64 / rs.len() as f64
    );

    let stuck_adm: f64 = stuck.iter().map(|r| r.adm).sum();
    let all_adm: f64 = rs.iter().map(|r| r.adm).sum();
    assert!(
        (stuck_adm / all_adm - 0.418).abs() < 0.01,
        "stuck ADM share {:.3}",
        stuck_adm / all_adm
    );
}

/// Guaranteed districts are not marginal cases. The median one is held more than three times
/// above what a refresh would give it, so the guarantee is not a rounding-error buffer — it is
/// the operative funding mechanism for those districts.
#[test]
fn the_median_guaranteed_district_is_far_above_its_formula_amount() {
    let f = StatewideFactors::fy2027();
    let mut ratios: Vec<f64> = rows()
        .iter()
        .filter(|r| r.on_guarantee())
        .map(|r| r.guarantee / r.refresh_delta(f.teacher_salary))
        .collect();
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = ratios[ratios.len() / 2];
    assert!(
        median > 3.0,
        "median guarantee-to-increase ratio {median:.2}"
    );
}
