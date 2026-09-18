//! R.C. 3317.017 against R.C. 3317.021, which is the section it names for every input it uses.
//!
//! # The question this was written to settle
//!
//! The corpus recorded H.B. 96 as having **changed the income term in local capacity** from
//! residents' total federal adjusted gross income to their median, on the strength of the act's
//! own list of changes: *"Requires the Tax Commissioner to certify the median federal adjusted
//! gross income of a district's residents for use in making computations for the district,
//! instead of the total federal adjusted gross income of residents as under prior law."*
//!
//! `crates/local-capacity` blends **both** — 60% valuation, 20% aggregate AGI, 20% median income
//! times returns — so either the crate was a biennium out of date or the corpus had read the act
//! wrongly. State share is a subtraction against base cost, so the answer moves every figure
//! downstream of it, and settling it was worth a fetch.
//!
//! # It is the second, and the reason is that item 9 is not about R.C. 3317.017 at all
//!
//! The income structure of R.C. 3317.017 is **identical** in all three enactments of the plan:
//! H.B. 110 (30 September 2021), H.B. 33 (3 October 2023) and H.B. 96 (30 September 2025) each
//! carry an aggregate term at (A)(2) and a median-times-returns term at (A)(3), weighted 0.20 and
//! 0.20 against valuation's 0.60. H.B. 96 did not touch either. The crate is the statute.
//!
//! What item 9 amends is **R.C. 3317.021**, the Tax Commissioner's certification duty — a section
//! R.C. 3317.017 names four times — once for each of the four income inputs the blend uses —
//! and this corpus had never fetched.
//!
//! # And reading it turns the question inside out
//!
//! R.C. 3317.017(A)(3) has required, since the plan was enacted, two quantities *"as certified
//! under section 3317.021 of the Revised Code"*: the district's **median federal** AGI, and the
//! **number of state tax returns** filed in it. Until 30 September 2025, R.C. 3317.021 certified
//! **neither**. Its division (A)(5) named the *total* federal AGI and the *median **Ohio*** AGI,
//! and no division named a count of returns at all.
//!
//! For four years and two budget cycles, then, a fifth of the local capacity blend rested on a
//! certification that did not exist in law. That is the explanation for an error this corpus
//! recorded against itself and could not account for: an earlier pass inferred the median term
//! from the district's **Ohio** median income, and the guess fit to within a few percent. It was
//! not a coincidence. Ohio median AGI is what R.C. 3317.021 actually certified.
//!
//! H.B. 96 closed that gap by **substitution** rather than addition, so it opened the mirror of
//! it: R.C. 3317.017(A)(2) still reads the *total* federal AGI "as certified under section
//! 3317.021", and R.C. 3317.021 no longer certifies it. The dangling reference moved from the
//! median term to the aggregate one, where it remains.
//!
//! Nothing in the department's arithmetic moved either way. The FY2027 workbook computes both
//! terms and `crates/local-capacity` reproduces its published capacity to 6.4e-6 for 609 of 609
//! districts. This is a question about what the text authorises, not about what was paid.
//!
//! Cited by `corpus/legislation/hb-96-2025.yml` and
//! `corpus/formula-component/fsfp-local-capacity-measure.yml`.

use project::statute::section;

/// The two sections, and the fact that they are the current text of the current budget.
fn capacity_and_certification() -> (&'static str, &'static str) {
    let capacity = section("3317.017");
    let certification = section("3317.021");
    for held in [capacity, certification] {
        assert!(
            held.effective.contains("2025") && held.legislation.contains("House Bill 96"),
            "{} is held at {} under {}; this file reads the enacted text of the current \
             biennium, and a record from another act cannot answer what that act did",
            held.number,
            held.effective,
            held.legislation
        );
    }
    (capacity.body, certification.body)
}

/// **The blend has an aggregate term and a median term, and the act did not change that.**
///
/// The claim that would have to be true for the corpus's reading to be right is that R.C.
/// 3317.017 no longer mentions total federal adjusted gross income. It mentions it twice, in the
/// text H.B. 96 enacted.
#[test]
fn the_capacity_blend_still_reads_both_an_aggregate_and_a_median_income() {
    let (capacity, _) = capacity_and_certification();

    let aggregate = capacity
        .matches("total federal adjusted gross income")
        .count();
    assert_eq!(
        aggregate, 2,
        "R.C. 3317.017(A)(2)(a) takes the lesser of a three-year average and a most-recent-year \
         figure, and names the total federal AGI once for each. Finding {aggregate} means the \
         aggregate term moved, which is the change item 9 of H.B. 96 was read as making."
    );

    let median = capacity
        .matches("median federal adjusted gross income")
        .count();
    assert!(
        median >= 4,
        "the median term appears at (A)(3)(a)(i) and three times in the capacity-percentage \
         scale at (A)(4); {median} occurrences is not that section"
    );

    // The weights, in the statute's own arithmetic at (A)(5), which is the line the crate is.
    for weight in ["X 0.60", "X 0.20"] {
        assert!(
            capacity.contains(weight),
            "R.C. 3317.017(A)(5) writes the blend as {weight} against its terms"
        );
    }
    assert_eq!(
        capacity.matches("X 0.20").count(),
        2,
        "two terms carry 0.20 — the aggregate and the median — which is what makes the blend \
         60/20/20 rather than 60/40"
    );

    // How hard the capacity section leans on the certification section, which is the measure of
    // what not reading it cost — and the citations are *all* income. The valuation term at (A)(1)
    // cites nothing, so every reference this section makes to the Tax Commissioner is to a figure
    // the blend's two income terms need, and half of them went unanswered for four years.
    //
    // Pinned because the count travelled into three doc comments and two corpus nodes from an
    // arithmetic I did once, and got there wrong twice on the way.
    assert_eq!(
        capacity.matches("3317.021").count(),
        4,
        "both halves of the aggregate term at (A)(2)(a) and both halves of the median term at \
         (A)(3)(a), and nothing else in the section"
    );
}

/// **The certification section does not certify what the aggregate term cites it for.**
///
/// This is the live drafting gap, in the text as enacted: R.C. 3317.017(A)(2) takes the total
/// federal AGI *as certified under section 3317.021*, and R.C. 3317.021 certifies a median.
#[test]
fn the_aggregate_term_cites_a_certification_that_no_longer_exists() {
    let (capacity, certification) = capacity_and_certification();

    assert!(
        capacity.contains(
            "The total federal adjusted gross income of the district's residents for the most \
             recent tax year for which data is available, as certified under section 3317.021"
        ),
        "R.C. 3317.017(A)(2)(a)(ii) cites the certification section by number for the aggregate \
         term; if that citation has gone, this finding has gone with it"
    );
    assert!(
        !certification.contains("total federal adjusted gross income"),
        "R.C. 3317.021 certifies a total federal AGI after all, and the gap is closed"
    );
    assert!(
        certification.contains("median federal adjusted gross income"),
        "(A)(5) is what H.B. 96 substituted; without it there is nothing behind the median term \
         either, which would be a different and larger finding"
    );
}

/// **What item 9 actually did, read off the two divisions it wrote.**
///
/// The returns count the median term multiplies by is new to the certification duty, and it is
/// dated: division (A)(6) is limited to FY2026 and FY2027, where (A)(5) beside it is permanent.
/// The two halves of one product therefore have different durations — the same shape as the
/// expiry clause `project::statute` reads on the plan's own sections.
#[test]
fn the_returns_count_the_median_term_multiplies_is_certified_only_for_this_biennium() {
    let (capacity, certification) = capacity_and_certification();

    assert!(
        capacity.contains(
            "The number of state tax returns filed by taxpayers residing in the district for the \
             most recent tax year for which data is available, as certified under section 3317.021"
        ),
        "R.C. 3317.017(A)(3)(a)(ii) is the other half of the median term"
    );

    let returns = certification
        .lines()
        .find(|line| line.contains("number of state tax returns"))
        .expect("R.C. 3317.021 certifies the count H.B. 96 added");
    assert!(
        returns.starts_with("(6) For fiscal years 2026 and 2027,"),
        "the count is division (A)(6) and it is dated; it reads {returns:?}"
    );

    // And the duty sentence that opens division (A) enumerates its divisions rather than taking
    // them as a whole. H.B. 96 appended (A)(6) without extending that range, so the section
    // describes the count in a division the certifying sentence does not reach. Recorded because
    // it is what the text says, not as a reading of what was intended.
    assert!(
        certification
            .contains("the information described in divisions (A)(1) to (5) of this section"),
        "division (A) names the range it certifies; if that range now reaches (6) this \
         observation is spent"
    );
}

/// The median Ohio AGI survives the substitution, and nothing in the plan reads it.
///
/// Worth pinning because it is the term an earlier pass mistook for the blend's third input, and
/// because its presence is what makes the mistake explicable: the certification section really
/// does hand the department an Ohio median, and really did not hand it a federal one.
#[test]
fn the_ohio_median_is_still_certified_and_the_funding_formula_never_asks_for_it() {
    let (capacity, certification) = capacity_and_certification();
    assert!(
        certification.contains("median Ohio adjusted gross income"),
        "(A)(5)'s second clause is unchanged by H.B. 96"
    );
    assert!(
        !capacity.contains("Ohio adjusted gross income"),
        "R.C. 3317.017 asks for federal figures throughout; an Ohio one appearing here would \
         mean the blend reads the term this corpus once guessed it read"
    );
}
