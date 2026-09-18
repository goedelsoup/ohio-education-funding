//! Which business property a district's valuation actually sits in, kept apart rather than summed.
//!
//! # The question, and the reason it has to be asked three times
//!
//! "Districts with large industrial tax bases" is one of the oldest characterisations in Ohio
//! school finance argument, and this repository has already been wrong about it once.
//! `education-agency/toledo-city` recorded the FY2016 tail as "mostly small districts with large
//! industrial tax bases"; [`crate::fy2016`] measured that against Table SD-1 and got `r = +0.12`,
//! the wrong sign, and the characterisation was withdrawn.
//!
//! What neither the claim nor its refutation did was *separate the bases*. Table SD-1 carries
//! industrial, mineral and public utility value in three columns, and a measure that adds them —
//! which is what [`crate::fy2016::District::business_share`] does, deliberately and under its own
//! caveat — is dominated by whichever is largest. In Ohio that is public utility, by an order of
//! magnitude, and it is not the column anybody arguing about industry means.
//!
//! # They are three different sets of districts
//!
//! At [`TAX_YEAR`], taking the sixty districts most concentrated in each base:
//!
//! | | industrial | mineral | public utility |
//! |---|--:|--:|--:|
//! | **industrial** | — | 2 | 3 |
//! | **mineral** | 2 | — | 23 |
//! | **public utility** | 3 | 23 | — |
//!
//! One district is in all three. So a finding about "business property" is a finding about a set
//! that barely exists, and three findings about three populations have been averaged into it.
//! See [`overlap`].
//!
//! The levels are as far apart as the memberships. Industrial value is a median **2.2%** of a
//! district's total and clears [`CONCENTRATED`] in **27** districts; mineral is a median of
//! *exactly zero* and clears it in **10**; public utility is a median 6.5% and clears it in
//! **196**. "A district with a large industrial tax base" names about one Ohio district in
//! twenty-three, and "a district with a large mineral tax base" about one in sixty.
//!
//! # And only one of the three behaves the way the argument says
//!
//! Against FY2027 guarantee status, where the statewide rate is 48.3%:
//!
//! | base, above 10% of total value | districts | on the guarantee | median 2-year ADM | median value/pupil |
//! |---|--:|--:|--:|--:|
//! | industrial | 27 | **44.4%** | −2.61% | $279,788 |
//! | mineral | 10 | **90.0%** | −4.44% | $423,625 |
//! | public utility | 196 | 46.9% | −3.29% | $246,965 |
//! | *every district* | 609 | 48.3% | −3.13% | $248,097 |
//!
//! The counts are of districts the department's model funds. The abstract carries 611 and one
//! public-utility district is outside the model — the island pair `project::baseline` names.
//!
//! **Industrial districts are not distinctive on any of the three.** Their guarantee rate is
//! *below* the statewide one and their enrollment is falling *more slowly* than the median
//! district's. Whatever holds half of Ohio above the formula, an industrial tax base is not it.
//!
//! **Mineral districts are decisive and there are ten of them.** Nine of the ten are on the
//! guarantee — all five above 15% are — they are losing pupils about **1.4 times** as fast as the
//! median district and they are worth **1.7 times** the statewide valuation per pupil. That is the profile the
//! industrial claim describes, attached to the Utica shale rather than to a factory.
//!
//! So the characterisation is not false so much as misattributed, and the misattribution is not
//! recoverable from a summed business share: adding the three columns puts 196 public-utility
//! districts and 27 industrial ones around the 10 the result is about. See [`concentrated_in`].
//!
//! # Two cautions
//!
//! **A share is not a level.** A district can be concentrated in industrial property because it
//! has a factory or because it has nothing else; Lockland Local at 26.3% industrial is funded at a
//! 69.9% state share and Cuyahoga Heights Local at 21.4% sits on the 10% minimum. Concentration
//! and wealth are separate axes and this module carries only the first — join
//! [`crate::sd1::TaxRow::value_per_pupil`] or the panel for the second.
//!
//! **The tax year is named.** `metric/assessed-valuation-per-pupil` published a table that was
//! silently two tax years, and the abstract now carries four. Every figure above is
//! [`TAX_YEAR`], stated in a constant for the reason [`crate::valuation::TAX_YEAR`] is.

use std::collections::BTreeMap;

use crate::sd1;

/// The tax year every share here is computed at.
///
/// The newest the abstract carries. Unlike [`crate::valuation::TAX_YEAR`], which is pinned to the
/// one year an identity against the profile report holds, nothing here joins to a published
/// per-pupil figure — so the newest composition is the right one, and the shares are stable
/// across the four years the abstract holds in any case.
pub const TAX_YEAR: u16 = 2024;

/// The share of total value above which a district is called concentrated in a base.
///
/// Ten percent, which is a round number and is chosen rather than derived. It is stated once
/// here because three tables in this module cut on it and a threshold written three times is a
/// threshold that will be changed twice.
pub const CONCENTRATED: f64 = 0.10;

/// Which of Table SD-1's business classes a measure is about.
///
/// An enum rather than three functions because the whole finding of this module is that the
/// three are not interchangeable, and a caller that has to name one cannot accidentally sum them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base {
    /// Factories and plant. The class the argument names and the one that does least work.
    Industrial,
    /// Oil and gas. Ten districts, and the class that behaves the way the argument describes.
    Mineral,
    /// Power stations, pipelines and transmission. The largest of the three by a wide margin.
    PublicUtility,
}

impl Base {
    /// The class's name, for a table that has to print it.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Industrial => "industrial",
            Self::Mineral => "mineral",
            Self::PublicUtility => "public utility",
        }
    }

    /// Every class, in the order the tables above print them.
    pub const ALL: [Self; 3] = [Self::Industrial, Self::Mineral, Self::PublicUtility];
}

/// One district's business-property composition at [`TAX_YEAR`].
///
/// Every field is a share of [`TaxBase::total_value`], so the three can be compared across
/// districts of any size — and cannot be added to anything measured in dollars.
#[derive(Debug, Clone, PartialEq)]
pub struct TaxBase {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's published name.
    pub name: String,
    /// Industrial value over total value.
    pub industrial: f64,
    /// Mineral value over total value.
    pub mineral: f64,
    /// Public utility value over total value.
    pub public_utility: f64,
    /// Commercial value over total value, carried because it is the rest of Class II and a
    /// reader checking that the classes account for the whole will want it.
    pub commercial: f64,
    /// Total taxable value, in dollars. The denominator, kept so a share can be undone.
    pub total_value: f64,
}

impl TaxBase {
    /// The share in one named class.
    #[must_use]
    pub const fn share(&self, base: Base) -> f64 {
        match base {
            Base::Industrial => self.industrial,
            Base::Mineral => self.mineral,
            Base::PublicUtility => self.public_utility,
        }
    }

    /// Industrial, mineral and public utility together.
    ///
    /// The measure [`crate::fy2016`] uses, provided here so that the comparison this module is
    /// about can be made rather than only described — and named `summed` rather than `business`
    /// so that reaching for it is a decision.
    #[must_use]
    pub fn summed(&self) -> f64 {
        self.industrial + self.mineral + self.public_utility
    }

    /// Whether this district clears [`CONCENTRATED`] in a named class.
    #[must_use]
    pub fn concentrated(&self, base: Base) -> bool {
        self.share(base) > CONCENTRATED
    }
}

/// Every district's composition at [`TAX_YEAR`], in IRN order.
///
/// Districts whose total value is absent or zero are dropped rather than carried with an
/// undefined share; the abstract has none at [`TAX_YEAR`] and the guard is against a later
/// vintage that does.
#[must_use]
pub fn frame() -> Vec<TaxBase> {
    let mut out: Vec<TaxBase> = sd1::rows()
        .into_iter()
        .filter(|row| row.tax_year == TAX_YEAR)
        .filter_map(|row| {
            let total = row.total_value.filter(|v| *v > 0.0)?;
            let share = |value: Option<f64>| value.unwrap_or(0.0) / total;
            Some(TaxBase {
                irn: row.irn,
                name: row.name,
                industrial: share(row.industrial_value),
                mineral: share(row.mineral_value),
                public_utility: share(row.public_utility_value),
                commercial: share(row.commercial_value),
                total_value: total,
            })
        })
        .collect();
    out.sort_by(|a, b| a.irn.cmp(&b.irn));
    out
}

/// The same, keyed by IRN for a join.
#[must_use]
pub fn by_irn() -> BTreeMap<String, TaxBase> {
    frame()
        .into_iter()
        .map(|row| (row.irn.clone(), row))
        .collect()
}

/// The districts concentrated in one class, most concentrated first.
#[must_use]
pub fn concentrated_in(base: Base) -> Vec<TaxBase> {
    let mut out: Vec<TaxBase> = frame()
        .into_iter()
        .filter(|row| row.concentrated(base))
        .collect();
    out.sort_by(|a, b| b.share(base).total_cmp(&a.share(base)));
    out
}

/// How many districts are in the top `count` of both classes.
///
/// The measure behind the overlap table in the module docs. `count` is a parameter rather than a
/// constant because the answer should not depend on it and the test checks that it does not.
#[must_use]
pub fn overlap(a: Base, b: Base, count: usize) -> usize {
    let top = |base: Base| {
        let mut rows = frame();
        rows.sort_by(|x, y| y.share(base).total_cmp(&x.share(base)));
        rows.into_iter()
            .take(count)
            .map(|row| row.irn)
            .collect::<std::collections::BTreeSet<_>>()
    };
    top(a).intersection(&top(b)).count()
}

/// The median share in one class across every district.
///
/// Zero for [`Base::Mineral`], which is the fact that makes a summed business share misleading:
/// more than half of Ohio's districts have no oil and gas value at all, so the column contributes
/// nothing to most of the sum and everything to the ten districts the sum is supposed to find.
#[must_use]
pub fn median_share(base: Base) -> f64 {
    let mut shares: Vec<f64> = frame().iter().map(|row| row.share(base)).collect();
    shares.sort_by(f64::total_cmp);
    if shares.is_empty() {
        return 0.0;
    }
    shares[shares.len() / 2]
}
