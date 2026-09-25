//! How the deduct ran before the Fair School Funding Plan, and the one programme it never ran on.
//!
//! Five scholarship nodes carried the same `[open]`: the pre-FY2022 mechanism was "a deduction"
//! and the question of *how* it ran was deferred to `dew-payment-reports`, a source the repository
//! does not hold. It was deferred to the wrong kind of document. The payment reports are the
//! per-district amount; the mechanism is stated in prose, biennium by biennium, in the committed
//! LSC greenbooks — and it is not one mechanism but four.
//!
//! **Autism and Jon Peterson** kept the student in the resident district's ADM and took the
//! scholarship out of that district's state aid. **Cleveland ran the other way round**: its
//! students were *not* counted in Cleveland's ADM, and the deduct fell on a fixed earmark of the
//! district's own aid rather than following a pupil. **The EdChoice expansion was never a deduct
//! at all** — it was a line item from its first year. And **traditional EdChoice** deducted a flat
//! per-pupil amount until H.B. 153 changed it to the scholarship actually awarded.
//!
//! The reason to pin this rather than write it down once is
//! [`the_greenbooks_stop_describing_a_deduct_the_year_the_plan_starts`]: the census splits exactly
//! at the plan's first fiscal year, and that year is computed from the act's General Assembly
//! rather than typed. A sentence about "the deduct era" is only as good as the boundary it claims,
//! and this file is where the boundary is checked.

use project::greenbook::{greenbook, greenbooks};

/// The first fiscal year of the Fair School Funding Plan, from the act that enacted it.
///
/// Computed rather than written: [`project::greenbook::Greenbook::first_fiscal_year`] derives it
/// from H.B. 110's General Assembly, so a re-extraction that shifted a record would fail here
/// rather than quietly move the boundary the rest of this file is stated against.
fn the_plans_first_year() -> u16 {
    greenbook("hb110").first_fiscal_year()
}

/// LSC's sentence for the two special-needs programmes: the student stays in the resident
/// district's ADM and the scholarship comes out of that district's aid.
///
/// This is the half of the mechanism the nodes recorded as unknown. It is the same two sentences
/// in every analysis from FY2010 to FY2021 — only the cap moves.
#[test]
fn the_autism_deduct_kept_the_student_in_the_resident_districts_adm() {
    let analysis = greenbook("hb59");
    let passage = analysis.around("Foundation payments also support the Autism Scholarship", 6);

    assert!(
        passage.contains("counted in their district's ADM for the purposes of the state funding"),
        "the Autism ADM sentence has moved: {passage}"
    );
    assert!(
        passage.contains("the lesser of the total fees charged by the"),
        "the Autism amount sentence has moved: {passage}"
    );
    assert!(
        passage.contains("deducted from the resident district's state aid"),
        "the Autism deduction sentence has moved: {passage}"
    );
    assert!(
        passage.contains("paid to the alternative provider"),
        "the Autism transfer sentence has moved: {passage}"
    );
}

/// Jon Peterson was funded "in the same way", and LSC says so in those words.
///
/// The one difference is the ceiling: Autism's is a flat dollar cap, and Jon Peterson's is the
/// lesser of the provider's tuition and what the formula would have paid for the student — the
/// formula amount plus the special education amount for the disability category. So the two
/// programmes share a mechanism and not an amount, which is why the corpus holds them as separate
/// nodes describing one deduct.
#[test]
fn jon_peterson_was_the_same_transfer_with_a_different_ceiling() {
    let analysis = greenbook("hb59");
    let passage = analysis.around("same way as that of the Autism program", 5);

    assert!(
        passage.contains("through a transfer of state aid from the"),
        "the JPSN transfer sentence has moved: {passage}"
    );
    assert!(
        passage.contains("resident district to the alternate provider"),
        "the JPSN transfer sentence has moved: {passage}"
    );
    assert!(
        passage.contains("counted in their"),
        "the JPSN ADM sentence has moved: {passage}"
    );

    let ceiling = analysis.around("amount of the scholarship cannot exceed", 4);
    assert!(
        ceiling.contains("the lesser of the tuition charged"),
        "the JPSN ceiling sentence has moved: {ceiling}"
    );
    assert!(
        ceiling.contains("the formula amount plus the special education amounts"),
        "the JPSN ceiling sentence has moved: {ceiling}"
    );
}

/// Cleveland ran on the opposite convention, and every analysis that describes it says so.
///
/// The pilot project's students were **not** counted in Cleveland's ADM. That inverts the other
/// two programmes: there is no pupil in the formula for the deduct to follow, so the deduct is not
/// per-pupil at all — it is a fixed earmark of the district's aid, set by the budget act and
/// carried forward at its previous level. H.B. 166 puts the whole of it at $23.5 million a year,
/// $1.0 million of that for tutoring.
///
/// This is the item the node called "the deduct-era mechanism", and it is a different question
/// from the other two rather than the same one.
#[test]
fn cleveland_took_the_deduct_off_the_district_and_not_off_a_pupil() {
    let analysis = greenbook("hb166");
    let passage = analysis.around("Cleveland Scholarship and Tutoring Program is partially", 4);

    assert!(
        passage.contains("partially supported through a"),
        "the Cleveland mechanism sentence has moved: {passage}"
    );
    assert!(
        passage.contains("deduction from the foundation funding calculated for the Cleveland"),
        "the Cleveland mechanism sentence has moved: {passage}"
    );

    let earmark = analysis.around("budget earmarks $23.5 million", 3);
    assert!(
        earmark.contains("from CMSD"),
        "the Cleveland earmark sentence has moved: {earmark}"
    );
    assert!(
        earmark.contains("$1.0 million in each fiscal year from this earmark"),
        "the Cleveland tutoring set-aside has moved: {earmark}"
    );

    // The negation, in the same document, in LSC's own words.
    let not_counted = analysis.flat().replace('\u{2019}', "'");
    assert!(
        not_counted.contains("students generally are not counted in Cleveland's ADM"),
        "H.B. 166 no longer states the Cleveland ADM convention"
    );
}

/// "Partially" is load-bearing: Cleveland always had a second channel beside the deduct.
///
/// The FY2008-09 analysis itemises them separately — a GRF appropriation for the programme, and a
/// deduction that in that biennium came out of *poverty-based assistance* rather than out of
/// foundation funding generally. So the source of the deduct moved over the programme's life while
/// the two-channel shape did not, and a node that says "the deduct" without saying which aid it
/// came out of is under-specified for the early years.
#[test]
fn the_cleveland_deduct_came_out_of_poverty_based_assistance_first() {
    let analysis = greenbook("hb119");
    let passage = analysis.around("Pilot Project Scholarship Program, but comes through a", 3);

    assert!(
        passage.contains("comes through a deduction from the state poverty-based"),
        "the FY2008 Cleveland deduction sentence has moved: {passage}"
    );
    assert!(
        passage.contains("allocated to the Cleveland Municipal School District"),
        "the FY2008 Cleveland deduction sentence has moved: {passage}"
    );

    // And the convention was already the inverted one, twelve years before H.B. 166 restated it.
    let inverted = analysis.around("Scholarship students are not counted in Cleveland", 2);
    assert!(
        inverted.contains("ADM for funding purposes"),
        "the FY2008 Cleveland ADM sentence has moved: {inverted}"
    );
}

/// The EdChoice expansion was a line item from its first year and never a deduct.
///
/// LSC states it as a contrast rather than in passing — "paid for directly, not through a
/// deduction of school district foundation funding" — and draws the consequence the other
/// programmes make necessary: because there is no deduct, the students are not in anybody's ADM.
///
/// That matters to the corpus because the expansion is the programme whose appropriation lines the
/// `edchoice-expansion` node names. Those lines are the whole of its funding for the deduct era,
/// not a supplement to one.
#[test]
fn the_expansion_was_a_line_item_before_the_plan_made_everything_one() {
    let introduced = greenbook("hb59");
    let passage = introduced.around("described more below, is paid for directly", 4);
    assert!(
        passage.contains("not through a deduction of school district"),
        "the FY2014 expansion sentence has moved: {passage}"
    );
    assert!(
        passage.contains("are not counted in their resident district's ADM"),
        "the FY2014 expansion ADM sentence has moved: {passage}"
    );

    // And the line it was paid from, named, two bienniums later.
    let named = greenbook("hb64").around("The budget funds these scholarships directly from", 3);
    assert!(
        named.contains("GRF line item 200574, EdChoice Expansion"),
        "the FY2016 expansion line item has moved: {named}"
    );
}

/// Traditional EdChoice deducted a flat amount, and H.B. 153 made it the scholarship instead.
///
/// Before FY2012 the deduct was a price list — $2,700 for a kindergartener and $5,200 for everyone
/// else — and bore no relation to what the student's school was paid. A district lost $5,200 for a
/// scholarship that might be worth $4,200. H.B. 153 replaced the flat figure with "the actual
/// amount of the scholarship", which is the first time the deduct and the award are the same
/// number.
///
/// So "the deduct" names two different quantities depending on the year, and the change is inside
/// the span the corpus's own participation archive covers.
#[test]
fn the_traditional_deduct_was_a_flat_price_until_h_b_153() {
    let before = greenbook("hb119").around("Scholarship students are counted in their resident", 4);
    assert!(
        before.contains("ADM for the purposes of calculating state base cost funding"),
        "the FY2008 EdChoice ADM sentence has moved: {before}"
    );
    assert!(
        before.contains("A deduction of $2,700 for"),
        "the FY2008 EdChoice deduction sentence has moved: {before}"
    );
    assert!(
        before.contains("$5,200 for a student in grades one through twelve"),
        "the FY2008 EdChoice deduction sentence has moved: {before}"
    );

    let change =
        greenbook("hb153").around("deducted from a school district's state aid for each", 2);
    assert!(
        change.contains("from $5,200 to the actual"),
        "the FY2012 EdChoice change sentence has moved: {change}"
    );
    assert!(
        change.contains("amount of the scholarship"),
        "the FY2012 EdChoice change sentence has moved: {change}"
    );
}

/// The plan abolished the deduct, and LSC hedges the sentence that says so.
///
/// "The deduct-and-transfer method of financing **generally** in place for the scholarships prior
/// to FY 2022" — and footnote 1 says what the hedge covers: a portion of the Cleveland programme
/// and *all* income-based EdChoice scholarships were already direct state payments, appropriated
/// separately from foundation aid. The plan did not convert those; it folded two existing direct
/// appropriations into one.
///
/// A node that says the pre-plan mechanism "was a deduction" is therefore right about three
/// programmes and wrong about the fourth, which is why this file separates them.
#[test]
fn the_plan_folded_in_two_programmes_that_were_already_paid_directly() {
    let plan = greenbook("hb110");

    let transition = plan.around("under the budget, funding for all scholarship programs", 3);
    assert!(
        transition.contains("is paid directly, instead of counting most"),
        "the H.B. 110 transition sentence has moved: {transition}"
    );
    assert!(
        transition.contains("scholarship students in their resident districts and deducting"),
        "the H.B. 110 transition sentence has moved: {transition}"
    );

    let hedged = plan.around("the deduct-and-transfer method of financing generally", 2);
    assert!(
        hedged.contains("generally in place for the scholarships prior to"),
        "the H.B. 110 hedge has moved: {hedged}"
    );

    let footnote = plan.around(
        "Under prior law, direct state payments financed a portion",
        4,
    );
    assert!(
        footnote.contains("all income-based EdChoice"),
        "H.B. 110's footnote 1 has moved: {footnote}"
    );
    assert!(
        footnote.contains("appropriated separately from school"),
        "H.B. 110's footnote 1 has moved: {footnote}"
    );

    // What replaced it, and the redefinition that made a per-pupil deduct unnecessary.
    let adm = plan.around("for the purposes of state scholarship funding is the", 2);
    assert!(
        adm.contains("number of students receiving the scholarship"),
        "H.B. 110's Enrolled ADM definition has moved: {adm}"
    );
}

/// The census splits at the plan's first year, and the split is computed rather than asserted.
///
/// Every greenbook that states the special-needs mechanism states a deduct, and not one of them is
/// from the plan era; the same holds of Cleveland's inverted ADM convention. That is the whole
/// evidentiary basis for calling FY2010-FY2021 "the deduct era" in the corpus, and it is checked
/// here against [`the_plans_first_year`] rather than against a year somebody typed.
///
/// The three earliest analyses describe neither because the programmes they describe did not exist
/// yet: Autism began in FY2004 and Jon Peterson in FY2013, and LSC only began itemising the
/// mechanism when the amounts grew large enough to matter.
#[test]
fn the_greenbooks_stop_describing_a_deduct_the_year_the_plan_starts() {
    let plan = the_plans_first_year();
    assert_eq!(
        plan, 2022,
        "H.B. 110's General Assembly no longer resolves to FY2022"
    );

    let mut deducting = Vec::new();
    let mut inverting = Vec::new();
    for analysis in greenbooks() {
        let flat = analysis.flat().replace(['\u{2019}', '\u{02b9}'], "'");
        if flat.contains("deducted from the resident district's state aid") {
            deducting.push(analysis.first_fiscal_year());
        }
        if flat.contains("not counted in Cleveland's ADM") {
            inverting.push(analysis.first_fiscal_year());
        }
    }

    assert_eq!(deducting, vec![2010, 2012, 2014, 2016, 2018, 2020]);
    assert_eq!(inverting, vec![2008, 2010, 2012, 2014, 2016, 2018, 2020]);

    assert!(
        deducting.iter().all(|&year| year < plan),
        "a plan-era greenbook describes a scholarship deduct: {deducting:?}"
    );
    assert!(
        inverting.iter().all(|&year| year < plan),
        "a plan-era greenbook states the Cleveland ADM convention: {inverting:?}"
    );

    // Cleveland's convention is stated one biennium earlier than the special-needs deduct, and
    // that gap is the programme's age: the pilot project ran from 1996 and the other two were new.
    assert_eq!(inverting.len() - deducting.len(), 1);
    assert_eq!(inverting[0], deducting[0] - 2);
}
