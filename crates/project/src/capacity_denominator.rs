//! What the local capacity measure divides by, and what choosing differently would cost.
//!
//! Ohio's capacity measure is a per-pupil quantity subtracted from another per-pupil quantity, and
//! its numerator and its denominator count different children. Assessed valuation is the whole tax
//! base and federal adjusted gross income is the whole resident income; neither moves when a child
//! leaves for a community school. Base cost enrolled ADM does. So a district whose residents send
//! 40% of their children elsewhere is measured as having its entire community's wealth behind the
//! 60% it still teaches.
//!
//! That is the hypothesis this module was built to test, and the answer has two halves that point
//! opposite ways.
//!
//! # The denominator cancels out of the base cost channel
//!
//! R.C. 3317.017 divides all three wealth terms by base cost enrolled ADM at (A)(1)(b), (A)(2)(b)
//! and (A)(3)(c), and division (B) multiplies the residual back by enrolled ADM. Write the blend
//! out and the count cancels:
//!
//! ```text
//! aid = (base cost per pupil - capacity per pupil) x enrolled ADM
//!     = base cost per pupil x enrolled ADM
//!       - (0.6 V + 0.2 AGI + 0.2 M x R) x rate x (enrolled ADM / base cost enrolled ADM)
//!       └──────────────── [`District::local_charge`], a wealth quantity ────────────────┘
//! ```
//!
//! So what Ohio charges a district against its base cost is **a number of dollars set by its
//! wealth**, and the pupil count enters only through the ratio of two counts that are within 1.6%
//! of each other at the median. A child leaving costs the district one base cost per pupil and
//! nothing else: `the_marginal_pupil_costs_one_base_cost` measures the residual at zero for the
//! 80 districts where the two counts coincide and *positive* — in the district's favour — for
//! every other, because base cost enrolled ADM is a three-year maximum and the departure takes a
//! year to reach it in full.
//!
//! Nor is the departed child unfunded. R.C. 3317.022 makes the community and STEM school unit and
//! the four scholarship units **funding units of their own**, and division (A)(1)(b) gives that
//! unit *the aggregate base cost* with no state share term at all. The state's outlay for a
//! district's resident children is therefore very nearly invariant to where they are taught, which
//! is the opposite of a double charge. See [`crate::panel::DistrictRecord::total_transfers`] for
//! the empirical half: no line in the FY2027 model deducts anything for them, because since
//! 30 September 2021 there is nothing to deduct.
//!
//! # It does not cancel out of the categorical channel
//!
//! Four of the six categoricals — special education, English learners, gifted and career-technical
//! — are multiplied by the district's **state share percentage**, which R.C. 3317.022(A)(3), (5)
//! and (6) attach to districts and to no other funding unit. A percentage is a ratio of two
//! per-pupil figures, so nothing cancels: the capacity denominator passes straight into it.
//!
//! [`weighted_categorical_entitlement`] measures what that costs. $2.43bn of special education,
//! English learner and career-technical entitlement is computed for the 609 districts and $0.81bn
//! of it is paid; the local share takes **66.6%**. The same weights against the same statewide
//! average base cost are paid to the community and STEM school unit in full. For Columbus City,
//! whose state share percentage is 10.25%, the state pays a community school **9.76 times** what
//! it pays the district for the same category-six pupil — and Columbus's percentage is 10.25%
//! because 30,000 of its resident children are counted in neither its numerator nor its
//! denominator.
//!
//! # Three denominators, and the plan already uses two of them
//!
//! [`Basis`] is not a menu this module invented. R.C. 3317.017 divides by base cost enrolled ADM;
//! R.C. 3317.0217(C)(1), one section away and built from *the same two certified numerators*,
//! divides by enrolled ADM less those open-enrolling in plus those open-enrolling out; and
//! R.C. 3317.03(A)(2) defines a total ADM that counts community school, STEM and scholarship
//! students as well. The department computes all three; the FY2027 model publishes the first two
//! in its own columns and the third only at FY2019, which is why [`Basis::Resident`] reads Table
//! SD-1 instead and `two_publishers_agree_about_which_districts_diverge` checks the two against
//! each other.
//!
//! The middle one is the finding. Targeted assistance reaches for a resident count and takes
//! exactly one of the nine channels R.C. 3317.03(A)(2) lists — **2,098 pupils statewide against a
//! resident-less-enrolled gap of 222,923, which is 0.9% of it**. See
//! `the_plans_own_resident_count_reaches_almost_none_of_the_gap`.

use std::collections::BTreeMap;

use edfund_core::{Adm, Dollars};
use local_capacity::StateShare;

use crate::panel::{
    panel, DistrictRecord, AVERAGE_BASE_COST_PER_PUPIL, CTE_ASSOCIATED_WEIGHT,
    CTE_BASE_COST_PER_PUPIL, CTE_WEIGHTS, ENGLISH_LEARNER_WEIGHTS, MINIMUM_STATE_SHARE,
    SPECIAL_EDUCATION_WEIGHTS, TA_MEDIAN_WEALTH_PER_PUPIL, TA_WEALTH_INDEX_FLOOR,
    TA_WEALTH_OFFSET_RATE, TA_WEALTH_RATE,
};

/// The tax year of Table SD-1 whose ADM is read as the resident count.
///
/// The most recent the abstract carries, and deliberately not
/// [`dispersion::valuation::TAX_YEAR`]: that constant fixes the year on which the *numerator*
/// identity between the two agencies holds, which is what makes the valuation divergence
/// attributable. Nothing here compares numerators, so the question is only which resident count
/// stands closest to the FY2027 model, and TY2024 is it.
pub const RESIDENT_TAX_YEAR: u16 = 2024;

/// Which children a district's wealth is divided by.
///
/// Each variant is a denominator some part of the Fair School Funding Plan actually uses. None of
/// them is this module's invention and none is a hypothetical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Basis {
    /// **R.C. 3317.017(A)(1)(b), (A)(2)(b), (A)(3)(c)** — base cost enrolled ADM, the three-year
    /// figure the capacity measure divides by and the one the department uses.
    BaseCostEnrolled,
    /// **R.C. 3317.0217(C)(1)** — enrolled ADM less the students of R.C. 3317.03(A)(1)(b) plus
    /// those of (A)(2)(d): open enrolment out, and open enrolment in only.
    ///
    /// Applied here to base cost enrolled ADM rather than to `[a]` enrolled ADM, so that the
    /// counterfactual changes the *concept* of the denominator and not its vintage as well.
    OpenEnrolmentAdjusted,
    /// The Department of Taxation's count on Table SD-1: every child resident in the district,
    /// community school, STEM and scholarship students included.
    Resident,
}

impl Basis {
    /// The section, or the publisher, this denominator comes from.
    #[must_use]
    pub const fn authority(self) -> &'static str {
        match self {
            Self::BaseCostEnrolled => "R.C. 3317.017(A)(1)(b)",
            Self::OpenEnrolmentAdjusted => "R.C. 3317.0217(C)(1)",
            Self::Resident => "Table SD-1, the Department of Taxation",
        }
    }
}

/// One district's capacity measure, with all three denominators beside it.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Information Retrieval Number. The join key, because 609 districts share about 580 names.
    pub irn: String,
    /// District name as the model publishes it.
    pub name: String,
    /// Base cost per pupil, the quantity capacity is subtracted from.
    pub base_cost_per_pupil: Dollars,
    /// The denominator R.C. 3317.017 names.
    pub base_cost_enrolled_adm: Adm,
    /// The multiplier division (B) names, which is not the same count.
    pub current_enrolled_adm: Adm,
    /// Base cost enrolled ADM under the targeted assistance adjustment.
    pub open_enrolment_adjusted_adm: Adm,
    /// Table SD-1's ADM for [`RESIDENT_TAX_YEAR`].
    pub resident_adm: Adm,
    /// The department's own total ADM for FY2019, from the targeted assistance supplement's
    /// eligibility test — a second instrument on the same question, at a different vintage.
    pub fy19_total_adm: Adm,
    /// And the enrolled ADM beside it, which is what makes the ratio readable.
    pub fy19_enrolled_adm: Adm,
    /// `[b1]` per pupil capacity amount times base cost enrolled ADM.
    ///
    /// The blend before it was divided by anything: `(0.6 V + 0.2 AGI + 0.2 M x R) x rate`. This
    /// is the quantity the measure actually charges, and it is a function of wealth alone —
    /// `the_local_charge_is_the_blend_before_any_division` proves it against
    /// [`local_capacity::local_capacity`] rather than asserting it.
    pub local_charge: Dollars,
    /// Special education, English learners, gifted and career-technical as paid: the four
    /// categoricals R.C. 3317.022 multiplies by the state share percentage.
    pub weighted_categoricals: Dollars,
}

impl District {
    /// The denominator on one basis.
    #[must_use]
    pub const fn adm_on(&self, basis: Basis) -> Adm {
        match basis {
            Basis::BaseCostEnrolled => self.base_cost_enrolled_adm,
            Basis::OpenEnrolmentAdjusted => self.open_enrolment_adjusted_adm,
            Basis::Resident => self.resident_adm,
        }
    }

    /// Local capacity per pupil on one basis.
    ///
    /// On [`Basis::BaseCostEnrolled`] this is the department's published `[b1]` by construction,
    /// because [`District::local_charge`] was formed from it.
    #[must_use]
    pub fn capacity_per_pupil_on(&self, basis: Basis) -> Dollars {
        self.local_charge / self.adm_on(basis)
    }

    /// The state share of base cost on one basis, floored where R.C. 3317.017(B)(1) floors it.
    ///
    /// # Panics
    ///
    /// Never for a district in [`frame`], which admits only positive base cost per pupil.
    #[must_use]
    pub fn state_share_on(&self, basis: Basis) -> StateShare {
        local_capacity::state_share(
            self.base_cost_per_pupil,
            self.capacity_per_pupil_on(basis),
            self.current_enrolled_adm,
            MINIMUM_STATE_SHARE,
        )
        .expect("the frame admits only positive base cost per pupil")
    }

    /// Everything the capacity measure reaches, on one basis: the state share of base cost plus
    /// the four categoricals that ride on the state share percentage.
    ///
    /// The categoricals are rescaled by the ratio of the two percentages, which is what
    /// R.C. 3317.022(A)(3), (5) and (6) do to them. It is the one modelled step here, and it is a
    /// linear one the statute writes out.
    #[must_use]
    pub fn aid_on(&self, basis: Basis) -> Dollars {
        let here = self.state_share_on(basis);
        let published = self.state_share_on(Basis::BaseCostEnrolled);
        let rescaled = if published.percentage > 0.0 {
            self.weighted_categoricals * here.percentage / published.percentage
        } else {
            self.weighted_categoricals
        };
        here.amount + rescaled
    }

    /// Resident children per child taught, on Table SD-1's count over the formula's.
    #[must_use]
    pub fn resident_ratio(&self) -> f64 {
        self.resident_adm / self.base_cost_enrolled_adm
    }
}

/// Every district in the FY2027 model that Table SD-1 can also name, with all three denominators.
///
/// All 609 join: the abstract carries 611 rows and the two it carries that the model does not are
/// the island districts.
#[must_use]
pub fn frame() -> Vec<District> {
    let resident: BTreeMap<String, Adm> = dispersion::sd1::rows()
        .into_iter()
        .filter(|row| row.tax_year == RESIDENT_TAX_YEAR)
        .filter_map(|row| row.adm.map(|adm| (row.irn, adm)))
        .collect();

    panel()
        .iter()
        .filter_map(|record| {
            let capacity = record.published_capacity_per_pupil?;
            let resident_adm = resident.get(&record.irn).copied()?;
            (record.base_cost_per_pupil > 0.0 && resident_adm > 0.0).then(|| {
                let assistance = &record.targeted_assistance;
                District {
                    irn: record.irn.clone(),
                    name: record.name.clone(),
                    base_cost_per_pupil: record.base_cost_per_pupil,
                    base_cost_enrolled_adm: record.base_cost_adm(),
                    current_enrolled_adm: record.current_year_adm,
                    open_enrolment_adjusted_adm: record.base_cost_adm()
                        - assistance.open_enrollment_in
                        + assistance.open_enrollment_out,
                    resident_adm,
                    fy19_total_adm: assistance.fy19_total_adm,
                    fy19_enrolled_adm: assistance.fy19_enrolled_adm,
                    local_charge: capacity * record.base_cost_adm(),
                    weighted_categoricals: weighted_categoricals(record),
                }
            })
        })
        .collect()
}

/// The four categoricals a district's state share percentage multiplies, as paid.
///
/// Targeted assistance and disadvantaged pupil impact aid are excluded because they do not ride on
/// it: R.C. 3317.0217 is its own equalisation and R.C. 3317.022(A)(4) is a flat amount times an
/// index times a count.
#[must_use]
fn weighted_categoricals(record: &DistrictRecord) -> Dollars {
    record.categoricals.special_education
        + record.categoricals.english_learners
        + record.categoricals.gifted
        + record.categoricals.career_technical
}

/// What special education, English learners and career-technical would pay at a state share of
/// one, and what they pay.
///
/// Returned as `(entitlement, paid)`. Gifted is excluded because its unit funding is not a weight
/// against a base cost and so has no gross form the panel's own columns can reconstruct.
///
/// The gap is what the local share takes off a categorical — and the community and STEM school
/// unit is charged none of it, because R.C. 3317.022(A)(3), (5) and (6) attach the multiplier to
/// districts by name.
#[must_use]
pub fn weighted_categorical_entitlement() -> (Dollars, Dollars) {
    let mut entitlement = 0.0;
    let mut paid = 0.0;
    for record in &panel() {
        let special: Dollars = (0..6)
            .map(|i| {
                record.special_education.adm[i]
                    * SPECIAL_EDUCATION_WEIGHTS[i]
                    * AVERAGE_BASE_COST_PER_PUPIL
            })
            .sum();
        let english: Dollars = (0..3)
            .map(|i| {
                record.english_learners.adm[i]
                    * ENGLISH_LEARNER_WEIGHTS[i]
                    * AVERAGE_BASE_COST_PER_PUPIL
            })
            .sum();
        let career: Dollars = (0..5)
            .map(|i| record.career_technical.fte[i] * CTE_WEIGHTS[i] * CTE_BASE_COST_PER_PUPIL)
            .sum::<Dollars>()
            + record.career_technical.total_fte() * CTE_ASSOCIATED_WEIGHT * CTE_BASE_COST_PER_PUPIL;
        entitlement += special + english + career;
        paid += record.categoricals.special_education
            + record.categoricals.english_learners
            + record.categoricals.career_technical;
    }
    (entitlement, paid)
}

/// How large a move counts as one, in dollars.
///
/// Not a tuning knob: 138 districts sit on the 10% floor under every denominator here, so their
/// aid is identical under all three and the only thing separating "unchanged" from "moved by a
/// millionth of a cent" is the order `(charge / adm) * adm` is evaluated in. A dollar is far below
/// anything this module reports and far above that.
pub const MATERIAL: Dollars = 1.0;

/// What re-basing the capacity denominator would move, against the statute's own choice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Exposure {
    /// Districts whose aid rises, and by how much in total.
    pub gainers: usize,
    /// The total gain, in dollars.
    pub gain: Dollars,
    /// Districts whose aid falls.
    pub losers: usize,
    /// The total loss, as a positive number of dollars.
    pub loss: Dollars,
    /// Districts on which the 10% minimum state share binds under the statute's denominator but
    /// not under this one.
    pub leave_the_floor: usize,
}

impl Exposure {
    /// Gain less loss.
    #[must_use]
    pub fn net(&self) -> Dollars {
        self.gain - self.loss
    }
}

/// [`Exposure`] for one denominator, over [`frame`].
#[must_use]
pub fn exposure(basis: Basis) -> Exposure {
    let mut out = Exposure {
        gainers: 0,
        gain: 0.0,
        losers: 0,
        loss: 0.0,
        leave_the_floor: 0,
    };
    for district in &frame() {
        let delta = district.aid_on(basis) - district.aid_on(Basis::BaseCostEnrolled);
        if delta > MATERIAL {
            out.gainers += 1;
            out.gain += delta;
        } else if delta < -MATERIAL {
            out.losers += 1;
            out.loss -= delta;
        }
        if district.state_share_on(Basis::BaseCostEnrolled).at_minimum
            && !district.state_share_on(basis).at_minimum
        {
            out.leave_the_floor += 1;
        }
    }
    out
}

/// The local charge across all 609 districts: what the measure asks Ohio's communities to bear,
/// with no pupil count in it.
#[must_use]
pub fn statewide_local_charge() -> Dollars {
    frame().iter().map(|d| d.local_charge).sum()
}

/// The statewide aid the capacity measure reaches, on one denominator.
#[must_use]
pub fn statewide(basis: Basis) -> Dollars {
    frame().iter().map(|d| d.aid_on(basis)).sum()
}

/// How closely a district's resident-to-funded ratio tracks its published state share percentage.
///
/// Pearson, over all 609. The falsifier this module was given: if the divergence is what makes a
/// district's share low, the two should move together. They do not — the correlation is 0.07, and
/// Youngstown has the highest ratio in the state with a 71% state share while Columbus has the
/// ninth and a 10% one. The denominator rides on top of the measure's progressivity rather than
/// reversing it.
#[must_use]
pub fn divergence_correlation() -> f64 {
    let pairs: Vec<(f64, f64)> = frame()
        .iter()
        .map(|d| {
            (
                d.resident_ratio(),
                d.state_share_on(Basis::BaseCostEnrolled).percentage,
            )
        })
        .collect();
    let n = pairs.len() as f64;
    let (mx, my) = (
        pairs.iter().map(|p| p.0).sum::<f64>() / n,
        pairs.iter().map(|p| p.1).sum::<f64>() / n,
    );
    let covariance: f64 = pairs.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
    let sx: f64 = pairs.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>().sqrt();
    let sy: f64 = pairs.iter().map(|p| (p.1 - my).powi(2)).sum::<f64>().sqrt();
    covariance / (sx * sy)
}

/// What re-basing targeted assistance's **own** denominator would move.
///
/// Returned as `(on the statute's adjusted count, on the resident count, districts that gain)`.
///
/// # Why this is the component that matters most
///
/// R.C. 3317.0217(C)(1) is the one place the plan already concedes the principle. It divides
/// weighted wealth by a **residence-flavoured** count — enrolled ADM less open enrolment in plus
/// open enrolment out — and then multiplies the resulting rate by **enrolled** ADM. So targeted
/// assistance already measures wealth against the children a district is responsible for and pays
/// against the children it teaches. That is exactly the correction this module's other functions
/// have to argue for; here it is the statute's own design.
///
/// It reaches one of the nine channels R.C. 3317.03(A)(2) lists. This measures what the other
/// eight would move if the same denominator ran to the full resident count and nothing else
/// changed — the multiplier stays enrolled ADM, because moving that would be a different
/// counterfactual and a much larger one.
///
/// # The two counts are one line apart and mixing them is a factor of 38
///
/// `[D]` divides by the adjusted count; `[F]` multiplies by enrolled ADM. Dividing *and*
/// multiplying by the adjusted count reproduces nothing — it misses the published wealth tier by
/// 38x — which is why `the_wealth_tier_reproduces_before_it_is_re_based` checks the reconstruction
/// against the department's own published `[F]` before the counterfactual is allowed to mean
/// anything. `[D]` is published rounded to the cent and `[E]` and `[F]` are computed from the
/// rounded figure, so the reconstruction rounds too.
#[must_use]
pub fn targeted_assistance_on_the_resident_count() -> (Dollars, Dollars, usize) {
    let resident: BTreeMap<String, Adm> = dispersion::sd1::rows()
        .into_iter()
        .filter(|row| row.tax_year == RESIDENT_TAX_YEAR)
        .filter_map(|row| row.adm.map(|adm| (row.irn, adm)))
        .collect();

    let (mut here, mut there, mut gainers) = (0.0, 0.0, 0);
    for record in &panel() {
        let Some(&resident_adm) = resident.get(&record.irn) else {
            continue;
        };
        let assistance = &record.targeted_assistance;
        if assistance.weighted_wealth <= 0.0 {
            continue;
        }
        let enrolled = record.categorical_enrolled_adm;
        let statute = wealth_tier(
            assistance.weighted_wealth,
            assistance.resident_adm(enrolled),
            enrolled,
        );
        let rebased = wealth_tier(assistance.weighted_wealth, resident_adm, enrolled);
        here += statute;
        there += rebased;
        if rebased - statute > MATERIAL {
            gainers += 1;
        }
    }
    (here, there, gainers)
}

/// The wealth tier of targeted assistance, on a denominator and a multiplier that need not agree.
///
/// R.C. 3317.0217(C)(1) through (C)(4). The rounding is the department's: `[D]` is published to
/// the cent and the index and amount are computed from the rounded figure, so a reconstruction
/// that keeps full precision does not reproduce.
fn wealth_tier(weighted_wealth: Dollars, denominator: Adm, multiplier: Adm) -> Dollars {
    if denominator <= 0.0 {
        return 0.0;
    }
    let per_pupil = (weighted_wealth / denominator * 100.0).round() / 100.0;
    if per_pupil <= 0.0 || TA_MEDIAN_WEALTH_PER_PUPIL / per_pupil < TA_WEALTH_INDEX_FLOOR {
        return 0.0;
    }
    (TA_MEDIAN_WEALTH_PER_PUPIL * TA_WEALTH_RATE - per_pupil * TA_WEALTH_OFFSET_RATE) * multiplier
}

/// The two different things re-basing the denominator does, separated.
///
/// Re-basing is not one operation. It divides the local charge by a larger count — which reduces
/// the charge itself, crediting a district for children the state funds in full through another
/// funding unit — **and** it raises the state share percentage, which R.C. 3317.022 multiplies
/// four categoricals by. Only the second is a channel where the statute's own arithmetic fails to
/// cancel, and only the second is a correction anyone has argued for.
///
/// [`exposure`] reports the two together, because that is what the counterfactual costs. This
/// splits them, and `the_exposure_is_two_operations_and_only_one_is_arguable` checks that they
/// add back to it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Decomposition {
    /// What dividing the charge by a larger count takes off the local share of base cost.
    ///
    /// The part #384 priced and said nobody should want: it credits a district with lower capacity
    /// per pupil on account of children already funded elsewhere.
    pub charge_relief: Dollars,
    /// What re-computing the **percentage** alone moves through the four categoricals.
    ///
    /// The charge is untouched; only the ratio R.C. 3317.022(A)(3), (5) and (6) multiply by
    /// changes. This is the correction confined to the channel that does not cancel.
    pub categorical: Dollars,
}

impl Decomposition {
    /// The two halves summed, which is [`exposure`]'s net.
    #[must_use]
    pub fn total(&self) -> Dollars {
        self.charge_relief + self.categorical
    }
}

/// [`Decomposition`] for one denominator, over [`frame`].
#[must_use]
pub fn decompose(basis: Basis) -> Decomposition {
    let mut out = Decomposition {
        charge_relief: 0.0,
        categorical: 0.0,
    };
    for district in &frame() {
        let here = district.state_share_on(basis);
        let published = district.state_share_on(Basis::BaseCostEnrolled);
        out.charge_relief += here.amount - published.amount;
        if published.percentage > 0.0 {
            out.categorical +=
                district.weighted_categoricals * (here.percentage / published.percentage - 1.0);
        }
    }
    out
}

/// What a denominator correction **confined to the categoricals** would move, and over how many.
///
/// The local charge, and therefore every district's state share of base cost, is left exactly as
/// R.C. 3317.017 computes it. Only the state share *percentage* — the ratio R.C. 3317.022(A)(3),
/// (5) and (6) attach to districts and to no other funding unit — is re-computed on `basis`.
///
/// This is the counterfactual the wholesale re-basing in [`exposure`] cannot stand in for. It is
/// smaller, and its loss side is smaller by more: the 119 net open-enrolment-in districts lose
/// through the charge, not through the percentage, so confining the correction very nearly
/// removes them from it.
#[must_use]
pub fn categorical_exposure(basis: Basis) -> Exposure {
    let mut out = Exposure {
        gainers: 0,
        gain: 0.0,
        losers: 0,
        loss: 0.0,
        leave_the_floor: 0,
    };
    for district in &frame() {
        let here = district.state_share_on(basis);
        let published = district.state_share_on(Basis::BaseCostEnrolled);
        let delta = if published.percentage > 0.0 {
            district.weighted_categoricals * (here.percentage / published.percentage - 1.0)
        } else {
            0.0
        };
        if delta > MATERIAL {
            out.gainers += 1;
            out.gain += delta;
        } else if delta < -MATERIAL {
            out.losers += 1;
            out.loss -= delta;
        }
        if published.at_minimum && !here.at_minimum {
            out.leave_the_floor += 1;
        }
    }
    out
}

/// What the correction confined to the categoricals would give the districts the repealed
/// supplement covered, and what it would give the rest.
///
/// Returned as `(to the qualifiers, to everyone else, how many qualifiers)`. Eligibility is
/// [`crate::panel::TargetedAssistance::qualifies`] — H.B. 110's own two FY2019 tests, an enrolled
/// ADM below 88% of total ADM and a wealth index above 1.6.
///
/// Supplemental targeted assistance was the General Assembly's own instrument against this effect
/// and H.B. 96 repealed it. The comparison is what says whether the repeal removed something the
/// size of the problem, and for whom.
#[must_use]
pub fn against_the_repealed_supplement(basis: Basis) -> (Dollars, Dollars, usize) {
    let qualifies: BTreeMap<String, bool> = panel()
        .iter()
        .map(|record| {
            (
                record.irn.clone(),
                record.targeted_assistance.qualifies().unwrap_or(false),
            )
        })
        .collect();

    let (mut covered, mut uncovered, mut qualifiers) = (0.0, 0.0, 0);
    for district in &frame() {
        let here = district.state_share_on(basis);
        let published = district.state_share_on(Basis::BaseCostEnrolled);
        if published.percentage <= 0.0 {
            continue;
        }
        let delta = district.weighted_categoricals * (here.percentage / published.percentage - 1.0);
        if qualifies.get(&district.irn).copied().unwrap_or(false) {
            qualifiers += 1;
            if delta > MATERIAL {
                covered += delta;
            }
        } else if delta > MATERIAL {
            uncovered += delta;
        }
    }
    (covered, uncovered, qualifiers)
}

/// How many resident children the plan's own open-enrolment adjustment reaches, against how many
/// Table SD-1 counts that base cost enrolled ADM does not.
///
/// Returned as `(the adjustment, the gap)`, both in pupils. The first is
/// R.C. 3317.0217(C)(1)'s whole correction; the second is every channel of
/// R.C. 3317.03(A)(2) together.
#[must_use]
pub fn resident_gap() -> (Adm, Adm) {
    let districts = frame();
    let adjustment = districts
        .iter()
        .map(|d| d.open_enrolment_adjusted_adm - d.base_cost_enrolled_adm)
        .sum();
    let gap = districts
        .iter()
        .map(|d| d.resident_adm - d.base_cost_enrolled_adm)
        .sum();
    (adjustment, gap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::TA_SUPPLEMENT_RETENTION;
    use local_capacity::{local_capacity, CapacityInputs};

    /// The charge is the blend before any division, and this checks it against the calculator.
    ///
    /// `capacity per pupil x base cost enrolled ADM` has to equal
    /// `(0.6 V + 0.2 AGI + 0.2 M x R) x rate`, in which no pupil count appears. Computing the
    /// right-hand side from [`local_capacity::local_capacity`]'s own intermediates and multiplying
    /// the denominator back in is a real check on the claim rather than a restatement of it.
    #[test]
    fn the_local_charge_is_the_blend_before_any_division() {
        let mut worst: f64 = 0.0;
        let mut checked = 0;
        for record in &panel() {
            let (Some(capacity), Some(median), Some(returns), Some(statewide), Some(benchmark)) = (
                record.published_capacity_per_pupil,
                record.median_income,
                record.tax_returns,
                record.statewide_median_income,
                record.benchmark_ratio,
            ) else {
                continue;
            };
            let adm = record.base_cost_adm();
            let result = local_capacity(&CapacityInputs {
                valuation_recent: record.valuation_three_year[0],
                valuation_three_year: record.valuation_three_year,
                agi_recent: record.agi_three_year[0],
                agi_three_year: record.agi_three_year,
                federal_median_income: median,
                tax_returns: returns,
                statewide_median_income: statewide,
                benchmark_ratio: benchmark,
                base_cost_enrolled_adm: adm,
            })
            .expect("the panel's inputs are in range");

            // The blend, reassembled with the denominator multiplied back out of every term.
            let blend = result.capacity_valuation * local_capacity::VALUATION_WEIGHT
                + result.capacity_gross_income * local_capacity::GROSS_INCOME_WEIGHT
                + median * returns * local_capacity::MEDIAN_INCOME_WEIGHT;
            let charge = blend * result.capacity_rate;
            worst = worst.max((charge / (capacity * adm) - 1.0).abs());
            checked += 1;
        }
        assert_eq!(
            checked, 609,
            "every district carries the inputs and the published answer"
        );
        assert!(
            worst < 1e-5,
            "the local charge differs from the undivided blend by {worst:.3e} at worst. It cannot: \
             the pupil count appears in all three terms and in nothing else, so multiplying it \
             back out has to leave a quantity set by wealth alone. A residual here means the \
             capacity denominator is no longer base cost enrolled ADM."
        );
    }

    /// A departing pupil costs a district exactly one base cost per pupil, and never more.
    ///
    /// The base cost channel's neutrality, measured rather than argued. Remove one pupil from both
    /// counts and the aid change is `-base cost per pupil` exactly where the two counts coincide —
    /// **80 of the 471 districts off the floor** — and above it everywhere else, because base cost
    /// enrolled ADM is a three-year maximum that takes a year to follow the departure down.
    ///
    /// The residual is never negative. If it were, the measure would be charging a district for a
    /// child it no longer teaches, which is the hypothesis this module tests and does not find.
    #[test]
    fn the_marginal_pupil_costs_one_base_cost() {
        let mut exact = 0;
        let mut worst_negative: f64 = 0.0;
        let mut largest: f64 = 0.0;
        let mut off_the_floor = 0;
        for district in &frame() {
            if district.state_share_on(Basis::BaseCostEnrolled).at_minimum {
                continue;
            }
            off_the_floor += 1;
            let before = (district.base_cost_per_pupil
                - district.local_charge / district.base_cost_enrolled_adm)
                * district.current_enrolled_adm;
            let after = (district.base_cost_per_pupil
                - district.local_charge / (district.base_cost_enrolled_adm - 1.0))
                * (district.current_enrolled_adm - 1.0);
            let residual = after - before + district.base_cost_per_pupil;
            if residual.abs() < 1e-6 {
                exact += 1;
            }
            worst_negative = worst_negative.min(residual);
            largest = largest.max(residual);
        }
        assert_eq!(off_the_floor, 471);
        assert_eq!(exact, 80);
        assert!(
            worst_negative > -1e-6,
            "a departing pupil costs one district {:.2} more than its base cost per pupil. The \
             capacity charge does not depend on the count, so it cannot: a negative residual means \
             base cost enrolled ADM has stopped being the greater of the average and the current \
             year.",
            -worst_negative
        );
        assert!(
            (largest - 1367.93).abs() < 0.01,
            "the largest residual moved to {largest:.2}; it is the district whose three-year \
             maximum sits furthest above its current enrolment"
        );
    }

    /// The plan's own resident adjustment reaches 0.9% of the divergence.
    ///
    /// R.C. 3317.0217(C)(1) takes exactly one of the nine channels R.C. 3317.03(A)(2) lists —
    /// open enrolment — and leaves community school, STEM and every scholarship student in
    /// neither the numerator nor the denominator of either measure.
    #[test]
    fn the_plans_own_resident_count_reaches_almost_none_of_the_gap() {
        let (adjustment, gap) = resident_gap();
        assert!(
            (adjustment - 2_098.0).abs() < 1.0,
            "adjustment was {adjustment:.0}"
        );
        assert!((gap - 222_923.0).abs() < 1.0, "gap was {gap:.0}");
        assert!(
            adjustment / gap < 0.01,
            "the adjustment reaches {:.1}% of the gap",
            adjustment / gap * 100.0
        );
    }

    /// The department's own total ADM agrees with Table SD-1 about which districts diverge.
    ///
    /// Two instruments, two publishers, five years apart: the FY2019 total ADM the targeted
    /// assistance supplement's eligibility test carries, and the Department of Taxation's TY2024
    /// count. They correlate at 0.89 over all 609, which is what makes the resident count a
    /// structural feature of a district rather than an artefact of one table.
    #[test]
    fn two_publishers_agree_about_which_districts_diverge() {
        let districts = frame();
        let pairs: Vec<(f64, f64)> = districts
            .iter()
            .filter(|d| d.fy19_enrolled_adm > 0.0)
            .map(|d| {
                (
                    d.fy19_total_adm / d.fy19_enrolled_adm,
                    d.resident_adm / d.current_enrolled_adm,
                )
            })
            .collect();
        assert_eq!(pairs.len(), 609);
        let n = pairs.len() as f64;
        let (mx, my) = (
            pairs.iter().map(|p| p.0).sum::<f64>() / n,
            pairs.iter().map(|p| p.1).sum::<f64>() / n,
        );
        let covariance: f64 = pairs.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum();
        let sx: f64 = pairs.iter().map(|p| (p.0 - mx).powi(2)).sum::<f64>().sqrt();
        let sy: f64 = pairs.iter().map(|p| (p.1 - my).powi(2)).sum::<f64>().sqrt();
        let correlation = covariance / (sx * sy);
        assert!(
            (correlation - 0.8911).abs() < 5e-4,
            "the two instruments correlate at {correlation:.4}"
        );
    }

    /// The exposure, in both directions, and the floor's part in it.
    #[test]
    fn re_basing_to_the_resident_count_moves_it_in_both_directions() {
        let exposure = exposure(Basis::Resident);
        assert_eq!(exposure.gainers, 385);
        assert_eq!(exposure.losers, 119);
        assert_eq!(exposure.leave_the_floor, 33);
        assert!(
            (exposure.gain / 1e6 - 899.9).abs() < 0.1,
            "gain {}",
            exposure.gain
        );
        assert!(
            (exposure.loss / 1e6 - 57.5).abs() < 0.1,
            "loss {}",
            exposure.loss
        );
        assert!(
            (exposure.net() / 1e6 - 842.5).abs() < 0.1,
            "net {}",
            exposure.net()
        );
    }

    /// And the plan's own alternative denominator moves almost nothing.
    #[test]
    fn the_open_enrolment_adjustment_is_not_where_the_money_is() {
        let exposure = exposure(Basis::OpenEnrolmentAdjusted);
        assert!(
            exposure.net().abs() < 25e6,
            "the open-enrolment adjustment moves {:.1}m, against the resident count's 842.5m",
            exposure.net() / 1e6
        );
        assert_eq!(
            exposure.leave_the_floor, 9,
            "nine districts leave the 10% floor on the plan's own adjustment, against 33 on the \
             resident count"
        );
    }

    /// Two thirds of the weighted categorical entitlement is taken by the local share, and the
    /// community and STEM school unit is charged none of it.
    #[test]
    fn the_local_share_takes_two_thirds_of_the_weighted_categoricals() {
        let (entitlement, paid) = weighted_categorical_entitlement();
        assert!((entitlement / 1e9 - 2.4323).abs() < 1e-3, "{entitlement}");
        assert!((paid / 1e9 - 0.8125).abs() < 1e-3, "{paid}");
        let taken = (entitlement - paid) / entitlement;
        assert!(
            (taken - 0.666).abs() < 1e-3,
            "the local share takes {taken:.4}"
        );
    }

    /// Re-basing is two operations, and only one of them is a correction anyone has argued for.
    ///
    /// The $842.5m divides exactly: **$659.1m** of relief on the local charge itself, which is
    /// what #384 priced and said nobody should want, and **$183.4m** in the categorical channel,
    /// where the statute's own arithmetic does not cancel. The sum is the check — if these were
    /// two views of one quantity rather than two halves of it, they would not add up.
    #[test]
    fn the_exposure_is_two_operations_and_only_one_is_arguable() {
        let split = decompose(Basis::Resident);
        assert!(
            (split.charge_relief / 1e6 - 659.1).abs() < 0.1,
            "charge relief {}",
            split.charge_relief
        );
        assert!(
            (split.categorical / 1e6 - 183.4).abs() < 0.1,
            "categorical {}",
            split.categorical
        );
        assert!(
            (split.total() - exposure(Basis::Resident).net()).abs() < 1.0,
            "the two halves come to {} and the exposure is {}. They are the same counterfactual \
             priced two ways, so a gap here means one of them is measuring something else.",
            split.total(),
            exposure(Basis::Resident).net()
        );
    }

    /// Confining the correction to the categoricals costs a fifth as much and harms a tenth as
    /// much.
    ///
    /// $183.4m against the wholesale $842.5m — and the loss side falls from $57.5m to **$7.8m**,
    /// by a factor of 7.4, because the 119 net open-enrolment-in districts lose through the
    /// charge and not through the percentage. The caveat #384 attached to the wholesale figure —
    /// that a correction priced on the gainers alone does not see the losers — very nearly
    /// dissolves once the correction is confined to the channel that does not cancel.
    #[test]
    fn the_correction_confined_to_the_categoricals_is_a_fifth_of_the_wholesale_one() {
        let confined = categorical_exposure(Basis::Resident);
        let wholesale = exposure(Basis::Resident);

        assert_eq!(confined.gainers, 385);
        assert_eq!(confined.losers, 119);
        assert_eq!(
            confined.leave_the_floor, 33,
            "the floor is what lifts a percentage that the charge cannot move"
        );
        assert!(
            (confined.gain / 1e6 - 191.2).abs() < 0.1,
            "gain {}",
            confined.gain
        );
        assert!(
            (confined.loss / 1e6 - 7.8).abs() < 0.1,
            "loss {}",
            confined.loss
        );
        assert!(
            (confined.net() / 1e6 - 183.4).abs() < 0.1,
            "net {}",
            confined.net()
        );
        assert!(
            wholesale.loss / confined.loss > 7.0,
            "the wholesale loss is {:.1} times the confined one, and the point of confining it is \
             that the losers lose through the charge rather than through the percentage",
            wholesale.loss / confined.loss
        );
    }

    /// The instrument H.B. 96 repealed was the size of the problem for the districts it reached,
    /// and reached a fifteenth of them.
    ///
    /// Supplemental targeted assistance paid **$52.5m to 36 districts** in FY2025 — H.B. 110's own
    /// answer to a district whose count excludes the children it has lost, gated on an FY2019
    /// enrolled ADM below 88% of total ADM and a wealth index above 1.6. A correction confined to
    /// the categoricals would give those same 36 districts **$48.7m**, within 8% of it.
    ///
    /// But it would give **$142.6m** to districts the supplement never covered, because the
    /// wealth gate excluded them. Columbus City teaches 65.6% of its resident children and would
    /// take $41.7m on its own — more than three quarters of what the supplement paid everyone —
    /// and its FY2019 wealth index is 1.197, under the 1.6 threshold.
    #[test]
    fn the_repealed_supplement_was_gated_on_a_wealth_test_that_excluded_the_largest_exposure() {
        let (covered, uncovered, qualifiers) = against_the_repealed_supplement(Basis::Resident);
        assert_eq!(qualifiers, 36, "H.B. 110's own two FY2019 tests");
        assert!(
            (covered / 1e6 - 48.7).abs() < 0.1,
            "to the 36 the supplement covered: {covered}"
        );
        assert!(
            (uncovered / 1e6 - 142.6).abs() < 0.1,
            "to the districts it never covered: {uncovered}"
        );
        assert!(
            uncovered > covered * 2.0,
            "the correction reaches {:.1}m outside the supplement's 36 districts against {:.1}m \
             inside it. An instrument aimed at this effect that misses three quarters of it is \
             aimed at something else as well.",
            uncovered / 1e6,
            covered / 1e6
        );

        let assistance: BTreeMap<String, _> = panel()
            .iter()
            .map(|record| (record.irn.clone(), record.targeted_assistance))
            .collect();
        let columbus = &assistance["043802"];
        assert!(
            columbus.fy19_enrolled_adm < TA_SUPPLEMENT_RETENTION * columbus.fy19_total_adm,
            "Columbus passes the count test at {:.4}",
            columbus.fy19_enrolled_adm / columbus.fy19_total_adm
        );
        assert_eq!(
            columbus.qualifies(),
            Some(false),
            "and fails the supplement on the wealth test alone, at an FY2019 index of {:.4}",
            columbus.fy19_wealth_index
        );
    }

    /// The wealth tier reproduces from the statute before any counterfactual is run on it.
    ///
    /// The guard on the measurement below, and it was earned: a first attempt divided *and*
    /// multiplied by the adjusted count and came out **38 times** the published figure, which is
    /// large enough to look like a finding rather than a mistake. `[D]` divides by the adjusted
    /// count and `[F]` multiplies by enrolled ADM, and the two are one line apart in the section.
    #[test]
    fn the_wealth_tier_reproduces_before_it_is_re_based() {
        let mut worst: f64 = 0.0;
        let mut checked = 0;
        for record in &panel() {
            let assistance = &record.targeted_assistance;
            if assistance.wealth_amount <= 1.0 {
                continue;
            }
            let enrolled = record.categorical_enrolled_adm;
            let computed = wealth_tier(
                assistance.weighted_wealth,
                assistance.resident_adm(enrolled),
                enrolled,
            );
            worst =
                worst.max((computed - assistance.wealth_amount).abs() / assistance.wealth_amount);
            checked += 1;
        }
        assert!(
            checked > 400,
            "only {checked} districts carry a wealth tier"
        );
        assert!(
            worst < 1e-5,
            "the wealth tier reconstruction is off by {worst:.3e} at worst. Until it reproduces \
             the department's own [F], nothing computed from it means anything — and the failure \
             mode is silent: using the adjusted count as the multiplier as well as the divisor \
             overstates the tier by a factor of 38 and still returns a plausible-looking number."
        );
    }

    /// Targeted assistance carries more of the denominator than the categoricals do.
    ///
    /// **$433.1m**, against the $183.4m the four categoricals carry — and it lands almost entirely
    /// on one side: $433.4m to 489 districts against $0.3m off 7, because only the divisor moves
    /// and the multiplier stays enrolled ADM.
    ///
    /// This is the component that already concedes the principle. R.C. 3317.0217(C)(1) divides by
    /// a residence-flavoured count *by design*; the question it leaves is only how far that count
    /// reaches, and the answer is one of nine channels and 2,098 pupils against 222,923.
    #[test]
    fn the_component_that_already_uses_a_resident_denominator_carries_the_most() {
        let (statute, rebased, gainers) = targeted_assistance_on_the_resident_count();
        assert!(
            (statute / 1e6 - 1030.3).abs() < 0.2,
            "the wealth tier on the statute's own count is {statute}"
        );
        assert!(
            (rebased / 1e6 - 1463.4).abs() < 0.2,
            "and on the resident count {rebased}"
        );
        assert_eq!(gainers, 489);

        let moved = rebased - statute;
        assert!((moved / 1e6 - 433.1).abs() < 0.2, "moved {moved}");
        assert!(
            moved > categorical_exposure(Basis::Resident).net() * 2.0,
            "targeted assistance carries {:.1}m against the categoricals' {:.1}m. The channel the \
             plan already equalises on a resident count is the one the denominator reaches \
             furthest into, which is the opposite of what a reading confined to R.C. 3317.022 sees.",
            moved / 1e6,
            categorical_exposure(Basis::Resident).net() / 1e6
        );
    }

    /// The divergence orders nothing, which is the hypothesis's own falsifier met.
    #[test]
    fn the_divergence_does_not_order_who_is_charged() {
        let correlation = divergence_correlation();
        assert!(
            correlation.abs() < 0.10,
            "the resident-to-funded ratio explains the state share percentage at {correlation:.4}, \
             which would make the denominator the thing that sets it"
        );
        let districts = frame();
        let youngstown = districts
            .iter()
            .find(|d| d.irn == "045161")
            .expect("in the model");
        let columbus = districts
            .iter()
            .find(|d| d.irn == "043802")
            .expect("in the model");
        let widest = districts
            .iter()
            .max_by(|a, b| a.resident_ratio().total_cmp(&b.resident_ratio()))
            .expect("the frame is not empty");
        assert_eq!(
            widest.irn, youngstown.irn,
            "the widest divergence is {}",
            widest.name
        );
        assert!(youngstown.resident_ratio() > columbus.resident_ratio());
        assert!(
            youngstown
                .state_share_on(Basis::BaseCostEnrolled)
                .percentage
                > columbus.state_share_on(Basis::BaseCostEnrolled).percentage,
            "the district with the larger divergence has the larger state share, which is the \
             opposite of what the hypothesis predicts"
        );
    }

    /// Every district joins, and the three denominators are three different numbers.
    #[test]
    fn the_frame_carries_all_three_counts_for_every_district() {
        let districts = frame();
        assert_eq!(districts.len(), 609);
        for district in &districts {
            assert!(district.base_cost_enrolled_adm > 0.0);
            assert!(district.resident_adm > 0.0);
            assert!(district.open_enrolment_adjusted_adm > 0.0);
        }
        let differ = districts
            .iter()
            .filter(|d| (d.resident_adm - d.base_cost_enrolled_adm).abs() > 1.0)
            .count();
        assert!(
            differ > 500,
            "only {differ} districts' counts differ by a pupil"
        );
    }
}
