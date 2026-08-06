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

**Stub — not implemented.** Blocked on two things: the parameter value series in
[`parameter/`](../corpus/parameter/) are all `[open]`, and the per-agency input series do not
exist until `dew-foundation` and `tax-abstract` are built. Implementation lands in
`crates/foundation/`.

Implement the Fair School Funding Plan first — it is the only regime with a live policy
question attached — then work backward.
