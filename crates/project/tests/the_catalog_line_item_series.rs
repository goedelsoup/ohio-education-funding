//! What each appropriation line was given, and the three ways this extract could have lied.
//!
//! [`the-catalog-of-budget-line-items`](../../../.yidam/decisions/the-catalog-of-budget-line-items.yml)
//! scoped this extraction and deliberately did not start it, because its predecessor — the
//! greenbook workbook attempt — produced 3,950 rows before anyone read them and was reverted for
//! two defects that a row count does not show. Both are properties of the source rather than of
//! the parser, and both have analogues here. These tests are where they are held.
//!
//! **The enacted appropriation can be silently superseded.** In the workbooks, the variant
//! carrying actuals restated a closed year and stated the enacted figure nowhere, so a whole
//! biennium produced zero appropriation rows without failing. The Catalog does the same thing and
//! labels it: an edition published at the start of a biennium carries two `Appropriation` columns,
//! and the edition a year later carries one `Adj. Approp.` in their place. Three kinds, never two.
//!
//! **A year does not identify a column.** There it was repeated year headings within one workbook.
//! Here it is the same fiscal year restated by up to four editions, which is why `edition` is a
//! column and not a detail — and why continuity has to be asserted per vintage rather than per
//! year.
//!
//! **And a document can be served twice under different names.** The workbook attempt found LSC
//! serving one biennium's two variants as one file. The Catalog does it too: the 2012 URL resolves,
//! returns a PDF, and is the 2011 edition byte for byte. It is not wired, and the edition count
//! here is what proves it stayed out.

use std::collections::{BTreeMap, BTreeSet};

/// The committed extract: one row per line item per fiscal year per edition.
const ITEMS: &str = include_str!("../fixtures/catalog-line-items.csv");

/// One row per line item per edition, carrying the act that authorises it.
const BASIS: &str = include_str!("../fixtures/catalog-line-item-basis.tsv");

struct Row {
    edition: u16,
    ali: String,
    fiscal_year: u16,
    kind: String,
    amount: Option<f64>,
}

fn rows() -> Vec<Row> {
    let mut lines = ITEMS.lines();
    assert_eq!(
        lines.next().unwrap_or_default().trim(),
        "edition,fund,ali,name,fiscal_year,kind,amount",
        "the catalog fixture header changed"
    );
    lines
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let f: Vec<&str> = l.split(',').map(str::trim).collect();
            Row {
                edition: f[0].parse().expect("edition"),
                ali: f[2].to_string(),
                fiscal_year: f[4].parse().expect("fiscal year"),
                kind: f[5].to_string(),
                amount: f.get(6).and_then(|v| v.parse().ok()),
            }
        })
        .collect()
}

#[test]
fn every_edition_that_exists_is_here_and_the_duplicate_is_not() {
    /*
     * Eighteen, not nineteen. The 2012 URL resolves and serves a PDF, and it is the 2011 edition
     * byte for byte — same SHA-256, same length. The first pass at this connector counted it as an
     * edition because it checked status codes; wiring it would have put every FY2012 and FY2013
     * enacted appropriation in this fixture twice, under two vintages that are one document.
     */
    let editions: BTreeSet<u16> = rows().iter().map(|r| r.edition).collect();
    assert_eq!(editions.len(), 18, "{editions:?}");
    assert!(!editions.contains(&2012), "the duplicate edition came back");
    assert_eq!(*editions.iter().next().unwrap(), 2006);
    assert_eq!(*editions.iter().next_back().unwrap(), 2025);
}

#[test]
fn the_two_bienniums_the_greenbook_cannot_reach_carry_enacted_appropriations() {
    /*
     * The reason this source was worth the phase. `the-greenbook-series` recorded both of these as
     * unretrievable: FY2006-07 because the 126th's greenbook has no line-item table at all, and
     * FY2012-13 because LSC serves that biennium's two workbook variants as one file.
     */
    let rows = rows();
    for year in [2006u16, 2007, 2012, 2013] {
        let enacted: Vec<&Row> = rows
            .iter()
            .filter(|r| r.fiscal_year == year && r.kind == "appropriation" && r.amount.is_some())
            .collect();
        assert!(
            enacted.len() > 100,
            "FY{year} has only {} enacted appropriation lines",
            enacted.len()
        );
    }
}

#[test]
fn an_adjusted_appropriation_is_never_recorded_as_an_enacted_one() {
    /*
     * The load-bearing distinction, and the one the workbook attempt lost. `Adj. Approp.` is what
     * an edition prints for a biennium already under way: the enacted figure revised by whatever
     * has happened since. Reading it as the appropriation would put a mid-course number where the
     * enacted one belongs — a plausible figure, in the right year, from the wrong claim.
     */
    let rows = rows();
    let kinds: BTreeSet<&str> = rows.iter().map(|r| r.kind.as_str()).collect();
    assert_eq!(
        kinds,
        ["actual", "adjusted", "appropriation"]
            .into_iter()
            .collect()
    );

    // Every edition is one shape or the other, and which one is a fact about the edition rather
    // than about any line within it.
    let mut shape: BTreeMap<u16, BTreeSet<&str>> = BTreeMap::new();
    for r in &rows {
        shape.entry(r.edition).or_default().insert(r.kind.as_str());
    }
    for (edition, kinds) in &shape {
        let enacted = kinds.contains("appropriation");
        let adjusted = kinds.contains("adjusted");
        assert!(
            enacted ^ adjusted,
            "edition {edition} mixes enacted and adjusted appropriations: {kinds:?}"
        );
    }
}

#[test]
fn foundation_funding_has_exactly_one_enacted_figure_per_year_from_fy2006() {
    /*
     * The series this whole extraction is for: GRF 200550, the main state aid line, as enacted,
     * with no year missing and no year claimed by two editions.
     *
     * "Exactly one" is the assertion that matters. Eighteen editions each restate six years, so a
     * naive extract would report FY2014 three or four times over with different values, and
     * summing or charting it would be wrong in a way no row count shows. One figure per year means
     * each came from the single edition published when that biennium was enacted.
     */
    let rows = rows();
    let mut by_year: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
    for r in rows
        .iter()
        .filter(|r| r.ali == "200550" && r.kind == "appropriation" && r.amount.is_some())
    {
        by_year.entry(r.fiscal_year).or_default().push(r.edition);
    }

    let years: Vec<u16> = by_year.keys().copied().collect();
    assert_eq!(years, (2006..=2027).collect::<Vec<u16>>(), "{by_year:?}");
    for (year, editions) in &by_year {
        assert_eq!(
            editions.len(),
            1,
            "FY{year} is claimed as enacted by {editions:?}"
        );
    }
}

#[test]
fn the_state_aid_line_grows_and_the_recession_shows() {
    /*
     * A shape check, not a value check — the fixture is regenerated from a source that can be
     * revised, and pinning twenty-two dollar figures would fail on a restatement rather than on an
     * error. What is pinned is the one feature of this series a reader would want to know is real:
     * it is not monotonic. FY2010 and FY2011 fall below FY2009, which is the biennium the federal
     * stimulus was substituting for state GRF.
     */
    let rows = rows();
    let enacted: BTreeMap<u16, f64> = rows
        .iter()
        .filter(|r| r.ali == "200550" && r.kind == "appropriation")
        .filter_map(|r| r.amount.map(|a| (r.fiscal_year, a)))
        .collect();

    assert!(
        enacted[&2027] > enacted[&2006] * 1.5,
        "the line barely grew"
    );
    assert!(
        enacted[&2010] < enacted[&2009],
        "FY2010 no longer falls below FY2009"
    );
    assert!(enacted[&2011] < enacted[&2009]);
}

#[test]
fn every_line_item_names_the_act_that_authorises_it() {
    /*
     * The field the four `fiscal-period` nodes were waiting on. `the-greenbook-series` refused to
     * fill them from LSC's index pages, which name the acts, because that would rest a corpus
     * claim on a page this repository does not hold. This is the same fact in a document that is
     * committed and digest-pinned.
     */
    let mut lines = BASIS.lines();
    assert_eq!(
        lines.next().unwrap_or_default(),
        "edition\tfund\tali\tname\tlegal_basis"
    );
    let mut checked = 0;
    let mut acts: BTreeMap<u16, String> = BTreeMap::new();
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let f: Vec<&str> = line.split('\t').collect();
        assert_eq!(f.len(), 5, "{line}");
        assert!(
            !f[4].trim().is_empty(),
            "{} {} has no legal basis",
            f[0],
            f[2]
        );
        checked += 1;
        if f[2] == "200550" {
            acts.insert(f[0].parse().expect("edition"), f[4].to_string());
        }
    }
    assert!(checked > 2_000, "only {checked} line items carry a basis");

    // The four bienniums whose `appropriating_bill` sat `[unentered]`, each named by the edition
    // published when it was enacted.
    for (edition, act) in [
        (2013u16, "H.B. 59 of the 130th"),
        (2015, "H.B. 64 of the 131st"),
        (2017, "H.B. 49 of the 132nd"),
        (2019, "H.B. 166 of the 133rd"),
    ] {
        assert!(
            acts[&edition].contains(act),
            "edition {edition} no longer names {act}: {}",
            acts[&edition]
        );
    }
}

#[test]
fn the_catalog_closes_the_two_gaps_and_contributes_nothing_else() {
    /*
     * The join rule, asserted from the outside. The workbook series carries an enacted figure for
     * every year from FY2002 to FY2027 except FY2006, FY2007, FY2012 and FY2013 — the two
     * bienniums `the-greenbook-series` recorded as unretrievable. The Catalog supplies exactly
     * those and is not consulted anywhere else.
     *
     * "Exactly those" is the part worth a test. The Catalog covers FY2006-FY2027, so a merge that
     * did not check coverage first would add a second enacted figure for eighteen years that
     * already have one, and every total downstream would roughly double without any row looking
     * wrong.
     */
    let enacted = project::appropriations::enacted_history(edfund_core::FiscalYear(2025));
    let years: Vec<u16> = enacted.iter().map(|y| y.fiscal_year).collect();
    assert_eq!(
        years,
        (2002..=2027).collect::<Vec<u16>>(),
        "the enacted series is no longer continuous"
    );

    // Item counts move with the department's structure, never by a factor of two.
    for pair in enacted.windows(2) {
        let (before, after) = (&pair[0], &pair[1]);
        assert!(
            after.items < before.items * 2,
            "FY{} has {} line items against FY{}'s {} — a year is being counted twice",
            after.fiscal_year,
            after.items,
            before.fiscal_year,
            before.items
        );
    }
}

#[test]
fn the_appropriation_grew_in_nominal_dollars_and_did_not_in_real_ones() {
    /*
     * Why the series was worth building. "Record investment in education" is a true sentence about
     * nearly every biennium here and an empty one: across FY2002-FY2025 the nominal appropriation
     * roughly doubles, and in constant dollars the same series is close to flat.
     *
     * Bounded rather than pinned, because the fixture is regenerated from sources that revise.
     */
    let h = project::appropriations::enacted_history(edfund_core::FiscalYear(2025));
    let first = h.first().expect("a series");
    let last = h
        .iter()
        .rev()
        .find(|y| y.real.is_some())
        .expect("a real year");

    let nominal_growth = last.nominal / first.nominal;
    let real_growth = last.real.expect("real") / first.real.expect("real");
    assert!(nominal_growth > 1.5, "nominal grew {nominal_growth:.2}x");
    assert!(
        real_growth < nominal_growth * 0.75,
        "real growth {real_growth:.2}x is no longer far below nominal {nominal_growth:.2}x"
    );
}
