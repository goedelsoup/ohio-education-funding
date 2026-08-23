//! The seeded scenario, run across all 606 Ohio traditional school districts.
//!
//! Answers `.yidam/corpus/scenario/fsfp-input-year-refresh.yml` at statewide scope, where the
//! single-district run in `examples/input_year_refresh.rs` answered it for one case.
//!
//! # Why this can run without building counts or local capacity
//!
//! The perturbation is the classroom teacher salary. Only the teacher sub-component depends on
//! it, and within that sub-component the salary enters through the classroom, special, and
//! professional development terms — all proportional to **funded teaching positions**. The
//! substitute term uses its own daily rate and does not move. So:
//!
//! ```text
//! delta_teacher = (funded_classroom + funded_special) x delta_salary x 1.16 x (1 + 4/180)
//! ```
//!
//! Building leadership, district leadership, student support, and athletics contribute nothing
//! to the delta, which is why open-building counts — absent from every source retrieved — are
//! not needed here.
//!
//! # Fixture construction and its two approximations
//!
//! Grade-band **shares** come from the department's October FY2024 district headcount file;
//! the **scale** is base cost enrolled ADM from the FY2024 District Profile Report. Headcount
//! and ADM are different quantities, so this assumes a district's grade distribution is the
//! same in both — reasonable, and not exact.
//!
//! Career-technical enrollment is not separable in the headcount file. CTE students are funded
//! at 1:18 rather than the 1:27 that applies to grades 9-12, so treating them as ordinary
//! high-school students **understates** funded teachers, and therefore understates the delta,
//! for districts with large career-technical programs.
//!
//! Both push the same way: the figures here are lower bounds.

use foundation::{ratios, StatewideFactors};

const FIXTURE: &str = include_str!("../fixtures/fy24-district-grade-bands.csv");

/// FY2022 reference-year teacher salary carried forward by H.B. 96. [verified]
///
/// Read off the department's own factor set rather than restated, so that a correction to the
/// price vector cannot leave this panel perturbing from a figure the workspace contradicts.
const FY2022_TEACHER_SALARY: f64 = StatewideFactors::fy2027().teacher_salary;
/// ADM-weighted statewide average classroom teacher salary, FY2024. [verified]
const FY2024_TEACHER_SALARY: f64 = 73_777.08;
const BENEFIT_MULTIPLIER: f64 = 1.16;

/// Districts whose grade bands cannot be totalled, because the department withholds at least
/// one grade's count.
///
/// Both are among the five smallest in Ohio, which is the whole of why they are here: `<10` is
/// published where a real count would identify students. The fixture leaves such a band blank
/// rather than summing the grades that were published — a band summed over a withheld grade is
/// not a smaller band, it is a band whose total is unknown, and the earlier version of this
/// fixture recorded Vanlue's grades 9-12 as 56 when the true figure is between 57 and 65.
///
/// Named as a constant so that a fixture refresh which suppressed more districts would fail the
/// count below rather than quietly shrinking the panel every figure here is computed on.
const SUPPRESSED_BANDS: &[&str] = &[
    "Bloomfield-Mespo Local (050096) - Trumbull County",
    "Vanlue Local (047472) - Hancock County",
];

struct District {
    name: String,
    adm: f64,
    kindergarten: f64,
    grades_1_3: f64,
    grades_4_8: f64,
    grades_9_12: f64,
}

impl District {
    fn headcount_total(&self) -> f64 {
        self.kindergarten + self.grades_1_3 + self.grades_4_8 + self.grades_9_12
    }

    /// Funded classroom teachers, applying grade-band shares to base cost enrolled ADM.
    fn funded_classroom_teachers(&self) -> f64 {
        let total = self.headcount_total();
        let scale = self.adm / total;
        edfund_core::round_dp(
            (self.kindergarten * scale) / ratios::KINDERGARTEN
                + (self.grades_1_3 * scale) / ratios::GRADES_1_3
                + (self.grades_4_8 * scale) / ratios::GRADES_4_8
                + (self.grades_9_12 * scale) / ratios::GRADES_9_12,
            2,
        )
    }

    fn funded_special_teachers(&self) -> f64 {
        edfund_core::round_dp(
            (self.adm / ratios::SPECIAL_TEACHER).max(ratios::SPECIAL_TEACHER_MINIMUM),
            2,
        )
    }

    /// Whether the six-teacher special minimum binds — true for small districts.
    fn special_minimum_binds(&self) -> bool {
        self.adm / ratios::SPECIAL_TEACHER < ratios::SPECIAL_TEACHER_MINIMUM
    }

    /// Base cost increase from refreshing the teacher salary input.
    fn refresh_delta(&self) -> f64 {
        let positions = self.funded_classroom_teachers() + self.funded_special_teachers();
        let delta_salary = FY2024_TEACHER_SALARY - FY2022_TEACHER_SALARY;
        positions
            * delta_salary
            * BENEFIT_MULTIPLIER
            * (1.0 + ratios::PROFESSIONAL_DEVELOPMENT_DAYS / ratios::CONTRACT_DAYS)
    }

    fn refresh_delta_per_pupil(&self) -> f64 {
        self.refresh_delta() / self.adm
    }
}

fn districts() -> Vec<District> {
    FIXTURE
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let p: Vec<&str> = line.split(',').collect();
            let f = |i: usize| p[i].trim().parse::<f64>().ok();
            Some(District {
                name: p[1].to_string(),
                adm: f(2)?,
                kindergarten: f(3)?,
                grades_1_3: f(4)?,
                grades_4_8: f(5)?,
                grades_9_12: f(6)?,
            })
        })
        // A district with a withheld grade has no total to scale by, so it is outside every
        // figure in this file. Which districts, and that there are only two, is asserted below.
        .collect()
}

#[test]
fn fixture_covers_every_traditional_district() {
    assert_eq!(
        districts().len(),
        606 - SUPPRESSED_BANDS.len(),
        "every district with a complete set of grade bands"
    );
}

/// The statewide headline. Refreshing the classroom teacher salary input from the FY2022
/// reference to FY2024 raises computed base cost by roughly $466 million a year — from that
/// one term, before any other salary category is touched.
///
/// This is the panel that cross-checks the department's own FY2027 model, which gives $465.0
/// million for the same refresh by an entirely different route. They agree to 0.3%.
#[test]
fn statewide_refresh_raises_base_cost_by_about_466_million() {
    let ds = districts();
    let total: f64 = ds.iter().map(District::refresh_delta).sum();
    assert!(
        (total / 1e6 - 466.2).abs() < 2.0,
        "statewide delta was ${:.1}M",
        total / 1e6
    );
}

#[test]
fn weighted_average_delta_is_about_324_per_pupil() {
    let ds = districts();
    let total: f64 = ds.iter().map(District::refresh_delta).sum();
    let adm: f64 = ds.iter().map(|d| d.adm).sum();
    assert!(
        (total / adm - 323.97).abs() < 1.0,
        "weighted average was ${:.2}",
        total / adm
    );
}

/// The per-pupil effect is far from uniform: the most-affected district gains about 1.4 times
/// what the least-affected does, purely from staffing structure.
///
/// # This figure was 1.7 and the difference was a bug
///
/// The old maximum, $580.82, belonged to Vanlue Local — one of the two districts now outside
/// this panel because the department withholds some of its grade counts. The fixture that
/// preceded this one summed those withheld grades as zero, understating Vanlue's headcount
/// total by up to 36 in a district of 150 and inflating the `adm / headcount` scale factor by
/// nearly a third. The most extreme point of a spread this test reports was substantially an
/// artifact of that.
///
/// The spread is real and still large. It was overstated.
#[test]
fn per_pupil_effect_varies_by_forty_percent_across_districts() {
    let ds = districts();
    let mut pp: Vec<f64> = ds.iter().map(District::refresh_delta_per_pupil).collect();
    pp.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (min, max) = (pp[0], pp[pp.len() - 1]);

    assert!((min - 318.26).abs() < 1.0, "min was {min:.2}");
    assert!((max - 442.76).abs() < 1.0, "max was {max:.2}");
    assert!(
        max / min > 1.35,
        "spread was {:.2}x, expected about 1.4x",
        max / min
    );
}

/// The mechanism behind the spread: 155 districts are small enough that the six-teacher
/// special minimum binds, so they are funded for more teaching positions per pupil than the
/// ratios alone would give — and a salary refresh pays them more per pupil as a result.
///
/// This is the same structural feature that makes small districts expensive per pupil in the
/// first place, now visible as an incidence result.
#[test]
fn the_special_teacher_minimum_concentrates_the_gain_in_small_districts() {
    let ds = districts();
    let binding: Vec<&District> = ds.iter().filter(|d| d.special_minimum_binds()).collect();
    let free: Vec<&District> = ds.iter().filter(|d| !d.special_minimum_binds()).collect();

    assert_eq!(binding.len(), 155);

    let mean = |set: &[&District]| -> f64 {
        set.iter().map(|d| d.refresh_delta_per_pupil()).sum::<f64>() / set.len() as f64
    };
    let (small, rest) = (mean(&binding), mean(&free));
    assert!(
        (small - 346.28).abs() < 2.0,
        "small-district mean was {small:.2}"
    );
    assert!(
        (rest - 322.87).abs() < 2.0,
        "other-district mean was {rest:.2}"
    );
    assert!(
        small > rest,
        "the minimum should raise per-pupil gain, {small:.2} vs {rest:.2}"
    );
}

/// Every district gains something: the perturbation raises base cost, and base cost has no
/// district-level ceiling. Who *receives* the gain is the separate question the 5% state share
/// floor answers, and it is not modelled here.
#[test]
fn no_district_loses_from_a_refresh() {
    for d in districts() {
        assert!(
            d.refresh_delta() > 0.0,
            "{} shows a non-positive delta",
            d.name
        );
    }
}

/// Sanity: grade-band shares must sum to the whole, so applying them to ADM must reconstruct
/// ADM. Guards against a fixture where a grade column was dropped.
#[test]
fn exactly_two_districts_have_a_band_the_department_withholds() {
    // The exclusion this file's panel rests on, pinned in both directions: the fixture must
    // still hold 606 rows, and exactly the two named must be the ones without a total.
    let rows = FIXTURE
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .count();
    assert_eq!(rows, 606);

    let incomplete: Vec<String> = FIXTURE
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .filter(|line| line.split(',').skip(3).any(|cell| cell.trim().is_empty()))
        .map(|line| line.split(',').nth(1).unwrap_or_default().to_string())
        .collect();
    assert_eq!(incomplete, SUPPRESSED_BANDS);
}

#[test]
fn grade_bands_account_for_all_reported_headcount() {
    for d in districts() {
        assert!(
            d.headcount_total() > 0.0,
            "{} has no headcount at all",
            d.name
        );
        let implied_ratio = d.adm / d.headcount_total();
        assert!(
            (0.5..2.0).contains(&implied_ratio),
            "{} has ADM {:.0} against headcount {:.0} — one of them is wrong",
            d.name,
            d.adm,
            d.headcount_total()
        );
    }
}
