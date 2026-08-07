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

**Stub — not implemented.** The earlier reading, that this belonged outside `crates/` because
the mature time-series tooling is Python, no longer holds: `packages/` has been retired and the
whole domain computer is Rust. Blocked on the historical series, not on the language.
