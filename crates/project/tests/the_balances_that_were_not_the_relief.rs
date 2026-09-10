//! The half of `metric/general-fund-cash-balance`'s open pair that the filings can answer.
//!
//! The node records that statewide ending cash rose every year from $8.37bn in FY2020 to $11.20bn
//! in FY2024 and then fell, and asks: *"How much of the FY2021-FY2024 build-up was ESSER booked in
//! the general fund, which the forecast lines cannot separate and the federal award data could."*
//!
//! # The lines can separate it
//!
//! `project::finances` carries two revenue totals. `total_revenue` is the district's own receipts
//! and excludes transfers by construction — which is why it does not close the cash identity.
//! `total_revenue_and_sources` is every dollar that entered the fund and does. Their difference is
//! transfers, advances and note proceeds: the only route relief money received into a special
//! revenue fund can take into the general fund.
//!
//! Nothing read the second column. The extractor writes it and every consumer took the first.
//!
//! # And the answer is: very little of it
//!
//! The line has the right shape — it doubles across the relief years and falls back when the
//! grants end — and the wrong size. The build-up does not track a district's relief at all
//! (r = 0.075), and under every baseline and slope the corpus can defend, relief accounts for
//! between 6% and 39% of it, most plausibly under a tenth.
//!
//! The other half of the node's pair, whether the FY2025 drawdown continues, needs a filing that
//! does not exist yet and is untouched here.

use project::transfers;

/// The column nobody had read, and the shape that makes it worth reading.
#[test]
fn transfers_into_the_general_fund_double_across_the_relief_years_and_fall_back() {
    let series = transfers::by_year();
    let at = |year: u16| series[&year].transfers() / 1e9;

    for (year, expected) in [
        (2020u16, 0.282),
        (2021, 0.362),
        (2022, 0.396),
        (2023, 0.533),
        (2024, 0.720),
        (2025, 0.434),
    ] {
        assert!(
            (at(year) - expected).abs() < 0.001,
            "FY{year} transfers are {:.4}bn",
            at(year)
        );
    }

    // Doubling and falling back is the shape relief would leave. It is necessary and nowhere near
    // sufficient, which is what the rest of this file is about.
    assert!(
        at(2024) > at(2020) * 2.0,
        "the line only reaches {:.2}x its FY2020 level",
        at(2024) / at(2020)
    );
    assert!(
        at(2025) < at(2024) * 0.7,
        "FY2025 holds {:.2} of FY2024, so the line did not fall back when the grants ended",
        at(2025) / at(2024)
    );

    // And the cash series the node states, from the same filings.
    assert!(
        (series[&2020].ending_cash / 1e9 - 8.367).abs() < 0.001
            && (series[&2024].ending_cash / 1e9 - 11.205).abs() < 0.001
            && (series[&2025].ending_cash / 1e9 - 9.135).abs() < 0.001,
        "the cash series moved and this test is reading a different panel"
    );
}

/// **The answer.** Districts that received more relief did not end up holding more cash.
#[test]
fn the_build_up_does_not_track_a_districts_relief() {
    let all = transfers::districts();
    assert_eq!(all.len(), 606, "districts all three sources reach");

    let correlation = transfers::build_up_against_relief();
    assert!(
        (correlation - 0.075).abs() < 5e-3,
        "relief against the cash build-up is {correlation:.4}"
    );
    assert!(
        correlation.abs() < 0.15,
        "and it is {correlation:.4}, which would be a relationship"
    );

    // The exposure split is real, so the null is not a failure to separate the groups: the most
    // relieved quartile received nearly three times the least.
    let size = all.len() / 4;
    let median = |slice: &[transfers::District], of: fn(&transfers::District) -> f64| {
        let mut sample: Vec<f64> = slice.iter().map(of).collect();
        sample.sort_by(f64::total_cmp);
        sample[sample.len() / 2]
    };
    let (poorest, richest) = (&all[..size], &all[3 * size..]);
    assert!(
        median(richest, |d| d.relief_per_pupil) > median(poorest, |d| d.relief_per_pupil) * 2.5,
        "the exposure split is {:.0} against {:.0}",
        median(poorest, |d| d.relief_per_pupil),
        median(richest, |d| d.relief_per_pupil)
    );
}

/// How much of the build-up relief can be held responsible for, under both baselines.
///
/// The point of asserting all four numbers rather than one is that the spread is the finding. A
/// counterfactual for the transfers line is a level it would have run at without the grants, and
/// the panel holds no clean one — FY2020 already carries ESSER I and FY2025 carries ESSER III's
/// liquidation. What survives the choice is the ceiling.
#[test]
fn relief_accounts_for_under_two_fifths_of_the_build_up_on_any_reading() {
    let mut widest: f64 = 0.0;
    for (against_both, excess_share, robust, least_squares) in [
        (false, 0.4692, 0.1508, 0.3938),
        (true, 0.3131, 0.0638, 0.0878),
    ] {
        let attribution = transfers::attribution(against_both);
        assert!(
            (attribution.excess_share() - excess_share).abs() < 5e-4,
            "excess transfers are {:.4} of the build-up against both={against_both}",
            attribution.excess_share()
        );
        let (robust_share, least_squares_share) = attribution.attributable();
        assert!(
            (robust_share - robust).abs() < 5e-4
                && (least_squares_share - least_squares).abs() < 5e-4,
            "against both={against_both} the two slopes give {robust_share:.4} and \
             {least_squares_share:.4}"
        );
        widest = widest.max(robust_share).max(least_squares_share);

        // The excess is a ceiling on the attributable part, and it has to stay one — a slope that
        // attributed more relief than there were excess dollars would be fitting noise.
        assert!(
            least_squares_share < attribution.excess_share(),
            "attributed {least_squares_share:.4} of a build-up whose excess is only {:.4}",
            attribution.excess_share()
        );
    }

    assert!(
        widest < 0.4,
        "the largest attributable share across both baselines is {widest:.4}"
    );
}

/// The aggregates the shares above are taken over, so a fixture revision moves a number and not a
/// conclusion silently.
#[test]
fn the_build_up_and_the_relief_it_is_read_against() {
    let attribution = transfers::attribution(true);
    assert!(
        (attribution.build_up / 1e9 - 1.833).abs() < 0.001,
        "the build-up over these districts is {:.4}bn",
        attribution.build_up / 1e9
    );
    assert!(
        (attribution.relief / 1e9 - 2.565).abs() < 0.001,
        "the relief they received is {:.4}bn",
        attribution.relief / 1e9
    );

    // The relief is larger than the build-up, which is what makes the question worth asking: on
    // size alone the balances could have been the grants.
    assert!(
        attribution.relief > attribution.build_up,
        "the relief is {:.3}bn against a build-up of {:.3}bn, and the question presupposes it is \
         the larger",
        attribution.relief / 1e9,
        attribution.build_up / 1e9
    );
}

/// The control from the other side, restated: the grants never entered as revenue.
///
/// `project::esser` established this while answering a different question — federal revenue rising
/// 139% across the peak while general-fund revenue fell. It is what makes the transfers line the
/// only route left, and so what makes the small number above an answer rather than a gap.
#[test]
fn the_grants_did_not_enter_the_general_fund_as_revenue() {
    let series = project::esser::federal_against_the_general_fund();
    let federal = |year: u16| series[&year].federal_revenue.expect("federal");
    assert!(
        federal(2022) / federal(2019) > 2.3,
        "the relief bulge is {:.2}x",
        federal(2022) / federal(2019)
    );

    let before = series[&2020].general_fund_revenue.expect("FY2020");
    let at_the_peak = series[&2022].general_fund_revenue.expect("FY2022");
    assert!(
        at_the_peak < before,
        "general-fund revenue went {before:.0} to {at_the_peak:.0}, so the grants may have \
         entered it after all"
    );
}
