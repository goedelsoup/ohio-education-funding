//! Writing the feed out.
//!
//! Hand-rolled, like everything here: no serialization crate, one `to_json` that appends to a
//! single buffer. [`crate::json`] holds the RAII writer that closes what it opens.

use crate::*;

pub(crate) fn escape(s: &str) -> String {
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

pub(crate) fn num(v: f64) -> String {
    if v.is_finite() {
        // Trim in place. `format!` then `.trim_end_matches(..).to_string()` allocates twice
        // for every number, and the feed carries roughly 73,000 of them.
        let mut out = format!("{v:.4}");
        out.truncate(out.trim_end_matches('0').trim_end_matches('.').len());
        out
    } else {
        "null".into()
    }
}

/// A fraction, to eight places.
///
/// [`num`] rounds to four, which is right for dollars and wrong for a share of one: a district
/// contributing 0.00060997 of a House district would be stored as 0.0006, and the shares of a
/// district split many ways would no longer sum to one in the feed even though they do in the
/// arithmetic that produced them. Eight places keeps the published figures self-consistent, so a
/// consumer adding them up gets the same answer this repository does.
pub(crate) fn share(v: f64) -> String {
    if v.is_finite() {
        format!("{v:.8}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        "null".into()
    }
}

/// One `FinanceYear` as a JSON object. Shared so the per-district and statewide arrays cannot
/// drift into different field names.
pub(crate) fn finance_year(y: &FinanceYear) -> String {
    format!(
        "{{\"fiscal_year\": {}, \"state_aid\": {}, \"local_tax\": {}, \
         \"total_revenue\": {}, \"total_expenditure\": {}, \"ending_cash\": {}}}",
        y.fiscal_year,
        num(y.state_aid),
        num(y.local_tax),
        num(y.total_revenue),
        num(y.total_expenditure),
        num(y.ending_cash)
    )
}

/// One fiscal year of the casino fund, per district. Two fields, and no third by design — see
/// [`District::casino`] for why a per-pupil figure is not one of them.
pub(crate) fn casino_year(y: &CasinoYear) -> String {
    format!(
        "{{\"fiscal_year\": {}, \"amount\": {}}}",
        y.fiscal_year,
        num(y.amount)
    )
}

/// `"<name>": {"<first>": [...], "<second>": [...], …}` — two parallel arrays and any scalars.
///
/// Closed here rather than by the caller. The first version left it open so that callers with a
/// trailing field could add one before the brace, and three of the five call sites then forgot to
/// close it — which nested `career_technical` inside `special_education` and made `dpia` a child
/// of `english_learners`. An object that is only valid if the caller remembers something is a
/// worse interface than one that takes the something.
pub(crate) fn array_pair(
    name: &str,
    first: &str,
    a: &[f64],
    second: &str,
    b: &[f64],
    extra: &[(&str, f64)],
) -> String {
    let mut s = format!("\"{name}\": ");
    {
        let mut o = json::Obj::new(&mut s);
        {
            let mut list = o.arr(first);
            for x in a {
                list.num(*x);
            }
        }
        {
            let mut list = o.arr(second);
            for x in b {
                list.num(*x);
            }
        }
        for (key, value) in extra {
            o.num(key, *value);
        }
    }
    s
}

/// `"<name>": {"k": v, …}` — numeric fields, boolean flags, then nullable numbers.
///
/// Closed here rather than by the caller, and `opts` is why that stays true. One call site
/// needed three nullable fields on the end and got them by chopping the closing brace back
/// off with `truncate(s.len() - 1)` — reproducing by hand the hazard this function was
/// closed to prevent. A parameter is the fix: a caller that can reopen the object will
/// eventually forget to close it.
pub(crate) fn fields(
    name: &str,
    entries: &[(&str, f64)],
    flags: &[(&str, bool)],
    opts: &[(&str, Option<f64>)],
) -> String {
    let mut s = format!("\"{name}\": ");
    {
        let mut o = json::Obj::new(&mut s);
        for (key, value) in entries {
            o.num(key, *value);
        }
        for (key, value) in flags {
            o.flag(key, *value);
        }
        for (key, value) in opts {
            o.opt(key, *value);
        }
    }
    s
}

pub(crate) fn opt(v: Option<f64>) -> String {
    v.map_or_else(|| "null".into(), num)
}

impl Bundle {
    /// Serialize to JSON.
    ///
    /// Deterministic: the same bundle always produces byte-identical output, so a committed
    /// feed diffs cleanly and a regenerated one shows only real changes.
    #[must_use]
    pub fn to_json(&self) -> String {
        // Measured against the committed feed: 6,188,834 bytes over 609 districts, ~10,160
        // each. The old hint of 320 reserved 3% of that, so the buffer doubled roughly five
        // times on every run and each doubling memcpy'd a multi-megabyte string.
        let mut s = String::with_capacity(self.districts.len() * 10_500 + 65_536);
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

        // Sorted on the way out rather than trusted to arrive sorted, so the committed feed is
        // byte-identical whatever order the caller assembled these in. Every fixture in this
        // repository has to rebuild identically from a clean checkout.
        let mut years: Vec<&SeriesYear> = self.series_years.iter().collect();
        years.sort_by(|a, b| a.series.cmp(&b.series));
        s.push_str("  \"series_years\": [\n");
        for (i, y) in years.iter().enumerate() {
            s.push_str("    ");
            {
                let mut o = json::Obj::new(&mut s);
                o.text("series", &y.series);
                o.text("kind", y.kind.as_str());
                o.text("label", &y.label);
                o.text("source", &y.source);
            }
            if i + 1 < years.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        let w = &self.statewide;
        s.push_str("  \"statewide\": {\n");
        s.push_str(&format!("    \"districts\": {},\n", w.districts));
        s.push_str(&format!("    \"on_guarantee\": {},\n", w.on_guarantee));
        s.push_str(&format!(
            "    \"at_millage_floor\": {},\n",
            w.at_millage_floor
        ));
        s.push_str(&format!(
            "    \"near_millage_floor\": {},\n",
            w.near_millage_floor
        ));
        s.push_str(&format!(
            "    \"median_voted_millage\": {},\n",
            num(w.median_voted_millage)
        ));
        s.push_str(&format!(
            "    \"median_effective_millage\": {},\n",
            num(w.median_effective_millage)
        ));
        s.push_str(&format!(
            "    \"median_millage_reduction\": {},\n",
            num(w.median_millage_reduction)
        ));
        for (key, value) in [
            ("median_yield_per_mill", w.median_yield_per_mill),
            ("min_yield_per_mill", w.min_yield_per_mill),
            ("max_yield_per_mill", w.max_yield_per_mill),
            ("median_sd1_value_per_pupil", w.median_sd1_value_per_pupil),
            ("median_regime_difference", w.median_regime_difference),
        ] {
            s.push_str(&format!("    \"{key}\": {},\n", num(value)));
        }
        for (key, value) in [
            (
                "districts_without_targeted_assistance",
                w.districts_without_targeted_assistance,
            ),
            ("below_charge_off_rate", w.below_charge_off_rate),
            (
                "charge_off_exceeds_base_cost",
                w.charge_off_exceeds_base_cost,
            ),
        ] {
            s.push_str(&format!("    \"{key}\": {value},\n"));
        }
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
        s.push_str("    \"finances\": [");
        s.push_str(
            &w.finances
                .iter()
                .map(finance_year)
                .collect::<Vec<_>>()
                .join(", "),
        );
        s.push_str("],\n");
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
                 \"median_performance_on_formula\": {}, \
                 \"median_federal_share\": {}, \"max_federal_share\": {}, \
                 \"federal_share_above_tenth\": {}, \
                 \"federal_share_vs_performance\": {}, \
                 \"federal_share_vs_performance_raw\": {}, \
                 \"growth_measures_disagree\": {}, \
                 \"growth_measures_determinate\": {}, \
                 \"growth_measures_disagree_materially\": {}, \
                 \"growth_measure_agreement\": {}}}\n",
                o.districts,
                num(o.poverty_vs_performance),
                num(o.guarantee_vs_performance),
                num(o.guarantee_vs_performance_controlled),
                num(o.spending_vs_growth_controlled),
                num(o.weighted_spending_vs_performance),
                num(o.enrolled_spending_vs_performance),
                num(o.median_performance_on_guarantee),
                num(o.median_performance_on_formula),
                num(o.median_federal_share),
                num(o.max_federal_share),
                o.federal_share_above_tenth,
                num(o.federal_share_vs_performance),
                num(o.federal_share_vs_performance_raw),
                o.growth_measures_disagree,
                o.growth_measures_determinate,
                o.growth_measures_disagree_materially,
                num(o.growth_measure_agreement),
            )),
        }
        s.push_str("  },\n");

        s.push_str("  \"checkpoints\": [\n");
        for (i, c) in self.checkpoints.iter().enumerate() {
            s.push_str("    ");
            {
                let mut o = json::Obj::new(&mut s);
                o.text("label", &c.label);
                {
                    let mut policy = o.obj("policy");
                    policy.text("guarantee", c.policy.guarantee);
                    policy.num("guarantee_argument", c.policy.guarantee_argument);
                    policy.num("base_cost_scale", c.policy.base_cost_scale);
                    policy.num("minimum_state_share", c.policy.minimum_state_share);
                    policy.num("phase_in_base_cost", c.policy.phase_in_base_cost);
                    policy.num("phase_in_categorical", c.policy.phase_in_categorical);
                }
                o.num("cost", c.cost);
                o.num("realized_aid", c.realized_aid);
                o.count("gainers", c.gainers);
                o.count("losers", c.losers);
                o.count("unmoved", c.unmoved);
                o.count("on_guarantee", c.on_guarantee);
                o.count("held_throughout", c.held_throughout);
                o.count("lifted_off", c.lifted_off);
                o.count("pushed_on", c.pushed_on);
            }
            if i + 1 < self.checkpoints.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        s.push_str("  \"drafts\": [\n");
        for (i, d) in self.drafts.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"slug\": \"{}\", \"provisions\": [",
                escape(&d.slug)
            ));
            for (j, p) in d.provisions.iter().enumerate() {
                s.push_str(&format!(
                    "\n      {{\"ordinal\": {}, \"title\": \"{}\", \"authority\": \"{}\", \
                     \"parameter\": \"{}\", \"lever\": \"{}\", \"proposed\": \"{}\", \
                     \"note\": \"{}\"}}{}",
                    p.ordinal,
                    escape(&p.title),
                    escape(&p.authority),
                    escape(&p.parameter),
                    escape(&p.lever),
                    escape(&p.proposed),
                    escape(&p.note),
                    if j + 1 < d.provisions.len() { "," } else { "" }
                ));
            }
            s.push_str("\n    ]}");
            if i + 1 < self.drafts.len() {
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

        match &self.deflator {
            None => s.push_str("  \"deflator\": null,\n"),
            Some(deflator) => s.push_str(&format!(
                "  \"deflator\": {{\"label\": \"{}\", \"points\": [{}]}},\n",
                escape(&deflator.label),
                deflator
                    .points
                    .iter()
                    .map(|(year, index)| format!(
                        "{{\"fiscal_year\": {year}, \"index\": {}}}",
                        num(*index)
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }

        match &self.national {
            None => s.push_str("  \"national\": null,\n"),
            Some(n) => {
                s.push_str(&format!(
                    "  \"national\": {{\"fiscal_year\": {}, \"ohio_local_rank\": {}, \
                     \"ohio_state_rank\": {}, \"ohio_spending_rank\": {}, \
                     \"ohio_property_tax_rank\": {}, \"independent_states\": {}, \
                     \"national_local_share\": {}, \"national_state_share\": {}, \
                     \"national_spending_per_pupil\": {}, \"states\": [",
                    n.fiscal_year,
                    n.ohio_local_rank,
                    n.ohio_state_rank,
                    n.ohio_spending_rank,
                    n.ohio_property_tax_rank,
                    n.independent_states,
                    num(n.national_local_share),
                    num(n.national_state_share),
                    num(n.national_spending_per_pupil),
                ));
                s.push_str(
                    &n.states
                        .iter()
                        .map(|state| {
                            let mut row = format!(
                                "{{\"fips\": \"{}\", \"name\": \"{}\", \"systems\": {}",
                                escape(&state.fips),
                                escape(&state.name),
                                state.systems
                            );
                            for (key, value) in [
                                ("enrollment", state.enrollment),
                                ("total_revenue", state.total_revenue),
                                ("federal_revenue", state.federal_revenue),
                                ("state_revenue", state.state_revenue),
                                ("local_revenue", state.local_revenue),
                                ("property_tax_revenue", state.property_tax_revenue),
                                ("parent_government_revenue", state.parent_government_revenue),
                                ("current_spending", state.current_spending),
                            ] {
                                row.push_str(&format!(", \"{key}\": {}", num(value)));
                            }
                            row.push('}');
                            row
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                s.push_str("]},\n");
            }
        }

        // One line per year, oldest first, because the thing a reader is looking for here is the
        // shape across years rather than any single one.
        s.push_str("  \"history\": [\n");
        for (i, h) in self.history.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"fiscal_year\": {}, \"districts\": {}",
                h.fiscal_year, h.districts
            ));
            for (key, value) in [
                ("local_share", h.local_share),
                ("state_share", h.state_share),
                ("federal_share", h.federal_share),
                ("poorest_local_per_pupil", h.poorest_local_per_pupil),
                ("richest_local_per_pupil", h.richest_local_per_pupil),
                ("gap_per_pupil", h.gap_per_pupil),
                ("state_closes_per_pupil", h.state_closes_per_pupil),
                ("federal_closes_per_pupil", h.federal_closes_per_pupil),
            ] {
                s.push_str(&format!(", \"{key}\": {}", num(value)));
            }
            s.push('}');
            if i + 1 < self.history.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        // `basis` is written on every row rather than only where it changes, because a consumer
        // reading one row must be able to tell what its share divides by without scanning for the
        // last row that said so. `streams` is written on every row for the stronger version of the
        // same reason: from FY2012 `share` is null, and a consumer meeting that has to be able to
        // tell a year with no single answer from a year whose figure went missing.
        s.push_str("  \"meal_program\": [\n");
        for (i, m) in self.meal_program.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"fiscal_year\": {}, \"sponsors\": {}, \"enrollment\": {}, \
                 \"approved\": {}, \"identified\": {}, \"share\": {}, \"floor\": {}, \
                 \"ceiling\": {}, \"without_applications\": {}, \"streams\": {}, \
                 \"basis\": \"{}\"}}",
                m.fiscal_year,
                m.sponsors,
                num(m.enrollment),
                num(m.approved),
                num(m.identified),
                m.share.map_or_else(|| "null".to_string(), num),
                num(m.floor),
                num(m.ceiling),
                num(m.without_applications),
                m.streams,
                escape(&m.basis)
            ));
            if i + 1 < self.meal_program.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        // Whole dollars per fiscal year and nothing else. There is deliberately no share, no
        // per-pupil and no per-district count on these rows: the fund's population is about a
        // thousand districts against this feed's 609, so every ratio a consumer might want from
        // here has a denominator that is not in the feed, and a row that offered one would be
        // inviting the join that makes it wrong.
        s.push_str("  \"casino\": [\n");
        for (i, c) in self.casino.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"fiscal_year\": {}, \"amount\": {}}}",
                c.fiscal_year,
                num(c.amount)
            ));
            if i + 1 < self.casino.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        // `source` on every row rather than only where it changes, for the same reason
        // `meal_program` writes its basis on every row: a consumer reading one row must be able to
        // tell which document it came from without scanning back for the last row that said.
        s.push_str("  \"appropriations\": [\n");
        for (i, a) in self.appropriations.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"fiscal_year\": {}, \"enacted\": {}, \"foundation_funding\": {}, \
                 \"items\": {}, \"source\": \"{}\"}}",
                a.fiscal_year,
                num(a.enacted),
                num(a.foundation_funding),
                a.items,
                escape(&a.source)
            ));
            if i + 1 < self.appropriations.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        // `established_by` is written even when empty, and `general_assembly` as `null` rather
        // than omitted, so a consumer can tell "this line names no founding act" from "this feed
        // predates the field".
        s.push_str("  \"appropriation_lines\": [\n");
        for (i, l) in self.appropriation_lines.iter().enumerate() {
            s.push_str(&format!(
                "    {{\"fund\": \"{}\", \"ali\": \"{}\", \"name\": \"{}\", \
                 \"established_by\": \"{}\", \"general_assembly\": {}, \"convened\": {}, \
                 \"discontinued\": {}}}",
                escape(&l.fund),
                escape(&l.ali),
                escape(&l.name),
                escape(&l.established_by),
                l.general_assembly
                    .map_or("null".to_string(), |v| v.to_string()),
                l.convened.map_or("null".to_string(), |v| v.to_string()),
                l.discontinued
            ));
            if i + 1 < self.appropriation_lines.len() {
                s.push(',');
            }
            s.push('\n');
        }
        s.push_str("  ],\n");

        // The two chambers. Written before the district array because a reader scanning the
        // feed meets the derived aggregate before the exact per-district figures it came from, and
        // the block's own `basis` field is where the estimate is labelled as one.
        for (key, seats) in [
            ("house_districts", &self.house_districts),
            ("senate_districts", &self.senate_districts),
        ] {
            s.push_str(&format!("  \"{key}\": [\n"));
            for (i, h) in seats.iter().enumerate() {
                s.push_str(&format!("    {{\"number\": \"{}\", ", escape(&h.number)));
                for (key, value) in [
                    ("adm", h.adm),
                    ("realized_aid", h.realized_aid),
                    ("base_cost_state_share", h.base_cost_state_share),
                    ("categorical_funding", h.categorical_funding),
                    ("guarantee", h.guarantee),
                ] {
                    s.push_str(&format!("\"{key}\": {}, ", num(value)));
                }
                s.push_str(&format!(
                    "\"districts_on_guarantee\": {}, \"districts_at_minimum_state_share\": {}, \
                 \"districts_wholly_inside\": {}, \"members\": [",
                    h.districts_on_guarantee,
                    h.districts_at_minimum_state_share,
                    h.districts_wholly_inside
                ));
                for (k, m) in h.members.iter().enumerate() {
                    if k > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&format!(
                        "{{\"irn\": \"{}\", \"name\": \"{}\", \"share\": {}, \
                     \"share_of_house_district\": {}, \"adm\": {}, \"realized_aid\": {}, \
                     \"wholly_inside\": {}}}",
                        escape(&m.irn),
                        escape(&m.name),
                        share(m.share),
                        share(m.share_of_house_district),
                        num(m.adm),
                        num(m.realized_aid),
                        m.wholly_inside
                    ));
                }
                s.push_str("]}");
                if i + 1 < seats.len() {
                    s.push(',');
                }
                s.push('\n');
            }
            s.push_str("  ],\n");
        }

        s.push_str("  \"districts\": [\n");

        for (i, d) in self.districts.iter().enumerate() {
            s.push_str("    {");
            s.push_str(&format!("\"irn\": \"{}\", ", escape(&d.irn)));
            s.push_str(&format!("\"name\": \"{}\", ", escape(&d.name)));
            s.push_str(&format!("\"county\": \"{}\", ", escape(&d.county)));
            s.push_str(&fields(
                "supplements",
                &[
                    ("performance", d.supplements.performance),
                    ("base_funding", d.supplements.base_funding),
                    ("growth", d.supplements.growth),
                    ("enrollment_change", d.supplements.enrollment_change),
                ],
                &[
                    ("performance_eligible", d.supplements.performance_eligible),
                    ("growth_eligible", d.supplements.growth_eligible),
                ],
                &[
                    ("stars", d.supplements.stars),
                    ("progress", d.supplements.progress),
                    ("growth_forgone", d.supplements.growth_forgone),
                ],
            ));
            s.push_str(", ");
            match &d.national {
                None => s.push_str("\"national\": null, "),
                Some(n) => {
                    s.push_str(&fields(
                        "national",
                        &[
                            ("local_share", n.local_share),
                            ("local_share_percentile", n.local_share_percentile),
                            ("revenue_per_pupil", n.revenue_per_pupil),
                            (
                                "revenue_per_pupil_percentile",
                                n.revenue_per_pupil_percentile,
                            ),
                            ("spending_per_pupil", n.spending_per_pupil),
                            (
                                "spending_per_pupil_percentile",
                                n.spending_per_pupil_percentile,
                            ),
                        ],
                        &[],
                        &[],
                    ));
                    s.push_str(", ");
                }
            }
            s.push_str(&fields(
                "transition",
                &[
                    ("funding_base", d.transition.funding_base),
                    ("open_enrollment_prior", d.transition.open_enrollment_prior),
                    (
                        "open_enrollment_current",
                        d.transition.open_enrollment_current,
                    ),
                    (
                        "open_enrollment_threshold",
                        d.transition.open_enrollment_threshold,
                    ),
                    (
                        "open_enrollment_adjustment",
                        d.transition.open_enrollment_adjustment,
                    ),
                    ("fy21_funding_base", d.transition.fy21_funding_base),
                    ("transition_supplement", d.transition.transition_supplement),
                ],
                &[],
                &[],
            ));
            s.push_str(", ");
            s.push_str(&array_pair(
                "preschool_special_education",
                "adm",
                &d.preschool_special_education.adm,
                "aid",
                &d.preschool_special_education.aid,
                &[
                    ("total", d.preschool_special_education.total),
                    (
                        "flat_component",
                        d.preschool_special_education.flat_component,
                    ),
                    ("unprorated", d.preschool_special_education.unprorated),
                ],
            ));
            s.push_str(", ");
            s.push_str(&fields(
                "transportation",
                &[
                    ("public_riders", d.transportation.public_riders),
                    ("nonpublic_riders", d.transportation.nonpublic_riders),
                    ("community_riders", d.transportation.community_riders),
                    ("weighted_riders", d.transportation.weighted_riders),
                    ("per_rider_base", d.transportation.per_rider_base),
                    ("per_mile_base", d.transportation.per_mile_base),
                    (
                        "effective_state_share",
                        d.transportation.effective_state_share,
                    ),
                    ("school_bus", d.transportation.school_bus),
                    ("mass_transit", d.transportation.mass_transit),
                    ("other", d.transportation.other),
                    ("efficiency", d.transportation.efficiency),
                    ("density", d.transportation.density),
                    ("efficiency_index", d.transportation.efficiency_index),
                    ("district_density", d.transportation.district_density),
                    ("fy21_base", d.transportation.fy21_base),
                    ("guarantee", d.transportation.guarantee),
                    ("total", d.transportation.total),
                    ("special_education", d.transportation.special_education),
                    (
                        "special_education_unprorated",
                        d.transportation.special_education_unprorated,
                    ),
                ],
                &[("paid_on_miles", d.transportation.paid_on_miles)],
                &[],
            ));
            s.push_str(", ");
            s.push_str("\"house_districts\": ");
            {
                let mut list = json::Arr::new(&mut s);
                for h in &d.house_districts {
                    let mut o = list.obj();
                    o.text("number", &h.number);
                    o.share("share", h.share);
                }
            }
            s.push_str(", ");
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
            {
                // Two tax years of the Department of Taxation's own table, written as an array so
                // the change between them — which is the whole reason two are carried — is a
                // thing the page can iterate rather than two parallel field sets.
                s.push_str("\"property_tax\": [");
                for (j, y) in d.property_tax.iter().enumerate() {
                    if j > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&format!("{{\"tax_year\": {}, ", y.tax_year));
                    for (name, value) in [
                        ("class1_value", y.class1_value),
                        ("class2_value", y.class2_value),
                        ("public_utility_value", y.public_utility_value),
                        ("total_value", y.total_value),
                        ("agricultural_value", y.agricultural_value),
                        ("residential_value", y.residential_value),
                        ("commercial_value", y.commercial_value),
                        ("industrial_value", y.industrial_value),
                        ("mineral_value", y.mineral_value),
                        ("railroad_value", y.railroad_value),
                        ("class1_rate", y.class1_rate),
                        ("class2_rate", y.class2_rate),
                        ("class1_taxes_charged", y.class1_taxes_charged),
                        ("class2_taxes_charged", y.class2_taxes_charged),
                        ("real_property_taxes_charged", y.real_property_taxes_charged),
                        (
                            "public_utility_taxes_charged",
                            y.public_utility_taxes_charged,
                        ),
                        ("value_per_pupil", y.value_per_pupil),
                        ("adm", y.adm),
                    ] {
                        s.push_str(&format!("\"{name}\": {}, ", num(value)));
                    }
                    s.truncate(s.trim_end_matches(' ').trim_end_matches(',').len());
                    s.push('}');
                }
                s.push_str("], ");
            }
            match &d.spending_by_function {
                None => s.push_str("\"spending_by_function\": null, "),
                Some(f) => {
                    s.push_str("\"spending_by_function\": {");
                    for (name, value) in [
                        ("adm", f.adm),
                        ("operating_per_pupil", f.operating_per_pupil),
                        ("classroom_instruction", f.classroom_instruction),
                        ("nonclassroom", f.nonclassroom),
                        ("instruction", f.instruction),
                        ("pupil_support", f.pupil_support),
                        ("instructional_staff_support", f.instructional_staff_support),
                        ("general_admin", f.general_admin),
                        ("school_admin", f.school_admin),
                        ("operations_maintenance", f.operations_maintenance),
                        ("pupil_transportation", f.pupil_transportation),
                        ("other_support", f.other_support),
                        ("food_service", f.food_service),
                    ] {
                        s.push_str(&format!("\"{name}\": {}, ", num(value)));
                    }
                    s.truncate(s.trim_end_matches(' ').trim_end_matches(',').len());
                    s.push_str("}, ");
                }
            }
            {
                // Twenty-two elements plus the two funded-position counts and the reconciliation
                // against the department's own figure. Written longhand for the same reason the
                // rest of this serializer is: no serde, so no derive.
                let b = &d.base_cost_build_up;
                s.push_str("\"base_cost_build_up\": {");
                for (name, value) in [
                    ("classroom_teachers", b.classroom_teachers),
                    ("special_teachers", b.special_teachers),
                    ("substitutes", b.substitutes),
                    ("professional_development", b.professional_development),
                    ("teachers", b.teachers),
                    ("counselors", b.counselors),
                    ("librarians", b.librarians),
                    ("wellness", b.wellness),
                    ("academic_cocurricular", b.academic_cocurricular),
                    ("safety", b.safety),
                    ("supplies", b.supplies),
                    ("technology", b.technology),
                    ("student_support", b.student_support),
                    ("superintendent", b.superintendent),
                    ("treasurer", b.treasurer),
                    ("other_administrators", b.other_administrators),
                    ("fiscal_support", b.fiscal_support),
                    ("emis", b.emis),
                    ("leadership_support", b.leadership_support),
                    ("itc", b.itc),
                    ("district_leadership", b.district_leadership),
                    ("building_leadership_staff", b.building_leadership_staff),
                    ("building_support", b.building_support),
                    ("building_operation", b.building_operation),
                    ("building_leadership", b.building_leadership),
                    ("athletic_cocurricular", b.athletic_cocurricular),
                    ("computed_aggregate", b.computed_aggregate),
                    ("published_aggregate", b.published_aggregate),
                    ("residual", b.residual),
                ] {
                    s.push_str(&format!("\"{name}\": {}, ", num(value)));
                }
                s.push_str(&format!(
                    "\"funded_classroom_teachers\": {}, \"funded_special_teachers\": {}}}, ",
                    num(b.funded_classroom_teachers),
                    num(b.funded_special_teachers)
                ));
            }
            s.push_str(&format!(
                "\"base_cost_state_share\": {}, ",
                num(d.base_cost_state_share)
            ));
            s.push_str(&format!(
                "\"categorical_funding\": {}, \"base_cost_denominated_categoricals\": {}, ",
                num(d.categorical_funding),
                num(d.base_cost_denominated_categoricals)
            ));
            // Special education, then the other five, then the six totals. Emitted once — this
            // block and the categoricals beside it were pasted twice, so every district in the
            // shipped feed carried both keys twice. JSON takes the last, so nothing was wrong on
            // the page and about 120KB of the payload was a copy of itself.
            s.push_str(&array_pair(
                "special_education",
                "adm",
                &d.special_education.adm,
                "aid",
                &d.special_education.aid,
                &[],
            ));
            s.push_str(", ");
            s.push_str(&array_pair(
                "career_technical",
                "fte",
                &d.career_technical.fte,
                "aid",
                &d.career_technical.aid,
                &[(
                    "associated_services",
                    d.career_technical.associated_services,
                )],
            ));
            s.push_str(", ");
            s.push_str(&array_pair(
                "english_learners",
                "adm",
                &d.english_learners.adm,
                "aid",
                &d.english_learners.aid,
                &[],
            ));
            s.push_str(", ");
            s.push_str(&fields(
                "dpia",
                &[
                    (
                        "economically_disadvantaged_adm",
                        d.dpia.economically_disadvantaged_adm,
                    ),
                    ("directly_certified_adm", d.dpia.directly_certified_adm),
                    ("weighted_adm", d.dpia.weighted_adm),
                    ("percentage", d.dpia.percentage),
                    ("index", d.dpia.index),
                ],
                &[],
                &[],
            ));
            s.push_str(", ");
            s.push_str(&fields(
                "targeted_assistance",
                &[
                    (
                        "property_valuation",
                        d.targeted_assistance.property_valuation,
                    ),
                    (
                        "federal_gross_income",
                        d.targeted_assistance.federal_gross_income,
                    ),
                    ("weighted_wealth", d.targeted_assistance.weighted_wealth),
                    ("capacity_index", d.targeted_assistance.capacity_index),
                    ("capacity_amount", d.targeted_assistance.capacity_amount),
                    ("wealth_per_pupil", d.targeted_assistance.wealth_per_pupil),
                    ("wealth_index", d.targeted_assistance.wealth_index),
                    ("wealth_amount", d.targeted_assistance.wealth_amount),
                    ("resident_adm", d.targeted_assistance.resident_adm),
                ],
                &[(
                    "supplement_eligible",
                    d.targeted_assistance.supplement_eligible,
                )],
                &[],
            ));
            s.push_str(", ");
            s.push_str(&fields(
                "gifted",
                &[
                    ("identification", d.gifted.identification),
                    ("referral", d.gifted.referral),
                    ("fte_k8", d.gifted.fte_k8),
                    ("fte_9_12", d.gifted.fte_9_12),
                    ("coordinator_units", d.gifted.coordinator_units),
                    ("coordinator_aid", d.gifted.coordinator_aid),
                    ("specialist_k8_units", d.gifted.specialist_k8_units),
                    ("specialist_k8_aid", d.gifted.specialist_k8_aid),
                    ("specialist_9_12_units", d.gifted.specialist_9_12_units),
                    ("specialist_9_12_aid", d.gifted.specialist_9_12_aid),
                ],
                &[("entirely_on_the_floor", d.gifted.entirely_on_the_floor)],
                &[],
            ));
            s.push_str(&format!(
                ", \"categorical_adm\": {}, ",
                num(d.categorical_adm)
            ));
            s.push_str(&fields(
                "categoricals",
                &[
                    ("targeted_assistance", d.categoricals.targeted_assistance),
                    ("special_education", d.categoricals.special_education),
                    ("dpia", d.categoricals.dpia),
                    ("english_learners", d.categoricals.english_learners),
                    ("gifted", d.categoricals.gifted),
                    ("career_technical", d.categoricals.career_technical),
                ],
                &[],
                &[],
            ));
            s.push_str(", ");
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
                "\"near_millage_floor\": {}, ",
                d.near_millage_floor()
            ));
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
                "\"voted_operating_millage\": {}, ",
                opt(d.voted_operating_millage)
            ));
            s.push_str("\"millage\": ");
            match &d.millage {
                None => s.push_str("null, "),
                Some(m) => {
                    s.push('{');
                    s.push_str(&format!("\"tax_year\": {}, ", m.tax_year));
                    for (key, value) in [
                        ("prior_rate", m.prior_rate),
                        ("observed_rate", m.observed_rate),
                        ("predicted_rate", m.predicted_rate),
                        ("residual", m.residual),
                        ("yield_per_mill_per_pupil", m.yield_per_mill_per_pupil),
                    ] {
                        s.push_str(&format!("\"{key}\": {}, ", num(value)));
                    }
                    s.push_str(&format!("\"at_floor\": {}, ", m.at_floor));
                    s.push_str(&format!(
                        "\"cumulative_reduction\": {}",
                        opt(m.cumulative_reduction)
                    ));
                    s.push_str("}, ");
                }
            }
            s.push_str("\"regime\": ");
            match &d.regime {
                None => s.push_str("null, "),
                Some(r) => {
                    s.push('{');
                    s.push_str(&format!(
                        "\"charge_off_mills\": {}, ",
                        num(r.charge_off_mills)
                    ));
                    s.push_str(&format!(
                        "\"recognized_share\": {}, \"reappraisal_year\": {}, ",
                        share(r.recognized_share),
                        r.reappraisal_year
                    ));
                    for (key, value) in [
                        ("charge_off_local_share", r.charge_off_local_share),
                        ("local_capacity", r.local_capacity),
                        ("aid_charge_off", r.aid_charge_off),
                        ("aid_fsfp", r.aid_fsfp),
                        ("difference", r.difference),
                        ("residual", r.residual),
                        ("mills_short_of_charge_off", r.mills_short_of_charge_off),
                        ("overstated_by", r.overstated_by),
                    ] {
                        s.push_str(&format!("\"{key}\": {}, ", opt(value)));
                    }
                    s.push_str(&format!("\"exceeds_base_cost\": {}", r.exceeds_base_cost));
                    s.push_str("}, ");
                }
            }
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
                     \"progress_effect_size\": {}, \
                     \"progress_effect_size_one_year\": {}, \"per_enrolled_pupil\": {}, \
                     \"per_equivalent_pupil\": {}, \
                     \"per_equivalent_pupil_federal\": {}, \
                     \"per_equivalent_pupil_state_local\": {}, \
                     \"economically_disadvantaged\": {}, \
                     \"english_learner\": {}, \"students_with_disabilities\": {}}}",
                    opt(o.performance_index),
                    opt(o.performance_index_prior),
                    opt(o.performance_index_earliest),
                    opt(o.progress_effect_size),
                    opt(o.progress_effect_size_one_year),
                    opt(o.per_enrolled_pupil),
                    opt(o.per_equivalent_pupil),
                    opt(o.per_equivalent_pupil_federal),
                    opt(o.per_equivalent_pupil_state_local),
                    opt(o.economically_disadvantaged),
                    opt(o.english_learner),
                    opt(o.students_with_disabilities),
                )),
            }
            s.push_str(", \"finances\": [");
            s.push_str(
                &d.finances
                    .iter()
                    .map(finance_year)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            s.push(']');
            s.push_str(", \"casino\": [");
            s.push_str(
                &d.casino
                    .iter()
                    .map(casino_year)
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            s.push(']');
            s.push_str(&format!(
                ", \"casino_counties\": {}",
                d.casino_counties
                    .map_or_else(|| "null".to_string(), |n| n.to_string())
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
mod helpers {
    use super::*;

    /// `num` trims to the shortest form that round-trips at four places.
    ///
    /// Added because its implementation changed — it used to allocate a second string to trim
    /// — and nothing pinned the output. The feed regenerating byte-identical caught it, but
    /// that is a 6 MB diff standing in for an assertion about six characters.
    #[test]
    fn a_number_is_trimmed_to_four_places_and_no_trailing_zeros() {
        assert_eq!(num(1.0), "1");
        assert_eq!(num(1.5), "1.5");
        assert_eq!(num(1.5000), "1.5");
        assert_eq!(num(0.0), "0");
        assert_eq!(num(-3.25), "-3.25");
        assert_eq!(num(1234.5678), "1234.5678");

        // Four places is the limit, and it rounds rather than truncates.
        assert_eq!(num(0.12345), "0.1235");
        assert_eq!(num(0.00004), "0");

        // A whole number keeps no decimal point, which is what the second trim is for.
        assert_eq!(num(42.0000), "42");

        // Non-finite is `null`, not a number, so a consumer never sees `inf` or `NaN`.
        assert_eq!(num(f64::INFINITY), "null");
        assert_eq!(num(f64::NEG_INFINITY), "null");
        assert_eq!(num(f64::NAN), "null");
    }
}
