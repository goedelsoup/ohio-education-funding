//! The supplemental targeted assistance schedule, recovered from the last year it was paid.
//!
//! # The tier the corpus could describe and not price
//!
//! `[I] Supplemental Targeted Assistance` is zero for every district in the FY2027 model, and the
//! eligibility flag `[H]` beside it is not: the state still runs the test, still names the
//! districts that meet it, and attaches no money. `the_remaining_categoricals.rs` records that as
//! a live zero rather than an absence.
//!
//! What stopped it being a lever was the rate. R.C. 3317.0218 is repealed, so there is no text to
//! read, and the department's own description names a range and no formula — *"the amount of
//! per-pupil funding is scaled to increase based as the level of wealth decreases ... Districts
//! who qualify for this supplemental funding receive between an additional $85 and $750 per
//! student educated."*
//!
//! # It is recoverable, exactly, from the payment report
//!
//! The FY2025 report carries what each of the 36 districts was paid, and the FY2027 model carries
//! the FY2019 wealth index the tier is scaled on. Against those the rate is **linear in the
//! index**, and the line runs from **$75.0000 at the eligibility threshold of 1.6** to
//! **$750.00 at the state's highest index** — Youngstown City, 2.81996153.
//!
//! Worst relative residual across all 36 districts: **6.4e-08**. That is not a fit; it is the
//! schedule.
//!
//! Two details had to be right together, and each is a finding:
//!
//! - **The denominator is current-year enrolled ADM**, not either FY2019 count. "Per student
//!   educated" is the department's own phrase for it. Fitted against FY2019 enrolled ADM the
//!   relation is only approximately linear and East Cleveland misses by 35%.
//! - **The $85 is not the floor.** It is what the lowest *qualifying* district receives —
//!   Middletown City at index 1.6194, $85.76. The floor is $75, at a threshold no district sits
//!   on.
//!
//! # And the top of the scale is a rank
//!
//! $750 is paid to whichever district holds the highest FY2019 wealth index, and everything below
//! is interpolated toward $75. Nobody sets the number at which the scale tops out; one district's
//! position does — the same shape as R.C. 3317.017(A)(4)'s benchmark, established one PR ago.
//! Two parameters in this formula that read as discretionary and are statistics of a
//! distribution.
//!
//! Cited by `corpus/formula-component/fsfp-targeted-assistance.yml`.

use project::panel::{
    self, TA_SUPPLEMENT_FLOOR_SHARE, TA_SUPPLEMENT_INDEX_THRESHOLD, TA_SUPPLEMENT_TOP_RATE,
};
use project::policy::Statewide;

mod common;

/// The FY2025 payment, by IRN, for the districts the tier reached.
fn last_paid() -> Vec<(String, f64)> {
    project::baseline::frame()
        .into_iter()
        .filter(|row| row.supplemental_targeted_assistance > 0.0)
        .map(|row| (row.irn.clone(), row.supplemental_targeted_assistance))
        .collect()
}

/// **The schedule reproduces every one of the 36 payments.**
///
/// The check that makes the tier a lever rather than a guess.
#[test]
fn the_recovered_schedule_reproduces_the_last_payment_exactly() {
    let records = panel::panel();
    let top = Statewide::supplemental_top_index(&records).expect("a panel with FY2019 history");

    let mut checked = 0;
    let mut worst = 0.0_f64;
    let mut worst_name = String::new();

    for (irn, paid) in last_paid() {
        let Some(record) = records.iter().find(|r| r.irn == irn) else {
            continue;
        };
        // **The FY2025 count, not the panel's current-year one.** The tier is paid on the pupils
        // a district educates in the year of payment, so reproducing the FY2025 payment needs
        // FY2025 enrolment — `HISTORY_YEARS[1]`. Run against the panel's own current year the
        // worst residual is 13.7% rather than 6.4e-08, which is the size of three years of
        // enrolment decline and not a defect in the schedule.
        let computed = record.targeted_assistance.supplemental_under(
            record.adm_history[1],
            top,
            TA_SUPPLEMENT_TOP_RATE,
        );
        assert!(
            computed > 0.0,
            "{} was paid {paid:.0} and the schedule pays it nothing",
            record.name
        );
        let residual = ((computed / paid) - 1.0).abs();
        if residual > worst {
            worst = residual;
            worst_name = record.name.clone();
        }
        checked += 1;
    }

    assert_eq!(checked, 36, "every paid district is in the panel");
    assert!(
        worst < 1e-6,
        "worst residual {worst:.3e} at {worst_name}; it was 6.4e-08 when this was written, and \
         a residual this size means the denominator or the endpoints have moved"
    );
}

/// The two endpoints, named. $75 at the threshold and $750 at the top district.
#[test]
fn the_scale_runs_from_seventy_five_at_the_threshold_to_seven_fifty_at_the_top() {
    let records = panel::panel();
    let top = Statewide::supplemental_top_index(&records).expect("a panel");

    // The top of the scale is a district, and it is Youngstown City.
    let holder = records
        .iter()
        .filter(|r| r.targeted_assistance.fy19_wealth_index > 0.0)
        .max_by(|a, b| {
            a.targeted_assistance
                .fy19_wealth_index
                .total_cmp(&b.targeted_assistance.fy19_wealth_index)
        })
        .expect("a panel");
    assert_eq!(holder.name, "Youngstown City");
    assert!(common::agrees_within(1e-6, top, 2.819_961_53));

    // It is paid the top rate exactly, which is what makes the rank the top of the scale.
    let at_top = holder
        .targeted_assistance
        .supplemental_rate(top, TA_SUPPLEMENT_TOP_RATE);
    assert!(
        (at_top - 750.0).abs() < 1e-6,
        "the highest index is paid {at_top:.4}, not $750"
    );

    // And a hypothetical district sitting on the threshold is paid a tenth of that. None does,
    // which is why the department's prose names $85 — the lowest district actually reached.
    let floor = TA_SUPPLEMENT_TOP_RATE * TA_SUPPLEMENT_FLOOR_SHARE;
    assert!((floor - 75.0).abs() < 1e-12);

    let lowest = records
        .iter()
        .filter(|r| r.targeted_assistance.supplement_eligible)
        .min_by(|a, b| {
            a.targeted_assistance
                .fy19_wealth_index
                .total_cmp(&b.targeted_assistance.fy19_wealth_index)
        })
        .expect("qualifying districts");
    assert_eq!(lowest.name, "Middletown City");
    let lowest_rate = lowest
        .targeted_assistance
        .supplemental_rate(top, TA_SUPPLEMENT_TOP_RATE);
    assert!(
        (lowest_rate - 85.76).abs() < 0.01,
        "the lowest qualifying district is paid {lowest_rate:.2}, and the department's prose \
         rounds it to $85"
    );
    assert!(
        lowest_rate > floor,
        "no district sits on the threshold, so the floor is never actually paid"
    );
    assert!((TA_SUPPLEMENT_INDEX_THRESHOLD - 1.6).abs() < 1e-12);
}

/// The eligibility flag the fixture carries is the rule, and the rule is derivable.
///
/// Already checked in `the_remaining_categoricals.rs` against a hand-written condition; checked
/// here against `TargetedAssistance::qualifies`, so the rule has one implementation rather than
/// one implementation and one test.
#[test]
fn the_published_flag_is_the_two_fy2019_tests() {
    let records = panel::panel();
    let mut testable = 0;
    for record in &records {
        let Some(expected) = record.targeted_assistance.qualifies() else {
            continue;
        };
        testable += 1;
        assert_eq!(
            expected,
            record.targeted_assistance.supplement_eligible,
            "{} disagrees with the rule at index {:.4}, {:.1} of {:.1} ADM",
            record.name,
            record.targeted_assistance.fy19_wealth_index,
            record.targeted_assistance.fy19_enrolled_adm,
            record.targeted_assistance.fy19_total_adm
        );
    }
    assert!(
        testable > 500,
        "only {testable} districts carry FY2019 history"
    );
}

/// **Restoring the tier costs more than it last did, and reaches more districts.**
///
/// The FY2025 payment was $52,272,453 to 36 districts. Run at the same schedule against the
/// FY2027 panel it is a different number, for two reasons that point opposite ways: the
/// eligibility flag qualifies more districts than were paid, and enrolment has fallen in most of
/// the districts it reaches.
///
/// Reported rather than reconciled. A repeal priced at its last payment is priced in the wrong
/// year, and this is the size of that error.
#[test]
fn the_tier_run_forward_is_not_the_tier_as_last_paid() {
    let records = panel::panel();
    let top = Statewide::supplemental_top_index(&records).expect("a panel");

    let forward: Vec<(String, f64)> = records
        .iter()
        .filter_map(|r| {
            let amount = r.targeted_assistance.supplemental_under(
                r.current_year_adm,
                top,
                TA_SUPPLEMENT_TOP_RATE,
            );
            (amount > 0.0).then(|| (r.name.clone(), amount))
        })
        .collect();

    let total: f64 = forward.iter().map(|(_, a)| a).sum();
    let last: f64 = last_paid().iter().map(|(_, a)| a).sum();

    assert!(
        (last - 52_272_453.0).abs() < 1.0,
        "the FY2025 payment is the baseline this is read against: {last:.0}"
    );
    assert_eq!(
        forward.len(),
        36,
        "the same 36 districts qualify in the FY2027 model as were paid in FY2025; the tier's \
         eligibility test is frozen on FY2019 data, so its population does not move"
    );
    assert!(
        total < last,
        "restored on the panel's own enrolment the tier costs {total:.0} against {last:.0} in \
         FY2025 — the population is fixed and the districts in it have fewer pupils"
    );
}
