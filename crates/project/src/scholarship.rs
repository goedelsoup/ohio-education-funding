//! What Ohio's four scholarship programmes pay, as R.C. 3317.022 and R.C. 3310.08 compute it.
//!
//! Every one of these amounts sits in the committed statute extract and none had been read.
//! `program/edchoice-expansion` records `amount: Not yet populated` and "Exact bands not yet
//! established"; `program/jon-peterson-special-needs` holds the first of six category supplements
//! and describes the rest as "other amounts for other categories".
//!
//! # There are no bands
//!
//! The expansion's award is a **continuous function of family income**, not a schedule of bands.
//! R.C. 3310.08 gives it as a formula in prose, and read back it is one line: above 450% of the
//! federal poverty guidelines the award **halves for every further 100 percentage points**, and
//! stops at a tenth of the base. See [`expansion_award`], which is the arithmetic rather than a
//! transcription of a table that does not exist.
//!
//! # All four are ceilings, and they are ceilings on different things
//!
//! No programme pays its stated amount as such. Each pays the lesser of that amount and what the
//! school or provider charges — but "what it charges" is defined four different ways, and the
//! differences are not cosmetic: the pilot project nets out *all* financial aid and adjustments,
//! EdChoice nets out only tuition discounts, and the two special-needs programmes read a tuition
//! or a fee for the programme rather than for the school. A comparison across them is a
//! comparison of four denominators.
//!
//! # What is not here
//!
//! The dollar value of 450% of the federal poverty guidelines, because it depends on family size
//! and the statute points outward for it (R.C. 5101.46). So this module gives the award as a
//! function of the income *ratio*, which is the form the statute states and the only form that is
//! self-contained.

use edfund_core::Dollars;

/// R.C. 3310.08(A)(1), the "constant multiplier". The award's decay rate.
pub const CONSTANT_MULTIPLIER: f64 = 0.50;

/// R.C. 3310.08(B)(1): the income ratio at or below which the full base amount is paid.
///
/// 450% of the federal poverty guidelines, written here as a multiple because that is what the
/// statute's own exponent uses — a "federal poverty level multiplier" of 4.5 makes the two
/// branches of (B) agree at the boundary, and one of 450 would make the award vanish there.
pub const FULL_AWARD_CEILING: f64 = 4.5;

/// R.C. 3310.08(A)(6): the floor, as a share of the base amount.
pub const MINIMUM_SHARE: f64 = 0.10;

/// R.C. 3317.022(A)(10)(a)(ii)(I), grades kindergarten through eight, as enacted by H.B. 33.
///
/// Not a fixed figure: the section directs it to "increase in future fiscal years by the same
/// percentage that the statewide average base cost per pupil increases", so the voucher ceiling
/// tracks the Fair School Funding Plan's own base cost by statute. This is the enacted value and
/// a caller comparing it against a later year's payments must index it first.
pub const EDCHOICE_BASE_K8: Dollars = 5_500.0;

/// The same for grades nine through twelve, and the same indexing.
pub const EDCHOICE_BASE_9_12: Dollars = 7_500.0;

/// R.C. 3317.022(A)(12)(a)(ii): the autism scholarship ceiling.
///
/// **Carries no indexing clause**, unlike every EdChoice and Jon Peterson amount in the same
/// division, which `program/autism-scholarship` records.
pub const AUTISM_CEILING: Dollars = 34_000.0;

/// R.C. 3317.022(A)(13)(a)(ii): the Jon Peterson award before its category supplement.
///
/// Indexed to the statewide average base cost per pupil, like the EdChoice amounts.
pub const JON_PETERSON_BASE: Dollars = 7_190.0;

/// R.C. 3317.022(A)(13)(a)(iii): the Jon Peterson ceiling. The same figure as [`AUTISM_CEILING`],
/// and like it carrying no indexing clause.
pub const JON_PETERSON_CEILING: Dollars = 34_000.0;

/// R.C. 3317.022(A)(13)(a)(ii)(I) to (VI), by special education category one through six.
///
/// Indexed to the special education category amounts under division (A)(3) rather than to base
/// cost, so these move with the weights in
/// [`formula-component/fsfp-special-education-weights`](../../.yidam/corpus/formula-component/fsfp-special-education-weights.yml)
/// and the base above moves with base cost. Two indexing rules inside one award.
pub const JON_PETERSON_SUPPLEMENTS: [Dollars; 6] =
    [2_855.0, 5_879.0, 12_879.0, 16_890.0, 22_560.0, 31_932.0];

/// The EdChoice Expansion award for a student, before the tuition ceiling.
///
/// `income_over_poverty` is the family's income as a multiple of the federal poverty guidelines —
/// 4.5 for a family at 450%. Below that the base amount is paid in full; above it the award is
/// `base * 0.5^(income_over_poverty - 4.5)`, floored at a tenth of the base.
///
/// R.C. 3310.08(B)(2) writes that as `base X (1 / constant multiplier)^4.5 X e^power equation`
/// with the power equation `federal poverty level multiplier X ln(constant multiplier)`. The two
/// are the same expression; this computes the statute's form so that a reader checking it against
/// the section is checking the section and not a simplification.
#[must_use]
pub fn expansion_award(base: Dollars, income_over_poverty: f64) -> Dollars {
    if income_over_poverty <= FULL_AWARD_CEILING {
        return base;
    }
    let power_equation = income_over_poverty * CONSTANT_MULTIPLIER.ln();
    let calculated =
        base * (1.0 / CONSTANT_MULTIPLIER).powf(FULL_AWARD_CEILING) * power_equation.exp();
    calculated.max(base * MINIMUM_SHARE)
}

/// The income ratio at which the award reaches its floor and stops falling.
///
/// Solving `0.5^(p - 4.5) = 0.1` gives 7.8219…, which is **782.19% of the federal poverty
/// guidelines** — the point past which a family's income no longer changes the award. Derived
/// rather than stated: the statute gives the floor as an amount and never says where it binds.
#[must_use]
pub fn minimum_award_threshold() -> f64 {
    FULL_AWARD_CEILING + MINIMUM_SHARE.ln() / CONSTANT_MULTIPLIER.ln()
}

/// The Jon Peterson award for a student in special education category `category`, one-based,
/// before the fees charged by the provider.
///
/// The least of the provider's fees, the base plus the category supplement, and the ceiling —
/// this returns the last two of those three, which is the part the statute fixes.
///
/// # Panics
///
/// On a category outside one to six. R.C. 3317.013 defines exactly six.
#[must_use]
pub fn jon_peterson_award(category: usize) -> Dollars {
    assert!(
        (1..=6).contains(&category),
        "R.C. 3317.013 defines six special education categories, not {category}"
    );
    (JON_PETERSON_BASE + JON_PETERSON_SUPPLEMENTS[category - 1]).min(JON_PETERSON_CEILING)
}

/// The department's own account of what the channel paid, and what it cannot settle.
///
/// # Why this lives beside the statute
///
/// Everything above computes what R.C. 3310.08 and 3317.022 *direct*. This reads what the
/// department reports it *paid*, so the two can be held against each other — which is the only
/// way to ask how much of the award schedule actually binds.
pub mod report {
    use std::collections::BTreeMap;

    use edfund_core::Dollars;

    /// The committed extract of the 2025 Scholarship Annual Report, covering 2024-25.
    const FIXTURE: &str = include_str!("../fixtures/scholarship-programs.csv");

    const EXPECTED_HEADER: &str = "program,name,students,expenditure,published_average";

    /// The fiscal year the report covers.
    ///
    /// Hand-written, and the only year label in this file that is. The extract has no year column
    /// — the report states its own coverage as "2024-25" in prose, once, in a title — so there is
    /// nothing in the fixture to derive it from. The registry key `scholarship-annual-2025` is the
    /// machine-readable statement of the same fact, and `project` cannot depend on `connect` to
    /// read it. Compare [`super::history::span`], which is derived because its fixture dates every
    /// row.
    pub const FISCAL_YEAR: u16 = 2025;

    /// The same year as the report itself names it, which is a school year and not a fiscal one.
    ///
    /// Derived from [`FISCAL_YEAR`] rather than typed a second time. A school year straddles two
    /// calendar years and is the reckoning the report's own title uses, so a consumer placing
    /// this channel beside an FY-labelled figure has to be told which kind of year it is on —
    /// see `.yidam/decisions/every-figure-says-its-year.yml`.
    #[must_use]
    pub fn school_year() -> String {
        format!("{}-{:02}", FISCAL_YEAR - 1, FISCAL_YEAR % 100)
    }

    /// Jon Peterson's statewide expenditure, which the report does not publish.
    ///
    /// The report gives this programme's spending as **six disability-category figures in a
    /// chart and never totals them**; this is their sum. It is a constant here and deliberately
    /// **not** a value in `scholarship-programs.csv`, whose `expenditure` column holds quoted
    /// figures and whose blank cell is the assertion that the report publishes none —
    /// `jon_peterson_publishes_participation_but_not_a_total` exists to keep it blank, on the
    /// ground that a derived total in a column of quoted ones is the kind of figure that gets
    /// quoted back as published.
    ///
    /// So it lives beside the reader, labelled, where [`channel`] can reach it and
    /// [`published_expenditure`] cannot. The catalog record
    /// `.yidam/catalog/dew-scholarship-annual-report.md` carries the same figure with the same
    /// label, and the largest single category is $74,946,785.41 of it.
    pub const JON_PETERSON_DERIVED_EXPENDITURE: Dollars = 103_944_388.15;

    /// The fixture's slug for the one programme whose expenditure is derived.
    pub const DERIVED: &str = "jon-peterson";

    /// One programme as the report gives it.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Programme {
        /// The report's own name for it.
        pub name: String,
        /// Participation. Every programme publishes one.
        pub students: f64,
        /// Statewide expenditure. Jon Peterson's is derived rather than published, and is the one
        /// programme where this is `None` in the fixture.
        pub expenditure: Option<Dollars>,
        /// The average award the report prints, which for three of the four is **not**
        /// [`Self::implied_average`].
        pub published_average: Option<Dollars>,
    }

    impl Programme {
        /// Expenditure over participation — the average with a denominator this corpus can see.
        ///
        /// Sits 1.5% to 3.4% below the published figure for every programme but autism, which
        /// reconciles to the cent. The report does not explain the gap, so both are carried and
        /// every reading below is computed on each.
        #[must_use]
        pub fn implied_average(&self) -> Option<Dollars> {
            (self.students > 0.0).then(|| self.expenditure.map(|paid| paid / self.students))?
        }
    }

    /// Every programme the report covers, keyed by the fixture's own slug.
    ///
    /// # Panics
    ///
    /// If the fixture's header is not the one this was written against, by way of
    /// [`edfund_core::csv::rows`].
    #[must_use]
    pub fn programmes() -> BTreeMap<String, Programme> {
        edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
            .map(|row| {
                (
                    row.str(0).to_string(),
                    Programme {
                        name: row.str(1).to_string(),
                        students: row.num(2).expect("every programme publishes participation"),
                        expenditure: row.num(3),
                        published_average: row.num(4),
                    },
                )
            })
            .collect()
    }

    /// What the channel paid, decomposed the way the report publishes it.
    ///
    /// Two totals rather than one, because only the first is fully sourced. See
    /// [`JON_PETERSON_DERIVED_EXPENDITURE`] for why the distinction is carried this far.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Channel {
        /// Participation across all five programmes. Every one of them publishes it.
        pub students: f64,
        /// What the four programmes that publish an expenditure paid.
        pub published: Dollars,
        /// Jon Peterson's, summed from the six category figures the report never totals.
        pub derived: Dollars,
        /// The two above. The quotable figure, and the one that is only partly quoted.
        pub total: Dollars,
        /// How many programmes the first figure covers, and how many the last one does.
        pub programmes: (usize, usize),
    }

    /// The four published expenditures, summed.
    ///
    /// Jon Peterson is absent by construction: its cell is blank in the fixture and
    /// [`Programme::expenditure`] is `None`, so nothing here has to know its name.
    #[must_use]
    pub fn published_expenditure() -> Dollars {
        programmes().values().filter_map(|p| p.expenditure).sum()
    }

    /// The channel, published and derived halves apart.
    ///
    /// # Panics
    ///
    /// If the fixture stops carrying [`DERIVED`], which would mean the report's programme list
    /// changed and the constant beside it is answering for a programme that is no longer there.
    #[must_use]
    pub fn channel() -> Channel {
        let all = programmes();
        assert!(
            all.contains_key(DERIVED),
            "the fixture no longer carries `{DERIVED}`, whose expenditure is a constant here"
        );
        let published = all.values().filter_map(|p| p.expenditure).sum();
        Channel {
            students: all.values().map(|p| p.students).sum(),
            published,
            derived: JON_PETERSON_DERIVED_EXPENDITURE,
            total: published + JON_PETERSON_DERIVED_EXPENDITURE,
            programmes: (
                all.values().filter(|p| p.expenditure.is_some()).count(),
                all.len(),
            ),
        }
    }
}

/// The six funding units R.C. 3317.022 computes and distributes state core foundation funding to.
///
/// # Why a unit is not a programme
///
/// The statute names six units and this repository holds five scholarship *programmes*, and the
/// two lists are not the same length. **The educational choice scholarship unit is one unit and
/// two programmes** — traditional EdChoice under R.C. 3310.03 and the income-based expansion
/// under R.C. 3310.032, which run concurrently under different eligibility and, since October
/// 2023, different award rules. Division (A)(10) computes them together.
///
/// So a page that puts "the units" beside "the programmes" and gets six of each has miscounted
/// somewhere, and [`units::Unit::programmes`] is the join that stops it: each unit names the report
/// slugs that sum to it, and the district and community units name none because the annual
/// report does not cover them.
///
/// # What is not a unit
///
/// Category 3 nonpublic support — Auxiliary Services, the Nonpublic Administrative Cost
/// Reimbursement and the reimbursement fund — moves under R.C. 3317.024, 3317.06, 3317.062,
/// 3317.063 and 3317.064 and under none of these divisions. See
/// [`super::ledger::nonpublic_support`], which is about a quarter of a billion dollars a year
/// that no reading of this list reaches.
pub mod units {
    /// One funding unit, as the opening of R.C. 3317.022 names it.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Unit {
        /// A stable key. Not the statute's words, which are a sentence rather than an
        /// identifier.
        pub slug: &'static str,
        /// The statute's own name for it, verbatim.
        pub name: &'static str,
        /// The division of R.C. 3317.022(A) its amount is computed under, where the opening
        /// names one. The district and community units are computed elsewhere and carry the
        /// section that computes them instead.
        pub authority: &'static str,
        /// The slugs in [`super::report::programmes`] that sum to this unit. Empty for the two
        /// units the scholarship annual report does not cover.
        pub programmes: &'static [&'static str],
    }

    /// All six, in the order the statute's opening sentence lists them.
    ///
    /// The district unit is first because the sentence puts it first, and because it is the only
    /// one of the six this site modelled until the block that reads this was written.
    pub const UNITS: [Unit; 6] = [
        Unit {
            slug: "district",
            name: "a city, local, or exempted village school district",
            authority: "R.C. 3317.022(A)",
            programmes: &[],
        },
        Unit {
            slug: "community-stem",
            name: "the community and STEM school unit",
            authority: "R.C. 3317.026",
            programmes: &[],
        },
        Unit {
            slug: "educational-choice",
            name: "the educational choice scholarship unit",
            authority: "R.C. 3317.022(A)(10)",
            programmes: &["traditional-edchoice", "edchoice-expansion"],
        },
        Unit {
            slug: "pilot-project",
            name: "the pilot project scholarship unit",
            authority: "R.C. 3317.022(A)(11)",
            programmes: &["cleveland"],
        },
        Unit {
            slug: "autism",
            name: "the autism scholarship unit",
            authority: "R.C. 3317.022(A)(12)",
            programmes: &["autism"],
        },
        Unit {
            slug: "jon-peterson",
            name: "the Jon Peterson special needs scholarship unit",
            authority: "R.C. 3317.022(A)(13)",
            programmes: &["jon-peterson"],
        },
    ];

    /// The unit with this slug.
    #[must_use]
    pub fn of(slug: &str) -> Option<Unit> {
        UNITS.into_iter().find(|unit| unit.slug == slug)
    }

    /// Where each programme is written up, keyed by the fixture's slug.
    ///
    /// The value is the corpus node's own file name under `.yidam/corpus/program/`, which is
    /// also the last segment of the address it renders at — `/wiki/program/<node>`. It is here
    /// rather than on [`Unit`] because a node documents a *programme* and the educational choice
    /// unit is two of them, so a unit-level field would have to choose one and lose the other.
    ///
    /// `the_programmes_name_the_nodes_that_document_them` keeps this list and the fixture's slugs
    /// in step; nothing in this crate can see the corpus itself, so the node names are checked by
    /// the web layer that resolves them.
    pub const NODES: [(&str, &str); 5] = [
        ("traditional-edchoice", "edchoice-scholarship"),
        ("edchoice-expansion", "edchoice-expansion"),
        ("cleveland", "cleveland-scholarship"),
        ("autism", "autism-scholarship"),
        ("jon-peterson", "jon-peterson-special-needs"),
    ];

    /// The corpus node documenting this programme.
    #[must_use]
    pub fn node(programme: &str) -> Option<&'static str> {
        NODES
            .into_iter()
            .find(|(slug, _)| *slug == programme)
            .map(|(_, node)| node)
    }
}

pub mod history {
    //! The department's archived participation series, FY1997 through FY2013.
    //!
    //! The deduct era, statewide, and the only source this repository holds that counts
    //! applications and payments as separate quantities. [`super::report`] is the same channel eleven
    //! years later under the Fair School Funding Plan; between FY2014 and FY2023 there is no
    //! series at all — only the quoted points [`super::bounds`] holds, which bound that hole and
    //! cannot be joined to either end of it.
    //!
    //! # The two measures are not two views of one number
    //!
    //! [`Measure::Applications`] counts requests and [`Measure::Used`] counts scholarships with at
    //! least one payment against them, which is the department's own phrase. The second is always
    //! the smaller and often by a great deal, so a "participants" figure taken from the wrong one
    //! overstates the channel — by 3.1x for Cleveland in FY1997.

    use std::collections::BTreeMap;
    use std::ops::RangeInclusive;

    use edfund_core::FiscalYear;

    /// The committed extract of the department's *Historical Scholarship Data* workbook.
    const FIXTURE: &str = include_str!("../fixtures/scholarship-history.csv");

    const EXPECTED_HEADER: &str = "program,fiscal_year,measure,total,new,renewal,new_low_income,\
renewal_low_income";

    /// Which quantity a row counts.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Measure {
        /// Requests made. Not a count of students served.
        Applications,
        /// Scholarships with at least one payment made against them.
        Used,
        /// Cleveland's tutoring grants, which are neither and which no other programme has.
        Tutoring,
    }

    impl Measure {
        /// The fixture's own spelling.
        fn from(cell: &str) -> Option<Self> {
            match cell {
                "applications" => Some(Self::Applications),
                "used" => Some(Self::Used),
                "tutoring" => Some(Self::Tutoring),
                _ => None,
            }
        }
    }

    /// One programme-year on one measure.
    ///
    /// Every field is optional because the workbook's coverage is ragged in both directions: the
    /// low-income split is only populated from FY2010 for Cleveland, Autism publishes a total and
    /// nothing else, and Jon Peterson has one year.
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct Counts {
        /// Everyone on this measure.
        pub total: Option<f64>,
        /// First-time recipients or applicants.
        pub new: Option<f64>,
        /// Continuing ones. See [`Observation`] for what this meant before FY2010.
        pub renewal: Option<f64>,
        /// Of the new, those qualifying as low income.
        pub new_low_income: Option<f64>,
        /// Of the renewals, those qualifying as low income.
        pub renewal_low_income: Option<f64>,
    }

    /// One row of the archive.
    ///
    /// # `renewal` changes meaning inside this series
    ///
    /// For Cleveland, applications-renewal in year *t* equals payments-total in year *t*-1
    /// exactly, for the twelve years FY1998 through FY2009 — a roll-forward of everyone paid last
    /// year rather than a count of anybody applying. From FY2010 it is counted independently and
    /// runs 10-14% below the prior year's payments. The fixture carries what the department
    /// published; the break is pinned in
    /// `crates/project/tests/the_renewal_that_was_last_years_payments.rs`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Observation {
        /// The programme slug, shared with [`super::report::programmes`] where both hold the
        /// programme.
        pub program: String,
        /// The fiscal year.
        pub year: FiscalYear,
        /// Which quantity this row counts.
        pub measure: Measure,
        /// The counts themselves.
        pub counts: Counts,
    }

    /// Every row of the archive.
    ///
    /// # Panics
    ///
    /// If the fixture's header is not the one this was written against, or if a row names a
    /// measure this module does not know — both of which mean the extractor changed shape.
    #[must_use]
    pub fn observations() -> Vec<Observation> {
        edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
            .map(|row| Observation {
                program: row.str(0).to_string(),
                year: FiscalYear(row.num(1).expect("every row names a fiscal year") as u16),
                measure: Measure::from(row.str(2)).expect("the fixture names a known measure"),
                counts: Counts {
                    total: row.num(3),
                    new: row.num(4),
                    renewal: row.num(5),
                    new_low_income: row.num(6),
                    renewal_low_income: row.num(7),
                },
            })
            .collect()
    }

    /// The fiscal years the archive covers, inclusive, read off the fixture.
    ///
    /// Derived rather than written down: this is one end of the hole
    /// [`super::bounds`] bounds, and a later edition of the workbook extending the series must move
    /// the hole with it rather than leaving a constant behind. `FY1997..=FY2013` today.
    ///
    /// # Panics
    ///
    /// If the archive holds no rows at all.
    #[must_use]
    pub fn span() -> RangeInclusive<u16> {
        let years: Vec<u16> = observations().iter().map(|o| o.year.0).collect();
        let first = years.iter().copied().min().expect("the archive has rows");
        let last = years.iter().copied().max().expect("the archive has rows");
        first..=last
    }

    /// One programme's series on one measure, by fiscal year.
    #[must_use]
    pub fn series(program: &str, measure: Measure) -> BTreeMap<FiscalYear, Counts> {
        observations()
            .into_iter()
            .filter(|o| o.program == program && o.measure == measure)
            .map(|o| (o.year, o.counts))
            .collect()
    }
}

pub mod jpsn {
    //! The Jon Peterson programme's own annual report, FY2023 through FY2025.
    //!
    //! One programme with a series of its own, because it had a report of its own. The department
    //! published a standalone JPSN annual report until the 2025 consolidated report absorbed it,
    //! and it published these two together and late: both PDFs were exported from Word on one
    //! afternoon in March 2025, nine months after FY2024 closed, and the Annual Reports page
    //! carried neither of them in the archive's June 2024 or February 2025 captures.
    //! [`dew-jpsn-annual-report`](../../.yidam/catalog/dew-jpsn-annual-report.md) holds that
    //! timeline. Those two plus the consolidated report's Jon Peterson section are FY2023, FY2024
    //! and FY2025: three consecutive years, and the first year-on-year change in this programme's
    //! participation that the department itself published anywhere.
    //!
    //! # What this closes, and what it does not
    //!
    //! [`super::history`] ends at FY2013 and [`super::report`] covers FY2025 alone. This reaches
    //! back two years further, for **one of the five programmes**. So the channel-wide
    //! participation hole is FY2014 through FY2022 rather than FY2014 through FY2023, and for Jon
    //! Peterson alone it is FY2014 through FY2022 as well — the two happen to coincide because
    //! FY2023 is this series' first year. Nothing here reaches the other four programmes, and
    //! [`super::bounds`] is still the only thing that speaks to the years in between.
    //!
    //! Each edition also prints a bar chart of total applications by fiscal year from FY2014
    //! onward, which is the series that would *fill* the hole rather than shorten it. Its bars
    //! carry no data labels in the text layer, so `pdftotext` reaches the axis and not the values,
    //! and the extractor does not attempt them.
    //!
    //! # The expenditure total is stated once and dated to another year
    //!
    //! Only the FY2023 edition writes a total in words, and it attributes it to the 2021-2022
    //! school year while reporting FY2023 throughout. [`Edition::stated_total`] and
    //! [`Edition::stated_total_year`] are separate fields for that reason, and
    //! [`Edition::total_is_its_own_year`] is the question rather than the answer. What licenses
    //! [`spending`] for the two editions that state nothing is that this one total equals the sum
    //! of its own six category figures to the cent: the department's own arithmetic, quoted.
    //!
    //! # Which figures in the FY2023 edition are FY2023's
    //!
    //! Open, and the open part is narrower than the whole edition. Its award table is identical to
    //! FY2024's, and `the_award_table_is_recomputed_each_year_and_two_editions_print_the_same_one`
    //! shows the table is a uniform fraction of the statutory vector with the fraction moving
    //! between editions — so printing one table twice means an index that did not move. Set beside
    //! the two documents having been produced in one sitting, the reading that impugns the table
    //! rather than the spending chart is as available as the reverse, and this module commits to
    //! neither: every column is carried as published.

    use std::collections::BTreeMap;
    use std::ops::RangeInclusive;

    use edfund_core::{Dollars, FiscalYear};

    /// The committed extract of the three editions, one row each.
    const FIXTURE: &str = include_str!("../fixtures/scholarship-jpsn.csv");

    const EXPECTED_HEADER: &str =
        "fiscal_year,students,providers,districts,stated_total,stated_total_fiscal_year";

    /// The same three editions' per-category series, long rather than wide.
    const CATEGORY_FIXTURE: &str = include_str!("../fixtures/scholarship-jpsn-categories.csv");

    const CATEGORY_EXPECTED_HEADER: &str = "fiscal_year,series,position,value";

    /// The programme slug, shared with the other three scholarship fixtures.
    pub const PROGRAM: &str = "jon-peterson";

    /// The six funding levels R.C. 3317.022 organizes the thirteen IDEA disability categories into.
    ///
    /// The same six [`super::JON_PETERSON_SUPPLEMENTS`] gives the statutory supplement for, which
    /// is what makes [`Series::Maximum`] comparable to the statute and the two charts not.
    pub const CATEGORIES: usize = 6;

    /// Which of the report's three per-category series a row belongs to.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Series {
        /// The maximum annual award per category, from the table that numbers its own rows.
        Maximum,
        /// The share of students in each category, in percentage points.
        Share,
        /// What was spent on each category, in dollars.
        Spending,
    }

    impl Series {
        /// The fixture's own spelling.
        fn from(cell: &str) -> Option<Self> {
            match cell {
                "maximum" => Some(Self::Maximum),
                "share" => Some(Self::Share),
                "spending" => Some(Self::Spending),
                _ => None,
            }
        }

        /// Whether a row's `position` is the category R.C. 3317.022 defines, or only a place in a
        /// chart's layout.
        ///
        /// **True for one series of the three.** The award maxima are printed in a table with a
        /// `#` column, so each amount arrives with its category stated. The two pie charts print
        /// their labels around the slices and `pdftotext` reads them in layout order, which moves
        /// between editions: the smallest disability share, near 0.7% in all three, is read first
        /// in FY2023 and fourth in both later ones. A fixed category cannot change position, so
        /// the position is not the category.
        ///
        /// What survives for the charts is the sum, which does not depend on order, and the
        /// largest value, which is identifiable by size. [`spending`] is the first of those.
        #[must_use]
        pub fn position_is_a_category(self) -> bool {
            self == Self::Maximum
        }
    }

    /// One edition of the report.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Edition {
        /// The fiscal year the edition's own July-to-June heading names.
        pub year: FiscalYear,
        /// Students who received services. The report's only participation figure in words.
        pub students: f64,
        /// Approved providers, which the report states twice and the extractor makes agree.
        pub providers: f64,
        /// Districts of residence the programme drew from. A reach figure, not a count of
        /// students, and the only column in this repository that measures it.
        pub districts: f64,
        /// The expenditure total the edition writes in words. `Some` for FY2023 alone.
        pub stated_total: Option<Dollars>,
        /// The fiscal year the edition attributes that total to, which is not its own.
        pub stated_total_year: Option<FiscalYear>,
    }

    impl Edition {
        /// Whether the stated total is attributed to this edition's own fiscal year, and `None`
        /// where the edition states no total.
        ///
        /// `Some(false)` for FY2023, the only edition that states one: it names the 2021-2022
        /// school year, two years behind the report around it. The identical phrase appears two
        /// sentences earlier in the paragraph describing House Bill 110's abolition of the
        /// district deduction, which is where a copy error would have come from — but the report
        /// does not say, and the later editions drop the sentence while keeping the chart, so
        /// nothing confirms it. The reading stays open.
        #[must_use]
        pub fn total_is_its_own_year(&self) -> Option<bool> {
            self.stated_total_year.map(|named| named == self.year)
        }

        /// What the department charts and never totals, over the students it says it served.
        ///
        /// Derived on both sides of the division from one edition, so it is the one figure here
        /// that cannot be affected by joining two editions together.
        #[must_use]
        pub fn spending_per_student(&self) -> Option<Dollars> {
            let paid = spending(self.year)?;
            (self.students > 0.0).then_some(paid / self.students)
        }
    }

    /// Every edition, by fiscal year.
    ///
    /// # Panics
    ///
    /// If the fixture's header is not the one this was written against, by way of
    /// [`edfund_core::csv::rows`], or if a row carries no fiscal year.
    #[must_use]
    pub fn editions() -> BTreeMap<FiscalYear, Edition> {
        edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
            .map(|row| {
                let year =
                    FiscalYear(row.num(0).expect("every edition names a fiscal year") as u16);
                (
                    year,
                    Edition {
                        year,
                        students: row.num(1).expect("every edition publishes participation"),
                        providers: row.num(2).expect("every edition publishes providers"),
                        districts: row.num(3).expect("every edition publishes districts"),
                        stated_total: row.num(4),
                        stated_total_year: row.num(5).map(|y| FiscalYear(y as u16)),
                    },
                )
            })
            .collect()
    }

    /// The fiscal years the editions cover, inclusive, read off the fixture.
    ///
    /// Derived rather than written down, like [`super::history::span`] and unlike
    /// [`super::report::FISCAL_YEAR`]: a fourth edition extending the series has to move the hole
    /// [`super::bounds`] describes rather than leave a constant behind. `FY2023..=FY2025` today.
    ///
    /// # Panics
    ///
    /// If the fixture holds no editions at all.
    #[must_use]
    pub fn span() -> RangeInclusive<u16> {
        let years: Vec<u16> = editions().keys().map(|y| y.0).collect();
        let first = years.iter().copied().min().expect("the fixture has rows");
        let last = years.iter().copied().max().expect("the fixture has rows");
        first..=last
    }

    /// One series of one edition, in the order the fixture holds it.
    ///
    /// For [`Series::Maximum`] that order is the category number the table prints. For the other
    /// two it is a chart's layout order and nothing more — see [`Series::position_is_a_category`].
    ///
    /// # Panics
    ///
    /// If the category fixture's header is not the one this was written against, or if a row names
    /// a series this module does not know.
    #[must_use]
    pub fn series(year: FiscalYear, which: Series) -> Vec<f64> {
        let mut found: Vec<(u8, f64)> =
            edfund_core::csv::delimited(CATEGORY_FIXTURE, CATEGORY_EXPECTED_HEADER, ',')
                .filter(|row| {
                    row.num(0).map(|y| y as u16) == Some(year.0)
                        && Series::from(row.str(1)) == Some(which)
                })
                .map(|row| {
                    (
                        row.num(2).expect("every row carries a position") as u8,
                        row.num(3).expect("every row carries a figure"),
                    )
                })
                .collect();
        found.sort_by_key(|(position, _)| *position);
        found.into_iter().map(|(_, value)| value).collect()
    }

    /// One edition's statewide expenditure, summed from the six category figures it charts.
    ///
    /// # Why summing this chart is quoting and not deriving
    ///
    /// Because the FY2023 edition does it itself. It prints the same chart and then writes "total
    /// expenditures for the JPSN program … sum to $81,773,133.70", which equals the sum of its own
    /// six figures to the cent. The department therefore treats that sum as the programme's total,
    /// and the two editions that print the chart without the sentence are not being given a total
    /// they never implied.
    ///
    /// Order-invariant, which matters: the charts' label order is a layout artefact
    /// ([`Series::position_is_a_category`]), and a sum is the reading that does not depend on it.
    ///
    /// Returns `None` for a year the fixture does not hold.
    #[must_use]
    pub fn spending(year: FiscalYear) -> Option<Dollars> {
        let categories = series(year, Series::Spending);
        (categories.len() == CATEGORIES).then(|| categories.iter().sum())
    }
}

pub mod bounds {
    //! Every dated scholarship quantity the Legislative Service Commission quotes inside the hole
    //! between the two committed series — and the reasons not one of them belongs on either.
    //!
    //! [`super::history`] runs FY1997 through FY2013 and [`super::report`] covers FY2025. Between
    //! them are nine fiscal years with no participation series at all — ten until [`super::jpsn`]
    //! was extracted, which reached FY2023 and FY2024 for one of the five programmes — and
    //! `.yidam/decisions/scholarship-reports-connector.yml` recorded that the hole "is not empty":
    //! LSC's budget analyses quote counts inside it. This is those counts, read out of the
    //! committed extracts rather than described — along with the overlap-era quotes that are the
    //! evidence for not splicing them.
    //!
    //! Unrelated to [`crate::bounds`], which is a census of the *formula's* floors and ceilings. A
    //! bound here is a constraint on a quantity nothing in this repository measures.
    //!
    //! # These are bounds and not observations
    //!
    //! Three things stop every row here from being a point on either series, and they are
    //! independent of each other:
    //!
    //! 1. **[`Precision`] — most carry the publisher's own hedge.** "About 87,000", "approximately
    //!    5,128", "over 480", "an estimated average". A hedged figure constrains a quantity; it
    //!    does not measure one.
    //! 2. **[`Denominator`] — the population differs from row to row, and is usually unstated.**
    //!    The FY2026-27 redbook counts *FTE students* for the two EdChoice programmes and
    //!    *students* for the other three, in one sequence of paragraphs. LSC counts *scholarships
    //!    awarded* for the traditional programme and *students participating* for the pilot
    //!    project. One row counts kindergarteners and one counts grades K-5. The archive's own two
    //!    denominators — applications made, and scholarships with a payment against them — are in
    //!    this vocabulary and **no row uses either**.
    //! 3. **Where a quote and the archive cover the same year, they disagree, and not by a rule.**
    //!    Five rows fall inside the archive's span, and the gaps against its payments column run
    //!    exactly 0.00% for Cleveland in FY2005, +0.98% for Cleveland in FY2012, −2.28% for the
    //!    traditional programme in the same year, and **−85%** for autism. A transcription that
    //!    reproduces the archive once and is out by a factor of seven elsewhere cannot be
    //!    extrapolated into the years where there is nothing to check it against.
    //!
    //! The third is why this is a fixture rather than a sentence.
    //! `crates/project/tests/the_points_that_bound_the_participation_hole.rs` asserts all of it,
    //! and asserts that no row is joinable — see [`Bound::spliceable`], which is the standing guard
    //! against a later row being appended to a series because it happened to look like a
    //! measurement.
    //!
    //! # Every row is checked against the document it came from
    //!
    //! [`Bound::quote`] carries the publisher's own words and [`source_text`] resolves the
    //! `source` column to the committed extract, so the test can assert that each quote appears
    //! verbatim in its source and contains the value's own numeral. The decision record's
    //! principle was that "a figure reaching the corpus through prose is a figure nothing
    //! recomputes"; this is the nearest available substitute — a transcription re-checked on every
    //! `cargo test` rather than one that was right the day it was typed.

    use edfund_core::FiscalYear;

    /// The committed census.
    const FIXTURE: &str = include_str!("../fixtures/scholarship-bounds.tsv");

    const EXPECTED_HEADER: &str =
        "program\tfiscal_year\tmeasure\tvalue\tdenominator\tprecision\tsource\tquote";

    /// Tab-delimited because [`Bound::quote`] is prose: "In FY 2014, 6,337 students…" carries both
    /// the publisher's commas and their thousands separators, and `edfund_core::csv::rows` honours
    /// no quoting.
    const DELIMITER: char = '\t';

    /// What a row counts.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Measure {
        /// Recipients, participants, or awards — whichever of the three the publisher counted.
        /// [`Denominator`] carries which.
        Participation,
        /// Cleveland's tutoring grants, the quantity
        /// [`super::history::Measure::Tutoring`] holds.
        Tutoring,
        /// A dollar total for a programme-year.
        Payments,
        /// An average award. Its denominator is never stated, which is the same defect the 2025
        /// annual report's three unreconciled averages have.
        AverageAward,
        /// The average Cleveland tutoring grant, an average over a different population again.
        AverageTutoringGrant,
        /// Chartered nonpublic schools — those participating, or, where the denominator is
        /// [`Denominator::Operating`], every one in the state.
        Schools,
        /// Special-needs providers registered to participate.
        Providers,
    }

    impl Measure {
        /// The fixture's own spelling.
        fn from(cell: &str) -> Option<Self> {
            match cell {
                "participation" => Some(Self::Participation),
                "tutoring" => Some(Self::Tutoring),
                "payments" => Some(Self::Payments),
                "average_award" => Some(Self::AverageAward),
                "average_tutoring_grant" => Some(Self::AverageTutoringGrant),
                "schools" => Some(Self::Schools),
                "providers" => Some(Self::Providers),
                _ => None,
            }
        }
    }

    /// The population a row's count is over.
    ///
    /// Nine of them across the census, and the two the department's own archive publishes are here
    /// so that a row claiming one parses and then fails [`Bound::spliceable`], rather than being
    /// unspellable and so unguarded.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Denominator {
        /// Applications made. The archive's first measure, and **no bound uses it**.
        Applications,
        /// Scholarships with at least one payment against them. The archive's second measure, and
        /// **no bound uses it either**.
        Paid,
        /// Students, uncounted as to whether they held an award all year.
        Students,
        /// Full-time equivalents. The redbook's unit for the two EdChoice programmes, and not the
        /// unit it uses for the other three in the same passage.
        Fte,
        /// Scholarships awarded — an award count, not a student count. A student holding one for
        /// part of a year is one scholarship and a fraction of an FTE.
        Scholarships,
        /// Kindergarteners only. The expansion's first year was one grade wide.
        Kindergarten,
        /// Grades K-5 only, which is how wide the expansion had grown by FY2019.
        K5,
        /// Every chartered nonpublic school in the state rather than the participating ones — the
        /// only denominator here that makes a count a denominator for other counts.
        Operating,
        /// The publisher gave none. Every dollar total and every average is here: a total has no
        /// denominator, and an average conceals one.
        Unstated,
    }

    impl Denominator {
        /// The fixture's own spelling.
        fn from(cell: &str) -> Option<Self> {
            match cell {
                "applications" => Some(Self::Applications),
                "paid" => Some(Self::Paid),
                "students" => Some(Self::Students),
                "fte" => Some(Self::Fte),
                "scholarships" => Some(Self::Scholarships),
                "kindergarten" => Some(Self::Kindergarten),
                "k5" => Some(Self::K5),
                "operating" => Some(Self::Operating),
                "unstated" => Some(Self::Unstated),
                _ => None,
            }
        }

        /// Whether this is one of the two measures [`super::history`] publishes.
        ///
        /// The archive is the series a bound would be appended to, so a bound naming one of its
        /// denominators is the one that could be spliced without a reader noticing.
        #[must_use]
        pub fn is_the_archives(self) -> bool {
            matches!(self, Self::Applications | Self::Paid)
        }
    }

    /// The publisher's own hedge, which is not a statement about rounding.
    ///
    /// [`Self::Exact`] means the sentence carries no hedge word — "totaling $405.3 million". That
    /// figure is still rounded to a tenth of a million; what it is not is qualified.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Precision {
        /// Stated without qualification.
        Exact,
        /// "About" or "approximately" — the same hedge in two words.
        About,
        /// "An estimated average", or a forward estimate in a table.
        Estimated,
        /// "Over" — one-sided, and the only precision here that constrains in a single direction.
        Over,
    }

    impl Precision {
        /// The fixture's own spelling.
        fn from(cell: &str) -> Option<Self> {
            match cell {
                "exact" => Some(Self::Exact),
                "about" => Some(Self::About),
                "estimated" => Some(Self::Estimated),
                "over" => Some(Self::Over),
                _ => None,
            }
        }

        /// Whether the publisher qualified the figure.
        #[must_use]
        pub fn is_hedged(self) -> bool {
            self != Self::Exact
        }
    }

    /// One quantity, as one document states it.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Bound {
        /// The programme slug, shared with the other two fixtures where both hold the programme.
        ///
        /// Three values name no single programme: `edchoice-combined`, for a school count LSC
        /// gives for the traditional and income-based programmes together; `nonpublic-sector`, for
        /// the state's whole chartered nonpublic sector; and `channel`, for the five programmes
        /// summed.
        pub program: String,
        /// The fiscal year the publisher dates it to.
        pub year: FiscalYear,
        /// What it counts.
        pub measure: Measure,
        /// The figure in whole units — dollars for [`Measure::Payments`] and the two averages,
        /// people or institutions otherwise.
        pub value: f64,
        /// The population it is over.
        pub denominator: Denominator,
        /// The publisher's hedge.
        pub precision: Precision,
        /// The committed extract it was read from, which [`source_text`] resolves.
        pub source: String,
        /// The publisher's own words, verbatim with line breaks collapsed.
        pub quote: String,
    }

    impl Bound {
        /// Which committed series this bound could be appended to without a reader noticing — and
        /// `None` for every row in the fixture, which is the point of having it.
        ///
        /// A bound is dangerous exactly when it *looks like* an observation on a series that
        /// already exists: the same denominator, no hedge, and a year the series reaches or is one
        /// short of. The test asserts this is `None` throughout, so a later extraction that adds a
        /// genuinely spliceable row fails the gate and forces the question instead of answering it
        /// by arriving.
        ///
        /// Deliberately not the whole argument against splicing — the module's third reason is a
        /// property of pairs of rows and no per-row predicate can express it.
        ///
        /// # Why the provider counts are not in it
        ///
        /// [`super::jpsn`] publishes an approved-provider count for each of its three years and
        /// the census quotes one for FY2024 — 454, unhedged, against the department's 500 for the
        /// same fiscal year. That pair is a **contradiction rather than a splice**: the year is
        /// occupied, so nothing can be appended to it without a reader seeing two values, and
        /// `the_two_publishers_agree_on_the_money_and_not_on_the_counts` holds the disagreement
        /// where it can be read. What makes a splice silent is an *empty* year, and this predicate
        /// is about those.
        ///
        /// It is also the case that nothing says the two count the same population — the
        /// department counts providers approved to participate and LSC's sentence counts providers
        /// students received services from — and [`Denominator::Unstated`] is the absence of a
        /// population, not one a series shares.
        #[must_use]
        pub fn spliceable(&self) -> Option<&'static str> {
            let archive = super::history::span();
            if self.denominator.is_the_archives() && self.year.0 <= archive.end() + 1 {
                return Some("the archive, whose own denominator it names");
            }
            if self.measure != Measure::Participation
                || self.denominator != Denominator::Students
                || self.precision.is_hedged()
            {
                return None;
            }
            if self.year.0 + 1 >= super::report::FISCAL_YEAR {
                return Some("the annual report, whose year and denominator it shares unhedged");
            }
            // The Jon Peterson series reaches two years further back than the consolidated report
            // does, so for that one programme an unhedged student count in FY2022 or later now has
            // a series beside it where until FY2023 was extracted there was only the hole.
            if self.program == super::jpsn::PROGRAM
                && self.year.0 + 1 >= *super::jpsn::span().start()
            {
                return Some("the Jon Peterson series, which reaches back into the hole");
            }
            None
        }

        /// Whether the archive covers this bound's year, so the two can be held against each other.
        #[must_use]
        pub fn inside_the_archive(&self) -> bool {
            super::history::span().contains(&self.year.0)
        }
    }

    /// Every row of the census, in the fixture's order — by fiscal year, then programme, then
    /// measure.
    ///
    /// # Panics
    ///
    /// If the fixture's header is not the one this was written against, or if a row names a
    /// measure, denominator or precision this module does not know — each of which means the
    /// extraction changed shape.
    #[must_use]
    pub fn census() -> Vec<Bound> {
        edfund_core::csv::delimited(FIXTURE, EXPECTED_HEADER, DELIMITER)
            .map(|row| Bound {
                program: row.str(0).to_string(),
                year: FiscalYear(row.num(1).expect("every row names a fiscal year") as u16),
                measure: Measure::from(row.str(2)).expect("the fixture names a known measure"),
                value: row.num(3).expect("every row carries a figure"),
                denominator: Denominator::from(row.str(4))
                    .expect("the fixture names a known denominator"),
                precision: Precision::from(row.str(5))
                    .expect("the fixture names a known precision"),
                source: row.str(6).to_string(),
                quote: row.str(7).to_string(),
            })
            .collect()
    }

    /// One programme's bounds on one measure, oldest first.
    #[must_use]
    pub fn of(program: &str, measure: Measure) -> Vec<Bound> {
        let mut found: Vec<Bound> = census()
            .into_iter()
            .filter(|b| b.program == program && b.measure == measure)
            .collect();
        found.sort_by_key(|b| b.year.0);
        found
    }

    /// The committed extract a `source` names, whitespace collapsed, for checking a quote against.
    ///
    /// The budget analyses of the enacted acts live inside one extract and are addressed by the
    /// record header [`crate::greenbook`] reads; the two FY2026-27 documents are extracts of their
    /// own, and `dew-redbook-table-5` names a table inside one of them rather than a second
    /// document.
    ///
    /// # Panics
    ///
    /// On a `source` naming no committed document, which means the fixture cites something this
    /// workspace does not hold.
    #[must_use]
    pub fn source_text(source: &str) -> String {
        match source {
            "dew-redbook" | "dew-redbook-table-5" => {
                flatten(crate::ledger::budget_analysis::REDBOOK)
            }
            "dew-greenbook" | "dew-greenbook-table-3" => {
                flatten(crate::ledger::budget_analysis::GREENBOOK)
            }
            record => crate::greenbook::greenbooks()
                .into_iter()
                .find(|g| g.id == record)
                .unwrap_or_else(|| panic!("no committed budget analysis is named {record}"))
                .flat(),
        }
    }

    /// Line breaks collapsed, which is what a quotation out of a PDF must be checked against.
    fn flatten(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

/// Which of the report's two averages a reading is taken on.
///
/// They differ by 1.5% to 3.4% and the report does not say why, so nothing here picks one. Every
/// finding is computed on both and is stated only where both agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Average {
    /// The figure the report prints.
    Published,
    /// Expenditure over participation.
    Implied,
}

impl Average {
    /// One programme's average on this basis.
    #[must_use]
    pub fn of(self, programme: &report::Programme) -> Option<Dollars> {
        match self {
            Self::Published => programme.published_average,
            Self::Implied => programme.implied_average(),
        }
    }
}

/// The largest share of pupils in grades K-8 consistent with an observed average award.
///
/// Both programmes pay the lesser of a tuition and a per-grade ceiling, and R.C. 3310.08(A)(2)
/// makes the expansion's own base amount *the same* ceiling. So an average award is bounded above
/// by the grade mix alone: `5,500w + 7,500(1 - w)`. Inverting it gives the most K-8-heavy mix that
/// could produce the observed figure, and any tuition that binds pushes the true share lower
/// still — so this is a ceiling on the K-8 share and not an estimate of it.
///
/// `None` where the average is outside the two ceilings, which for an average *below* the K-8
/// amount is the interesting case: see [`shortfall_no_mix_explains`].
#[must_use]
pub fn greatest_k8_share(average: Dollars) -> Option<f64> {
    ((EDCHOICE_BASE_K8..=EDCHOICE_BASE_9_12).contains(&average))
        .then(|| (EDCHOICE_BASE_9_12 - average) / (EDCHOICE_BASE_9_12 - EDCHOICE_BASE_K8))
}

/// How far an average award falls below the lowest ceiling any grade mix could produce.
///
/// The K-8 amount is the floor of the schedule: a programme paying every student the full ceiling
/// cannot average less than [`EDCHOICE_BASE_K8`] whatever its grade mix. An average below it is
/// therefore evidence that something *other than* the mix is reducing awards — the income decay,
/// a tuition below the ceiling, or both — and this is how much of it no mix can account for.
///
/// `None` where the average clears the floor, which is the ordinary case and not a finding.
#[must_use]
pub fn shortfall_no_mix_explains(average: Dollars) -> Option<Dollars> {
    (average < EDCHOICE_BASE_K8).then_some(EDCHOICE_BASE_K8 - average)
}

/// The mean decay factor implied if income decay alone explained a programme's shortfall against
/// a reference programme on the same grade mix.
///
/// **This is a reading and not a measurement**, and it is offered so that its size can be argued
/// with rather than guessed at. Three quantities move an average award — grade mix, the R.C.
/// 3310.08 decay, and a tuition below the ceiling — and two published averages cannot separate
/// three unknowns. Holding the first equal and the third absent attributes the whole gap to the
/// second, which is the largest the decay can possibly be.
#[must_use]
pub fn decay_explaining_the_whole_gap(expansion: Dollars, reference: Dollars) -> Option<f64> {
    (reference > 0.0 && expansion > 0.0).then_some(expansion / reference)
}

/// The income ratio at which R.C. 3310.08 pays `factor` of the base amount.
///
/// The inverse of the decay in [`expansion_award`], for a factor between [`MINIMUM_SHARE`] and 1.
/// Below the floor the award no longer varies with income, so no ratio is recoverable and this
/// returns `None` rather than a number past where the statute stops distinguishing families.
#[must_use]
pub fn income_paying(factor: f64) -> Option<f64> {
    ((MINIMUM_SHARE..=1.0).contains(&factor))
        .then(|| FULL_AWARD_CEILING + factor.ln() / CONSTANT_MULTIPLIER.ln())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two branches of R.C. 3310.08(B) meet, which is what fixes what the exponent means.
    #[test]
    fn the_award_is_continuous_at_four_hundred_and_fifty_per_cent() {
        let base = EDCHOICE_BASE_K8;
        assert!((expansion_award(base, FULL_AWARD_CEILING) - base).abs() < 1e-9);
        assert!((expansion_award(base, FULL_AWARD_CEILING + 1e-9) - base).abs() < 1e-4);
        assert_eq!(expansion_award(base, 1.0), base);
    }

    /// It halves for every further hundred percentage points, exactly.
    #[test]
    fn each_further_hundred_per_cent_of_poverty_halves_the_award() {
        let base = EDCHOICE_BASE_9_12;
        for step in 0..3 {
            let here = expansion_award(base, FULL_AWARD_CEILING + f64::from(step));
            let next = expansion_award(base, FULL_AWARD_CEILING + f64::from(step) + 1.0);
            assert!(
                (next / here - CONSTANT_MULTIPLIER).abs() < 1e-9,
                "at {step} steps the award falls by {:.6}, not by half",
                next / here
            );
        }
    }

    #[test]
    fn the_floor_is_a_tenth_of_the_base_and_binds_at_seven_hundred_and_eighty_two_per_cent() {
        let threshold = minimum_award_threshold();
        assert!((threshold - 7.821_928_094_887_362).abs() < 1e-12);
        for base in [EDCHOICE_BASE_K8, EDCHOICE_BASE_9_12] {
            let floor = base * MINIMUM_SHARE;
            assert!((expansion_award(base, threshold) - floor).abs() < 1e-9);
            assert!((expansion_award(base, 50.0) - floor).abs() < 1e-9);
            // Just inside the threshold the award is still above the floor, so the floor is a
            // floor rather than a second regime.
            assert!(expansion_award(base, threshold - 0.01) > floor);
        }
        assert!((EDCHOICE_BASE_K8 * MINIMUM_SHARE - 550.0).abs() < 1e-9);
        assert!((EDCHOICE_BASE_9_12 * MINIMUM_SHARE - 750.0).abs() < 1e-9);
    }

    /// The award never rises with income, which a formula written as an exponential of a
    /// negative log could get wrong by a sign and still look plausible.
    #[test]
    fn the_award_never_rises_with_income() {
        let mut previous = f64::MAX;
        for step in 0..200 {
            let award = expansion_award(EDCHOICE_BASE_K8, f64::from(step) / 20.0);
            assert!(award <= previous + 1e-9, "the award rose at {step}");
            previous = award;
        }
    }

    /// The Jon Peterson formula exceeds its own ceiling for the most severely disabled category.
    #[test]
    fn the_sixth_category_is_capped_by_a_ceiling_that_is_not_indexed() {
        let uncapped: Vec<Dollars> = JON_PETERSON_SUPPLEMENTS
            .iter()
            .map(|supplement| JON_PETERSON_BASE + supplement)
            .collect();
        assert_eq!(
            uncapped
                .iter()
                .filter(|a| **a > JON_PETERSON_CEILING)
                .count(),
            1,
            "exactly one category's computed award exceeds the ceiling"
        );
        assert!((uncapped[5] - 39_122.0).abs() < 1e-9);
        assert!((uncapped[5] - JON_PETERSON_CEILING - 5_122.0).abs() < 1e-9);
        assert!((jon_peterson_award(6) - JON_PETERSON_CEILING).abs() < 1e-9);

        // Category five is the next to reach it, and it is not far.
        assert!((uncapped[4] - 29_750.0).abs() < 1e-9);
        assert!(JON_PETERSON_CEILING - uncapped[4] < 5_000.0);

        // The two ceilings are the same figure in two divisions.
        assert!((JON_PETERSON_CEILING - AUTISM_CEILING).abs() < f64::EPSILON);
    }

    #[test]
    fn the_categories_rise_and_there_are_six_of_them() {
        assert!(JON_PETERSON_SUPPLEMENTS.windows(2).all(|w| w[1] > w[0]));
        for category in 1..=6 {
            assert!(jon_peterson_award(category) > 0.0);
        }
        assert!(std::panic::catch_unwind(|| jon_peterson_award(7)).is_err());
        assert!(std::panic::catch_unwind(|| jon_peterson_award(0)).is_err());
    }
}
