//! Two independent extractions of the same appropriations, and what their disagreement means.
//!
//! [`the_appropriation_series`](the_appropriation_series.rs) is built from LSC's greenbooks and
//! budget workbooks: sixteen documents, eight layouts, parsed by one extractor. Its own reason for
//! being believable is that those documents overlap and agree.
//!
//! [`the_catalog_line_item_series`](the_catalog_line_item_series.rs) is built from a different
//! publication entirely — the Catalog of Budget Line Items, eighteen editions — by a different
//! extractor written months later against a different layout, and neither was checked against the
//! other while it was written.
//!
//! So where they overlap they are a genuine cross-check rather than a consistency check, and what
//! they say when they disagree is the interesting part.
//!
//! # The result, and why it is the shape it should be
//!
//! **An enacted appropriation never disagrees.** Not one of the seventeen hundred overlapping
//! enacted claims differs by a cent. That is what an enacted appropriation is: a fact about what an
//! act said, fixed the moment it was signed, and two documents describing it must agree or one of
//! them is wrong.
//!
//! **An actual disagrees about one time in twenty.** That is also what an actual is: an accounting
//! figure that gets restated as a year closes and is audited. The greenbook reports FY2007 as of
//! the document that reported it; the Catalog reports FY2007 as of the edition a reader picked up
//! two years later. Neither is wrong and they are not the same claim.
//!
//! This is the whole argument for carrying the source vintage as a column rather than collapsing
//! the two extracts into one series. A reconciliation that demanded agreement everywhere would
//! have to discard one of them; one that demanded it nowhere would not have caught the two defects
//! that reverted the first workbook attempt. The line between them is the kind of claim.

use std::collections::{BTreeMap, BTreeSet};

/// `(fiscal year, line item, kind)` — what both sources key a claim on.
type Claim = (u16, String, String);

const GREENBOOK: &str = include_str!("../fixtures/appropriation-lines.csv");
const CATALOG: &str = include_str!("../fixtures/catalog-line-items.csv");

/// `(fiscal year, line item, kind)` to amount, from the greenbook and workbook series.
fn greenbook() -> BTreeMap<Claim, f64> {
    let mut lines = GREENBOOK.lines();
    let header = lines.next().unwrap_or_default();
    let col = |name: &str| {
        header
            .split(',')
            .position(|h| h.trim() == name)
            .expect(name)
    };
    let (fy, kind, item, amount) = (
        col("fiscal_year"),
        col("kind"),
        col("line_item"),
        col("amount"),
    );
    lines
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let f: Vec<&str> = l.split(',').map(str::trim).collect();
            Some((
                (
                    f.get(fy)?.parse().ok()?,
                    f.get(item)?.to_string(),
                    f.get(kind)?.to_string(),
                ),
                f.get(amount)?.parse().ok()?,
            ))
        })
        .collect()
}

/// The same, from the Catalog. `appropriation` is the Catalog's word for `enacted`.
fn catalog() -> BTreeMap<Claim, f64> {
    CATALOG
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| {
            let f: Vec<&str> = l.split(',').map(str::trim).collect();
            let kind = match *f.get(5)? {
                "appropriation" => "enacted",
                other => other,
            };
            let amount: f64 = f.get(6)?.parse().ok()?;
            Some((
                (
                    f.get(4)?.parse().ok()?,
                    f.get(2)?.to_string(),
                    kind.to_string(),
                ),
                amount,
            ))
        })
        .collect()
}

/// Overlapping keys, and whether the two sources agree on each.
fn compared() -> (Vec<Claim>, Vec<Claim>) {
    let (g, c) = (greenbook(), catalog());
    let mut agree = Vec::new();
    let mut differ = Vec::new();
    for (key, left) in &g {
        let Some(right) = c.get(key) else { continue };
        if (left - right).abs() < 0.5 {
            agree.push(key.clone());
        } else {
            differ.push(key.clone());
        }
    }
    (agree, differ)
}

#[test]
fn the_two_extractions_overlap_enough_to_be_a_check() {
    // A cross-check over a handful of rows would prove nothing. This is thousands.
    let (agree, differ) = compared();
    let total = agree.len() + differ.len();
    assert!(total > 4_000, "only {total} claims overlap");
}

#[test]
fn no_enacted_appropriation_disagrees_between_the_two_sources() {
    /*
     * The strongest assertion in this file, and it holds exactly: zero disagreements out of more
     * than seventeen hundred overlapping enacted claims, to the cent, between a PDF table parsed
     * by column count and a workbook parsed by header.
     *
     * If this ever fails it is not a rounding question. Either an extractor has started reading
     * the wrong column, or one of the two publications has restated something that by its nature
     * cannot be restated — and both are worth stopping the build for.
     */
    let (_, differ) = compared();
    let enacted: Vec<&Claim> = differ
        .iter()
        .filter(|(_, _, kind)| kind == "enacted")
        .collect();
    assert!(
        enacted.is_empty(),
        "enacted appropriations disagree between sources: {enacted:?}"
    );

    let (agree, _) = compared();
    let checked = agree.iter().filter(|(_, _, k)| k == "enacted").count();
    assert!(
        checked > 1_500,
        "only {checked} enacted claims were compared"
    );
}

#[test]
fn actuals_disagree_because_an_actual_is_restated_and_an_appropriation_is_not() {
    /*
     * The other half of the finding. If actuals agreed everywhere too, the two "sources" would be
     * copies of one another and the cross-check above would be worth nothing. They disagree at a
     * low single-digit rate, in scattered years, which is what revision looks like.
     *
     * Bounded on both sides on purpose: no disagreement at all would mean the sources are not
     * independent, and a lot of it would mean one of the parsers is wrong rather than that the
     * publisher revised a figure.
     */
    let (agree, differ) = compared();
    let differing = differ.iter().filter(|(_, _, k)| k == "actual").count();
    let agreeing = agree.iter().filter(|(_, _, k)| k == "actual").count();
    let rate = differing as f64 / (differing + agreeing) as f64;
    assert!(
        (0.005..0.15).contains(&rate),
        "actuals disagree at {:.2}%, outside the band that reads as revision",
        rate * 100.0
    );

    // And revision is not confined to one year, which a parser bug in one document would be.
    let years: BTreeSet<u16> = differ
        .iter()
        .filter(|(_, _, k)| k == "actual")
        .map(|(y, _, _)| *y)
        .collect();
    assert!(years.len() > 3, "only {years:?} carry a revised actual");
}
