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
//! valuation** — an H.B. 920-adjusted figure that is deliberately lower than total taxable value
//! for districts whose reduction factors have bitten.
//!
//! So a district's charge-off could fall with no change to the rate, and two accurate statements
//! of "the charge-off was 23 mills" can describe materially different local shares. That
//! distinction is why [`ValuationBase`] is carried on every rate rather than left to prose.
//!
//! **This corpus holds total taxable valuation and not recognized valuation.** Every figure
//! computed here is therefore on the DeRolph-era base, and overstates the local share of any
//! district whose recognized valuation was reduced. Getting past it needs the `tax-abstract`
//! connector, which is declared and blocked.
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
    /// Valuation after H.B. 920 recognition adjustments — lower wherever reduction factors bind.
    Recognized,
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
/// The series is **not complete**: the Evidence-Based Model years and the Bridge formula decade
/// are not established here, and secondary reporting that gives 22 mills for FY2010-11 and 20
/// mills later is not sourced well enough to commit. See the parameter node.
pub const RATES: &[ChargeOffRate] = &[
    ChargeOffRate {
        mills: 20.0,
        effective: "in force when DeRolph was filed in 1991",
        authority: "144 Ohio Laws, Part III, 3987, 4122",
        base: ValuationBase::TotalTaxable,
    },
    ChargeOffRate {
        mills: 20.5,
        effective: "raised during the pendency of DeRolph",
        authority: "Am.Sub.H.B. No. 152, Section 36.12, 145 Ohio Laws, Part III, 4432-4433",
        base: ValuationBase::TotalTaxable,
    },
    ChargeOffRate {
        mills: 23.0,
        effective: "in force at DeRolph I, decided March 1997",
        authority: "R.C. 3317.022, as recited at DeRolph I ¶97",
        base: ValuationBase::TotalTaxable,
    },
    ChargeOffRate {
        mills: 23.0,
        effective: "FY2008, the last year the corpus has a described base for",
        authority: "LSC, School Funding Complete Resource (November 2008), Charge-off Rate",
        base: ValuationBase::Recognized,
    },
];

/// The terminal rate of the mechanism, and the one a counterfactual should use.
///
/// 23 mills is what the charge-off had reached and held. Running an older rate against current
/// valuations would compare the Fair School Funding Plan against a policy Ohio had already left.
pub const TERMINAL_MILLS: f64 = 23.0;

/// Joint vocational school districts were charged off at a different rate entirely.
///
/// Carried because it is the one place the "uniform statewide" description of the mechanism is
/// false, and a diff run over JVSDs with the district rate would be wrong by a factor of 46.
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
        let mills: Vec<f64> = RATES.iter().map(|r| r.mills).collect();
        assert_eq!(mills, vec![20.0, 20.5, 23.0, 23.0]);
        assert!(
            mills.windows(2).all(|w| w[1] >= w[0]),
            "the rate never fell"
        );
        assert_eq!(RATES[0].base, ValuationBase::TotalTaxable);
        assert_eq!(
            RATES[RATES.len() - 1].base,
            ValuationBase::Recognized,
            "the last described base is the H.B. 920-adjusted one"
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
