//! One district as the department modelled it, and what its own figures imply.
//!
//! [`DistrictRecord`] is the join of everything the two component files compute: the published
//! amounts, the inputs behind them, and the handful of derived quantities that are only
//! recoverable from a whole record rather than from any one component.
//!
//! The derivations are where the epistemic care sits. Several of them return [`Option`] because
//! the quantity is *censored* rather than merely absent —
//! [`DistrictRecord::implied_local_capacity_per_pupil`] is the case that motivated the pattern.

use edfund_core::{Adm, Dollars};
use foundation::DistrictEnrollment;

use crate::panel::categoricals::{
    CareerTechnical, Categoricals, Dpia, EnglishLearners, Gifted, SpecialEducation,
    TargetedAssistance,
};
use crate::panel::supplements::{
    PerformanceSupplement, PreschoolSpecialEducation, Supplements, Transition, Transportation,
};
use crate::panel::{HISTORY_YEARS, MINIMUM_STATE_SHARE};
use crate::series::Observation;

/// One district as the department modelled it.
#[derive(Debug, Clone, PartialEq)]
pub struct DistrictRecord {
    /// Information Retrieval Number, the stable statewide identifier.
    pub irn: String,
    /// District name as published.
    pub name: String,
    /// Grade-band ADM and building count, ready for [`foundation`].
    pub enrollment: DistrictEnrollment,
    /// Funded classroom teachers, as the department computed them.
    pub funded_classroom_teachers: f64,
    /// Funded special teachers, as the department computed them.
    pub funded_special_teachers: f64,
    /// Aggregate base cost, all five sub-components.
    pub aggregate_base_cost: Dollars,
    /// Aggregate base cost divided by base cost enrolled ADM.
    pub base_cost_per_pupil: Dollars,
    /// The state's share of base cost alone, before every categorical.
    pub base_cost_state_share: Dollars,
    /// Base cost share plus targeted assistance, special education, DPIA, English learner,
    /// gifted, and career-technical. This is formula aid.
    pub core_foundation_funding: Dollars,
    /// `[R] Total State Support` — every state payment the report carries.
    ///
    /// Wider than [`DistrictRecord::realized_aid`], which is core foundation funding plus the
    /// guarantee. The difference is transportation, preschool special education, special
    /// education transportation, and the performance supplement: real money, paid to the
    /// district, and **outside the base the guarantee holds it at**.
    ///
    /// The distinction is not pedantic. Comparing `realized_aid` against a figure that behaves
    /// like this one — a district's booked state receipts, for instance — compares a narrow
    /// construction against a broad one and produces a shortfall that is definitional rather
    /// than real. This corpus did exactly that for two phases.
    pub total_state_support: Dollars,
    /// `U - Total Transfers` — educational service center charges plus other adjustments.
    ///
    /// **Not the voucher channel.** Under the Fair School Funding Plan community and STEM
    /// students are funded directly rather than deducted from the resident district, and no line
    /// in this report carries a scholarship deduction. Cleveland Municipal's transfers are
    /// -$3.8M against $322.6M of total state support; a deduction channel would be two orders
    /// larger. Carried so that claim is checkable rather than asserted — see
    /// [`crate::finances`] and the `deduction` skill for what is still missing.
    pub total_transfers: Dollars,
    /// `S - Educational Service Center` alone — negative for 606 of 609 districts and never
    /// positive, which is what a charge looks like rather than a channel.
    pub service_center_charge: Dollars,
    /// `T - Other Adjustments` alone — the report's only unlabelled line, and therefore the whole
    /// of the space in which a voucher or community-school deduction could still be sitting.
    ///
    /// It cannot be sitting there, and this field is what settles it. The negative half totals
    /// $95.6m across 577 districts — 1.12% of total state support — against Ohio scholarship
    /// spending on the order of a billion a year. A deduction channel would have to be ten times
    /// the size of the entire residual it would have to hide in.
    ///
    /// The question was open since genesis and was previously answered by direction: transfers
    /// run both ways, so the *line* is not a deduction. That left "a deduction could be inside
    /// its negative half" unresolved, because `total_transfers` mixes the service-centre charge
    /// with the residual. Splitting them measures the residual on its own.
    pub other_adjustments: Dollars,
    /// `V - Net State Funding` — total state support after transfers.
    pub net_state_funding: Dollars,
    /// Local capacity per pupil **as the department publishes it**, from `Detail_SFPR`.
    ///
    /// [`DistrictRecord::implied_local_capacity_per_pupil`] recovers this by subtracting state
    /// aid from base cost, which works for 470 districts and is impossible for the rest: where
    /// the minimum state share binds, the subtraction gives a number smaller than the truth and
    /// the corpus returns `None` rather than a plausible wrong figure.
    ///
    /// It does not have to be recovered. The department computes it and publishes it on a sheet
    /// this repository did not read for eleven phases, for every district including the ones the
    /// subtraction censors.
    pub published_capacity_per_pupil: Option<Dollars>,
    /// `[b4] State Share Percentage`, likewise published rather than inferred.
    pub published_state_share: Option<f64>,
    /// Assessed valuation for the three tax years the capacity measure blends, newest first.
    pub valuation_three_year: [Dollars; 3],
    /// Federal adjusted gross income for its three tax years, newest first.
    pub agi_three_year: [Dollars; 3],
    /// Tax returns filed in the district, the count the median-income term multiplies.
    pub tax_returns: Option<f64>,
    /// `[I5] TY23 Federal Median Income with ADJ Factor` — the blend's third term.
    ///
    /// **Federal**, and adjusted. An earlier pass here inferred the term from the profile report's
    /// Ohio median income, because the `Local_Capacity` sheet that states all of this had not been
    /// opened. That fits approximately — well enough to look right — and is the wrong measure:
    /// Columbus's federal median is $46,395 against an Ohio median of $31,555.
    pub median_income: Option<Dollars>,
    /// `[I7] TY23 Statewide Federal Median Income` — the denominator of the income ratio.
    ///
    /// $54,546.64, published on the sheet rather than derivable. The median of district medians
    /// is $41,502 and is a different quantity.
    pub statewide_median_income: Option<Dollars>,
    /// `[C5]` — the income ratio of the 40th highest district, which tops out the sliding scale.
    ///
    /// Published rather than reconstructed. Reconstructing it from the panel gave 1.4151 against
    /// a published 1.46504, and it is a discretionary number rather than a derived one.
    pub benchmark_ratio: Option<f64>,
    /// `[C6] Local Capacity Percentage` — the rate, as the department computes it.
    pub published_capacity_rate: Option<f64>,
    /// The six categorical programs, each read rather than inferred as a lump.
    pub categoricals: Categoricals,
    /// The second-largest categorical, decomposed into the six weighted categories.
    pub special_education: SpecialEducation,
    /// The third-largest, and the one whose mechanism is neither a weight nor an equalisation.
    pub dpia: Dpia,
    /// The largest, and the only equalisation among the six.
    pub targeted_assistance: TargetedAssistance,
    /// Five weights against a career-technical base cost, plus associated services.
    pub career_technical: CareerTechnical,
    /// Three weights that descend rather than ascend.
    pub english_learners: EnglishLearners,
    /// Two per-pupil amounts and three kinds of unit, with floors no other categorical has.
    pub gifted: Gifted,
    /// `[a] Enrolled ADM` — a seventh pupil count, and the one four of the six categoricals are
    /// paid on.
    ///
    /// Not [`DistrictRecord::current_year_adm`], which is `[b3] FY26 Enrolled ADM`, and not
    /// [`DistrictRecord::base_cost_adm`], which averages three years. Akron's are 18,892.45 and
    /// 18,842.45 — fifty pupils apart on the same sheet, four columns from each other.
    ///
    /// Targeted assistance, gifted, career-technical and English learners are all computed against
    /// this one. Reproducing any of them from `current_year_adm` gets an answer that is close, and
    /// wrong, which is the failure mode that is hardest to notice.
    pub categorical_enrolled_adm: Adm,
    /// The performance supplement, and the ratings that gate it.
    pub performance: PerformanceSupplement,
    /// The base funding supplement and the enrollment growth supplement.
    pub supplements: Supplements,
    /// Transportation, and special education transportation beside it.
    pub transportation: Transportation,
    /// Preschool special education: a flat $4,000 a pupil plus the weights at half.
    pub preschool_special_education: PreschoolSpecialEducation,
    /// The guarantee's machinery, and the transition supplement stacked on it.
    pub transition: Transition,
    /// The county the department attributes the district to, from `Base_Cost`.
    ///
    /// One county per district, which is a **simplification the department makes and this corpus
    /// inherits**. School district boundaries follow historical township and municipal lines and
    /// cross county lines freely; the calculator picks one anyway, presumably the seat of the
    /// district's administrative office. So a county grouping built on this is the department's
    /// own attribution rather than a geographic fact, and a district's pupils and valuation are
    /// not all inside the county named here.
    ///
    /// That is acceptable for comparison — the question "how does my district compare with its
    /// neighbours" wants a peer group, not a polygon — and it is not acceptable for anything that
    /// sums to a county total and calls it the county's. The site does the former.
    pub county: String,
    /// Temporary transitional aid guarantee.
    pub guarantee: Dollars,
    /// Enrolled ADM in each of [`HISTORY_YEARS`].
    pub adm_history: [Adm; 3],
    /// Current-year enrolled ADM — the last of [`HISTORY_YEARS`].
    ///
    /// Distinct from base cost ADM and used as a distinct denominator: R.C. 3317.017 multiplies
    /// the state share *per pupil* by current-year enrolled ADM, while base cost is computed on
    /// the three-year figure. Dividing the state share by the wrong one turns a hard floor at
    /// exactly 10% into a smear from 9.2% upward, which is how this was found.
    pub current_year_adm: Adm,
    /// Assessed valuation per pupil, FY2023. The only observation there is.
    pub valuation_per_pupil: Option<Dollars>,
}

impl DistrictRecord {
    /// Base cost enrolled ADM — the three-year average the formula funds on.
    #[must_use]
    pub fn base_cost_adm(&self) -> Adm {
        self.enrollment.base_cost_enrolled_adm
    }

    /// Whether the district is funded by the guarantee rather than the formula.
    #[must_use]
    pub fn on_guarantee(&self) -> bool {
        self.guarantee > 0.0
    }

    /// State aid as the district actually receives it.
    #[must_use]
    pub fn realized_aid(&self) -> Dollars {
        self.core_foundation_funding + self.guarantee
    }

    /// The level the guarantee holds this district at, recoverable only when it is on it.
    ///
    /// For a district on the guarantee this is its FY2020 receipt — a Bridge formula year Ohio
    /// froze rather than computed. For a district on formula the baseline is unobservable from
    /// here, because the guarantee is the only thing that reveals it.
    #[must_use]
    pub fn guarantee_baseline(&self) -> Option<Dollars> {
        self.on_guarantee().then(|| self.realized_aid())
    }

    /// Everything in formula aid that is not base cost: the categoricals.
    ///
    /// Held apart because they respond to different levers. Raising base cost moves the first
    /// and not the second, and the FY2022 phase-in applied 16.67% to base cost and **0%** to
    /// Disadvantaged Pupil Impact Aid — so a single "phase-in percentage" is already known to
    /// be the wrong shape for this question.
    #[must_use]
    pub fn categorical_funding(&self) -> Dollars {
        (self.core_foundation_funding - self.base_cost_state_share).max(0.0)
    }

    /// The part of [`Self::categorical_funding`] priced in the statewide average base cost.
    ///
    /// Special education, English learners and career-technical are each
    /// `weight x $8,241.61 x count x state share`, where the multiplicand is
    /// [`super::categoricals::AVERAGE_BASE_COST_PER_PUPIL`] — one number the department computes over every district.
    /// **So a change in base cost per pupil moves these mechanically**, and a scenario that scales
    /// base cost and leaves them alone is not describing anything Ohio would do.
    ///
    /// # What is deliberately not in here
    ///
    /// **DPIA and targeted assistance**, which are the two largest categoricals at $1.89bn
    /// together. They are *index*-driven: a district's poverty share over the state's, its wealth
    /// over a median. Under a change that moves every district, numerator and denominator move
    /// together and the effect largely cancels. That is why the exposure is a quarter of base cost
    /// rather than three quarters, and it is a real property of the mechanisms rather than a
    /// simplification.
    ///
    /// **Gifted**, whose $54.4m is mostly unit funding at stated salary prices — $85,776 for a
    /// coordinator, $89,378 and $80,974 for the two specialist bands. A cost-input refresh that
    /// raised teacher salaries would raise these too, but by their own amount and not by this
    /// factor: nothing in the calculator links them. Scaling them here would be an assumption
    /// wearing the same clothes as an identity.
    ///
    /// **Preschool special education's weighted half**, $45.7m, which *is* denominated the same
    /// way — but sits outside `[H] Foundation Funding` and so outside
    /// [`Self::categorical_funding`] entirely. The scenario has never modelled it. It is the
    /// difference between the $858m the corpus recorded as exposed and the $812m reachable here.
    #[must_use]
    pub fn base_cost_denominated_categoricals(&self) -> Dollars {
        self.special_education.total()
            + self.english_learners.total()
            + self.career_technical.total()
    }

    /// The state's share of base cost, as a fraction of base cost per pupil.
    ///
    /// Per pupil on both sides, with the department's denominators: the share amount is on
    /// current-year enrolled ADM, base cost per pupil is on the three-year figure.
    #[must_use]
    pub fn state_share_fraction(&self) -> f64 {
        if self.base_cost_per_pupil <= 0.0 || self.current_year_adm <= 0.0 {
            return 0.0;
        }
        (self.base_cost_state_share / self.current_year_adm) / self.base_cost_per_pupil
    }

    /// Whether the minimum state share is what sets this district's base cost aid.
    #[must_use]
    pub fn at_minimum_state_share(&self) -> bool {
        (self.state_share_fraction() - MINIMUM_STATE_SHARE).abs() < 0.0005
    }

    /// Local capacity per pupil, implied by what the state pays.
    ///
    /// The formula is `state share per pupil = base cost per pupil − local capacity per pupil`,
    /// so local capacity is recoverable by subtraction — **except** where the floor binds,
    /// which is precisely where local capacity is highest and the subtraction would give a
    /// number smaller than the truth. `None` there rather than a plausible wrong figure.
    #[must_use]
    pub fn implied_local_capacity_per_pupil(&self) -> Option<Dollars> {
        (!self.at_minimum_state_share() && self.current_year_adm > 0.0)
            .then(|| self.base_cost_per_pupil - self.base_cost_state_share / self.current_year_adm)
    }

    /// Enrolled ADM as a series, for [`crate::series::project`].
    #[must_use]
    pub fn adm_observations(&self) -> Vec<Observation> {
        HISTORY_YEARS
            .iter()
            .zip(self.adm_history)
            .map(|(fiscal_year, value)| Observation {
                fiscal_year: *fiscal_year,
                value,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::panel;
    use edfund_core::FiscalYear;

    #[test]
    fn formula_aid_splits_into_base_cost_and_categoricals() {
        for record in panel() {
            let recombined = record.base_cost_state_share + record.categorical_funding();
            assert!(
                (recombined - record.core_foundation_funding).abs() < 0.01,
                "{}: {recombined} vs {}",
                record.name,
                record.core_foundation_funding
            );
        }
    }

    #[test]
    fn the_minimum_state_share_binds_for_the_wealthiest_districts() {
        let panel = panel();
        let at_floor: Vec<&DistrictRecord> = panel
            .iter()
            .filter(|r| r.at_minimum_state_share())
            .collect();
        assert_eq!(
            at_floor.len(),
            138,
            "the FY2027 model puts 138 districts on the minimum state share"
        );
        // And nothing below it: a floor, not a tendency.
        let lowest = panel
            .iter()
            .map(DistrictRecord::state_share_fraction)
            .filter(|f| *f > 0.0)
            .fold(f64::MAX, f64::min);
        assert!(
            lowest >= MINIMUM_STATE_SHARE - 0.0005,
            "a district below the floor at {lowest}"
        );
        // Wealth is why they are there: every one should be above the statewide median.
        let mut valuations: Vec<f64> = panel.iter().filter_map(|r| r.valuation_per_pupil).collect();
        valuations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = valuations[valuations.len() / 2];
        let above = at_floor
            .iter()
            .filter(|r| r.valuation_per_pupil.is_some_and(|v| v > median))
            .count();
        assert!(
            above * 4 >= at_floor.len() * 3,
            "only {above} of {} floor districts are above median wealth",
            at_floor.len()
        );
    }

    #[test]
    fn local_capacity_is_not_recovered_where_the_floor_binds() {
        // Subtraction gives a number there, and the number is wrong: the floor has already
        // truncated what it is derived from.
        for record in panel() {
            if record.at_minimum_state_share() {
                assert_eq!(
                    record.implied_local_capacity_per_pupil(),
                    None,
                    "{}",
                    record.name
                );
            }
        }
    }

    #[test]
    fn implied_local_capacity_is_positive_and_below_base_cost_where_it_is_recoverable() {
        for record in panel() {
            let Some(capacity) = record.implied_local_capacity_per_pupil() else {
                continue;
            };
            assert!(
                capacity > -1.0 && capacity < record.base_cost_per_pupil,
                "{}: {capacity}",
                record.name
            );
        }
    }

    #[test]
    fn a_guarantee_baseline_exists_only_for_a_guaranteed_district() {
        let panel = panel();
        for record in &panel {
            assert_eq!(record.guarantee_baseline().is_some(), record.on_guarantee());
        }
        assert!(panel.iter().filter(|r| r.on_guarantee()).count() > 250);
    }

    #[test]
    fn the_adm_history_reads_as_a_dated_series() {
        let record = &panel()[0];
        let observations = record.adm_observations();
        assert_eq!(observations.len(), 3);
        assert_eq!(observations[0].fiscal_year, FiscalYear(2024));
        assert_eq!(observations[2].fiscal_year, FiscalYear(2026));
    }
}
