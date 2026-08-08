//! The district panel: the department's FY2027 model, as records.
//!
//! Every district Ohio funds, with what the department computed for it. The fixture is embedded
//! at compile time, so this is pure and deterministic like the calculators — no filesystem, no
//! clock, no network — and a projection run years from now reads the same numbers.
//!
//! # What the columns mean, and one that was mislabelled
//!
//! `enrolled_adm_fy24/25/26` are the three years R.C. 3317.011 averages to get base cost
//! enrolled ADM for FY2027. The department's `Base_Cost` sheet labels the same three columns
//! FY22/FY23/FY24 and is stale; the earlier version of this fixture inherited that, so every
//! enrollment-trend figure in the corpus was named for the wrong pair of years. See
//! `crates/connect/src/fixtures.rs`.
//!
//! **FY2026 is not fully an actual.** The calculator is published in December 2025, before that
//! fiscal year closes, so the last observation in every district's history is partly a
//! departmental estimate. A projection that starts from it inherits that, and the label
//! "observed" is doing some work it should not have to do alone.

use edfund_core::{Adm, Dollars, FiscalYear};
use foundation::DistrictEnrollment;

use crate::series::Observation;

/// The department's FY2027 funding model, one row per district.
const FIXTURE: &str = include_str!("../../foundation/fixtures/fy27-department-model.csv");

/// The fiscal year the model computes.
pub const MODEL_YEAR: FiscalYear = FiscalYear(2027);

/// The three fiscal years of enrolled ADM the model averages.
pub const HISTORY_YEARS: [FiscalYear; 3] = [FiscalYear(2024), FiscalYear(2025), FiscalYear(2026)];

/// The minimum state share of base cost operative in the FY2027 model.
///
/// The department's `Notes` sheet states `0.1` for FY2026 and FY2027. It is **not** the 5% the
/// Fair School Funding Plan was enacted with; each biennial budget sets it, and it doubled.
/// 138 of 609 districts sit exactly on it.
pub const MINIMUM_STATE_SHARE: f64 = local_capacity::MINIMUM_STATE_SHARE_FY2027;

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

/// Column positions in the fixture. One place to edit when the header changes.
mod column {
    pub const IRN: usize = 0;
    pub const NAME: usize = 1;
    pub const BASE_COST_ADM: usize = 2;
    pub const BUILDINGS: usize = 3;
    pub const KINDERGARTEN: usize = 4;
    pub const GRADES_1_3: usize = 5;
    pub const GRADES_4_8: usize = 6;
    pub const GRADES_9_12: usize = 7;
    pub const CAREER_TECHNICAL: usize = 8;
    pub const GRADES_9_12_TOTAL: usize = 9;
    pub const FUNDED_CLASSROOM: usize = 10;
    pub const FUNDED_SPECIAL: usize = 11;
    pub const AGGREGATE_BASE_COST: usize = 13;
    pub const BASE_COST_PER_PUPIL: usize = 14;
    pub const GUARANTEE: usize = 15;
    pub const ADM_FY24: usize = 16;
    pub const ADM_FY25: usize = 17;
    pub const ADM_FY26: usize = 18;
    pub const VALUATION_PER_PUPIL: usize = 19;
    pub const CORE_FOUNDATION: usize = 20;
    pub const BASE_COST_STATE_SHARE: usize = 21;
    pub const TOTAL_STATE_SUPPORT: usize = 22;
    pub const TOTAL_TRANSFERS: usize = 23;
    /// `S - Educational Service Center`, split out of the transfer total.
    pub const SERVICE_CENTER: usize = 24;
    /// `T - Other Adjustments`, the residual and the only unlabelled line in the report.
    pub const OTHER_ADJUSTMENTS: usize = 25;
    pub const NET_STATE_FUNDING: usize = 26;
}

/// The header this loader expects, so a fixture reshaped without updating [`column`] fails
/// loudly rather than reading the wrong column.
const EXPECTED_HEADER: &str = "irn,district,base_cost_enrolled_adm,school_buildings,\
adm_kindergarten,adm_grades_1_3,adm_grades_4_8_non_cte,adm_grades_9_12_non_cte,adm_cte,\
adm_grades_9_12_total,funded_classroom_teachers,funded_special_teachers,teacher_base_cost,\
aggregate_base_cost,base_cost_per_pupil,temp_transitional_aid_guarantee,enrolled_adm_fy24,\
enrolled_adm_fy25,enrolled_adm_fy26,assessed_valuation_per_pupil_fy23,core_foundation_funding,\
base_cost_state_share,total_state_support,total_transfers,service_center_charge,other_adjustments,net_state_funding";

/// Every district in the department's FY2027 model.
///
/// # Panics
///
/// If the embedded fixture's header is not the one this loader was written against. That is a
/// build-time mistake, not a runtime condition, and reading shifted columns silently would put
/// wrong numbers into a scenario.
#[must_use]
pub fn panel() -> Vec<DistrictRecord> {
    let mut lines = FIXTURE.lines();
    let header = lines.next().unwrap_or_default().trim();
    assert_eq!(
        header, EXPECTED_HEADER,
        "the FY2027 fixture header changed; update project::panel::column before reading it"
    );

    lines
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let fields: Vec<&str> = line.split(',').map(str::trim).collect();
            let number = |index: usize| {
                fields
                    .get(index)
                    .and_then(|value| value.parse::<f64>().ok())
            };
            let required = |index: usize| number(index).unwrap_or(0.0);

            let base_cost_adm = number(column::BASE_COST_ADM)?;
            if base_cost_adm <= 0.0 {
                return None;
            }
            Some(DistrictRecord {
                irn: fields.get(column::IRN)?.to_string(),
                name: fields.get(column::NAME)?.to_string(),
                enrollment: DistrictEnrollment {
                    kindergarten: required(column::KINDERGARTEN),
                    grades_1_3: required(column::GRADES_1_3),
                    grades_4_8: required(column::GRADES_4_8),
                    grades_9_12: required(column::GRADES_9_12),
                    career_technical: required(column::CAREER_TECHNICAL),
                    grades_9_12_total: required(column::GRADES_9_12_TOTAL),
                    base_cost_enrolled_adm: base_cost_adm,
                    open_buildings: required(column::BUILDINGS),
                    athletics_eligible: true,
                },
                funded_classroom_teachers: required(column::FUNDED_CLASSROOM),
                funded_special_teachers: required(column::FUNDED_SPECIAL),
                aggregate_base_cost: required(column::AGGREGATE_BASE_COST),
                base_cost_per_pupil: required(column::BASE_COST_PER_PUPIL),
                base_cost_state_share: required(column::BASE_COST_STATE_SHARE),
                core_foundation_funding: required(column::CORE_FOUNDATION),
                total_state_support: required(column::TOTAL_STATE_SUPPORT),
                total_transfers: required(column::TOTAL_TRANSFERS),
                service_center_charge: required(column::SERVICE_CENTER),
                other_adjustments: required(column::OTHER_ADJUSTMENTS),
                net_state_funding: required(column::NET_STATE_FUNDING),
                guarantee: required(column::GUARANTEE),
                adm_history: [
                    required(column::ADM_FY24),
                    required(column::ADM_FY25),
                    required(column::ADM_FY26),
                ],
                current_year_adm: required(column::ADM_FY26),
                valuation_per_pupil: number(column::VALUATION_PER_PUPIL),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_every_district_in_the_model() {
        assert_eq!(panel().len(), 609);
    }

    #[test]
    fn base_cost_adm_is_the_greater_of_the_three_year_average_and_the_current_year() {
        // Not the plain average. For 105 of 609 districts the published figure is the current
        // year instead, and in every one of those the current year is the larger.
        //
        // The asymmetry is the mechanism: a growing district is funded on this year's students
        // immediately, and a shrinking one keeps two years of students it no longer has. Both
        // directions are cushioned in the same direction, toward the district.
        let panel = panel();
        let mut on_current_year = 0;
        for record in &panel {
            let average = record.adm_history.iter().sum::<f64>() / 3.0;
            let expected = average.max(record.current_year_adm);
            assert!(
                (expected - record.base_cost_adm()).abs() < 0.01,
                "{}: max(avg {average:.4}, FY26 {:.4}) vs published {:.4}",
                record.name,
                record.current_year_adm,
                record.base_cost_adm()
            );
            if record.current_year_adm > average + 0.01 {
                on_current_year += 1;
            }
        }
        assert_eq!(on_current_year, 105);
    }

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
