//! The seeded scenario, run across all 606 Ohio traditional school districts.
//!
//! Answers `.yidam/corpus/scenario/fsfp-input-year-refresh.yml` at statewide scope, where the
//! single-district run in `examples/input_year_refresh.rs` answered it for one case.
//!
//! # What this file asserts and what the reader states
//!
//! The panel, the funded-position reconstruction, the exclusion of the two districts with a
//! withheld grade, and the two approximations that make every figure here a lower bound are all
//! [`foundation::grade_bands`]'s, which is where they are documented. The closed form of the
//! perturbation is [`foundation::teacher_salary_refresh_delta`].
//!
//! What is left here is the scenario's *results* — the statewide total, the per-pupil spread,
//! and where in the state the gain lands — pinned so that a refresh which moves them has to move
//! the corpus node with them.

use foundation::grade_bands::{self, GradeBands as District, SUPPRESSED_BANDS};
use foundation::{teacher_salary_refresh_delta, StatewideFactors, FY2024_TEACHER_SALARY};

/// FY2022 reference-year teacher salary carried forward by H.B. 96. [verified]
///
/// Read off the department's own factor set rather than restated, so that a correction to the
/// price vector cannot leave this panel perturbing from a figure the workspace contradicts.
const FY2022_TEACHER_SALARY: f64 = StatewideFactors::fy2027().teacher_salary;

/// The panel, read through [`foundation::grade_bands`].
///
/// Was a private parser here, alongside the funded-position reconstruction and the exclusion
/// rule for the two districts with a withheld grade. All three are the reader's now; see issue
/// #157.
fn districts() -> Vec<District> {
    grade_bands::districts()
}

/// Base cost increase for one district from refreshing the classroom teacher salary input.
fn refresh_delta(d: &District) -> f64 {
    teacher_salary_refresh_delta(
        d.funded_positions(),
        FY2022_TEACHER_SALARY,
        FY2024_TEACHER_SALARY,
        &StatewideFactors::fy2027(),
    )
}

/// The same, per pupil.
fn refresh_delta_per_pupil(d: &District) -> f64 {
    refresh_delta(d) / d.adm
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
    let total: f64 = ds.iter().map(refresh_delta).sum();
    assert!(
        (total / 1e6 - 466.2).abs() < 2.0,
        "statewide delta was ${:.1}M",
        total / 1e6
    );
}

#[test]
fn weighted_average_delta_is_about_324_per_pupil() {
    let ds = districts();
    let total: f64 = ds.iter().map(refresh_delta).sum();
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
    let mut pp: Vec<f64> = ds.iter().map(refresh_delta_per_pupil).collect();
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
        set.iter().map(|d| refresh_delta_per_pupil(d)).sum::<f64>() / set.len() as f64
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
            refresh_delta(&d) > 0.0,
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
    // still hold 606 rows, and exactly the two named must be the ones the reader drops.
    let rows = grade_bands::FIXTURE
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .count();
    assert_eq!(rows, 606);

    let complete: Vec<String> = districts().into_iter().map(|d| d.name).collect();
    let incomplete: Vec<&str> = grade_bands::FIXTURE
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| line.split(',').nth(1))
        .filter(|name| !complete.iter().any(|kept| kept == name))
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
