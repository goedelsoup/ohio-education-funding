//! What changed when Ohio stopped charging districts on property alone.
//!
//! The corpus aligns exactly one component pair across the Fair School Funding Plan and its
//! predecessor: local capacity replaces the charge-off local share. This is that comparison, run
//! at FY2027 inputs with the plan's own computed base cost held fixed, so the only thing that
//! moves is how much of that cost the district is deemed able to bear.
//!
//! # What the residual does and does not verify here
//!
//! The residual comes out **exactly zero for 463 of the 470 districts where both sides can be
//! valued**, and that is a check on the substitution rather than a discovery about the regimes.
//! Holding base cost fixed means the local share is the only thing that can differ, so a nonzero
//! residual would mean the arithmetic had gone wrong somewhere. It did not.
//!
//! A true Bridge-formula-against-FSFP diff would have an enormous residual, because the corpus
//! has no Bridge components at all. Nothing here should be read as showing that the two regimes
//! differ only in this component.

use project::panel::{panel, DistrictRecord};
use regime_diff::{at_fy2027, panel_at_fy2027, RegimeDiff, ALIGNED, TERMINAL_MILLS, UNALIGNED};

fn find<'a>(districts: &'a [DistrictRecord], name: &str) -> &'a DistrictRecord {
    districts
        .iter()
        .find(|r| r.name == name)
        .expect("in the FY2027 model")
}

/// The alignment registry matches the corpus's `replaces` graph, which is one edge.
#[test]
fn the_corpus_supports_exactly_one_aligned_component_and_says_why_for_the_rest() {
    assert_eq!(ALIGNED.len(), 1);
    assert_eq!(ALIGNED[0].predecessor, "charge-off-local-share");
    assert_eq!(ALIGNED[0].successor, "fsfp-local-capacity-measure");
    // Base cost and the guarantee are named as unalignable rather than silently omitted.
    let unaligned: Vec<&str> = UNALIGNED.iter().map(|u| u.component).collect();
    assert!(unaligned.contains(&"fsfp-base-cost-calculation"));
    assert!(unaligned.contains(&"temporary-transitional-aid-guarantee"));
}

/// Coverage, stated before any finding rests on it.
#[test]
fn the_comparison_reaches_470_districts_and_the_reasons_it_misses_the_rest_are_distinct() {
    let districts = panel();
    let diffs = panel_at_fy2027(&districts, TERMINAL_MILLS);
    assert_eq!(diffs.len(), 609);

    let complete = diffs.iter().filter(|d| d.is_complete()).count();
    let no_valuation = diffs
        .iter()
        .filter(|d| d.components[0].predecessor.is_none())
        .count();
    let censored = diffs
        .iter()
        .filter(|d| d.components[0].successor.is_none())
        .count();

    assert_eq!(complete, 470);
    // Two different failures. Three districts are missing from the profile report entirely; 138
    // are present and have their local capacity censored by the minimum state share, which is
    // not a gap in the data but a consequence of the mechanism being compared.
    assert_eq!(no_valuation, 3);
    assert_eq!(censored, 138);
    assert_eq!(
        censored,
        districts
            .iter()
            .filter(|r| r.at_minimum_state_share())
            .count()
    );

    // The totals survive the censoring even where the component row does not, so the difference
    // stays visible for 606 districts while its attribution does not.
    let totals = diffs
        .iter()
        .filter(|d| d.total_difference().is_some())
        .count();
    assert_eq!(totals, 606);
    let attributable = diffs.iter().filter(|d| d.residual().is_some()).count();
    assert_eq!(attributable, 470);
}

/// The one aligned component explains the whole difference wherever neither regime's floor binds.
#[test]
fn the_residual_is_zero_except_where_the_charge_off_runs_past_the_cost_it_is_subtracted_from() {
    let districts = panel();
    let diffs = panel_at_fy2027(&districts, TERMINAL_MILLS);

    let mut exact = 0;
    let mut truncated = 0;
    for (record, diff) in districts.iter().zip(&diffs) {
        let Some(residual) = diff.residual() else {
            continue;
        };
        if residual.abs() <= 0.005 {
            exact += 1;
            continue;
        }
        truncated += 1;

        // The charge-off had no minimum state share, so a district whose deemed local share
        // exceeded its cost received nothing rather than a negative amount. The residual is
        // exactly that truncation, and it is negative because the truncation is what stopped the
        // old mechanism taking more.
        let charge_off = diff.components[0].predecessor.expect("valued");
        let overshoot = charge_off - record.base_cost_per_pupil;
        assert!(overshoot > 0.0, "{}", record.name);
        assert!(
            (residual + overshoot).abs() < 0.01,
            "{}: residual {residual:.2} against an overshoot of {overshoot:.2}",
            record.name
        );
    }
    assert_eq!(exact, 463);
    assert_eq!(truncated, 7);
}

/// **The reform's stated purpose, measured.** The charge-off was blind to income; local capacity
/// is 40% income. A district whose wealth is in its earnings rather than its property was
/// treated as middling by the old mechanism and is not by the new one.
#[test]
fn a_high_income_district_pays_far_more_under_local_capacity_than_the_charge_off_asked() {
    let districts = panel();
    let ottawa_hills = find(&districts, "Ottawa Hills Local");
    let diff = at_fy2027(ottawa_hills, TERMINAL_MILLS);

    // Its valuation per pupil is unremarkable — below the statewide 40th percentile — so a
    // property-only measure sees an ordinary district.
    let valuation = ottawa_hills
        .valuation_per_pupil
        .expect("in the profile report");
    assert!(
        (valuation - 176_195.0).abs() < 1.0,
        "valuation per pupil {valuation:.0}"
    );

    let charge_off = diff.components[0].predecessor.expect("valued");
    let capacity = diff.components[0].successor.expect("not at the floor");
    assert!((charge_off - 4_052.49).abs() < 0.5);
    assert!((capacity - 6_845.07).abs() < 0.5);
    assert!(
        capacity > charge_off * 1.65,
        "local capacity should charge it far more: {capacity:.2} against {charge_off:.2}"
    );

    // Which costs it two thirds of its base cost aid.
    assert!((diff.total_difference().expect("both sides") + 2_792.58).abs() < 0.5);
}

/// And the blend runs the other way too, which is the half of the reform that gets less
/// attention: a district rich in property and poor in income is charged **less**.
#[test]
fn a_property_rich_low_income_district_pays_less_under_local_capacity() {
    let districts = panel();
    let jefferson = find(&districts, "Jefferson Township Local");
    let diff = at_fy2027(jefferson, TERMINAL_MILLS);

    let valuation = jefferson
        .valuation_per_pupil
        .expect("in the profile report");
    assert!(valuation > 500_000.0, "valuation per pupil {valuation:.0}");

    let charge_off = diff.components[0].predecessor.expect("valued");
    let capacity = diff.components[0].successor.expect("not at the floor");
    assert!((charge_off - 11_807.96).abs() < 0.5);
    assert!((capacity - 8_714.77).abs() < 0.5);
    assert!(
        capacity < charge_off,
        "the income terms should pull it below a property-only charge"
    );
    assert!((diff.total_difference().expect("both sides") - 3_093.18).abs() < 0.5);

    // So the two mechanisms disagree about which of these districts is wealthy, and neither
    // ordering is obviously wrong — that disagreement is the substance of the reform, not an
    // artifact of one district.
    let ottawa = at_fy2027(find(&districts, "Ottawa Hills Local"), TERMINAL_MILLS);
    assert!(ottawa.total_difference().expect("both") < 0.0);
    assert!(diff.total_difference().expect("both") > 0.0);
    assert!(
        jefferson.valuation_per_pupil > find(&districts, "Ottawa Hills Local").valuation_per_pupil,
        "the one the charge-off treated as richer is the one local capacity treats as poorer"
    );
}

/// Direction across the state, and it is not one-sided.
#[test]
fn the_charge_off_would_pay_193_districts_more_and_413_less() {
    let districts = panel();
    let diffs = panel_at_fy2027(&districts, TERMINAL_MILLS);

    let favours_fsfp = diffs
        .iter()
        .filter(|d| d.total_difference().is_some_and(|t| t > 0.01))
        .count();
    let favours_charge_off = diffs
        .iter()
        .filter(|d| d.total_difference().is_some_and(|t| t < -0.01))
        .count();
    assert_eq!(favours_fsfp, 413);
    assert_eq!(favours_charge_off, 193);
    assert_eq!(favours_fsfp + favours_charge_off, 606);

    // 81 districts would receive no base cost aid at all under a 23-mill charge-off, because it
    // had no floor. The Fair School Funding Plan's minimum state share is what replaced that.
    let zeroed = diffs
        .iter()
        .filter(|d| d.predecessor_total == Some(0.0))
        .count();
    assert_eq!(zeroed, 81);
}

/// Incidence across wealth, which does **not** run the way the reform's usual description
/// implies: the wealthiest quintile is the one the Fair School Funding Plan treats most
/// generously relative to the charge-off.
#[test]
fn the_wealthiest_quintile_gains_most_from_the_change_of_mechanism() {
    let districts = panel();
    let diffs = panel_at_fy2027(&districts, TERMINAL_MILLS);

    let mut rows: Vec<(f64, f64)> = districts
        .iter()
        .zip(&diffs)
        .filter_map(|(r, d)| Some((r.valuation_per_pupil?, d.total_difference()?)))
        .collect();
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("no NaN valuations"));

    let means: Vec<f64> = (0..5)
        .map(|q| {
            let band = &rows[q * rows.len() / 5..(q + 1) * rows.len() / 5];
            band.iter().map(|(_, d)| d).sum::<f64>() / band.len() as f64
        })
        .collect();

    assert!((means[0] - 154.85).abs() < 1.0, "Q1 {:.2}", means[0]);
    assert!((means[4] - 873.92).abs() < 1.0, "Q5 {:.2}", means[4]);
    for mean in &means {
        assert!(*mean > 0.0, "every quintile does better under the plan");
    }
    assert!(
        means[4] > means[0] * 4.0,
        "the top quintile gains {:.0} against the bottom's {:.0}",
        means[4],
        means[0]
    );

    // And it is not simply the 81 zeroed districts dragging the average. Splitting the top
    // quintile by whether the charge-off would have zeroed it leaves both halves gaining
    // similarly, so the minimum state share is part of the story and not the whole of it.
    let top = &rows[rows.len() * 4 / 5..];
    let split = |zeroed: bool| {
        let band: Vec<f64> = districts
            .iter()
            .zip(&diffs)
            .filter_map(|(r, d)| Some((r.valuation_per_pupil?, r, d.total_difference()?)))
            .filter(|(v, ..)| *v >= top[0].0)
            .filter(|(_, r, _)| {
                (0.023 * r.valuation_per_pupil.unwrap_or(0.0) >= r.base_cost_per_pupil) == zeroed
            })
            .map(|(_, _, d)| d)
            .collect();
        (band.len(), band.iter().sum::<f64>() / band.len() as f64)
    };
    let (zeroed_count, zeroed_mean) = split(true);
    let (rest_count, rest_mean) = split(false);
    assert_eq!((zeroed_count, rest_count), (81, 41));
    assert!((zeroed_mean - 908.63).abs() < 1.0, "{zeroed_mean:.2}");
    assert!((rest_mean - 805.35).abs() < 1.0, "{rest_mean:.2}");
    assert!((zeroed_mean - rest_mean).abs() < zeroed_mean * 0.2);
}

/// A diff whose component row is censored still reports its total, and refuses to attribute it.
#[test]
fn a_floored_district_shows_the_difference_and_declines_to_explain_it() {
    let districts = panel();
    let shaker = find(&districts, "Shaker Heights City");
    assert!(shaker.at_minimum_state_share());

    let diff: RegimeDiff = at_fy2027(shaker, TERMINAL_MILLS);
    assert!(diff.components[0].successor.is_none());
    assert!(diff.components[0].predecessor.is_some());
    assert!(!diff.is_complete());
    assert_eq!(diff.residual(), None);
    assert_eq!(diff.attributed(), None);
    assert_eq!(diff.attributed_share(), None);

    // The number is still there, and it is large.
    let total = diff.total_difference().expect("both totals are computable");
    assert!((total + 2_312.86).abs() < 0.5, "{total:.2}");
}
