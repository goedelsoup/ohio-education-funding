//! Two more nodes whose "remains unpopulated" was a statement about the corpus, not about Ohio.
//!
//! `education-agency/electronic-classroom-of-tomorrow` closes on "Peak enrollment and per-district
//! deduction impact remain unpopulated", and `education-agency/cleveland-municipal` on
//! "Scholarship dollar amounts and the state funding series by year remain unpopulated".
//!
//! `dispersion::ohio_panel` carries Cleveland in all fifteen years it holds and **ECOT in eight**,
//! FY2009 through FY2017 with FY2014 missing as it is for everyone — the school was a community
//! school and therefore never `comparable`, which is exactly why every district measure in this
//! crate looks past it and why nobody had looked.
//!
//! # ECOT, measured
//!
//! Peak enrolment is **14,153 in FY2016**, nearly double its FY2009 count. Peak state revenue is
//! **$109.4m in the same year**, which is independent corroboration of the "$108 million in state
//! funding" the node quotes from a hearing officer's report this corpus does not hold — and FY2016
//! is the year the State Board voted to claw $60m of it back.
//!
//! Across the eight years the panel holds, ECOT received **$666.6m** of state money. In FY2016
//! that made it the **eighth largest recipient of state education money among all 982 Ohio
//! agencies**, behind seven city districts and ahead of every other community school in the state
//! by more than half again.
//!
//! # And it was funded above the districts it drew from
//!
//! ECOT's state revenue per pupil runs $7,249 to $7,913 across the eight years, against a
//! statewide district average of $5,241 to $6,036 — about **1.40 times** the average district in
//! FY2016, every year and never below 1.28. Under the deduct mechanism that money was subtracted from
//! resident districts' foundation payments, so the comparison is not incidental: districts gave up
//! state aid to fund a school receiving more per pupil than they did.
//!
//! What the panel cannot do is apportion it. `share_of_district_state_revenue` gives 1.24% of what
//! Ohio's districts received in FY2016, which is the aggregate; which districts and in what
//! proportion still needs the department's deduct file.

use dispersion::exemplars::{self, CLEVELAND, ECOT};

/// The panel carries both agencies, and ECOT's run ends where the school did.
#[test]
fn the_panel_carries_ecot_for_eight_years_and_stops_when_it_closed() {
    let ecot = exemplars::history(ECOT);
    assert_eq!(ecot.len(), 8, "years the panel holds ECOT");
    assert_eq!(ecot[0].fiscal_year, 2009);
    assert_eq!(ecot[7].fiscal_year, 2017, "ECOT closed in January 2018");
    // Eight years across a nine-year span, because FY2014 is absent from the archive for every
    // agency — not because ECOT is missing from it.
    assert!(!ecot.iter().any(|y| y.fiscal_year == 2014));

    // It is never comparable, which is why no district measure in this crate sees it.
    assert!(
        !dispersion::ohio_panel::panel()
            .iter()
            .any(|r| r.irn == ECOT && r.comparable),
        "ECOT is being counted as a district somewhere"
    );

    // And the directory agrees on both ends, which is what makes the panel's silence after FY2017
    // a closure rather than a gap.
    let editions: Vec<u16> = dispersion::lea_directory::panel()
        .into_iter()
        .filter(|a| a.irn == ECOT)
        .map(|a| a.opens)
        .collect();
    assert_eq!(editions.first(), Some(&2000));
    assert_eq!(editions.last(), Some(&2018));

    let cleveland = exemplars::history(CLEVELAND);
    assert_eq!(cleveland.len(), 15, "years the panel holds Cleveland");
}

/// **The number the node said was unpopulated.**
#[test]
fn ecots_enrolment_peaked_at_fourteen_thousand_in_the_year_of_the_first_clawback() {
    let (year, peak) = exemplars::peak(ECOT, |y| y.enrolment).expect("ECOT is in the panel");
    assert_eq!(year, 2016, "the peak year");
    assert!((peak - 14_153.0).abs() < 1.0, "peak enrolment is {peak:.0}");

    // It nearly doubled over the panel's run.
    let growth =
        exemplars::change_between(ECOT, 2009, 2016, |y| y.enrolment).expect("both years present");
    assert!(
        growth.nominal > 0.94 && growth.nominal < 0.95,
        "enrolment grew {:+.4}",
        growth.nominal
    );

    // And the state money peaked in the same year, at the figure the node quotes from a source
    // this corpus does not hold.
    let (revenue_year, revenue) =
        exemplars::peak(ECOT, |y| y.state * y.enrolment).expect("ECOT is in the panel");
    assert_eq!(revenue_year, 2016);
    assert!(
        (revenue / 1e6 - 109.371).abs() < 0.01,
        "peak state revenue is ${:.3}m",
        revenue / 1e6
    );
    // The node states "$108 million in state funding" for the audited year. The survey is a
    // different source on the same quantity and lands within two per cent of it.
    assert!(
        (revenue / 1e6 / 108.0 - 1.0).abs() < 0.02,
        "the two sources disagree by {:.3}",
        revenue / 1e6 / 108.0 - 1.0
    );
}

/// How large it was, stated against the agencies it was drawing from.
#[test]
fn ecot_was_the_eighth_largest_recipient_of_state_money_in_ohio() {
    assert_eq!(exemplars::rank_by_state_revenue(ECOT, 2016), Some(8));

    // The seven ahead of it are all traditional city districts, so it was the largest community
    // school in the state by state money by a wide margin.
    let mut agencies: Vec<(f64, String, bool)> = dispersion::ohio_panel::panel()
        .into_iter()
        .filter(|r| r.fiscal_year == 2016)
        .map(|r| (r.state_revenue, r.irn, r.comparable))
        .collect();
    agencies.sort_by(|a, b| b.0.total_cmp(&a.0));
    assert!(
        agencies[..7].iter().all(|(_, _, comparable)| *comparable),
        "one of the seven above ECOT is not a district"
    );
    let next_community = agencies[8..]
        .iter()
        .find(|(_, _, comparable)| !comparable)
        .expect("another community school");
    assert!(
        agencies[7].0 > next_community.0 * 1.5,
        "ECOT at ${:.1}m against the next community school at ${:.1}m",
        agencies[7].0 / 1e6,
        next_community.0 / 1e6
    );

    // Eight years of it.
    let total: f64 = exemplars::history(ECOT)
        .iter()
        .map(|y| y.state * y.enrolment)
        .sum();
    assert!(
        (total / 1e6 - 666.6).abs() < 0.5,
        "ECOT received ${:.1}m across the panel's run",
        total / 1e6
    );
}

/// **The half of the deduction question the panel can answer.**
#[test]
fn ecot_was_funded_above_the_districts_its_money_came_out_of() {
    let averages = exemplars::district_state_revenue();
    let mut ratios = Vec::new();
    for year in exemplars::history(ECOT) {
        let (total, enrolment) = averages[&year.fiscal_year];
        ratios.push((year.fiscal_year, year.state / (total / enrolment)));
    }

    assert!(
        ratios.iter().all(|(_, ratio)| *ratio > 1.27),
        "ECOT falls below 1.27x the average district somewhere: {ratios:?}"
    );
    let fy2016 = ratios
        .iter()
        .find(|(year, _)| *year == 2016)
        .expect("FY2016")
        .1;
    assert!(
        (fy2016 - 1.3995).abs() < 0.005,
        "FY2016 ratio is {fy2016:.4}"
    );

    // The aggregate draw, which is what a per-district apportionment would have to sum to.
    let share = exemplars::share_of_district_state_revenue(ECOT, 2016).expect("FY2016");
    assert!(
        (share - 0.0124).abs() < 0.0002,
        "ECOT drew {share:.4} of what Ohio's districts received"
    );
    // It roughly doubled as a share across the panel's run, which is the growth the node's
    // narrative is about seen from the districts' side.
    let opening = exemplars::share_of_district_state_revenue(ECOT, 2009).expect("FY2009");
    assert!(
        share > opening * 2.0,
        "the draw went from {opening:.4} to {share:.4}"
    );
}

/// Cleveland's state funding series, which the node said it did not have.
#[test]
fn clevelands_state_series_runs_from_fy2009_and_its_enrolment_fell_a_third() {
    let state = exemplars::change(CLEVELAND, |y| y.state);
    assert!(
        (state.from - 11_103.0).abs() < 1.0 && (state.to - 11_011.0).abs() < 1.0,
        "state revenue per pupil went {:.0} to {:.0}",
        state.from,
        state.to
    );
    // Flat in nominal dollars across fifteen years is a third of its value gone.
    assert!(
        state.nominal.abs() < 0.02 && state.real < -0.29,
        "nominal {:+.4}, real {:+.4}",
        state.nominal,
        state.real
    );

    let enrolment = exemplars::change(CLEVELAND, |y| y.enrolment);
    assert!(
        (enrolment.nominal + 0.3225).abs() < 0.005,
        "enrolment change is {:+.4}",
        enrolment.nominal
    );

    // And its share of the state's money to districts fell with it, from 5.7% to 3.7%.
    let opening = exemplars::share_of_district_state_revenue(CLEVELAND, 2009).expect("FY2009");
    let closing = exemplars::share_of_district_state_revenue(CLEVELAND, 2024).expect("FY2024");
    assert!(
        (opening - 0.0572).abs() < 0.0005 && (closing - 0.0366).abs() < 0.0005,
        "share went {opening:.4} to {closing:.4}"
    );
}

/// Cleveland shows the FY2016 step Toledo does, and unlike Toledo it comes back.
///
/// Recorded because `education-agency/toledo-city` opens the question of what the step was, and a
/// second large district behaving differently across it is evidence about the answer rather than
/// a repetition of the observation.
#[test]
fn cleveland_falls_at_fy2016_and_recovers_where_toledo_does_not() {
    let at = |irn: &str, year: u16| {
        exemplars::history(irn)
            .into_iter()
            .find(|y| y.fiscal_year == year)
            .expect("a year the panel holds")
            .state
    };

    // Both fall hard at FY2016.
    for irn in [CLEVELAND, exemplars::TOLEDO] {
        assert!(
            at(irn, 2016) < at(irn, 2015) * 0.75,
            "{irn} FY2016 is {:.0} against FY2015's {:.0}",
            at(irn, 2016),
            at(irn, 2015)
        );
    }

    // Cleveland regains its FY2015 level in nominal dollars by FY2024; Toledo does not.
    assert!(
        at(CLEVELAND, 2024) > at(CLEVELAND, 2015) * 0.87,
        "Cleveland recovers to {:.0} against {:.0}",
        at(CLEVELAND, 2024),
        at(CLEVELAND, 2015)
    );
    assert!(
        at(exemplars::TOLEDO, 2024) < at(exemplars::TOLEDO, 2015),
        "Toledo is at {:.0} against {:.0}",
        at(exemplars::TOLEDO, 2024),
        at(exemplars::TOLEDO, 2015)
    );
}
