//! The limit of what an appropriation series can show, measured rather than asserted.
//!
//! `casino-tax-distribution` carried an open question for several phases: whether the General
//! Assembly set foundation aid lower because casino money reaches districts outside the formula.
//! It said settling this "needs the appropriation history, which is `lsc-budget` work" — a
//! sentence that reads as a task waiting to be done.
//!
//! The task is done. The enacted foundation aid appropriation runs continuously from FY2002 to
//! FY2027, and it does not settle the question. These tests hold why, so the open question stays
//! closed against the version of itself that would otherwise return.
//!
//! # The detection floor
//!
//! An appropriation total moves every year for many reasons. A substitution can only be read off
//! the total if it is larger than the total's ordinary movement — and in constant dollars this one
//! moves by a median of roughly $236 million a year. Anything smaller is arithmetically inside the
//! noise.
//!
//! # The comparison that makes it a finding rather than a shrug
//!
//! The lottery's substitution *was* legible, in a movement of $97.6 million — smaller than this
//! series' median annual variation. It was legible because Fund 7017 is itemized inside the
//! appropriation table, so the figure is read rather than inferred. Casino money never enters the
//! table, so inference from the total is the only test available, and the noise floor is more than
//! twice the movement that was readable in the lottery's case.
//!
//! Detectability follows from whether an earmark has a line, not from how large it is.
//!
//! # And the channel itself is now measured, which is what makes the null result conclusive
//!
//! When this file was written the comparison had one side. The size of the casino channel was
//! unknown — the node said establishing it was what would settle whether the total could register
//! the money at all — so "smaller than the noise floor" was an inference from the shape of the
//! problem rather than a measurement.
//!
//! `tax-casino` retrieves it: eighteen half-yearly per-district distributions, nine complete
//! fiscal years, FY2016 through FY2024. The largest is **$114.2 million**. That is under half the
//! noise floor, and it is *larger* than the lottery movement that was legible — so the two
//! channels are now compared at their real sizes rather than at an assumed one, and they still
//! differ only in whether the money has a line.

use edfund_core::FiscalYear;
use project::appropriations::{foundation_movements, foundation_noise_floor};

/// The base year every real figure below is stated in.
const BASE: FiscalYear = FiscalYear(2025);

/// The lottery's substitution, as the appropriation tables print it.
///
/// Read rather than asserted: `project::budget_analysis` computes it from the redbook and the
/// greenbook, and this file's whole argument is that this number was *legible* where the casino
/// channel's is not. Pinning it as a literal would have made the comparison a comparison with a
/// constant.
fn lottery_movement() -> f64 {
    project::budget_analysis::enactment_movement(project::budget_analysis::LOTTERY_LINE)
}

/// Year-over-year movements in the foundation aid appropriation, in constant FY2025 dollars.
///
/// The series, the deflation and the noise floor all live in
/// [`project::appropriations`](project::appropriations::foundation_movements) now. They were three
/// private functions here, and the median one of them computed was not the median the corpus
/// publishes — see [`foundation_noise_floor`] for what that cost.
fn movements() -> Vec<(u16, f64)> {
    foundation_movements(BASE)
}

#[test]
fn the_series_is_long_enough_for_a_noise_floor_to_mean_something() {
    // Two decades of annual movements. A floor computed over four years would be an anecdote.
    let moves = movements();
    assert!(moves.len() > 20, "only {} movements", moves.len());
}

#[test]
fn the_ordinary_annual_movement_is_larger_than_a_channel_this_size_could_be() {
    /*
     * The measurement the node's conclusion rests on. Reported as a median rather than a mean
     * because two years dominate the mean — FY2006's $1.5bn rise and FY2010's $1bn fall — and a
     * noise floor set by the outliers would overstate how hard the question is.
     */
    let mut magnitudes: Vec<f64> = movements().into_iter().map(|(_, m)| m.abs()).collect();
    magnitudes.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    let median = foundation_noise_floor(BASE);

    // To the million, not to a hundred-million band. The band this used to carry —
    // `(200_000_000.0..300_000_000.0)` — held both the $236m the corpus publishes and the $252m
    // this file was computing from a different definition of "median". #158.
    assert!(
        (median / 1e6).round() == 236.0,
        "the median annual movement is ${median:.0}, and the corpus publishes $236 million"
    );
    assert!(
        (magnitudes.iter().sum::<f64>() / magnitudes.len() as f64 / 1e6).round() == 349.0,
        "the mean the node quotes beside the median has moved"
    );
    // And the spread is wide, which is the other half of why inference from the total fails.
    assert!(
        magnitudes[0] < 50_000_000.0,
        "the quietest year moved a lot"
    );
    assert!(
        *magnitudes.last().expect("non-empty") > 1_000_000_000.0,
        "the loudest year moved little"
    );
}

#[test]
fn nothing_distinguishable_happens_when_the_channel_comes_online() {
    /*
     * Ohio's casinos were authorised in 2009 and the county student fund began distributing in the
     * FY2012-FY2013 window. If the General Assembly had cut foundation aid against the new money
     * in an amount the total could show, it would appear here.
     *
     * It does not — and the test asserts the weaker, true thing: those years are unremarkable
     * among their neighbours. That is not evidence of no substitution. It is evidence that this
     * series cannot see one, which is what the node now says.
     */
    let moves = movements();
    let mut magnitudes: Vec<f64> = moves.iter().map(|(_, m)| m.abs()).collect();
    magnitudes.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    let largest = *magnitudes.last().expect("non-empty");

    for year in [2012u16, 2013] {
        let moved = moves
            .iter()
            .find(|(y, _)| *y == year)
            .map(|(_, m)| m.abs())
            .unwrap_or_else(|| panic!("FY{year} is not in the series"));
        assert!(
            moved < largest / 2.0,
            "FY{year} moved ${moved:.0}, which is no longer unremarkable"
        );
    }
}

#[test]
fn the_lottery_movement_would_have_been_invisible_by_this_test() {
    /*
     * The comparison that turns a null result into a finding. The lottery's substitution was
     * readable at $97.6 million because Fund 7017 has a line in the appropriation table. Had the
     * only available test been inference from the total — which is the casino channel's situation
     * — a movement that size would have sat below the median annual variation and shown nothing.
     *
     * So the two channels differ in whether the money is itemized, not in whether it is large.
     */
    let lottery_movement = lottery_movement();
    let median = foundation_noise_floor(BASE);

    assert!(
        lottery_movement < median,
        "the lottery movement of ${lottery_movement:.0} now exceeds the ${median:.0} noise \
         floor, which breaks the argument that itemisation rather than size is what made it \
         legible"
    );
}

#[test]
fn the_whole_channel_is_smaller_than_the_floor_in_every_year_it_has() {
    /*
     * The measurement the node had been promising for four phases. Not "the substitution is
     * smaller than the noise" — the *entire channel* is, in each of the nine years there is a
     * figure for. A substitution can be at most the whole of it, so no arrangement of the money
     * produces a movement this series could distinguish.
     */
    let median = foundation_noise_floor(BASE);

    let years = dispersion::casino::by_fiscal_year();
    assert_eq!(years.len(), 9, "the panel is nine complete fiscal years");
    let largest = years.values().copied().fold(f64::MIN, f64::max);
    assert!(
        largest < median / 2.0,
        "the channel's largest year is ${largest:.0} against a ${median:.0} floor, which is no          longer the comfortable margin the node's conclusion rests on"
    );
}

#[test]
fn the_channel_is_larger_than_the_lottery_movement_that_was_legible() {
    /*
     * The whole finding in one comparison, and it only exists now that both sides are measured.
     *
     * The lottery's substitution was readable at $97,638,202 because Fund 7017 is itemized inside
     * the appropriation table. The casino channel is *bigger* than that in its last three fiscal
     * years and is invisible, because it enters no table. Size is not what separates them.
     */
    let lottery_movement = lottery_movement();

    let years = dispersion::casino::by_fiscal_year();
    let above: Vec<u16> = years
        .iter()
        .filter(|(_, total)| **total > lottery_movement)
        .map(|(year, _)| *year)
        .collect();
    assert_eq!(
        above,
        vec![2022, 2023, 2024],
        "which years exceed the legible lottery movement has changed"
    );
}

#[test]
fn the_per_district_series_does_not_reach_the_years_the_channel_began() {
    /*
     * The limit of what the new source adds, stated so it is not quietly overclaimed.
     *
     * `nothing_distinguishable_happens_when_the_channel_comes_online` looks at FY2012 and FY2013.
     * The per-district series starts in FY2016, because the distributions before August 2015 were
     * published as PDFs. So the onset years are still argued from magnitude — the channel was
     * *smaller* then, and the FY2016 figure bounds it — rather than measured directly.
     */
    let years = dispersion::casino::by_fiscal_year();
    let first = *years.keys().next().expect("nine years");
    assert_eq!(first, 2016);
    assert!(
        first > 2013,
        "the series now reaches the onset; the argument there can be a measurement instead"
    );
    // What bounds the onset years: the earliest half-years are the smallest in the series apart
    // from the closure, and every one of them is far inside the floor.
    let earliest = years[&2016];
    assert!(earliest < 100_000_000.0, "FY2016 came to ${earliest:.0}");
}
