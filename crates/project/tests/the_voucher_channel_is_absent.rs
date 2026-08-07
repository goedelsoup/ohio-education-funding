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
    // This is recorded as an open question rather than closed by inference. Settling it needs
    // the department's own deduction reporting, which is what the `deduction` calculator is
    // blocked on and what a search of both public hosts did not find.
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
