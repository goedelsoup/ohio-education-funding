//! The charge-off local share, and the statutory millage that drove it.
//!
//! For roughly three decades Ohio decided how much of a district's cost the district itself
//! should bear by multiplying a **statutory millage, uniform statewide**, by the district's
//! valuation, and subtracting the product from the computed cost. The state paid the remainder.
//!
//! # The rate series, which the corpus carried as `[open]` for its whole life
//!
//! The parameter node recorded "not yet populated" and warned that the values had to come from
//! statute rather than recollection. They are now sourced, and the progression is not the flat
//! rate the summary suggested — see [`RATES`].
//!
//! # The base changed while the rate stood still
//!
//! [`DeRolph I`](https://www.supremecourt.ohio.gov/rod/docs/pdf/0/1997/1997-ohio-84.pdf) at ¶97
//! describes the charge-off as *total taxable value* times a percentage. By FY2008 the
//! Legislative Service Commission describes the same 23 mills applied to **recognized
//! valuation**.
//!
//! So a district's charge-off could fall with no change to the rate, and two accurate statements
//! of "the charge-off was 23 mills" can describe materially different local shares. That
//! distinction is why [`ValuationBase`] is carried on every rate rather than left to prose.
//!
//! **What recognized valuation is was recorded wrongly here for fourteen phases**, and the
//! correction is in [`recognized_valuation`](crate::recognized_valuation). It is not an H.B. 920
//! adjustment and has nothing to do with which districts' reduction factors have bitten; it is a
//! three-year phase-in of the valuation growth a reappraisal or update produces. The corpus is now
//! able to compute it — 8.2% of statewide taxable value in TY2024, $793m of charge-off at 23 mills
//! — and [`crate::ChargeOffBase`] is how a caller says which base it wants.
//!
//! # Why the mechanism was thought to be wrong
//!
//! The charge-off assumes a district can levy at the statutory rate. Under H.B. 920 a district's
//! effective operating millage falls as valuation rises, so a district whose effective rate had
//! dropped below the charge-off was charged for revenue it could not collect — phantom revenue.
//! Ohio's own answer was a patch rather than a fix: the **charge-off supplement**, known as gap
//! aid, which reimbursed the difference and cost about $73.5 million across 145 districts in
//! FY2008. A mechanism needing a supplement for a quarter of the state is the argument for
//! replacing it, and local capacity is what replaced it.

use edfund_core::Dollars;

/// What the millage is applied to. Not interchangeable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValuationBase {
    /// Total taxable value of real and tangible personal property, per R.C. 3317.02(D).
    TotalTaxable,
    /// Valuation with a reappraisal's inflationary increase phased in over three years.
    ///
    /// Lower than total taxable value for a district whose county has revalued in the last two
    /// years, and equal to it otherwise. See [`recognized_valuation`](crate::recognized_valuation)
    /// — including for what this doc comment used to say, which was a different mechanism.
    Recognized,
}

/// Which districts a rate applies to.
///
/// Every rate before FY2010 applied to all of them, which is what "uniform statewide" meant and
/// what made the parameter a parameter. The Evidence-Based Model is the exception this exists
/// for: it charged one rate against two bases and chose between them on the district's own
/// effective millage, so a series that carried only a rate and a base could not hold it.
///
/// The same argument as [`ValuationBase`]: a distinction that changes a district's local share
/// belongs in the type rather than in the `effective` string, where nothing can read it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Applicability {
    /// Every city, local and exempted village district.
    EveryDistrict,
    /// Districts at or very near the twenty-mill floor — a class 1 effective current expense rate
    /// at or below [`FLOOR_TEST_MILLS`].
    AtTheTwentyMillFloor,
    /// Every district above that threshold.
    AboveTheTwentyMillFloor,
}

/// The effective class 1 current expense rate the Evidence-Based Model split its bases at.
///
/// LSC calls 20.1 mills "very near or at the 20-mill floor", which is a tenth of a mill of slack
/// above [`parameter/twenty-mill-floor`](../../.yidam/corpus/parameter/twenty-mill-floor.yml) — a
/// tolerance for the publication rounding that node measures, written into the formula itself.
pub const FLOOR_TEST_MILLS: f64 = 20.1;

/// Which document a rate and its base were read out of.
///
/// Carried as a field rather than inferred from [`ChargeOffRate::authority`], because tests need
/// to separate the rates that can be checked against the committed DeRolph extract from the one
/// that cannot, and a substring search for `"LSC"` over a prose citation is not that test. It
/// silently reclassifies a rate when the string is reworded, and silently matches nothing when a
/// rate from a third source is added — of which there will be several, since the series is
/// deliberately incomplete.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourcedTo {
    /// DeRolph I ¶97, which recites the progression with its session-law citations.
    ///
    /// Every rate marked this way is quoted verbatim in
    /// [the committed extract](../../regime-diff/fixtures/derolph-opinions.txt).
    DeRolphI,
    /// The Legislative Service Commission's *School Funding Complete Resource* (November 2008).
    Lsc,
    /// The Legislative Service Commission's education greenbook for one enacted budget act.
    ///
    /// Twelve of them are committed, and they are the only source this corpus holds that states a
    /// rate and a base for the years after FY2009: Ohio Laws' version archive for R.C. 3317.022
    /// begins on 1 July 2014, after the mechanism was retired. Read by
    /// [`project::greenbook`](../../project/src/greenbook.rs).
    Greenbook,
}

/// One statutory charge-off rate, with the authority that set it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChargeOffRate {
    /// The rate in mills. 23 mills is 2.3% of valuation.
    pub mills: f64,
    /// When it was operative, as the source describes it.
    pub effective: &'static str,
    /// The statute or session law, quoted closely enough to find.
    pub authority: &'static str,
    /// What the rate multiplies.
    pub base: ValuationBase,
    /// Which districts it multiplies it for.
    pub applies_to: Applicability,
    /// The document this entry was transcribed from.
    pub sourced_to: SourcedTo,
}

impl ChargeOffRate {
    /// The rate as a fraction of valuation.
    #[must_use]
    pub fn fraction(&self) -> f64 {
        self.mills / 1_000.0
    }
}

/// The charge-off rate series, oldest first.
///
/// Sourced from DeRolph I ¶97, which recites the progression with its session-law citations, and
/// from the Legislative Service Commission's *School Funding Complete Resource* for the rate and
/// base still operative in FY2008. Both are catalogued.
///
/// The series **ends at FY2011, and that is the whole series**. It was carried as incomplete on
/// the assumption that the Evidence-Based Model and Bridge formula rates existed and had not been
/// found. The Evidence-Based Model's is here, and it is the only entry that needs
/// [`Applicability`]. The Bridge formula's is absent because there is none: LSC states that the
/// state share index "does not result in a uniform charge-off rate", and that an FY2014 local
/// share expressed as one would vary from 11.3 to 22.9 mills across districts, averaging 20.6.
/// The "20 mills" of secondary reporting is that average, and entering it here would put a
/// distribution's mean in a table of legislated rates.
pub const RATES: &[ChargeOffRate] = &[
    ChargeOffRate {
        mills: 20.0,
        effective: "in force when DeRolph was filed in 1991",
        authority: "144 Ohio Laws, Part III, 3987, 4122",
        base: ValuationBase::TotalTaxable,
        applies_to: Applicability::EveryDistrict,
        sourced_to: SourcedTo::DeRolphI,
    },
    ChargeOffRate {
        mills: 20.5,
        effective: "raised during the pendency of DeRolph",
        authority: "Am.Sub.H.B. No. 152, Section 36.12, 145 Ohio Laws, Part III, 4432-4433",
        base: ValuationBase::TotalTaxable,
        applies_to: Applicability::EveryDistrict,
        sourced_to: SourcedTo::DeRolphI,
    },
    ChargeOffRate {
        mills: 23.0,
        effective: "in force at DeRolph I, decided March 1997",
        authority: "R.C. 3317.022, as recited at DeRolph I ¶97",
        base: ValuationBase::TotalTaxable,
        applies_to: Applicability::EveryDistrict,
        sourced_to: SourcedTo::DeRolphI,
    },
    ChargeOffRate {
        mills: 23.0,
        effective: "FY2008, the last year of the uniform mechanism the corpus has a base for",
        authority: "LSC, School Funding Complete Resource (November 2008), Charge-off Rate",
        base: ValuationBase::Recognized,
        applies_to: Applicability::EveryDistrict,
        sourced_to: SourcedTo::Lsc,
    },
    ChargeOffRate {
        mills: 22.0,
        effective: "FY2010-11, for districts at or very near the twenty-mill floor",
        authority: "LSC, H.B. 1 greenbook, 128th General Assembly, Local Share of the Adequacy \
                    Amount",
        base: ValuationBase::TotalTaxable,
        applies_to: Applicability::AtTheTwentyMillFloor,
        sourced_to: SourcedTo::Greenbook,
    },
    ChargeOffRate {
        mills: 22.0,
        effective: "FY2010-11, for every other district",
        authority: "LSC, H.B. 1 greenbook, 128th General Assembly, Local Share of the Adequacy \
                    Amount",
        base: ValuationBase::Recognized,
        applies_to: Applicability::AboveTheTwentyMillFloor,
        sourced_to: SourcedTo::Greenbook,
    },
];

/// The terminal rate of the mechanism, and the one a counterfactual should use.
///
/// 23 mills is what the charge-off had reached and held. Running an older rate against current
/// valuations would compare the Fair School Funding Plan against a policy Ohio had already left.
pub const TERMINAL_MILLS: f64 = 23.0;

/// Joint vocational school districts are charged off at a different rate entirely.
///
/// Carried because it is the one place the "uniform statewide" description of the mechanism is
/// false, and a diff run over JVSDs with the district rate would be wrong by a factor of 46.
///
/// **Present tense.** The Fair School Funding Plan replaced the charge-off with local capacity for
/// city, local and exempted village districts and not for these: LSC's H.B. 110 greenbook computes
/// a JVSD's local share as this half-mill against "the lesser of the district's three-year average
/// valuation or most recent valuation", and calls it "consistent with the prior formula for
/// JVSDs". The rate held and the base gained a floor — the same shape as everywhere else here.
pub const JVSD_MILLS: f64 = 0.5;

/// A district's deemed local contribution per pupil.
///
/// Per pupil on both sides, because that is the unit the FSFP local capacity measure is defined
/// in and the only unit in which the two mechanisms can be set beside one another.
#[must_use]
pub fn local_share_per_pupil(valuation_per_pupil: Dollars, mills: f64) -> Dollars {
    valuation_per_pupil * mills / 1_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Evidence-Based Model is one rate against two bases, and the series can say so.
    ///
    /// Every other entry is `EveryDistrict`, which is what made "uniform statewide" an accurate
    /// description for eighteen years and stopped being one in FY2010. The two rows here are the
    /// same rate and the same authority and differ only in which districts they reach and what
    /// they multiply — which is the whole content of the finding, and would have been invisible
    /// had it been written into `effective`.
    #[test]
    fn the_evidence_based_model_is_one_rate_against_two_bases() {
        let model: Vec<&ChargeOffRate> = RATES
            .iter()
            .filter(|r| r.sourced_to == SourcedTo::Greenbook)
            .collect();
        assert_eq!(model.len(), 2);
        assert!(model.iter().all(|r| r.mills == 22.0));
        assert_eq!(model[0].authority, model[1].authority);

        assert_eq!(model[0].applies_to, Applicability::AtTheTwentyMillFloor);
        assert_eq!(model[0].base, ValuationBase::TotalTaxable);
        assert_eq!(model[1].applies_to, Applicability::AboveTheTwentyMillFloor);
        assert_eq!(model[1].base, ValuationBase::Recognized);

        // Note the direction, which is the part that surprises: the district *at* the floor is
        // charged against the larger base. The model moved the districts least able to raise the
        // revenue being assumed onto the valuation measure that does not defer a reappraisal's
        // growth. `FLOOR_TEST_MILLS` carries the threshold and why it sits above twenty.
    }

    /// The series stops at FY2011 on purpose, and the reason is not a missing source.
    ///
    /// Nothing after the Evidence-Based Model belongs here. The Bridge formula has no uniform rate
    /// to enter and the Fair School Funding Plan charges city, local and exempted village
    /// districts nothing of the kind — [`JVSD_MILLS`] is the one part of the mechanism that
    /// survives, and it is a constant rather than a series because it has never moved.
    #[test]
    fn nothing_after_the_evidence_based_model_is_a_rate() {
        assert_eq!(RATES.len(), 6);
        assert_eq!(
            RATES.iter().filter(|r| r.applies_to == Applicability::EveryDistrict).count(),
            4,
            "a fifth universal rate appeared; the Bridge formula does not have one and the plan              replaced the mechanism"
        );
        assert!(RATES.iter().all(|r| r.mills >= 20.0));
        assert!(
            RATES.iter().all(|r| r.mills != JVSD_MILLS),
            "the joint vocational half-mill is not a member of this series: it applies to a              different class of district and is still in force"
        );
    }

    #[test]
    fn twenty_three_mills_is_two_point_three_percent() {
        let rate = RATES
            .iter()
            .find(|r| r.mills == TERMINAL_MILLS)
            .expect("the terminal rate is in the series");
        assert!((rate.fraction() - 0.023).abs() < 1e-12);
        assert!((local_share_per_pupil(200_000.0, TERMINAL_MILLS) - 4_600.0).abs() < 1e-9);
    }

    #[test]
    fn the_rate_rose_over_the_series_and_the_base_narrowed() {
        // Both movements matter and they point opposite ways: a higher rate charges a district
        // more, a narrower base charges it less. Reading either alone misstates the mechanism.
        let mills: Vec<f64> = RATES
            .iter()
            .filter(|r| r.applies_to == Applicability::EveryDistrict)
            .map(|r| r.mills)
            .collect();
        assert_eq!(mills, vec![20.0, 20.5, 23.0, 23.0]);
        assert!(
            mills.windows(2).all(|w| w[1] >= w[0]),
            "the rate never fell"
        );
        assert_eq!(RATES[0].base, ValuationBase::TotalTaxable);
        assert_eq!(
            RATES[RATES.len() - 1].base,
            ValuationBase::Recognized,
            "the last described base is the reappraisal-phased one"
        );
    }

    #[test]
    fn every_rate_carries_an_authority_that_can_be_looked_up() {
        for rate in RATES {
            assert!(
                rate.authority.contains("Ohio Laws")
                    || rate.authority.contains("R.C.")
                    || rate.authority.contains("LSC"),
                "{} has no citable authority",
                rate.mills
            );
        }
    }

    #[test]
    fn a_joint_vocational_district_is_charged_off_at_a_different_rate() {
        // The magnitude is the point: a diff run over a JVSD at the district rate overstates its
        // local share forty-sixfold, so the two rates are not interchangeable defaults.
        let ratio = TERMINAL_MILLS / JVSD_MILLS;
        assert!((ratio - 46.0).abs() < 1e-9, "{ratio}");
        assert!((local_share_per_pupil(200_000.0, JVSD_MILLS) - 100.0).abs() < 1e-9);
    }
}
