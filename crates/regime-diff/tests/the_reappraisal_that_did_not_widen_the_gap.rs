//! The cause `doctrine/equity` left open, tested against the three arms Ohio's calendar provides.
//!
//! The node recorded a break in the equalization series at FY2023 and named two candidates for
//! it: *"Ohio's triennial property reappraisals landed across FY2023 and would raise local revenue
//! in the richest quartile mechanically… and the Fair School Funding Plan's own phase-in moves
//! state aid over the same years. Separating the two needs the valuation series beside this one,
//! and two observations cannot do it."*
//!
//! # Three answers, and the order they arrive in matters
//!
//! **There are not two observations.** Ohio's counties revalue on a staggered calendar and Table
//! SD-1 carries four tax years, so every district has one event year and two quiet ones. The
//! valuation jump is 24 to 55 times the background rate, so the arms separate without a threshold.
//!
//! **The mechanism runs the other way.** A reappraisal reaches revenue only where H.B. 920's
//! reduction factors have stopped, which is the twenty-mill floor, and the floor is where poor
//! districts are — 46.1% of the poorest quartile against 12.5% of the richest. Removing the
//! reappraisals from the data moves the FY2024 gap *up* by $212.
//!
//! **And the break was an artefact.** One five-pupil island district entered the survey's
//! comparable population in FY2023; see `dispersion::ohio_panel::MIN_ENROLMENT`. There was
//! nothing to explain.
//!
//! The first two do not depend on the third. Had the break been real, this is still the evidence
//! that the reappraisals did not cause it.

use regime_diff::reappraisal_incidence as incidence;

/// The calendar identifies the events, and it is not close.
#[test]
fn every_cohort_jumps_in_its_own_year_and_sits_still_in_the_others() {
    let all = incidence::districts();
    assert_eq!(
        all.len(),
        608,
        "districts both sources reach across the window"
    );

    for (event, cohort, pinned) in [
        (2022u16, 123usize, 0.216),
        (2023, 304, 0.357),
        (2024, 181, 0.320),
    ] {
        let group: Vec<&incidence::District> =
            all.iter().filter(|d| d.event_tax_year == event).collect();
        assert_eq!(group.len(), cohort, "districts on the TY{event} calendar");

        let jump = median(group.iter().filter_map(|d| growth(d, event)));
        assert!(
            (jump - pinned).abs() < 5e-3,
            "TY{event} cohort's own year moved class-I value by {jump:.4}"
        );

        // And every quiet year of the same cohort is an ordinary year.
        for quiet in [2022u16, 2023, 2024] {
            if quiet == event {
                continue;
            }
            let ordinary = median(group.iter().filter_map(|d| growth(d, quiet)));
            assert!(
                ordinary < 0.02,
                "TY{event} cohort moved {ordinary:.4} in TY{quiet}, which is not a quiet year"
            );
            assert!(
                jump > ordinary * 20.0,
                "TY{event}'s event year is {jump:.4} against a background of {ordinary:.4}, \
                 which is no longer the separation this rests on"
            );
        }
    }
}

/// The floor is where the poor districts are, and it falls monotonically across the distribution.
///
/// This is the whole answer to the node's mechanism. H.B. 920 holds a district's yield flat as
/// values rise, so a reappraisal is a revenue event only at the twenty-mill floor. If the floor
/// were a rich-district phenomenon the node's reading would follow; it is the reverse.
#[test]
fn the_districts_a_reappraisal_reaches_are_the_poorest_ones() {
    let quartiles = incidence::by_wealth();
    assert_eq!(
        quartiles.map(|q| q.districts),
        [152, 152, 152, 152],
        "districts per wealth quartile"
    );

    let at_the_floor = quartiles.map(|q| q.at_the_floor);
    for (i, expected) in [0.461, 0.395, 0.283, 0.125].iter().enumerate() {
        assert!(
            (at_the_floor[i] - expected).abs() < 5e-3,
            "quartile {i} sits {:.4} at the floor",
            at_the_floor[i]
        );
    }
    for pair in at_the_floor.windows(2) {
        assert!(
            pair[0] > pair[1],
            "the floor share stopped falling across the distribution: {at_the_floor:?}"
        );
    }
    assert!(
        at_the_floor[0] > at_the_floor[3] * 3.0,
        "the poorest quartile is {:.4} against the richest {:.4}",
        at_the_floor[0],
        at_the_floor[3]
    );
}

/// So local revenue grew fastest at the bottom, which is the opposite of the recorded reading.
#[test]
fn local_revenue_grew_fastest_in_the_quartile_with_the_least_of_it() {
    let quartiles = incidence::by_wealth();
    let growth = quartiles.map(|q| q.local_growth);
    for (i, expected) in [0.2251, 0.1856, 0.1795, 0.1129].iter().enumerate() {
        assert!(
            (growth[i] - expected).abs() < 5e-4,
            "quartile {i} grew {:.4}",
            growth[i]
        );
    }
    assert!(
        growth[0] > growth[3] * 1.8,
        "the poorest quartile grew {:.4} against the richest {:.4}",
        growth[0],
        growth[3]
    );

    // In dollars the rich still gain more, because the base is three times larger. That is the
    // arithmetic of a level gap and it is why growth rates alone would not settle the question —
    // hence the counterfactual below.
    let (poor, rich) = (quartiles[0], quartiles[3]);
    assert!(
        rich.local_per_pupil * rich.local_growth > poor.local_per_pupil * poor.local_growth,
        "the richest quartile gained {:.0} per pupil against the poorest's {:.0}",
        rich.local_per_pupil * rich.local_growth,
        poor.local_per_pupil * poor.local_growth
    );
}

/// **The answer.** Take the reappraisals out and the gap does not narrow — in FY2024 it widens.
///
/// Each district is put on its own quiet-year growth rate for the whole window, so what is removed
/// is the event and not the trend. The node's hypothesis predicts a gap that shrinks substantially
/// when this is done. It moves $26 the other way in FY2023 and $212 the wrong way in FY2024.
#[test]
fn removing_the_reappraisals_does_not_close_the_gap() {
    let fy2023 = incidence::gap(2023);
    let fy2024 = incidence::gap(2024);

    assert!(
        (fy2023.observed - 9_810.0).abs() < 1.0 && (fy2024.observed - 10_527.0).abs() < 1.0,
        "the observed gaps are {:.0} and {:.0}",
        fy2023.observed,
        fy2024.observed
    );

    assert!(
        fy2023.attributable().abs() < 100.0,
        "FY2023 attributes {:.0} of the gap to reappraisal",
        fy2023.attributable()
    );
    assert!(
        fy2024.attributable() < 0.0,
        "FY2024 attributes {:.0} to reappraisal, and the sign is the finding",
        fy2024.attributable()
    );

    // Stated against the movement it was offered to explain: the gap grew $937 across the window,
    // and reappraisal accounts for none of it.
    let grew = fy2024.observed - incidence::gap(2022).observed;
    assert!(
        fy2024.attributable().abs() < grew * 0.25,
        "reappraisal moved {:.0} of a {:.0} widening",
        fy2024.attributable(),
        grew
    );
}

/// The corrected series, restated here because this test file is where the node's question lands.
///
/// The two years the corpus recorded as a break close 44.9% and 43.9% of the gap, inside the
/// 40.0%-48.8% band the other eleven years of FY2012-FY2024 hold.
#[test]
fn the_two_years_the_corpus_called_a_break_are_inside_the_band() {
    let by_year = dispersion::ohio_panel::equalization_by_year();
    let band: Vec<f64> = by_year
        .iter()
        .filter(|(y, _)| (2012..=2024).contains(*y))
        .map(|(_, e)| e.state_share())
        .collect();
    let (low, high) = (
        band.iter().copied().fold(f64::MAX, f64::min),
        band.iter().copied().fold(f64::MIN, f64::max),
    );
    assert!(
        (low - 0.4002).abs() < 5e-4 && (high - 0.4884).abs() < 5e-4,
        "the band runs {low:.4} to {high:.4}"
    );
    for year in [2023u16, 2024] {
        let share = by_year[&year].state_share();
        assert!(
            share > low && share < high,
            "FY{year} closes {share:.4}, outside a band it should be inside"
        );
    }
}

/// Class-I value growth for one district in one tax year, or `None` where either year is absent.
fn growth(district: &incidence::District, tax_year: u16) -> Option<f64> {
    let now = district.class1_value.get(&tax_year)?;
    let before = district.class1_value.get(&(tax_year - 1))?;
    Some(now / before - 1.0)
}

fn median(values: impl Iterator<Item = f64>) -> f64 {
    let mut sample: Vec<f64> = values.collect();
    sample.sort_by(f64::total_cmp);
    sample.get(sample.len() / 2).copied().unwrap_or(f64::NAN)
}
