//! Writing the feed out.
//!
//! Hand-rolled, like everything here: no serialization crate, one `to_json` that appends to a
//! single buffer. [`crate::json`] holds the RAII writer that closes what it opens.

use crate::*;

pub(crate) use edfund_core::decimal::escape;

/// A dollar or a count, to four places.
///
/// `null` rather than a literal for a non-finite value: `NaN` and `inf` are not JSON, and a
/// serializer that emitted either would produce a document no consumer could parse.
pub(crate) fn num(v: f64) -> String {
    if v.is_finite() {
        edfund_core::decimal::trimmed(v, 4)
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
        edfund_core::decimal::trimmed(v, 8)
    } else {
        "null".into()
    }
}

/// One `FinanceYear`, appended to an open array. Shared so the per-district and statewide arrays
/// cannot drift into different field names.
pub(crate) fn finance_year(list: &mut json::Arr<'_>, y: &FinanceYear) {
    let mut o = list.obj();
    o.count("fiscal_year", y.fiscal_year);
    o.opt("state_aid", y.state_aid);
    o.opt("local_tax", y.local_tax);
    o.opt("total_revenue", y.total_revenue);
    o.opt("total_expenditure", y.total_expenditure);
    o.opt("ending_cash", y.ending_cash);
}

/// One fiscal year of the casino fund, per district. Two fields, and no third by design — see
/// [`District::casino`] for why a per-pupil figure is not one of them.
pub(crate) fn casino_year(list: &mut json::Arr<'_>, y: &CasinoYear) {
    let mut o = list.obj();
    o.count("fiscal_year", y.fiscal_year);
    o.num("amount", y.amount);
}

/// `"<name>": {"<first>": [...], "<second>": [...], …}` — two parallel arrays and any scalars.
///
/// Written into the caller's object rather than returned as a `String`, which is what removed the
/// `push_str(", ")` that used to follow every call: the separator is the container's business now.
/// The object still closes itself. The first version left it open so that callers with a trailing
/// field could add one before the brace, and three of the five call sites then forgot to close it
/// — which nested `career_technical` inside `special_education` and made `dpia` a child of
/// `english_learners`.
pub(crate) fn array_pair(
    o: &mut json::Obj<'_>,
    name: &str,
    first: &str,
    a: &[f64],
    second: &str,
    b: &[f64],
    extra: &[(&str, f64)],
) {
    let mut inner = o.obj(name);
    {
        let mut list = inner.arr(first);
        for x in a {
            list.num(*x);
        }
    }
    {
        let mut list = inner.arr(second);
        for x in b {
            list.num(*x);
        }
    }
    for (key, value) in extra {
        inner.num(key, *value);
    }
}

/// `"<name>": {"k": v, …}` — numeric fields, boolean flags, then nullable numbers.
///
/// Three parameters rather than one, because a caller that can reopen the object will eventually
/// forget to close it: one call site needed three nullable fields on the end and got them by
/// chopping the closing brace back off with `truncate(s.len() - 1)`, reproducing by hand the
/// hazard this function was closed to prevent.
pub(crate) fn fields(
    o: &mut json::Obj<'_>,
    name: &str,
    entries: &[(&str, f64)],
    flags: &[(&str, bool)],
    opts: &[(&str, Option<f64>)],
) {
    let mut inner = o.obj(name);
    for (key, value) in entries {
        inner.num(key, *value);
    }
    for (key, value) in flags {
        inner.flag(key, *value);
    }
    for (key, value) in opts {
        inner.opt(key, *value);
    }
}

pub(crate) fn opt(v: Option<f64>) -> String {
    v.map_or_else(|| "null".into(), num)
}

impl Bundle {
    /// Serialize to JSON.
    ///
    /// Deterministic: the same bundle always produces byte-identical output, so a committed
    /// feed diffs cleanly and a regenerated one shows only real changes.
    ///
    /// # The punctuation is the writer's, not this function's
    ///
    /// This was 1,058 lines that maintained JSON syntax by hand: 84 `push_str(&format!(…))`,
    /// 68 bare `push_str("`, 17 raw `push` of a brace or a comma, separators in four
    /// incompatible styles, and two places that wrote a trailing `", "` and then chopped it back
    /// off with `truncate`. Every one of those is a place a document could go out of balance and
    /// nothing would say so until a consumer failed to parse 6.19 MB.
    ///
    /// Now every delimiter comes from the writer in `json.rs`, which closes what it opens in `Drop`
    /// and tracks its own separators. The bytes are unchanged — the committed feed is the regression
    /// test, and it is identical to the byte.
    #[must_use]
    pub fn to_json(&self) -> String {
        // Measured against the committed feed: 6,188,834 bytes over 609 districts, ~10,160
        // each. The old hint of 320 reserved 3% of that, so the buffer doubled roughly five
        // times on every run and each doubling memcpy'd a multi-megabyte string.
        let mut s = String::with_capacity(self.districts.len() * 10_500 + 65_536);
        {
            let mut doc = json::Obj::block(&mut s, "  ");
            doc.text("contract_version", &self.contract_version);
            doc.text("provenance", &self.provenance);
            doc.count("fiscal_year", self.fiscal_year);

            // Sorted on the way out rather than trusted to arrive sorted, so the committed feed
            // is byte-identical whatever order the caller assembled these in. Every fixture in
            // this repository has to rebuild identically from a clean checkout.
            let mut years: Vec<&SeriesYear> = self.series_years.iter().collect();
            years.sort_by(|a, b| a.series.cmp(&b.series));
            {
                let mut list = doc.block_arr("series_years", "    ");
                for y in &years {
                    let mut o = list.obj();
                    o.text("series", &y.series);
                    o.text("kind", y.kind.as_str());
                    o.text("label", &y.label);
                    o.text("source", &y.source);
                }
            }

            let w = &self.statewide;
            {
                let mut st = doc.block_obj("statewide", "    ");
                st.count("districts", w.districts);
                st.count("on_guarantee", w.on_guarantee);
                st.count("at_millage_floor", w.at_millage_floor);
                st.count("near_millage_floor", w.near_millage_floor);
                st.num("median_voted_millage", w.median_voted_millage);
                st.num("median_effective_millage", w.median_effective_millage);
                st.num("median_millage_reduction", w.median_millage_reduction);
                for (key, value) in [
                    ("median_yield_per_mill", w.median_yield_per_mill),
                    ("min_yield_per_mill", w.min_yield_per_mill),
                    ("max_yield_per_mill", w.max_yield_per_mill),
                    ("median_sd1_value_per_pupil", w.median_sd1_value_per_pupil),
                    ("median_regime_difference", w.median_regime_difference),
                ] {
                    st.num(key, value);
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
                    st.count(key, value);
                }
                st.count("at_minimum_state_share", w.at_minimum_state_share);
                st.num("median_valuation_per_pupil", w.median_valuation_per_pupil);
                st.num(
                    "median_operating_expenditure_per_pupil",
                    w.median_operating_expenditure_per_pupil,
                );
                st.num("wealth_neutrality_formula", w.wealth_neutrality_formula);
                st.num("wealth_neutrality_realized", w.wealth_neutrality_realized);
                st.num("guarantee_total", w.guarantee_total);
                st.num("realized_aid_total", w.realized_aid_total);
                st.num("minimum_state_share", w.minimum_state_share);
                st.num(
                    "targeted_assistance_median_weighted_wealth",
                    w.targeted_assistance_median_weighted_wealth,
                );
                st.num(
                    "targeted_assistance_median_wealth_per_pupil",
                    w.targeted_assistance_median_wealth_per_pupil,
                );
                st.num("preschool_appropriation", w.preschool_appropriation);
                // Eight places, like `sigma` above and unlike everything else here: the district
                // page prints the factor as published and the feed's four-place default would
                // round 0.968_544_48 to 0.9685, which is a different number that happens to start
                // the same way.
                st.raw(
                    "preschool_proration",
                    &format!("{:.8}", w.preschool_proration),
                );
                st.num("preschool_total", w.preschool_total);
                {
                    let mut list = st.arr("finances");
                    for y in &w.finances {
                        finance_year(&mut list, y);
                    }
                }
                match &w.outcomes {
                    None => st.raw("outcomes", "null"),
                    Some(o) => {
                        let mut out = st.obj("outcomes");
                        out.count("districts", o.districts);
                        for (key, value) in [
                            ("poverty_vs_performance", o.poverty_vs_performance),
                            ("guarantee_vs_performance", o.guarantee_vs_performance),
                            (
                                "guarantee_vs_performance_controlled",
                                o.guarantee_vs_performance_controlled,
                            ),
                            (
                                "spending_vs_growth_controlled",
                                o.spending_vs_growth_controlled,
                            ),
                            (
                                "weighted_spending_vs_performance",
                                o.weighted_spending_vs_performance,
                            ),
                            (
                                "enrolled_spending_vs_performance",
                                o.enrolled_spending_vs_performance,
                            ),
                            (
                                "median_performance_on_guarantee",
                                o.median_performance_on_guarantee,
                            ),
                            (
                                "median_performance_on_formula",
                                o.median_performance_on_formula,
                            ),
                            ("median_federal_share", o.median_federal_share),
                            ("max_federal_share", o.max_federal_share),
                        ] {
                            out.num(key, value);
                        }
                        out.count("federal_share_above_tenth", o.federal_share_above_tenth);
                        out.num(
                            "federal_share_vs_performance",
                            o.federal_share_vs_performance,
                        );
                        out.num(
                            "federal_share_vs_performance_raw",
                            o.federal_share_vs_performance_raw,
                        );
                        out.count("growth_measures_disagree", o.growth_measures_disagree);
                        out.count("growth_measures_determinate", o.growth_measures_determinate);
                        out.count(
                            "growth_measures_disagree_materially",
                            o.growth_measures_disagree_materially,
                        );
                        out.num("growth_measure_agreement", o.growth_measure_agreement);
                    }
                }
            }

            {
                let mut list = doc.block_arr("checkpoints", "    ");
                for c in &self.checkpoints {
                    let mut o = list.obj();
                    o.text("label", &c.label);
                    {
                        let mut policy = o.obj("policy");
                        policy.text("guarantee", c.policy.guarantee);
                        policy.num("guarantee_argument", c.policy.guarantee_argument);
                        policy.num("base_cost_scale", c.policy.base_cost_scale);
                        policy.num("minimum_state_share", c.policy.minimum_state_share);
                        policy.num("phase_in_general", c.policy.phase_in_general);
                        policy.num("phase_in_dpia", c.policy.phase_in_dpia);
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
                    o.count("at_minimum_state_share", c.at_minimum_state_share);
                    o.num("guarantee", c.guarantee);
                    o.num("formula_aid", c.formula_aid);
                }
            }

            {
                let mut list = doc.block_arr("drafts", "    ");
                for d in &self.drafts {
                    let mut o = list.obj();
                    o.text("slug", &d.slug);
                    let mut provisions = o.block_arr("provisions", "      ");
                    for p in &d.provisions {
                        let mut e = provisions.obj();
                        e.count("ordinal", p.ordinal);
                        e.text("title", &p.title);
                        e.text("authority", &p.authority);
                        e.text("parameter", &p.parameter);
                        e.text("lever", &p.lever);
                        e.text("proposed", &p.proposed);
                        e.text("note", &p.note);
                    }
                }
            }

            match &self.projection {
                None => doc.raw("projection", "null"),
                Some(p) => {
                    let mut pr = doc.block_obj("projection", "    ");
                    pr.count("base_year", p.base_year);
                    pr.count("horizon", p.horizon);
                    pr.text("method", &p.method);
                    pr.num("damping", p.damping);
                    // Six places, not the four `num` gives: sigma is a growth rate around 0.02,
                    // and rounding it to 0.0234 would move a ten-year band by enough to fail its
                    // own checkpoint.
                    pr.raw("sigma", &format!("{:.6}", p.sigma));
                    pr.num("z", p.z);
                    pr.text("prior_source", &p.prior_source);
                    pr.num("statute_ends", f64::from(p.statute_ends));
                    let mut list = pr.block_arr("checkpoints", "      ");
                    for c in &p.checkpoints {
                        let mut o = list.obj();
                        o.text("label", &c.label);
                        {
                            let mut policy = o.obj("policy");
                            policy.text("guarantee", c.policy.guarantee);
                            policy.num("guarantee_argument", c.policy.guarantee_argument);
                            policy.num("base_cost_scale", c.policy.base_cost_scale);
                            policy.num("minimum_state_share", c.policy.minimum_state_share);
                            policy.num("phase_in_general", c.policy.phase_in_general);
                            policy.num("phase_in_dpia", c.policy.phase_in_dpia);
                        }
                        o.count("fiscal_year", c.fiscal_year);
                        o.num("realized_aid", c.realized_aid);
                        o.num("low", c.low);
                        o.num("high", c.high);
                        o.num("adm", c.adm);
                        o.count("on_guarantee", c.on_guarantee);
                    }
                }
            }

            match &self.deflator {
                None => doc.raw("deflator", "null"),
                Some(deflator) => {
                    let mut o = doc.obj("deflator");
                    o.text("label", &deflator.label);
                    {
                        let mut list = o.arr("points");
                        for (year, index) in &deflator.points {
                            let mut e = list.obj();
                            e.count("fiscal_year", year);
                            e.num("index", *index);
                        }
                    }
                    // Written even when empty. A consumer that has to ask whether the field is
                    // there before trusting the points has learnt nothing from it.
                    let mut list = o.arr("uncovered");
                    for year in &deflator.uncovered {
                        list.count(*year);
                    }
                }
            }

            match &self.national {
                None => doc.raw("national", "null"),
                Some(n) => {
                    let mut o = doc.obj("national");
                    o.count("fiscal_year", n.fiscal_year);
                    o.count("ohio_local_rank", n.ohio_local_rank);
                    o.count("ohio_state_rank", n.ohio_state_rank);
                    o.count("ohio_spending_rank", n.ohio_spending_rank);
                    o.count("ohio_property_tax_rank", n.ohio_property_tax_rank);
                    o.count("independent_states", n.independent_states);
                    o.num("national_local_share", n.national_local_share);
                    o.num("national_state_share", n.national_state_share);
                    o.num("national_spending_per_pupil", n.national_spending_per_pupil);
                    let mut list = o.arr("states");
                    for state in &n.states {
                        let mut e = list.obj();
                        e.text("fips", &state.fips);
                        e.text("name", &state.name);
                        e.count("systems", state.systems);
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
                            e.num(key, value);
                        }
                    }
                }
            }

            // One line per year, oldest first, because the thing a reader is looking for here is
            // the shape across years rather than any single one.
            {
                let mut list = doc.block_arr("history", "    ");
                for h in &self.history {
                    let mut o = list.obj();
                    o.count("fiscal_year", h.fiscal_year);
                    o.count("districts", h.districts);
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
                        o.num(key, value);
                    }
                }
            }

            // `basis` is written on every row rather than only where it changes, because a
            // consumer reading one row must be able to tell what its share divides by without
            // scanning for the last row that said so. `streams` is written on every row for the
            // stronger version of the same reason: from FY2012 `share` is null, and a consumer
            // meeting that has to be able to tell a year with no single answer from a year whose
            // figure went missing.
            {
                let mut list = doc.block_arr("meal_program", "    ");
                for m in &self.meal_program {
                    let mut o = list.obj();
                    o.count("fiscal_year", m.fiscal_year);
                    o.count("sponsors", m.sponsors);
                    o.num("enrollment", m.enrollment);
                    o.num("approved", m.approved);
                    o.num("identified", m.identified);
                    o.opt("share", m.share);
                    o.num("floor", m.floor);
                    o.num("ceiling", m.ceiling);
                    o.num("without_applications", m.without_applications);
                    o.flag("comparable", m.comparable);
                    o.count("streams", m.streams);
                    o.text("basis", &m.basis);
                }
            }

            // Whole dollars per fiscal year and nothing else. There is deliberately no share, no
            // per-pupil and no per-district count on these rows: the fund's population is about a
            // thousand districts against this feed's 609, so every ratio a consumer might want
            // from here has a denominator that is not in the feed, and a row that offered one
            // would be inviting the join that makes it wrong.
            {
                let mut list = doc.block_arr("casino", "    ");
                for c in &self.casino {
                    let mut o = list.obj();
                    o.count("fiscal_year", c.fiscal_year);
                    o.num("amount", c.amount);
                }
            }

            // `source` on every row rather than only where it changes, for the same reason
            // `meal_program` writes its basis on every row: a consumer reading one row must be
            // able to tell which document it came from without scanning back for the last row
            // that said.
            {
                let mut list = doc.block_arr("appropriations", "    ");
                for a in &self.appropriations {
                    let mut o = list.obj();
                    o.count("fiscal_year", a.fiscal_year);
                    o.num("enacted", a.enacted);
                    o.num("foundation_funding", a.foundation_funding);
                    o.count("items", a.items);
                    o.text("source", &a.source);
                }
            }

            // `established_by` is written even when empty, and `general_assembly` as `null`
            // rather than omitted, so a consumer can tell "this line names no founding act" from
            // "this feed predates the field".
            {
                let mut list = doc.block_arr("appropriation_lines", "    ");
                for l in &self.appropriation_lines {
                    let mut o = list.obj();
                    o.text("fund", &l.fund);
                    o.text("ali", &l.ali);
                    o.text("name", &l.name);
                    o.text("established_by", &l.established_by);
                    o.opt_count("general_assembly", l.general_assembly);
                    o.opt_count("convened", l.convened);
                    o.flag("discontinued", l.discontinued);
                }
            }

            // The two chambers. Written before the district array because a reader scanning the
            // feed meets the derived aggregate before the exact per-district figures it came
            // from, and the block's own `basis` field is where the estimate is labelled as one.
            for (key, seats) in [
                ("house_districts", &self.house_districts),
                ("senate_districts", &self.senate_districts),
            ] {
                let mut list = doc.block_arr(key, "    ");
                for h in seats.iter() {
                    let mut o = list.obj();
                    o.text("number", &h.number);
                    for (key, value) in [
                        ("adm", h.adm),
                        ("realized_aid", h.realized_aid),
                        ("base_cost_state_share", h.base_cost_state_share),
                        ("categorical_funding", h.categorical_funding),
                        ("guarantee", h.guarantee),
                    ] {
                        o.num(key, value);
                    }
                    o.count("districts_on_guarantee", h.districts_on_guarantee);
                    o.count(
                        "districts_at_minimum_state_share",
                        h.districts_at_minimum_state_share,
                    );
                    o.count("districts_wholly_inside", h.districts_wholly_inside);
                    let mut members = o.arr("members");
                    for m in &h.members {
                        let mut e = members.obj();
                        e.text("irn", &m.irn);
                        e.text("name", &m.name);
                        e.share("share", m.share);
                        e.share("share_of_house_district", m.share_of_house_district);
                        e.num("adm", m.adm);
                        e.num("realized_aid", m.realized_aid);
                        e.flag("wholly_inside", m.wholly_inside);
                    }
                }
            }

            let mut list = doc.block_arr("districts", "    ");
            for d in &self.districts {
                let mut o = list.obj();
                o.text("irn", &d.irn);
                o.text("name", &d.name);
                o.text("county", &d.county);
                fields(
                    &mut o,
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
                );
                match &d.national {
                    None => o.raw("national", "null"),
                    Some(n) => fields(
                        &mut o,
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
                    ),
                }
                fields(
                    &mut o,
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
                );
                array_pair(
                    &mut o,
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
                );
                fields(
                    &mut o,
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
                );
                {
                    let mut seats = o.arr("house_districts");
                    for h in &d.house_districts {
                        let mut e = seats.obj();
                        e.text("number", &h.number);
                        e.share("share", h.share);
                    }
                }
                o.num("adm", d.adm);
                o.num("current_year_adm", d.current_year_adm);
                o.num("base_cost_per_pupil", d.base_cost_per_pupil);
                o.num("aggregate_base_cost", d.aggregate_base_cost);
                {
                    // Two tax years of the Department of Taxation's own table, written as an
                    // array so the change between them — which is the whole reason two are
                    // carried — is a thing the page can iterate rather than two parallel field
                    // sets.
                    let mut list = o.arr("property_tax");
                    for y in &d.property_tax {
                        let mut e = list.obj();
                        e.count("tax_year", y.tax_year);
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
                            e.num(name, value);
                        }
                    }
                }
                match &d.spending_by_function {
                    None => o.raw("spending_by_function", "null"),
                    Some(f) => {
                        let mut e = o.obj("spending_by_function");
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
                            e.num(name, value);
                        }
                    }
                }
                {
                    // Twenty-two elements plus the two funded-position counts and the
                    // reconciliation against the department's own figure. Written longhand for
                    // the same reason the rest of this serializer is: no serde, so no derive.
                    let b = &d.base_cost_build_up;
                    let mut e = o.obj("base_cost_build_up");
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
                        ("funded_classroom_teachers", b.funded_classroom_teachers),
                        ("funded_special_teachers", b.funded_special_teachers),
                    ] {
                        e.num(name, value);
                    }
                }
                o.num("base_cost_state_share", d.base_cost_state_share);
                o.num("categorical_funding", d.categorical_funding);
                o.num(
                    "base_cost_denominated_categoricals",
                    d.base_cost_denominated_categoricals,
                );
                o.num("dpia_funding", d.dpia_funding);
                o.num("general_funding_base", d.general_funding_base);
                o.num("dpia_funding_base", d.dpia_funding_base);
                o.num("guarantee_floor", d.guarantee_floor);
                // Special education, then the other five, then the six totals. Emitted once —
                // this block and the categoricals beside it were pasted twice, so every district
                // in the shipped feed carried both keys twice. JSON takes the last, so nothing
                // was wrong on the page and about 120KB of the payload was a copy of itself.
                array_pair(
                    &mut o,
                    "special_education",
                    "adm",
                    &d.special_education.adm,
                    "aid",
                    &d.special_education.aid,
                    &[],
                );
                array_pair(
                    &mut o,
                    "career_technical",
                    "fte",
                    &d.career_technical.fte,
                    "aid",
                    &d.career_technical.aid,
                    &[(
                        "associated_services",
                        d.career_technical.associated_services,
                    )],
                );
                array_pair(
                    &mut o,
                    "english_learners",
                    "adm",
                    &d.english_learners.adm,
                    "aid",
                    &d.english_learners.aid,
                    &[],
                );
                fields(
                    &mut o,
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
                );
                fields(
                    &mut o,
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
                );
                fields(
                    &mut o,
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
                );
                o.num("categorical_adm", d.categorical_adm);
                fields(
                    &mut o,
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
                );
                o.num("formula_aid_per_pupil", d.formula_aid_per_pupil);
                o.num("realized_aid_per_pupil", d.realized_aid_per_pupil);
                o.num("guarantee", d.guarantee);
                o.flag("on_guarantee", d.on_guarantee());
                o.flag("at_millage_floor", d.at_millage_floor());
                o.flag("near_millage_floor", d.near_millage_floor());
                o.flag("at_minimum_state_share", d.at_minimum_state_share);
                o.opt("valuation_per_pupil", d.valuation_per_pupil);
                o.opt("effective_class1_millage", d.effective_class1_millage);
                o.opt("voted_operating_millage", d.voted_operating_millage);
                match &d.millage {
                    None => o.raw("millage", "null"),
                    Some(m) => {
                        let mut e = o.obj("millage");
                        e.count("tax_year", m.tax_year);
                        for (key, value) in [
                            ("prior_rate", m.prior_rate),
                            ("observed_rate", m.observed_rate),
                            ("predicted_rate", m.predicted_rate),
                            ("residual", m.residual),
                            ("yield_per_mill_per_pupil", m.yield_per_mill_per_pupil),
                        ] {
                            e.num(key, value);
                        }
                        e.flag("at_floor", m.at_floor);
                        e.opt("cumulative_reduction", m.cumulative_reduction);
                    }
                }
                match &d.regime {
                    None => o.raw("regime", "null"),
                    Some(r) => {
                        let mut e = o.obj("regime");
                        e.num("charge_off_mills", r.charge_off_mills);
                        e.share("recognized_share", r.recognized_share);
                        e.count("reappraisal_year", r.reappraisal_year);
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
                            e.opt(key, value);
                        }
                        e.flag("exceeds_base_cost", r.exceeds_base_cost);
                    }
                }
                o.opt(
                    "operating_expenditure_per_pupil",
                    d.operating_expenditure_per_pupil,
                );
                o.opt("economically_disadvantaged", d.economically_disadvantaged);
                o.opt("enrollment_change", d.enrollment_change);
                {
                    let mut history = o.arr("adm_history");
                    for value in d.adm_history {
                        history.num(value);
                    }
                }
                match &d.outcome {
                    None => o.raw("outcome", "null"),
                    Some(out) => {
                        let mut e = o.obj("outcome");
                        for (key, value) in [
                            ("performance_index", out.performance_index),
                            ("performance_index_prior", out.performance_index_prior),
                            ("performance_index_earliest", out.performance_index_earliest),
                            ("progress_effect_size", out.progress_effect_size),
                            (
                                "progress_effect_size_one_year",
                                out.progress_effect_size_one_year,
                            ),
                            ("per_enrolled_pupil", out.per_enrolled_pupil),
                            ("per_equivalent_pupil", out.per_equivalent_pupil),
                            (
                                "per_equivalent_pupil_federal",
                                out.per_equivalent_pupil_federal,
                            ),
                            (
                                "per_equivalent_pupil_state_local",
                                out.per_equivalent_pupil_state_local,
                            ),
                            ("economically_disadvantaged", out.economically_disadvantaged),
                            ("english_learner", out.english_learner),
                            ("students_with_disabilities", out.students_with_disabilities),
                        ] {
                            e.opt(key, value);
                        }
                    }
                }
                {
                    let mut finances = o.arr("finances");
                    for y in &d.finances {
                        finance_year(&mut finances, y);
                    }
                }
                {
                    let mut casino = o.arr("casino");
                    for y in &d.casino {
                        casino_year(&mut casino, y);
                    }
                }
                o.opt_count("casino_counties", d.casino_counties);
            }
        }
        s.push('\n');
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
