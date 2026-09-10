//! Where an Ohio accountability rating determines a funding amount, and when it has.
//!
//! # The question six budget acts each carried separately
//!
//! Six `legislation` nodes record an `accountability_effect` of "Not assessed against
//! `accountability-regime` here", and two of them say what the assessment would be for. H.B. 119
//! notes that Closing the Achievement Gap "pays on the academic watch and academic emergency
//! building counts, which makes it one of the earliest points where Ohio's accountability ratings
//! determine a funding amount." H.B. 64 notes that its bonuses "pay on report-card measures and so
//! join the funding and accountability systems at a point neither `accountability-regime` nor the
//! formula nodes currently record."
//!
//! Asked once instead of six times, that is a single question with a chronology for an answer, and
//! the twelve LSC greenbooks in [`crate::greenbook`] cover every biennium from FY2002 to FY2024.
//!
//! # A rating has determined a formula payment in exactly two episodes
//!
//! Searched across all twelve for the terms that only occur where a rating drives a formula
//! component — `academic distress index`, `graduation bonus`, `third grade reading bonus`,
//! `performance bonus` — the hits fall in **four greenbooks and no others**: H.B. 119 for
//! FY2008-09, H.B. 64 and H.B. 49 for FY2016-19, and H.B. 166, which mentions them only to say
//! what its freeze did to them. Eight of the twelve carry none. [`quiet`] is that list, and it is
//! as much the finding as [`COUPLINGS`] is.
//!
//! # And the direction reversed
//!
//! **Closing the Achievement Gap paid on failure.** Its subsidy was
//! `0.0015 × formula amount × poverty index × academic distress index × formula ADM`, where the
//! academic distress index is a district's share of buildings in academic watch or emergency over
//! the state's, and a district qualified only with *both* that index and its poverty index at 1.0
//! or above. Thirty-one districts qualified. More failing buildings meant more money, and only for
//! poor districts.
//!
//! It also paid on improvement, in its second year and in the other direction: a district that
//! qualified in FY2008 and lowered its distress percentage received its FY2008 subsidy times
//! 1.035, and one that did not improve received the same amount flat.
//!
//! **The bonuses paid on success.** `Graduation rate × 0.075 × formula amount × graduate count ×
//! state share index`, and the same shape for third-grade reading proficiency. Better results
//! meant more money — fifty times the rate of the earlier programme — and the state share index
//! made the payment larger in poorer districts.
//!
//! **And the freeze severed the link for districts while leaving it for charters.** H.B. 166 held
//! every traditional district and JVSD at its FY2019 foundation aid, so a district's bonus stopped
//! moving with its report card — but it required each community and STEM school's graduation and
//! third grade reading bonuses to be "recalculated each year using a formula amount of $6,020".
//! For two years the only Ohio schools whose funding still moved with their own results were
//! community and STEM schools.
//!
//! That version had never carried the equaliser. LSC describes it as "identical to the calculation
//! of the payment for traditional districts except that it does not use the state share index", so
//! the un-equalised form is the one that survived the freeze — and the one Ohio readopted.
//!
//! **The supplement now in force pays on success and equalises nothing.** H.B. 96's performance
//! supplement is $13 per pupil per qualifying rating, flat at any valuation. It is outside the
//! greenbook window and is recorded at `parameter/performance-supplement-rate`, which measures the
//! consequence: per pupil it runs 2.56 times higher in the least-poor fifth of districts than in
//! the poorest.
//!
//! So Ohio has coupled ratings to formula dollars three times in twenty years, and each time on a
//! different principle: to the worst-off and poorest, then to the best-performing with a wealth
//! equaliser, then to the best-performing without one.
//!
//! # Two other routes, which are not formula components
//!
//! A rating reaches a district's money two further ways, and both are older than either episode
//! above.
//!
//! **Earmarked appropriations to rated districts.** H.B. 95 set aside $3.7m in FY2004 and $5.9m in
//! FY2005 for academic emergency districts to buy intervention services, and funded professional
//! development for the Ohio Graduation Test at $4.6m a year in grants to districts in academic
//! emergency. Money to low-rated districts, outside the formula.
//!
//! **Scholarship eligibility.** From H.B. 66 in 2005 a rating made a district's students eligible
//! for a scholarship deducted from that district — originally buildings in academic emergency for
//! three or more consecutive years, and today the lowest fifth of buildings by performance index
//! score under R.C. 3310.03(A)(1). This is the only route where a rating *removes* money, it is
//! the largest of the five by dollars, and it is the one still operating. See
//! `program/edchoice-scholarship`.

use crate::greenbook;

/// What a coupling does with the rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Pays more where the rating is worse.
    OnFailure,
    /// Pays more where the rating is better.
    OnSuccess,
}

/// How the money reaches — or leaves — the district.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// A component of the foundation formula, computed for every district.
    Formula,
    /// An appropriation earmarked to districts carrying a rating.
    Earmark,
    /// A rating that makes a district's students eligible for a scholarship deducted from it.
    Deduction,
}

/// Whether the payment carries Ohio's wealth equaliser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Equalised {
    /// Multiplied by the state share index, or restricted to districts above a poverty threshold.
    Yes,
    /// Paid gross, the same per pupil at any valuation.
    No,
    /// Not a per-district payment, so the question does not arise.
    NotApplicable,
}

/// One point at which an accountability rating determines a funding amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coupling {
    /// What it is called.
    pub name: &'static str,
    /// The act that created it.
    pub act: &'static str,
    /// The greenbook that establishes it, or `""` where the act is outside the committed set.
    pub greenbook: &'static str,
    /// First and last fiscal year it operated, as this corpus can establish them. The last is the
    /// last year *evidenced here*, not a repeal date.
    pub years: (u16, u16),
    /// Which way the rating pushes the money.
    pub direction: Direction,
    /// How it reaches the district.
    pub route: Route,
    /// Whether it carries the wealth equaliser.
    pub equalised: Equalised,
    /// A phrase from the named greenbook that establishes it, checked by test.
    pub evidence: &'static str,
}

/// Every coupling this corpus can evidence, oldest first.
pub const COUPLINGS: &[Coupling] = &[
    Coupling {
        name: "Ohio Graduation Test professional development grants",
        act: "H.B. 95 of the 125th General Assembly",
        greenbook: "hb95",
        years: (2004, 2005),
        direction: Direction::OnFailure,
        route: Route::Earmark,
        equalised: Equalised::NotApplicable,
        evidence: "academic emergency for five days of embedded professional development",
    },
    Coupling {
        name: "Intervention services set-aside for academic emergency districts",
        act: "H.B. 95 of the 125th General Assembly",
        greenbook: "hb95",
        years: (2004, 2005),
        direction: Direction::OnFailure,
        route: Route::Earmark,
        equalised: Equalised::NotApplicable,
        evidence: "academic emergency districts to provide intervention services",
    },
    Coupling {
        name: "EdChoice scholarship eligibility",
        act: "H.B. 66 of the 126th General Assembly",
        greenbook: "hb66",
        // Still operating; the corpus evidences the origin here and the present rule at
        // `program/edchoice-scholarship`, so the second year is the greenbook's own biennium.
        years: (2006, 2007),
        direction: Direction::OnFailure,
        route: Route::Deduction,
        equalised: Equalised::NotApplicable,
        evidence: "in academic emergency for three or more consecutive years",
    },
    Coupling {
        name: "Closing the Achievement Gap",
        act: "H.B. 119 of the 127th General Assembly",
        greenbook: "hb119",
        years: (2008, 2009),
        direction: Direction::OnFailure,
        route: Route::Formula,
        equalised: Equalised::Yes,
        evidence: "0.0015 x formula amount x poverty index x academic distress index",
    },
    Coupling {
        name: "Graduation bonus",
        act: "H.B. 64 of the 131st General Assembly",
        greenbook: "hb64",
        // The district version stops at the freeze: H.B. 166 held every district at FY2019.
        years: (2016, 2019),
        direction: Direction::OnSuccess,
        route: Route::Formula,
        equalised: Equalised::Yes,
        evidence: "Graduation rate x 0.075 x Formula amount x Graduate count x State share index",
    },
    Coupling {
        name: "Third grade reading bonus",
        act: "H.B. 64 of the 131st General Assembly",
        greenbook: "hb64",
        years: (2016, 2019),
        direction: Direction::OnSuccess,
        route: Route::Formula,
        equalised: Equalised::Yes,
        evidence: "Third Grade Reading Bonus",
    },
    Coupling {
        name: "Graduation and third grade reading bonuses for community and STEM schools",
        act: "H.B. 64 of the 131st General Assembly",
        // Evidenced from H.B. 49, which is where LSC sets the charter calculation beside the
        // district one and says what the difference is.
        greenbook: "hb49",
        // Outlives the district version: H.B. 166 kept recalculating these through FY2021 while
        // holding districts at FY2019.
        years: (2016, 2021),
        direction: Direction::OnSuccess,
        route: Route::Formula,
        equalised: Equalised::No,
        evidence: "identical to the calculation of the payment for traditional districts except that it does not use the state share index",
    },
    Coupling {
        name: "Performance supplement",
        act: "H.B. 96 of the 136th General Assembly",
        // Outside the committed greenbooks, which stop at H.B. 33. Evidenced instead by
        // `parameter/performance-supplement-rate` and its own test.
        greenbook: "",
        years: (2026, 2027),
        direction: Direction::OnSuccess,
        route: Route::Formula,
        equalised: Equalised::No,
        evidence: "",
    },
];

/// The terms that occur in a greenbook only where a rating drives a formula component.
pub const FORMULA_TERMS: [&str; 4] = [
    "academic distress index",
    "graduation bonus",
    "third grade reading bonus",
    "performance bonus",
];

/// The greenbooks carrying no rating-driven formula payment at all.
///
/// Nine of the twelve. The bienniums with nothing in them are the reason the two episodes read as
/// episodes rather than as a policy Ohio has continuously held.
///
/// Searched against [`greenbook::Greenbook::flat`] and not `mentions`. These are PDF extractions
/// and a two-word term straddles a line break often enough to matter — `mentions` reads the
/// publisher's own line endings, so it would silently under-report the noisy bienniums, which is
/// the direction that would invent this finding rather than find it.
#[must_use]
pub fn quiet() -> Vec<&'static str> {
    greenbook::greenbooks()
        .into_iter()
        .filter(|g| {
            let flat = g.flat().to_lowercase();
            !FORMULA_TERMS
                .iter()
                .any(|term| flat.contains(&term.to_lowercase()))
        })
        .map(|g| g.bill)
        .collect()
}

/// A quantity LSC states in prose, tied to the sentence that states it.
///
/// The greenbooks are prose and a figure taken from one cannot be recomputed from a column. What
/// *is* checkable is that the sentence carrying the number is still there verbatim, so this
/// returns `value` where the phrase is present and [`f64::NAN`] where it is not — never zero,
/// which a figure pinned at zero would silently accept.
#[must_use]
pub fn stated(bill: &str, phrase: &str, value: f64) -> f64 {
    if quotes(bill, phrase) {
        value
    } else {
        f64::NAN
    }
}

/// Whether a greenbook carries a phrase, with the publisher's line breaks collapsed first.
///
/// The check every quotation in this module's tests goes through. See [`quiet`] for why.
#[must_use]
pub fn quotes(bill: &str, phrase: &str) -> bool {
    greenbook::greenbook(bill)
        .flat()
        .to_lowercase()
        .contains(&phrase.to_lowercase())
}

/// The couplings that are formula components, oldest first.
#[must_use]
pub fn formula_couplings() -> Vec<&'static Coupling> {
    COUPLINGS
        .iter()
        .filter(|c| c.route == Route::Formula)
        .collect()
}
