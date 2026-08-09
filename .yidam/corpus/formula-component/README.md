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
| [fsfp-targeted-assistance](fsfp-targeted-assistance.yml) | Fair School Funding Plan | Equalisation off weighted wealth, in two additive tiers |
| [fsfp-special-education-weights](fsfp-special-education-weights.yml) | Fair School Funding Plan | Six ascending weights on categories of disability |
| [fsfp-disadvantaged-pupil-impact-aid](fsfp-disadvantaged-pupil-impact-aid.yml) | Fair School Funding Plan | Two poverty counts blended, indexed on the state's, squared |
| [fsfp-career-technical-weights](fsfp-career-technical-weights.yml) | Fair School Funding Plan | Five weights on a career-technical base cost, plus associated services |
| [fsfp-english-learner-weights](fsfp-english-learner-weights.yml) | Fair School Funding Plan | Three descending weights by time in the country |
| [fsfp-gifted-units](fsfp-gifted-units.yml) | Fair School Funding Plan | Two per-pupil amounts and three kinds of clamped staffing unit |
| [fsfp-transportation](fsfp-transportation.yml) | Fair School Funding Plan | Two competing rate bases, two supplements, and its own guarantee |
| [fsfp-preschool-special-education](fsfp-preschool-special-education.yml) | Fair School Funding Plan | A flat $4,000 a pupil plus the six weights at half, prorated |
| [fsfp-performance-supplement](fsfp-performance-supplement.yml) | Fair School Funding Plan | The only payment gated on a measured outcome |
| [fsfp-enrolment-supplements](fsfp-enrolment-supplements.yml) | Fair School Funding Plan | $40 for every pupil, and $250 for a district that grew 3% |
| [fsfp-formula-transition-supplement](fsfp-formula-transition-supplement.yml) | Fair School Funding Plan | A second hold-harmless, on a FY2021 base that includes transportation |

**The six categoricals are one class of node and four kinds of mechanism.** They were a single
residual for eight phases — core foundation funding less the state share of base cost, exact and
uninterrogable — and folding them into one node would have repeated that conflation at the
ontology level. They do not behave alike:

| Shape | Components | What its parameters are |
|---|---|---|
| Weight x count | special education, career-technical, English learners | a weight vector and a base cost |
| Blend and index | DPIA | blend weights, a per-pupil amount, a statewide share |
| Units and floors | gifted | unit prices, divisors, a floor pair and a cap |
| Equalisation | targeted assistance | a wealth blend, two rates, ADM brackets, an index floor |

Three consequences the single node would have hidden. **Career-technical does not share a base cost
with the other two weighted programs** — $9,855.62 against $8,241.61 — so a shared parameter edge
from all three would be wrong. **Targeted assistance belongs beside the local capacity measure**,
not beside the weights: both blend valuation with federal adjusted gross income, at different
weights, and both are about the tax base rather than the pupil. And **gifted's parameters are
salary prices**, which move with the base cost build-up's staffing refresh rather than with any
weight.

**And Ohio holds districts harmless against FY2021 in three separate places.** The guarantee
compares foundation funding against `[H2] Funding Base`. The formula transition supplement compares
*everything the formula pays* against a larger `[L1] FY21 Funding Base`. Transportation holds its
own against FY2021 transportation funding alone. Three bases, three sets of districts — 294, 144
and 38 — and none nested in another: 17 districts draw the supplement while drawing nothing from
the guarantee.

The guarantee is the component to read last and think about first. In FY2027 it governs the
funding of 48.3% of Ohio districts, and its incidence rises with property wealth — so it
systematically counteracts the local capacity measure that sits two rows above it in this table.
