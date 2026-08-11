//! The FY2024 interstate cross-section, and what of OCG White Paper 015's Section 9 it settles.
//!
//! That paper's cross-state table was the largest block of it this corpus could not check: the
//! F-33 fixture was FY2022 and the paper quotes FY2024. It is now held, and the result splits.
//!
//! **The revenue mix reproduces.** Ohio's state, local and federal shares land within a point of
//! the published figures, and so do the four neighbours. The paper's load-bearing Section 9
//! claim — that Ohio's distinctive feature is its reliance on local revenue, well above the
//! national average — is confirmed.
//!
//! **The per-pupil dollars do not.** Aggregating the individual-unit file scatters around the
//! published figures without a consistent sign — Indiana +0.5%, Kentucky +1.1%, Pennsylvania
//! +3.2%, Ohio −3.7%, Michigan −9.8%. A uniform bias would point at a definition and could be
//! corrected; a state-specific one cannot be, and it means unit-file aggregation is not a
//! substitute for the Bureau's published state table on this measure.
//!
//! That is recorded as an open question about the aggregation rather than as an error in the
//! paper. It quotes a published Census figure; this corpus cannot rebuild that figure from the
//! unit file, which is a fact about the corpus.
//!
//! Cited by `catalog/ocg-white-paper-015.md`.

const FY2024: &str = include_str!("../fixtures/census-f33-states-fy2024.csv");
const FY2022: &str = include_str!("../fixtures/census-f33-states.csv");

/// Column indices in the state fixture.
mod col {
    pub const NAME: usize = 1;
    pub const ENROLLMENT: usize = 3;
    pub const TOTAL_REVENUE: usize = 4;
    pub const FEDERAL_REVENUE: usize = 5;
    pub const STATE_REVENUE: usize = 6;
    pub const LOCAL_REVENUE: usize = 7;
    pub const CURRENT_SPENDING: usize = 10;
}

#[derive(Debug, Clone, Copy)]
struct Shares {
    state: f64,
    local: f64,
    federal: f64,
    per_pupil: f64,
}

fn shares(csv: &str, state: &str) -> Shares {
    let row: Vec<f64> = csv
        .lines()
        .skip(1)
        .find(|l| l.split(',').nth(col::NAME) == Some(state))
        .unwrap_or_else(|| panic!("{state} is not in the fixture"))
        .split(',')
        .map(|v| v.trim().parse::<f64>().unwrap_or(0.0))
        .collect();
    let total = row[col::TOTAL_REVENUE];
    Shares {
        state: row[col::STATE_REVENUE] / total,
        local: row[col::LOCAL_REVENUE] / total,
        federal: row[col::FEDERAL_REVENUE] / total,
        // Money is reported in thousands; enrolment is a headcount.
        per_pupil: row[col::CURRENT_SPENDING] * 1000.0 / row[col::ENROLLMENT],
    }
}

/// Sum every state, which is the corpus's stand-in for the national row.
fn national(csv: &str) -> Shares {
    let mut totals = [0.0f64; 5];
    for line in csv.lines().skip(1) {
        let row: Vec<f64> = line
            .split(',')
            .map(|v| v.trim().parse::<f64>().unwrap_or(0.0))
            .collect();
        for (slot, index) in [
            col::ENROLLMENT,
            col::TOTAL_REVENUE,
            col::FEDERAL_REVENUE,
            col::STATE_REVENUE,
            col::LOCAL_REVENUE,
        ]
        .into_iter()
        .enumerate()
        {
            totals[slot] += row[index];
        }
    }
    let spending: f64 = csv
        .lines()
        .skip(1)
        .map(|l| {
            l.split(',')
                .nth(col::CURRENT_SPENDING)
                .and_then(|v| v.trim().parse::<f64>().ok())
                .unwrap_or(0.0)
        })
        .sum();
    Shares {
        state: totals[3] / totals[1],
        local: totals[4] / totals[1],
        federal: totals[2] / totals[1],
        per_pupil: spending * 1000.0 / totals[0],
    }
}

#[test]
fn the_fixture_holds_fifty_states_and_the_district_of_columbia() {
    assert_eq!(FY2024.lines().count() - 1, 51);
    assert_eq!(FY2022.lines().count() - 1, 51);
}

/// **The revenue mix reproduces**, which is the part of Section 9 the paper's argument rests on.
#[test]
fn the_published_revenue_shares_reproduce_within_a_point() {
    // (state, published state share, published local share, published federal share)
    let published = [
        ("Ohio", 0.346, 0.536, 0.118),
        ("Indiana", 0.559, 0.327, 0.114),
        ("Kentucky", 0.459, 0.370, 0.171),
    ];
    for (state, state_share, local_share, federal_share) in published {
        let got = shares(FY2024, state);
        for (label, actual, expected) in [
            ("state", got.state, state_share),
            ("local", got.local, local_share),
            ("federal", got.federal, federal_share),
        ] {
            assert!(
                (actual - expected).abs() < 0.015,
                "{state} {label} share is {actual:.3} against a published {expected:.3}"
            );
        }
    }
}

/// The claim Section 9 is actually making: Ohio leans on local revenue far more than the nation.
#[test]
fn ohio_relies_on_local_revenue_far_more_than_the_nation() {
    let ohio = shares(FY2024, "Ohio");
    let us = national(FY2024);
    assert!(
        ohio.local > us.local + 0.09,
        "Ohio local {:.3} against national {:.3}",
        ohio.local,
        us.local
    );
    assert!(
        ohio.state < us.state - 0.09,
        "Ohio state {:.3} against national {:.3}",
        ohio.state,
        us.state
    );
    // And it is not a rounding story: Ohio is on the local-heavy side of its neighbours too.
    for neighbour in ["Indiana", "Michigan", "Kentucky"] {
        assert!(ohio.local > shares(FY2024, neighbour).local);
    }
    assert!(ohio.local < shares(FY2024, "Pennsylvania").local + 0.03);
}

/// **The per-pupil dollars do not reproduce**, and the errors are pinned so they cannot drift
/// into looking like agreement.
///
/// The scatter is what makes this unresolvable from here. Indiana, Kentucky and Pennsylvania
/// come out high and Ohio and Michigan low, so no single spending definition or unit filter
/// explains it; the Bureau's published state figures rest on something these rows do not carry.
/// Recorded as a limit on the corpus, not as an error in anything that quotes the published
/// table.
#[test]
fn the_published_per_pupil_figures_cannot_be_rebuilt_from_the_unit_file() {
    let published = [
        ("Ohio", 17_257.0, -0.037),
        ("Indiana", 13_622.0, 0.005),
        ("Michigan", 18_314.0, -0.098),
        ("Pennsylvania", 21_091.0, 0.032),
        ("Kentucky", 14_596.0, 0.011),
    ];
    let mut signs = (0, 0);
    for (state, expected, recorded) in published {
        let error = shares(FY2024, state).per_pupil / expected - 1.0;
        assert!(
            (error - recorded).abs() < 0.008,
            "{state} is {error:+.3} from the published figure, not the {recorded:+.3} recorded"
        );
        if error > 0.0 {
            signs.0 += 1;
        } else {
            signs.1 += 1;
        }
    }
    // Both directions are represented, which is the point: this is not a correctable offset.
    assert!(signs.0 >= 2 && signs.1 >= 2, "{signs:?}");
}

/// The two years are on the same basis as each other, so the corpus can at least compare them.
#[test]
fn ohio_moved_the_way_the_corpus_already_recorded_between_the_two_years() {
    let (then, now) = (shares(FY2022, "Ohio"), shares(FY2024, "Ohio"));
    // The published FY2022 local share, reproduced three ways in this repository, is 51.8%.
    assert!((then.local - 0.518).abs() < 0.005);
    // Local and state both rise as the relief money leaves and stops inflating the denominator.
    // The state share ticking up over these two years is not in tension with White Paper 015's
    // two-decade decline; it is the arithmetic of a shrinking federal share.
    assert!(now.local > then.local + 0.01);
    assert!(now.state > then.state);
    assert!(now.federal < then.federal - 0.02);
}
