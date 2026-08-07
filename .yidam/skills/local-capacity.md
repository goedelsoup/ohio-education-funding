---
name: local-capacity
description: Compute an agency's local wealth position and state share under either the charge-off mechanism or the FSFP capacity measure
---

# Calculator: local-capacity (stub)

**Computes.** An agency's deemed local contribution and the resulting state share, under either
mechanism: the [charge-off](../corpus/formula-component/charge-off-local-share.yml) for
FY1992-FY2021, or the
[FSFP local capacity measure](../corpus/formula-component/fsfp-local-capacity-measure.yml)
for FY2022 forward.

**Reads.** Both components; the
[charge-off millage parameter](../corpus/parameter/local-share-charge-off-millage.yml);
per-agency valuation and resident income series from `crates/`.

**Returns.** Local capacity figure, deemed local share, state share percentage — populating
[`metric/state-share-percentage`](../corpus/metric/state-share-percentage.yml).

## Both mechanisms, one calculator

Deliberate. The interesting question is not what either mechanism returns but how they differ
for the same agency, and holding them in one place forces the inputs to be aligned so the
comparison is real. A district that looked wealthy under a valuation-only charge-off and looks
poorer under a valuation-plus-income capacity measure is the case the Fair School Funding Plan
was built for, and this calculator is where that claim becomes a number.

Return both figures for any agency-year where both are computable, not just the one in force.

## Status

**FSFP side implemented** in [`crates/local-capacity/`](../../crates/local-capacity/src/lib.rs).
16 tests.

Six tests reproduce the department's published worked example line by line — capacity
valuation, capacity gross income, all three per-pupil terms, the income ratio, the sliding
capacity rate, and the $2,027.00 bottom line. `state_share` and `state_share_percentage`
implement R.C. 3317.017(B) and (C) including the 5% floor, and report whether the floor was
binding so that a wealth-neutrality analysis can exclude floored districts rather than
silently flattening its own signal.

**Not implemented: the charge-off side.** The pre-FY2022 mechanism has no code, so the
cross-regime comparison this skill was specified to support cannot yet be run. That needs
[`local-share-charge-off-millage`](../corpus/parameter/local-share-charge-off-millage.yml)
populated first — its series is still `[open]`.
