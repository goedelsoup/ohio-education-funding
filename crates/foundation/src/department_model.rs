//! The department's own FY2027 funding model, read as the check on this crate.
//!
//! # What the fixture is
//!
//! `FY27 TRAD State Foundation Funding Calculator`, the Department of Education and Workforce's
//! working spreadsheet for the terminal year of the Fair School Funding Plan phase-in. It
//! carries, per district: base cost enrolled ADM broken into the exact grade bands the formula
//! uses, funded teacher counts, teacher base cost, aggregate base cost, base cost per pupil, and
//! the temporary transitional aid guarantee.
//!
//! It is the strongest check this crate has. Not one published worked example but **609
//! districts**, spanning three orders of magnitude of enrolment, against a factor set
//! ([`crate::StatewideFactors::fy2027`]) two reference years newer than the one the
//! implementation was originally written against.
//!
//! # Why this reader is here and what it deliberately does not carry
//!
//! The fixture lives in this crate and had no reader in it: the FY2027 columns were parsed
//! privately inside `tests/department_model_fy27.rs`, again inside a `dispersion` test for the
//! guarantee column alone, and a third time — completely — by `project::panel`. See issue #157.
//!
//! This module carries **the base cost build-up and the aid totals**, which is what a
//! `foundation` claim is about. The full 150-column record — categoricals, supplements,
//! transportation, targeted assistance — is `project::panel`'s, because reproducing it needs
//! the weights and rates that live there. Two readers over one file with different jobs, rather
//! than four with the same one.
//!
//! # The ADM column names on the source sheet are stale
//!
//! [`ModelDistrict::adm_fy24`], `adm_fy25` and `adm_fy26` are named for the years the `ADM Data`
//! sheet declares. The `Base_Cost` sheet labels the same three columns FY22/FY23/FY24, which is
//! wrong — base cost enrolled ADM is their average, and an FY2027 calculation averages FY2024
//! through FY2026. An earlier version of this fixture carried the stale names.

use std::sync::OnceLock;

use edfund_core::Dollars;

use crate::DistrictEnrollment;

/// The committed model.
pub const FIXTURE: &str = include_str!("../fixtures/fy27-department-model.csv");

/// The leading columns of the header this reader indexes.
///
/// The fixture carries about 150 columns and this module reads the first twenty-one; the rest
/// are `project::panel`'s, which asserts the whole header. Checking a prefix is what makes an
/// *inserted* column here a loud failure, which is the failure this reader can suffer.
const EXPECTED_PREFIX: &str = "irn,district,base_cost_enrolled_adm,school_buildings,\
adm_kindergarten,adm_grades_1_3,adm_grades_4_8_non_cte,adm_grades_9_12_non_cte,adm_cte,\
adm_grades_9_12_total,funded_classroom_teachers,funded_special_teachers,teacher_base_cost,\
aggregate_base_cost,base_cost_per_pupil,temp_transitional_aid_guarantee,enrolled_adm_fy24,\
enrolled_adm_fy25,enrolled_adm_fy26,assessed_valuation_per_pupil_fy23,core_foundation_funding";

/// One district's base cost build-up, as the department computes it.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelDistrict {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's published name.
    pub name: String,
    /// Base cost enrolled ADM — the average of the three years below, and the denominator of
    /// every per-pupil figure in the model.
    pub base_cost_adm: f64,
    /// Open school buildings, which prices building leadership.
    pub buildings: f64,
    /// Kindergarten ADM.
    pub kindergarten: f64,
    /// Grades 1-3 ADM.
    pub grades_1_3: f64,
    /// Grades 4-8 ADM, excluding career-technical.
    pub grades_4_8: f64,
    /// Grades 9-12 ADM, excluding career-technical.
    pub grades_9_12: f64,
    /// Career-technical ADM, funded at 1:18 rather than the 1:27 that applies to grades 9-12.
    pub career_technical: f64,
    /// All grades 9-12 ADM, career-technical included. Prices guidance counsellors.
    pub grades_9_12_total: f64,
    /// Funded classroom teachers, as the department counts them.
    pub funded_classroom_teachers: f64,
    /// Funded special teachers — art, music, physical education, electives.
    pub funded_special_teachers: f64,
    /// Teacher base cost, the largest component of the build-up.
    pub teacher_base_cost: Dollars,
    /// Aggregate base cost, before per-pupil division.
    pub aggregate_base_cost: Dollars,
    /// Base cost per pupil, as published.
    pub base_cost_per_pupil: Dollars,
    /// Temporary transitional aid — the guarantee. Zero where the district is on the formula.
    pub guarantee: Dollars,
    /// Enrolled ADM, FY2024. See the module note on these three names.
    pub adm_fy24: f64,
    /// Enrolled ADM, FY2025.
    pub adm_fy25: f64,
    /// Enrolled ADM, FY2026.
    pub adm_fy26: f64,
    /// Assessed valuation per pupil, TY2023. Absent for three districts.
    pub valuation_per_pupil: Option<f64>,
    /// Core foundation funding — state aid as the formula computes it, before the guarantee.
    pub core_foundation: Dollars,
}

impl ModelDistrict {
    /// The grade-band enrolment this crate's build-up takes.
    ///
    /// `athletics_eligible` is true for every traditional district in this model, which is the
    /// population the fixture covers.
    #[must_use]
    pub fn enrollment(&self) -> DistrictEnrollment {
        DistrictEnrollment {
            kindergarten: self.kindergarten,
            grades_1_3: self.grades_1_3,
            grades_4_8: self.grades_4_8,
            grades_9_12: self.grades_9_12,
            career_technical: self.career_technical,
            grades_9_12_total: self.grades_9_12_total,
            base_cost_enrolled_adm: self.base_cost_adm,
            open_buildings: self.buildings,
            athletics_eligible: true,
        }
    }

    /// Funded teaching positions — classroom and special together.
    ///
    /// The quantity a salary refresh scales: see [`crate::teacher_salary_refresh_delta`].
    #[must_use]
    pub fn funded_positions(&self) -> f64 {
        self.funded_classroom_teachers + self.funded_special_teachers
    }

    /// Whether the guarantee, rather than the formula, is what determines this district's aid.
    #[must_use]
    pub fn on_guarantee(&self) -> bool {
        self.guarantee > 0.0
    }

    /// Enrolment change from FY2024 to FY2026, as a fraction.
    #[must_use]
    pub fn enrollment_change(&self) -> f64 {
        if self.adm_fy24 > 0.0 {
            self.adm_fy26 / self.adm_fy24 - 1.0
        } else {
            0.0
        }
    }

    /// State aid per pupil as the formula computes it, before the guarantee.
    #[must_use]
    pub fn formula_aid_per_pupil(&self) -> Dollars {
        self.per_pupil(self.core_foundation)
    }

    /// State aid per pupil as the district actually receives it.
    #[must_use]
    pub fn realized_aid_per_pupil(&self) -> Dollars {
        self.per_pupil(self.core_foundation + self.guarantee)
    }

    fn per_pupil(&self, dollars: Dollars) -> Dollars {
        if self.base_cost_adm > 0.0 {
            dollars / self.base_cost_adm
        } else {
            0.0
        }
    }
}

/// Every district in the department's model.
///
/// # Panics
///
/// If the fixture's header no longer begins with the columns this reader indexes, or a row's
/// width differs from the header's — both by way of [`edfund_core::csv::rows`], which holds the
/// uniform-width invariant these fixtures are written under.
#[must_use]
pub fn districts() -> Vec<ModelDistrict> {
    cached().clone()
}

/// The fixture, parsed once.
///
/// `OnceLock` for the reason `project::panel`'s reader has one: the parse is pure and the file
/// is compiled in, and a lookup helper that re-read it per call turned a scan over districts
/// into a quadratic one.
fn cached() -> &'static Vec<ModelDistrict> {
    static ROWS: OnceLock<Vec<ModelDistrict>> = OnceLock::new();
    ROWS.get_or_init(parse)
}

fn parse() -> Vec<ModelDistrict> {
    let header = FIXTURE.lines().next().unwrap_or_default().trim();
    assert!(
        header.starts_with(EXPECTED_PREFIX),
        "the department model's leading columns have moved; this reader indexes them by position"
    );
    edfund_core::csv::rows(FIXTURE, header)
        .filter_map(|row| {
            let irn = row.str(0);
            if irn.is_empty() {
                return None;
            }
            Some(ModelDistrict {
                irn: irn.to_string(),
                name: row.str(1).to_string(),
                // `required` rather than `num` throughout: every one of these columns is
                // populated on all 609 rows, which `every_column_this_reader_needs_is_present`
                // holds, and a guarantee of zero is a real zero rather than an absence.
                base_cost_adm: row.required(2),
                buildings: row.required(3),
                kindergarten: row.required(4),
                grades_1_3: row.required(5),
                grades_4_8: row.required(6),
                grades_9_12: row.required(7),
                career_technical: row.required(8),
                grades_9_12_total: row.required(9),
                funded_classroom_teachers: row.required(10),
                funded_special_teachers: row.required(11),
                teacher_base_cost: row.required(12),
                aggregate_base_cost: row.required(13),
                base_cost_per_pupil: row.required(14),
                guarantee: row.required(15),
                adm_fy24: row.required(16),
                adm_fy25: row.required(17),
                adm_fy26: row.required(18),
                // The one column the department leaves blank — three districts have no
                // published valuation per pupil, and reading that as zero would put them at the
                // poor end of every wealth measure.
                valuation_per_pupil: row.num(19),
                core_foundation: row.required(20),
            })
        })
        .collect()
}

/// The guarantee, keyed by IRN, for callers that need only that column.
///
/// The join `dispersion`'s report-card suite makes: splitting 607 rated districts on whether
/// the FY2027 model funds them off-formula.
#[must_use]
pub fn guarantees() -> std::collections::BTreeMap<String, Dollars> {
    cached()
        .iter()
        .map(|d| (d.irn.clone(), d.guarantee))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_model_covers_609_districts() {
        assert_eq!(districts().len(), 609);
    }

    /// Every column read as a bare number really is populated, which is what licenses
    /// `required` above rather than an `Option` per field.
    #[test]
    fn every_column_this_reader_needs_is_present() {
        let width = EXPECTED_PREFIX.split(',').count();
        for row in edfund_core::csv::rows(FIXTURE, FIXTURE.lines().next().unwrap_or_default()) {
            for index in 2..width {
                // Column 19 is the valuation, which the department does leave blank.
                if index == 19 {
                    continue;
                }
                assert!(
                    row.num(index).is_some(),
                    "column {index} is empty on {}",
                    row.str(0)
                );
            }
        }
    }

    /// Three districts have no published valuation per pupil, and it is absent rather than zero.
    #[test]
    fn the_valuation_column_is_the_only_one_with_absences() {
        let missing = districts()
            .iter()
            .filter(|d| d.valuation_per_pupil.is_none())
            .count();
        assert_eq!(missing, 3);
    }

    /// The grade bands sum to base cost enrolled ADM, which is the premise of every
    /// reconstruction in `tests/department_model_fy27.rs`.
    #[test]
    fn the_grade_bands_sum_to_base_cost_adm() {
        for d in districts() {
            let bands =
                d.kindergarten + d.grades_1_3 + d.grades_4_8 + d.grades_9_12 + d.career_technical;
            assert!(
                (bands - d.base_cost_adm).abs() < d.base_cost_adm * 0.001 + 0.01,
                "{}: bands sum to {bands} against an ADM of {}",
                d.name,
                d.base_cost_adm
            );
        }
    }
}
