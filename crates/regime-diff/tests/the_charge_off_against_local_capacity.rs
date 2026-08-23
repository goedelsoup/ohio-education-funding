//! What changed when Ohio stopped charging districts on property alone.
//!
//! The corpus aligns exactly one component pair across the Fair School Funding Plan and its
//! predecessor: local capacity replaces the charge-off local share. This is that comparison, run
//! at FY2027 inputs with the plan's own computed base cost held fixed, so the only thing that
//! moves is how much of that cost the district is deemed able to bear.
//!
//! # What the residual does and does not verify here
//!
//! The residual comes out **exactly zero for 465 of the 470 districts where both sides can be
//! valued**, and that is a check on the substitution rather than a discovery about the regimes.
//! Holding base cost fixed means the local share is the only thing that can differ, so a nonzero
//! residual would mean the arithmetic had gone wrong somewhere. It did not.
//!
//! A true Bridge-formula-against-FSFP diff would have an enormous residual, because the corpus
//! has no Bridge components at all. Nothing here should be read as showing that the two regimes
//! differ only in this component.

use project::panel::{panel, DistrictRecord};
use std::collections::HashMap;

use regime_diff::recognized_valuation::{self, Recognition};
use regime_diff::{
    at_fy2027, panel_at_fy2027, ChargeOffBase, RegimeDiff, ALIGNED, TERMINAL_MILLS, UNALIGNED,
};

/// The base every comparison here runs on: recognized valuation at TY2024.
///
/// Not total taxable value, which is what these tests asserted against until the corpus learned
/// what recognized valuation is. The charge-off was applied to the recognized figure by FY2008 and
/// running it against the full one overstates it — by a median of $493 per pupil, enough to move
/// findings and not only figures. See `recognized_valuation_against_the_abstract.rs`.
fn recognized() -> HashMap<String, Recognition> {
    recognized_valuation::from_abstract(2024)
}

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
    let base = recognized();
    let diffs = panel_at_fy2027(&districts, TERMINAL_MILLS, ChargeOffBase::Recognized(&base));
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
    let base = recognized();
    let diffs = panel_at_fy2027(&districts, TERMINAL_MILLS, ChargeOffBase::Recognized(&base));

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
    // 465 and 5, against 463 and 7 on total taxable value: a smaller charge-off runs past its
    // base cost in two fewer districts.
    assert_eq!(exact, 465);
    assert_eq!(truncated, 5);
}

/// **The reform's stated purpose, measured.** The charge-off was blind to income; local capacity
/// is 40% income. A district whose wealth is in its earnings rather than its property was
/// treated as middling by the old mechanism and is not by the new one.
#[test]
fn a_high_income_district_pays_far_more_under_local_capacity_than_the_charge_off_asked() {
    let districts = panel();
    let base = recognized();
    let ottawa_hills = find(&districts, "Ottawa Hills Local");
    let diff = at_fy2027(
        ottawa_hills,
        TERMINAL_MILLS,
        ChargeOffBase::Recognized(&base),
    );

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
    // $3,402 against $4,052 on the base this test used to assume. Ottawa Hills is in Lucas
    // County, reappraised in TY2024, so 2/3 of a large revaluation is still deferred.
    assert!((charge_off - 3_402.39).abs() < 0.5);
    assert!((capacity - 6_845.07).abs() < 0.5);
    // 2.01x on the recognized base, against 1.69x on total taxable value. The threshold moved
    // with the base; a bound written for the old ratio would pass on either.
    assert!(
        capacity > charge_off * 2.0,
        "local capacity should charge it far more: {capacity:.2} against {charge_off:.2}"
    );

    // Which costs it two thirds of its base cost aid.
    assert!((diff.total_difference().expect("both sides") + 3_442.68).abs() < 0.5);
}

/// And the blend runs the other way too, which is the half of the reform that gets less
/// attention: a district rich in property and poor in income is charged **less**.
#[test]
fn a_property_rich_low_income_district_pays_less_under_local_capacity() {
    let districts = panel();
    let base = recognized();
    let jefferson = find(&districts, "Jefferson Township Local");
    let diff = at_fy2027(jefferson, TERMINAL_MILLS, ChargeOffBase::Recognized(&base));

    let valuation = jefferson
        .valuation_per_pupil
        .expect("in the profile report");
    assert!(valuation > 500_000.0, "valuation per pupil {valuation:.0}");

    let charge_off = diff.components[0].predecessor.expect("valued");
    let capacity = diff.components[0].successor.expect("not at the floor");
    assert!((charge_off - 11_157.75).abs() < 0.5);
    assert!((capacity - 8_714.77).abs() < 0.5);
    assert!(
        capacity < charge_off,
        "the income terms should pull it below a property-only charge"
    );
    assert!((diff.total_difference().expect("both sides") - 2_442.98).abs() < 0.5);

    // So the two mechanisms disagree about which of these districts is wealthy, and neither
    // ordering is obviously wrong — that disagreement is the substance of the reform, not an
    // artifact of one district.
    let ottawa = at_fy2027(
        find(&districts, "Ottawa Hills Local"),
        TERMINAL_MILLS,
        ChargeOffBase::Recognized(&base),
    );
    assert!(ottawa.total_difference().expect("both") < 0.0);
    assert!(diff.total_difference().expect("both") > 0.0);
    assert!(
        jefferson.valuation_per_pupil > find(&districts, "Ottawa Hills Local").valuation_per_pupil,
        "the one the charge-off treated as richer is the one local capacity treats as poorer"
    );
}

/// **Direction across the state, and the correction reversed it.**
///
/// On total taxable value this test asserted 413 districts better off under the plan against 193
/// under the charge-off, and it was named for those numbers. Against the base the charge-off
/// actually used it comes out the other way: **316 districts would have done better under the
/// charge-off and 290 under the plan**, and the median district is $45 per pupil *worse* off under
/// the plan where the corpus previously recorded it as $289 better.
///
/// The old figure was not a small error. Overstating the charge-off by a median $493 per pupil
/// systematically flattered the mechanism that replaced it.
///
/// # This is a TY2024 answer and the year is doing some of the work
///
/// Recognized valuation defers most where a reappraisal was largest, and Ohio's 2023–24 revaluation
/// cycle was extraordinary — a median 28.6% jump against the roughly 15% LSC's own worked example
/// assumes. The statewide deferral is 8.2% here where LSC put the long-run average nearer 2%. So
/// the reversal is real for TY2024 and would be narrower in an ordinary year. What is not
/// year-dependent is the direction: total taxable value always overstates the charge-off, so every
/// figure the corpus published on that base flattered the plan by some amount.
#[test]
fn the_charge_off_would_pay_316_districts_more_and_290_less() {
    let districts = panel();
    let base = recognized();
    let diffs = panel_at_fy2027(&districts, TERMINAL_MILLS, ChargeOffBase::Recognized(&base));

    let favours_fsfp = diffs
        .iter()
        .filter(|d| d.total_difference().is_some_and(|t| t > 0.01))
        .count();
    let favours_charge_off = diffs
        .iter()
        .filter(|d| d.total_difference().is_some_and(|t| t < -0.01))
        .count();
    assert_eq!(favours_fsfp, 290);
    assert_eq!(favours_charge_off, 316);
    assert_eq!(favours_fsfp + favours_charge_off, 606);

    let mut totals: Vec<f64> = diffs.iter().filter_map(|d| d.total_difference()).collect();
    totals.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    let median = totals[totals.len() / 2];
    assert!((median + 44.62).abs() < 0.5, "median {median:.2}");

    // The same comparison on the base the corpus used to assume, so the size of the correction is
    // asserted rather than described. It moves 123 districts across the line.
    let old = panel_at_fy2027(&districts, TERMINAL_MILLS, ChargeOffBase::TotalTaxable);
    let old_favours_fsfp = old
        .iter()
        .filter(|d| d.total_difference().is_some_and(|t| t > 0.01))
        .count();
    assert_eq!(old_favours_fsfp, 413);
    assert_eq!(old_favours_fsfp - favours_fsfp, 123);

    // 65 districts would receive no base cost aid at all under a 23-mill charge-off, because it
    // had no floor. The Fair School Funding Plan's minimum state share is what replaced that. On
    // the overstated base the corpus reported 81, so a sixth of that finding was the wrong base.
    let zeroed = diffs
        .iter()
        .filter(|d| d.predecessor_total == Some(0.0))
        .count();
    assert_eq!(zeroed, 65);
    assert_eq!(
        old.iter()
            .filter(|d| d.predecessor_total == Some(0.0))
            .count(),
        81
    );
}

/// **Incidence across wealth, and this is where the wrong base did the most damage.**
///
/// The old finding was that every quintile does better under the plan, most of all the richest —
/// a striking claim, and the corpus published it. Against the base the charge-off actually used,
/// **only the top quintile still gains**. The bottom four all do better under the charge-off, by
/// $118 to $365 per pupil, and the gradient across them runs the wrong way: the poorer the
/// quintile the *less* it loses.
///
/// So the shape survives the correction and the level does not. The plan is still relatively more
/// generous to property-rich districts than the charge-off was — that was never an artifact — but
/// "every quintile gains" was, and it was an artifact of charging every district 8% too much on
/// the counterfactual side.
///
/// The bottom four quintiles moving together by roughly the median overstatement, while the top
/// one stays far above it, is itself the signature of the error: recognized valuation is close to
/// a uniform proportional discount, so it shifts a level and leaves an ordering alone.
#[test]
fn only_the_wealthiest_quintile_still_gains_once_the_base_is_right() {
    let districts = panel();
    let base = recognized();
    let diffs = panel_at_fy2027(&districts, TERMINAL_MILLS, ChargeOffBase::Recognized(&base));

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

    assert!((means[0] + 118.47).abs() < 1.0, "Q1 {:.2}", means[0]);
    assert!((means[4] - 632.23).abs() < 1.0, "Q5 {:.2}", means[4]);

    // Only the top quintile gains, and the four below it worsen monotonically with wealth.
    assert!(means[4] > 0.0, "the top quintile still gains");
    for mean in &means[..4] {
        assert!(*mean < 0.0, "quintiles one to four lose: {mean:.2}");
    }
    for pair in means[..4].windows(2) {
        assert!(
            pair[1] < pair[0],
            "each poorer quintile should lose less than the next richer: {pair:?}"
        );
    }

    // The old base is what produced the "every quintile gains" reading, and it is asserted here so
    // the correction cannot be quietly undone by someone restoring a default.
    let old = panel_at_fy2027(&districts, TERMINAL_MILLS, ChargeOffBase::TotalTaxable);
    let mut old_rows: Vec<(f64, f64)> = districts
        .iter()
        .zip(&old)
        .filter_map(|(r, d)| Some((r.valuation_per_pupil?, d.total_difference()?)))
        .collect();
    old_rows.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("no NaN valuations"));
    let old_q1 = {
        let band = &old_rows[..old_rows.len() / 5];
        band.iter().map(|(_, d)| d).sum::<f64>() / band.len() as f64
    };
    assert!((old_q1 - 154.85).abs() < 1.0, "old Q1 {old_q1:.2}");
    assert!(
        old_q1 > 0.0 && means[0] < 0.0,
        "the sign of the bottom quintile is what the base decides"
    );

    // Within the top quintile the gain is still not simply the zeroed districts dragging an
    // average — but the two halves are no longer close, which the old base concealed. The 65
    // districts the charge-off would zero gain $900 per pupil against $326 for the other 57.
    let top = &rows[rows.len() * 4 / 5..];
    let split = |zeroed: bool| {
        let band: Vec<f64> = districts
            .iter()
            .zip(&diffs)
            .filter_map(|(r, d)| Some((r.valuation_per_pupil?, r, d.total_difference()?)))
            .filter(|(v, ..)| *v >= top[0].0)
            .filter(|(_, r, _)| {
                let recognized = 0.023
                    * ChargeOffBase::Recognized(&base).ratio_for(&r.irn)
                    * r.valuation_per_pupil.unwrap_or(0.0);
                (recognized >= r.base_cost_per_pupil) == zeroed
            })
            .map(|(_, _, d)| d)
            .collect();
        (band.len(), band.iter().sum::<f64>() / band.len() as f64)
    };
    let (zeroed_count, zeroed_mean) = split(true);
    let (rest_count, rest_mean) = split(false);
    assert_eq!((zeroed_count, rest_count), (65, 57));
    assert!((zeroed_mean - 900.40).abs() < 1.0, "{zeroed_mean:.2}");
    assert!((rest_mean - 326.41).abs() < 1.0, "{rest_mean:.2}");
    assert!(
        zeroed_mean > rest_mean,
        "the districts the charge-off would zero gain most from the floor that replaced it"
    );
}

/// A diff whose component row is censored still reports its total, and refuses to attribute it.
#[test]
fn a_floored_district_shows_the_difference_and_declines_to_explain_it() {
    let districts = panel();
    let base = recognized();
    let shaker = find(&districts, "Shaker Heights City");
    assert!(shaker.at_minimum_state_share());

    let diff: RegimeDiff = at_fy2027(shaker, TERMINAL_MILLS, ChargeOffBase::Recognized(&base));
    assert!(diff.components[0].successor.is_none());
    assert!(diff.components[0].predecessor.is_some());
    assert!(!diff.is_complete());
    assert_eq!(diff.residual(), None);
    assert_eq!(diff.attributed(), None);
    assert_eq!(diff.attributed_share(), None);

    // The number is still there, and it is large.
    let total = diff.total_difference().expect("both totals are computable");
    assert!((total + 3_102.58).abs() < 0.5, "{total:.2}");
}
