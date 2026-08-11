//! The appropriation-line series, and the check that makes it trustworthy.
//!
//! Sixteen workbooks — two per biennium for eight bienniums — with eight distinct layouts between
//! them, and no two labelling their columns alike. The reason this fixture can be believed is not
//! that the parser is careful; it is that **the documents overlap and agree**. A fiscal year is
//! reported by up to four of them: as an enacted appropriation in the act that made it, as a
//! prior-year actual in the next act's workbook, and again in each revised variant. Where they
//! overlap they must give the same figure to the cent, and the test below is that assertion.
//!
//! The first attempt at this fixture had no such check and was wrong in two ways at once — change
//! columns read as amounts, and a whole biennium's appropriations silently absent. Both would have
//! been caught here.

const SERIES: &str = include_str!("../fixtures/appropriation-lines.csv");

#[derive(Debug, Clone)]
struct Line {
    general_assembly: u16,
    fiscal_year: u16,
    kind: String,
    source: String,
    line_item: String,
    amount: f64,
}

fn series() -> Vec<Line> {
    let mut lines = SERIES.lines();
    let header = lines.next().expect("the fixture has a header");
    assert_eq!(
        header,
        "general_assembly,bill,fiscal_year,kind,source,fund_group,fund,line_item,title,amount",
        "the fixture's columns changed and this reader did not"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let field: Vec<&str> = line.split(',').collect();
            Line {
                general_assembly: field[0].parse().expect("general assembly"),
                fiscal_year: field[2].parse().expect("fiscal year"),
                kind: field[3].to_string(),
                source: field[4].to_string(),
                line_item: field[7].to_string(),
                // The title may itself contain a comma, so the amount is taken from the end
                // rather than by index.
                amount: field[field.len() - 1].parse().expect("amount"),
            }
        })
        .collect()
}

#[test]
fn documents_that_overlap_agree_to_the_cent() {
    // The check the whole fixture rests on. Nothing here trusts the parser; it trusts that four
    // independently laid-out workbooks cannot agree by accident.
    use std::collections::HashMap;
    let mut claims: HashMap<(u16, String, String), Vec<(String, f64)>> = HashMap::new();
    for line in series() {
        claims
            .entry((line.fiscal_year, line.kind.clone(), line.line_item.clone()))
            .or_default()
            .push((line.source.clone(), line.amount));
    }

    let overlapping: Vec<_> = claims.iter().filter(|(_, v)| v.len() > 1).collect();
    assert!(
        overlapping.len() > 500,
        "only {} claims are corroborated by a second document; the series has lost its \
         cross-check",
        overlapping.len()
    );

    for ((year, kind, item), reports) in overlapping {
        let first = reports[0].1;
        for (source, amount) in reports {
            assert!(
                (amount - first).abs() < 0.005,
                "FY{year} {kind} for line item {item} is {amount} in {source} and {first} in \
                 {}; two documents disagree about the same figure",
                reports[0].0
            );
        }
    }
}

#[test]
fn every_kind_is_one_of_the_three_the_extractor_recognises() {
    // A fourth value here means a column label was classified by a rule nobody wrote down.
    for line in series() {
        assert!(
            matches!(line.kind.as_str(), "enacted" | "actual" | "adjusted"),
            "unexpected kind {:?} in {}",
            line.kind,
            line.source
        );
    }
}

#[test]
fn the_enacted_series_is_continuous_from_fy2014_to_fy2027() {
    // What the connector exists for. FY2012-13 is absent on purpose and is asserted absent below:
    // the 129th's two variants are the same file served under two names, so that biennium has
    // actuals and no enacted column anywhere.
    let years: std::collections::BTreeSet<u16> = series()
        .iter()
        .filter(|line| line.kind == "enacted")
        .map(|line| line.fiscal_year)
        .collect();
    for year in 2014..=2027 {
        assert!(
            years.contains(&year),
            "FY{year} has no enacted appropriation"
        );
    }
    assert!(
        !years.contains(&2012) && !years.contains(&2013),
        "FY2012-13 acquired an enacted figure; the 129th's gap has closed and the note in \
         `sources/lsc-budget.md` should say so"
    );
}

#[test]
fn every_line_item_belongs_to_the_department_of_education() {
    // Appropriation line items are numbered by agency and 200 is the department's. The workbooks
    // carry every agency in the state, so a stray prefix here means the filter has slipped and
    // the series is quietly reporting somebody else's budget.
    for line in series() {
        assert!(
            line.line_item.len() == 6
                && line.line_item.starts_with("200")
                && line.line_item.chars().all(|c| c.is_ascii_digit()),
            "line item {:?} is not a department appropriation line",
            line.line_item
        );
    }
}

#[test]
fn foundation_funding_is_the_largest_line_in_every_year_it_appears() {
    // A shape check rather than a value check. `200550 Foundation Funding` is the formula itself
    // and dwarfs everything else in the department's budget; if some other line ever outranks it,
    // either the extraction has mixed up its columns or something extraordinary has happened and
    // the corpus should know which.
    use std::collections::HashMap;
    let mut by_year: HashMap<(u16, u16), Vec<(String, f64)>> = HashMap::new();
    for line in series().iter().filter(|line| line.kind == "enacted") {
        by_year
            .entry((line.general_assembly, line.fiscal_year))
            .or_default()
            .push((line.line_item.clone(), line.amount));
    }
    for ((ga, year), mut items) in by_year {
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("no NaN amounts"));
        assert_eq!(
            items[0].0, "200550",
            "in FY{year} the largest enacted line in the {ga}th General Assembly's act is {} at \
             {}, not Foundation Funding",
            items[0].0, items[0].1
        );
    }
}
