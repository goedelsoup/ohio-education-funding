//! What the transportation formula's general proration factor is, and when it bound.
//!
//! `formula-component/fsfp-transportation` closes on an `[open]`: *"What the general proration
//! factor `[F3]` is for. It is 1.0 in FY2027 and it multiplies the whole transportation total, so
//! it is a dial that has been used and may be again; nothing here records when."*
//!
//! It is not a dial. It is an appropriation divided by an entitlement, and LSC states it as a
//! formula: `Adjustment percentage = (Earmarked appropriation)/(Total statewide allocation)`. The
//! corpus already has the concept — `parameter/appropriation-proration-factor` is the node for
//! exactly this shape, and records that a factor of 1.0 "says the entitlement was computed, the
//! appropriation was compared against it, and the two did not bind this year". This file supplies
//! the years in which they did.
//!
//! # The supplement is the proof
//!
//! Two bienniums created a **supplemental transportation payment** whose formula is
//! `formula allocation − prorated aid`, paid to a targeted set of districts. A payment defined as
//! the difference between the full amount and the prorated one cannot exist unless the proration
//! binds, and it cannot be worth appropriating for unless the difference is material — $25.3m in
//! FY2014 alone. So the factor was below 1 in FY2010, FY2011, FY2014 and FY2015, and the corpus
//! can say so without holding the factor itself.
//!
//! Who the supplement rescued changed. H.B. 1 selected on **wealth per pupil and ridership
//! density**, both at or below the median; H.B. 59 selected on **state share index ≥ 0.5** and the
//! same density test. Both are "poor and sparse", reached by different instruments — the first
//! through parity aid's wealth measure, the second through the state share index.
//!
//! # And for one biennium the line did not pay for transportation
//!
//! *"in FY 2012 and FY 2013, the formula was not used and instead the appropriation for pupil
//! transportation provided funding for the bridge formula."* ALI 200502 stayed in the budget and
//! stopped being a transportation appropriation, which is a thing an appropriation series read
//! by line item cannot see.

mod common;

use project::greenbook::{self, Greenbook};
use project::ledger::appropriations;

/// The FY2010-11 budget's analysis — Am. Sub. H.B. 1 of the 128th, which wrote the proration out.
fn wrote_it() -> Greenbook<'static> {
    greenbook::greenbook("hb1")
}

/// The FY2014-15 budget's — Am. Sub. H.B. 59 of the 130th, which brought the formula back.
fn revived_it() -> Greenbook<'static> {
    greenbook::greenbook("hb59")
}

/// The proration is an appropriation clamp, and two analyses state it in as many words.
#[test]
fn the_proration_is_an_appropriation_clamp_and_lsc_states_the_formula() {
    let lsc = wrote_it().flat();
    assert!(lsc.contains(
        "In order to keep the total statewide payment to the amount earmarked for such purposes \
         in item 200502, Pupil Transportation, the percentage the appropriation amount is of the \
         current year\u{02B9}s total allocation is applied to each district\u{02B9}s allocation."
    ));
    assert!(lsc.contains(
        "Adjustment percentage = (Earmarked appropriation)/(Total statewide allocation) \
         District's prorated transportation payment = (District's transportation allocation) x \
         (Adjustment percentage)"
    ));

    // And the same instruction, put plainly, in the budget that revived the formula.
    assert!(revived_it().flat().contains(
        "ODE is required to prorate the calculated amount for each district to fit within the \
         appropriation."
    ));
}

/// A supplement defined as full minus prorated is proof the proration bound.
///
/// Four fiscal years of it, under two different eligibility rules, and in FY2014-15 with an
/// earmark of its own. The factor itself is not published in either biennium; the payment that
/// exists only when it is below one, is.
#[test]
fn the_supplement_is_the_proof_the_clamp_bound() {
    assert!(wrote_it().flat().contains(
        "The budget requires a supplemental transportation payment be granted to districts with \
         both wealth per pupil and bus ridership density at or below the state median."
    ));
    assert!(wrote_it().flat().contains(
        "Qualifying districts are paid the difference between the full calculated amount for \
         transportation and the prorated payment the district would otherwise receive, phased in \
         at 30% in FY 2010 and 70% in FY 2011."
    ));

    let later = revived_it().flat();
    assert!(later.contains(
        "supplemental transportation payment for districts that have a state share index of 0.5 \
         or greater and pupil density at or below the state median."
    ));
    assert!(later.contains(
        "Supplemental transportation aid = Transportation formula allocation \u{2013} Prorated \
         transportation aid"
    ));
}

/// What it divides by is an earmark inside the line, and the earmarks reconcile with the ledger.
///
/// The denominator is not ALI 200502. It is the "Prorated Transportation Aid" earmark within it —
/// $413,385,915 in FY2014 — which is 82% of the line. The other six earmarks pay for special
/// education transportation, payments in lieu, driver training, and the supplement that undoes
/// the proration.
///
/// The seven sum to the line, and the line is the one `project::ledger` already holds from a
/// different document. Two publishers, one figure, to the dollar.
#[test]
fn the_denominator_is_an_earmark_inside_the_line() {
    assert!(revived_it().flat().contains(
        "200502, Pupil Transportation Earmarks FY 2014 FY 2015 Bus Driver Training $ 838,930 \
         $ 838,930 Special Education Transportation $ 60,469,220 $ 60,469,220 Payments In Lieu of \
         Transportation $ 5,000,000 $ 2,500,000 Supplemental Transportation Aid $ 25,300,000 \
         $ 23,100,000 Prorated Transportation Aid $ 413,385,915 $ 434,055,210 Remainder $ 19,462 \
         $ 50,167 Total Funding: Pupil Transportation $ 505,013,527 $ 521,013,527"
    ));

    let earmarks = [
        (838_930.0, 838_930.0),
        (60_469_220.0, 60_469_220.0),
        (5_000_000.0, 2_500_000.0),
        (25_300_000.0, 23_100_000.0),
        (413_385_915.0, 434_055_210.0),
        (19_462.0, 50_167.0),
    ];
    let enacted = |year: u16| {
        appropriations::lines()
            .into_iter()
            .find(|line| {
                line.line_item == "200502" && line.fiscal_year == year && line.kind == "enacted"
            })
            .unwrap_or_else(|| panic!("no enacted 200502 for FY{year}"))
            .amount
    };
    assert!(common::close(
        earmarks.iter().map(|e| e.0).sum::<f64>(),
        enacted(2014)
    ));
    assert!(common::close(
        earmarks.iter().map(|e| e.1).sum::<f64>(),
        enacted(2015)
    ));

    // The share of the line the proration's denominator actually is.
    let share = 413_385_915.0 / enacted(2014);
    assert!(
        (0.81..0.83).contains(&share),
        "the earmark is {share:.4} of the line"
    );
}

/// And for one biennium the transportation line did not pay for transportation.
///
/// The formula was not run in FY2012 or FY2013; ALI 200502 funded the Bridge formula instead. So
/// the proration has no value in those years because there was no entitlement to divide, and an
/// appropriation series read by line item shows two ordinary years where the money changed
/// purpose entirely.
#[test]
fn for_one_biennium_the_line_did_not_pay_for_transportation() {
    assert!(revived_it().flat().contains(
        "in FY 2012 and FY 2013, the formula was not used and instead the appropriation for pupil \
         transportation provided funding for the bridge formula"
    ));

    // The line is uninterrupted across those years, which is why the ledger cannot show it.
    let actuals: Vec<(u16, f64)> = appropriations::lines()
        .into_iter()
        .filter(|line| line.line_item == "200502" && line.kind == "actual")
        .map(|line| (line.fiscal_year, line.amount))
        .filter(|(year, _)| (2012..=2013).contains(year))
        .collect();
    assert_eq!(actuals.len(), 2);
    for (year, amount) in actuals {
        assert!(amount > 430_000_000.0, "FY{year} spent {amount}");
    }
}
