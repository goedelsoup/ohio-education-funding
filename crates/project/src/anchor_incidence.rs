//! Who an anchor rule reaches outside the enrollment cluster, by wealth and by poverty, on total
//! state support — and whether #400's shapes reach the same districts.
//!
//! [`crate::rolling_anchor`] priced three anchor rules and [`crate::decline_adjustment`] four
//! supplements, both statewide and against the **89** of [`crate::enrollment_decline`]'s cluster,
//! and neither said who else the money reaches. Between 157 and 253 districts change standing
//! depending on the rule, and at FY2036 a 1% cap holds more districts than the enacted anchor while
//! writing less guarantee and *raising* total state support. A rule that is cheaper on the headline
//! and dearer in aggregate is redistributing, and this module says in which direction.
//!
//! # The enacted anchor's own population is the wealthy, and everything is measured against it
//!
//! Outside the cluster, the FY2027 guarantee holds **none** of the 103 least-wealthy districts by
//! assessed valuation per pupil and **77** of the 105 wealthiest. That is the corpus's 13%, 37%,
//! 68%, 76% quartile gradient with the enrollment cluster taken out, and it is the fact every
//! result below rests on: an alternative anchor is a *change* from this rule, so anything that
//! narrows the enacted floor takes money from the wealthy, and anything that reaches districts
//! the enacted floor does not — which are the ones the formula pays — gives money to the poor.
//! There is no rule here that runs the other way.
//!
//! # A ratchet adds money, and adds it at the bottom
//!
//! A prior-year anchor catches every district whose formula amount falls, and the districts with
//! the most formula aid per pupil to lose are the poor ones. Walked to FY2032 under the shipped
//! projection, outside the cluster:
//!
//! | quintile | by valuation per pupil | by disadvantaged share |
//! |---|--:|--:|
//! | 1 (least wealthy / least poor) | **$138.58** a pupil | $4.06 |
//! | 2 | $77.16 | $11.34 |
//! | 3 | $26.33 | $31.72 |
//! | 4 | $5.15 | $51.87 |
//! | 5 (wealthiest / poorest) | **$3.75** | **$108.08** |
//!
//! Monotone on both axes. **253** districts gain and none lose, the ten largest gainers hold
//! 25.8% of the gain, and the largest is Toledo City at $3.2m. This is the dearest rule priced and
//! its extra money is progressive: it goes where the enacted floor does not reach.
//!
//! # A cap takes money, and takes it at the top
//!
//! A cap on the annual fall lets a held district's floor decline, so it is a cut to the districts
//! the enacted anchor holds. At FY2032 a 2% cap takes **nothing** from the least-wealthy fifth
//! — its two held districts are fully backstopped, so `[K]` returns every dollar — and **$83.51** a
//! pupil from the fourth fifth. By poverty it takes $101.52 from the second-least-poor fifth and
//! **$5.33** from the poorest. **145** districts lose and one gains, Kelleys Island Local by
//! $12,011.69.
//!
//! The poorest fifth's small loss is the backstop, not the cap. Its guarantee falls $24.7m and its
//! total state support $1.6m: **93.4%** of the cut comes back on `[K]`, against **35.9%** in the
//! least-poor fifth. So the absorption [`crate::rolling_anchor::absorbed`] measured at 76.3%
//! statewide has a gradient of its own, and it runs the same way — `[K]` catches the poor and
//! not the wealthy, which is what [`crate::hold_harmless::retirement`] found about retiring the
//! guarantee outright. Incidence measured on `[I]` would have called the cap's cut to the
//! poorest fifth $24.7m; on total state support it is $1.6m.
//!
//! # The 1% cap's aggregate increase is a transfer, broad on both sides
//!
//! At FY2036 on the undamped projection a 1% cap holds 469 districts against 388 and statewide
//! total state support is $7.7m higher. Outside the cluster that is **112** districts gaining
//! **$53,969,540** and **119** losing **$43,682,097** — a net $10.3m that is the difference of two
//! numbers five times its size. The gainers are the poor and the losers the wealthy: **+$141.73** a
//! pupil in the least-wealthy fifth against **−$55.40** in the wealthiest, **+$123.40** in the
//! poorest against **−$68.15** in the least poor. The ten largest gainers hold **36.5%** of the
//! gain, the largest is Maple Heights City at $3.8m, and 77 of the 112 gainers are in the two
//! least-wealthy fifths. Not a few large districts: a general shift down the wealth distribution.
//!
//! Why a cap can *give*: by FY2036 a formula district's aid has fallen far enough that 99% of last
//! year's is above its FY2020 base. The cap then holds it where the enacted anchor would let it
//! fall, and the districts in that condition are the ones the formula has been paying. At 2% the
//! same rule is a net cut again — $5.0m gained against $54.6m lost outside the cluster — because
//! fewer districts fall faster than 2% a year. The rate decides the sign, and never the direction.
//!
//! # #400's shapes agree with this one
//!
//! The same cut, at the modelled year, on total state support, outside the cluster:
//!
//! | shape | least-wealthy fifth | wealthiest fifth | poorest fifth | least-poor fifth |
//! |---|--:|--:|--:|--:|
//! | rolling pupil count | **$166.55** | **$11.43** | $109.79 | $11.68 |
//! | mirror `[M]`, inside `[H]` | $114.22 | **$20.42** | $78.69 | $25.40 |
//! | mirror `[M]`, beside `[L]`, `[M]`, `[O]` | $114.22 | **$96.88** | $99.10 | $91.36 |
//! | dated phase-down | $0.00 | $0.00 | $0.00 | $0.00 |
//!
//! The rolling count and the mirror inside `[H]` are progressive for the same reason the ratchet
//! is: they are written inside the floors, and the floors absorb them where the wealthy are held.
//! The mirror written **beside** every floor is the only shape with no gradient at all — $114
//! against $97 by wealth, and hump-shaped by poverty — because it pays on the roll of any district
//! past the threshold, whatever its wealth, and no floor reads the line. The dated phase-down
//! reaches nobody outside the cluster by construction.
//!
//! So the shapes do not disagree about who they reach. Every one of the seven is progressive
//! against the enacted anchor or neutral, and none is regressive. The choice between them is
//! **not distributional in direction**; it is the sign and the size — whether to add money at the
//! bottom (a ratchet, the rolling count, either mirror) or take it at the top (a cap), and at a
//! long enough horizon the 1% cap does both. That is the fiscal framing #390 and #400 used, and
//! it survives the cut.
//!
//! # What this rests on
//!
//! The supplement arms of [`under_supplement`] are the per-district form of the aggregates
//! `decline_adjustment` publishes, and `tests/who_a_capped_or_rolling_anchor_reaches.rs` asserts
//! each sums to its published total to the cent before any cut of it is read. The anchor arms are
//! [`crate::rolling_anchor::walk`]'s own standings, joined on IRN.
//!
//! # Not measured here
//!
//! **Community and STEM schools.** Every figure is over the 609 districts, as in #383, #390 and
//! #400.
//!
//! **A district's own trajectory.** The bands are rules cut on positions, not lists of districts;
//! 50 of the 89 arrived in one year and the same instability applies outside it.
//!
//! **A lever.** An incidence cut is a module, as every anchor rule and every shape was.

use std::collections::{BTreeMap, BTreeSet};

use edfund_core::{Adm, Dollars};

use crate::decline_adjustment::{self, Line};
use crate::panel::DistrictRecord;
use crate::policy::{apply, GuaranteeRule, Policy, Statewide};
use crate::rolling_anchor::{Standing, HELD};

/// The axis a cut is taken on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Assessed valuation per pupil, FY2023 — [`DistrictRecord::valuation_per_pupil`].
    Valuation,
    /// The economically disadvantaged share, as the DPIA percentage —
    /// [`crate::panel::Dpia::percentage`].
    Poverty,
}

impl Axis {
    /// The district's position on the axis, or `None` where it publishes nothing.
    #[must_use]
    pub fn of(self, record: &DistrictRecord) -> Option<f64> {
        match self {
            Self::Valuation => record.valuation_per_pupil,
            Self::Poverty => Some(record.dpia.percentage),
        }
    }

    /// A short label for a table.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Valuation => "valuation per pupil",
            Self::Poverty => "economically disadvantaged share",
        }
    }
}

/// One district under two rules, on total state support.
///
/// The "rule in force" is whatever the comparison's control is: the enacted anchor for a walk,
/// current law for a supplement.
#[derive(Debug, Clone, PartialEq)]
pub struct Movement {
    /// Information Retrieval Number.
    pub irn: String,
    /// District name.
    pub name: String,
    /// Enrolled ADM in the year compared — the same under both rules.
    pub adm: Adm,
    /// Assessed valuation per pupil, FY2023.
    pub valuation_per_pupil: Option<Dollars>,
    /// The DPIA percentage.
    pub poverty: f64,
    /// Total state support under the rule in force.
    pub as_enacted: Dollars,
    /// Total state support under the alternative.
    pub alternative: Dollars,
    /// Whether a floor determines the district's aid under the rule in force.
    pub held_as_enacted: bool,
    /// And under the alternative.
    pub held_under_alternative: bool,
    /// `[I]` under the rule in force.
    pub guarantee_as_enacted: Dollars,
    /// `[I]` under the alternative.
    pub guarantee_under_alternative: Dollars,
}

impl Movement {
    /// What the alternative changes, in total state support.
    #[must_use]
    pub fn delta(&self) -> Dollars {
        self.alternative - self.as_enacted
    }

    /// The same per pupil.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.delta() / self.adm
    }

    /// Held under the alternative and not under the rule in force.
    #[must_use]
    pub fn enters(&self) -> bool {
        self.held_under_alternative && !self.held_as_enacted
    }

    /// Held under the rule in force and not under the alternative.
    #[must_use]
    pub fn leaves(&self) -> bool {
        self.held_as_enacted && !self.held_under_alternative
    }

    /// Whether the district's standing differs between the two rules.
    #[must_use]
    pub fn changes_standing(&self) -> bool {
        self.held_as_enacted != self.held_under_alternative
    }
}

/// Two walks, joined on IRN.
///
/// A district one walk carries and the other does not is skipped; both walks run the same panel
/// under the same projection, so the join is total in practice and [`Band::districts`] says so.
#[must_use]
pub fn between_walks(
    panel: &[DistrictRecord],
    as_enacted: &BTreeMap<String, Standing>,
    alternative: &BTreeMap<String, Standing>,
) -> Vec<Movement> {
    panel
        .iter()
        .filter_map(|record| {
            let before = as_enacted.get(&record.irn)?;
            let after = alternative.get(&record.irn)?;
            Some(Movement {
                irn: record.irn.clone(),
                name: record.name.clone(),
                adm: after.adm,
                valuation_per_pupil: record.valuation_per_pupil,
                poverty: record.dpia.percentage,
                as_enacted: before.total_state_support(),
                alternative: after.total_state_support(),
                held_as_enacted: before.on_the_floor(),
                held_under_alternative: after.on_the_floor(),
                guarantee_as_enacted: before.guarantee(),
                guarantee_under_alternative: after.guarantee(),
            })
        })
        .collect()
}

/// One of #400's shapes, as a per-district movement at the modelled year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Supplement {
    /// `[M]` mirrored at its own rate, written on a stated line.
    Mirror(Line),
    /// The state share paid on the rolling three-year count.
    RollingCount,
    /// The guarantee retired for the cluster members the FY2026 model found on it.
    DatedPhaseDown,
}

impl Supplement {
    /// A short label for a table.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mirror(Line::Beside) => "mirror [M], beside [L], [M], [O]",
            Self::Mirror(Line::Foundation) => "mirror [M], inside [H]",
            Self::RollingCount => "rolling pupil count",
            Self::DatedPhaseDown => "dated phase-down",
        }
    }
}

/// One of #400's shapes against current law, district by district, at the modelled year.
///
/// Each arm is the per-district form of the aggregate `decline_adjustment` publishes, and the
/// tests assert the sum reproduces that aggregate to the cent before any cut of it is read.
#[must_use]
pub fn under_supplement(panel: &[DistrictRecord], shape: Supplement) -> Vec<Movement> {
    let law = Policy::current_law();
    let statewide = Statewide::under(panel, &law);
    let rate = crate::panel::ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL;
    let prior = crate::prior_model::by_irn();
    let cluster: BTreeSet<String> = crate::enrollment_decline::cluster(panel)
        .into_iter()
        .map(|record| record.irn.clone())
        .collect();

    panel
        .iter()
        .filter(|record| record.current_year_adm > 0.0)
        .map(|record| {
            let before = apply(record, &law, &statewide, record.current_year_adm);
            let (after_total, held_after, guarantee_after) = match shape {
                Supplement::Mirror(line) => {
                    let amount = decline_adjustment::payment(record, rate);
                    let received = decline_adjustment::arrives(record, &before, amount, line);
                    let floor = record.guarantee_floor();
                    let formula = match line {
                        Line::Foundation => before.formula_aid + amount,
                        Line::Beside => before.formula_aid,
                    };
                    let guarantee = (floor - formula).max(0.0);
                    (
                        before.total_state_support() + received,
                        guarantee > HELD,
                        guarantee,
                    )
                }
                Supplement::RollingCount => {
                    let after = apply(record, &law, &statewide, record.base_cost_adm());
                    (
                        after.total_state_support(),
                        after.on_guarantee,
                        after.guarantee,
                    )
                }
                Supplement::DatedPhaseDown => {
                    let keyed = cluster.contains(&record.irn)
                        && prior
                            .get(&record.irn)
                            .is_some_and(|year| year.guarantee > 0.0);
                    if keyed {
                        let retired = Policy {
                            guarantee: GuaranteeRule::Removed,
                            ..law
                        };
                        let after = apply(record, &retired, &statewide, record.current_year_adm);
                        (
                            after.total_state_support(),
                            after.on_guarantee,
                            after.guarantee,
                        )
                    } else {
                        (
                            before.total_state_support(),
                            before.on_guarantee,
                            before.guarantee,
                        )
                    }
                }
            };
            Movement {
                irn: record.irn.clone(),
                name: record.name.clone(),
                adm: record.current_year_adm,
                valuation_per_pupil: record.valuation_per_pupil,
                poverty: record.dpia.percentage,
                as_enacted: before.total_state_support(),
                alternative: after_total,
                held_as_enacted: before.on_guarantee,
                held_under_alternative: held_after,
                guarantee_as_enacted: before.guarantee,
                guarantee_under_alternative: guarantee_after,
            }
        })
        .collect()
}

/// One band of an axis, and what the alternative does to it.
#[derive(Debug, Clone, PartialEq)]
pub struct Band {
    /// `1` is the lowest band on the axis: least wealthy, or least poor.
    pub rank: usize,
    /// Districts in it.
    pub districts: usize,
    /// Their enrolled ADM between them.
    pub adm: Adm,
    /// Change in their total state support, summed.
    pub delta: Dollars,
    /// Districts whose total state support rises.
    pub gainers: usize,
    /// Districts whose total state support falls.
    pub losers: usize,
    /// Districts held under the alternative and not under the rule in force.
    pub enters: usize,
    /// Districts held under the rule in force and not under the alternative.
    pub leaves: usize,
    /// Districts held under the rule in force.
    pub held_as_enacted: usize,
    /// Districts held under the alternative.
    pub held_under_alternative: usize,
    /// Change in `[I]`, summed.
    pub guarantee_delta: Dollars,
    /// Upper-middle value of the axis in the band, for the reader who wants to know where it sits.
    pub median_position: f64,
}

impl Band {
    /// The change per pupil.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.delta / self.adm
    }

    /// The share of the band's change in `[I]` that never reaches total state support.
    ///
    /// [`crate::rolling_anchor::absorbed`] for one band: `1.0` is a cut to the guarantee that
    /// `[K]` returns in full. `NaN` where the guarantee does not move.
    #[must_use]
    pub fn absorbed_share(&self) -> f64 {
        if self.guarantee_delta.abs() <= HELD {
            return f64::NAN;
        }
        1.0 - self.delta / self.guarantee_delta
    }
}

/// A district's position on an axis, or `None` where it publishes nothing to be placed on.
fn position(movement: &Movement, axis: Axis) -> Option<f64> {
    match axis {
        Axis::Valuation => movement.valuation_per_pupil,
        Axis::Poverty => Some(movement.poverty),
    }
}

/// The bands themselves, before anything is summed over them.
///
/// [`by`] is this function plus a sum, and [`crate::decline_reach::split_by`] is this function
/// plus a different sum — so a band there and a band here hold the same districts by
/// construction rather than by two sorts agreeing.
///
/// Bands are equal in count, not in width: `width` is the placed districts over `bands` and the
/// last band takes the remainder, so with 520 placed and five bands the last holds 105.
pub fn banded<'a>(
    movements: &'a [Movement],
    axis: Axis,
    bands: usize,
    exclude: &BTreeSet<String>,
) -> Vec<Vec<&'a Movement>> {
    assert!(bands > 0, "at least one band");
    let mut placed: Vec<(f64, &Movement)> = movements
        .iter()
        .filter(|movement| !exclude.contains(&movement.irn))
        .filter_map(|movement| Some((position(movement, axis)?, movement)))
        .collect();
    placed.sort_by(|a, b| a.0.total_cmp(&b.0));

    let width = placed.len() / bands;
    (0..bands)
        .map(|index| {
            let slice = if index + 1 == bands {
                &placed[index * width..]
            } else {
                &placed[index * width..(index + 1) * width]
            };
            slice.iter().map(|(_, movement)| *movement).collect()
        })
        .collect()
}

/// Cut a set of movements into equal bands on an axis.
///
/// Districts publishing nothing on the axis are dropped, as [`crate::transport::floor::incidence`]
/// drops them, because the axis is the position. `exclude` is the set to leave out — the cluster,
/// for the question this module answers — and a district in it is neither banded nor counted.
#[must_use]
pub fn by(
    movements: &[Movement],
    axis: Axis,
    bands: usize,
    exclude: &BTreeSet<String>,
) -> Vec<Band> {
    banded(movements, axis, bands, exclude)
        .into_iter()
        .enumerate()
        .map(|(index, slice)| {
            let mut out = Band {
                rank: index + 1,
                districts: slice.len(),
                adm: 0.0,
                delta: 0.0,
                gainers: 0,
                losers: 0,
                enters: 0,
                leaves: 0,
                held_as_enacted: 0,
                held_under_alternative: 0,
                guarantee_delta: 0.0,
                median_position: slice
                    .get(slice.len() / 2)
                    .and_then(|movement| position(movement, axis))
                    .unwrap_or(0.0),
            };
            for movement in slice {
                out.adm += movement.adm;
                out.delta += movement.delta();
                out.guarantee_delta +=
                    movement.guarantee_under_alternative - movement.guarantee_as_enacted;
                if movement.delta() > HELD {
                    out.gainers += 1;
                } else if movement.delta() < -HELD {
                    out.losers += 1;
                }
                if movement.enters() {
                    out.enters += 1;
                }
                if movement.leaves() {
                    out.leaves += 1;
                }
                if movement.held_as_enacted {
                    out.held_as_enacted += 1;
                }
                if movement.held_under_alternative {
                    out.held_under_alternative += 1;
                }
            }
            out
        })
        .collect()
}

/// How a change in total state support is spread across districts.
#[derive(Debug, Clone, PartialEq)]
pub struct Spread {
    /// Districts whose total state support rises.
    pub gainers: usize,
    /// Districts whose total state support falls.
    pub losers: usize,
    /// What the gainers gain between them.
    pub gained: Dollars,
    /// What the losers lose between them, as a positive number.
    pub lost: Dollars,
    /// The share of what is gained that the ten largest gainers gain.
    pub top_ten_share_of_gains: f64,
    /// The largest single gain, and whose it is.
    pub largest_gain: (String, Dollars),
    /// The largest single loss, and whose it is, as a positive number.
    pub largest_loss: (String, Dollars),
}

impl Spread {
    /// The net change.
    #[must_use]
    pub fn net(&self) -> Dollars {
        self.gained - self.lost
    }
}

/// Measure the spread of a change across the districts not excluded.
#[must_use]
pub fn spread(movements: &[Movement], exclude: &BTreeSet<String>) -> Spread {
    let mut gains: Vec<(String, Dollars)> = Vec::new();
    let mut out = Spread {
        gainers: 0,
        losers: 0,
        gained: 0.0,
        lost: 0.0,
        top_ten_share_of_gains: 0.0,
        largest_gain: (String::new(), 0.0),
        largest_loss: (String::new(), 0.0),
    };
    for movement in movements
        .iter()
        .filter(|movement| !exclude.contains(&movement.irn))
    {
        let delta = movement.delta();
        if delta > HELD {
            out.gainers += 1;
            out.gained += delta;
            gains.push((movement.name.clone(), delta));
            if delta > out.largest_gain.1 {
                out.largest_gain = (movement.name.clone(), delta);
            }
        } else if delta < -HELD {
            out.losers += 1;
            out.lost -= delta;
            if -delta > out.largest_loss.1 {
                out.largest_loss = (movement.name.clone(), -delta);
            }
        }
    }
    gains.sort_by(|a, b| b.1.total_cmp(&a.1));
    if out.gained > 0.0 {
        out.top_ten_share_of_gains =
            gains.iter().take(10).map(|g| g.1).sum::<Dollars>() / out.gained;
    }
    out
}
