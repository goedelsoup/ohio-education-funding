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
//! # Why hand-rolled JSON
//!
//! The workspace has no external dependencies, deliberately — a committed
//! [`scenario`](../../.yidam/corpus/scenario/) result should be reproducible years from now
//! without a dependency resolution succeeding first. Serializing a fixed, known schema is a
//! few dozen lines, so that constraint costs nothing here.

#![forbid(unsafe_code)]

use edfund_core::Dollars;

/// The bundle schema version. Bump on any change to field names, units, or semantics.
pub const CONTRACT_VERSION: &str = "1.0.0";

/// One district, as the web layer needs it.
#[derive(Debug, Clone, PartialEq)]
pub struct District {
    /// Information Retrieval Number, the stable statewide identifier.
    pub irn: String,
    /// District name as published, including county.
    pub name: String,
    /// Base cost enrolled ADM, FY2027 model.
    pub adm: f64,
    /// District base cost per pupil, FY2027.
    pub base_cost_per_pupil: Dollars,
    /// State aid per pupil as the formula computes it, before the guarantee.
    pub formula_aid_per_pupil: Dollars,
    /// State aid per pupil as the district receives it.
    pub realized_aid_per_pupil: Dollars,
    /// Temporary transitional aid guarantee, total dollars.
    pub guarantee: Dollars,
    /// Assessed valuation per pupil, FY2023.
    pub valuation_per_pupil: Option<Dollars>,
    /// Effective Class 1 operating millage, TY2023.
    pub effective_class1_millage: Option<f64>,
    /// Total operating expenditure per pupil, FY2024.
    pub operating_expenditure_per_pupil: Option<Dollars>,
    /// Share of students economically disadvantaged, FY2024, as a fraction.
    pub economically_disadvantaged: Option<f64>,
    /// Enrollment change FY2024 to FY2026, as a fraction. FY2026 is partly departmental
    /// estimate rather than actual, since the calculator is published before that year closes.
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
#[derive(Debug, Clone, PartialEq)]
pub struct Statewide {
    /// Number of districts in the bundle.
    pub districts: usize,
    /// Districts funded by the guarantee.
    pub on_guarantee: usize,
    /// Districts at the 20-mill floor.
    pub at_millage_floor: usize,
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
}

/// The exported feed.
#[derive(Debug, Clone, PartialEq)]
pub struct Bundle {
    /// Schema version; see [`CONTRACT_VERSION`].
    pub contract_version: String,
    /// What the figures describe and where they came from.
    pub provenance: String,
    /// Statewide aggregates.
    pub statewide: Statewide,
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
        format!("{:.4}", v)
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
        let mut s = String::with_capacity(self.districts.len() * 220 + 1024);
        s.push_str("{\n");
        s.push_str(&format!(
            "  \"contract_version\": \"{}\",\n",
            escape(&self.contract_version)
        ));
        s.push_str(&format!(
            "  \"provenance\": \"{}\",\n",
            escape(&self.provenance)
        ));
        let w = &self.statewide;
        s.push_str("  \"statewide\": {\n");
        s.push_str(&format!("    \"districts\": {},\n", w.districts));
        s.push_str(&format!("    \"on_guarantee\": {},\n", w.on_guarantee));
        s.push_str(&format!(
            "    \"at_millage_floor\": {},\n",
            w.at_millage_floor
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
            "    \"guarantee_total\": {}\n",
            num(w.guarantee_total)
        ));
        s.push_str("  },\n  \"districts\": [\n");
        for (i, d) in self.districts.iter().enumerate() {
            s.push_str("    {");
            s.push_str(&format!("\"irn\": \"{}\", ", escape(&d.irn)));
            s.push_str(&format!("\"name\": \"{}\", ", escape(&d.name)));
            s.push_str(&format!("\"adm\": {}, ", num(d.adm)));
            s.push_str(&format!(
                "\"base_cost_per_pupil\": {}, ",
                num(d.base_cost_per_pupil)
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
            base_cost_per_pupil: 8_100.0,
            formula_aid_per_pupil: 6_400.0,
            realized_aid_per_pupil: 6_400.0,
            guarantee: 0.0,
            valuation_per_pupil: Some(279_983.24),
            effective_class1_millage: Some(20.0),
            operating_expenditure_per_pupil: Some(11_986.62),
            economically_disadvantaged: Some(0.3881),
            enrollment_change: Some(-0.03),
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
        let b = Bundle {
            contract_version: CONTRACT_VERSION.into(),
            provenance: "test".into(),
            statewide: zero_statewide(),
            districts: vec![odd],
        };
        let json = b.to_json();
        assert!(json.contains(r#"St. \"Mary\" \\ Local"#));
    }

    fn zero_statewide() -> Statewide {
        Statewide {
            districts: 1,
            on_guarantee: 0,
            at_millage_floor: 1,
            median_valuation_per_pupil: 0.0,
            median_operating_expenditure_per_pupil: 0.0,
            wealth_neutrality_formula: 0.0,
            wealth_neutrality_realized: 0.0,
            guarantee_total: 0.0,
        }
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
        let b = Bundle {
            contract_version: CONTRACT_VERSION.into(),
            provenance: "test".into(),
            statewide: zero_statewide(),
            districts: vec![sparse],
        };
        let json = b.to_json();
        assert!(json.contains("\"valuation_per_pupil\": null"));
        assert!(
            !json.contains("\"valuation_per_pupil\": 0"),
            "a missing value must not be indistinguishable from zero"
        );
    }

    #[test]
    fn serialization_is_deterministic() {
        let b = Bundle {
            contract_version: CONTRACT_VERSION.into(),
            provenance: "test".into(),
            statewide: zero_statewide(),
            districts: vec![sample(), sample()],
        };
        assert_eq!(b.to_json(), b.to_json());
    }

    #[test]
    fn the_bundle_declares_its_contract_version() {
        let b = Bundle {
            contract_version: CONTRACT_VERSION.into(),
            provenance: "test".into(),
            statewide: zero_statewide(),
            districts: vec![],
        };
        assert!(b.to_json().contains("\"contract_version\": \"1.0.0\""));
    }
}
