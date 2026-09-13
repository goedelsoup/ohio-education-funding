//! Ohio's transportation floor, what it is worth, and which districts it pays.
//!
//! `draft-legislation/fund-the-plan-and-retire-the-guarantee` carries this, `[verified]`:
//!
//! > **Transportation reimbursement (provision 4).** Outside the model. The panel carries no
//! > transportation series, so there is nothing for a lever to move.
//!
//! The panel carries twenty-five per-district transportation columns — riders by school type,
//! miles, the two supplements, the FY2021 base and its guarantee, and the total. There is a great
//! deal for a lever to move, and this file moves it.
//!
//! # The floor is not the formula's floor
//!
//! R.C. 3317.0212(E)(1)(c) pays the calculated transportation amount at the greater of a
//! district's own state share percentage and a fixed minimum. H.B. 96 raised that minimum
//! **41.67% → 45.83% → 50%** across FY2025-FY2027. It is five times the formula's 10% minimum
//! state share and binds on far more of the state.
//!
//! # What had to be established before any of it could be priced
//!
//! The department publishes transportation **net**: each payment component already has the applied
//! share inside it. So the calculated amount has to be recovered before another floor can be put
//! on it, and the recovery is checked rather than assumed — see
//! `the_recovered_amount_reproduces_the_statutory_bases`, which reproduces `[A] School Bus` from
//! the rider and mile rates on every district, and the statewide reconstruction which reproduces
//! the published total.
//!
//! # The finding
//!
//! The floor is worth **$289.6M** — two fifths of Ohio's transportation aid is paid by the floor
//! rather than by the district's own share. And it runs **up** the wealth distribution: $3.18 per
//! pupil in the least wealthy fifth of districts against **$405.92 in the wealthiest**.
//!
//! That is the opposite shape from the rest of the formula, and it is what a proposal to move this
//! floor needs stated beside it.

use project::panel::panel;
use project::transport::{self, floor};

/// The year the rate series ends, which is the one with a published floor of 50%.
const YEAR: u16 = 2027;

/// Per weighted rider and per bus mile, FY2027, from the department's own rate sheet.
const PER_RIDER: f64 = 1_337.175;
const PER_MILE: f64 = 6.867;

/// The mile base counts a school year.
const DAYS: f64 = 180.0;

fn floor_in_force() -> f64 {
    transport::rates_for(YEAR)
        .expect("FY2027 is in the rate series")
        .minimum_state_share
}

/// **The licence.** The recovered calculated amount is the statute's, on every district.
///
/// `[A] School Bus` gross is the greater of the weighted riders at the rider rate and the bus miles
/// at the mile rate across a 180-day year. Reproducing it from two published rates on every
/// district is what says the division by the applied share recovered the right quantity — a wrong
/// divisor would reproduce neither base.
#[test]
fn the_recovered_amount_reproduces_the_statutory_bases() {
    let in_force = floor_in_force();
    assert!(
        (in_force - 0.5).abs() < 1e-9,
        "FY2027's floor is {in_force}"
    );

    let mut checked = 0;
    let mut on_the_mile_base = 0;
    for record in panel() {
        let Some(share) = record.published_state_share else {
            continue;
        };
        let bus = record.transportation.school_bus;
        if bus <= 0.0 || share <= 0.0 {
            continue;
        }
        let gross = bus / share.max(in_force);
        let rider_base = record.transportation.weighted_riders * PER_RIDER;
        let mile_base = record.transportation.bus_miles * PER_MILE * DAYS;
        let want = rider_base.max(mile_base);
        if mile_base > rider_base {
            on_the_mile_base += 1;
        }
        assert!(
            (gross - want).abs() < f64::max(1.0, want * 1e-6),
            "{} recovers {gross} against a statutory {want}",
            record.irn
        );
        checked += 1;
    }
    assert_eq!(
        checked, 601,
        "districts with a school bus payment and a share"
    );

    // The module docs say 350 districts are paid on the mile base. Recovered independently here.
    assert_eq!(on_the_mile_base, 350, "districts paid on the mile base");
}

/// And the total is the components plus the guarantee, which is what makes the sum recoverable.
#[test]
fn the_published_total_is_the_components_plus_the_guarantee() {
    let mut checked = 0;
    for record in panel() {
        let transportation = &record.transportation;
        if transportation.components() <= 0.0 {
            continue;
        }
        assert!(
            (transportation.total - (transportation.components() + transportation.guarantee)).abs()
                < 1.0,
            "{} totals {} against components {} and guarantee {}",
            record.irn,
            transportation.total,
            transportation.components(),
            transportation.guarantee
        );
        checked += 1;
    }
    assert!(checked > 600, "only {checked} districts checked");

    // And the reconstruction at the floor in force reproduces the published programme.
    let reconstructed = floor::statewide_under(YEAR, floor_in_force()).expect("FY2027");
    assert!(
        (reconstructed - 726_015_823.57).abs() < 1.0,
        "reconstructed {reconstructed}"
    );
}

/// **The finding.** Two fifths of transportation aid is the floor rather than the formula.
#[test]
fn the_floor_pays_two_fifths_of_the_programme() {
    let at_floor = floor::statewide_under(YEAR, floor_in_force()).expect("FY2027");
    let at_own_share = floor::statewide_under(YEAR, 0.0).expect("FY2027");
    let value = floor::value_of_the_floor(YEAR).expect("FY2027");

    assert!(
        (at_own_share - 436_404_230.36).abs() < 1.0,
        "own share {at_own_share}"
    );
    assert!(
        (value - 289_611_593.21).abs() < 1.0,
        "the floor is worth {value}"
    );
    assert!((value - (at_floor - at_own_share)).abs() < 1e-6);

    let share_of_programme = value / at_floor;
    assert!(
        (0.39..0.41).contains(&share_of_programme),
        "the floor is {share_of_programme} of the programme"
    );
}

/// **And it runs up the wealth distribution, monotonically.**
///
/// $3.18 per pupil in the least wealthy fifth against $405.92 in the wealthiest, and every district
/// in the top two quintiles is on the floor against twelve of 120 in the bottom. A floor that pays
/// a district whose *own* share is low is a payment to districts with high local capacity, which is
/// what a low state share means.
#[test]
fn the_floor_runs_up_the_wealth_distribution() {
    let bands = floor::incidence(YEAR).expect("FY2027");
    assert_eq!(bands.len(), 5);

    let per_pupil: Vec<f64> = bands
        .iter()
        .map(|band| band.per_pupil().expect("the band has pupils"))
        .collect();
    assert!((per_pupil[0] - 3.18).abs() < 0.05, "Q1 is {}", per_pupil[0]);
    assert!(
        (per_pupil[4] - 405.92).abs() < 0.5,
        "Q5 is {}",
        per_pupil[4]
    );

    // Monotone, which is the claim rather than the two endpoints.
    for pair in per_pupil.windows(2) {
        assert!(pair[1] > pair[0], "the gradient reverses: {pair:?}");
    }
    assert!(
        per_pupil[4] / per_pupil[0] > 100.0,
        "the ratio is {}",
        per_pupil[4] / per_pupil[0]
    );

    // And the count, which is the same fact without the dollars.
    assert_eq!(bands[0].on_floor, 12, "least wealthy fifth on the floor");
    assert_eq!(bands[3].on_floor, bands[3].districts, "all of Q4");
    assert_eq!(bands[4].on_floor, bands[4].districts, "all of Q5");
}

/// What H.B. 96's own sequence was worth, and what the draft's provision would now do.
///
/// **The provision is stated against a superseded baseline.** It proposes 37.5% against a stated
/// 29.17%, and `parameter/transportation-cost-rates` records H.B. 96 raising the floor 41.67% →
/// 45.83% → 50%. At FY2027 law the provision is a **reduction of 12.5 percentage points**, worth
/// −$118.0M, and it falls on exactly the districts the gradient above names.
///
/// Priced as written rather than silently corrected, because a platform needs to know that a plank
/// drafted against last biennium's law now says the opposite of what it means.
#[test]
fn the_drafts_provision_now_prices_as_a_reduction() {
    let current = floor::statewide_under(YEAR, floor_in_force()).expect("FY2027");
    let as_drafted = floor::statewide_under(YEAR, 0.375).expect("FY2027");
    assert!(as_drafted < current, "the provision should now be a cut");

    let delta = as_drafted - current;
    assert!(
        (delta + 118_105_482.0).abs() < 1000.0,
        "the provision is worth {delta}"
    );

    // H.B. 96's own two steps, for scale against it.
    let previous = floor::statewide_under(YEAR, 0.4167).expect("FY2027");
    let middle = floor::statewide_under(YEAR, 0.4583).expect("FY2027");
    assert!((current - previous - 81_631_450.0).abs() < 1000.0);
    assert!((current - middle - 42_160_627.0).abs() < 1000.0);
    assert!(
        previous < middle && middle < current,
        "the sequence should rise"
    );
}

/// The floor is monotone in itself, which a payment defined by a maximum has to be.
#[test]
fn raising_the_floor_never_pays_a_district_less() {
    let districts = floor::paid(YEAR).expect("FY2027");
    let sample: Vec<_> = districts.iter().take(40).collect();

    for district in sample {
        let mut previous = f64::NEG_INFINITY;
        for step in 0..=20 {
            let candidate = f64::from(step) * 0.05;
            let paid = district.under(candidate);
            assert!(
                paid >= previous - 1e-6,
                "{} falls at a floor of {candidate}",
                district.irn
            );
            previous = paid;
        }
        // A floor below the district's own share changes nothing at all.
        assert!(
            (district.under(0.0) - district.under(district.state_share)).abs() < 1e-6,
            "{} moves below its own share",
            district.irn
        );
    }
}
