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
fn the_enacted_series_covers_the_bienniums_the_documents_reach_and_says_where_it_does_not() {
    // Two gaps, both the source's doing and both asserted so they cannot close unnoticed.
    //
    // FY2006-07: the 126th General Assembly's greenbook has no line-item detail page at all —
    // five section markers in each of the other four documents and none in that one, so its
    // appropriations exist only in narrower per-category tables.
    //
    // FY2012-13: LSC serves that biennium's two workbook variants at different URLs with
    // byte-identical content, so nothing anywhere states what was enacted for it.
    let years: std::collections::BTreeSet<u16> = series()
        .iter()
        .filter(|line| line.kind == "enacted")
        .map(|line| line.fiscal_year)
        .collect();
    for year in (2002..=2005).chain(2008..=2011).chain(2014..=2027) {
        assert!(
            years.contains(&year),
            "FY{year} has no enacted appropriation"
        );
    }
    for (year, why) in [
        (2006, "the 126th's greenbook has no line-item table"),
        (2007, "the 126th's greenbook has no line-item table"),
        (2012, "the 129th's two workbook variants are the same file"),
        (2013, "the 129th's two workbook variants are the same file"),
    ] {
        assert!(
            !years.contains(&year),
            "FY{year} acquired an enacted figure; {why} no longer holds and the connector's \
             blocker should say so"
        );
    }
}

#[test]
fn the_series_reaches_fy1999_as_spending() {
    // The earliest column in the earliest greenbook. The blocker said the pre-2000 record was out
    // of reach; two years of it are in the 124th's own table.
    let years: std::collections::BTreeSet<u16> = series()
        .iter()
        .filter(|line| line.kind == "actual")
        .map(|line| line.fiscal_year)
        .collect();
    assert!(years.contains(&1999) && years.contains(&2000));
    assert_eq!(*years.iter().next().expect("actuals exist"), 1999);
}

#[test]
fn the_two_greenbooks_that_overlap_agree_about_fy2001() {
    // The only genuine cross-document check the PDF era has, and the one that says the clustering
    // reader works: the 124th and the 125th both report FY2001 spending, on separately laid-out
    // pages whose columns sit at different character positions, and the reconciliation would have
    // refused to write this fixture if any of the 102 shared claims had disagreed.
    //
    // It is the only one. FY2010 also shows two documents, and those two are the 129th's
    // `-enacted` and `-actuals` entries, which are byte-identical files served under two names —
    // corroboration by the same bytes twice, which is none.
    let shared = series()
        .iter()
        .filter(|line| line.fiscal_year == 2001 && line.kind == "actual" && line.documents > 1)
        .count();
    assert!(
        shared > 90,
        "only {shared} FY2001 claims are reported by both greenbooks; the overlap that validates \
         the PDF reader has gone"
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
fn the_formula_is_the_largest_line_in_every_year_and_changes_its_number_once() {
    /*
     * A shape check, and a finding the extension back to FY2002 produced.
     *
     * The formula's own appropriation line is the largest in the department's budget in every
     * year of the series — but it is not the same line item throughout. Through FY2005 it is
     * `200501 Base Cost Funding`, the Foundation Base Cost Formula's line; from FY2008 it is
     * `200550 Foundation Funding`, retitled `Foundation Funding - All Students` at FY2022 when
     * the Fair School Funding Plan began.
     *
     * **The changeover is not visible here.** It happened between FY2005 and FY2008, and
     * FY2006-07 is the biennium whose greenbook has no line-item table — so this series can say
     * the number changed and cannot say in which act. That is worth asserting rather than
     * smoothing over: anything walking `200550` backwards will find it starts at FY2008 and
     * should not conclude the formula did.
     */
    use std::collections::HashMap;
    let mut by_year: HashMap<u16, Vec<(String, f64)>> = HashMap::new();
    for line in series().iter().filter(|line| line.kind == "enacted") {
        by_year
            .entry(line.fiscal_year)
            .or_default()
            .push((line.line_item.clone(), line.amount));
    }
    for (year, mut items) in by_year {
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("no NaN amounts"));
        let expected = if year <= 2005 { "200501" } else { "200550" };
        assert_eq!(
            items[0].0, expected,
            "in FY{year} the largest enacted line is {} at {}, not the formula's",
            items[0].0, items[0].1
        );
    }

    let numbers: std::collections::BTreeSet<u16> = series()
        .iter()
        .filter(|line| line.kind == "enacted" && line.line_item == "200550")
        .map(|line| line.fiscal_year)
        .collect();
    assert_eq!(
        *numbers.iter().next().expect("200550 appears"),
        2008,
        "200550 now reaches further back than FY2008; the biennium the renumbering happened in \
         may have become visible"
    );
}
