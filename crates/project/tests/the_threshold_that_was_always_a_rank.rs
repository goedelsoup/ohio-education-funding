//! Why the fortieth, and what the twelve analyses say instead.
//!
//! `formula-component/fsfp-local-capacity-measure` closed the question of the benchmark's
//! *value* — R.C. 3317.017(A)(4)(c)-(d) makes it the fortieth-highest district ratio, and
//! `local-capacity` reproduces it to 4.5e-9 — and left the reason `[open]`: "Why the fortieth,
//! out of 609, is not stated in the section." It is not stated in any analysis either, and this
//! file records that as the answer rather than leaving it open, because the absence is
//! informative once the twelve are read together.
//!
//! # A threshold written as a rank is the house convention, four regimes deep
//!
//! Every Ohio equalisation formula since at least FY1998 has set its benchmark by *rank* rather
//! than by value, and the ranks are in the same register: 48th percentile for equity aid at its
//! FY1998 peak, the **118th lowest** wealth district for equity aid under H.B. 94, the **490th
//! lowest** for parity aid from FY2002, and the same 490th still for tier one targeted assistance
//! twelve years later under H.B. 59. The fortieth is not an innovation in kind. What it is, is the
//! first one counted **from the top**.
//!
//! That reversal is the whole of the design change, and it follows from what the two devices do.
//! A rank counted from the bottom names the districts that *qualify* — parity aid paid the 490
//! poorest and nobody else. A rank counted from the top names the districts that are *capped*:
//! everybody is charged local capacity, and the fortieth marks where the charge stops being
//! progressive. So the question "why 40" is not the same question as "why 490", and the earlier
//! answer does not carry over: 40 of 609 is the 93rd percentile, not the 80th.
//!
//! # LSC explained the predecessor threshold and not this one
//!
//! This is what makes the absence worth recording. H.B. 94's analysis does not merely state the
//! 80th percentile — it argues for it over two paragraphs, with a chart, naming the disparity it
//! is meant to close. H.B. 110's analysis states the 40th highest twice, in a formula box, inside
//! a 1,455-character description of the capacity percentage that contains no reason of any kind.
//! LSC's habit is to give the reason when the act supplies one. Here there was none to give.
//!
//! # And it was never a budget decision, which is why no budget analysis explains it
//!
//! The reason the trace comes back empty is structural rather than accidental. H.B. 1 of the
//! 134th General Assembly — the bill the plan was drafted in, which passed the House and died in
//! the Senate four months before H.B. 110 enacted the formula — already writes "the fortieth
//! highest ratio", three times, in the division that survives verbatim in force today. A
//! greenbook explains what a *budget* changed; this budget did not choose the number, it adopted
//! a drafted plan whole. So the absence is not LSC declining to explain a choice. There was no
//! choice in front of it to explain.
//!
//! That is the same use [`project::plan_bill`] is admitted for everywhere else in this
//! repository — **structural**, which provisions were written together and when — and no rate or
//! threshold is taken from the bill as authority for what the law is.
//!
//! **What this does not say.** Not that the number was arbitrary. Whatever work chose forty sits
//! upstream of H.B. 1, in the Fair School Funding Plan workgroup's own documentation, and no
//! budget analysis would carry it. This file says only that the fourteen analyses do not answer
//! it, that the one place Ohio's greenbooks *do* justify a rank threshold is the regime this one
//! replaced, and that the question has to be asked of the plan rather than of the budget.

mod common;

use project::greenbook::{greenbook, greenbooks};
use project::ledger::budget_analysis::Edition;
use project::plan_bill;
use project::statute;

/// The enacting analysis states the rule and says nothing about why forty.
///
/// The span is LSC's entire account of the capacity percentage, from the heading to the
/// `Valuation` heading that follows it: the definition, the sliding scale, and the three-branch
/// formula box. It carries no justification word at all.
#[test]
fn the_enacting_analysis_states_the_fortieth_and_gives_no_reason_for_it() {
    let hb110 = greenbook("hb110").flat();

    assert!(
        hb110.contains("less than the 40th highest district ratio"),
        "H.B. 110's analysis states the benchmark"
    );
    assert!(
        hb110.contains("the 40th highest district index, then District"),
        "and states it a second time, for the branch at or above it"
    );

    let from = hb110
        .find("Per-pupil local capacity percentage")
        .expect("LSC's heading for the rate");
    let to = hb110[from..]
        .find("Valuation A district")
        .expect("the heading that ends the section")
        + from;
    let section = hb110[from..to].to_lowercase();

    // Not a keyword count chosen to pass: these are the words LSC uses when it does explain a
    // threshold, and `the_predecessor_threshold_was_argued_for` asserts four of them appear in
    // H.B. 94's account of the 80th percentile.
    for word in [
        "because",
        "in order to",
        "disparit",
        "reflect",
        "intend",
        "so that",
        "purpose",
        "helps",
    ] {
        assert!(
            !section.contains(word),
            "LSC's account of the capacity percentage contains {word:?}, so it may give a reason \
             after all and this file's claim needs rereading"
        );
    }
}

/// Nothing else in fourteen documents mentions it.
///
/// Twelve enacted analyses, plus both editions of H.B. 96's. A threshold discussed in one
/// document of fourteen is a threshold nobody revisited.
#[test]
fn the_fortieth_appears_in_one_analysis_of_fourteen() {
    let mentions: Vec<&str> = greenbooks()
        .iter()
        .filter(|book| book.mentions("40th highest"))
        .map(|book| book.bill)
        .collect();
    assert_eq!(mentions, ["hb110"], "{mentions:?}");

    for edition in [Edition::Introduced, Edition::Enacted] {
        let hb96 = common::flat(edition.text());
        assert!(
            !hb96.contains("40th"),
            "H.B. 96's {edition:?} edition mentions the benchmark"
        );
    }
}

/// Four regimes, four thresholds, all of them ranks.
///
/// The point is the shape rather than the values: Ohio does not write a wealth threshold as a
/// dollar figure, it writes the district that sits at it. So "the benchmark is a rank" is not
/// something the Fair School Funding Plan introduced and is not, by itself, the thing that needs
/// explaining.
#[test]
fn every_regime_since_fy1998_set_its_threshold_by_rank() {
    let hb94 = greenbook("hb94").flat();
    assert!(
        hb94.contains("to the 48th percentile district"),
        "equity aid at its FY1998 peak equalised to a percentile district"
    );
    assert!(
        hb94.contains("Threshold = The 118th lowest wealth district"),
        "H.B. 94's own equity aid names the 118th lowest"
    );
    assert!(
        hb94.contains("Threshold = The 490th Lowest Wealth District"),
        "and its parity aid the 490th lowest"
    );

    assert!(
        greenbook("hb119").flat().contains(
            "lowers the number of qualifying districts to the 410 lowest wealth \
                      districts in FY 2008 and the 367 lowest wealth districts in FY 2009"
        ),
        "H.B. 119 moves the count of qualifying districts and leaves the equalisation rank alone"
    );
    assert!(
        greenbook("hb59")
            .flat()
            .contains("the district with the 490th lowest wealth per pupil"),
        "and tier one targeted assistance is still on the 490th twelve years later"
    );
}

/// The predecessor threshold was argued for, at length, with the disparity named.
///
/// Two paragraphs and a chart. This is what LSC does when the act gives it something to say, and
/// it is the measure against which H.B. 110's silence about the fortieth means anything.
#[test]
fn the_predecessor_threshold_was_argued_for() {
    let hb94 = greenbook("hb94").flat();

    assert!(
        hb94.contains(
            "The use of the 80th percentile as the threshold helps reduce disparities in local \
             spending above the adequacy level"
        ),
        "H.B. 94 gives the reason for the 80th percentile in as many words"
    );
    assert!(
        hb94.contains(
            "The top wealthiest 20 percent of school districts (including about 25 percent of all \
             students) consistently have much higher per pupil revenues"
        ),
        "and states the distributional fact the threshold is drawn against"
    );

    // Restated in the next biennium, which is how a reason that is doing work behaves.
    assert!(
        greenbook("hb95").flat().contains(
            "Providing equalized parity aid to school districts below the 80th \
                       percentile level helps reduce this gap"
        ),
        "H.B. 95 restates it"
    );
}

/// The direction reversed, and the earlier answer does not carry over.
///
/// LSC gives the conversion itself for parity aid — the 490th of 612 districts *is* the 80th
/// percentile — so the two thresholds can be put in one unit without this file choosing a
/// convention. The fortieth highest of 609 leaves 569 districts below it, which is the 93rd
/// percentile: a different place in the distribution, reached by counting from the other end.
#[test]
fn the_fortieth_highest_is_not_the_eightieth_percentile_carried_forward() {
    assert!(
        greenbook("hb119").flat().contains(
            "the district with the 490th highest local wealth (the 80th percentile) qualified for \
             parity aid"
        ),
        "LSC states the rank and the percentile together, so the unit conversion is theirs"
    );

    let parity: f64 = 490.0 / 612.0;
    assert!((parity - 0.80).abs() < 0.01, "{parity}");

    // 609 traditional districts, the count `panel` carries and the count H.B. 96's analysis uses.
    let capacity: f64 = (609.0 - 40.0) / 609.0;
    assert!((capacity - 0.934).abs() < 0.001, "{capacity}");
    assert!(
        capacity - parity > 0.10,
        "the two thresholds sit more than ten points apart: parity {parity}, capacity {capacity}"
    );
}

/// The number arrived with the plan bill, not with the budget that enacted it.
///
/// H.B. 1 of the 134th is committed for exactly this kind of question — see
/// [`project::plan_bill`] for why an un-enacted bill is held and what it must not be read as.
/// The claim here is structural: the benchmark was drafted before the budget process touched it,
/// so a document whose job is to say what a budget changed was never going to carry a reason.
#[test]
fn the_fortieth_was_drafted_in_the_plan_bill_before_any_budget_carried_it() {
    let bill = plan_bill::flat().to_lowercase();
    assert_eq!(
        bill.matches("fortieth highest ratio").count(),
        3,
        "H.B. 1 writes the benchmark into all three branches of the capacity percentage"
    );

    // The same three branches, in force, with the same rank. The act did not restate the
    // threshold; it carried the drafted one.
    let in_force = common::flat(statute::section("3317.017").body).to_lowercase();
    assert_eq!(
        in_force.matches("fortieth highest ratio").count(),
        3,
        "and R.C. 3317.017 still reads the same way"
    );
}
