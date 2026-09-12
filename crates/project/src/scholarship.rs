//! What Ohio's four scholarship programmes pay, as R.C. 3317.022 and R.C. 3310.08 compute it.
//!
//! Every one of these amounts sits in the committed statute extract and none had been read.
//! `program/edchoice-expansion` records `amount: Not yet populated` and "Exact bands not yet
//! established"; `program/jon-peterson-special-needs` holds the first of six category supplements
//! and describes the rest as "other amounts for other categories".
//!
//! # There are no bands
//!
//! The expansion's award is a **continuous function of family income**, not a schedule of bands.
//! R.C. 3310.08 gives it as a formula in prose, and read back it is one line: above 450% of the
//! federal poverty guidelines the award **halves for every further 100 percentage points**, and
//! stops at a tenth of the base. See [`expansion_award`], which is the arithmetic rather than a
//! transcription of a table that does not exist.
//!
//! # All four are ceilings, and they are ceilings on different things
//!
//! No programme pays its stated amount as such. Each pays the lesser of that amount and what the
//! school or provider charges — but "what it charges" is defined four different ways, and the
//! differences are not cosmetic: the pilot project nets out *all* financial aid and adjustments,
//! EdChoice nets out only tuition discounts, and the two special-needs programmes read a tuition
//! or a fee for the programme rather than for the school. A comparison across them is a
//! comparison of four denominators.
//!
//! # What is not here
//!
//! The dollar value of 450% of the federal poverty guidelines, because it depends on family size
//! and the statute points outward for it (R.C. 5101.46). So this module gives the award as a
//! function of the income *ratio*, which is the form the statute states and the only form that is
//! self-contained.

use edfund_core::Dollars;

/// R.C. 3310.08(A)(1), the "constant multiplier". The award's decay rate.
pub const CONSTANT_MULTIPLIER: f64 = 0.50;

/// R.C. 3310.08(B)(1): the income ratio at or below which the full base amount is paid.
///
/// 450% of the federal poverty guidelines, written here as a multiple because that is what the
/// statute's own exponent uses — a "federal poverty level multiplier" of 4.5 makes the two
/// branches of (B) agree at the boundary, and one of 450 would make the award vanish there.
pub const FULL_AWARD_CEILING: f64 = 4.5;

/// R.C. 3310.08(A)(6): the floor, as a share of the base amount.
pub const MINIMUM_SHARE: f64 = 0.10;

/// R.C. 3317.022(A)(10)(a)(ii)(I), grades kindergarten through eight, as enacted by H.B. 33.
///
/// Not a fixed figure: the section directs it to "increase in future fiscal years by the same
/// percentage that the statewide average base cost per pupil increases", so the voucher ceiling
/// tracks the Fair School Funding Plan's own base cost by statute. This is the enacted value and
/// a caller comparing it against a later year's payments must index it first.
pub const EDCHOICE_BASE_K8: Dollars = 5_500.0;

/// The same for grades nine through twelve, and the same indexing.
pub const EDCHOICE_BASE_9_12: Dollars = 7_500.0;

/// R.C. 3317.022(A)(12)(a)(ii): the autism scholarship ceiling.
///
/// **Carries no indexing clause**, unlike every EdChoice and Jon Peterson amount in the same
/// division, which `program/autism-scholarship` records.
pub const AUTISM_CEILING: Dollars = 34_000.0;

/// R.C. 3317.022(A)(13)(a)(ii): the Jon Peterson award before its category supplement.
///
/// Indexed to the statewide average base cost per pupil, like the EdChoice amounts.
pub const JON_PETERSON_BASE: Dollars = 7_190.0;

/// R.C. 3317.022(A)(13)(a)(iii): the Jon Peterson ceiling. The same figure as [`AUTISM_CEILING`],
/// and like it carrying no indexing clause.
pub const JON_PETERSON_CEILING: Dollars = 34_000.0;

/// R.C. 3317.022(A)(13)(a)(ii)(I) to (VI), by special education category one through six.
///
/// Indexed to the special education category amounts under division (A)(3) rather than to base
/// cost, so these move with the weights in
/// [`formula-component/fsfp-special-education-weights`](../../.yidam/corpus/formula-component/fsfp-special-education-weights.yml)
/// and the base above moves with base cost. Two indexing rules inside one award.
pub const JON_PETERSON_SUPPLEMENTS: [Dollars; 6] =
    [2_855.0, 5_879.0, 12_879.0, 16_890.0, 22_560.0, 31_932.0];

/// The EdChoice Expansion award for a student, before the tuition ceiling.
///
/// `income_over_poverty` is the family's income as a multiple of the federal poverty guidelines —
/// 4.5 for a family at 450%. Below that the base amount is paid in full; above it the award is
/// `base * 0.5^(income_over_poverty - 4.5)`, floored at a tenth of the base.
///
/// R.C. 3310.08(B)(2) writes that as `base X (1 / constant multiplier)^4.5 X e^power equation`
/// with the power equation `federal poverty level multiplier X ln(constant multiplier)`. The two
/// are the same expression; this computes the statute's form so that a reader checking it against
/// the section is checking the section and not a simplification.
#[must_use]
pub fn expansion_award(base: Dollars, income_over_poverty: f64) -> Dollars {
    if income_over_poverty <= FULL_AWARD_CEILING {
        return base;
    }
    let power_equation = income_over_poverty * CONSTANT_MULTIPLIER.ln();
    let calculated =
        base * (1.0 / CONSTANT_MULTIPLIER).powf(FULL_AWARD_CEILING) * power_equation.exp();
    calculated.max(base * MINIMUM_SHARE)
}

/// The income ratio at which the award reaches its floor and stops falling.
///
/// Solving `0.5^(p - 4.5) = 0.1` gives 7.8219…, which is **782.19% of the federal poverty
/// guidelines** — the point past which a family's income no longer changes the award. Derived
/// rather than stated: the statute gives the floor as an amount and never says where it binds.
#[must_use]
pub fn minimum_award_threshold() -> f64 {
    FULL_AWARD_CEILING + MINIMUM_SHARE.ln() / CONSTANT_MULTIPLIER.ln()
}

/// The Jon Peterson award for a student in special education category `category`, one-based,
/// before the fees charged by the provider.
///
/// The least of the provider's fees, the base plus the category supplement, and the ceiling —
/// this returns the last two of those three, which is the part the statute fixes.
///
/// # Panics
///
/// On a category outside one to six. R.C. 3317.013 defines exactly six.
#[must_use]
pub fn jon_peterson_award(category: usize) -> Dollars {
    assert!(
        (1..=6).contains(&category),
        "R.C. 3317.013 defines six special education categories, not {category}"
    );
    (JON_PETERSON_BASE + JON_PETERSON_SUPPLEMENTS[category - 1]).min(JON_PETERSON_CEILING)
}

/// The department's own account of what the channel paid, and what it cannot settle.
///
/// # Why this lives beside the statute
///
/// Everything above computes what R.C. 3310.08 and 3317.022 *direct*. This reads what the
/// department reports it *paid*, so the two can be held against each other — which is the only
/// way to ask how much of the award schedule actually binds.
pub mod report {
    use std::collections::BTreeMap;

    use edfund_core::Dollars;

    /// The committed extract of the 2025 Scholarship Annual Report, covering 2024-25.
    const FIXTURE: &str = include_str!("../fixtures/scholarship-programs.csv");

    const EXPECTED_HEADER: &str = "program,name,students,expenditure,published_average";

    /// One programme as the report gives it.
    #[derive(Debug, Clone, PartialEq)]
    pub struct Programme {
        /// The report's own name for it.
        pub name: String,
        /// Participation. Every programme publishes one.
        pub students: f64,
        /// Statewide expenditure. Jon Peterson's is derived rather than published, and is the one
        /// programme where this is `None` in the fixture.
        pub expenditure: Option<Dollars>,
        /// The average award the report prints, which for three of the four is **not**
        /// [`Self::implied_average`].
        pub published_average: Option<Dollars>,
    }

    impl Programme {
        /// Expenditure over participation — the average with a denominator this corpus can see.
        ///
        /// Sits 1.5% to 3.4% below the published figure for every programme but autism, which
        /// reconciles to the cent. The report does not explain the gap, so both are carried and
        /// every reading below is computed on each.
        #[must_use]
        pub fn implied_average(&self) -> Option<Dollars> {
            (self.students > 0.0).then(|| self.expenditure.map(|paid| paid / self.students))?
        }
    }

    /// Every programme the report covers, keyed by the fixture's own slug.
    ///
    /// # Panics
    ///
    /// If the fixture's header is not the one this was written against, by way of
    /// [`edfund_core::csv::rows`].
    #[must_use]
    pub fn programmes() -> BTreeMap<String, Programme> {
        edfund_core::csv::rows(FIXTURE, EXPECTED_HEADER)
            .map(|row| {
                (
                    row.str(0).to_string(),
                    Programme {
                        name: row.str(1).to_string(),
                        students: row.num(2).expect("every programme publishes participation"),
                        expenditure: row.num(3),
                        published_average: row.num(4),
                    },
                )
            })
            .collect()
    }
}

/// Which of the report's two averages a reading is taken on.
///
/// They differ by 1.5% to 3.4% and the report does not say why, so nothing here picks one. Every
/// finding is computed on both and is stated only where both agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Average {
    /// The figure the report prints.
    Published,
    /// Expenditure over participation.
    Implied,
}

impl Average {
    /// One programme's average on this basis.
    #[must_use]
    pub fn of(self, programme: &report::Programme) -> Option<Dollars> {
        match self {
            Self::Published => programme.published_average,
            Self::Implied => programme.implied_average(),
        }
    }
}

/// The largest share of pupils in grades K-8 consistent with an observed average award.
///
/// Both programmes pay the lesser of a tuition and a per-grade ceiling, and R.C. 3310.08(A)(2)
/// makes the expansion's own base amount *the same* ceiling. So an average award is bounded above
/// by the grade mix alone: `5,500w + 7,500(1 - w)`. Inverting it gives the most K-8-heavy mix that
/// could produce the observed figure, and any tuition that binds pushes the true share lower
/// still — so this is a ceiling on the K-8 share and not an estimate of it.
///
/// `None` where the average is outside the two ceilings, which for an average *below* the K-8
/// amount is the interesting case: see [`shortfall_no_mix_explains`].
#[must_use]
pub fn greatest_k8_share(average: Dollars) -> Option<f64> {
    ((EDCHOICE_BASE_K8..=EDCHOICE_BASE_9_12).contains(&average))
        .then(|| (EDCHOICE_BASE_9_12 - average) / (EDCHOICE_BASE_9_12 - EDCHOICE_BASE_K8))
}

/// How far an average award falls below the lowest ceiling any grade mix could produce.
///
/// The K-8 amount is the floor of the schedule: a programme paying every student the full ceiling
/// cannot average less than [`EDCHOICE_BASE_K8`] whatever its grade mix. An average below it is
/// therefore evidence that something *other than* the mix is reducing awards — the income decay,
/// a tuition below the ceiling, or both — and this is how much of it no mix can account for.
///
/// `None` where the average clears the floor, which is the ordinary case and not a finding.
#[must_use]
pub fn shortfall_no_mix_explains(average: Dollars) -> Option<Dollars> {
    (average < EDCHOICE_BASE_K8).then_some(EDCHOICE_BASE_K8 - average)
}

/// The mean decay factor implied if income decay alone explained a programme's shortfall against
/// a reference programme on the same grade mix.
///
/// **This is a reading and not a measurement**, and it is offered so that its size can be argued
/// with rather than guessed at. Three quantities move an average award — grade mix, the R.C.
/// 3310.08 decay, and a tuition below the ceiling — and two published averages cannot separate
/// three unknowns. Holding the first equal and the third absent attributes the whole gap to the
/// second, which is the largest the decay can possibly be.
#[must_use]
pub fn decay_explaining_the_whole_gap(expansion: Dollars, reference: Dollars) -> Option<f64> {
    (reference > 0.0 && expansion > 0.0).then_some(expansion / reference)
}

/// The income ratio at which R.C. 3310.08 pays `factor` of the base amount.
///
/// The inverse of the decay in [`expansion_award`], for a factor between [`MINIMUM_SHARE`] and 1.
/// Below the floor the award no longer varies with income, so no ratio is recoverable and this
/// returns `None` rather than a number past where the statute stops distinguishing families.
#[must_use]
pub fn income_paying(factor: f64) -> Option<f64> {
    ((MINIMUM_SHARE..=1.0).contains(&factor))
        .then(|| FULL_AWARD_CEILING + factor.ln() / CONSTANT_MULTIPLIER.ln())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two branches of R.C. 3310.08(B) meet, which is what fixes what the exponent means.
    #[test]
    fn the_award_is_continuous_at_four_hundred_and_fifty_per_cent() {
        let base = EDCHOICE_BASE_K8;
        assert!((expansion_award(base, FULL_AWARD_CEILING) - base).abs() < 1e-9);
        assert!((expansion_award(base, FULL_AWARD_CEILING + 1e-9) - base).abs() < 1e-4);
        assert_eq!(expansion_award(base, 1.0), base);
    }

    /// It halves for every further hundred percentage points, exactly.
    #[test]
    fn each_further_hundred_per_cent_of_poverty_halves_the_award() {
        let base = EDCHOICE_BASE_9_12;
        for step in 0..3 {
            let here = expansion_award(base, FULL_AWARD_CEILING + f64::from(step));
            let next = expansion_award(base, FULL_AWARD_CEILING + f64::from(step) + 1.0);
            assert!(
                (next / here - CONSTANT_MULTIPLIER).abs() < 1e-9,
                "at {step} steps the award falls by {:.6}, not by half",
                next / here
            );
        }
    }

    #[test]
    fn the_floor_is_a_tenth_of_the_base_and_binds_at_seven_hundred_and_eighty_two_per_cent() {
        let threshold = minimum_award_threshold();
        assert!((threshold - 7.821_928_094_887_362).abs() < 1e-12);
        for base in [EDCHOICE_BASE_K8, EDCHOICE_BASE_9_12] {
            let floor = base * MINIMUM_SHARE;
            assert!((expansion_award(base, threshold) - floor).abs() < 1e-9);
            assert!((expansion_award(base, 50.0) - floor).abs() < 1e-9);
            // Just inside the threshold the award is still above the floor, so the floor is a
            // floor rather than a second regime.
            assert!(expansion_award(base, threshold - 0.01) > floor);
        }
        assert!((EDCHOICE_BASE_K8 * MINIMUM_SHARE - 550.0).abs() < 1e-9);
        assert!((EDCHOICE_BASE_9_12 * MINIMUM_SHARE - 750.0).abs() < 1e-9);
    }

    /// The award never rises with income, which a formula written as an exponential of a
    /// negative log could get wrong by a sign and still look plausible.
    #[test]
    fn the_award_never_rises_with_income() {
        let mut previous = f64::MAX;
        for step in 0..200 {
            let award = expansion_award(EDCHOICE_BASE_K8, f64::from(step) / 20.0);
            assert!(award <= previous + 1e-9, "the award rose at {step}");
            previous = award;
        }
    }

    /// The Jon Peterson formula exceeds its own ceiling for the most severely disabled category.
    #[test]
    fn the_sixth_category_is_capped_by_a_ceiling_that_is_not_indexed() {
        let uncapped: Vec<Dollars> = JON_PETERSON_SUPPLEMENTS
            .iter()
            .map(|supplement| JON_PETERSON_BASE + supplement)
            .collect();
        assert_eq!(
            uncapped
                .iter()
                .filter(|a| **a > JON_PETERSON_CEILING)
                .count(),
            1,
            "exactly one category's computed award exceeds the ceiling"
        );
        assert!((uncapped[5] - 39_122.0).abs() < 1e-9);
        assert!((uncapped[5] - JON_PETERSON_CEILING - 5_122.0).abs() < 1e-9);
        assert!((jon_peterson_award(6) - JON_PETERSON_CEILING).abs() < 1e-9);

        // Category five is the next to reach it, and it is not far.
        assert!((uncapped[4] - 29_750.0).abs() < 1e-9);
        assert!(JON_PETERSON_CEILING - uncapped[4] < 5_000.0);

        // The two ceilings are the same figure in two divisions.
        assert!((JON_PETERSON_CEILING - AUTISM_CEILING).abs() < f64::EPSILON);
    }

    #[test]
    fn the_categories_rise_and_there_are_six_of_them() {
        assert!(JON_PETERSON_SUPPLEMENTS.windows(2).all(|w| w[1] > w[0]));
        for category in 1..=6 {
            assert!(jon_peterson_award(category) > 0.0);
        }
        assert!(std::panic::catch_unwind(|| jon_peterson_award(7)).is_err());
        assert!(std::panic::catch_unwind(|| jon_peterson_award(0)).is_err());
    }
}
