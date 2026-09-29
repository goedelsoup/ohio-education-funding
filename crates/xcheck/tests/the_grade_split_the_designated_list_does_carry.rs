//! The grade shape of who may claim a traditional EdChoice scholarship, and what it costs the
//! published average.
//!
//! `program/edchoice-scholarship` holds an arithmetic open question. Every traditional EdChoice
//! award is the lesser of a tuition and a per-grade ceiling — $5,500 for kindergarten through
//! eight, $7,500 for nine through twelve — so the department's published average of $6,808.75
//! caps the share of recipients in the lower band at **0.345625**, and lower still if any tuition
//! binds. The node put two limbs against that: either the programme is overwhelmingly a
//! high-school one, or the published average is on a basis the arithmetic does not fit. It said
//! the corpus could not tell which, because the participation split by grade band is not
//! published.
//!
//! The split by grade band of *participation* is still not published. What is published, and what
//! was not read here, is the split by grade band of **eligibility**: the designated list carries
//! the grades each building serves, and [`dispersion::building`] carries what each building
//! enrols. Joined, they say how many pupils sit on each side of the ceiling boundary in the
//! buildings the statute designates.
//!
//! # Why that does not close the question, and what it does instead
//!
//! Eligibility is not participation, and no arithmetic here turns one into the other. What it
//! does is price the first limb. Read the eligible population as the pupils still enrolled in a
//! designated building plus the programme's own recipients — the two groups the corpus can
//! count — and the published average's ceiling on the recipient mix forces a take-up rate on each
//! side. The first limb stops being "the programme skews towards high schools", which is
//! plausible, and becomes a specific and large claim about participation, which is checkable
//! against one document nobody has published.
//!
//! # What the eligible population here is not
//!
//! It is a floor. A student residing in a designated building's attendance zone who never
//! enrolled there is eligible under R.C. 3310.03 and appears in neither term, and a continuing
//! recipient whose building has since left the list is in the second term without being in the
//! first. Both push the measured take-up rates down; which way they push the *ratio* between them
//! depends on a band composition nobody publishes, which is the same absence this file is
//! measuring around.

use dispersion::designated::Band;
use dispersion::designated_editions::{self as editions, BandPopulation, Edition};
use project::scholarship;

/// Designated buildings in one edition, counted by band.
fn counted(which: Edition) -> [usize; 3] {
    let mut counts = [0usize; 3];
    for building in editions::edition(which).iter().filter(|d| d.designated) {
        counts[match building.band() {
            Band::ThroughEight => 0,
            Band::NineAndAbove => 1,
            Band::Both => 2,
        }] += 1;
    }
    counts
}

/// The recipient count and the largest lower-band mix the report's average allows.
fn traditional_edchoice(basis: scholarship::Average) -> (f64, f64) {
    let programmes = scholarship::report::programmes();
    let edchoice = &programmes["traditional-edchoice"];
    let average = basis
        .of(edchoice)
        .expect("the report carries this programme on both bases");
    let ceiling = scholarship::greatest_k8_share(average)
        .expect("the average lies between the two grade ceilings");
    (edchoice.students, ceiling)
}

/// Every grade cell in every edition reads as one of the three bands, and the counts are stable.
///
/// The read is total by construction — [`dispersion::designated::band`] panics on a cell it
/// cannot place — so the value of this test is the second half: three separately published
/// editions put the same four buildings in five on the lower side of the ceiling.
#[test]
fn every_designated_building_falls_on_one_side_of_the_award_ceiling_or_straddles_it() {
    let expected = [
        (Edition::Y2425, [340, 74, 46]),
        (Edition::Y2526, [367, 80, 47]),
        (Edition::Y2627, [378, 89, 46]),
    ];
    for (which, counts) in expected {
        let measured = counted(which);
        assert_eq!(measured, counts, "{}", which.school_year());
        assert_eq!(
            measured.iter().sum::<usize>(),
            editions::edition(which)
                .iter()
                .filter(|d| d.designated)
                .count()
        );
    }
    assert_eq!(expected[2].1[0] + expected[2].1[1] + expected[2].1[2], 513);
}

/// Two thirds of the pupils the designated list makes eligible are in kindergarten through eight,
/// and that is the reading most favourable to the other side.
///
/// Every building serving grades on both sides of the boundary is handed to the high-school side
/// here, which is the assignment that makes the lower band as small as it can be. It is still
/// about 65% in each of the three editions, and the spread across them is under a point — so the
/// shape is a property of the designated set rather than of one year's list.
#[test]
fn the_eligible_population_is_about_two_thirds_kindergarten_through_eight() {
    let expected = [
        (
            Edition::Y2425,
            [132_817.0, 51_981.0, 19_651.0],
            8,
            0.649_634,
        ),
        (
            Edition::Y2526,
            [146_498.0, 57_095.0, 20_246.0],
            1,
            0.654_479,
        ),
        (
            Edition::Y2627,
            [152_720.0, 64_160.0, 15_215.0],
            0,
            0.658_006,
        ),
    ];
    for (which, pupils, missing, share) in expected {
        let population = editions::eligible_by_band(which);
        assert_eq!(
            [
                population.through_eight,
                population.nine_and_above,
                population.straddling
            ],
            pupils,
            "{}",
            which.school_year()
        );
        assert_eq!(population.unrated, missing, "{}", which.school_year());
        assert!(
            (population.least_k8_share() - share).abs() < 1e-6,
            "{} reads {}",
            which.school_year(),
            population.least_k8_share()
        );

        // The other extreme assignment of the straddling buildings, for the bracket.
        let generous = population.through_eight + population.straddling;
        assert!(generous / (generous + population.nine_and_above) > 0.72);
    }
}

/// The finding. For the published average to be a grade mix, one designated high-school pupil in
/// every four would have to be holding a scholarship.
///
/// $6,808.75 caps the recipient mix at 0.345625 in kindergarten through eight. Against the
/// eligible population the 2024-25 list and the 2024-25 report card measure, that mix requires a
/// take-up of **28.0%** on the high-school side and **10.0%** on the other — the lower band's
/// rate is barely a third of the upper's, in a population where the lower band is two pupils in
/// three.
///
/// And 28.0% is the floor of the claim, not the middle of it: it puts every straddling building
/// on the high-school side. Assign them the other way and the required high-school take-up is
/// 34.9%.
#[test]
fn the_published_average_prices_the_high_school_limb_at_one_pupil_in_four() {
    let (recipients, ceiling) = traditional_edchoice(scholarship::Average::Published);
    assert!((ceiling - 0.345_625).abs() < 1e-9);

    let population: BandPopulation = editions::eligible_by_band(Edition::Y2425);
    let (lower, upper) = population.required_take_up(recipients, ceiling, true);
    assert!((lower - 0.099_808).abs() < 1e-6, "{lower}");
    assert!((upper - 0.280_174).abs() < 1e-6, "{upper}");
    assert!(
        (lower / upper - 0.356_237).abs() < 1e-6,
        "{}",
        lower / upper
    );

    let (_, straddling) = population.required_take_up(recipients, ceiling, false);
    assert!((straddling - 0.349_114).abs() < 1e-6, "{straddling}");
}

/// The same arithmetic on the report's own implied average moves it, and not out of the finding.
///
/// Three of the report's four published averages are not expenditure over participation, and the
/// implied one — $6,645.62 — is the lower of the pair, so it allows a larger share of the lower
/// band: 42.7% rather than 34.6%. That still needs a high-school take-up above a fifth against a
/// lower-band take-up under an eighth, so which of the two averages is read does not decide the
/// question.
#[test]
fn the_implied_average_relaxes_the_mix_without_relaxing_the_finding() {
    let (recipients, ceiling) = traditional_edchoice(scholarship::Average::Implied);
    assert!(ceiling > 0.427 && ceiling < 0.428, "{ceiling}");

    let population = editions::eligible_by_band(Edition::Y2425);
    let (lower, upper) = population.required_take_up(recipients, ceiling, true);
    assert!(lower < 0.125, "{lower}");
    assert!(upper > 0.20, "{upper}");
    assert!(lower / upper < 0.62, "{}", lower / upper);
}
