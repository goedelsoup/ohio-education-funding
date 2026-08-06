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

**Stub — not implemented.** Blocked on `tax-abstract` for valuation, `census-f33` or an
income source for the income component, and on establishing the FSFP capacity weighting, which
is `[open]` in the component node. Implementation lands in `crates/local-capacity/`.
