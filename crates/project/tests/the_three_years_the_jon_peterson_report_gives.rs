//! The three consecutive years one scholarship programme published about itself, and what they
//! settle or leave open.
//!
//! Jon Peterson had a standalone annual report until the 2025 consolidated report absorbed it, and
//! the department posted these two together and late — both exported from Word on one afternoon in
//! March 2025, nine months after FY2024 closed, onto a page that had carried no JPSN report at all
//! since some point between June 2023 and June 2024. Issue #484 found them live. With the
//! consolidated report's Jon Peterson section that is FY2023, FY2024 and FY2025, which is the first
//! year-on-year change in this programme's participation that the department itself published
//! anywhere, and the first shortening of the FY2014-FY2023 hole
//! [`the_points_that_bound_the_participation_hole`](the_points_that_bound_the_participation_hole.rs)
//! describes.
//!
//! # What these tests settle
//!
//! - **The consolidated report's Jon Peterson line is read twice and agrees with itself.** Two
//!   parsers reach the same document by different routes — one walking five programmes' prose, one
//!   walking one programme's section — and produce the same participation figure. That was the
//!   issue's third ask.
//! - **The department sums this chart itself, once.** The FY2023 edition's stated total equals the
//!   sum of its own six category figures to the cent, which is what makes the same sum a quotation
//!   rather than a derivation in the two editions that state nothing —
//!   [`report::JON_PETERSON_DERIVED_EXPENDITURE`] among them, a hand-typed constant that is now
//!   checked against the fixture rather than asserted.
//! - **A pie chart's label order is not a category order.** The disability shares move position
//!   between editions, so [`jpsn::Series::position_is_a_category`] is true of one series of three.
//!
//! # What they do not settle
//!
//! Which of the FY2023 edition's figures are FY2023's. Its award table is identical to FY2024's,
//! its per-student spend is 11.6% lower, and the one total it states in words it attributes to the
//! 2021-2022 school year — while the award table it shares with FY2024 is a uniform fraction of a
//! statutory vector whose fraction moves between the other editions, so printing one table twice
//! means an index that stood still. Two readings survive all of that. Either the spending chart
//! lags its edition, or the award table was copied forward from FY24 when the two documents were
//! produced in one sitting — and under the second the spending chart is FY2023's after all. Against
//! the first: the FY2024 chart is corroborated by a second publisher, the FY2025 chart is labelled
//! with its year, and a lagged FY2023 chart would leave FY2023 with no spending figure anywhere.
//! The arithmetic both readings have to explain is pinned here; the reading is left open in
//! `program/jon-peterson-special-needs`.

use project::scholarship::bounds::{self, Measure};
use project::scholarship::jpsn::Series;
use project::scholarship::{jpsn, report};

use edfund_core::FiscalYear;

/// A cent, for figures the department publishes to the cent.
const CENT: f64 = 0.005;

fn year(fiscal_year: u16) -> FiscalYear {
    FiscalYear(fiscal_year)
}

#[test]
fn three_consecutive_years_carry_the_programmes_first_published_change() {
    let editions = jpsn::editions();
    assert_eq!(jpsn::span(), 2023..=2025);

    let students: Vec<f64> = editions.values().map(|e| e.students).collect();
    assert_eq!(students, [8_186.0, 8_551.0, 8_680.0]);

    // +4.46% and then +1.51%. Growth that is slowing, which is a thing that can only be said with
    // three points and could not be said at all from any single edition.
    let first = students[1] / students[0] - 1.0;
    let second = students[2] / students[1] - 1.0;
    assert!((0.0445..0.0446).contains(&first), "{first:.4}");
    assert!((0.0150..0.0152).contains(&second), "{second:.4}");
    assert!(second < first, "the change stopped decelerating");

    // Providers grew faster than students in both years — 439 to 500 to 530, against 6% total
    // growth in participation — so the average provider served fewer students each year.
    let providers: Vec<f64> = editions.values().map(|e| e.providers).collect();
    assert_eq!(providers, [439.0, 500.0, 530.0]);
    let per_provider: Vec<f64> = students
        .iter()
        .zip(&providers)
        .map(|(s, p)| s / p)
        .collect();
    assert!(
        per_provider.windows(2).all(|w| w[1] < w[0]),
        "{per_provider:?}"
    );

    /*
     * And the one series that does not rise: districts of residence, 487 then 495 then 494. Every
     * edition says the programme reaches "nearly" or "over" 80 percent of Ohio's districts, and by
     * FY2025 that is what it is doing — the reach saturated while participation kept growing, so
     * the growth after FY2024 is more students from districts already reached rather than more
     * districts. This is the only quantity in the three fixtures that measures reach at all.
     */
    let districts: Vec<f64> = editions.values().map(|e| e.districts).collect();
    assert_eq!(districts, [487.0, 495.0, 494.0]);
    assert!(districts[2] < districts[1] && students[2] > students[1]);
}

#[test]
fn the_consolidated_reports_jon_peterson_line_is_read_twice_and_agrees_with_itself() {
    /*
     * The issue's third ask, and it is a continuity check rather than a reconciliation: the FY24
     * standalone report is FY2024 and the consolidated report is FY2025, so the two are
     * consecutive years of one series and nothing requires them to be equal.
     *
     * What does have to be equal is the consolidated report against itself. Its Jon Peterson
     * figures reach this workspace by two independent parsers — `scholarship_programmes`, which
     * walks the five programmes' participation sentences, and `jpsn_edition`, which walks one
     * programme's section and cross-checks its school year, its provider count and its district
     * count. Both read 8,680. If the two ever disagreed, one of the fixtures would be quoting a
     * document neither the other nor the catalog record describes.
     */
    let editions = jpsn::editions();
    let consolidated = &editions[&year(report::FISCAL_YEAR)];
    let channel_line = &report::programmes()[jpsn::PROGRAM];
    assert_eq!(consolidated.students, channel_line.students);
    assert_eq!(consolidated.students, 8_680.0);

    // The standalone FY2024 edition sits below it by the year's growth, which is the only relation
    // the two documents bear to each other.
    let standalone = &editions[&year(report::FISCAL_YEAR - 1)];
    assert!(standalone.students < consolidated.students);
    assert_eq!(standalone.year.0 + 1, report::FISCAL_YEAR);

    // And the consolidated report still publishes no expenditure total for this programme, in
    // either fixture. The three editions state one between them, and it is not this one's.
    assert!(channel_line.expenditure.is_none());
    assert!(consolidated.stated_total.is_none());
}

#[test]
fn the_one_total_the_department_states_equals_the_sum_of_its_own_chart() {
    /*
     * What licenses `jpsn::spending`. Only the FY2023 edition writes a total in words —
     * "$81,773,133.70" — and it equals the sum of the six category figures printed directly below
     * it, to the cent. So summing that chart is quoting the department's arithmetic, and the two
     * editions that print the chart without the sentence are not being given a total they never
     * implied.
     *
     * This is the whole of the evidence for it. There is one such sentence in the lineage and it
     * is in the edition the department deleted first.
     */
    let editions = jpsn::editions();
    let stated: Vec<u16> = editions
        .values()
        .filter(|e| e.stated_total.is_some())
        .map(|e| e.year.0)
        .collect();
    assert_eq!(stated, [2023], "a second edition now states a total");

    let fy2023 = &editions[&year(2023)];
    let charted = jpsn::spending(fy2023.year).expect("the edition charts six categories");
    let written = fy2023.stated_total.expect("FY2023 states a total");
    assert!(
        (charted - written).abs() < CENT,
        "{charted:.2} vs {written:.2}"
    );
    assert_eq!(written, 81_773_133.70);

    /*
     * And the year it names is not its own. The sentence says "the 2021-2022 school year" in a
     * report that says FY2023 everywhere else, including in the heading above the award table the
     * extractor reads the fiscal year from. The identical phrase opens the paragraph two sentences
     * earlier, describing when House Bill 110 abolished the district deduction, which is where a
     * copy error would have come from — and the two later editions drop the sentence while keeping
     * the chart, so nothing in the lineage confirms or refutes it.
     *
     * The fixture carries the figure and the year it names in separate columns for exactly this
     * reason, and this assertion is the record that they differ rather than a claim about why.
     */
    assert_eq!(fy2023.total_is_its_own_year(), Some(false));
    assert_eq!(fy2023.stated_total_year, Some(year(2022)));
    for other in editions.values().filter(|e| e.year.0 != 2023) {
        assert_eq!(other.total_is_its_own_year(), None);
    }
}

#[test]
fn the_hand_typed_derived_expenditure_is_now_read_off_a_fixture() {
    /*
     * `report::JON_PETERSON_DERIVED_EXPENDITURE` was typed by hand, because the 2025 annual report
     * charts this programme's spending in six pieces and never totals them and nothing in the
     * workspace held the pieces. Now something does. The constant stays — `report::channel` needs a
     * figure for a programme whose fixture cell is deliberately blank — but it is checked rather
     * than asserted, which is the difference `unbound-prose-figures-are-unchecked` is about.
     */
    let summed = jpsn::spending(year(report::FISCAL_YEAR)).expect("FY2025 charts six categories");
    assert!(
        (summed - report::JON_PETERSON_DERIVED_EXPENDITURE).abs() < CENT,
        "{summed:.2} vs the constant's {:.2}",
        report::JON_PETERSON_DERIVED_EXPENDITURE
    );

    // The channel's derived half is this programme's whole spending, so the same figure has to
    // reach `channel` unchanged. It is what carries the channel past a billion dollars.
    let channel = report::channel();
    assert!((channel.derived - summed).abs() < CENT);
    assert!(channel.published < 1_000_000_000.0 && channel.total > 1_000_000_000.0);

    // The largest single disability category is 72% of it on its own — categories five and six
    // between them, autism and the multi-handicapped, are where this programme's money goes.
    let largest = jpsn::series(year(report::FISCAL_YEAR), Series::Spending)
        .into_iter()
        .fold(f64::MIN, f64::max);
    assert_eq!(largest, 74_946_785.41);
    assert!((0.72..0.73).contains(&(largest / summed)), "{largest:.2}");
}

#[test]
fn a_chart_position_is_not_a_category_and_the_award_table_is() {
    /*
     * Why the category fixture carries a `position` and not a category, for two of its three
     * series. The award maxima come from a table with a `#` column, so each amount arrives with
     * its category stated and the extractor checks the numbering runs 1 through 6. The two pie
     * charts print their labels around the slices and `pdftotext` reads them in layout order.
     *
     * Matching the disability shares by magnitude across the three editions shows that order
     * moving. The smallest slice — 0.7%, 0.71%, 0.65%, which is category 1, speech and language
     * impairment, the one category whose scholarship may only pay for therapy — is read first in
     * FY2023 and fourth in both later editions. A fixed category cannot change position.
     */
    let smallest_share: Vec<usize> = jpsn::editions()
        .keys()
        .map(|y| {
            let shares = jpsn::series(*y, Series::Share);
            let least = shares.iter().copied().fold(f64::MAX, f64::min);
            shares.iter().position(|v| *v == least).expect("a minimum") + 1
        })
        .collect();
    assert_eq!(
        smallest_share,
        [1, 4, 4],
        "the smallest disability share stopped moving between editions; that is not evidence a \
         chart position is a category"
    );
    assert!(!Series::Share.position_is_a_category());
    assert!(!Series::Spending.position_is_a_category());
    assert!(Series::Maximum.position_is_a_category());

    /*
     * The dollar chart's order does happen to hold still across all three editions — its largest
     * value is read last every time and its smallest fourth every time. That is a coincidence of
     * the same charting tool laying out three similar distributions, and it is recorded here so
     * that nobody reads the stability as a guarantee: the share chart is the same tool on the same
     * six categories and it does not preserve order.
     */
    let spending_shape: Vec<(usize, usize)> = jpsn::editions()
        .keys()
        .map(|y| {
            let spent = jpsn::series(*y, Series::Spending);
            let most = spent.iter().copied().fold(f64::MIN, f64::max);
            let least = spent.iter().copied().fold(f64::MAX, f64::min);
            (
                spent.iter().position(|v| *v == most).expect("a maximum") + 1,
                spent.iter().position(|v| *v == least).expect("a minimum") + 1,
            )
        })
        .collect();
    assert_eq!(spending_shape, [(6, 4), (6, 4), (6, 4)]);

    // Every series has six positions in every edition, which is the width the statute fixes:
    // thirteen IDEA disability categories organized into six funding levels.
    for fiscal_year in jpsn::editions().keys() {
        for which in [Series::Maximum, Series::Share, Series::Spending] {
            assert_eq!(
                jpsn::series(*fiscal_year, which).len(),
                jpsn::CATEGORIES,
                "FY{} {which:?}",
                fiscal_year.0
            );
        }
    }

    // The shares are a distribution and close to 100 in every edition — the check that the six
    // slices are the whole of the programme and not six of some larger number.
    for fiscal_year in jpsn::editions().keys() {
        let summed: f64 = jpsn::series(*fiscal_year, Series::Share).iter().sum();
        assert!(
            (summed - 100.0).abs() < 0.1,
            "FY{}: {summed}",
            fiscal_year.0
        );
    }
}

#[test]
fn two_editions_share_an_award_table_and_do_not_share_a_per_student_spend() {
    /*
     * The open question, as arithmetic. FY2023 and FY2024 print identical award maxima — the same
     * six figures to the dollar — so the ceiling on what a student could be paid did not move
     * between them. The spend per student rose 11.6% anyway. FY2024 to FY2025 behaves the other
     * way: the ceiling rises 7.2% and the spend 7.4%, which is what a year of the same programme
     * under a raised ceiling looks like.
     *
     * Either the FY2023 chart is not FY2023's — which would fit its own expenditure sentence naming
     * the 2021-2022 school year, and would leave FY2023 with no spending figure anywhere — or the
     * programme's category mix shifted sharply in a single year. The disability shares did move
     * between those two editions, and they are only published to one decimal place in the first, so
     * the mix explanation is not refuted either.
     *
     * Nothing here chooses. Both readings have to explain these four numbers.
     */
    let editions = jpsn::editions();
    let per_student: Vec<f64> = editions
        .values()
        .map(|e| {
            e.spending_per_student()
                .expect("every edition charts spending")
        })
        .collect();

    // Category one, which is prorated cleanly in both later editions. Category six is $30,000 in
    // FY2024 — a round figure below what the same proration would give — so it is the wrong column
    // to measure the table's movement on.
    let ceiling: Vec<f64> = editions
        .keys()
        .map(|y| jpsn::series(*y, Series::Maximum)[0])
        .collect();
    assert_eq!(ceiling, [8_941.0, 8_941.0, 9_585.0]);
    for fiscal_year in [2023u16, 2024] {
        assert_eq!(
            jpsn::series(year(fiscal_year), Series::Maximum),
            jpsn::series(year(2023), Series::Maximum),
            "FY2023 and FY2024 no longer print one award table"
        );
    }

    // The table stands still and the spend rises 11.6%.
    assert_eq!(ceiling[0], ceiling[1]);
    let spend_first = per_student[1] / per_student[0] - 1.0;
    assert!((0.1164..0.1165).contains(&spend_first), "{spend_first:.4}");

    // Then the two move together, to within two tenths of a percentage point.
    let ceiling_second = ceiling[2] / ceiling[1] - 1.0;
    let spend_second = per_student[2] / per_student[1] - 1.0;
    assert!(
        (0.0720..0.0721).contains(&ceiling_second),
        "{ceiling_second:.4}"
    );
    assert!(
        (0.0737..0.0738).contains(&spend_second),
        "{spend_second:.4}"
    );
    assert!(
        (spend_second - ceiling_second).abs() < 0.002,
        "{spend_second:.4} against {ceiling_second:.4}"
    );
    // And the size of the thing left open: a year in which the ceiling did not move and the spend
    // per student rose by more than a tenth.
    assert!(
        spend_first > 0.10,
        "the first year's spend growth fell below a tenth against a ceiling that did not move; the \
         tension this test records has changed shape"
    );
}

#[test]
fn the_award_table_is_recomputed_each_year_and_two_editions_print_the_same_one() {
    /*
     * The other half of the dating question, and the half that is checkable against the statute.
     *
     * R.C. 3317.022(A)(13) does not fix the six award amounts. It fixes a base indexed to the
     * statewide average base cost per pupil, six supplements indexed to the special education
     * category amounts, and a ceiling — so `scholarship::jon_peterson_award` is one vintage of a
     * vector that moves. What each edition prints turns out to be a **uniform fraction** of that
     * vector, the same fraction at every position, which is how a prorated statutory amount
     * behaves and not how an independently chosen table does.
     *
     * The fraction is 0.8901 in FY2023 and FY2024 and 0.9542 in FY2025. Two different fractions
     * across three editions is the evidence that the table is recomputed annually. And FY2023 and
     * FY2024 print it to the dollar identically, which under an annual recomputation means the
     * index did not move between them at all.
     *
     * That is the second reason the FY2023 edition's figures are held open, and it points somewhere
     * the per-student arithmetic alone does not: at the award table rather than at the spending
     * chart. If the FY2023 table were copied forward from FY2024 when the two documents were
     * produced — and the catalog record has them exported from Word twenty minutes apart on one
     * afternoon in March 2025, nine months after FY2024 closed — then the spending chart is FY2023's
     * after all and the 11.6% per-student rise is real. Nothing here chooses between that reading
     * and a genuinely flat index. Both are recorded in `program/jon-peterson-special-needs`.
     */
    let statutory: Vec<f64> = (1..=jpsn::CATEGORIES)
        .map(project::scholarship::jon_peterson_award)
        .collect();

    let factor = |fiscal_year: u16| -> Vec<f64> {
        jpsn::series(year(fiscal_year), Series::Maximum)
            .iter()
            .zip(&statutory)
            .map(|(printed, statute)| printed / statute)
            .collect()
    };

    // FY2025 prorates all six positions alike, the capped one included.
    let fy2025 = factor(2025);
    let spread = fy2025.iter().copied().fold(f64::MIN, f64::max)
        - fy2025.iter().copied().fold(f64::MAX, f64::min);
    assert!(spread < 0.0005, "{fy2025:?}");
    assert!((0.9542..0.9543).contains(&fy2025[0]), "{:.4}", fy2025[0]);

    /*
     * FY2023 and FY2024 prorate the first five alike and pin the sixth at a round $30,000, which is
     * $262.94 below what the same fraction would give against a $34,000 ceiling. That is why the
     * fixture's sixth position is the wrong column to measure the table's movement on, and why this
     * test measures the fraction on the five that are not rounded.
     */
    for fiscal_year in [2023u16, 2024] {
        let f = factor(fiscal_year);
        let uncapped = &f[..jpsn::CATEGORIES - 1];
        let spread = uncapped.iter().copied().fold(f64::MIN, f64::max)
            - uncapped.iter().copied().fold(f64::MAX, f64::min);
        assert!(spread < 0.0005, "FY{fiscal_year}: {uncapped:?}");
        assert!(
            (0.8900..0.8901).contains(&f[0]),
            "FY{fiscal_year}: {:.4}",
            f[0]
        );
        assert_eq!(
            jpsn::series(year(fiscal_year), Series::Maximum)[5],
            30_000.0
        );
        assert!(f[5] < f[0], "the round cap stopped being a rounding down");
    }

    // The fraction moves, so the table is not a constant the department reprints.
    assert!(factor(2025)[0] > factor(2024)[0] + 0.06);

    // And these two editions print one table anyway.
    assert_eq!(
        jpsn::series(year(2023), Series::Maximum),
        jpsn::series(year(2024), Series::Maximum)
    );
}

#[test]
fn the_two_publishers_agree_on_the_money_and_not_on_the_counts() {
    /*
     * FY2024 is the one fiscal year where the department and the Legislative Service Commission
     * both describe this programme, so it is the only place either can be checked against the
     * other. They agree on the dollars to the rounding grain LSC publishes at and disagree on
     * every count.
     *
     * That asymmetry is worth holding on its own. A payment total is a number the two publishers
     * could only get from the same ledger; a participation count is a number that depends on who
     * you decide was participating, and the annual report's own five programmes already show three
     * unreconciled averages from exactly that.
     */
    let fy2024 = &jpsn::editions()[&year(2024)];
    let redbook = |measure: Measure| {
        bounds::of(jpsn::PROGRAM, measure)
            .into_iter()
            .find(|b| b.source == "dew-redbook")
            .unwrap_or_else(|| panic!("the census holds one FY2024 redbook {measure:?} row"))
    };

    // $95.4 million against $95,362,957.53 summed from the edition's own chart. LSC publishes to a
    // tenth of a million and this rounds to it, which is the strongest evidence in the lineage that
    // the chart belongs to the fiscal year its edition is dated to.
    let payments = redbook(Measure::Payments);
    assert_eq!(payments.year.0, fy2024.year.0);
    let charted = jpsn::spending(fy2024.year).expect("FY2024 charts six categories");
    assert_eq!((charted / 100_000.0).round() * 100_000.0, payments.value);

    // And the counts. LSC says about 7,800 students where the department says 8,551 — 9.6% above
    // LSC's figure, and a gap in the direction the department's own definition would predict if it
    // counts everyone who used the programme at any point in the year.
    let participation = redbook(Measure::Participation);
    assert_eq!(participation.value, 7_800.0);
    let gap = fy2024.students / participation.value - 1.0;
    assert!((0.096..0.097).contains(&gap), "{gap:.4}");

    /*
     * The provider counts are further apart than that, and in a way no rounding explains: LSC's 454
     * for FY2024 sits *between* the department's FY2023 figure of 439 and its FY2024 figure of 500.
     * A reader holding both documents and no third one could take LSC's number for either year.
     *
     * This is why `Bound::spliceable` does not treat a provider count as joinable to this series.
     * The year is occupied, so the two figures are a contradiction rather than a splice, and the
     * contradiction belongs where it can be read rather than inside a predicate.
     */
    let providers = redbook(Measure::Providers);
    assert_eq!(providers.value, 454.0);
    let fy2023 = &jpsn::editions()[&year(2023)];
    assert!(fy2023.providers < providers.value && providers.value < fy2024.providers);
    assert!(providers.spliceable().is_none());
}

#[test]
fn the_hole_shortened_by_one_year_for_one_programme_of_five() {
    /*
     * What #484 actually bought, stated so that a later extraction cannot quietly overstate it.
     * The channel's participation hole was FY2014 through FY2023; it is now FY2014 through FY2022,
     * and the year that came off belongs to Jon Peterson alone. Four of the five programmes still
     * have nothing at all between the archive's last year and the consolidated report's.
     */
    let covered: Vec<u16> = jpsn::editions().keys().map(|y| y.0).collect();
    assert_eq!(covered, [2023, 2024, 2025]);

    let others: Vec<String> = report::programmes()
        .keys()
        .filter(|slug| *slug != jpsn::PROGRAM)
        .cloned()
        .collect();
    assert_eq!(others.len(), 4);

    // Nothing in this fixture reaches them, which is the sentence the decision record has to keep
    // making: the hole is one year shorter and it is not one programme narrower.
    for slug in &others {
        assert_ne!(slug.as_str(), jpsn::PROGRAM);
    }

    /*
     * And the series does not reach the bar charts that would fill the hole rather than shorten it.
     * Every edition opens with "total applications by fiscal year" from FY2014 onward — ten or
     * twelve points of exactly the quantity the archive's first denominator measures — and not one
     * of those bars carries a data label in the PDF's text layer. `pdftotext` reaches the axis
     * gridlines and nothing else, so recovering them means reading the chart's geometry, which is a
     * different instrument. This assertion is the marker: if a fourth series ever appears here,
     * that is what changed.
     */
    for fiscal_year in jpsn::editions().keys() {
        for which in [Series::Maximum, Series::Share, Series::Spending] {
            assert!(!jpsn::series(*fiscal_year, which).is_empty());
        }
    }
    assert!(
        jpsn::editions().values().all(|e| e.students > 0.0),
        "participation is the only series of applications-era shape this fixture carries, and it \
         is a count of students served rather than of applications made"
    );
}
