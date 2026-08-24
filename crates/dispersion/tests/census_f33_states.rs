//! Where Ohio sits among the states, and the two ways this file lies to a careless reader.
//!
//! The Census Bureau's Annual Survey of School System Finances is the only federal source the
//! corpus holds and the only one that can answer a comparative question. Everything else is Ohio
//! describing itself, which is enough to say what Ohio does and not enough to say whether it is
//! unusual — and the central holding of *DeRolph* is a claim of the second kind.
//!
//! Both traps below produced a wrong answer during extraction, and neither produced an error.
//! They are now stated on [`dispersion::census_states`], which is the crate's reader of this
//! file; this suite asserts them against that reader rather than against a copy of its own.
//!
//! Cited by `catalog/census-f33-school-system-finances.md`.

use dispersion::census_states::{self, national_share, ohio, rank_of, StateFinance};

fn states() -> Vec<StateFinance> {
    census_states::states()
}

#[test]
fn the_fixture_is_fifty_states_and_the_district_of_columbia() {
    let all = states();
    assert_eq!(all.len(), 51);
    assert_eq!(
        all.iter().filter(|s| s.enrollment > 0.0).count(),
        51,
        "every row must carry enrolment; the aggregation excludes systems without it"
    );
}

/// Column 0 of the survey is the Census Bureau's own state ordering and column 4 is FIPS.
///
/// They agree through the early alphabet and diverge after, both being two-digit and zero-padded,
/// so filtering the wrong one returns a full and entirely wrong answer. Ohio is FIPS 39 and
/// Census 36; Census 39 is Pennsylvania. The first run of this extractor reported Pennsylvania's
/// figures under Ohio's name and nothing complained.
///
/// The assertion is on a quantity nobody would confuse: Ohio had roughly 1.55 million pupils in
/// FY2022 and Pennsylvania roughly 1.72 million.
#[test]
fn ohio_is_fips_thirty_nine_and_not_census_thirty_nine() {
    let all = states();
    let oh = ohio();
    assert_eq!(oh.fips, "39");
    assert!(
        (1_500_000.0..1_600_000.0).contains(&oh.enrollment),
        "Ohio's enrolment is {:.0}; Pennsylvania's is about 1.72m and would land outside this",
        oh.enrollment
    );
    assert_eq!(
        all.iter().find(|s| s.fips == "42").map(|s| s.name.as_str()),
        Some("Pennsylvania")
    );
}

/// Nine states report zero school property tax and levy a great deal of it.
///
/// Their districts are dependent agencies of a city or county, so the tax belongs to the parent
/// government and arrives as an appropriation. Ranking all fifty-one on property tax share would
/// put Massachusetts and Virginia — which lean on property tax about as hard as anywhere — at the
/// bottom of the measure.
#[test]
fn the_states_reporting_no_property_tax_are_the_ones_with_dependent_districts() {
    let all = states();
    let silent: Vec<&StateFinance> = all.iter().filter(|s| s.property_tax == 0.0).collect();
    assert!(
        silent.len() >= 8,
        "expected the dependent-district states, found {}",
        silent.len()
    );

    for state in &silent {
        // Not that they raise nothing locally — they raise it through the parent government.
        assert!(
            state.parent_government > state.local_revenue * 0.5 || state.local_revenue < 100_000.0,
            "{} reports no property tax and no parent-government revenue either, which is \
             neither structure and means the column map has moved",
            state.name
        );
    }

    let names: Vec<&str> = silent.iter().map(|s| s.name.as_str()).collect();
    for expected in ["Massachusetts", "Virginia", "Maryland", "North Carolina"] {
        assert!(
            names.contains(&expected),
            "{expected} should be in this group"
        );
    }
}

/// Local revenue is the aggregate that survives the difference, so it is what Ohio is ranked on.
#[test]
fn local_revenue_includes_the_parent_appropriation_and_is_comparable_across_both() {
    for state in states() {
        assert!(
            state.parent_government <= state.local_revenue + 1.0,
            "{}: parent contributions cannot exceed local revenue",
            state.name
        );
        assert!(
            state.property_tax <= state.local_revenue + 1.0,
            "{}: property tax cannot exceed local revenue",
            state.name
        );
    }
    // And Ohio, an independent-district state, routes none of its local revenue that way.
    assert_eq!(ohio().parent_government, 0.0);
    assert!(ohio().independent());
}

/// The finding. Ohio spends about the national average and raises it differently.
#[test]
fn ohio_is_high_on_local_share_and_low_on_state_share() {
    let all = states();
    let oh = ohio();

    let local_rank = rank_of(&oh, StateFinance::local_share);
    let state_rank = rank_of(&oh, StateFinance::state_share);
    assert!(
        local_rank <= 10,
        "Ohio should be near the top on local share; it ranks {local_rank}"
    );
    assert!(
        state_rank >= 42,
        "Ohio should be near the bottom on state share; it ranks {state_rank}"
    );

    let national_local = national_share(|s| s.local_revenue);
    assert!(
        oh.local_share() > national_local + 0.05,
        "Ohio {:.3} against a national {:.3}",
        oh.local_share(),
        national_local
    );

    // Spending and the federal share are unremarkable, which is what makes the split the story.
    let national_spending = all.iter().map(|s| s.current_spending).sum::<f64>() * 1_000.0
        / all.iter().map(|s| s.enrollment).sum::<f64>();
    let ohio_spending = oh.spending_per_pupil();
    assert!(
        (ohio_spending - national_spending).abs() / national_spending < 0.10,
        "Ohio spends {ohio_spending:.0} against a national {national_spending:.0}"
    );
    assert!(
        (oh.federal_share() - national_share(|s| s.federal_revenue)).abs() < 0.02,
        "Ohio's federal share should be within two points of the national figure"
    );
}

/// Revenue by source has to add up, or the columns are not what they are labelled.
#[test]
fn the_three_revenue_sources_partition_total_revenue() {
    for state in states() {
        let parts = state.federal_revenue + state.state_revenue + state.local_revenue;
        let relative = (parts - state.total_revenue).abs() / state.total_revenue;
        assert!(
            relative < 0.001,
            "{}: federal + state + local is {parts:.0} against a total of {:.0}",
            state.name,
            state.total_revenue
        );
    }
}
