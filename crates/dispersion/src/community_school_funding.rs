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
//! # The rate is a schedule, and only one year of it has a model behind it
//!
//! [`EQUITY_RATES`] is three years and two authorities. FY2027's $400 is verified against this
//! workbook, which states the rate in a header cell and pays every recipient that rate times its
//! enrolled ADM to the cent. FY2026's $500 and FY2025's $650 come from the enacted act and the
//! greenbook, which state all three in one sentence each; no department model for either year has
//! been retrieved. [`EquityRate::verified_against_a_model`] is the difference, and it is carried
//! as data rather than as a footnote because the three numbers are otherwise indistinguishable.

use std::collections::BTreeMap;

const FIXTURE: &str = include_str!("../fixtures/fy27-community-school-funding.csv");

const EXPECTED_HEADER: &str = "irn,school,county,designation,enrolled_adm,base_cost,\
    special_education,dpia,english_learners,career_technical,core_foundation_funding,\
    transportation,equity_supplement,formula_transition_supplement,base_funding_supplement,\
    facilities,total_state_support";

/// How many columns a row of the extract has.
const WIDTH: usize = 17;

/// The fiscal year the extract models. A projection: FY2027 has not happened.
pub const FISCAL_YEAR: u16 = 2027;

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
    /// Only FY2027 is. The distinction is carried rather than noted because $650, $500 and $400
    /// look equally solid in a list and are not equally solid.
    pub verified_against_a_model: bool,
}

/// The supplement's rate in each year it has had one.
///
/// H.B. 33 established it at $650 in FY2024 and FY2025; H.B. 96 codified it into R.C. 3317.022
/// and cut it to $500 then $400. FY2024 is not listed: this corpus has retrieved no source
/// stating that year's payments, and the greenbook sentence the other three come from names
/// FY2025 as its baseline.
pub const EQUITY_RATES: [EquityRate; 3] = [
    EquityRate {
        fiscal_year: 2025,
        per_pupil: 650.0,
        verified_against_a_model: false,
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

/// One school as the department's FY2027 model funds it.
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
    pub base_funding_supplement: f64,
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

/// Every school in the department's FY2027 model.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, a row is not the full width,
/// or a designation is outside the department's three — each of which means the extract is not
/// the file this reader expects.
#[must_use]
pub fn schools() -> Vec<School> {
    let mut lines = FIXTURE.lines();
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
                base_funding_supplement: dollars(14),
                facilities: dollars(15),
                total_state_support: dollars(16),
            }
        })
        .collect()
}

/// The model, by IRN. School names are not unique; IRNs are.
#[must_use]
pub fn by_irn() -> BTreeMap<String, School> {
    schools()
        .into_iter()
        .map(|school| (school.irn.clone(), school))
        .collect()
}

/// The schools the supplement is actually paid to.
#[must_use]
pub fn recipients() -> Vec<School> {
    schools()
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
    /// What the department's model pays: the enacted rule at the enacted FY2027 rate.
    pub enacted: f64,
    /// The same recipients at the $650 H.B. 33 set, which H.B. 96 cut.
    pub at_the_former_rate: f64,
    /// What extending eligibility to STEM schools would add at the enacted rate — the change the
    /// redbook attributes to the executive proposal and the act did not make.
    pub extending_to_stem: f64,
    /// And to e-schools, which no version of this budget proposed. Here because "not extended to
    /// STEM" and "not extended to anyone" are different statements and the first is the true one.
    pub extending_to_e_schools: f64,
}

/// The supplement's cost in FY2027, with the two rules it was measured against.
#[must_use]
pub fn equity_cost() -> EquityCost {
    let all = schools();
    let paid = |designation: Designation, rate: f64| -> f64 {
        all.iter()
            .filter(|school| school.designation == designation)
            .map(|school| school.enrolled_adm * rate)
            .sum()
    };
    let enacted_rate = equity_rate(FISCAL_YEAR).unwrap_or_default();
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
pub fn segments() -> Vec<Segment> {
    let all = schools();
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
pub fn total_state_support() -> f64 {
    schools().iter().map(|s| s.total_state_support).sum()
}

/// The supplement as a share of what the state pays these schools altogether.
#[must_use]
pub fn equity_share_of_state_support() -> f64 {
    let total = total_state_support();
    if total == 0.0 {
        return 0.0;
    }
    equity_cost().enacted / total
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cent, for the equalities the department's own arithmetic makes exact.
    const CENT: f64 = 0.01;

    #[test]
    fn every_school_the_department_models_is_read() {
        let all = schools();
        assert_eq!(all.len(), 355, "the FY2027 simulator models 355 schools");
        assert_eq!(
            by_irn().len(),
            all.len(),
            "and each of them once, keyed on an IRN that is unique where the name is not"
        );
        assert!(
            !all.iter().any(|s| s.name.contains("State of Ohio")),
            "the department's aggregate row is not a school"
        );
    }

    #[test]
    fn the_supplement_is_paid_to_the_site_based_schools_and_to_nobody_else() {
        for school in schools() {
            if school.designation.receives_equity_supplement() {
                assert!(
                    school.equity_supplement > 0.0,
                    "{} is site-based and paid nothing",
                    school.irn
                );
            } else {
                assert_eq!(
                    school.equity_supplement,
                    0.0,
                    "{} is {} and was paid {}",
                    school.irn,
                    school.designation.label(),
                    school.equity_supplement
                );
            }
        }
        assert_eq!(recipients().len(), 324);
    }

    #[test]
    fn the_thirty_one_that_are_not_paid_are_the_e_schools_and_the_stem_schools() {
        // The finding the redbook contradicts. Both counts, because "not paid" is 31 schools and
        // the interesting half of it is the 8.
        let segments = segments();
        let find = |designation: Designation| {
            *segments
                .iter()
                .find(|s| s.designation == designation)
                .expect("every designation is a segment")
        };
        let (site, e_school, stem) = (
            find(Designation::SiteBased),
            find(Designation::ESchool),
            find(Designation::Stem),
        );
        assert_eq!((site.schools, e_school.schools, stem.schools), (324, 23, 8));
        assert_eq!(e_school.equity_supplement, 0.0);
        assert_eq!(stem.equity_supplement, 0.0);
        assert!(
            stem.enrolled_adm > 4_500.0,
            "and they are not paid nothing because they are empty: {} ADM",
            stem.enrolled_adm
        );
    }

    #[test]
    fn every_recipient_is_paid_the_published_rate_times_its_enrolled_adm() {
        // The same identity the extractor enforces against the workbook, held here against the
        // committed file — so a fixture edited by hand fails as loudly as a reposted workbook.
        let rate = equity_rate(FISCAL_YEAR).expect("FY2027 has a rate");
        for school in recipients() {
            assert!(
                (school.equity_supplement - school.enrolled_adm * rate).abs() <= CENT,
                "{} is paid {} against {} ADM at {rate}",
                school.irn,
                school.equity_supplement,
                school.enrolled_adm
            );
        }
    }

    #[test]
    fn only_the_year_with_a_model_behind_it_claims_to_have_one() {
        let verified: Vec<u16> = EQUITY_RATES
            .iter()
            .filter(|rate| rate.verified_against_a_model)
            .map(|rate| rate.fiscal_year)
            .collect();
        assert_eq!(
            verified,
            [FISCAL_YEAR],
            "FY2026 and FY2025 are read off the act; no department model for either is held"
        );
        assert_eq!(equity_rate(2027), Some(400.0));
        assert_eq!(equity_rate(2026), Some(500.0));
        assert_eq!(equity_rate(2025), Some(650.0));
        assert_eq!(equity_rate(2024), None, "H.B. 33's first year is not held");
    }

    #[test]
    fn a_counterfactual_applies_the_eligibility_rule_before_the_rate() {
        // `supplement_at` is not a multiplication. An e-school's ADM times a rate is a number the
        // statute does not produce, and the guard is the difference between a counterfactual and
        // a claim.
        let all = by_irn();
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
        let cost = equity_cost();
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
        let share = equity_share_of_state_support();
        assert!(
            (0.0228..0.0231).contains(&share),
            "2.29% of total state support: {share}"
        );
        assert!((total_state_support() - 1_511_789_771.13).abs() <= 1.0);
    }

    #[test]
    fn the_base_funding_supplement_is_paid_to_everyone_which_the_equity_supplement_is_not() {
        // The contrast that stops "STEM schools are funded differently" becoming a general
        // property of the file. Two per-pupil lines, one eligibility rule between them.
        for school in schools() {
            assert!(
                school.base_funding_supplement > 0.0,
                "{} is {} and receives no base funding supplement",
                school.irn,
                school.designation.label()
            );
        }
    }
}
