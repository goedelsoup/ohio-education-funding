//! How large the scholarship channel is, and two places the department's own report disagrees
//! with itself.
//!
//! [`the_voucher_channel_is_absent`](the_voucher_channel_is_absent.rs) established where this
//! money is *not*: not in the FY2027 funding calculator, because under the Fair School Funding
//! Plan each scholarship programme is its own funding unit paid directly rather than deducted
//! from a resident district. Its argument turned partly on scale — a deduction channel would have
//! to be roughly ten times the size of the $95.6 million residual it could have hidden in.
//!
//! That "roughly" is now a number. These tests hold it, from the department's own annual report
//! rather than from an estimate.
//!
//! # Why the report's averages are carried rather than corrected
//!
//! Three of the four published averages are not expenditure over participation. Autism's
//! reconciles to the cent and the other three sit above the implied figure by 1.5% to 3.4%. That
//! pattern is a denominator rather than an arithmetic slip — plausibly an average over students
//! holding a full-year award rather than over everyone who used the programme at any point — but
//! the report does not say, so the fixture carries both numbers and these tests pin the
//! disagreement instead of resolving it. If a later edition explains it, the explanation belongs
//! in the catalog record and this file should stop describing it as unexplained.

use project::scholarship::report::{channel, programmes, school_year, FISCAL_YEAR};
use project::scholarship::units::{self, UNITS};

/// A cent, for figures the report publishes to the cent.
const CENT: f64 = 0.005;

#[test]
fn the_report_covers_the_five_programmes_the_deduction_calculator_names() {
    // If the department adds or renames one, the `deduction` skill's list of four programmes plus
    // the expansion has gone stale, and so has every programme node in the corpus.
    let p = programmes();
    let mut keys: Vec<&str> = p.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "autism",
            "cleveland",
            "edchoice-expansion",
            "jon-peterson",
            "traditional-edchoice"
        ]
    );
}

#[test]
fn the_channel_is_a_billion_dollars_and_the_absent_deduction_argument_survives() {
    /*
     * The scale claim `the_voucher_channel_is_absent` rests on. That test measured the only place
     * a deduction could have hidden — the negative half of `T - Other Adjustments`, $95.6 million
     * across 577 districts — and argued the channel is far too large to fit. This is the other
     * side of that comparison, and it is not an estimate.
     */
    let p = programmes();
    let published: f64 = p.values().filter_map(|x| x.expenditure).sum();

    // $991.2 million, and deliberately not "about a billion": this is the sum of the four
    // programmes that publish a total. Jon Peterson does not, and adding its derived $103.9
    // million is what carries the channel past a billion. The distinction matters because the
    // round number is the one that gets quoted, and only one of the two is fully sourced.
    assert!(
        (991_191_150.0..991_191_151.0).contains(&published),
        "${published:.2}"
    );
    assert!(published < 1_000_000_000.0);

    // The scale argument in `the_voucher_channel_is_absent` survives on the sourced figure alone,
    // without needing the derived one: an order of magnitude above the residual a deduction could
    // have hidden in.
    assert!(published > 95_600_000.0 * 10.0, "${published:.2}");
}

#[test]
fn jon_peterson_publishes_participation_but_not_a_total() {
    /*
     * Its spending appears only as six disability-category figures in a chart. The fixture leaves
     * the column empty rather than summing them, because a derived total in a column of quoted
     * ones is the kind of figure that gets quoted back as published. The sum is stated in the
     * catalog record, labelled as derived.
     */
    let p = programmes();
    let jpsn = &p["jon-peterson"];
    assert_eq!(jpsn.students, 8_680.0);
    assert!(jpsn.expenditure.is_none(), "the report gained a JPSN total");
    assert!(jpsn.published_average.is_none());
}

#[test]
fn three_of_four_published_averages_do_not_equal_expenditure_over_participation() {
    /*
     * The finding this file exists for. Each of these is a correct-looking number that a reader
     * would reasonably compute a different way and get a different answer, which is the exact
     * shape of error `web/src/lib/denominators.ts` exists to catch on the formula side.
     */
    let p = programmes();
    let mut reconciles = Vec::new();
    let mut diverges = Vec::new();
    for (key, prog) in &p {
        let (Some(spend), Some(published)) = (prog.expenditure, prog.published_average) else {
            continue;
        };
        let implied = spend / prog.students;
        if (published - implied).abs() < 0.005 {
            reconciles.push(key.as_str());
        } else {
            // Always the published figure above the implied one, never below.
            assert!(
                published > implied,
                "{key}: published {published:.2} is below implied {implied:.2}, which inverts \
                 the pattern this test records"
            );
            let gap = (published - implied) / implied;
            assert!(
                (0.01..0.05).contains(&gap),
                "{key}: the gap is {:.2}%, outside the 1-5% band observed",
                gap * 100.0
            );
            diverges.push(key.as_str());
        }
    }
    assert_eq!(reconciles, ["autism"], "which programme reconciles changed");
    assert_eq!(diverges.len(), 3, "{diverges:?}");
}

#[test]
fn the_executive_summary_overstates_the_sum_of_its_own_programmes() {
    /*
     * The report's opening line says the five programmes served "more than 175,000" students. The
     * five counts it publishes sum to 166,587. Double-counting a student who holds two
     * scholarships would push the sum *up* toward the headline, not explain a shortfall, so the
     * headline is drawing on something the programme sections do not contain.
     *
     * Pinned because 175,000 is the quotable number and 166,587 is the one this corpus can
     * source. Anything here that cites participation must cite the second.
     */
    let p = programmes();
    let summed: f64 = p.values().map(|x| x.students).sum();
    assert_eq!(summed, 166_587.0);
    assert!(
        summed < 175_000.0,
        "the parts now reach the headline; the discrepancy has closed and the catalog record \
         should say so"
    );
}

#[test]
fn the_channel_carries_its_derived_half_beside_its_published_one() {
    /*
     * `channel` is what the feed publishes, and the whole point of it is that it does not hand a
     * consumer one number. $991.2 million is quoted by the department; $103.9 million is summed
     * from six category figures it displays without totalling; $1.095 billion is the sum and is
     * the figure that gets quoted back. A block that flattened the three would have made the
     * distinction unavailable to every page downstream of it.
     */
    let c = channel();

    assert!(
        (c.published - 991_191_150.12).abs() < CENT,
        "{:.2}",
        c.published
    );
    assert!(
        (c.derived - 103_944_388.15).abs() < CENT,
        "{:.2}",
        c.derived
    );
    assert!((c.total - 1_095_135_538.27).abs() < CENT, "{:.2}", c.total);
    assert_eq!(c.students, 166_587.0);

    // Four programmes publish an expenditure and five publish participation. The pair is in the
    // struct so a page can say "four of five" without counting the fixture itself.
    assert_eq!(c.programmes, (4, 5));

    // The published half stays short of a billion and the whole channel clears it. That is the
    // sentence the round number comes from, and it is only true of one of the two.
    assert!(c.published < 1_000_000_000.0);
    assert!(c.total > 1_000_000_000.0);
}

#[test]
fn the_school_year_is_derived_from_the_fiscal_one() {
    /*
     * A school year straddles two calendar years and the report names itself by one. Derived
     * rather than typed, so the two cannot disagree — `every-figure-says-its-year` records what
     * it costs when a label and the data under it are two separate statements.
     */
    assert_eq!(school_year(), "2024-25");
    assert_eq!(FISCAL_YEAR, 2025);
}

#[test]
fn six_units_and_five_programmes_join_on_the_educational_choice_unit() {
    /*
     * The count that trips a reader. R.C. 3317.022 names six funding units; the annual report
     * covers five programmes; and the two lists are different lengths because division (A)(10)
     * computes traditional EdChoice and the income-based expansion as one unit.
     *
     * Every programme slug a unit names has to exist in the fixture, or a page rendering "the
     * educational choice scholarship unit" silently drops half of it.
     */
    let p = programmes();
    assert_eq!(UNITS.len(), 6);

    let named: Vec<&str> = UNITS
        .iter()
        .flat_map(|u| u.programmes.iter().copied())
        .collect();
    assert_eq!(named.len(), p.len(), "{named:?}");
    for slug in &named {
        assert!(
            p.contains_key(*slug),
            "unit names `{slug}`, absent from the report"
        );
    }

    // The two units the report does not cover, and the one that covers two programmes.
    let uncovered: Vec<&str> = UNITS
        .iter()
        .filter(|u| u.programmes.is_empty())
        .map(|u| u.slug)
        .collect();
    assert_eq!(uncovered, ["district", "community-stem"]);
    assert_eq!(
        units::of("educational-choice").map(|u| u.programmes.len()),
        Some(2)
    );

    // Every unit says which division computes it. A unit with no authority is a unit a page
    // cannot link, and the two computed outside division (A) carry the section that does.
    for unit in UNITS {
        assert!(unit.authority.starts_with("R.C. 3317.02"), "{}", unit.slug);
    }
}

#[test]
fn the_programmes_name_the_nodes_that_document_them() {
    /*
     * Every programme in the fixture has a write-up, and every write-up names a programme.
     *
     * The map is what the statewide funding-units block links against, so a slug that drifts out
     * of the fixture becomes a dead `/wiki/program/` link on a published page. This crate cannot
     * see `.yidam/corpus/`, so what it can check is the half that lives here: the keys are the
     * fixture's own slugs and nothing is missing from either side. The node names themselves are
     * resolved by the web layer, which renders them.
     */
    let fixture = programmes();
    assert_eq!(
        units::NODES.len(),
        fixture.len(),
        "every programme the report covers should name a node"
    );
    for (slug, node) in units::NODES {
        assert!(
            fixture.contains_key(slug),
            "`{slug}` names a node but is not a programme in the fixture"
        );
        assert!(
            !node.is_empty() && node.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
            "`{node}` has to be a corpus file name, which is lowercase and hyphens"
        );
        assert_eq!(units::node(slug), Some(node));
    }
    assert_eq!(units::node("no-such-programme"), None);
}
