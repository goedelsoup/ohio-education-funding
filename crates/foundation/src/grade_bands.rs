//! FY2024 grade-band headcounts, scaled to ADM — the panel that runs a scenario statewide.
//!
//! # What this is for
//!
//! [`crate::department_model`] is the department's own FY2027 calculation, and it is the right
//! panel when the question is "does this crate reproduce the department". This is the panel for
//! the other question: **what a change to an input would do**, across all 606 traditional
//! districts, from sources the corpus holds rather than from a departmental result.
//!
//! It answers `.yidam/corpus/scenario/fsfp-input-year-refresh.yml` at statewide scope, where
//! `examples/input_year_refresh.rs` answers it for one district.
//!
//! # Why a scenario over this panel needs no building counts
//!
//! The perturbation is the classroom teacher salary. Only the teacher sub-component depends on
//! it, and within that sub-component the salary enters through the classroom, special and
//! professional development terms — all proportional to **funded teaching positions**. The
//! substitute term uses its own daily rate and does not move. Building leadership, district
//! leadership, student support and athletics contribute nothing. See
//! [`crate::teacher_salary_refresh_delta`], which is that arithmetic.
//!
//! # Two approximations, and they push the same way
//!
//! Grade-band **shares** come from the department's October FY2024 district headcount file; the
//! **scale** is base cost enrolled ADM from the FY2024 District Profile Report. Headcount and
//! ADM are different quantities, so this assumes a district's grade distribution is the same in
//! both — reasonable, and not exact.
//!
//! Career-technical enrolment is not separable in the headcount file. CTE students are funded at
//! 1:18 rather than the 1:27 that applies to grades 9-12, so treating them as ordinary
//! high-school students **understates** funded teachers, and therefore understates any delta,
//! for districts with large career-technical programmes.
//!
//! Both push the same way: figures computed over this panel are lower bounds.
//!
//! # A district with a withheld grade has no total to scale by
//!
//! `<10` is published where a real count would identify students, and the fixture leaves such a
//! band **blank** rather than summing the grades that were published — a band summed over a
//! withheld grade is not a smaller band, it is a band whose total is unknown. Two districts are
//! affected, both among the five smallest in Ohio, and [`districts`] excludes them.
//! [`SUPPRESSED_BANDS`] names them, so that a refresh which suppressed more would fail a count
//! rather than quietly shrink the panel.

use std::sync::OnceLock;

use crate::ratios;

/// The committed grade-band panel.
pub const FIXTURE: &str = include_str!("../fixtures/fy24-district-grade-bands.csv");

/// The header this reader was written against.
pub const EXPECTED_HEADER: &str = "irn,district,enrolled_adm_fy24,headcount_kindergarten,\
headcount_grades_1_3,headcount_grades_4_8,headcount_grades_9_12";

/// The districts whose grade bands cannot be totalled, because a grade's count is withheld.
///
/// See the module note. Named rather than counted so that a fixture refresh which suppressed a
/// third district says which.
pub const SUPPRESSED_BANDS: &[&str] = &[
    "Bloomfield-Mespo Local (050096) - Trumbull County",
    "Vanlue Local (047472) - Hancock County",
];

/// One district's grade-band headcounts and the ADM they are scaled to.
#[derive(Debug, Clone, PartialEq)]
pub struct GradeBands {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's published name.
    pub name: String,
    /// Base cost enrolled ADM, FY2024 — the scale.
    pub adm: f64,
    /// Kindergarten headcount.
    pub kindergarten: f64,
    /// Grades 1-3 headcount.
    pub grades_1_3: f64,
    /// Grades 4-8 headcount.
    pub grades_4_8: f64,
    /// Grades 9-12 headcount, career-technical included. See the module note.
    pub grades_9_12: f64,
}

impl GradeBands {
    /// The headcount across all four bands.
    #[must_use]
    pub fn headcount_total(&self) -> f64 {
        self.kindergarten + self.grades_1_3 + self.grades_4_8 + self.grades_9_12
    }

    /// Funded classroom teachers, applying grade-band shares to base cost enrolled ADM.
    ///
    /// Rounded to two decimals where the department rounds, which is before the multiplication
    /// that prices them.
    #[must_use]
    pub fn funded_classroom_teachers(&self) -> f64 {
        let total = self.headcount_total();
        if total <= 0.0 {
            return 0.0;
        }
        let scale = self.adm / total;
        edfund_core::round_dp(
            (self.kindergarten * scale) / ratios::KINDERGARTEN
                + (self.grades_1_3 * scale) / ratios::GRADES_1_3
                + (self.grades_4_8 * scale) / ratios::GRADES_4_8
                + (self.grades_9_12 * scale) / ratios::GRADES_9_12,
            2,
        )
    }

    /// Funded special teachers — one per 150 pupils, never fewer than six.
    #[must_use]
    pub fn funded_special_teachers(&self) -> f64 {
        edfund_core::round_dp(
            (self.adm / ratios::SPECIAL_TEACHER).max(ratios::SPECIAL_TEACHER_MINIMUM),
            2,
        )
    }

    /// Whether the six-teacher special minimum binds — true for small districts.
    ///
    /// Where it binds, a scenario's per-pupil effect is larger than the formula's ratios alone
    /// would give, because the district is funded above them.
    #[must_use]
    pub fn special_minimum_binds(&self) -> bool {
        self.adm / ratios::SPECIAL_TEACHER < ratios::SPECIAL_TEACHER_MINIMUM
    }

    /// Funded teaching positions — classroom and special together.
    #[must_use]
    pub fn funded_positions(&self) -> f64 {
        self.funded_classroom_teachers() + self.funded_special_teachers()
    }
}

/// Every district with a complete set of grade bands.
///
/// The two in [`SUPPRESSED_BANDS`] are absent: they have no total to scale by, so they are
/// outside every figure computed over this panel.
///
/// # Panics
///
/// If the fixture's header is not [`EXPECTED_HEADER`], or a row's width differs from it — both
/// by way of [`edfund_core::csv::rows`].
#[must_use]
pub fn districts() -> Vec<GradeBands> {
    cached().clone()
}

/// The fixture, parsed once.
///
/// `OnceLock` for the reason `project::panel`'s reader has one: the parse is pure and
/// the file is compiled in, and a lookup helper that re-read it per call turned a scan
/// over districts into a quadratic one.
fn cached() -> &'static Vec<GradeBands> {
    static ROWS: OnceLock<Vec<GradeBands>> = OnceLock::new();
    ROWS.get_or_init(parse)
}

fn parse() -> Vec<GradeBands> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            let irn = row.str(0);
            if irn.is_empty() {
                return None;
            }
            Some(GradeBands {
                irn: irn.to_string(),
                name: row.str(1).to_string(),
                adm: row.num(2)?,
                kindergarten: row.num(3)?,
                grades_1_3: row.num(4)?,
                grades_4_8: row.num(5)?,
                grades_9_12: row.num(6)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_is_every_district_with_a_complete_set_of_bands() {
        assert_eq!(districts().len(), 606 - SUPPRESSED_BANDS.len());
    }

    /// The suppressed districts are absent from the panel and present in the fixture, which is
    /// the difference between "excluded" and "not published".
    #[test]
    fn the_suppressed_districts_are_named_and_excluded() {
        let names: Vec<String> = districts().into_iter().map(|d| d.name).collect();
        for suppressed in SUPPRESSED_BANDS {
            assert!(
                !names.iter().any(|n| n == suppressed),
                "{suppressed} should have no total to scale by"
            );
            assert!(
                FIXTURE.contains(suppressed),
                "{suppressed} should still be a row of the fixture"
            );
        }
        assert_eq!(
            FIXTURE.lines().filter(|l| !l.trim().is_empty()).count() - 1,
            606
        );
    }

    /// The six-teacher minimum binds only at the small end, which is what makes it worth
    /// carrying as a predicate rather than folding into the count.
    #[test]
    fn the_special_teacher_minimum_binds_only_for_small_districts() {
        for d in districts() {
            assert_eq!(
                d.special_minimum_binds(),
                d.adm < ratios::SPECIAL_TEACHER * ratios::SPECIAL_TEACHER_MINIMUM,
                "{}: {} pupils",
                d.name,
                d.adm
            );
        }
    }
}
