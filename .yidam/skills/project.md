---
name: project
description: Forward-project per-agency enrollment, assessed valuation, and revenue series with uncertainty intervals, as inputs to scenario runs
---

# Calculator: project (stub)

**Computes.** Forward series for the inputs a funding simulation consumes — average daily
membership, assessed valuation, and local revenue — per agency, with intervals.

**Reads.** Historical per-agency series from `crates/`;
[`education-agency`](../corpus/education-agency/) nodes for structural facts that constrain a
projection, such as 20-mill floor status.

**Returns.** Projected series with explicit intervals and a record of the method used.

## The weaker half of prediction, and it must say so

Simulation and projection are not the same epistemic act. Re-running a formula with a changed
parameter is deterministic: given the inputs, the answer is the answer. Projecting enrollment
five years out is a forecast, with everything that implies.

A [`scenario`](../corpus/scenario/) that mixes the two must report them separately, so that the
policy effect is not contaminated by forecast error. The seeded scenario in this corpus does
exactly that — its FY2026 leg uses observed inputs and its FY2027 leg requires projection, and
the node instructs that the two be reported apart.

Ohio-specific constraints the method must respect: valuation moves in reappraisal steps on a
county cycle rather than smoothly; enrollment is declining in most of the state and rising in a
few suburban counties, so a statewide trend applied uniformly is wrong everywhere; and revenue
projection depends on floor status, which is itself a projected quantity.

## Status

**Implemented** in [`crates/project`](../../crates/project/). Enrollment projection with
intervals, a policy simulator over it, and — since the outcome axis arrived — the district
crosswalk and the funding-to-report-card join that every joined statement in this corpus is
now computed on.

The earlier reading — that this belonged outside `crates/` because the mature time-series
tooling is Python — no longer holds: `packages/` has been retired and the whole domain computer
is Rust. It was also the wrong reason. What the projection needed was not a library but a
history, and it still does.

### The rule above is enforced by the types

A [`Run`](../../crates/project/src/report.rs) holds a `PolicyEffect` with no interval and an
optional `EnrollmentEffect` with one. There is no field containing their sum, and the CLI prints
them under separate headings with a sentence saying why.

The [platform](../../web/) now enforces the same rule structurally rather than by wording: the
simulation and the forecast are separate cards, gated on separate checks, and the forecast is
always below. The bundle contract carries `adm_history` per district and a `projection` block
with `ForecastCheckpoint`s, and the browser must reproduce every one **at both ends of the band**
before it draws it — a page that reproduced the point and got the interval wrong would look
correct. The design answer that unblocked it was to make the range the headline number and demote
the point to a footnote; see [web/README.md](../../web/README.md).

The corpus's own forecast — total state aid at FY2032 of **$7,125M in a band of $6,889M to
$7,379M** under current law, widening to **±5.9%** with the guarantee removed — is now reproduced
by a third implementation, in TypeScript, from the feed. [verified]

### The Ohio-specific constraints, and how each fared

**Valuation moves in reappraisal steps on a county cycle.** Not implemented, and not because it
is hard: the corpus has exactly one valuation observation per district. `Method::Assumed` lets a
caller supply a rate and records it as the caller's assumption. Blocked on
[`tax-abstract`](../../crates/connect/sources/tax-abstract.md). Local capacity is 60% valuation,
so **every projection of the local side of Ohio school funding is currently an assumption**.

**A statewide enrollment trend applied uniformly is wrong everywhere.** Respected — every
district is projected from its own three observations, and 105 of 609 are growing.

**Revenue projection depends on floor status, which is itself projected.** Not reached. Revenue
is not projected at all; only enrollment is, and only aid is computed from it.

### What is still missing

A longer enrollment history. Three observations per district is enough to fit a trend and not
enough to estimate how wrong it might be, so the intervals rest on a cross-sectional prior
rather than on each district's own variability. The `nces-ccd` connector is where a real panel
would come from — and it is also what would handle consolidation, without which a long series
is silently wrong.
