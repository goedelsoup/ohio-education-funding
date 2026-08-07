//! Which districts appear in which published cross-section, and why the counts differ.
//!
//! This corpus holds three district panels and they do not agree on how many districts Ohio
//! has:
//!
//! | Panel | Districts | Source |
//! |---|---|---|
//! | FY2027 funding model | 609 | the department's foundation funding calculator |
//! | 2024-25 report card | 607 | achievement, growth, and need |
//! | FY2024 District Profile Report | 606 | valuation, millage, expenditure |
//!
//! Three counts coexisting with no crosswalk was carried as an open item through three phases.
//! It matters more than a tidiness complaint: every statement joining an input to an outcome is
//! computed on whichever intersection the author happened to take, and without a crosswalk the
//! reader cannot tell which one, or whether the districts that dropped out were a random three.
//!
//! # They are not random
//!
//! The whole discrepancy is **three districts, and they are the three smallest in Ohio** —
//! Kelleys Island Local at 3.5 pupils, Put-in-Bay Local at 63.3, and College Corner Local at
//! 114.4. The fourth-smallest district, Vanlue Local at 123.9, appears in all three panels.
//! Every other district in the state appears in all three, so the complete panel is **606**.
//!
//! That the exclusions track size is `[verified]` from the data. *Why* each publication draws its
//! line where it does is `[inference]`: neither the report card nor the profile report states a
//! minimum-size rule in the files this corpus holds, and Put-in-Bay's appearing in one but not
//! the other means there is more than one rule at work.
//!
//! # What that means for a joined result
//!
//! Dropping the three costs 0.03% of statewide enrolled ADM, so no aggregate moves. But the
//! three are exceptional in ways a small-district question would care about: Kelleys Island has
//! **no students in grades 9-12 at all** and a base cost of **$371,449 per pupil**, forty-five
//! times the statewide average, because a superintendent and a building do not get cheaper when
//! divided by 3.5 children. Any claim about fixed costs in small districts that is computed on
//! the complete panel has silently excluded its most extreme case.

use std::collections::BTreeSet;

/// The department's FY2027 funding model.
const FY27_MODEL: &str = include_str!("../../foundation/fixtures/fy27-department-model.csv");
/// The FY2024 District Profile Report.
const PROFILE: &str = include_str!("../../dispersion/fixtures/cupp-fy24-district-data.csv");
/// The 2024-25 Ohio School Report Card.
const REPORT_CARD: &str =
    include_str!("../../dispersion/fixtures/report-card-2425-district-data.csv");

/// A district that is missing from at least one panel, with what is known about why.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exception {
    /// Information Retrieval Number.
    pub irn: &'static str,
    /// District name.
    pub name: &'static str,
    /// What is distinctive about it. Size in every case, but not only size.
    pub note: &'static str,
}

/// Every district absent from at least one panel.
///
/// Pinned as a constant rather than derived, so that a fixture refresh which adds or removes a
/// district fails a test instead of quietly changing what "the complete panel" means.
pub const EXCEPTIONS: &[Exception] = &[
    Exception {
        irn: "046797",
        name: "Kelleys Island Local",
        note: "3.5 enrolled ADM, one building, no grades 9-12. The smallest district in Ohio \
               and the most extreme case of fixed cost per pupil in the state, at $371,449. \
               Absent from both the report card and the profile report.",
    },
    Exception {
        irn: "048975",
        name: "Put-In-Bay Local",
        note: "63.3 enrolled ADM on a Lake Erie island. Present in the report card and absent \
               from the profile report, which is why a single size threshold cannot be the \
               whole explanation. Its 77 weighted pupils are also where the report card's \
               whole-number ADM publishing produces the largest rounding error in the file.",
    },
    Exception {
        irn: "064964",
        name: "College Corner Local",
        note: "114.4 enrolled ADM, and an 87.0% state share of base cost — the third highest in \
               Ohio. Absent from both the report card and the profile report.",
    },
];

/// Which panels one district appears in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name, as the funding model publishes it.
    pub name: String,
    /// Present in the FY2027 funding model.
    pub funding_model: bool,
    /// Present in the FY2024 District Profile Report.
    pub profile_report: bool,
    /// Present in the 2024-25 report card.
    pub report_card: bool,
}

impl Coverage {
    /// Whether the district appears in all three panels.
    #[must_use]
    pub const fn complete(&self) -> bool {
        self.funding_model && self.profile_report && self.report_card
    }
}

/// How many districts each panel holds, and how many are in all of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counts {
    /// FY2027 funding model.
    pub funding_model: usize,
    /// FY2024 District Profile Report.
    pub profile_report: usize,
    /// 2024-25 report card.
    pub report_card: usize,
    /// Districts present in all three.
    pub complete: usize,
}

fn keys(csv: &str) -> BTreeSet<String> {
    csv.lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| line.split(',').next())
        .map(|irn| irn.trim().to_string())
        .filter(|irn| !irn.is_empty())
        .collect()
}

/// Coverage for every district in any panel, in IRN order.
///
/// The funding model is the spine: it is the only panel that covers every district Ohio funds,
/// and a district absent from it would be a district the state does not pay, which none of
/// these files contains.
#[must_use]
pub fn coverage() -> Vec<Coverage> {
    let model = keys(FY27_MODEL);
    let profile = keys(PROFILE);
    let report_card = keys(REPORT_CARD);

    let mut names = std::collections::BTreeMap::new();
    for line in FY27_MODEL.lines().skip(1) {
        let mut fields = line.split(',');
        if let (Some(irn), Some(name)) = (fields.next(), fields.next()) {
            names.insert(irn.trim().to_string(), name.trim().to_string());
        }
    }

    let mut all: BTreeSet<&String> = BTreeSet::new();
    all.extend(&model);
    all.extend(&profile);
    all.extend(&report_card);

    all.into_iter()
        .map(|irn| Coverage {
            irn: irn.clone(),
            name: names.get(irn).cloned().unwrap_or_default(),
            funding_model: model.contains(irn),
            profile_report: profile.contains(irn),
            report_card: report_card.contains(irn),
        })
        .collect()
}

/// The IRNs present in all three panels — the panel a joined statement is computed on.
#[must_use]
pub fn complete_panel() -> Vec<String> {
    coverage()
        .into_iter()
        .filter(Coverage::complete)
        .map(|c| c.irn)
        .collect()
}

/// Panel sizes and their intersection.
#[must_use]
pub fn counts() -> Counts {
    let coverage = coverage();
    Counts {
        funding_model: coverage.iter().filter(|c| c.funding_model).count(),
        profile_report: coverage.iter().filter(|c| c.profile_report).count(),
        report_card: coverage.iter().filter(|c| c.report_card).count(),
        complete: coverage.iter().filter(|c| c.complete()).count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_counts_are_the_ones_the_corpus_has_been_quoting() {
        let counts = counts();
        assert_eq!(counts.funding_model, 609);
        assert_eq!(counts.report_card, 607);
        assert_eq!(counts.profile_report, 606);
        assert_eq!(counts.complete, 606);
    }

    #[test]
    fn the_whole_discrepancy_is_the_three_pinned_districts() {
        let incomplete: Vec<String> = coverage()
            .into_iter()
            .filter(|c| !c.complete())
            .map(|c| c.irn)
            .collect();
        let expected: Vec<String> = EXCEPTIONS.iter().map(|e| e.irn.to_string()).collect();
        assert_eq!(incomplete, expected);
    }

    #[test]
    fn no_panel_holds_a_district_the_funding_model_does_not() {
        // The model is the spine. A district in the report card but not in the model would be
        // one Ohio reports on and does not fund, which would be a finding rather than a join
        // problem — so it is asserted rather than assumed.
        for entry in coverage() {
            assert!(
                entry.funding_model,
                "{} ({}) is outside the funding model",
                entry.name, entry.irn
            );
        }
    }

    #[test]
    fn put_in_bay_is_in_one_missing_panel_and_not_the_other() {
        // The fact that rules out a single size threshold: it is smaller than College Corner
        // and appears in a panel College Corner does not.
        let coverage = coverage();
        let find = |irn: &str| coverage.iter().find(|c| c.irn == irn).unwrap();
        let put_in_bay = find("048975");
        let college_corner = find("064964");
        assert!(put_in_bay.report_card && !put_in_bay.profile_report);
        assert!(!college_corner.report_card && !college_corner.profile_report);
    }

    #[test]
    fn the_exceptions_are_the_three_smallest_districts() {
        let mut sizes: Vec<(f64, String)> = crate::panel::panel()
            .into_iter()
            .map(|record| (record.current_year_adm, record.irn))
            .collect();
        sizes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let smallest: Vec<&String> = sizes.iter().take(3).map(|(_, irn)| irn).collect();
        for exception in EXCEPTIONS {
            assert!(
                smallest.iter().any(|irn| *irn == exception.irn),
                "{} is an exception but not among the three smallest",
                exception.name
            );
        }
        // And the fourth-smallest is not an exception, which is what makes it a size effect
        // rather than a coincidence of three small districts.
        let fourth = &sizes[3].1;
        assert!(!EXCEPTIONS.iter().any(|e| e.irn == *fourth));
    }

    #[test]
    fn dropping_the_exceptions_costs_almost_no_enrollment() {
        let panel = crate::panel::panel();
        let total: f64 = panel.iter().map(|r| r.current_year_adm).sum();
        let dropped: f64 = panel
            .iter()
            .filter(|r| EXCEPTIONS.iter().any(|e| e.irn == r.irn))
            .map(|r| r.current_year_adm)
            .sum();
        assert!(
            dropped / total < 0.0005,
            "the three cost {:.4}% of statewide ADM",
            100.0 * dropped / total
        );
    }

    #[test]
    fn kelleys_island_is_the_extreme_case_the_complete_panel_excludes() {
        let record = crate::panel::panel()
            .into_iter()
            .find(|r| r.irn == "046797")
            .expect("in the funding model");
        assert!(record.base_cost_per_pupil > 300_000.0);
        assert_eq!(
            record.enrollment.grades_9_12_total, 0.0,
            "no high school is part of why it is exceptional"
        );
    }
}
