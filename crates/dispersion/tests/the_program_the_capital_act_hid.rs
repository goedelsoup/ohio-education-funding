//! `program/classroom-facilities-assistance` says its amount is not populated. The panel has it.
//!
//! The node gives one reason for two blank fields — the program's appropriation "is in the capital
//! appropriations act, which no connector retrieves" — and the reason is about the paying side. The
//! survey counts the money as it arrives at the district, and it has done so for every year of the
//! panel under `C35`, the item the survey calls "nonspecified".
//!
//! # What is established here
//!
//! That `C35` is Ohio's construction money, on four independent readings; that $12.7bn of it
//! reached districts between FY2009 and FY2024 against $33.3bn of capital spending; that every one
//! of the 606 districts received some; and that the state's share falls monotonically with
//! district wealth, which is the program's own rule measured rather than described.

use dispersion::facilities;

/// **The identification.** Nothing else in the survey's state block tracks capital outlay.
#[test]
fn the_nonspecified_item_tracks_capital_outlay_in_every_year_and_formula_aid_never_does() {
    let capital = facilities::against_capital();
    assert_eq!(capital.len(), 15, "years the panel carries both columns");
    let (lowest, highest) = (
        capital
            .iter()
            .map(|(_, r)| *r)
            .fold(f64::INFINITY, f64::min),
        capital
            .iter()
            .map(|(_, r)| *r)
            .fold(f64::NEG_INFINITY, f64::max),
    );
    assert!(
        (lowest - 0.4733).abs() < 0.0005 && (highest - 0.7172).abs() < 0.0005,
        "the correlations run {lowest:.4} to {highest:.4}"
    );

    // The control: if this were simply "state money goes where building happens", general formula
    // assistance would show it too. It is negative in two thirds of the years.
    let formula = facilities::formula_against_capital();
    let strongest = formula
        .iter()
        .map(|(_, r)| *r)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (strongest - 0.1122).abs() < 0.0005,
        "formula assistance reaches {strongest:.4} against capital"
    );
    assert_eq!(
        formula.iter().filter(|(_, r)| *r < 0.0).count(),
        10,
        "years in which formula assistance runs against capital outlay"
    );

    // The last two years come from the Bureau's own file rather than from NCES, and they behave
    // like the thirteen that do not — so this is not one publisher's recoding.
    for year in [2023, 2024] {
        let reading = capital
            .iter()
            .find(|(y, _)| *y == year)
            .unwrap_or_else(|| panic!("FY{year}"))
            .1;
        assert!(reading > 0.45, "FY{year} reads {reading:.4}");
    }
}

/// And it lands where building happened, which is a within-district fact.
#[test]
fn the_money_arrives_in_years_districts_were_building() {
    let (stray, total) = facilities::outside_build_years();
    assert!(
        (total / 1e9 - 12.686).abs() < 0.002,
        "the column totals ${:.3}bn",
        total / 1e9
    );
    assert!(
        stray / total < 0.05,
        "{:.1}% of it lands where nothing was built",
        100.0 * stray / total
    );

    // Holding the district fixed: its biggest year of state capital money is a year its own
    // capital spending beat its own median.
    let (matched, examined) = facilities::peaks_are_build_years();
    assert!(
        examined > 400,
        "only {examined} districts had a peak to read"
    );
    #[allow(clippy::cast_precision_loss)]
    let rate = matched as f64 / examined as f64;
    assert!(
        (rate - 0.818).abs() < 0.005,
        "{matched} of {examined} peaks are build years ({rate:.4})"
    );
}

/// **The amount the node records as not populated.**
#[test]
fn twelve_point_seven_billion_reached_ohios_districts_and_every_district_got_some() {
    let districts = facilities::frame();
    assert_eq!(districts.len(), 606);

    let received: f64 = districts.iter().map(|d| d.received).sum();
    let spent: f64 = districts.iter().map(|d| d.spent).sum();
    assert!(
        (received / 1e9 - 12.6512).abs() < 0.002 && (spent / 1e9 - 33.2835).abs() < 0.002,
        "${:.4}bn received against ${:.4}bn spent",
        received / 1e9,
        spent / 1e9
    );
    assert!(
        (received / spent - 0.3801).abs() < 0.0005,
        "the state paid {:.4} of it",
        received / spent
    );

    // The program is described as a queue ordered by poverty, and it is — and it still reached
    // every district in the state.
    let smallest = districts
        .iter()
        .map(|d| d.received)
        .fold(f64::INFINITY, f64::min);
    assert!(
        smallest > 100_000.0,
        "the smallest fifteen-year total is ${smallest:.0}"
    );
}

/// **The rule the node states, measured rather than described.**
#[test]
fn the_states_share_falls_monotonically_with_district_wealth() {
    let quintiles = facilities::by_wealth();
    assert!(
        (quintiles[0] - 0.5756).abs() < 0.0005 && (quintiles[4] - 0.1895).abs() < 0.0005,
        "the gradient runs {quintiles:?}"
    );
    assert!(
        quintiles.windows(2).all(|w| w[0] > w[1]),
        "it is not monotone: {quintiles:?}"
    );
    assert!(
        quintiles[0] > quintiles[4] * 3.0,
        "the poorest fifth's share is {:.4} against the wealthiest's {:.4}",
        quintiles[0],
        quintiles[4]
    );
    assert!(
        (facilities::share_against_wealth() + 0.4405).abs() < 0.0005,
        "share against log wealth is {:+.4}",
        facilities::share_against_wealth()
    );
}

/// The choice of measure is checkable, because the other one flattens the finding.
#[test]
fn averaging_annual_ratios_would_have_lost_a_third_of_the_gradient() {
    let lifetime = facilities::by_wealth();
    let annual = facilities::by_wealth_annual();

    // Both agree about the poorest fifth and diverge everywhere else, because a wealthy district's
    // state money and its own money arrive in different years.
    assert!((lifetime[0] - annual[0]).abs() < 0.005);
    assert!(
        annual[4] > lifetime[4] * 1.5,
        "the wealthiest fifth reads {:.4} annually against {:.4} summed",
        annual[4],
        lifetime[4]
    );

    let spread = |q: [f64; 5]| q[0] - q[4];
    assert!(
        (spread(annual) / spread(lifetime) - 0.664).abs() < 0.005,
        "the annual gradient is {:.3} of the summed one",
        spread(annual) / spread(lifetime)
    );
}

/// And the series says the program was already shrinking when the corpus's longest panel opens.
#[test]
fn the_derolph_construction_response_had_passed_its_peak_before_fy2009() {
    let by_year = facilities::by_year();
    assert_eq!(by_year.len(), 15);

    let at = |year: u16| by_year[&year].0;
    assert!(
        (at(2009) / 1e9 - 1.3566).abs() < 0.0005,
        "FY2009 is ${:.4}bn",
        at(2009) / 1e9
    );
    assert!(
        (at(2013) / 1e9 - 0.4925).abs() < 0.0005,
        "FY2013 is ${:.4}bn",
        at(2013) / 1e9
    );
    assert!(
        at(2013) < at(2009) / 2.5,
        "the fall is {:.3}",
        at(2013) / at(2009)
    );

    // And FY2009 is still the largest year in the panel, in nominal dollars, sixteen years on.
    let largest = by_year
        .iter()
        .max_by(|a, b| a.1 .0.total_cmp(&b.1 .0))
        .expect("years");
    assert_eq!(*largest.0, 2009, "the largest year the panel holds");
}
