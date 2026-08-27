//! Export a versioned JSON feed of the corpus's district-level findings.
//!
//! [`web/`](../../../web/) consumes a bundled export rather than reading
//! [`.yidam/corpus/`](../../../.yidam/corpus/) directly. The corpus is markdown and YAML written
//! for traversal by people and agents; the platform needs numbers for 609 districts. This
//! crate is the seam between them.
//!
//! # Contract version
//!
//! The bundle carries [`CONTRACT_VERSION`]. A consumer that does not recognise it should
//! refuse to render rather than guess, because a field silently changing meaning is worse than
//! a page that does not load. Bump it on any change to field names or units.
//!
//! # Checkpoints, and why a duplicated implementation is acceptable here
//!
//! The scenario builder in the web layer re-derives what `project::policy::apply` computes, in
//! TypeScript, so that moving a slider does not require a round trip. Two implementations of
//! the same formula is normally a bad trade: they drift, and the one nobody runs is the one
//! that is wrong.
//!
//! [`Checkpoint`] is the answer. The bundle carries Rust-computed results for a set of named
//! policies, and the page verifies its own arithmetic against them **before** it will render a
//! scenario. If the two disagree the page says so and disables the tab. The duplication is then
//! load-bearing in only one direction: the Rust is authoritative and the TypeScript has to prove
//! it agrees, on every page load, against the real 609-district panel.
//!
//! # Why hand-rolled JSON
//!
//! The workspace has no external dependencies, deliberately — a committed
//! [`scenario`](../../../.yidam/corpus/scenario/) result should be reproducible years from now
//! without a dependency resolution succeeding first. Serializing a fixed, known schema is a
//! few dozen lines, so that constraint costs nothing here.

#![forbid(unsafe_code)]

mod json;

pub mod build;

use edfund_core::Dollars;

/// The bundle schema version. Bump on any change to field names, units, or semantics.
///
/// `38.0.0` makes the five `finances` figures nullable — `state_aid`, `local_tax`,
/// `total_revenue`, `total_expenditure`, `ending_cash` — in both the per-district and the
/// statewide arrays. A five-year forecast filing may carry a district's year and not every line
/// of it, and the extractor wrote the absence as `0`. Toronto City (IRN 044917) published $0 of
/// expenditure and $0 of cash against $9.86M of revenue for FY2020-FY2022 on that reading.
///
/// Breaking rather than additive for the same reason `35.0.0` was: the old values parsed and
/// plotted. A consumer summing the array, ranking on `ending_cash`, or dividing by
/// `total_expenditure` got a number, and the number was wrong. `null` is a reported absence and
/// must be dropped from an aggregate rather than added to it. The statewide totals are unchanged
/// — they already excluded what was never reported, by adding zero.
///
/// `37.0.0` added `casino`: nine fiscal years of the gross casino revenue county student fund, per
/// district and statewide. Breaking rather than additive because of what it does to every other
/// per-district figure in the feed — this is money the district receives that **no** figure above
/// it counts, so a consumer that already presented `realized_aid_per_pupil` or a `finances` row as
/// "what the state sends this district" was making a claim the feed can now contradict. The block
/// carries no per-pupil figure on purpose: it is apportioned on a fifth pupil count, defined by
/// R.C. 5753.11 and shared with nothing else here, so dividing it by any ADM in this feed produces
/// a number that looks comparable and is not.
///
/// `36.0.0` added `drafts`, the provisions of each `draft-legislation` node — including the ones
/// no lever can run, which travel with the rest so the count a cost must be read beside cannot be
/// dropped between the fixture and the page.
///
/// `35.0.0` puts the three report-card shares on the same scale as every other share the bundle
/// publishes. `outcome.economically_disadvantaged`, `outcome.english_learner` and
/// `outcome.students_with_disabilities` were passed through as the report card writes them —
/// **0 to 100** — while `District::economically_disadvantaged`, `dpia.percentage`,
/// `regime.recognized_share`, `transportation.effective_state_share` and every `national` share
/// are fractions. Two fields named `economically_disadvantaged` in one document, 100× apart, both
/// typed `Option<f64>` and neither saying which it was.
///
/// Breaking in the quietest and worst way available: a consumer reading the old values gets a
/// plausible number that is wrong by two orders of magnitude, and a percentage rendered through a
/// helper expecting a fraction reads `10000%` rather than failing. Nothing in this workspace
/// computed on them — they are a passthrough into the bundle, and the partial correlations control
/// on the *profile* report's share, which was always a fraction — so the change is to the
/// published units and to nothing else.
///
/// `34.0.0` and `33.0.0` extend `appropriations` back to **FY1998** and adds a third value to its `source`
/// field, `act`. Breaking on the enum: a consumer switching on `workbook` or `catalog` now meets a
/// third case. The four new years are read from the enrolled acts rather than from any analysis of
/// them, because the greenbook series begins with the 124th General Assembly and the Catalog
/// reaches FY2002. They stop at FY1998 because the legislature's own index of what it holds stops
/// at the 122nd.
///
/// The same revision corrects every enacted total from FY2002 to FY2013. `200906 Tangible Tax
/// Exemption - Education` is a tax reimbursement and was not excluded, so it sat inside the
/// department: $73,500,000 in FY2002, 0.94%, decaying to $10,707,622 by FY2009. Because it phases
/// out it inflated the early years more than the late ones, understating real growth.
///
/// `32.0.0` reworked `meal_program`. It now runs FY1998 through FY2014 rather than FY2001 through
/// FY2011, and `share` became **nullable**. Breaking in the strongest sense available here: a
/// consumer that reads the block the way `29.0.0` published it will find a null where it expects a
/// number, which is the intent. From FY2012 the report is published as three files — Traditional,
/// Provision 2 and Community Eligibility — and only the first still counts applications. Adding
/// them gives a share that falls thirteen points in three years because the poorest sponsors
/// stopped collecting forms, so those Octobers carry `floor` and `ceiling` and no share at all.
/// `streams` says which kind of year a row is.
///
/// The same revision corrects FY2001. The file is the only comma-delimited one in the series, nine
/// of its rows carry a comma inside a school name, and read positionally two of them put a site
/// IRN in Cleveland City's enrollment column — 192,147 pupils against its real 73,562. Every
/// figure `29.0.0` and later published for FY2001 was computed on that: the share was 27.7% and is
/// 29.5%.
///
/// `31.0.0` added `appropriation_lines`: the department's line items with the act that created
/// each, from the Catalog's `originally established by` clause. Additive in shape and breaking in
/// meaning, because it changes what a consumer can say about the budget — not how much it is but
/// how old it is, and roughly half the lines answer "unknown" rather than being filled from an
/// earlier edition that reused their number.
///
/// `30.0.0` added `appropriations`: what the General Assembly set aside for the department, by
/// fiscal year, FY2002 through FY2027, continuous for the first time. Breaking rather than
/// additive because it changes what the feed *is* in the same way `28.0.0` did — every other
/// figure here is an output of the funding system, and this is an input to it. An appropriation is
/// a ceiling and a payment is what was made; differencing the two produces a number that means
/// nothing, and the deflator now reaches FY2002 so the series can be read in constant dollars,
/// which is the only way it can be read at all.
///
/// `29.0.0` added `meal_program`: the free and reduced-price lunch share for FY2001 through
/// FY2011, from the Office for Child Nutrition's MR-81. Additive in shape and breaking in
/// meaning, on the same reasoning as `28.0.0` — this is now the third population and the third
/// enrollment count in one feed, and it is the only one whose denominator changes *inside* its
/// own series, at FY2010. Every row carries [`MealProgramYear::basis`] so a consumer cannot plot
/// the series as continuous without having been told it is not. It carries no dollars and is
/// absent from the deflator by design.
///
/// `28.0.0` added the historical axis: a `history` array carrying the Census Bureau's survey of
/// Ohio school systems for FY2009 through FY2022, and a `deflator` extended to cover it. Breaking
/// rather than additive because it changes what the feed *is*. Every other figure here is the Fair
/// School Funding Plan computing one year for the 609 traditional districts; these are roughly 950
/// agencies a year measured by somebody else on a different enrollment count, and the two do not
/// reconcile. A consumer that rendered them on one axis without saying which was which would be
/// making the mistake the block exists to let it avoid.
///
/// `9.0.0` wired the [`millage`] crate in. Breaking because [`District::at_millage_floor`]
/// changes answer for 21 districts: it compared the effective Class I rate to a literal `20.0`,
/// so a district charging *less* than twenty mills was reported as being above the floor with
/// reduction factors operative, which is the opposite of its position. The feed also gains
/// [`District::voted_operating_millage`] — a column that was in the profile CSV from the start
/// and never parsed — and the [`MillageAnalysis`] block computed from it.
///
/// `7.0.0` and `8.0.0` added the base cost build-up, Table SD-1 and spending by function.
///
/// `6.0.0` added the price index and the statewide financial aggregates. Breaking because the
/// feed now carries figures a consumer can deflate, and a page that shows the FY2020-FY2025
/// panel in nominal dollars is not merely imprecise — across a span in which CPI-U rose 25.1%,
/// a nominal statement about it can have the wrong sign.
///
/// `5.0.0` added the actuals: a `finances` array per district carrying six closed fiscal years
/// of what it received, raised, spent, and held. Additive in shape but breaking in meaning — it
/// is the first per-district figure in the feed that is a record rather than a model, and a
/// consumer that rendered it beside the FY2027 calculator's output without saying which was
/// which would present a measurement and a projection as the same kind of claim.
///
/// `4.0.0` added the projection axis: `adm_history` on every district, so the page can carry
/// enrollment forward itself, and a `projection` block holding the forecast's method, its prior,
/// and [`ForecastCheckpoint`]s the page must reproduce before it may draw a band. Breaking rather
/// than additive because `adm_history` is required, not nullable — a district without it cannot
/// be projected, and a feed that omitted it would produce a page silently missing half its
/// panel.
///
/// `3.0.0` added the outcome axis: a nullable `outcome` object per district carrying the
/// Performance Index, the Progress effect size, need shares, and spending on both denominators,
/// plus the statewide correlations that say how to read them. Nullable because three districts
/// have no report card — see [`project::crosswalk`].
///
/// `2.0.0` added the scenario inputs and checkpoints, and renamed the enrollment-change years
/// from FY2022-FY2024 to FY2024-FY2026 — the years the department's `ADM Data` sheet declares.
/// The values did not change; what they are called did, which is exactly the kind of silent
/// meaning change the version guard exists for.
pub const CONTRACT_VERSION: &str = "39.0.0";

mod model;
mod serialize;

pub use model::*;
