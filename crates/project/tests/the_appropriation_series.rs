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
    documents: usize,
    line_item: String,
    amount: f64,
}

fn series() -> Vec<Line> {
    let mut lines = SERIES.lines();
    let header = lines.next().expect("the fixture has a header");
    assert_eq!(
        header,
        "general_assembly,bill,fiscal_year,kind,source,documents,fund_group,fund,line_item,\
         title,amount",
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
                documents: field[5].parse().expect("documents"),
                line_item: field[8].to_string(),
                // The title may itself contain a comma, so the amount is taken from the end
                // rather than by index.
                amount: field[field.len() - 1].parse().expect("amount"),
            }
        })
        .collect()
}

#[test]
fn most_of_the_series_is_corroborated_by_a_second_document() {
    // The property the fixture rests on, now asserted at build time rather than here: agreement
    // between documents is checked in `fixtures::reconcile`, which refuses to write a fixture
    // where two of them disagree. What survives into the fixture is the count, and this is the
    // guard that the overlap has not quietly vanished — a series where every figure came from
    // exactly one document would still parse, still sum, and have lost its whole cross-check.
    let lines = series();
    let corroborated = lines.iter().filter(|line| line.documents > 1).count();
    assert!(
        corroborated > 900,
        "only {corroborated} of {} claims are corroborated by a second document",
        lines.len()
    );
    assert!(
        lines.iter().all(|line| line.documents >= 1),
        "a claim reports being in no document at all"
    );
}

#[test]
fn one_row_per_claim_so_the_series_can_be_summed() {
    // The defect this deduplication exists for. Two documents legitimately report the same
    // enacted figure, so the raw extract counted every appropriation twice: `200550 Foundation
    // Funding` came out at $17.5 billion against a true $8.7 billion, on a departmental budget of
    // $15.3 billion. Exactly double is the error most likely to pass a sanity check.
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    for line in series() {
        assert!(
            seen.insert((line.fiscal_year, line.kind.clone(), line.line_item.clone())),
            "FY{} {} for line item {} appears more than once; summing this series would \
             double-count it",
            line.fiscal_year,
            line.kind,
            line.line_item
        );
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

#[test]
fn the_series_reconciles_with_the_greenbook_it_came_from_to_within_a_quarter_percent() {
    /*
     * The check against something outside the extraction. LSC's FY2026-27 greenbook prints a
     * fund-group table in its own "Quick look" section: $14,881,272,733 for FY2026 and
     * $15,300,066,884 for FY2027, as the department's total budget. Summing the enacted lines
     * here should land on that, and it lands near it.
     *
     * **The property tax reimbursement lines have to come out first**, and that is the
     * greenbook's own doing rather than an adjustment invented to make the numbers meet: it
     * states that those items "are included in the State Revenue Distributions (RDF) section of
     * the budget", so they are numbered `200xxx` and are not part of the department's total.
     * `200903` alone is $1.3 billion, so the difference is not subtle.
     *
     * **What is left over is stated rather than tuned away.** After removing them the sum is
     * short by $12.15 million in FY2026 and $31.15 million in FY2027 — 0.08% and 0.20%, and
     * unexplained. It would be easy to find some combination of line items that closes the
     * gap exactly; that would be fitting the answer, and the residual would stop being visible
     * the moment it changed. The tolerance is deliberately loose enough to hold the current
     * residual and tight enough that a column mix-up could not hide inside it.
     */
    const RDF: [&str; 2] = ["200903", "200417"];
    for (year, published) in [(2026u16, 14_881_272_733.0f64), (2027, 15_300_066_884.0)] {
        let total: f64 = series()
            .iter()
            .filter(|line| {
                line.kind == "enacted"
                    && line.fiscal_year == year
                    && !RDF.contains(&line.line_item.as_str())
            })
            .map(|line| line.amount)
            .sum();
        let residual = published - total;
        assert!(
            residual.abs() / published < 0.0025,
            "FY{year} sums to {total} against the greenbook's {published}, a residual of \
             {residual} — more than a quarter of a percent, so something has moved"
        );
        assert!(
            residual > 0.0,
            "FY{year} now sums above the greenbook's own total, which the residual has never \
             done; a line item is being counted that the department's total excludes"
        );
    }
}
