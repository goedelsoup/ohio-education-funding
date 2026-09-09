//! Every scholarship amount, read off the sections the corpus already holds.
//!
//! `program/edchoice-expansion` records `amount: Not yet populated. [open] Varies by grade band
//! and family income`, and separately "Exact bands not yet established. [open]". Both point at
//! R.C. 3310.08 and R.C. 3317.022, and both have been in `crates/project/fixtures/revised-code.txt`
//! since the extract was wired. `the_statute_behind_the_weights.rs` asserts that 3310.08 is one of
//! the sections held and reads nothing out of it.
//!
//! # The premise of the question is wrong, which is the finding
//!
//! There are no bands. H.B. 33 replaced eligibility gating with a **continuous function of family
//! income**: at or below 450% of the federal poverty guidelines the full base amount, and above it
//! an exponential decay that halves the award for every further hundred percentage points until it
//! reaches a tenth of the base. A node waiting for a table of bands would wait forever.
//!
//! # What the sections give and what they do not
//!
//! They give every ceiling in the channel — two for EdChoice, one for the pilot project, one for
//! autism, and a base plus six category supplements under a ceiling for Jon Peterson — and the
//! indexing rule attached to each. They do not give the dollar value of 450% of poverty, which
//! depends on family size and which the statute reaches by pointing at R.C. 5101.46. So the award
//! is stated here as a function of the income *ratio*, which is what R.C. 3310.08 itself does.

use project::scholarship::{self, EDCHOICE_BASE_9_12, EDCHOICE_BASE_K8};
use project::statute;

/// R.C. 3317.022's text, flattened enough to search across the publisher's line breaks.
fn funding_section() -> String {
    statute::section("3317.022")
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The same for the award calculation section.
fn award_section() -> String {
    statute::section("3310.08")
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The base amounts, the two grade bands, and what they are a ceiling on.
///
/// Division (A)(10)(a) is a **lesser-of**: the school's base tuition net of any discounts the
/// student qualifies for, or the amount below. So the figure a node quotes as "the award" is a
/// ceiling and the published average award is not evidence about it without knowing tuition.
#[test]
fn the_edchoice_ceiling_is_two_amounts_and_two_grade_bands() {
    let section = funding_section();
    assert!(section.contains(
        "$5,500, if the student is in grades kindergarten through eight, or $7,500, if the \
         student is in grades nine through twelve"
    ));
    assert!(
        section.contains(
            "(a) For each student in the funding unit's enrolled ADM, determine the lesser of the \
             following: (i) The base tuition of the chartered nonpublic school in which the \
             student is enrolled minus the total amount of any applicable tuition discounts"
        ),
        "the EdChoice amount is a ceiling on net tuition, not a grant"
    );
    assert!((EDCHOICE_BASE_K8 - 5_500.0).abs() < f64::EPSILON);
    assert!((EDCHOICE_BASE_9_12 - 7_500.0).abs() < f64::EPSILON);

    // Two grade bands and no others — nine grades against four, which is what makes the split
    // coarse. The phrase appears twice because the pilot project unit carries the same two.
    assert_eq!(
        section.matches("grades kindergarten through eight").count(),
        2
    );
}

/// The voucher ceiling is pegged to the public formula's own base cost.
///
/// A structural fact the corpus does not hold anywhere: the scholarship maximum is not a number
/// the General Assembly revisits, it is a number that rises with the statewide average base cost
/// per pupil. Raising base cost raises every EdChoice and pilot-project award ceiling with it.
#[test]
fn the_ceiling_rises_with_the_statewide_average_base_cost_per_pupil() {
    let section = funding_section();
    let indexed = "shall increase in future fiscal years by the same percentage that the \
                   statewide average base cost per pupil increases in future fiscal years";
    assert_eq!(
        section.matches(indexed).count(),
        3,
        "three amounts are indexed to base cost: the EdChoice ceiling, the pilot project's, and \
         the Jon Peterson base"
    );

    // The two $34,000 ceilings are not among them, and the way to show it is that every
    // indexing clause in the section names the division it indexes, and neither names these.
    assert!(section.contains("(ii) $34,000."), "the autism ceiling");
    assert!(
        section.contains("(iii) $34,000."),
        "the Jon Peterson ceiling"
    );
    for clause in [
        "The amounts specified in division (A)(10)(a)(ii)(I) of this section shall increase",
        "The amounts specified in division (A)(11)(a)(ii) of this section shall increase",
        "The amount specified in division (A)(13)(a)(ii) of this section shall increase",
        "The amounts specified in divisions (A)(13)(a)(ii)(I) to (VI) of this section shall \
         increase",
    ] {
        assert!(
            section.contains(clause),
            "an indexing clause moved: {clause}"
        );
    }
    assert_eq!(
        section
            .matches("shall increase in future fiscal years")
            .count(),
        4,
        "four indexing clauses, and none of them names (A)(12) or (A)(13)(a)(iii)"
    );
    for unindexed in ["(A)(12)", "(A)(13)(a)(iii)"] {
        assert!(
            !section.contains(&format!(
                "division {unindexed} of this section shall increase"
            )),
            "{unindexed} gained an indexing clause and this reading needs redoing"
        );
    }
}

/// There are no income bands. There is a curve, and it is one line of arithmetic.
///
/// R.C. 3310.08 writes it in four defined terms and two branches. Read back:
/// `award = base * 0.5^(income/poverty - 4.5)`, floored at a tenth of the base. The award halves
/// for every further hundred percentage points of family income relative to poverty.
#[test]
fn the_expansion_award_is_a_continuous_curve_and_not_a_schedule_of_bands() {
    let section = award_section();
    assert!(section.contains("\"Constant multiplier\" means 0.50."));
    assert!(section.contains(
        "\"Power equation\" means the following formula: The federal poverty level multiplier X \
         ln(constant multiplier)"
    ));
    assert!(section.contains(
        "\"Minimum amount\" means an amount equal to the student's base amount multiplied by ten \
         per cent"
    ));
    assert!(section.contains(
        "at or below four hundred fifty per cent of the federal poverty guidelines for the fiscal \
         year, the base amount"
    ));
    assert!(
        section.contains("The base amount X (1 / the constant multiplier)^4.5 X e^power equation")
    );

    // The section states no band, threshold or table beyond one boundary and one floor. "Per
    // cent" appears three times: the floor's ten, and the 450% boundary once in each of (B)'s two
    // branches — which is the whole of the section's structure.
    assert_eq!(section.matches("per cent").count(), 3);
    assert_eq!(section.matches("four hundred fifty per cent").count(), 2);
    assert_eq!(section.matches("ten per cent").count(), 1);

    // And the curve, at the points a reader would quote.
    let schedule = [
        (4.5, 5_500.00),
        (5.5, 2_750.00),
        (6.5, 1_375.00),
        (7.5, 687.50),
        (8.0, 550.00),
    ];
    for (ratio, expected) in schedule {
        let award = scholarship::expansion_award(EDCHOICE_BASE_K8, ratio);
        assert!(
            (award - expected).abs() < 0.005,
            "at {ratio:.1}x poverty the award is {award:.2}, not {expected:.2}"
        );
    }
}

/// Where the floor binds, which the statute states as an amount and never as an income.
#[test]
fn the_award_stops_falling_at_seven_hundred_and_eighty_two_per_cent_of_poverty() {
    let threshold = scholarship::minimum_award_threshold();
    assert!(
        (threshold * 100.0 - 782.19).abs() < 0.01,
        "the floor binds at {:.2}% of poverty",
        threshold * 100.0
    );
    // Past it, income is no longer an input to the award at all.
    assert!(
        (scholarship::expansion_award(EDCHOICE_BASE_9_12, 8.0)
            - scholarship::expansion_award(EDCHOICE_BASE_9_12, 80.0))
        .abs()
            < 1e-9
    );
}

/// Four programmes, four ceilings, and four different things they are a ceiling on.
///
/// The differences are not cosmetic. The pilot project nets out **all** financial aid, discounts
/// and adjustments; EdChoice nets out tuition discounts only; autism reads the tuition of the
/// special education programme; Jon Peterson reads the provider's fees. Two programmes sharing a
/// dollar ceiling do not therefore pay alike.
#[test]
fn every_programme_is_a_lesser_of_and_the_lesser_thing_differs() {
    let section = funding_section();
    assert!(section.contains(
        "the net tuition and fees charged to a student shall be the tuition amount specified by \
         the alternative school minus all other financial aid, discounts, and adjustments"
    ));
    assert!(section.contains(
        "(i) The tuition charged for the student's special education program, as that term is \
         defined in section 3310.41 of the Revised Code; (ii) $34,000."
    ));
    assert!(section.contains(
        "(i) The amount of fees charged for that school year by the student's alternative public \
         provider or registered private provider"
    ));

    // The pilot project shares EdChoice's ceiling and not its measure of tuition.
    assert_eq!(
        section
            .matches("$5,500, if the student is in grades kindergarten through eight")
            .count(),
        2,
        "the educational choice unit and the pilot project unit both carry these amounts"
    );
}

/// The Jon Peterson formula exceeds its own ceiling, before any indexing.
///
/// The award is the least of the provider's fees, `$7,190 + a category supplement`, and $34,000.
/// For category six the middle term is **$39,122**, so the ceiling binds by $5,122 for the most
/// severely disabled students in the programme — and the ceiling is the one figure in the whole
/// division with no indexing clause, while the base and the supplements each have one.
///
/// `program/jon-peterson-special-needs` holds the base and the first supplement and describes the
/// rest as "other amounts for other categories". This is the rest, and it is why the ceiling
/// matters.
#[test]
fn the_jon_peterson_ceiling_already_binds_the_severest_category() {
    let section = funding_section();
    for (division, amount) in [
        ("(I)", "$2,855"),
        ("(II)", "$5,879"),
        ("(III)", "$12,879"),
        ("(IV)", "$16,890"),
        ("(V)", "$22,560"),
        ("(VI)", "$31,932"),
    ] {
        assert!(
            section.contains(&format!("of the Revised Code, {amount};"))
                || section.contains(&format!("of the Revised Code, {amount}.")),
            "{division}'s supplement is no longer {amount}"
        );
    }
    assert!(section.contains("(ii) $7,190 plus an amount determined as follows"));

    assert!((scholarship::jon_peterson_award(5) - 29_750.0).abs() < 1e-9);
    assert!((scholarship::jon_peterson_award(6) - 34_000.0).abs() < 1e-9);

    // The supplements are indexed to the special education categories, the base to base cost —
    // two rules inside one award, and the ceiling under neither.
    assert!(section.contains(
        "shall increase in future fiscal years by the same percentage that the amounts calculated \
         by the general assembly for those categories of special education services under \
         division (A)(3) of this section increase"
    ));
}
