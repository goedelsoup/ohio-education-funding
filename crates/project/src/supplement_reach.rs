//! Whether the performance supplement moves any district's position, which is the question
//! `formula-component/fsfp-performance-supplement` measured up to and stopped at.
//!
//! # The question
//!
//! That node establishes the supplement is distributed inversely to need — $55.92 per pupil to the
//! least-poor fifth of districts against $21.84 to the poorest, monotonically — and then declines
//! to draw the obvious conclusion, on a ground worth quoting: "$55.7m against $7.28bn of state aid
//! is 0.8%, so a district's whole supplement is smaller than the noise in most other lines — and a
//! regressive 0.8% inside a progressive $7.28bn is a different object from a regressive program.
//! The gradient is measured; whether it moves any district's position is not."
//!
//! It moves them, barely, and in the direction the gradient predicts.
//!
//! # Rank
//!
//! Order Ohio's 609 districts by total state support per pupil and take the supplement away.
//! **370 of them change place** — and the mean move is **1.07 places**, the largest in the state
//! is **8**, and **four districts cross a quintile boundary**. See [`rank_moves`] and
//! [`quintile_crossings`].
//!
//! # Dispersion, and the sign that is easy to get backwards
//!
//! The coefficient of variation of state support per pupil is **0.5242** with the supplement and
//! **0.5281** without it. The supplement *narrows* the spread of state aid — and that is the
//! regressive reading rather than a refutation of it. State aid is dispersed on purpose: Ohio's
//! poorest quartile of districts receives $10,265 per pupil and its least-poor $4,851, and that
//! gap is the equalisation. A payment that adds proportionally more at the bottom of the aid
//! distribution than at the top compresses the gap, which is what "inversely to need" means when
//! need and state aid run opposite. See [`dispersion()`].
//!
//! The size is what the node was asking about. The compression is **0.0040 of a coefficient of
//! variation, 0.75% of the dispersion** — and in relative terms the supplement is worth 1.14% of
//! the least-poor quartile's state aid against 0.22% of the poorest's, a ratio of five to one
//! where the per-pupil ratio is 2.56. See [`by_poverty_quartile`].
//!
//! # Why it can move a position at all
//!
//! The supplement is paid on line `[O]`, outside `[H] Foundation Funding`, and the guarantee holds
//! a district at a base computed from `[H]`. So a guaranteed district receives the supplement **on
//! top** rather than having it absorbed — the one payment in the formula that is not cancelled for
//! a district the guarantee is carrying. **294 districts are on the guarantee and 217 of them are
//! paid a supplement**, so this is most of the guaranteed half of the state rather than an edge
//! case. See [`paid_on_the_guarantee`]. Being outside the guarantee is what the node records as
//! the supplement's exposure; it is also the only purchase on a district's position it has.

use std::collections::BTreeMap;

use crate::panel::{self, DistrictRecord};

/// How many parts the poverty and aid distributions are cut into here.
pub const BANDS: usize = 4;

/// One district's state support per pupil, with the supplement and without it.
#[derive(Debug, Clone, PartialEq)]
pub struct Shift {
    /// Information Retrieval Number.
    pub irn: String,
    /// The district's name in the department's model.
    pub name: String,
    /// Total state support per pupil as the model publishes it.
    pub with: f64,
    /// The same with the performance supplement removed.
    pub without: f64,
    /// The district's economically disadvantaged share, as the DPIA percentage.
    pub poverty: f64,
}

impl Shift {
    /// What the supplement is worth to this district, per pupil.
    #[must_use]
    pub fn supplement(&self) -> f64 {
        self.with - self.without
    }
}

/// Every district the model carries both an aid total and an enrolment for.
#[must_use]
pub fn frame() -> Vec<Shift> {
    panel::panel()
        .iter()
        .filter_map(|record: &DistrictRecord| {
            let adm = record.categorical_enrolled_adm;
            (adm > 0.0 && record.total_state_support > 0.0).then(|| Shift {
                irn: record.irn.clone(),
                name: record.name.clone(),
                with: record.total_state_support / adm,
                without: (record.total_state_support - record.performance.amount) / adm,
                poverty: record.dpia.percentage,
            })
        })
        .collect()
}

/// Positions on one measure, highest first, indexed as the frame is.
fn positions(frame: &[Shift], pick: fn(&Shift) -> f64) -> Vec<usize> {
    let mut order: Vec<usize> = (0..frame.len()).collect();
    order.sort_by(|a, b| pick(&frame[*b]).total_cmp(&pick(&frame[*a])));
    let mut out = vec![0; frame.len()];
    for (place, index) in order.into_iter().enumerate() {
        out[index] = place;
    }
    out
}

/// How far the supplement moves districts in the order of state support per pupil.
///
/// Returned as `(districts that move, mean absolute move, largest move)`. The mean is over every
/// district and not only the movers, which is the figure that answers "does this reorder Ohio".
#[must_use]
pub fn rank_moves() -> (usize, f64, i64) {
    let frame = frame();
    if frame.is_empty() {
        return (0, 0.0, 0);
    }
    let with = positions(&frame, |s| s.with);
    let without = positions(&frame, |s| s.without);

    let moves: Vec<i64> = (0..frame.len())
        .map(|index| {
            i64::try_from(with[index]).unwrap_or(0) - i64::try_from(without[index]).unwrap_or(0)
        })
        .collect();
    let changed = moves.iter().filter(|move_| **move_ != 0).count();
    #[allow(clippy::cast_precision_loss)]
    let mean = moves.iter().map(|move_| move_.abs()).sum::<i64>() as f64 / moves.len() as f64;
    let largest = moves.iter().map(|move_| move_.abs()).max().unwrap_or(0);
    (changed, mean, largest)
}

/// The districts the supplement carries across a quintile boundary of state support per pupil.
///
/// Named rather than counted, because four names out of 609 is the answer and a count is not.
#[must_use]
pub fn quintile_crossings() -> Vec<String> {
    let frame = frame();
    let with = positions(&frame, |s| s.with);
    let without = positions(&frame, |s| s.without);
    let band = |place: usize| place * 5 / frame.len().max(1);

    (0..frame.len())
        .filter(|index| band(with[*index]) != band(without[*index]))
        .map(|index| frame[index].name.clone())
        .collect()
}

/// The coefficient of variation of state support per pupil, `(with, without)`.
///
/// The supplement narrows it. State aid is dispersed deliberately — toward need — so narrowing it
/// is the regressive reading rather than an equalising one.
#[must_use]
pub fn dispersion() -> (f64, f64) {
    let frame = frame();
    let cv = |pick: fn(&Shift) -> f64| {
        let values: Vec<f64> = frame.iter().map(pick).collect();
        if values.is_empty() {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        if mean == 0.0 {
            return 0.0;
        }
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
        variance.sqrt() / mean
    };
    (cv(|s| s.with), cv(|s| s.without))
}

/// Mean state support per pupil by quartile of district poverty, poorest last.
///
/// Each entry is `(with the supplement, without it)`. The pair says what the supplement is worth
/// against the aid a band already receives, which is the comparison the per-pupil gradient alone
/// cannot make: $55 on $4,851 and $22 on $10,265 are a wider gap than 2.56 to one.
///
/// # Panics
///
/// If the frame holds fewer than [`BANDS`] districts, which means the panel moved.
#[must_use]
pub fn by_poverty_quartile() -> [(f64, f64); BANDS] {
    let mut frame = frame();
    assert!(frame.len() >= BANDS, "the frame is too small to band");
    frame.sort_by(|a, b| a.poverty.total_cmp(&b.poverty));

    let mut out = [(0.0, 0.0); BANDS];
    for (index, slot) in out.iter_mut().enumerate() {
        let start = index * frame.len() / BANDS;
        let end = (index + 1) * frame.len() / BANDS;
        let band = &frame[start..end];
        #[allow(clippy::cast_precision_loss)]
        let n = band.len() as f64;
        *slot = (
            band.iter().map(|s| s.with).sum::<f64>() / n,
            band.iter().map(|s| s.without).sum::<f64>() / n,
        );
    }
    out
}

/// What the supplement is worth as a share of the state aid each poverty quartile already gets.
///
/// Poorest last, so a falling series is the gradient stated in the units that decide whether it
/// compresses the aid distribution.
#[must_use]
pub fn relative_to_aid() -> [f64; BANDS] {
    let mut out = [0.0; BANDS];
    for (slot, (with, without)) in out.iter_mut().zip(by_poverty_quartile()) {
        *slot = if with > 0.0 {
            (with - without) / with
        } else {
            0.0
        };
    }
    out
}

/// Districts on the guarantee, and how many of them the supplement still pays.
///
/// Returned as `(on the guarantee, of those, paid a supplement)`. The guarantee holds a district
/// at a base computed from `[H]` foundation funding; the supplement is line `[O]`, outside it. So
/// a guaranteed district receives the supplement **on top** rather than having it absorbed — which
/// is the one place in the formula where a payment to a guaranteed district is not cancelled by
/// the guarantee, and the reason the supplement can move a district's aid position at all.
#[must_use]
pub fn paid_on_the_guarantee() -> (usize, usize) {
    let panel = panel::panel();
    let guaranteed: Vec<&DistrictRecord> = panel
        .iter()
        .filter(|record| record.on_guarantee())
        .collect();
    let paid = guaranteed
        .iter()
        .filter(|record| record.performance.amount > 0.0)
        .count();
    (guaranteed.len(), paid)
}

/// The frame keyed by IRN, for a caller that wants one district.
#[must_use]
pub fn by_irn() -> BTreeMap<String, Shift> {
    frame()
        .into_iter()
        .map(|shift| (shift.irn.clone(), shift))
        .collect()
}
