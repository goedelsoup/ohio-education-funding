//! What a district's money was spent *on*, entered into the models that only knew how much.
//!
//! # The limit three places in this repository named
//!
//! [`crate::least_squares`]'s own documentation said it: "Ohio districts differ in what they
//! spend money *on*, and no column in this workspace measures that."
//! `metric/progress-value-added` said it as the largest of its unmodelled omissions — "**what the
//! money was actually spent on**, which no column in this workspace measures. A district that
//! spends more on instruction and one that spends more on transportation are the same observation
//! here." Both were true when written and neither was true when read.
//!
//! [`crate::functions`] carries FY2025 operating spending split into the eleven functions the
//! department publishes it under, per pupil, for 607 districts. It was wired to compare two named
//! districts and never entered into a model. This module joins it to the report card and the
//! profile report so the spending variable in `report_card_2425.rs`'s specification can be taken
//! apart.
//!
//! # They are not the same observation
//!
//! The growth relationship is the classroom half of the money and nothing else. Entering the
//! department's own two-way split in place of the total, against the three-year Progress effect
//! size and the same five controls:
//!
//! | | standardised | t |
//! |---|--:|--:|
//! | classroom instruction per pupil | +0.243 | +5.11 |
//! | non-classroom per pupil | −0.032 | −0.61 |
//!
//! The total-spending coefficient of +0.209 that `metric/progress-value-added` reports is those
//! two averaged under a constraint that they be equal. Released from it, one of them is the whole
//! result and the other is indistinguishable from zero.
//!
//! # And composition carries information beyond level
//!
//! The sharper form asks whether *how* a district spends predicts anything once *how much* is
//! held fixed. Entering total operating spending and the classroom share together:
//!
//! | outcome | total | classroom share |
//! |---|--:|--:|
//! | three-year growth | +0.231 (t +5.04) | +0.122 (t +2.89) |
//! | attainment level | −0.056 (t −2.18) | +0.095 (t +4.01) |
//!
//! On level the two run in opposite directions, which is the finding: **spending more associates
//! with lower attainment and spending a larger share of it in the classroom associates with
//! higher attainment**, in the same model, on the same districts. A point of classroom share is
//! $163 per pupil at the median district — the two coefficients are not measured on comparable
//! scales, and the standardisation is what makes them readable together.
//!
//! # Two things this is not
//!
//! **Not a lever.** The classroom share is not a dial a board turns. It correlates +0.38 with log
//! enrolment and −0.40 with transportation spending per pupil: a small rural district buses
//! children a long way and cannot spend that money twice. Log enrolment is among the controls and
//! [`on_growth_with_transport`] adds transportation spending as its own regressor, which leaves
//! the share coefficient at +0.11 (t +2.48) — the right check, and still a description.
//!
//! **Not equally firm everywhere.** On the 303 districts whose English-learner share the
//! department did not suppress, the share coefficient falls to +0.087 with t = +1.56. The
//! estimate moves little and the sample halves, so this reads as power rather than
//! contradiction — but it is below the conventional bar and is stated rather than omitted. The
//! total-spending coefficient in the same subsample rises to +0.352.
//!
//! # Where the money is not classroom money
//!
//! Entered one function at a time against growth, plant operations and maintenance is the only
//! reliably negative one — −0.126, t = −2.97 — and pupil transportation, the function the corpus
//! named as its example, is −0.035 and indistinguishable from zero. The example was well chosen
//! and the answer is not the one it suggests: transportation spending is not what separates these
//! districts, and building maintenance is.

use std::collections::BTreeMap;

use crate::{functions, profile, report_card, Regression};

/// One district on every variable the models here use.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's published name.
    pub name: String,
    /// Total operating expenditure per pupil, FY2025, on the headcount denominator.
    pub operating: f64,
    /// The department's classroom roll-up: instruction, pupil support, instructional staff
    /// support.
    pub classroom: f64,
    /// Everything else. Adds to [`District::classroom`] to give [`District::operating`].
    pub nonclassroom: f64,
    /// Instruction alone, the largest single function.
    pub instruction: f64,
    /// Pupil support — counselling, health, attendance.
    pub pupil_support: f64,
    /// Instructional staff support.
    pub instructional_staff_support: f64,
    /// District-level administration.
    pub general_admin: f64,
    /// Building-level administration.
    pub school_admin: f64,
    /// Operations and maintenance of plant.
    pub operations_maintenance: f64,
    /// Pupil transportation.
    pub pupil_transportation: f64,
    /// Other support services.
    pub other_support: f64,
    /// Food service.
    pub food_service: f64,
    /// Economically disadvantaged share, from the profile report, in points.
    pub disadvantaged: f64,
    /// English-learner share, imputed at zero where the department suppressed it.
    pub english_learner: f64,
    /// Whether that imputation was applied to this district.
    pub english_learner_imputed: bool,
    /// Students with disabilities, share.
    pub disability: f64,
    /// Natural log of headcount enrolment.
    pub log_enrolment: f64,
    /// Assessed valuation per pupil, in thousands.
    pub valuation: f64,
    /// Performance Index — the attainment *level* measure.
    pub level: f64,
    /// Progress effect size, the published three-year average — the *growth* measure.
    pub growth: f64,
    /// Progress effect size on one year alone, which is year-matched to the spending.
    pub growth_one_year: f64,
}

impl District {
    /// Classroom spending as a percentage of operating spending.
    ///
    /// In points rather than a fraction, so a coefficient reads per point — and a point is $163
    /// per pupil at the median district, which is the scale worth carrying in mind.
    #[must_use]
    pub fn classroom_share(&self) -> f64 {
        self.classroom / self.operating * 100.0
    }

    /// The same with food service out of the denominator.
    ///
    /// Food service is largely federally reimbursed and scales with poverty, so it enters the
    /// share as a mechanical function of need rather than of choice. Removing it is the check
    /// that the composition result is not that.
    #[must_use]
    pub fn classroom_share_excluding_food(&self) -> f64 {
        self.classroom / (self.operating - self.food_service) * 100.0
    }

    /// Administration, district and building levels together.
    #[must_use]
    pub fn administration(&self) -> f64 {
        self.general_admin + self.school_admin
    }
}

/// Every district present on the report card, the function file and the profile report.
///
/// 606 of the 607 the department rates — the same population `report_card_2425.rs` fits its
/// models over, which is what makes the coefficients here comparable to the ones the corpus
/// already publishes rather than merely similar.
#[must_use]
pub fn frame() -> Vec<District> {
    let by_function: BTreeMap<String, functions::Functions> = functions::districts()
        .into_iter()
        .map(|f| (f.irn.clone(), f))
        .collect();
    let by_profile: BTreeMap<String, profile::ProfileDistrict> = profile::districts()
        .into_iter()
        .map(|d| (d.irn.clone(), d))
        .collect();

    report_card::report_cards()
        .into_iter()
        .filter_map(|card| {
            let spending = by_function.get(&card.irn)?;
            let profile = by_profile.get(&card.irn)?;
            Some(District {
                irn: card.irn.clone(),
                name: card.name.clone(),
                operating: spending.operating?,
                classroom: spending.classroom_instruction?,
                nonclassroom: spending.nonclassroom?,
                instruction: spending.instruction?,
                pupil_support: spending.pupil_support?,
                instructional_staff_support: spending.instructional_staff_support?,
                general_admin: spending.general_admin?,
                school_admin: spending.school_admin?,
                operations_maintenance: spending.operations_maintenance?,
                pupil_transportation: spending.pupil_transportation?,
                other_support: spending.other_support?,
                food_service: spending.food_service?,
                disadvantaged: profile.economically_disadvantaged? * 100.0,
                english_learner: card.english_learner.unwrap_or(0.0),
                english_learner_imputed: card.english_learner.is_none(),
                disability: card.students_with_disabilities?,
                log_enrolment: card.unweighted_adm?.ln(),
                valuation: profile.valuation_per_pupil? / 1000.0,
                level: card.performance_index?,
                growth: card.progress_effect_size?,
                growth_one_year: card.progress_effect_size_one_year?,
            })
        })
        .collect()
}

/// A named way of reading one number off a district — the shape every regressor here takes.
///
/// The name is carried beside the function so a caller reading `standardized[2]` can say what it
/// is a coefficient on, which is the whole of why these are pairs rather than bare functions.
pub type Regressor = (&'static str, fn(&District) -> f64);

/// The five controls every model here carries, in order.
///
/// Economic disadvantage, English-learner share, disability share, log enrolment, and assessed
/// valuation per pupil — the specification OCG White Paper 013 named as its own priority next
/// step, and the one `report_card_2425.rs` fits. Held as functions so a caller adds regressors in
/// front of them and reads a coefficient off a known index.
pub const CONTROLS: [Regressor; 5] = [
    ("economically disadvantaged", |d| d.disadvantaged),
    ("english learner share", |d| d.english_learner),
    ("students with disabilities", |d| d.disability),
    ("log enrolment", |d| d.log_enrolment),
    ("valuation per pupil, thousands", |d| d.valuation),
];

/// Fit `outcome` on `regressors` plus [`CONTROLS`], over `districts`.
///
/// The regressors come first, so `standardized[i]` and `t_statistics[i + 1]` are the *i*th one.
///
/// # Panics
///
/// If the frame is empty, degenerate or collinear — all of which mean the fixtures have changed
/// under this module rather than that a model failed to converge.
#[must_use]
pub fn fit(
    districts: &[District],
    regressors: &[Regressor],
    outcome: fn(&District) -> f64,
) -> Regression {
    let mut columns: Vec<Vec<f64>> = regressors
        .iter()
        .chain(CONTROLS.iter())
        .map(|(_, pick)| districts.iter().map(pick).collect())
        .collect();
    let response: Vec<f64> = districts.iter().map(outcome).collect();
    // Guards the caller against silently fitting an empty design, which `least_squares` reports
    // as an error the callers below would have to unwrap anyway.
    assert!(!columns.is_empty() && !response.is_empty(), "empty frame");
    columns.shrink_to_fit();
    crate::least_squares(&columns, &response).expect("the committed frame fits")
}

/// The department's own two-way split of the money, against growth.
///
/// The model that answers the question. `standardized[0]` is classroom spending and
/// `standardized[1]` is everything else.
#[must_use]
pub fn split_on_growth(districts: &[District]) -> Regression {
    fit(
        districts,
        &[
            ("classroom instruction", |d| d.classroom),
            ("non-classroom", |d| d.nonclassroom),
        ],
        |d| d.growth,
    )
}

/// How much and how, together — total spending and the classroom share.
///
/// `standardized[0]` is the level of spending and `standardized[1]` its composition.
#[must_use]
pub fn composition_on(districts: &[District], outcome: fn(&District) -> f64) -> Regression {
    fit(
        districts,
        &[
            ("operating per pupil", |d| d.operating),
            ("classroom share", District::classroom_share),
        ],
        outcome,
    )
}

/// The same with transportation spending entered as its own regressor.
///
/// The check that the composition result is not rurality wearing a different name: a district
/// that buses children a long way spends that money once, and its classroom share falls whether
/// or not anyone chose it.
#[must_use]
pub fn on_growth_with_transport(districts: &[District]) -> Regression {
    fit(
        districts,
        &[
            ("operating per pupil", |d| d.operating),
            ("classroom share", District::classroom_share),
            ("pupil transportation", |d| d.pupil_transportation),
        ],
        |d| d.growth,
    )
}

/// Every published function entered at once, against growth.
///
/// Nine regressors and then the controls. The roll-ups are left out because they are sums of
/// these and would be collinear with them.
#[must_use]
pub fn every_function_on_growth(districts: &[District]) -> Regression {
    fit(districts, &FUNCTIONS, |d| d.growth)
}

/// The nine functions, in the order [`every_function_on_growth`] enters them.
pub const FUNCTIONS: [Regressor; 9] = [
    ("instruction", |d| d.instruction),
    ("pupil support", |d| d.pupil_support),
    ("instructional staff support", |d| {
        d.instructional_staff_support
    }),
    ("general administration", |d| d.general_admin),
    ("school administration", |d| d.school_admin),
    ("operations and maintenance", |d| d.operations_maintenance),
    ("pupil transportation", |d| d.pupil_transportation),
    ("other support", |d| d.other_support),
    ("food service", |d| d.food_service),
];
