# formula-component

The individual calculation steps a funding regime is built from. A component is a mechanism,
not a number — it says how a quantity is derived, while the numbers it consumes live in
[`parameter/`](../parameter/). Keeping the two apart is what makes regimes comparable: the
Fair School Funding Plan and its predecessor both compute a local share, and stating how each
does it is more informative than reporting that the resulting figures differ.

This is also the level at which `regime-diff` operates. A district that gained under a new
regime gained through some specific component, and a single net delta hides which one.

See the class definition: [formula-component.ont.yml](../formula-component.ont.yml).

## Instances

| Node | Regime | What it computes |
|------|--------|------------------|
| [charge-off-local-share](charge-off-local-share.yml) | Equal Yield, Foundation Base Cost | Deemed local contribution from a fixed millage against valuation |
| [fsfp-local-capacity-measure](fsfp-local-capacity-measure.yml) | Fair School Funding Plan | District share from valuation and resident income |
| [fsfp-base-cost-calculation](fsfp-base-cost-calculation.yml) | Fair School Funding Plan | District-specific base cost from staffing and salary inputs |
| [temporary-transitional-aid-guarantee](temporary-transitional-aid-guarantee.yml) | Fair School Funding Plan | Hold-harmless that overrides the formula result |

The guarantee is the component to read last and think about first. In FY2027 it governs the
funding of 48.3% of Ohio districts, and its incidence rises with property wealth — so it
systematically counteracts the local capacity measure that sits two rows above it in this table.
