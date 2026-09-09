//! Four Fair School Funding Plan parameters, before the biennium the corpus computes.
//!
//! Six `unfilled:` entries across four `parameter/` nodes ask the same question in the same words
//! and give the same reason for not answering it:
//!
//! | node | field | why |
//! |---|---|---|
//! | `dpia-per-pupil-amount` | the $422 before FY2026 | "Prior versions of R.C. 3317.022 in Ohio Laws' version archive" |
//! | `gifted-funding-rates` | the rates before FY2026 | "Prior versions of R.C. 3317.051 and 3317.022 in Ohio Laws' version archive" |
//! | `minimum-state-share` | the enacting act and division that set 5% | "Ohio Laws' version archive for the section" |
//! | `minimum-state-share` | the value for FY2023 through FY2025 | "The intervening versions of R.C. 3317.017 would settle it" |
//! | `preschool-special-education-amounts` | the flat grant before FY2026 | "Prior versions of R.C. 3317.0213 in Ohio Laws' version archive" |
//! | `preschool-special-education-amounts` | whether the 0.50 has always been 0.50 | "Same source" |
//!
//! The archive is a real obstacle — it renders per version and publishes no data — and it is not
//! the only route. `parameter/local-share-charge-off-millage` made the same argument about the
//! charge-off era and `the_charge_off_after_the_charge_off.rs` answered it out of
//! [`project::greenbook`]. The same twelve documents carry the plan era, because LSC's job in a
//! greenbook is to say what each budget changed and by how much.
//!
//! # What a greenbook is evidence of
//!
//! LSC's account of an act as enrolled — not the act, and where the two differ the act governs.
//! For a *parameter value* that distinction is narrow: these are figures LSC read out of the bill
//! and printed beside the prior value, which is exactly the pair a series needs and exactly what
//! the codified text does not give, since a section carries only its current numbers.
//!
//! What none of this gives is the **division** that sets a value. The `minimum-state-share` node
//! asks for the act *and* the division; this settles the act and leaves the division where it was.

use project::greenbook::{self, Greenbook};

/// The FY2022-23 budget's analysis — Am. Sub. H.B. 110 of the 134th, which enacted the plan.
fn plan() -> Greenbook<'static> {
    greenbook::greenbook("hb110")
}

/// The FY2024-25 budget's — Am. Sub. H.B. 33 of the 135th.
fn next_biennium() -> Greenbook<'static> {
    greenbook::greenbook("hb33")
}

/// Disadvantaged pupil impact aid was $422 from the first year of the plan, and $272 before it.
///
/// `parameter/dpia-per-pupil-amount` asks what the $422 was before FY2026 and expects the answer
/// to be a series. It is not: the amount was set at $422 when the plan replaced economically
/// disadvantaged funding, and the greenbook prints the figure it replaced beside it.
#[test]
fn the_disadvantaged_amount_has_been_four_hundred_and_twenty_two_since_the_plan_began() {
    assert_eq!(plan().first_fiscal_year(), 2022);
    assert!(plan().flat().contains(
        "Economically disadvantaged funds from the prior funding formula are renamed \
         disadvantaged pupil impact aid (DPIA) with the base per-pupil amount increased from $272 \
         to $422"
    ));
    // And the next budget does not move it, which is what makes FY2022-25 one value rather than
    // a series the corpus is missing.
    assert!(
        !next_biennium()
            .flat()
            .contains("disadvantaged pupil impact aid (DPIA) with the base"),
        "H.B. 33 restated the DPIA amount and this reading needs redoing"
    );
}

/// The gifted rates before FY2026, and there are four of them rather than the three in the node.
///
/// H.B. 110 states identification, referral and professional development together, with the
/// figure each replaced, plus the staffing ratio the node carries separately. H.B. 33 then
/// continues professional development alone.
#[test]
fn every_gifted_rate_the_plan_set_is_printed_beside_the_one_it_replaced() {
    let enacted = plan().flat();
    assert!(enacted.contains(
        "The budget increases per-pupil amounts for gifted identification funds from $5.50 to $24 \
         per enrolled ADM in grades K-6 and adds gifted referral funding equal to $2.50 per \
         enrolled ADM as well as per-pupil funding for gifted professional development of $7.00 \
         in FY 2022 and $14.00 in FY 2023."
    ));
    assert!(enacted.contains(
        "It also establishes a ratio of one gifted intervention specialist unit to 140 gifted \
         students enrolled in grades K-12."
    ));

    // Professional development is the one that keeps moving, and H.B. 33 gives its next two
    // years. H.B. 96 then repeals it, which `legislation/hb-96-2025` records.
    assert!(next_biennium().flat().contains(
        "Per-pupil gifted professional development funds increase from $14 in FY 2023 to $21 in \
         FY 2024 and $28 in FY 2025."
    ));
}

/// The minimum state share was 5% for the plan's first biennium and 10% from FY2024.
///
/// `parameter/minimum-state-share` records that "the floor moved from 5% to 10% and the corpus
/// does not hold which budget act moved it or whether it moved once". It moved once, and H.B. 33
/// moved it — the act's own analysis says so in the sentence that lists what the budget does.
///
/// H.B. 110's greenbook states the 5% as a **formula box** rather than as a rate, and the box
/// says something the node does not: the floor is the complement of a 95% capacity test. A
/// district whose per-pupil local capacity reaches 95% of its base cost per pupil is paid 5% of
/// base cost, and one below that threshold is paid the difference. The floor is not a separate
/// rule bolted onto the formula; it is the second branch of the state share itself.
#[test]
fn the_floor_was_five_per_cent_for_one_biennium_and_moved_once() {
    assert!(plan().flat().contains(
        "If Per-pupil local capacity amount / District\u{2019}s base cost per pupil > 95%, then \
         Per-pupil state share of base cost = District\u{2019}s base cost per pupil x 5%"
    ));

    let later = next_biennium().flat();
    assert!(later.contains("increases the minimum state share percentage from 5% to 10%"));
    assert!(
        later.contains(
            "The minimum state share percentage applies to an estimated 55 (9%) districts in \
             FY 2024 and 68 (11%) districts in FY 2025."
        ),
        "the count of districts the floor reaches is what makes it a live parameter rather than a \
         formality"
    );

    // One move, not two: no later analysis restates the percentage.
    for book in greenbook::greenbooks() {
        if book.first_fiscal_year() > 2024 {
            assert!(
                !book.flat().contains("minimum state share percentage from"),
                "FY{} moves the floor again",
                book.first_fiscal_year()
            );
        }
    }
}

/// The transportation floor is a series, and three of its five values are here.
///
/// H.B. 33 gives FY2023 through FY2025 in one sentence, and `legislation/hb-96-2025` records the
/// remaining two — 45.83% in FY2026 and 50% in FY2027. Together that is every year the plan has
/// run, which no node held.
#[test]
fn the_transportation_floor_rises_every_year_the_plan_has_run() {
    assert!(next_biennium().flat().contains(
        "The minimum state share for transportation increases from 33.33% in FY 2023 to 37.50% in \
         FY 2024 and 41.67% in FY 2025."
    ));

    // The five published values are eight to twelve twenty-fourths, rounded to two places. That
    // is what says the series is one schedule rather than three separate decisions — it runs from
    // a third to a half in equal steps of a twenty-fourth, and each budget enacted the next two.
    let published: [f64; 5] = [33.33, 37.50, 41.67, 45.83, 50.00];
    for (step, stated) in (8..=12).zip(published) {
        let exact = f64::from(step) / 24.0 * 100.0;
        assert!(
            (stated - (exact * 100.0).round() / 100.0).abs() < 1e-9,
            "{stated:.2}% is not {step}/24 rounded to two places ({exact:.4})"
        );
    }
}

/// The preschool flat grant is $4,000 on both sides of the regime change.
///
/// `parameter/preschool-special-education-amounts` asks what the grant was before FY2026 and
/// whether the 0.50 has always been 0.50. H.B. 110's analysis answers the first in the course of
/// describing what it changed — **"Under prior law, each school district received $4,000 for each
/// preschool student with disabilities"** — and what it changed was the *other* term, the
/// per-student special education aid, from a category amount times a state share index to a
/// weight times base cost times a state share percentage.
///
/// So the flat grant is the part of this component that the Fair School Funding Plan did not
/// touch, and it is the part a constant-dollar series would say most about — a nominal $4,000
/// unmoved since at least FY2014, which
/// [`the_half_day_multiplier_is_older_than_the_reason_given_for_it`] dates.
#[test]
fn the_preschool_flat_grant_survived_the_regime_change_unchanged() {
    assert!(plan().flat().contains(
        "Under prior law, each school district received $4,000 for each preschool student with \
         disabilities plus additional special education aid based on the applicable special \
         education category amount for each student and the resident district\u{2019}s state \
         share index."
    ));
    assert!(
        plan()
            .flat()
            .contains("The base amount of $4,000 remains unchanged."),
        "the plan's own analysis says the grant did not move"
    );

    // And it is still $4,000 two budgets later, now beside the plan's terms rather than the
    // prior formula's.
    assert!(next_biennium().flat().contains(
        "receives $4,000 for each preschool student with disabilities plus additional special \
         education aid based on the applicable special education weights for each student, the \
         statewide average base cost per pupil for the fiscal year, and the resident \
         district\u{2019}s state share percentage."
    ));
}

/// The halving has been 0.5 for six consecutive bienniums, and its reason is younger than it is.
///
/// "Whether the 0.50 has always been 0.50" reaches further than the plan era. Every enacted budget
/// analysis from **H.B. 59 of the 130th (FY2014-15) through H.B. 33 of the 135th (FY2024-25)**
/// states the same multiplier beside the same $4,000 flat grant — six in a row, across the change
/// from the Bridge formula to the Fair School Funding Plan, and across the change from a state
/// share *index* to a state share *percentage* in the term the halving is applied to.
///
/// The first of the six states the rule and no reason: *"Special education aid is then multiplied
/// by 0.5."* The justification — "to reflect the half-day nature of those programs" — appears from
/// H.B. 64 of the 131st onward and has been repeated verbatim in every analysis since. A rule
/// older than its published rationale is worth recording as such, because a reader who finds the
/// rationale in the current document will date the rule to it. [inference]
///
/// The bound on this is the extract's own: no analysis before H.B. 59 discusses the component at
/// all, so FY2014 is where the evidence starts and not where the practice does.
#[test]
fn the_half_day_multiplier_is_older_than_the_reason_given_for_it() {
    let stating: Vec<u16> = greenbook::greenbooks()
        .iter()
        .filter(|book| book.flat().contains("multiplied by 0.5"))
        .map(Greenbook::first_fiscal_year)
        .collect();
    assert_eq!(stating, vec![2014, 2016, 2018, 2020, 2022, 2024]);

    let reasoned: Vec<u16> = greenbook::greenbooks()
        .iter()
        .filter(|book| book.flat().contains("half-day nature of those programs"))
        .map(Greenbook::first_fiscal_year)
        .collect();
    assert_eq!(reasoned, vec![2016, 2018, 2020, 2022, 2024]);
    assert!(
        greenbook::greenbook("hb59")
            .flat()
            .contains("Special education aid is then multiplied by 0.5. "),
        "the first statement of the rule carries no reason, which is the finding"
    );

    // The flat grant travels with it, in every one of the six.
    for book in greenbook::greenbooks() {
        assert_eq!(
            book.flat().contains("multiplied by 0.5"),
            book.flat().contains("$4,000 for each preschool"),
            "FY{} states one of the grant and the multiplier without the other",
            book.first_fiscal_year()
        );
    }
}
