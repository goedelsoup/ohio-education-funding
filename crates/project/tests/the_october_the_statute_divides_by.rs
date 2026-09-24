//! Forty-nine Octobers of chartered nonpublic enrolment, and the one the statute divides by.
//!
//! R.C. 3317.024(E)(2)(d) names a denominator: the average daily membership in chartered nonpublic
//! schools "as determined as of the last day of October of each school year". The corpus held no
//! such count and said so — `program/auxiliary-services` carried it as an unfilled field and every
//! per-pupil figure in the node was published as a bound. The department has in fact published it
//! every year since 1977, as attachments on one CMS page that no connector owned.
//!
//! [`project::ledger::nonpublic_enrolment`] is the reader for the two fixtures that page now
//! produces. This is what it is held to, and the questions are four.
//!
//! **Whether the series is whole.** Forty-nine Octobers with no gap, from a page whose files come
//! in six different shapes and three of whose compilations are named for the years their sheets
//! *end* in.
//!
//! **Whether the quotient closes.** It does, to ten cents, and three readings had to be right at
//! once — see [`the_published_rate_reproduces_on_the_october_the_statute_names`].
//!
//! **Whether two of the department's own files agree.** For four of the six Octobers both describe,
//! they do not, and the extract carries both rather than picking.
//!
//! **What it still cannot say.** Not one file in any era carries the district a building sits in,
//! and R.C. 3317.06 pays through that district under the first of the two routes. The building
//! panel is keyed on IRN because the alternative does not work: October 2023 holds 711 schools
//! under 601 names.

use std::collections::{BTreeMap, BTreeSet};

use project::ledger::nonpublic_enrolment as enrolment;
use project::ledger::nonpublic_support as nonpublic;

/// A dollar figure is compared to the cent.
const CENT: f64 = 0.005;

#[test]
fn the_series_reaches_every_october_since_1977() {
    /*
     * The first question about a series assembled out of twenty-three files is whether it has a
     * hole in it, and the shape of these files is what makes that worth asserting rather than
     * assuming. Four of them are compilations named for the school year their *last* sheet ends
     * in — `1978-1979-Nonpublic.xls` holds 1977-78 and 1978-79, `2000-2008-NonPublic.xls` holds
     * 1999-00 through 2007-08 — so an off-by-one in reading a file name is an off-by-one in the
     * series, and it would present as a missing year rather than as an error.
     */
    let (first, last) = enrolment::span();
    assert_eq!(
        (first, last),
        (1978, 2026),
        "the span of the sector extract"
    );

    let annual: Vec<u16> = enrolment::octobers()
        .iter()
        .filter(|october| october.is_annual())
        .map(|october| october.fiscal_year)
        .collect();
    assert_eq!(annual.len(), 49, "forty-nine Octobers");

    let held: BTreeSet<u16> = annual.iter().copied().collect();
    assert_eq!(
        held.len(),
        annual.len(),
        "no October read twice: {annual:?}"
    );
    for fiscal_year in first..=last {
        assert!(held.contains(&fiscal_year), "FY{fiscal_year} is missing");
    }

    // And the label the extract derives agrees with what the department named the sheet.
    let october = enrolment::october(2025).expect("October 2024");
    assert_eq!(october.school_year, "2024-25");
    assert_eq!(october.source, "nonpublic-2024");
}

#[test]
fn the_published_rate_reproduces_on_the_october_the_statute_names() {
    /*
     * The probe #469 asked for, and it closes.
     *
     * It did not before, and the reason was never the arithmetic. R.C. 3317.024(E)(2)(d) divides
     * "the total amount appropriated for the implementation of sections 3317.06 and 3317.062" by
     * an October membership; what the corpus had was the department's landscape sheet, 173,156
     * pupils for 2023-24, which is the wrong October for FY2025 and produced $963.39 against the
     * $913 LSC publishes in both editions.
     *
     * Three readings had to be right at once for the gap to close, and each is refutable on its
     * own:
     *
     *   - the numerator is net of the College Credit Plus earmark, $2,739,015 of ALI 200511 that
     *     implements neither section the quotient names;
     *   - the October is the school year's, so FY2025 divides by October 2024 and not October
     *     2025;
     *   - and the count is of pupils living in Ohio, 179,693, rather than all 181,171 the file
     *     totals. The whole-total reading gives $905.65, which nobody publishes.
     *
     * $164,077,754 over 179,693 is $913.1004. LSC states $913.
     */
    let rate = nonpublic::auxiliary_rate_fy2025().expect("FY2025 and October 2024");
    assert!((rate - 913.100_421_274_061_9).abs() < 1e-9, "{rate}");
    assert!(
        (rate - nonpublic::PUBLISHED_AUXILIARY_RATE_FY2025).abs() < 0.11,
        "the reproduction stopped landing on the published rate: {rate}"
    );

    let october = enrolment::october(2025).expect("October 2024");
    assert!((october.published_in_state.expect("in state") - 179_693.0).abs() < CENT);
    assert!((october.published_out_of_state.expect("out of state") - 1_478.0).abs() < CENT);
    assert!((october.published_total.expect("total") - 181_171.0).abs() < CENT);
    assert_eq!(october.buildings, Some(722));

    // The reading that does not reproduce, kept because it is what rules the other one in.
    let appropriated = nonpublic::latest(nonpublic::AUXILIARY_SERVICES, 2025)
        .and_then(|claim| claim.amount)
        .expect("FY2025");
    let on_the_whole_total = (appropriated - nonpublic::COLLEGE_CREDIT_PLUS_FY2025)
        / october.published_total.expect("total");
    assert!(
        (on_the_whole_total - 905.651_312_848_082_7).abs() < 1e-9,
        "{on_the_whole_total}"
    );
    assert!(
        (on_the_whole_total - nonpublic::PUBLISHED_AUXILIARY_RATE_FY2025).abs() > 5.0,
        "the two denominators stopped being distinguishable, \
         which would mean the in-state reading is no longer evidence"
    );
}

#[test]
fn the_landscape_sheet_and_the_october_file_count_the_same_pupils() {
    /*
     * The cross-publisher check #469 asked for, and the two agree exactly.
     *
     * `Ohio's Education Landscape 2023-2024` is a fact sheet from one office; the October file is
     * a data extract from another, with a different layout, a different file format and a masking
     * convention the fact sheet does not use. They are not copies of each other. On 2023-24 they
     * state the same two numbers: 173,156 pupils and 711 chartered nonpublic schools.
     *
     * This is now what the landscape figure is for. It was the denominator, and as a denominator
     * it was a year early — which is the whole of why the per-pupil figure used to miss.
     */
    let landscape = nonpublic::chartered_nonpublic_enrolment();
    let october = enrolment::october(2024).expect("October 2023");

    // The same school year, spelled the two ways the two publications spell it.
    assert_eq!(nonpublic::ENROLMENT_VINTAGE, "2023-2024");
    assert_eq!(october.school_year, "2023-24");
    assert!((landscape - 173_156.0).abs() < CENT, "{landscape}");
    assert!(
        (october.published_total.expect("a total") - landscape).abs() < CENT,
        "two publications of one department stopped agreeing: {october:?}"
    );
    assert_eq!(
        october.buildings,
        Some(711),
        "711 chartered nonpublic schools"
    );

    // And the counts LSC publishes are not a third disagreement: they are later Octobers of this
    // same series, which is why the redbook's 747 sits above the greenbook's 725 above 711.
    assert_eq!(enrolment::schools(2024), Some(711));
    assert_eq!(enrolment::schools(2025), Some(722));
    assert_eq!(enrolment::schools(2026), Some(749));
}

#[test]
fn the_masked_cells_leave_a_bound_and_the_published_total_sits_inside_it() {
    /*
     * The department writes `<10` over a cell it will not publish, and the extraction refuses to
     * read that as a zero — see the `censored-cells-have-a-second-column` finding, where reading
     * it as zero understated a sector by thousands. What it carries instead is an interval: the
     * sum of what survived, and that sum plus nine for every mask.
     *
     * The interval is not decorative. From October 2015 the department states its own sector total
     * on a separate sheet, computed before the masking, and the test that total has to pass is
     * that it lies inside the interval its own building sheet permits. It does, for every October
     * that states one — with a single exception that is a finding rather than a tolerance, pinned
     * below.
     */
    let mut checked = 0;
    for october in enrolment::octobers().iter().filter(|o| o.is_annual()) {
        let (Some(k12), Some(published)) = (october.k12, october.published_in_state) else {
            continue;
        };
        if october.fiscal_year == 2016 {
            continue; // Pinned in its own test below: this file contradicts itself.
        }
        assert!(
            k12.contains(published),
            "FY{} publishes {published} outside [{}, {}]",
            october.fiscal_year,
            k12.floor,
            k12.ceiling
        );
        checked += 1;
    }
    assert_eq!(
        checked, 10,
        "ten Octobers state a total over a building sheet"
    );

    // The width of the interval is nine a masked cell, which is the mask's own meaning and not a
    // convention this repository chose.
    let october = enrolment::october(2025).expect("October 2024");
    let k12 = october.k12.expect("a building sheet");
    assert_eq!(k12.censored, 5_137);
    assert!((k12.width() - 9.0 * 5_137.0).abs() < CENT, "{k12:?}");
    assert!(!k12.is_exact());
}

#[test]
fn two_of_the_departments_own_files_describe_six_octobers_and_disagree() {
    /*
     * `2014-2019-NonPublic.xlsx` restates six Octobers the annual files had already published, and
     * for four of them it does not agree with them. The extract carries both rows rather than
     * reconciling them, keyed on the basis, because a disagreement between two files of one
     * department is the finding.
     *
     * Two of the four are explicable and two are not. The compilation counts preschool through
     * twelve where the annual files count kindergarten through twelve, so its totals should sit
     * *above* — and on FY2014 and FY2015 they are the only statement, the annual files of those
     * years publishing no total at all. FY2018 reconciles exactly, to the pupil, which is what
     * makes the other three worth reporting rather than shrugging at.
     *
     * FY2019 differs by fourteen pupils. FY2017 differs by 1,537 — and the number the compilation
     * gives as the *total*, 171,426, is exactly what the annual file gives as its *in-state*
     * figure, so the two files disagree about what the column means rather than about the count.
     * October 2016's file is also the one that turned out to be the previous October's building
     * sheet under a new name.
     */
    let reconciles = [(2018, 169_091.0)];
    let differs = [(2017, 172_963.0, 171_426.0), (2019, 166_808.0, 166_794.0)];

    for (fiscal_year, agreed) in reconciles {
        let annual = enrolment::october(fiscal_year).expect("the annual file");
        let restated = enrolment::restatement(fiscal_year).expect("the compilation");
        assert!((annual.published_total.expect("a total") - agreed).abs() < CENT);
        assert!((restated.published_total.expect("a total") - agreed).abs() < CENT);
    }

    for (fiscal_year, annual_total, restated_total) in differs {
        let annual = enrolment::october(fiscal_year).expect("the annual file");
        let restated = enrolment::restatement(fiscal_year).expect("the compilation");
        assert!((annual.published_total.expect("a total") - annual_total).abs() < CENT);
        assert!((restated.published_total.expect("a total") - restated_total).abs() < CENT);
        assert!(
            (annual_total - restated_total).abs() > CENT,
            "FY{fiscal_year} stopped disagreeing, which is a finding and not a fix"
        );
    }

    // The FY2017 coincidence, stated rather than described.
    let annual = enrolment::october(2017).expect("the annual file");
    let restated = enrolment::restatement(2017).expect("the compilation");
    assert!(
        (annual.published_in_state.expect("in state") - restated.published_total.expect("a total"))
            .abs()
            < CENT,
        "the compilation's total is the annual file's in-state figure"
    );

    // And October 2016's file states no building count of its own, because its building sheet is
    // October 2015's under October 2016's name.
    assert_eq!(annual.buildings, None);
    assert_eq!(annual.k12, None);
}

#[test]
fn october_2015s_own_two_sheets_do_not_describe_the_same_pupils() {
    /*
     * The one October whose published total falls outside the bound its own building sheet leaves,
     * and it falls *below* the floor — below the sum of the unmasked cells, which no reading of
     * the masking can produce.
     *
     * `Revised-FY16-nonpub.xls` states 144,512 in-state on its totals sheet over a building sheet
     * whose published cells already add to 147,309. The 2014-2019 compilation says 172,990 for the
     * same October. So the file disagrees with itself and with the department's own later
     * restatement of it, and this repository does not choose: the reader returns what was
     * published and this test is the flag on it.
     *
     * It matters for exactly one thing, which is why it is pinned rather than worked around: it is
     * the earliest October a per-pupil rate could be computed for, and it should not be.
     */
    let october = enrolment::october(2016).expect("October 2015");
    let k12 = october.k12.expect("a building sheet");
    let published = october.published_in_state.expect("a totals sheet");

    assert!((published - 144_512.0).abs() < CENT, "{published}");
    assert!((k12.floor - 147_309.0).abs() < CENT, "{k12:?}");
    assert!(
        published < k12.floor,
        "the totals sheet came back inside the bound, which would make this file consistent"
    );

    let restated = enrolment::restatement(2016).expect("the compilation");
    assert!((restated.published_total.expect("a total") - 172_990.0).abs() < CENT);
    assert!(
        k12.contains(172_990.0),
        "the compilation's figure, unlike the file's own, is consistent with the building sheet"
    );
}

#[test]
fn the_building_panel_is_keyed_on_irn_because_the_names_repeat() {
    /*
     * Issue #469 asked for the extract to be keyed on building IRN and on the district the
     * building sits in. The first half is done here and the second cannot be: no nonpublic file in
     * any era carries a district column, and the department's Chartered Nonpublic School
     * Information page publishes no directory to join against. The connector records it as what it
     * is still blocked on.
     *
     * The reason a name join was not attempted as a substitute is one line of arithmetic. October
     * 2023's 711 schools carry 601 distinct names. Sixteen of them are called `St Mary`, nine
     * `St Joseph`, seven `St Peter` — and the public districts they would be joined to have the
     * same problem from the other side, 607 districts under 580 names. A join on that is not a
     * weaker answer than a join on IRN; it is a different and wrong one.
     */
    let buildings = enrolment::buildings(2024);
    assert_eq!(buildings.len(), 711, "the sector row's count");
    assert_eq!(
        enrolment::schools(2024),
        Some(buildings.len()),
        "the two panels stopped agreeing on how many schools October 2023 held"
    );

    let irns: BTreeSet<&str> = buildings.iter().map(|school| school.irn.as_str()).collect();
    assert_eq!(irns.len(), buildings.len(), "every IRN is distinct");

    let mut names: BTreeMap<&str, usize> = BTreeMap::new();
    for school in &buildings {
        *names.entry(school.school.as_str()).or_default() += 1;
    }
    assert_eq!(names.len(), 601, "and the names are not");
    assert_eq!(names.get("St Mary"), Some(&16));
    assert_eq!(names.get("St Joseph"), Some(&9));
    assert_eq!(names.get("St Peter"), Some(&7));

    // What the building rows do carry, and what they stop carrying. County and school type are
    // published only through October 2006, which is the other thing a resident-side join would
    // have needed.
    let early = enrolment::buildings(1978);
    assert_eq!(early.len(), 788);
    assert!(early.iter().all(|school| !school.county.is_empty()));
    assert!(buildings.iter().all(|school| school.county.is_empty()));
}

#[test]
fn the_membership_is_held_from_october_2015_and_not_before() {
    /*
     * What the series can and cannot be divided by, stated as a boundary rather than discovered
     * one call at a time.
     *
     * Before October 2015 the department published no sector total of its own: the only sector
     * figure obtainable from those files is the sum of a masked building sheet, which is an
     * interval. An interval is not a denominator, so `per_pupil` answers nothing for those years
     * and the older Octobers are held as bounds and read as such.
     *
     * The other end is the same shape for the opposite reason. FY2027's quotient divides by
     * October 2026, which has not happened.
     */
    assert_eq!(enrolment::membership(2015), None, "October 2014");
    assert!(enrolment::october(2015).expect("the file").k12.is_some());

    assert!((enrolment::membership(2016).expect("October 2015") - 144_512.0).abs() < CENT);
    assert!((enrolment::membership(2026).expect("October 2025") - 184_069.0).abs() < CENT);
    assert_eq!(enrolment::membership(2027), None, "October 2026");

    assert_eq!(
        nonpublic::per_pupil(nonpublic::AUXILIARY_SERVICES, 2027),
        None,
        "a line with an appropriation and no denominator has no rate"
    );
    let whole = nonpublic::per_pupil(nonpublic::AUXILIARY_SERVICES, 2025).expect("FY2025");
    assert!((whole - 928.343_168_626_490_8).abs() < 1e-9, "{whole}");
}
