//! Export a versioned JSON feed of the corpus's district-level findings.
//!
//! [`web/`](../../web/) consumes a bundled export rather than reading
//! [`.yidam/corpus/`](../../.yidam/corpus/) directly. The corpus is markdown and YAML written
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
//! [`scenario`](../../.yidam/corpus/scenario/) result should be reproducible years from now
//! without a dependency resolution succeeding first. Serializing a fixed, known schema is a
//! few dozen lines, so that constraint costs nothing here.

#![forbid(unsafe_code)]

use edfund_core::Dollars;

/// The bundle schema version. Bump on any change to field names, units, or semantics.
///
/// `2.0.0` added the scenario inputs and checkpoints, and renamed the enrollment-change years
/// from FY2022-FY2024 to FY2024-FY2026 — the years the department's `ADM Data` sheet declares.
/// The values did not change; what they are called did, which is exactly the kind of silent
/// meaning change the version guard exists for.
pub const CONTRACT_VERSION: &str = "2.0.0";

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
            "    \"minimum_state_share\": {}\n",
            num(w.minimum_state_share)
        ));
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
        s.push_str("  ],\n  \"districts\": [\n");

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
                "\"enrollment_change\": {}",
                opt(d.enrollment_change)
            ));
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
        }
    }

    fn bundle(districts: Vec<District>, checkpoints: Vec<Checkpoint>) -> Bundle {
        Bundle {
            contract_version: CONTRACT_VERSION.into(),
            provenance: "test".into(),
            fiscal_year: 2027,
            statewide: zero_statewide(),
            checkpoints,
            districts,
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
            .contains("\"contract_version\": \"2.0.0\""));
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
