//! Who each priced decline response actually pays: the districts the floor caught, or the
//! districts that lost pupils.
//!
//! Every pricing of a decline response in this arc has been against the enrollment cluster's
//! **89** — [`crate::rolling_anchor`]'s anchor rules, [`crate::decline_adjustment`]'s four
//! shapes, [`crate::anchor_incidence`]'s cut of them by wealth and poverty. But
//! [`crate::lost_pupils`] found the 89 are the third wealth fifth of a larger population, of
//! which **64 siblings** are formula districts the floor does not hold and which are poorer than
//! the cluster. This module reads the pricings that already exist over that second population.
//! Nothing here is computed twice: [`anchor_incidence::under_supplement`] and
//! [`anchor_incidence::between_walks`] produce the per-district arms, [`crate::lost_pupils::siblings`]
//! and [`crate::size_incidence`] produce the populations, and [`over`] is a filter and a sum.
//!
//! # 1. The seven instruments fall into three families, and two of them are disjoint
//!
//! [`over`] priced on the 64 and on the 89 ([`SHAPES`] names the frame each row runs):
//!
//! | instrument | statewide | of the 64 | to the 64 | per pupil | of the 89 | to the 89 | per pupil |
//! |---|--:|--:|--:|--:|--:|--:|--:|
//! | mirror beside `[L]`,`[M]`,`[O]` | $181.3m | 60 | $19,692,849 | $241.95 | 81 | $46,020,644 | $233.76 |
//! | mirror inside `[H]` | $66.4m | 59 | $19,112,357 | $234.82 | 14 | $4,195,897 | $21.31 |
//! | rolling pupil count | $70.1m | 60 | $22,359,204 | $274.71 | 29 | $3,902,794 | $19.82 |
//! | dated phase-down | −$0.7m | **0** | $0 | $0.00 | 7 | −$695,296 | −$3.53 |
//! | ratchet, FY2032 | $52.9m | 59 | $14,794,631 | $186.81 | **0** | $0 | $0.00 |
//! | 2% cap, FY2032 | −$63.3m | 2 | −$18,947 | −$0.24 | 18 | −$3,949,559 | −$20.61 |
//! | 1% cap, FY2036 undamped | $7.7m | 40 | $22,526,491 | $334.67 | 13 | −$2,590,697 | −$15.86 |
//!
//! The two zeros are the finding, and neither is a rounding. A **ratchet** — hold each district
//! at its own prior year — moves 59 of the 64 and **not one of the 89**, because the enacted
//! guarantee already holds all 89 and a second floor under a floor pays nothing. A **dated
//! phase-down** — roll the FY2027 guarantee list down on a schedule — touches 7 of the 89 and
//! **not one of the 64**, because the 64 are not on that list and the date is why. The two
//! populations the issue asks about are not merely different; the two most sharply drawn
//! instruments are each blind to one of them.
//!
//! The mirror and the rolling count are the family that reaches both, and they reach both for
//! one reason: their eligibility test is *lost pupils*, which both populations pass. So the
//! design question does not decompose into "the floor's districts or the declining ones". It
//! decomposes into what the rule is keyed to:
//!
//! - keyed to **a list on a date** — the dated phase-down: 7 of the 89, 0 of the 64;
//! - keyed to **aid that falls** — the ratchet and the caps: 0 to 18 of the 89, 2 to 59 of
//!   the 64;
//! - keyed to **pupils that leave** — the mirror and the rolling count: 81 and 29 of the 89,
//!   60 of the 64.
//!
//! Only the third family is a decline response in the sense the arc has been arguing about. The
//! first two are a guarantee policy and an aid-path policy that happen to move declining
//! districts.
//!
//! Which of the two populations a decline response is *for* is a design question and this module
//! does not answer it. What it establishes is the shape of the choice. The 89 are held, poorer
//! than the state and larger; the 64 are not held, poorer than the 89, and small — and the 64
//! are where the marginal dollar changes a district's position rather than being absorbed. A
//! rule keyed to the floor's list is a rule about the first group only, and no amount of
//! generosity in it reaches the second. The difference between the two populations is wealth,
//! and a rule keyed to a date decides that difference silently.
//!
//! # 2. The 64 district by district, before any fifth
//!
//! The siblings are small districts and a band's per-pupil figure hides them, so [`per_district`]
//! is the first reading. Under the rolling pupil count the largest sibling payment is **Maple
//! Heights City's $1,780,414** on 2,612 pupils; the next two are Mansfield City and Euclid City;
//! **four of the 64** move by less than a cent — St Bernard-Elmwood Place City, McComb Local,
//! Western Reserve Local and Liberty Local. The order is not district size, because that shape
//! recomputes the formula on a rolling count and pays most where the roll fell fastest.
//!
//! Under the mirror the order *is* district size, and that is worth saying because it is easy to
//! read the shape as paying for lost pupils. It does not:
//! [`decline_adjustment::payment`] is the enrolled ADM of an eligible district times the rate, so
//! an eligible district is paid **$250 on every pupil it still has**, and **56 of the 64** receive
//! exactly that. The decline test is a gate, not a quantity.
//!
//! # 3. The placement question is a question about the 89, not about the 64
//!
//! `formula-component/temporary-transitional-aid-guarantee` established that the same mirror is
//! worth $46.0m beside the floors and $4.2m inside `[H]`, an order of magnitude on where the line
//! sits. On the siblings the same choice is worth **$19,692,849 against $19,112,357** —
//! [`absorption`] puts the absorbed share at **0.0295** on the 64 against **0.9088** on the 89,
//! and **67 of the 81** cluster districts paid inside `[H]` receive nothing at all while **4 of
//! 60** siblings lose anything. A debate about `[L1]`'s subtrahend is a debate about districts
//! the floor already holds.
//!
//! # 4. Of the progressive gradient, roughly a third is the siblings — and it survives them
//!
//! [`split_by`] cuts [`anchor_incidence::banded`]'s own bands into the population and everyone
//! else, so a band here is a band there by construction. The rolling count by valuation per
//! pupil, the cluster excluded as [`anchor_incidence::by`] excludes it:
//!
//! | wealth fifth | 1 | 2 | 3 | 4 | 5 |
//! |---|--:|--:|--:|--:|--:|
//! | the band, per pupil | $166.55 | $83.44 | $35.67 | $10.04 | $11.43 |
//! | siblings in it | 22 | 17 | 11 | 6 | 7 |
//! | siblings, per pupil | $452.18 | $250.78 | $173.08 | $73.76 | $90.73 |
//! | everyone else, per pupil | **$119.29** | $64.63 | $27.14 | $8.06 | $8.76 |
//! | the siblings' share of the band | 0.3855 | 0.3036 | 0.2836 | 0.2215 | 0.2585 |
//!
//! Of the least-wealthy fifth's $166.55 a pupil, **$13,135,541 of $34,075,370** is the 64. So the
//! answer to "is the gradient this population" is no, in the strong form: take the siblings out
//! and the band still pays **$119.29** against the wealthiest fifth's $8.76, a ratio of 13.6
//! where the ratio with them is 14.6. The siblings raise the poor end; they do not make it.
//!
//! The mirror inside `[H]` is the cleaner case, because there the siblings are paid **flat** —
//! $244.06, $247.35, $222.89, $189.68, $242.77 across the five fifths, which is the $250 rate
//! less the districts it does not reach and the little it absorbs — while everyone else runs
//! $92.73, $66.50, $39.36, $8.57, $12.93. The whole gradient of that shape is a property of the
//! districts around the siblings, not of the siblings.
//!
//! # 5. A shape paid inside `[H]` is self-extinguishing for them, and the date arrives first
//!
//! [`crate::margin::headroom`] gives 62 of the 64 a clock; the median is **4.14 years**, 31
//! arrive within four and 40 within six. [`by_headroom`] splits the 64 on that clock and reads
//! the inside-`[H]` mirror's absorbed share off each half: **0.0760** on the nearer half,
//! **0.0000** on the further. That is the mechanism stated as a measurement — the
//! shape pays a district until the district reaches the floor, and then the floor takes the
//! payment back. The 89 are what the 64 become, and the 89's absorbed share is 0.9088.
//!
//! So a mirror inside `[H]` is self-extinguishing for the siblings over roughly the horizon
//! their own decline sets, and a mirror beside the floors is not: [`absorption`] on `[L]`,
//! `[M]`, `[O]` is 0.0000 for both populations and stays there by construction, since those are
//! the three lines outside `[L1]`'s subtrahend.
//!
//! The date is the sharper half of the answer. R.C. 3317.019 anchors the guarantee at FY2027,
//! and the median sibling reaches its own floor about **FY2031** — after it. [`on_the_floor`]
//! walks the panel and counts: at the shipped damping **5 of 64** are on the floor by FY2032 and
//! still 5 by FY2036; undamped, **38 of 64** by FY2036. Either way a shape keyed to the FY2027
//! list cannot reach a district that joins the list in FY2031, which is why the dated phase-down
//! pays them nothing in the table above and would pay them nothing in any year.
//!
//! # 6. The 153 and the 169 price differently, and both are reported
//!
//! [`crate::size_incidence::cluster_and_siblings`] is the **union** of the two populations, 153
//! districts; [`crate::size_incidence::shrinking_at_the_cluster_rate`] is what the phrase
//! "districts shrinking at the cluster's rate" actually names, and it is **169**. Neither
//! contains the other. The mirror beside the floors is worth $65,713,493 over the union at
//! $236.16 a pupil and $69,915,878 over the rate-defined population at $243.96; the 2% cap costs
//! the union $14.65 a pupil and the rate-defined population $57.61, which is nearly four times
//! as much. The distinction is load-bearing here and not only where it was first drawn.

use std::collections::BTreeSet;

use edfund_core::{Adm, Dollars, FiscalYear};

use crate::anchor_incidence::{self, Axis, Movement, Supplement};
use crate::decline_adjustment::{self, Line};
use crate::lost_pupils::Standing;
use crate::margin;
use crate::panel::{DistrictRecord, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL};
use crate::policy::{self, Policy, Statewide};
use crate::rolling_anchor::{self, Anchor};
use crate::series::Method;

/// A district whose delta is smaller than this is not moved by the instrument.
///
/// The same half-cent [`anchor_incidence`] counts gainers and losers at, so a district counted
/// here and a district counted there are counted alike.
const MOVED: Dollars = 0.005;

/// The four supplement shapes, in the order the tables in this module and in
/// `formula-component/temporary-transitional-aid-guarantee` print them.
pub const SHAPES: [Supplement; 4] = [
    Supplement::Mirror(Line::Beside),
    Supplement::Mirror(Line::Foundation),
    Supplement::RollingCount,
    Supplement::DatedPhaseDown,
];

/// The IRNs of a population, as [`over`] and [`split_by`] want it.
pub fn irns(rows: &[&Standing]) -> BTreeSet<String> {
    rows.iter().map(|row| row.irn.clone()).collect()
}

/// An anchor rule's per-district arm, against the rule in force.
///
/// [`rolling_anchor::walk`] under [`Anchor::Fixed`] is the enacted path; `anchor` is the
/// alternative; [`anchor_incidence::between_walks`] differences them. The year and method are the
/// caller's because the answer depends on both: a ratchet at FY2032 and a cap at FY2036 are two
/// frames, not one series.
pub fn against_the_enacted_anchor(
    panel: &[DistrictRecord],
    through: FiscalYear,
    method: Method,
    anchor: Anchor,
) -> Vec<Movement> {
    let enacted = rolling_anchor::walk(panel, through, method, Anchor::Fixed);
    let alternative = rolling_anchor::walk(panel, through, method, anchor);
    anchor_incidence::between_walks(panel, &enacted, &alternative)
}

/// What an instrument does to one population.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reach {
    /// Districts of the population the frame carries at all.
    pub districts: usize,
    /// Of them, the ones whose aid moves by more than half a cent.
    pub moved: usize,
    /// Of those, the ones that gain.
    pub gainers: usize,
    /// And the ones that lose.
    pub losers: usize,
    /// Districts the alternative puts on the floor that the enacted rule does not.
    pub enters: usize,
    /// The population's aid under the alternative less its aid as enacted, in dollars.
    pub delta: Dollars,
    /// The population's ADM, as the frame counts it.
    pub adm: Adm,
}

impl Reach {
    /// The delta over the population's own ADM.
    ///
    /// Zero on an empty population rather than a NaN, because a shape that reaches nobody is a
    /// row in a table and not an error.
    pub fn per_pupil(&self) -> Dollars {
        if self.adm > 0.0 {
            self.delta / self.adm
        } else {
            0.0
        }
    }

    /// This population's dollars as a share of another's — the whole frame's, usually.
    pub fn share_of(&self, whole: &Reach) -> f64 {
        if whole.delta.abs() > 0.0 {
            self.delta / whole.delta
        } else {
            0.0
        }
    }
}

/// Sum a frame of movements over a population.
///
/// Pass an empty set for the statewide row: no IRN is excluded and every movement counts.
pub fn over(movements: &[Movement], population: &BTreeSet<String>) -> Reach {
    let mut out = Reach {
        districts: 0,
        moved: 0,
        gainers: 0,
        losers: 0,
        enters: 0,
        delta: 0.0,
        adm: 0.0,
    };
    let inside = |movement: &Movement| population.is_empty() || population.contains(&movement.irn);
    for movement in movements.iter().filter(|movement| inside(movement)) {
        out.districts += 1;
        out.delta += movement.delta();
        out.adm += movement.adm;
        if movement.delta() > MOVED {
            out.moved += 1;
            out.gainers += 1;
        } else if movement.delta() < -MOVED {
            out.moved += 1;
            out.losers += 1;
        }
        if movement.enters() {
            out.enters += 1;
        }
    }
    out
}

/// One of [`anchor_incidence::by`]'s bands, cut into a population and everyone else.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Split {
    /// 1 for the band lowest on the axis.
    pub rank: usize,
    /// Districts in the band.
    pub districts: usize,
    /// Of them, the ones in the population.
    pub inside: usize,
    /// The band's dollars.
    pub delta: Dollars,
    /// The population's share of them.
    pub inside_delta: Dollars,
    /// The band's ADM.
    pub adm: Adm,
    /// The population's share of it.
    pub inside_adm: Adm,
}

impl Split {
    /// The band's dollars over the band's ADM — [`anchor_incidence::Band::per_pupil`]'s figure.
    pub fn per_pupil(&self) -> Dollars {
        if self.adm > 0.0 {
            self.delta / self.adm
        } else {
            0.0
        }
    }

    /// The same for the population inside the band.
    pub fn inside_per_pupil(&self) -> Dollars {
        if self.inside_adm > 0.0 {
            self.inside_delta / self.inside_adm
        } else {
            0.0
        }
    }

    /// The same for the band with the population taken out, which is the figure that says
    /// whether a gradient is the population or the band.
    pub fn outside_per_pupil(&self) -> Dollars {
        let adm = self.adm - self.inside_adm;
        if adm > 0.0 {
            (self.delta - self.inside_delta) / adm
        } else {
            0.0
        }
    }

    /// The population's share of the band's dollars.
    pub fn inside_share(&self) -> f64 {
        if self.delta.abs() > 0.0 {
            self.inside_delta / self.delta
        } else {
            0.0
        }
    }
}

/// Cut a frame into bands on an axis and each band into a population and everyone else.
///
/// The bands are [`anchor_incidence::banded`]'s, which [`anchor_incidence::by`] is also written
/// over, so `exclude` means what it means there — the cluster, for every figure in this module —
/// and a district excluded from the banding is in no [`Split`] either.
pub fn split_by(
    movements: &[Movement],
    axis: Axis,
    bands: usize,
    exclude: &BTreeSet<String>,
    population: &BTreeSet<String>,
) -> Vec<Split> {
    anchor_incidence::banded(movements, axis, bands, exclude)
        .into_iter()
        .enumerate()
        .map(|(index, band)| {
            let mut out = Split {
                rank: index + 1,
                districts: band.len(),
                inside: 0,
                delta: 0.0,
                inside_delta: 0.0,
                adm: 0.0,
                inside_adm: 0.0,
            };
            for movement in band {
                out.delta += movement.delta();
                out.adm += movement.adm;
                if population.contains(&movement.irn) {
                    out.inside += 1;
                    out.inside_delta += movement.delta();
                    out.inside_adm += movement.adm;
                }
            }
            out
        })
        .collect()
}

/// A population's own districts under one instrument, largest movement first.
///
/// The issue this module closes asks for the per-district arm before the per-fifth one, because
/// the siblings are small districts and a fifth's per-pupil figure hides them.
pub fn per_district<'a>(
    movements: &'a [Movement],
    population: &BTreeSet<String>,
) -> Vec<&'a Movement> {
    let mut out: Vec<&Movement> = movements
        .iter()
        .filter(|movement| population.contains(&movement.irn))
        .collect();
    out.sort_by(|a, b| b.delta().total_cmp(&a.delta()));
    out
}

/// What a supplement is worth gross against what arrives, for one population on one line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Absorption {
    /// Districts of the population the supplement is payable to at all.
    pub paid: usize,
    /// What it is worth before the floors see it.
    pub gross: Dollars,
    /// What arrives after them.
    pub received: Dollars,
    /// Districts that receive less than the gross.
    pub reduced: usize,
    /// Districts that receive nothing at all.
    pub extinguished: usize,
}

impl Absorption {
    /// The share of the gross the floors take back.
    pub fn absorbed_share(&self) -> f64 {
        if self.gross > 0.0 {
            1.0 - self.received / self.gross
        } else {
            0.0
        }
    }
}

/// Price [`decline_adjustment::payment`] on one line for one population.
///
/// The gross is the same on both lines; `line` decides only what
/// [`decline_adjustment::arrives`] lets through. [`Line::Beside`] is `[L]`, `[M]` and `[O]`, the
/// three lines outside `[L1]`'s subtrahend, so its absorbed share is zero by construction and is
/// reported rather than assumed.
pub fn absorption(
    panel: &[DistrictRecord],
    line: Line,
    population: &BTreeSet<String>,
) -> Absorption {
    let law = Policy::current_law();
    let statewide = Statewide::under(panel, &law);
    let mut out = Absorption {
        paid: 0,
        gross: 0.0,
        received: 0.0,
        reduced: 0,
        extinguished: 0,
    };
    for record in panel.iter().filter(|r| population.contains(&r.irn)) {
        let amount = decline_adjustment::payment(record, ENROLLMENT_GROWTH_SUPPLEMENT_PER_PUPIL);
        if amount <= MOVED {
            continue;
        }
        let baseline = policy::apply(record, &law, &statewide, record.current_year_adm);
        let arrives = decline_adjustment::arrives(record, &baseline, amount, line);
        out.paid += 1;
        out.gross += amount;
        out.received += arrives;
        if amount - arrives > MOVED {
            out.reduced += 1;
        }
        if arrives <= MOVED {
            out.extinguished += 1;
        }
    }
    out
}

/// The same population split at the median years-to-the-floor, nearest half first.
///
/// A district with no clock — [`margin::headroom`] returns `None` for one whose roll is not
/// falling — sorts to the far end, which is where it belongs: the floor never reaches it.
pub fn by_headroom(
    panel: &[DistrictRecord],
    line: Line,
    population: &BTreeSet<String>,
) -> (Absorption, Absorption) {
    let clocks = margin::headroom(panel, &Policy::current_law());
    let mut ordered: Vec<(f64, &str)> = population
        .iter()
        .map(|irn| {
            let years = clocks
                .iter()
                .find(|clock| &clock.irn == irn)
                .and_then(|clock| clock.years)
                .unwrap_or(f64::INFINITY);
            (years, irn.as_str())
        })
        .collect();
    ordered.sort_by(|a, b| a.0.total_cmp(&b.0));
    let half = ordered.len() / 2;
    let side = |slice: &[(f64, &str)]| {
        let set: BTreeSet<String> = slice.iter().map(|(_, irn)| (*irn).to_string()).collect();
        absorption(panel, line, &set)
    };
    (side(&ordered[..half]), side(&ordered[half..]))
}

/// How long a population has before its own guarantee floor reaches it.
#[derive(Debug, Clone, PartialEq)]
pub struct Clock {
    /// Districts of the population whose roll is falling, so the floor does reach them.
    pub falling: usize,
    /// The rest, which it never does.
    pub never: usize,
    /// The upper-middle of the falling districts' years, as the crates take a median.
    pub median_years: f64,
    /// How many arrive within four years, and within six.
    pub within: (usize, usize),
}

/// Read [`margin::headroom`]'s clock over a population.
pub fn clock(panel: &[DistrictRecord], population: &BTreeSet<String>) -> Clock {
    let clocks = margin::headroom(panel, &Policy::current_law());
    let mut years: Vec<f64> = clocks
        .iter()
        .filter(|clock| population.contains(&clock.irn))
        .filter_map(|clock| clock.years)
        .collect();
    years.sort_by(f64::total_cmp);
    Clock {
        falling: years.len(),
        never: population.len() - years.len(),
        median_years: years.get(years.len() / 2).copied().unwrap_or(0.0),
        within: (
            years.iter().filter(|y| **y <= 4.0).count(),
            years.iter().filter(|y| **y <= 6.0).count(),
        ),
    }
}

/// How many of a population the enacted rule has put on the floor by a given year.
///
/// The walk is [`Anchor::Fixed`] — the rule in force — so this is the arrival the date question
/// turns on and not a counterfactual.
pub fn on_the_floor(
    panel: &[DistrictRecord],
    through: FiscalYear,
    method: Method,
    population: &BTreeSet<String>,
) -> usize {
    let walk = rolling_anchor::walk(panel, through, method, Anchor::Fixed);
    population
        .iter()
        .filter(|irn| {
            walk.get(*irn)
                .is_some_and(rolling_anchor::Standing::on_the_floor)
        })
        .count()
}
