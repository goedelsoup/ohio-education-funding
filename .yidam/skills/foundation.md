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

**Complete** in [`crates/foundation/`](../../crates/foundation/src/lib.rs). 18 tests.

All five statutory sub-components and all 22 elements are implemented: teacher
[3317.011(D)], student support [(E)], district leadership and accountability [(F)], building
leadership and operation [(G)], and athletic co-curricular [(H)].

**`aggregate_base_cost` now exists**, and the end-to-end test is the strong one: the five
sub-components sum to the published aggregate of **$148,960,701.95**, a figure that also
appears independently on the department's state share report. That verifies the components
against each other rather than each against its own screenshot.

Two structural features the implementation surfaced, neither of which is obvious from the
statute text:

- **Administrator salaries are derived, not looked up.** Only the superintendent has an explicit
  salary band; other district administrators are priced at 82.8% of it and building leaders at
  79.38%. A change to the superintendent band moves four elements. There is a test for this.
- **The superintendent band is a ramp.** $160,000 above 4,000 ADM, $80,000 below 500, linearly
  interpolated between — the only place in base cost where a price varies with district size.
  Tested for continuity at both thresholds.

Also load-bearing and separately tested: funded counts round to two decimals **before**
multiplication, and building operation subtracts the safety per-pupil amount so safety is not
funded twice across sub-components.

Blocked on nothing for a single district. To run across all 606 it needs per-district
grade-band enrollment and open-building counts, which the FY2024 District Profile Report does
not carry.
