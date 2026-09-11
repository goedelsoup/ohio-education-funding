//! `parameter/base-cost-per-pupil` asked for a constant-dollar companion series, and for two
//! stretches of the nominal series it did not hold. All three were already in this workspace.
//!
//! The checks here are of two kinds. The first kind holds the table to its sources: every amount
//! in [`project::base_cost::SERIES`] carries the sentence the Legislative Service Commission
//! wrote, and the sentence has to be in the committed greenbook and to contain the amount. The
//! second kind is what the series says once it is deflated.

use project::base_cost::{
    self, Basis, Stated, Year, BASE_YEAR, DEDUCTION, EXACT_FY2024_AVERAGE, SERIES,
};

/// `4420.0` as `$4,420`, which is how LSC prints an amount and therefore how a quote carries it.
fn printed(dollars: f64) -> String {
    let whole = dollars.round() as i64;
    let digits = whole.abs().to_string();
    let mut out = String::from("$");
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

fn statements(row: &Year) -> Vec<(&'static str, Stated)> {
    let mut out = vec![("stated", row.stated)];
    if let Some(realized) = row.realized {
        out.push(("realized", realized));
    }
    out
}

#[test]
fn every_quote_is_in_the_greenbook_it_names() {
    let mut missing = Vec::new();
    for row in SERIES.iter().chain(DEDUCTION) {
        for (which, statement) in statements(row) {
            let body = base_cost::analysis(statement.act);
            if !body.contains(statement.quote) {
                missing.push(format!(
                    "FY{} {which}: {} does not say {:?}",
                    row.fiscal_year, statement.act, statement.quote
                ));
            }
        }
        if let Some(freeze) = row.frozen {
            let body = base_cost::analysis(freeze.act);
            if !body.contains(freeze.quote) {
                missing.push(format!(
                    "FY{} freeze: {} does not say {:?}",
                    row.fiscal_year, freeze.act, freeze.quote
                ));
            }
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

#[test]
fn every_stated_amount_appears_in_its_own_quote() {
    // A quote that does not carry its own figure is a citation, not a source: it would go on
    // passing the check above after the number beside it was edited.
    for row in SERIES.iter().chain(DEDUCTION) {
        for (which, statement) in statements(row) {
            let Some(dollars) = statement.dollars else {
                continue;
            };
            assert!(
                statement.quote.contains(&printed(dollars)),
                "FY{} {which}: {:?} is not in {:?}",
                row.fiscal_year,
                printed(dollars),
                statement.quote
            );
        }
    }
}

#[test]
fn the_indexation_reproduces_the_series() {
    // FY1999 is the one year Ohio measured the base cost. Everything through FY2009 is that
    // figure compounded at a rate each act states, and reproducing the chain is what makes the
    // extracted amounts a series rather than eight numbers that happen to rise.
    let at = |fy: u16| base_cost::nominal(fy).expect("in the series");
    let close = |left: f64, right: f64, what: &str| {
        assert!(
            (left - right).abs() < 1.0,
            "{what}: {left:.2} against {right:.2}"
        );
    };
    // Three years of 2.8% from the measured FY1999 base, plus $12 for the twentieth graduation
    // credit, which H.B. 94 names separately.
    close(
        at(1999) * 1.028_f64.powi(3) + 12.0,
        at(2002),
        "FY1999 to FY2002",
    );
    close(at(2002) * 1.028, at(2003), "FY2003 at 2.8%");
    close(at(2003) * 1.022, at(2004), "FY2004 at 2.2%");
    close(at(2004) * 1.022, at(2005), "FY2005 at 2.2%");
    close(at(2006) * 1.0227, at(2007), "FY2007 at 2.27%");
    close(at(2007) * 1.03, at(2008), "FY2008 at 3.0%");
    close(at(2008) * 1.03, at(2009), "FY2009 at 3.0%");
}

#[test]
fn four_years_have_no_statewide_amount() {
    // Not an unrecorded amount — none. The Evidence-Based Model priced teachers rather than
    // pupils and the Bridge formula priced each district's own prior year, so the node's request
    // for "the Evidence-Based Model's computed adequacy amount for FY2010-11" asks for a number
    // that was never computed.
    assert_eq!(base_cost::gap(), vec![2010, 2011, 2012, 2013]);
    for fy in base_cost::gap() {
        assert!(base_cost::nominal(fy).is_none(), "FY{fy}");
    }
}

#[test]
fn the_deduction_ran_through_the_gap_and_fell() {
    // What stopped was the statement of what a pupil costs. What did not stop was the statement
    // of what one is worth taking away.
    let mut previous = 5_732.0_f64;
    for row in DEDUCTION {
        let amount = row.stated.dollars.expect("every deduction year has one");
        assert!(
            amount <= previous,
            "FY{} rose to {amount} from {previous}",
            row.fiscal_year
        );
        previous = amount;
    }
    assert_eq!(
        DEDUCTION
            .iter()
            .map(|row| row.fiscal_year)
            .collect::<Vec<_>>(),
        base_cost::gap(),
        "the deduction covers exactly the years the base cost does not"
    );
    assert!(previous < 5_732.0 * 0.99, "{previous}");

    // The nominal fall is $79 and reads as nothing. Four years of prices is what it actually was.
    let real = base_cost::deduction_real_change();
    assert!((real + 0.0890).abs() < 0.001, "{real}");
    assert!(
        real * 6.0 < previous / 5_732.0 - 1.0,
        "{real} against the nominal fall"
    );
}

#[test]
fn a_frozen_year_is_its_anchors_amount() {
    for row in SERIES {
        let Some(freeze) = row.frozen else { continue };
        assert_eq!(
            base_cost::nominal(row.fiscal_year),
            base_cost::nominal(freeze.anchor),
            "FY{} is frozen at FY{}",
            row.fiscal_year,
            freeze.anchor
        );
    }
    let runs = base_cost::freezes();
    assert_eq!(
        runs.iter().map(|run| run.anchor).collect::<Vec<_>>(),
        vec![2019, 2022, 2024],
    );
    assert_eq!(runs[0].years, vec![2020, 2021]);
    assert_eq!(runs[1].years, vec![2023]);
    assert_eq!(runs[2].years, vec![2025, 2026, 2027]);
}

#[test]
fn the_frozen_years_are_one_number() {
    // The node's series prints FY2025 at $8,240, FY2026 at $8,242 and FY2027 at $8,242, which
    // reads as a rise. There is no rise: H.B. 33 computed one amount for FY2024 and two acts have
    // carried it since, and the printed figures differ by less than the department's own cents.
    let printed = [8_240.0, 8_241.0, 8_242.0];
    for figure in printed {
        assert!(
            (figure - EXACT_FY2024_AVERAGE).abs() < 2.0,
            "{figure} against {EXACT_FY2024_AVERAGE}"
        );
    }
    let first = base_cost::nominal(2024).expect("FY2024 has an amount");
    for fy in [2025, 2026, 2027] {
        assert_eq!(base_cost::nominal(fy), Some(first), "FY{fy}");
    }
}

#[test]
fn the_estimate_at_enactment_is_not_what_gets_frozen() {
    // Under the Fair School Funding Plan the amount is a quotient over data the department has
    // not collected when the act passes, so the biennium's own greenbook can only estimate it.
    // H.B. 110 estimated FY2022 at $7,202 and the year came in at $7,352 — and $7,352 is what
    // FY2023 was then frozen at, so the miss is carried for two years rather than one.
    let fy2022 = base_cost::year(2022).expect("in the series");
    assert!(fy2022.stated.estimate);
    let estimate = fy2022.stated.dollars.expect("an estimate is an amount");
    let realized = base_cost::nominal(2022).expect("FY2022 has an amount");
    assert!(
        (realized / estimate - 1.0 - 0.0208).abs() < 0.001,
        "{realized} on {estimate}"
    );
    assert_eq!(base_cost::nominal(2023), Some(realized));
}

#[test]
fn the_nominal_series_rises_and_the_real_one_falls() {
    // The whole reason the crate exists, on the parameter the corpus calls the most consequential
    // number in Ohio school funding.
    let (nominal, real) = base_cost::growth(2002, 2026).expect("both years carry an amount");
    assert!((nominal - 0.7121).abs() < 0.001, "{nominal}");
    assert!((real + 0.0777).abs() < 0.001, "{real}");
    assert!(nominal > 0.0 && real < 0.0);
}

#[test]
fn the_high_water_mark_is_fy2003() {
    let (fy, real) = base_cost::peak();
    assert_eq!(fy, 2003);
    assert!((real - 8_996.89).abs() < 1.0, "{real}");
    let now = base_cost::real(2026).expect("FY2026 deflates to itself");
    assert!(now < real, "{now} against {real}");
}

#[test]
fn every_era_loses_ground_in_real_terms() {
    // Reported per era because deflation answers the price objection to a cross-era comparison
    // and not the kind objection: a legislated amount and a computed quotient are different
    // quantities however they are denominated.
    let eras = base_cost::eras();
    let expected = [
        ((2002, 2009), 0.1907, -0.0069),
        ((2014, 2021), 0.0479, -0.0808),
        ((2022, 2026), 0.1211, -0.0053),
    ];
    assert_eq!(eras.len(), expected.len());
    for (era, (span, nominal, real)) in eras.iter().zip(expected) {
        assert_eq!(era.span, span);
        assert!((era.change.0 - nominal).abs() < 0.001, "{era:?}");
        assert!((era.change.1 - real).abs() < 0.001, "{era:?}");
        assert!(era.change.0 > 0.0 && era.change.1 < 0.0, "{era:?}");
    }
    assert_eq!(eras[0].basis, Basis::Legislated);
    assert_eq!(eras[2].basis, Basis::Computed);
}

#[test]
fn every_freeze_is_a_real_cut_and_they_are_getting_longer() {
    let runs = base_cost::freezes();
    let mut losses = Vec::new();
    for run in &runs {
        let (restated, paid, loss) = base_cost::erosion(run).expect("every anchor deflates");
        assert!(restated > paid, "FY{} {restated} {paid}", run.anchor);
        assert!(loss < 0.0, "FY{} {loss}", run.anchor);
        losses.push((run.anchor, restated, paid, loss));
    }
    let lengths: Vec<usize> = runs.iter().map(|run| run.years.len()).collect();
    assert_eq!(lengths, vec![2, 1, 3]);

    let (_, restated, paid, loss) = losses[2];
    assert!((loss + 0.0592).abs() < 0.001, "{loss}");
    assert!(
        (restated - paid - 518.83).abs() < 1.0,
        "{restated} against {paid}"
    );
}

#[test]
fn the_last_year_of_the_series_cannot_be_deflated() {
    // FY2027 ends 30 June 2027. There is no June 2027 index and there will not be one until the
    // year is over, so the real series stops a year short of the nominal one and says so.
    assert!(base_cost::nominal(2027).is_some());
    assert!(base_cost::real(2027).is_none());
    assert!(base_cost::real(2026).is_some());

    let rows = base_cost::real_series(BASE_YEAR);
    assert_eq!(rows.len(), SERIES.len());
    let undeflatable: Vec<u16> = rows
        .iter()
        .filter(|(row, nominal, real)| {
            nominal.is_some() && real.is_none() && row.basis != Basis::Absent
        })
        .map(|(row, _, _)| row.fiscal_year)
        .collect();
    assert_eq!(undeflatable, vec![2027]);
}
