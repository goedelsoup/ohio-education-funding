//! A bill that is not law, and what it would cost.
//!
//! The runner behind the [`draft-legislation`](../../../.yidam/corpus/draft-legislation/) class.
//! A draft is a list of provisions; some of them bind to a lever in [`crate::policy`] and can be
//! priced, and the rest cannot. This module exists to make the second group impossible to lose.
//!
//! # There is no method returning a bare cost
//!
//! [`price`] is the only way to obtain a figure, and it returns a [`Priced`] which cannot be
//! constructed without the provisions it failed to price. That is deliberate, and it is the same
//! device `scenario_delta::Aggregate` uses one level down: a total there cannot be built without
//! the count of districts it fails to reach, and a total here cannot be built without the count of
//! provisions it fails to cost.
//!
//! The failure it guards against is the *easy* one rather than an exotic one. The provisions that
//! bind to levers are exactly the provisions that produce a number, so a reader — or an author
//! working quickly — who prices what prices and reports the sum has described a different bill
//! from the one in front of them, and nothing in the arithmetic says so.
//!
//! # Provisions do not add, and the residual is reported rather than absorbed
//!
//! [`Priced::attribution`] runs each provision alone against current law, and
//! [`Priced::residual`] is the difference between their sum and the combined run. It is routinely
//! large. Raising base cost costs the state more per point the higher it goes, because each
//! increment lifts districts off the guarantee onto a formula that then has to pay them, so a
//! draft that raises base cost *and* touches the guarantee is worth materially less than its two
//! halves priced separately.
//!
//! Reporting it follows `regime_diff`, which returns the residual its decomposition cannot
//! explain rather than distributing it across the terms that can be.
//!
//! # The fixture is authoritative
//!
//! `fixtures/draft-provisions.tsv` is what runs; the corpus node is what a reader reads. The two
//! are held together by `tests/a_draft_cannot_hide_what_it_did_not_price.rs`, on the pattern
//! `web/tests/unit/year.spec.ts` already uses for statewide constants — the computed side wins,
//! and the prose may not contradict it.

use std::collections::BTreeMap;

use edfund_core::Dollars;

use crate::panel::DistrictRecord;
use crate::policy::{GuaranteeRule, Policy};
use crate::report::{simulate, PolicyEffect};

/// The committed extract: one row per provision per draft.
const FIXTURE: &str = include_str!("../fixtures/draft-provisions.tsv");

const EXPECTED_HEADER: &str =
    "draft\tordinal\ttitle\tauthority\tparameter\tlever\tbaseline\tproposed\tnote";

/// The columns of [`EXPECTED_HEADER`], named where they are read.
///
/// Tab-delimited, not comma: a provision's title and note are prose.
mod column {
    pub const DRAFT: usize = 0;
    pub const ORDINAL: usize = 1;
    pub const TITLE: usize = 2;
    pub const AUTHORITY: usize = 3;
    pub const PARAMETER: usize = 4;
    pub const LEVER: usize = 5;
    pub const BASELINE: usize = 6;
    pub const PROPOSED: usize = 7;
    pub const NOTE: usize = 8;
}

/// One lever a provision moves, already parsed.
///
/// The five variants are the five fields of [`Policy`] and there is no sixth, which is the whole
/// constraint this module operates under. A provision naming anything else is unpriced.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lever {
    /// What happens to the temporary transitional aid guarantee.
    Guarantee(GuaranteeRule),
    /// Multiplier on aggregate base cost.
    BaseCostScale(f64),
    /// The minimum state share of base cost.
    MinimumStateShare(f64),
    /// How far the district moves from its FY2020 funding base toward the computed
    /// amount, for every core foundation component except DPIA.
    PhaseInGeneral(f64),
    /// The same interpolation for DPIA, against its own FY2019 base.
    PhaseInDpia(f64),
}

impl Lever {
    /// Read a lever from a provision's `lever` and `proposed` columns.
    ///
    /// # Errors
    ///
    /// If the lever key is not one of the five, or its value does not parse.
    pub fn parse(key: &str, proposed: &str) -> Result<Self, String> {
        let number = || {
            proposed
                .parse::<f64>()
                .map_err(|_| format!("{key} wants a number, got {proposed:?}"))
        };
        match key {
            "guarantee" => Ok(Self::Guarantee(GuaranteeRule::parse(proposed)?)),
            "base-cost" => Ok(Self::BaseCostScale(number()?)),
            "min-share" => Ok(Self::MinimumStateShare(number()?)),
            "phase-in" => Ok(Self::PhaseInGeneral(number()?)),
            "phase-in-dpia" => Ok(Self::PhaseInDpia(number()?)),
            other => Err(format!(
                "unknown lever {other:?}; the five are guarantee, base-cost, min-share, \
                 phase-in, phase-in-dpia"
            )),
        }
    }

    /// The key this lever is written as, in the fixture and in the site's query string.
    ///
    /// The inverse of [`Lever::parse`], and it exists so the vocabulary is written down once. The
    /// five strings are load-bearing in three places — the committed fixture, the feed, and the
    /// `?g=&base=&min=&pb=&pc=` the scenario page reads — and a second hand-written mapping is how
    /// two of those would come to disagree.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Guarantee(_) => "guarantee",
            Self::BaseCostScale(_) => "base-cost",
            Self::MinimumStateShare(_) => "min-share",
            Self::PhaseInGeneral(_) => "phase-in",
            Self::PhaseInDpia(_) => "phase-in-dpia",
        }
    }

    /// Apply this lever to a policy, leaving the other four where they were.
    #[must_use]
    pub const fn applied_to(self, policy: Policy) -> Policy {
        match self {
            Self::Guarantee(rule) => Policy {
                guarantee: rule,
                ..policy
            },
            Self::BaseCostScale(scale) => Policy {
                base_cost_scale: scale,
                ..policy
            },
            Self::MinimumStateShare(share) => Policy {
                minimum_state_share: share,
                ..policy
            },
            Self::PhaseInGeneral(fraction) => Policy {
                phase_in_general: fraction,
                ..policy
            },
            Self::PhaseInDpia(fraction) => Policy {
                phase_in_dpia: fraction,
                ..policy
            },
        }
    }
}

/// One clause of a draft: a change it would make, and whether anything here can price it.
#[derive(Debug, Clone, PartialEq)]
pub struct Provision {
    /// The draft this belongs to, as the corpus node is slugged.
    pub draft: String,
    /// Its position in the draft, one-based.
    pub ordinal: u16,
    /// What it does, in one line.
    pub title: String,
    /// The Revised Code section it would amend, or `uncodified`.
    pub authority: String,
    /// The corpus `parameter` node it binds, where one exists.
    pub parameter: String,
    /// The lever it moves. `None` is the whole point of this type: it means unpriced.
    pub lever: Option<Lever>,
    /// The value in force, for the readable statement.
    pub baseline: String,
    /// The value the provision would put there.
    pub proposed: String,
    /// Why it does not price, or what the run is sized against where it does.
    pub note: String,
}

impl Provision {
    /// Whether any lever in [`Policy`] expresses this provision.
    ///
    /// Derived from `lever` rather than declared in a column of its own. A separate `runnable`
    /// field could disagree with the binding beside it, and the registry in `connect` had already
    /// established that a status a test derives outlives a status a README asserts.
    #[must_use]
    pub const fn is_priced(&self) -> bool {
        self.lever.is_some()
    }
}

/// A bill that is not law: its provisions, in the order it states them.
#[derive(Debug, Clone, PartialEq)]
pub struct Draft {
    /// The slug, which is also the corpus node's filename.
    pub slug: String,
    /// Every provision, priced and unpriced alike.
    pub provisions: Vec<Provision>,
}

impl Draft {
    /// The combined policy: every priced provision applied to current law at once.
    ///
    /// This and not the sum of the parts is the draft's policy. Two provisions applied together
    /// are not two provisions applied twice, because the guarantee is a `max` and a district can
    /// only cross it once.
    #[must_use]
    pub fn policy(&self) -> Policy {
        self.provisions
            .iter()
            .filter_map(|p| p.lever)
            .fold(Policy::current_law(), |policy, lever| {
                lever.applied_to(policy)
            })
    }

    /// The provisions no lever reaches.
    #[must_use]
    pub fn unpriced(&self) -> Vec<&Provision> {
        self.provisions.iter().filter(|p| !p.is_priced()).collect()
    }

    /// The provisions a lever does reach.
    #[must_use]
    pub fn priced(&self) -> Vec<&Provision> {
        self.provisions.iter().filter(|p| p.is_priced()).collect()
    }
}

/// What one provision costs on its own, against current law.
#[derive(Debug, Clone, PartialEq)]
pub struct Attribution {
    /// The provision's position in the draft.
    pub ordinal: u16,
    /// What it does, in one line.
    pub title: String,
    /// Its cost run alone. Not a share of the combined figure — see [`Priced::residual`].
    pub cost: Dollars,
}

/// A draft's cost, inseparable from what the figure leaves out.
///
/// Constructible only by [`price`], which always computes both halves. There is no way to obtain
/// the dollar figure without also holding [`Priced::unpriced`], and that is the type's only
/// reason for existing.
#[derive(Debug, Clone)]
pub struct Priced {
    /// The slug of the draft this prices.
    pub slug: String,
    combined: PolicyEffect,
    attribution: Vec<Attribution>,
    unpriced: Vec<Provision>,
}

impl Priced {
    /// What the draft costs the state, in the dollars of the model's own year. `None` when no
    /// provision priced.
    ///
    /// Safe to expose as a number here and nowhere else: a caller holding a [`Priced`] is already
    /// holding [`Priced::unpriced`], so the count it must be reported beside cannot have been
    /// dropped on the way.
    ///
    /// # `None` rather than zero, and the difference is the whole point
    ///
    /// A draft every one of whose provisions falls outside the model produces a combined policy
    /// identical to current law, so the arithmetic gives `0.00` — and `0.00` in a cost column
    /// reads as *this bill is free*, which is the opposite of *this bill was not priced*. The
    /// first version of this type returned a bare `Dollars` and would have published the first
    /// reading for [`hb-643-136-introduced`](../../../.yidam/corpus/draft-legislation/), a real
    /// pending bill whose entire effect is in the scholarship channel this workspace does not
    /// carry.
    ///
    /// That is not a hypothetical shape. It is the ordinary one for a bill about vouchers,
    /// transportation, or facilities, and a type that could not distinguish it from a costless
    /// bill would be wrong about most of what the General Assembly passes.
    #[must_use]
    pub fn cost(&self) -> Option<Dollars> {
        if self.attribution.is_empty() {
            return None;
        }
        Some(self.combined.cost())
    }

    /// The full effect of the combined run — districts, guarantee, the whole table.
    #[must_use]
    pub const fn effect(&self) -> &PolicyEffect {
        &self.combined
    }

    /// Each priced provision run alone against current law.
    #[must_use]
    pub fn attribution(&self) -> &[Attribution] {
        &self.attribution
    }

    /// The provisions this figure does not include.
    #[must_use]
    pub fn unpriced(&self) -> &[Provision] {
        &self.unpriced
    }

    /// How many provisions the draft has, priced and unpriced.
    #[must_use]
    pub fn provisions(&self) -> usize {
        self.attribution.len() + self.unpriced.len()
    }

    /// The combined cost less the sum of the provisions priced alone.
    ///
    /// A finding rather than an error. It is the interaction between levers, and for a draft that
    /// both raises base cost and lowers the guarantee it is large and **positive**: the combined
    /// run is less of a cut than the two priced apart and added, because pricing them apart
    /// double-counts the districts the refresh lifts off the floor and the phase-out would then
    /// have cut. On the fixture's two-lever draft the parts say -$219.0M, the whole says -$143.9M,
    /// and the residual is +$75.1M.
    ///
    /// Zero for a single-provision draft by construction, which is the check on the arithmetic,
    /// and `None` for a draft with nothing priced, where there is nothing for the parts to fail to
    /// sum to.
    #[must_use]
    pub fn residual(&self) -> Option<Dollars> {
        Some(self.cost()? - self.attribution.iter().map(|a| a.cost).sum::<Dollars>())
    }
}

/// Price a draft: the combined run, each provision alone, and everything left over.
///
/// The only constructor of [`Priced`], so the unpriced list cannot be skipped.
#[must_use]
pub fn price(draft: &Draft, panel: &[DistrictRecord]) -> Priced {
    let attribution = draft
        .provisions
        .iter()
        .filter_map(|provision| {
            let lever = provision.lever?;
            let alone = simulate(panel, &lever.applied_to(Policy::current_law()));
            Some(Attribution {
                ordinal: provision.ordinal,
                title: provision.title.clone(),
                cost: alone.cost(),
            })
        })
        .collect();
    Priced {
        slug: draft.slug.clone(),
        combined: simulate(panel, &draft.policy()),
        attribution,
        unpriced: draft.unpriced().into_iter().cloned().collect(),
    }
}

/// Every draft in the committed fixture, keyed by slug.
///
/// # Panics
///
/// If the fixture's header is not the one this was written against, or a row names a lever that
/// does not exist. Both are authoring errors in a committed file rather than conditions a caller
/// can recover from, and a draft that silently loses a provision is the failure this whole module
/// is built to prevent — so it fails loudly instead of returning a shorter bill.
#[must_use]
pub fn drafts() -> BTreeMap<String, Draft> {
    // The row-width assertion this used to carry itself now lives in the reader, which checks
    // every row against the header. What stays here is the part that is this module's own: a
    // provision naming a lever that does not exist is unpriced, and must not load quietly.
    let mut out: BTreeMap<String, Draft> = BTreeMap::new();
    for row in edfund_core::csv::delimited(FIXTURE, EXPECTED_HEADER, '\t') {
        let slug = row.str(column::DRAFT).to_string();
        let title = row.str(column::TITLE).to_string();
        let ordinal: u16 = row.str(column::ORDINAL).parse().unwrap_or_else(|_| {
            panic!("a draft provision needs a numeric ordinal: {slug} {title:?}")
        });
        let key = row.str(column::LEVER);
        let lever = if key.is_empty() {
            None
        } else {
            Some(
                Lever::parse(key, row.str(column::PROPOSED))
                    .unwrap_or_else(|e| panic!("{slug} provision {ordinal}: {e}")),
            )
        };
        out.entry(slug.clone())
            .or_insert_with(|| Draft {
                slug: slug.clone(),
                provisions: Vec::new(),
            })
            .provisions
            .push(Provision {
                draft: slug,
                ordinal,
                title,
                authority: row.str(column::AUTHORITY).to_string(),
                parameter: row.str(column::PARAMETER).to_string(),
                lever,
                baseline: row.str(column::BASELINE).to_string(),
                proposed: row.str(column::PROPOSED).to_string(),
                note: row.str(column::NOTE).to_string(),
            });
    }
    for draft in out.values_mut() {
        draft.provisions.sort_by_key(|p| p.ordinal);
    }
    out
}

/// One draft by slug.
#[must_use]
pub fn draft(slug: &str) -> Option<Draft> {
    drafts().remove(slug)
}
