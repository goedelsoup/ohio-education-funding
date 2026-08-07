---
name: foundation
description: Re-run a named Ohio funding regime for a fiscal period against a parameter set, returning per-agency aid broken out by component
---

# Calculator: foundation (stub)

The simulation engine. Everything else in the prediction half of this repository depends on it.

**Computes.** Per-agency state foundation aid under a named
[`funding-regime`](../corpus/funding-regime/), for a given
[`fiscal-period`](../corpus/fiscal-period/), against a supplied parameter set — broken out by
[`formula-component`](../corpus/formula-component/) rather than returned as a single total.

**Reads.** The regime's `composed-of` components; each component's `governed-by`
[parameters](../corpus/parameter/) at their values for the period; per-agency enrollment and
valuation series from `crates/`.

**Returns.** For each agency: computed cost, deemed local share, computed state aid,
post-phase-in state aid, and the component-level breakdown. Plus a flag for whether the agency
is on formula or held on a cap or guarantee — without that flag the output is misleading for
every off-formula district.

## Contract

Pure and deterministic. No network, no clock. Given the same parameter set and input series it
must return the same result, because [`scenario`](../corpus/scenario/) nodes commit its output
as a knowledge claim and a non-reproducible claim is not one.

The parameter set is an explicit argument, not read from ambient state. That is what makes a
counterfactual expressible: a scenario is this calculator invoked twice with two parameter sets.

## Status

**Partially implemented** in [`crates/foundation/`](../../crates/foundation/src/lib.rs).
14 tests.

Teacher base cost [R.C. 3317.011(D)] is complete and verified: six tests reproduce the
department's published worked example — funded classroom teachers 851.52, special teachers
135.61, and all four cost components summing to $87,984,148.77. Five of the seven student
support elements [3317.011(E)] are implemented.

**There is deliberately no `aggregate_base_cost` function.** District leadership,
building leadership, and athletic co-curricular are not implemented, so any total would be
wrong by roughly a third while looking authoritative — and would propagate into every state
share and scenario downstream. The crate exposes what is verified and nothing that pretends to
be more.

One implementation detail is load-bearing and has its own test: funded teacher counts are
rounded to two decimals **before** multiplication, as the department does. Keeping full
precision instead moves the special teacher component by hundreds of dollars on a mid-sized
district and no longer matches the published figure.

A further test asserts the property the seeded scenario depends on — refreshing the salary
inputs changes the price terms without moving staffing at all.
