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

use crate::outcomes::{PROFILE, PROFILE_HEADER, REPORT_CARD, REPORT_CARD_HEADER};
use crate::panel::panel;
use std::collections::{BTreeMap, BTreeSet};

/// The one column this module reads, in all three panels.
///
/// It can read only this one because all three identify a district the same way — IRN first,
/// name second — and that agreement is what makes a crosswalk computable at all. The name comes
/// from [`panel`] now, so only the key is read here; the tests below still assert both halves of
/// the agreement, because it is the second half that makes taking the name from one panel and
/// the keys from three a sound thing to do.
mod column {
    pub const IRN: usize = 0;
}

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

/// Every IRN in one panel.
///
/// The header is named at each call site rather than skipped. All three panels used to be read
/// with `.skip(1)` and their columns taken on faith, and the two ways that could go wrong are
/// not equally visible:
///
/// A column inserted *ahead* of the IRN collapses every panel to a single key, which the
/// pinned counts below would catch — loudly, though the message would be about 609 becoming 1
/// rather than about a moved column. A column inserted *between* the IRN and the name leaves
/// the keys correct and replaces all 609 district names with the new column's contents, and
/// nothing caught that at all: the full suite passed with every name in `coverage` reading
/// the same wrong string.
fn keys(csv: &str, header: &str) -> BTreeSet<String> {
    edfund_core::csv::rows(csv, header)
        .map(|row| row.str(column::IRN).to_string())
        .filter(|irn| !irn.is_empty())
        .collect()
}

/// Coverage for every district in any panel, in IRN order.
///
/// The funding model is the spine: it is the only panel that covers every district Ohio funds,
/// and a district absent from it would be a district the state does not pay, which none of
/// these files contains.
///
/// It is taken from [`panel`] rather than re-read here, which is not only a saved parse. This
/// module used to embed the model a second time and derive its own IRN set and name map from it,
/// so "the funding model" meant one thing in [`mod@crate::panel`] and a separately-computed thing
/// here, and the two agreed by coincidence rather than by construction — [`panel`] drops a row
/// with no base cost ADM and this did not.
#[must_use]
pub fn coverage() -> Vec<Coverage> {
    let model = panel();
    let names: BTreeMap<&str, &str> = model
        .iter()
        .map(|record| (record.irn.as_str(), record.name.as_str()))
        .collect();
    let profile = keys(PROFILE, PROFILE_HEADER);
    let report_card = keys(REPORT_CARD, REPORT_CARD_HEADER);

    let mut all: BTreeSet<&str> = names.keys().copied().collect();
    all.extend(profile.iter().map(String::as_str));
    all.extend(report_card.iter().map(String::as_str));

    all.into_iter()
        .map(|irn| Coverage {
            irn: irn.to_string(),
            name: (*names.get(irn).unwrap_or(&"")).to_string(),
            funding_model: names.contains_key(irn),
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
    use crate::panel::fixture::EXPECTED_HEADER as FY27_MODEL_HEADER;

    /// The invariant that lets one reader serve three files.
    ///
    /// `keys` takes column zero from each panel and [`panel`] takes column one from the model.
    /// If a panel ever renames or reorders those two, the header assertion catches it — but this
    /// states the shared shape directly, so the reason the three are readable by one function is
    /// written down rather than inferred from three long constants.
    #[test]
    fn all_three_panels_identify_a_district_the_same_way() {
        const NAME: usize = 1;

        for (panel, header) in [
            ("FY2027 funding model", FY27_MODEL_HEADER),
            ("FY2024 District Profile Report", PROFILE_HEADER),
            ("2024-25 report card", REPORT_CARD_HEADER),
        ] {
            let columns: Vec<&str> = header.split(',').collect();
            assert_eq!(
                columns[column::IRN],
                "irn",
                "{panel} column {}",
                column::IRN
            );
            assert_eq!(columns[NAME], "district", "{panel} column {NAME}");
        }
    }

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
