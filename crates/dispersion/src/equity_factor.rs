//! The dispersion statistic that is also a federal parameter.
//!
//! # A number this crate already computed, read as the statute reads it
//!
//! Every equity finding here quotes a coefficient of variation, and until now none of them could
//! say whether the number was large. `dispersion/operating-expenditure-coefficient-of-variation`
//! is 0.2016 for Ohio in FY2024, and 0.2016 against *what* had no answer: Ohio describing itself
//! cannot say whether Ohio is unusual, and no other state's distribution had been computed.
//!
//! It turns out the federal government computes exactly this statistic, for all fifty-one
//! jurisdictions, and pays states on it. Under 20 U.S.C. 6337 — the Education Finance Incentive
//! Grant, the fourth and least-discussed of Title I Part A's sub-formulas — a state's **equity
//! factor** is the enrolment-weighted coefficient of variation of per-pupil expenditure across its
//! local education agencies, counting only agencies enrolling more than 200 pupils. It enters the
//! allocation twice, in opposite directions:
//!
//! - the state's per-child rate is multiplied by **1.30 minus its equity factor** — so a
//!   wider-spread state is paid less per formula child; and
//! - the *within*-state weighting of poverty ladders steepens in three bands at 0.10 and 0.20 —
//!   so a wider-spread state's money is concentrated harder on its poorest counties. At the top
//!   bracket the weight is 4.0, 6.0 or 8.0 depending on the band.
//!
//! Congress reads dispersion as a fault to be priced and, having priced it, as a reason to
//! redistribute inside the state. The same statistic does both.
//!
//! # Where Ohio lands
//!
//! **Ninth of fifty-one**, at 0.1987, against a median state at 0.1587. Only Alaska,
//! Massachusetts, Vermont, Idaho, Louisiana, Montana, New York and South Dakota spread wider.
//!
//! And Ohio is the state closest to the 0.20 band edge — 0.0013 below it, nearer than every other
//! state bar Washington, which sits 0.0001 above the other edge at 0.10. The corpus's own separate
//! Ohio measure, on the department's operating expenditure for FY2024 rather than the Census
//! Bureau's current spending for FY2022, is 0.2016 — on the far side of the same line.
//!
//! # The band question, bracketed rather than asserted
//!
//! Two things this fixture cannot reproduce sit between [`by_state`] and the Secretary's own
//! computation. The statute counts a poverty child 1.4 times in the pupil count, and the poverty
//! count is the Census Bureau's small-area estimate, which no fixture here holds; and the
//! expenditure definition ED applies is its own rather than `TCURELSC`.
//!
//! The first can be bounded instead of guessed. The adjustment can only lower a state's factor
//! where its high-poverty agencies are its high-spending ones, and in Ohio they are — economically
//! disadvantaged share against operating expenditure per pupil runs r = +0.38. Applying the
//! statutory 1.4 to Ohio's *economically disadvantaged* count, which is roughly three times the
//! Census poverty count and so a deliberate over-application, gives 0.1684. The true value lies
//! between that and the unadjusted 0.1981, and **both ends are inside the middle band.**
//!
//! So the answer to whether the equity factor has been material for Ohio is: materially on the
//! rate, and not at all on the band. Ohio's spread costs it 3.5% of its per-formula-child rate
//! against a median state and 8.2% against a state at the 0.10 floor, every year, and it has never
//! been close enough to 0.20 for the ladder to steepen — closest in the country, and still on the
//! near side, with the one correction the fixture cannot apply pushing further away.
//!
//! # What is not established here
//!
//! The effort factor, the other multiplier, is per-pupil expenditure over per-capita income
//! relative to the national ratio. No fixture here carries state personal income, so it is not
//! computed. The statute bounds it to [0.95, 1.05], which is the useful thing to know about it:
//! end to end it can move a state's rate by a tenth, against a ladder that at 0.20 doubles the
//! top-bracket weight.

use crate::{national_peers, profile};

/// The fiscal year of the national district panel this is computed over.
///
/// [`national_peers::FISCAL_YEAR`] restated rather than re-derived, because a reader of an equity
/// factor needs the year and should not have to follow a module link for it.
pub const FISCAL_YEAR: u16 = national_peers::FISCAL_YEAR;

/// Agencies enrolling this many pupils or fewer are outside the computation.
///
/// 20 U.S.C. 6337(b)(1)(B)(iii): the Secretary "shall include only those local educational
/// agencies with an enrolment of more than 200 students". Strictly more, so the comparison is
/// `> MIN_ENROLMENT` and an agency at exactly 200 is out.
pub const MIN_ENROLMENT: f64 = 200.0;

/// The two equity-factor values at which the within-state poverty ladder changes.
///
/// A factor below 0.10 takes the flattest ladder, one from 0.10 up to but not including 0.20 the
/// middle, and 0.20 or above the steepest.
pub const BAND_EDGES: [f64; 2] = [0.10, 0.20];

/// The constant the equity factor is subtracted from to give the state's per-child rate.
///
/// 20 U.S.C. 6337(b)(1)(A)(iii): the amount is multiplied by "1.30 minus such State's equity
/// factor". A perfectly uniform state is paid at 1.30 and Alaska, at 0.4565, at 0.8435.
pub const STATE_RATE_CONSTANT: f64 = 1.30;

/// How many times a poverty child counts in the pupil figure the statute divides by.
///
/// 20 U.S.C. 6337(b)(1)(B)(iv). Not applied by [`by_state`], which has no poverty count; used by
/// [`ohio_bracket`] against a deliberately over-large stand-in. See the module note.
pub const POVERTY_WEIGHT: f64 = 1.4;

/// Which within-state poverty ladder a state's equity factor selects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Equity factor below 0.10 — the flattest ladder, topping out at 4.0.
    Flattest,
    /// From 0.10 up to but not including 0.20 — topping out at 6.0.
    Middle,
    /// 0.20 or above — topping out at 8.0.
    Steepest,
}

impl Band {
    /// The band a factor falls in.
    #[must_use]
    pub fn of(factor: f64) -> Self {
        if factor < BAND_EDGES[0] {
            Self::Flattest
        } else if factor < BAND_EDGES[1] {
            Self::Middle
        } else {
            Self::Steepest
        }
    }

    /// The five multipliers this band applies to a county's formula children, by the share of the
    /// county's 5-17 population they are: not more than 15.00%, then to 19.00%, 24.20%, 29.20%,
    /// and above.
    ///
    /// 20 U.S.C. 6337(b)(2). The first rung is 1.0 in every band; the ladders separate above it.
    #[must_use]
    pub fn weights(self) -> [f64; 5] {
        match self {
            Self::Flattest => [1.0, 1.75, 2.5, 3.25, 4.0],
            Self::Middle => [1.0, 1.5, 3.0, 4.5, 6.0],
            Self::Steepest => [1.0, 2.0, 4.0, 6.0, 8.0],
        }
    }
}

/// One state's equity factor and what it is computed over.
#[derive(Debug, Clone, PartialEq)]
pub struct StateEquity {
    /// Two-letter postal code.
    pub state: String,
    /// Agencies the computation includes — comparable, reporting spending, over the floor.
    pub agencies: usize,
    /// Pupils across them, which is also the weight.
    pub pupils: f64,
    /// Enrolment-weighted mean expenditure per pupil.
    pub mean_per_pupil: f64,
    /// The equity factor: the weighted coefficient of variation of per-pupil expenditure.
    pub factor: f64,
}

impl StateEquity {
    /// The ladder this state's factor selects.
    #[must_use]
    pub fn band(&self) -> Band {
        Band::of(self.factor)
    }

    /// `1.30 - factor`, the multiplier on the state's per-formula-child amount.
    #[must_use]
    pub fn state_rate(&self) -> f64 {
        STATE_RATE_CONSTANT - self.factor
    }

    /// How far the factor sits from the nearer of the two band edges.
    ///
    /// The quantity that says whether a state's band is a settled fact or a coin toss against
    /// measurement error, which for Ohio is the whole question.
    #[must_use]
    pub fn distance_to_edge(&self) -> f64 {
        BAND_EDGES
            .iter()
            .map(|edge| (self.factor - edge).abs())
            .fold(f64::INFINITY, f64::min)
    }
}

/// The enrolment-weighted coefficient of variation of a set of `(pupils, per_pupil)` pairs.
///
/// Weighted rather than plain, because the statute weights "according to the number of pupils
/// served by the local educational agency". For Ohio the two differ by a fifth — 0.1987 weighted
/// against 0.1779 plain — and in the direction that matters: Ohio's large districts are its
/// extreme ones, so the statute's own weighting is the one that puts Ohio near the edge.
fn weighted_cv(observations: &[(f64, f64)]) -> Option<f64> {
    let pupils: f64 = observations.iter().map(|(w, _)| w).sum();
    if pupils <= 0.0 {
        return None;
    }
    let mean: f64 = observations.iter().map(|(w, x)| w * x).sum::<f64>() / pupils;
    if mean <= 0.0 {
        return None;
    }
    let variance: f64 = observations
        .iter()
        .map(|(w, x)| w * (x - mean) * (x - mean))
        .sum::<f64>()
        / pupils;
    Some(variance.max(0.0).sqrt() / mean)
}

/// Every state's equity factor, widest spread first.
///
/// Built over [`national_peers::national_panel`]'s comparable agencies — the survey's own
/// `AGCHRT != 1` and `SCHLEV == 03` filter — which is not optional here and not a matter of taste.
/// The fixture carries the comparable set for every state **plus all 357 of Ohio's community
/// schools, joint vocational districts and service centres**, which no other state's row set
/// includes. Computed over everything the file holds, Ohio reads 0.2267 and second in the nation;
/// that number is the fixture's Ohio-only extra rows and not a fact about America.
#[must_use]
pub fn by_state() -> Vec<StateEquity> {
    let mut grouped: std::collections::BTreeMap<String, Vec<(f64, f64)>> =
        std::collections::BTreeMap::new();
    for district in national_peers::national_panel() {
        if !district.comparable || district.enrollment <= MIN_ENROLMENT {
            continue;
        }
        let Some(per_pupil) = district.spending_per_pupil().filter(|s| *s > 0.0) else {
            continue;
        };
        grouped
            .entry(district.state.clone())
            .or_default()
            .push((district.enrollment, per_pupil));
    }

    let mut out: Vec<StateEquity> = grouped
        .into_iter()
        .filter_map(|(state, observations)| {
            let pupils: f64 = observations.iter().map(|(w, _)| w).sum();
            let mean = observations.iter().map(|(w, x)| w * x).sum::<f64>() / pupils;
            Some(StateEquity {
                state,
                agencies: observations.len(),
                pupils,
                mean_per_pupil: mean,
                factor: weighted_cv(&observations)?,
            })
        })
        .collect();
    out.sort_by(|a, b| b.factor.total_cmp(&a.factor).then(a.state.cmp(&b.state)));
    out
}

/// Ohio's row of [`by_state`].
///
/// # Panics
///
/// If the national fixture stops carrying Ohio, which would break far more than this.
#[must_use]
pub fn ohio() -> StateEquity {
    by_state()
        .into_iter()
        .find(|s| s.state == "OH")
        .expect("the national panel carries Ohio")
}

/// A state's position in the ranking, widest spread first and one-based.
///
/// Returns `None` for a code the panel does not carry.
#[must_use]
pub fn rank(state: &str) -> Option<usize> {
    by_state()
        .iter()
        .position(|s| s.state == state)
        .map(|index| index + 1)
}

/// The median state's equity factor — the benchmark Ohio's number never had.
///
/// # Panics
///
/// If the panel carries no states at all.
#[must_use]
pub fn median_factor() -> f64 {
    let mut factors: Vec<f64> = by_state().iter().map(|s| s.factor).collect();
    factors.sort_by(f64::total_cmp);
    assert!(!factors.is_empty(), "the national panel carries no states");
    factors[factors.len() / 2]
}

/// Ohio's equity factor with and without the statutory poverty adjustment, over one set of
/// districts.
///
/// Returns `(adjusted, unadjusted)` — the lower and upper ends of the bracket, in that order.
///
/// The adjustment counts a poverty child [`POVERTY_WEIGHT`] times in the pupil figure. The
/// statute's poverty count is the Census Bureau's small-area estimate and no fixture here holds
/// it, so this substitutes the department's **economically disadvantaged** share, which for Ohio
/// runs about three times the Census poverty rate. That is the point: it over-applies the
/// adjustment on purpose, so the pair brackets rather than estimates. The true factor is above
/// the first and at or below the second.
///
/// Both ends are computed over the districts that join [`profile::districts`] by IRN, so the
/// unadjusted end here is not exactly [`ohio`]'s — a handful of agencies in the national panel
/// have no profile row.
///
/// # Panics
///
/// If no Ohio district in the national panel joins the profile report, which would mean the IRN
/// join has broken.
#[must_use]
pub fn ohio_bracket() -> (f64, f64) {
    let disadvantaged: std::collections::BTreeMap<String, f64> = profile::districts()
        .into_iter()
        .filter_map(|d| d.economically_disadvantaged.map(|share| (d.irn, share)))
        .collect();

    let mut adjusted = Vec::new();
    let mut plain = Vec::new();
    for district in national_peers::national_panel() {
        if district.state != "OH" || !district.comparable || district.enrollment <= MIN_ENROLMENT {
            continue;
        }
        let Some(spending) = district.current_spending.filter(|s| *s > 0.0) else {
            continue;
        };
        let Some(share) = disadvantaged.get(&district.irn) else {
            continue;
        };
        let counted = district.enrollment * (1.0 + (POVERTY_WEIGHT - 1.0) * share);
        adjusted.push((counted, spending / counted));
        plain.push((district.enrollment, spending / district.enrollment));
    }

    assert!(
        !plain.is_empty(),
        "no Ohio district joins the profile report"
    );
    (
        weighted_cv(&adjusted).expect("the adjusted set has pupils and spending"),
        weighted_cv(&plain).expect("the plain set has pupils and spending"),
    )
}
