# parameter

The dialable numbers. Each node here is one value that a funding regime consumes and the
General Assembly sets, carrying a series across fiscal periods and a link to the enactment
behind each change.

This class exists because of the prediction half of this repository's goal. A policy proposal
in this domain is almost always an edit to one of these values — raise the base cost, change
the phase-in percentage, move the floor — and a counterfactual needs something to bind to.
Parameters are separated from [`formula-component/`](../formula-component/) for that reason:
the component says how the calculation runs, the parameter says what number it runs on, and
only the second is what a proposal changes.

Note the asymmetry with [`metric/`](../metric/). A parameter is an input policy chooses; a
metric is an output policy produces. Confusing the two is the most common way arguments in
this domain go wrong — a base cost per pupil is a decision, a spending per pupil is a
consequence, and they are frequently quoted as though they were the same kind of thing.

**Every value series in this class is currently incomplete.** The parameters are correctly
identified and their mechanisms described, but the year-by-year numbers must be extracted
from statute and the LSC simulations before any figure here is load-bearing. This is the
single largest known gap in the seeded corpus. [open]

See the class definition: [parameter.ont.yml](../parameter.ont.yml).

## Instances

| Node | Unit | Set by |
|------|------|--------|
| [base-cost-per-pupil](base-cost-per-pupil.yml) | dollars per pupil | Biennial budget |
| [twenty-mill-floor](twenty-mill-floor.yml) | mills | H.B. 920, R.C. 319.301 |
| [local-share-charge-off-millage](local-share-charge-off-millage.yml) | mills | Biennial budget |
| [fsfp-phase-in-percentage](fsfp-phase-in-percentage.yml) | percent | Biennial budget |
