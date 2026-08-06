---
name: regime-diff
description: Difference two funding regimes at component level for the same agency-year, attributing the change to specific mechanisms rather than reporting a net total
---

# Calculator: regime-diff (stub)

**Computes.** For one agency and one fiscal period, what each of two
[`funding-regime`](../corpus/funding-regime/) would pay, decomposed to the
[`formula-component`](../corpus/formula-component/) level.

**Reads.** Both regimes' components and parameters; the agency's input series.

**Returns.** Per component: the amount under each regime and the difference. Plus a residual
line for anything the decomposition does not account for — if the residual is large, the
component mapping between the two regimes is wrong and the result should not be trusted.

## Why component-level

A district that gained under the Fair School Funding Plan gained through a specific mechanism.
It might be the district-specific
[base cost](../corpus/formula-component/fsfp-base-cost-calculation.yml), or the shift from
charge-off to [local capacity](../corpus/formula-component/fsfp-local-capacity-measure.yml), or
the end of the community school deduct. These have different policy implications and different
durability, and a single net delta conflates them.

The `replaces` edge between components is what makes the mapping possible. Where no `replaces`
edge exists, the components are not comparable and the calculator should say so rather than
guess an alignment.

## Status

**Stub — not implemented.** Depends on `foundation` supporting both regimes. Implementation
lands in `crates/regime-diff/`.

The first useful run is the Bridge formula against the Fair School Funding Plan for FY2022 —
the transition year, where both are computable and the political argument about winners and
losers actually happened.
