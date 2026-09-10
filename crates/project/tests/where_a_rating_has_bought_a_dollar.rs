//! The `accountability_effect` six budget acts each recorded as unassessed, asked once.
//!
//! H.B. 95, H.B. 119, H.B. 59, H.B. 64, H.B. 49 and H.B. 166 all carry a property saying the act
//! was "not assessed against `accountability-regime` here". Two of them say what the assessment is
//! for: H.B. 119's Closing the Achievement Gap "pays on the academic watch and academic emergency
//! building counts", and H.B. 64's bonuses "join the funding and accountability systems at a point
//! neither `accountability-regime` nor the formula nodes currently record."
//!
//! Asked as one question — where has an Ohio rating determined a funding amount? — the twelve
//! committed LSC greenbooks answer it for every biennium from FY2002 to FY2024.
//!
//! # Two episodes, eight quiet bienniums, and a reversal
//!
//! A rating drove a formula payment in four greenbooks and no others. It paid on *failure* in
//! FY2008-09 and on *success* in FY2016-19, and the supplement in force now pays on success
//! without the wealth equaliser the middle episode carried.
//!
//! One of the six acts is answered by an absence: H.B. 59 rebuilt the report card and carried the
//! third-grade reading guarantee while coupling no dollars to any rating at all.
//!
//! H.B. 166 is answered by something better than an absence. Its freeze held every district at
//! FY2019 foundation aid, severing the link for them, while requiring community and STEM schools'
//! bonuses to be recalculated each year — so for two years the only Ohio schools whose funding
//! moved with their own results were charters and STEM schools, on the version of the bonus that
//! never carried the wealth equaliser.

use project::rating_payments::{self as ratings, Direction, Equalised, Route};

/// Every coupling is in the greenbook it names.
///
/// The phrases are the ones LSC wrote, matched literally. A greenbook re-extraction that reflowed
/// them would fail here, which is the point: the chronology is a claim about documents this
/// repository holds, not a summary of them.
#[test]
fn each_coupling_is_evidenced_in_its_own_greenbook() {
    let mut checked = 0;
    for coupling in ratings::COUPLINGS {
        if coupling.greenbook.is_empty() {
            // H.B. 96 is outside the committed set, which stops at H.B. 33. Asserted as absent
            // rather than skipped silently, so that adding its greenbook later turns this red.
            assert!(
                coupling.evidence.is_empty(),
                "{} names evidence with no greenbook to find it in",
                coupling.name
            );
            assert!(
                !project::greenbook::greenbooks()
                    .iter()
                    .any(|g| g.first_fiscal_year() >= coupling.years.0),
                "a greenbook now reaches {}, so {} can be evidenced like the rest",
                coupling.years.0,
                coupling.name
            );
            continue;
        }
        assert!(
            ratings::quotes(coupling.greenbook, coupling.evidence),
            "{} cites {} for \"{}\" and it is not there",
            coupling.name,
            coupling.greenbook,
            coupling.evidence
        );
        checked += 1;
    }
    assert_eq!(checked, 7, "couplings evidenced against a greenbook");
}

/// A rating drove a formula payment in three greenbooks, and nine carry nothing.
///
/// This is the shape of the answer. Had the terms been scattered across the twelve, the two
/// episodes would be an artefact of where anyone happened to look.
#[test]
fn nine_of_the_twelve_bienniums_couple_no_dollar_to_a_rating() {
    let quiet = ratings::quiet();
    assert_eq!(
        quiet,
        vec!["hb94", "hb95", "hb66", "hb1", "hb153", "hb59", "hb110", "hb33"],
        "the greenbooks with no rating-driven formula payment"
    );
    assert_eq!(quiet.len(), 8, "quiet bienniums");

    // And the four that are not quiet are the ones the couplings name.
    let noisy: Vec<&str> = project::greenbook::greenbooks()
        .into_iter()
        .map(|g| g.bill)
        .filter(|bill| !quiet.contains(bill))
        .collect();
    assert_eq!(noisy, vec!["hb119", "hb64", "hb49", "hb166"]);

    // H.B. 59 is the one act of the six whose answer is a pure absence: a budget that rebuilt the
    // report card and coupled no dollar to it.
    assert!(
        quiet.contains(&"hb59"),
        "hb59 was expected to couple nothing"
    );

    // H.B. 166 is not quiet, and reading it as quiet is the mistake this test exists to prevent.
    // `mentions` searches the publisher's own line endings and misses the phrase, which is how it
    // read as quiet on the first pass.
    assert!(!quiet.contains(&"hb166"));
    let hb166 = project::greenbook::greenbook("hb166");
    assert!(
        !hb166.mentions("third grade reading bonus"),
        "the line break this test is about has gone, so the flattening no longer proves anything"
    );
}

/// **The finding.** Ohio reversed the direction, then dropped the equaliser.
#[test]
fn the_formula_couplings_reverse_direction_and_then_lose_the_equaliser() {
    let formula = ratings::formula_couplings();
    assert_eq!(formula.len(), 5, "formula couplings across the whole span");

    // Oldest pays on failure; everything after it pays on success.
    assert_eq!(formula[0].name, "Closing the Achievement Gap");
    assert_eq!(formula[0].direction, Direction::OnFailure);
    assert!(
        formula[1..]
            .iter()
            .all(|c| c.direction == Direction::OnSuccess),
        "the reversal is not clean: {:?}",
        formula.iter().map(|c| c.direction).collect::<Vec<_>>()
    );

    // The equalised versions are the district ones, and the un-equalised ones are the charter
    // version and the supplement now in force.
    let equalised: Vec<&str> = formula
        .iter()
        .filter(|c| c.equalised == Equalised::Yes)
        .map(|c| c.name)
        .collect();
    assert_eq!(
        equalised,
        vec![
            "Closing the Achievement Gap",
            "Graduation bonus",
            "Third grade reading bonus"
        ]
    );
    let gross: Vec<&str> = formula
        .iter()
        .filter(|c| c.equalised == Equalised::No)
        .map(|c| c.name)
        .collect();
    assert_eq!(
        gross,
        vec![
            "Graduation and third grade reading bonuses for community and STEM schools",
            "Performance supplement"
        ],
        "the un-equalised form is the charter one and the one Ohio readopted"
    );

    // The un-equalised charter version outlives the equalised district one, which is the freeze.
    let district = formula
        .iter()
        .find(|c| c.name == "Graduation bonus")
        .unwrap();
    let charter = formula
        .iter()
        .find(|c| c.equalised == Equalised::No && c.years.0 == 2016)
        .unwrap();
    assert_eq!(district.years, (2016, 2019));
    assert_eq!(charter.years, (2016, 2021));

    // The gaps are what make these episodes: nothing between FY2010 and FY2015, and nothing for
    // anyone between FY2022 and FY2025.
    assert_eq!(formula[0].years, (2008, 2009));
    assert_eq!(
        formula.last().unwrap().act,
        "H.B. 96 of the 136th General Assembly"
    );
    assert_eq!(formula.last().unwrap().years.0, 2026);
}

/// The other two routes are older than either episode, and one of them is still running.
#[test]
fn a_rating_reaches_a_districts_money_two_ways_that_are_not_the_formula() {
    let earmarks: Vec<&ratings::Coupling> = ratings::COUPLINGS
        .iter()
        .filter(|c| c.route == Route::Earmark)
        .collect();
    assert_eq!(
        earmarks.len(),
        2,
        "earmarked appropriations to rated districts"
    );
    assert!(
        earmarks.iter().all(|c| c.years.0 == 2004),
        "both H.B. 95 earmarks should open in FY2004"
    );

    let deductions: Vec<&ratings::Coupling> = ratings::COUPLINGS
        .iter()
        .filter(|c| c.route == Route::Deduction)
        .collect();
    assert_eq!(deductions.len(), 1);
    assert_eq!(deductions[0].name, "EdChoice scholarship eligibility");

    // It is the only route on which a rating takes money away, and it predates the first formula
    // coupling by two years.
    let earliest_formula = ratings::formula_couplings()[0].years.0;
    assert!(
        deductions[0].years.0 < earliest_formula,
        "the deduction opens at FY{} against the first formula coupling at FY{earliest_formula}",
        deductions[0].years.0
    );
}

/// Closing the Achievement Gap, in the detail no node held.
///
/// The corpus recorded that it "pays on the academic watch and academic emergency building counts"
/// and nothing further. The greenbook carries the formula, the qualification rule, the count of
/// districts that met it, and a second-year term that pays for improvement — which is the one
/// place an Ohio rating has ever bought a dollar for getting *better* while still being paid for
/// being bad.
#[test]
fn closing_the_achievement_gap_paid_on_failure_and_then_on_improvement() {
    assert!(
        ratings::quotes("hb119", "Academic distress index = (% of district’s buildings in AE or AW) / (% of state’s buildings in AE or AW)"),
        "the index definition is not in the greenbook as written"
    );
    assert!(
        ratings::quotes("hb119", "Qualifying districts have academic distress indices and poverty indices at least equal to 1.0"),
        "the qualification rule is not there as written"
    );
    assert!(
        ratings::quotes("hb119", "Thirty-one districts have both academic distress indices and poverty indices greater than or equal to one"),
        "the count of qualifying districts is not there as written"
    );

    // The second-year terms, which run in opposite directions.
    assert!(
        ratings::quotes("hb119", "have an academic distress percentage lower than FY 2008 = FY 2008 subsidy amount x 1.035"),
        "the improvement kicker is not there as written"
    );
    assert!(
        ratings::quotes(
            "hb119",
            "equal to or greater than FY 2008 = FY 2008 subsidy amount"
        ),
        "the flat term for districts that did not improve is not there as written"
    );

    // And the statewide base the index is taken against.
    assert!(
        ratings::quotes("hb119", "out of a total of 3,867 buildings, 447 (11.6%) were in academic watch or academic emergency"),
        "the FY2008 statewide building counts are not there as written"
    );
}

/// The freeze severed the link for districts and left it running for charters.
///
/// This is H.B. 166's `accountability_effect`, and it is the one of the six that is neither a
/// finding nor an absence. The act does not touch the report card's relation to money in general;
/// it holds districts still and lets community and STEM schools keep moving.
#[test]
fn the_freeze_left_only_charters_funding_moving_with_their_results() {
    assert!(
        ratings::quotes(
            "hb166",
            "every traditional school district and JVSD receive the same amount of foundation aid in FY 2020 and FY 2021 as it received in FY 2019"
        ),
        "the freeze is not in the greenbook as written"
    );
    assert!(
        ratings::quotes(
            "hb166",
            "each community and STEM school’s graduation and third grade reading bonuses, which are paid directly by the state, must be recalculated each year using a formula amount of $6,020"
        ),
        "the carve-out is not in the greenbook as written"
    );

    // And the version that kept running is the one without the equaliser.
    let charter = ratings::COUPLINGS
        .iter()
        .find(|c| c.name.contains("community and STEM"))
        .expect("the charter coupling");
    assert_eq!(charter.equalised, Equalised::No);
    assert_eq!(charter.years.1, 2021);
}

/// The rate is the thing that moved most, and it moved by a factor of fifty.
///
/// Closing the Achievement Gap paid 0.15% of the formula amount per pupil; the bonuses paid 7.5%.
/// Both are shares of the same quantity in the same formula, so they are directly comparable, and
/// the corpus already pins the second at `project/outcome-bonus-share-of-the-formula-amount`.
#[test]
fn the_second_episode_paid_fifty_times_the_rate_of_the_first() {
    assert!(
        ratings::quotes("hb119", "0.0015 x formula amount"),
        "the FY2008 rate is not in the greenbook as written"
    );
    // LSC states the same number the other way round two paragraphs earlier, which is the check
    // that 0.0015 is a share and not a typo for something else.
    assert!(
        ratings::quotes("hb119", "on a per student basis, 0.15% of the"),
        "the FY2008 rate is not restated as a percentage"
    );

    assert!(
        ratings::quotes("hb64", "7.5% of the formula amount"),
        "the FY2016 rate is not in the greenbook as written"
    );
    assert!(
        (0.075_f64 / 0.0015 - 50.0).abs() < 1e-9,
        "the two rates are no longer a factor of fifty apart"
    );
}
