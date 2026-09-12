//! Nine and a quarter billion dollars that passed through district foundation payments.
//!
//! Under every Ohio funding regime before the Fair School Funding Plan, a community school was
//! paid by deducting its pupils' funding from the district they lived in. The district was
//! credited with the money and never held it.
//!
//! `education-agency/electronic-classroom-of-tomorrow` records the distributional question this
//! raises and says it "needs the department's deduct file and is not established here". The
//! survey has carried most of it all along, in the column
//! [`dispersion::survey_basis`] introduced to explain why Ohio's state
//! revenue breaks at FY2016 — and then never asked who was paying. These tests read it as a
//! distribution.
//!
//! # What is established and what is not
//!
//! **Established:** how much, by which districts, in what proportion, for eleven years.
//!
//! **Not established:** which *receiving school* any district's payment went to, because `V92` is
//! one number per district per year. ECOT's own apportionment therefore stays open, and
//! [`dispersion::deduct::share_of_pool`] bounds it with the statewide average instead of
//! substituting for it — see `ecot_was_about_a_ninth_of_every_deduct_on_average`.
//!
//! **Also not established: that the deduct stopped.** FY2022 reads as a collapse and the module
//! docs say why that cannot be relied on.

use dispersion::deduct::{self, Bearer};
use dispersion::survey_basis::DEDUCT_REPORTED;

/// Columbus, Cleveland, Toledo, Cincinnati, Dayton — the five largest bearers, in order.
const BIG_FIVE: [(&str, &str); 5] = [
    ("043802", "Columbus"),
    ("043786", "Cleveland"),
    ("044909", "Toledo"),
    ("043752", "Cincinnati"),
    ("043844", "Dayton"),
];

fn total(bearers: &[Bearer]) -> f64 {
    bearers.iter().map(|bearer| bearer.paid).sum()
}

/// **The licence for everything else here.** Two sets of respondents, one transaction.
///
/// Districts report what they paid community schools; community schools report what they received
/// from the state. Under the deduct those are the same dollars, reported independently by parties
/// with no reason to agree, and across the middle of the window they land within four per cent.
///
/// The ends are the caution rather than the finding. FY2010 is 11.5% light — the column is still
/// filling in after the FY2009 zeros `survey_basis` excludes outright — and FY2021 is 7.7% heavy.
/// Neither is large enough to move the shares below, all of which are ratios within a year.
#[test]
fn the_column_agrees_with_the_counterparty_that_reports_the_other_side() {
    for year in 2012..=2019 {
        let Some(ratio) = deduct::counterparty(year) else {
            continue; // FY2014 is absent from the archive for every state.
        };
        assert!(
            (0.98..=1.04).contains(&ratio),
            "FY{year} counterparty is {ratio}, and the two sides should describe one transaction"
        );
    }

    // And the two years that do not, which is why the window's first year is a floor.
    let first = deduct::counterparty(DEDUCT_REPORTED[0]).expect("FY2010 is reported");
    let last = deduct::counterparty(DEDUCT_REPORTED[1]).expect("FY2021 is reported");
    assert!(
        (first - 0.885).abs() < 0.001,
        "FY2010 counterparty is {first}"
    );
    assert!(
        (last - 1.077).abs() < 0.001,
        "FY2021 counterparty is {last}"
    );
    assert!(
        first < 0.95,
        "the window opens light against its counterparty"
    );
}

/// **The finding.** Nearly every district paid, and almost nobody paid much.
///
/// The corpus says the deduct was "spread across most districts in the state". By count that is
/// exactly right — 604 of 615 — and it is the half that misleads: ten districts carried 62.8% of
/// it and fifty carried 79.1%. A statement true of the count and false of the money.
#[test]
fn six_hundred_districts_paid_and_ten_of_them_paid_most_of_it() {
    let cumulative = deduct::cumulative();
    assert_eq!(
        cumulative.len(),
        615,
        "comparable districts across the window"
    );

    let paying = cumulative.iter().filter(|b| b.paid > 0.0).count();
    assert_eq!(paying, 604, "districts that paid something at some point");

    let paid = total(&cumulative);
    assert!(
        (paid - 9_285_168_000.0).abs() < 1000.0,
        "the window totals {paid}"
    );

    let top_ten = deduct::concentration(&cumulative, 10).expect("something was paid");
    let top_fifty = deduct::concentration(&cumulative, 50).expect("something was paid");
    assert!((top_ten - 0.6280).abs() < 0.0005, "top ten bear {top_ten}");
    assert!(
        (top_fifty - 0.7911).abs() < 0.0005,
        "top fifty bear {top_fifty}"
    );
}

/// And the five that carried it are Ohio's big-city districts, each by a quarter of its own money.
///
/// The share is the point rather than the dollars. Columbus was credited with state aid and passed
/// **more than a third of it straight out**; a reader looking at Columbus's foundation payment is
/// looking at a number two thirds of which stayed.
#[test]
fn columbus_never_kept_a_third_of_the_state_money_it_was_credited_with() {
    let cumulative = deduct::cumulative();
    let order: Vec<&str> = cumulative.iter().take(5).map(|b| b.irn.as_str()).collect();
    let expected: Vec<&str> = BIG_FIVE.iter().map(|(irn, _)| *irn).collect();
    assert_eq!(order, expected, "the five largest bearers, largest first");

    let find = |irn: &str| {
        cumulative
            .iter()
            .find(|b| b.irn == irn)
            .expect("in the panel")
            .clone()
    };

    let columbus = find("043802");
    assert!(
        (columbus.paid - 1_458_937_000.0).abs() < 1000.0,
        "Columbus paid {}",
        columbus.paid
    );
    let share = columbus.share().expect("credited with something");
    assert!(
        (share - 0.3529).abs() < 0.0005,
        "Columbus's share is {share}"
    );

    // None of the other four is far behind, and every one of them is above a fifth.
    for (irn, name) in BIG_FIVE.iter().skip(1) {
        let share = find(irn).share().expect("credited with something");
        assert!(
            (0.20..0.30).contains(&share),
            "{name} passed through {share}"
        );
    }
}

/// The deduct grew against the money it came out of, and most of the growth is real.
///
/// From 6.99% of credited state aid in FY2010 to 9.06% in FY2021, peaking at 9.78% in FY2015.
/// Part of the rise at the start is the column filling in rather than the sector growing — see
/// the counterparty test — so the first year is read as a floor and the *peak* is the claim.
#[test]
fn the_deduct_ran_between_a_fourteenth_and_a_tenth_of_credited_state_aid() {
    let years = deduct::by_year();
    assert_eq!(
        years.len(),
        11,
        "twelve years less the archive's missing FY2014"
    );
    assert!(
        !years.contains_key(&2014),
        "FY2014 is absent for every state"
    );

    let share = |year: u16| {
        years
            .get(&year)
            .expect("a reported year")
            .share_of_gross()
            .expect("districts are credited with something")
    };
    assert!(
        (share(2010) - 0.0699).abs() < 0.0005,
        "FY2010 {}",
        share(2010)
    );
    assert!(
        (share(2015) - 0.0978).abs() < 0.0005,
        "FY2015 {}",
        share(2015)
    );
    assert!(
        (share(2021) - 0.0906).abs() < 0.0005,
        "FY2021 {}",
        share(2021)
    );

    let peak = years
        .values()
        .filter_map(|year| year.share_of_gross())
        .fold(f64::NEG_INFINITY, f64::max);
    assert!((peak - 0.0978).abs() < 0.0005, "the peak share is {peak}");

    // Every year is inside the band, which is what makes the range a description of the period
    // rather than of its endpoints.
    for year in years.values() {
        let share = year.share_of_gross().expect("credited");
        assert!(
            (0.065..0.10).contains(&share),
            "FY{} is {share}, outside the band",
            year.fiscal_year
        );
    }
}

/// Concentration is flat across eleven years, which rules out one explanation of it.
///
/// If the top ten's share were rising, the story would be a sector consolidating into the cities.
/// It does not move: the band is three and a half points wide across a decade in which the money
/// grew by 42%. The distribution was set early and the sector grew inside its shape.
#[test]
fn the_concentration_does_not_move_across_the_window() {
    let mut shares: Vec<(u16, f64)> = Vec::new();
    for year in deduct::by_year().keys().copied() {
        let bearers = deduct::bearers(year);
        shares.push((
            year,
            deduct::concentration(&bearers, 10).expect("something was paid"),
        ));
    }
    assert_eq!(shares.len(), 11);

    let lowest = shares.iter().map(|(_, s)| *s).fold(f64::INFINITY, f64::min);
    let highest = shares
        .iter()
        .map(|(_, s)| *s)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (0.61..0.62).contains(&lowest),
        "lowest top-ten share {lowest}"
    );
    assert!(
        (0.64..0.66).contains(&highest),
        "highest top-ten share {highest}"
    );
    assert!(
        highest - lowest < 0.04,
        "the top ten's share moves {} across the window",
        highest - lowest
    );
}

/// ECOT was about a ninth of every district's deduct, on the statewide average.
///
/// **This is a bound and not an apportionment**, and the distinction is the whole reason the
/// node's question stays open. It says what fraction of the statewide pool ECOT drew; it does not
/// say that any particular district paid ECOT that fraction, and a district whose residents
/// enrolled with ECOT more than average paid more. Naming districts needs the department's file.
///
/// What it does establish is the scale. ECOT drew **10.6% of every dollar Ohio's districts paid
/// community schools** across the eight years it overlaps the reported window, rising from 8.4%
/// to 11.9%. One school, against roughly three hundred.
#[test]
fn ecot_was_about_a_ninth_of_every_deduct_on_average() {
    const ECOT: &str = "133413";

    let first = deduct::share_of_pool(ECOT, 2010).expect("ECOT is in FY2010");
    let peak = deduct::share_of_pool(ECOT, 2016).expect("ECOT is in FY2016");
    assert!((first - 0.0844).abs() < 0.0005, "FY2010 share {first}");
    assert!((peak - 0.1189).abs() < 0.0005, "FY2016 share {peak}");

    // Across every year the two overlap, weighted by the pool rather than averaged over years.
    let mut drawn = 0.0;
    let mut pool = 0.0;
    for year in deduct::by_year().keys().copied() {
        let Some(share) = deduct::share_of_pool(ECOT, year) else {
            continue; // ECOT closed in FY2018 and the pool runs on without it.
        };
        let paid = total(&deduct::bearers(year));
        drawn += share * paid;
        pool += paid;
    }
    let overall = drawn / pool;
    assert!(
        (overall - 0.1059).abs() < 0.0005,
        "ECOT drew {overall} of the pool"
    );

    // And it is absent after it closed, rather than carried at zero.
    assert_eq!(deduct::share_of_pool(ECOT, 2019), None);
}

/// The window is closed at both ends, and closed rather than empty.
///
/// `survey_basis` establishes that the column cannot be read outside FY2010-FY2021 — a zero and an
/// unreported value are the same character. This module returns nothing there rather than a run of
/// zeros, because a run of zeros reads as a finding and this one would be a false one.
#[test]
fn nothing_is_returned_outside_the_years_the_column_can_be_read_in() {
    for year in [2009u16, 2022, 2023, 2024] {
        assert!(
            deduct::bearers(year).is_empty(),
            "FY{year} is outside the reported window and must not answer"
        );
        assert_eq!(deduct::counterparty(year), None, "FY{year}");
    }
    assert_eq!(DEDUCT_REPORTED, [2010, 2021]);

    // Inside it, every year answers.
    for year in DEDUCT_REPORTED[0]..=DEDUCT_REPORTED[1] {
        if year == 2014 {
            continue;
        }
        assert!(!deduct::bearers(year).is_empty(), "FY{year} should answer");
    }
}
