//! The outcome side of a district, joined to the funding side.
//!
//! For most of this repository's life every metric it held was an input: expenditure,
//! valuation, millage, state share. The outcome axis arrived with the 2024-25 report card, and
//! until now it lived only in one crate's integration test — which meant the corpus could
//! compute achievement against spending, and could not compute achievement against *anything in
//! the funding model*, because the two panels were never joined in code.
//!
//! [`joined`] is that join. It is what makes the question this corpus is uniquely placed to ask
//! computable at all: **do the districts the guarantee protects do better?**
//!
//! # Two poverty measures, and they are not the same variable
//!
//! Both are carried here on purpose. The profile report publishes an economically-disadvantaged
//! headcount share for FY2024; the report card publishes one for 2024-25 that is **top-coded by
//! community eligibility**, so a district where every student is certified reads 100% whatever
//! its actual composition. Against the Performance Index the first gives −0.846 and the second
//! −0.734. Swapping one for the other while calling it a vintage correction would weaken a
//! corpus finding by a tenth and look like tidying.
//!
//! [`ReportCard::economically_disadvantaged`] is the report card's. The profile report's stays
//! where it is, in [`mod@crate::panel`]'s sibling fixture, and callers controlling for poverty
//! should say which one they used.

use edfund_core::Dollars;

use crate::panel::{panel, DistrictRecord};

/// The 2024-25 Ohio School Report Card, one row per district.
const REPORT_CARD: &str =
    include_str!("../../dispersion/fixtures/report-card-2425-district-data.csv");
/// The FY2024 District Profile Report, for the corpus's primary poverty measure.
const PROFILE: &str = include_str!("../../dispersion/fixtures/cupp-fy24-district-data.csv");

/// What the report card publishes for one district.
#[derive(Debug, Clone, PartialEq)]
pub struct ReportCard {
    /// Information Retrieval Number.
    pub irn: String,
    /// Performance Index, 2024-25. Ohio's attainment-level measure.
    pub performance_index: Option<f64>,
    /// Performance Index, 2023-24.
    pub performance_index_prior: Option<f64>,
    /// Performance Index, 2022-23.
    pub performance_index_earliest: Option<f64>,
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
    /// Economically disadvantaged share, 2024-25. Top-coded; see the module note.
    pub economically_disadvantaged: Option<f64>,
    /// English learner share, 2024-25.
    pub english_learner: Option<f64>,
    /// Students with disabilities share, 2024-25.
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
        match (self.operating_expenditure, self.unweighted_adm) {
            (Some(total), Some(pupils)) if pupils > 0.0 => Some(total / pupils),
            _ => None,
        }
    }
}

/// Column positions in the report-card fixture.
mod column {
    pub const IRN: usize = 0;
    pub const PERFORMANCE_INDEX: usize = 2;
    pub const PERFORMANCE_INDEX_PRIOR: usize = 3;
    pub const PERFORMANCE_INDEX_EARLIEST: usize = 4;
    pub const UNWEIGHTED_ADM: usize = 5;
    pub const WEIGHTED_ADM: usize = 6;
    pub const OPERATING_EXPENDITURE: usize = 7;
    pub const PER_EQUIVALENT_PUPIL: usize = 8;
    pub const PER_EQUIVALENT_PUPIL_FEDERAL: usize = 9;
    pub const PER_EQUIVALENT_PUPIL_STATE_LOCAL: usize = 10;
    pub const PROGRESS_EFFECT_SIZE: usize = 12;
    pub const PROGRESS_EFFECT_SIZE_ONE_YEAR: usize = 13;
    pub const ECON_DISADVANTAGED: usize = 14;
    pub const ENGLISH_LEARNER: usize = 15;
    pub const STUDENTS_WITH_DISABILITIES: usize = 16;
}

/// Column positions in the profile-report fixture.
mod profile_column {
    pub const IRN: usize = 0;
    pub const ECON_DISADVANTAGED: usize = 3;
}

/// The header `column` is written against. Asserted on every read: this module indexed the
/// report card by bare position with nothing checking that the positions still meant what
/// they were written to mean.
const REPORT_CARD_HEADER: &str =
    "irn,district,performance_index_2425,performance_index_2324,performance_index_2223,\
unweighted_adm_fy25,weighted_adm_fy25,operating_expenditures_fy25,\
exp_per_equivalent_pupil_fy25,exp_per_equivalent_pupil_federal_fy25,\
exp_per_equivalent_pupil_state_local_fy25,progress_composite_2425,progress_effect_size_2425,\
progress_effect_size_1yr_2425,econ_disadvantaged_pct_2425,english_learner_pct_2425,\
students_with_disabilities_pct_2425";

/// The header `profile_column` is written against, asserted for the same reason.
const PROFILE_HEADER: &str = "irn,district,enrolled_adm_fy24,econ_disadvantaged_pct_fy24,\
assessed_valuation_per_pupil_fy23,current_operating_millage_ty23,\
effective_class1_millage_ty23,operating_expenditure_per_pupil_fy24,\
state_revenue_per_pupil_fy24,local_revenue_per_pupil_fy24";

/// Every district the report card covers.
#[must_use]
pub fn report_cards() -> Vec<ReportCard> {
    edfund_core::csv::rows(REPORT_CARD, REPORT_CARD_HEADER)
        .filter_map(|row| {
            let irn = row.str(column::IRN);
            if irn.is_empty() {
                return None;
            }
            Some(ReportCard {
                irn: irn.to_string(),
                performance_index: row.num(column::PERFORMANCE_INDEX),
                performance_index_prior: row.num(column::PERFORMANCE_INDEX_PRIOR),
                performance_index_earliest: row.num(column::PERFORMANCE_INDEX_EARLIEST),
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

/// A district with its funding and its outcomes on the same record.
#[derive(Debug, Clone, PartialEq)]
pub struct Joined {
    /// The funding side: base cost, state share, guarantee, enrollment.
    pub funding: DistrictRecord,
    /// The outcome side: achievement, growth, need.
    pub outcome: ReportCard,
    /// Economically disadvantaged share from the **profile report**, FY2024.
    ///
    /// The corpus's primary poverty measure, and distinct from
    /// [`ReportCard::economically_disadvantaged`]. See the module note.
    pub economically_disadvantaged: Option<f64>,
}

impl Joined {
    /// Whether the guarantee is what determines this district's aid.
    #[must_use]
    pub fn on_guarantee(&self) -> bool {
        self.funding.on_guarantee()
    }

    /// The guarantee as a share of everything the district receives from the state.
    ///
    /// A continuous reading of guarantee dependence, where [`Joined::on_guarantee`] is the
    /// binary one. Both are reported because they can disagree: a district can be barely on the
    /// guarantee or almost entirely funded by it.
    #[must_use]
    pub fn guarantee_share(&self) -> f64 {
        let realized = self.funding.realized_aid();
        if realized <= 0.0 {
            return 0.0;
        }
        self.funding.guarantee / realized
    }

    /// Realized state aid per current-year pupil.
    #[must_use]
    pub fn realized_aid_per_pupil(&self) -> Dollars {
        if self.funding.current_year_adm <= 0.0 {
            return 0.0;
        }
        self.funding.realized_aid() / self.funding.current_year_adm
    }
}

/// The funding model joined to the report card, on the districts both cover.
///
/// This is the 606 of [`crate::crosswalk`] — every district in all three panels. The three it
/// omits and why are pinned there rather than left to the reader to notice.
#[must_use]
pub fn joined() -> Vec<Joined> {
    let report_cards = report_cards();
    let poverty: Vec<(&str, Option<f64>)> = edfund_core::csv::rows(PROFILE, PROFILE_HEADER)
        .map(|row| {
            (
                row.str(profile_column::IRN),
                row.num(profile_column::ECON_DISADVANTAGED),
            )
        })
        .collect();

    panel()
        .into_iter()
        .filter_map(|funding| {
            let outcome = report_cards
                .iter()
                .find(|card| card.irn == funding.irn)?
                .clone();
            let economically_disadvantaged = poverty
                .iter()
                .find(|(irn, _)| *irn == funding.irn)
                .and_then(|(_, share)| *share);
            // Present in the report card but not the profile report is Put-in-Bay alone, and a
            // joined record with no primary poverty measure cannot be controlled. Excluding it
            // here keeps `joined()` and `crosswalk::complete_panel()` the same set of districts.
            economically_disadvantaged?;
            Some(Joined {
                funding,
                outcome,
                economically_disadvantaged,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crosswalk;

    #[test]
    fn the_report_card_covers_the_districts_the_crosswalk_says_it_does() {
        assert_eq!(report_cards().len(), crosswalk::counts().report_card);
    }

    #[test]
    fn the_join_is_the_complete_panel_and_nothing_else() {
        let joined = joined();
        assert_eq!(joined.len(), 606);
        assert_eq!(joined.len(), crosswalk::complete_panel().len());
        let joined_irns: Vec<&String> = joined.iter().map(|j| &j.funding.irn).collect();
        for irn in crosswalk::complete_panel() {
            assert!(joined_irns.contains(&&irn), "{irn} missing from the join");
        }
    }

    #[test]
    fn the_two_poverty_measures_are_carried_separately_and_differ() {
        // If these were ever collapsed into one field the corpus's -0.846 would quietly become
        // -0.734. They are on the record as different variables.
        let joined = joined();
        let disagreements = joined
            .iter()
            .filter(|j| {
                match (
                    j.economically_disadvantaged,
                    j.outcome.economically_disadvantaged,
                ) {
                    // One is a fraction, the other a percentage; compare on a common scale.
                    (Some(profile), Some(card)) => (profile * 100.0 - card).abs() > 5.0,
                    _ => false,
                }
            })
            .count();
        assert!(
            disagreements > 100,
            "only {disagreements} districts differ between the two poverty measures"
        );
    }

    #[test]
    fn the_report_cards_share_is_top_coded_and_the_profile_reports_is_not() {
        let joined = joined();
        let card_at_hundred = joined
            .iter()
            .filter(|j| j.outcome.economically_disadvantaged == Some(100.0))
            .count();
        let profile_at_one = joined
            .iter()
            .filter(|j| j.economically_disadvantaged == Some(1.0))
            .count();
        assert!(
            card_at_hundred > profile_at_one,
            "top-coding should pile districts at the ceiling: {card_at_hundred} vs {profile_at_one}"
        );
    }

    #[test]
    fn spending_per_enrolled_pupil_is_recoverable_and_lower_than_per_weighted_pupil() {
        // The weighted count is the enrolled count scaled up, so dividing by it always gives
        // the smaller number. Getting this backwards would invert the denominator finding.
        let joined = joined();
        let mut checked = 0;
        for record in &joined {
            let (Some(unweighted), Some(weighted)) = (
                record.outcome.per_enrolled_pupil(),
                record.outcome.per_equivalent_pupil,
            ) else {
                continue;
            };
            assert!(
                unweighted >= weighted - 0.01,
                "{}: per-enrolled {unweighted:.2} below per-weighted {weighted:.2}",
                record.funding.name
            );
            checked += 1;
        }
        assert!(checked > 600);
    }

    #[test]
    fn guarantee_share_is_zero_off_the_guarantee_and_bounded_on_it() {
        for record in joined() {
            let share = record.guarantee_share();
            if record.on_guarantee() {
                assert!(
                    share > 0.0 && share < 1.0,
                    "{}: {share}",
                    record.funding.name
                );
            } else {
                assert_eq!(share, 0.0, "{}", record.funding.name);
            }
        }
    }

    #[test]
    fn the_performance_index_series_is_present_for_three_years() {
        let joined = joined();
        for field in [
            "performance_index",
            "performance_index_prior",
            "performance_index_earliest",
        ] {
            let present = joined
                .iter()
                .filter(|j| match field {
                    "performance_index" => j.outcome.performance_index.is_some(),
                    "performance_index_prior" => j.outcome.performance_index_prior.is_some(),
                    _ => j.outcome.performance_index_earliest.is_some(),
                })
                .count();
            assert!(present > 590, "{field}: only {present} districts");
        }
    }
}
