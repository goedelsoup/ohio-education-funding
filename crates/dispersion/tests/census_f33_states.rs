//! Where Ohio sits among the states, and the two ways this file lies to a careless reader.
//!
//! The Census Bureau's Annual Survey of School System Finances is the only federal source the
//! corpus holds and the only one that can answer a comparative question. Everything else is Ohio
//! describing itself, which is enough to say what Ohio does and not enough to say whether it is
//! unusual — and the central holding of *DeRolph* is a claim of the second kind.
//!
//! Both traps below produced a wrong answer during extraction, and neither produced an error.
//!
//! Cited by `catalog/census-f33-school-system-finances.md`.

const F33: &str = include_str!("../fixtures/census-f33-states.csv");

struct State {
    fips: String,
    name: String,
    enrollment: f64,
    total_revenue: f64,
    federal_revenue: f64,
    state_revenue: f64,
    local_revenue: f64,
    property_tax: f64,
    parent_government: f64,
    current_spending: f64,
}

impl State {
    fn local_share(&self) -> f64 {
        self.local_revenue / self.total_revenue
    }
    fn state_share(&self) -> f64 {
        self.state_revenue / self.total_revenue
    }
    fn independent(&self) -> bool {
        self.parent_government < self.local_revenue * 0.10
    }
}

fn states() -> Vec<State> {
    F33.lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let p: Vec<&str> = line.split(',').collect();
            let n = |i: usize| p[i].trim().parse::<f64>().unwrap_or(0.0);
            State {
                fips: p[0].trim().to_string(),
                name: p[1].trim().to_string(),
                enrollment: n(3),
                total_revenue: n(4),
                federal_revenue: n(5),
                state_revenue: n(6),
                local_revenue: n(7),
                property_tax: n(8),
                parent_government: n(9),
                current_spending: n(10),
            }
        })
        .collect()
}

fn ohio(all: &[State]) -> &State {
    all.iter().find(|s| s.name == "Ohio").expect("Ohio")
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
    let oh = ohio(&all);
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
    let silent: Vec<&State> = all.iter().filter(|s| s.property_tax == 0.0).collect();
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
    let all = states();
    for state in &all {
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
    assert_eq!(ohio(&all).parent_government, 0.0);
    assert!(ohio(&all).independent());
}

/// The finding. Ohio spends about the national average and raises it differently.
#[test]
fn ohio_is_high_on_local_share_and_low_on_state_share() {
    let all = states();
    let oh = ohio(&all);

    let rank = |key: &dyn Fn(&State) -> f64| all.iter().filter(|s| key(s) > key(oh)).count() + 1;

    let local_rank = rank(&State::local_share);
    let state_rank = rank(&State::state_share);
    assert!(
        local_rank <= 10,
        "Ohio should be near the top on local share; it ranks {local_rank}"
    );
    assert!(
        state_rank >= 42,
        "Ohio should be near the bottom on state share; it ranks {state_rank}"
    );

    let total = |pick: &dyn Fn(&State) -> f64| all.iter().map(pick).sum::<f64>();
    let national_local = total(&|s| s.local_revenue) / total(&|s| s.total_revenue);
    assert!(
        oh.local_share() > national_local + 0.05,
        "Ohio {:.3} against a national {:.3}",
        oh.local_share(),
        national_local
    );

    // Spending and the federal share are unremarkable, which is what makes the split the story.
    let national_spending = total(&|s| s.current_spending) * 1_000.0 / total(&|s| s.enrollment);
    let ohio_spending = oh.current_spending * 1_000.0 / oh.enrollment;
    assert!(
        (ohio_spending - national_spending).abs() / national_spending < 0.10,
        "Ohio spends {ohio_spending:.0} against a national {national_spending:.0}"
    );
    let national_federal = total(&|s| s.federal_revenue) / total(&|s| s.total_revenue);
    assert!(
        (oh.federal_revenue / oh.total_revenue - national_federal).abs() < 0.02,
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
