//! `formula-component/fsfp-performance-supplement` measures the gradient and stops there.
//!
//! Its closing sentence is the question: "a regressive 0.8% inside a progressive $7.28bn is a
//! different object from a regressive program. The gradient is measured; whether it moves any
//! district's position is not."
//!
//! It moves them, barely, and in the direction the gradient predicts. Ordered by state support per
//! pupil, 370 of 609 districts change place — by a mean of 1.07 places, a largest of 8, and four
//! crossings of a quintile boundary. The coefficient of variation of state aid narrows by 0.0040,
//! which is 0.75% of it.
//!
//! # The sign is the part worth getting right
//!
//! Narrowing the spread of *state aid* is the regressive reading rather than a refutation of it.
//! Ohio disperses state aid on purpose — $10,265 per pupil to the poorest quartile of districts
//! against $4,851 to the least-poor — and that gap is the equalisation. A payment worth 1.14% of
//! the least-poor quartile's aid and 0.22% of the poorest's compresses it.

use project::supplement_reach as reach;

/// **The answer.** It reorders Ohio by about one place.
#[test]
fn the_supplement_moves_three_hundred_and_seventy_districts_by_a_mean_of_one_place() {
    assert_eq!(
        reach::frame().len(),
        609,
        "districts in the department's model"
    );

    let (changed, mean, largest) = reach::rank_moves();
    assert_eq!(changed, 370, "districts whose rank changes");
    assert!(
        (mean - 1.0673).abs() < 0.0005,
        "the mean absolute move is {mean:.4} places"
    );
    assert_eq!(largest, 8, "and the largest in the state");

    // More than half the state moves and the movement is one place, which is the shape of the
    // answer: the supplement is not decorative and it is not consequential either.
    assert!(changed > reach::frame().len() / 2);
    assert!(mean < 2.0);
}

/// Four districts out of 609 cross a boundary that means anything.
#[test]
fn four_districts_cross_a_quintile_of_state_aid_and_they_are_nameable() {
    let crossings = reach::quintile_crossings();
    assert_eq!(
        crossings,
        vec![
            "Fremont City",
            "Monroeville Local",
            "Madison Local",
            "Fort Frye Local"
        ],
        "the districts a $13-a-star payment carries across a quintile"
    );
}

/// **The direction, which is easy to state backwards.** It narrows the equalisation.
#[test]
fn the_supplement_compresses_the_spread_of_state_aid_by_three_quarters_of_one_per_cent() {
    let (with, without) = reach::dispersion();
    assert!(
        (with - 0.524162).abs() < 0.0005 && (without - 0.528117).abs() < 0.0005,
        "the coefficient of variation is {with:.6} with the supplement and {without:.6} without"
    );
    // Removing it *widens* the distribution, so the supplement narrows it — and state aid is
    // dispersed toward need, so narrowing it is the regressive reading.
    assert!(without > with);
    let relative = (without - with) / with;
    assert!(
        (relative - 0.007546).abs() < 0.0005,
        "the compression is {relative:.6} of the dispersion"
    );
}

/// And why it compresses: it is worth five times as much to the districts that receive least.
#[test]
fn it_is_worth_five_times_as_much_against_the_aid_of_the_least_poor_quartile() {
    let bands = reach::by_poverty_quartile();
    assert!(
        (bands[0].0 - 4850.57).abs() < 0.01 && (bands[3].0 - 10265.49).abs() < 0.01,
        "state aid per pupil runs {:.2} to {:.2} across poverty quartiles",
        bands[0].0,
        bands[3].0
    );
    // State aid rises monotonically with poverty. That is what makes a payment falling with
    // poverty a payment rising as a share of aid.
    assert!(
        bands.windows(2).all(|w| w[0].0 < w[1].0),
        "state aid is not monotone in poverty: {bands:?}"
    );

    let relative = reach::relative_to_aid();
    assert!(
        (relative[0] - 0.011408).abs() < 0.0005 && (relative[3] - 0.002173).abs() < 0.0005,
        "the supplement is worth {:.4} of the least-poor quartile's aid and {:.4} of the poorest's",
        relative[0],
        relative[3]
    );
    assert!(
        relative.windows(2).all(|w| w[0] > w[1]),
        "and it should fall monotonically as a share too: {relative:?}"
    );
    // The node's published gradient is 2.56 per pupil. Against the aid a district already has it
    // is more than five, which is the number that decides whether the distribution compresses.
    let ratio = relative[0] / relative[3];
    assert!(
        ratio > 5.0,
        "the ratio against aid is {ratio:.2}, not above five"
    );
}

/// The reason it has any purchase at all: the guarantee does not absorb it.
#[test]
fn most_guaranteed_districts_are_paid_a_supplement_on_top_of_the_guarantee() {
    let (guaranteed, paid) = reach::paid_on_the_guarantee();
    assert_eq!((guaranteed, paid), (294, 217));
    // Nearly three quarters of the guaranteed half of the state, so this is the ordinary case
    // rather than an edge one: the supplement is line `[O]` and the guarantee is computed on
    // `[H]`, and nothing in between cancels it.
    assert!(paid * 10 > guaranteed * 7);
}
