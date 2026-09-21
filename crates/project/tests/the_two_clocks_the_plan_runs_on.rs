//! The Fair School Funding Plan prices what a school costs on data frozen by statute and what a
//! district can pay on a window that rolls forward every year, and the department's two published
//! models are one year apart, so the consequence is measurable rather than argued.
//!
//! Two corpus nodes recorded their versions of this as unanswerable "because the corpus holds one
//! year of the calculator". The cache held both workbooks the whole time.
//!
//! # And the General Assembly was shown it
//!
//! `fsfp-local-capacity-measure` left a second `[open]` beside the arithmetic: "Whether the
//! General Assembly intends the two clocks to run at different speeds, or has ever been shown this
//! comparison, is not established by anything here." It is established by LSC, in the document
//! legislators were given when the bill was introduced.
//!
//! H.B. 96's **redbook** carries a section headed *State share of the base cost, property values,
//! and local capacity*. It names both clocks — base cost inputs held "at FY 2022 levels", the
//! statewide weighted capacity per pupil rising 8.2% then 7.2% — states the consequence as a
//! series, **38.4% to 35.0% to 32.2%**, and names the knock-on onto the categoricals. LSC's
//! projection sits within a point of what this file computes from the department's own two
//! workbooks, which is a reconciliation of the finding against an independent estimate as well as
//! an answer about awareness.
//!
//! # The enacted analysis drops it
//!
//! That section is in the redbook and in no other document. H.B. 96's **greenbook** — the analysis
//! of the act as passed, and the one a later reader reaches for — keeps the phase-in table and one
//! clause of the Quick look, "while maintaining FY 2022 base cost inputs", and carries neither the
//! capacity growth rates nor the state share series nor the sentence joining them. So the record
//! of what the General Assembly was told is in the edition that describes a bill nobody enacted,
//! and the edition that describes the law is silent. That is [`Edition`] earning its keep a second
//! time: the two documents differ here by a whole finding, not by three words of column heading.
//!
//! **What this does not settle.** Awareness is not intent. LSC told the legislature what holding
//! the cost side still would do to the state share and the legislature enacted the freeze; that is
//! as far as the documents go, and no analysis says the divergence was the point.

mod common;

use project::greenbook::greenbook;
use project::ledger::budget_analysis::Edition;
use project::panel;
use project::prior_model::{self, Quantity};

#[test]
fn the_cost_side_holds_still_and_the_capacity_side_does_not() {
    // The whole finding in two numbers. Neither is a forecast: both are what the department
    // published, a year apart.
    let (_, cost) = prior_model::median_change(Quantity::BaseCostPerPupil);
    let (_, capacity) = prior_model::median_change(Quantity::CapacityPerPupil);
    assert!((cost - 0.000_504).abs() < 0.000_01, "{cost}");
    assert!((capacity - 0.093_390).abs() < 0.000_01, "{capacity}");
    assert!(
        capacity > cost * 100.0,
        "capacity {capacity} against base cost {cost}"
    );
}

#[test]
fn the_state_share_falls_and_it_is_the_capacity_side_that_does_it() {
    let (points, fell, of) = prior_model::state_share_fall();
    assert!((points - 0.041_125).abs() < 0.000_01, "{points}");
    assert_eq!((fell, of), (540, 609));

    // The counterfactual is the argument. Take FY2027's own base cost and pair it with FY2026's
    // capacity: the median state share comes back to within a sixth of a point of where FY2026
    // published it, so essentially none of the fall is the cost side.
    let (published, held) = prior_model::holding_capacity_still();
    assert!((published - 0.360_033).abs() < 0.000_01, "{published}");
    assert!((held - 0.412_331).abs() < 0.000_01, "{held}");

    let fy2026 = prior_model::pairs(Quantity::StateSharePercentage);
    let mut medians: Vec<f64> = fy2026.iter().map(|pair| pair.fy2026).collect();
    medians.sort_by(f64::total_cmp);
    let before = medians[medians.len() / 2];
    assert!(
        (held - before).abs() < 0.002,
        "holding capacity still gives {held}, FY2026 published {before}"
    );
}

#[test]
fn the_state_pays_for_less_of_the_cost_of_ohios_schools() {
    let (before, after) = prior_model::aggregate_state_share();
    assert!((before - 0.353_501).abs() < 0.000_01, "{before}");
    assert!((after - 0.315_237).abs() < 0.000_01, "{after}");
    assert!(before - after > 0.03, "{before} to {after}");
}

#[test]
fn more_districts_are_on_the_floor_and_the_committed_figure_is_the_later_one() {
    // The FY2027 count is the 138 `project/districts-at-the-minimum-state-share` already pins, so
    // the two years are on one rule and the earlier one is the news.
    let (before, after) = prior_model::at_the_floor();
    assert_eq!((before, after), (105, 138));
    assert_eq!(
        after,
        panel::panel()
            .iter()
            .filter(|record| record.at_minimum_state_share())
            .count(),
        "the published [b4] and the panel's derived share disagree about the floor"
    );
    assert_eq!(prior_model::fell_onto_the_floor().len(), 33);
}

#[test]
fn the_decision_in_the_capacity_rule_is_stable_and_what_it_multiplies_is_not() {
    // `fsfp-local-capacity-measure` framed its open question exactly this way: "a question about
    // a distribution rather than about a decision". The fortieth-highest district's income ratio
    // is the decision, and it moves a tenth of a per cent.
    let (before, after) = prior_model::benchmark_ratio();
    assert!((before - 1.466_709_42).abs() < 1e-8, "{before}");
    assert!((after - 1.465_036_36).abs() < 1e-8, "{after}");
    let ratio_move = (after - before) / before;
    assert!(ratio_move.abs() < 0.002, "{ratio_move}");

    let (_, rate) = prior_model::median_change(Quantity::LocalCapacityPercentage);
    assert!(rate.abs() < 0.002, "the rate moved {rate}");

    let (_, capacity) = prior_model::median_change(Quantity::CapacityPerPupil);
    assert!(
        capacity > ratio_move.abs() * 40.0 && capacity > rate.abs() * 40.0,
        "capacity {capacity}, ratio {ratio_move}, rate {rate}"
    );
}

#[test]
fn the_disadvantaged_blend_moved_and_the_counts_under_it_moved_further() {
    let ((before_ed, before_dc), (after_ed, after_dc)) = prior_model::dpia_blend();
    assert!((before_ed - 0.75).abs() < 1e-12 && (before_dc - 0.25).abs() < 1e-12);
    assert!((after_ed - 0.65).abs() < 1e-12 && (after_dc - 0.35).abs() < 1e-12);

    // The economically disadvantaged count is the same input in both workbooks, as both label it.
    let ed = prior_model::pairs(Quantity::EconomicallyDisadvantagedAdm);
    assert_eq!(ed.iter().filter(|pair| pair.unchanged()).count(), 593);

    // The directly certified count is not. Same label, same collection named on the `Directions`
    // sheet, and a median 16.8% lower a year later — so the weight moved toward a count that was
    // itself restated downward, and the two compound.
    let (_, dc) = prior_model::median_change(Quantity::DirectlyCertifiedAdm);
    assert!((dc + 0.167_989).abs() < 0.000_01, "{dc}");
    assert_eq!(
        prior_model::pairs(Quantity::DirectlyCertifiedAdm)
            .iter()
            .filter(|pair| pair.unchanged())
            .count(),
        0
    );

    let (_, weighted) = prior_model::median_change(Quantity::WeightedDisadvantagedAdm);
    assert!((weighted + 0.077_251).abs() < 0.000_01, "{weighted}");
}

#[test]
fn the_statewide_disadvantaged_share_moved_and_the_aid_fell_with_it() {
    // `fsfp-disadvantaged-pupil-impact-aid` establishes that R.C. 3317.02(I)(1)(a)(i) makes this
    // a computation rather than a constant, and recorded "how far it has moved between biennia is
    // still unrecorded here". Here.
    let (before, after) = prior_model::dpia_statewide_percentage();
    assert!((before - 0.565_990_245).abs() < 1e-9, "{before}");
    assert!((after - 0.533_380_310_606_710_3).abs() < 1e-12, "{after}");
    assert!(before - after > 0.03, "{before} to {after}");

    let (paid_before, paid_after) = prior_model::dpia_total();
    assert!((paid_before - 567_673_868.76).abs() < 1.0, "{paid_before}");
    assert!((paid_after - 525_094_312.31).abs() < 1.0, "{paid_after}");
    let fall = (paid_after - paid_before) / paid_before;
    assert!((fall + 0.075_007).abs() < 0.000_01, "{fall}");

    let (fell, of) = prior_model::fell(Quantity::DpiaAid);
    assert_eq!((fell, of), (562, 609));
}

#[test]
fn the_prior_model_covers_the_same_districts_the_panel_does() {
    // 611 rows against the panel's 609. The two extra are the Lake Erie island districts the
    // department computes no base cost for — the same pair `temporary-transitional-aid-guarantee`
    // records as absent from every committed panel.
    let prior = prior_model::frame();
    assert_eq!(prior.len(), 611);
    assert_eq!(prior_model::pairs(Quantity::CapacityPerPupil).len(), 609);

    let records = panel::panel();
    let panel: std::collections::BTreeSet<&str> = records.iter().map(|r| r.irn.as_str()).collect();
    let missing: Vec<&str> = prior
        .iter()
        .map(|row| row.name.as_str())
        .zip(prior.iter().map(|row| row.irn.as_str()))
        .filter(|(_, irn)| !panel.contains(irn))
        .map(|(name, _)| name)
        .collect();
    assert_eq!(missing.len(), 2, "{missing:?}");
    assert!(
        missing.iter().all(|name| name.contains("Bass")),
        "{missing:?}"
    );
}

#[test]
fn the_fy2026_blend_reproduces_its_own_published_count() {
    // The check that makes the 75/25 a reading rather than an assumption: it has to reproduce the
    // department's own `d1` from its own `d1a` and `d1b`, on every district.
    let (weights, _) = prior_model::dpia_blend();
    let mut worst: f64 = 0.0;
    for row in prior_model::frame() {
        let computed = weights.0 * row.dpia_econ_disadvantaged_adm
            + weights.1 * row.dpia_directly_certified_adm;
        worst = worst.max((computed - row.dpia_weighted_adm).abs());
    }
    assert!(worst < 0.01, "worst residual {worst}");
}

/// LSC published the series, and its cause, in the analysis of the bill as introduced.
///
/// Four figures and one sentence of mechanism. The mechanism is the corpus's own: capacity inputs
/// rise, cost inputs do not, and the difference between the two is the state's share.
#[test]
fn the_analysis_of_the_introduced_bill_states_the_falling_state_share_and_why() {
    let redbook = common::flat(Edition::Introduced.text());

    assert!(
        redbook.contains(
            "base cost inputs, primarily the salaries and related costs that drive most school \
             operating expenses, at FY 2022 levels"
        ),
        "the cost clock, stated as a freeze"
    );
    assert!(
        redbook.contains("the statewide weighted capacity per pupil")
            && redbook.contains("is projected to increase by 8.2% in FY 2026 and 7.2% in FY 2027"),
        "the capacity clock, stated as a rate of increase"
    );
    assert!(
        redbook.contains(
            "largely due to increases in property values and income levels used in computing \
             district per-pupil local contributions"
        ),
        "and the two joined: the state share falls because the capacity side moved"
    );
    assert!(
        redbook.contains(
            "averaging 35.0% statewide in FY 2026 and 32.2% in FY 2027, which are progressively \
             lower than the estimated statewide average of 38.4% for FY 2025"
        ),
        "with the consequence as a three-year series"
    );
    assert!(
        redbook.contains("This in turn reduces projected funding for the categorical components"),
        "and the knock-on onto every component the percentage multiplies"
    );
}

/// The series has a published start, in the analysis that enacted the plan.
///
/// 42.8% in FY2022 against 32.2% projected for FY2027. Both are LSC's, five bienniums apart, so
/// the decline is visible in the Legislative Service Commission's own numbers without this corpus
/// computing anything.
#[test]
fn the_enacting_analysis_published_the_percentage_the_series_starts_from() {
    assert!(
        greenbook("hb110").flat().contains(
            "The average state share percentage for all traditional districts is estimated to be \
             42.8% in FY 2022"
        ),
        "H.B. 110 states the statewide average state share percentage for the plan's first year"
    );
}

/// LSC's projection and the department's two workbooks agree to within a point.
///
/// Not an identity, and the test does not assert one. LSC projected the introduced bill before
/// either calculator existed and says "averaging ... statewide"; this crate divides the aggregate
/// state share by the aggregate base cost across 609 districts of the enacted models. Two methods,
/// two vintages, two editions of the bill — and 35.0% against 35.35%, 32.2% against 31.52%.
#[test]
fn the_redbooks_projection_and_the_published_models_are_within_a_point() {
    let (fy2026, fy2027) = prior_model::aggregate_state_share();

    assert!(
        (fy2026 - 0.350).abs() < 0.01,
        "FY2026 {fy2026} against LSC's 35.0%"
    );
    assert!(
        (fy2027 - 0.322).abs() < 0.01,
        "FY2027 {fy2027} against LSC's 32.2%"
    );

    // The fall over the same year is what has to agree, and it does to about a point: LSC
    // projected 2.8 points, the two published models give 3.8, and both are an order of magnitude
    // more than the 0.05% the cost side moves across the same interval.
    let theirs = 0.350 - 0.322;
    let ours = fy2026 - fy2027;
    assert!(ours > 0.02 && theirs > 0.02, "{ours} against {theirs}");
    assert!((ours - theirs).abs() < 0.011, "{ours} against {theirs}");

    let (_, cost) = prior_model::median_change(Quantity::BaseCostPerPupil);
    assert!(
        ours > cost * 50.0,
        "state share fell {ours}, base cost moved {cost}"
    );
}

/// The enacted analysis keeps the phase-in and drops the comparison.
///
/// One clause survives — the freeze, in the Quick look — and the capacity side, the percentages,
/// and the sentence joining them do not. A reader who goes to the greenbook because it is the
/// document about the law gets the phase-in table and no account of what runs against it.
#[test]
fn the_enacted_analysis_keeps_one_clock_and_drops_the_other() {
    let greenbook = common::flat(Edition::Enacted.text());

    assert!(
        greenbook.contains(
            "The budget completes the phase-in of the school funding formula, known as the Fair \
             School Funding Plan, in FY 2026 and FY 2027, while maintaining FY 2022 base cost \
             inputs"
        ),
        "the freeze survives into the enacted analysis, in one clause of the Quick look"
    );
    assert!(
        greenbook.contains("increasing the phase-in percentages to 83.33% and 100%, respectively"),
        "and so does the phase-in, with its own table"
    );

    for absent in [
        "statewide weighted capacity",
        "35.0%",
        "32.2%",
        "38.4",
        "property values and income",
    ] {
        assert!(
            !greenbook.contains(absent),
            "the enacted analysis carries {absent:?}, so the comparison did not drop out of it \
             and this file's claim needs rereading"
        );
    }
}
