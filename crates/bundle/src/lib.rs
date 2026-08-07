//! Export a versioned JSON feed of the corpus's district-level findings.
//!
//! [`web/`](../../../web/) consumes a bundled export rather than reading
//! [`.yidam/corpus/`](../../../.yidam/corpus/) directly. The corpus is markdown and YAML written
//! for traversal by people and agents; the platform needs numbers for 609 districts. This
//! crate is the seam between them.
//!
//! # Contract version
//!
//! The bundle carries [`CONTRACT_VERSION`]. A consumer that does not recognise it should
//! refuse to render rather than guess, because a field silently changing meaning is worse than
//! a page that does not load. Bump it on any change to field names or units.
//!
//! # Checkpoints, and why a duplicated implementation is acceptable here
//!
//! The scenario builder in the web layer re-derives what `project::policy::apply` computes, in
//! TypeScript, so that moving a slider does not require a round trip. Two implementations of
//! the same formula is normally a bad trade: they drift, and the one nobody runs is the one
//! that is wrong.
//!
//! [`Checkpoint`] is the answer. The bundle carries Rust-computed results for a set of named
//! policies, and the page verifies its own arithmetic against them **before** it will render a
//! scenario. If the two disagree the page says so and disables the tab. The duplication is then
//! load-bearing in only one direction: the Rust is authoritative and the TypeScript has to prove
//! it agrees, on every page load, against the real 609-district panel.
//!
//! # Why hand-rolled JSON
//!
//! The workspace has no external dependencies, deliberately — a committed
//! [`scenario`](../../../.yidam/corpus/scenario/) result should be reproducible years from now
//! without a dependency resolution succeeding first. Serializing a fixed, known schema is a
//! few dozen lines, so that constraint costs nothing here.

#![forbid(unsafe_code)]

use edfund_core::Dollars;

/// The bundle schema version. Bump on any change to field names, units, or semantics.
///
/// `4.0.0` added the projection axis: `adm_history` on every district, so the page can carry
/// enrollment forward itself, and a `projection` block holding the forecast's method, its prior,
/// and [`ForecastCheckpoint`]s the page must reproduce before it may draw a band. Breaking rather
/// than additive because `adm_history` is required, not nullable — a district without it cannot
/// be projected, and a feed that omitted it would produce a page silently missing half its
/// panel.
///
/// `3.0.0` added the outcome axis: a nullable `outcome` object per district carrying the
/// Performance Index, the Progress effect size, need shares, and spending on both denominators,
/// plus the statewide correlations that say how to read them. Nullable because three districts
/// have no report card — see [`project::crosswalk`].
///
/// `2.0.0` added the scenario inputs and checkpoints, and renamed the enrollment-change years
/// from FY2022-FY2024 to FY2024-FY2026 — the years the department's `ADM Data` sheet declares.
/// The values did not change; what they are called did, which is exactly the kind of silent
/// meaning change the version guard exists for.
pub const CONTRACT_VERSION: &str = "4.0.0";

/// The outcome side of a district, where the report card covers it.
///
/// # Two spending figures and two poverty figures, both on purpose
///
/// `per_equivalent_pupil` divides by a need-weighted count and is the department's published
/// figure; `per_enrolled_pupil` divides by the headcount. Against a composition-driven outcome
/// the first is substantially a composition proxy, and the corpus's central denominator finding
/// is the gap between them. Shipping only one would make that finding unstateable in the
/// interface that is supposed to explain it.
///
/// `economically_disadvantaged` is the report card's, which is top-coded by community
/// eligibility. The profile report's untop-coded share stays on [`District`] itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistrictOutcome {
    /// Performance Index, 2024-25. Ohio's attainment-level measure.
    pub performance_index: Option<f64>,
    /// Performance Index, 2023-24.
    pub performance_index_prior: Option<f64>,
    /// Performance Index, 2022-23.
    pub performance_index_earliest: Option<f64>,
    /// Value-added effect size — Ohio's growth measure, already a three-year average.
    pub progress_effect_size: Option<f64>,
    /// Operating expenditure per enrolled pupil, FY2025.
    pub per_enrolled_pupil: Option<Dollars>,
    /// Operating expenditure per need-weighted pupil, FY2025. The published figure.
    pub per_equivalent_pupil: Option<Dollars>,
    /// Economically disadvantaged share, 2024-25, top-coded.
    pub economically_disadvantaged: Option<f64>,
    /// English learner share, 2024-25.
    pub english_learner: Option<f64>,
    /// Students with disabilities share, 2024-25.
    pub students_with_disabilities: Option<f64>,
}

/// Statewide relationships between the funding side and the outcome side.
///
/// Every one is a correlation over the joined panel and none identifies an effect. They are in
/// the feed rather than left to the page to compute, because the page would then have to choose
/// which poverty measure to control for, and that choice moves the answer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutcomeStatewide {
    /// Districts with both a funding record and a report card.
    pub districts: usize,
    /// Poverty against the Performance Index. The dominant relationship in the data.
    pub poverty_vs_performance: f64,
    /// Guarantee status against the Performance Index, raw.
    pub guarantee_vs_performance: f64,
    /// The same, holding poverty constant.
    pub guarantee_vs_performance_controlled: f64,
    /// Spending per enrolled pupil against growth, holding poverty constant.
    pub spending_vs_growth_controlled: f64,
    /// Spending per *weighted* pupil against the Performance Index, raw — the published
    /// near-zero figure whose denominator the corpus disputes.
    pub weighted_spending_vs_performance: f64,
    /// Spending per *enrolled* pupil against the Performance Index, raw.
    pub enrolled_spending_vs_performance: f64,
    /// Median Performance Index among districts on the guarantee.
    pub median_performance_on_guarantee: f64,
    /// Median Performance Index among districts on the formula.
    pub median_performance_on_formula: f64,
}

/// One district, as the web layer needs it.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Information Retrieval Number, the stable statewide identifier.
    pub irn: String,
    /// District name as published.
    pub name: String,
    /// Base cost enrolled ADM — the greater of the three-year average and the current year.
    pub adm: f64,
    /// Current-year enrolled ADM, FY2026. The denominator the state share is paid on.
    pub current_year_adm: f64,
    /// District base cost per pupil, FY2027.
    pub base_cost_per_pupil: Dollars,
    /// Aggregate base cost, all five sub-components.
    pub aggregate_base_cost: Dollars,
    /// The state's share of base cost alone, before every categorical.
    pub base_cost_state_share: Dollars,
    /// Targeted assistance, special education, DPIA, English learner, gifted, career-technical.
    pub categorical_funding: Dollars,
    /// State aid per pupil as the formula computes it, before the guarantee.
    pub formula_aid_per_pupil: Dollars,
    /// State aid per pupil as the district receives it.
    pub realized_aid_per_pupil: Dollars,
    /// Temporary transitional aid guarantee, total dollars.
    pub guarantee: Dollars,
    /// Whether the minimum state share is what sets this district's base cost aid.
    pub at_minimum_state_share: bool,
    /// Assessed valuation per pupil, FY2023.
    pub valuation_per_pupil: Option<Dollars>,
    /// Effective Class 1 operating millage, TY2023.
    pub effective_class1_millage: Option<f64>,
    /// Total operating expenditure per pupil, FY2024.
    pub operating_expenditure_per_pupil: Option<Dollars>,
    /// Share of students economically disadvantaged, FY2024, as a fraction.
    pub economically_disadvantaged: Option<f64>,
    /// Enrollment change FY2024 to FY2026, as a fraction. FY2026 is partly departmental
    /// estimate, since the calculator is published before that year closes.
    pub enrollment_change: Option<f64>,
    /// Enrolled ADM for FY2024, FY2025, FY2026 — the three years the department's `ADM Data`
    /// sheet carries.
    ///
    /// Shipped as the series rather than only as [`District::enrollment_change`] because the
    /// page projects from it. Three points is not enough to estimate this district's own
    /// variability, which is exactly why the interval comes from the cross-sectional spread
    /// instead; see [`Projection::sigma`].
    pub adm_history: [f64; 3],
    /// Achievement, growth, and need. `None` for the three districts with no report card.
    pub outcome: Option<DistrictOutcome>,
}

impl District {
    /// Whether the district is funded by the guarantee rather than the formula.
    #[must_use]
    pub fn on_guarantee(&self) -> bool {
        self.guarantee > 0.0
    }

    /// Whether the district sits at the 20-mill floor, where valuation growth reaches revenue.
    #[must_use]
    pub fn at_millage_floor(&self) -> bool {
        self.effective_class1_millage
            .is_some_and(|m| (m - 20.0).abs() < 0.005)
    }

    /// The FY2020 baseline the guarantee holds this district at, recoverable only when it is
    /// on the guarantee.
    #[must_use]
    pub fn implied_fy2020_baseline_per_pupil(&self) -> Option<Dollars> {
        self.on_guarantee().then_some(self.realized_aid_per_pupil)
    }
}

/// Statewide context, so a consumer can position any district without recomputing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Statewide {
    /// Number of districts in the bundle.
    pub districts: usize,
    /// Districts funded by the guarantee.
    pub on_guarantee: usize,
    /// Districts at the 20-mill floor.
    pub at_millage_floor: usize,
    /// Districts whose base cost aid is set by the minimum state share.
    pub at_minimum_state_share: usize,
    /// Median assessed valuation per pupil.
    pub median_valuation_per_pupil: Dollars,
    /// Median operating expenditure per pupil.
    pub median_operating_expenditure_per_pupil: Dollars,
    /// Correlation between valuation per pupil and formula aid per pupil.
    pub wealth_neutrality_formula: f64,
    /// Correlation between valuation per pupil and realized aid per pupil.
    pub wealth_neutrality_realized: f64,
    /// Total guarantee dollars.
    pub guarantee_total: Dollars,
    /// Total realized state aid.
    pub realized_aid_total: Dollars,
    /// The minimum state share this model operates under.
    pub minimum_state_share: f64,
    /// How the funding side relates to the outcome side. `None` if no district joined.
    pub outcomes: Option<OutcomeStatewide>,
}

/// A policy, in the shape the web layer sends it back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolicyShape {
    /// `as-enacted`, `removed`, `rebase`, or `phase-out`.
    pub guarantee: &'static str,
    /// The factor or remaining share, where the rule takes one.
    pub guarantee_argument: f64,
    /// Multiplier on aggregate base cost.
    pub base_cost_scale: f64,
    /// Minimum state share of base cost.
    pub minimum_state_share: f64,
    /// Appropriated fraction of base cost aid.
    pub phase_in_base_cost: f64,
    /// Appropriated fraction of categorical aid.
    pub phase_in_categorical: f64,
}

/// A Rust-computed result the web layer must reproduce before it is allowed to compute more.
///
/// See the crate note. This is what makes a second implementation of the formula acceptable.
#[derive(Debug, Clone, PartialEq)]
pub struct Checkpoint {
    /// Human label, shown if the check fails.
    pub label: String,
    /// The policy that produced it. Without this a consumer could verify a number while
    /// computing a different scenario from the one the number belongs to.
    pub policy: PolicyShape,
    /// Change in total state aid against current law.
    pub cost: Dollars,
    /// Total realized aid under the policy.
    pub realized_aid: Dollars,
    /// Districts whose aid rises.
    pub gainers: usize,
    /// Districts whose aid falls.
    pub losers: usize,
    /// Districts the policy does not reach.
    pub unmoved: usize,
    /// Districts on the guarantee under the policy.
    pub on_guarantee: usize,
}

/// A Rust-computed *forecast* the web layer must reproduce before it may draw a band.
///
/// The same discipline as [`Checkpoint`], applied to the harder half. Reproducing a simulation
/// checks one function; reproducing a forecast checks the projection, the prior, the compounding
/// of the interval with the horizon, and the decision to re-run the whole formula at each end of
/// the enrollment band rather than scale the central answer — which matters because the
/// guarantee is a `max` and the aid curve has a kink no scaling reproduces.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCheckpoint {
    /// Human label, shown if the check fails.
    pub label: String,
    /// The policy held fixed across the horizon.
    pub policy: PolicyShape,
    /// The fiscal year projected to.
    pub fiscal_year: u16,
    /// Total realized aid at the central enrollment estimate.
    pub realized_aid: Dollars,
    /// Total realized aid at the low end of the enrollment band.
    pub low: Dollars,
    /// Total realized aid at the high end.
    pub high: Dollars,
    /// Projected total ADM.
    pub adm: f64,
    /// Districts on the guarantee at projected enrollment.
    pub on_guarantee: usize,
}

/// How this feed's forecasts were made, and what their interval rests on.
///
/// The page carries its own copy of the projection so a slider does not need a round trip, as it
/// does for the formula. This block is what makes that acceptable: the method and its parameters
/// so the page runs the same one, and [`Projection::checkpoints`] so it has to prove it did.
#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    /// The last observed fiscal year. Everything past it is forecast.
    pub base_year: u16,
    /// The furthest year the checkpoints reach, and the furthest the page should offer.
    pub horizon: u16,
    /// `damped`, `cagr`, `linear`, or `flat`.
    pub method: String,
    /// Per-year decay applied to the fitted growth rate. 1.0 is undamped.
    pub damping: f64,
    /// Standard deviation of annual enrolled-ADM growth **across districts**.
    ///
    /// Not this district's variability — three observations cannot give that. It is how much
    /// districts differ from one another, used as a floor on the uncertainty.
    pub sigma: f64,
    /// Standard deviations spanned on each side of the point.
    pub z: f64,
    /// What produced [`Projection::sigma`]. Printed wherever the band is.
    pub prior_source: String,
    /// Forecasts the consumer must reproduce.
    pub checkpoints: Vec<ForecastCheckpoint>,
}

/// The exported feed.
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    /// Schema version; see [`CONTRACT_VERSION`].
    pub contract_version: String,
    /// What the figures describe and where they came from.
    pub provenance: String,
    /// The fiscal year the model computes.
    pub fiscal_year: u16,
    /// Statewide aggregates.
    pub statewide: Statewide,
    /// Reference results the consumer must reproduce.
    pub checkpoints: Vec<Checkpoint>,
    /// How to project, and the forecasts that check the projection. `None` disables the band.
    pub projection: Option<Projection>,
    /// Per-district records.
    pub districts: Vec<District>,
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn num(v: f64) -> String {
    if v.is_finite() {
        format!("{v:.4}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        "null".into()
    }
}

fn opt(v: Option<f64>) -> String {
    v.map_or_else(|| "null".into(), num)
}

impl Bundle {
    /// Serialize to JSON.
    ///
    /// Deterministic: the same bundle always produces byte-identical output, so a committed
    /// feed diffs cleanly and a regenerated one shows only real changes.
    #[must_use]
    pub fn to_json(&self) -> String {
        let mut s = String::with_capacity(self.districts.len() * 320 + 4096);
        s.push_str("{\n");
        s.push_str(&format!(
            "  \"contract_version\": \"{}\",\n",
            escape(&self.contract_version)
        ));
        s.push_str(&format!(
            "  \"provenance\": \"{}\",\n",
            escape(&self.provenance)
        ));
        s.push_str(&format!("  \"fiscal_year\": {},\n", self.fiscal_year));

        let w = &self.statewide;
        s.push_str("  \"statewide\": {\n");
        s.push_str(&format!("    \"districts\": {},\n", w.districts));
        s.push_str(&format!("    \"on_guarantee\": {},\n", w.on_guarantee));
        s.push_str(&format!(
            "    \"at_millage_floor\": {},\n",
            w.at_millage_floor
        ));
        s.push_str(&format!(
            "    \"at_minimum_state_share\": {},\n",
            w.at_minimum_state_share
        ));
        s.push_str(&format!(
            "    \"median_valuation_per_pupil\": {},\n",
            num(w.median_valuation_per_pupil)
        ));
        s.push_str(&format!(
            "    \"median_operating_expenditure_per_pupil\": {},\n",
            num(w.median_operating_expenditure_per_pupil)
        ));
        s.push_str(&format!(
            "    \"wealth_neutrality_formula\": {},\n",
            num(w.wealth_neutrality_formula)
        ));
        s.push_str(&format!(
            "    \"wealth_neutrality_realized\": {},\n",
            num(w.wealth_neutrality_realized)
        ));
        s.push_str(&format!(
            "    \"guarantee_total\": {},\n",
            num(w.guarantee_total)
        ));
        s.push_str(&format!(
            "    \"realized_aid_total\": {},\n",
            num(w.realized_aid_total)
        ));
        s.push_str(&format!(
            "    \"minimum_state_share\": {},\n",
            num(w.minimum_state_share)
        ));
        match &w.outcomes {
            None => s.push_str("    \"outcomes\": null\n"),
            Some(o) => s.push_str(&format!(
                "    \"outcomes\": {{\"districts\": {}, \"poverty_vs_performance\": {}, \
                 \"guarantee_vs_performance\": {}, \
                 \"guarantee_vs_performance_controlled\": {}, \
                 \"spending_vs_growth_controlled\": {}, \
                 \"weighted_spending_vs_performance\": {}, \
                 \"enrolled_spending_vs_performance\": {}, \
                 \"median_performance_on_guarantee\": {}, \
                 \"median_performance_on_formula\": {}}}\n",
                o.districts,
                num(o.poverty_vs_performance),
                num(o.guarantee_vs_performance),
                num(o.guarantee_vs_performance_controlled),
                num(o.spending_vs_growth_controlled),
                num(o.weighted_spending_vs_performance),
                num(o.enrolled_spending_vs_performance),
                num(o.median_performance_on_guarantee),
                num(o.median_performance_on_formula),
            )),
        }
        s.push_str("  },\n");

        s.push_str("  \"checkpoints\": [\n");
        for (i, c) in self.checkpoints.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"label\": \"{}\", \"policy\": {{\"guarantee\": \"{}\", \
                 \"guarantee_argument\": {}, \"base_cost_scale\": {}, \
                 \"minimum_state_share\": {}, \"phase_in_base_cost\": {}, \
                 \"phase_in_categorical\": {}}}, \"cost\": {}, \"realized_aid\": {}, \
                 \"gainers\": {}, \"losers\": {}, \"unmoved\": {}, \"on_guarantee\": {}}}",
                escape(&c.label),
                escape(c.policy.guarantee),
                num(c.policy.guarantee_argument),
                num(c.policy.base_cost_scale),
                num(c.policy.minimum_state_share),
                num(c.policy.phase_in_base_cost),
                num(c.policy.phase_in_categorical),
                num(c.cost),
                num(c.realized_aid),
                c.gainers,
                c.losers,
                c.unmoved,
                c.on_guarantee
            ));
            if i + 1 < self.checkpoints.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        match &self.projection {
            None => s.push_str("  \"projection\": null,\n"),
            Some(p) => {
                s.push_str("  \"projection\": {\n");
                s.push_str(&format!("    \"base_year\": {},\n", p.base_year));
                s.push_str(&format!("    \"horizon\": {},\n", p.horizon));
                s.push_str(&format!("    \"method\": \"{}\",\n", escape(&p.method)));
                s.push_str(&format!("    \"damping\": {},\n", num(p.damping)));
                // Six places, not the four `num` gives: sigma is a growth rate around 0.02, and
                // rounding it to 0.0234 would move a ten-year band by enough to fail its own
                // checkpoint.
                s.push_str(&format!("    \"sigma\": {:.6},\n", p.sigma));
                s.push_str(&format!("    \"z\": {},\n", num(p.z)));
                s.push_str(&format!(
                    "    \"prior_source\": \"{}\",\n",
                    escape(&p.prior_source)
                ));
                s.push_str("    \"checkpoints\": [\n");
                for (i, c) in p.checkpoints.iter().enumerate() {
                    s.push_str(&format!(
                        "      {{\"label\": \"{}\", \"policy\": {{\"guarantee\": \"{}\", \
                         \"guarantee_argument\": {}, \"base_cost_scale\": {}, \
                         \"minimum_state_share\": {}, \"phase_in_base_cost\": {}, \
                         \"phase_in_categorical\": {}}}, \"fiscal_year\": {}, \
                         \"realized_aid\": {}, \"low\": {}, \"high\": {}, \"adm\": {}, \
                         \"on_guarantee\": {}}}",
                        escape(&c.label),
                        escape(c.policy.guarantee),
                        num(c.policy.guarantee_argument),
                        num(c.policy.base_cost_scale),
                        num(c.policy.minimum_state_share),
                        num(c.policy.phase_in_base_cost),
                        num(c.policy.phase_in_categorical),
                        c.fiscal_year,
                        num(c.realized_aid),
                        num(c.low),
                        num(c.high),
                        num(c.adm),
                        c.on_guarantee
                    ));
                    if i + 1 < p.checkpoints.len() {
                        s.push(',');
                    }
                    s.push('\n');
                }
                s.push_str("    ]\n  },\n");
            }
        }

        s.push_str("  \"districts\": [\n");

        for (i, d) in self.districts.iter().enumerate() {
            s.push_str("    {");
            s.push_str(&format!("\"irn\": \"{}\", ", escape(&d.irn)));
            s.push_str(&format!("\"name\": \"{}\", ", escape(&d.name)));
            s.push_str(&format!("\"adm\": {}, ", num(d.adm)));
            s.push_str(&format!(
                "\"current_year_adm\": {}, ",
                num(d.current_year_adm)
            ));
            s.push_str(&format!(
                "\"base_cost_per_pupil\": {}, ",
                num(d.base_cost_per_pupil)
            ));
            s.push_str(&format!(
                "\"aggregate_base_cost\": {}, ",
                num(d.aggregate_base_cost)
            ));
            s.push_str(&format!(
                "\"base_cost_state_share\": {}, ",
                num(d.base_cost_state_share)
            ));
            s.push_str(&format!(
                "\"categorical_funding\": {}, ",
                num(d.categorical_funding)
            ));
            s.push_str(&format!(
                "\"formula_aid_per_pupil\": {}, ",
                num(d.formula_aid_per_pupil)
            ));
            s.push_str(&format!(
                "\"realized_aid_per_pupil\": {}, ",
                num(d.realized_aid_per_pupil)
            ));
            s.push_str(&format!("\"guarantee\": {}, ", num(d.guarantee)));
            s.push_str(&format!("\"on_guarantee\": {}, ", d.on_guarantee()));
            s.push_str(&format!("\"at_millage_floor\": {}, ", d.at_millage_floor()));
            s.push_str(&format!(
                "\"at_minimum_state_share\": {}, ",
                d.at_minimum_state_share
            ));
            s.push_str(&format!(
                "\"valuation_per_pupil\": {}, ",
                opt(d.valuation_per_pupil)
            ));
            s.push_str(&format!(
                "\"effective_class1_millage\": {}, ",
                opt(d.effective_class1_millage)
            ));
            s.push_str(&format!(
                "\"operating_expenditure_per_pupil\": {}, ",
                opt(d.operating_expenditure_per_pupil)
            ));
            s.push_str(&format!(
                "\"economically_disadvantaged\": {}, ",
                opt(d.economically_disadvantaged)
            ));
            s.push_str(&format!(
                "\"enrollment_change\": {}, ",
                opt(d.enrollment_change)
            ));
            s.push_str(&format!(
                "\"adm_history\": [{}, {}, {}], ",
                num(d.adm_history[0]),
                num(d.adm_history[1]),
                num(d.adm_history[2])
            ));
            match &d.outcome {
                None => s.push_str("\"outcome\": null"),
                Some(o) => s.push_str(&format!(
                    "\"outcome\": {{\"performance_index\": {}, \
                     \"performance_index_prior\": {}, \
                     \"performance_index_earliest\": {}, \
                     \"progress_effect_size\": {}, \"per_enrolled_pupil\": {}, \
                     \"per_equivalent_pupil\": {}, \"economically_disadvantaged\": {}, \
                     \"english_learner\": {}, \"students_with_disabilities\": {}}}",
                    opt(o.performance_index),
                    opt(o.performance_index_prior),
                    opt(o.performance_index_earliest),
                    opt(o.progress_effect_size),
                    opt(o.per_enrolled_pupil),
                    opt(o.per_equivalent_pupil),
                    opt(o.economically_disadvantaged),
                    opt(o.english_learner),
                    opt(o.students_with_disabilities),
                )),
            }
            s.push('}');
            if i + 1 < self.districts.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ]\n}\n");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> District {
        District {
            irn: "049056".into(),
            name: "Northern Local".into(),
            adm: 2_193.81,
            current_year_adm: 2_107.80,
            base_cost_per_pupil: 8_100.0,
            aggregate_base_cost: 17_769_861.0,
            base_cost_state_share: 6_000_000.0,
            categorical_funding: 8_038_562.0,
            formula_aid_per_pupil: 6_400.0,
            realized_aid_per_pupil: 6_400.0,
            guarantee: 0.0,
            at_minimum_state_share: false,
            valuation_per_pupil: Some(279_983.24),
            effective_class1_millage: Some(20.0),
            operating_expenditure_per_pupil: Some(11_986.62),
            economically_disadvantaged: Some(0.3881),
            enrollment_change: Some(-0.03),
            adm_history: [2_173.0, 2_140.0, 2_107.8],
            outcome: Some(DistrictOutcome {
                performance_index: Some(89.9),
                performance_index_prior: Some(89.1),
                performance_index_earliest: Some(88.4),
                progress_effect_size: Some(0.0),
                per_enrolled_pupil: Some(14_512.0),
                per_equivalent_pupil: Some(11_986.62),
                economically_disadvantaged: Some(38.8),
                english_learner: Some(0.4),
                students_with_disabilities: Some(15.2),
            }),
        }
    }

    fn zero_statewide() -> Statewide {
        Statewide {
            districts: 1,
            on_guarantee: 0,
            at_millage_floor: 1,
            at_minimum_state_share: 0,
            median_valuation_per_pupil: 0.0,
            median_operating_expenditure_per_pupil: 0.0,
            wealth_neutrality_formula: 0.0,
            wealth_neutrality_realized: 0.0,
            guarantee_total: 0.0,
            realized_aid_total: 0.0,
            minimum_state_share: 0.1,
            outcomes: Some(OutcomeStatewide {
                districts: 606,
                poverty_vs_performance: -0.846,
                guarantee_vs_performance: 0.187,
                guarantee_vs_performance_controlled: 0.035,
                spending_vs_growth_controlled: 0.146,
                weighted_spending_vs_performance: -0.015,
                enrolled_spending_vs_performance: -0.337,
                median_performance_on_guarantee: 89.9,
                median_performance_on_formula: 85.6,
            }),
        }
    }

    fn bundle(districts: Vec<District>, checkpoints: Vec<Checkpoint>) -> Bundle {
        Bundle {
            contract_version: CONTRACT_VERSION.into(),
            provenance: "test".into(),
            fiscal_year: 2027,
            statewide: zero_statewide(),
            checkpoints,
            projection: None,
            districts,
        }
    }

    fn projection() -> Projection {
        Projection {
            base_year: 2026,
            horizon: 2036,
            method: "damped".into(),
            damping: 0.85,
            sigma: 0.023_456_7,
            z: 1.0,
            prior_source: "cross-sectional spread of district annual enrolled-ADM growth".into(),
            checkpoints: vec![ForecastCheckpoint {
                label: "current law, FY2032".into(),
                policy: checkpoint().policy,
                fiscal_year: 2032,
                realized_aid: 7_100_000_000.0,
                low: 6_860_000_000.0,
                high: 7_350_000_000.0,
                adm: 1_500_000.0,
                on_guarantee: 320,
            }],
        }
    }

    fn checkpoint() -> Checkpoint {
        Checkpoint {
            label: "guarantee removed".into(),
            policy: PolicyShape {
                guarantee: "removed",
                guarantee_argument: 0.0,
                base_cost_scale: 1.0,
                minimum_state_share: 0.1,
                phase_in_base_cost: 1.0,
                phase_in_categorical: 1.0,
            },
            cost: -879_000_000.0,
            realized_aid: 6_402_000_000.0,
            gainers: 0,
            losers: 294,
            unmoved: 315,
            on_guarantee: 0,
        }
    }

    #[test]
    fn a_district_with_no_guarantee_is_on_formula() {
        assert!(!sample().on_guarantee());
    }

    #[test]
    fn exactly_twenty_mills_counts_as_the_floor() {
        assert!(sample().at_millage_floor());
        let above = District {
            effective_class1_millage: Some(37.09),
            ..sample()
        };
        assert!(!above.at_millage_floor());
        let none = District {
            effective_class1_millage: None,
            ..sample()
        };
        assert!(!none.at_millage_floor());
    }

    #[test]
    fn the_fy2020_baseline_is_only_recoverable_on_the_guarantee() {
        assert_eq!(sample().implied_fy2020_baseline_per_pupil(), None);
        let guaranteed = District {
            guarantee: 1_000_000.0,
            realized_aid_per_pupil: 7_100.0,
            ..sample()
        };
        assert_eq!(
            guaranteed.implied_fy2020_baseline_per_pupil(),
            Some(7_100.0)
        );
    }

    #[test]
    fn json_escapes_quotes_and_backslashes_in_district_names() {
        let odd = District {
            name: r#"St. "Mary" \ Local"#.into(),
            ..sample()
        };
        assert!(bundle(vec![odd], vec![])
            .to_json()
            .contains(r#"St. \"Mary\" \\ Local"#));
    }

    #[test]
    fn missing_values_serialize_as_null_not_zero() {
        let sparse = District {
            valuation_per_pupil: None,
            effective_class1_millage: None,
            operating_expenditure_per_pupil: None,
            economically_disadvantaged: None,
            enrollment_change: None,
            ..sample()
        };
        let json = bundle(vec![sparse], vec![]).to_json();
        assert!(json.contains("\"valuation_per_pupil\": null"));
        assert!(
            !json.contains("\"valuation_per_pupil\": 0"),
            "a missing value must not be indistinguishable from zero"
        );
    }

    #[test]
    fn serialization_is_deterministic() {
        let b = bundle(vec![sample(), sample()], vec![checkpoint()]);
        assert_eq!(b.to_json(), b.to_json());
    }

    #[test]
    fn the_bundle_declares_its_contract_version() {
        assert!(bundle(vec![], vec![])
            .to_json()
            .contains("\"contract_version\": \"4.0.0\""));
    }

    #[test]
    fn a_feed_without_a_projection_says_null_rather_than_omitting_the_key() {
        // A consumer must be able to tell "this feed cannot be projected" from "this feed is
        // from a build that predates projection". The first disables a band; the second is a
        // contract mismatch and should have been caught by the version guard.
        assert!(bundle(vec![], vec![])
            .to_json()
            .contains("\"projection\": null"));
    }

    #[test]
    fn the_projection_block_carries_its_method_and_the_prior_the_band_rests_on() {
        let b = Bundle {
            projection: Some(projection()),
            ..bundle(vec![sample()], vec![checkpoint()])
        };
        let json = b.to_json();
        assert!(json.contains("\"method\": \"damped\""));
        assert!(json.contains("\"damping\": 0.85"));
        assert!(json.contains("\"base_year\": 2026"));
        assert!(json.contains("cross-sectional spread"));
    }

    #[test]
    fn sigma_keeps_six_places_because_four_would_move_a_ten_year_band() {
        // `num` rounds to four, which turns 0.0234567 into 0.0235 — a 0.2% shift in the half
        // width at a ten-year horizon, which is enough to fail the checkpoint it exists to pass.
        let json = Bundle {
            projection: Some(projection()),
            ..bundle(vec![], vec![])
        }
        .to_json();
        assert!(json.contains("\"sigma\": 0.023457"), "{json}");
    }

    #[test]
    fn a_forecast_checkpoint_carries_both_ends_of_its_band() {
        // A point with no interval is the thing this whole axis exists to not ship.
        let json = Bundle {
            projection: Some(projection()),
            ..bundle(vec![], vec![])
        }
        .to_json();
        assert!(json.contains("\"realized_aid\": 7100000000"));
        assert!(json.contains("\"low\": 6860000000"));
        assert!(json.contains("\"high\": 7350000000"));
        assert!(json.contains("\"fiscal_year\": 2032"));
    }

    #[test]
    fn every_district_carries_the_three_years_the_projection_is_fitted_from() {
        // Not nullable: a district without a history cannot be projected, and a page that
        // silently dropped it would report a statewide total over a subset of the panel.
        let json = bundle(vec![sample()], vec![]).to_json();
        assert!(
            json.contains("\"adm_history\": [2173, 2140, 2107.8]"),
            "{json}"
        );
    }

    #[test]
    fn a_district_without_a_report_card_serializes_a_null_outcome() {
        // Three districts have none. `null` rather than an object of nulls, so a consumer can
        // tell "no report card" from "a report card with nothing in it".
        let none = District {
            outcome: None,
            ..sample()
        };
        let json = bundle(vec![none], vec![]).to_json();
        assert!(json.contains("\"outcome\": null"));
        assert!(!json.contains("\"performance_index\""));
    }

    #[test]
    fn the_outcome_block_carries_both_spending_denominators() {
        // The corpus's central denominator finding is the gap between them. Shipping one would
        // make it unstateable in the interface meant to explain it.
        let json = bundle(vec![sample()], vec![]).to_json();
        assert!(json.contains("\"per_enrolled_pupil\": 14512"));
        assert!(json.contains("\"per_equivalent_pupil\": 11986.62"));
    }

    #[test]
    fn the_statewide_outcomes_carry_the_raw_and_the_controlled_figure() {
        // A page showing +0.187 without +0.035 beside it would be stating the confound as a
        // finding, which is the specific thing this axis was built to prevent.
        let json = bundle(vec![], vec![]).to_json();
        assert!(json.contains("\"guarantee_vs_performance\": 0.187"));
        assert!(json.contains("\"guarantee_vs_performance_controlled\": 0.035"));
    }

    #[test]
    fn checkpoints_carry_the_policy_that_produced_them() {
        let json = bundle(vec![], vec![checkpoint()]).to_json();
        assert!(json.contains("\"guarantee\": \"removed\""));
        assert!(json.contains("\"cost\": -879000000"));
        assert!(json.contains("\"unmoved\": 315"));
    }

    #[test]
    fn an_empty_checkpoint_list_still_produces_valid_json() {
        assert!(bundle(vec![sample()], vec![])
            .to_json()
            .contains("\"checkpoints\": [\n  ],"));
    }

    #[test]
    fn the_scenario_inputs_are_present_for_every_district() {
        // The web layer cannot re-derive a policy without these four.
        let json = bundle(vec![sample()], vec![]).to_json();
        for field in [
            "aggregate_base_cost",
            "base_cost_state_share",
            "categorical_funding",
            "current_year_adm",
        ] {
            assert!(json.contains(field), "{field} missing from the feed");
        }
    }
}
