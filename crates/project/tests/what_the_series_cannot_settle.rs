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

use std::collections::BTreeMap;

use deflate::CpiSeries;
use edfund_core::FiscalYear;
use project::appropriations::enacted_lines;

/// The lines the formula itself is paid from, across the renumbering at FY2006.
///
/// `200501` is the GRF line before the change and `200550` after; `200612` is the Lottery half
/// throughout. The two GRF lines coexist for six years with `200501` at exactly $0.00, so summing
/// all three double-counts nothing — established in `crates/bundle`, where the same constant lives.
const FOUNDATION_LINES: [&str; 3] = ["200501", "200550", "200612"];

/// The **foundation aid** appropriation in constant dollars, oldest first.
///
/// Not the department's whole appropriation, which is a different series and a different question.
/// The substitution claim is about what the General Assembly set *foundation aid* at, so the noise
/// floor has to be the noise in foundation aid. The two differ by about a factor of two — the
/// department as a whole moves a median $449m a year against foundation aid's $236m — and using
/// the wrong one would overstate how hard the question is by exactly that factor.
fn real_series() -> Vec<(u16, f64)> {
    // `enacted_lines`, not `lines`: the workbook fixture alone is missing FY2006-07 and FY2012-13,
    // and a noise floor computed over a series with holes in it is measuring the holes.
    let cpi = CpiSeries::cpi_u_june();
    let mut nominal: BTreeMap<u16, f64> = BTreeMap::new();
    for line in enacted_lines() {
        if FOUNDATION_LINES.contains(&line.line_item.as_str()) {
            *nominal.entry(line.fiscal_year).or_default() += line.amount;
        }
    }
    nominal
        .into_iter()
        .filter_map(|(year, amount)| {
            cpi.convert(amount, FiscalYear(year), FiscalYear(2025))
                .ok()
                .map(|deflated| (year, deflated.value))
        })
        .collect()
}

/// Year-over-year movements in constant dollars.
fn movements() -> Vec<(u16, f64)> {
    let series = real_series();
    series
        .windows(2)
        .map(|pair| (pair[1].0, pair[1].1 - pair[0].1))
        .collect()
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
    let median = magnitudes[magnitudes.len() / 2];

    assert!(
        (200_000_000.0..300_000_000.0).contains(&median),
        "the median annual movement is ${median:.0}, outside the band this node quotes"
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
    const LOTTERY_MOVEMENT: f64 = 97_638_202.0;

    let mut magnitudes: Vec<f64> = movements().into_iter().map(|(_, m)| m.abs()).collect();
    magnitudes.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
    let median = magnitudes[magnitudes.len() / 2];

    assert!(
        LOTTERY_MOVEMENT < median,
        "the lottery movement of ${LOTTERY_MOVEMENT:.0} now exceeds the ${median:.0} noise floor, \
         which breaks the argument that itemisation rather than size is what made it legible"
    );
}
