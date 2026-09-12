//! Which of the Fair School Funding Plan's dollar parameters move with prices, and which do not.
//!
//! # The question this answers
//!
//! `parameter/gifted-funding-rates` says it in its own `kind:` field:
//!
//! > A price on a post is a legislated *salary assumption*, and it does not move with the labour
//! > market — so it behaves like a legislated parameter in law and like a deteriorating one in
//! > substance, in the way a nominal figure always does. The corpus's `deflator` is the tool for
//! > that and no constant-dollar companion series exists here yet.
//!
//! `the_parameters_before_this_biennium` says it again about the preschool flat grant: *"it is the
//! part a constant-dollar series would say most about — a nominal $4,000 unmoved since at least
//! FY2014"*. [`crate::base_cost`] did this for the one parameter that has its own module, and
//! found the statewide average base cost per pupil rising 12.11% in cash across FY2022-FY2026 and
//! falling 0.53% against CPI-U. This is the same instrument pointed at the parameters beside it.
//!
//! # The finding, in one table
//!
//! | parameter | nominal | set | FY2026 $ | real |
//! |---|--:|---|--:|--:|
//! | preschool flat grant | $4,000 | FY2014 | $5,604.56 | **−28.6%** |
//! | DPIA per pupil | $422 | FY2022 | $475.61 | −11.3% |
//! | gifted identification | $24.00 | FY2022 | $27.05 | −11.3% |
//! | gifted referral | $2.50 | FY2022 | $2.82 | −11.3% |
//! | gifted coordinator unit | $85,776 | FY2022 | $96,672.30 | −11.3% |
//! | gifted K-8 specialist unit | $89,378 | FY2022 | $100,731.87 | −11.3% |
//! | gifted 9-12 specialist unit | $80,974 | FY2022 | $91,260.29 | −11.3% |
//!
//! Against which the transportation minimum state share is **not** frozen: it runs 33.33%, 37.50%,
//! 41.67%, 45.83%, 50.00% across FY2023-FY2027, which is eight to twelve twenty-fourths in equal
//! steps. One schedule, written into successive budgets, and the only dollar-affecting parameter
//! in the plan that rises by rule rather than by amendment.
//!
//! So within a single formula, some parameters index and some do not — and nothing in the statute
//! says which should. The difference is legislative attention, not design.
//!
//! # Why this is not a complaint about inflation
//!
//! A legislature is entitled to let a nominal amount decay; that is a policy choice and it is
//! reversible in any budget. What makes it worth measuring is that **the decay is invisible in the
//! document that sets it**. R.C. 3317.022 prints `$422` in FY2022 and `$422` in FY2027, and a
//! reader of the section cannot see that the second one buys 11.3% less. This is the state-side
//! twin of what [H.B. 920](../../../.yidam/corpus/legislation/hb-920-1976.yml) does to the local
//! share — the same erosion, reached by legislative inaction rather than by a reduction factor.
//!
//! # The index, and the year it stops
//!
//! June CPI-U of the year the fiscal year ends, which is [`deflator`]'s convention and the Ohio
//! Auditor's. [`BASE_YEAR`] is FY2026 and not FY2027, because **June 2027 has not happened**:
//! the panel this module reads is the department's FY2027 model, so every erosion below is stated
//! against a price level one year before the payment it is applied to. That makes each figure a
//! **lower bound**. [`crate::base_cost`] handles the same boundary the same way, and its FY2027
//! row carries a dash rather than a number.

use std::collections::BTreeMap;

use deflator::CpiSeries;
use edfund_core::{Dollars, FiscalYear};

use crate::panel::DistrictRecord;

/// The year every constant-dollar figure here is stated in.
///
/// FY2026 rather than FY2027 because June 2027 has no price index yet. See the module docs.
pub const BASE_YEAR: FiscalYear = FiscalYear(2026);

/// How a parameter's nominal value is maintained over time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Maintenance {
    /// Unchanged in nominal terms since the year given, and unchanged through FY2027.
    ///
    /// The year is the first fiscal year the current amount applied, evidenced by the enacting
    /// act's own analysis rather than by the codified section — a section carries only its
    /// current numbers, which is exactly why the freeze is hard to see.
    Frozen {
        /// The first fiscal year the current amount applied.
        since: u16,
    },
    /// Raised by a schedule successive budgets have continued.
    Scheduled,
}

/// Whether a parameter's aid sits inside `[H] Foundation Funding`, and so under the guarantee.
///
/// This is the difference between an erosion a held district absorbs and one it feels in full,
/// and it is not a detail: the most eroded parameter here is outside.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Inside formula aid. An increase reaches a district only above its guarantee floor.
    InFormula,
    /// Outside `[H]`, paid beside the formula. An increase reaches every district in full.
    BesideFormula,
}

/// One legislated dollar amount in the plan.
#[derive(Debug, Clone, Copy)]
pub struct Parameter {
    /// Short name, as the corpus node writes it.
    pub name: &'static str,
    /// The nominal amount in force through FY2027.
    pub nominal: Dollars,
    /// How it has been maintained.
    pub maintenance: Maintenance,
    /// Where its aid sits relative to the guarantee.
    pub standing: Standing,
    /// The statutory division that prints the amount.
    pub section: &'static str,
    /// The sentence in a committed LSC analysis that dates it, checked by the tests.
    pub dated_by: &'static str,
}

/// Every dollar parameter of the plan whose value this workspace can date and deflate.
///
/// Not every parameter of the plan. The weights — special education, English learner,
/// career-technical — are ratios and do not deflate; what they multiply is the statewide average
/// base cost per pupil, which [`crate::base_cost`] already carries. The minimum state share and
/// the half-day multiplier are likewise ratios.
pub const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "preschool flat grant",
        nominal: 4_000.0,
        maintenance: Maintenance::Frozen { since: 2014 },
        standing: Standing::BesideFormula,
        section: "R.C. 3317.0213",
        dated_by: "The base amount of $4,000 remains unchanged.",
    },
    Parameter {
        name: "DPIA per pupil",
        nominal: 422.0,
        maintenance: Maintenance::Frozen { since: 2022 },
        standing: Standing::InFormula,
        section: "R.C. 3317.022(A)(4)(a)(i)",
        dated_by: "with the base per-pupil amount increased from $272 \
                   to $422",
    },
    Parameter {
        name: "gifted identification",
        nominal: 24.0,
        maintenance: Maintenance::Frozen { since: 2022 },
        standing: Standing::InFormula,
        section: "R.C. 3317.022(A)(6)(a)",
        dated_by: "from $5.50 to $24 per enrolled ADM in grades K-6",
    },
    Parameter {
        name: "gifted referral",
        nominal: 2.50,
        maintenance: Maintenance::Frozen { since: 2022 },
        standing: Standing::InFormula,
        section: "R.C. 3317.022(A)(6)(a)",
        dated_by: "gifted referral funding equal to $2.50 per enrolled ADM",
    },
    Parameter {
        name: "gifted coordinator unit",
        nominal: 85_776.0,
        maintenance: Maintenance::Frozen { since: 2022 },
        standing: Standing::InFormula,
        section: "R.C. 3317.051(C)(1)",
        dated_by: "one gifted intervention specialist unit to 140 gifted students",
    },
    Parameter {
        name: "gifted K-8 specialist unit",
        nominal: 89_378.0,
        maintenance: Maintenance::Frozen { since: 2022 },
        standing: Standing::InFormula,
        section: "R.C. 3317.051(C)(1)",
        dated_by: "one gifted intervention specialist unit to 140 gifted students",
    },
    Parameter {
        name: "gifted 9-12 specialist unit",
        nominal: 80_974.0,
        maintenance: Maintenance::Frozen { since: 2022 },
        standing: Standing::InFormula,
        section: "R.C. 3317.051(C)(1)",
        dated_by: "one gifted intervention specialist unit to 140 gifted students",
    },
];

/// The transportation minimum state share, FY2023 through FY2027.
///
/// Eight to twelve twenty-fourths, rounded to two places, as each budget enacted the next two
/// years of it. The one parameter in the plan that rises by rule.
pub const TRANSPORTATION_MINIMUM_SHARE: [(u16, f64); 5] = [
    (2023, 33.33),
    (2024, 37.50),
    (2025, 41.67),
    (2026, 45.83),
    (2027, 50.00),
];

impl Parameter {
    /// The first fiscal year this amount applied, for a frozen parameter.
    #[must_use]
    pub const fn frozen_since(&self) -> Option<u16> {
        match self.maintenance {
            Maintenance::Frozen { since } => Some(since),
            Maintenance::Scheduled => None,
        }
    }

    /// What the amount would be in [`BASE_YEAR`] dollars had it kept its value.
    ///
    /// `None` for a parameter that is not frozen, which has no single vintage to deflate from.
    #[must_use]
    pub fn real(&self) -> Option<Dollars> {
        let since = self.frozen_since()?;
        CpiSeries::cpi_u_june()
            .convert(self.nominal, FiscalYear(since), BASE_YEAR)
            .ok()
            .map(|deflated| deflated.value)
    }

    /// The fraction of its value the amount has lost, as a positive number.
    ///
    /// `1 − index(set) / index(BASE_YEAR)`. A rate frozen since FY2022 returns `0.1127`.
    #[must_use]
    pub fn erosion(&self) -> Option<f64> {
        let real = self.real()?;
        (real > 0.0).then(|| 1.0 - self.nominal / real)
    }

    /// What the amount would have to be to have kept pace, as a multiplier on aid.
    ///
    /// `index(BASE_YEAR) / index(set) − 1`. This is the right factor to apply to a *payment*,
    /// because every one of these parameters enters its formula linearly: aid is
    /// `rate × count × share`, so scaling the rate scales the aid by the same fraction.
    #[must_use]
    pub fn shortfall_factor(&self) -> Option<f64> {
        let real = self.real()?;
        (self.nominal > 0.0).then(|| real / self.nominal - 1.0)
    }
}

/// The four components the parameters above are paid through.
///
/// Gifted's five amounts all land in `[F]`, so they share one component and one factor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Component {
    /// `[D]` — Disadvantaged Pupil Impact Aid.
    Dpia,
    /// `[F]` — gifted identification, referral and the three unit kinds.
    Gifted,
    /// Preschool special education, outside `[H]`.
    PreschoolSpecialEducation,
}

impl Component {
    /// Where this component's aid sits relative to the guarantee.
    #[must_use]
    pub const fn standing(self) -> Standing {
        match self {
            Self::Dpia | Self::Gifted => Standing::InFormula,
            Self::PreschoolSpecialEducation => Standing::BesideFormula,
        }
    }

    /// The parameter that prices it, for the factor.
    #[must_use]
    fn parameter(self) -> &'static Parameter {
        let name = match self {
            Self::Dpia => "DPIA per pupil",
            Self::Gifted => "gifted identification",
            Self::PreschoolSpecialEducation => "preschool flat grant",
        };
        PARAMETERS
            .iter()
            .find(|p| p.name == name)
            .expect("every component is priced by a parameter in the table")
    }

    /// One district's aid from this component, as the department's model computes it.
    #[must_use]
    pub fn aid(self, record: &DistrictRecord) -> Dollars {
        match self {
            Self::Dpia => record.categoricals.dpia,
            Self::Gifted => record.categoricals.gifted,
            Self::PreschoolSpecialEducation => record.preschool_special_education.total,
        }
    }
}

/// What one district loses to the freeze, and how much of it the guarantee absorbs.
#[derive(Debug, Clone)]
pub struct Incidence {
    /// The district.
    pub irn: String,
    /// Its name.
    pub name: String,
    /// What the frozen parameters would pay it if they had held their real value, less what they
    /// pay it now — before the guarantee.
    pub computed: Dollars,
    /// The part of [`Self::computed`] that would actually reach the district.
    ///
    /// Equal to `computed` for a district on formula, and for every district's share of the
    /// components that sit beside the formula. Less than it for a district the guarantee holds.
    pub delivered: Dollars,
    /// [`Self::delivered`] per enrolled pupil.
    pub per_pupil: f64,
    /// Whether the guarantee currently pays this district.
    pub on_guarantee: bool,
    /// The computed shortfall by component, largest first.
    pub by_component: Vec<(Component, Dollars)>,
}

/// The per-district incidence of the freeze, across a panel.
///
/// # What the counterfactual is
///
/// Each frozen parameter is raised to the amount that would have kept its [`BASE_YEAR`] value,
/// every count and every share held where the department's model has them, and the resulting aid
/// is compared with the model's own. It is not a forecast and not a proposal; it is the size of
/// what the nominal freeze has taken, priced in the year it is taken.
///
/// # Why the guarantee has to be in it
///
/// A district the guarantee holds is paid `max(formula, floor)`, so an increase in formula aid
/// reaches it only above the floor. The two components inside `[H]` are therefore absorbed for a
/// held district, in whole or in part. The third is not: preschool special education sits outside
/// `[H] Foundation Funding` — [`DistrictRecord::categorical_funding`] says so — so its shortfall
/// reaches every district in full, guaranteed or not.
///
/// That is the sharpest thing here. **The most eroded parameter is the one the guarantee cannot
/// cushion**, and it is eroded most because it has been frozen longest.
#[must_use]
pub fn incidence(records: &[DistrictRecord]) -> Vec<Incidence> {
    records
        .iter()
        .map(|record| {
            let mut by_component: Vec<(Component, Dollars)> = Vec::new();
            let mut in_formula = 0.0;
            let mut beside = 0.0;

            for component in [
                Component::Dpia,
                Component::Gifted,
                Component::PreschoolSpecialEducation,
            ] {
                let factor = component
                    .parameter()
                    .shortfall_factor()
                    .expect("every priced component is frozen and so has a factor");
                let shortfall = component.aid(record) * factor;
                if shortfall <= 0.0 {
                    continue;
                }
                by_component.push((component, shortfall));
                match component.standing() {
                    Standing::InFormula => in_formula += shortfall,
                    Standing::BesideFormula => beside += shortfall,
                }
            }
            by_component.sort_by(|a, b| b.1.total_cmp(&a.1));

            // The guarantee is a `max`, so only the part of an increase that lifts formula aid
            // above the floor is delivered. Districts on formula have `floor <= core` already and
            // this reduces to the whole increase.
            let core = record.core_foundation_funding;
            let floor = record.guarantee_floor();
            let delivered_in_formula = (core + in_formula).max(floor) - core.max(floor);

            let delivered = delivered_in_formula + beside;
            let adm = record.enrollment.base_cost_enrolled_adm;
            Incidence {
                irn: record.irn.clone(),
                name: record.name.clone(),
                computed: in_formula + beside,
                delivered,
                per_pupil: if adm > 0.0 { delivered / adm } else { 0.0 },
                on_guarantee: record.on_guarantee(),
                by_component,
            }
        })
        .collect()
}

/// Statewide totals of [`incidence`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Statewide {
    /// The shortfall before the guarantee.
    pub computed: Dollars,
    /// The shortfall that would reach districts.
    pub delivered: Dollars,
    /// How many districts are held by the guarantee.
    pub held: usize,
    /// How many districts the delivered shortfall reaches at all.
    pub reached: usize,
}

/// [`incidence`] summed.
#[must_use]
pub fn statewide(records: &[DistrictRecord]) -> Statewide {
    let rows = incidence(records);
    Statewide {
        computed: rows.iter().map(|r| r.computed).sum(),
        delivered: rows.iter().map(|r| r.delivered).sum(),
        held: rows.iter().filter(|r| r.on_guarantee).count(),
        reached: rows.iter().filter(|r| r.delivered > 0.0).count(),
    }
}

/// The computed shortfall by component, statewide.
#[must_use]
pub fn by_component(records: &[DistrictRecord]) -> BTreeMap<Component, Dollars> {
    let mut out: BTreeMap<Component, Dollars> = BTreeMap::new();
    for row in incidence(records) {
        for (component, shortfall) in row.by_component {
            *out.entry(component).or_default() += shortfall;
        }
    }
    out
}
