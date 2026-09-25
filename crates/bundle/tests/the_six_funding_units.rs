//! The block that puts the other five funding units beside the district one.
//!
//! Against the real build rather than a fixture, because the whole point of the block is that its
//! figures come from three sources this workspace already holds and the district unit's figure is
//! the same one every other statewide card shows. A fixture would prove the emitter works and
//! nothing about the join.

use bundle::*;
use std::collections::BTreeSet;

/// A cent. The scholarship report publishes to the cent and the units have to add up to it.
const CENT: f64 = 0.005;

fn units() -> FundingUnits {
    bundle::build::build()
        .funding_units
        .expect("the real build sizes all six")
}

#[test]
fn there_are_six_and_the_statute_orders_them() {
    let f = units();
    assert_eq!(f.authority, "R.C. 3317.022");
    let slugs: Vec<&str> = f.units.iter().map(|u| u.slug.as_str()).collect();
    assert_eq!(
        slugs,
        [
            "district",
            "community-stem",
            "educational-choice",
            "pilot-project",
            "autism",
            "jon-peterson",
        ],
        "the order is the statute's opening sentence, and the district unit is first because \
         the sentence puts it first"
    );
    for unit in &f.units {
        assert!(
            unit.authority.starts_with("R.C. 3317.02"),
            "`{}` should name the section that computes it, and names `{}`",
            unit.slug,
            unit.authority
        );
        assert!(unit.amount > 0.0, "`{}` is sized at nothing", unit.slug);
    }
}

#[test]
fn the_district_unit_is_the_figure_the_rest_of_the_feed_already_publishes() {
    /*
     * Not approximately, and not from a second computation of it.
     *
     * The table this block feeds sits on the statewide page, a scroll away from cards computed
     * off `statewide.realized_aid_total`. Two figures for the district unit differing in the
     * fourth digit would be read as two claims about the same year rather than as a rounding.
     */
    let built = bundle::build::build();
    let f = built.funding_units.as_ref().expect("the units");
    let district = f
        .units
        .iter()
        .find(|u| u.slug == "district")
        .expect("the district unit");
    assert_eq!(district.amount, built.statewide.realized_aid_total);
    assert_eq!(district.recipients, Some(built.districts.len()));
    assert_eq!(district.recipients_noun, "districts");
    assert_eq!(district.derived, 0.0, "nothing here is derived");
}

#[test]
fn the_four_scholarship_units_sum_to_the_channel_the_report_publishes() {
    /*
     * $1,095,135,538.27 over 166,587 students, of which $103,944,388.15 is derived.
     *
     * The six units are the statute's partition and the five programmes are the report's, and
     * the two meet on the educational choice unit, which pays both EdChoice programmes. If a
     * programme fell out of a unit here the channel would come out smaller and nothing else in
     * this feed would notice.
     */
    let f = units();
    let reported: Vec<&FundingUnit> = f.units.iter().filter(|u| u.basis == "report").collect();
    assert_eq!(reported.len(), 4, "four of the six come from the report");

    let total: f64 = reported.iter().map(|u| u.amount).sum();
    assert!(
        (total - 1_095_135_538.27).abs() < CENT,
        "the channel is ${total:.2}"
    );
    let students: f64 = reported.iter().filter_map(|u| u.students).sum();
    assert!((students - 166_587.0).abs() < f64::EPSILON);

    let derived: f64 = reported.iter().map(|u| u.derived).sum();
    assert!(
        (derived - 103_944_388.15).abs() < CENT,
        "exactly Jon Peterson's is derived, and it is ${derived:.2}"
    );
    let published = total - derived;
    assert!(
        published < 1e9 && total > 1e9,
        "the published part is under a billion and the whole is over it, which is why the two \
         are carried separately"
    );

    let programmes: Vec<&FundingUnitProgramme> =
        reported.iter().flat_map(|u| u.programmes.iter()).collect();
    assert_eq!(programmes.len(), 5, "five programmes across four units");
    assert_eq!(
        programmes.iter().filter(|p| p.derived).count(),
        1,
        "one programme's expenditure is this project's sum rather than the report's"
    );
    let slugs: BTreeSet<&str> = programmes.iter().map(|p| p.slug.as_str()).collect();
    assert_eq!(slugs.len(), 5, "a programme is in one unit");
    for p in &programmes {
        assert!(
            !p.node.is_empty(),
            "`{}` has no write-up to link to",
            p.slug
        );
    }
}

#[test]
fn no_scholarship_unit_claims_a_district() {
    /*
     * The report says which districts Jon Peterson's students came from — 494 of them — and says
     * nothing about which district any scholarship was charged against, because under
     * R.C. 3317.022 none of them is. The per-district file it links returns an Angular shell.
     *
     * So the recipients column is null for these four, and it is null rather than zero: the
     * question has no answer, which is not the same as the answer being none.
     */
    for unit in units().units.iter().filter(|u| u.basis == "report") {
        assert_eq!(unit.recipients, None, "`{}` names recipients", unit.slug);
        assert!(unit.recipients_noun.is_empty());
    }
}

#[test]
fn the_units_are_on_two_bases_and_every_one_of_them_says_which() {
    /*
     * Two of the six are a model of FY2027 and four are an account of 2024-25. A table that
     * summed them would be adding a projection to an actual, and a table that labelled them once
     * at the top would be telling the reader they are the same kind of claim.
     *
     * The guard is that every unit names a `series` key that resolves in `series_years`, and
     * that the keys it resolves to are not all on one reckoning.
     */
    let built = bundle::build::build();
    let f = built.funding_units.as_ref().expect("the units");
    let mut kinds = BTreeSet::new();
    for unit in &f.units {
        assert!(
            unit.basis == "model" || unit.basis == "report",
            "`{}` has basis `{}`",
            unit.slug,
            unit.basis
        );
        let year = built
            .series_years
            .iter()
            .find(|y| y.series == unit.series)
            .unwrap_or_else(|| {
                panic!(
                    "`{}` names series `{}`, which is not in the index",
                    unit.slug, unit.series
                )
            });
        assert!(!year.label.is_empty());
        kinds.insert(year.kind.as_str());
    }
    assert_eq!(
        kinds.len(),
        2,
        "the six units are on one reckoning, which would mean the block stopped carrying the \
         distinction it exists for"
    );
    let index = built
        .series_years
        .iter()
        .find(|y| y.series == f.nonpublic_support.series)
        .expect("the nonpublic support year");
    assert_eq!(
        index.label,
        format!("FY{}", f.nonpublic_support.fiscal_year),
        "the label is derived from the year in the block, not written beside it"
    );
}

#[test]
fn category_three_travels_with_the_units_and_is_not_one_of_them() {
    /*
     * Auxiliary Services, the Nonpublic Administrative Cost Reimbursement, and the reimbursement
     * fund that is a reflux of the first. They move under R.C. 3317.024, 3317.06, 3317.062,
     * 3317.063 and 3317.064 — none of which is R.C. 3317.022 — so they are beside the six rather
     * than among them, and a quarter of a billion dollars is too much to leave off a page about
     * what the state pays nonpublic schools.
     */
    let f = units();
    let support = &f.nonpublic_support;
    assert_eq!(support.lines.len(), 3);
    let lines: Vec<&str> = support.lines.iter().map(|l| l.ali.as_str()).collect();
    assert_eq!(lines, ["200511", "200532", "200659"]);

    let summed: f64 = support.lines.iter().map(|l| l.amount).sum();
    assert!(
        (summed - support.total).abs() < CENT,
        "the lines sum to ${summed:.2} and the total says ${:.2}",
        support.total
    );
    for line in &support.lines {
        assert!(!line.node.is_empty(), "`{}` has no write-up", line.ali);
        assert!(
            !line.authority.contains("3317.022"),
            "`{}` is inside the six funding units, which would make this block wrong",
            line.ali
        );
    }
    // `200659` is spending down a balance rather than receiving new money, so it is a property
    // of the auxiliary services programme rather than a programme of its own.
    assert_eq!(support.lines[0].node, support.lines[2].node);

    for unit in &f.units {
        assert!(
            !support.lines.iter().any(|l| l.name == unit.name),
            "a Category 3 line is being published as a funding unit"
        );
    }
}

#[test]
fn the_block_survives_into_the_document() {
    let feed = bundle::build::build().to_json();
    assert!(feed.contains("\"funding_units\": {"));
    assert!(
        feed.contains("\"recipients\": null"),
        "the scholarship units' absent recipient count is emitted as null rather than dropped"
    );
    assert!(
        feed.contains("\"derived\": 0,"),
        "a unit with nothing derived says so rather than omitting the key"
    );
}
