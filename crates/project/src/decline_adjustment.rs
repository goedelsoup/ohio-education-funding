//! What a declining-enrollment adjustment costs, and why the rate is not the decision.
//!
//! [`crate::enrollment_decline`] characterized the **89 districts holding $108,568,921.47** that
//! are on the guarantee for no reason but having fewer children than in FY2020, named four
//! candidate shapes, and deliberately priced none of them. This prices all four, on total state
//! support, against that cluster.
//!
//! # Placement decides what a payment is worth, and no rate can overcome it
//!
//! Every member of the cluster is on the guarantee, **69 of the 89 also draw `[K]`**, and **68 of
//! the 89 are fully backstopped** — their FY2021 total `[L1]` is at or above `[H2] + [J]`, so a
//! dollar taken off foundation funding is a dollar `[K]` puts back
//! ([`fully_backstopped`]). A response to them therefore runs into two floors before it runs
//! into a price, and the same dollars are worth wildly different amounts depending on which line
//! they are written on:
//!
//! | mirrored `[M]` paid on | statewide | reaching the 89 | of eligible members, reached |
//! |---|--:|--:|--:|
//! | a new line beside `[L]`, `[M]`, `[O]` | $181,321,579.23 | $46,020,643.67 | 81 of 81 |
//! | inside `[H] Foundation Funding` | $66,385,014.71 | **$4,195,897.42** | **14 of 81** |
//!
//! Inside `[H]`, **90.9%** of what the rule writes for this cluster is absorbed and 67 of its 81
//! eligible members receive nothing whatever. Statewide 200 of the 383 eligible districts receive
//! nothing and 29 receive part.
//!
//! So there are **three** placements and not two. [`crate::supplement_reach`] asked whether a
//! payment sits inside `[H]`; [`crate::hold_harmless`] corrected the boundary to
//! [`crate::biennium::Measure::FoundationAid`], `[H] + [I]`. Both are inside `[L1]`'s subtrahend.
//! The third boundary is that subtrahend itself — `[H] + [I] + [J]` — and `[L]`, `[M]` and `[O]`
//! are the only lines outside it. That is why the performance supplement reaches a guaranteed
//! district, and it is the whole of what makes a declining-enrollment supplement reachable.
//!
//! It is a placement and not a law of nature: a later act that re-based `[L1]` on a total
//! including the new line would absorb it too. See [`Line`].
//!
//! # Mirroring `[M]`: symmetric in form, four and a half times the money
//!
//! `[M]` pays [`crate::panel::ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL`] — $250 — on the whole roll
//! of a district whose three-year enrollment change cleared
//! [`crate::panel::ENROLLMENT_GROWTH_THRESHOLD`], 3%. The exact mirror pays the same rate on the
//! same count past [`MIRROR_THRESHOLD`]. It is not the same size:
//!
//! | | districts | cost |
//! |---|--:|--:|
//! | `[M]` as enacted | 43 | $39,379,553.15 |
//! | its mirror | **383** | **$181,321,579.23** |
//!
//! **Decline is the ordinary case in Ohio and growth is the exception**, so a rule symmetric in
//! its wording is nine times as wide in its population and 4.6 times as expensive. And it is
//! poorly aimed at the cluster it would be written for: only **25.4%** of it arrives there
//! ([`Arrival::targeting`]). Calibrating the rate to deliver the cluster's whole guarantee needs
//! **$589.78 a pupil** and costs **$427,762,124.28** statewide — 3.9 times the floor it replaces
//! ([`replacing_the_floor`]).
//!
//! Two defects transfer with the form.
//!
//! **The cliff transfers, and it is larger.** Paying on the whole roll rather than the pupils
//! lost turns the threshold into a cliff: 15 districts sit in the last three tenths of a point
//! before it, forgoing $13,480,937.41, and **Akron City at −2.81% would draw $4,723,112.21** —
//! eleven times New Lexington's $430,477 on the growth side. See [`cliff`].
//!
//! **The window transfers, and it disagrees with the cluster.** `[M]`'s measure is `M1`, the
//! change from its own FY2023 base, and the cluster is defined from FY2020. **Eight members clear
//! no decline threshold at all on `[M]`'s window and two of them read as growing** — Southington
//! Local at +1.89% and Sebring Local at +0.27% — while holding $503,461.00 of guarantee between
//! them for enrollment lost before FY2023. The mirror's own window is the reason, not its
//! threshold.
//!
//! `M1` is read and never reconstructed here. The department computes it from two columns that
//! both say FY2023, and `tests/the_two_columns_that_both_say_fy2023.rs` is what that costs.
//!
//! # A rolling pupil count: the right mechanism, reaching almost nobody
//!
//! R.C. 3317.017 computes base cost on the three-year average
//! [`DistrictRecord::base_cost_adm`] and then pays the state share on **current-year** enrolled
//! ADM, which for a shrinking district is the smaller of the two. Paying on the rolling count
//! instead is the shape that needs no new device at all — it is a count substitution, and
//! [`crate::policy::apply`] already takes the count as an argument.
//!
//! It costs **$70,105,642.08** statewide and delivers **$3,902,794.07** to the cluster: **5.6%**,
//! or **$19.82 a pupil** against the $464.06 the floor pays the median member. The floors are
//! why. What it does do is move 20 of the 89 off the guarantee onto the formula, which is the
//! structural result rather than the fiscal one: it substitutes a rule for a floor without
//! substituting any money.
//!
//! Of the statewide cost, **$48,799,954.14 of gross repricing is categorical** rather than base
//! cost, and [`Rolling::categorical`] carries it separately for the reason
//! [`crate::refresh::Refreshed::categorical_delta`] does — a reader who holds that the
//! substitution should reach only base cost's own count can take it out. It is not a clean
//! subtraction where a floor binds, so the figure is gross.
//!
//! # A dated phase-down retires $695,296
//!
//! [`crate::enrollment_decline`] recorded the crossing rate as evidence against any date-keyed
//! instrument: 39 of the 89 were already on the guarantee in the FY2026 model and 50 arrived
//! within the year, so a rule keyed to a roll reaches the first group and misses the second.
//!
//! That objection is real and it is not the binding one. Retiring the guarantee for the 39 the
//! FY2026 model would have named saves **$67,392,975.21 of foundation aid and $695,295.72 of
//! total state support** — **32 of the 39 are made whole by `[K]`**. Undated, across all 89, the
//! same rule reports $108,568,921.47 and saves **$3,628,568.90**, with 69 made whole. A phase-down
//! of this population is not a saving that lands on the wrong districts; it is not a saving.
//!
//! **The guarantee is this cluster's nominal payer and `[K]` is its real one.** The
//! $108,568,921.47 is a line attribution: the money survives the line's repeal.
//!
//! # Nothing: the total does not move, and the provenance does
//!
//! Do nothing and the cluster is not cut. Its total state support is **flat** — every floor holds
//! it — while its enrollment falls, so the per-pupil amount rises and the line the money arrives
//! on migrates out of the formula:
//!
//! | | ADM | `[I]` | `[K]` | total support | per pupil | held share |
//! |---|--:|--:|--:|--:|--:|--:|
//! | FY2027 modelled | 196,869.5 | $108.6m | $37.8m | $1,448.70m | $7,358.70 | 10.1% |
//! | FY2032, shipped method | 191,607.9 | $144.6m | $40.3m | $1,448.19m | $7,558.09 | 12.8% |
//! | FY2036, undamped | 163,353.9 | $335.7m | $53.8m | $1,445.70m | $8,850.12 | **26.9%** |
//!
//! So the honest baseline is not austerity. It is that a quarter of what these districts receive
//! ends up riding on a hold-harmless, one of which — `[K]`, H.B. 110 Section 265.225 — is
//! uncodified and renewed a biennium at a time. **That, and not a funding shortfall, is what
//! pricing a shape buys: the same money on a rule that exists.**
//!
//! The shipped row is a floor on the drift and the undamped row is the honest end of it.
//! `tests/the_trend_the_damping_discards.rs` measures what
//! [`crate::series::DEFAULT_DAMPING`] discards — the fastest-declining quarter of Ohio districts
//! are forecast about 8% high at five years — and this cluster is that quarter. Both are
//! reported, at [`nothing`].
//!
//! # What each shape rewards at the margin
//!
//! Asked of every shape whether or not it is quantified, per [`Shape::rewards_at_the_margin`]:
//!
//! - **Mirroring `[M]` on the whole roll** rewards *crossing the threshold* and then penalises
//!   further decline — an eligible district loses $250 for every additional pupil it loses. That
//!   perverse-looking property is the good half of the design, and it is `[M]`'s own.
//! - **Mirroring it on the pupils lost** — $14,236,934.77 statewide, [`marginal`] — pays a
//!   district for each child it does not enroll, at the same rate, forever. It is cheaper and it
//!   is the version to refuse.
//! - **A rolling pupil count** rewards nothing. It re-times a loss rather than pricing it: the
//!   pupil is still worth the same and arrives in the count more slowly.
//! - **A dated phase-down** rewards being on the floor before the date, which is unearnable, and
//!   nothing thereafter.
//! - **Nothing** rewards crossing the floor, which is the behaviour already in force.
//!
//! # No lever was added, and none is needed
//!
//! The issue that commissioned this noted that a threshold-plus-rate shape is not a scalar and
//! that adding a [`crate::policy::Policy`] field is a sixteen-file cross-stack change. None of
//! the four needs one. [`Shape::needs_a_lever`] is false four times: the mirror is a per-district
//! amount laid beside an [`crate::policy::Outcome`], the rolling count is an argument
//! [`crate::policy::apply`] already takes, the phase-down is
//! [`crate::policy::GuaranteeRule`] restricted to a set — a per-district rule, which is a module
//! and not a scalar — and doing nothing is [`crate::policy::Policy::current_law`]. This module is
//! the same answer [`crate::refresh`] gave to the same question.
//!
//! # What is not priced here
//!
//! **A window wider than three years.** The rolling count uses the department's own published
//! three-year average. A fourth year exists in `[M1a]` and is a **different count** — the two
//! FY2023 columns disagree in 608 of 611 districts by up to 4.6%, and splicing them reverses the
//! sign of the persistence [`crate::series::DEFAULT_DAMPING`] parameterises. A four-year average
//! is not available from this fixture at any price worth paying.
//!
//! **Any shape's incidence by wealth or poverty.** Four costs and their targeting are here. Who
//! else the money reaches — 383 eligible districts of which 216 are on the guarantee and 83 sit
//! on the minimum state share — is the next question and is not this one.
//!
//! **Community and STEM schools.** Every figure is over the 609 districts the FY2027 model
//! carries. H.B. 96 extends `[K]` to those schools too, so a statewide cost of anything touching
//! it is larger than anything computed here.

use edfund_core::{Adm, Dollars, FiscalYear};

use crate::hold_harmless::transition_supplement_under;
use crate::panel::{
    DistrictRecord, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL, ENROLLMENT_GROWTH_THRESHOLD,
};
use crate::policy::{apply, GuaranteeRule, Outcome, Policy, Statewide};
use crate::series::{project, Method, Prior};

/// The three-year change a district has to be past for the mirrored supplement to pay it.
///
/// `[M]`'s threshold with its sign reversed, and nothing else. A mirror that chose its own
/// threshold would be answering a different question than "what does symmetry cost".
pub const MIRROR_THRESHOLD: f64 = -ENROLLMENT_GROWTH_THRESHOLD;

/// Smallest movement counted as arriving: half a cent, as [`crate::refresh::MOVED`].
pub const MOVED: Dollars = 0.005;

/// Where a payment is written, which decides how much of it survives to the district.
///
/// Three placements exist in the FY2027 model and this enum carries the two a design would
/// choose between. The third — inside `[J] Transportation`, which `[K]` counts and the guarantee
/// does not — is not a candidate for an enrollment adjustment and is absorbed exactly as
/// [`Self::Foundation`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line {
    /// Inside `[H] Foundation Funding`. The guarantee absorbs it first and `[K]` absorbs the rest.
    Foundation,
    /// A new line beside `[L]`, `[M]` and `[O]` — outside `[L1]`'s subtrahend, so neither floor
    /// sees it.
    ///
    /// Outside *as the lines stand*. `[L1]` is a frozen FY2021 total and an act that re-based it
    /// on a total including this line would absorb the payment like any other.
    Beside,
}

impl Line {
    /// A short label for a table.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Foundation => "inside [H]",
            Self::Beside => "beside [L], [M], [O]",
        }
    }
}

/// The four shapes [`crate::enrollment_decline`] named and this module prices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// `[M]` with its sign reversed: a per-pupil rate on the whole roll past a threshold.
    Mirror,
    /// Pay the state share on the rolling multi-year count base cost is computed on.
    RollingCount,
    /// Retire the guarantee for the districts a named year found on it.
    DatedPhaseDown,
    /// Current law.
    Nothing,
}

impl Shape {
    /// A short label for a table.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mirror => "mirror [M]",
            Self::RollingCount => "rolling pupil count",
            Self::DatedPhaseDown => "dated phase-down",
            Self::Nothing => "nothing",
        }
    }

    /// What the shape pays for one more pupil lost, in words.
    ///
    /// Stated for every shape whether or not its cost is quantified, because a cost without it
    /// is half an answer: an adjustment for decline that pays per pupil lost is cheap and is a
    /// standing invitation, and one that pays on the whole roll is expensive and is not.
    #[must_use]
    pub const fn rewards_at_the_margin(self) -> &'static str {
        match self {
            Self::Mirror => {
                "crossing the threshold, and then nothing: an eligible district loses the \
                 per-pupil rate for every further pupil it loses, because the payment is on the \
                 whole roll"
            }
            Self::RollingCount => {
                "nothing. It re-times a loss rather than pricing it — the pupil is worth what it \
                 was and leaves the count more slowly"
            }
            Self::DatedPhaseDown => {
                "being on the floor before the date, which cannot be earned after it, and nothing \
                 at all thereafter"
            }
            Self::Nothing => {
                "crossing the floor, which is the behaviour already in force: a district whose \
                 formula amount falls below its FY2020 base is held there"
            }
        }
    }

    /// Whether pricing the shape needs a new [`Policy`] field.
    ///
    /// False for all four. Kept as a method rather than a comment because the question was asked
    /// of this work before it began and the answer is a property of each shape.
    #[must_use]
    pub const fn needs_a_lever(self) -> bool {
        false
    }
}

/// Whether the mirrored supplement's threshold reaches a district.
///
/// On `[M]`'s own published measure, [`crate::panel::Supplements::enrollment_change`], which is
/// read and never reconstructed: the department computes it from a FY2023 column that is not the
/// base cost window's FY2023.
#[must_use]
pub fn eligible(record: &DistrictRecord) -> bool {
    record.supplements.enrollment_change <= MIRROR_THRESHOLD
}

/// What the mirrored supplement writes for one district at a stated rate.
///
/// On the whole roll, as `[M]` pays, and on `[M]`'s own count —
/// [`DistrictRecord::categorical_enrolled_adm`], which reproduces the published `[M]` column
/// exactly where a growth district draws it.
#[must_use]
pub fn payment(record: &DistrictRecord, rate: Dollars) -> Dollars {
    if eligible(record) {
        record.categorical_enrolled_adm * rate
    } else {
        0.0
    }
}

/// What the mirrored supplement would write if it paid only on the pupils lost.
///
/// The marginal design, and the reason it is priced is to be able to refuse it: this is a
/// standing per-child payment for a child not enrolled. Over the district's own three-year
/// window, so that the rule and its measure agree.
#[must_use]
pub fn marginal_payment(record: &DistrictRecord, rate: Dollars) -> Dollars {
    if !eligible(record) {
        return 0.0;
    }
    (record.supplements.adm_fy23 - record.current_year_adm).max(0.0) * rate
}

/// What a payment arrives as, once the floors have taken what they take.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arrival {
    /// What the rule writes cheques for, before any floor.
    pub gross: Dollars,
    /// What arrives, in total state support, across the panel.
    pub statewide: Dollars,
    /// The part of that arriving in the enrollment cluster.
    pub cluster: Dollars,
    /// What the rule writes for the cluster, before any floor.
    pub cluster_gross: Dollars,
    /// Districts the rule writes a cheque for.
    pub paid: usize,
    /// Districts receiving the whole of it.
    pub in_full: usize,
    /// Districts receiving part.
    pub partly: usize,
    /// Districts receiving none of it.
    ///
    /// Counted rather than taken as a residual, so that "the three classes partition the paid
    /// districts" is an assertion and not an identity.
    pub not_at_all: usize,
}

impl Arrival {
    /// What the floors took.
    #[must_use]
    pub fn absorbed(&self) -> Dollars {
        self.gross - self.statewide
    }

    /// That, as a share of what the rule wrote.
    #[must_use]
    pub fn absorbed_share(&self) -> f64 {
        if self.gross.abs() <= MOVED {
            return 0.0;
        }
        self.absorbed() / self.gross
    }

    /// The same for the cluster alone, which is the figure a design is aimed at.
    #[must_use]
    pub fn cluster_absorbed_share(&self) -> f64 {
        if self.cluster_gross.abs() <= MOVED {
            return 0.0;
        }
        (self.cluster_gross - self.cluster) / self.cluster_gross
    }

    /// The share of what arrives that arrives in the cluster.
    ///
    /// The comparable figure across shapes: dollars delivered to the 89 per dollar of statewide
    /// cost.
    #[must_use]
    pub fn targeting(&self) -> f64 {
        if self.statewide.abs() <= MOVED {
            return 0.0;
        }
        self.cluster / self.statewide
    }
}

/// What one district actually receives of a payment, given where it is written.
///
/// `baseline` is the district's own [`Outcome`] at current law, from the same run, so that a
/// payment of nothing arrives as exactly nothing rather than as a rounding artefact — the trap
/// [`crate::hold_harmless::CENT`] records.
///
/// For [`Line::Beside`] the answer is the payment: no floor in the FY2027 model reads a line
/// outside `[L1]`'s subtrahend, which `tests/the_floors_the_guarantee_denominator_left_out.rs`
/// asserts to the cent. It is computed through the same ladder anyway, so that the two placements
/// differ in this function by where the money enters and in nothing else.
#[must_use]
pub fn arrives(
    record: &DistrictRecord,
    baseline: &Outcome,
    amount: Dollars,
    line: Line,
) -> Dollars {
    let (formula, beside) = match line {
        Line::Foundation => (baseline.formula_aid + amount, 0.0),
        Line::Beside => (baseline.formula_aid, amount),
    };
    let realized = formula.max(record.guarantee_floor());
    let supplement = transition_supplement_under(record, realized, baseline.transportation);
    let after = realized + baseline.transportation + supplement + beside;
    let before = baseline.realized_aid + baseline.transportation + baseline.transition_supplement;
    after - before
}

/// The cluster, as a membership test rather than a list.
fn membership(panel: &[DistrictRecord]) -> Vec<String> {
    crate::enrollment_decline::cluster(panel)
        .into_iter()
        .map(|record| record.irn.clone())
        .collect()
}

/// Price the mirrored supplement at a stated rate and placement.
#[must_use]
pub fn mirror(panel: &[DistrictRecord], rate: Dollars, line: Line) -> Arrival {
    let law = Policy::current_law();
    let statewide = Statewide::under(panel, &law);
    let cluster = membership(panel);

    let mut out = Arrival {
        gross: 0.0,
        statewide: 0.0,
        cluster: 0.0,
        cluster_gross: 0.0,
        paid: 0,
        in_full: 0,
        partly: 0,
        not_at_all: 0,
    };
    for record in panel {
        let amount = payment(record, rate);
        if amount <= MOVED {
            continue;
        }
        let baseline = apply(record, &law, &statewide, record.current_year_adm);
        let received = arrives(record, &baseline, amount, line);
        out.gross += amount;
        out.statewide += received;
        out.paid += 1;
        if cluster.contains(&record.irn) {
            out.cluster_gross += amount;
            out.cluster += received;
        }
        if received >= amount - MOVED {
            out.in_full += 1;
        } else if received <= MOVED {
            out.not_at_all += 1;
        } else {
            out.partly += 1;
        }
    }
    out
}

/// The mirrored supplement paid on the pupils lost rather than the whole roll.
///
/// Priced so that the cheaper design can be compared against the one to prefer, and always
/// [`Line::Beside`]: a marginal payment absorbed by a floor is neither design.
#[must_use]
pub fn marginal(panel: &[DistrictRecord], rate: Dollars) -> Arrival {
    let law = Policy::current_law();
    let statewide = Statewide::under(panel, &law);
    let cluster = membership(panel);

    let mut out = Arrival {
        gross: 0.0,
        statewide: 0.0,
        cluster: 0.0,
        cluster_gross: 0.0,
        paid: 0,
        in_full: 0,
        partly: 0,
        not_at_all: 0,
    };
    for record in panel {
        let amount = marginal_payment(record, rate);
        if amount <= MOVED {
            continue;
        }
        let baseline = apply(record, &law, &statewide, record.current_year_adm);
        let received = arrives(record, &baseline, amount, Line::Beside);
        out.gross += amount;
        out.statewide += received;
        out.paid += 1;
        out.in_full += 1;
        if cluster.contains(&record.irn) {
            out.cluster_gross += amount;
            out.cluster += received;
        }
    }
    out
}

/// The rate that would deliver the cluster's whole guarantee, and what it costs everyone else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Calibration {
    /// The cluster's guarantee, which the rate is solved against.
    pub target: Dollars,
    /// The per-pupil rate that delivers it, paid [`Line::Beside`].
    pub rate: Dollars,
    /// What that rate costs across the panel.
    pub statewide: Dollars,
}

impl Calibration {
    /// The statewide cost as a multiple of the floor being replaced.
    #[must_use]
    pub fn multiple(&self) -> f64 {
        if self.target <= 0.0 {
            return 0.0;
        }
        self.statewide / self.target
    }
}

/// Solve the mirrored supplement's rate against the cluster's guarantee.
///
/// Exact arithmetic rather than a search, and only because the payment is written
/// [`Line::Beside`]: outside every floor the cheque is linear in the rate, so the rate that
/// doubles the delivery doubles the cost. Inside `[H]` it is not linear — the guarantee is a
/// `max` — and there is no closed form.
#[must_use]
pub fn replacing_the_floor(panel: &[DistrictRecord]) -> Calibration {
    let at_reference = mirror(panel, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL, Line::Beside);
    let target: Dollars = crate::enrollment_decline::cluster(panel)
        .iter()
        .map(|record| record.guarantee)
        .sum();
    if at_reference.cluster <= MOVED {
        return Calibration {
            target,
            rate: 0.0,
            statewide: 0.0,
        };
    }
    let factor = target / at_reference.cluster;
    Calibration {
        target,
        rate: ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL * factor,
        statewide: at_reference.statewide * factor,
    }
}

/// The districts sitting just below the mirrored threshold, and what the cliff is worth to them.
#[derive(Debug, Clone, PartialEq)]
pub struct Cliff {
    /// How wide a band below the threshold was looked at, as a fraction.
    pub band: f64,
    /// Districts in it.
    pub districts: usize,
    /// What they would have drawn had they cleared the threshold.
    pub forgone: Dollars,
    /// The largest single forgone amount, and whose it is.
    pub largest: (String, Dollars),
}

/// Measure the cliff the mirrored supplement inherits from `[M]`.
///
/// `band` is how far below [`MIRROR_THRESHOLD`] to look, as a fraction — 0.003 for the three
/// tenths of a percentage point `[M]`'s own cliff was found on.
#[must_use]
pub fn cliff(panel: &[DistrictRecord], band: f64, rate: Dollars) -> Cliff {
    let mut out = Cliff {
        band,
        districts: 0,
        forgone: 0.0,
        largest: (String::new(), 0.0),
    };
    for record in panel {
        let change = record.supplements.enrollment_change;
        if change <= MIRROR_THRESHOLD || change > MIRROR_THRESHOLD + band {
            continue;
        }
        let amount = record.categorical_enrolled_adm * rate;
        out.districts += 1;
        out.forgone += amount;
        if amount > out.largest.1 {
            out.largest = (record.name.clone(), amount);
        }
    }
    out
}

/// What paying the state share on the rolling count does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rolling {
    /// Change in total state support across the panel.
    pub statewide: Dollars,
    /// The same on [`crate::biennium::Measure::FoundationAid`], for the comparison the constraint
    /// on this work exists to force.
    pub foundation: Dollars,
    /// The gross categorical repricing inside that, before any floor.
    ///
    /// Carried separately for the reason [`crate::refresh::Refreshed::categorical_delta`] is: the
    /// six categoricals are paid on their own count, and a reader who holds that only base cost's
    /// count should move can take this out. **Gross**, and therefore not a clean subtraction —
    /// where a floor binds, part of it never arrived to be removed.
    pub categorical: Dollars,
    /// The part of the statewide change arriving in the cluster.
    pub cluster: Dollars,
    /// Districts whose total state support moves at all.
    pub movers: usize,
    /// Districts the substitution carries off the guarantee onto the formula.
    pub lifted_off: usize,
    /// Of those, how many are in the cluster.
    pub cluster_lifted_off: usize,
}

impl Rolling {
    /// The share of what arrives that arrives in the cluster.
    #[must_use]
    pub fn targeting(&self) -> f64 {
        if self.statewide.abs() <= MOVED {
            return 0.0;
        }
        self.cluster / self.statewide
    }
}

/// Price the rolling pupil count across the panel.
///
/// The substitution is [`DistrictRecord::current_year_adm`] to
/// [`DistrictRecord::base_cost_adm`] — the department's own published three-year average, not a
/// mean of the `ADM Data` columns, which is a different number for 96 districts and by as much as
/// 448 pupils for one.
#[must_use]
pub fn rolling_count(panel: &[DistrictRecord]) -> Rolling {
    let law = Policy::current_law();
    let statewide = Statewide::under(panel, &law);
    let cluster = membership(panel);

    let mut out = Rolling {
        statewide: 0.0,
        foundation: 0.0,
        categorical: 0.0,
        cluster: 0.0,
        movers: 0,
        lifted_off: 0,
        cluster_lifted_off: 0,
    };
    for record in panel {
        if record.current_year_adm <= 0.0 {
            continue;
        }
        let before = apply(record, &law, &statewide, record.current_year_adm);
        let after = apply(record, &law, &statewide, record.base_cost_adm());
        let moved = after.total_state_support() - before.total_state_support();
        let ratio = record.base_cost_adm() / record.current_year_adm;
        let lifted = before.on_guarantee && !after.on_guarantee;

        out.statewide += moved;
        out.foundation += after.realized_aid - before.realized_aid;
        out.categorical += record.categorical_funding() * (ratio - 1.0);
        if moved.abs() > MOVED {
            out.movers += 1;
        }
        if lifted {
            out.lifted_off += 1;
        }
        if cluster.contains(&record.irn) {
            out.cluster += moved;
            if lifted {
                out.cluster_lifted_off += 1;
            }
        }
    }
    out
}

/// What a phase-down keyed to a year's roll actually retires.
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseDown {
    /// Cluster members the keying year found on the guarantee.
    pub dated: usize,
    /// Cluster members that arrived after it, whom the rule cannot reach.
    pub missed: usize,
    /// What the missed members hold.
    pub missed_dollars: Dollars,
    /// What the rule takes off foundation aid — the figure a run reports.
    pub reported: Dollars,
    /// What it takes off total state support, once `[K]` has risen to meet it.
    pub actual: Dollars,
    /// Districts in the dated set whose cheque does not move at all.
    pub made_whole: usize,
}

impl PhaseDown {
    /// The share of the reported saving that `[K]` puts back.
    #[must_use]
    pub fn absorbed_share(&self) -> f64 {
        if self.reported <= MOVED {
            return 0.0;
        }
        (self.reported - self.actual) / self.reported
    }
}

/// Price a phase-down of the cluster's guarantee, keyed to the FY2026 model's roll.
///
/// `rule` is a parameter for the reason [`crate::hold_harmless::retirement`] takes one: removal, a
/// half phase-out and a rebase reach the same districts and differ only in dollars.
///
/// Keyed districts are those the FY2026 model paid a guarantee, from [`crate::prior_model`]; the
/// rest of the cluster is left at current law, which is what a dated instrument does and what a
/// scalar lever cannot express.
#[must_use]
pub fn dated_phase_down(panel: &[DistrictRecord], rule: GuaranteeRule) -> PhaseDown {
    let law = Policy::current_law();
    let statewide = Statewide::under(panel, &law);
    let moved = Policy {
        guarantee: rule,
        ..law
    };
    let prior = crate::prior_model::by_irn();

    let mut out = PhaseDown {
        dated: 0,
        missed: 0,
        missed_dollars: 0.0,
        reported: 0.0,
        actual: 0.0,
        made_whole: 0,
    };
    for record in crate::enrollment_decline::cluster(panel) {
        let keyed = prior
            .get(&record.irn)
            .is_some_and(|year| year.guarantee > 0.0);
        if !keyed {
            out.missed += 1;
            out.missed_dollars += record.guarantee;
            continue;
        }
        out.dated += 1;
        let before = apply(record, &law, &statewide, record.current_year_adm);
        let after = apply(record, &moved, &statewide, record.current_year_adm);
        out.reported += before.realized_aid - after.realized_aid;
        let lost = before.total_state_support() - after.total_state_support();
        out.actual += lost;
        if lost <= MOVED {
            out.made_whole += 1;
        }
    }
    out
}

/// Whether `[K]` holds this district whole against any cut to `[H]`, `[I]` or `[J]`.
///
/// `[L1] >= [H2] + [J]`: the FY2021 total is at or above everything the guarantee and
/// transportation could be worth, so there is nothing a cut to them can take. A district can be
/// insulated without drawing `[K]` today, which is why this is a test on the bases rather than on
/// the published column.
#[must_use]
pub fn fully_backstopped(record: &DistrictRecord) -> bool {
    record.transition.fy21_funding_base
        >= record.transition.funding_base + record.transportation.total
}

/// Where the cluster arrives if nothing is done.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trajectory {
    /// The year projected to.
    pub fiscal_year: FiscalYear,
    /// Projected enrolled ADM across the cluster.
    pub adm: Adm,
    /// Formula aid at that enrollment.
    pub formula_aid: Dollars,
    /// The guarantee, `[I]`.
    pub guarantee: Dollars,
    /// The backstop, `[K]`.
    pub transition_supplement: Dollars,
    /// Realized aid, transportation and `[K]` together.
    pub total_state_support: Dollars,
    /// Cluster members the guarantee pays at that enrollment.
    pub on_the_floor: usize,
}

impl Trajectory {
    /// Total state support per projected pupil.
    #[must_use]
    pub fn per_pupil(&self) -> Dollars {
        if self.adm <= 0.0 {
            return 0.0;
        }
        self.total_state_support / self.adm
    }

    /// The share of what the cluster receives that arrives through a hold-harmless.
    ///
    /// The quantity the do-nothing case is actually about: not how much these districts are paid,
    /// which barely moves, but how much of it depends on `[I]` and `[K]` standing.
    #[must_use]
    pub fn held_share(&self) -> f64 {
        if self.total_state_support <= 0.0 {
            return 0.0;
        }
        (self.guarantee + self.transition_supplement) / self.total_state_support
    }
}

/// The cluster under current law at a projected enrollment.
///
/// `method` is carried per district through [`DistrictRecord::projection_method`], so a district
/// the F-33 survey does not reach falls back rather than shrinking toward a rate nobody measured.
/// At [`crate::series::DEFAULT_DAMPING`] the answer is a **lower bound** on the drift: damping
/// applies a fitted rate for about 1.43 year-equivalents however long the horizon, and this
/// cluster is the fastest-declining quarter of the state, which
/// `tests/the_trend_the_damping_discards.rs` measures as forecast about 8% high at five years.
#[must_use]
pub fn nothing(
    panel: &[DistrictRecord],
    through: FiscalYear,
    method: Method,
    prior: Prior,
) -> Trajectory {
    let law = Policy::current_law();
    let statewide = Statewide::under(panel, &law);

    let mut out = Trajectory {
        fiscal_year: through,
        adm: 0.0,
        formula_aid: 0.0,
        guarantee: 0.0,
        transition_supplement: 0.0,
        total_state_support: 0.0,
        on_the_floor: 0,
    };
    for record in crate::enrollment_decline::cluster(panel) {
        let series = project(
            &record.adm_observations(),
            through,
            record.projection_method(method),
            prior,
        );
        let Some(point) = series
            .iter()
            .find(|projection| projection.fiscal_year == through)
            .map(|projection| projection.point)
        else {
            continue;
        };
        let outcome = apply(record, &law, &statewide, point);
        out.adm += point;
        out.formula_aid += outcome.formula_aid;
        out.guarantee += outcome.guarantee;
        out.transition_supplement += outcome.transition_supplement;
        out.total_state_support += outcome.total_state_support();
        if outcome.on_guarantee {
            out.on_the_floor += 1;
        }
    }
    out
}
