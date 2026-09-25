//! The two ALEC provisions the model-policy node called unfamiliar, read against Ohio.
//!
//! [`parental-choice-scholarship-act`] recorded two provisions of ALEC's template voucher statute
//! as having "no established Ohio counterpart": the state keeping whatever a district would have
//! received in excess of the scholarship, and an award set from **state and local** per-pupil
//! support. Both were filed as open questions rather than absences, on the ground that the corpus
//! had not looked.
//!
//! It had the documents. The first provision was Ohio law — for two fiscal years, in the
//! traditional EdChoice programme, and H.B. 1 of the 128th General Assembly widened the residual
//! from both ends in the same paragraph that describes it. H.B. 153 abolished it. The second
//! provision has an Ohio counterpart of a different kind: the committed statute extract computes a
//! per-pupil amount out of "state and local" revenue exactly once, and it moves money inside a
//! district rather than out of one.
//!
//! The residual is the reason to pin this rather than write it down once. Nothing in the greenbook
//! names it — it is what is left when the deduct is larger than the award, stated in two different
//! sentences — so every figure it rests on is read back out of LSC's prose here rather than
//! written into this file, and a re-extraction that moved one of them fails rather than quietly
//! changing the answer.
//!
//! [`parental-choice-scholarship-act`]: ../../.yidam/corpus/model-policy/parental-choice-scholarship-act.yml

use project::greenbook::greenbook;
use project::statute;

/// The passage in which LSC states the deduct and the award for FY2010-11 EdChoice.
///
/// Wide enough to cross the page break the FY2009 maxima sit on the far side of.
fn the_edchoice_paragraph() -> String {
    greenbook("hb1").around("Scholarship students are counted in their resident", 20)
}

/// The one sentence of `passage` containing `anchor`.
///
/// Exactly one: an anchor matching twice means the paragraph has been re-extracted around a
/// different boundary, and taking the first match would hide that.
fn sentence(passage: &str, anchor: &str) -> String {
    let found: Vec<&str> = passage
        .split(". ")
        .filter(|part| part.contains(anchor))
        .collect();
    assert_eq!(
        found.len(),
        1,
        "{anchor:?} is not in exactly one sentence of: {passage}"
    );
    found[0].to_string()
}

/// Every whole-dollar amount a sentence states, in the order it states them.
///
/// LSC writes these as `$5,200`, and the order is the reading order of the grade bands, which is
/// what makes an unsorted list the right shape: K-8 before 9-12 in every sentence here.
fn dollars(text: &str) -> Vec<u32> {
    text.split('$')
        .skip(1)
        .filter_map(|rest| {
            let digits: String = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == ',')
                .filter(char::is_ascii_digit)
                .collect();
            digits.parse().ok()
        })
        .collect()
}

/// ALEC's "kept by the state" provision was Ohio law, and the arithmetic is LSC's own.
///
/// The model directs that "any aid the school district would have received for the student in
/// excess of the funds needed for a scholarship will be kept by the state". H.B. 1 set the
/// traditional EdChoice deduct at one figure for every scholarship and the maximum award below it
/// in both grade bands, so a district lost more than the scholarship could pay and the state kept
/// the difference — the model's provision, in Ohio, for FY2010 and FY2011.
///
/// The residuals are a floor in both bands: the maxima are ceilings, so every student whose
/// tuition fell below one widened the gap further.
#[test]
fn the_state_kept_the_difference_for_two_fiscal_years() {
    let passage = the_edchoice_paragraph();

    let deducts = dollars(&sentence(&passage, "Under the budget, the deduction is"));
    assert_eq!(
        deducts,
        vec![5_200],
        "the H.B. 1 EdChoice deduction sentence no longer states one figure: {passage}"
    );

    let maxima = dollars(&sentence(
        &passage,
        "decreases the maximum amount of the scholarships",
    ));
    assert_eq!(
        maxima,
        vec![4_200, 5_000],
        "the H.B. 1 EdChoice maximum-award sentence has moved: {passage}"
    );

    let residual: Vec<u32> = maxima.iter().map(|award| deducts[0] - award).collect();
    assert_eq!(
        residual,
        vec![1_000, 200],
        "the residual the two stated figures imply has changed"
    );
}

/// The same budget widened the residual from both ends, and one end had been negative.
///
/// Prior law deducted less for a kindergartener than for anyone else, and the maxima had been
/// indexed to the base cost since FY2007 — reaching figures by FY2009 that this test compares
/// against the deduct that replaced them. In the older band the award had been worth *more* than
/// the deduct paying for it, so the district's loss was smaller than the state's outlay. H.B. 1
/// raised the kindergarten deduct to the general figure and cut both maxima below it in one
/// instrument.
///
/// This is what makes the finding a finding rather than a coincidence of two numbers. The residual
/// did not drift into existence; a budget moved the deduct up and the award down at once.
#[test]
fn the_budget_that_created_the_residual_moved_both_figures() {
    let passage = the_edchoice_paragraph();

    let prior = dollars(&sentence(&passage, "Under prior law, a deduction of"));
    assert_eq!(
        prior,
        vec![2_700, 5_200],
        "the prior-law EdChoice deduction sentence has moved: {passage}"
    );

    let fy2009 = dollars(&sentence(&passage, "As a result of these annual increases"));
    assert_eq!(
        fy2009,
        vec![4_500, 5_300],
        "the FY2009 EdChoice maximum sentence has moved: {passage}"
    );

    let deduct = dollars(&sentence(&passage, "Under the budget, the deduction is"))[0];
    assert!(
        fy2009[1] > deduct,
        "the FY2009 9-12 maximum no longer exceeds the deduct that replaced it"
    );
    assert!(
        prior[0] < deduct,
        "the kindergarten deduct no longer rose under this budget"
    );
}

/// H.B. 153 abolished it, by making the deduct the award.
///
/// "Reduces the amount deducted from a school district's state aid for each scholarship from
/// $5,200 to the actual amount of the scholarship" — once the two quantities are the same number
/// there is no excess for the state to keep. The provision therefore has an Ohio span with both
/// ends: FY2010 through FY2011.
#[test]
fn h_b_153_abolished_the_residual_by_making_the_two_quantities_one() {
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

/// The inverse provision is two pages earlier, in the same document, for the other programme.
///
/// Where EdChoice let the state keep what the scholarship did not need, Cleveland returned it:
/// "Any funds that are not needed to cover the costs of the program are disbursed to CMSD." One
/// greenbook therefore states both dispositions of the same residual, for two scholarship
/// programmes running in the same year — so Ohio's answer to the model's provision is not one
/// answer, and a node that records only the EdChoice half is recording a choice as though it were
/// a rule.
#[test]
fn cleveland_returned_the_residual_the_same_budget_let_the_state_keep() {
    let passage =
        greenbook("hb1").around("Pilot Project Scholarship Program, through a deduction", 20);

    assert!(
        passage.contains("Any funds that are not needed to cover the costs"),
        "the H.B. 1 Cleveland residual sentence has moved: {passage}"
    );
    assert!(
        passage.contains("are disbursed to CMSD"),
        "the H.B. 1 Cleveland residual sentence has moved: {passage}"
    );
    assert!(
        passage.contains("maintains the total deduction at its FY 2009 level"),
        "the H.B. 1 Cleveland earmark sentence has moved: {passage}"
    );
}

/// The model's other provision: Ohio computes a per-pupil amount from state and local revenue
/// exactly once, and it is not a scholarship.
///
/// ALEC sets the award at the lesser of the school's cost per pupil or "the dollar amount the
/// resident school district would have received to serve and educate the eligible student from
/// state and local sources". The phrase appears once in the whole committed statute extract, in
/// R.C. 3302.12 — where a district that has replaced a failing building's principal and teaching
/// staff must fund that building at "the per pupil amount of state and local revenues received by
/// the district" times its enrolment.
///
/// So the basis exists in Ohio law and is aimed the other way: it moves money from a district to a
/// school the district still operates, on an accountability trigger, rather than out of the
/// district with a departing student. Every Ohio scholarship award is a flat statutory dollar
/// figure or a function of family income, and none of them carries a local term at all.
#[test]
fn the_only_state_and_local_per_pupil_amount_in_the_code_stays_inside_the_district() {
    let carrying: Vec<&str> = statute::sections()
        .into_iter()
        .filter(|section| section.body.contains("state and local"))
        .map(|section| section.number)
        .collect();

    assert_eq!(
        carrying,
        vec!["3302.12"],
        "the sections computing an amount from state and local revenue are no longer the one this \
         finding is stated against"
    );

    let section = statute::section("3302.12");
    assert!(
        section
            .body
            .contains("per pupil amount of state and local revenues received by the district"),
        "R.C. 3302.12's state-and-local payment clause has moved: {}",
        section.title
    );
    assert!(
        section.body.contains("distribute funding to the school"),
        "R.C. 3302.12 no longer directs the payment to a school the district operates"
    );
}
