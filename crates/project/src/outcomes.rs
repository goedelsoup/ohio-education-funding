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
//! [`ReportCard::economically_disadvantaged`] is the report card's;
//! [`dispersion::profile::ProfileDistrict::economically_disadvantaged`] is the profile report's,
//! and [`Joined::economically_disadvantaged`] carries that one. Callers controlling for poverty
//! should say which they used.

use std::collections::BTreeMap;

use edfund_core::Dollars;

use crate::panel::{panel, DistrictRecord};

/// The 2024-25 Ohio School Report Card, and the FY2024 District Profile Report.
///
/// Re-exported rather than re-read. Both fixtures are `dispersion`'s and their readers are now
/// [`dispersion::report_card`] and [`dispersion::profile`]; this module carried a second reader
/// of each, and `dispersion`'s own tests carried a third. Nothing checked that any two of them
/// agreed about what a row is. See issue #157.
pub use dispersion::report_card::{report_cards, ReportCard};

pub(crate) use dispersion::profile::{EXPECTED_HEADER as PROFILE_HEADER, FIXTURE as PROFILE};
pub(crate) use dispersion::report_card::{
    EXPECTED_HEADER as REPORT_CARD_HEADER, FIXTURE as REPORT_CARD,
};

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
    // Keyed rather than scanned. This was two linear searches per district — 609 records against
    // 607 report cards and 606 profile rows, ~740,000 string comparisons for a join of 606 — in a
    // crate that reaches for `BTreeMap` for exactly this shape everywhere else.
    let report_cards: BTreeMap<String, ReportCard> = report_cards()
        .into_iter()
        .map(|card| (card.irn.clone(), card))
        .collect();
    let poverty: BTreeMap<String, Option<f64>> = dispersion::profile::districts()
        .into_iter()
        .map(|district| (district.irn, district.economically_disadvantaged))
        .collect();

    panel()
        .into_iter()
        .filter_map(|funding| {
            let outcome = report_cards.get(&funding.irn)?.clone();
            let economically_disadvantaged = poverty.get(&funding.irn).copied().flatten();
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
