//! The four things the corpus's worked pair recorded as not held, all of them committed.
//!
//! `education-agency/perrysburg-exempted-village` closes on "Not yet held: valuation per pupil,
//! millage, guarantee status, and anything before FY2024", and `education-agency/toledo-city` on
//! the same list plus audited-statement figures. Three of the four are in the profile report and
//! the FY2027 model; the fourth is in `dispersion::ohio_panel`, which opens at FY2009.
//!
//! # The pair, over fifteen years
//!
//! Both districts lost state aid in real terms. Only Perrysburg could replace it: its local
//! revenue per pupil grew 22.3% in constant dollars while Toledo's fell 8.1%, and Perrysburg's
//! enrolment grew 20% while Toledo's fell 21%.
//!
//! # And they are not separated by effort
//!
//! Perrysburg's *effective* Class I rate is the higher of the two, both are rolled back by about
//! 45%, and neither is on the twenty-mill floor. Perrysburg is worth twice as much per pupil.
//! The gap is the base.
//!
//! # The break, and what it does not disturb
//!
//! Toledo's state revenue per pupil falls about 30% in real terms at FY2016 and stays there. It
//! is a tail rather than a trend — the median district *rises* — and it leaves the equalization
//! band `doctrine/equity` publishes untouched, because a quartile mean over 152 districts cannot
//! see six large ones move.

use dispersion::exemplars::{self, PERRYSBURG, TOLEDO};

/// The history exists, on both districts, for every year the panel holds.
#[test]
fn the_panel_carries_both_districts_from_fy2009() {
    for irn in [TOLEDO, PERRYSBURG] {
        let years = exemplars::history(irn);
        assert_eq!(years.len(), 15, "years for {irn}");
        assert_eq!(years[0].fiscal_year, exemplars::SPAN[0]);
        assert_eq!(years[14].fiscal_year, exemplars::SPAN[1]);
        // FY2014 is absent from the archive, which is why fifteen years span sixteen.
        assert!(
            !years.iter().any(|y| y.fiscal_year == 2014),
            "the panel has gained FY2014, so `SPAN` no longer skips it"
        );
    }
}

/// **The finding.** The state's real contribution fell for both, and only one could make it up.
#[test]
fn both_lost_state_aid_in_real_terms_and_only_perrysburg_replaced_it() {
    let toledo_state = exemplars::change(TOLEDO, |y| y.state);
    let perrysburg_state = exemplars::change(PERRYSBURG, |y| y.state);
    assert!(
        (toledo_state.real + 0.343).abs() < 0.005,
        "Toledo state real change is {:+.4}",
        toledo_state.real
    );
    assert!(
        (perrysburg_state.real + 0.228).abs() < 0.005,
        "Perrysburg state real change is {:+.4}",
        perrysburg_state.real
    );
    assert!(
        toledo_state.real < 0.0 && perrysburg_state.real < 0.0,
        "the shared half of the finding is that both fell"
    );

    // Toledo's state revenue per pupil is below its FY2009 level in *nominal* dollars, which is
    // the form of the fact that needs no deflator to be startling.
    assert!(
        toledo_state.nominal < 0.0 && toledo_state.to < toledo_state.from,
        "Toledo: {:.0} to {:.0}",
        toledo_state.from,
        toledo_state.to
    );

    let toledo_local = exemplars::change(TOLEDO, |y| y.local);
    let perrysburg_local = exemplars::change(PERRYSBURG, |y| y.local);
    assert!(
        (toledo_local.real + 0.081).abs() < 0.005 && (perrysburg_local.real - 0.223).abs() < 0.005,
        "local real: Toledo {:+.4}, Perrysburg {:+.4}",
        toledo_local.real,
        perrysburg_local.real
    );
    assert!(
        toledo_local.real < 0.0 && perrysburg_local.real > 0.0,
        "the divergence is the finding and the two should have opposite signs"
    );

    // Both nominal series rise, which is how a district can lose ground while its own accounts
    // show growth every year.
    assert!(toledo_local.nominal > 0.3 && perrysburg_local.nominal > 0.7);

    // And the populations moved in opposite directions too.
    let enrolment = |irn| exemplars::change(irn, |y| y.enrolment).nominal;
    assert!(
        enrolment(TOLEDO) < -0.2 && enrolment(PERRYSBURG) > 0.19,
        "enrolment: Toledo {:+.3}, Perrysburg {:+.3}",
        enrolment(TOLEDO),
        enrolment(PERRYSBURG)
    );
}

/// The three standing measures the nodes said were not held, and the surprise in them.
#[test]
fn neither_district_is_on_the_floor_and_the_wealthier_one_taxes_harder() {
    let toledo = exemplars::standing(TOLEDO).expect("Toledo joins all three panels");
    let perrysburg = exemplars::standing(PERRYSBURG).expect("Perrysburg joins all three panels");

    assert!(
        (toledo.valuation_per_pupil - 135_506.0).abs() < 1.0
            && (perrysburg.valuation_per_pupil - 270_202.0).abs() < 1.0,
        "valuation per pupil: {:.0} and {:.0}",
        toledo.valuation_per_pupil,
        perrysburg.valuation_per_pupil
    );
    assert!(
        perrysburg.valuation_per_pupil > toledo.valuation_per_pupil * 1.9,
        "the wealth ratio is {:.2}",
        perrysburg.valuation_per_pupil / toledo.valuation_per_pupil
    );

    // **The surprise.** The richer district's effective rate is the higher one, so the disparity
    // is not a story about a poor district declining to tax itself.
    assert!(
        perrysburg.effective_millage > toledo.effective_millage,
        "effective millage: Toledo {:.2}, Perrysburg {:.2}",
        toledo.effective_millage,
        perrysburg.effective_millage
    );
    assert!(
        (toledo.effective_millage - 35.55).abs() < 0.01
            && (perrysburg.effective_millage - 39.15).abs() < 0.01
    );

    // H.B. 920 removes about the same fraction from each, so it is not the differentiator either.
    assert!(
        (toledo.rollback() - perrysburg.rollback()).abs() < 0.03,
        "rollbacks are {:.3} and {:.3}",
        toledo.rollback(),
        perrysburg.rollback()
    );
    assert!(toledo.rollback() > 0.4 && perrysburg.rollback() > 0.4);

    // And neither is at the floor, so neither is held there by the mechanism `twenty-mill-floor`
    // is about.
    assert!(!toledo.at_floor && !perrysburg.at_floor);
}

/// The pair is the worked case of the composition finding, at the two ends of the distribution.
#[test]
fn the_pair_sits_at_opposite_ends_of_the_classroom_share_distribution() {
    let toledo = exemplars::standing(TOLEDO).expect("Toledo");
    let perrysburg = exemplars::standing(PERRYSBURG).expect("Perrysburg");

    assert!(
        (toledo.classroom_share - 63.43).abs() < 0.05
            && (perrysburg.classroom_share - 73.11).abs() < 0.05,
        "shares are {:.2} and {:.2}",
        toledo.classroom_share,
        perrysburg.classroom_share
    );
    assert!(
        toledo.classroom_share_percentile < 20.0 && perrysburg.classroom_share_percentile > 94.0,
        "percentiles are {:.1} and {:.1}",
        toledo.classroom_share_percentile,
        perrysburg.classroom_share_percentile
    );

    // Toledo spends more per pupil and less of it in the classroom, which is the state-wide
    // relationship `dispersion::composition` measures, at its two extremes.
    let frame = dispersion::composition::frame();
    let of = |irn: &str| {
        frame
            .iter()
            .find(|d| d.irn == irn)
            .expect("in the frame")
            .clone()
    };
    let (t, p) = (of(TOLEDO), of(PERRYSBURG));
    assert!(
        t.operating > p.operating && t.classroom_share() < p.classroom_share(),
        "Toledo {:.0} at {:.2}, Perrysburg {:.0} at {:.2}",
        t.operating,
        t.classroom_share(),
        p.operating,
        p.classroom_share()
    );
    // The gap opens in plant and building administration, not in instruction.
    assert!(t.operations_maintenance > p.operations_maintenance * 2.5);
    assert!(t.school_admin > p.school_admin * 2.4);
    assert!(
        t.instruction > p.instruction && t.instruction < p.instruction * 1.25,
        "instruction is {:.0} against {:.0}",
        t.instruction,
        p.instruction
    );
}

/// The break in Toledo's series, bounded rather than explained.
#[test]
fn the_fy2016_step_is_a_tail_and_not_a_trend() {
    let steps = exemplars::fy2016_step();
    assert!(steps.len() > 590, "districts measured: {}", steps.len());

    let median = steps[steps.len() / 2].1;
    assert!(
        median > 0.05,
        "the median district's step is {median:+.4}, so this is not a general fall"
    );

    let deep: Vec<&(String, f64, f64)> = steps.iter().filter(|s| s.1 < -0.20).collect();
    assert_eq!(deep.len(), 27, "districts falling more than a fifth");
    let pupils: f64 = deep.iter().map(|s| s.2).sum();
    let all: f64 = steps.iter().map(|s| s.2).sum();
    assert!(
        (pupils / all - 0.067).abs() < 0.005,
        "the tail holds {:.4} of the panel's pupils",
        pupils / all
    );

    // Toledo is in the fall and Perrysburg is not, which is what makes the pair readable at all.
    let step = |irn: &str| steps.iter().find(|s| s.0 == irn).expect("in the panel").1;
    assert!(
        step(TOLEDO) < -0.15,
        "Toledo's step is {:+.4}",
        step(TOLEDO)
    );
    assert!(
        step(PERRYSBURG).abs() < 0.10,
        "Perrysburg's step is {:+.4}",
        step(PERRYSBURG)
    );
}

/// And the break leaves the corpus's published equalization series alone.
///
/// The reason is the one `a break in a series is a population change first` records from the other
/// direction: a quartile mean over about 152 districts is one tiny district wide, and by the same
/// arithmetic it is six enormous districts deep. Checked rather than assumed, because a 30% fall
/// in Ohio's fourth-largest district running through a published band would be a correction.
#[test]
fn the_break_does_not_reach_the_equalization_band() {
    let series = dispersion::ohio_panel::equalization_by_year();
    let poorest = |year: u16| {
        let rows = dispersion::ohio_panel::panel();
        let mut quartile: Vec<(f64, f64)> = rows
            .iter()
            .filter(|r| {
                r.comparable
                    && r.fiscal_year == year
                    && r.enrollment >= dispersion::ohio_panel::MIN_ENROLMENT
            })
            .map(|r| {
                (
                    r.local_revenue / r.enrollment,
                    r.state_revenue / r.enrollment,
                )
            })
            .collect();
        quartile.sort_by(|a, b| a.0.total_cmp(&b.0));
        let cut = quartile.len() / 4;
        quartile[..cut].iter().map(|r| r.1).sum::<f64>() / cut as f64
    };

    // The poorest quartile's state revenue per pupil *rises* across the break.
    assert!(
        poorest(2016) > poorest(2015),
        "poorest quartile state revenue went {:.0} to {:.0}",
        poorest(2015),
        poorest(2016)
    );

    // And the published band holds either side of it.
    for year in [2015u16, 2016, 2017] {
        let entry = &series[&year];
        let rate = entry.state_closes / entry.gap;
        assert!(
            rate > 0.39 && rate < 0.50,
            "FY{year} closes {rate:.4} of the gap, outside the band the corpus states"
        );
    }
}

/// `state_revenue` carries capital money and `current_spending` does not, so one year of one
/// district's state revenue is not its aid.
///
/// The caution the history has to be read under, asserted on the district that makes it obvious.
#[test]
fn a_build_year_puts_state_revenue_above_current_spending() {
    let carey = exemplars::history("045260");
    let fy2015 = carey
        .iter()
        .find(|y| y.fiscal_year == 2015)
        .expect("Carey is in the panel");
    assert!(
        fy2015.state * fy2015.enrolment / 1e6 > 20.0,
        "Carey's FY2015 state revenue is ${:.1}m",
        fy2015.state * fy2015.enrolment / 1e6
    );
    assert!(
        fy2015
            .revenue_over_spending()
            .expect("Carey reports spending")
            > 3.0,
        "Carey's FY2015 revenue is {:.2}x its current spending",
        fy2015.revenue_over_spending().unwrap()
    );

    // Its own neighbours in time are ordinary, so this is a year and not a district.
    for year in [2013u16, 2016] {
        let row = carey
            .iter()
            .find(|y| y.fiscal_year == year)
            .expect("a year");
        assert!(
            row.revenue_over_spending().expect("spending") < 1.6,
            "FY{year} is {:.2}x",
            row.revenue_over_spending().unwrap()
        );
    }
}
