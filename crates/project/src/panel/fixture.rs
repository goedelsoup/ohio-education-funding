//! The fixture, the header it must have, and the only code that knows a column number.
//!
//! Nothing else in [`super`] refers to a column index. That is the point of the file: the
//! arithmetic in [`super::categoricals`] and [`super::supplements`] is about weights and rates,
//! and if it could reach the column table it could read the wrong one.

use foundation::DistrictEnrollment;

use crate::panel::categoricals::{
    CareerTechnical, Categoricals, Dpia, EnglishLearners, Gifted, SpecialEducation,
    TargetedAssistance,
};
use crate::panel::record::DistrictRecord;
use crate::panel::supplements::{
    PerformanceSupplement, PreschoolSpecialEducation, Supplements, Transition, Transportation,
};

/// The department's FY2027 funding model, one row per district.
const FIXTURE: &str = include_str!("../../../foundation/fixtures/fy27-department-model.csv");

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
    /// `[b1] Per Pupil Capacity Amount` from `Detail_SFPR` — the department's own local capacity.
    pub const CAPACITY_PER_PUPIL: usize = 27;
    /// `[b4] State Share Percentage`.
    pub const STATE_SHARE_PERCENTAGE: usize = 28;
    pub const VALUATION_TY25: usize = 29;
    pub const VALUATION_TY24: usize = 30;
    pub const VALUATION_TY23: usize = 31;
    pub const AGI_TY24: usize = 32;
    pub const AGI_TY23: usize = 33;
    pub const AGI_TY22: usize = 34;
    pub const TAX_RETURNS: usize = 35;
    pub const FEDERAL_MEDIAN_INCOME: usize = 36;
    pub const STATEWIDE_MEDIAN_INCOME: usize = 37;
    pub const BENCHMARK_RATIO: usize = 38;
    pub const PUBLISHED_CAPACITY_RATE: usize = 39;
    pub const TARGETED_ASSISTANCE: usize = 40;
    pub const SPECIAL_EDUCATION: usize = 41;
    pub const DPIA: usize = 42;
    pub const ENGLISH_LEARNERS: usize = 43;
    pub const GIFTED: usize = 44;
    pub const CATEGORICAL_CTE: usize = 45;
    /// Six special education ADM counts, then the six aid amounts they produce.
    pub const SPED_FIRST_ADM: usize = 46;
    pub const SPED_FIRST_AID: usize = 52;
    pub const DPIA_ECON_DISADVANTAGED_ADM: usize = 58;
    pub const DPIA_DIRECTLY_CERTIFIED_ADM: usize = 59;
    pub const DPIA_WEIGHTED_ADM: usize = 60;
    pub const DPIA_PERCENTAGE: usize = 61;
    pub const DPIA_INDEX: usize = 62;
    /// Five career-technical FTE counts, then the five aid amounts, then associated services.
    pub const CTE_FIRST_FTE: usize = 63;
    pub const CTE_FIRST_AID: usize = 68;
    pub const CTE_ASSOCIATED_SERVICES: usize = 73;
    /// Three English learner ADM counts, then the three aid amounts.
    pub const EL_FIRST_ADM: usize = 74;
    pub const EL_FIRST_AID: usize = 77;
    pub const GIFTED_ADM_K6: usize = 80;
    pub const GIFTED_FTE_K8: usize = 81;
    pub const GIFTED_FTE_9_12: usize = 82;
    pub const GIFTED_IDENTIFICATION: usize = 83;
    pub const GIFTED_REFERRAL: usize = 84;
    pub const GIFTED_PROFESSIONAL_DEVELOPMENT: usize = 85;
    pub const GIFTED_COORDINATOR_UNITS: usize = 86;
    pub const GIFTED_COORDINATOR_AID: usize = 87;
    pub const GIFTED_SPECIALIST_K8_UNITS: usize = 88;
    pub const GIFTED_SPECIALIST_K8_AID: usize = 89;
    pub const GIFTED_SPECIALIST_9_12_UNITS: usize = 90;
    pub const GIFTED_SPECIALIST_9_12_AID: usize = 91;
    pub const TA_OPEN_ENROLLMENT_IN: usize = 92;
    pub const TA_OPEN_ENROLLMENT_OUT: usize = 93;
    pub const TA_FY19_WEALTH_INDEX: usize = 94;
    pub const TA_FY19_ENROLLED_ADM: usize = 95;
    pub const TA_FY19_TOTAL_ADM: usize = 96;
    pub const TA_PROPERTY_VALUATION: usize = 97;
    pub const TA_FEDERAL_GROSS_INCOME: usize = 98;
    pub const TA_WEIGHTED_WEALTH: usize = 99;
    pub const TA_CAPACITY_INDEX: usize = 100;
    pub const TA_CAPACITY_AMOUNT: usize = 101;
    pub const TA_WEALTH_PER_PUPIL: usize = 102;
    pub const TA_WEALTH_INDEX: usize = 103;
    pub const TA_WEALTH_AMOUNT: usize = 104;
    pub const TA_SUPPLEMENTAL: usize = 105;
    pub const TA_SUPPLEMENT_ELIGIBLE: usize = 106;
    /// `[a] Enrolled ADM` — the count the four categorical sheets are paid on.
    pub const CATEGORICAL_ENROLLED_ADM: usize = 107;
    pub const COUNTY: usize = 108;
    pub const PERFORMANCE_STARS: usize = 109;
    pub const PERFORMANCE_PROGRESS: usize = 110;
    pub const PERFORMANCE_PROGRESS_PRIOR: usize = 111;
    pub const PERFORMANCE_ELIGIBLE: usize = 112;
    pub const PERFORMANCE_SUPPLEMENT: usize = 113;
    pub const BASE_FUNDING_SUPPLEMENT: usize = 114;
    pub const ADM_FY23: usize = 115;
    pub const ENROLLMENT_CHANGE_THREE_YEAR: usize = 116;
    pub const GROWTH_SUPPLEMENT_ELIGIBLE: usize = 117;
    pub const ENROLLMENT_GROWTH_SUPPLEMENT: usize = 118;
    /// Transportation: eleven inputs then nine payments, in sheet order.
    pub const TRANS_FIRST_INPUT: usize = 119;
    pub const TRANS_FIRST_PAYMENT: usize = 132;
    /// The guarantee's machinery, emitted before the preschool block.
    pub const FUNDING_BASE: usize = 141;
    pub const FUNDING_BASE_ECON_DIS: usize = 142;
    pub const OPEN_ENROLLMENT_PRIOR: usize = 143;
    pub const OPEN_ENROLLMENT_CURRENT: usize = 144;
    pub const OPEN_ENROLLMENT_THRESHOLD: usize = 145;
    pub const OPEN_ENROLLMENT_ADJUSTMENT: usize = 146;
    pub const FY21_FUNDING_BASE: usize = 147;
    pub const FORMULA_TRANSITION_SUPPLEMENT: usize = 148;
    /// Preschool special education: six counts, six amounts, and their total.
    pub const PREK_FIRST_ADM: usize = 149;
    pub const PREK_FIRST_AID: usize = 155;
    pub const PREK_TOTAL: usize = 161;
}

/// The header this loader expects, so a fixture reshaped without updating [`column`] fails
/// loudly rather than reading the wrong column.
///
/// Shared with [`crate::crosswalk`], which reads the same fixture for its IRN column. One
/// constant per fixture rather than one per reader: two copies could drift apart, and the
/// stale one would keep asserting a header the file no longer has.
pub(crate) const EXPECTED_HEADER: &str = "irn,district,base_cost_enrolled_adm,school_buildings,\
adm_kindergarten,adm_grades_1_3,adm_grades_4_8_non_cte,adm_grades_9_12_non_cte,adm_cte,\
adm_grades_9_12_total,funded_classroom_teachers,funded_special_teachers,teacher_base_cost,\
aggregate_base_cost,base_cost_per_pupil,temp_transitional_aid_guarantee,enrolled_adm_fy24,\
enrolled_adm_fy25,enrolled_adm_fy26,assessed_valuation_per_pupil_fy23,core_foundation_funding,\
base_cost_state_share,total_state_support,total_transfers,service_center_charge,other_adjustments,net_state_funding,capacity_per_pupil,state_share_percentage,valuation_ty25,valuation_ty24,valuation_ty23,agi_ty24,agi_ty23,agi_ty22,tax_returns,federal_median_income,statewide_median_income,benchmark_ratio,capacity_rate,targeted_assistance,special_education,dpia,english_learners,gifted,career_technical,sped_adm_cat1,sped_adm_cat2,sped_adm_cat3,sped_adm_cat4,sped_adm_cat5,sped_adm_cat6,sped_aid_cat1,sped_aid_cat2,sped_aid_cat3,sped_aid_cat4,sped_aid_cat5,sped_aid_cat6,dpia_econ_disadvantaged_adm,dpia_directly_certified_adm,dpia_weighted_adm,dpia_percentage,dpia_index,\
cte_fte_cat1,cte_fte_cat2,cte_fte_cat3,cte_fte_cat4,cte_fte_cat5,\
cte_aid_cat1,cte_aid_cat2,cte_aid_cat3,cte_aid_cat4,cte_aid_cat5,cte_associated_services,\
el_adm_cat1,el_adm_cat2,el_adm_cat3,el_aid_cat1,el_aid_cat2,el_aid_cat3,\
gifted_adm_k6,gifted_fte_k8,gifted_fte_9_12,gifted_identification,gifted_referral,\
gifted_professional_development,gifted_coordinator_units,gifted_coordinator_aid,\
gifted_specialist_k8_units,gifted_specialist_k8_aid,gifted_specialist_9_12_units,\
gifted_specialist_9_12_aid,ta_open_enrollment_in,ta_open_enrollment_out,ta_fy19_wealth_index,\
ta_fy19_enrolled_adm,ta_fy19_total_adm,ta_property_valuation,ta_federal_gross_income,\
ta_weighted_wealth,ta_capacity_index,ta_capacity_amount,ta_wealth_per_pupil,ta_wealth_index,\
ta_wealth_amount,ta_supplemental,ta_supplement_eligible,categorical_enrolled_adm,county,\
performance_stars,performance_progress,performance_progress_prior,performance_eligible,\
performance_supplement,base_funding_supplement,enrolled_adm_fy23,enrollment_change_three_year,\
growth_supplement_eligible,enrollment_growth_supplement,\
trans_public_riders,trans_nonpublic_riders,trans_community_riders,trans_weighted_riders,\
trans_mass_transit_riders,trans_other_riders,trans_bus_miles,trans_assigned_buses,\
trans_rider_capacity_target,trans_efficiency_index,trans_district_density,trans_square_miles,trans_reported_sped_cost,trans_school_bus,\
trans_mass_transit,trans_other,trans_efficiency,trans_density,trans_fy21_base,trans_guarantee,\
trans_total,trans_special_education,\
funding_base,funding_base_econ_dis,open_enrollment_fte_prior,open_enrollment_fte_current,\
open_enrollment_threshold,open_enrollment_adjustment,fy21_funding_base,\
formula_transition_supplement,\
prek_sped_adm_cat1,prek_sped_adm_cat2,prek_sped_adm_cat3,prek_sped_adm_cat4,prek_sped_adm_cat5,\
prek_sped_adm_cat6,prek_sped_aid_cat1,prek_sped_aid_cat2,prek_sped_aid_cat3,prek_sped_aid_cat4,\
prek_sped_aid_cat5,prek_sped_aid_cat6,prek_sped_total";

/// Every district in the department's FY2027 model.
///
/// # Panics
///
/// If the embedded fixture's header is not the one this loader was written against. That is a
/// build-time mistake, not a runtime condition, and reading shifted columns silently would put
/// wrong numbers into a scenario.
#[must_use]
pub fn panel() -> Vec<DistrictRecord> {
    edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
        .filter_map(|row| {
            let base_cost_adm = row.num(column::BASE_COST_ADM)?;
            if base_cost_adm <= 0.0 {
                return None;
            }
            Some(DistrictRecord {
                irn: row.str(column::IRN).to_string(),
                name: row.str(column::NAME).to_string(),
                enrollment: DistrictEnrollment {
                    kindergarten: row.required(column::KINDERGARTEN),
                    grades_1_3: row.required(column::GRADES_1_3),
                    grades_4_8: row.required(column::GRADES_4_8),
                    grades_9_12: row.required(column::GRADES_9_12),
                    career_technical: row.required(column::CAREER_TECHNICAL),
                    grades_9_12_total: row.required(column::GRADES_9_12_TOTAL),
                    base_cost_enrolled_adm: base_cost_adm,
                    open_buildings: row.required(column::BUILDINGS),
                    athletics_eligible: true,
                },
                funded_classroom_teachers: row.required(column::FUNDED_CLASSROOM),
                funded_special_teachers: row.required(column::FUNDED_SPECIAL),
                aggregate_base_cost: row.required(column::AGGREGATE_BASE_COST),
                base_cost_per_pupil: row.required(column::BASE_COST_PER_PUPIL),
                base_cost_state_share: row.required(column::BASE_COST_STATE_SHARE),
                core_foundation_funding: row.required(column::CORE_FOUNDATION),
                total_state_support: row.required(column::TOTAL_STATE_SUPPORT),
                total_transfers: row.required(column::TOTAL_TRANSFERS),
                service_center_charge: row.required(column::SERVICE_CENTER),
                other_adjustments: row.required(column::OTHER_ADJUSTMENTS),
                published_capacity_per_pupil: row.num(column::CAPACITY_PER_PUPIL),
                published_state_share: row.num(column::STATE_SHARE_PERCENTAGE),
                valuation_three_year: [
                    row.required(column::VALUATION_TY25),
                    row.required(column::VALUATION_TY24),
                    row.required(column::VALUATION_TY23),
                ],
                agi_three_year: [
                    row.required(column::AGI_TY24),
                    row.required(column::AGI_TY23),
                    row.required(column::AGI_TY22),
                ],
                tax_returns: row.num(column::TAX_RETURNS),
                median_income: row.num(column::FEDERAL_MEDIAN_INCOME),
                statewide_median_income: row.num(column::STATEWIDE_MEDIAN_INCOME),
                benchmark_ratio: row.num(column::BENCHMARK_RATIO),
                published_capacity_rate: row.num(column::PUBLISHED_CAPACITY_RATE),
                categoricals: Categoricals {
                    targeted_assistance: row.required(column::TARGETED_ASSISTANCE),
                    special_education: row.required(column::SPECIAL_EDUCATION),
                    dpia: row.required(column::DPIA),
                    english_learners: row.required(column::ENGLISH_LEARNERS),
                    gifted: row.required(column::GIFTED),
                    career_technical: row.required(column::CATEGORICAL_CTE),
                },
                dpia: Dpia {
                    economically_disadvantaged_adm: row
                        .required(column::DPIA_ECON_DISADVANTAGED_ADM),
                    directly_certified_adm: row.required(column::DPIA_DIRECTLY_CERTIFIED_ADM),
                    weighted_adm: row.required(column::DPIA_WEIGHTED_ADM),
                    percentage: row.required(column::DPIA_PERCENTAGE),
                    index: row.required(column::DPIA_INDEX),
                },
                special_education: SpecialEducation {
                    adm: std::array::from_fn(|k| row.required(column::SPED_FIRST_ADM + k)),
                    aid: std::array::from_fn(|k| row.required(column::SPED_FIRST_AID + k)),
                },
                targeted_assistance: TargetedAssistance {
                    open_enrollment_in: row.required(column::TA_OPEN_ENROLLMENT_IN),
                    open_enrollment_out: row.required(column::TA_OPEN_ENROLLMENT_OUT),
                    fy19_wealth_index: row.required(column::TA_FY19_WEALTH_INDEX),
                    fy19_enrolled_adm: row.required(column::TA_FY19_ENROLLED_ADM),
                    fy19_total_adm: row.required(column::TA_FY19_TOTAL_ADM),
                    property_valuation: row.required(column::TA_PROPERTY_VALUATION),
                    federal_gross_income: row.required(column::TA_FEDERAL_GROSS_INCOME),
                    weighted_wealth: row.required(column::TA_WEIGHTED_WEALTH),
                    capacity_index: row.required(column::TA_CAPACITY_INDEX),
                    capacity_amount: row.required(column::TA_CAPACITY_AMOUNT),
                    wealth_per_pupil: row.required(column::TA_WEALTH_PER_PUPIL),
                    wealth_index: row.required(column::TA_WEALTH_INDEX),
                    wealth_amount: row.required(column::TA_WEALTH_AMOUNT),
                    supplemental: row.required(column::TA_SUPPLEMENTAL),
                    supplement_eligible: row.required(column::TA_SUPPLEMENT_ELIGIBLE) > 0.5,
                },
                career_technical: CareerTechnical {
                    fte: std::array::from_fn(|k| row.required(column::CTE_FIRST_FTE + k)),
                    aid: std::array::from_fn(|k| row.required(column::CTE_FIRST_AID + k)),
                    associated_services: row.required(column::CTE_ASSOCIATED_SERVICES),
                },
                english_learners: EnglishLearners {
                    adm: std::array::from_fn(|k| row.required(column::EL_FIRST_ADM + k)),
                    aid: std::array::from_fn(|k| row.required(column::EL_FIRST_AID + k)),
                },
                gifted: Gifted {
                    adm_k6: row.required(column::GIFTED_ADM_K6),
                    fte_k8: row.required(column::GIFTED_FTE_K8),
                    fte_9_12: row.required(column::GIFTED_FTE_9_12),
                    identification: row.required(column::GIFTED_IDENTIFICATION),
                    referral: row.required(column::GIFTED_REFERRAL),
                    professional_development: row.required(column::GIFTED_PROFESSIONAL_DEVELOPMENT),
                    coordinator_units: row.required(column::GIFTED_COORDINATOR_UNITS),
                    coordinator_aid: row.required(column::GIFTED_COORDINATOR_AID),
                    specialist_k8_units: row.required(column::GIFTED_SPECIALIST_K8_UNITS),
                    specialist_k8_aid: row.required(column::GIFTED_SPECIALIST_K8_AID),
                    specialist_9_12_units: row.required(column::GIFTED_SPECIALIST_9_12_UNITS),
                    specialist_9_12_aid: row.required(column::GIFTED_SPECIALIST_9_12_AID),
                },
                categorical_enrolled_adm: row.required(column::CATEGORICAL_ENROLLED_ADM),
                county: row.str(column::COUNTY).to_string(),
                performance: PerformanceSupplement {
                    stars: row.num(column::PERFORMANCE_STARS),
                    progress: row.num(column::PERFORMANCE_PROGRESS),
                    progress_prior: row.num(column::PERFORMANCE_PROGRESS_PRIOR),
                    eligible: row.required(column::PERFORMANCE_ELIGIBLE) > 0.5,
                    amount: row.required(column::PERFORMANCE_SUPPLEMENT),
                },
                transportation: {
                    let input = |k: usize| row.required(column::TRANS_FIRST_INPUT + k);
                    let pay = |k: usize| row.required(column::TRANS_FIRST_PAYMENT + k);
                    Transportation {
                        public_riders: input(0),
                        nonpublic_riders: input(1),
                        community_riders: input(2),
                        weighted_riders: input(3),
                        mass_transit_riders: input(4),
                        other_riders: input(5),
                        bus_miles: input(6),
                        assigned_buses: input(7),
                        rider_capacity_target: input(8),
                        efficiency_index: input(9),
                        district_density: input(10),
                        square_miles: input(11),
                        reported_sped_cost: input(12),
                        school_bus: pay(0),
                        mass_transit: pay(1),
                        other: pay(2),
                        efficiency: pay(3),
                        density: pay(4),
                        fy21_base: pay(5),
                        guarantee: pay(6),
                        total: pay(7),
                        special_education: pay(8),
                    }
                },
                transition: Transition {
                    funding_base: row.required(column::FUNDING_BASE),
                    funding_base_econ_dis: row.required(column::FUNDING_BASE_ECON_DIS),
                    open_enrollment_prior: row.required(column::OPEN_ENROLLMENT_PRIOR),
                    open_enrollment_current: row.required(column::OPEN_ENROLLMENT_CURRENT),
                    open_enrollment_threshold: row.required(column::OPEN_ENROLLMENT_THRESHOLD),
                    open_enrollment_adjustment: row.required(column::OPEN_ENROLLMENT_ADJUSTMENT),
                    fy21_funding_base: row.required(column::FY21_FUNDING_BASE),
                    transition_supplement: row.required(column::FORMULA_TRANSITION_SUPPLEMENT),
                },
                preschool_special_education: PreschoolSpecialEducation {
                    adm: std::array::from_fn(|k| row.required(column::PREK_FIRST_ADM + k)),
                    aid: std::array::from_fn(|k| row.required(column::PREK_FIRST_AID + k)),
                    total: row.required(column::PREK_TOTAL),
                },
                supplements: Supplements {
                    base_funding: row.required(column::BASE_FUNDING_SUPPLEMENT),
                    adm_fy23: row.required(column::ADM_FY23),
                    enrollment_change: row.required(column::ENROLLMENT_CHANGE_THREE_YEAR),
                    growth_eligible: row.required(column::GROWTH_SUPPLEMENT_ELIGIBLE) > 0.5,
                    growth: row.required(column::ENROLLMENT_GROWTH_SUPPLEMENT),
                },
                net_state_funding: row.required(column::NET_STATE_FUNDING),
                guarantee: row.required(column::GUARANTEE),
                adm_history: [
                    row.required(column::ADM_FY24),
                    row.required(column::ADM_FY25),
                    row.required(column::ADM_FY26),
                ],
                current_year_adm: row.required(column::ADM_FY26),
                valuation_per_pupil: row.num(column::VALUATION_PER_PUPIL),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    ///
    /// The header assertion in [`panel`] catches a *renamed* column and the row-length check in
    /// `connect` catches a *missing* one. Neither catches the two disagreeing about **order**,
    /// which is what happened when the transition block was emitted before the preschool block and
    /// listed after it: every value in both blocks was in the wrong field, the row length was
    /// right, the header was right, and the fixture parsed.
    ///
    /// A magnitude check per block is the cheap guard. Each of these is a statewide total that
    /// could not plausibly be produced by a neighbouring column, so a shifted block fails here
    /// rather than in whichever test happens to touch it first.
    #[test]
    fn each_block_of_the_fixture_lands_in_its_own_columns() {
        let panel = panel();
        let total = |pick: fn(&DistrictRecord) -> f64| panel.iter().map(pick).sum::<f64>();

        for (label, got, low, high) in [
            (
                "base cost state share",
                total(|r| r.base_cost_state_share),
                3.0e9,
                4.5e9,
            ),
            (
                "targeted assistance",
                total(|r| r.categoricals.targeted_assistance),
                1.2e9,
                1.5e9,
            ),
            (
                "special education",
                total(|r| r.special_education.total()),
                0.6e9,
                0.8e9,
            ),
            ("DPIA", total(|r| r.categoricals.dpia), 0.4e9, 0.6e9),
            ("gifted", total(|r| r.gifted.total()), 40e6, 70e6),
            (
                "career-technical",
                total(|r| r.career_technical.total()),
                40e6,
                70e6,
            ),
            (
                "English learners",
                total(|r| r.english_learners.total()),
                25e6,
                50e6,
            ),
            (
                "transportation",
                total(|r| r.transportation.total),
                0.6e9,
                0.8e9,
            ),
            (
                "special education transport",
                total(|r| r.transportation.special_education),
                150e6,
                220e6,
            ),
            (
                "preschool special education",
                total(|r| r.preschool_special_education.total),
                130e6,
                170e6,
            ),
            (
                "performance supplement",
                total(|r| r.performance.amount),
                45e6,
                70e6,
            ),
            (
                "base supplement",
                total(|r| r.supplements.base_funding),
                45e6,
                70e6,
            ),
            (
                "growth supplement",
                total(|r| r.supplements.growth),
                25e6,
                55e6,
            ),
            (
                "transition supplement",
                total(|r| r.transition.transition_supplement),
                50e6,
                80e6,
            ),
            ("guarantee", total(|r| r.guarantee), 0.7e9, 1.1e9),
        ] {
            assert!(
                got > low && got < high,
                "{label} totals {got:.0}, outside the band this block should produce — most \
                 likely the fixture's columns and its header disagree about order"
            );
        }
    }

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
}
