//! The 2024-25 Ohio School Report Card, one row per district.
//!
//! # Why the reader is here and not where it was
//!
//! The fixture is this crate's, and it had two readers: a public one in `project::outcomes`,
//! which joins the outcome side to the funding model, and a private one inside this crate's own
//! `tests/report_card_2425.rs`, which is where every coefficient the corpus quotes off this file
//! is computed. A third, also private, sat in `tests/expenditure_functions_fy25.rs`. They
//! indexed the same columns by hand and nothing related them. `project::outcomes` now re-exports
//! this module rather than restating it. See issue #157.
//!
//! # One numerator and two denominators, which is the whole subject
//!
//! The department publishes one FY2025 operating expenditure total per district and two pupil
//! counts to divide it by: [`ReportCard::unweighted_adm`], a headcount, and
//! [`ReportCard::weighted_adm`], the same count weighted upward for disadvantage, English
//! learners and disability. The published per-pupil figure uses the weighted one.
//!
//! The choice moves the headline result from nothing to something — against the Performance
//! Index, -0.015 on the published divisor and -0.337 on the headcount — because the weight
//! ratio is very nearly a poverty index, and dividing by it removes most of what the Performance
//! Index measures. Both are computed in `tests/report_card_2425.rs`; neither is the "right" one
//! without a stated question, and a figure quoted from here must name its divisor.
//!
//! # The two ADM columns are published at different precisions
//!
//! `unweighted_adm` carries four decimals and `weighted_adm` is rounded to whole pupils, so a
//! per-weighted-pupil figure recomputed for a small district carries quantisation noise the
//! per-headcount figure does not: Put-in-Bay Local, at 77 weighted pupils, reconstructs $86
//! below its published $46,716 while Akron City lands within a cent.

use std::sync::OnceLock;

use edfund_core::Dollars;

/// The committed report card.
pub const FIXTURE: &str = include_str!("../fixtures/report-card-2425-district-data.csv");

/// The header this reader was written against.
///
/// Asserted on every read. This file was indexed by bare position with nothing checking that
/// the positions still meant what they were written to mean.
pub const EXPECTED_HEADER: &str =
    "irn,district,performance_index_2425,performance_index_2324,performance_index_2223,\
unweighted_adm_fy25,weighted_adm_fy25,operating_expenditures_fy25,\
exp_per_equivalent_pupil_fy25,exp_per_equivalent_pupil_federal_fy25,\
exp_per_equivalent_pupil_state_local_fy25,progress_composite_2425,progress_effect_size_2425,\
progress_effect_size_1yr_2425,econ_disadvantaged_pct_2425,english_learner_pct_2425,\
students_with_disabilities_pct_2425";

/// What the report card publishes for one district.
#[derive(Debug, Clone, PartialEq)]
pub struct ReportCard {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's published name.
    pub name: String,
    /// Performance Index, 2024-25. Ohio's attainment-level measure.
    pub performance_index: Option<f64>,
    /// Performance Index, 2023-24.
    pub performance_index_prior: Option<f64>,
    /// Performance Index, 2022-23.
    pub performance_index_earliest: Option<f64>,
    /// Value-added composite, 2024-25.
    ///
    /// Scales with student count — a large district's composite reflects how many pupils were
    /// measured as well as how far they moved — so it does not compare districts.
    /// [`ReportCard::progress_effect_size`] is the one that does.
    pub progress_composite: Option<f64>,
    /// Value-added effect size, 2024-25 — already a three-year average as published.
    ///
    /// Ohio's growth measure, and it ranks districts differently enough from the Performance
    /// Index that an outcome-based adequacy standard has to choose one.
    pub progress_effect_size: Option<f64>,
    /// Value-added effect size over a single year.
    pub progress_effect_size_one_year: Option<f64>,
    /// Enrolled headcount, FY2025. Published to four decimals.
    pub unweighted_adm: Option<f64>,
    /// Pupil count weighted upward for disadvantage, English learners, and disability.
    /// Published as a whole number, which is where small districts pick up rounding error.
    pub weighted_adm: Option<f64>,
    /// Total operating expenditure, FY2025.
    pub operating_expenditure: Option<Dollars>,
    /// Operating expenditure per *weighted* pupil — the department's published per-pupil figure.
    pub per_equivalent_pupil: Option<Dollars>,
    /// The federal part of [`ReportCard::per_equivalent_pupil`].
    ///
    /// The report card splits its per-pupil spending figure by the origin of the money, and the
    /// two parts sum to the whole for all 607 districts carrying it. The federal share runs from
    /// 0.7% to 29.0%, which makes it the only field here that says how exposed a district is to
    /// a decision taken outside Ohio.
    pub per_equivalent_pupil_federal: Option<Dollars>,
    /// The state and local part. Adds to `per_equivalent_pupil_federal` to give the whole.
    pub per_equivalent_pupil_state_local: Option<Dollars>,
    /// Economically disadvantaged share, 2024-25, as a **percentage**.
    ///
    /// Top-coded by community eligibility, so a district where every student is certified reads
    /// 100 whatever its actual composition — 87 of them do here, against 37 at the ceiling of
    /// the profile report's own measure. See [`crate::profile`], which carries that one.
    pub economically_disadvantaged: Option<f64>,
    /// English learner share, 2024-25, as a percentage. Absent where the department suppressed
    /// a count under ten.
    pub english_learner: Option<f64>,
    /// Students with disabilities share, 2024-25, as a percentage.
    pub students_with_disabilities: Option<f64>,
}

impl ReportCard {
    /// Operating expenditure per *unweighted* pupil.
    ///
    /// The divisor the corpus computes on when it wants a spending measure rather than a
    /// composition proxy. Dividing by the weighted count builds need into the denominator, and
    /// against a composition-driven outcome that is most of what the resulting number measures.
    #[must_use]
    pub fn per_enrolled_pupil(&self) -> Option<Dollars> {
        self.per_pupil_on(self.unweighted_adm)
    }

    /// Operating expenditure per *weighted* pupil, recomputed rather than read.
    ///
    /// Reconstructs [`ReportCard::per_equivalent_pupil`] up to the whole-pupil rounding of the
    /// denominator, which is the check `tests/report_card_2425.rs` makes on all 607 rows.
    #[must_use]
    pub fn per_weighted_pupil(&self) -> Option<Dollars> {
        self.per_pupil_on(self.weighted_adm)
    }

    /// The ratio of the weighted count to the headcount.
    ///
    /// Never below one — the weighting only ever adds pupils — and correlated at 0.80 with the
    /// profile report's disadvantage share, which is why it behaves as a poverty index when it
    /// is used as a denominator.
    #[must_use]
    pub fn weight_ratio(&self) -> Option<f64> {
        match (self.weighted_adm, self.unweighted_adm) {
            (Some(weighted), Some(headcount)) if headcount > 0.0 => Some(weighted / headcount),
            _ => None,
        }
    }

    fn per_pupil_on(&self, pupils: Option<f64>) -> Option<Dollars> {
        match (self.operating_expenditure, pupils) {
            (Some(total), Some(pupils)) if pupils > 0.0 => Some(total / pupils),
            _ => None,
        }
    }
}

/// Column positions in the fixture.
mod column {
    pub const IRN: usize = 0;
    pub const NAME: usize = 1;
    pub const PERFORMANCE_INDEX: usize = 2;
    pub const PERFORMANCE_INDEX_PRIOR: usize = 3;
    pub const PERFORMANCE_INDEX_EARLIEST: usize = 4;
    pub const UNWEIGHTED_ADM: usize = 5;
    pub const WEIGHTED_ADM: usize = 6;
    pub const OPERATING_EXPENDITURE: usize = 7;
    pub const PER_EQUIVALENT_PUPIL: usize = 8;
    pub const PER_EQUIVALENT_PUPIL_FEDERAL: usize = 9;
    pub const PER_EQUIVALENT_PUPIL_STATE_LOCAL: usize = 10;
    pub const PROGRESS_COMPOSITE: usize = 11;
    pub const PROGRESS_EFFECT_SIZE: usize = 12;
    pub const PROGRESS_EFFECT_SIZE_ONE_YEAR: usize = 13;
    pub const ECON_DISADVANTAGED: usize = 14;
    pub const ENGLISH_LEARNER: usize = 15;
    pub const STUDENTS_WITH_DISABILITIES: usize = 16;
}

/// Every district the report card covers.
///
/// # Panics
///
/// If the fixture's header is not [`EXPECTED_HEADER`], or a row's width differs from it — both
/// by way of [`edfund_core::csv::rows`].
#[must_use]
pub fn report_cards() -> Vec<ReportCard> {
    cached().clone()
}

/// The fixture, parsed once.
///
/// `OnceLock` for the reason `project::panel`'s reader has one: the parse is pure and
/// the file is compiled in, and a lookup helper that re-read it per call turned a scan
/// over districts into a quadratic one.
fn cached() -> &'static Vec<ReportCard> {
    static ROWS: OnceLock<Vec<ReportCard>> = OnceLock::new();
    ROWS.get_or_init(parse)
}

fn parse() -> Vec<ReportCard> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            let irn = row.str(column::IRN);
            if irn.is_empty() {
                return None;
            }
            Some(ReportCard {
                irn: irn.to_string(),
                name: row.str(column::NAME).to_string(),
                performance_index: row.num(column::PERFORMANCE_INDEX),
                performance_index_prior: row.num(column::PERFORMANCE_INDEX_PRIOR),
                performance_index_earliest: row.num(column::PERFORMANCE_INDEX_EARLIEST),
                progress_composite: row.num(column::PROGRESS_COMPOSITE),
                progress_effect_size: row.num(column::PROGRESS_EFFECT_SIZE),
                progress_effect_size_one_year: row.num(column::PROGRESS_EFFECT_SIZE_ONE_YEAR),
                unweighted_adm: row.num(column::UNWEIGHTED_ADM),
                weighted_adm: row.num(column::WEIGHTED_ADM),
                operating_expenditure: row.num(column::OPERATING_EXPENDITURE),
                per_equivalent_pupil: row.num(column::PER_EQUIVALENT_PUPIL),
                per_equivalent_pupil_federal: row.num(column::PER_EQUIVALENT_PUPIL_FEDERAL),
                per_equivalent_pupil_state_local: row.num(column::PER_EQUIVALENT_PUPIL_STATE_LOCAL),
                economically_disadvantaged: row.num(column::ECON_DISADVANTAGED),
                english_learner: row.num(column::ENGLISH_LEARNER),
                students_with_disabilities: row.num(column::STUDENTS_WITH_DISABILITIES),
            })
        })
        .collect()
}

/// The district with this IRN, if the report card rates it.
#[must_use]
pub fn district(irn: &str) -> Option<ReportCard> {
    cached().iter().find(|card| card.irn == irn).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rated_population_is_607_districts() {
        let cards = report_cards();
        assert_eq!(cards.len(), 607);
        assert!(cards.iter().all(|c| c.performance_index.is_some()));
    }

    /// The weighting only ever adds pupils, which is what makes the ratio readable as a need
    /// index rather than as a correction of either sign.
    #[test]
    fn the_weighted_count_never_falls_below_the_headcount() {
        for card in report_cards() {
            if let Some(ratio) = card.weight_ratio() {
                assert!(ratio >= 1.0, "{}: weight ratio {ratio}", card.name);
            }
        }
    }

    /// The published per-pupil figure is the weighted one, reconstructible up to the whole-pupil
    /// rounding of its denominator. This is what licenses recomputing it on the other divisor.
    #[test]
    fn the_published_figure_divides_by_the_weighted_count() {
        let mut checked = 0;
        for card in report_cards() {
            let (Some(computed), Some(published), Some(pupils)) = (
                card.per_weighted_pupil(),
                card.per_equivalent_pupil,
                card.weighted_adm,
            ) else {
                continue;
            };
            // Half a pupil of rounding in the denominator, plus a dollar for the published
            // figure's own rounding.
            let tolerance = published * (0.5 / pupils) + 1.0;
            assert!(
                (computed - published).abs() < tolerance,
                "{} reconstructs to {computed:.2} against a published {published:.0}",
                card.name
            );
            checked += 1;
        }
        assert_eq!(checked, 607);
    }

    /// This share is a percentage. [`crate::profile`]'s is a fraction, and the two are different
    /// variables besides.
    #[test]
    fn the_disadvantage_share_is_a_percentage() {
        let shares: Vec<f64> = report_cards()
            .iter()
            .filter_map(|c| c.economically_disadvantaged)
            .collect();
        assert!(shares.iter().any(|s| *s > 1.0));
        assert!(shares.iter().all(|s| (0.0..=100.0).contains(s)));
    }
}
