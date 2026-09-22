//! Why R.C. 3317.0217(B) compares two whole-district totals, which is a question about the
//! component it inherited rather than about the Fair School Funding Plan.
//!
//! # The `[open]` this answers, and the premise in it that was wrong
//!
//! `the_second_size_term_the_plan_keeps_outside_base_cost.rs` established that the capacity tier
//! is the plan's second size-dependent term and left one thing unestablished: **why division (B)
//! is written against a total.** Two routes were closed off while the question was filed — the
//! section carries no recital, and H.B. 110's greenbook derives nothing — and the framing that
//! survived was that the predecessor "was per-pupil" and that the pupil count was *dropped* when
//! capacity aid was merged into targeted assistance.
//!
//! The predecessor's **payment** was per-pupil. Its **comparison** was not. Am. Sub. H.B. 64's
//! analysis sets capacity aid out in three formula boxes, and the first of them reads
//! `District capacity amount = Three-year average valuation x 0.001` — one mill on a
//! whole-district total — compared against `Median capacity amount`, which is the same quantity
//! for the median district. Total against total, six years before the plan.
//!
//! So (B) did not choose a total. It kept one.
//!
//! # And the biennium that chose it says why
//!
//! LSC, twice, in the two bienniums the component ran under its own name: capacity aid *"targets
//! funding to smaller districts with relatively low total property valuation"*, and is *"based on
//! the amount a district can raise with one mill"*. A one-mill yield is a whole-district dollar
//! amount by construction — that is what the property tax raises — and comparing it to the median
//! district's compares revenue-raising capacity rather than wealth per pupil. The total is the
//! instrument, and reaching small districts is the stated purpose.
//!
//! That recital is in the greenbook of the act that **created** the component, not in the
//! greenbook of the act that absorbed it. `greenbooks-answer-prior-value-questions` in its plainer
//! form: a greenbook explains what a budget changed, so the argument for a component is in the
//! biennium that added it and nowhere afterwards.
//!
//! # What the plan actually did to both tiers, which is one transformation and not two
//!
//! Each predecessor computed an **index** — a median (or statewide, or threshold-district) wealth
//! over the district's own — and each used that index as a **multiplier** in the payment. H.B.
//! 59's tier one: `(threshold wealth per pupil - district wealth per pupil) x target millage x
//! District wealth index`. Capacity aid: `per-pupil amount x Formula ADM x multiplier x Capacity
//! ratio`.
//!
//! R.C. 3317.0217 computes both indices — (B)(3) and (C)(3) — and **multiplies by neither**. Each
//! survives only as a gate: capacity index below 1 pays zero, wealth index below 0.8 pays zero.
//! The payment is a flat millage on a shortfall in both divisions.
//!
//! Once that is the form, the pupil count follows from the shortfall and is not a separate
//! decision:
//!
//! | | shortfall | pupil count |
//! |---|---|---|
//! | (B) capacity | `median weighted wealth - district's`, a whole-district total | none |
//! | (C) wealth | `median per pupil x 0.014 - district's per pupil x 0.0112` | `x enrolled ADM` |
//!
//! (C) multiplies by ADM because a per-pupil shortfall is not money until it does. (B) does not,
//! because a difference of two whole-district totals already is. The ADM multiplier was not
//! dropped from the capacity tier; **the capacity tier never had a per-pupil shortfall to multiply
//! back up**, and capacity aid's ADM multiplier was the thing that turned a *dimensionless ratio*
//! into dollars. Change the comparison from a ratio to a difference and there is nothing left for
//! it to do.
//!
//! # What that costs, and what stands where the multiplier stood
//!
//! It is not a free restatement, because the two forms behave differently in the limit. Capacity
//! aid was `x Formula ADM`, so it went to zero with the pupil count, and its per-pupil generosity
//! was bounded absolutely by the 2.5 cap on the ratio — `per-pupil amount x multiplier x 2.5`, one
//! statewide number, the same ceiling for every district. (B) has no ADM factor and no cap, so as
//! weighted wealth falls its amount approaches `0.008 x median weighted wealth` — **$3,137,210.45**
//! on the FY2027 medians — with no pupil count dividing it.
//!
//! Kelleys Island Local has **3.51** enrolled pupils. Division (B)(4)(b)(i) computes
//! **$2,461,241.79** for it, which is **$701,208.49 a pupil**. The only thing between that and the
//! payment file is the 200/400/600 ramp, which pays it nothing. All **17** districts the cliff and
//! the 5% shelf reach would draw more per pupil than **$3,773.82**, which is the most any district
//! the tier pays in full draws.
//!
//! So the ramp stands where capacity aid's **cap** stood, not where its ADM multiplier stood. Both
//! bound the same divergence; they bound it differently, and only one of them is written as a
//! bound on generosity rather than as a bracket on a pupil count. The tier's remaining size
//! dependence — slope 0.9855 of `ln(weighted wealth)` on `ln(ADM)` — is what capacity aid's ADM
//! multiplier used to work against, and the plan removed the multiplier and kept the measure.
//!
//! # Where the record stops
//!
//! Nothing committed says the drafters intended any of this. H.B. 110's greenbook mentions capacity
//! aid **once**, in the sentence announcing that it is replaced, and never again: the document that
//! announces the merger never states what the merged component did. H.B. 1 of the 134th repeals
//! R.C. 3317.0218 and enacts a new R.C. 3317.0218 under the same number for a different payment,
//! in one bill, so the change was drafted upstream of any budget and there was no choice in front
//! of LSC to explain. That is #392's shape and it is recorded as an answer, not as a gap.

mod common;

use project::greenbook::{self, Greenbook};
use project::panel::categoricals::{
    TargetedAssistance, TA_CAPACITY_MINIMUM_ADM, TA_CAPACITY_RAMP_START, TA_CAPACITY_RATE,
    TA_MEDIAN_WEIGHTED_WEALTH,
};
use project::panel::panel;
use project::size_terms::capacity_tier_unramped;
use project::statute;

/// A cent, the tolerance every dollar figure here is asserted at.
const CENT: f64 = 0.01;

/// Am. Sub. H.B. 59 of the 130th — the analysis that set out the targeted assistance (C) descends
/// from.
fn first() -> Greenbook<'static> {
    greenbook::greenbook("hb59")
}

/// Am. Sub. H.B. 64 of the 131st — the analysis that argues for capacity aid and gives its boxes.
fn added() -> Greenbook<'static> {
    greenbook::greenbook("hb64")
}

/// Am. Sub. H.B. 49 of the 132nd — the one other biennium that describes capacity aid.
fn kept() -> Greenbook<'static> {
    greenbook::greenbook("hb49")
}

/// Am. Sub. H.B. 110 of the 134th — the analysis that announces the merger.
fn merged() -> Greenbook<'static> {
    greenbook::greenbook("hb110")
}

/// **Capacity aid's own comparison was already between two whole-district totals.**
///
/// The premise the question was filed on — that the predecessor was per-pupil — is true of the
/// payment and false of the comparison. One mill on a three-year average *total* valuation, on
/// both sides of the quotient.
#[test]
fn capacity_aids_comparison_was_already_between_two_totals() {
    let lsc = added().flat();
    assert_eq!(added().first_fiscal_year(), 2016);

    // The measure: a whole-district total, and a millage on it rather than a rate per pupil.
    assert!(lsc.contains("District capacity amount = Three-year average valuation x 0.001"));
    assert!(lsc.contains(
        "The capacity ratio is calculated by multiplying each district's three-year average \
         total property valuation by 0.001 to determine its capacity amount and then dividing \
         the statewide median capacity amount by the district's capacity amount."
    ));

    // Neither side of it is divided by a pupil count. The word "pupil" enters this component
    // exactly once, and it enters the *payment* rather than the measurement.
    assert!(lsc.contains(
        "Capacity ratio = The lesser of [(Median capacity amount / District capacity amount) \
         \u{2013} 1] or 2.5"
    ));
    assert!(lsc.contains(
        "Capacity aid per-pupil amount = Median capacity amount / Average formula ADM of all \
         districts with capacity amounts below the median capacity amount"
    ));
}

/// **And the biennium that added it says what the total is for, twice.**
///
/// The recital the current section does not carry. LSC states the target population and the
/// instrument in one sentence, and restates it two years later when the multiplier moves.
#[test]
fn lsc_states_the_purpose_of_the_component_the_tier_inherits_and_states_it_twice() {
    const PURPOSE: &str =
        "targets funding to smaller districts with relatively low total property valuation";
    const INSTRUMENT: &str = "based on the amount a district can raise with one mill";

    assert_eq!(kept().first_fiscal_year(), 2018);
    let created = added().flat();
    let retained = kept().flat();

    assert!(created.contains(PURPOSE), "H.B. 64 states the purpose");
    assert!(retained.contains(PURPOSE), "H.B. 49 restates it verbatim");
    assert!(created.contains(INSTRUMENT));
    assert!(retained.contains(INSTRUMENT));

    // It is a recital and not a restatement of arithmetic: the sentence names a population
    // ("smaller districts") that none of the three formula boxes below it mentions.
    assert!(created.contains(
        "The budget adds a new funding component that targets funding to smaller districts with \
         relatively low total property valuation."
    ));

    // And it is only ever in these two. The plan's own analysis never says what the component it
    // absorbs was for.
    let naming: Vec<u16> = greenbook::greenbooks()
        .iter()
        .filter(|g| g.flat().contains(PURPOSE))
        .map(Greenbook::first_fiscal_year)
        .collect();
    assert_eq!(naming, vec![2016, 2018]);
}

/// **Both predecessors multiplied by their index; R.C. 3317.0217 multiplies by neither.**
///
/// One transformation applied to two components. The index survives in both divisions as a
/// computed quantity used only in a comparison, which is the drafting fingerprint of an instrument
/// carried over rather than designed.
#[test]
fn the_indices_survive_as_gates_and_neither_of_them_multiplies() {
    // The predecessors, each with its index inside the payment.
    assert!(first().flat().contains(
        "Tier one targeted assistance per pupil = (Wealth per pupil of 490th lowest wealth \
         district - District wealth per pupil) x Target millage x District wealth index"
    ));
    assert!(added().flat().contains(
        "Capacity aid = Capacity aid per-pupil amount x Formula ADM x Capacity aid multiplier x \
         Capacity ratio"
    ));

    let code = statute::section("3317.0217").body.to_string();

    // Each index is computed, in its own numbered division.
    assert!(code.contains(
        "Compute each district's capacity index for that fiscal year by dividing the median \
         weighted wealth of all school districts in this state for that fiscal year by the \
         district's weighted wealth for that fiscal year"
    ));
    assert!(code.contains(
        "Compute each district's wealth index for that fiscal year by dividing the median \
         weighted wealth per pupil of all school districts in this state for that fiscal year by \
         the district's weighted wealth per pupil for that fiscal year"
    ));

    // And each is spent entirely on a threshold.
    assert!(code.contains("The district's capacity index is less than 1."));
    assert!(code.contains(
        "If the district's wealth index computed under division (C)(3) of this section for that \
         fiscal year is less than 0.8, the district's wealth amount for that fiscal year shall be \
         zero."
    ));

    // The two payment formulas are the pair below, and neither carries an index factor.
    const CAPACITY_PAYMENT: &str = "(The median weighted wealth of all school districts in this \
                                    state for that fiscal year X 0.008) - (the district's \
                                    weighted wealth for that fiscal year X 0.008)";
    const WEALTH_PAYMENT: &str = "[(The median weighted wealth per pupil of all school districts \
                                  in this state for that fiscal year X 0.014) - (the district's \
                                  weighted wealth per pupil for that fiscal year X 0.0112)] X the \
                                  district's enrolled ADM for that fiscal year";
    assert!(code.contains(CAPACITY_PAYMENT));
    assert!(code.contains(WEALTH_PAYMENT));
    assert!(!CAPACITY_PAYMENT.contains("index"));
    assert!(!WEALTH_PAYMENT.contains("index"));
}

/// **The pupil count goes with the shortfall, and (B) has no per-pupil shortfall.**
///
/// The asymmetry the question was about, stated as what it is: (C) multiplies by enrolled ADM
/// because a per-pupil shortfall is not money until it does, and (B) does not because a difference
/// of two totals already is.
#[test]
fn the_pupil_count_follows_the_shortfall_rather_than_the_tier() {
    let code = statute::section("3317.0217").body.to_string();

    // (C)'s shortfall is per pupil, and ADM is the last factor in it.
    assert!(code.contains("X 0.0112)] X the district's enrolled ADM for that fiscal year"));
    // (B)'s is not, and (B)(4)(b)(i) ends at the subtraction.
    assert!(code.contains(
        "(The median weighted wealth of all school districts in this state for that fiscal year X \
         0.008) - (the district's weighted wealth for that fiscal year X 0.008)\n"
    ));

    // Enrolled ADM does appear in (B) — three times, all of them in the ramp's own brackets, and
    // never as a multiplier on the amount. That is what makes the ramp a size *provision* rather
    // than the size *term*.
    let capacity = code
        .split("(C) The department shall calculate")
        .next()
        .expect("the section has a division (C)");
    assert_eq!(capacity.matches("enrolled ADM").count(), 5);
    for phrase in [
        "The district's enrolled ADM is less than 200.",
        "greater than or equal to 200 but less than or equal to 400",
        "greater than 400 and less than 600",
        "(the district's enrolled ADM for that fiscal year - 400)/200",
        "greater than or equal to 600",
    ] {
        assert!(capacity.contains(phrase), "the ramp reads {phrase}");
    }
}

/// **The predecessor went to zero with its pupil count and (B) does not.**
///
/// What the ADM multiplier was doing that nothing in the successor does. Capacity aid was `x
/// Formula ADM`, so a district with almost no pupils drew almost nothing and its per-pupil
/// generosity was capped absolutely at `per-pupil amount x multiplier x 2.5`. (B)'s amount
/// approaches a whole-district constant instead, and no pupil count divides it.
#[test]
fn the_capacity_amount_has_a_positive_limit_at_no_pupils_at_all() {
    let districts = panel();
    assert_eq!(districts.len(), 609);

    // The limit, which is the amount (B) computes for a district of no wealth whatever.
    let limit = TA_CAPACITY_RATE * TA_MEDIAN_WEIGHTED_WEALTH;
    assert!(
        (limit - 3_137_210.45).abs() < CENT,
        "eight mills of the median district's weighted wealth is {limit:.2}"
    );

    // The smallest district in the panel, and what (B)(4)(b)(i) computes for it before the ramp.
    let smallest = districts
        .iter()
        .min_by(|a, b| a.base_cost_adm().total_cmp(&b.base_cost_adm()))
        .expect("the panel is not empty");
    assert_eq!(smallest.name, "Kelleys Island Local");
    assert!((smallest.base_cost_adm() - 5.24).abs() < CENT);

    let unramped = capacity_tier_unramped(smallest);
    assert!(
        (unramped - 2_461_241.79).abs() < CENT,
        "the unramped amount is {unramped:.2}"
    );
    // The tier and its ramp are both on **enrolled** ADM, which is the count this is per.
    assert!((smallest.current_year_adm - 3.51).abs() < CENT);
    let per_pupil = unramped / smallest.current_year_adm;
    assert!(
        (per_pupil - 701_208.49).abs() < 0.5,
        "which is {per_pupil:.2} a pupil"
    );

    // The ramp is the whole of what stands between that and the payment file.
    assert!(smallest.targeted_assistance.capacity_amount.abs() < CENT);

    // And it is not one district. The 17 the cliff and the shelf reach — nothing below 200
    // enrolled ADM, five per cent to 400 — would every one of them draw more per pupil than any
    // district the tier pays in full.
    let cliff: Vec<&_> = districts
        .iter()
        .filter(|d| d.current_year_adm <= TA_CAPACITY_RAMP_START)
        .collect();
    assert_eq!(cliff.len(), 17);
    assert_eq!(
        districts
            .iter()
            .filter(|d| d.current_year_adm < TA_CAPACITY_MINIMUM_ADM)
            .count(),
        5
    );

    let full_rate_ceiling = districts
        .iter()
        .filter(|d| TargetedAssistance::capacity_size_share(d.current_year_adm) >= 1.0)
        .map(|d| capacity_tier_unramped(d) / d.current_year_adm)
        .fold(0.0_f64, f64::max);
    assert!(
        (full_rate_ceiling - 3_773.82).abs() < 0.5,
        "the most any full-rate district would draw per pupil is {full_rate_ceiling:.2}"
    );
    for district in &cliff {
        let each = capacity_tier_unramped(district) / district.current_year_adm;
        assert!(
            each > full_rate_ceiling,
            "{} would draw {each:.2} a pupil",
            district.name
        );
    }

    // What the ramp holds back, statewide, which is the figure the node already carries.
    let withheld: f64 = districts
        .iter()
        .map(|d| {
            capacity_tier_unramped(d)
                * (1.0 - TargetedAssistance::capacity_size_share(d.current_year_adm))
        })
        .sum();
    assert!(
        (withheld - 91_859_433.06).abs() < CENT,
        "the ramp withholds {withheld:.2}"
    );
}

/// **The analysis that announces the merger says nothing about what it merged.**
///
/// "Capacity aid" appears once in H.B. 110's greenbook, in the sentence replacing it. The species
/// this corpus has now found three times: a change presented as a new formula rather than as a
/// moved part — the threshold in `fsfp-targeted-assistance`, the benchmark rank in
/// `local-capacity-percentage`, and the pupil count here.
#[test]
fn the_merger_is_announced_in_one_sentence_and_never_described() {
    let lsc = merged().flat();
    assert_eq!(merged().first_fiscal_year(), 2022);

    assert_eq!(
        lsc.matches("capacity aid").count() + lsc.matches("Capacity aid").count(),
        1,
        "the replaced component is named once in the whole analysis"
    );
    assert!(lsc.contains(
        "The budget replaces the targeted assistance and capacity aid components of the previous \
         formula with a targeted assistance payment consisting of a \u{201c}wealth amount\u{201d} \
         and a \u{201c}capacity amount.\u{201d}"
    ));

    // No account of the predecessor's arithmetic anywhere in it — neither the multiplier that
    // went, nor the cap, nor the mill the ratio was built on.
    for absent in [
        "Formula ADM x Capacity aid multiplier",
        "capacity ratio",
        "raise with one mill",
    ] {
        assert!(
            !lsc.to_lowercase().contains(&absent.to_lowercase()),
            "the analysis discusses {absent} after all and this reading needs redoing"
        );
    }
}

/// **And the change was drafted upstream of any budget.**
///
/// #392's closing method note, applied before recording a silence. H.B. 1 of the 134th repeals
/// R.C. 3317.0218 — capacity aid's section — and enacts a *new* R.C. 3317.0218 under the same
/// number for supplemental targeted assistance, in one document, with division (B) already reading
/// as it reads in force. A greenbook explains what a budget changed; this budget adopted a drafted
/// plan whole.
#[test]
fn the_bill_the_plan_was_drafted_in_repeals_and_reuses_the_same_section_number() {
    let bill = project::plan_bill::flat();

    // Capacity aid's section, named as the payment it was.
    assert!(
        bill.contains("Capacity aid funds calculated under section 3317.0218 of the Revised Code")
    );
    // Repealed and re-enacted under the same number, in the same title clause.
    assert!(bill
        .contains("to enact new sections 3314.085, 3317.017, 3317.0215, 3317.0217, and 3317.0218"));
    assert!(bill.contains("3317.0216, 3317.0217, 3317.0218, 3326.41, and 3328.33 of the Revised Code are hereby repealed"));
    assert!(bill.contains(
        "Sec. 3317.0218. For each fiscal year, the department of education shall compute and pay \
         supplemental targeted assistance"
    ));

    // And (B) is already the difference of two totals, with the ramp already in it.
    assert!(bill.contains(
        "(The median weighted wealth of all school districts in this state for that fiscal year X \
         0.008) \u{2013} (the district's weighted wealth for that fiscal year X 0.008)"
    ));
    assert!(bill.contains("The district's enrolled ADM is less than 200."));

    // The 0.008 pair is the only place a whole-district wealth is charged; (C)'s pair is per
    // pupil and carries the enrolled ADM factor, exactly as in force.
    assert!(bill.contains("X 0.0112)] X the district's enrolled ADM for that fiscal year"));
}
