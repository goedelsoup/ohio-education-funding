//! Policy levers, and what pulling one does to a district.
//!
//! # Why these levers and not others
//!
//! Each one is a dial that a real Ohio budget bill has turned, and each acts on a quantity the
//! committed model actually carries. A lever that would need data the corpus does not have —
//! changing the pupil-teacher ratios, say, which needs the base cost rebuilt from grade bands —
//! belongs in [`foundation`], not here, and is reached through [`Policy::base_cost_scale`]
//! rather than pretended at.
//!
//! # The guarantee is the lever that matters most
//!
//! Just under half of Ohio's districts are funded by the temporary transitional aid guarantee
//! rather than by the formula, holding more than half the state's students. For them the
//! formula is not what determines their aid — the guarantee is, and its baseline is what they
//! received in **FY2020**, a Bridge formula year Ohio froze rather than computed. Any policy
//! change to the formula reaches them only to the extent it lifts them off it.
//!
//! That is why [`GuaranteeRule`] is not a boolean. Removing the guarantee, re-basing it, and
//! phasing it out are three different policies with very different distributions, and the
//! difference between them is most of the argument.

use edfund_core::Dollars;

use crate::panel::{DistrictRecord, MINIMUM_STATE_SHARE};

/// What happens to the temporary transitional aid guarantee.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuaranteeRule {
    /// Current law: hold every district at its FY2020 receipt.
    AsEnacted,
    /// Hold at a multiple of the FY2020 baseline.
    ///
    /// `0.9` cuts every guaranteed district's floor by a tenth. Above `1.0` this is a raise,
    /// which is not a policy anyone has proposed but is the natural other direction.
    Rebased {
        /// Multiplier on the FY2020 baseline.
        factor: f64,
    },
    /// Close the gap between formula and baseline by a fraction, keeping the rest.
    ///
    /// `remaining = 0.0` is identical to [`GuaranteeRule::Removed`] and `1.0` to
    /// [`GuaranteeRule::AsEnacted`]; the interesting values are in between, because that is
    /// what a multi-year phase-out looks like in any single year of it.
    PhasedOut {
        /// Share of the guarantee still paid.
        remaining: f64,
    },
    /// Fund every district on the formula alone.
    Removed,
}

impl GuaranteeRule {
    /// Read a rule from the string form the CLI and the draft fixture both use.
    ///
    /// `as-enacted`, `removed`, `rebase:<factor>`, `phase-out:<remaining>`. It lives here rather
    /// than in either caller because two of them exist now — the `--guarantee` flag and a
    /// [`mod@crate::drafts`] provision — and a rule with two hand-written parsers is a rule with two
    /// vocabularies the day one of them gains a variant.
    ///
    /// # Errors
    ///
    /// If the rule is not one of the four, or if one that takes an argument is given something
    /// that is not a number.
    pub fn parse(raw: &str) -> Result<Self, String> {
        let (kind, argument) = raw.split_once(':').unwrap_or((raw, ""));
        let fraction = || {
            argument
                .parse::<f64>()
                .map_err(|_| format!("{kind} wants a number after the colon, got {argument:?}"))
        };
        match kind {
            "as-enacted" => Ok(Self::AsEnacted),
            "removed" => Ok(Self::Removed),
            "rebase" => Ok(Self::Rebased {
                factor: fraction()?,
            }),
            "phase-out" => Ok(Self::PhasedOut {
                remaining: fraction()?,
            }),
            other => Err(format!(
                "unknown guarantee rule {other:?}; try as-enacted, removed, rebase:0.9, phase-out:0.5"
            )),
        }
    }
}

/// A set of levers, together with what they are relative to.
///
/// The identity is [`Policy::current_law`]: applied to the panel it reproduces the department's
/// own model, which is what makes every delta meaningful. A test asserts that.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Policy {
    /// What happens to the guarantee.
    pub guarantee: GuaranteeRule,
    /// Multiplier on aggregate base cost.
    ///
    /// This is how an input-year refresh is expressed. Refreshing the cost inputs from FY2018
    /// to FY2022 prices raised statewide base cost by about 3.1%; that is `1.031` here rather
    /// than a re-derivation, because the department's own funded position counts are better
    /// than any reconstruction of them.
    ///
    /// # It moves the categoricals priced in base cost, and this is a choice
    ///
    /// Special education, English learners and career-technical are each `weight x $8,241.61 x
    /// count x state share`, where the multiplicand is the statewide average base cost per pupil.
    /// This lever moves them too — $812m of exposure, about 22% on top of what base cost alone
    /// does. See [`DistrictRecord::base_cost_denominated_categoricals`].
    ///
    /// The department's own simulator does **not** do this. Its sheet says so: "the statewide
    /// average values … are NOT recalculated based on the data changes specific to your district."
    /// That is the right behaviour for a tool where a user changes one district and everything
    /// else stays put. It is the wrong behaviour here, where a policy lever moves all 609 at once
    /// and the statewide average is precisely what would move first.
    ///
    /// So this site diverges from the department's tool deliberately, and says so on the page.
    /// Recorded at `.yidam/decisions/scenario-models-ohio.yml`.
    pub base_cost_scale: f64,
    /// The minimum state share of base cost.
    ///
    /// Set by each biennial budget rather than fixed in permanent law. The Fair School Funding
    /// Plan was enacted with 5%; the department's FY2027 model states **10%**, and 138 of 609
    /// districts sit exactly on it.
    pub minimum_state_share: f64,
    /// Fraction of computed base cost aid actually appropriated.
    pub phase_in_base_cost: f64,
    /// Fraction of computed categorical aid actually appropriated.
    ///
    /// Separate from base cost on purpose. In FY2022 the headline phase-in was 16.67% and
    /// Disadvantaged Pupil Impact Aid was phased in at **0%** — so a district's realized
    /// phase-in depended on its funding mix, and the districts DPIA exists to serve got less
    /// than the headline. One dial cannot express that; two can.
    pub phase_in_categorical: f64,
}

impl Policy {
    /// The identity. Reproduces the department's model exactly.
    #[must_use]
    pub const fn current_law() -> Self {
        Self {
            guarantee: GuaranteeRule::AsEnacted,
            base_cost_scale: 1.0,
            minimum_state_share: MINIMUM_STATE_SHARE,
            phase_in_base_cost: 1.0,
            phase_in_categorical: 1.0,
        }
    }

    /// Whether this policy changes anything.
    #[must_use]
    pub fn is_current_law(&self) -> bool {
        *self == Self::current_law()
    }
}

/// What a policy does to one district.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name.
    pub name: String,
    /// Current-year enrolled ADM used for the computation.
    pub adm: f64,
    /// Formula aid under the policy, before the guarantee.
    pub formula_aid: Dollars,
    /// Aid the district would actually receive.
    pub realized_aid: Dollars,
    /// The guarantee top-up, which is zero when the formula is what pays.
    pub guarantee: Dollars,
    /// Realized aid under current law, for comparison.
    pub baseline_realized_aid: Dollars,
    /// Whether the guarantee is what determines this district's aid.
    pub on_guarantee: bool,
    /// Whether the minimum state share is what determines its base cost aid.
    pub at_minimum_state_share: bool,
}

impl Outcome {
    /// Change against current law, in dollars.
    #[must_use]
    pub fn delta(&self) -> Dollars {
        self.realized_aid - self.baseline_realized_aid
    }

    /// Change against current law, per pupil.
    #[must_use]
    pub fn delta_per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.delta() / self.adm
    }

    /// Realized aid per pupil.
    #[must_use]
    pub fn realized_per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.realized_aid / self.adm
    }
}

/// Apply a policy to one district at a given current-year enrolled ADM.
///
/// `current_year_adm` is passed separately from the record so a projected enrollment can be
/// substituted for the modelled one; that substitution is the whole of what makes a run a
/// forecast rather than a simulation, and keeping it at the call site keeps it visible.
///
/// # How base cost responds, and where that stops being knowable
///
/// The state's share of base cost is `(base cost per pupil − local capacity per pupil) ×
/// current-year enrolled ADM`. Local capacity does not move when base cost does, so **a dollar
/// of extra base cost per pupil is a dollar of extra state share per pupil** for a district
/// above the floor — not a proportional share of it. Scaling the state share proportionally is
/// the obvious-looking mistake and understates the cost of a base cost increase by the local
/// share, which statewide is most of it.
///
/// For a district *at* the minimum state share the pass-through cannot be computed, because the
/// floor censors the quantity it would be computed from: all that is known is that local
/// capacity exceeds `1 − minimum` of base cost, not by how much. Such a district is held at the
/// floor here, which is right if its capacity is well above the threshold and understates its
/// gain if it is only just above. 138 districts are in that condition, and distinguishing them
/// needs the `tax-abstract` connector rather than a better guess.
#[must_use]
pub fn apply(record: &DistrictRecord, policy: &Policy, current_year_adm: f64) -> Outcome {
    let base_cost_per_pupil = record.base_cost_per_pupil * policy.base_cost_scale;
    let floor_per_pupil = base_cost_per_pupil * policy.minimum_state_share;

    let modelled_share_per_pupil = if record.current_year_adm > 0.0 {
        record.base_cost_state_share / record.current_year_adm
    } else {
        0.0
    };
    let increase_per_pupil = base_cost_per_pupil - record.base_cost_per_pupil;

    let censored = record.at_minimum_state_share();
    let residual_per_pupil = modelled_share_per_pupil + increase_per_pupil;
    let at_minimum = censored || residual_per_pupil < floor_per_pupil;

    // Computed in aggregate rather than by multiplying a per-pupil figure back up, so that
    // current law reproduces the department's dollars exactly. The fixture carries state share
    // to the cent; dividing by ADM and multiplying back loses a few of them, and an identity
    // that is only approximately the identity makes every delta below it approximate too.
    // Categoricals are per-pupil quantities computed on their own counts, which the fixture
    // does not carry separately. Scaling them by the change in enrolled ADM is the defensible
    // approximation: it is exact at modelled enrollment, which is where every deterministic
    // result in this crate is computed.
    let adm_ratio = if record.current_year_adm > 0.0 {
        current_year_adm / record.current_year_adm
    } else {
        1.0
    };

    let base_cost_aid = if censored {
        // Already on the floor as published, so its aid *is* the floor: scale the modelled
        // value by the ratio of the new minimum to the modelled one, by the base cost scale,
        // and by ADM. Deriving it from `base_cost_per_pupil` instead would lose the cents the
        // fixture rounds, and current law would stop being exactly the identity.
        //
        // The condition is `censored`, not `at_minimum`. Using the latter — which also covers a
        // district pulled *below* a raised floor — sends a district whose published share is not
        // the floor through a formula that assumes it is, and multiplies its share by the ratio
        // of the minimums instead of setting it to the floor. That was the bug, and the Rust
        // tests here did not catch it: they asserted nobody loses aid and every gainer sits
        // below the new floor, and the wrong branch satisfies both.
        record.base_cost_state_share
            * adm_ratio
            * policy.base_cost_scale
            * (policy.minimum_state_share / MINIMUM_STATE_SHARE)
    } else if residual_per_pupil < floor_per_pupil {
        // Above the published floor but below a raised one: the floor is what it gets.
        floor_per_pupil * current_year_adm
    } else {
        record.base_cost_state_share * adm_ratio + increase_per_pupil * current_year_adm
    } * policy.phase_in_base_cost;

    // Categoricals, with the base-cost-denominated ones moved by the same factor as base cost.
    //
    // Special education, English learners and career-technical are each `weight x $8,241.61 x
    // count x state share`, and that multiplicand is the statewide average base cost per pupil.
    // Scaling base cost without scaling them describes a world where the department raised the
    // average and then declined to use it in three formulas that are written in terms of it.
    //
    // This is a deliberate divergence from the department's own simulator, which holds every
    // statewide constant fixed because it is built for changing one district at a time. See
    // `.yidam/decisions/scenario-models-ohio.yml`. The lever's meaning is now "the statewide
    // average base cost per pupil moved by this factor", and everything priced in it follows.
    let denominated = record.base_cost_denominated_categoricals();
    let categoricals =
        (record.categorical_funding() - denominated) + denominated * policy.base_cost_scale;

    let formula_aid = base_cost_aid + categoricals * policy.phase_in_categorical * adm_ratio;

    let baseline = record.guarantee_baseline().unwrap_or(0.0);
    let held_at = match policy.guarantee {
        GuaranteeRule::AsEnacted => baseline,
        GuaranteeRule::Rebased { factor } => baseline * factor,
        GuaranteeRule::PhasedOut { remaining } => {
            formula_aid + remaining * (baseline - formula_aid).max(0.0)
        }
        GuaranteeRule::Removed => 0.0,
    };
    let realized_aid = formula_aid.max(held_at);

    Outcome {
        irn: record.irn.clone(),
        name: record.name.clone(),
        adm: current_year_adm,
        formula_aid,
        realized_aid,
        guarantee: (realized_aid - formula_aid).max(0.0),
        baseline_realized_aid: record.realized_aid(),
        on_guarantee: realized_aid > formula_aid + 0.005,
        at_minimum_state_share: at_minimum,
    }
}

/// Apply a policy across the panel at modelled enrollment.
#[must_use]
pub fn apply_all(panel: &[DistrictRecord], policy: &Policy) -> Vec<Outcome> {
    panel
        .iter()
        .map(|record| apply(record, policy, record.current_year_adm))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::panel;

    fn sample() -> DistrictRecord {
        panel()
            .into_iter()
            .find(|r| r.on_guarantee() && r.valuation_per_pupil.is_some())
            .expect("a guaranteed district")
    }

    fn on_formula() -> DistrictRecord {
        panel()
            .into_iter()
            .find(|r| !r.on_guarantee() && !r.at_minimum_state_share())
            .expect("a formula district above the floor")
    }

    #[test]
    fn current_law_reproduces_the_departments_model() {
        // The identity that makes every delta below meaningful.
        for record in panel() {
            let outcome = apply(&record, &Policy::current_law(), record.current_year_adm);
            assert!(
                (outcome.realized_aid - record.realized_aid()).abs() < 0.02,
                "{}: {} vs {}",
                record.name,
                outcome.realized_aid,
                record.realized_aid()
            );
            assert_eq!(
                outcome.on_guarantee,
                record.on_guarantee(),
                "{}",
                record.name
            );
        }
    }

    #[test]
    fn removing_the_guarantee_takes_a_guaranteed_district_down_to_formula() {
        let record = sample();
        let outcome = apply(
            &record,
            &Policy {
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
            record.current_year_adm,
        );
        assert_eq!(outcome.guarantee, 0.0);
        assert!(outcome.delta() < 0.0);
        assert!((outcome.realized_aid - record.core_foundation_funding).abs() < 0.02);
    }

    #[test]
    fn a_phase_out_at_zero_is_removal_and_at_one_is_current_law() {
        let record = sample();
        let at = |remaining| {
            apply(
                &record,
                &Policy {
                    guarantee: GuaranteeRule::PhasedOut { remaining },
                    ..Policy::current_law()
                },
                record.current_year_adm,
            )
            .realized_aid
        };
        let removed = apply(
            &record,
            &Policy {
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
            record.current_year_adm,
        )
        .realized_aid;
        assert!((at(0.0) - removed).abs() < 0.02);
        assert!((at(1.0) - record.realized_aid()).abs() < 0.02);
        // And halfway is halfway.
        assert!((at(0.5) - (removed + record.realized_aid()) / 2.0).abs() < 0.02);
    }

    #[test]
    fn the_guarantee_does_not_touch_a_district_already_on_formula() {
        let record = on_formula();
        for rule in [
            GuaranteeRule::Removed,
            GuaranteeRule::Rebased { factor: 0.5 },
            GuaranteeRule::PhasedOut { remaining: 0.25 },
        ] {
            let outcome = apply(
                &record,
                &Policy {
                    guarantee: rule,
                    ..Policy::current_law()
                },
                record.current_year_adm,
            );
            assert!(
                (outcome.delta()).abs() < 0.02,
                "{rule:?} moved a formula district"
            );
        }
    }

    #[test]
    fn a_base_cost_increase_passes_through_dollar_for_dollar_per_pupil_above_the_floor() {
        // Per pupil, not proportionally: the state pays the residual, and the residual absorbs
        // the whole per-pupil increase because local capacity does not move.
        //
        // Per *current-year* pupil, though, not per base-cost pupil. A shrinking district is
        // funded on the three-year average for base cost and on this year's count for the
        // state share, so it does not collect the whole aggregate increase — the two ADM
        // figures in the formula are different numbers and this is where that shows.
        let record = on_formula();
        let scale = 1.10;
        let outcome = apply(
            &record,
            &Policy {
                base_cost_scale: scale,
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
            record.current_year_adm,
        );
        let gained = outcome.formula_aid - record.core_foundation_funding;
        let per_pupil_increase = record.base_cost_per_pupil * (scale - 1.0);
        // The lever now moves two things, and the test separates them rather than summing to a
        // single number that could be right for the wrong reasons. Base cost aid still passes
        // through dollar for dollar per pupil; on top of that, the three categoricals priced in
        // the statewide average base cost move by the same factor.
        let from_base_cost = per_pupil_increase * record.current_year_adm;
        let from_categoricals = record.base_cost_denominated_categoricals() * (scale - 1.0);
        assert!(from_categoricals > 0.0, "the fixture district has none");
        assert!(
            (gained - from_base_cost - from_categoricals).abs() < 0.02,
            "{} got {gained:.2}, expected {:.2} + {from_categoricals:.2}",
            record.name,
            from_base_cost
        );

        // And the base cost part is more than a proportional share of the increase, which is the
        // mistake this guards against. The gap is the whole local share: a district getting 20% of
        // its base cost from the state collects five times what proportional scaling would give
        // it, and one already getting 70% collects only about half as much again.
        //
        // Against `from_base_cost`, not `gained`. The identity is a fact about the residual the
        // state pays, and the categorical term is not part of that residual — it is a separate
        // program that happens to be priced in the same number. Testing the sum against it passed
        // only while the categoricals did not move.
        let proportional = record.base_cost_state_share * (scale - 1.0);
        assert!(
            from_base_cost > proportional,
            "{from_base_cost:.0} vs {proportional:.0}"
        );
        assert!(
            (from_base_cost / proportional - 1.0 / record.state_share_fraction()).abs() < 0.05,
            "the ratio should be the reciprocal of the state share fraction"
        );

        // The categorical term scales with the base cost factor and with nothing else — in
        // particular it is not touched by the state share, because the published amounts already
        // have the district's share inside them.
        assert!(
            (from_categoricals / record.base_cost_denominated_categoricals() - (scale - 1.0)).abs()
                < 1e-9
        );
    }

    #[test]
    fn a_district_at_the_floor_gets_only_its_share_of_a_base_cost_increase() {
        let record = panel()
            .into_iter()
            .find(|r| r.at_minimum_state_share() && !r.on_guarantee())
            .expect("a floor district on formula");
        let scale = 1.10;
        let outcome = apply(
            &record,
            &Policy {
                base_cost_scale: scale,
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
            record.current_year_adm,
        );
        let gain = outcome.formula_aid - record.core_foundation_funding;
        let full = record.aggregate_base_cost * (scale - 1.0);
        assert!(gain > 0.0 && gain < full * 0.2, "{gain} of {full}");
        assert!(outcome.at_minimum_state_share);
    }

    #[test]
    fn a_district_pulled_below_a_raised_floor_receives_exactly_the_floor() {
        // The test that was missing. A district above the published minimum but below a raised
        // one must get `minimum × base cost per pupil × current-year ADM` — not its published
        // share scaled by the ratio of the minimums, which is only right for a district that
        // was already on the floor. The web layer's independent implementation found this; the
        // Rust tests around it did not, because the wrong answer is still an increase and still
        // confined to districts below the new floor.
        let raised = 0.15;
        let record = panel()
            .into_iter()
            .find(|r| !r.at_minimum_state_share() && r.state_share_fraction() < raised - 0.001)
            .expect("a district between the published floor and a 15% one");

        let outcome = apply(
            &record,
            &Policy {
                minimum_state_share: raised,
                ..Policy::current_law()
            },
            record.current_year_adm,
        );
        let expected = raised * record.base_cost_per_pupil * record.current_year_adm
            + record.categorical_funding();
        // Asserted on formula aid rather than realized: every such district happens to be on
        // the guarantee, so its receipt does not move even though its computed formula does.
        assert!(
            (outcome.formula_aid - expected).abs() < 0.02,
            "{}: {:.2} vs {:.2}",
            record.name,
            outcome.formula_aid,
            expected
        );
        assert!(outcome.at_minimum_state_share);
    }

    #[test]
    fn raising_the_minimum_state_share_reaches_the_districts_below_the_new_floor() {
        // 15%, half again above the 10% current law sets. It lifts the districts already on the
        // floor *and* pulls in those between the old floor and the new one — which is what a
        // floor does, and is why "the districts at the minimum" is the wrong set to reason
        // about when the minimum is what is being changed.
        let raised = 0.15;
        let policy = Policy {
            minimum_state_share: raised,
            ..Policy::current_law()
        };
        let panel = panel();
        let mut newly_floored = 0;
        for (record, outcome) in panel.iter().zip(apply_all(&panel, &policy)) {
            assert!(outcome.delta() >= -0.02, "{} lost aid", record.name);
            let below = record.state_share_fraction() < raised - 0.0005;
            if outcome.delta() > 0.02 {
                assert!(below, "{} gained and is above the new floor", record.name);
                if !record.at_minimum_state_share() {
                    newly_floored += 1;
                }
            }
        }
        assert!(
            newly_floored > 0,
            "raising a floor must catch districts that were above the old one"
        );
    }

    #[test]
    fn the_two_phase_in_dials_move_different_money() {
        let record = on_formula();
        let base_only = apply(
            &record,
            &Policy {
                phase_in_base_cost: 0.5,
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
            record.current_year_adm,
        );
        let categorical_only = apply(
            &record,
            &Policy {
                phase_in_categorical: 0.5,
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
            record.current_year_adm,
        );
        assert!(
            (base_only.formula_aid - categorical_only.formula_aid).abs() > 1.0,
            "halving base cost and halving categoricals cannot cost the same"
        );
        // Together they halve everything.
        let both = apply(
            &record,
            &Policy {
                phase_in_base_cost: 0.5,
                phase_in_categorical: 0.5,
                guarantee: GuaranteeRule::Removed,
                ..Policy::current_law()
            },
            record.current_year_adm,
        );
        assert!((both.formula_aid - record.core_foundation_funding / 2.0).abs() < 0.02);
    }

    #[test]
    fn falling_enrollment_reduces_formula_aid_but_not_below_the_guarantee() {
        let record = sample();
        let outcome = apply(
            &record,
            &Policy::current_law(),
            record.current_year_adm * 0.8,
        );
        assert!(outcome.formula_aid < record.core_foundation_funding);
        assert!(
            (outcome.realized_aid - record.realized_aid()).abs() < 0.02,
            "the guarantee is exactly what stops enrollment loss reaching a guaranteed district"
        );
    }

    #[test]
    fn current_law_is_recognisable_as_the_identity() {
        assert!(Policy::current_law().is_current_law());
        assert!(!Policy {
            base_cost_scale: 1.01,
            ..Policy::current_law()
        }
        .is_current_law());
    }
}
