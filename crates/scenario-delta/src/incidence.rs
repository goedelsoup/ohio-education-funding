//! Who the change lands on, crossed against the axis that is supposed to explain it.
//!
//! A statewide total says what a policy costs. It does not say whether the money reaches the
//! districts the policy was argued for, and in Ohio that is usually the whole question: the Fair
//! School Funding Plan exists because *DeRolph* held that a formula's **distribution** can be
//! unconstitutional while its total is adequate.
//!
//! # Equal counts are the wrong default, and one axis proves it
//!
//! The obvious binning puts an equal number of districts in each band. On assessed valuation
//! that is fine — valuation is continuous and no two districts share a value.
//!
//! On **state share** it is wrong, and wrong exactly where the axis matters most. 138 of 609
//! districts sit at the minimum state share — 23% of the panel, more than a quintile — because a
//! floor is a mass point, not a tail. Equal-count binning cuts that group across a band boundary
//! and reports two adjacent bands with the same axis value and different incidence, which is an
//! artefact of the cut rather than a fact about Ohio. The first version of this module did
//! precisely that: it produced `Q1 [0.100 .. 0.100]` beside `Q2 [0.100 .. 0.272]`.
//!
//! So a band boundary is pushed forward past any run of equal values, and the bands come out
//! uneven and honest rather than even and invented.
//!
//! # Why "equal" needs a tolerance, which is the part that is easy to get wrong
//!
//! **No district's state share is exactly `0.10`.** The 138 are spread from `0.099999379565715`
//! upward — the fraction is recovered by dividing published dollars by published ADM, so the
//! floor arrives as a cluster six decimal places wide, not a repeated value. Tie-coherence
//! written as `==` therefore does nothing at all on the one axis it was written for, and does it
//! silently: the table still renders, still sums, and still looks like five quintiles.
//!
//! [`Axis::resolution`] is the fix and it is a required field rather than a default, so every
//! call site states the precision at which two districts count as the same. [`Axis::state_share`]
//! uses `0.0005`, the same tolerance [`project::panel::DistrictRecord::at_minimum_state_share`]
//! uses to decide the question directly.
//!
//! # Nothing is dropped silently
//!
//! Three districts have no assessed valuation in the committed data. They go to
//! [`Incidence::unclassified`], which carries their dollars as well as their count, so the bands
//! plus the unclassified add to the statewide total exactly. A test asserts that. A table whose
//! parts do not sum to the whole has lost districts somewhere and gives the reader no way to see
//! it.

use edfund_core::{Adm, Dollars};

use crate::{Delta, MOVED};

/// What to cross a delta against, and at what precision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Axis {
    /// What the bands are bands of.
    pub label: &'static str,
    /// How many bands to cut, at most. Fewer come back when a mass point spans a boundary.
    pub bands: usize,
    /// Values within this of each other are the same value for the purpose of a band boundary.
    ///
    /// Required rather than defaulted. A computed axis almost never repeats a value exactly, so
    /// a zero tolerance silently disables the mass-point handling this module exists for.
    pub resolution: f64,
}

impl Axis {
    /// An axis with an explicit precision.
    #[must_use]
    pub const fn new(label: &'static str, bands: usize, resolution: f64) -> Self {
        Self {
            label,
            bands,
            resolution,
        }
    }

    /// Assessed valuation per pupil, in quintiles, to the dollar.
    ///
    /// Ohio's wealth measure and the axis every equity claim about the formula is made across.
    #[must_use]
    pub const fn valuation() -> Self {
        Self::new("assessed valuation per pupil", 5, 1.0)
    }

    /// State share of base cost, in quintiles, to the same tolerance the panel uses.
    #[must_use]
    pub const fn state_share() -> Self {
        Self::new("state share of base cost", 5, 0.0005)
    }
}

/// One band of the axis, and what the perturbation did inside it.
#[derive(Debug, Clone, PartialEq)]
pub struct Stratum {
    /// `Q1` is the lowest band of the axis.
    pub label: String,
    /// Lowest axis value in the band.
    pub lower: f64,
    /// Highest axis value in the band.
    pub upper: f64,
    /// Districts in the band.
    pub districts: usize,
    /// Enrolled ADM in the band.
    pub adm: Adm,
    /// Change in state aid across the band.
    pub dollars: Dollars,
    /// Districts in the band the perturbation moved.
    pub moved: usize,
    /// Districts in the band the guarantee paid under both policies.
    pub held_throughout: usize,
}

impl Stratum {
    /// Change per pupil in the band. The comparable figure across bands of unequal size.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.dollars / self.adm
    }

    /// Share of the band's districts the perturbation moves.
    #[must_use]
    pub fn reaches(&self) -> f64 {
        if self.districts == 0 {
            return 0.0;
        }
        self.moved as f64 / self.districts as f64
    }
}

/// The districts an axis cannot place, kept where they can be seen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Unclassified {
    /// How many districts have no observation on this axis.
    pub districts: usize,
    /// Enrolled ADM across them.
    pub adm: Adm,
    /// Change in state aid across them — carried so the table sums to the total.
    pub dollars: Dollars,
}

/// A perturbation's incidence across one axis.
#[derive(Debug, Clone, PartialEq)]
pub struct Incidence {
    /// The axis, at the precision it was cut at.
    pub axis: Axis,
    /// The bands, lowest first. May be fewer than [`Axis::bands`] when a mass point is wide.
    pub strata: Vec<Stratum>,
    /// Districts the axis could not place.
    pub unclassified: Unclassified,
}

impl Incidence {
    /// Total change across every band and the unclassified. Equal to the statewide total.
    #[must_use]
    pub fn dollars(&self) -> Dollars {
        self.strata.iter().map(|s| s.dollars).sum::<Dollars>() + self.unclassified.dollars
    }

    /// Ratio of the top band's per-pupil change to the bottom band's.
    ///
    /// The one-number summary of how a policy runs across the axis: below 1 it favours the
    /// bottom. `None` when the bottom band gets nothing — the case a ratio cannot describe, and
    /// the case a floor produces. Report the two per-pupil figures instead of forcing it.
    #[must_use]
    pub fn gradient(&self) -> Option<f64> {
        let first = self.strata.first()?;
        let last = self.strata.last()?;
        (first.per_pupil().abs() > MOVED).then(|| last.per_pupil() / first.per_pupil())
    }
}

/// Cut the panel into bands along an axis, keeping equal values together.
///
/// `key` returns `None` for a district the axis cannot place; those go to
/// [`Incidence::unclassified`] rather than into a band.
///
/// # Panics
///
/// If [`Axis::bands`] is zero. Zero bands is a caller mistake, and an empty incidence would let
/// it through as a table that shows nothing.
#[must_use]
pub fn across<F>(deltas: &[Delta], axis: &Axis, key: F) -> Incidence
where
    F: Fn(&Delta) -> Option<f64>,
{
    assert!(axis.bands > 0, "an incidence table needs at least one band");

    let mut classified: Vec<(f64, &Delta)> = Vec::new();
    let mut unclassified = Unclassified {
        districts: 0,
        adm: 0.0,
        dollars: 0.0,
    };
    for delta in deltas {
        match key(delta) {
            Some(value) => classified.push((value, delta)),
            None => {
                unclassified.districts += 1;
                unclassified.adm += delta.adm;
                unclassified.dollars += delta.dollars();
            }
        }
    }
    classified.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let total = classified.len();
    let mut strata = Vec::new();
    let mut start = 0;
    for band in 0..axis.bands {
        if start >= total {
            break;
        }
        // The equal-count boundary, then pushed forward past every value indistinguishable from
        // the one it lands on, so the floor's 138 districts stay in one band. Measured against
        // the boundary value rather than the running one: comparing each value to its
        // predecessor lets a long shallow gradient chain one band across the whole axis.
        let mut end = ((band + 1) * total / axis.bands).max(start + 1).min(total);
        if band + 1 < axis.bands {
            let boundary = classified[end - 1].0;
            while end < total && (classified[end].0 - boundary).abs() <= axis.resolution {
                end += 1;
            }
        } else {
            end = total;
        }

        let rows = &classified[start..end];
        strata.push(Stratum {
            label: format!("Q{}", strata.len() + 1),
            lower: rows[0].0,
            upper: rows[rows.len() - 1].0,
            districts: rows.len(),
            adm: rows.iter().map(|(_, d)| d.adm).sum(),
            dollars: rows.iter().map(|(_, d)| d.dollars()).sum(),
            moved: rows.iter().filter(|(_, d)| d.moved()).count(),
            held_throughout: rows
                .iter()
                .filter(|(_, d)| d.standing.off_formula())
                .count(),
        });
        start = end;
    }

    Incidence {
        axis: *axis,
        strata,
        unclassified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ScenarioDelta, Standing};
    use project::panel::panel;
    use project::policy::Policy;

    fn refresh() -> ScenarioDelta {
        ScenarioDelta::between(
            &panel(),
            &Policy::current_law(),
            &Policy {
                base_cost_scale: 1.05,
                ..Policy::current_law()
            },
        )
    }

    fn row(value: Option<f64>, dollars: Dollars) -> Delta {
        Delta {
            irn: format!("{value:?}{dollars}"),
            name: "somewhere local".into(),
            adm: 100.0,
            baseline: 1_000_000.0,
            perturbed: 1_000_000.0 + dollars,
            standing: Standing::FormulaThroughout,
            valuation_per_pupil: value,
            state_share_fraction: 0.0,
        }
    }

    #[test]
    fn the_bands_and_the_unclassified_sum_to_the_statewide_total() {
        let delta = refresh();
        let incidence = across(&delta.deltas, &Axis::valuation(), |d| d.valuation_per_pupil);
        assert!((incidence.dollars() - delta.total().dollars).abs() < 1.0);
        let placed: usize = incidence.strata.iter().map(|s| s.districts).sum();
        assert_eq!(
            placed + incidence.unclassified.districts,
            delta.deltas.len()
        );
    }

    #[test]
    fn districts_without_an_observation_are_set_aside_rather_than_binned() {
        let deltas = vec![
            row(Some(1.0), 10.0),
            row(None, 25.0),
            row(Some(2.0), 10.0),
            row(None, 5.0),
        ];
        let incidence = across(&deltas, &Axis::new("test", 2, 0.0), |d| {
            d.valuation_per_pupil
        });
        assert_eq!(incidence.unclassified.districts, 2);
        assert_eq!(incidence.unclassified.dollars, 30.0);
        assert_eq!(
            incidence.strata.iter().map(|s| s.districts).sum::<usize>(),
            2
        );
    }

    #[test]
    fn a_mass_point_is_never_split_across_two_bands() {
        // Five districts, three sharing a value that straddles the equal-count boundary. Equal
        // counts would put two in the first band and one in the second; keeping ties together
        // puts all three in the first.
        let deltas = vec![
            row(Some(1.0), 1.0),
            row(Some(5.0), 1.0),
            row(Some(5.0), 1.0),
            row(Some(5.0), 1.0),
            row(Some(9.0), 1.0),
        ];
        let incidence = across(&deltas, &Axis::new("test", 2, 0.0), |d| {
            d.valuation_per_pupil
        });
        assert_eq!(incidence.strata[0].districts, 4);
        assert_eq!(incidence.strata[0].upper, 5.0);
        assert_eq!(incidence.strata[1].districts, 1);
        assert_eq!(incidence.strata[1].lower, 9.0);
    }

    #[test]
    fn a_cluster_is_held_together_only_at_a_resolution_that_can_see_it() {
        // The defect this module shipped with, as a test. The values differ in the sixth decimal
        // — which is what the real floor looks like — so exact equality splits them and the
        // tolerance the panel itself uses does not.
        let deltas = vec![
            row(Some(0.0999993795), 1.0),
            row(Some(0.0999993816), 1.0),
            row(Some(0.0999993835), 1.0),
            row(Some(0.4), 1.0),
        ];
        let split = across(&deltas, &Axis::new("state share", 2, 0.0), |d| {
            d.valuation_per_pupil
        });
        assert_eq!(split.strata[0].districts, 2, "exact equality cannot see it");

        let whole = across(&deltas, &Axis::new("state share", 2, 0.0005), |d| {
            d.valuation_per_pupil
        });
        assert_eq!(whole.strata[0].districts, 3);
        assert_eq!(whole.strata[1].districts, 1);
    }

    #[test]
    fn the_bands_are_ordered_and_do_not_overlap() {
        let delta = refresh();
        let incidence = across(&delta.deltas, &Axis::valuation(), |d| d.valuation_per_pupil);
        assert_eq!(incidence.strata.len(), 5);
        for pair in incidence.strata.windows(2) {
            assert!(pair[0].upper <= pair[1].lower, "{pair:?}");
        }
    }

    #[test]
    fn a_gradient_is_absent_rather_than_infinite_when_the_bottom_band_gets_nothing() {
        let deltas = vec![row(Some(1.0), 0.0), row(Some(2.0), 500.0)];
        let incidence = across(&deltas, &Axis::new("test", 2, 0.0), |d| {
            d.valuation_per_pupil
        });
        assert_eq!(incidence.gradient(), None);
    }

    #[test]
    #[should_panic(expected = "at least one band")]
    fn zero_bands_is_a_caller_mistake_and_says_so() {
        let _ = across(&[], &Axis::new("test", 0, 0.0), |_| None);
    }
}
