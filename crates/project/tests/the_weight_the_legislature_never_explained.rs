//! Where the 2.0 on a non-public rider came from, and who — if anyone — said why.
//!
//! `formula-component/fsfp-transportation` carries
//! `public + 2.0 × non-public + 1.5 × community or STEM` with the justification for the 2.0 marked
//! `[open]`. The transportation tests measure the weight's effect; none of them asks where it came
//! from. Four committed documents answer it, and they answer it in two different registers.
//!
//! # It is not inherited. The predecessor was a different instrument, and it was repealed
//!
//! Ohio has compensated this cost under three formulas and never with the same device twice.
//! H.B. 1 of the 128th General Assembly paid a **nontraditional rider adjustment**,
//! `(nontraditional riders / total riders) × 0.1 × base payment` — a proportional uplift on the
//! whole payment, with community school and non-public riders in one undifferentiated count.
//! H.B. 59 **removed** it for FY2014, in as many words, and no formula between then and FY2021
//! carried anything in its place. So there was nothing to inherit: eight years separate the
//! repeal of the old device from the enactment of the new one, and the old one did not
//! distinguish a non-public rider from a community school rider at all.
//!
//! # It originated with the Fair School Funding Plan, already at 2.0
//!
//! H.B. 1 of the 134th General Assembly — the plan as Callender and Sweeney introduced it in
//! February 2021, four months before H.B. 110 enacted the formula — carries both weights verbatim
//! in R.C. 3317.0212(E)(1), and the bill's own strike-and-insert shows what they replaced:
//! *"(1) Multiply Calculate the sum of the following:"*. Prior law multiplied one rate by one
//! count. The plan turned that product into a weighted sum, and the 2.0 was in the first draft
//! the corpus holds. An un-enacted bill cannot establish what the law is; what it establishes is
//! drafting, which is exactly the question.
//!
//! # No legislative analysis gives a reason — and the weights are mentioned twice in fifteen
//!
//! Across twelve greenbooks, the H.B. 96 greenbook, the executive-budget redbook and the enacted
//! comparison document, the transportation weights appear **twice**, both in H.B. 110's
//! greenbook: once as item (4) of a six-item list of changes, and once inside a calculation box.
//! Neither says why. H.B. 33 and H.B. 96 do not restate them. The same document discusses
//! non-public transportation at length three pages later — pickup windows, mass transit
//! transfers, compliance deductions — and never connects any of it to the multiple.
//!
//! # The department gives one, five times, and it is about cost
//!
//! The administering agency is not silent. Every edition of the line-by-line explanation that
//! describes this formula — FY2022 through FY2026 — says:
//!
//! > Community school, STEM school and non-public riders are weighted to reflect the typically
//! > higher costs that districts incur to transport these students.
//!
//! That is a stated rationale from the payer, repeated without variation for five years. It is
//! also **one sentence for both weights**. It explains why a premium exists; it does not explain
//! why the premium for a non-public rider is a third larger than the premium for a community
//! school rider. So the `[open]` closes on the *existence* of the weight and narrows to its
//! *size* — which no committed document addresses, and which the plan bill already carried.
//!
//! # A by-catch: the plan bill settles a schedule the corpus had been inferring
//!
//! The node reasoned that the minimum state share's published values are twenty-fourths in equal
//! annual steps, "one schedule set across two budget acts rather than five separate decisions",
//! and marked it `[inference]`. The plan bill writes all six years out as fractions in one
//! clause, in February 2021, before any of the three acts that enacted them. The schedule begins
//! a year earlier than the inference supposed — at **seven** twenty-fourths in FY2022, not eight
//! in FY2023 — and the rounding runs the other way from what a reader would guess: the bill wrote
//! `forty-five and five-sixths`, and the enacted section rounds it to
//! `forty-five and eighty-three hundredths`.

use edfund_core::records::records;
use project::act;
use project::greenbook::{greenbook, greenbooks};
use project::plan_bill;
use project::statute;

/// LSC's FY2026-27 analysis, committed apart from the twelve. See `ledger::budget_analysis`.
const HB96_GREENBOOK: &str = include_str!("../fixtures/dew-greenbook.txt");

/// LSC's analysis of the executive budget proposal for the same biennium.
const HB96_REDBOOK: &str = include_str!("../fixtures/dew-redbook.txt");

/// LSC's provision-by-provision comparison document for the enacted act.
const HB96_ENACTED: &str = include_str!("../fixtures/enacted-school-funding.txt");

/// The department's own line-by-line specification of the payment report, one record per year.
const SFPR: &str = include_str!("../fixtures/dew-sfpr-line-by-line.txt");

/// The editions that explain the Fair School Funding Plan's transportation component.
///
/// FY2021 is excluded because it explains H.B. 166's formula, which has no weights — the same
/// five the 180-day check in `the_department_explains_its_own_method` runs over.
const FSFP_EDITIONS: &[&str] = &[
    "sfpr-line-by-line-fy22",
    "sfpr-line-by-line-fy23",
    "sfpr-line-by-line-fy24",
    "sfpr-line-by-line-fy25",
    "sfpr-line-by-line-fy26",
];

/// The department's sentence, as it stands in every edition that carries it.
const THE_DEPARTMENTS_REASON: &str = "Community school, STEM school and nonpublic riders are \
                                      weighted to reflect the typically higher costs that \
                                      districts incur to transport these students.";

/// One edition of the line-by-line explanation, flattened.
fn edition(id: &str) -> String {
    records(SFPR)
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("{id} is not in the committed extract"))
        .body
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whitespace collapsed, which is what a quotation out of a PDF has to be checked against.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Division (E)(1) of the plan bill's R.C. 3317.0212 — the weighted sum and nothing else.
///
/// Bounded at the mile base, which is the next numbered item, so the efficiency and density
/// divisions stay out of a search that is about what the weights are justified by.
fn weighted_sum_clause() -> String {
    let section = plan_bill::section("3317.0212").expect("the plan bill enacts R.C. 3317.0212");
    let from = section
        .find("(1) Multiply Calculate the sum of the following:")
        .expect("the weighted sum's own stem");
    let to = section[from..]
        .find("(2) Multiply the statewide transportation cost per mile")
        .expect("the mile base follows it");
    section[from..from + to].to_string()
}

/// **The weights are new text in the plan bill, and they replaced a bare product.**
///
/// The bill prints prior law struck and new law inserted, run together in the extract: the stem
/// reads "Multiply Calculate the sum of the following", where "Multiply" is the verb being
/// removed. Prior law took one rate times one count; the plan made it a weighted sum of three.
#[test]
fn the_plan_bill_wrote_both_weights_over_an_unweighted_product() {
    let section = plan_bill::section("3317.0212").expect("the plan bill enacts R.C. 3317.0212");

    assert!(
        section.contains("(1) Multiply Calculate the sum of the following:"),
        "the strike-and-insert that turns a product into a sum is the evidence that the weights \
         are new: prior law multiplied one rate by one count"
    );
    assert!(
        section.contains(
            "(c) 2.0 times the statewide transportation cost per student times the number of \
             students counted in the district's qualifying ridership for the current fiscal year \
             who are enrolled in nonpublic schools."
        ),
        "the 2.0 is in the plan as introduced, four months before H.B. 110 enacted it"
    );
    assert!(
        section.contains(
            "(b) 1.5 times the statewide transportation cost per student times the number of \
             students counted in the district's qualifying ridership for the current fiscal year \
             who are enrolled in community schools"
        ),
        "and so is the 1.5, in the clause immediately before it"
    );
}

/// **Nothing in the weighted-sum clause says why**, and the clause is where it would be said.
///
/// Scoped to division (E)(1) rather than the section: (F) directs the department to adjust the
/// efficiency target "to reflect" a district's density, which is a reason for a different
/// parameter and would make a section-wide word search meaningless.
#[test]
fn the_plan_bill_states_no_reason_for_either_multiple() {
    let clause = weighted_sum_clause();

    for tell in [
        "because",
        "cost of transporting",
        "reflect",
        "in order to",
        "higher",
    ] {
        assert!(
            !clause.to_lowercase().contains(tell),
            "the weighted sum as the plan bill wrote it contains {tell:?}, which would be a \
             reason this test has been asserting is absent"
        );
    }
}

/// **The enacting analysis lists the weights as a change and leaves it there.**
///
/// Item (4) of six. A legislator reading H.B. 110's greenbook was told the weights exist and what
/// they are, twice, and given no ground for either number.
#[test]
fn the_enacting_analysis_names_the_weights_without_a_reason() {
    let hb110 = greenbook("hb110").flat();

    assert!(
        hb110.contains(
            "The budget makes numerous changes to the prior formula for transportation \
                        funding."
        ),
        "the change list is where the weights are introduced"
    );
    assert!(
        hb110.contains(
            "(4) applying weights of 1.5 for community school students and 2.0 for nonpublic \
             school students transported by a district, (5) creating an efficiency"
        ),
        "item (4) of six, stated and not explained"
    );
    assert!(
        hb110.contains("Statewide cost per nonpublic school rider = Statewide cost per rider x 2"),
        "and restated inside the calculation box, still without a reason"
    );
}

/// **The same document handles non-public transportation at length and never joins the two.**
///
/// Three pages after the calculation box, H.B. 110's analysis has a heading of its own for
/// community and non-public transportation — pickup windows, mass transit transfers, compliance
/// deductions. It is regulatory throughout. The act treated the duty and the price of the duty as
/// unrelated subjects.
#[test]
fn the_analysis_treats_the_duty_and_the_weight_as_separate_subjects() {
    let hb110 = greenbook("hb110").flat();

    assert!(
        hb110.contains(
            "The budget contains various provisions that facilitate the transportation of \
             community school and nonpublic school students."
        ),
        "the non-public transportation heading is in the same analysis"
    );

    let at = hb110
        .find("The budget contains various provisions that facilitate")
        .expect("the heading was just asserted");
    let rest = &hb110[at..];
    let to = rest
        .find("Payments in lieu of transportation")
        .expect("the next heading bounds the passage");
    assert!(
        !rest[..to].contains("weight"),
        "the passage about transporting non-public pupils never mentions the weight that pays \
         for it"
    );
}

/// **The predecessor was a proportional uplift that did not distinguish the two populations.**
///
/// H.B. 1 of the 128th General Assembly, the last formula before the plan to pay anything for
/// these riders at all: one tenth of the base payment, scaled by the non-traditional share of
/// ridership, with non-public and community school pupils in a single count.
#[test]
fn the_predecessor_formula_used_a_different_instrument_and_one_population() {
    let hb1 = greenbook("hb1");
    assert_eq!(
        hb1.general_assembly, 128,
        "the `hb1` record is the 128th General Assembly's act, not the plan bill"
    );

    let flat = hb1.flat();
    assert!(
        flat.contains(
            "Nontraditional riders are nonpublic or community school students who are transported \
             by their resident school district."
        ),
        "one definition covering both populations"
    );
    assert!(
        flat.contains(
            "(Number of nontraditional riders) / (Total number of riders) x 0.1 x base payment"
        ),
        "and one rate for both — a share of the payment, not a weight on a rider"
    );
    assert!(
        !flat.contains("2.0 times") && !flat.contains("weights of"),
        "no per-rider multiple anywhere in the predecessor formula"
    );
}

/// **And it was repealed for FY2014, eight years before the plan's weights took effect.**
///
/// H.B. 59 says so in as many words, and restates it in the line item note. Nothing carried
/// through: the FY2014 formula paid on the greater of the two bases and nothing else.
#[test]
fn the_predecessor_device_was_removed_and_nothing_replaced_it_until_the_plan() {
    let hb59 = greenbook("hb59").flat();

    assert!(
        hb59.contains(
            "by removing the formula adjustments for (1) nontraditional ridership, (2) high \
             school ridership,"
        ),
        "H.B. 59 removes the non-traditional rider adjustment"
    );
    assert!(
        hb59.contains(
            "As a result, funding is based only on the greater of per rider or per mile costs for \
             each district."
        ),
        "and states what is left, which is the two bases and nothing else"
    );

    // The bienniums between the repeal and the plan. `mentions` is case-insensitive and the whole
    // document is searched, so a re-enactment under any heading would fail here.
    for bill in ["hb64", "hb49", "hb166"] {
        assert!(
            !greenbook(bill).mentions("nontraditional rider"),
            "{bill}: the adjustment was not revived, so there was nothing for the plan to inherit"
        );
    }
}

/// **The weights are mentioned twice in every legislative analysis this corpus holds.**
///
/// Fifteen documents: twelve greenbooks, H.B. 96's greenbook, the redbook for the same biennium,
/// and the enacted comparison. Both mentions are in H.B. 110's. H.B. 33 and H.B. 96 amended this
/// section — the minimum state share moves in each — and neither restated the multiples.
#[test]
fn no_committed_legislative_analysis_but_one_mentions_the_weights_at_all() {
    let carriers: Vec<&str> = greenbooks()
        .into_iter()
        .filter(|g| {
            let flat = g.flat();
            flat.contains("2.0 for nonpublic school students")
                || flat.contains("Statewide cost per nonpublic school rider")
        })
        .map(|g| g.bill)
        .collect();
    assert_eq!(
        carriers,
        vec!["hb110"],
        "one of twelve greenbooks mentions the transportation weights"
    );

    for (name, text) in [
        ("H.B. 96 greenbook", HB96_GREENBOOK),
        ("H.B. 96 redbook", HB96_REDBOOK),
        ("the enacted comparison", HB96_ENACTED),
    ] {
        let flat = flat(text);
        assert!(
            !flat.contains("nonpublic school rider") && !flat.contains("2.0 for nonpublic"),
            "{name} mentions the weights, which would make H.B. 110's analysis not the only one"
        );
    }
}

/// **The department states a reason, in five consecutive editions, without a word changing.**
///
/// This is what closes the `[open]`, so it is asserted on every edition that should carry it
/// rather than on the one that was read first — the FY2022 edition spells the noun "milage" and
/// the four after it "mileage", which is the kind of drift that makes a single-edition quotation
/// a hostage.
#[test]
fn the_department_states_the_reason_in_every_edition_that_explains_the_formula() {
    for id in FSFP_EDITIONS {
        assert!(
            edition(id).contains(THE_DEPARTMENTS_REASON),
            "{id}: does not carry the department's rationale for the rider weights"
        );
    }

    assert!(
        !edition("sfpr-line-by-line-fy21").contains("weighted to reflect"),
        "FY2021 explains H.B. 166's formula, which has no weights — its silence is about the year"
    );
}

/// **One sentence for both weights, which is why the size is still unexplained.**
///
/// The department's reason is a cost claim about a class of riders — community, STEM and
/// non-public together. A rationale that covers 1.5 and 2.0 with one clause cannot be a rationale
/// for the difference between them, and no committed document states one.
#[test]
fn the_stated_reason_covers_both_weights_and_so_explains_neither_size() {
    let fy26 = edition("sfpr-line-by-line-fy26");
    let at = fy26
        .find(THE_DEPARTMENTS_REASON)
        .expect("the sentence was just asserted to be in this edition");
    let sentence = &fy26[at..at + THE_DEPARTMENTS_REASON.len()];

    for population in ["Community school", "STEM school", "nonpublic"] {
        assert!(
            sentence.contains(population),
            "the reason names {population}, so it is one reason for all three"
        );
    }
    assert!(
        !sentence.contains("2.0") && !sentence.contains("1.5"),
        "and it names neither multiple, so it does not reach the size of either"
    );

    // Nowhere else does the department put a number to the reason.
    for id in FSFP_EDITIONS {
        let body = edition(id);
        assert!(
            !body.contains("2.0 times") && !body.contains("twice the statewide"),
            "{id}: states the multiple explicitly, which would be a place the size could be \
             argued for"
        );
    }
}

/// **The mile leg of the weights is younger than the rider leg**, which no source dates.
///
/// The node reads R.C. 3317.0212(E)(1)(b) as putting the same two multiples on miles driven, and
/// in the section as it stands that is exactly right. It was not always. The plan bill's mile base
/// is a bare product, and H.B. 583 — the plan's own corrections act, September 2022 — reprints the
/// section with the mile base still bare, at the same address `(E)(1)(b)` the weighted version now
/// occupies. So the 2.0 was legislated onto the rider base in 2021 and onto the mile base in some
/// later act, and no committed analysis mentions the second occasion at all.
#[test]
fn the_mile_base_was_unweighted_in_both_earlier_texts_and_is_weighted_now() {
    let plan = plan_bill::section("3317.0212").expect("the plan bill enacts R.C. 3317.0212");
    assert!(
        plan.contains(
            "(2) Multiply the statewide transportation cost per mile by the district's total \
             number of miles driven for school bus service in the current fiscal year."
        ),
        "the plan bill's mile base is one rate times one count"
    );

    let hb583 = act::flat(act::HB583);
    assert!(
        hb583.contains(
            "(b) Multiply the statewide transportation cost per mile by the district's total \
             number of miles driven for school bus service in the current fiscal year."
        ),
        "and so is H.B. 583's, fourteen months after the plan was enacted — at the address the \
         weighted version now occupies"
    );

    let now = flat(statute::section("3317.0212").body);
    assert!(
        now.contains(
            "(iii) 2.0 times the statewide transportation cost per mile times the number of miles \
             driven for school bus service as reported for qualifying riders for the current \
             fiscal year who are enrolled in nonpublic schools."
        ),
        "the section in force weights the miles, so the change happened after H.B. 583"
    );
}

/// **The whole minimum-share schedule was written in one clause, in February 2021.**
///
/// The node inferred that the published values are consecutive twenty-fourths and therefore one
/// decision rather than five. They are, and the bill writes them as fractions rather than as
/// percentages, which is how a drafter writes a series: seven twenty-fourths in FY2022 through
/// twelve in FY2027, four years before the last of the three acts that enacted them.
#[test]
fn the_plan_bill_already_carried_every_year_of_the_minimum_share_schedule() {
    let section = plan_bill::section("3317.0212").expect("the plan bill enacts R.C. 3317.0212");

    // (a) and (b) are amendments, so prior law's struck figure runs into the inserted one.
    for fraction in [
        "twenty-nine and one-sixth per cent",  //  7/24, FY2022
        "thirty-three and one-third per cent", //  8/24, FY2023
        "thirty-seven and one-half per cent",  //  9/24, FY2024
        "forty-one and two-thirds per cent",   // 10/24, FY2025
        "forty-five and five-sixths per cent", // 11/24, FY2026
        "fifty per cent",                      // 12/24, FY2027
    ] {
        assert!(
            section.contains(fraction),
            "the plan bill does not carry {fraction:?}, so the schedule was not written at once"
        );
    }

    // And the enacted section rounds the eleventh twenty-fourth rather than carrying the fraction.
    let now = flat(statute::section("3317.0212").body);
    assert!(
        now.contains("forty-five and eighty-three hundredths per cent"),
        "the section in force states FY2026 as a rounded decimal where the bill wrote a fraction"
    );
    assert!(
        !now.contains("five-sixths"),
        "and does not carry the fraction alongside it"
    );
}
