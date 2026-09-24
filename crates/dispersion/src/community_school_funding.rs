//! What Ohio pays its community and STEM schools, and the one line no district receives.
//!
//! [`super::community_schools`] holds this population as a *history* — who opened, who never
//! did, who lasted — from the federal directory. It holds no money. This module is the money, from
//! the department's own FY2027 simulator, and it exists for one line in particular: the
//! **community school equity supplement** of R.C. 3317.022.
//!
//! # Why this is here and not in a calculator crate
//!
//! Because nothing in them can join it. A community school has no IRN in the 609-district panel,
//! no assessed valuation, no charge-off and no local share — it has no taxing authority, so its
//! state share is 100% by construction rather than by formula. `project::policy` prices eight
//! levers over that panel and this population intersects it in nothing, which is the whole of why
//! the supplement is a reported figure here rather than a ninth lever there. The record is
//! `decisions/a-supplement-paid-to-a-population-the-panel-does-not-hold`.
//!
//! # Who is paid, which is not everybody in the file
//!
//! Of the 355 schools the department models, **324 site-based community schools receive the
//! supplement and 31 do not** — 23 e-schools and 8 STEM schools, paid exactly zero against 35,501
//! and 4,586 enrolled ADM between them. [`Designation`] carries that distinction because the
//! money cannot: a school paid nothing and a school absent from the file look identical in a
//! column of dollars.
//!
//! **The published analysis of the introduced budget says otherwise, and it is describing a
//! different bill.** The redbook records that the executive proposal "expands eligibility to STEM
//! schools". The enacted act did not: item 10 of its own list pays community schools "that are not
//! internet- or computer-based", the greenbook says "site-based community schools", and the
//! department's model pays the eight STEM schools nothing. The corpus reads the *enacted* rule,
//! and `connect::fixtures::community_schools` fails the rebuild if a future file departs from it.
//!
//! # The rate is a schedule, and the middle year of it will never have a model behind it
//!
//! [`EQUITY_RATES`] is three years and two authorities. FY2027's $400 and FY2025's $650 are each
//! verified against the department's own model for that year: both workbooks write the formula
//! out — `Equity Supplement [a*$400]`, `[a*$650]` — and pay every recipient that rate times its
//! enrolled ADM to the cent, on 324 schools and on 317.
//!
//! **FY2026's $500 is read off the act and stays that way.** Not "not yet sought": the department
//! publishes one community school calculator at a time and replaces it in place, and the FY2026
//! edition was replaced before any archive took a copy. The Internet Archive holds exactly two of
//! these workbooks across the whole `State-Funding-For-Schools` tree — FY2019 and FY2025 — the
//! archived captures of the community school page link no calculator in any month of 2025, and
//! the FY2027 workbook, which carries the previous year's sheet as a hidden leftover, states no
//! $500 anywhere in it. The department's FY2026 actuals are published, but as school totals with
//! no line breakout, so they cannot separate this line out either. A reader should stop looking.
//!
//! [`EquityRate::verified_against_a_model`] carries that distinction as data rather than as a
//! footnote, because $650, $500 and $400 are otherwise indistinguishable in a list.
//!
//! # Two vintages, and what the second one is for
//!
//! [`Vintage`] is FY2025 and FY2027, and everything that reads schools takes one. They are not a
//! series: two years two apart, over a population that opened and closed schools between them,
//! under a formula whose phase-in moved from 0.6667 to 1 and which gained a line in between. The
//! FY2025 model is held to verify a rate, and the FY2027 model is what the corpus's figures are
//! computed from. A caller differencing them is measuring four changes at once.

use std::collections::BTreeMap;

const FY27_FIXTURE: &str = include_str!("../fixtures/fy27-community-school-funding.csv");
const FY25_FIXTURE: &str = include_str!("../fixtures/fy25-community-school-funding.csv");

const EXPECTED_HEADER: &str = "irn,school,county,designation,enrolled_adm,base_cost,\
    special_education,dpia,english_learners,career_technical,core_foundation_funding,\
    transportation,equity_supplement,formula_transition_supplement,base_funding_supplement,\
    facilities,total_state_support";

/// How many columns a row of the extract has.
const WIDTH: usize = 17;

/// The fiscal year the corpus's community school figures come from. A projection: FY2027 has not
/// happened.
pub const FISCAL_YEAR: u16 = Vintage::CURRENT.fiscal_year();

/// Which year's model a reader is asking for.
///
/// Two, and they are not a series — see the module note. The department publishes one of these a
/// year and replaces the previous one in place, so which years exist here is a fact about what
/// was archived rather than about what was published.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Vintage {
    /// FY2025, the last year of the $650 rate H.B. 33 set. From the Internet Archive.
    Fy2025,
    /// FY2027, at $400. What the corpus's figures are computed from.
    Fy2027,
}

impl Vintage {
    /// The year the corpus's headline community school figures come from.
    pub const CURRENT: Self = Self::Fy2027;

    /// Both, oldest first.
    pub const ALL: [Self; 2] = [Self::Fy2025, Self::Fy2027];

    /// The fiscal year it models.
    #[must_use]
    pub const fn fiscal_year(self) -> u16 {
        match self {
            Self::Fy2025 => 2025,
            Self::Fy2027 => 2027,
        }
    }

    /// The committed extract.
    const fn fixture(self) -> &'static str {
        match self {
            Self::Fy2025 => FY25_FIXTURE,
            Self::Fy2027 => FY27_FIXTURE,
        }
    }
}

/// What kind of school the department is funding, which decides whether it is paid the supplement.
///
/// Three values from one published column. The department writes an `E-School Designation` of
/// `N`, `Y` or `STEM`, which answers two questions at once: `Y` and `STEM` are both "not a
/// site-based community school" and they are not each other. A STEM school is a creature of
/// R.C. 3326 that this formula funds alongside community schools rather than a kind of one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Designation {
    /// A site-based community school. The 324 the supplement is paid to.
    SiteBased,
    /// An internet- or computer-based community school, which the act names and excludes.
    ESchool,
    /// A STEM school under R.C. 3326, excluded because it is not a community school at all.
    Stem,
}

impl Designation {
    /// Read the extract's spelling.
    ///
    /// Named `parse` rather than `from_str` because `FromStr` is the wrong shape here: it would
    /// have to name an error type for a column with exactly three legal values, and the caller
    /// wants the absent case, not a description of it.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "site-based" => Some(Self::SiteBased),
            "e-school" => Some(Self::ESchool),
            "stem" => Some(Self::Stem),
            _ => None,
        }
    }

    /// A short name for a table or a legend.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::SiteBased => "Site-based community school",
            Self::ESchool => "E-school",
            Self::Stem => "STEM school",
        }
    }

    /// Whether R.C. 3317.022, as H.B. 96 enacted it, pays this kind of school the supplement.
    #[must_use]
    pub const fn receives_equity_supplement(self) -> bool {
        matches!(self, Self::SiteBased)
    }

    /// The three, in the order the department's own column distinguishes them.
    pub const ALL: [Self; 3] = [Self::SiteBased, Self::ESchool, Self::Stem];
}

/// One year of the supplement's per-pupil rate, and what stands behind it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquityRate {
    /// The fiscal year.
    pub fiscal_year: u16,
    /// Dollars per pupil of enrolled ADM.
    pub per_pupil: f64,
    /// Whether a department funding model for that year has been retrieved and checked against
    /// this rate, as opposed to the rate being read off the act.
    ///
    /// FY2025 and FY2027 are; FY2026 is not and will not be — see the module note for what was
    /// searched. The distinction is carried rather than noted because $650, $500 and $400 look
    /// equally solid in a list and are not equally solid.
    pub verified_against_a_model: bool,
}

/// The supplement's rate in each year it has had one.
///
/// H.B. 33 established it at $650 in FY2024 and FY2025; H.B. 96 codified it into R.C. 3317.022
/// and cut it to $500 then $400. FY2024 is not listed: this corpus has retrieved no source
/// stating that year's payments, and the greenbook sentence the other three come from names
/// FY2025 as its baseline.
///
/// Two of the three are verified against the department's own model for the year. The middle one
/// is not and cannot be: the FY2026 calculator was replaced in place before any archive copied
/// it. That is a closed question, not an open one.
pub const EQUITY_RATES: [EquityRate; 3] = [
    EquityRate {
        fiscal_year: 2025,
        per_pupil: 650.0,
        verified_against_a_model: true,
    },
    EquityRate {
        fiscal_year: 2026,
        per_pupil: 500.0,
        verified_against_a_model: false,
    },
    EquityRate {
        fiscal_year: 2027,
        per_pupil: 400.0,
        verified_against_a_model: true,
    },
];

/// The rate in one fiscal year, or `None` in a year the supplement had none.
#[must_use]
pub fn equity_rate(fiscal_year: u16) -> Option<f64> {
    EQUITY_RATES
        .iter()
        .find(|rate| rate.fiscal_year == fiscal_year)
        .map(|rate| rate.per_pupil)
}

/// One school as the department's model for a [`Vintage`] funds it.
///
/// Every money field is dollars for the year. There is no local share and no state share
/// percentage: both would be constants.
#[derive(Debug, Clone, PartialEq)]
pub struct School {
    /// The school's IRN. The only unique key — school names are not, and neither are sponsors.
    pub irn: String,
    /// Its name as the simulator writes it.
    pub name: String,
    /// The county the simulator attributes it to. Empty for the handful it leaves blank.
    pub county: String,
    /// Site-based, e-school or STEM.
    pub designation: Designation,
    /// Enrolled ADM, projected. The base every per-pupil line here multiplies.
    pub enrolled_adm: f64,
    /// Aggregate base cost.
    pub base_cost: f64,
    /// Special education, all six categories.
    pub special_education: f64,
    /// Disadvantaged pupil impact aid.
    pub dpia: f64,
    /// English learner funding.
    pub english_learners: f64,
    /// Career-technical education.
    pub career_technical: f64,
    /// Core foundation funding: the five above, after the phase-in.
    pub core_foundation_funding: f64,
    /// Transportation.
    pub transportation: f64,
    /// The equity supplement. Zero for every e-school and STEM school.
    pub equity_supplement: f64,
    /// The formula transition supplement, against an FY2021 base. Eight schools receive it.
    pub formula_transition_supplement: f64,
    /// The base funding supplement, $40 a pupil — paid to **all** 355, e-schools included.
    ///
    /// `None` in FY2025, where the line did not exist: H.B. 96 created it. Not zero, because a
    /// school paid nothing and a payment that had not been legislated are different facts, and
    /// only the second is true of that year. A caller summing this across vintages has to say
    /// which it means.
    pub base_funding_supplement: Option<f64>,
    /// Facilities funding.
    pub facilities: f64,
    /// Total state support: everything above, as the department totals it.
    pub total_state_support: f64,
}

impl School {
    /// The supplement this school would be paid at `rate`, whatever it is actually paid.
    ///
    /// Eligibility first: a rate times an e-school's ADM is a number the statute does not produce,
    /// and returning it unmarked is how a counterfactual becomes a claim.
    #[must_use]
    pub fn supplement_at(&self, rate: f64) -> f64 {
        if self.designation.receives_equity_supplement() {
            self.enrolled_adm * rate
        } else {
            0.0
        }
    }
}

/// Every school in the department's model for `vintage`.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, a row is not the full width,
/// or a designation is outside the department's three — each of which means the extract is not
/// the file this reader expects.
#[must_use]
pub fn schools(vintage: Vintage) -> Vec<School> {
    let mut lines = vintage.fixture().lines();
    let header = lines.next().unwrap_or_default().trim();
    assert_eq!(
        header, EXPECTED_HEADER,
        "the community school funding fixture's columns have moved"
    );
    lines
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let cells: Vec<&str> = line.split(',').collect();
            assert_eq!(
                cells.len(),
                WIDTH,
                "the community school funding fixture is uniform-width; this row is not: {line}"
            );
            let dollars = |index: usize| cells[index].trim().parse::<f64>().unwrap_or(0.0);
            // An empty cell is a line the year did not have. Every other column is present in
            // both vintages, so this distinction is only ever drawn where it is true.
            let optional = |index: usize| cells[index].trim().parse::<f64>().ok();
            School {
                irn: cells[0].trim().to_string(),
                name: cells[1].trim().to_string(),
                county: cells[2].trim().to_string(),
                designation: Designation::parse(cells[3]).unwrap_or_else(|| {
                    panic!(
                        "the community school funding fixture gives {} the designation {:?}",
                        cells[0], cells[3]
                    )
                }),
                enrolled_adm: dollars(4),
                base_cost: dollars(5),
                special_education: dollars(6),
                dpia: dollars(7),
                english_learners: dollars(8),
                career_technical: dollars(9),
                core_foundation_funding: dollars(10),
                transportation: dollars(11),
                equity_supplement: dollars(12),
                formula_transition_supplement: dollars(13),
                base_funding_supplement: optional(14),
                facilities: dollars(15),
                total_state_support: dollars(16),
            }
        })
        .collect()
}

/// The model, by IRN. School names are not unique; IRNs are.
#[must_use]
pub fn by_irn(vintage: Vintage) -> BTreeMap<String, School> {
    schools(vintage)
        .into_iter()
        .map(|school| (school.irn.clone(), school))
        .collect()
}

/// The schools the supplement is actually paid to.
#[must_use]
pub fn recipients(vintage: Vintage) -> Vec<School> {
    schools(vintage)
        .into_iter()
        .filter(|school| school.equity_supplement > 0.0)
        .collect()
}

/// What the supplement costs, and what it would cost under the rules it was nearly given.
///
/// The counterfactuals are here rather than left to a caller because each of them needs the
/// eligibility rule applied before the arithmetic, and that is exactly the step a caller
/// multiplying a total ADM by a rate would skip.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquityCost {
    /// What the department's model pays: the enacted rule at that year's enacted rate.
    pub enacted: f64,
    /// The same recipients at the $650 H.B. 33 set, which H.B. 96 cut.
    ///
    /// In FY2025 this equals [`Self::enacted`], because $650 *is* that year's rate. That equality
    /// is not a degenerate case to be skipped over: it is the check that the rate H.B. 33 set is
    /// the rate the department paid, computed over 317 schools one at a time.
    pub at_the_former_rate: f64,
    /// What extending eligibility to STEM schools would add at the enacted rate — the change the
    /// redbook attributes to the executive proposal and the act did not make.
    pub extending_to_stem: f64,
    /// And to e-schools, which no version of this budget proposed. Here because "not extended to
    /// STEM" and "not extended to anyone" are different statements and the first is the true one.
    pub extending_to_e_schools: f64,
}

/// The supplement's cost in one vintage, with the two rules it was measured against.
#[must_use]
pub fn equity_cost(vintage: Vintage) -> EquityCost {
    let all = schools(vintage);
    let paid = |designation: Designation, rate: f64| -> f64 {
        all.iter()
            .filter(|school| school.designation == designation)
            .map(|school| school.enrolled_adm * rate)
            .sum()
    };
    let enacted_rate = equity_rate(vintage.fiscal_year()).unwrap_or_default();
    let former_rate = equity_rate(2025).unwrap_or_default();
    EquityCost {
        enacted: all.iter().map(|school| school.equity_supplement).sum(),
        at_the_former_rate: paid(Designation::SiteBased, former_rate),
        extending_to_stem: paid(Designation::Stem, enacted_rate),
        extending_to_e_schools: paid(Designation::ESchool, enacted_rate),
    }
}

/// One designation's share of the model: how many schools, how many pupils, how much money.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    /// Which kind of school.
    pub designation: Designation,
    /// How many of them.
    pub schools: usize,
    /// Their enrolled ADM.
    pub enrolled_adm: f64,
    /// Their equity supplement, which is zero for two of the three.
    pub equity_supplement: f64,
    /// Their total state support.
    pub total_state_support: f64,
}

/// The model split the way the supplement splits it.
#[must_use]
pub fn segments(vintage: Vintage) -> Vec<Segment> {
    let all = schools(vintage);
    Designation::ALL
        .iter()
        .map(|designation| {
            let group: Vec<&School> = all
                .iter()
                .filter(|school| school.designation == *designation)
                .collect();
            Segment {
                designation: *designation,
                schools: group.len(),
                enrolled_adm: group.iter().map(|s| s.enrolled_adm).sum(),
                equity_supplement: group.iter().map(|s| s.equity_supplement).sum(),
                total_state_support: group.iter().map(|s| s.total_state_support).sum(),
            }
        })
        .collect()
}

/// Total state support across every school in the model.
#[must_use]
pub fn total_state_support(vintage: Vintage) -> f64 {
    schools(vintage).iter().map(|s| s.total_state_support).sum()
}

/// The supplement as a share of what the state pays these schools altogether.
#[must_use]
pub fn equity_share_of_state_support(vintage: Vintage) -> f64 {
    let total = total_state_support(vintage);
    if total == 0.0 {
        return 0.0;
    }
    equity_cost(vintage).enacted / total
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cent, for the equalities the department's own arithmetic makes exact.
    const CENT: f64 = 0.01;

    /// What each vintage models, as a population. Not a series: schools opened and closed in
    /// between, so the two counts are two facts rather than a trend.
    const POPULATIONS: [(Vintage, usize, usize, usize, usize); 2] = [
        (Vintage::Fy2025, 343, 317, 18, 8),
        (Vintage::Fy2027, 355, 324, 23, 8),
    ];

    fn segment(vintage: Vintage, designation: Designation) -> Segment {
        *segments(vintage)
            .iter()
            .find(|s| s.designation == designation)
            .expect("every designation is a segment")
    }

    #[test]
    fn every_school_the_department_models_is_read() {
        for (vintage, total, ..) in POPULATIONS {
            let all = schools(vintage);
            assert_eq!(all.len(), total, "{vintage:?} models {total} schools");
            assert_eq!(
                by_irn(vintage).len(),
                all.len(),
                "and each of them once, keyed on an IRN that is unique where the name is not"
            );
            assert!(
                !all.iter().any(|s| s.name.contains("State of Ohio")),
                "the department's aggregate row is not a school"
            );
        }
    }

    #[test]
    fn the_supplement_is_paid_to_the_site_based_schools_and_to_nobody_else() {
        for (vintage, _, site_based, ..) in POPULATIONS {
            for school in schools(vintage) {
                if school.designation.receives_equity_supplement() {
                    assert!(
                        school.equity_supplement > 0.0,
                        "{} is site-based and paid nothing in {vintage:?}",
                        school.irn
                    );
                } else {
                    assert_eq!(
                        school.equity_supplement,
                        0.0,
                        "{} is {} and was paid {} in {vintage:?}",
                        school.irn,
                        school.designation.label(),
                        school.equity_supplement
                    );
                }
            }
            assert_eq!(recipients(vintage).len(), site_based);
        }
    }

    #[test]
    fn the_schools_that_are_not_paid_are_the_e_schools_and_the_stem_schools_in_both_years() {
        // The finding the redbook contradicts, held twice. Two years apart, under two rates, and
        // across a $150 cut: the eligibility rule is the one thing about this line that did not
        // move, which is what makes "the cut is the rate, not the population" a measurement
        // rather than a reading of one file.
        for (vintage, _, site, e, stem) in POPULATIONS {
            let (site_based, e_school, stem_school) = (
                segment(vintage, Designation::SiteBased),
                segment(vintage, Designation::ESchool),
                segment(vintage, Designation::Stem),
            );
            assert_eq!(
                (site_based.schools, e_school.schools, stem_school.schools),
                (site, e, stem),
                "{vintage:?}"
            );
            assert_eq!(e_school.equity_supplement, 0.0, "{vintage:?}");
            assert_eq!(stem_school.equity_supplement, 0.0, "{vintage:?}");
            assert!(
                stem_school.enrolled_adm > 2_000.0,
                "and they are not paid nothing because they are empty: {} ADM in {vintage:?}",
                stem_school.enrolled_adm
            );
        }
    }

    #[test]
    fn every_recipient_is_paid_the_published_rate_times_its_enrolled_adm() {
        // The same identity the extractor enforces against the workbook, held here against the
        // committed files — so a fixture edited by hand fails as loudly as a reposted workbook.
        for vintage in Vintage::ALL {
            let year = vintage.fiscal_year();
            let rate = equity_rate(year).expect("both vintages are years with a rate");
            for school in recipients(vintage) {
                assert!(
                    (school.equity_supplement - school.enrolled_adm * rate).abs() <= CENT,
                    "{} is paid {} against {} ADM at {rate} in FY{year}",
                    school.irn,
                    school.equity_supplement,
                    school.enrolled_adm
                );
            }
        }
    }

    #[test]
    fn the_two_years_with_a_model_behind_them_are_the_two_that_claim_one() {
        // FY2026 is the gap and it is a permanent one. The department publishes one of these
        // calculators at a time and replaces it in place; the FY2026 edition was replaced before
        // any archive took a copy, and its rate is the one number in this schedule that no
        // retrievable departmental document states per school.
        let verified: Vec<u16> = EQUITY_RATES
            .iter()
            .filter(|rate| rate.verified_against_a_model)
            .map(|rate| rate.fiscal_year)
            .collect();
        assert_eq!(verified, [2025, 2027]);
        assert_eq!(
            verified,
            Vintage::ALL.map(Vintage::fiscal_year).to_vec(),
            "a year claims a model exactly when a vintage of it is held"
        );
        assert_eq!(equity_rate(2027), Some(400.0));
        assert_eq!(equity_rate(2026), Some(500.0));
        assert_eq!(equity_rate(2025), Some(650.0));
        assert_eq!(equity_rate(2024), None, "H.B. 33's first year is not held");
    }

    #[test]
    fn the_fy2025_model_is_what_verifies_the_rate_h_b_33_set() {
        // $650 was carried from the act's own sentence until this file was retrieved. The
        // department's FY2025 workbook pays it school by school, so the counterfactual at $650
        // and the enacted cost are the same number — and that identity is the verification.
        let cost = equity_cost(Vintage::Fy2025);
        // To the rounding and no closer: `enacted` sums 317 payments the extract stores in cents,
        // and `at_the_former_rate` multiplies each unrounded ADM by $650. Half a cent a school is
        // $1.59, so a dollar and a half is the tolerance the comparison actually has.
        assert!(
            (cost.enacted - cost.at_the_former_rate).abs() <= 1.59,
            "FY2025's enacted rate is the $650 rate: {cost:?}"
        );
        assert!(
            (cost.enacted - 54_846_889.80).abs() <= 1.0,
            "and the department's own statewide total: {}",
            cost.enacted
        );
    }

    #[test]
    fn a_counterfactual_applies_the_eligibility_rule_before_the_rate() {
        // `supplement_at` is not a multiplication. An e-school's ADM times a rate is a number the
        // statute does not produce, and the guard is the difference between a counterfactual and
        // a claim.
        let all = by_irn(Vintage::CURRENT);
        let e_school = all
            .values()
            .find(|s| s.designation == Designation::ESchool)
            .expect("the model carries e-schools");
        assert!(e_school.enrolled_adm > 0.0);
        assert_eq!(e_school.supplement_at(400.0), 0.0);

        let site = all
            .values()
            .find(|s| s.designation == Designation::SiteBased)
            .expect("the model carries site-based schools");
        assert!((site.supplement_at(400.0) - site.equity_supplement).abs() <= CENT);
    }

    #[test]
    fn the_rate_cut_is_most_of_what_separates_the_enacted_supplement_from_the_introduced_one() {
        // The redbook estimates $58.0m a year for the executive proposal, which kept $650 and
        // added STEM schools. The act cut the rate and did not add them. Both halves are
        // computable from this one file, and the rate is much the larger.
        let cost = equity_cost(Vintage::CURRENT);
        assert!((cost.enacted - 34_650_906.27).abs() <= CENT, "{cost:?}");

        let from_the_rate = cost.at_the_former_rate - cost.enacted;
        assert!(
            (from_the_rate - 21_656_816.41).abs() <= CENT,
            "the cut from $650 to $400 over the same recipients: {from_the_rate}"
        );
        assert!(
            (cost.extending_to_stem - 1_834_208.52).abs() <= CENT,
            "and extending to the eight STEM schools at the enacted rate: {}",
            cost.extending_to_stem
        );
        assert!(
            from_the_rate > cost.extending_to_stem * 10.0,
            "the rate is the larger term by an order of magnitude"
        );
    }

    #[test]
    fn the_supplement_is_a_small_share_of_what_these_schools_are_paid() {
        let share = equity_share_of_state_support(Vintage::CURRENT);
        assert!(
            (0.0228..0.0231).contains(&share),
            "2.29% of total state support: {share}"
        );
        assert!((total_state_support(Vintage::CURRENT) - 1_511_789_771.13).abs() <= 1.0);
    }

    #[test]
    fn the_base_funding_supplement_is_paid_to_everyone_which_the_equity_supplement_is_not() {
        // The contrast that stops "STEM schools are funded differently" becoming a general
        // property of the file. Two per-pupil lines, one eligibility rule between them.
        for school in schools(Vintage::Fy2027) {
            assert!(
                school.base_funding_supplement.unwrap_or_default() > 0.0,
                "{} is {} and receives no base funding supplement",
                school.irn,
                school.designation.label()
            );
        }
    }

    #[test]
    fn a_line_the_year_did_not_have_reads_as_absent_rather_than_as_zero() {
        // H.B. 96 created the base funding supplement. FY2025 has no such column, and `None` is
        // what says so — a zero would be the department having computed the line and paid
        // nothing, which is the claim a reader summing the column would come away with.
        assert!(schools(Vintage::Fy2025)
            .iter()
            .all(|school| school.base_funding_supplement.is_none()));
        assert!(schools(Vintage::Fy2027)
            .iter()
            .all(|school| school.base_funding_supplement.is_some()));
    }
}
