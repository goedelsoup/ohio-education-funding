//! The FY2016 question `education-agency/toledo-city` opened, closed against the survey itself.
//!
//! The node recorded that Toledo's state revenue per pupil falls about 30% at FY2016 and never
//! recovers, that twenty-seven districts fall more than a fifth while the median district rises,
//! and that the cause was not established. `dispersion::fy2016` then explained the median rise as
//! H.B. 64's capacity aid and left the tail open against three named non-explanations.
//!
//! # There is no tail
//!
//! From FY2016 the Census Bureau subtracts a district's payments to community schools from its
//! general formula assistance, to remove the double count Ohio's deduct creates, and before
//! FY2016 it does not. The panel's state column is therefore two quantities, and a district's
//! apparent fall across that year is the size of its own deduct.
//!
//! On one basis **thirteen of Ohio's fourteen largest districts gained** across the step, and the
//! twenty-seven deep fallers become nineteen holding a sixth as many pupils.
//!
//! # What survives
//!
//! The industrial reading stays refuted and the capacity-aid finding stays standing — see
//! `what_fell_out_of_the_state_column.rs`, which is rewritten onto the corrected basis rather
//! than deleted, because the two withdrawals are different and only one of them was this module's.

use dispersion::survey_basis::{self, Basis};

/// **The locator.** A district's deduct predicts its change in exactly one year.
#[test]
fn the_deduct_predicts_the_move_at_fy2016_and_at_no_other_transition() {
    let transitions = survey_basis::transitions();
    assert_eq!(
        (
            transitions.len(),
            transitions[0].from,
            transitions.last().expect("pairs").to
        ),
        (11, 2010, 2022),
        "the pairs the deduct can be measured across"
    );

    let (broken, rest): (
        Vec<&survey_basis::Transition>,
        Vec<&survey_basis::Transition>,
    ) = transitions.iter().partition(|t| t.to == 2016);
    assert_eq!(broken.len(), 1);
    let at_break = broken[0];
    assert_eq!(at_break.from, 2015);
    assert!(
        (at_break.as_published + 0.2506).abs() < 0.0005,
        "the published column reads {:+.4} against the deduct",
        at_break.as_published
    );

    // Putting both years on one basis removes it.
    assert!(
        (at_break.corrected + 0.0183).abs() < 0.0005,
        "corrected, it reads {:+.4}",
        at_break.corrected
    );

    // And no ordinary year comes close, on either basis, so the threshold separates rather than
    // fits: the gap between FY2016 and the next largest is a factor of two and a half.
    let loudest = rest
        .iter()
        .map(|t| t.as_published.abs())
        .fold(0.0_f64, f64::max);
    assert!(
        (loudest - 0.1026).abs() < 0.0005,
        "the loudest ordinary year reads {loudest:.4}"
    );
    assert!(at_break.as_published.abs() > loudest * 2.0);
    assert_eq!(survey_basis::break_year(), Some(2016));
}

/// **The withdrawal.** Every big city gained across the year it was published as losing.
#[test]
fn the_five_districts_the_corpus_published_as_falling_all_gained() {
    let published = survey_basis::step(Basis::AsPublished);
    let corrected = survey_basis::step(Basis::Gross);
    let of = |steps: &[(String, f64, f64)], irn: &str| {
        steps
            .iter()
            .find(|(i, _, _)| i == irn)
            .unwrap_or_else(|| panic!("{irn} is not in the step"))
            .1
    };

    // (name, IRN, as published, on one basis)
    for (name, irn, was, now) in [
        ("Columbus", "043802", -0.2922, 0.1493),
        ("Toledo", "044909", -0.2049, 0.1388),
        ("Cleveland", "043786", -0.1955, 0.0787),
        ("Cincinnati", "043752", -0.1577, 0.0612),
        ("Dayton", "043844", -0.0350, 0.3214),
    ] {
        assert!(
            (of(&published, irn) - was).abs() < 0.0005,
            "{name} published {:+.4}, not {was:+.4}",
            of(&published, irn)
        );
        assert!(
            (of(&corrected, irn) - now).abs() < 0.0005,
            "{name} corrected {:+.4}, not {now:+.4}",
            of(&corrected, irn)
        );
        assert!(of(&corrected, irn) > 0.0, "{name} should gain");
    }

    // The sign flip is not confined to the five: of the fourteen largest districts, thirteen gain.
    let mut largest = corrected.clone();
    largest.sort_by(|a, b| b.2.total_cmp(&a.2));
    let gained = largest[..14].iter().filter(|(_, s, _)| *s > 0.0).count();
    assert_eq!(gained, 13, "of the fourteen largest districts");
}

/// The pair of numbers the node published, and what they become.
#[test]
fn twenty_seven_districts_holding_a_fifteenth_of_the_state_become_nineteen_holding_a_sixtieth() {
    let (count, share) = survey_basis::fallers(Basis::AsPublished, 0.20);
    assert_eq!(
        count, 27,
        "districts falling more than a fifth, as published"
    );
    assert!(
        (share - 0.0670).abs() < 0.0005,
        "they hold {share:.4} of the panel's pupils"
    );

    let (count, share) = survey_basis::fallers(Basis::Gross, 0.20);
    assert_eq!(count, 19, "and on one basis");
    assert!((share - 0.0166).abs() < 0.0005, "holding {share:.4}");

    // The median district rises on both, and rises further once the artifact is out of it.
    let median = |basis| {
        let steps = survey_basis::step(basis);
        steps[steps.len() / 2].1
    };
    assert!((median(Basis::AsPublished) - 0.0956).abs() < 0.0005);
    assert!((median(Basis::Gross) - 0.1374).abs() < 0.0005);
    assert!((median(Basis::Net) - 0.1416).abs() < 0.0005);
}

/// **The mechanism, stated as an identity.** The fall is the deduct.
#[test]
fn the_statewide_fall_is_the_size_of_the_deduct_it_stopped_counting() {
    let (fall, deduct) = dispersion::fy2016::fall_against_deduct();
    assert!(
        (fall / 1e9 + 0.8499).abs() < 0.0005,
        "districts fell ${:.4}bn",
        fall / 1e9
    );
    assert!(
        (deduct / 1e9 - 0.9200).abs() < 0.0005,
        "and reported ${:.4}bn of deduct",
        deduct / 1e9
    );
    // The fall is not *more* than the deduct, which is what rules out money having left as well.
    assert!(
        fall.abs() < deduct,
        "the fall exceeds what stopped counting"
    );
    assert!(
        fall.abs() / deduct > 0.9,
        "the fall is {:.3} of the deduct",
        fall.abs() / deduct
    );

    // Per district, the same thing: the deduct is a third of what the big cities were credited
    // with, and they fall by about a third.
    for (name, irn, share) in [
        ("Columbus", "043802", 0.3675),
        ("Toledo", "044909", 0.3529),
        ("Cincinnati", "043752", 0.3234),
        ("Cleveland", "043786", 0.2887),
        ("Dayton", "043844", 0.2800),
    ] {
        let measured = survey_basis::deduct_share(irn, 2015).expect("FY2015");
        assert!(
            (measured - share).abs() < 0.0005,
            "{name}'s deduct is {measured:.4} of its credited state revenue"
        );
    }
}

/// The break reaches the local share too, in the other direction.
#[test]
fn most_of_the_jump_in_ohios_reported_local_share_at_fy2016_is_the_adjustment() {
    let published = survey_basis::shares(Basis::AsPublished);
    let corrected = survey_basis::shares(Basis::Net);

    let (was, now) = (published[&2015].0, published[&2016].0);
    assert!(
        (was - 0.5019).abs() < 0.0005 && (now - 0.5284).abs() < 0.0005,
        "the published local share goes {was:.4} to {now:.4}"
    );
    let (rebased, unchanged) = (corrected[&2015].0, corrected[&2016].0);
    assert!(
        (rebased - 0.5239).abs() < 0.0005,
        "on one basis FY2015 is {rebased:.4}"
    );
    // FY2016 needs no restating, which is the point: only the earlier years move.
    assert!((unchanged - now).abs() < 1e-12);

    let apparent = now - was;
    let real = unchanged - rebased;
    assert!(
        real / apparent < 0.2,
        "the adjustment accounts for {:.1}% of the jump",
        100.0 * (1.0 - real / apparent)
    );
}

/// The two years the correction cannot be applied to, and why.
#[test]
fn the_deduct_column_is_sound_from_fy2010_to_fy2021_and_says_so_by_being_zero_outside_them() {
    let panel = dispersion::ohio_panel::panel();
    let zeros = |year: u16| {
        let rows: Vec<_> = panel
            .iter()
            .filter(|r| r.comparable && r.fiscal_year == year)
            .collect();
        let zero = rows
            .iter()
            .filter(|r| r.charter_payments.unwrap_or(0.0) == 0.0)
            .count();
        (zero, rows.len())
    };

    // FY2009 and FY2022 report a deduct nobody believes: Toledo and Dayton are among FY2009's
    // zeros and both report tens of millions the following year.
    let (zero_2009, all_2009) = zeros(2009);
    assert!(
        zero_2009 > 100,
        "FY2009 has {zero_2009} zeros of {all_2009}, which would make it usable"
    );
    let (zero_2022, all_2022) = zeros(2022);
    assert!(
        zero_2022 * 2 > all_2022,
        "FY2022 has {zero_2022} zeros of {all_2022}"
    );
    for (name, irn) in [("Toledo", "044909"), ("Dayton", "043844")] {
        let at = |year: u16| {
            panel
                .iter()
                .find(|r| r.irn == irn && r.fiscal_year == year)
                .and_then(|r| r.charter_payments)
                .expect("a year the panel holds")
        };
        assert_eq!(at(2009), 0.0, "{name} reports a FY2009 deduct after all");
        assert!(at(2010) > 40e6, "{name} reports ${:.0} in FY2010", at(2010));
    }

    // So the two bases stop where the column stops being believable, and between them they cover
    // every year: gross to FY2021, net from FY2010 to the end of the panel.
    let covered = |basis: Basis| {
        let mut years: Vec<u16> = panel
            .iter()
            .filter(|r| r.comparable && basis.of(r).is_some())
            .map(|r| r.fiscal_year)
            .collect();
        years.sort_unstable();
        years.dedup();
        years
    };
    let gross = covered(Basis::Gross);
    assert_eq!((gross[0], *gross.last().unwrap()), (2009, 2021));
    let net = covered(Basis::Net);
    assert_eq!((net[0], *net.last().unwrap()), (2010, 2024));
}
