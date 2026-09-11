//! The Fair School Funding Plan prices what a school costs on data frozen by statute and what a
//! district can pay on a window that rolls forward every year, and the department's two published
//! models are one year apart, so the consequence is measurable rather than argued.
//!
//! Two corpus nodes recorded their versions of this as unanswerable "because the corpus holds one
//! year of the calculator". The cache held both workbooks the whole time.

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
