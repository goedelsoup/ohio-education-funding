//! Where the voucher and community school money is not.
//!
//! The `deduction` calculator has been a stub since genesis, recorded as blocked on
//! `dew-foundation`. That connector is now wired with three sources and the block did not lift,
//! which meant the recorded reason was wrong: the FY2027 funding calculator does not carry the
//! deduction channel at all, and no amount of extracting more of it will produce one.
//!
//! These tests make that a measurement rather than an assertion, so a future phase does not
//! repeat the search. The transfer lines are the only candidate in the report, and what rules
//! them out as a *line* is direction rather than size: a deduction can only reduce, and twenty
//! districts receive a positive transfer.
//!
//! That is a narrower claim than it first looks and the tests keep it narrow. `U` is `S + T` —
//! service centre charges plus unlabelled "Other Adjustments" — so ruling out the line does not
//! rule out a deduction inside its negative half, and some of those are large. What is
//! established is that this workbook cannot settle it.
//!
//! # Why the channel is absent rather than hidden
//!
//! Under the Fair School Funding Plan, community and STEM school students are funded **directly**
//! by the state rather than deducted from the resident district's foundation payment. That was
//! the mechanism change H.B. 110 made in FY2022, and it is the single most common source of
//! error in Ohio funding analysis spanning the transition — a district's foundation payment
//! rises in FY2022 for reasons that have nothing to do with the formula. See
//! [`.yidam/skills/deduction.md`](../../../.yidam/skills/deduction.md).

use project::panel::panel;

/// Cleveland Municipal. Large, urban, and the district with the most community school
/// enrollment in the state — so if any district's report carried a deduction, this one would.
const CLEVELAND: &str = "043786";

#[test]
fn the_transfer_lines_are_not_large_enough_to_be_a_deduction_channel() {
    let panel = panel();
    let cleveland = panel
        .iter()
        .find(|record| record.irn == CLEVELAND)
        .expect("in the model");

    // Roughly 1% of total state support. A voucher and community school deduction for Cleveland
    // would be a large fraction of it, not a rounding line.
    let share = cleveland.total_transfers.abs() / cleveland.total_state_support;
    assert!(
        share < 0.02,
        "Cleveland's transfers are {share:.4} of total state support — large enough to be worth \
         a second look as a deduction channel"
    );
    assert!(cleveland.total_state_support > 300_000_000.0);
}

#[test]
fn transfers_run_in_both_directions_so_the_line_is_not_a_deduction() {
    // The decisive fact, and it is not the one this test was first written to check.
    //
    // The first version asserted that the transfer share has no long right tail, on the
    // reasoning that a deduction would scale with a district's community school enrollment. The
    // tail is real — the 99th percentile is 30% of total state support — so that reasoning was
    // wrong and the assumption was mine rather than the data's.
    //
    // What actually settles it is direction. **A deduction can only reduce.** Twenty districts
    // receive a positive transfer, so `U - Total Transfers` is a net adjustment running both
    // ways and cannot be a deduction channel as a line.
    let panel = panel();
    let positive = panel
        .iter()
        .filter(|record| record.total_transfers > 1.0)
        .count();
    let negative = panel
        .iter()
        .filter(|record| record.total_transfers < -1.0)
        .count();
    assert_eq!(positive, 20, "districts receiving a positive transfer");
    assert_eq!(negative, 589);
    assert!(
        positive > 0,
        "if every transfer were negative this line could not be ruled out by direction alone"
    );
}

#[test]
fn what_is_inside_the_negative_transfers_is_not_settled_by_this_workbook() {
    // The honest limit of the finding above. `U` is `S + T` — educational service center charges
    // plus "Other Adjustments" — and the report labels neither component further. Ruling the
    // *line* out as a deduction channel does not rule out a deduction sitting inside its
    // negative half, and the magnitudes leave room: Shawnee Local's transfers are 44% of its
    // total state support, which is a great deal of service centre.
    //
    // This was recorded as an open question rather than closed by inference, and it stayed open
    // for as long as `total_transfers` was all the corpus carried — the total mixes the service
    // centre charge with the residual, so its size says nothing about either.
    //
    // **It is now closed**, by splitting the two. See
    // `the_unlabelled_residual_is_too_small_to_hide_a_deduction` below. This test is kept because
    // the bound it states is still true and still the reason the question was worth asking.
    let panel = panel();
    let large = panel
        .iter()
        .filter(|record| record.total_state_support > 0.0)
        .filter(|record| {
            record.total_transfers < 0.0
                && record.total_transfers.abs() / record.total_state_support > 0.25
        })
        .count();
    assert!(
        large > 0,
        "no district has a transfer large enough to be worth asking about, which would close \
         the open question this test exists to keep open"
    );
}

#[test]
fn net_state_funding_is_total_state_support_plus_transfers() {
    // The identity that says the transfer lines are the whole of what sits between the two, so
    // there is no unlabelled residual for a deduction to be hiding in.
    for record in panel() {
        let implied = record.total_state_support + record.total_transfers;
        assert!(
            (implied - record.net_state_funding).abs() < 1.0,
            "{}: {} + {} != {}",
            record.name,
            record.total_state_support,
            record.total_transfers,
            record.net_state_funding
        );
    }
}

#[test]
fn the_report_pays_more_than_the_guarantee_holds_a_district_at() {
    // The distinction the previous phase's correction turned on, pinned here too because it is
    // the same shape of error: the report carries several state payments outside the base the
    // guarantee is computed against, and a comparison that ignores them manufactures a gap.
    let panel = panel();
    let wider = panel
        .iter()
        .filter(|record| record.total_state_support > record.realized_aid() + 1.0)
        .count();
    assert!(
        wider * 10 > panel.len() * 9,
        "only {wider} of {} districts are paid more than their formula-plus-guarantee base",
        panel.len()
    );
}

/// The question, closed — by magnitude rather than by inference.
///
/// The tests above rule the transfer *line* out as a deduction channel because it runs in both
/// directions, and then deliberately leave open whether a deduction could sit inside its negative
/// half. That was the honest state of it while `total_transfers` was all the corpus carried,
/// because the total mixes two things.
///
/// The FY2027 report separates them: `S - Educational Service Center` and `T - Other Adjustments`.
/// S is negative for 606 of 609 districts and positive for none, which is a charge behaving like
/// a charge. T is the report's only unlabelled line and therefore the entire remaining hiding
/// place — and it is far too small to be one. Ohio's scholarship programs run on the order of a
/// billion dollars a year against a residual whose negative half is $95.6m.
///
/// No line in the calculator is named for a scholarship or a community school, in either the
/// thirty-column summary or the fifty-eight-column detail. This is the arithmetic saying the same
/// thing.
#[test]
fn the_unlabelled_residual_is_too_small_to_hide_a_deduction() {
    let panel = panel();

    let service_centre: Vec<f64> = panel.iter().map(|r| r.service_center_charge).collect();
    assert_eq!(
        service_centre.iter().filter(|v| **v > 0.0).count(),
        0,
        "the service centre line should never pay a district; it is a charge"
    );
    assert!(
        service_centre.iter().filter(|v| **v < 0.0).count() > 600,
        "and nearly every district should carry one"
    );

    // The residual, on its own, in the only direction a deduction could run.
    let hiding_place: f64 = panel
        .iter()
        .map(|r| r.other_adjustments)
        .filter(|v| *v < 0.0)
        .sum::<f64>()
        .abs();
    let support: f64 = panel.iter().map(|r| r.total_state_support).sum();

    assert!(
        hiding_place / support < 0.02,
        "the unlabelled residual is {:.2}% of total state support; above two percent it would \
         be worth reopening the question",
        hiding_place / support * 100.0
    );

    // Ohio scholarship spending is around a billion a year. Whatever the exact figure, a channel
    // carrying it could not fit inside a residual an order of magnitude smaller.
    const SCHOLARSHIP_ORDER_OF_MAGNITUDE: f64 = 1_000_000_000.0;
    assert!(
        hiding_place * 5.0 < SCHOLARSHIP_ORDER_OF_MAGNITUDE,
        "the residual is {hiding_place:.0}, which is not small enough against scholarship \
         spending for this argument to hold"
    );

    // And the two components still reconcile to the total the report publishes.
    for record in &panel {
        let parts = record.service_center_charge + record.other_adjustments;
        assert!(
            (parts - record.total_transfers).abs() < 1.0,
            "{}: S + T is {parts} against a published total of {}",
            record.name,
            record.total_transfers
        );
    }
}
