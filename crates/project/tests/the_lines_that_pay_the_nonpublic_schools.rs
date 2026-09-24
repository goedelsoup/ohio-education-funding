//! The quarter of a billion a year that reaches private schooling without being a scholarship.
//!
//! The corpus's account of state money in nonpublic education was the five scholarship
//! programmes. The department's own budget structure has a second heading —
//! **Category 3: Nonpublic School Support** — and its three lines have been in
//! `fixtures/catalog-line-items.csv` since the 2006 edition without anything reading them.
//! [`project::ledger::nonpublic_support`] is the reader; this is what it is held to.
//!
//! Three questions, and the third one used to be the one that did not close.
//!
//! **What was paid.** Eight years of each line, at the vintage that answers for the year. Pinned
//! rather than described, because the corpus quotes these figures and a series quoted from a
//! document nobody re-reads is the #120 shape.
//!
//! **Whether the catalog contradicts itself.** Four editions restate a given year, and the claim
//! the reader rests on is that they never *disagree*: an edition changes a year's figure only by
//! changing its basis, `appropriation` → `adjusted` → `actual`. Asserted over all 182 lines and
//! 3,303 cells of the fixture rather than over these three, because a three-line version would
//! pass on a fixture that had gone wrong everywhere else.
//!
//! **What a pupil is worth.** R.C. 3317.024(E)(2)(d) really does divide an appropriation by a
//! membership, and for as long as the corpus held no October membership this file published a
//! bound instead of a rate. It now holds forty-nine of them — see
//! `the_october_the_statute_divides_by`, which owns the series and the reproduction — and the
//! quotient comes out at $913.10 against the $913 LSC publishes. What is left here is the part
//! that never was about the denominator: two published rates for FY2025 imply two different
//! memberships five per cent apart, and only one of them is a membership at all.

use edfund_core::FiscalYear;
use project::ledger::nonpublic_enrolment as enrolment;
use project::ledger::nonpublic_support as nonpublic;
use project::statute;

/// The year every real figure here is stated in.
const BASE: FiscalYear = FiscalYear(2025);

/// A dollar figure is compared to the cent.
const CENT: f64 = 0.005;

/// One pinned year: what it was, on what basis, and how much.
type Pin = (u16, &'static str, f64);

/// One line's eight years, named by its ALI.
type Line = (&'static str, [Pin; 8]);

#[test]
fn the_three_lines_carry_eight_years_each_at_one_vintage() {
    /*
     * The pins. Each row is (fiscal year, kind, amount), and the kind is pinned beside the
     * amount on purpose: the last two years of any series are appropriations and the rest are
     * actuals, so a pin that carried only the number would go on passing through the FY2026
     * close that turns $170,292,963 into whatever was spent.
     */
    let expected: [Line; 3] = [
        (
            nonpublic::AUXILIARY_SERVICES,
            [
                (2020, "actual", 154_097_444.0),
                (2021, "actual", 151_872_301.0),
                (2022, "actual", 156_052_027.0),
                (2023, "actual", 158_189_613.0),
                (2024, "actual", 162_864_614.0),
                (2025, "actual", 166_816_769.0),
                (2026, "appropriation", 170_292_963.0),
                (2027, "appropriation", 172_262_613.0),
            ],
        ),
        (
            nonpublic::ADMINISTRATIVE_COST_REIMBURSEMENT,
            [
                (2020, "actual", 62_223_628.0),
                (2021, "actual", 68_853_796.0),
                (2022, "actual", 70_759_968.0),
                (2023, "actual", 71_500_744.0),
                (2024, "actual", 73_440_062.0),
                (2025, "actual", 75_337_397.0),
                (2026, "appropriation", 76_935_110.0),
                (2027, "appropriation", 77_824_960.0),
            ],
        ),
        (
            nonpublic::AUXILIARY_SERVICES_REIMBURSEMENT,
            [
                (2020, "actual", 130_517.0),
                (2021, "actual", 280_551.0),
                (2022, "actual", 542_449.0),
                (2023, "actual", 383_481.0),
                (2024, "actual", 573_700.0),
                (2025, "actual", 534_753.0),
                (2026, "appropriation", 650_000.0),
                (2027, "appropriation", 650_000.0),
            ],
        ),
    ];

    for (ali, rows) in expected {
        let series = nonpublic::series(ali, BASE);
        for (fiscal_year, kind, amount) in rows {
            let year = nonpublic::at(&series, fiscal_year)
                .unwrap_or_else(|| panic!("{ali} carries no FY{fiscal_year}"));
            assert!(
                (year.nominal - amount).abs() < CENT,
                "{ali} FY{fiscal_year}: {} against {amount}",
                year.nominal
            );
            assert_eq!(year.kind, kind, "{ali} FY{fiscal_year}");
        }
    }
}

#[test]
fn the_category_is_a_quarter_of_a_billion_a_year() {
    let category = nonpublic::category_three(BASE);
    for (fiscal_year, total) in [
        (2020, 216_451_589.0),
        (2021, 221_006_648.0),
        (2022, 227_354_444.0),
        (2023, 230_073_838.0),
        (2024, 236_878_376.0),
        (2025, 242_688_919.0),
        (2026, 247_878_073.0),
        (2027, 250_737_573.0),
    ] {
        let year = nonpublic::at(&category, fiscal_year).expect("a year of the category");
        assert!(
            (year.nominal - total).abs() < CENT,
            "FY{fiscal_year}: {} against {total}",
            year.nominal
        );
    }

    // The category reaches back as far as the fixture does, and the sum is over three lines in
    // every one of those years — so no year of it is a partial total wearing a full one's label.
    assert_eq!(
        category.first().map(|year| year.fiscal_year),
        Some(2002),
        "the category no longer starts where the catalog does"
    );
    assert!(
        category.iter().all(|year| year.kind != "mixed"),
        "an edition closed some of the three lines for a year and not the others"
    );
}

#[test]
fn the_auxiliary_line_grew_by_a_twelfth_and_fell_by_an_eighth() {
    /*
     * The reason the corpus states the category in constant dollars. Nominal totals rise in
     * nearly every biennium here, which makes "record support for nonpublic schools" a true and
     * empty sentence; the FY2020-FY2025 window covers the inflation of 2021-2023 and the line
     * loses a seventh of its purchasing power across it.
     *
     * FY2020 rather than the start of the fixture because FY2027 has no deflated figure — the
     * CPI-U series cannot reach a year that has not happened — and a growth rate wants both
     * endpoints on the same footing.
     */
    let window = |ali: &str| -> Vec<_> {
        nonpublic::series(ali, BASE)
            .into_iter()
            .filter(|year| (2020..=2025).contains(&year.fiscal_year))
            .collect()
    };

    let (nominal, real) =
        nonpublic::growth(&window(nonpublic::AUXILIARY_SERVICES)).expect("both endpoints deflate");
    assert!(
        (nominal - 0.082_540_791_526_691_36).abs() < 1e-12,
        "{nominal}"
    );
    assert!((real - -0.134_812_427_933_921).abs() < 1e-12, "{real}");
    assert!(real < 0.0 && nominal > 0.0, "the sign that makes the point");

    let (nominal, real) = nonpublic::growth(&window(nonpublic::ADMINISTRATIVE_COST_REIMBURSEMENT))
        .expect("both endpoints deflate");
    assert!(
        (nominal - 0.210_752_240_290_456_92).abs() < 1e-12,
        "{nominal}"
    );
    assert!((real - -0.032_343_354_310_784_72).abs() < 1e-12, "{real}");

    let category: Vec<_> = nonpublic::category_three(BASE)
        .into_iter()
        .filter(|year| (2020..=2025).contains(&year.fiscal_year))
        .collect();
    let (nominal, real) = nonpublic::growth(&category).expect("both endpoints deflate");
    assert!(
        (nominal - 0.121_215_695_949_453_25).abs() < 1e-12,
        "{nominal}"
    );
    assert!((real - -0.103_902_695_091_219_19).abs() < 1e-12, "{real}");
}

#[test]
fn an_edition_changes_a_year_only_by_changing_its_basis() {
    /*
     * The invariant the whole reader rests on, and it is stronger than the three lines need.
     * 3,303 (line, year) cells over 182 line items, up to four editions each, and not one pair
     * of editions publishes the same cell on the same basis at two different numbers. Every
     * edition-to-edition difference in this fixture is `appropriation` -> `adjusted` -> `actual`.
     *
     * That is what makes `series` taking the newest edition a choice of *vintage* rather than an
     * arbitration between two claims about one quantity — the distinction
     * `ledger::appropriations` records getting wrong once, by mixing two publishers' actuals into
     * one series whose vintage changed halfway along without saying so.
     */
    let (cells, lines) = nonpublic::restatable_cells();
    assert_eq!(cells, 3_303, "the fixture's size moved");
    assert_eq!(lines, 182, "the fixture's line count moved");

    let disagreements = nonpublic::same_kind_disagreements();
    assert!(
        disagreements.is_empty(),
        "two editions publish one cell on one basis at two amounts: {disagreements:?}"
    );

    for ali in nonpublic::CATEGORY_THREE {
        let restated = nonpublic::restatements(ali);
        assert_eq!(restated.len(), 27, "{ali}");
        let standing: Vec<_> = restated
            .iter()
            .filter(|restatement| !restatement.is_kind_transition())
            .collect();
        assert!(standing.is_empty(), "{ali}: {standing:?}");
    }
}

#[test]
fn the_fy2025_restatement_is_thirty_six_thousand_dollars() {
    /*
     * The worked instance: the 2024 edition carries FY2025 as an adjusted appropriation of
     * $166,853,000 and the 2025 edition replaces it with an actual of $166,816,769. A $36,231
     * difference on a $167m line — two hundredths of one per cent — which is what a restatement
     * looks like when nothing went wrong, and the reason the invariant above is worth asserting
     * rather than eyeballing.
     */
    let restated = nonpublic::restatements(nonpublic::AUXILIARY_SERVICES);
    let fy2025: Vec<_> = restated
        .iter()
        .filter(|restatement| restatement.fiscal_year == 2025)
        .collect();

    // Two, because the year appears in three editions: the 2023 edition's appropriation is
    // re-published unchanged as the 2024 edition's adjusted figure — a basis moving with no
    // money moving, which is itself worth pinning — and the 2025 edition closes it.
    assert_eq!(fy2025.len(), 2, "{fy2025:?}");
    assert!(
        (fy2025[0].shift()).abs() < CENT,
        "the appropriation was adjusted after all: {:?}",
        fy2025[0]
    );

    let restatement = fy2025[1];
    assert_eq!(restatement.from.0, 2024);
    assert_eq!(restatement.from.1, "adjusted");
    assert!((restatement.from.2 - 166_853_000.0).abs() < CENT);
    assert_eq!(restatement.to.0, 2025);
    assert_eq!(restatement.to.1, "actual");
    assert!((restatement.to.2 - 166_816_769.0).abs() < CENT);
    assert!((restatement.shift() - -36_231.0).abs() < CENT, "the shift");
}

#[test]
fn auxiliary_services_per_pupil_is_a_rate_now_and_was_a_bound() {
    /*
     * The probe the issue asked for, and the answer it used to have.
     *
     * R.C. 3317.024(E)(2)(d) divides "the total amount appropriated for the implementation of
     * sections 3317.06 and 3317.062" by the October average daily membership in chartered
     * nonpublic schools. The corpus held no October membership. What it held was the department's
     * landscape sheet — 173,156 pupils in 711 chartered nonpublic schools for **2023-24** — and
     * dividing FY2025 by that gave $963.39 against the $913 LSC publishes in both editions.
     *
     * The reading recorded here at the time was that the gap was the denominator and its direction
     * known: a count too small makes the quotient too large, so $963.39 was an upper bound. The
     * direction was right and the reason given for it was wrong. The landscape count was not short
     * of the redbook's 747 schools and the greenbook's 725; those are later Octobers of the same
     * series, which runs 711, 722, 749. It was short by a year, and only by a year.
     *
     * What the October series settles is asserted in `the_october_the_statute_divides_by`. What is
     * asserted here is the shape of the correction: the old denominator, the old quotient, and the
     * fact that the new one lands on the published rate.
     */
    let landscape = nonpublic::chartered_nonpublic_enrolment();
    assert!((landscape - 173_156.0).abs() < CENT, "{landscape}");

    let appropriated = nonpublic::latest(nonpublic::AUXILIARY_SERVICES, 2025)
        .and_then(|claim| claim.amount)
        .expect("FY2025");
    let on_the_landscape_count = appropriated / landscape;
    assert!(
        (on_the_landscape_count - 963.390_058_675_414).abs() < 1e-9,
        "{on_the_landscape_count}"
    );
    assert!(
        on_the_landscape_count > nonpublic::PUBLISHED_AUXILIARY_RATE_FY2025,
        "the old denominator stopped being too small, \
         which would make the bound argument the wrong way round"
    );

    let rate = nonpublic::auxiliary_rate_fy2025().expect("FY2025 and October 2024");
    assert!((rate - 913.100_421_274_061_9).abs() < 1e-9, "{rate}");
    assert!(
        (rate - nonpublic::PUBLISHED_AUXILIARY_RATE_FY2025).abs() < 0.11,
        "the quotient stopped reproducing the published rate: {rate}"
    );

    // What the published rate implies as a denominator, taken both ways the numerator can be read:
    // on the whole line, and on the line net of the College Credit Plus earmark, which is money
    // the statute's quotient has no reason to include. The second is the one the October count
    // answers, and it answers it to twenty pupils.
    let whole =
        nonpublic::implied_membership(appropriated, nonpublic::PUBLISHED_AUXILIARY_RATE_FY2025)
            .expect("a positive rate");
    let net = nonpublic::implied_membership(
        appropriated - nonpublic::COLLEGE_CREDIT_PLUS_FY2025,
        nonpublic::PUBLISHED_AUXILIARY_RATE_FY2025,
    )
    .expect("a positive rate");
    assert!((whole - 182_712.780_941_949_6).abs() < 1e-6, "{whole}");
    assert!((net - 179_712.764_512_595_84).abs() < 1e-6, "{net}");

    let counted = enrolment::membership(2025).expect("October 2024");
    assert!(
        (net - counted - 19.764_512_595_837).abs() < 1e-6,
        "{net} v {counted}"
    );
    assert!(
        (net / counted - 1.0).abs() < 0.000_2,
        "the inversion of a rate rounded to the dollar lands within two hundredths of a per cent"
    );
    assert!(
        whole > counted + 3_000.0,
        "and the whole-line reading does not, which is what rules the earmark out of the quotient"
    );
}

#[test]
fn the_two_published_rates_do_not_share_a_denominator() {
    /*
     * The part of the per-pupil question that was never about the missing denominator.
     *
     * LSC publishes two FY2025 per-pupil figures for Category 3: $913 for auxiliary services and
     * a $440 ceiling for the administrative cost reimbursement. Inverting each against its own
     * appropriation gives 179,713 pupils and 171,221 pupils — five per cent apart. They cannot
     * both be the October membership, and now that the October membership is held it is clear
     * which one is: the department counted 179,693 pupils in October 2024, so the auxiliary
     * figure lands on it and the reimbursement's sits 4.7% below.
     *
     * Nor should they be. Auxiliary services is a quotient: the appropriation divided by a count,
     * so its implied denominator *is* the count. The reimbursement is not — R.C. 3317.063 pays
     * actual prior-year costs up to a ceiling, and a school that spent less is paid less, so the
     * appropriation over the ceiling is an upper bound on the pupils in schools claiming the
     * maximum rather than a membership at all. Two numbers that disagree because they are two
     * different quantities, and the corpus states them as such rather than picking one.
     */
    let auxiliary = nonpublic::implied_membership(
        nonpublic::latest(nonpublic::AUXILIARY_SERVICES, 2025)
            .and_then(|claim| claim.amount)
            .expect("FY2025")
            - nonpublic::COLLEGE_CREDIT_PLUS_FY2025,
        nonpublic::PUBLISHED_AUXILIARY_RATE_FY2025,
    )
    .expect("a positive rate");
    let administrative = nonpublic::implied_membership(
        nonpublic::latest(nonpublic::ADMINISTRATIVE_COST_REIMBURSEMENT, 2025)
            .and_then(|claim| claim.amount)
            .expect("FY2025"),
        nonpublic::PUBLISHED_ADMINISTRATIVE_RATE_FY2025,
    )
    .expect("a positive rate");

    assert!(
        (administrative - 171_221.356_818_181_8).abs() < 1e-6,
        "{administrative}"
    );
    let spread = auxiliary / administrative - 1.0;
    assert!(
        (spread - 0.049_593_157_373_650_33).abs() < 1e-12,
        "the two implied memberships: {spread}"
    );
    assert!(
        spread > 0.04,
        "the two rates stopped disagreeing about the denominator, \
         which would be a finding and not a fix"
    );

    // And which of the two is the membership, now that the membership is a number we hold.
    let counted = enrolment::membership(2025).expect("October 2024");
    assert!(
        (auxiliary / counted - 1.0).abs() < 0.000_2,
        "auxiliary: {auxiliary} against {counted}"
    );
    let short = 1.0 - administrative / counted;
    assert!((short - 0.047_145_092_918_578_824).abs() < 1e-12, "{short}");
}

#[test]
fn the_reimbursement_is_rationed_below_the_rate_the_statute_allows() {
    /*
     * The difference between the two mechanisms, made arithmetic.
     *
     * R.C. 3317.063 sets no rate of its own: it pays actual costs "not to exceed the per-pupil
     * amount specified by the general assembly for that school year". The General Assembly
     * specified $475 for FY2024 through FY2027, and LSC records that in FY2025 "the amount
     * appropriated permitted DEW to pay up to $440 per student". So the binding limit was the
     * appropriation and not the ceiling, and a school's claim was cut by 7.4% before the ceiling
     * was reached.
     *
     * A direct appropriation has no such gap to have: its rate is whatever the quotient comes to.
     */
    let (shortfall, share) = nonpublic::rationing_fy2025();
    assert!((shortfall - 35.0).abs() < CENT, "{shortfall}");
    assert!((share - 0.073_684_210_526_315_78).abs() < 1e-12, "{share}");
    const {
        assert!(
            nonpublic::PUBLISHED_ADMINISTRATIVE_RATE_FY2025
                < nonpublic::AUTHORIZED_ADMINISTRATIVE_RATE,
            "the appropriation stopped being the binding limit"
        );
    }
}

#[test]
fn the_reflux_line_is_three_thousandths_of_the_category() {
    /*
     * Why R.C. 3317.064 is folded into the auxiliary services node as a property rather than
     * given one of its own.
     *
     * It is not a programme. Its fund's revenue is a transfer of whatever the R.C. 4141.47
     * auxiliary services personnel unemployment compensation fund holds in excess of its claims,
     * and 4141.47(A) says that fund "shall consist of moneys paid into the fund pursuant to
     * section 3317.06" — GRF 200511's own unspent money, two steps removed. Both LSC editions add
     * the fact that settles it: no transfers have occurred since FY 2013, so twelve years of
     * spending on this line have been drawdown of a balance.
     *
     * The size is the second reason and the weaker one: $534,753 against $242.7 million.
     */
    let category = nonpublic::at(&nonpublic::category_three(BASE), 2025).expect("FY2025");
    let mobile = nonpublic::latest(nonpublic::AUXILIARY_SERVICES_REIMBURSEMENT, 2025)
        .and_then(|claim| claim.amount)
        .expect("FY2025");
    let share = mobile / category.nominal;
    assert!((share - 0.002_203_450_417_940_178).abs() < 1e-15, "{share}");

    let auxiliary = nonpublic::latest(nonpublic::AUXILIARY_SERVICES, 2025)
        .and_then(|claim| claim.amount)
        .expect("FY2025");
    let dominant = auxiliary / category.nominal;
    assert!(
        (dominant - 0.687_368_709_240_490_6).abs() < 1e-15,
        "{dominant}"
    );

    assert_eq!(nonpublic::LAST_TRANSFER_YEAR, 2013);
}

#[test]
fn the_category_is_a_quarter_of_what_the_scholarships_pay() {
    /*
     * The scale claim, on one measure rather than two.
     *
     * Both sides are the money as reported rather than as appropriated: Category 3's FY2025
     * figures are actuals, and the scholarship side is the department's own 2025 Scholarship
     * Annual Report covering 2024-25. Four of the five programmes publish expenditure; Jon
     * Peterson's is not published, so the denominator is short by one programme and the ratio
     * here is an **upper** bound on Category 3's share.
     *
     * Even as a bound it carries the point: a channel the corpus did not model is a quarter the
     * size of the one it did. And the two are not parallel in the budget — the scholarships are
     * funded inside an earmark of GRF ALI 200550, so they have no line of their own, while these
     * three have nothing else in theirs.
     */
    let scholarships: f64 = project::scholarship::report::programmes()
        .values()
        .filter_map(|programme| programme.expenditure)
        .sum();
    assert!(
        (scholarships - 991_191_150.12).abs() < CENT,
        "{scholarships}"
    );

    let category = nonpublic::at(&nonpublic::category_three(BASE), 2025).expect("FY2025");
    let share = category.nominal / scholarships;
    assert!((share - 0.244_845_728_264_037_17).abs() < 1e-12, "{share}");
    assert!(
        (0.20..0.30).contains(&share),
        "a quarter is no longer the right word"
    );
}

#[test]
fn the_section_the_line_is_named_for_does_not_allocate_it() {
    /*
     * The reading that moved two of this issue's premises, and the reason three sections beyond
     * the three it named were vendored.
     *
     * R.C. 3317.06 is a *uses* section. Its seventeen divisions say what may be bought with
     * "moneys paid to school districts under division (E)(1) of section 3317.024" and none of
     * them says how much is paid. The allocating sentence is R.C. 3317.024(E)(2)(d), and it is
     * the one that makes the per-pupil probe above a statutory question rather than an analyst's
     * convenience.
     *
     * R.C. 3317.062 is the same list again for the other route: what a school may spend the money
     * on when it is paid directly instead of through its district.
     */
    let uses = statute::section("3317.06");
    assert!(
        uses.body.starts_with(
            "Moneys paid to school districts under division (E)(1) of section 3317.024"
        ),
        "3317.06 no longer opens as a uses section: {}",
        &uses.body[..120.min(uses.body.len())]
    );
    // It does name a membership, and the distinction is the point: 3317.06's estimated ADM
    // apportions the money *among districts*, while the rate per pupil is set one section
    // earlier. A section can divide a total between recipients without deciding the total.
    assert!(
        uses.body.contains(
            "estimated annual average daily membership in nonpublic elementary and high schools \
             located in the district"
        ),
        "3317.06's apportionment basis moved"
    );
    assert!(
        !uses.body.contains("divided by"),
        "3317.06 acquired a quotient"
    );

    let payment = statute::section("3317.024");
    assert!(
        payment.body.contains(
            "shall equal the total amount appropriated for the implementation of sections 3317.06 \
             and 3317.062 of the Revised Code divided by the average daily membership in grades \
             kindergarten through twelve in chartered nonpublic elementary and high schools within \
             the state as determined as of the last day of October of each school year"
        ),
        "the allocating sentence moved out of 3317.024"
    );

    // Both routes exist, and the second one is why "auxiliary services" is a nonpublic
    // entitlement with a district route rather than a district programme serving nonpublic
    // pupils. The 4% is what an intermediary may keep.
    assert!(
        payment.body.contains(
            "An amount for auxiliary services paid directly to each chartered nonpublic school"
        ),
        "the direct-payment route left 3317.024"
    );
    assert!(
        payment.body.contains("does not exceed four per cent"),
        "the designated organisation's fee ceiling left 3317.024"
    );

    let direct = statute::section("3317.062");
    assert!(
        direct
            .body
            .starts_with("(A) Moneys paid to chartered nonpublic schools under division (E)(2)"),
        "3317.062 no longer governs the direct route"
    );
}

#[test]
fn the_mobile_unit_fund_is_fed_from_the_auxiliary_services_line() {
    /*
     * The two steps that make ALI 200659 a reflux rather than a third appropriation, each read
     * off the section that performs it.
     */
    let reimbursement = statute::section("3317.064");
    assert!(
        reimbursement.body.contains(
            "excess moneys in the auxiliary services personnel unemployment compensation fund"
        ),
        "3317.064's revenue source moved"
    );

    let unemployment = statute::section("4141.47");
    assert!(
        unemployment.body.contains(
            "The fund shall consist of moneys paid into the fund pursuant to section 3317.06 of \
             the Revised Code"
        ),
        "4141.47 no longer says where its money comes from"
    );
}

#[test]
fn the_reimbursement_pays_costs_and_the_appropriation_pays_a_quotient() {
    /*
     * The evidence for giving the two nodes different `mechanism` properties, taken from the
     * sections rather than from the line titles.
     *
     * R.C. 3317.063 has every mark of a reimbursement and the auxiliary services sections have
     * none of them: actual prior-year costs, an application, separate accounts, audit, and
     * repayment of anything paid above cost.
     */
    let reimbursement = statute::section("3317.063");
    for phrase in [
        "shall annually reimburse each chartered nonpublic school for the actual mandated service",
        "incurred by such school during the preceding school year",
        "shall maintain a separate account or system of accounts",
        "shall not exceed the per-pupil amount specified by the general assembly",
        "said school shall immediately reimburse the state in such excess amount",
    ] {
        assert!(
            reimbursement.body.contains(phrase),
            "3317.063 no longer says: {phrase}"
        );
    }

    /*
     * The auxiliary services sections have none of those marks. 3317.06 does use the word —
     * districts and educational service centers are reimbursed for the cost of *administering*
     * the services, and a district is reimbursed for storing a closed school's records — but
     * neither is a payment to a nonpublic school for what that school spent, and neither is
     * measured against a per-pupil ceiling. The three phrases that make 3317.063 a reimbursement
     * of the school's own costs are absent from it.
     */
    let uses = statute::section("3317.06");
    for absent in [
        "preceding school year",
        "per-pupil amount specified by the general assembly",
        "separate account or system of accounts",
    ] {
        assert!(
            !uses.body.contains(absent),
            "3317.06 acquired a reimbursement's machinery: {absent}"
        );
    }
    assert!(
        uses.body
            .contains("shall be reimbursed for administrative costs"),
        "3317.06's district-side reimbursement moved, \
         and the contrast above is drawn against it"
    );
}

#[test]
fn h_b_96_amended_the_uses_and_created_no_line() {
    /*
     * Why H.B. 96 is a `revisions` entry and an edge rather than a new node. The act adds two
     * permitted uses to R.C. 3317.06 — mental health services, and retired Ohio peace officers
     * among those a school may hire for security — and appropriates to the same three ALIs. The
     * committed section carries the act as its own source, so the corpus's claim that the text it
     * reads is the amended text is checked rather than asserted.
     */
    let uses = statute::section("3317.06");
    assert!(
        uses.legislation.contains("House Bill 96"),
        "3317.06's vendored version is not H.B. 96's: {}",
        uses.legislation
    );
    assert!(
        uses.body
            .contains("To provide diagnostic mental health or psychological services"),
        "division (D)'s mental health service is gone"
    );
    assert!(
        uses.body.contains(
            "To provide therapeutic mental health, psychological, and speech and hearing services"
        ),
        "division (E)'s mental health service is gone"
    );
    assert!(
        uses.body.contains("from a retired Ohio peace officer"),
        "division (P)'s retired peace officer is gone"
    );

    // And the act's own analysis says exactly that much and no more about the line.
    const ANALYSIS: &str = include_str!("../fixtures/enacted-school-funding.txt");
    assert!(
        ANALYSIS.contains(
            "The act permits chartered nonpublic schools to use auxiliary services funds to provide"
        ),
        "the final analysis no longer carries the auxiliary services item"
    );
}
