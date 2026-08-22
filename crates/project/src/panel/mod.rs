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
//!
//! # Four concerns, four modules
//!
//! The module is laid out along the line the department's own model draws, stated at
//! [`DistrictRecord::total_state_support`]: core foundation funding is what the guarantee holds a
//! district at, and transportation, preschool special education, and the performance supplement
//! are real money paid outside it.
//!
//! | Module | What it holds |
//! |---|---|
//! | [`categoricals`] | the six weights and formulas *inside* foundation funding |
//! | [`supplements`] | the five components *outside* it |
//! | [`record`] | [`DistrictRecord`], and what one district's figures imply |
//! | [`fixture`] | the CSV, its header, and the only code that knows a column number |
//!
//! Every item is re-exported here, so `project::panel::SPECIAL_EDUCATION_WEIGHTS` and its
//! fifty-odd siblings mean what they meant when this was one file.

pub mod categoricals;
pub mod fixture;
pub mod record;
pub mod supplements;

pub use categoricals::{
    CareerTechnical, Categoricals, Dpia, EnglishLearners, Gifted, SpecialEducation,
    TargetedAssistance, AVERAGE_BASE_COST_PER_PUPIL, CTE_ASSOCIATED_WEIGHT,
    CTE_BASE_COST_PER_PUPIL, CTE_WEIGHTS, DPIA_BLEND, DPIA_PER_PUPIL, DPIA_STATEWIDE_PERCENTAGE,
    ENGLISH_LEARNER_WEIGHTS, GIFTED_COORDINATOR_DIVISOR, GIFTED_COORDINATOR_UNIT_BOUNDS,
    GIFTED_COORDINATOR_UNIT_PRICE, GIFTED_IDENTIFICATION_PER_PUPIL, GIFTED_REFERRAL_PER_PUPIL,
    GIFTED_SPECIALIST_DIVISOR, GIFTED_SPECIALIST_UNIT_FLOOR, GIFTED_SPECIALIST_UNIT_PRICES,
    SPECIAL_EDUCATION_WEIGHTS, TA_CAPACITY_FULL_AT, TA_CAPACITY_MINIMUM_ADM,
    TA_CAPACITY_RAMP_START, TA_CAPACITY_RATE, TA_CAPACITY_SMALL_SHARE, TA_MEDIAN_WEALTH_PER_PUPIL,
    TA_MEDIAN_WEIGHTED_WEALTH, TA_WEALTH_BLEND, TA_WEALTH_INDEX_FLOOR, TA_WEALTH_OFFSET_RATE,
    TA_WEALTH_RATE,
};
pub use fixture::panel;
pub use record::DistrictRecord;
pub use supplements::{
    PerformanceSupplement, PreschoolSpecialEducation, Supplements, Transition, Transportation,
    BASE_FUNDING_SUPPLEMENT_PER_PUPIL, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL,
    ENROLLMENT_GROWTH_THRESHOLD, OPEN_ENROLLMENT_CLAWBACK_PER_FTE, OPEN_ENROLLMENT_THRESHOLD_FLOOR,
    OPEN_ENROLLMENT_THRESHOLD_FRACTION, PERFORMANCE_PROGRESS_THRESHOLD, PERFORMANCE_STAR_THRESHOLD,
    PERFORMANCE_SUPPLEMENT_PER_POINT, PREK_SPED_APPROPRIATION, PREK_SPED_FLAT_PER_PUPIL,
    PREK_SPED_PRORATION, PREK_SPED_WEIGHT_FRACTION, TRANSPORT_COMMUNITY_WEIGHT,
    TRANSPORT_DENSITY_PIVOT, TRANSPORT_DENSITY_RATE, TRANSPORT_EFFICIENCY_BAND,
    TRANSPORT_EFFICIENCY_CEILING, TRANSPORT_MASS_TRANSIT_RATE, TRANSPORT_MINIMUM_STATE_SHARE,
    TRANSPORT_NONPUBLIC_WEIGHT, TRANSPORT_OTHER_RATE, TRANSPORT_PER_MILE, TRANSPORT_PER_RIDER,
    TRANSPORT_PRORATION, TRANSPORT_SCHOOL_DAYS, TRANSPORT_SPED_PRORATION,
};

use edfund_core::FiscalYear;

/// The fiscal year the model computes.
pub const MODEL_YEAR: FiscalYear = FiscalYear(2027);

/// The three fiscal years of enrolled ADM the model averages.
pub const HISTORY_YEARS: [FiscalYear; 3] = [FiscalYear(2024), FiscalYear(2025), FiscalYear(2026)];

/// The minimum state share of base cost operative in the FY2027 model.
///
/// The department's `Notes` sheet states `0.1` for FY2026 and FY2027. It is **not** the 5% the
/// Fair School Funding Plan was enacted with; each biennial budget sets it, and it doubled.
/// 138 of 609 districts sit exactly on it.
pub const MINIMUM_STATE_SHARE: f64 = edfund_core::MINIMUM_STATE_SHARE_FY2027;
