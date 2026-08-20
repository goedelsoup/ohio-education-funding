//! What the committed casino panel has to keep being true about.
//!
//! `dispersion::casino`'s own tests hold the shape of the series. These hold the two things that
//! can only be checked against another fixture — that the channel reaches every district the
//! formula funds, and that it reaches four hundred more — and the arithmetic the corpus node
//! quotes, so that a refetch which changes a figure changes the node with it.

use std::collections::BTreeSet;

use dispersion::casino;

/// The FY2027 department model, which is the formula's own list of districts.
const FY27: &str = include_str!("../../foundation/fixtures/fy27-department-model.csv");

fn formula_districts() -> BTreeSet<String> {
    FY27.lines()
        .skip(1)
        .filter_map(|line| line.split(',').next())
        .filter(|irn| irn.len() == 6 && irn.bytes().all(|b| b.is_ascii_digit()))
        .map(str::to_string)
        .collect()
}

#[test]
fn every_district_the_formula_funds_is_paid_from_this_channel_too() {
    // The claim that makes the channel worth measuring at all: it is not a programme some
    // districts qualify for. Every one of the 609 districts in the department's own model appears
    // in the last distribution, and nothing has to be true about them to get there.
    let paid: BTreeSet<String> = casino::panel()
        .into_iter()
        .filter(|row| row.month == "2024-01")
        .map(|row| row.irn)
        .collect();
    let formula = formula_districts();
    assert_eq!(formula.len(), 609);
    let missing: Vec<&String> = formula.difference(&paid).collect();
    assert!(missing.is_empty(), "not paid: {missing:?}");
}

#[test]
fn and_four_hundred_more_that_the_formula_does_not_fund_this_way() {
    /*
     * R.C. 5753.11(A)(1) defines "public school district" for this fund to include community
     * schools, STEM schools, JVSDs and college-preparatory boarding schools. So the denominator is
     * not the formula's, and a per-pupil figure built by dividing this money by a traditional
     * district count would be wrong by the whole of the difference.
     *
     * This is also the boundary drawn opposite to IDEA Part B, which excludes JVSDs entirely.
     */
    let paid: BTreeSet<String> = casino::panel()
        .into_iter()
        .filter(|row| row.month == "2024-01")
        .map(|row| row.irn)
        .collect();
    assert_eq!(paid.len(), 1001);
    assert_eq!(paid.difference(&formula_districts()).count(), 392);
}

#[test]
fn the_fiscal_year_totals_are_the_ones_the_corpus_node_quotes() {
    // Nine figures, to the dollar. A refetch that moves one moves the node, which is the point of
    // pinning them somewhere a test reads rather than only in prose.
    let years = casino::by_fiscal_year();
    let expected: [(u16, f64); 9] = [
        (2016, 90_832_042.96),
        (2017, 89_356_177.56),
        (2018, 92_029_468.31),
        (2019, 93_928_001.63),
        (2020, 95_985_938.04),
        (2021, 73_873_804.95),
        (2022, 109_385_274.99),
        (2023, 113_107_107.80),
        (2024, 114_177_214.22),
    ];
    for (year, total) in expected {
        let got = years[&year];
        assert!(
            (got - total).abs() < 0.02,
            "FY{year} is ${got:.2}, the node says ${total:.2}"
        );
    }
}

#[test]
fn the_channel_spans_four_orders_of_magnitude_across_districts() {
    /*
     * Recorded because it is the first thing anyone will want from a per-district file and the
     * first thing that will be misread. The spread is a spread in *students*, not in need: the
     * fund is apportioned on head count with no weighting of any kind, which is what distinguishes
     * it from every other channel in this corpus.
     */
    let mut last: Vec<(f64, String, String)> = casino::panel()
        .into_iter()
        .filter(|row| row.month == "2024-01")
        .map(|row| (row.amount, row.irn, row.district))
        .collect();
    last.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite"));

    let (smallest, small_irn, _) = &last[0];
    let (largest, large_irn, _) = last.last().expect("non-empty");
    assert_eq!(small_irn, "046797", "Kelleys Island, 25 students or so");
    assert_eq!(large_irn, "043802", "Columbus");
    assert!(*smallest < 300.0, "${smallest:.2} for the smallest");
    assert!(*largest > 1_400_000.0, "${largest:.2} for the largest");
}

#[test]
fn the_three_statewide_e_schools_are_paid_from_all_eighty_eight_counties() {
    // The apportionment made visible. An e-school's students are resident everywhere, so it draws
    // from every county fund — which is why the published sheets are keyed on (county, IRN) and
    // why `counties` is carried rather than a district's home county.
    let statewide: Vec<String> = casino::panel()
        .into_iter()
        .filter(|row| row.month == "2024-01" && row.counties == Some(88))
        .map(|row| row.irn)
        .collect();
    assert_eq!(statewide, ["000236", "142950", "143396"]);
}
